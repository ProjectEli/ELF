//! t04: `elf validate` — Registry↔로그 정합·번호·cross-ref 검사, escalation·`--check` 게이트.

use assert_cmd::Command;
use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{SessionNewOptions, run_session_new};
use elf_cli::validate::{ValidateError, run_validate, run_validate_opts};
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

const REG: &str = "2_Log/Wiki/Session_Registry.tsv";

#[test]
fn clean_project_has_no_findings() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path()); // init = S001 로그 + 등록
    let r = run_validate(&root).unwrap();
    assert_eq!(r.issues, 0, "{:?}", r.lines);
    assert_eq!(r.warnings, 0, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("ok")), "{:?}", r.lines);
}

#[test]
fn unregistered_log_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005_log.md"), "# S005\n").unwrap(); // 등록 없는 로그
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S005") && l.contains("not in the registry")),
        "{:?}",
        r.lines
    );
    assert!(r.issues >= 1);
}

// S030: 세션 id 변형(접미사 등)은 규칙 문서가 아니라 CLI 게이트가 차단 — 비정격 `*_log.md`는
// 스캔 체계 밖으로 침묵 이탈하므로 validate가 issue로 표면화.
#[test]
fn malformed_session_log_name_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005-a_log.md"), "# S005-a\n").unwrap(); // 접미사 변형
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::write(root.join("2_Log/Archive/Sfoo_log.md"), "# bad\n").unwrap(); // 비숫자 변형
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("malformed session log name: 2_Log/S005-a_log.md")),
        "{:?}",
        r.lines
    );
    assert!(
        r.lines.iter().any(|l| l.contains("2_Log/Archive/Sfoo_log.md")),
        "{:?}",
        r.lines
    );
    assert!(r.issues >= 2);
    // 정격 로그(S001)는 비영향
    assert!(!r.lines.iter().any(|l| l.contains("malformed") && l.contains("S001_log.md")));
}

#[test]
fn phantom_registry_row_is_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let mut reg = fs::read_to_string(root.join(REG)).unwrap();
    reg.push_str("S002\t2026-06-13\tGhost\tComplete\t-\tArchive/S002_log.md\n"); // 로그 없는 행
    fs::write(root.join(REG), reg).unwrap();
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S002") && l.contains("no log file")),
        "{:?}",
        r.lines
    );
}

#[test]
fn gap_is_warning_not_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S003_log.md"), "# S003\n").unwrap();
    let mut reg = fs::read_to_string(root.join(REG)).unwrap();
    reg.push_str("S003\t2026-06-13\tThree\tComplete\t-\tArchive/S003_log.md\n");
    fs::write(root.join(REG), reg).unwrap();
    let r = run_validate(&root).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("gap") && l.contains("S002")), "{:?}", r.lines);
    assert_eq!(r.issues, 0, "gap는 warning이어야 함: {:?}", r.lines); // S003 등록+로그 → issue 아님
}

#[test]
fn multiple_active_is_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    run_session_new(&root, &SessionNewOptions { title: "Second".into(), date: "2026-06-13".into() })
        .unwrap(); // S002도 활성
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("multiple active") && l.contains("S001") && l.contains("S002")),
        "{:?}",
        r.lines
    );
    assert_eq!(r.issues, 0, "{:?}", r.lines); // 둘 다 등록+연속 → 정합 issue 없음
}

#[test]
fn broken_cross_ref_is_issue_valid_is_not() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("1_Concept")).unwrap();
    fs::write(root.join("1_Concept/Real.md"), "x").unwrap();
    let p = root.join("2_Log/S001_log.md");
    let mut c = fs::read_to_string(&p).unwrap();
    c.push_str("\nsee [plan](../1_Concept/Nope.md) and [ok](../1_Concept/Real.md)\n");
    fs::write(&p, c).unwrap();

    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("broken cross-ref") && l.contains("Nope.md")),
        "{:?}",
        r.lines
    );
    assert!(
        !r.lines.iter().any(|l| l.contains("Real.md")),
        "유효 링크는 미발견이어야 함: {:?}",
        r.lines
    );
}

#[test]
fn malformed_registry_escalates() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join(REG),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n",
    )
    .unwrap();
    match run_validate(&root) {
        Err(ValidateError::Escalation(e)) => {
            assert_eq!(e.line, 2);
            assert!(e.to_string().contains("agent-action:"));
        }
        _ => panic!("expected escalation"),
    }
}

// ── figure-embed (Figure-Embed Enforcement) ──────────

fn seed_fig(root: &Path, name: &str) {
    let viz = root.join("6_Exp/64_Viz/S001");
    fs::create_dir_all(&viz).unwrap();
    fs::write(viz.join(name), b"x").unwrap(); // figure 생성(내용 무관)
}

fn append_s001(root: &Path, extra: &str) {
    let p = root.join("2_Log/S001_log.md");
    let mut c = fs::read_to_string(&p).unwrap();
    c.push_str(extra);
    fs::write(&p, c).unwrap();
}

#[test]
fn unembedded_figure_is_warning_not_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    seed_fig(&root, "S001_fig.png"); // 생성만, 로그 미임베딩
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S001_fig.png") && l.contains("not embedded")),
        "{:?}",
        r.lines
    );
    assert_eq!(r.issues, 0, "embed 누락은 기본 warn: {:?}", r.lines);
    assert!(r.warnings >= 1);
}

