//! 버전을 바꾸는 `elf update`의 목표와 사용자 동의(v3.0, S044 t60 C안).
//!
//! - 목표 = `--to <줄>`(생략하면 이 CLI의 줄). 목표 줄 < 현재 줄 = 거부(R5, 되돌리기는 git), 두 세대 이상 건너뛰기 =
//!   거부(이관은 세대마다 검토와 커밋), 목표 줄 ≠ 이 CLI 줄 = 목표 버전이 실행(R2, 보관소로 넘김).
//! - 같은 줄 = 관리 파일 갱신, 동의 없이 적용. 높은 줄(MINOR 이동, 세대 이관) = 아무것도 쓰지 않고 요약과 확인
//!   코드를 출력(exit 3). 에이전트는 요약을 사용자에게 보이고 승인을 받은 뒤 `--approve <코드>`. 코드 = 목표, 고정값,
//!   git HEAD와 작업 트리 상태, 계획 줄의 해시 → 그사이 무엇이 바뀌면 코드가 맞지 않아 다시 요약. 터미널(사람)이면 y/N.

use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::pin::{self, Line, Pin};
use crate::{dispatch, embed, migrate, update};

/// 요약에 보일 계획 줄 상한(전체는 `elf update --dry-run`)
const PLAN_SHOW: usize = 40;

/// 갱신 관문 결과.
#[derive(Debug)]
pub enum Gate {
    /// 이 프로세스에서 진행. 앞에 출력할 줄
    Proceed(Vec<String>),
    /// 목표 버전으로 넘김
    Dispatch { exe: PathBuf, args: Vec<OsString>, line: Line, version: String },
    /// 멈춤(종료 코드, 출력 줄)
    Stop(i32, Vec<String>),
}

/// 고정값 줄 → 목표 줄 분류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Same,
    Up,
    Down,
    /// 두 세대 이상 건너뜀 — 다음 세대 첫 줄
    Skip(Line),
}

pub fn classify(from: Line, to: Line) -> Move {
    if to == from {
        return Move::Same;
    }
    if to < from {
        return Move::Down;
    }
    let (gf, gt) = (from.generation(), to.generation());
    if gt > gf + 1 {
        let next = pin::GENERATIONS.iter().find(|g| g.number == gf + 1).map(|g| g.first).unwrap_or(to);
        return Move::Skip(next);
    }
    Move::Up
}

/// `--to` 값(`3.0`, `v2.27`, `3.0.1` — 줄만 씀) → 줄.
pub fn parse_target(to: &str) -> Option<Line> {
    pin::parse(to).map(|v| v.line())
}

/// 넘길 인수: 3.0 이전 버전(`--to`, `--approve`를 모름)에게는 둘을 뺌.
pub fn forward_args(args: &[OsString], target_major: u32) -> Vec<OsString> {
    if target_major >= 3 {
        return args.to_vec();
    }
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        let s = a.to_string_lossy();
        if s == "--to" || s == "--approve" {
            skip = true;
        } else if !(s.starts_with("--to=") || s.starts_with("--approve=")) {
            out.push(a.clone());
        }
    }
    out
}

