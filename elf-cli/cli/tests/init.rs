//! t03 FS 통합: `elf init`이 generator와 구조 동등한 트리를 만드는지 tempdir에서 검증.

use elf_cli::init::{run_init, InitError, InitOptions};
use elf_cli::{embed, manifest};
use tempfile::tempdir;

fn opts(name: &str) -> InitOptions {
    InitOptions {
        name: name.into(),
        preset: "full".into(),
        modules: None,
        categories: Vec::new(),
        lang: "ko-KR".into(),
        date: "2026-06-12".into(),
    }
}

#[test]
fn full_init_creates_expected_tree() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("P1")).unwrap();

    for dir in [
        "0_Meta",
        "1_Concept/11_Literature",
        "1_Concept/12_Planning/Wiki",
        "1_Concept/12_Planning/Archive",
        "1_Concept/13_Ideas",
        "2_Log/Wiki",
        "2_Log/Archive",
        "3_HW/31_Component/Design",
        "4_Fab/41_Recipes",
        "5_SW/53_Libs",
        "6_Exp/61_Sim/Data",
        "7_Paper/73_Presentations",
    ] {
        assert!(target.join(dir).is_dir(), "missing dir: {dir}");
    }
    // 일반 폴더 = .gitkeep
    assert!(target.join("1_Concept/13_Ideas/.gitkeep").is_file());
    // raw 데이터 폴더 = `*` + `!.gitignore` (git 추적 제외)
    let raw = std::fs::read_to_string(target.join("6_Exp/62_Empirical/Raw/.gitignore")).unwrap();
    assert_eq!(raw, "*\n!.gitignore\n");
    assert!(!target.join("6_Exp/62_Empirical/Raw/.gitkeep").exists());
    // 빈 루트 파일
    assert!(target.join(".gitattributes").is_file());
    assert!(target.join("LICENSE").is_file());
}

/// 불변식: managed/hybrid dest는 embed 정본과 바이트 동일 (placeholder 치환 금지)
#[test]
fn managed_and_hybrid_are_byte_identical_to_embed() {
    let tmp = tempdir().unwrap();
    let o = opts("P2");
    let target = run_init(tmp.path(), &o).unwrap();

    // 배포된 lang view 기준 (ko = base만; en companion은 ko 프로젝트에 미배포) — P016
    for e in &manifest::embedded().for_lang(&o.lang).files {
        if e.tier == manifest::Tier::Seed {
            continue;
        }
        let rel = e.src.strip_prefix("templates/").unwrap();
        let expected = embed::TEMPLATES.get_file(rel).unwrap().contents();
        let actual = std::fs::read(target.join(&e.dest)).unwrap();
        assert_eq!(actual, expected, "managed/hybrid not byte-identical: {}", e.dest);
    }
}

#[test]
fn seed_substitution_applied_and_no_placeholders_remain() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("MyProj")).unwrap();

    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("MyProj"));
    assert!(!readme.contains("PLACEHOLDER_"));

    let rule = std::fs::read_to_string(target.join("0_Meta/ProjectRule.md")).unwrap();
    assert!(rule.contains("MyProj"));
    assert!(rule.contains("2026-06-12"));
    assert!(!rule.contains("[프로젝트명]"));

    let registry = std::fs::read_to_string(target.join("2_Log/Wiki/Session_Registry.tsv")).unwrap();
    assert!(!registry.contains("YYYY-MM-DD"));
}

