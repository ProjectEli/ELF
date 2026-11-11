//! 실행 위임(v3.0, S044 t51, t59 · v3.9 S059 판별 폴더) — PATH의 `elf`(실행기, `~/.elf/bin/`)는 판을 골라
//! 보관소(`~/.elf/versions/vX.Y.Z/`)의 그 판 실행 파일로 인수, 표준 입출력, 종료 코드를 그대로 넘김. 작업은 하지 않으므로
//! CLI 갱신(새 판 받기와 `current` 교체)이 실행 중인 다른 프로젝트의 판 파일을 건드리지 않음.
//!
//! - 프로젝트 안 = 그 줄의 보관된 최신 patch(같은 줄, 이전 줄, 더 높은 줄 모두 — 프로젝트별 격리). 이 실행 파일과 같은
//!   줄이면 네트워크 없이 보관된 것 중 이 판 이상만 봄. 다른 줄은 하루 한 번 발행 목록으로 새 patch를 받음(R1), 없으면
//!   받아서 설치(첫 사용).
//! - 프로젝트 밖과 판에 묶이지 않는 명령(`--help`, `--version`, `self`, `init`, `update`, `check version`) = 기본 판
//!   (`~/.elf/current`, 없으면 이 실행 파일).
//! - 구할 수 없을 때: 같은 세대의 기본 판 이하 줄 = 기본 판으로 실행하고 안내(막지 않음, S053), 2.27 줄(1세대) = 내장
//!   이전 형식 모드, 그 밖(프로젝트가 더 높음) = 읽기만(쓰기 명령 거부, `elf self update` 안내).
//! - 목록이 오래됐을 때(고정 판이나 그 줄이 같은 날 받은 목록에 없음): 간격 제한으로 다시 받고, 그래도 정하지 못하면
//!   목록 없이 고정 판을 바로 받음(S053).
//! - 이전 줄 프로젝트에는 새 줄로 옮길 수 있다는 안내를 프로젝트마다 하루 한 번.
//! - 재귀 방지: 넘겨받은 프로세스에 `ELF_DISPATCHED`(실행기 판). `ELF_DISPATCH=off` = 위임 끔(시험, CI, 진단),
//!   `trace` = 결정을 표준 오류에 한 줄.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::pin::{self, Line, Ver};
use crate::{launcher, session, store, update};

pub const ENV_DISPATCH: &str = "ELF_DISPATCH";
pub const ENV_DISPATCHED: &str = "ELF_DISPATCHED";

/// 위임 결정.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// 이 프로세스에서 실행
    Here,
    /// 보관소 실행 파일로 넘김(`installed` = 이번에 받음, `older` = 기본 판보다 이전 줄 프로젝트 — 안내 표시 기록 위치)
    Run { line: Line, version: Ver, exe: PathBuf, installed: bool, older: Option<PathBuf> },
    /// 줄의 판을 구할 수 없어 기본 판으로 실행(같은 세대의 기본 판 이하 줄 — 막지 않음). `exe` None = 이 프로세스
    Fallback { line: Line, version: Ver, exe: Option<PathBuf>, why: String, root: PathBuf },
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

/// 판 `v`를 실행할 결정: 보관소에 있고 이 실행 파일이 아니면 넘김, 아니면 이 프로세스.
fn to_version(v: &Ver, older: Option<PathBuf>) -> Decision {
    match store::exe_path(v).filter(|e| e.is_file()) {
        Some(exe) if !launcher::is_me(&exe) => Decision::Run { line: v.line(), version: v.clone(), exe, installed: false, older },
        _ => Decision::Here,
    }
}

/// 위임 결정(보관소 설치 포함 — 받는 동안 표준 오류에 안내 한 줄).
pub fn decide(args: &[OsString], cwd: &Path, launcher: &str) -> Decision {
    if std::env::var(ENV_DISPATCH).is_ok_and(|v| v == "off") || std::env::var_os(ENV_DISPATCHED).is_some() {
        return Decision::Here;
    }
    let Some(me) = pin::parse(launcher) else { return Decision::Here };
    launcher::bootstrap(&me);
    let cur = launcher::current_or(&me);
    let project = if launcher_only(args) { None } else { pin_root(cwd) };
    let Some(root) = project else { return to_version(&cur, None) };
    // 읽을 수 없는 고정값, 더 높은 schema = 기본 판의 판정과 관문이 처리(R7)
    let Ok(Some(p)) = pin::read(&root) else { return to_version(&cur, None) };
    let line = p.line;
    let older = (line < cur.line() && line.generation() == cur.line().generation()).then(|| root.clone());
    // 실행기 위치가 아닌 실행 파일(개발 빌드, 시험, 보관소 판을 직접 실행)의 줄 = 네트워크 없이 보관된 것 중 이 판 이상,
    // 없으면 이 프로세스. 실행기는 같은 줄도 아래 규칙(하루 한 번 새 patch 확인)으로 보관소 판에 넘김.
    if line == me.line() && !launcher::running_as_launcher() {
        return match store::newest_installed(line) {
            Some((v, _)) if v >= me => to_version(&v, older),
            _ => Decision::Here,
        };
    }
    // 고정 판(manifest `elf_version`) — 목록이 오래됐는지 판정하고, 목록 없이 바로 받을 때 씀
    let pinned = match &p.source {
        pin::Source::Manifest(raw) => pin::parse(raw),
        pin::Source::Inferred(_) => None,
    };
    match resolve_line(line, pinned.as_ref()) {
        Ok((version, exe, installed)) => {
            if launcher::is_me(&exe) {
                Decision::Here
            } else {
                Decision::Run { line, version, exe, installed, older }
            }
        }
        Err(why) if line.generation() == cur.line().generation() && line <= cur.line() => {
            let exe = store::exe_path(&cur).filter(|e| e.is_file() && !launcher::is_me(e));
            Decision::Fallback { line, version: cur, exe, why, root }
        }
        Err(why) => Decision::Unavailable { line, why },
    }
}