fn git_out(root: &Path, args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// 확인 코드: 목표, 고정값, git HEAD와 작업 트리 상태(`.elf/runtime/` 제외), 계획 줄의 sha256 앞 6자리.
pub fn approval_code(root: &Path, from: &str, to: Line, plan: &[String]) -> String {
    let mut h = Sha256::new();
    h.update(from.as_bytes());
    h.update(format!("|{to}|").as_bytes());
    h.update(git_out(root, &["rev-parse", "HEAD"]).trim().as_bytes());
    let status = git_out(root, &["status", "--porcelain", "--untracked-files=all", "--", "."]);
    for l in status.lines().filter(|l| !l.contains(".elf/runtime/")) {
        h.update(l.as_bytes());
        h.update(b"\n");
    }
    for l in plan {
        h.update(l.as_bytes());
        h.update(b"\n");
    }
    h.finalize().iter().take(3).map(|b| format!("{b:02x}")).collect()
}

fn pin_desc(p: &Pin) -> String {
    match &p.source {
        pin::Source::Manifest(raw) => raw.clone(),
        pin::Source::Inferred(why) => format!("none recorded ({why})"),
    }
}

/// 동의 요약 줄.
fn summary(p: &Pin, to: Line, plan: &[String], code: &str) -> Vec<String> {
    let (gf, gt) = (p.line.generation(), to.generation());
    let what = if gt > gf {
        format!("structure generation {gf} → {gt}: the log layout is migrated")
    } else {
        format!("same structure generation {gt}: managed files and the recorded version only, no migration")
    };
    let mut out = vec![
        format!("update needs approval: project line {} → {to} ({what})", p.line),
        format!("  project: {}; this elf: {}", pin_desc(p), embed::version()),
        "  plan (`elf update --dry-run` prints the same):".into(),
    ];
    out.extend(plan.iter().take(PLAN_SHOW).map(|l| format!("    {l}")));
    if plan.len() > PLAN_SHOW {
        out.push(format!("    … {} more line(s) — `elf update --dry-run`", plan.len() - PLAN_SHOW));
    }
    out.push(format!(
        "  other PCs on this repository: after pull, elf 3.0 or newer runs the project with elf {to}.x (downloaded automatically); elf 2.28.1 or older needs `elf self update` first"
    ));
    out.push("  undo: git (revert the update commit)".into());
    out.push(format!("approval code: {code}"));
    out.push(format!(
        "AI agent: show this summary to the user and ask for approval. Only after the user approves, run `elf update --to {to} --approve {code}`, quote the user's approval in the session log, and commit the update by itself. (Run in a terminal, elf asks [y/N] instead.)"
    ));
    out
}

/// 동의 요약용 계획 줄(쓰지 않음): 프로젝트 = `elf update --dry-run`의 줄, 로그만 있는 폴더 = 이관 계획 줄.
fn plan_lines(root: &Path, log_only: bool) -> Result<Vec<String>, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if log_only {
        return match migrate::run(root, true, &today).map_err(|e| e.to_string())? {
            migrate::Outcome::Planned(l) | migrate::Outcome::Applied(l) | migrate::Outcome::Blocked(l) => Ok(l),
        };
    }
    let r = update::run_update(root, &update::UpdateOptions { dry_run: true, force: false }).map_err(|e| e.to_string())?;
    Ok(r.lines)
}

fn ask_terminal(lines: &[String]) -> bool {
    for l in lines.iter().filter(|l| !l.starts_with("AI agent:")) {
        println!("[elf] {l}");
    }
    print!("[elf] apply this update? [y/N] ");
    let _ = std::io::stdout().flush();
    let mut a = String::new();
    std::io::stdin().read_line(&mut a).is_ok() && matches!(a.trim(), "y" | "Y" | "yes" | "Yes")
}

