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
            lang: "ko-KR".into(),
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
fn en_project_reports_i18n_companions_info() {
    let tmp = tempdir().unwrap();
    let root = run_init(
        tmp.path(),
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "en-US".into(),
            date: "2026-06-13".into(),
        },
    )
    .unwrap();
    let r = run_doctor(&root, &env(true));
    let c = check(&r, "i18n");
    assert_eq!(c.health, Health::Info, "{:?}", r.checks);
    assert!(c.detail.contains("companion") && c.detail.contains("operative source = *.md"));
    assert_eq!(r.warnings(), 0, "{:?}", r.checks); // i18n은 Info — 경고 증가 없음
}

#[test]
fn ko_project_has_no_i18n_check() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // lang ko-KR
    let r = run_doctor(&root, &env(true));
    assert!(!r.checks.iter().any(|c| c.label == "i18n"), "ko 프로젝트엔 i18n 검사 비적용");
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

// ── overlay (EliRule §2.7) ───────────────────────────

#[test]
fn overlay_with_reasoned_removals_is_ok() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join("0_Meta/LLMcliche.project.md"),
        "# LLMcliche — project overlay\n## 추가 (add)\n- foo\n## 제외 (remove)\n- novel — 사유: 소설 연구 도메인 용어\n",
    )
    .unwrap();
    let r = run_doctor(&root, &env(true));
    let c = check(&r, "overlay");
    assert_eq!(c.health, Health::Ok, "{:?}", r.checks);
    assert!(c.detail.contains("LLMcliche.project.md") && c.detail.contains("active"));
    assert_eq!(r.warnings(), 0, "{:?}", r.checks);
}

#[test]
fn overlay_removal_without_reason_warns() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join("0_Meta/highIFjournals.project.md"),
        "## 제외 (remove)\n- sciencedirect.com\n",
    )
    .unwrap();
    let r = run_doctor(&root, &env(true));
    let c = check(&r, "overlay");
    assert_eq!(c.health, Health::Warn, "{:?}", r.checks);
    assert!(c.detail.contains("without a reason"));
}

#[test]
fn overlay_for_non_overlayable_base_warns() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // LogConvention = 구조·규약 파일 — overlay 비허용 (positive list 밖)
    fs::write(root.join("0_Meta/LogConvention.project.md"), "## 추가 (add)\n- x\n").unwrap();
    let r = run_doctor(&root, &env(true));
    let c = check(&r, "overlay");
    assert_eq!(c.health, Health::Warn, "{:?}", r.checks);
    assert!(c.detail.contains("no overlayable base"));
}

#[test]
fn absent_overlay_reports_nothing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let r = run_doctor(&root, &env(true));
    assert!(
        !r.checks.iter().any(|c| c.label == "overlay"),
        "overlay는 선택 — 부재 시 무보고: {:?}",
        r.checks
    );
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
