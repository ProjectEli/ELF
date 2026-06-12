//! CLI e2e smoke: 빌드 바이너리의 인터페이스 계약 (exit code 규약 포함).

use assert_cmd::Command;

/// tests/ 기준 ../../VERSION = elf-cli/VERSION (SSOT)
const VERSION_SSOT: &str = include_str!("../../VERSION");

#[test]
fn version_flag_prints_ssot_version() {
    Command::cargo_bin("elf")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(VERSION_SSOT.trim()));
}

/// bare 실행 = help + exit 2 (clap arg_required_else_help — usage 규약)
#[test]
fn bare_run_shows_help_with_usage_exit() {
    Command::cargo_bin("elf")
        .unwrap()
        .assert()
        .failure()
        .code(2);
}

/// init e2e: 생성 성공(0) → 동일 이름 재실행 refuse(3)
#[test]
fn init_e2e_success_then_refuse() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(tmp.path())
        .args(["init", "Proj", "--preset", "minimal"])
        .assert()
        .success()
        .stdout(predicates::str::contains("created"));

    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(tmp.path())
        .args(["init", "Proj", "--preset", "minimal"])
        .assert()
        .failure()
        .code(3);
}

/// usage 오류(미지 플래그) = exit 2
#[test]
fn unknown_flag_is_usage_error() {
    Command::cargo_bin("elf")
        .unwrap()
        .args(["init", "X", "--wild"])
        .assert()
        .failure()
        .code(2);
}
