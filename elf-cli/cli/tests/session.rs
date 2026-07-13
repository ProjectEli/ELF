//! t01 통합: `elf session new` — init 프로젝트에 세션 생성·등록, escalation 동작.

use assert_cmd::Command;
use elf_cli::init::{run_init, InitOptions};
use elf_cli::session::{run_session_new, SessionError, SessionNewOptions};
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
            date: "2026-06-13".into(),
        },
    )
    .unwrap()
}

fn opts(title: &str) -> SessionNewOptions {
    SessionNewOptions { title: title.into(), date: "2026-06-13".into() }
}

const REG: &str = "2_Log/Wiki/Session_Registry.tsv";

#[test]
fn new_session_increments_after_s001_and_registers() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // init이 S001 로그+등록 생성

    let res = run_session_new(&root, &opts("Wavelength Sweep")).unwrap();
    assert_eq!(res.id, "S002");
    assert!(res.warnings.is_empty(), "clean project: {:?}", res.warnings);

    // 로그 파일 생성 + 치환 완료
    let log = fs::read_to_string(root.join("2_Log/S002_log.md")).unwrap();
    assert!(log.contains("# S002: Wavelength Sweep"));
    assert!(log.contains("2026-06-13"));
    assert!(!log.contains("S{NNN}"));
    assert!(!log.contains("[세션 제목]"));
    assert!(log.contains("> **Created**: 2026-06-13\\")); // hard break 보존

    // Registry 행 추가 (기존 S001 보존 + S002 append)
    let reg = fs::read_to_string(root.join(REG)).unwrap();
    assert!(reg.contains("S001\t"));
    assert!(reg.lines().any(|l| l.starts_with("S002\t") && l.contains("Wavelength Sweep") && l.ends_with("★ 활성\t-\t-")));
}

#[test]
fn consecutive_news_keep_incrementing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert_eq!(run_session_new(&root, &opts("A")).unwrap().id, "S002");
    assert_eq!(run_session_new(&root, &opts("B")).unwrap().id, "S003");
    assert_eq!(run_session_new(&root, &opts("C")).unwrap().id, "S004");
    let reg = fs::read_to_string(root.join(REG)).unwrap();
    assert_eq!(reg.lines().filter(|l| l.starts_with("S0")).count(), 4); // S001..S004
}

#[test]
fn number_derives_from_archive_too() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 과거 세션이 Archive로 이동된 상황 연출
    fs::write(root.join("2_Log/Archive/S005_log.md"), "# S005\n").unwrap();
    assert_eq!(run_session_new(&root, &opts("Next")).unwrap().id, "S006");
}

#[test]
fn malformed_registry_escalates_and_writes_nothing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 4열 행 주입
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\t2026-06-13\tFoo\t★ 활성\n").unwrap();

    match run_session_new(&root, &opts("X")) {
        Err(SessionError::Escalation(e)) => {
            assert_eq!(e.line, 2);
            assert!(e.to_string().contains("agent-action:"));
        }
        other => panic!("expected escalation, got {other:?}"),
    }
    // 무변경: S002 로그 미생성
    assert!(!root.join("2_Log/S002_log.md").exists());
}

#[test]
fn tab_in_title_refused() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    match run_session_new(&root, &opts("bad\ttitle")) {
        Err(SessionError::BadTitle(_)) => {}
        other => panic!("expected BadTitle, got {other:?}"),
    }
    assert!(!root.join("2_Log/S002_log.md").exists());
}

#[test]
fn unregistered_log_bumps_number_and_warns() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // S002 로그가 등록 없이 존재 → 다음은 max+1=S003(건너뜀) + 미등록 경고(검사는 validate 권유)
    fs::write(root.join("2_Log/S002_log.md"), "# S002\n").unwrap();
    let res = run_session_new(&root, &opts("Next")).unwrap();
    assert_eq!(res.id, "S003");
    assert!(
        res.warnings.iter().any(|w| w.contains("S002") && w.contains("not in registry")),
        "{:?}",
        res.warnings
    );
}

// ── e2e ──────────────────────────────────────────────

#[test]
fn e2e_session_new_success() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["session", "new", "My Session"])
        .assert()
        .success()
        .stdout(predicates::str::contains("S002"));
}