/// P016: `--lang en` → KO operative 정본 유지 + EN companion 동반 + seed는 EN variant로 대체
#[test]
fn en_init_deploys_companions_and_variants() {
    let tmp = tempdir().unwrap();
    let o = InitOptions { lang: "en-US".into(), ..opts("EnProj") };
    let target = run_init(tmp.path(), &o).unwrap();

    // KO operative 정본은 그대로 배포
    assert!(target.join(".elf/managed/EliRule.md").is_file());

    // EN companion 동반 배포 + embed 정본과 byte-identical (managed, 비치환)
    let comp = std::fs::read(target.join(".elf/managed/EliRule.en.md")).unwrap();
    let expected = embed::TEMPLATES.get_file("meta/EliRule.en.md").unwrap().contents();
    assert_eq!(comp, expected, "EN companion must be byte-identical to embed");
    for c in ["LogConvention", "AI_PARA_Framework", "LLMcliche", "highIFjournals"] {
        assert!(target.join(format!(".elf/managed/{c}.en.md")).is_file(), "missing companion: {c}");
    }

    // seed variant: README.md = EN 내용(base 대체) + placeholder 치환
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("Project Overview"), "README should be EN variant");
    assert!(!readme.contains("프로젝트 개요"), "README should not be KO base");
    assert!(readme.contains("EnProj") && !readme.contains("PLACEHOLDER_"));

    // ProjectRule = EN scaffold + name 치환
    let pr = std::fs::read_to_string(target.join("0_Meta/ProjectRule.md")).unwrap();
    assert!(pr.contains("Project Overview"), "ProjectRule should be EN scaffold");
    assert!(pr.contains("EnProj") && !pr.contains("[Project Name]"));
}

#[test]
fn s001_log_derived_from_session_template() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("P3")).unwrap();
    let s001 = std::fs::read_to_string(target.join("2_Log/S001_log.md")).unwrap();
    assert!(s001.contains("S001"));
    assert!(!s001.contains("S{NNN}"));
    assert!(s001.contains("2026-06-12"));
}

#[test]
fn elf_control_plane_written() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("P4")).unwrap();

    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(target.join(".elf/config.json")).unwrap())
            .unwrap();
    assert_eq!(config["name"], "P4");
    assert_eq!(config["lang"], "ko-KR");
    assert_eq!(config["created"], "2026-06-12");
    assert_eq!(config["layout"], "managed", "신규 init = managed 레이아웃");

    let version = std::fs::read_to_string(target.join(".elf/version")).unwrap();
    assert_eq!(version, format!("{}\n", embed::version()));

    // manifest stamp = embed 정본 사본 + 파싱 가능
    let stamp_text = std::fs::read_to_string(target.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp_text, embed::MANIFEST_JSON);
    assert!(manifest::parse(&stamp_text).is_ok());
}

#[test]
fn minimal_preset_creates_core_only() {
    let tmp = tempdir().unwrap();
    let mut o = opts("P5");
    o.preset = "minimal".into();
    let target = run_init(tmp.path(), &o).unwrap();
    assert!(target.join("0_Meta").is_dir());
    assert!(target.join("2_Log/Wiki").is_dir());
    assert!(!target.join("3_HW").exists());
    assert!(!target.join("7_Paper").exists());
}

#[test]
fn custom_modules_override_preset() {
    let tmp = tempdir().unwrap();
    let mut o = opts("P6");
    o.modules = Some(vec!["sw".into()]);
    let target = run_init(tmp.path(), &o).unwrap();
    assert!(target.join("5_SW/51_FW").is_dir());
    assert!(!target.join("3_HW").exists());
    assert!(!target.join("6_Exp").exists());
}

#[test]
fn refuse_when_target_exists() {
    let tmp = tempdir().unwrap();
    run_init(tmp.path(), &opts("DUP")).unwrap();
    match run_init(tmp.path(), &opts("DUP")) {
        Err(InitError::TargetExists(_)) => {}
        other => panic!("expected TargetExists, got {other:?}"),
    }
}

#[test]
fn qa_preset_creates_question_archive_and_skips_research() {
    let tmp = tempdir().unwrap();
    let mut o = opts("MyQA");
    o.preset = "qa".into();
    let target = run_init(tmp.path(), &o).unwrap();

    // qa 유형 콘텐츠 — 규칙은 루트 CLAUDE.md(LLM 자동 로드)
    assert!(target.join("CLAUDE.md").is_file());
    assert!(target.join(".elf/managed/templates/bundle_template.md").is_file());
    assert!(!target.join("0_Meta").exists(), "qa는 0_Meta 미사용");
    // 기본 = 카테고리 0개 (수요 기반 생성 — CLAUDE.md 규약)
    assert!(!target.join("일상질문").exists(), "기본 qa는 사전 카테고리 없음");
    // seed README 치환 (qa도 공용 seed 경로)
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("MyQA") && !readme.contains("PLACEHOLDER_"));

    // 연구 유형 격리 — 세션·연구 managed 파일 미생성
    assert!(!target.join("2_Log/S001_log.md").exists());
    assert!(!target.join(".elf/managed/EliRule.md").exists());
    assert!(!target.join(".elf/managed/LogConvention.md").exists());
    assert!(!target.join("2_Log/Wiki/Session_Registry.tsv").exists());
    assert!(!target.join("3_HW").exists());
    // P2: qa는 빈 .gitattributes·LICENSE cruft 없음, 불필요한 templates/.gitkeep 없음
    assert!(!target.join(".gitattributes").exists());
    assert!(!target.join("LICENSE").exists());
    assert!(!target.join("templates/.gitkeep").exists());

    // .elf stamp = qa manifest (update가 qa 파일만 관리)
    let stamp = std::fs::read_to_string(target.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, embed::MANIFEST_QA_JSON);
    assert!(manifest::parse(&stamp).is_ok());
    // 공용 spine — config/version 동일
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(target.join(".elf/config.json")).unwrap())
            .unwrap();
    assert_eq!(config["name"], "MyQA");
}