/// `elf update` 관문. `root` = 프로젝트 루트 또는 로그만 있는 폴더(`log_only`).
pub fn gate_update(root: &Path, log_only: bool, to: Option<&str>, approve: Option<&str>, dry_run: bool) -> Gate {
    let launcher = embed::version();
    let Some(me) = pin::parse(launcher).map(|v| v.line()) else { return Gate::Proceed(Vec::new()) };
    let target = match to {
        Some(t) => match parse_target(t) {
            Some(l) => l,
            None => return Gate::Stop(2, vec![format!("error: --to {t} is not a line (e.g. --to 3.0)")]),
        },
        None => me,
    };
    // 읽을 수 없는 고정값과 더 높은 schema = 기존 경로(run_update)가 보고와 거부
    let Ok(Some(p)) = pin::read(root) else { return Gate::Proceed(Vec::new()) };
    match classify(p.line, target) {
        Move::Down if to.is_none() => {
            return Gate::Stop(
                3,
                vec![format!(
                    "refuse: the project is on line {}, newer than this elf ({launcher}) — update this elf with `elf self update`; moving a project to a lower line is refused (undo an update with git)",
                    p.line
                )],
            );
        }
        Move::Down => {
            return Gate::Stop(3, vec![format!("refuse: --to {target} is lower than the project's line {} — moving down is refused; undo an update with git", p.line)]);
        }
        Move::Skip(next) => {
            return Gate::Stop(
                3,
                vec![format!(
                    "refuse: line {} → {target} crosses more than one structure generation — update one generation at a time, first `elf update --to {next}` (review and commit each migration)",
                    p.line
                )],
            );
        }
        Move::Same | Move::Up => {}
    }
    // 목표 버전이 실행(R2)
    if target != me {
        if std::env::var_os(dispatch::ENV_DISPATCHED).is_some() || std::env::var(dispatch::ENV_DISPATCH).is_ok_and(|v| v == "off") {
            return Gate::Stop(3, vec![format!("refuse: this update needs elf {target}.x, and handing over is off here (ELF_DISPATCH=off or already handed over)")]);
        }
        return match dispatch::resolve_line(target) {
            Ok((v, exe, _)) => {
                let args: Vec<OsString> = std::env::args_os().skip(1).collect();
                Gate::Dispatch { exe, args: forward_args(&args, v.major), line: target, version: v.to_string() }
            }
            Err(e) => Gate::Stop(1, vec![format!("error: cannot obtain elf {target}.x for this update ({e}) — connect and retry, or `elf self install {target}`")]),
        };
    }
    if classify(p.line, target) == Move::Same {
        return Gate::Proceed(Vec::new());
    }
    if dry_run {
        return Gate::Proceed(vec![format!(
            "note: applying this changes the project line {} → {target} and needs the user's approval (`elf update` prints a summary and an approval code)",
            p.line
        )]);
    }
    // 이관 전제와 결정(경고면 결정 파일만 쓰고 멈춤) — 결정이 끝난 계획에만 동의를 받음
    if migrate::needed(root) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        match migrate::check_ready(root, &today) {
            Ok(migrate::Outcome::Blocked(lines)) => return Gate::Stop(3, lines),
            Ok(_) => {}
            Err(e) => return Gate::Stop(1, vec![format!("io error: {e}")]),
        }
    }
    let plan = match plan_lines(root, log_only) {
        Ok(l) => l,
        Err(e) => return Gate::Stop(1, vec![format!("error: {e}")]),
    };
    let code = approval_code(root, &pin_desc(&p), target, &plan);
    let lines = summary(&p, target, &plan, &code);
    match approve {
        Some(a) if a.trim().eq_ignore_ascii_case(&code) => Gate::Proceed(vec![format!("approved update (code {code}): line {} → {target}", p.line)]),
        Some(a) => {
            let mut out = vec![format!("refuse: approval code {a} does not match the current plan (something changed since the summary) — review the new summary")];
            out.extend(lines);
            Gate::Stop(3, out)
        }
        None if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            if ask_terminal(&lines) {
                Gate::Proceed(vec![format!("approved in the terminal: line {} → {target}", p.line)])
            } else {
                Gate::Stop(3, vec!["not applied — nothing written".into()])
            }
        }
        None => Gate::Stop(3, lines),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_moves() {
        let l = pin::line_of;
        assert_eq!(classify(l(3, 0), l(3, 0)), Move::Same);
        assert_eq!(classify(l(2, 28), l(3, 0)), Move::Same, "2.28.x is the 3.0 line");
        assert_eq!(classify(l(2, 27), l(3, 0)), Move::Up);
        assert_eq!(classify(l(3, 0), l(3, 2)), Move::Up);
        assert_eq!(classify(l(3, 1), l(3, 0)), Move::Down);
        assert_eq!(classify(l(3, 0), l(2, 27)), Move::Down);
        // 표에 없는 세대(2.0 이전 = 0) → 2세대: 다음 세대(1)의 첫 줄부터
        assert_eq!(classify(l(1, 9), l(3, 0)), Move::Skip(l(2, 0)));
    }

    #[test]
    fn targets_and_forwarded_args() {
        assert_eq!(parse_target("3.0"), Some(pin::line_of(3, 0)));
        assert_eq!(parse_target("v2.28.1"), Some(pin::line_of(3, 0)));
        assert_eq!(parse_target("latest"), None);
        let a = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        let args = a(&["update", "--to", "2.27", "--dry-run", "--approve=abc123", "--force"]);
        assert_eq!(forward_args(&args, 2), a(&["update", "--dry-run", "--force"]));
        assert_eq!(forward_args(&args, 3), args);
    }
}
