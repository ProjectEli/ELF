//! t03 FS 통합: `elf init`이 generator와 구조 동등한 트리를 만드는지 tempdir에서 검증.

use elf_cli::init::{run_init, InitError, InitOptions};
use elf_cli::{embed, manifest};
use tempfile::tempdir;

fn opts(name: &str) -> InitOptions {
    InitOptions {
        name: name.into(),
        preset: "full".into(),
        modules: None,
        lang: "한국어".into(),
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
        "templates",
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
    let target = run_init(tmp.path(), &opts("P2")).unwrap();

    for e in &manifest::embedded().files {
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

    // 치환 대상 아닌 seed(AI_Sync)는 정본 그대로
    let sync = std::fs::read(target.join("0_Meta/AI_Sync.md")).unwrap();
    assert_eq!(sync, embed::TEMPLATES.get_file("meta/AI_Sync.md").unwrap().contents());
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
    assert_eq!(config["lang"], "한국어");
    assert_eq!(config["created"], "2026-06-12");

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
fn unknown_preset_errors_with_valid_list() {
    let tmp = tempdir().unwrap();
    let mut o = opts("P7");
    o.preset = "wild".into();
    match run_init(tmp.path(), &o) {
        Err(InitError::Plan(msg)) => assert!(msg.contains("full"), "msg: {msg}"),
        other => panic!("expected Plan error, got {other:?}"),
    }
}
