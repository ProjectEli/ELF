//! `elf check logs` 통합 — 구조 개편(P022): 헤더 형식, status별 값, 번호, 링크, figure embed, Registry.md, 이전 형식.

use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{SessionNewOptions, run_session_new};
use elf_cli::validate::run_validate;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn new_project(tmp: &Path) -> PathBuf {
    let root = run_init(
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
    .unwrap();
    run_session_new(&root, &SessionNewOptions { title: "t".into(), date: "2026-11-09".into(), goal: "g".into(), gov: false }).unwrap();
    root
}

fn log(root: &Path) -> PathBuf {
    root.join("2_Log/S/S001_log.md")
}

fn append(root: &Path, text: &str) {
    let mut c = fs::read_to_string(log(root)).unwrap();
    c.push_str(text);
    fs::write(log(root), c).unwrap();
}

fn lines(root: &Path) -> (usize, usize, String) {
    let r = run_validate(root).unwrap();
    (r.issues, r.warnings, r.lines.join("\n"))
}

#[test]
fn clean_project_has_no_findings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let (i, w, l) = lines(&root);
    assert_eq!((i, w), (0, 0), "{l}");
    assert!(l.contains("ok:"));
}

#[test]
fn header_format_error_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let c = fs::read_to_string(log(&root)).unwrap().replace("status: ongoing", "status: Active");
    fs::write(log(&root), c).unwrap();
    let (i, _, l) = lines(&root);
    assert!(i >= 1 && l.contains("S001_log.md: header line 4"), "{l}");
}

#[test]
fn malformed_name_issue_and_gap_and_multiple_active_warnings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S/S201-a_log.md"), "x").unwrap();
    let t = fs::read_to_string(log(&root)).unwrap().replace("# S001", "# S003");
    fs::write(root.join("2_Log/S/S003_log.md"), t).unwrap();
    let (_, _, l) = lines(&root);
    assert!(l.contains("issue: malformed session log name: 2_Log/S/S201-a_log.md"), "{l}");
    assert!(l.contains("warn: gap in session numbering: S002 missing"), "{l}");
    assert!(l.contains("warn: multiple active sessions (S001, S003)"), "{l}");
    assert!(l.contains("Registry.md differs from the log headers"), "S003 placed by hand: {l}");
}

#[test]
fn broken_cross_ref_is_issue_valid_is_not() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("1_Concept/12_Planning")).unwrap();
    fs::write(root.join("1_Concept/12_Planning/P001_a.md"), "# P001").unwrap();
    append(&root, "\n[ok](../../1_Concept/12_Planning/P001_a.md) [bad](../../1_Concept/12_Planning/P999.md)\n");
    let (i, _, l) = lines(&root);
    assert_eq!(i, 1, "{l}");
    assert!(l.contains("broken cross-ref → ../../1_Concept/12_Planning/P999.md"));
}

#[test]
fn figure_embed_warnings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("6_Exp/64_Viz/S001")).unwrap();
    for f in ["a.png", "b.png", "c.png"] {
        fs::write(root.join(format!("6_Exp/64_Viz/S001/{f}")), b"x").unwrap();
    }
    append(&root, "\n![Fig1](../../6_Exp/64_Viz/S001/a.png)\n| Figure | `../../6_Exp/64_Viz/S001/b.png` |\n<!-- noembed: c.png -->\n");
    let (i, w, l) = lines(&root);
    assert_eq!((i, w), (0, 1), "{l}");
    assert!(l.contains("figure 'b.png'"), "table path is not an embed: {l}");
}

#[test]
fn unfilled_phase1_template_warns_for_ongoing_only() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let tpl = elf_cli::embed::TEMPLATES.get_file("log/trialTemplate.md").unwrap().contents_utf8().unwrap();
    append(&root, &format!("\n{}", tpl.replace("t{NN}", "t01")));
    let (_, _, l) = lines(&root);
    assert!(l.contains("t01: title is still the template placeholder"), "{l}");
    assert!(l.contains("'### 목표' still holds only the template text"), "{l}");
}

#[test]
fn earlier_layout_is_one_issue_pointing_to_update() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::write(root.join("2_Log/S005_log.md"), "# S005: old\n").unwrap();
    let r = run_validate(root).unwrap();
    assert_eq!(r.issues, 1);
    assert!(r.lines[0].contains("earlier log layout") && r.lines[0].contains("elf update"));
}

#[test]
fn complete_without_conclusion_warns() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let c = fs::read_to_string(log(&root)).unwrap().replace("status: ongoing", "status: complete");
    fs::write(log(&root), c).unwrap();
    let (_, _, l) = lines(&root);
    assert!(l.contains("conclusion is empty while complete"), "{l}");
}

#[test]
fn e2e_check_logs_exit_codes() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(["check", "logs"]).current_dir(&root).assert().code(0);
    append(&root, "\n[bad](nope.md)\n");
    assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(["check", "logs"]).current_dir(&root).assert().code(4);
    assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(["validate", "--strict"]).current_dir(&root).assert().code(2);
}