/// 줄을 실행할 보관소 실행 파일(보관된 최신, 하루 한 번 확인한 새 patch는 받음, 없으면 받음 — 받는 동안 표준 오류에
/// 안내 한 줄). `pinned` = 프로젝트가 고정한 판(있으면). 반환 = (버전, 실행 파일, 이번에 받았는가). Err = 구할 수 없는
/// 실제 이유.
pub fn resolve_line(line: Line, pinned: Option<&Ver>) -> Result<(Ver, PathBuf, bool), String> {
    let stored = store::newest_installed(line);
    let mut list = store::releases_daily();
    // 오래된 목록(S053): 보관된 것이 없고, 고정 판이나 그 줄이 목록에 없으면 같은 날이라도 다시 받음(간격 제한)
    let missing = |l: &Option<Vec<Ver>>| match l {
        Some(l) => store::newest_release(line, l).is_none() || pinned.is_some_and(|p| p.pre.is_none() && !l.contains(p)),
        None => true,
    };
    let mut rechecked = false;
    if stored.is_none()
        && missing(&list)
        && let Some(fresh) = store::releases_recheck()
    {
        list = Some(fresh);
        rechecked = true;
    }
    let newest = list.as_deref().and_then(|l| store::newest_release(line, l));
    // 목록으로 정하지 못하면 고정 판(정식 판)을 목록 없이 바로 받음
    let direct = pinned.filter(|p| p.line() == line && p.pre.is_none());
    let Some((version, fetch)) = choose(stored.as_ref().map(|s| &s.0), newest.as_ref()).or_else(|| direct.map(|p| (p.clone(), true))) else {
        return Err(match (&list, pinned) {
            (None, _) => "not in the version store, and the release list cannot be reached (offline?)".into(),
            (Some(_), Some(p)) if p.pre.is_some() => format!("the project was written by a pre-release ({p}), which is not published"),
            (Some(_), _) if rechecked => format!("no published release of line {line}"),
            (Some(_), _) => format!("no published release of line {line} in the release list checked within the last hour"),
        });
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
            None => Err(format!("{version} could not be downloaded ({e})")),
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

/// 이전 줄 안내 표시 기록(프로젝트마다 하루 한 번).
pub const OLDER_NOTE: &str = ".elf/runtime/older_line_note";

/// 이전 줄 프로젝트 안내 — 프로젝트마다 하루 한 번(표준 오류). 표시 기록을 쓰지 못해도 안내는 함(실행을 막지 않음).
fn older_note(root: &Path, msg: &str) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mark = root.join(OLDER_NOTE);
    if std::fs::read_to_string(&mark).is_ok_and(|t| t.trim() == today) {
        return;
    }
    eprintln!("[elf] note: {msg}");
    if let Some(d) = mark.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let _ = std::fs::write(&mark, today);
}

/// `main` 첫 단계: 위임하면 그 종료 코드로 끝냄(돌아오지 않음). 아니면 돌아와 이 프로세스에서 실행.
pub fn maybe_dispatch() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else { return };
    let launcher = crate::embed::version();
    let trace = std::env::var(ENV_DISPATCH).is_ok_and(|v| v == "trace");
    let cur = pin::parse(launcher).map(|me| launcher::current_or(&me));
    let to = cur.as_ref().map(|c| c.line().to_string()).unwrap_or_default();
    match decide(&args, &cwd, launcher) {
        Decision::Here => {
            if let Some(me) = pin::parse(launcher)
                && store::exe_path(&me).is_some_and(|e| launcher::is_me(&e))
            {
                launcher::touch(&me);
            }
        }
        Decision::Run { line, version, exe, older, .. } => {
            if trace {
                eprintln!("[elf] dispatch: project line {line} → {version} ({})", store::shown(&exe));
            }
            if let Some(root) = older {
                older_note(&root, &format!(
                    "this project is on line {line} and runs elf {version} (its own version). `elf update` moves it to {to} after the user approves"
                ));
            }
            launcher::touch(&version);
            std::process::exit(run(&exe, &args, launcher));
        }
        Decision::Fallback { line, version, exe, why, root } => {
            if trace {
                eprintln!("[elf] dispatch: project line {line} is not available ({why}) — running the default {version}");
            }
            older_note(&root, &format!(
                "this project is on line {line}, which is not available ({why}); elf {version} runs it as is (its ELF files stay on {line}). `elf update` moves it to {to} after the user approves"
            ));
            if let Some(exe) = exe {
                launcher::touch(&version);
                std::process::exit(run(&exe, &args, launcher));
            }
        }
        Decision::Unavailable { line, why } => {
            if line == pin::earlier_line() {
                eprintln!(
                    "[elf] note: elf {line}.x is not available ({why}) — running this CLI's built-in 2.27 compatibility mode"
                );
            } else {
                let msg = format!(
                    "project line {line} needs elf {line}.x, which is not available ({why}) — commands that write refuse because this CLI is older than the project; connect and retry, run `elf self install {line}`, or `elf self update`"
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