#[test]
fn embedded_figure_has_no_finding() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    seed_fig(&root, "S001_fig.png");
    append_s001(&root, "\n### 관찰\n![Fig1: 축 설명](../6_Exp/64_Viz/S001/S001_fig.png)\n");
    let r = run_validate(&root).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("S001_fig.png")), "{:?}", r.lines);
}

#[test]
fn table_path_only_still_warns() {
    // 표에 경로만 기재(임베딩 아님) → 여전히 누락 경고 (S156 t04 갭 재현)
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    seed_fig(&root, "S001_fig.png");
    append_s001(&root, "\n| Figure | `../6_Exp/64_Viz/S001/S001_fig.png` |\n");
    let r = run_validate(&root).unwrap();
    assert!(
        r.lines.iter().any(|l| l.contains("S001_fig.png") && l.contains("not embedded")),
        "{:?}",
        r.lines
    );
}

#[test]
fn noembed_comment_suppresses_warning() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    seed_fig(&root, "S001_fig.png");
    append_s001(&root, "\n<!-- noembed: S001_fig.png -->\n");
    let r = run_validate(&root).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("S001_fig.png")), "{:?}", r.lines);
}

#[test]
fn strict_promotes_embed_miss_to_issue() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    seed_fig(&root, "S001_fig.png");
    let r = run_validate_opts(&root, true).unwrap();
    assert!(r.issues >= 1, "strict 시 embed 누락은 issue: {:?}", r.lines);
}

// ── e2e ──────────────────────────────────────────────

#[test]
fn e2e_validate_clean_exits_0() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0 issue"));
}

#[test]
fn e2e_validate_check_gates_with_exit_4() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(root.join("2_Log/S005_log.md"), "# S005\n").unwrap(); // 미등록 = issue
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate", "--check"])
        .assert()
        .failure()
        .code(4);
}

#[test]
fn e2e_validate_malformed_exits_5_with_marker() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(
        root.join(REG),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\tbad\n",
    )
    .unwrap();
    Command::cargo_bin("elf")
        .unwrap()
        .current_dir(&root)
        .args(["validate"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicates::str::contains("agent-action:"));
}

#[test]
fn deprecated_whole_log_satisfies_registry_and_numbering() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // S002: 전체 폐기 상태(Deprecated/에 로그, Registry Status Deprecated) — 번호는 보존, 구조 검사 대상 아님
    fs::create_dir_all(root.join("2_Log/Deprecated")).unwrap();
    fs::write(
        root.join("2_Log/Deprecated/S002_log.md"),
        "---\ndeprecated: 2026-10-08\nsource: 2_Log/S002_log.md\n---\n\n# S002: T\n\n> **Status**: Deprecated\n\n## t01: x\n\n### 결과 (bad)\n- 구조 검사 대상 아님\n",
    )
    .unwrap();
    let mut reg = fs::read_to_string(root.join(REG)).unwrap();
    reg.push_str("S002\t2026-06-13\tDep\tDeprecated\t-\tDeprecated/S002_log.md\n");
    fs::write(root.join(REG), reg).unwrap();
    let r = run_validate(&root).unwrap();
    assert_eq!(r.issues, 0, "{:?}", r.lines);
    assert!(!r.lines.iter().any(|l| l.contains("multiple active")), "{:?}", r.lines);
    assert!(!r.lines.iter().any(|l| l.contains("non-canonical")), "{:?}", r.lines);
    // 같은 번호가 루트에도 있으면 중복
    fs::write(root.join("2_Log/S002_log.md"), "# S002: dup\n").unwrap();
    let r = run_validate(&root).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("duplicate log number S002") && l.contains("Deprecated/")), "{:?}", r.lines);
}

#[test]
fn partial_file_is_neither_duplicate_nor_unregistered() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("2_Log/Deprecated")).unwrap();
    fs::write(root.join("2_Log/Deprecated/S001_log.partial.md"), "---\ndeprecated: partial\nsource: 2_Log/S001_log.md\n---\n").unwrap();
    fs::write(root.join("2_Log/Deprecated/S201-a_log.md"), "# bad\n").unwrap(); // 비정격 이름은 Deprecated/에서도 검출
    let r = run_validate(&root).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("duplicate")), "{:?}", r.lines);
    assert!(!r.lines.iter().any(|l| l.contains("not in the registry")), "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.contains("malformed session log name: 2_Log/Deprecated/S201-a_log.md")), "{:?}", r.lines);
}

#[test]
fn figure_embedded_in_partial_file_counts_for_the_session() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::create_dir_all(root.join("6_Exp/64_Viz/S001")).unwrap();
    fs::write(root.join("6_Exp/64_Viz/S001/a.png"), "png").unwrap();
    let r = run_validate(&root).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("figure 'a.png'")), "{:?}", r.lines);
    fs::create_dir_all(root.join("2_Log/Deprecated")).unwrap();
    fs::write(
        root.join("2_Log/Deprecated/S001_log.partial.md"),
        "---\ndeprecated: partial\nsource: 2_Log/S001_log.md\n---\n\n## t01: x\n> **Deprecated**: 2026-10-08 · S001-D01 · t01\n\n<!-- deprecated:begin S001-D01 -->\n![a](../../6_Exp/64_Viz/S001/a.png)\n<!-- deprecated:end S001-D01 -->\n",
    )
    .unwrap();
    let r = run_validate(&root).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("figure 'a.png'")), "{:?}", r.lines);
}
