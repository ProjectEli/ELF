//! `elf check [version|setup|files|logs]` — 진단 통합 (구 `doctor`, `status`, `validate`). **읽기전용**.
//!
//! 범위: version = CLI와 프로젝트 manifest, 판정 한 줄(S044 t41) / setup = git과 훅, autoread, experimental /
//! files = ELF 관리 파일, `.elf` 기록,
//! overlay, AGENTS 연결, tsa / logs = 세션 로그, Registry, figure embed. 기존 검사 함수를 재사용해
//! 판정은 바꾸지 않고 범위로 묶기만 한다. 판정 = issue(종료 4), warn, ok, info.
//! 범위마다 적용 가능 여부를 따로 판정 — `.elf/`가 없으면 files, `2_Log/`가 없으면 logs를 건너뜀
//! (로그만 쓰는 폴더에서도 logs 검사는 동작해야 함).

use std::path::Path;

use crate::{doctor, status, update, validate, version};

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

    pub fn render(&self) -> String {
        let tag = match self.level {
            Level::Ok => "ok",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Issue => "issue",
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

/// CLI(버전, 설치 기록), 프로젝트 manifest(형식, `elf_version`), 판정 한 줄. 프로젝트 밖 = CLI 항목만.
fn run_version(cwd: &Path, env: &doctor::DoctorEnv) -> ScopeReport {
    let mut r = ScopeReport::new(Scope::Version);
    r.items.push(Item::new(Level::Info, format!("CLI: elf {}", env.version)));
    if env.receipt_present {
        r.items.push(Item::new(Level::Ok, "install receipt: found — `elf self update` works"));
    } else {
        r.items.push(Item::new(Level::Info, "install receipt: absent — `elf self update` unavailable (dev/manual build)"));
    }
    let v = match update::find_project_root(cwd) {
        Some(root) => version::verdict_with(&root, &env.version),
        None => version::Verdict::NoProject,
    };
    match &v {
        version::Verdict::Earlier(_) => {
            r.items.push(Item::new(Level::Info, "project manifest: .elf/manifest.json — earlier format (schema 1)"))
        }
        version::Verdict::Same(p) | version::Verdict::Older(p) | version::Verdict::Newer(p) => {
            r.items.push(Item::new(Level::Info, format!("project manifest: .elf/manifest.json — schema 2, elf_version {p}")))
        }
        _ => {}
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
    let Some(root) = crate::session::find_log_root(cwd) else {
        r.skipped = Some("no 2_Log/ directory found".into());
        return r;
    };
    match validate::run_validate(&root) {
        Ok(v) => {
            for line in v.lines {
                let item = if let Some(x) = line.strip_prefix("issue: ") {
                    Item::new(Level::Issue, x)
                } else if let Some(x) = line.strip_prefix("warn: ") {
                    Item::new(Level::Warn, x)
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
    r
}
