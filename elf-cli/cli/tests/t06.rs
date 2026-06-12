//! t06 stage 3: `elf self-update` 인터페이스 계약.
//! 파일명에 update를 안 쓰는 이유 = Windows UAC installer detection (tests/t04.rs 참조).
//! 네트워크 비의존: receipt 부재 환경(LOCALAPPDATA/HOME을 빈 tempdir로 오버라이드)에서
//! graceful 에러 + 인스톨러 안내만 검증 — 실제 갱신 동작은 릴리즈 설치본에서 수동 검증.

use assert_cmd::Command;
use tempfile::tempdir;

fn cmd_without_receipt(tmp: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("elf").unwrap();
    // axoupdater의 receipt 탐색 경로를 빈 tempdir로 — Windows: LOCALAPPDATA, unix: HOME/XDG
    c.env("LOCALAPPDATA", tmp)
        .env("HOME", tmp)
        .env("XDG_CONFIG_HOME", tmp);
    c
}

#[test]
fn self_update_without_receipt_errors_with_installer_hint() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .arg("self-update")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("install receipt"))
        .stderr(predicates::str::contains("elf-cli-installer.ps1"));
}

#[test]
fn selfupdate_alias_routes_to_same_command() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .arg("selfupdate")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("install receipt"));
}

#[test]
fn update_self_flag_routes_to_self_update() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .args(["update", "--self"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("install receipt"));
}

/// `--self`는 프로젝트 갱신 플래그와 배타 (usage 오류 = exit 2)
#[test]
fn update_self_conflicts_with_force_and_dry_run() {
    Command::cargo_bin("elf").unwrap()
        .args(["update", "--self", "--force"])
        .assert()
        .failure()
        .code(2);
    Command::cargo_bin("elf").unwrap()
        .args(["update", "--self", "--dry-run"])
        .assert()
        .failure()
        .code(2);
}
