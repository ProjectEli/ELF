//! 배포 명세(manifest) 파싱 — elf-cli/manifest.json (schema: elf-manifest/1).
//! src = CLI 번들 상대(templates/...), dest = 프로젝트 루트 상대. (P009 §3)

use serde::Deserialize;

pub const SCHEMA: &str = "elf-manifest/1";

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Default, Clone, Deserialize)]
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
    /// 이 entry가 겨냥하는 BCP-47 primary subtag (예: "en"). None = base(PROJECT_LANG-중립 정본). (P016)
    #[serde(default)]
    pub lang: Option<String>,
    /// i18n 역할: Companion(비operative `*.en.md` sibling) | Variant(base dest 대체). None = 일반. (P016)
    #[serde(default)]
    pub role: Option<Role>,
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

/// i18n entry 역할 (P016 §9). 비operative companion vs base 대체 variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// `*.en.md` sibling — operative 정본과 함께 배포, 비operative 정보용(인간 독해).
    Companion,
    /// base dest를 해당 lang 정본으로 대체 (seed README/ProjectRule 등 사용자 소유 파일).
    Variant,
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

/// QA preset(experimental) 정본 manifest — 질문 아카이브 유형 (연구 manifest와 분리).
pub fn embedded_qa() -> Manifest {
    parse(crate::embed::MANIFEST_QA_JSON).expect("embedded qa manifest must be valid")
}

/// general preset(experimental) 정본 manifest — 목표지향 비연구 유형 (중립 파일은 연구와 src 공유).
pub fn embedded_general() -> Manifest {
    parse(crate::embed::MANIFEST_GENERAL_JSON).expect("embedded general manifest must be valid")
}

/// BCP-47 태그 → primary subtag(소문자). "en-US"→"en", "ko-KR"→"ko", ""→"".
pub fn lang_primary(tag: &str) -> String {
    tag.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase()
}

impl Manifest {
    /// 프로젝트 언어(BCP-47)에 맞춰 배포 entry를 해석한 사본 (P016 §9).
    /// - base(lang=None): 같은 dest의 variant가 이 lang에 있으면 제외(대체됨), 아니면 포함.
    /// - companion: lang 일치 + 비-ko 일 때 포함(`*.en.md` sibling).
    /// - variant: lang 일치 시 포함(base dest 대체).
    /// ko(또는 미지정)는 base만 남아 현행 동작과 동일.
    pub fn for_lang(&self, tag: &str) -> Manifest {
        let p = lang_primary(tag);
        let replaced: std::collections::BTreeSet<&str> = self
            .files
            .iter()
            .filter(|e| {
                e.role == Some(Role::Variant)
                    && e.lang.as_deref().map(lang_primary) == Some(p.clone())
            })
            .map(|e| e.dest.as_str())
            .collect();
        let files = self
            .files
            .iter()
            .filter(|e| match (e.lang.as_deref(), e.role) {
                (None, _) => !replaced.contains(e.dest.as_str()),
                (Some(l), Some(Role::Companion)) => lang_primary(l) == p && p != "ko",
                (Some(l), _) => lang_primary(l) == p,
            })
            .cloned()
            .collect();
        Manifest {
            schema: self.schema.clone(),
            generated: self.generated.clone(),
            note: self.note.clone(),
            files,
            dirs: self.dirs.clone(),
        }
    }
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
