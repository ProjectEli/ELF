//! 세션 수명주기 통합 — 구조 개편(P022): `2_Log/S/`, `2_Log/G/`, YAML 헤더, `--goal`, `close --from`, Registry.md.

use elf_cli::header::{self, Status};
use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{CloseOptions, SessionError, SessionNewOptions, find_log_root, run_session_close, run_session_new};
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
            date: "2026-11-09".into(),
        },
    )
    .unwrap()
}

fn opts(title: &str, gov: bool) -> SessionNewOptions {
    SessionNewOptions { title: title.into(), date: "2026-11-09".into(), goal: format!("{title} 목표"), gov }
}

fn close(root: &Path, id: Option<&str>, body: Option<&str>) -> Result<elf_cli::session::CloseResult, SessionError> {
    run_session_close(root, &CloseOptions { id: id.map(str::to_string), body: body.map(str::to_string), date: "2026-11-10".into() })
}

#[test]
fn init_has_no_session_and_numbers_start_at_one_per_series() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert!(root.join("2_Log/S").is_dir());
    assert!(!root.join("2_Log/S/S001_log.md").exists(), "init creates no session");
    assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("| 세션 | status |"));
    assert!(!root.join("2_Log/Wiki/Session_Registry.tsv").exists());
    assert_eq!(run_session_new(&root, &opts("a", false)).unwrap().id, "S001");
    assert_eq!(run_session_new(&root, &opts("b", false)).unwrap().id, "S002");
    assert_eq!(run_session_new(&root, &opts("rule", true)).unwrap().id, "G001");
    assert!(root.join("2_Log/G/Registry.md").is_file());
}

#[test]
fn concurrent_session_new_yields_unique_numbers() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let r = root.clone();
            std::thread::spawn(move || run_session_new(&r, &opts(&format!("p{i}"), false)).unwrap().id)
        })
        .collect();
    let mut ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8, "{ids:?}");
}

#[test]
fn goal_required_and_legacy_layout_refused() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let mut o = opts("a", false);
    o.goal = "  ".into();
    assert!(matches!(run_session_new(&root, &o), Err(SessionError::BadTitle(m)) if m.contains("--goal")));
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    assert!(run_session_new(&root, &opts("a", false)).is_ok(), "an empty Archive/ is not the earlier layout");
    fs::write(root.join("2_Log/Archive/S007_log.md"), "# S007: old\n").unwrap();
    assert!(matches!(run_session_new(&root, &opts("b", false)), Err(SessionError::Legacy(_))));
}

#[test]
fn close_writes_header_keeps_file_in_place_and_reports_scoped_findings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    run_session_new(&root, &opts("a", false)).unwrap();
    let p = root.join("2_Log/S/S001_log.md");
    fs::write(&p, format!("{}\n## t01: x\n\n### 목표 (Goal)\n- g\n\n### 관찰 (Observation)\n- o\n", fs::read_to_string(&p).unwrap())).unwrap();
    assert!(matches!(close(&root, None, None), Err(SessionError::Rejected(v)) if v.iter().any(|x| x.contains("approval"))));
    let r = close(&root, None, Some("---\nconclusion: '결론'\nuser_note: ''\n---\n")).unwrap();
    assert_eq!(r.log_rel, "2_Log/S/S001_log.md");
    assert!(r.warnings.iter().any(|w| w.contains("check (S001)") && w.contains("가설")), "{:?}", r.warnings);
    let (h, _) = header::parse(&fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!((h.status, h.conclusion.as_str()), (Status::Complete, "결론"));
    assert!(p.is_file(), "not moved");
    assert!(!root.join("2_Log/Archive").exists());
}

#[test]
fn close_target_selection() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert!(matches!(close(&root, None, Some("---\nconclusion: 'x'\n---")), Err(SessionError::NoOpenSession)));
    run_session_new(&root, &opts("a", false)).unwrap();
    run_session_new(&root, &opts("g", true)).unwrap();
    assert!(matches!(close(&root, None, Some("---\nconclusion: 'x'\n---")), Err(SessionError::MultipleOpen(v)) if v.len() == 2));
    assert_eq!(close(&root, Some("G001"), Some("---\nconclusion: 'x'\n---")).unwrap().id, "G001");
    assert!(matches!(close(&root, Some("S009"), Some("---\nconclusion: 'x'\n---")), Err(SessionError::NotFound(_))));
}

#[test]
fn find_log_root_finds_2log_from_subdir() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let sub = root.join("6_Exp");
    fs::create_dir_all(&sub).unwrap();
    assert_eq!(find_log_root(&sub).unwrap(), root);
}

#[test]
fn e2e_session_new_and_close() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let out = assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .args(["session", "new", "SNR 비교", "--goal", "파장별 SNR 비교"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("created 2_Log/S/S001_log.md (S001)"));
    // --goal 없이 = usage 오류(clap)
    assert_cmd::Command::cargo_bin("elf").unwrap().args(["session", "new", "x"]).current_dir(&root).assert().code(2);
    // 종료 파일 없이 = 거부(exit 3)
    assert_cmd::Command::cargo_bin("elf").unwrap().args(["session", "close"]).current_dir(&root).assert().code(3);
    let f = root.join("close.md");
    fs::write(&f, "---\nconclusion: '810 nm 최적'\n---\n").unwrap();
    let out = assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .args(["session", "close", "--from"])
        .arg(&f)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("status complete"));
}

#[test]
fn e2e_outside_project_exits_1() {
    let tmp = tempdir().unwrap();
    assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .args(["session", "new", "x", "--goal", "g"])
        .current_dir(tmp.path())
        .assert()
        .code(1);
}
