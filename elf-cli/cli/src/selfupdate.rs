//! `elf self update` — 판별 폴더 갱신(v3.9 S059).
//!
//! 새 판을 보관소(`~/.elf/versions/vX.Y.Z/`)에 받아 sha256과 실행을 확인한 뒤 기본 판(`~/.elf/current`)만 바꿈.
//! 실행 중인 다른 프로젝트의 판 파일은 건드리지 않음(프로젝트는 자기 줄의 판으로 계속 실행). 실행기
//! (`~/.elf/bin/elf`)는 새 판의 실행기 규약 번호가 더 클 때만 이름 바꾸기 두 번으로 교체. 이어서 보관소 정리
//! (`launcher::cleanup`). 3.8 이하는 axoupdater가 실행기 자리에서 새 판을 받는 동안 실행 파일이 비었음(S059 원인).

use crate::{embed, launcher, pin, store};

/// 갱신할 수 없을 때 안내할 수동 설치 명령(S008: ExecutionPolicy Bypass 형태가 표준).
pub const INSTALLER_HINT: &str = r#"powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex""#;

/// 갱신 결과 줄(앞에 `[elf] `를 붙여 출력). Err = 바뀐 것 없음.
pub fn run_self_update() -> Result<Vec<String>, String> {
    let me = pin::parse(embed::version()).ok_or("this build has no version")?;
    let cur = launcher::current_or(&me);
    let (list, stale) = store::releases(true)
        .map_err(|e| format!("self update failed: {e}.\n  To reinstall manually:\n  {INSTALLER_HINT}"))?;
    let mut out = Vec::new();
    if let Some(d) = stale {
        out.push(format!("note: the release list could not be refreshed — using the one from {d}"));
    }
    let newest = list.iter().filter(|v| v.pre.is_none()).max().cloned().ok_or("the release list has no published release")?;
    if newest <= cur {
        out.push(format!("already up to date ({cur})"));
    } else {
        let inst = store::install(&newest)?;
        if launcher::protocol_of(&inst.exe) == 0 {
            if inst.fresh {
                let _ = store::uninstall(&newest);
            }
            return Err(format!("{newest} was downloaded but does not run here — nothing changed"));
        }
        launcher::write_current(&newest)?;
        out.push(format!(
            "updated to {newest} — was {cur}. Projects keep running their own versions; outside projects and for new projects {newest} runs"
        ));
        match launcher::replace_launcher(&inst.exe) {
            Ok(true) => out.push("the launcher was replaced (its dispatch rules changed)".into()),
            Ok(false) => {}
            Err(e) => out.push(format!("warn: the launcher was not replaced ({e}); the current one keeps working")),
        }
    }
    let (removed, skipped) = launcher::cleanup(&launcher::current_or(&me), None);
    out.extend(removed);
    out.extend(skipped);
    Ok(out)
}
