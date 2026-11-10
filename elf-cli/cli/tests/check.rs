//! check: `elf check [setup|files|logs]` — 진단 통합(구 doctor, status, validate는 숨김 별칭).
//! 검증: 범위별 요약과 상세, 적용 불가 범위 처리(`.elf/` 없는 폴더), 종료 코드(문제 4),
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
    let out = Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(args).current_dir(dir).output().unwrap();
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
    for s in ["check version:", "check setup:", "check files:", "check logs:"] {
        assert!(out.contains(s), "missing {s}:\n{out}");
    }
    assert!(out.find("check version:") < out.find("check setup:"), "version first: {out}");
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
    assert!(out.contains("ok: .elf baseline") || out.contains("info: .elf baseline"), "{out}"); // 진단 묶음
    assert!(!out.contains("stamp") && !out.contains(".elf version"), "version items moved to `check version`: {out}");
    assert!(out.contains("ok: AGENTS.md"), "{out}"); // 관리 파일 상태
    assert!(out.contains("check files: 0 issue(s)"), "{out}");
    let (code, out, _) = elf(&root, &["check", "setup"]);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("elf version") && !out.contains("install receipt"), "{out}");
    assert!(out.contains("autoread"), "{out}");
    let (code, out, _) = elf(&root, &["check", "version"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("info: CLI: elf ") && out.contains("install receipt"), "{out}");
    assert!(out.contains("schema 2, elf_version") && out.contains("ok: verdict: project"), "{out}");
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
fn files_warns_when_agents_import_is_inside_backticks() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let (_, out, _) = elf(&root, &["check", "files"]);
    assert!(out.contains("CLAUDE.md → @AGENTS.md pointer connected"), "{out}");
    // 백틱 안 import = Claude Code가 import하지 않음 → warn (S043, 구현은 부분 문자열만 봐서 connected로 오판)
    fs::write(root.join("CLAUDE.md"), "> 규칙 정본은 `@AGENTS.md`\n").unwrap();
    let (_, out, _) = elf(&root, &["check", "files"]);
    assert!(out.contains("warn: agent entry: CLAUDE.md mentions @AGENTS.md only where Claude Code does not import it"), "{out}");
    assert!(!out.contains("pointer connected"), "{out}");
}

#[test]
fn leftover_registry_tsv_is_earlier_layout_issue() {
    // P022: TSV는 폐지 — 남아 있으면(형식이 깨졌어도) 이전 형식 1건 issue, 이관은 `elf update`
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n").unwrap();
    let (code, out, _) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("earlier layout") && out.contains("elf update"), "{out}");
    assert_eq!(elf(&root, &["check"]).0, 4);
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

/// `check version`의 고정값 줄(v3.0 S044 t56): manifest 고정값과 줄, 2.28.x = 3.0 줄, 이전 형식 추론(R6),
/// 더 높은 manifest schema(R7)는 판정 issue와 `elf update` 거부.
#[test]
fn check_version_shows_pin_line_and_generation() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let cli = elf_cli::embed::version();
    let line = elf_cli::pin::parse(cli).unwrap().line();
    let (code, out, _) = elf(&root, &["check", "version"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("CLI: elf {cli} (line {line}, structure generation 2)")), "{out}");
    assert!(out.contains(&format!("pin: {cli} → line {line} (structure generation 2) — changed only by `elf update`")), "{out}");
    // 2.28.x로 고정된 프로젝트 = 3.0 줄
    let mp = root.join(".elf/manifest.json");
    let m = fs::read_to_string(&mp).unwrap();
    fs::write(&mp, m.replace(&format!("\"elf_version\": \"{cli}\""), "\"elf_version\": \"v2.28.1\"")).unwrap();
    let (code, out, _) = elf(&root, &["check", "version"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("pin: v2.28.1 → line 3.0 (structure generation 2; 2.28.x is the 3.0 line)"), "{out}");
    if line.to_string() == "3.0" {
        assert!(out.contains("ok: verdict: project v2.28.1") && out.contains("same line 3.0 (2.28.x is the 3.0 line)"), "{out}");
    } else {
        assert!(out.contains(&format!("project v2.28.1 is older than CLI {cli}")), "{out}");
    }
    // 더 높은 manifest schema
    fs::write(&mp, m.replace("elf-manifest/2", "elf-manifest/3")).unwrap();
    let (code, out, _) = elf(&root, &["check", "version"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("schema elf-manifest/3 is newer than CLI"), "{out}");
    let (code, out, err) = elf(&root, &["update"]);
    assert_eq!(code, 3, "{out}{err}");
    assert!(err.contains("elf self update"), "{err}");
    // .elf/ 없이 2.27 형식 로그만 있는 폴더 = 2.27 줄 추론
    let old = tmp.path().join("old");
    fs::create_dir_all(old.join("2_Log")).unwrap();
    fs::write(old.join("2_Log/S001_log.md"), "# S001: a\n\n> **Status**: ★ 활성\n").unwrap();
    let (_, out, _) = elf(&old, &["check", "version"]);
    assert!(out.contains("log-only folder (no .elf/): pin: none recorded (earlier log layout, no .elf/) → line 2.27 inferred (structure generation 1)"), "{out}");
}
