//! 판별 폴더와 실행기(v3.9, S059): 모든 판 = `~/.elf/versions/vX.Y.Z/elf[.exe]`, 기본 판 = `~/.elf/current`(한 줄),
//! PATH의 `~/.elf/bin/elf[.exe]` = 실행기(판을 골라 그 판 폴더의 실행 파일로 넘김, 작업은 하지 않음).
//!
//! - 갱신(`elf self update`) = 새 판을 보관소에 받고 `current`만 바꿈 — 실행 중인 다른 프로젝트의 판 파일과 실행기를
//!   건드리지 않음. 실행기는 규약 번호(`PROTOCOL`)가 바뀐 판에서만 미리 받은 파일로 이름 바꾸기 두 번에 교체.
//! - 첫 실행(이관): 실행기 위치에서 돌 때 `current`가 없으면 자기 자신을 보관소에 넣고 `current`를 씀.
//! - 정리(`self update`, `self install` 때만): ① 줄마다 최신 patch만 ② 다른 줄은 `current` 줄과 최근 `KEEP_DAYS`일에
//!   쓴 줄만(판마다 마지막 사용 날짜를 하루 한 번 기록) ③ 지울 수 없는(실행 중인) 파일은 건너뜀.

use std::fs;
use std::path::{Path, PathBuf};

use crate::pin::{self, Ver};
use crate::store;

/// 실행기 규약 번호 — 실행기의 판 고르기 규칙이 바뀌면 올림(같으면 갱신 때 실행기를 바꾸지 않음).
pub const PROTOCOL: u32 = 1;
/// 실행기 규약 번호를 묻는 숨김 인수(이전 판은 모름 = 0).
pub const PROTOCOL_ARG: &str = "--launcher-protocol";
/// 다른 줄을 남기는 기간(마지막 사용 기준).
pub const KEEP_DAYS: i64 = 30;
const CURRENT: &str = "current";
const LAST_USED: &str = "last_used";

fn bin_dir() -> Option<PathBuf> {
    store::home().map(|h| h.join("bin"))
}

/// 실행기 경로(`~/.elf/bin/elf[.exe]`, 있든 없든).
pub fn launcher_path() -> Option<PathBuf> {
    bin_dir().map(|d| d.join(store::exe_name()))
}

/// 경로 비교용(가능하면 실제 경로, Windows는 대소문자 무시).
fn same_file(a: &Path, b: &Path) -> bool {
    let key = |p: &Path| {
        let s = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().replace('\\', "/");
        if cfg!(windows) { s.to_lowercase() } else { s }
    };
    key(a) == key(b)
}

/// 이 프로세스가 실행기 위치(`~/.elf/bin/`)의 파일인가.
pub fn running_as_launcher() -> bool {
    match (std::env::current_exe(), launcher_path()) {
        (Ok(me), Some(l)) => l.is_file() && same_file(&me, &l),
        _ => false,
    }
}

/// 이 프로세스가 `exe`인가.
pub fn is_me(exe: &Path) -> bool {
    std::env::current_exe().is_ok_and(|me| same_file(&me, exe))
}

/// `~/.elf/current`의 판.
pub fn read_current() -> Option<Ver> {
    let text = fs::read_to_string(store::home()?.join(CURRENT)).ok()?;
    pin::parse(text.trim())
}

/// `current`를 원자적으로 바꿈(임시 파일 + 이름 바꾸기).
pub fn write_current(v: &Ver) -> Result<(), String> {
    let home = store::home().ok_or("no home folder for the version store (set ELF_HOME)")?;
    fs::create_dir_all(&home).map_err(|e| format!("{}: {e}", home.display()))?;
    let tmp = home.join(format!(".{CURRENT}-{}", std::process::id()));
    fs::write(&tmp, format!("{v}\n")).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, home.join(CURRENT)).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", home.join(CURRENT).display())
    })
}

/// 기본 판: `current`, 없으면 이 실행 파일의 판.
pub fn current_or(me: &Ver) -> Ver {
    read_current().unwrap_or_else(|| me.clone())
}

