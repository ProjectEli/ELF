//! 실행 위임(v3.0, S044 t51, t59) — PATH의 `elf`(런처)가 프로젝트 고정값의 줄을 보고, 자기와 다른 줄이면 버전
//! 보관소의 그 줄 최신 버전으로 인수, 표준 입출력, 종료 코드를 그대로 넘겨 실행.
//!
//! - 런처가 직접 처리: 인수 없음과 `--help`, `--version`, `self`(실행 파일과 보관소), `init`(새 프로젝트 = 런처 줄),
//!   `update`(버전 이동은 대상 버전이 실행, R2), `check version`(버전 판정).
//! - 줄의 실행 파일 = 보관소의 최신 정식 patch. 하루 한 번 발행 목록을 짧게 확인해 더 새 patch가 있으면 받음(R1).
//!   없으면 받아서 설치(첫 사용).
//! - 구할 수 없을 때: 2.27 줄(1세대) = 내장 이전 형식 모드로 이 프로세스에서 실행, 그 밖 = 읽기만(쓰기 명령 거부).
//! - 재귀 방지: 넘겨받은 프로세스에 `ELF_DISPATCHED`(런처 버전). `ELF_DISPATCH=off` = 위임 끔(시험, CI, 진단),
//!   `trace` = 결정을 표준 오류에 한 줄.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::pin::{self, Line, Ver};
use crate::{session, store, update};

pub const ENV_DISPATCH: &str = "ELF_DISPATCH";
pub const ENV_DISPATCHED: &str = "ELF_DISPATCHED";

/// 위임 결정.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// 이 프로세스에서 실행
    Here,
    /// 보관소 실행 파일로 넘김(`installed` = 이번에 받음)
    Run { line: Line, version: Ver, exe: PathBuf, installed: bool },
    /// 줄의 실행 파일을 구할 수 없음 — 이유
    Unavailable { line: Line, why: String },
}

/// 런처가 직접 처리하는 명령인가(첫 인수 기준 — 위임 대상 버전의 인수 문법과 무관하게 판단).
pub fn launcher_only(args: &[OsString]) -> bool {
    let a: Vec<String> = args.iter().map(|s| s.to_string_lossy().into_owned()).collect();
    let Some(i) = a.iter().position(|x| !x.starts_with('-')) else { return true };
    match a[i].as_str() {
        "self" | "self-update" | "selfupdate" | "init" | "update" | "help" => true,
        "check" => a.get(i + 1).is_some_and(|s| s == "version"),
        _ => false,
    }
}

/// 보관된 최신과 발행 목록의 최신 → 실행할 버전과 받아야 하는가.
pub fn choose(stored: Option<&Ver>, newest_known: Option<&Ver>) -> Option<(Ver, bool)> {
    match (stored, newest_known) {
        (Some(s), Some(n)) if n > s => Some((n.clone(), true)),
        (Some(s), _) => Some((s.clone(), false)),
        (None, Some(n)) => Some((n.clone(), true)),
        (None, None) => None,
    }
}

/// 고정값을 읽을 폴더: 프로젝트 루트, 없으면 `.elf/` 없는 2.0~2.4.0 프로젝트, 없으면 `.elf/` 없이 로그만 있는 폴더.
fn pin_root(cwd: &Path) -> Option<PathBuf> {
    update::find_project_root(cwd)
        .or_else(|| crate::earlier::find_root(cwd))
        .or_else(|| session::find_log_root(cwd).filter(|lr| !lr.join(".elf").join("manifest.json").exists()))
}

/// 위임 결정(보관소 설치 포함 — 받는 동안 표준 오류에 안내 한 줄).
pub fn decide(args: &[OsString], cwd: &Path, launcher: &str) -> Decision {
    if std::env::var(ENV_DISPATCH).is_ok_and(|v| v == "off") || std::env::var_os(ENV_DISPATCHED).is_some() || launcher_only(args) {
        return Decision::Here;
    }
    let Some(root) = pin_root(cwd) else { return Decision::Here };
    // 읽을 수 없는 고정값, 더 높은 schema = 이 프로세스의 판정과 관문이 처리(R7)
    let Ok(Some(p)) = pin::read(&root) else { return Decision::Here };
    let Some(me) = pin::parse(launcher) else { return Decision::Here };
    if p.line == me.line() {
        return Decision::Here;
    }
    let line = p.line;
    match resolve_line(line) {
        Ok((version, exe, installed)) => Decision::Run { line, version, exe, installed },
        Err(why) => Decision::Unavailable { line, why },
    }
}

