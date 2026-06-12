//! 컴파일타임 embed — SSOT = elf-cli/ (templates/, manifest.json, VERSION).
//! 경로 전제: 이 crate는 elf-cli/cli/ 에 위치 (S007 §0 레이아웃).

use include_dir::{include_dir, Dir};

/// elf-cli/templates/ 전체 (배포 정본). 파일 추가/삭제 시 코드 수정 불요.
pub static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../templates");

/// elf-cli/manifest.json (배포 명세: src/dest/tier/sha256) — manifest.rs가 파싱
pub static MANIFEST_JSON: &str = include_str!("../../manifest.json");

/// elf-cli/VERSION 원문 (개행 포함 가능)
static VERSION_RAW: &str = include_str!("../../VERSION");

/// 표시 버전 (예: "v2.4-dev")
pub fn version() -> &'static str {
    VERSION_RAW.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty_v_prefixed() {
        let v = version();
        assert!(!v.is_empty());
        assert!(v.starts_with('v'), "VERSION must start with 'v': {v}");
        assert_eq!(v, v.trim());
    }

    #[test]
    fn templates_contain_core_files() {
        for path in [
            "meta/EliRule.md",
            "meta/LogConvention.md",
            "log/sessionTemplate.md",
            "root/README.md",
            "root/.claudeignore",
            "root/.gitignore",
        ] {
            assert!(TEMPLATES.get_file(path).is_some(), "missing embed: {path}");
        }
    }

    #[test]
    fn manifest_is_embedded_and_lf() {
        assert!(MANIFEST_JSON.contains("\"schema\": \"elf-manifest/1\""));
        assert!(!MANIFEST_JSON.contains('\r'), "CRLF in embedded manifest");
    }

    /// CRLF embed 오염 가드 (Git_Hook_Governance §5 교훈의 embed 버전).
    /// working tree가 CRLF로 checkout되면 여기서 실패 → .gitattributes eol=lf 복구 신호.
    #[test]
    fn embedded_templates_have_no_crlf() {
        fn walk(dir: &Dir) {
            for f in dir.files() {
                if let Some(s) = f.contents_utf8() {
                    assert!(!s.contains('\r'), "CRLF in embedded {}", f.path().display());
                }
            }
            for d in dir.dirs() {
                walk(d);
            }
        }
        walk(&TEMPLATES);
    }
}
