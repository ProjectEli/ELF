//! `elf check [version|setup|files|logs]` — 진단 통합 (구 `doctor`, `status`, `validate`). **읽기전용** — 단 logs 범위는
//! 폴더 한 단계 차이로 깨진 링크와 생성 파일 표 경로를 고쳐 쓰고 고친 것을 출력(v3.7 S056, `.elf/config.json` `fix_links`).
//!
//! 범위: version = CLI와 프로젝트 manifest, 판정 한 줄(S044 t41) / setup = git과 훅, autoread, experimental /
//! files = ELF 관리 파일, `.elf` 기록,
//! overlay, AGENTS 연결, tsa / logs = 세션 로그, Registry, figure embed. 기존 검사 함수를 재사용해
//! 판정은 바꾸지 않고 범위로 묶기만 한다. 판정 = issue(종료 4), warn, ok, info, fixed(고쳐 씀 — 요약에도 표시).
//! 범위마다 적용 가능 여부를 따로 판정 — `.elf/`가 없으면 files, `2_Log/`가 없으면 logs를 건너뜀
//! (로그만 쓰는 폴더에서도 logs 검사는 동작해야 함).

use std::path::Path;

use crate::{doctor, pin, session, status, store, update, validate, version};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Version,
    Setup,
    Files,
    Logs,
}

impl Scope {
    pub fn name(self) -> &'static str {
        match self {
            Scope::Version => "version",
            Scope::Setup => "setup",
            Scope::Files => "files",
            Scope::Logs => "logs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Info,
    Warn,
    Issue,
    /// 고쳐 씀(v3.7 S056) — 문제는 아니지만 바뀐 파일이 있으므로 요약에도 표시
    Fixed,
}

#[derive(Debug)]
pub struct Item {
    pub level: Level,
    pub text: String,
}

impl Item {
    fn new(level: Level, text: impl Into<String>) -> Self {
        Item { level, text: text.into() }
    }

    /// 요약 출력 대상(문제, 경고).
    pub fn is_finding(&self) -> bool {
        matches!(self.level, Level::Warn | Level::Issue)
    }

    /// 요약 출력 대상(문제, 경고, 고쳐 씀).
    pub fn in_summary(&self) -> bool {
        self.is_finding() || self.level == Level::Fixed
    }

    pub fn render(&self) -> String {
        let tag = match self.level {
            Level::Ok => "ok",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Issue => "issue",
            Level::Fixed => "fixed",
        };
        format!("{tag}: {}", self.text)
    }
}

#[derive(Debug)]
pub struct ScopeReport {
    pub scope: Scope,
    pub items: Vec<Item>,
    /// 적용 불가 사유(`.elf/` 없음 등) — Some이면 검사하지 않음
    pub skipped: Option<String>,
}

impl ScopeReport {
    fn new(scope: Scope) -> Self {
        ScopeReport { scope, items: Vec::new(), skipped: None }
    }

    pub fn issues(&self) -> usize {
        self.items.iter().filter(|i| i.level == Level::Issue).count()
    }

