//! t04 FS 통합: `elf update`의 무손실 보장을 tempdir 실트리에서 검증.
//! 시나리오 구성: init으로 실프로젝트 생성 → stamp/파일/baseline 조작으로 상태 연출 → update.
//!
//! 파일명이 update.rs가 아닌 이유: 테스트 바이너리명이 `update-<hash>.exe`가 되면
//! Windows UAC installer detection(update/setup/install/patch 키워드)이 권한 상승을
//! 요구해 실행 자체가 거부됨(os error 740). t06 self-update 바이너리 작명 시에도 동일 주의.

use elf_cli::init::{run_init, InitOptions};
use elf_cli::update::{run_update, UpdateOptions, MARKER_END, MARKER_START};
use elf_cli::{embed, hash, manifest};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn new_project(tmp: &Path, name: &str) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: name.into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "한국어".into(),
            date: "2026-06-12".into(),
        },
    )
    .unwrap()
}

fn plain() -> UpdateOptions {
    UpdateOptions { dry_run: false, force: false }
}

/// stamp의 특정 dest 항목 sha256을 가짜 구버전 해시로 바꿔 "정본이 그 후 갱신됨"을 연출
fn tamper_stamp_sha(root: &Path, dest: &str, fake_sha: &str) {
    let p = root.join(".elf/manifest.json");
    let m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    let mut m = m;
    for f in m["files"].as_array_mut().unwrap() {
        if f["dest"] == dest {
            f["sha256"] = serde_json::Value::String(fake_sha.into());
        }
    }
    fs::write(&p, serde_json::to_string_pretty(&m).unwrap()).unwrap();
}

// ── 행렬: managed ───────────────────────────────────────────

#[test]
fn fresh_project_update_is_idempotent() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let before = fs::read(root.join("0_Meta/EliRule.md")).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.changed, 0, "fresh tree must be all up-to-date: {:?}", report.lines);
    assert_eq!(report.conflicts, 0);
    assert_eq!(fs::read(root.join("0_Meta/EliRule.md")).unwrap(), before);
}

#[test]
fn managed_unedited_with_newer_template_is_overwritten() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 연출: 배포 시점엔 "old content"였고 stamp도 그 해시였다고 가정
    let dest = "0_Meta/EliRule.md";
    fs::write(root.join(dest), b"old content\n").unwrap();
    tamper_stamp_sha(&root, dest, &hash::sha256_lf(b"old content\n"));

    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l == &format!("updated: {dest}")));
    // 새 정본 바이트로 교체됨
    let expected = embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents();
    assert_eq!(fs::read(root.join(dest)).unwrap(), expected);
}

#[test]
fn managed_user_edit_is_preserved_with_elf_new() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = "0_Meta/EliRule.md";
    fs::write(root.join(dest), b"my custom rules\n").unwrap(); // stamp는 정본 해시 그대로

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    // 원본 보존
    assert_eq!(fs::read(root.join(dest)).unwrap(), b"my custom rules\n");
    // 새 버전은 .elf-new로
    let elf_new = fs::read(root.join(format!("{dest}.elf-new"))).unwrap();
    assert_eq!(elf_new, embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents());
}

#[test]
fn managed_user_edit_force_overwrites() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = "0_Meta/EliRule.md";
    fs::write(root.join(dest), b"my custom rules\n").unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert_eq!(report.conflicts, 0);
    assert_eq!(
        fs::read(root.join(dest)).unwrap(),
        embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents()
    );
    assert!(report.lines.iter().any(|l| l.contains("force-updated")));
}

#[test]
fn managed_missing_is_restored() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = "0_Meta/highIFjournals.md";
    fs::remove_file(root.join(dest)).unwrap();

    run_update(&root, &plain()).unwrap();
    assert_eq!(
        fs::read(root.join(dest)).unwrap(),
        embed::TEMPLATES.get_file("meta/highIFjournals.md").unwrap().contents()
    );
}

// ── 행렬: seed / instance ───────────────────────────────────

#[test]
fn seed_and_instance_are_never_touched() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join("0_Meta/ProjectRule.md"), b"customized by user\n").unwrap();
    fs::write(root.join("2_Log/S001_log.md"), b"my session notes\n").unwrap();

    run_update(&root, &plain()).unwrap();
    assert_eq!(fs::read(root.join("0_Meta/ProjectRule.md")).unwrap(), b"customized by user\n");
    assert_eq!(fs::read(root.join("2_Log/S001_log.md")).unwrap(), b"my session notes\n");
    assert!(!root.join("0_Meta/ProjectRule.md.elf-new").exists());
}

// ── 행렬: hybrid (마커블록) ─────────────────────────────────

