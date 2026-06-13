//! t04: `elf validate` — Registry↔로그 정합·번호·cross-ref 검사, escalation·`--check` 게이트.

use assert_cmd::Command;
use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{SessionNewOptions, run_session_new};
use elf_cli::validate::{ValidateError, run_validate};
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
            lang: "한국어".into(),
            date: "2026-06-13".into(),
        },
    )
    .unwrap()
}

const REG: &str = "2_Log/Wiki/Session_Registry.tsv";

#[test]
fn clean_project_has_no_findings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // init = S001 로그 + 등록
    let r = run_validate(&root).unwrap();
    assert_eq!(r.issues, 0, "{:?}", r.lines);
    assert_eq!(r.warnings, 0, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("ok")), "{:?}", r.lines);
}

#[test]
fn unregistered_log_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005_log.md"), "# S005\n").unwrap(); // 등록 없는 로그
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S005") && l.contains("not in the registry")),
        "{:?}",
        r.lines
    );
    assert!(r.issues >= 1);
}

#[test]
fn phantom_registry_row_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let mut reg = fs::read_to_string(root.join(REG)).unwrap();
    reg.push_str("S002\t2026-06-13\tGhost\tComplete\t-\tArchive/S002_log.md\n"); // 로그 없는 행
    fs::write(root.join(REG), reg).unwrap();
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S002") && l.contains("no log file")),
        "{:?}",
        r.lines
    );
}

#[test]
fn gap_is_warning_not_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S003_log.md"), "# S003\n").unwrap();
    let mut reg = fs::read_to_string(root.join(REG)).unwrap();
    reg.push_str("S003\t2026-06-13\tThree\tComplete\t-\tArchive/S003_log.md\n");
    fs::write(root.join(REG), reg).unwrap();
    let r = run_validate(&root).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("gap") && l.contains("S002")), "{:?}", r.lines);
    assert_eq!(r.issues, 0, "gap는 warning이어야 함: {:?}", r.lines); // S003 등록+로그 → issue 아님
}

#[test]
fn multiple_active_is_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    run_session_new(&root, &SessionNewOptions { title: "Second".into(), date: "2026-06-13".into() })
        .unwrap(); // S002도 활성
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("multiple active") && l.contains("S001") && l.contains("S002")),
        "{:?}",
        r.lines
    );
    assert_eq!(r.issues, 0, "{:?}", r.lines); // 둘 다 등록+연속 → 정합 issue 없음
}

#[test]
fn broken_cross_ref_is_issue_valid_is_not() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("1_Concept")).unwrap();
    fs::write(root.join("1_Concept/Real.md"), "x").unwrap();
    let p = root.join("2_Log/S001_log.md");
    let mut c = fs::read_to_string(&p).unwrap();
    c.push_str("\nsee [plan](../1_Concept/Nope.md) and [ok](../1_Concept/Real.md)\n");
    fs::write(&p, c).unwrap();

    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("broken cross-ref") && l.contains("Nope.md")),
        "{:?}",
        r.lines
    );
    assert!(
        !r.lines.iter().any(|l| l.contains("Real.md")),
        "유효 링크는 미발견이어야 함: {:?}",
        r.lines
    );
}

#[test]
fn malformed_registry_escalates() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join(REG),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n",
    )
    .unwrap();
    match run_validate(&root) {
        Err(ValidateError::Escalation(e)) => {
            assert_eq!(e.line, 2);
            assert!(e.to_string().contains("agent-action:"));
        }
        _ => panic!("expected escalation"),
    }
}

// ── e2e ──────────────────────────────────────────────

#[test]
fn e2e_validate_clean_exits_0() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0 issue"));
}

#[test]
fn e2e_validate_check_gates_with_exit_4() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005_log.md"), "# S005\n").unwrap(); // 미등록 = issue
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate", "--check"])
        .assert()
        .failure()
        .code(4);
}

#[test]
fn e2e_validate_malformed_exits_5_with_marker() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join(REG),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n",
    )
    .unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicates::str::contains("agent-action:"));
}
