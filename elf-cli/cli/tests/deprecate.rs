//! S038 t04 통합: `elf deprecate` — 단위별 폐기·복원 왕복, 조작 후 구조 검사 거부, 전체 폐기의 Registry·헤더,
//! 부분+전체 공존, list 대조, 종료(close) 후 이동 표기 링크, 계획 문서 블록, 거부 조건·exit code.

use assert_cmd::Command;
use elf_cli::deprecate::{DeprecateError, DeprecateOptions, run_deprecate, run_list, run_restore};
use elf_cli::init::{InitOptions, run_init};
use elf_cli::session::{CloseOptions, SessionNewOptions, run_session_close, run_session_new};
use elf_cli::validate::trial_structure_findings;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const REG: &str = "2_Log/Wiki/Session_Registry.tsv";
const DATE: &str = "2026-10-07";
const BEGIN: &str = "<!-- deprecate:begin -->";
const END: &str = "<!-- deprecate:end -->";

fn header(sid: &str, title: &str, status: &str, rel: &str, handoff: &str) -> String {
    format!(
        "# {sid}: {title}\n\n> **Created**: 2026-10-01\\\n> **Modified**: 2026-10-02\\\n> **Status**: {status}\\\n> **목표**: {title}\\\n> **관련**: {rel}\\\n> **Handoff**: {handoff}\n\n---\n\n"
    )
}

fn trial(tid: &str, title: &str, sid: &str, fig: &str, obs_extra: &str) -> String {
    format!(
        "## {tid}: {title}\n\n### 목표 (Goal)\n- {title} 응답 측정\n\n### 조건 (Conditions)\n- fc = 1 kHz\n\n### 가설 (Hypothesis)\n- 차수에 비례한 감쇠\n\n### 예상 (Prediction)\n- {fig} 산출\n\n### 관찰 (Observation)\n- 측정 기울기 기록\n- ![{fig}: 주파수 응답](../6_Exp/64_Viz/{sid}/{sid}_{fig}.png)\n{obs_extra}\n### 해석 (Interpretation)\n- 가설 적중 여부: 적중\n- 공차 범위 내 편차\n\n### 교훈 (Lessons)\n- 케이블 길이 영향 확인\n\n### 생성 파일 (Files)\n\n| 유형 | 파일 |\n|------|------|\n| Script | `6_Exp/63_Analysis/Scripts/{sid}_{tid}.m` |\n| Figure | `6_Exp/64_Viz/{sid}/{sid}_{fig}.png` |\n"
    )
}

