//! [이식] ELF v2.27.4 `tests/trial.rs` — 이전 형식(2.27) 저장소에서 2.28.1이 2.27.4와 같이 동작하는지(동결 모듈, 명령 분기).
//! fixture = `legacy_common::legacy_init`(2.27.4 init 결과). init, update 기능 테스트는 새 구조 테스트가 담당해 뺌.

mod legacy_common;
use legacy_common::legacy_init;
// S021 통합: `elf trial new`(정본 stub append) + L2 진입 파일 배치, pointer tier 거동.

use elf_cli::init::InitOptions;
use elf_cli::legacy::session::{SessionError, SessionNewOptions, run_session_new};
use elf_cli::legacy::trial::{TrialNewOptions, run_trial_new};
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
    legacy_init(tmp, &init_opts()).unwrap()
}

fn topts(title: Option<&str>, session: Option<&str>) -> TrialNewOptions {
    TrialNewOptions {
        title: title.map(String::from),
        session: session.map(String::from),
        date: "2026-07-06".into(),
        body: None,
    }
}

// ── trial new ───────────────────────────────────────────────────

/// trial new 출력의 embed 리마인더 — 행동 시점 주입(trial 시작 = figure 작성 이전 보장, S027)
#[test]
fn trial_new_stdout_carries_embed_reminder() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let out = assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .env("ELF_DISPATCH", "off")
        .current_dir(&root)
        .args(["trial", "new", "reminder-check"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        text.contains("embed") && text.contains("### 관찰"),
        "embed reminder missing in trial new output: {text}"
    );
    assert!(text.contains("expected figures"), "{text}");
}

#[test]
fn trial_new_from_file_appends_body_and_replaces_handoff() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let log = root.join("2_Log/S001_log.md");
    let elf = |args: &[&str]| assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").current_dir(&root).args(args).output().unwrap();
    // 셸 escape가 깨지기 쉬운 문자(백틱, `$`, 역슬래시, 따옴표)를 파일로 전달
    let body = "> **Handoff**: 상태 = `a\\b` 확인; 미완료 = $X; 참조 t01\n\n### 목표 (Goal)\n- \"따옴표\" 'quote' `code` $HOME \\n\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n";
    fs::write(tmp.path().join("t.md"), body).unwrap();
    let out = elf(&["trial", "new", "파일 입력", "--from", tmp.path().join("t.md").to_str().unwrap()]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("appended t01") && text.contains("from file") && text.contains("Handoff replaced"), "{text}");
    assert!(text.contains("Phase 2 sections (관찰, 해석, 교훈, 생성 파일)"), "{text}");
    let c = fs::read_to_string(&log).unwrap();
    assert!(c.contains("## t01: 파일 입력\n\n### 목표 (Goal)\n- \"따옴표\" 'quote' `code` $HOME \\n"), "{c}");
    assert!(c.contains("> **Handoff**: 상태 = `a\\b` 확인; 미완료 = $X; 참조 t01\n"));
    assert!(c.contains("### 관찰 (Observation)"));
    // 검사 실패 = 덧붙이지 않음, 로그 불변, exit 3
    fs::write(tmp.path().join("bad.md"), "### 목표 (Goal)\n- [이 작업의 구체적 목표]\n").unwrap();
    let out = elf(&["trial", "new", "bad", "--from", tmp.path().join("bad.md").to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(3));
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(err.contains("trial not appended") && err.contains("missing '### 조건'") && err.contains("template text"), "{err}");
    assert_eq!(fs::read_to_string(&log).unwrap(), c);
    // 제목 없음 = 거부
    let out = elf(&["trial", "new", "--from", tmp.path().join("t.md").to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--from needs a trial title"));
}

#[test]
fn trial_new_appends_canonical_stub_at_end() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // init = S001 활성 로그(헤더만 — v2.22 t01 stub 제거)

    let before = fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(!before.contains("## t"), "fresh log carries no trial stub");

    let r = run_trial_new(&root, &topts(Some("첫 작업"), None)).unwrap();
    assert_eq!(r.trial, "t01");
    assert_eq!(r.session, "S001");

    let log = fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(log.contains("## t01: 첫 작업"));
    // 정본 stub 렌더: placeholder 치환 완료 + 정본 헤딩 포함
    assert!(!log.contains("t{NN}"));
    assert!(log.contains("64_Viz/S001/"));
    assert!(log.contains("### 가설 (Hypothesis)"));
    // 첫 trial은 헤더 규범 주석 뒤에 바로(구분선 중복 없음), 본문 말미(EOF append)
    assert!(!log.contains("---\n\n---"));
    assert!(log.find("-->").unwrap() < log.find("## t01").unwrap());
    assert!(!log.contains("## 다음 세션 후보"));
    assert!(log.trim_end().ends_with("| Figure | `경로` |"), "stub appended at end of body");
    // 헤더 Modified 갱신 (hard break 보존)
    assert!(log.contains("> **Modified**: 2026-07-06\\"));

    // 연속 증번: t01 < t02, trial 사이 구분선
    assert_eq!(run_trial_new(&root, &topts(None, None)).unwrap().trial, "t02");
    let log2 = fs::read_to_string(root.join("2_Log/S001_log.md")).unwrap();
    assert!(log2.contains("## t02: [작업 제목]")); // 제목 생략 → placeholder 유지
    assert!(log2.find("## t01").unwrap() < log2.find("## t02").unwrap());
    assert!(log2.contains("\n\n---\n\n## t02"), "trials separated by a rule");
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
        fs::read_to_string(root.join("2_Log/S002_log.md")).unwrap().contains("## t01: 지정")
    );
}

// ── L2 배치, pointer tier ──────────────────────────────────────




#[test]
fn close_warns_on_pending_handoff_nonblocking() {
    use elf_cli::legacy::session::{CloseOptions, run_session_close};
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let p = root.join("2_Log/S001_log.md");
    let c = fs::read_to_string(&p)
        .unwrap()
        .replace("> **Handoff**: -", "> **Handoff**: 설계 확정; 미완료 = 문서 반영; 참조 t01");
    fs::write(&p, c).unwrap();

    // close (v2.20: 후보 절 게이트 없음 — force는 호환용 no-op; 경고는 비차단으로 함께 반환)
    let r = run_session_close(&root, &CloseOptions { id: None, force: true }).unwrap();
    assert_eq!(r.id, "S001");
    assert!(
        r.warnings.iter().any(|w| w.contains("pending") && w.contains("문서 반영")),
        "{:?}",
        r.warnings
    );

    // 소거된 Handoff는 경고 없음
    let root2 = new_project(&tmp.path().join("sub2"));
    let p2 = root2.join("2_Log/S001_log.md");
    let c2 = fs::read_to_string(&p2)
        .unwrap()
        .replace("> **Handoff**: -", "> **Handoff**: 완료; -; 참조 t01");
    fs::write(&p2, c2).unwrap();
    let r2 = run_session_close(&root2, &CloseOptions { id: None, force: true }).unwrap();
    assert!(r2.warnings.is_empty(), "{:?}", r2.warnings);
}


