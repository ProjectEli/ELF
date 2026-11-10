//! 링크와 경로 참조(v3.7 S056): 깨진 링크 검사를 로그 밖 문서와 모든 파일 유형으로 넓힘, 폴더 한 단계 차이
//! 자동 수정(직접 실행한 `elf check`, `elf check logs`만, `fix_links` 설정), 프로젝트 밖 경로 경고,
//! 생성 파일 표 오탐 수정. 판정에는 프로젝트 루트와 파일 존재 여부만 씀(폴더 구조, git 무관).

use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{SessionNewOptions, run_session_new};
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
            date: "2026-11-11".into(),
        },
    )
    .unwrap();
    run_session_new(&root, &SessionNewOptions { title: "t".into(), date: "2026-11-11".into(), goal: "g".into(), gov: false }).unwrap();
    root
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

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap()
}

const P1: &str = "### 목표 (Goal)\n- g\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n";

fn p2(files_rows: &str, observation: &str) -> String {
    format!(
        "### 관찰 (Observation)\n- {observation}\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n\n| 유형 | 파일 |\n|---|---|\n{files_rows}"
    )
}

/// 문서를 Archive로 옮긴 뒤 남은 `../` 링크 — 첫 실행에 고치고(fixed 줄), 두 번째 실행은 고칠 것이 없음.
#[test]
fn moved_document_links_are_fixed_once() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    put(&root, "6_Exp/64_Viz/S001/a.png", "x");
    put(&root, "1_Concept/12_Planning/P001.md", "x");
    // 원래 1_Concept/12_Planning/에서 쓴 링크를 Archive/로 옮긴 상태(줄바꿈 CRLF, 퍼센트 표기, 앵커와 제목 유지)
    put(
        &root,
        "1_Concept/12_Planning/Archive/P000_old.md",
        "# old\r\n![f](../../6_Exp/64_Viz/S001/a.png \"cap\")\r\n[p](P001.md#sec) [q](../P001.md)\r\n",
    );
    let (code, out) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("fixed: 1_Concept/12_Planning/Archive/P000_old.md: 2 link(s) one folder level"), "{out}");
    assert!(out.contains("fixed: 2 link(s) in 1 file(s)") && out.contains("\"fix_links\": false"), "{out}");
    assert_eq!(
        read(&root, "1_Concept/12_Planning/Archive/P000_old.md"),
        "# old\r\n![f](../../../6_Exp/64_Viz/S001/a.png \"cap\")\r\n[p](../P001.md#sec) [q](../P001.md)\r\n",
        "only the paths change; the link that resolves is kept"
    );
    let (_, again) = elf(&root, &["check", "logs"]);
    assert!(!again.contains("fixed:"), "idempotent: {again}");
}

/// `elf check`(전체 요약)도 고치고, 요약에 fixed 줄이 보임.
#[test]
fn check_all_fixes_and_shows_fixed_in_summary() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    put(&root, "6_Exp/a.png", "x");
    put(&root, "2_Log/Wiki/Archive/w.md", "![f](../../6_Exp/a.png)\n");
    let (_, out) = elf(&root, &["check"]);
    assert!(out.contains("fixed: 2_Log/Wiki/Archive/w.md: 1 link(s)"), "{out}");
    assert_eq!(read(&root, "2_Log/Wiki/Archive/w.md"), "![f](../../../6_Exp/a.png)\n");
}

/// 설정으로 끄면 알리기만 하고 파일은 그대로. check setup이 현재 값을 보임.
#[test]
fn fix_links_false_only_reports() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let cfg = root.join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    v["fix_links"] = serde_json::Value::Bool(false);
    fs::write(&cfg, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    put(&root, "6_Exp/a.png", "x");
    put(&root, "1_Concept/Archive/d.md", "![f](../6_Exp/a.png)\n");
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("1_Concept/Archive/d.md: 1 broken link(s) are one folder level off") && !out.contains("fixed:"), "{out}");
    assert_eq!(read(&root, "1_Concept/Archive/d.md"), "![f](../6_Exp/a.png)\n");
    let (_, setup) = elf(&root, &["check", "setup"]);
    assert!(setup.contains("link fix: off"), "{setup}");
}