/// 첫 실행(이관): 실행기 위치에서 돌고 `current`가 없으면 자기 자신을 보관소에 넣고 `current`를 씀. 실패는 조용히 넘김
/// (다음 실행에서 다시 시도 — 명령 실행을 막지 않음). 남은 `.previous` 실행기도 지움.
pub fn bootstrap(me: &Ver) {
    if !running_as_launcher() {
        return;
    }
    if let Some(prev) = previous_path() {
        let _ = fs::remove_file(prev);
    }
    if read_current().is_some() {
        return;
    }
    let (Ok(src), Some(dst)) = (std::env::current_exe(), store::exe_path(me)) else { return };
    if !dst.is_file() {
        let Some(dir) = dst.parent() else { return };
        if fs::create_dir_all(dir).is_err() || fs::copy(&src, &dst).is_err() {
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dst, fs::Permissions::from_mode(0o755));
        }
    }
    let _ = write_current(me);
}

/// 판의 마지막 사용 날짜를 오늘로(하루 한 번만 씀, 실패는 넘김).
pub fn touch(v: &Ver) {
    let Some(dir) = store::versions_dir().map(|d| d.join(v.to_string())) else { return };
    if !dir.is_dir() {
        return;
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let f = dir.join(LAST_USED);
    if fs::read_to_string(&f).is_ok_and(|t| t.trim() == today) {
        return;
    }
    let _ = fs::write(f, format!("{today}\n"));
}

/// 마지막 사용 날짜: `last_used`, 없으면 설치 날짜(`install.json`), 둘 다 없으면 None(판단 불가 = 남김).
pub fn last_used(v: &Ver) -> Option<chrono::NaiveDate> {
    let dir = store::versions_dir()?.join(v.to_string());
    let parse = |s: &str| chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok();
    if let Some(d) = fs::read_to_string(dir.join(LAST_USED)).ok().and_then(|t| parse(&t)) {
        return Some(d);
    }
    let info: serde_json::Value = serde_json::from_str(&fs::read_to_string(dir.join("install.json")).ok()?).ok()?;
    parse(info.get("installed")?.as_str()?)
}

/// 정리 계획(순수): 지울 판 = ① 줄마다 최신이 아닌 patch ② `current` 줄이 아니고 마지막 사용이 `KEEP_DAYS`일보다 오래된 줄.
/// `installed` = (판, 마지막 사용) 새 순. `current`와 `keep`(방금 직접 설치한 판)은 지우지 않음.
pub fn plan_cleanup(
    installed: &[(Ver, Option<chrono::NaiveDate>)],
    current: &Ver,
    keep: Option<&Ver>,
    today: chrono::NaiveDate,
) -> Vec<(Ver, &'static str)> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for (v, used) in installed {
        if v == current || keep == Some(v) {
            seen.push(v.line());
            continue;
        }
        if seen.contains(&v.line()) {
            out.push((v.clone(), "a newer patch of the line is kept"));
            continue;
        }
        seen.push(v.line());
        if v.line() == current.line() {
            continue;
        }
        if let Some(d) = used
            && (today - *d).num_days() > KEEP_DAYS
        {
            out.push((v.clone(), "not used for more than 30 days"));
        }
    }
    out
}

/// 정리 실행 — 지운 판과 이유, 지울 수 없어 건너뛴 판(실행 중 등). 다음 정리 때 다시 시도. `keep` = 방금 직접 설치한 판.
pub fn cleanup(current: &Ver, keep: Option<&Ver>) -> (Vec<String>, Vec<String>) {
    let installed: Vec<(Ver, Option<chrono::NaiveDate>)> =
        store::installed().into_iter().map(|(v, _)| { let u = last_used(&v); (v, u) }).collect();
    let today = chrono::Local::now().date_naive();
    let mut removed = Vec::new();
    let mut skipped = Vec::new();
    for (v, why) in plan_cleanup(&installed, current, keep, today) {
        match store::uninstall(&v) {
            Ok(()) => removed.push(format!("removed {v} ({why})")),
            Err(e) => skipped.push(format!("kept {v} for now — {e}; retried at the next cleanup")),
        }
    }
    (removed, skipped)
}

/// 실행 파일의 실행기 규약 번호(`--launcher-protocol`로 물음, 넘김 없이). 모르면 0.
pub fn protocol_of(exe: &Path) -> u32 {
    std::process::Command::new(exe)
        .arg(PROTOCOL_ARG)
        .env(crate::dispatch::ENV_DISPATCHED, "launcher-protocol")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0)
}

