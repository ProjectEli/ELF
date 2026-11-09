//! v2.28.1 핫픽스 — CLI 갱신과 저장소 이관 분리(S044 t50): 이관하지 않은 2.27 형식 저장소는 2.27.4처럼 기록되고,
//! 새 구조 전용 명령만 이관 안내, 이관(`elf update`) 뒤에는 새 구조로 동작.

mod legacy_common;
use legacy_common::legacy_init;

use elf_cli::init::InitOptions;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn opts() -> InitOptions {
    InitOptions { name: "P".into(), preset: "minimal".into(), modules: None, categories: Vec::new(), lang: "ko-KR".into(), date: "2026-11-10".into() }
}

fn elf(root: &Path, args: &[&str]) -> (i32, String, String) {
    let o = assert_cmd::Command::cargo_bin("elf").unwrap().args(args).current_dir(root).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

#[test]
fn earlier_layout_keeps_recording_like_2_27() {
    let tmp = tempdir().unwrap();
    let root = legacy_init(tmp.path(), &opts()).unwrap();
    // S001 = init의 틀. 닫고 새 세션(2.27 사용법: --goal 없음)
    let (code, out, err) = elf(&root, &["session", "close"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("closed S001 → 2_Log/Archive/S001_log.md") && out.contains("earlier log layout"), "{out}");
    let (code, out, err) = elf(&root, &["session", "new", "파장 비교"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("created 2_Log/S002_log.md (S002) + registry row"), "{out}");
    assert!(fs::read_to_string(root.join("2_Log/Wiki/Session_Registry.tsv")).unwrap().contains("S002\t"));
    // trial new --from: 2.27 표기(첫 줄 > **Handoff**:)
    fs::write(
        root.join("p1.md"),
        "> **Handoff**: 상태; 미완료 = t01 확인 대기\n### 목표 (Goal)\n- a\n\n### 조건 (Conditions)\n- b\n\n### 가설 (Hypothesis)\n- c\n\n### 예상 (Prediction)\n- d\n",
    )
    .unwrap();
    let (code, out, err) = elf(&root, &["trial", "new", "첫 시도", "--from", "p1.md"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("appended t01 to 2_Log/S002_log.md") && out.contains("header Handoff replaced"), "{out}");
    let log = fs::read_to_string(root.join("2_Log/S002_log.md")).unwrap();
    assert!(log.contains("> **Handoff**: 상태; 미완료 = t01 확인 대기") && log.contains("## t01: 첫 시도"), "{log}");
    // 점검: 2.27 검사 + 이관 안내 info, issue 아님
    let (code, out, _) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("info: this project uses the earlier log layout"), "{out}");
    assert!(!out.contains("run `elf update` to migrate (LogConvention §7)"), "not an issue any more: {out}");
    let (code, out, _) = elf(&root, &["validate", "--check"]);
    assert_eq!(code, 0, "{out}");
    // autoread: 2.27 방식으로 진행 중 세션 Handoff를 실음
    let (_, out, _) = elf(&root, &["autoread"]);
    assert!(out.contains("S002: 상태; 미완료 = t01 확인 대기") && out.contains("earlier log layout"), "{out}");
    // 폐기(2.27: Deprecated/ 부분 파일)
    let (code, out, err) = elf(&root, &["deprecate", "S002", "--trial", "t01", "--reason", "r"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(root.join("2_Log/Deprecated/S002_log.partial.md").is_file(), "{out}");
}

#[test]
fn new_only_commands_point_to_migration_in_earlier_layout() {
    let tmp = tempdir().unwrap();
    let root = legacy_init(tmp.path(), &opts()).unwrap();
    for args in [
        vec!["log"],
        vec!["log", "show", "S001"],
        vec!["trial", "write", "--from", "x.md"],
        vec!["session", "new", "g", "--gov"],
        vec!["session", "close", "--from", "x.md"],
        vec!["deprecate", "show", "S001-D01"],
    ] {
        let (code, out, err) = elf(&root, &args);
        assert_eq!(code, 3, "{args:?}: {out}{err}");
        assert!(err.contains("available after this project is migrated"), "{args:?}: {err}");
    }
    assert!(!root.join("2_Log/S").exists() && !root.join("2_Log/G").exists(), "nothing written");
}

#[test]
fn new_structure_keeps_2_28_rules_for_the_old_surface() {
    let tmp = tempdir().unwrap();
    let root = elf_cli::init::run_init(tmp.path(), &opts()).unwrap();
    let (code, _, err) = elf(&root, &["session", "new", "x"]);
    assert_eq!(code, 2, "--goal is required in the new structure: {err}");
    assert!(err.contains("--goal is required"), "{err}");
    assert_eq!(elf(&root, &["session", "fix-headers"]).0, 2);
    assert_eq!(elf(&root, &["deprecate", "purge", "S001-D01"]).0, 2);
    assert_eq!(elf(&root, &["session", "new", "x", "--goal", "g"]).0, 0);
}

#[test]
fn migration_switches_the_project_to_the_new_structure() {
    let tmp = tempdir().unwrap();
    let root = legacy_init(tmp.path(), &opts()).unwrap();
    assert_eq!(elf(&root, &["session", "new", "둘째"]).0, 0);
    let git = |args: &[&str]| {
        let o = std::process::Command::new("git").arg("-C").arg(&root).args(args).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base"]);
    let (code, out, err) = elf(&root, &["update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(root.join("2_Log/S/S002_log.md").is_file() && !root.join("2_Log/Wiki/Session_Registry.tsv").exists());
    // 이관 뒤: 2.27 사용법은 새 규칙으로
    assert_eq!(elf(&root, &["session", "new", "x"]).0, 2);
    assert_eq!(elf(&root, &["log"]).0, 0);
}
