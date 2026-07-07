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

/// BCP-47 태그 → primary subtag(소문자). "en-US"→"en", "ko-KR"→"ko", ""→"".
pub fn lang_primary(tag: &str) -> String {
    tag.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase()
}

/// 프로젝트 레이아웃 (S024/B — managed payload 위치). `.elf/config.json`의 `layout` 필드.
/// 필드 부재(구 프로젝트) = Legacy. 신규 init = Managed. 이전은 `elf migrate`(opt-in)만.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// 규칙 payload가 `0_Meta/`·`templates/`에 배치 (pre-relocation 레이아웃)
    #[default]
    Legacy,
    /// 규칙 payload = `.elf/managed/`(+`.elf/managed/templates/`), `0_Meta/` = 프로젝트 전용
    Managed,
}

/// managed tier dest의 정규형(managed 레이아웃 좌표) 변환. legacy 표기(`0_Meta/`·`templates/`)를
/// `.elf/managed/`로 올림. managed tier 외(seed ProjectRule·Registry 등)는 불변 — 이동 대상 아님.
pub fn dest_to_managed(dest: &str, tier: Tier) -> String {
    if tier != Tier::Managed {
        return dest.to_string();
    }
    if let Some(rest) = dest.strip_prefix("0_Meta/") {
        return format!(".elf/managed/{rest}");
    }
    if let Some(rest) = dest.strip_prefix("templates/") {
        return format!(".elf/managed/templates/{rest}");
    }
    dest.to_string()
}

/// managed 레이아웃 dest → legacy 표기. `.elf/managed/` 밖은 불변.
pub fn dest_to_legacy(dest: &str) -> String {
    if let Some(rest) = dest.strip_prefix(".elf/managed/templates/") {
        return format!("templates/{rest}");
    }
    if let Some(rest) = dest.strip_prefix(".elf/managed/") {
        return format!("0_Meta/{rest}");
    }
    dest.to_string()
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

    /// dest를 프로젝트 레이아웃 좌표로 정규화한 사본 (S024/B).
    /// 입력 dest의 표기(구 stamp=legacy, 신 manifest=managed)와 무관하게 동작 —
    /// 정규형(managed)으로 통일 후 목표 레이아웃으로 변환하므로, 구 stamp와 신 manifest를
    /// 같은 좌표계에서 plan_update로 비교할 수 있다.
    pub fn for_layout(&self, layout: Layout) -> Manifest {
        let files = self
            .files
            .iter()
            .map(|e| {
                let canon = dest_to_managed(&e.dest, e.tier);
                let dest = match layout {
                    Layout::Managed => canon,
                    Layout::Legacy => dest_to_legacy(&canon),
                };
                Entry { dest, ..e.clone() }
            })
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
    fn dest_mapping_is_tier_aware_and_idempotent() {
        // managed tier: legacy 표기 → managed 정규형
        assert_eq!(dest_to_managed("0_Meta/EliRule.md", Tier::Managed), ".elf/managed/EliRule.md");
        assert_eq!(
            dest_to_managed("templates/trialTemplate.md", Tier::Managed),
            ".elf/managed/templates/trialTemplate.md"
        );
        // 이미 정규형이면 불변 (멱등)
        assert_eq!(
            dest_to_managed(".elf/managed/EliRule.md", Tier::Managed),
            ".elf/managed/EliRule.md"
        );
        // seed(ProjectRule·Registry)는 이동 대상 아님
        assert_eq!(dest_to_managed("0_Meta/ProjectRule.md", Tier::Seed), "0_Meta/ProjectRule.md");
        // root 파일 불변
        assert_eq!(dest_to_managed("AGENTS.md", Tier::Managed), "AGENTS.md");
        assert_eq!(dest_to_managed(".gitignore", Tier::Hybrid), ".gitignore");
        // 역방향
        assert_eq!(dest_to_legacy(".elf/managed/EliRule.md"), "0_Meta/EliRule.md");
        assert_eq!(
            dest_to_legacy(".elf/managed/templates/sessionTemplate.md"),
            "templates/sessionTemplate.md"
        );
        assert_eq!(dest_to_legacy("AGENTS.md"), "AGENTS.md");
    }

    #[test]
    fn for_layout_normalizes_old_and_new_stamps_to_same_coords() {
        // 신 manifest(managed dest) + 구 stamp(legacy dest) → 같은 레이아웃 좌표로 수렴
        let new_style = parse(
            r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "templates/meta/EliRule.md", "dest": ".elf/managed/EliRule.md", "tier": "managed", "sha256": "n" },
            { "src": "templates/meta/ProjectRule.md", "dest": "0_Meta/ProjectRule.md", "tier": "seed", "sha256": "p" }
        ] }"#,
        )
        .unwrap();
        let old_style = parse(
            r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "templates/meta/EliRule.md", "dest": "0_Meta/EliRule.md", "tier": "managed", "sha256": "o" },
            { "src": "templates/meta/ProjectRule.md", "dest": "0_Meta/ProjectRule.md", "tier": "seed", "sha256": "p" }
        ] }"#,
        )
        .unwrap();
        for layout in [Layout::Legacy, Layout::Managed] {
            let a = new_style.for_layout(layout);
            let b = old_style.for_layout(layout);
            assert_eq!(a.files[0].dest, b.files[0].dest, "{layout:?} 좌표 불일치");
            assert_eq!(a.files[1].dest, "0_Meta/ProjectRule.md", "seed는 레이아웃 무관");
        }
        assert_eq!(new_style.for_layout(Layout::Legacy).files[0].dest, "0_Meta/EliRule.md");
        assert_eq!(old_style.for_layout(Layout::Managed).files[0].dest, ".elf/managed/EliRule.md");
    }

    #[test]
    fn rejects_unknown_tier() {
        let json = r#"{ "schema": "elf-manifest/1", "files": [
            { "src": "a", "dest": "b", "tier": "wild", "sha256": "ab" }
        ] }"#;
        assert!(parse(json).is_err());
    }
}