    pub fn warnings(&self) -> usize {
        self.items.iter().filter(|i| i.level == Level::Warn).count()
    }
}

pub fn run_scope(cwd: &Path, env: &doctor::DoctorEnv, scope: Scope) -> ScopeReport {
    match scope {
        Scope::Version => run_version(cwd, env),
        Scope::Setup => run_setup(cwd, env),
        Scope::Files => run_files(cwd, env),
        Scope::Logs => run_logs(cwd),
    }
}

fn push_doctor(r: &mut ScopeReport, d: doctor::DoctorReport) {
    for c in d.checks {
        let level = match c.health {
            doctor::Health::Ok => Level::Ok,
            doctor::Health::Info => Level::Info,
            doctor::Health::Warn => Level::Warn,
        };
        r.items.push(Item::new(level, format!("{}: {}", c.label, c.detail)));
    }
}

/// CLI(버전과 줄, 설치 기록), 프로젝트 manifest(형식, `elf_version`), 고정값(줄, 구조 세대), 판정 한 줄.
/// 프로젝트 밖 = CLI 항목만(`.elf/` 없이 로그만 있는 폴더는 고정값 추론 결과도).
fn run_version(cwd: &Path, env: &doctor::DoctorEnv) -> ScopeReport {
    let mut r = ScopeReport::new(Scope::Version);
    let cli_line = pin::parse(&env.version).map(|v| format!(" (line {}, structure generation {})", v.line(), v.line().generation())).unwrap_or_default();
    r.items.push(Item::new(Level::Info, format!("CLI: elf {}{cli_line}", env.version)));
    if env.receipt_present {
        r.items.push(Item::new(Level::Ok, "install receipt: found — `elf self update` works"));
    } else {
        r.items.push(Item::new(Level::Info, "install receipt: absent — `elf self update` unavailable (dev/manual build)"));
    }
    let root = update::find_project_root(cwd);
    let earlier_root = if root.is_none() { crate::earlier::find_root(cwd) } else { None };
    let v = match &root {
        Some(root) => version::verdict_with(root, &env.version),
        None if earlier_root.is_some() => version::Verdict::EarlierLayout,
        None => version::Verdict::NoProject,
    };
    match &v {
        version::Verdict::Earlier(_) => {
            r.items.push(Item::new(Level::Info, "project manifest: .elf/manifest.json — earlier format (schema 1)"))
        }
        version::Verdict::Same(p) | version::Verdict::Older(p) | version::Verdict::Newer(p) => {
            r.items.push(Item::new(Level::Info, format!("project manifest: .elf/manifest.json — schema 2, elf_version {p}")))
        }
        version::Verdict::NewerSchema(s) => {
            r.items.push(Item::new(Level::Info, format!("project manifest: .elf/manifest.json — schema {s} (newer than this CLI)")))
        }
        _ => {}
    }
    // 고정값(R1, R6): 프로젝트 루트, 없으면 `.elf/` 없이 로그만 있는 폴더
    let pin_root = root.or(earlier_root).or_else(|| session::find_log_root(cwd).filter(|lr| !lr.join(".elf").join("manifest.json").exists()));
    let pinned = pin_root.as_ref().and_then(|pr| pin::read(pr).ok().flatten().map(|p| (pr.clone(), p)));
    if let Some((pr, p)) = &pinned {
        let where_ = if pr.join(".elf").join("manifest.json").exists() {
            ""
        } else if crate::earlier::footprint(pr) {
            "ELF 2.0–2.4.0 folder (no .elf/): "
        } else {
            "log-only folder (no .elf/): "
        };
        r.items.push(Item::new(Level::Info, format!("{where_}{}", pin::describe(p))));
    }
    // 버전 보관소(v3.0 S044 t58, 읽기만): 위치와 개수, 고정값 줄을 실행할 파일
    let stored = store::installed();
    let dir = store::versions_dir().map(|d| store::shown(&d)).unwrap_or_else(|| "(no home folder — set ELF_HOME)".into());
    r.items.push(Item::new(Level::Info, format!("version store: {dir} — {} version(s)", stored.len())));
    if let Some((_, p)) = &pinned {
        let l = p.line;
        if pin::parse(&env.version).map(|v| v.line()) == Some(l) {
            r.items.push(Item::new(Level::Ok, format!("line {l}: this CLI runs the commands here")));
        } else {
            match store::newest_installed(l) {
                Some((v, _)) => r.items.push(Item::new(Level::Ok, format!("line {l}: {v} in the version store — commands here run with it (`self`, `init`, `update`, `check version` stay with this CLI)"))),
                None => r.items.push(Item::new(Level::Info, format!("line {l}: not in the version store — the first command here installs it (or `elf self install {l}`)"))),
            }
        }
    }
    let (k, line) = version::verdict_line(&v, &env.version);
    let level = match k {
        'o' => Level::Ok,
        'w' => Level::Warn,
        'i' => Level::Issue,
        _ => Level::Info,
    };
    r.items.push(Item::new(level, format!("verdict: {line}")));
    r
}

fn run_setup(cwd: &Path, env: &doctor::DoctorEnv) -> ScopeReport {
    let mut r = ScopeReport::new(Scope::Setup);
    push_doctor(&mut r, doctor::setup_checks(cwd, env));
    r
}

fn run_files(cwd: &Path, env: &doctor::DoctorEnv) -> ScopeReport {
    let mut r = ScopeReport::new(Scope::Files);
    let Some(root) = update::find_project_root(cwd) else {
        r.skipped = Some("not an ELF project (.elf/manifest.json not found)".into());
        return r;
    };
    push_doctor(&mut r, doctor::files_checks(&root, env));
    // 관리 파일 상태 — 갱신 대기와 충돌 = issue(구 `status --check` 기준), 버전과 tsa 줄은 위 검사가 보고
    match status::run_status_parts(&root, false) {
        Ok(s) => {
            for (kind, line) in s.kinds.iter().zip(s.lines) {
                let (level, text) = match kind {
                    status::LineKind::Ok => (Level::Ok, line.strip_prefix("ok: ").map(String::from).unwrap_or(line)),
                    status::LineKind::Info => (Level::Info, line),
                    status::LineKind::Warn => {
                        (Level::Warn, line.strip_prefix("warn: ").map(String::from).unwrap_or(line))
                    }
                    status::LineKind::Pending | status::LineKind::Conflict => (Level::Issue, line),
                };
                r.items.push(Item::new(level, text));
            }
        }
        Err(e) => r.items.push(Item::new(Level::Issue, format!("managed files: status failed: {e}"))),
    }
    r
}

fn run_logs(cwd: &Path) -> ScopeReport {
    let mut r = ScopeReport::new(Scope::Logs);
    // qa preset(v3.8 S057): 세션 로그 대신 번들 검사 — 같은 명령, 범위를 넓힘
    if let Some(root) = update::find_project_root(cwd)
        && crate::qa::is_qa(&root)
    {
        let lines: Result<Vec<String>, validate::ValidateError> =
            Ok(crate::qa::run_check(&root, crate::links::fix_enabled(&root)));
        push_lines(&mut r, lines);
        return r;
    }
    let Some(root) = crate::session::find_log_root(cwd) else {
        r.skipped = Some("no 2_Log/ directory found".into());
        return r;
    };
    // 이전 형식 저장소 = 2.27.4 검사(동결 모듈) + 이관 안내 info (v2.28.1)
    let lines = if crate::legacy::active(&root) {
        Ok(crate::legacy::cli::check_logs_lines(&root))
    } else {
        validate::run_validate_opts(&root, crate::links::fix_enabled(&root)).map(|v| v.lines)
    };
    push_lines(&mut r, lines);
    r
}

/// 검사 줄의 접두(`issue:`, `warn:`, `fixed:`, `ok:`, `hint:`/`info:`)를 수준으로 옮김.
fn push_lines(r: &mut ScopeReport, lines: Result<Vec<String>, validate::ValidateError>) {
    match lines {
        Ok(lines) => {
            for line in lines {
                let item = if let Some(x) = line.strip_prefix("issue: ") {
                    Item::new(Level::Issue, x)
                } else if let Some(x) = line.strip_prefix("warn: ") {
                    Item::new(Level::Warn, x)
                } else if let Some(x) = line.strip_prefix("fixed: ") {
                    Item::new(Level::Fixed, x)
                } else if let Some(x) = line.strip_prefix("ok: ") {
                    Item::new(Level::Ok, x)
                } else if let Some(x) = line.strip_prefix("hint: ").or_else(|| line.strip_prefix("info: ")) {
                    Item::new(Level::Info, x)
                } else {
                    Item::new(Level::Info, line)
                };
                r.items.push(item);
            }
        }
        Err(validate::ValidateError::Io(e)) => r.items.push(Item::new(Level::Issue, format!("io error: {e}"))),
    }
}