fn w(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn r(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap()
}

/// t02 시험용 프로젝트 재현: S001(Archive·2 trial) · S002(루트·3 trial) · S003(루트·1 trial) · P001 · 더미 figure
fn fixture(tmp: &Path) -> PathBuf {
    let root = run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-10-01".into(),
        },
    )
    .unwrap();
    for (sid, figs) in [("S001", vec!["fig1"]), ("S002", vec!["fig1", "fig2", "fig3"]), ("S003", vec!["fig1"])] {
        for f in figs {
            w(&root, &format!("6_Exp/64_Viz/{sid}/{sid}_{f}.png"), "png");
        }
        for t in ["t01", "t02", "t03"] {
            w(&root, &format!("6_Exp/63_Analysis/Scripts/{sid}_{t}.m"), "% dummy\n");
        }
    }
    w(&root, "2_Log/Wiki/Filter_Notes.md", "# Filter Notes\n\n- 4차 필터 채택 근거: S002 t02\n");
    w(
        &root,
        "1_Concept/12_Planning/P001_plan.md",
        "# P001: 필터 평가 계획\n\n> **Status**: 진행\n\n---\n\n## 1. 배경\n\n- 차수별 응답 비교 필요\n\n## 2. 평가 절차\n\n- 2차 → 4차 순서로 측정\n\n초기안: 6차 필터까지 확대 측정.\n\n- 6차는 [S002](../../2_Log/S002_log.md) 결과 확인 후 결정\n- 참고 그림 ![2차 응답](../../6_Exp/64_Viz/S002/S002_fig1.png)\n\n## 3. 일정\n\n- 10월 1주: 측정\n",
    );
    // S001: 작성 후 실제 close → Archive
    let s001 = header("S001", "기준 측정", "★ 활성", "[P001](../1_Concept/12_Planning/P001_plan.md)", "기준 응답 확보; 미완료 -; 참조 t01")
        + &trial("t01", "1차 필터", "S001", "fig1", "")
        + "\n---\n\n"
        + &trial("t02", "1차 필터 재측정", "S001", "fig1", "- 계획: [P001](../1_Concept/12_Planning/P001_plan.md)\n");
    w(&root, "2_Log/S001_log.md", &s001);
    run_session_close(&root, &CloseOptions { id: Some("S001".into()), force: false }).unwrap();
    // S002 · S003
    run_session_new(&root, &SessionNewOptions { title: "필터 차수 비교".into(), date: "2026-10-02".into() }).unwrap();
    let s002 = header("S002", "필터 차수 비교", "★ 활성", "S001 · [P001](../1_Concept/12_Planning/P001_plan.md)", "t02 4차 채택·t03 재현 확인; 미완료 = t03 조건 확대; 참조 t02 관찰")
        + &trial("t01", "2차 필터", "S002", "fig1", "")
        + "\n---\n\n"
        + &trial("t02", "4차 필터", "S002", "fig2", "- 기준: [S001](Archive/S001_log.md) · 계획: [P001](../1_Concept/12_Planning/P001_plan.md) · 요약: [Notes](Wiki/Filter_Notes.md)\n")
        + "\n---\n\n"
        + &trial("t03", "재현 측정", "S002", "fig3", "\n재현 측정에서 t02와 같은 기울기 확인.\n\n| 회차 | 기울기 |\n|---|---|\n| 1 | -78 dB/dec |\n| 2 | -79 dB/dec |\n");
    w(&root, "2_Log/S002_log.md", &s002);
    run_session_new(&root, &SessionNewOptions { title: "전체 폐기 시험".into(), date: "2026-10-03".into() }).unwrap();
    let s003 = header("S003", "전체 폐기 시험", "★ 활성", "[S002](S002_log.md)", "-")
        + &trial("t01", "잘못된 조건", "S003", "fig1", "- 비교: [S002](S002_log.md) · [S001](Archive/S001_log.md)\n");
    w(&root, "2_Log/S003_log.md", &s003);
    root
}

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, root, out);
            } else {
                out.insert(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"), fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn opts(target: &str) -> DeprecateOptions {
    DeprecateOptions { target: target.into(), date: DATE.into(), ..Default::default() }
}

fn trial_opts(target: &str, tid: &str, section: Option<&str>) -> DeprecateOptions {
    DeprecateOptions { trial: Some(tid.into()), section: section.map(str::to_string), ..opts(target) }
}

fn assert_same_tree(a: &BTreeMap<String, Vec<u8>>, b: &BTreeMap<String, Vec<u8>>) {
    let diff: Vec<&String> = a.keys().chain(b.keys()).filter(|k| a.get(*k) != b.get(*k)).collect();
    assert!(diff.is_empty(), "tree differs at {diff:?}");
}

#[test]
fn trial_unit_moves_body_keeps_heading_and_roundtrips() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);

    let mut o = trial_opts("S002", "t02", None);
    o.replaced_by = Some("S002 t03".into());
    o.reason = Some("측정 조건 오류".into());
    let rep = run_deprecate(&root, &o).unwrap();
    assert!(rep.changed);
    assert_eq!(rep.lines[0], "deprecated S002-D01 (t02) → 2_Log/Deprecated/S002_log.partial.md");
    assert!(rep.lines.iter().any(|l| l == "review: handoff: 2_Log/S002_log.md:8 mentions t02"), "{:?}", rep.lines);
    assert!(rep.lines.iter().any(|l| l == "review: ref: 2_Log/Wiki/Filter_Notes.md:3 refers to S002 t02"), "{:?}", rep.lines);
    assert!(rep.lines.iter().any(|l| l.starts_with("review: figure: ../6_Exp/64_Viz/S002/S002_fig2.png")), "{:?}", rep.lines);
    assert!(rep.lines.iter().any(|l| l.contains("S002_t02.m is listed")), "{:?}", rep.lines);
    assert!(!rep.lines.iter().any(|l| l.starts_with("warn:")), "{:?}", rep.lines);

    let origin = r(&root, "2_Log/S002_log.md");
    assert!(origin.contains("## t02: 4차 필터\n\n> **Deprecated** → [Deprecated/S002_log.partial.md](Deprecated/S002_log.partial.md) (S002-D01 · t02 · 2026-10-07 · replaced_by: S002 t03)\n\n---\n\n## t03:"), "{origin}");
    assert!(!origin.contains("S002_fig2.png"));
    let cont = r(&root, "2_Log/Deprecated/S002_log.partial.md");
    assert!(cont.starts_with("---\ndeprecated: partial\nsource: 2_Log/S002_log.md\n---\n\n# S002: 필터 차수 비교\n\n## t02: 4차 필터\n> **Deprecated**: 2026-10-07 · S002-D01 · t02 · replaced_by: S002 t03 · reason: 측정 조건 오류\n\n<!-- deprecated:begin S002-D01 -->\n### 목표 (Goal)"), "{cont}");
    assert!(cont.ends_with("<!-- deprecated:end S002-D01 -->\n"));
    // 링크 재계산: 그림·`../`·같은 폴더·하위 폴더 전부 Deprecated/ 기준으로 유효
    for t in ["../../6_Exp/64_Viz/S002/S002_fig2.png", "../Archive/S001_log.md", "../../1_Concept/12_Planning/P001_plan.md", "../Wiki/Filter_Notes.md"] {
        assert!(cont.contains(&format!("]({t})")), "{cont}");
        assert!(root.join("2_Log/Deprecated").join(t).exists(), "{t}");
    }
    // trial 번호 유지
    assert!(trial_structure_findings(&origin).is_empty());

    let rep = run_restore(&root, "S002-D01").unwrap();
    assert_eq!(rep.lines[0], "restored S002-D01 → 2_Log/S002_log.md (t02)");
    assert!(!root.join("2_Log/Deprecated/S002_log.partial.md").exists());
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn section_unit_moves_heading_and_leaves_structure_clean() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    run_deprecate(&root, &trial_opts("S002", "t01", Some("해석"))).unwrap();
    let origin = r(&root, "2_Log/S002_log.md");
    let t01 = &origin[origin.find("## t01:").unwrap()..origin.find("## t02:").unwrap()];
    assert!(!t01.contains("### 해석"), "{t01}");
    assert!(t01.contains("> **Deprecated** → [Deprecated/S002_log.partial.md](Deprecated/S002_log.partial.md) (S002-D01 · t01 해석 · 2026-10-07)\n\n### 교훈 (Lessons)"), "{t01}");
    assert!(trial_structure_findings(&origin).is_empty(), "{:?}", trial_structure_findings(&origin));
    let cont = r(&root, "2_Log/Deprecated/S002_log.partial.md");
    assert!(cont.contains("<!-- deprecated:begin S002-D01 -->\n### 해석 (Interpretation)\n- 가설 적중 여부: 적중\n- 공차 범위 내 편차\n<!-- deprecated:end S002-D01 -->"), "{cont}");
    run_restore(&root, "S002-D01").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn guard_refuses_moves_that_introduce_structure_findings() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    // Phase 1 절 단독 폐기(관찰이 남음) → 거부
    let e = run_deprecate(&root, &trial_opts("S002", "t01", Some("가설"))).unwrap_err();
    assert!(matches!(&e, DeprecateError::Refuse(m) if m.contains("missing '### 가설'")), "{e}");
    // 판정 줄을 포함한 표시 블록 → 거부 (표시 줄 자체는 조작 전 기준에서 제외됨)
    let p = root.join("2_Log/S002_log.md");
    let t = fs::read_to_string(&p).unwrap().replacen(
        "- 가설 적중 여부: 적중\n- 공차 범위 내 편차\n",
        &format!("{BEGIN}\n- 가설 적중 여부: 적중\n{END}\n- 공차 범위 내 편차\n"),
        1,
    );
    fs::write(&p, &t).unwrap();
    let e = run_deprecate(&root, &DeprecateOptions { marked: true, ..opts("S002") }).unwrap_err();
    assert!(matches!(&e, DeprecateError::Refuse(m) if m.contains("must start with")), "{e}");
    fs::write(&p, s0["2_Log/S002_log.md"].clone()).unwrap();
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn marked_and_lines_units_roundtrip() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    let p = root.join("2_Log/S002_log.md");
    let t = fs::read_to_string(&p).unwrap();
    let a = "- ![fig3: 주파수 응답](../6_Exp/64_Viz/S002/S002_fig3.png)\n";
    let b = "| 2 | -79 dB/dec |\n";
    fs::write(&p, t.replacen(a, &format!("{BEGIN}\n{a}"), 1).replacen(b, &format!("{b}{END}\n"), 1)).unwrap();
    let rep = run_deprecate(&root, &DeprecateOptions { marked: true, ..opts("S002") }).unwrap();
    assert_eq!(rep.lines[0], "deprecated S002-D01 (t03 관찰 part) → 2_Log/Deprecated/S002_log.partial.md");
    let origin = r(&root, "2_Log/S002_log.md");
    assert!(!origin.contains(BEGIN) && !origin.contains(END) && !origin.contains("-79 dB/dec"));
    let cont = r(&root, "2_Log/Deprecated/S002_log.partial.md");
    assert!(cont.contains("## t03: 재현 측정\n> **Deprecated**: 2026-10-07 · S002-D01 · t03 관찰 part\n\n<!-- deprecated:begin S002-D01 -->\n- ![fig3: 주파수 응답](../../6_Exp/64_Viz/S002/S002_fig3.png)\n"), "{cont}");
    assert!(cont.contains("| 2 | -79 dB/dec |\n<!-- deprecated:end S002-D01 -->"), "{cont}");
    run_restore(&root, "S002-D01").unwrap();
    assert_same_tree(&snapshot(&root), &s0); // 표시 줄은 복원되지 않음 = 표시 전 원문

    // 줄 범위 + --expect
    let lines: Vec<&str> = t.split('\n').collect();
    let la = lines.iter().position(|l| l.starts_with("재현 측정에서")).unwrap() + 1;
    let lb = lines.iter().position(|l| l.starts_with("| 2 | -79")).unwrap() + 1;
    let o = DeprecateOptions { lines: Some((la, lb)), expect: Some(lines[la - 1].into()), ..opts("S002") };
    run_deprecate(&root, &o).unwrap();
    assert!(!r(&root, "2_Log/S002_log.md").contains("-79 dB/dec"));
    run_restore(&root, "S002-D01").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
    let bad = DeprecateOptions { lines: Some((la, lb)), expect: Some("다른 줄".into()), ..opts("S002") };
    assert!(matches!(run_deprecate(&root, &bad).unwrap_err(), DeprecateError::Refuse(m) if m.contains("--expect mismatch")));
    assert!(matches!(run_deprecate(&root, &DeprecateOptions { lines: Some((la, lb)), ..opts("S002") }).unwrap_err(), DeprecateError::Refuse(_)));
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn archive_origin_links_parent_and_keeps_same_depth_links() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    run_deprecate(&root, &trial_opts("S001", "t02", None)).unwrap();
    let origin = r(&root, "2_Log/Archive/S001_log.md");
    assert!(origin.contains("> **Deprecated** → [Deprecated/S001_log.partial.md](../Deprecated/S001_log.partial.md) (S001-D01 · t02 · 2026-10-07)"), "{origin}");
    let cont = r(&root, "2_Log/Deprecated/S001_log.partial.md");
    assert!(cont.contains("](../../6_Exp/64_Viz/S001/S001_fig1.png)") && cont.contains("](../../1_Concept/12_Planning/P001_plan.md)"), "{cont}");
    run_restore(&root, "S001-D01").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn whole_document_moves_with_yaml_header_status_and_registry() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    let mut o = opts("S003");
    o.replaced_by = Some("S002".into());
    o.reason = Some("조건 설정 \"오류\"".into());
    let rep = run_deprecate(&root, &o).unwrap();
    assert_eq!(rep.lines[0], "deprecated S003 (whole) → 2_Log/Deprecated/S003_log.md");
    assert!(!root.join("2_Log/S003_log.md").exists());
    let dep = r(&root, "2_Log/Deprecated/S003_log.md");
    assert!(dep.starts_with("---\ndeprecated: 2026-10-07\nsource: 2_Log/S003_log.md\nreplaced_by: \"S002\"\nreason: \"조건 설정 \\\"오류\\\"\"\nstatus_before: \"★ 활성\"\n---\n\n# S003: 전체 폐기 시험\n"), "{dep}");
    assert!(dep.contains("> **Status**: Deprecated\\\n"), "{dep}");
    assert!(dep.contains("[S002](../S002_log.md)") && dep.contains("[S001](../Archive/S001_log.md)") && dep.contains("](../../6_Exp/64_Viz/S003/S003_fig1.png)"), "{dep}");
    let reg = r(&root, REG);
    assert!(reg.lines().any(|l| l.starts_with("S003\t") && l.ends_with("\tDeprecated\t-\tDeprecated/S003_log.md")), "{reg}");

    let rep = run_restore(&root, "S003").unwrap();
    assert_eq!(rep.lines[0], "restored S003 → 2_Log/S003_log.md (whole)");
    assert!(rep.lines.iter().any(|l| l == "registry: S003 → ★ 활성, -"), "{:?}", rep.lines);
    assert_same_tree(&snapshot(&root), &s0);

    // Archive 로그 전체 → 복원 시 Registry는 Complete·Archive 경로
    run_deprecate(&root, &opts("S001")).unwrap();
    assert!(r(&root, REG).lines().any(|l| l.starts_with("S001\t") && l.contains("\tDeprecated\t")));
    run_restore(&root, "S001").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn partial_and_whole_coexist_and_restore_in_either_order() {
    for whole_first in [true, false] {
        let tmp = tempdir().unwrap();
        let root = fixture(tmp.path());
        let s0 = snapshot(&root);
        run_deprecate(&root, &trial_opts("S002", "t02", None)).unwrap();
        run_deprecate(&root, &opts("S002")).unwrap();
        assert!(root.join("2_Log/Deprecated/S002_log.md").is_file() && root.join("2_Log/Deprecated/S002_log.partial.md").is_file());
        let whole = r(&root, "2_Log/Deprecated/S002_log.md");
        assert!(whole.contains("[Deprecated/S002_log.partial.md](S002_log.partial.md) (S002-D01 ·"), "{whole}");
        if whole_first {
            run_restore(&root, "S002").unwrap();
            run_restore(&root, "S002-D01").unwrap();
        } else {
            run_restore(&root, "S002-D01").unwrap();
            run_restore(&root, "S002").unwrap();
        }
        assert_same_tree(&snapshot(&root), &s0);
    }
}

