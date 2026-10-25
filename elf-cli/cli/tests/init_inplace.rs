//! P018/S015: in-place init (`run_init_ex` in_place=true) — 비파괴 배치 검증.
//! 기존 폴더를 제자리 ELF화하되 사용자 파일을 절대 덮어쓰지 않음(user skip / managed, hybrid .elf-new).

use elf_cli::init::{run_init_ex, InitError, InitOptions};
use std::fs;
use tempfile::tempdir;

fn opts(name: &str) -> InitOptions {
    InitOptions {
        name: name.into(),
        preset: "full".into(),
        modules: None,
        categories: Vec::new(),
        lang: "ko-KR".into(),
        date: "2026-06-26".into(),
    }
}

#[test]
fn in_place_empty_dir_scaffolds_and_marks_elf() {
    let tmp = tempdir().unwrap();
    let r = run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();
    assert!(tmp.path().join(".elf/manifest.json").is_file());
    assert!(tmp.path().join(".elf/managed/EliRule.md").is_file());
    assert!(tmp.path().join("README.md").is_file());
    assert!(r.elf_new.is_empty() && r.skipped.is_empty()); // 빈 폴더 = 충돌 없음
}

#[test]
fn in_place_preserves_existing_user_files() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("mydata.txt"), b"user content").unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.rs"), b"fn main(){}").unwrap();

    let r = run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();

    assert_eq!(fs::read(tmp.path().join("mydata.txt")).unwrap(), b"user content");
    assert_eq!(fs::read(tmp.path().join("src/main.rs")).unwrap(), b"fn main(){}");
    assert!(tmp.path().join(".elf/config.json").is_file());
    assert!(!r.created.is_empty());
}

#[test]
fn in_place_existing_gitignore_kept_elf_new_written() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join(".gitignore"), b"node_modules/\n").unwrap();

    let r = run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();

    // 사용자 .gitignore 불변 (hybrid → 덮어쓰지 않음)
    assert_eq!(fs::read(tmp.path().join(".gitignore")).unwrap(), b"node_modules/\n");
    // ELF본은 .elf-new로 병기
    assert!(tmp.path().join(".gitignore.elf-new").is_file());
    assert!(r.elf_new.iter().any(|n| n == ".gitignore.elf-new"));
}

#[test]
fn in_place_existing_user_readme_skipped_no_elf_new() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("README.md"), b"# My existing readme").unwrap();

    let r = run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();

    assert_eq!(fs::read(tmp.path().join("README.md")).unwrap(), b"# My existing readme");
    assert!(r.skipped.iter().any(|s| s == "README.md"));
    assert!(!tmp.path().join("README.md.elf-new").exists()); // user tier는 .elf-new 안 함
}

#[test]
fn in_place_already_elf_refuses() {
    let tmp = tempdir().unwrap();
    run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();
    match run_init_ex(tmp.path(), &opts("Proj"), true, false, false) {
        Err(InitError::AlreadyElf(_)) => {}
        other => panic!("expected AlreadyElf, got {other:?}"),
    }
}

#[test]
fn in_place_dry_run_writes_nothing() {
    let tmp = tempdir().unwrap();
    let r = run_init_ex(tmp.path(), &opts("Proj"), true, true, false).unwrap();
    assert!(!tmp.path().join(".elf").exists());
    assert!(!tmp.path().join(".elf/managed/EliRule.md").exists());
    assert!(!r.created.is_empty()); // plan은 산출(미기록)
}

#[test]
fn in_place_force_overwrites_existing_user_file() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("README.md"), b"old").unwrap();
    run_init_ex(tmp.path(), &opts("Proj"), true, false, true).unwrap();
    let readme = fs::read_to_string(tmp.path().join("README.md")).unwrap();
    assert_ne!(readme, "old"); // force → ELF user tier 파일로 교체
    assert!(readme.contains("Proj"));
}

// ── manifest 없는 구버전 폴더의 제자리 도입 (v2.27.1) ─────────────────────────

/// 구버전 생성기 폴더 재현: `.elf/`에 config.json과 version만, 세션 기록은 Archive에만.
fn old_folder(root: &std::path::Path) {
    fs::create_dir_all(root.join(".elf")).unwrap();
    fs::write(
        root.join(".elf/config.json"),
        "{\n  \"name\": \"OldProj\",\n  \"lang\": \"한국어\",\n  \"created\": \"2026-06-12\",\n  \"tsa\": true\n}\n",
    )
    .unwrap();
    fs::write(root.join(".elf/version"), "v2.4.2\n").unwrap();
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::write(root.join("2_Log/Archive/S001_log.md"), "# S001: first\n").unwrap();
}

#[test]
fn in_place_adopt_keeps_existing_config_keys() {
    let tmp = tempdir().unwrap();
    old_folder(tmp.path());
    run_init_ex(tmp.path(), &opts("OldProj"), true, false, false).unwrap();
    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tmp.path().join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(cfg["created"], "2026-06-12", "생성일 보존");
    assert_eq!(cfg["tsa"], true, "그 밖의 키 보존");
    assert_eq!(cfg["preset"], "full");
    assert!(tmp.path().join(".elf/manifest.json").is_file());
}

#[test]
fn in_place_adopt_does_not_create_duplicate_s001() {
    let tmp = tempdir().unwrap();
    old_folder(tmp.path());
    let r = run_init_ex(tmp.path(), &opts("OldProj"), true, false, false).unwrap();
    assert!(!tmp.path().join("2_Log/S001_log.md").exists(), "Archive에 S001이 있으면 새로 만들지 않음");
    assert!(r.notes.iter().any(|s| s.contains("already has session records")), "{:?}", r.notes);
}

/// e2e: `--preset`, `--lang`을 주지 않으면 기존 config 값을 씀(구 lang 표기도 그대로).
#[test]
fn e2e_in_place_adopt_uses_existing_preset_and_lang() {
    let tmp = tempdir().unwrap();
    old_folder(tmp.path());
    let cfg_path = tmp.path().join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();
    v["preset"] = "experimental".into();
    fs::write(&cfg_path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .args(["init", "--yes"])
        .current_dir(tmp.path())
        .assert()
        .success();
    let cfg: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();
    assert_eq!(cfg["preset"], "exp", "기존 preset(구 이름) → 현행 이름");
    assert_eq!(cfg["lang"], "한국어", "기존 lang 유지");
    assert_eq!(cfg["name"], "OldProj", "기존 name 유지");
    assert_eq!(cfg["created"], "2026-06-12");
    assert!(tmp.path().join("6_Exp").is_dir() && !tmp.path().join("3_HW").exists(), "exp 모듈로 도입");
}

/// `--modules` = config `preset: custom` + `modules` 목록(실제 모듈과 표기 일치).
#[test]
fn modules_are_recorded_as_custom_preset() {
    let tmp = tempdir().unwrap();
    let mut o = opts("Mod");
    o.modules = Some(vec!["sw".into()]);
    run_init_ex(tmp.path(), &o, true, false, false).unwrap();
    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tmp.path().join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(cfg["preset"], "custom");
    assert_eq!(cfg["modules"], serde_json::json!(["sw"]));
}

