//! `elf self-update` — 바이너리 자체 갱신 (t06 stage 3, P011 §6.1).
//!
//! axoupdater **라이브러리 모드**: 별도 updater 바이너리는 파일명 update 키워드로
//! Windows UAC installer detection(권한 상승 요구)에 걸림 — S007 t04 실측.
//! cargo-dist 인스톨러가 남긴 install receipt를 읽어 GitHub Releases 확인 → atomic swap.
//! receipt 없는 설치(dev 빌드·수동 배치)는 동작 불가 — 인스톨러 명령 안내 후 종료.

use axoupdater::AxoUpdater;

/// receipt 부재 시 안내할 수동 갱신 명령 (S008: ExecutionPolicy Bypass 형태가 표준)
pub const INSTALLER_HINT: &str = r#"powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex""#;

pub fn run_self_update() -> Result<String, String> {
    let mut updater = AxoUpdater::new_for("elf-cli");
    updater.load_receipt().map_err(|e| {
        format!(
            "install receipt를 찾지 못함 ({e}).\n  self-update는 인스톨러로 설치된 elf에서만 동작합니다. 수동 갱신:\n  {INSTALLER_HINT}"
        )
    })?;

    match updater.run_sync() {
        Ok(Some(result)) => Ok(format!(
            "updated to {} (restart not required) — was {}",
            result.new_version,
            env!("CARGO_PKG_VERSION")
        )),
        Ok(None) => Ok(format!("already up to date ({})", env!("CARGO_PKG_VERSION"))),
        Err(e) => Err(format!("self-update 실패: {e}")),
    }
}