fn previous_path() -> Option<PathBuf> {
    bin_dir().map(|d| d.join(format!("{}.previous", store::exe_name())))
}

/// 실행기 교체(파일만): `src`를 `dst.new`로 복사한 뒤 이름 바꾸기 두 번(현재 → `.previous`, new → `dst`).
/// 빈 시간 = 두 이름 바꾸기 사이뿐. 실패하면 되돌림. `.previous`는 지울 수 있으면 지움(실행 중이면 다음 실행 때).
pub fn swap_in(src: &Path, dst: &Path, previous: &Path) -> Result<(), String> {
    let new = dst.with_file_name(format!("{}.new", store::exe_name()));
    fs::copy(src, &new).map_err(|e| format!("{}: {e}", new.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&new, fs::Permissions::from_mode(0o755));
    }
    let _ = fs::remove_file(previous);
    let had = dst.is_file();
    if had {
        fs::rename(dst, previous).map_err(|e| {
            let _ = fs::remove_file(&new);
            format!("{}: {e}", dst.display())
        })?;
    }
    if let Err(e) = fs::rename(&new, dst) {
        if had {
            let _ = fs::rename(previous, dst);
        }
        let _ = fs::remove_file(&new);
        return Err(format!("{}: {e}", dst.display()));
    }
    let _ = fs::remove_file(previous);
    Ok(())
}

/// 실행기 바꾸기: 실행기가 있고 새 판의 규약 번호가 더 클 때만. 반환 = 바꿨는가.
pub fn replace_launcher(new_exe: &Path) -> Result<bool, String> {
    let (Some(dst), Some(prev)) = (launcher_path(), previous_path()) else { return Ok(false) };
    if !dst.is_file() {
        return Ok(false);
    }
    if protocol_of(&dst) >= protocol_of(new_exe) {
        return Ok(false);
    }
    swap_in(new_exe, &dst, &prev).map(|_| true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Ver {
        pin::parse(s).unwrap()
    }

    fn d(s: &str) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    }

    #[test]
    fn cleanup_keeps_newest_patch_current_line_and_recent_lines() {
        let today = d("2026-12-31").unwrap();
        let installed = vec![
            (v("v3.9.2"), d("2026-12-31")),
            (v("v3.9.1"), d("2026-12-30")),
            (v("v3.8.1"), d("2026-12-20")),
            (v("v3.8.0"), d("2026-12-20")),
            (v("v3.4.0"), d("2026-10-01")),
            (v("v3.0.1"), None),
            (v("v2.27.4"), d("2026-11-30")),
        ];
        let plan: Vec<String> = plan_cleanup(&installed, &v("v3.9.2"), None, today).into_iter().map(|(x, _)| x.to_string()).collect();
        assert_eq!(plan, vec!["v3.9.1", "v3.8.0", "v3.4.0", "v2.27.4"]);
    }

    #[test]
    fn current_is_never_removed_even_if_older_than_a_stored_patch() {
        let installed = vec![(v("v3.9.3"), d("2026-01-01")), (v("v3.9.2"), d("2026-01-01"))];
        let plan = plan_cleanup(&installed, &v("v3.9.2"), None, d("2026-12-31").unwrap());
        assert!(plan.is_empty(), "{plan:?}");
        // 직접 설치한 옛 patch는 그 명령에서 지우지 않음
        let installed = vec![(v("v3.8.1"), None), (v("v3.8.0"), None)];
        assert!(plan_cleanup(&installed, &v("v3.9.2"), Some(&v("v3.8.0")), d("2026-12-31").unwrap()).is_empty());
    }

    #[test]
    fn swap_in_replaces_with_two_renames_and_cleans_up() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src.bin");
        let dst = tmp.path().join(store::exe_name());
        let prev = tmp.path().join("prev");
        fs::write(&src, "new").unwrap();
        fs::write(&dst, "old").unwrap();
        swap_in(&src, &dst, &prev).unwrap();
        assert_eq!(fs::read_to_string(&dst).unwrap(), "new");
        assert!(!prev.exists() && !tmp.path().join(format!("{}.new", store::exe_name())).exists());
    }
}