#[test]
fn hybrid_clean_block_is_replaced_and_user_area_preserved() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 연출: 배포본이 구버전 블록이었다고 가정 — baseline·현재 파일 모두 구블록 + 사용자 규칙
    let old_template = format!("{MARKER_START}\nold-rule/\n{MARKER_END}\n");
    let current = format!("{old_template}\n# user rules\nmy_data/\n");
    fs::write(root.join(".gitignore"), &current).unwrap();
    fs::write(root.join(".elf/baseline/.gitignore"), &old_template).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l.contains("merged block")), "{:?}", report.lines);
    let merged = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(!merged.contains("old-rule/"), "old block must be replaced");
    assert!(merged.contains("*.asv"), "new block content present");
    assert!(merged.contains("my_data/"), "user area preserved");
    // baseline 갱신 → 다음 update에서 편집 오탐 없음
    let report2 = run_update(&root, &plain()).unwrap();
    assert_eq!(report2.conflicts, 0);
}

#[test]
fn hybrid_edited_block_is_kept_with_elf_new() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 사용자가 블록 안을 직접 수정 (baseline은 init이 기록한 정본)
    let current = fs::read_to_string(root.join(".gitignore")).unwrap();
    let hacked = current.replace("*.asv", "*.asv\nuser-inserted-inside-block/");
    fs::write(root.join(".gitignore"), &hacked).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    assert!(report.lines.iter().any(|l| l.contains("block edited")), "{:?}", report.lines);
    // 파일 보존 + .elf-new 산출
    assert_eq!(fs::read_to_string(root.join(".gitignore")).unwrap(), hacked);
    assert!(root.join(".gitignore.elf-new").exists());
}

#[test]
fn hybrid_edited_block_force_replaces() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let current = fs::read_to_string(root.join(".gitignore")).unwrap();
    fs::write(root.join(".gitignore"), current.replace("*.asv", "hacked/")).unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert_eq!(report.conflicts, 0);
    let merged = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(merged.contains("*.asv"));
    assert!(!merged.contains("hacked/"));
}

#[test]
fn hybrid_missing_markers_kept_unless_force() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".gitignore"), b"only-user-rules/\n").unwrap();

    // 기본: 미변경 + 경고
    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    assert_eq!(fs::read(root.join(".gitignore")).unwrap(), b"only-user-rules/\n");

    // --force: 상단 재삽입 + 기존 내용 보존
    let report2 = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert!(report2.lines.iter().any(|l| l.contains("reinserted")));
    let rebuilt = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(rebuilt.starts_with(MARKER_START));
    assert!(rebuilt.contains("only-user-rules/"));
}

// ── dry-run / re-stamp / 프로젝트 인식 ──────────────────────

#[test]
fn dry_run_writes_nothing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = "0_Meta/EliRule.md";
    fs::write(root.join(dest), b"edited\n").unwrap();
    fs::write(root.join(".elf/version"), "v0.0\n").unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert_eq!(report.conflicts, 1); // 보고는 됨
    assert!(!root.join(format!("{dest}.elf-new")).exists(), "dry-run must not write .elf-new");
    assert_eq!(fs::read_to_string(root.join(".elf/version")).unwrap(), "v0.0\n", "dry-run must not re-stamp");
}

#[test]
fn update_always_restamps_manifest_and_version() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".elf/version"), "v0.0\n").unwrap();
    tamper_stamp_sha(&root, "0_Meta/EliRule.md", "deadbeef");

    run_update(&root, &plain()).unwrap();
    assert_eq!(
        fs::read_to_string(root.join(".elf/version")).unwrap(),
        format!("{}\n", embed::version())
    );
    // stamp가 embed 정본으로 복원
    let stamp = fs::read_to_string(root.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, embed::MANIFEST_JSON);
    assert!(manifest::parse(&stamp).is_ok());
}

#[test]
fn non_elf_dir_errors_and_root_is_found_from_subdir() {
    let tmp = tempdir().unwrap();
    assert!(elf_cli::update::find_project_root(tmp.path()).is_none());

    let root = new_project(tmp.path(), "P");
    let sub = root.join("2_Log/Wiki");
    assert_eq!(elf_cli::update::find_project_root(&sub).unwrap(), root);
}

#[test]
fn git_dirty_tree_produces_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let git_ok = std::process::Command::new("git")
        .arg("-C").arg(&root).arg("init").output()
        .map(|o| o.status.success()).unwrap_or(false);
    if !git_ok {
        eprintln!("git unavailable — skipping");
        return;
    }
    // untracked 파일 다수 = dirty
    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l.contains("dirty")), "{:?}", report.lines);
}
