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

/// help 출력에 내부 dev 표기(trial/세션/플랜 번호·_dev 등) 누출 금지 — v2.4.1 누출 회귀 게이트 (P011 t09)
#[test]
fn help_outputs_contain_no_internal_dev_markers() {
    let marker = regex_lite_match;
    for args in [
        vec!["--help"],
        vec!["init", "--help"],
        vec!["update", "--help"],
        vec!["status", "--help"],
        vec!["self-update", "--help"],
    ] {
        let out = Command::cargo_bin("elf").unwrap().args(&args).output().unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !marker(&text),
            "internal dev marker leaked in help of {args:?}:\n{text}"
        );
    }
}

/// 내부 표기 패턴: t## / S### / P### / _dev / "gated" (정규식 crate 없이 수동 검사)
fn regex_lite_match(text: &str) -> bool {
    if text.contains("_dev") || text.contains("gated") {
        return true;
    }
    let bytes = text.as_bytes();
    for (i, w) in bytes.windows(3).enumerate() {
        let prev_alnum = i > 0 && bytes[i - 1].is_ascii_alphanumeric();
        // t## (예: t06)
        if w[0] == b't' && w[1].is_ascii_digit() && w[2].is_ascii_digit() && !prev_alnum {
            return true;
        }
        // S###/P### (예: S007, P011)
        if (w[0] == b'S' || w[0] == b'P')
            && w[1].is_ascii_digit()
            && w[2].is_ascii_digit()
            && !prev_alnum
            && bytes.get(i + 3).is_some_and(|b| b.is_ascii_digit())
        {
            return true;
        }
    }
    false
}