#[test]
fn e2e_malformed_registry_exits_5_with_marker() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n").unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["session", "new", "X"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicates::str::contains("agent-action:"));
}

// ── session close (t02) ──────────────────────────────

use elf_cli::session::{find_log_root, run_session_close, CloseOptions};

/// sessionTemplate의 '다음 세션 후보' placeholder를 실제 내용으로 (close 게이트 통과용)
fn fill_next_section(root: &std::path::Path, id: &str) {
    let p = root.join(format!("2_Log/{id}_log.md"));
    let c = std::fs::read_to_string(&p).unwrap().replace("- [후속 가설 1-3항]", "- 실제 후속 가설");
    std::fs::write(&p, c).unwrap();
}

/// close 전 자동 validate — 닫는 세션 스코프 경고만 보고, 비차단 (S027 #7: close 직전 = 마지막 검증 기회)
#[test]
fn close_reports_validate_findings_scoped_and_nonblocking() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // S001 활성
    fill_next_section(&root, "S001");
    // 닫는 세션(S001)의 embed 누락 연출: 64_Viz/S001에 그림 존재, 본문 embed 없음
    let viz1 = root.join("6_Exp/64_Viz/S001");
    fs::create_dir_all(&viz1).unwrap();
    fs::write(viz1.join("plot.png"), b"png").unwrap();
    // 타 세션(S002)의 embed 누락 연출 — 스코프 밖(닫는 세션 아님) 확인용
    elf_cli::session::run_session_new(
        &root,
        &elf_cli::session::SessionNewOptions { title: "other".into(), date: "2026-07-13".into() },
    )
    .unwrap();
    let viz2 = root.join("6_Exp/64_Viz/S002");
    fs::create_dir_all(&viz2).unwrap();
    fs::write(viz2.join("other.png"), b"png").unwrap();

    // 타 로그(S002) 본문에 닫는 세션 파일명이 인용된 깨진 링크 연출 — 접두 매칭이
    // 본문 인용을 스코프로 오귀속하지 않는지 확인(broken cross-ref → …S001_log.md)
    let s002 = root.join("2_Log/S002_log.md");
    let c = fs::read_to_string(&s002)
        .unwrap()
        .replace("- [이 작업의 구체적 목표]", "- [과거 로그](../S001_log.md) 참조");
    fs::write(&s002, c).unwrap();

    let r = run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
    // 비차단: close 자체는 성공(Archive 이동 완료)
    assert!(root.join("2_Log/Archive/S001_log.md").is_file());
    // 닫는 세션의 embed 경고 포함
    assert!(
        r.warnings.iter().any(|w| w.contains("validate (S001)") && w.contains("plot.png")),
        "closing-session validate finding missing: {:?}",
        r.warnings
    );
    // 스코프: 타 세션(S002) 경고는 미포함 — embed도, 본문 인용(broken cross-ref)도
    assert!(
        !r.warnings.iter().any(|w| w.contains("other.png")),
        "out-of-scope finding leaked into close warnings: {:?}",
        r.warnings
    );
    assert!(
        !r.warnings.iter().any(|w| w.contains("cross-ref")),
        "another log's broken cross-ref citing this session was misattributed: {:?}",
        r.warnings
    );
}

#[test]
fn close_archives_and_updates_registry() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // S001 활성
    fill_next_section(&root, "S001");

    let r = run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
    assert_eq!(r.id, "S001");
    assert!(!root.join("2_Log/S001_log.md").exists()); // root에서 제거
    let archived = fs::read_to_string(root.join("2_Log/Archive/S001_log.md")).unwrap(); // prefix 없이
    assert!(archived.contains("> **Status**: Complete"));
    let reg = fs::read_to_string(root.join(REG)).unwrap();
    assert!(reg.lines().any(|l| l.starts_with("S001\t") && l.contains("\tComplete\t") && l.contains("Archive/S001_log.md")));
}

#[test]
fn close_deepens_relative_cross_refs() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // S001 활성
    fill_next_section(&root, "S001");
    let p = root.join("2_Log/S001_log.md");
    let mut c = fs::read_to_string(&p).unwrap();
    c.push_str("\nsee [plan](../1_Concept/P.md) and [ext](https://e.com) and [img](../../elf-cli/x.md)\n");
    fs::write(&p, c).unwrap();

    run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
    let arch = fs::read_to_string(root.join("2_Log/Archive/S001_log.md")).unwrap();
    assert!(arch.contains("[plan](../../1_Concept/P.md)"), "../ 1개 보정: {arch}");
    assert!(arch.contains("[img](../../../elf-cli/x.md)"));
    assert!(arch.contains("[ext](https://e.com)")); // URL 불변
}

