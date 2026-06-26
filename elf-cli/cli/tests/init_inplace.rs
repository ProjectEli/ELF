//! P018/S015: in-place init (`run_init_ex` in_place=true) — 비파괴 배치 검증.
//! 기존 폴더를 제자리 ELF화하되 사용자 파일을 절대 덮어쓰지 않음(seed skip / managed·hybrid .elf-new).

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
    assert!(tmp.path().join("0_Meta/EliRule.md").is_file());
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
fn in_place_existing_seed_readme_skipped_no_elf_new() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("README.md"), b"# My existing readme").unwrap();

    let r = run_init_ex(tmp.path(), &opts("Proj"), true, false, false).unwrap();

    assert_eq!(fs::read(tmp.path().join("README.md")).unwrap(), b"# My existing readme");
    assert!(r.skipped.iter().any(|s| s == "README.md"));
    assert!(!tmp.path().join("README.md.elf-new").exists()); // seed는 .elf-new 안 함
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
    assert!(!tmp.path().join("0_Meta/EliRule.md").exists());
    assert!(!r.created.is_empty()); // plan은 산출(미기록)
}

#[test]
fn in_place_force_overwrites_existing_seed() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("README.md"), b"old").unwrap();
    run_init_ex(tmp.path(), &opts("Proj"), true, false, true).unwrap();
    let readme = fs::read_to_string(tmp.path().join("README.md")).unwrap();
    assert_ne!(readme, "old"); // force → ELF seed로 교체
    assert!(readme.contains("Proj"));
}
