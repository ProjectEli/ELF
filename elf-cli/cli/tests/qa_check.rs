//! qa preset 번들 검사(v3.8 S057): `preset: qa` 프로젝트에서 `elf check logs`가 번들을 검사(모두 경고) —
//! frontmatter, category와 폴더, status와 위치, 이름(카테고리 README가 있으면 생략), 같은 날짜 글자 중복,
//! 관련 문서와 대체 문서 경로. archive로 옮겨진 번들을 가리키는 경로는 직접 실행한 check가 고침(`fix_links`).

use elf_cli::init::{InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn qa_project(tmp: &Path) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: "Q".into(),
            preset: "qa".into(),
            modules: None,
            categories: vec!["Daily".into(), "IT".into()],
            lang: "ko-KR".into(),
            date: "2026-11-11".into(),
        },
    )
    .unwrap()
}

fn elf(root: &Path, args: &[&str]) -> (i32, String) {
    let out = assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(args).current_dir(root).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)),
    )
}

fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn bundle(cat: &str, status: &str, extra_body: &str) -> String {
    format!(
        "---\ndate: 2026-11-11 10:00:00\ncategory: {cat}\ntags: [a, b, c]\nstatus: {status}   # active | archive | deprecated\n---\n{extra_body}\n# 목표 (Objective)\n- x\n"
    )
}

#[test]
fn clean_qa_project_reports_bundles_and_no_warnings() {
    let tmp = tempdir().unwrap();
    let root = qa_project(tmp.path());
    put(&root, "Daily/20261111A_Clean.md", &bundle("Daily", "active", ""));
    put(&root, "IT/archive/20261110A_Done.md", &bundle("IT", "archive", ""));
    put(&root, "IT/archive/20261109A_Wrong.md", &bundle("IT", "deprecated", "> 대체 문서: `IT/archive/20261110A_Done.md`\n"));
    let (code, out) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("info: qa bundles: 3 in 2 categories"), "{out}");
    assert!(out.contains("ok: bundle frontmatter") && !out.contains("warn:"), "{out}");
    assert!(!out.contains("no 2_Log/"), "qa projects are no longer skipped: {out}");
}

#[test]
fn format_and_location_problems_are_warned() {
    let tmp = tempdir().unwrap();
    let root = qa_project(tmp.path());
    put(&root, "Daily/20261111A_NoFm.md", "# 목표\n");
    put(&root, "Daily/20261111B_Missing.md", "---\ndate: 2026-11-11 10:00:00\ncategory: Daily\nstatus: active\n---\n");
    put(&root, "Daily/20261111C_BadDate.md", &bundle("Daily", "active", "").replace("2026-11-11 10:00:00", "2026/11/11"));
    put(&root, "Daily/20261111D_WrongCat.md", &bundle("IT", "active", ""));
    put(&root, "Daily/20261111E_DepRoot.md", &bundle("Daily", "deprecated", ""));
    put(&root, "Daily/archive/20261110A_ActiveArch.md", &bundle("Daily", "active", ""));
    put(&root, "Daily/20261111F_EmptyThread.md", &bundle("Daily", "active", "").replace("status: active", "status: active\nthread:"));
    put(&root, "Daily/bad name.md", &bundle("Daily", "active", ""));
    put(&root, "Daily/archive/20261111A_Dup.md", &bundle("Daily", "archive", "")); // 20261111A 중복(루트 NoFm)
    let (code, out) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "warnings only: {out}");
    for want in [
        "Daily/20261111A_NoFm.md: no frontmatter",
        "Daily/20261111B_Missing.md: the frontmatter lacks tags",
        "Daily/20261111C_BadDate.md: date `2026/11/11` is not YYYY-MM-DD HH:MM:SS",
        "Daily/20261111D_WrongCat.md: category `IT` differs from its folder `Daily`",
        "Daily/20261111E_DepRoot.md: status deprecated, but the bundle is at the category root",
        "Daily/archive/20261110A_ActiveArch.md: status active, but the bundle is in archive/",
        "Daily/20261111F_EmptyThread.md: thread is empty",
        "Daily/bad name.md: the name is not YYYYMMDD<L>_<Subject>.md",
        "Daily: 20261111A is used by 2 bundles",
    ] {
        assert!(out.contains(want), "missing `{want}`:\n{out}");
    }
}

/// 카테고리 README = 그 카테고리 규칙(이름 검사 생략, README는 번들 아님). `_` 폴더와 참고 자료 폴더는 번들 검사 밖.
#[test]
fn category_readme_underscore_and_reference_folders() {
    let tmp = tempdir().unwrap();
    let root = qa_project(tmp.path());
    put(&root, "YT/README.md", "# YT 규칙\n이름 = Y###_<Subject>.md\n");
    put(&root, "YT/Y001_Talk.md", &bundle("YT", "active", ""));
    put(&root, "YT/_pipeline/notes.md", "no frontmatter, [x](missing.png)\n");
    put(&root, "Daily/wiki/game.md", "no frontmatter\n");
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("qa bundles: 1 in 3 categories"), "{out}");
    assert!(!out.contains("warn:"), "{out}");
}

/// 관련 문서가 archive로 옮겨진 번들을 가리키면 고치고(두 번째 실행 = 0), 끄면 알리기만, 대상이 없으면 경고.
/// archive로 옮긴 번들 안의 마크다운 링크는 폴더 한 단계 차이로 고침.
#[test]
fn related_paths_to_moved_bundles_are_fixed() {
    let tmp = tempdir().unwrap();
    let root = qa_project(tmp.path());
    put(&root, "IT/archive/20261101A_Old.md", &bundle("IT", "archive", "> 관련 문서: [n](20261111A_New.md)\n"));
    put(&root, "IT/20261111A_New.md", &bundle("IT", "active", "> 관련 문서: `IT/20261101A_Old.md`, `IT/20261001A_Gone.md`\n"));
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("fixed: IT/20261111A_New.md: 1 path(s) to bundles moved to or from archive/ (e.g. `IT/20261101A_Old.md` → `IT/archive/20261101A_Old.md`)"), "{out}");
    assert!(out.contains("fixed: IT/archive/20261101A_Old.md: 1 path(s)") && out.contains("→ `../20261111A_New.md`"), "{out}");
    assert!(out.contains("IT/20261111A_New.md: 1 related or replaced-by path(s) do not exist → `IT/20261001A_Gone.md`"), "{out}");
    let new = fs::read_to_string(root.join("IT/20261111A_New.md")).unwrap();
    assert!(new.contains("> 관련 문서: `IT/archive/20261101A_Old.md`, `IT/20261001A_Gone.md`"), "{new}");
    let (_, again) = elf(&root, &["check", "logs"]);
    assert!(!again.contains("fixed:"), "idempotent: {again}");
}

#[test]
fn fix_links_false_only_reports_moved_paths() {
    let tmp = tempdir().unwrap();
    let root = qa_project(tmp.path());
    let cfg = root.join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    v["fix_links"] = serde_json::Value::Bool(false);
    fs::write(&cfg, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    put(&root, "IT/archive/20261101A_Old.md", &bundle("IT", "archive", ""));
    let text = bundle("IT", "active", "> Related: `IT/20261101A_Old.md`\n");
    put(&root, "IT/20261111A_New.md", &text);
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("IT/20261111A_New.md: 1 path(s) point to bundles moved to or from archive/") && !out.contains("fixed:"), "{out}");
    assert_eq!(fs::read_to_string(root.join("IT/20261111A_New.md")).unwrap(), text);
}
