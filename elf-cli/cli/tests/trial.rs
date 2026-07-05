//! S021 통합: `elf trial new`(정본 stub append) + L2 진입 파일 배치·pointer tier 거동.

use elf_cli::init::{InitOptions, run_init, run_init_ex};
use elf_cli::session::{SessionError, SessionNewOptions, run_session_new};
use elf_cli::trial::{TrialNewOptions, run_trial_new};
use elf_cli::update::{UpdateOptions, run_update};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn init_opts() -> InitOptions {
    InitOptions {
        name: "P".into(),
        preset: "minimal".into(),
        modules: None,
        categories: Vec::new(),
        lang: "ko-KR".into(),
        date: "2026-07-06".into(),
    }
}

fn new_project(tmp: &Path) -> PathBuf {
    run_init(tmp, &init_opts()).unwrap()
}

fn topts(title: Option<&str>, session: Option<&str>) -> TrialNewOptions {
    TrialNewOptions {
        title: title.map(String::from),
        session: session.map(String::from),
        date: "2026-07-06".into(),
    }
}

// ── trial new ───────────────────────────────────────────────────

#[test]
fn trial_new_appends_canonical_stub_before_next_section() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // init = S001 활성 로그(t01 stub 포함)

    let r = run_trial_new(&root, &topts(Some("두 번째 작업"), None)).unwrap();
    assert_eq!(r.trial, "t02");
    assert_eq!(r.session, "S001");

    let log = fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(log.contains("## t02: 두 번째 작업"));
    // 정본 stub 렌더: placeholder 치환 완료 + 정본 헤딩 포함
    assert!(!log.contains("t{NN}"));
    assert!(log.contains("64_Viz/S001/"));
    assert!(log.contains("### 가설 (Hypothesis)"));
    // 삽입 위치: t01 < t02 < 다음 세션 후보
    let t01 = log.find("## t01").unwrap();
    let t02 = log.find("## t02").unwrap();
    let next = log.find("## 다음 세션 후보").unwrap();
    assert!(t01 < t02 && t02 < next);
    // 헤더 Modified 갱신 (hard break 보존)
    assert!(log.contains("> **Modified**: 2026-07-06\\"));

    // 연속 증번
    assert_eq!(run_trial_new(&root, &topts(None, None)).unwrap().trial, "t03");
    let log2 = fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(log2.contains("## t03: [작업 제목]")); // 제목 생략 → placeholder 유지
}

#[test]
fn trial_new_errors_without_open_session() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let p = root.join("2_Log/S001_log.md");
    let closed = fs::read_to_string(&p)
        .unwrap()
        .replace("> **Status**: ★ 활성", "> **Status**: Complete");
    fs::write(&p, closed).unwrap();

    match run_trial_new(&root, &topts(None, None)) {
        Err(SessionError::NoOpenSession) => {}
        other => panic!("expected NoOpenSession, got {other:?}"),
    }
}

#[test]
fn trial_new_with_multiple_open_requires_session_flag() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    run_session_new(&root, &SessionNewOptions { title: "B".into(), date: "2026-07-06".into() })
        .unwrap();

    match run_trial_new(&root, &topts(None, None)) {
        Err(SessionError::MultipleOpen(ids)) => assert_eq!(ids, vec!["S001", "S002"]),
        other => panic!("expected MultipleOpen, got {other:?}"),
    }
    let r = run_trial_new(&root, &topts(Some("지정"), Some("S002"))).unwrap();
    assert_eq!(r.session, "S002");
    assert!(
        fs::read_to_string(root.join("2_Log/S002_log.md")).unwrap().contains("## t02: 지정")
    );
}

// ── L2 배치 · pointer tier ──────────────────────────────────────

#[test]
fn init_places_agents_digest_and_claude_pointer() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());

    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("ELF 에이전트 진입 규칙"));
    assert!(agents.contains("elf trial new")); // 스캐폴드 지시 탑재
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.contains("@AGENTS.md")); // 포인터 연결

    // ko 프로젝트: EN companion 미배포
    assert!(!root.join("AGENTS.en.md").exists());
}

#[test]
fn en_project_gets_agents_companion() {
    let tmp = tempdir().unwrap();
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp.path(), &o).unwrap();
    assert!(root.join("AGENTS.md").is_file()); // operative는 KO 정본 그대로
    assert!(root.join("AGENTS.en.md").is_file());
}

#[test]
fn inplace_init_keeps_existing_claude_md_without_elf_new() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "user rules\n").unwrap();

    let report = run_init_ex(tmp.path(), &init_opts(), true, false, false).unwrap();
    assert!(report.skipped.iter().any(|s| s == "CLAUDE.md"), "{:?}", report.skipped);
    // 기존 파일 무손상 + `.elf-new` 병기 없음 (pointer tier — t06)
    assert_eq!(fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap(), "user rules\n");
    assert!(!tmp.path().join("CLAUDE.md.elf-new").exists());
    // 규칙 본문(AGENTS.md)은 무충돌 정상 배치
    assert!(tmp.path().join("AGENTS.md").is_file());
}

#[test]
fn update_keeps_user_claude_md_without_conflict() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("CLAUDE.md"), "custom claude rules\n").unwrap();

    let rep = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(fs::read_to_string(root.join("CLAUDE.md")).unwrap(), "custom claude rules\n");
    assert!(!root.join("CLAUDE.md.elf-new").exists());
    assert_eq!(rep.conflicts, 0, "{:?}", rep.lines);
    assert!(
        rep.lines.iter().any(|l| l.contains("pointer (kept as-is): CLAUDE.md")),
        "{:?}",
        rep.lines
    );
}
