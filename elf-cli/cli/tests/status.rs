//! t05 통합: `elf status`의 상태 분류(읽기전용)와 `--check` 게이트 exit code.

use assert_cmd::Command;
use elf_cli::init::{run_init, InitOptions};
use elf_cli::status::run_status;
use elf_cli::{hash, update};
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
            lang: "ko-KR".into(),
            date: "2026-06-12".into(),
        },
    )
    .unwrap()
}

fn tamper_stamp_sha(root: &Path, dest: &str, fake_sha: &str) {
    let p = root.join(".elf/manifest.json");
    let mut m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    for f in m["files"].as_array_mut().unwrap() {
        if f["dest"] == dest {
            f["sha256"] = serde_json::Value::String(fake_sha.into());
        }
    }
    fs::write(&p, serde_json::to_string_pretty(&m).unwrap()).unwrap();
}

#[test]
fn clean_project_has_no_findings_and_is_readonly() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let before = fs::read(root.join(".elf/manifest.json")).unwrap();

    let r = run_status(&root).unwrap();
    assert_eq!(r.findings(), 0, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.contains("project = CLI")));
    // 읽기전용 보장: stamp 무변경
    assert_eq!(fs::read(root.join(".elf/manifest.json")).unwrap(), before);
}

#[test]
fn outdated_managed_is_pending() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = ".elf/managed/EliRule.md";
    fs::write(root.join(dest), b"old deployed content\n").unwrap();
    tamper_stamp_sha(&root, dest, &hash::sha256_lf(b"old deployed content\n"));

    let r = run_status(&root).unwrap();
    assert_eq!(r.pending, 1);
    assert_eq!(r.conflicts, 0);
    assert!(r.lines.iter().any(|l| l.starts_with("outdated: .elf/managed/EliRule.md")));
}

#[test]
fn edited_managed_is_conflict() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".elf/managed/EliRule.md"), b"user custom\n").unwrap();

    let r = run_status(&root).unwrap();
    assert_eq!(r.conflicts, 1);
    assert!(r.lines.iter().any(|l| l.starts_with("edited: .elf/managed/EliRule.md")));
}

#[test]
fn missing_managed_is_pending() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::remove_file(root.join(".elf/managed/highIFjournals.md")).unwrap();

    let r = run_status(&root).unwrap();
    assert!(r.pending >= 1);
    assert!(r.lines.iter().any(|l| l.starts_with("missing: .elf/managed/highIFjournals.md")));
}

#[test]
fn hybrid_user_area_change_is_ok_block_edit_is_conflict() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");

    // 사용자 영역 추가만 → ok (findings 0)
    let current = fs::read_to_string(root.join(".gitignore")).unwrap();
    fs::write(root.join(".gitignore"), format!("{current}\nmy_data/\n")).unwrap();
    let r = run_status(&root).unwrap();
    assert_eq!(r.findings(), 0, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.contains("ok (block current")));

    // 블록 내부 수정 → conflict
    let cur2 = fs::read_to_string(root.join(".gitignore")).unwrap();
    fs::write(root.join(".gitignore"), cur2.replace("*.asv", "hacked/")).unwrap();
    let r2 = run_status(&root).unwrap();
    assert_eq!(r2.conflicts, 1);
    assert!(r2.lines.iter().any(|l| l.starts_with("block edited: .gitignore")));
}

#[test]
fn hybrid_block_outdated_via_baseline_is_pending() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let (ms, me) = (update::MARKER_START, update::MARKER_END);
    let old_template = format!("{ms}\nold-rule/\n{me}\n");
    fs::write(root.join(".gitignore"), format!("{old_template}\nuser/\n")).unwrap();
    fs::write(root.join(".elf/baseline/.gitignore"), &old_template).unwrap();

    let r = run_status(&root).unwrap();
    assert_eq!(r.pending, 1, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("block outdated: .gitignore")));
}

#[test]
fn version_mismatch_is_warned() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".elf/version"), "v0.0\n").unwrap();

    let r = run_status(&root).unwrap();
    assert!(r.warnings >= 1);
    assert!(r.lines.iter().any(|l| l.contains("v0.0") && l.contains("≠ CLI")));
}

// ── e2e: exit code 게이트 ───────────────────────────────────

#[test]
fn check_flag_gates_with_exit_4() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");

    // clean → exit 0
    Command::cargo_bin("elf").unwrap()
        .current_dir(&root)
        .args(["status", "--check"])
        .assert()
        .success();

    // 편집 발생 → exit 4
    fs::write(root.join(".elf/managed/EliRule.md"), b"edited\n").unwrap();
    Command::cargo_bin("elf").unwrap()
        .current_dir(&root)
        .args(["status", "--check"])
        .assert()
        .failure()
        .code(4);

    // --check 없으면 보고만, exit 0
    Command::cargo_bin("elf").unwrap()
        .current_dir(&root)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("edited: .elf/managed/EliRule.md"));
}

#[test]
fn non_elf_dir_errors_with_exit_1() {
    let tmp = tempdir().unwrap();
    Command::cargo_bin("elf").unwrap()
        .current_dir(tmp.path())
        .args(["status"])
        .assert()
        .failure()
        .code(1);
}
