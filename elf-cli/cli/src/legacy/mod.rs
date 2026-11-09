//! 이전 형식(ELF 2.27 로그 구조) 호환 — v2.28.1 핫픽스(S044 t50, P023).
//!
//! 전환 시점 = 저장소마다 `elf update`(이관). CLI를 2.28로 올려도 이관하지 않은 저장소는 2.27.4와 같이 기록됨:
//! 로그 명령(session, trial new, deprecate, validate, check logs, autoread의 세션 목록)을 v2.27.4 코드를 동결한
//! 이 모듈로 처리. 새 명령(`trial write`, `elf log`, `deprecate show`)은 이관 뒤 사용 안내.
//! 판정 = 파일 상태: 새 구조 폴더(`2_Log/S/`, `2_Log/G/`)가 없고 이전 형식 흔적(루트, Archive, Deprecated 로그, TSV)이나
//! 이전 형식 프로젝트 manifest(schema 1)가 있으면 이전 형식 모드. 둘이 섞인 저장소는 새 구조 모드(`elf update`가 정리).
//! 동결 모듈은 버그 수정만 하고, 이전 형식 저장소 이관이 끝나면 제거(예정 2.30).

pub mod cli;
pub mod deprecate;
pub mod embed;
pub mod session;
pub mod trial;
pub mod validate;

use std::fs;
use std::path::Path;

/// 이전 형식 모드 안내(쓰기 명령 성공 뒤 1줄, autoread digest, check logs info).
pub const NOTICE: &str = "this project uses the earlier log layout (ELF 2.27 format) — logs are written as before; run `elf update --dry-run` to preview the migration when you are ready";

/// 이 저장소를 이전 형식 모드로 다루는가.
pub fn active(root: &Path) -> bool {
    if crate::logs::Series::ALL.iter().any(|s| root.join(s.dir()).is_dir()) {
        return false;
    }
    if !crate::logs::legacy_markers(root).is_empty() {
        return true;
    }
    fs::read_to_string(root.join(".elf").join("manifest.json"))
        .ok()
        .and_then(|t| crate::manifest::parse(&t).ok())
        .is_some_and(|m| m.schema == crate::manifest::SCHEMA_V1)
}

/// autoread용 진행 중 세션 (id, Handoff) — v2.27.4 `active_session_handoffs` 그대로(2_Log 루트, 번호 내림차순).
pub fn active_session_handoffs(root: &Path) -> Vec<(String, String)> {
    let dir = root.join("2_Log");
    let mut rows: Vec<(u32, String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(num) = session::log_num(&name) else { continue };
            let Ok(content) = fs::read_to_string(e.path()) else { continue };
            let Some(status) = session::header_status(&content) else { continue };
            if !session::is_open_status(&status) {
                continue;
            }
            let handoff = session::header_handoff(&content).unwrap_or_else(|| "-".into());
            rows.push((num, format!("S{num:03}"), handoff));
        }
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.0));
    rows.into_iter().map(|(_, id, h)| (id, h)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_follows_file_state() {
        let t = tempfile::tempdir().unwrap();
        let r = t.path();
        fs::create_dir_all(r.join("2_Log/Wiki")).unwrap();
        assert!(!active(r), "nothing yet = new structure");
        fs::write(r.join("2_Log/Wiki/Session_Registry.tsv"), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\n").unwrap();
        assert!(active(r), "TSV = earlier layout");
        fs::create_dir_all(r.join("2_Log/S")).unwrap();
        assert!(!active(r), "mixed = new structure (elf update tidies)");
        let t2 = tempfile::tempdir().unwrap();
        fs::create_dir_all(t2.path().join(".elf")).unwrap();
        fs::write(t2.path().join(".elf/manifest.json"), r#"{ "schema": "elf-manifest/1", "files": [] }"#).unwrap();
        assert!(active(t2.path()), "schema 1 manifest without logs = earlier layout");
    }
}
