//! S024/B: `elf migrate` — legacy(0_Meta/·templates/) → managed(.elf/managed/) 이전 +
//! 레이아웃 이중 지원(legacy 프로젝트의 update가 legacy 경로에 계속 배포) 회귀 가드.

use elf_cli::init::{run_init, InitOptions};
use elf_cli::migrate::{run_migrate, MigrateError, MigrateOptions};
use elf_cli::status::run_status;
use elf_cli::update::{run_update, UpdateOptions};
use elf_cli::{embed, manifest};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn new_project(tmp: &Path) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-07-07".into(),
        },
    )
    .unwrap()
}

/// 신규(managed) 프로젝트를 구(legacy) 프로젝트로 되돌려 pre-relocation 상태를 재현:
/// 배포 파일을 legacy 경로로 이동 + 구식 stamp(legacy dest 표기) + config에서 layout 제거.
fn make_legacy(root: &Path) {
    let m = manifest::embedded().for_lang("ko-KR");
    for e in &m.files {
        let canon = manifest::dest_to_managed(&e.dest, e.tier);
        let legacy = manifest::dest_to_legacy(&canon);
        if canon != legacy && root.join(&canon).is_file() {
            fs::create_dir_all(root.join(&legacy).parent().unwrap()).unwrap();
            fs::rename(root.join(&canon), root.join(&legacy)).unwrap();
        }
    }
    let old_stamp = embed::MANIFEST_JSON
        .replace(".elf/managed/templates/", "templates/")
        .replace(".elf/managed/", "0_Meta/");
    fs::write(root.join(".elf/manifest.json"), old_stamp).unwrap();
    let cfg_path = root.join(".elf/config.json");
    let mut v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cfg_path).unwrap()).unwrap();
    v.as_object_mut().unwrap().remove("layout");
    fs::write(&cfg_path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

#[test]
fn legacy_update_keeps_deploying_to_legacy_paths() {
    // 레이아웃 이중 지원의 핵심 가드: 미이전 프로젝트의 update는 legacy 경로를 유지하고
    // .elf/managed/를 만들지 않으며, obsolete 오탐도 없어야 한다.
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    make_legacy(&root);

    fs::remove_file(root.join("0_Meta/EliRule.md")).unwrap();
    let report = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert!(root.join("0_Meta/EliRule.md").is_file(), "legacy 경로에 재생성돼야 함");
    assert!(!root.join(".elf/managed/EliRule.md").exists(), "migrate 전 .elf/managed 생성 금지");
    assert_eq!(report.conflicts, 0, "{:?}", report.lines);
    assert_eq!(report.warnings, 0, "obsolete 오탐 금지: {:?}", report.lines);
    assert!(
        report.lines.iter().any(|l| l.starts_with("layout: legacy")),
        "legacy 공지 note 출력: {:?}",
        report.lines
    );

    // 최신 버전·무변경 재실행에도 공지 유지 (재발견 채널 — S024 t05 핵심 케이스)
    let again = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(again.changed, 0, "{:?}", again.lines);
    assert!(
        again.lines.iter().any(|l| l.starts_with("layout: legacy")),
        "무변경 update에도 공지: {:?}",
        again.lines
    );
    // dry-run에도 공지 (상태 표시는 미리보기에서도 참)
    let dry = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert!(dry.lines.iter().any(|l| l.starts_with("layout: legacy")), "{:?}", dry.lines);
    // warning 아님 — 게이트 무영향
    assert_eq!(again.warnings, 0, "공지는 note — warning 불산입: {:?}", again.lines);

    let s = run_status(&root).unwrap();
    assert_eq!(s.findings(), 0, "{:?}", s.lines);
    assert!(
        s.lines.iter().any(|l| l.starts_with("layout: legacy")),
        "status에도 공지: {:?}",
        s.lines
    );
    assert_eq!(s.warnings, 0, "status 공지도 findings·warnings 불산입: {:?}", s.lines);
}

#[test]
fn managed_project_has_no_layout_note() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // 신규 init = managed
    let u = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert!(!u.lines.iter().any(|l| l.starts_with("layout:")), "{:?}", u.lines);
    let s = run_status(&root).unwrap();
    assert!(!s.lines.iter().any(|l| l.starts_with("layout:")), "{:?}", s.lines);
}