/// 위와 아래 모두 찾아지면(판단 불가) 고치지 않음, 루트 밖으로 나가는 수정도 하지 않음 — 깨진 링크로 경고.
#[test]
fn ambiguous_or_outside_fixes_are_not_made() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 1_Concept/A/d.md의 `../x.png`: 위(`../../x.png` = 루트/x.png)와 아래(`x.png` = 1_Concept/A/x.png) 모두 있음
    put(&root, "x.png", "x");
    put(&root, "1_Concept/A/x.png", "x");
    put(&root, "1_Concept/A/d.md", "![f](../x.png)\n");
    // 루트 바로 아래 문서의 `../y.png` → 위로 고치면 루트 밖
    put(&root, "top.md", "![g](y.png) ![h](../../y.png)\n");
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(!out.contains("fixed:"), "{out}");
    assert!(out.contains("1_Concept/A/d.md: 1 broken link(s) → `../x.png`"), "{out}");
    assert!(out.contains("top.md: 2 broken link(s)"), "{out}");
    assert_eq!(read(&root, "1_Concept/A/d.md"), "![f](../x.png)\n");
}

/// 링크로 보지 않는 것: 한국어 문장의 `](`, URL, 앵커, 자리표시, 코드 안. `.elf/` 같은 점 폴더는 보지 않음.
#[test]
fn prose_and_code_are_not_links() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    put(
        &root,
        "2_Log/Wiki/notes.md",
        "Supp [6](supp_v3 2026-10-29 저장본), [15](27), [x](§17.9), [[S220]](재검사 결손)\n\
         [u](https://e.com/a.md) [h](#sec) [p](S{NNN}/f.png) `[c](code.png)`\n```\n[k](fence.png)\n```\n",
    );
    put(&root, ".hidden/doc.md", "![f](nope.png)\n");
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(!out.contains("notes.md") && !out.contains("doc.md"), "{out}");
}

/// 닫힌 로그: 그림 embed가 한 단계 어긋나면 고치고, 대상이 없는 것은 경고(문서별 한 줄). `.md` 상호 참조 깨짐은 issue.
#[test]
fn closed_log_links_are_fixed_and_broken_ones_warned() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    put(&root, "close.md", "---\nconclusion: '끝'\nhandoff: '완료; 미완료 = 없음'\n---\n");
    assert_eq!(elf(&root, &["session", "close", "S001", "--from", "close.md"]).0, 0);
    put(&root, "2_Log/6_Exp/old.png", "x"); // `6_Exp/old.png`(로그 폴더 기준)는 한 단계 위 `../6_Exp/old.png`에 있음
    let log = root.join("2_Log/S/S001_log.md");
    let mut text = fs::read_to_string(&log).unwrap();
    text.push_str("\n![f](6_Exp/old.png)\n![g](../6_Exp/gone.png)\n[s](S999_log.md)\n");
    fs::write(&log, &text).unwrap();
    let (code, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("fixed: S001_log.md: 1 link(s) one folder level (e.g. `6_Exp/old.png` → `../6_Exp/old.png`)"), "{out}");
    assert!(out.contains("S001_log.md: 1 broken link(s) → `../6_Exp/gone.png`"), "{out}");
    assert!(out.contains("issue: S001_log.md: broken cross-ref → S999_log.md") && code == 4, "{out}");
    assert!(read(&root, "2_Log/S/S001_log.md").contains("![f](../6_Exp/old.png)"));
}

