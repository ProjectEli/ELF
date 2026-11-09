//! `elf deprecate` 통합(e2e) — 압축 보관(.elf/deprecated/<ID>.md.xz), 흔적, show, restore, list, 종료 코드.
//! 단위별 세부(trial, 절, 표시 블록, 전체, 계획 문서)는 src/deprecate.rs 단위 테스트.

use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{SessionNewOptions, run_session_new};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn project(tmp: &Path) -> PathBuf {
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
    let p = root.join("2_Log/S/S001_log.md");
    let mut c = fs::read_to_string(&p).unwrap();
    c.push_str("\n## t01: a\n\n### 목표 (Goal)\n- x\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- 틀린 가설\n\n### 예상 (Prediction)\n- p\n\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 불일치\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n- 없음\n\n---\n\n## t02: b\n\n### 목표 (Goal)\n- y\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n");
    fs::write(&p, c).unwrap();
    root
}

fn elf(root: &Path, args: &[&str]) -> std::process::Output {
    assert_cmd::Command::cargo_bin("elf").unwrap().args(args).current_dir(root).output().unwrap()
}

#[test]
fn e2e_trial_deprecate_show_list_restore() {
    let tmp = tempdir().unwrap();
    let root = project(tmp.path());
    let before = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    let out = elf(&root, &["deprecate", "S001", "--trial", "t01", "--reason", "조건 오류", "--replaced-by", "S001 t02"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("deprecated S001-D01 (t01) → .elf/deprecated/S001-D01.md.xz"), "{so}");
    assert!(so.contains("commit the document change and its .elf/deprecated/ store file together"));
    let after = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(!after.contains("틀린 가설"));
    // grep으로도 원문이 보이지 않음(압축)
    let raw = fs::read(root.join(".elf/deprecated/S001-D01.md.xz")).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("틀린 가설"));
    assert!(String::from_utf8_lossy(&elf(&root, &["deprecate", "show", "S001-D01"]).stdout).contains("틀린 가설"));
    let list = String::from_utf8_lossy(&elf(&root, &["deprecate"]).stdout).into_owned();
    assert!(list.contains("S001-D01 · 2026-") && list.contains("1 deprecated item(s)"), "{list}");
    assert_cmd::Command::cargo_bin("elf").unwrap().args(["check", "logs"]).current_dir(&root).assert().code(0);
    assert!(elf(&root, &["deprecate", "--restore", "S001-D01"]).status.success());
    let back = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    let strip = |s: &str| s.lines().filter(|l| !l.starts_with("modified:")).collect::<Vec<_>>().join("\n");
    assert_eq!(strip(&back), strip(&before));
}

#[test]
fn e2e_exit_codes_and_usage() {
    let tmp = tempdir().unwrap();
    let root = project(tmp.path());
    assert_eq!(elf(&root, &["deprecate", "S009"]).status.code(), Some(1));
    assert_eq!(elf(&root, &["deprecate", "S001", "--trial", "t09"]).status.code(), Some(1));
    assert_eq!(elf(&root, &["deprecate", "--trial", "t01"]).status.code(), Some(2));
    assert_eq!(elf(&root, &["deprecate", "S001", "--trial", "t01", "--section", "가설"]).status.code(), Some(3));
    assert_eq!(elf(&root, &["deprecate", "README.md"]).status.code(), Some(3));
    assert_eq!(elf(&root, &["deprecate", "purge", "S001-D01"]).status.code(), Some(2), "purge was merged into deprecate");
    let dry = elf(&root, &["deprecate", "S001", "--dry-run"]);
    assert!(String::from_utf8_lossy(&dry.stdout).contains("dry-run — nothing written"));
    assert!(!root.join(".elf/deprecated").exists());
}