#[test]
fn close_default_picks_single_open() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fill_next_section(&root, "S001");
    let r = run_session_close(&root, &CloseOptions { id: None, force: false }).unwrap();
    assert_eq!(r.id, "S001");
}

#[test]
fn close_refuses_without_filled_next_section() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // S001은 placeholder 다음세션후보
    match run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }) {
        Err(SessionError::MissingNextSection(_)) => {}
        other => panic!("expected MissingNextSection, got {other:?}"),
    }
    assert!(root.join("2_Log/S001_log.md").exists()); // 무변경
    run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: true }).unwrap(); // --force 통과
    assert!(root.join("2_Log/Archive/S001_log.md").exists());
}

#[test]
fn close_multiple_open_requires_id() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    run_session_new(&root, &opts("Second")).unwrap(); // S002도 활성
    match run_session_close(&root, &CloseOptions { id: None, force: false }) {
        Err(SessionError::MultipleOpen(ids)) => assert_eq!(ids, vec!["S001", "S002"]),
        other => panic!("expected MultipleOpen, got {other:?}"),
    }
}

#[test]
fn close_nonexistent_errors() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    match run_session_close(&root, &CloseOptions { id: Some("S099".into()), force: true }) {
        Err(SessionError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn find_log_root_finds_2log_from_subdir() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert_eq!(find_log_root(&root.join("2_Log/Wiki")).unwrap(), root);
    let empty = tempdir().unwrap();
    assert!(find_log_root(empty.path()).is_none());
}

#[test]
fn e2e_close_refuse_then_force() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf").unwrap().current_dir(&root)
        .args(["session", "close", "S001"]).assert().failure().code(3);
    Command::cargo_bin("elf").unwrap().current_dir(&root)
        .args(["session", "close", "S001", "--force"]).assert().success()
        .stdout(predicates::str::contains("closed S001"));
}

// ── fix-headers ──────────────────────────────────────

#[test]
fn fix_headers_processes_log_and_archive() {
    use elf_cli::session::run_fix_headers;
    let tmp = tempdir().unwrap();
    let base = tmp.path();
    fs::create_dir_all(base.join("2_Log/Archive")).unwrap();
    let old = "# S001: T\n\n> **Created**: x\n> **Modified**: y\n> **Handoff**: -\n\n---\n";
    fs::write(base.join("2_Log/S005_log.md"), old).unwrap();
    fs::write(base.join("2_Log/Archive/S001_log.md"), old).unwrap();
    fs::write(base.join("2_Log/Wiki_skip.txt"), "ignored").ok(); // .md 아님 — skip (실제론 Wiki/)

    // dry-run: 변경 안 함
    let dry = run_fix_headers(base, true).unwrap();
    assert_eq!(dry.len(), 2);
    assert_eq!(fs::read_to_string(base.join("2_Log/S005_log.md")).unwrap(), old);

    // apply
    let applied = run_fix_headers(base, false).unwrap();
    assert_eq!(applied.len(), 2);
    for f in ["2_Log/S005_log.md", "2_Log/Archive/S001_log.md"] {
        let c = fs::read_to_string(base.join(f)).unwrap();
        assert!(c.contains("> **Created**: x\\\n"), "{f}");
        assert!(c.contains("> **Modified**: y\\\n"), "{f}");
        assert!(c.contains("> **Handoff**: -\n"), "{f} last line no backslash");
    }
    // 재실행 = 멱등(0)
    assert!(run_fix_headers(base, false).unwrap().is_empty());
}

#[test]
fn e2e_fix_headers_command() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // init S001 헤더는 이미 hard-break 적용본(템플릿) → 변경 0
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["session", "fix-headers", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0 file(s)"));
}

#[test]
fn e2e_outside_project_exits_1() {
    let tmp = tempdir().unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(tmp.path())
        .args(["session", "new", "X"])
        .assert()
        .failure()
        .code(1);
}
