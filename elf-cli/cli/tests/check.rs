//! check: `elf check [setup|files|logs]` — 진단 통합(구 doctor, status, validate는 숨김 별칭).
//! 검증: 범위별 요약과 상세, 적용 불가 범위 처리(`.elf/` 없는 폴더), 종료 코드(문제 4, 검사 불능 5),
//! 별칭의 기존 출력과 종료 코드 유지.

use assert_cmd::Command;
use elf_cli::init::{InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const REG: &str = "2_Log/Wiki/Session_Registry.tsv";

fn new_project(tmp: &Path) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-10-25".into(),
        },
    )
    .unwrap()
}

fn elf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::cargo_bin("elf").unwrap().args(args).current_dir(dir).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// 등록되지 않은 로그 = logs issue.
fn add_unregistered_log(root: &Path) {
    fs::write(
        root.join("2_Log/S099_log.md"),
        "# S099: x\n\n> **Created**: 2026-10-25\\\n> **Modified**: 2026-10-25\\\n> **Status**: ★ 활성\\\n> **목표**: t\\\n> **관련**: -\\\n> **Handoff**: -\n\n---\n",
    )
    .unwrap();
}

#[test]
fn check_all_summarizes_three_scopes_and_exits_0_when_clean() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let (code, out, _) = elf(&root, &["check"]);
    assert_eq!(code, 0, "{out}");
    for s in ["check setup:", "check files:", "check logs:"] {
        assert!(out.contains(s), "missing {s}:\n{out}");
    }
    assert!(out.lines().last().unwrap_or("").contains("check: 0 issue(s)"), "{out}");
    // 요약 모드는 ok/info 줄을 싣지 않음
    assert!(!out.contains("ok: "), "{out}");
}

#[test]
fn check_scope_prints_full_detail() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let (code, out, _) = elf(&root, &["check", "files"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("ok: .elf stamp"), "{out}"); // 진단 묶음
    assert!(out.contains("ok: AGENTS.md"), "{out}"); // 관리 파일 상태
    assert!(out.contains("check files: 0 issue(s)"), "{out}");
    let (code, out, _) = elf(&root, &["check", "setup"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("info: elf version"), "{out}");
    assert!(out.contains("autoread"), "{out}");
}

#[test]
fn folder_without_elf_skips_files_but_checks_logs() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::remove_dir_all(root.join(".elf")).unwrap(); // `_dev`처럼 로그만 있는 폴더
    let (code, out, _) = elf(&root, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("check files: skipped"), "{out}");
    assert!(out.contains("check logs: 0 issue(s)"), "{out}");
    // 범위를 직접 지정하면 오류
    let (code, _, err) = elf(&root, &["check", "files"]);
    assert_eq!(code, 1);
    assert!(err.contains("not an ELF project"), "{err}");
}

#[test]
fn logs_issue_exits_4_and_alias_keeps_old_exit_codes() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    add_unregistered_log(&root);
    let (code, out, _) = elf(&root, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("  issue: "), "{out}");
    assert_eq!(elf(&root, &["check", "logs"]).0, 4);
    // 별칭: 기존 출력 형식과 종료 코드
    let (code, out, _) = elf(&root, &["validate"]);
    assert_eq!(code, 0);
    assert!(out.contains("validate: 1 issue(s)") || out.contains("validate: 2 issue(s)"), "{out}");
    assert_eq!(elf(&root, &["validate", "--check"]).0, 4);
}

#[test]
fn files_conflict_is_issue_and_status_alias_unchanged() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let agents = root.join("AGENTS.md");
    let t = fs::read_to_string(&agents).unwrap();
    fs::write(&agents, format!("{t}\nuser edit\n")).unwrap();
    let (code, out, _) = elf(&root, &["check", "files"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("issue: edited: AGENTS.md"), "{out}");
    let (code, out, _) = elf(&root, &["status"]);
    assert_eq!(code, 0);
    assert!(out.contains("edited: AGENTS.md") && out.contains("status: 0 pending, 1 conflicts"), "{out}");
    assert_eq!(elf(&root, &["status", "--check"]).0, 4);
}

#[test]
fn malformed_registry_exits_5() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n").unwrap();
    assert_eq!(elf(&root, &["check"]).0, 5);
    assert_eq!(elf(&root, &["check", "logs"]).0, 5);
}

#[test]
fn doctor_alias_keeps_old_output() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let (code, out, _) = elf(&root, &["doctor"]);
    assert_eq!(code, 0);
    assert!(out.contains("[OK  ] ELF project") && out.contains("doctor: "), "{out}");
}

#[test]
fn outside_any_project_setup_still_runs() {
    let tmp = tempdir().unwrap();
    let (code, out, _) = elf(tmp.path(), &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("check setup: 0 issue(s)"), "{out}");
    assert!(out.contains("check files: skipped") && out.contains("check logs: skipped"), "{out}");
}

#[test]
fn removed_and_hidden_forms() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // `--strict` 삭제
    assert_eq!(elf(&root, &["validate", "--strict"]).0, 2);
    assert_eq!(elf(&root, &["check", "--strict"]).0, 2);
    // 알 수 없는 범위
    assert_eq!(elf(&root, &["check", "everything"]).0, 2);
    // help: 새 명령은 보이고 구 이름은 숨김
    let (_, help, _) = elf(&root, &["--help"]);
    for shown in ["check", "self", "deprecate", "tsa", "autoread"] {
        assert!(help.contains(&format!("  {shown} ")), "{shown} not in help:\n{help}");
    }
    for hidden in ["doctor", "status", "validate", "self-update"] {
        assert!(!help.contains(&format!("  {hidden} ")), "{hidden} visible in help:\n{help}");
    }
}
