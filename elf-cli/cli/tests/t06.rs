//! t06 stage 3: `elf self update` 인터페이스 계약(구 이름 `self-update`, `selfupdate`, `update --self`는 숨김 별칭).
//! 파일명에 update를 안 쓰는 이유 = Windows UAC installer detection (tests/t04.rs 참조).
//! 네트워크 비의존: 보관소(`ELF_HOME`)와 발행 원본(`ELF_RELEASES`)을 빈 tempdir로 — 발행 목록을 읽을 수 없을 때
//! graceful 에러 + 인스톨러 안내만 검증(v3.9 S059: self update는 receipt 대신 보관소로 갱신). 갱신 동작 자체는
//! tests/dispatch.rs의 로컬 발행 원본 시험.

use assert_cmd::Command;
use tempfile::tempdir;

fn cmd_without_receipt(tmp: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("elf").unwrap();
    c.env("ELF_DISPATCH", "off");
    // receipt 탐색 경로, 보관소, 발행 원본을 빈 tempdir로 — 실제 ~/.elf와 네트워크를 쓰지 않음
    let rel = tmp.join("rel");
    std::fs::create_dir_all(&rel).unwrap();
    c.env("LOCALAPPDATA", tmp)
        .env("HOME", tmp)
        .env("XDG_CONFIG_HOME", tmp)
        .env("ELF_HOME", tmp.join("home"))
        .env("ELF_RELEASES", rel);
    c
}

#[test]
fn self_update_without_receipt_errors_with_installer_hint() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .args(["self", "update"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot read the release list"))
        .stderr(predicates::str::contains("elf-cli-installer.ps1"));
}

#[test]
fn legacy_self_update_name_routes_to_same_command() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .arg("self-update")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot read the release list"))
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
        .stderr(predicates::str::contains("cannot read the release list"));
}

#[test]
fn update_self_flag_routes_to_self_update() {
    let tmp = tempdir().unwrap();
    cmd_without_receipt(tmp.path())
        .args(["update", "--self"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot read the release list"));
}

/// `--self`는 프로젝트 갱신 플래그와 배타 (usage 오류 = exit 2)
#[test]
fn update_self_conflicts_with_force_and_dry_run() {
    Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off")
        .args(["update", "--self", "--force"])
        .assert()
        .failure()
        .code(2);
    Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off")
        .args(["update", "--self", "--dry-run"])
        .assert()
        .failure()
        .code(2);
}

/// `elf self`만 실행 = 하위 명령 필요(usage) — 실수로 실행 파일 갱신이 일어나지 않음
#[test]
fn bare_self_requires_subcommand() {
    Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").arg("self").assert().failure().code(2);
}