/// 줄을 실행할 보관소 실행 파일(보관된 최신, 하루 한 번 확인한 새 patch는 받음, 없으면 받음 — 받는 동안 표준 오류에
/// 안내 한 줄). 반환 = (버전, 실행 파일, 이번에 받았는가). Err = 구할 수 없는 이유.
pub fn resolve_line(line: Line) -> Result<(Ver, PathBuf, bool), String> {
    let stored = store::newest_installed(line);
    let list = store::releases_daily();
    let newest = list.as_deref().and_then(|l| store::newest_release(line, l));
    let Some((version, fetch)) = choose(stored.as_ref().map(|s| &s.0), newest.as_ref()) else {
        return Err("not in the version store, and the release list cannot be read".into());
    };
    if !fetch {
        return Ok((version, stored.map(|s| s.1).unwrap_or_default(), false));
    }
    eprintln!("[elf] line {line}: installing {version} into the version store …");
    match store::install(&version) {
        Ok(i) => Ok((version, i.exe, i.fresh)),
        // 새 patch를 받지 못하면 보관된 것으로
        Err(e) => match store::newest_installed(line) {
            Some((v, exe)) => {
                eprintln!("[elf] warn: {e} — running the stored {v}");
                Ok((v, exe, false))
            }
            None => Err(e),
        },
    }
}

/// 보관소 실행 파일로 넘겨 실행 → 종료 코드.
pub fn run(exe: &Path, args: &[OsString], launcher: &str) -> i32 {
    let mut c = std::process::Command::new(exe);
    c.args(args).env(ENV_DISPATCHED, launcher);
    exec(c, exe)
}

/// unix = 프로세스 교체(신호와 pid 그대로), 성공하면 돌아오지 않음.
#[cfg(unix)]
fn exec(mut c: std::process::Command, exe: &Path) -> i32 {
    use std::os::unix::process::CommandExt;
    let e = c.exec();
    eprintln!("[elf] error: cannot run {}: {e}", store::shown(exe));
    1
}

/// Windows = 자식 프로세스(표준 입출력 상속)를 기다려 그 종료 코드.
#[cfg(not(unix))]
fn exec(mut c: std::process::Command, exe: &Path) -> i32 {
    match c.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("[elf] error: cannot run {}: {e}", store::shown(exe));
            1
        }
    }
}

/// `main` 첫 단계: 위임하면 그 종료 코드로 끝냄(돌아오지 않음). 아니면 돌아와 이 프로세스에서 실행.
pub fn maybe_dispatch() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else { return };
    let launcher = crate::embed::version();
    let trace = std::env::var(ENV_DISPATCH).is_ok_and(|v| v == "trace");
    match decide(&args, &cwd, launcher) {
        Decision::Here => {}
        Decision::Run { line, version, exe, .. } => {
            if trace {
                eprintln!("[elf] dispatch: project line {line} → {version} ({})", store::shown(&exe));
            }
            std::process::exit(run(&exe, &args, launcher));
        }
        Decision::Unavailable { line, why } => {
            if line == pin::earlier_line() {
                eprintln!(
                    "[elf] note: elf {line}.x is not available ({why}) — running this CLI's built-in 2.27 compatibility mode"
                );
            } else {
                let msg = format!(
                    "project line {line} needs elf {line}.x, which is not available ({why}) — commands that write refuse; connect and retry, run `elf self install {line}`, or move the project with `elf update`"
                );
                eprintln!("[elf] warn: {msg}");
                crate::version::set_unavailable(msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    #[test]
    fn launcher_only_commands() {
        for v in [&[][..], &["--version"], &["-h"], &["self", "update"], &["self", "list"], &["init", "P"], &["update", "--dry-run"], &["check", "version"], &["help", "log"]] {
            assert!(launcher_only(&a(v)), "{v:?}");
        }
        for v in [&["check"][..], &["check", "logs"], &["session", "new", "x"], &["log"], &["autoread", "session-start"], &["trial", "new", "t"]] {
            assert!(!launcher_only(&a(v)), "{v:?}");
        }
    }

    #[test]
    fn choose_prefers_a_newer_release_then_stored() {
        let v = |s: &str| pin::parse(s).unwrap();
        assert_eq!(choose(Some(&v("2.27.3")), Some(&v("2.27.4"))), Some((v("2.27.4"), true)));
        assert_eq!(choose(Some(&v("2.27.4")), Some(&v("2.27.4"))), Some((v("2.27.4"), false)));
        assert_eq!(choose(Some(&v("2.27.4")), None), Some((v("2.27.4"), false)));
        assert_eq!(choose(None, Some(&v("2.27.4"))), Some((v("2.27.4"), true)));
        assert_eq!(choose(None, None), None);
    }
}
