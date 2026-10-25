//! `elf check [setup|files|logs]` — 진단 통합 (구 `doctor`, `status`, `validate`). **읽기전용**.
//!
//! 범위: setup = CLI, 설치, git과 훅, autoread, experimental / files = ELF 관리 파일, `.elf` 기록,
//! overlay, AGENTS 연결, tsa / logs = 세션 로그, Registry, figure embed. 기존 검사 함수를 재사용해
//! 판정은 바꾸지 않고 범위로 묶기만 한다. 판정 = issue(종료 4), warn, ok, info.
//! 범위마다 적용 가능 여부를 따로 판정 — `.elf/`가 없으면 files, `2_Log/`가 없으면 logs를 건너뜀
//! (로그만 쓰는 폴더에서도 logs 검사는 동작해야 함).

use std::path::Path;

use crate::{doctor, session, status, update, validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Setup,
    Files,
    Logs,
}

impl Scope {
    pub fn name(self) -> &'static str {
        match self {
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
    /// 검사 불능(Registry 파싱 불가 등 — 종료 5)
    pub escalation: Option<String>,
}

impl ScopeReport {
    fn new(scope: Scope) -> Self {
        ScopeReport { scope, items: Vec::new(), skipped: None, escalation: None }
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
    let Some(root) = session::find_log_root(cwd) else {
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
                } else if let Some(x) = line.strip_prefix("hint: ") {
                    Item::new(Level::Info, x)
                } else {
                    Item::new(Level::Info, line)
                };
                r.items.push(item);
            }
        }
        Err(validate::ValidateError::Escalation(e)) => r.escalation = Some(e.to_string()),
        Err(validate::ValidateError::Io(e)) => r.items.push(Item::new(Level::Issue, format!("io error: {e}"))),
    }
    r
}
