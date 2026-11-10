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
    assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(args).current_dir(root).output().unwrap()
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
fn en_project_gets_english_agents() {
    let tmp = tempdir().unwrap();
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp.path(), &o).unwrap();
    assert!(fs::read_to_string(root.join("AGENTS.md")).unwrap().contains("Standing duties"));
    assert!(!root.join("AGENTS.en.md").exists());
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

// ── 영어 프로젝트 절 제목(P017, issue #9) ─────────────────────────

fn en_project(tmp: &Path) -> PathBuf {
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp, &o).unwrap();
    run_session_new(&root, &SessionNewOptions { title: "sweep".into(), date: "2026-11-09".into(), goal: "g".into(), gov: false }).unwrap();
    root
}

#[test]
fn en_project_writes_english_headings() {
    let tmp = tempdir().unwrap();
    let root = en_project(tmp.path());
    // 배포 틀 = 영어 variant
    let tpl = fs::read_to_string(root.join(".elf/managed/templates/trialTemplate.md")).unwrap();
    assert!(tpl.contains("### Conditions\n") && !tpl.contains("### 조건"), "{tpl}");
    let stpl = fs::read_to_string(root.join(".elf/managed/templates/sessionTemplate.md")).unwrap();
    assert!(stpl.contains("<!-- ELF norms"), "{stpl}");
    // session new = 영어 규범 주석
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(log.contains("# S001: sweep\n") && log.contains("<!-- ELF norms") && !log.contains("ELF 규범"), "{log}");
    // trial new(틀만) = 영어 제목과 절
    let out = elf(&root, &["trial", "new"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("fill Phase 1 (Goal/Conditions/Hypothesis/Prediction)"));
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(log.contains("## t01: [task title]") && log.contains("### Prediction\n") && !log.contains("### 예상"), "{log}");
    // 빈 틀은 check logs가 영어 이름으로 알림
    let out = elf(&root, &["check", "logs"]);
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("template placeholder '[task title]'") && so.contains("'### Goal' still holds only the template text"), "{so}");
}

#[test]
fn en_project_normalizes_korean_headings_and_label() {
    let tmp = tempdir().unwrap();
    let root = en_project(tmp.path());
    // 한국어 표기로 써도 영어 표기로 바로잡음(두 표기 = 같은 절)
    fs::write(root.join("p1.md"), P1).unwrap();
    let out = elf(&root, &["trial", "new", "sweep", "--from", "p1.md"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("heading '### 조건 (Conditions)' → '### Conditions'"), "{so}");
    assert!(so.contains("list expected figures in Prediction, and embed each into `### Observation`"), "{so}");
    fs::write(root.join("p2.md"), P2).unwrap();
    let out = elf(&root, &["trial", "write", "--from", "p2.md"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    for h in ["### Goal\n", "### Conditions\n", "### Hypothesis\n", "### Prediction\n", "### Observation\n", "### Interpretation\n", "### Lessons\n", "### Files\n"] {
        assert!(log.contains(h), "missing {h:?}: {log}");
    }
    assert!(log.contains("- Prediction match: 일치") && !log.contains("### 관찰"), "{log}");
    // 영어 표기 로그에 check logs 경고 없음
    let out = elf(&root, &["check", "logs"]);
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("0 issue(s), 0 warning(s)"), "{so}");
}

#[test]
fn ko_project_still_writes_korean_headings_and_accepts_english() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let en = "### Goal\n- a\n\n### Conditions\n- b\n\n### Hypothesis\n- c\n\n### Prediction\n- d\n";
    fs::write(root.join("p1.md"), en).unwrap();
    assert!(elf(&root, &["trial", "new", "x", "--from", "p1.md"]).status.success());
    fs::write(root.join("p2.md"), "### Observation\n- o\n\n### Interpretation\n- Prediction match: match\n\n### Lessons\n- l\n\n### Files\n- f\n").unwrap();
    assert!(elf(&root, &["trial", "write", "--from", "p2.md"]).status.success());
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(log.contains("### 목표 (Goal)\n- a") && log.contains("### 생성 파일 (Files)\n- f") && log.contains("- 예상 일치 정도: match"), "{log}");
    assert!(log.contains("<!-- ELF 규범"), "{log}");
}

#[test]
fn update_moves_en_project_to_english_templates() {
    let tmp = tempdir().unwrap();
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp.path(), &o).unwrap();
    // 3.3 이전 상태 재현: 배포 파일과 stamp가 한국어 틀
    let ko = elf_cli::embed::TEMPLATES.get_file("log/trialTemplate.md").unwrap().contents_utf8().unwrap();
    fs::write(root.join(".elf/managed/templates/trialTemplate.md"), ko).unwrap();
    let stamp_path = root.join(".elf/manifest.json");
    let stamp = fs::read_to_string(&stamp_path).unwrap();
    let ko_sha = elf_cli::hash::sha256_lf(ko.as_bytes());
    let line = stamp.lines().find(|l| l.contains("\"src\": \"templates/log/trialTemplate.en.md\"")).unwrap().to_string();
    let en_sha = line.split("\"sha256\": \"").nth(1).unwrap().split('"').next().unwrap().to_string();
    let old = line.replace("trialTemplate.en.md", "trialTemplate.md").replace(&en_sha, &ko_sha).replace(", \"lang\": \"en\", \"role\": \"variant\"", "");
    fs::write(&stamp_path, stamp.replace(&line, &old)).unwrap();
    let rep = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(rep.conflicts, 0, "{:?}", rep.lines);
    let tpl = fs::read_to_string(root.join(".elf/managed/templates/trialTemplate.md")).unwrap();
    assert!(tpl.contains("### Conditions\n") && !tpl.contains("### 조건"), "{:?}", rep.lines);
}

