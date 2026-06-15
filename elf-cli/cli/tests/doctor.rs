//! t05: `elf doctor` — 환경+프로젝트 종합 진단(읽기전용), 프로젝트 안/밖 동작.

use assert_cmd::Command;
use elf_cli::doctor::{DoctorEnv, Health, run_doctor};
use elf_cli::init::{InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn env(receipt: bool) -> DoctorEnv {
    DoctorEnv { version: elf_cli::embed::version().to_string(), receipt_present: receipt }
}

fn new_project(tmp: &Path) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "한국어".into(),
            date: "2026-06-13".into(),
        },
    )
    .unwrap()
}

fn check<'a>(r: &'a elf_cli::doctor::DoctorReport, label: &str) -> &'a elf_cli::doctor::Check {
    r.checks.iter().find(|c| c.label == label).unwrap_or_else(|| panic!("no check '{label}'"))
}

#[test]
fn outside_project_reports_env_only_no_warnings() {
    let tmp = tempdir().unwrap();
    let r = run_doctor(tmp.path(), &env(false));
    assert!(check(&r, "ELF project").detail.contains("not inside"));
    assert!(!r.checks.iter().any(|c| c.label == ".elf stamp")); // 프로젝트 검사 스킵
    assert!(r.checks.iter().any(|c| c.label == "elf version")); // 환경 검사 수행
    assert_eq!(r.warnings(), 0, "{:?}", r.checks);
}

#[test]
fn clean_project_all_ok_no_warnings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let r = run_doctor(&root, &env(true));
    assert_eq!(check(&r, ".elf stamp").health, Health::Ok);
    assert_eq!(check(&r, ".elf version").health, Health::Ok);
    assert_eq!(check(&r, "managed files").health, Health::Ok);
    assert_eq!(check(&r, "install receipt").health, Health::Ok);
    assert_eq!(r.warnings(), 0, "{:?}", r.checks);
}

#[test]
fn version_mismatch_is_warn() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(".elf/version"), "0.0.0-fake\n").unwrap();
    let r = run_doctor(&root, &env(true));
    assert_eq!(check(&r, ".elf version").health, Health::Warn);
    assert!(r.warnings() >= 1);
}

#[test]
fn malformed_stamp_is_warn() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(".elf/manifest.json"), "{ not json").unwrap();
    let r = run_doctor(&root, &env(true));
    assert_eq!(check(&r, ".elf stamp").health, Health::Warn);
}

#[test]
fn absent_receipt_is_info_not_warn() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let r = run_doctor(&root, &env(false));
    assert_eq!(check(&r, "install receipt").health, Health::Info); // 부재는 경고 아님
}

// ── e2e ──────────────────────────────────────────────

#[test]
fn e2e_doctor_outside_project_exits_0() {
    let tmp = tempdir().unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(tmp.path())
        .args(["doctor"])
        .assert()
        .success() // 프로젝트 밖에서도 정상 동작(환경 진단만)
        .stdout(predicates::str::contains("elf version"));
}

#[test]
fn e2e_doctor_in_project_reports_elf() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicates::str::contains(".elf stamp"));
}
