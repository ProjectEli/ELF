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
        "2_Log/S",
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
        if e.tier == manifest::Tier::User {
            continue;
        }
        let rel = e.src.strip_prefix("templates/").unwrap();
        let expected = embed::TEMPLATES.get_file(rel).unwrap().contents();
        let actual = std::fs::read(target.join(&e.dest)).unwrap();
        assert_eq!(actual, expected, "managed/hybrid not byte-identical: {}", e.dest);
    }
}

#[test]
fn user_tier_substitution_applied_and_no_placeholders_remain() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("MyProj")).unwrap();

    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("MyProj"));
    assert!(!readme.contains("PLACEHOLDER_"));

    let rule = std::fs::read_to_string(target.join("0_Meta/ProjectRule.md")).unwrap();
    assert!(rule.contains("MyProj"));
    assert!(rule.contains("2026-06-12"));
    assert!(!rule.contains("[프로젝트명]"));

    // 세션 Registry.md = 도구 생성(치환 대상 아님), TSV 없음(P022)
    assert!(!target.join("2_Log/Wiki/Session_Registry.tsv").exists());
    assert!(target.join("2_Log/S/Registry.md").is_file());
}

/// v3.4(S050 t02): `--lang en` → 영어 규칙 세트가 같은 경로의 정본(번역본 `.en.md` 없음), 한글 0건.
#[test]
fn en_init_deploys_english_rule_set() {
    let tmp = tempdir().unwrap();
    let o = InitOptions { lang: "en-US".into(), ..opts("EnProj") };
    let target = run_init(tmp.path(), &o).unwrap();

    // 영어 규칙 문서 = 같은 경로, embed 영어본과 byte-identical
    for (dest, src) in [
        (".elf/managed/EliRule.md", "meta/EliRule.en.md"),
        (".elf/managed/LogConvention.md", "meta/LogConvention.en.md"),
        (".elf/managed/AI_PARA_Framework.md", "meta/AI_PARA_Framework.en.md"),
        (".elf/managed/LLMcliche.md", "meta/LLMcliche.en.md"),
        (".elf/managed/highIFjournals.md", "meta/highIFjournals.en.md"),
        ("AGENTS.md", "root/AGENTS.en.md"),
        (".editorconfig", "root/.editorconfig.en"),
        (".elf/managed/templates/trialTemplate.md", "log/trialTemplate.en.md"),
    ] {
        let got = std::fs::read(target.join(dest)).unwrap();
        assert_eq!(got, embed::TEMPLATES.get_file(src).unwrap().contents(), "{dest} must be {src}");
    }
    for c in ["EliRule", "LogConvention", "AI_PARA_Framework", "LLMcliche", "highIFjournals"] {
        assert!(!target.join(format!(".elf/managed/{c}.en.md")).exists(), "no translation copy: {c}");
    }
    assert!(!target.join("AGENTS.en.md").exists());

    // user tier variant: README.md, ProjectRule = 영어 + placeholder 치환
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("Project Overview") && readme.contains("EnProj") && !readme.contains("PLACEHOLDER_"));
    let pr = std::fs::read_to_string(target.join("0_Meta/ProjectRule.md")).unwrap();
    assert!(pr.contains("Project Overview") && pr.contains("EnProj") && !pr.contains("[Project Name]"));

    assert_eq!(hangul_files(&target), Vec::<String>::new(), "an English project has no Korean text");
}

/// v3.4(S050 t02): qa preset `--lang en`도 한글 0건.
#[test]
fn en_qa_init_has_no_korean() {
    let tmp = tempdir().unwrap();
    let o = InitOptions { lang: "en-US".into(), preset: "qa".into(), categories: vec!["Daily".into()], ..opts("EnQa") };
    let target = run_init(tmp.path(), &o).unwrap();
    let agents = std::fs::read_to_string(target.join("AGENTS.md")).unwrap();
    assert!(agents.contains("question archive (QA preset)"), "{agents}");
    assert_eq!(hangul_files(&target), Vec::<String>::new());
}