#[test]
fn list_reports_entries_and_marker_block_mismatches() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    assert_eq!(run_list(&root).unwrap().lines, vec!["deprecated: 0 document(s), 0 block(s)"]);
    run_deprecate(&root, &trial_opts("S002", "t01", Some("교훈"))).unwrap();
    run_deprecate(&root, &trial_opts("S002", "t02", None)).unwrap();
    run_deprecate(&root, &opts("S003")).unwrap();
    let rep = run_list(&root).unwrap();
    assert_eq!(
        rep.lines,
        vec![
            "S002-D01 · 2026-10-07 · t01 교훈 · 2_Log/Deprecated/S002_log.partial.md",
            "S002-D02 · 2026-10-07 · t02 · 2_Log/Deprecated/S002_log.partial.md",
            "S003 · 2026-10-07 · whole · source: 2_Log/S003_log.md · 2_Log/Deprecated/S003_log.md",
            "deprecated: 1 document(s), 2 block(s)",
        ]
    );
    let p = root.join("2_Log/S002_log.md");
    let good = fs::read_to_string(&p).unwrap();
    fs::write(&p, good.replacen("(S002-D02 ·", "(S002-D99 ·", 1)).unwrap();
    let rep = run_list(&root).unwrap();
    assert!(rep.lines.contains(&"warn: S002-D02: no move marker in 2_Log/S002_log.md".to_string()), "{:?}", rep.lines);
    assert!(rep.lines.contains(&"warn: S002-D99: move marker in 2_Log/S002_log.md has no block in 2_Log/Deprecated/S002_log.partial.md".to_string()), "{:?}", rep.lines);
}