// ── 영어 프로젝트 한국어 완전 대체(v3.4, S050 t02) ─────────────────

fn has_hangul(t: &str) -> bool {
    t.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

fn hangul_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if p.is_dir() {
                walk(&p, root, out);
            } else if let Ok(t) = fs::read_to_string(&p)
                && has_hangul(&t)
            {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn en_project_commands_write_and_print_no_korean() {
    let tmp = tempdir().unwrap();
    let root = en_project(tmp.path());
    fs::write(root.join("p1.md"), "### Goal\n- a\n\n### Conditions\n- b\n\n### Hypothesis\n- c\n\n### Prediction\n- d\n").unwrap();
    fs::write(root.join("p2.md"), "### Observation\n- o\n\n### Interpretation\n- Prediction match: match\n\n### Lessons\n- l\n\n### Files\n- f\n").unwrap();
    fs::write(root.join("close.md"), "---\nconclusion: 'done'\n---\n").unwrap();
    let mut outputs = String::new();
    for args in [
        vec!["trial", "new", "first", "--from", "p1.md"],
        vec!["trial", "write", "--from", "p2.md"],
        vec!["trial", "new", "second", "--from", "p1.md"],
        vec!["deprecate", "2_Log/S/S001_log.md", "--trial", "t02", "--reason", "wrong input", "--replaced-by", "t03"],
        vec!["deprecate", "list"],
        vec!["deprecate", "show", "S001-D01"],
        vec!["check"],
        vec!["autoread"],
        vec!["session", "close", "S001"],
        vec!["session", "close", "S001", "--from", "close.md"],
        vec!["log", "show", "S001"],
    ] {
        let out = elf(&root, &args);
        outputs.push_str(&format!("$ elf {}\n{}{}", args.join(" "), String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
    }
    let log = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(log.contains("## t02: (deprecated S001-D01)") && log.contains("· reason: wrong input · replaced by: t03"), "{log}");
    for f in ["p1.md", "p2.md", "close.md"] {
        fs::remove_file(root.join(f)).unwrap();
    }
    assert_eq!(hangul_files(&root), Vec::<String>::new());
    let bad: Vec<&str> = outputs.lines().filter(|l| has_hangul(l)).collect();
    assert!(bad.is_empty(), "Korean in output: {bad:?}\n{outputs}");
}

#[test]
fn en_deprecation_restores_and_korean_markers_still_restore() {
    let tmp = tempdir().unwrap();
    let root = en_project(tmp.path());
    fs::write(root.join("p1.md"), "### Goal\n- a\n\n### Conditions\n- b\n\n### Hypothesis\n- c\n\n### Prediction\n- d\n").unwrap();
    assert!(elf(&root, &["trial", "new", "first", "--from", "p1.md"]).status.success());
    let before = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(elf(&root, &["deprecate", "2_Log/S/S001_log.md", "--trial", "t01", "--reason", "x"]).status.success());
    assert!(fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap().contains("## t01: (deprecated S001-D01)"));
    let out = elf(&root, &["deprecate", "--restore", "S001-D01"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let after = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(after.contains("## t01: first") && !after.contains("(deprecated"), "{after}");
    assert_eq!(after.lines().filter(|l| l.starts_with("### ")).count(), before.lines().filter(|l| l.starts_with("### ")).count());
    // 3.4 이전에 한국어로 쓴 흔적 제목도 복원(영어 프로젝트에서)
    assert!(elf(&root, &["deprecate", "2_Log/S/S001_log.md", "--trial", "t01"]).status.success());
    let t = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    let id = t.split("(deprecated ").nth(1).unwrap().split(')').next().unwrap().to_string();
    fs::write(root.join("2_Log/S/S001_log.md"), t.replace(&format!("(deprecated {id})"), &format!("(폐기 {id})"))).unwrap();
    let out = elf(&root, &["deprecate", "--restore", &id]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let t = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(t.contains("## t01: first") && !t.contains(&id), "{t}");
}

/// 3.3 영어 프로젝트(한국어 정본 + 영어 companion)를 `elf update`로 옮김: 정본이 영어로, companion 제거, 충돌 0.
#[test]
fn update_replaces_korean_rules_and_companions_in_en_project() {
    let tmp = tempdir().unwrap();
    let mut o = init_opts();
    o.lang = "en-US".into();
    let root = run_init(tmp.path(), &o).unwrap();
    let tpl = |src: &str| elf_cli::embed::TEMPLATES.get_file(src).unwrap().contents_utf8().unwrap().to_string();
    let sha = |t: &str| elf_cli::hash::sha256_lf(t.as_bytes());
    // (영어 src, 3.3의 한국어 src, 정본 dest, 3.3 companion dest)
    let pairs = [
        ("templates/meta/EliRule.en.md", "templates/meta/EliRule.md", ".elf/managed/EliRule.md", ".elf/managed/EliRule.en.md"),
        ("templates/meta/LogConvention.en.md", "templates/meta/LogConvention.md", ".elf/managed/LogConvention.md", ".elf/managed/LogConvention.en.md"),
        ("templates/root/AGENTS.en.md", "templates/root/AGENTS.md", "AGENTS.md", "AGENTS.en.md"),
    ];
    let stamp_path = root.join(".elf/manifest.json");
    let mut stamp: serde_json::Value = serde_json::from_str(&fs::read_to_string(&stamp_path).unwrap()).unwrap();
    let files = stamp["files"].as_array_mut().unwrap();
    for (en_src, ko_src, dest, comp) in pairs {
        let ko = tpl(ko_src.trim_start_matches("templates/"));
        let en = tpl(en_src.trim_start_matches("templates/"));
        fs::write(root.join(dest), &ko).unwrap();
        fs::write(root.join(comp), &en).unwrap();
        // stamp = 정본 manifest 전체: 한국어 base 항목은 그대로, 영어 variant 항목을 3.3의 companion으로
        assert_eq!(files.iter().find(|e| e["dest"] == dest && e.get("role").is_none()).unwrap()["src"], ko_src);
        let e = files.iter_mut().find(|e| e["dest"] == dest && e["role"] == "variant").unwrap();
        *e = serde_json::json!({ "src": en_src, "dest": comp, "tier": "managed", "sha256": sha(&en), "lang": "en", "role": "companion" });
    }
    fs::write(&stamp_path, serde_json::to_string_pretty(&stamp).unwrap()).unwrap();
    let rep = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(rep.conflicts, 0, "{:?}", rep.lines);
    for (en_src, _, dest, comp) in pairs {
        assert_eq!(fs::read_to_string(root.join(dest)).unwrap(), tpl(en_src.trim_start_matches("templates/")), "{dest}: {:?}", rep.lines);
        assert!(!root.join(comp).exists(), "{comp} removed: {:?}", rep.lines);
    }
}
