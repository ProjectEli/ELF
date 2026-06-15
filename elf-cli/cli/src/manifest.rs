//! 배포 명세(manifest) 파싱 — elf-cli/manifest.json (schema: elf-manifest/1).
//! src = CLI 번들 상대(templates/...), dest = 프로젝트 루트 상대. (P009 §3)

use serde::Deserialize;

pub const SCHEMA: &str = "elf-manifest/1";

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub schema: String,
    #[serde(default)]
    pub generated: String,
    #[serde(default)]
    pub note: String,
    pub files: Vec<Entry>,
    /// 폴더 scaffold 데이터 (S007 결정 2026-06-12: 하드코딩 대신 manifest로 단일소스화)
    #[serde(default)]
    pub dirs: Dirs,
}

#[derive(Debug, Default, Deserialize)]
pub struct Dirs {
    /// 항상 생성 (Core 0~2 + templates)
    #[serde(default)]
    pub core: Vec<String>,
    /// 모듈 키 → 폴더 목록 (hw/fab/sw/exp/paper)
    #[serde(default)]
    pub modules: std::collections::BTreeMap<String, Vec<String>>,
    /// preset 이름 → 모듈 키 목록
    #[serde(default)]
    pub presets: std::collections::BTreeMap<String, Vec<String>>,
    /// .gitkeep 대신 `*`+`!.gitignore` 를 받는 원본 데이터 폴더
    #[serde(default)]
    pub raw_data: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub src: String,
    pub dest: String,
    pub tier: Tier,
    /// 정본(src)의 sha256 (LF 정규화 바이트 기준 — hash::sha256_lf)
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// ELF 소유 — update가 덮어씀 (편집 감지 시 보호)
    Managed,
    /// init 전용 — update 절대 미접근
    Seed,
    /// 마커블록만 ELF 소유 — 블록 교체 + 사용자 영역 보존 (S007 §4.1)
    Hybrid,
}

pub fn parse(json: &str) -> Result<Manifest, String> {
    let m: Manifest = serde_json::from_str(json).map_err(|e| format!("manifest parse: {e}"))?;
    if m.schema != SCHEMA {
        return Err(format!("unsupported manifest schema: {} (expect {SCHEMA})", m.schema));
    }
    Ok(m)
}

/// 바이너리에 embed된 정본 manifest. 빌드 시점에 유효성이 테스트로 보증됨.
pub fn embedded() -> Manifest {
    parse(crate::embed::MANIFEST_JSON).expect("embedded manifest must be valid")
}

/// QA preset(experimental) 정본 manifest — 질문 아카이브 archetype (연구 manifest와 분리).
pub fn embedded_qa() -> Manifest {
    parse(crate::embed::MANIFEST_QA_JSON).expect("embedded qa manifest must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_schema() {
        let json = r#"{ "schema": "elf-manifest/999", "files": [] }"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn parses_minimal_entry() {
        let json = r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "templates/x.md", "dest": "0_Meta/x.md", "tier": "managed", "sha256": "ab" }
        ] }"#;
        let m = parse(json).unwrap();
        assert_eq!(m.files.len(), 1);
        assert_eq!(m.files[0].tier, Tier::Managed);
    }

    #[test]
    fn rejects_unknown_tier() {
        let json = r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "a", "dest": "b", "tier": "wild", "sha256": "ab" }
        ] }"#;
        assert!(parse(json).is_err());
    }
}