#[test]
fn refusals_and_not_found_leave_files_untouched() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    assert!(matches!(run_deprecate(&root, &opts("0_Meta/ProjectRule.md")).unwrap_err(), DeprecateError::Refuse(m) if m.contains("out of scope")));
    assert!(matches!(run_deprecate(&root, &opts("S099")).unwrap_err(), DeprecateError::NotFound(_)));
    assert!(matches!(run_deprecate(&root, &trial_opts("P001", "t01", None)).unwrap_err(), DeprecateError::Refuse(m) if m.contains("session logs only")));
    assert!(matches!(run_deprecate(&root, &trial_opts("S002", "t09", None)).unwrap_err(), DeprecateError::NotFound(_)));
    assert!(matches!(run_deprecate(&root, &DeprecateOptions { marked: true, ..opts("S002") }).unwrap_err(), DeprecateError::Refuse(m) if m.contains("no <!-- deprecate:begin")));
    assert!(matches!(run_restore(&root, "S002-D07").unwrap_err(), DeprecateError::NotFound(_)));
    assert!(matches!(run_restore(&root, "S002").unwrap_err(), DeprecateError::Refuse(m) if m.contains("not deprecated")));
    // 표시 짝 불일치 · trial 헤딩 포함 블록
    let p = root.join("2_Log/S002_log.md");
    let t = fs::read_to_string(&p).unwrap();
    fs::write(&p, t.replacen("- 측정 기울기 기록\n", &format!("{BEGIN}\n- 측정 기울기 기록\n"), 1)).unwrap();
    assert!(matches!(run_deprecate(&root, &DeprecateOptions { marked: true, ..opts("S002") }).unwrap_err(), DeprecateError::Refuse(m) if m.contains("without")));
    fs::write(&p, t.replacen("---\n\n## t02:", &format!("---\n\n{BEGIN}\n## t02:"), 1).replacen("### 목표 (Goal)\n- 4차 필터 응답 측정\n", &format!("### 목표 (Goal)\n- 4차 필터 응답 측정\n{END}\n"), 1)).unwrap();
    assert!(matches!(run_deprecate(&root, &DeprecateOptions { marked: true, ..opts("S002") }).unwrap_err(), DeprecateError::Refuse(m) if m.contains("trial heading")));
    fs::write(&p, &t).unwrap();
    // 이미 폐기된 문서 재폐기 · 단위 중복 지정
    run_deprecate(&root, &opts("S003")).unwrap();
    assert!(matches!(run_deprecate(&root, &opts("S003")).unwrap_err(), DeprecateError::Refuse(m) if m.contains("already deprecated")));
    run_restore(&root, "S003").unwrap();
    assert!(matches!(run_deprecate(&root, &DeprecateOptions { marked: true, ..trial_opts("S002", "t01", None) }).unwrap_err(), DeprecateError::Refuse(m) if m.contains("one unit")));
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn planning_doc_marked_block_roundtrips_with_context_heading() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    let p = root.join("1_Concept/12_Planning/P001_plan.md");
    let t = fs::read_to_string(&p).unwrap();
    let a = "초기안: 6차 필터까지 확대 측정.\n";
    let b = "- 참고 그림 ![2차 응답](../../6_Exp/64_Viz/S002/S002_fig1.png)\n";
    fs::write(&p, t.replacen(a, &format!("{BEGIN}\n{a}"), 1).replacen(b, &format!("{b}{END}\n"), 1)).unwrap();
    let rep = run_deprecate(&root, &DeprecateOptions { marked: true, reason: Some("범위 축소".into()), ..opts("P001") }).unwrap();
    assert_eq!(rep.lines[0], "deprecated P001-D01 (part) → 1_Concept/12_Planning/Deprecated/P001_plan.partial.md");
    assert!(!rep.lines.iter().any(|l| l.starts_with("review: ref:")), "partial plan block does not scan references: {:?}", rep.lines);
    let cont = r(&root, "1_Concept/12_Planning/Deprecated/P001_plan.partial.md");
    assert_eq!(cont, "---\ndeprecated: partial\nsource: 1_Concept/12_Planning/P001_plan.md\n---\n\n# P001: 필터 평가 계획\n\n## 2. 평가 절차\n> **Deprecated**: 2026-10-07 · P001-D01 · part · reason: 범위 축소\n\n<!-- deprecated:begin P001-D01 -->\n초기안: 6차 필터까지 확대 측정.\n\n- 6차는 [S002](../../../2_Log/S002_log.md) 결과 확인 후 결정\n- 참고 그림 ![2차 응답](../../../6_Exp/64_Viz/S002/S002_fig1.png)\n<!-- deprecated:end P001-D01 -->\n");
    run_restore(&root, "P001-D01").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
    // 계획 문서 전체: Status 줄이 있으면 같은 규칙
    run_deprecate(&root, &opts("P001")).unwrap();
    let dep = r(&root, "1_Concept/12_Planning/Deprecated/P001_plan.md");
    assert!(dep.contains("status_before: \"진행\"") && dep.contains("> **Status**: Deprecated\n"), "{dep}");
    run_restore(&root, "P001").unwrap();
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn close_after_partial_deprecation_keeps_marker_link_and_restore_matches_plain_close() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let reference = {
        let tmp2 = tempdir().unwrap();
        let root2 = fixture(tmp2.path());
        run_session_close(&root2, &CloseOptions { id: Some("S002".into()), force: false }).unwrap();
        (r(&root2, "2_Log/Archive/S002_log.md"), r(&root2, REG))
    };
    run_deprecate(&root, &trial_opts("S002", "t02", None)).unwrap();
    run_session_close(&root, &CloseOptions { id: Some("S002".into()), force: false }).unwrap();
    let arch = r(&root, "2_Log/Archive/S002_log.md");
    assert!(arch.contains("[Deprecated/S002_log.partial.md](../Deprecated/S002_log.partial.md) (S002-D01 ·"), "{arch}");
    assert!(root.join("2_Log/Archive/../Deprecated/S002_log.partial.md").is_file());
    let rep = run_restore(&root, "S002-D01").unwrap();
    assert_eq!(rep.lines[0], "restored S002-D01 → 2_Log/Archive/S002_log.md (t02)");
    assert_eq!(r(&root, "2_Log/Archive/S002_log.md"), reference.0);
    assert_eq!(r(&root, REG), reference.1);
}