#[test]
fn qa_categories_flag_pre_creates_folders() {
    let tmp = tempdir().unwrap();
    let mut o = opts("CatQA");
    o.preset = "qa".into();
    o.categories = vec!["일상질문".into(), "메모".into()];
    let target = run_init(tmp.path(), &o).unwrap();
    for c in ["일상질문", "일상질문/archive", "메모", "메모/archive"] {
        assert!(target.join(c).is_dir(), "missing category dir: {c}");
    }
    assert!(target.join("일상질문/.gitkeep").is_file());
    assert!(!target.join("LLMHowto").exists(), "미지정 카테고리는 미생성");
}

#[test]
fn general_preset_scaffolds_neutral_project() {
    let tmp = tempdir().unwrap();
    let mut o = opts("MyGen");
    o.preset = "general".into();
    let target = run_init(tmp.path(), &o).unwrap();

    // general 전용 EliRule/LogConvention + 공유 AI_PARA·Registry
    assert!(target.join(".elf/managed/EliRule.md").is_file());
    assert!(target.join(".elf/managed/LogConvention.md").is_file());
    assert!(target.join(".elf/managed/AI_PARA_Framework.md").is_file());
    assert!(target.join("2_Log/Wiki/Session_Registry.tsv").is_file());
    // general EliRule = 학술 제외(외부 문헌 검색·highIFjournals 부재) + general 명시
    let eli = std::fs::read_to_string(target.join(".elf/managed/EliRule.md")).unwrap();
    assert!(eli.contains("general"));
    assert!(!eli.contains("highIFjournals") && !eli.contains("외부 문헌 검색"));
    assert!(!target.join(".elf/managed/highIFjournals.md").exists());
    // general은 세션 사용 → S001 생성 (qa와 차이)
    assert!(target.join("2_Log/S001_log.md").is_file());
    // 학술 폴더 미생성
    assert!(!target.join("6_Exp").exists());
    assert!(!target.join("7_Paper").exists());
    assert!(!target.join("1_Concept/11_Literature").exists());
    // 공유 spine 템플릿
    assert!(target.join(".elf/managed/templates/sessionTemplate.md").is_file());
    // general 전용 README (seed 치환)
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("MyGen") && readme.contains("general") && !readme.contains("PLACEHOLDER_"));
    // .elf stamp = general manifest
    let stamp = std::fs::read_to_string(target.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, embed::MANIFEST_GENERAL_JSON);
    assert!(manifest::parse(&stamp).is_ok());
}

#[test]
fn qa_invalid_category_is_refused() {
    let tmp = tempdir().unwrap();
    let mut o = opts("BadCat");
    o.preset = "qa".into();
    o.categories = vec!["../escape".into()];
    match run_init(tmp.path(), &o) {
        Err(InitError::Plan(msg)) => assert!(msg.contains("category"), "msg: {msg}"),
        other => panic!("expected Plan error, got {other:?}"),
    }
}

#[test]
fn unknown_preset_errors_with_valid_list() {
    let tmp = tempdir().unwrap();
    let mut o = opts("P7");
    o.preset = "wild".into();
    match run_init(tmp.path(), &o) {
        Err(InitError::Plan(msg)) => assert!(msg.contains("full"), "msg: {msg}"),
        other => panic!("expected Plan error, got {other:?}"),
    }
}
