//! trial 통합 — `trial new`(만들기, Phase 1) + `trial write`(Phase 1 수정, Phase 2), 진입 파일 배치, pointer tier.

use elf_cli::init::{InitOptions, run_init, run_init_ex};
use elf_cli::session::{SessionError, SessionNewOptions, run_session_new};
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
        date: "2026-11-09".into(),
    }
}

fn new_project(tmp: &Path) -> PathBuf {
    let root = run_init(tmp, &init_opts()).unwrap();
    run_session_new(&root, &SessionNewOptions { title: "t".into(), date: "2026-11-09".into(), goal: "g".into(), gov: false }).unwrap();
    root
}

const P1: &str = "---\nhandoff: '상태; 미완료 = t01 Phase 1 사용자 확인 대기'\n---\n### 목표 (Goal)\n- 파장별 SNR\n\n### 조건 (Conditions)\n- 광자 1e8\n\n### 가설 (Hypothesis)\n- 810 우위\n\n### 예상 (Prediction)\n- 810 최대\n";
const P2: &str = "### 관찰 (Observation)\n- 810 최대\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n\n### 교훈 (Lessons)\n- 1e9 필요\n\n### 생성 파일 (Files)\n- 없음\n";

fn elf(root: &Path, args: &[&str]) -> std::process::Output {
    assert_cmd::Command::cargo_bin("elf").unwrap().args(args).current_dir(root).output().unwrap()
}

#[test]
fn e2e_new_from_write_revise_and_phase2() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("p1.md"), P1).unwrap();
    fs::write(root.join("p2.md"), P2).unwrap();
    let out = elf(&root, &["trial", "new", "3파장 sweep", "--from", "p1.md"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("t01 Phase 1 written") && so.contains("stop — show Phase 1 to the user"), "{so}");
    assert!(so.contains("header handoff replaced"));
    // Phase 1 수정
    fs::write(root.join("p1.md"), P1.replace("광자 1e8", "광자 1e9")).unwrap();
    assert!(elf(&root, &["trial", "write", "--from", "p1.md"]).status.success());
    // Phase 2
    let out = elf(&root, &["trial", "write", "t01", "--from", "p2.md"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(log.contains("광자 1e9") && log.contains("### 관찰 (Observation)") && log.contains("(2 writes)"));
    // Phase 2 뒤 Phase 1 수정 = 거부, 로그 그대로
    let out = elf(&root, &["trial", "write", "--from", "p1.md"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already has Phase 2"));
    assert_eq!(fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap(), log);
}

#[test]
fn e2e_new_from_refuses_phase2_sections_and_mixed_files() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("all.md"), format!("{P1}\n{P2}")).unwrap();
    let out = elf(&root, &["trial", "new", "x", "--from", "all.md"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("one phase only"));
    assert!(!fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap().contains("## t01"));
}

#[test]
fn trial_needs_one_ongoing_session_or_flag() {
    let tmp = tempdir().unwrap();
    let root = run_init(tmp.path(), &init_opts()).unwrap();
    assert_eq!(elf(&root, &["trial", "new", "x"]).status.code(), Some(1));
    for gov in [false, true] {
        run_session_new(&root, &SessionNewOptions { title: "t".into(), date: "d".into(), goal: "g".into(), gov }).unwrap();
    }
    let out = elf(&root, &["trial", "new", "x"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("multiple ongoing sessions (S001, G001)"));
    assert!(elf(&root, &["trial", "new", "x", "--session", "G001"]).status.success());
    assert!(fs::read_to_string(root.join("2_Log/G/G001_log.md")).unwrap().contains("## t01: x"));
    let _ = SessionError::NoOpenSession;
}

// ── 진입 파일, pointer tier ──────────────────────────────────────

#[test]
fn init_places_agents_digest_and_claude_pointer() {
    let tmp = tempdir().unwrap();
    let root = run_init(tmp.path(), &init_opts()).unwrap();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("ELF 에이전트 진입 규칙"));
    assert!(agents.contains("elf trial new") && agents.contains("elf trial write"));
    assert!(fs::read_to_string(root.join("CLAUDE.md")).unwrap().contains("@AGENTS.md"));
    assert!(!root.join("AGENTS.en.md").exists());
}

#[test]
fn en_project_gets_agents_companion() {
    let tmp = tempdir().unwrap();
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp.path(), &o).unwrap();
    assert!(root.join("AGENTS.md").is_file());
    assert!(root.join("AGENTS.en.md").is_file());
    assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("| Session | status |"));
}

#[test]
fn inplace_init_keeps_existing_claude_md_without_elf_new() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("CLAUDE.md"), "user rules\n").unwrap();
    let report = run_init_ex(tmp.path(), &init_opts(), true, false, false).unwrap();
    assert!(report.skipped.iter().any(|s| s == "CLAUDE.md"), "{:?}", report.skipped);
    assert_eq!(fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap(), "user rules\n");
    assert!(!tmp.path().join("CLAUDE.md.elf-new").exists());
    assert!(tmp.path().join("AGENTS.md").is_file());
}

#[test]
fn dirty_warning_fires_only_on_tracked_changes_or_unborn_head() {
    fn git(root: &Path, args: &[&str]) {
        let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().expect("git runs");
        assert!(o.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&o.stderr));
    }
    fn update_lines(root: &Path) -> String {
        run_update(root, &UpdateOptions { dry_run: true, force: false }).unwrap().lines.join("\n")
    }
    let tmp = tempdir().unwrap();
    let root = run_init(tmp.path(), &init_opts()).unwrap();
    let l = update_lines(&root);
    assert!(!l.contains("would mix") && !l.contains("no git commit"), "{l}");
    git(&root, &["init", "-q"]);
    assert!(update_lines(&root).contains("no git commit yet"));
    git(&root, &["add", "-A"]);
    git(&root, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "init"]);
    let l = update_lines(&root);
    assert!(!l.contains("would mix") && !l.contains("no git commit"), "{l}");
    fs::write(root.join("scratch.txt"), "tmp\n").unwrap();
    assert!(!update_lines(&root).contains("would mix"));
    let readme = root.join("README.md");
    let mut c = fs::read_to_string(&readme).unwrap();
    c.push_str("\nlocal edit\n");
    fs::write(&readme, c).unwrap();
    assert!(update_lines(&root).contains("would mix in one tree"));
}

#[test]
fn update_keeps_user_claude_md_without_conflict() {
    let tmp = tempdir().unwrap();
    let root = run_init(tmp.path(), &init_opts()).unwrap();
    fs::write(root.join("CLAUDE.md"), "custom claude rules\n").unwrap();
    let rep = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(fs::read_to_string(root.join("CLAUDE.md")).unwrap(), "custom claude rules\n");
    assert!(!root.join("CLAUDE.md.elf-new").exists());
    assert_eq!(rep.conflicts, 0, "{:?}", rep.lines);
}
