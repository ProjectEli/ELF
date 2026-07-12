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
    /// data overlay 허용 — 유효 규칙 = base ⊕ `0_Meta/<이름>.project.md` (EliRule §2.7).
    /// true = 항목 add/remove/override를 프로젝트가 overlay로 자가운영(base 직접 수정 불요).
    #[serde(default)]
    pub overlayable: bool,
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
    /// 부재 시 생성(init·update 공통), 존재 시 **절대 불변경**(내용 무관 — `.elf-new` 병기도 없음).
    /// 얇은 진입 어댑터(CLAUDE.md `@AGENTS.md` 포인터) 전용 — 기존 사용자 파일 소유권 존중 (S021 t06)
    Pointer,
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

/// manifest 계보(프로젝트 유형 정체성) — 연구/qa/general 중 어느 정본 세트를 따르는가 (S026).
/// init이 `.elf/config.json`의 `preset`에 기록하고 update/status가 소비한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Research,
    Qa,
    General,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Research => "research",
            Kind::Qa => "qa",
            Kind::General => "general",
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// preset 문자열 → 계보. init 분기와 update/status 해석의 **단일 소스** —
/// full/experimental/software/minimal(폴더 preset)은 전부 연구 계보.
pub fn kind_from_preset(preset: &str) -> Kind {
    match preset {
        "qa" => Kind::Qa,
        "general" => Kind::General,
        _ => Kind::Research,
    }
}

/// 계보별 embedded 정본 manifest.
pub fn embedded_kind(kind: Kind) -> Manifest {
    match kind {
        Kind::Research => embedded(),
        Kind::Qa => embedded_qa(),
        Kind::General => embedded_general(),
    }
}

/// 계보별 정본 manifest JSON 원문 — `.elf/manifest.json` stamp 기록용 (init·update re-stamp 공용).
pub fn kind_json(kind: Kind) -> &'static str {
    match kind {
        Kind::Research => crate::embed::MANIFEST_JSON,
        Kind::Qa => crate::embed::MANIFEST_QA_JSON,
        Kind::General => crate::embed::MANIFEST_GENERAL_JSON,
    }
}

/// BCP-47 태그 → primary subtag(소문자). "en-US"→"en", "ko-KR"→"ko", ""→"".
pub fn lang_primary(tag: &str) -> String {
    tag.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase()
}

impl Manifest {
    /// stamp의 계보를 src 경로 시그니처로 판정 — 결정적(no-LLM) 문자열 규칙 (S026 t02).
    /// 근거: 3 정본 manifest의 src 구성이 계보별 상호배타(qa 전 항목 `templates/qa/`,
    /// general만 `templates/general/` 보유, 연구는 둘 다 없음) — 상호배타성은 unit test로 고정.
    /// `preset` 미기재 구버전 config의 fallback 추론 + config 선언과의 대조 게이트에 사용.
    pub fn src_signature(&self) -> Kind {
        if self.files.iter().any(|e| e.src.starts_with("templates/qa/")) {
            Kind::Qa
        } else if self.files.iter().any(|e| e.src.starts_with("templates/general/")) {
            Kind::General
        } else {
            Kind::Research
        }
    }

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
    fn overlayable_parses_and_defaults_false() {
        let json = r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "templates/meta/LLMcliche.md", "dest": "0_Meta/LLMcliche.md", "tier": "managed", "sha256": "ab", "overlayable": true },
            { "src": "templates/meta/EliRule.md", "dest": "0_Meta/EliRule.md", "tier": "managed", "sha256": "cd" }
        ] }"#;
        let m = parse(json).unwrap();
        assert!(m.files[0].overlayable);
        assert!(!m.files[1].overlayable, "미지정 entry는 overlay 비허용이 기본");
    }

    #[test]
    fn rejects_unknown_tier() {
        let json = r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "a", "dest": "b", "tier": "wild", "sha256": "ab" }
        ] }"#;
        assert!(parse(json).is_err());
    }

    // ── 계보 (Kind — S026) ─────────────────────────────

    #[test]
    fn kind_from_preset_maps_folder_presets_to_research() {
        assert_eq!(kind_from_preset("qa"), Kind::Qa);
        assert_eq!(kind_from_preset("general"), Kind::General);
        for p in ["full", "experimental", "software", "minimal", "unknown-preset"] {
            assert_eq!(kind_from_preset(p), Kind::Research, "preset {p}");
        }
    }

    /// src 시그니처의 전제 = 3 정본 manifest의 계보별 상호배타 — 정본 구성이 바뀌어
    /// 전제가 깨지면 여기서 적발 (update/status의 preset 추론·게이트가 이 전제에 의존).
    #[test]
    fn embedded_manifests_have_mutually_exclusive_signatures() {
        assert_eq!(embedded().src_signature(), Kind::Research);
        assert_eq!(embedded_qa().src_signature(), Kind::Qa);
        assert_eq!(embedded_general().src_signature(), Kind::General);
        // 상호배타의 원자 검증: 연구 manifest에 qa/general src 부재, qa↔general 교차 부재
        assert!(embedded().files.iter().all(|e| {
            !e.src.starts_with("templates/qa/") && !e.src.starts_with("templates/general/")
        }));
        assert!(embedded_qa().files.iter().all(|e| !e.src.starts_with("templates/general/")));
        assert!(embedded_general().files.iter().all(|e| !e.src.starts_with("templates/qa/")));
    }

    #[test]
    fn kind_json_roundtrips_to_matching_signature() {
        for k in [Kind::Research, Kind::Qa, Kind::General] {
            assert_eq!(parse(kind_json(k)).unwrap().src_signature(), k);
        }
    }
}