#[test]
fn migrate_moves_payload_and_flips_layout() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    make_legacy(&root);
    // 미병합 .elf-new 병기본도 함께 이동돼야 함
    fs::write(root.join("0_Meta/EliRule.md.elf-new"), b"pending merge\n").unwrap();

    let report = run_migrate(&root, &MigrateOptions { dry_run: false }).unwrap();
    assert!(report.moved >= 3, "rules+templates 이동: {:?}", report.lines);
    assert!(root.join(".elf/managed/EliRule.md").is_file());
    assert!(root.join(".elf/managed/templates/trialTemplate.md").is_file());
    assert!(root.join(".elf/managed/EliRule.md.elf-new").is_file(), ".elf-new 동반 이동");
    assert!(!root.join("0_Meta/EliRule.md").exists());
    assert!(root.join("0_Meta/ProjectRule.md").is_file(), "seed는 0_Meta 잔류");

    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(cfg["layout"], "managed");
    assert_eq!(cfg["lang"], "ko-KR", "기존 config 필드 보존");

    // 이전 후 status·update가 신경로 좌표에서 정상 (구식 stamp도 정규화되어 비교됨)
    let s = run_status(&root).unwrap();
    assert_eq!(s.findings(), 0, "{:?}", s.lines);
    let u = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(u.conflicts, 0, "{:?}", u.lines);
    // managed 전환 후 공지 소멸 (유계 수명)
    assert!(!u.lines.iter().any(|l| l.starts_with("layout:")), "{:?}", u.lines);
    assert!(!s.lines.iter().any(|l| l.starts_with("layout:")), "{:?}", s.lines);
}

#[test]
fn migrate_dry_run_moves_nothing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    make_legacy(&root);

    let report = run_migrate(&root, &MigrateOptions { dry_run: true }).unwrap();
    assert_eq!(report.moved, 0);
    assert!(report.lines.iter().any(|l| l.starts_with("would move:")), "{:?}", report.lines);
    assert!(root.join("0_Meta/EliRule.md").is_file(), "dry-run은 무이동");
    assert!(!root.join(".elf/managed/EliRule.md").exists());
    // config도 불변 (layout 미기록)
    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert!(cfg.get("layout").is_none());
}

#[test]
fn migrate_on_managed_project_is_already_managed() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // 신규 init = managed
    match run_migrate(&root, &MigrateOptions { dry_run: false }) {
        Err(MigrateError::AlreadyManaged) => {}
        other => panic!("expected AlreadyManaged, got {other:?}"),
    }
}

#[test]
fn migrate_reports_old_path_references_without_rewriting() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    make_legacy(&root);
    let log = root.join("2_Log/S001_log.md");
    fs::write(&log, "참조: 0_Meta/EliRule.md 그리고 templates/trialTemplate.md\n").unwrap();

    let report = run_migrate(&root, &MigrateOptions { dry_run: false }).unwrap();
    assert!(
        report.refs.iter().any(|r| r.contains("S001_log.md") && r.contains("0_Meta/EliRule.md")),
        "{:?}",
        report.refs
    );
    // 사용자 파일은 재작성하지 않음
    let text = fs::read_to_string(&log).unwrap();
    assert!(text.contains("0_Meta/EliRule.md"), "사용자 파일 무변경");
}

#[test]
fn migrate_aborts_when_both_locations_exist() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    make_legacy(&root);
    // 충돌 연출: 신경로에 파일 선점
    fs::create_dir_all(root.join(".elf/managed")).unwrap();
    fs::write(root.join(".elf/managed/EliRule.md"), b"stray\n").unwrap();
    match run_migrate(&root, &MigrateOptions { dry_run: false }) {
        Err(MigrateError::TargetExists(d)) => assert!(d.contains("EliRule.md")),
        other => panic!("expected TargetExists, got {other:?}"),
    }
    // 무이동 보장 (계획 단계 중단)
    assert!(root.join("0_Meta/LogConvention.md").is_file());
}