/// 프로젝트 안 한글이 든 파일(루트 기준 경로, `.git` 제외).
fn hangul_files(root: &std::path::Path) -> Vec<String> {
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if p.is_dir() {
                walk(&p, root, out);
            } else if let Ok(t) = std::fs::read_to_string(&p)
                && t.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
            {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// P022: init은 세션을 만들지 않음 — 첫 세션은 `elf session new --goal`(목표 필수). 빈 Registry.md만 생성.
#[test]
fn init_creates_no_session_and_empty_registry() {
    let tmp = tempdir().unwrap();
    let target = run_init(tmp.path(), &opts("P3")).unwrap();
    assert!(target.join("2_Log/S").is_dir());
    assert!(!target.join("2_Log/S/S001_log.md").exists());
    assert!(!target.join("2_Log/S001_log.md").exists());
    let reg = std::fs::read_to_string(target.join("2_Log/S/Registry.md")).unwrap();
    assert!(reg.contains("| 세션 | status |") && !reg.contains("S001"), "{reg}");
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
    assert_eq!(config["preset"], "full", "계보 정체성 영속화 (S026)");
    assert_eq!(config["created"], "2026-06-12");

    // 버전 = manifest elf_version(schema 2) — `.elf/version` 없음 (P022 §8)
    assert!(!target.join(".elf/version").exists());
    let stamp_text = std::fs::read_to_string(target.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp_text, manifest::project_json(manifest::Kind::Research));
    let m = manifest::parse(&stamp_text).unwrap();
    assert_eq!((m.schema.as_str(), m.elf_version.as_deref()), (manifest::SCHEMA, Some(embed::version())));
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

    // qa 유형 콘텐츠 — 규칙 정본 = 루트 AGENTS.md(3.6부터 CLAUDE.md 없음), 프로젝트 규칙 = 0_Meta/ProjectRule.md
    assert!(target.join("AGENTS.md").is_file() && !target.join("CLAUDE.md").exists());
    assert!(target.join(".elf/managed/templates/bundle_template.md").is_file());
    // 0_Meta는 사용자 소유 규칙 파일 1개만(v2.27 — 연구 전용 파일 없음)
    let meta: Vec<_> = std::fs::read_dir(target.join("0_Meta")).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(meta, vec![std::ffi::OsString::from("ProjectRule.md")], "qa 0_Meta = ProjectRule만");
    // 기본 = 카테고리 0개 (수요 기반 생성 — CLAUDE.md 규약)
    assert!(!target.join("일상질문").exists(), "기본 qa는 사전 카테고리 없음");
    // user tier README 치환 (qa도 공용 user tier 경로)
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("MyQA") && !readme.contains("PLACEHOLDER_"));

    // 연구 유형 격리 — 세션, 연구 managed 파일 미생성
    assert!(!target.join("2_Log/S001_log.md").exists());
    assert!(!target.join(".elf/managed/EliRule.md").exists());
    assert!(!target.join(".elf/managed/LogConvention.md").exists());
    assert!(!target.join("2_Log/Wiki/Session_Registry.tsv").exists());
    assert!(!target.join("3_HW").exists());
    // P2: qa는 빈 .gitattributes, LICENSE cruft 없음, 불필요한 templates/.gitkeep 없음
    assert!(!target.join(".gitattributes").exists());
    assert!(!target.join("LICENSE").exists());
    assert!(!target.join("templates/.gitkeep").exists());

    // .elf stamp = qa manifest (update가 qa 파일만 관리)
    let stamp = std::fs::read_to_string(target.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, manifest::project_json(manifest::Kind::Qa));
    assert!(manifest::parse(&stamp).is_ok());
    // 공용 spine — config/version 동일 + preset 정체성 기록 (S026)
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(target.join(".elf/config.json")).unwrap())
            .unwrap();
    assert_eq!(config["name"], "MyQA");
    assert_eq!(config["preset"], "qa");
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

/// general preset은 v2.27에서 삭제(별칭 없음) — 미지 preset으로 거부.
#[test]
fn general_preset_is_rejected_as_unknown() {
    let tmp = tempdir().unwrap();
    let mut o = opts("MyGen");
    o.preset = "general".into();
    match run_init(tmp.path(), &o) {
        Err(InitError::Plan(m)) => assert!(m.contains("unknown preset: general"), "{m}"),
        other => panic!("expected Plan error, got {other:?}"),
    }
    assert!(!tmp.path().join("MyGen").exists(), "거부 시 폴더 미생성");
}

/// qa = 정식 preset(v2.27): 사용자 소유 규칙 파일 `0_Meta/ProjectRule.md` 생성(이름, 날짜 치환),
/// README와 AGENTS가 규칙 정본(AGENTS.md)과 ProjectRule을 안내.
#[test]
fn qa_preset_creates_project_rule_and_current_readme() {
    let tmp = tempdir().unwrap();
    let mut o = opts("MyQa");
    o.preset = "qa".into();
    let target = run_init(tmp.path(), &o).unwrap();
    let pr = std::fs::read_to_string(target.join("0_Meta/ProjectRule.md")).unwrap();
    assert!(pr.contains("ProjectRule: MyQa") && pr.contains("2026-06-12"), "{pr}");
    assert!(pr.contains("## 3. 외부 반출 완화"));
    let readme = std::fs::read_to_string(target.join("README.md")).unwrap();
    assert!(readme.contains("`AGENTS.md`") && readme.contains("0_Meta/ProjectRule.md"), "{readme}");
    assert!(!readme.contains("experimental") && !readme.contains("운영 규칙은 루트 `CLAUDE.md`"), "{readme}");
    let agents = std::fs::read_to_string(target.join("AGENTS.md")).unwrap();
    assert!(agents.contains("0_Meta/ProjectRule.md") && !agents.contains("experimental"));
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

// ── preset 이름 exp, sw (v2.26) — 구 이름 experimental, software는 CLI 숨김 별칭 ───

#[test]
fn exp_and_sw_presets_create_main_module_plus_paper() {
    for (preset, main_dir, absent) in [("exp", "6_Exp", "5_SW"), ("sw", "5_SW", "6_Exp")] {
        let tmp = tempdir().unwrap();
        let mut o = opts("P");
        o.preset = preset.into();
        let root = run_init(tmp.path(), &o).unwrap();
        assert!(root.join(main_dir).is_dir(), "{preset}: {main_dir}");
        assert!(root.join("7_Paper").is_dir(), "{preset}: 7_Paper");
        assert!(!root.join(absent).exists(), "{preset}: {absent} must not exist");
    }
}

#[test]
fn e2e_legacy_preset_names_are_accepted_and_recorded_as_new() {
    for (old, new) in [("experimental", "exp"), ("software", "sw")] {
        let tmp = tempdir().unwrap();
        assert_cmd::Command::cargo_bin("elf")
            .unwrap()
            .env("ELF_DISPATCH", "off")
            .args(["init", "P", "--preset", old])
            .current_dir(tmp.path())
            .assert()
            .success();
        let config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join("P/.elf/config.json")).unwrap()).unwrap();
        assert_eq!(config["preset"], new);
    }
}