/// 진행 중 로그: 생성 파일 표의 `../` 경로 한 단계 차이는 고치고, 프로젝트 밖 경로(환경 변수, 홈, 루트 탈출)는
/// 경고, 범위 줄임과 맨 이름 폴더는 검사하지 않음. Phase 2 쓰기는 알리기만 하고 고치지 않음.
#[test]
fn ongoing_log_files_table_fix_outside_and_false_positives() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    put(&root, "6_Exp/64_Viz/S001/r.png", "x");
    put(&root, "p1.md", P1);
    assert_eq!(elf(&root, &["trial", "new", "x", "--from", "p1.md"]).0, 0);
    let rows = "| Figure | `../6_Exp/64_Viz/S001/r.png` |\n\
                | Tmp | `%TEMP%/hook_bench.py`, `~/scratch/a.py`, `../../../../outside/b.py` |\n\
                | Range | `6_Exp/64_Viz/S001/r_p1~p5.png`, `supp_t11/` |\n";
    put(&root, "p2.md", &p2(rows, "![o](../../../../elsewhere/c.png)"));
    let (code, out) = elf(&root, &["trial", "write", "--from", "p2.md"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("`../6_Exp/64_Viz/S001/r.png`, which does not exist; `../../6_Exp/64_Viz/S001/r.png` does (one folder level)"), "{out}");
    for p in ["%TEMP%/hook_bench.py", "~/scratch/a.py", "../../../../outside/b.py"] {
        assert!(out.contains(&format!("`{p}`, which is outside the project")), "{p}: {out}");
    }
    assert!(out.contains("t01: 1 link(s) point outside the project"), "{out}");
    assert!(!out.contains("r_p1~p5") && !out.contains("supp_t11"), "range and bare folder are skipped: {out}");
    let log_rel = "2_Log/S/S001_log.md";
    assert!(read(&root, log_rel).contains("`../6_Exp/64_Viz/S001/r.png`"), "trial write never fixes");
    let (_, out) = elf(&root, &["validate"]);
    assert!(!out.contains("fixed:") && read(&root, log_rel).contains("`../6_Exp/64_Viz/S001/r.png`"), "the hidden alias stays read-only: {out}");
    let (_, out) = elf(&root, &["check", "logs"]);
    assert!(out.contains("fixed: S001_log.md: 1 link(s)"), "{out}");
    assert!(read(&root, log_rel).contains("`../../6_Exp/64_Viz/S001/r.png`"));
    assert!(out.contains("S001_log.md: 1 link(s) point outside the project → `../../../../elsewhere/c.png`"), "{out}");
}

/// 하위 폴더 프로젝트(S056 t02): 경계 = `.git`이 있는 가장 가까운 상위 폴더(없으면 프로젝트 루트). 같은 저장소의
/// 파일은 프로젝트 폴더 밖이어도 안으로 보고, 저장소가 없으면 프로젝트 루트 밖은 밖.
#[test]
fn nested_project_uses_the_repository_root_as_the_limit() {
    for with_git in [true, false] {
        let tmp = tempdir().unwrap();
        let repo = tmp.path();
        if with_git {
            fs::write(repo.join(".git"), "gitdir: elsewhere\n").unwrap(); // worktree처럼 파일이어도 됨
        }
        fs::create_dir_all(repo.join("proj")).unwrap();
        let root = new_project(&repo.join("proj"));
        put(repo, "code/src/a.rs", "x");
        put(&root, "p1.md", P1);
        assert_eq!(elf(&root, &["trial", "new", "x", "--from", "p1.md"]).0, 0);
        put(&root, "p2.md", &p2("| Code | `../../../../code/src/a.rs`, `../../../../../beyond/b.rs` |\n", "o"));
        let (_, out) = elf(&root, &["trial", "write", "--from", "p2.md"]);
        assert_eq!(
            out.contains("`../../../../code/src/a.rs`, which is outside the project"),
            !with_git,
            "with_git={with_git}: {out}"
        );
        assert!(out.contains("`../../../../../beyond/b.rs`, which is outside the project"), "beyond the repository: {out}");
    }
}