#[test]
fn dry_run_writes_nothing_but_reports() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    let s0 = snapshot(&root);
    let rep = run_deprecate(&root, &DeprecateOptions { dry_run: true, ..trial_opts("S002", "t02", None) }).unwrap();
    assert!(!rep.changed);
    assert_eq!(rep.lines[0], "deprecated S002-D01 (t02) → 2_Log/Deprecated/S002_log.partial.md");
    assert!(rep.lines.iter().any(|l| l.starts_with("review: handoff:")), "{:?}", rep.lines);
    assert_same_tree(&snapshot(&root), &s0);
}

#[test]
fn restore_warns_on_missing_link_targets() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    run_deprecate(&root, &trial_opts("S002", "t02", None)).unwrap();
    fs::remove_file(root.join("6_Exp/64_Viz/S002/S002_fig2.png")).unwrap();
    let rep = run_restore(&root, "S002-D01").unwrap();
    assert!(rep.lines.iter().any(|l| l == "warn: link target not found: ../6_Exp/64_Viz/S002/S002_fig2.png (2_Log/S002_log.md)"), "{:?}", rep.lines);
}

#[test]
fn cli_exit_codes_and_list() {
    let tmp = tempdir().unwrap();
    let root = fixture(tmp.path());
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "list"]).assert().success()
        .stdout(predicates::str::contains("[elf] deprecated: 0 document(s), 0 block(s)"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "S099"]).assert().code(1)
        .stderr(predicates::str::contains("[elf] not found: session S099"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "0_Meta/ProjectRule.md"]).assert().code(3)
        .stderr(predicates::str::contains("[elf] refuse: out of scope"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "S002", "--trial", "t02", "--dry-run"]).assert().success()
        .stdout(predicates::str::contains("[elf] dry-run — nothing written"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "S002", "--trial", "t02", "--reason", "x"]).assert().success()
        .stdout(predicates::str::contains("[elf] deprecated S002-D01 (t02)"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "--restore", "S002-D01"]).assert().success()
        .stdout(predicates::str::contains("[elf] restored S002-D01"));
    Command::cargo_bin("elf").unwrap().current_dir(&root).args(["deprecate", "S002", "--lines", "x-y", "--expect", "a"]).assert().code(2);
}
