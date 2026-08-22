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

// S030: session new는 Registry 손상에 내성 — 파일 index(로그)로 번호를 내고 경고를 노출한다.
// (구 동작 "손상=Escalation 중단·무변경"의 의도적 전환 — 멀티에이전트 가용성 우선. validate/close의
//  Registry 파싱 escalation은 그대로 유지 — 진단·종료는 정합한 Registry를 요구.)
#[test]
fn malformed_registry_falls_back_to_log_index_with_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 4열 행 주입 (컬럼 수 불일치 = 손상)
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\t2026-06-13\tFoo\t★ 활성\n").unwrap();

    let res = run_session_new(&root, &opts("X")).unwrap();
    assert_eq!(res.id, "S002"); // 파일 index(S001)+1 — Registry 무시
    assert!(res.warnings.iter().any(|w| w.contains("unparseable")), "{:?}", res.warnings);
    assert!(root.join("2_Log/S002_log.md").exists()); // 로그는 생성됨(중단 아님)
    // 새 행은 raw append(손상부 보존·데이터 무손실)
    let reg = fs::read_to_string(root.join(REG)).unwrap();
    assert!(reg.lines().any(|l| l.starts_with("S002\t")));
}

// 현실 시나리오: worktree merge 충돌 마커로 Registry가 깨져도 파일 index로 번호를 낸다.
#[test]
fn merge_conflict_registry_still_numbers_from_log_index() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005_log.md"), "# S005\n").unwrap(); // 파일 index를 S005까지
    fs::write(
        root.join(REG),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\n\
<<<<<<< HEAD\nS002\ta\tb\t★ 활성\t-\t-\n=======\nS003\tc\td\t★ 활성\t-\t-\n>>>>>>> branch\n",
    )
    .unwrap();

    let res = run_session_new(&root, &opts("Y")).unwrap();
    assert_eq!(res.id, "S006"); // max log(S005)+1, 손상 Registry 무시
    assert!(res.warnings.iter().any(|w| w.contains("unparseable")));
    assert!(root.join("2_Log/S006_log.md").exists());
}

// §12.13 ①: 동시 세션 생성 원자성 — create_new(O_EXCL) 번호 예약 + Registry O_APPEND.
// barrier로 8스레드 동시 출발: 전 성공·번호 전부 고유·로그 실존·Registry 행 무손실(lost update 0).
#[test]
fn concurrent_session_new_yields_unique_numbers_and_loses_no_registry_rows() {
    use std::sync::{Arc, Barrier};
    let tmp = tempdir().unwrap();
    let root = Arc::new(new_project(tmp.path())); // S001 존재
    const N: usize = 8;
    let barrier = Arc::new(Barrier::new(N));
    let mut handles = Vec::new();
    for i in 0..N {
        let root = Arc::clone(&root);
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            let o = SessionNewOptions { title: format!("parallel {i}"), date: "2026-07-15".into() };
            barrier.wait(); // 동시 출발 강제
            run_session_new(&root, &o).map(|r| r.id)
        }));
    }
    let mut ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap().unwrap()).collect();
    ids.sort();
    // 전부 성공 + 번호 고유 (S002..S009)
    let expect: Vec<String> = (2..2 + N as u32).map(|n| format!("S{n:03}")).collect();
    assert_eq!(ids, expect, "unique sequential ids");
    // 로그 파일 전부 실존 + 제목 렌더 (덮어쓰기 없음)
    for id in &ids {
        let c = fs::read_to_string(root.join(format!("2_Log/{id}_log.md"))).unwrap();
        assert!(c.contains(&format!("# {id}:")), "{id} content intact");
    }
    // Registry 행 무손실 — 전체 rewrite였다면 일부 소실
    let reg = fs::read_to_string(root.join(REG)).unwrap();
    for id in &ids {
        assert!(reg.lines().any(|l| l.starts_with(&format!("{id}\t"))), "{id} row present:\n{reg}");
    }
    assert!(reg.lines().any(|l| l.starts_with("S001\t")), "pre-existing row kept");
}

// 번호 선점 파일이 있으면 재시도 없이도 다음 번호로 (파일 index 정본 — create_new가 존재 파일과 충돌하지 않음)
#[test]
fn preseeded_log_file_advances_number() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S002_log.md"), "# S002 (manual)\n").unwrap(); // 등록 없는 선점
    let res = run_session_new(&root, &opts("next")).unwrap();
    assert_eq!(res.id, "S003");
    // 선점 파일 불변 (덮어쓰기 없음)
    assert_eq!(fs::read_to_string(root.join("2_Log/S002_log.md")).unwrap(), "# S002 (manual)\n");
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
fn e2e_malformed_registry_falls_back_with_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join(REG), "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n").unwrap();
    // S030: 손상 Registry여도 exit 0 — 파일 index로 번호 할당 + 경고 노출(중단 아님)
    let assert = Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["session", "new", "X"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    assert!(stdout.contains("unparseable"), "{stdout}");
    assert!(stdout.contains("S002"), "{stdout}");
}

// ── session close (t02) ──────────────────────────────

use elf_cli::session::{find_log_root, run_session_close, CloseOptions};

/// close 전 자동 validate — 닫는 세션 스코프 경고만 보고, 비차단 (S027 #7: close 직전 = 마지막 검증 기회)
#[test]
fn close_reports_validate_findings_scoped_and_nonblocking() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // S001 활성
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
    let r = run_session_close(&root, &CloseOptions { id: None, force: false }).unwrap();
    assert_eq!(r.id, "S001");
}

/// v2.20: 구 `## 다음 세션 후보` 게이트 제거 — 절이 없어도 close 성공, `--force`는 호환용 no-op.
#[test]
fn close_no_longer_requires_next_section() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // v2.20 템플릿 = 후보 절 없음
    let log = std::fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(!log.contains("## 다음 세션 후보"), "new template must not carry the next-section block");
    run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
    assert!(root.join("2_Log/Archive/S001_log.md").exists());
    // --force: 여전히 수용되나 동작 차이 없음
    let root2 = new_project(&tmp.path().join("sub2"));
    run_session_close(&root2, &CloseOptions { id: Some("S001".into()), force: true }).unwrap();
    assert!(root2.join("2_Log/Archive/S001_log.md").exists());
}

/// 구 템플릿(≤ v2.19) 로그 — 절이 남아 있어도 close 동작 동일(하위호환: 절 유효·요구 없음).
#[test]
fn close_accepts_legacy_log_with_next_section() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let p = root.join("2_Log/S001_log.md");
    let c = std::fs::read_to_string(&p).unwrap()
        + "\n---\n\n## 다음 세션 후보 (Next-Session Hypothesis)\n\n### 가설 후보\n- [후속 가설 1-3항]\n";
    std::fs::write(&p, c).unwrap(); // placeholder 상태(구 게이트라면 refuse였을 형태)
    run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
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
fn e2e_close_succeeds_without_force_and_force_is_noop() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf").unwrap().current_dir(&root)
        .args(["session", "close", "S001"]).assert().success()
        .stdout(predicates::str::contains("closed S001"));
    let root2 = new_project(&tmp.path().join("sub2"));
    Command::cargo_bin("elf").unwrap().current_dir(&root2)
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
