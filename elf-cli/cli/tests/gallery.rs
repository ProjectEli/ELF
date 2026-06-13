//! t03: `elf gallery` — 64_Viz 스캔 → `_gallery.md` 생성, 부재 시 정상 종료.

use assert_cmd::Command;
use elf_cli::gallery::run_gallery;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const NOW: &str = "2026-06-13 09:30";

/// 64_Viz/<session>/<file> 픽스처 생성.
fn put(root: &Path, session: &str, file: &str) {
    let dir = root.join("6_Exp/64_Viz").join(session);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(file), b"x").unwrap();
}

#[test]
fn generates_gallery_grouped_by_session() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    put(root, "S001", "sweep.png");
    put(root, "S001", "fit.svg");
    put(root, "S002", "map.jpg");
    put(root, "S001", "notes.txt"); // 비이미지 — 무시

    let r = run_gallery(root, NOW).unwrap();
    assert!(r.viz_present);
    assert_eq!(r.sessions, 2);
    assert_eq!(r.images, 3); // png+svg+jpg (txt 제외)

    let md = fs::read_to_string(root.join("6_Exp/64_Viz/_gallery.md")).unwrap();
    assert!(md.contains("## S001"));
    assert!(md.contains("![sweep](S001/sweep.png)"));
    assert!(md.contains("![fit](S001/fit.svg)"));
    assert!(md.contains("## S002"));
    assert!(md.contains("![map](S002/map.jpg)"));
    assert!(!md.contains("notes")); // 비이미지 미포함
    // 세션 순서: S001 before S002
    assert!(md.find("## S001").unwrap() < md.find("## S002").unwrap());
}

#[test]
fn session_without_images_is_skipped() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    put(root, "S001", "fig.png");
    fs::create_dir_all(root.join("6_Exp/64_Viz/S002")).unwrap(); // 이미지 없는 세션
    fs::write(root.join("6_Exp/64_Viz/S002/readme.md"), b"x").unwrap();

    let r = run_gallery(root, NOW).unwrap();
    assert_eq!(r.sessions, 1);
    let md = fs::read_to_string(root.join("6_Exp/64_Viz/_gallery.md")).unwrap();
    assert!(md.contains("## S001"));
    assert!(!md.contains("## S002")); // 이미지 없으면 헤더 없음
}

#[test]
fn missing_viz_dir_is_graceful() {
    let tmp = tempdir().unwrap();
    let r = run_gallery(tmp.path(), NOW).unwrap();
    assert!(!r.viz_present);
    assert_eq!(r.sessions, 0);
    assert_eq!(r.images, 0);
    assert!(!tmp.path().join("6_Exp/64_Viz/_gallery.md").exists()); // 무작성
}

#[test]
fn empty_viz_writes_header_only() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("6_Exp/64_Viz")).unwrap(); // 빈 64_Viz
    let r = run_gallery(root, NOW).unwrap();
    assert!(r.viz_present);
    assert_eq!(r.images, 0);
    let md = fs::read_to_string(root.join("6_Exp/64_Viz/_gallery.md")).unwrap();
    assert!(md.contains("# Visualization Gallery"));
    assert!(!md.contains("## "));
}

// ── e2e ──────────────────────────────────────────────

#[test]
fn e2e_gallery_writes_and_reports() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    put(root, "S001", "a.png");
    // gallery는 find_log_root(2_Log/) 기반 루트 탐지 → 2_Log/ 필요
    fs::create_dir_all(root.join("2_Log")).unwrap();

    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(root)
        .args(["gallery"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1 session(s)"))
        .stdout(predicates::str::contains("1 image(s)"));
    assert!(root.join("6_Exp/64_Viz/_gallery.md").exists());
}

#[test]
fn e2e_gallery_missing_viz_exits_0() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("2_Log")).unwrap(); // 루트 탐지용, 64_Viz는 없음
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(root)
        .args(["gallery"])
        .assert()
        .success() // 부재 시에도 exit 0 (안내)
        .stdout(predicates::str::contains("not found"));
}
