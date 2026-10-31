//! `elf trial new` — 활성 세션 로그에 현행 정본 trialTemplate stub을 append (S021 t03, t07).
//!
//! 스캐폴드 원리: 규칙 텍스트를 복제하지 않고 **정본 구조를 인스턴스화** — 에이전트의
//! 선례 모방 행동이 참조할 대상을 항상 현행 정본으로 유지한다(모방 대상의 자동 갱신).
//! CLI 바이너리가 현행 템플릿을 embed하므로 생성 stub은 개정 시점과 무관하게 최신.
//!
//! `--from <파일>`(S044): 에이전트가 파일 쓰기 도구로 쓴 trial 본문을 검사 후 덧붙임 — 틀을 붙인 뒤
//! 셸이나 스크립트로 채우는 단계의 escape 실패와 큰 로그 전체 읽기를 없앰. 첫 줄 Handoff 교체.

use std::fs;
use std::path::Path;

use crate::embed;
use crate::session::{SessionError, header_status, is_open_status, log_num, session_num};

pub struct TrialNewOptions {
    /// trial 제목 — None이면 template placeholder(`[작업 제목]`) 유지
    pub title: Option<String>,
    /// 대상 세션(S###) — None이면 유일 활성 세션 자동 선택
    pub session: Option<String>,
    /// YYYY-MM-DD — 헤더 Modified 갱신용(주입형; 테스트 결정성)
    pub date: String,
    /// `--from` 파일 내용 — `### ` 절로 된 trial 본문. 첫 줄이 `> **Handoff**:`이면 헤더 Handoff 교체
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct TrialNewResult {
    pub session: String,
    /// 생성된 trial id (예: "t09")
    pub trial: String,
    pub log_rel: String,
    /// `--from`이 Phase 1 절만 담았을 때 틀에서 채운 Phase 2 절
    pub filled_from_template: Vec<String>,
    /// `--from` 첫 줄로 헤더 Handoff를 교체했는가
    pub handoff_replaced: bool,
}

/// Phase 2 절 — `--from`이 Phase 1 절만 담으면 틀 문구로 채움(실행 뒤 작성).
const PHASE2_SECTIONS: [&str; 4] = ["관찰", "해석", "교훈", "생성 파일"];
const HANDOFF_PREFIX: &str = "> **Handoff**:";

/// `--from` 내용 → (Handoff 줄, 본문). BOM 제거, CRLF → LF. 첫 비어 있지 않은 줄이 Handoff 줄이면 분리.
pub fn split_handoff(text: &str) -> (Option<String>, String) {
    let text = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let mut lines = text.lines().skip_while(|l| l.trim().is_empty()).peekable();
    let handoff = match lines.peek() {
        Some(l) if l.starts_with(HANDOFF_PREFIX) => lines.next().map(|l| l.trim_end().to_string()),
        _ => None,
    };
    let body: Vec<&str> = lines.collect();
    (handoff, body.join("\n").trim_matches('\n').to_string())
}

/// 틀(렌더된 stub)에서 `### <key>` 절 블록(다음 `### ` 전까지) 추출.
fn template_section(stub: &str, key: &str) -> Option<String> {
    let mut out: Option<Vec<&str>> = None;
    for l in stub.lines() {
        if let Some(k) = crate::validate::section_key(l) {
            if out.is_some() {
                break;
            }
            if k == key {
                out = Some(vec![l]);
            }
            continue;
        }
        if let Some(v) = out.as_mut() {
            v.push(l);
        }
    }
    out.map(|v| v.join("\n").trim_end().to_string())
}

/// `--from` 본문 → trial 텍스트(`## tNN: 제목` + 본문, Phase 1만이면 Phase 2 틀 추가) 또는 덧붙이지 않을 이유.
/// 검사: `#`, `## ` 제목 금지, Phase 1 절 4개 필수, Phase 2 절은 전부 있거나 전부 없음, 정본 절 이름과 순서,
/// 해석 첫 줄, Phase 1 절이 틀 문구 그대로인지(`check logs`와 같은 검사 재사용).
pub fn compose_trial(body: &str, trial_id: &str, session_id: &str, title: &str) -> Result<(String, Vec<String>), Vec<String>> {
    let tpl = crate::validate::trial_template();
    let mut problems = Vec::new();
    for (n, l) in body.lines().enumerate() {
        // 코드 블록 안도 금지 — trial 번호 수집과 check logs가 블록을 구분하지 않음
        if l.starts_with("# ") || l.starts_with("## ") {
            problems.push(format!(
                "line {}: '{}' — the body holds only '### ' sections; the trial heading comes from the title argument",
                n + 1,
                l.chars().take(40).collect::<String>()
            ));
        }
    }
    let keys: Vec<&str> = body.lines().filter_map(crate::validate::section_key).collect();
    for k in crate::validate::PHASE1_SECTIONS {
        if !keys.contains(&k) {
            problems.push(format!("missing '### {k}' — Phase 1 sections are required (LogConvention §5.1)"));
        }
    }
    let phase2: Vec<&str> = PHASE2_SECTIONS.into_iter().filter(|k| keys.contains(k)).collect();
    let stub = render_stub(tpl, trial_id, session_id, Some(title));
    let mut text = format!("## {trial_id}: {title}\n\n{}", body.trim_matches('\n'));
    let mut filled = Vec::new();
    if phase2.is_empty() {
        for k in PHASE2_SECTIONS {
            if let Some(sec) = template_section(&stub, k) {
                text.push_str("\n\n");
                text.push_str(&sec);
                filled.push(k.to_string());
            }
        }
    } else if phase2.len() < PHASE2_SECTIONS.len() {
        for k in PHASE2_SECTIONS.into_iter().filter(|k| !phase2.contains(k)) {
            problems.push(format!(
                "missing '### {k}' — give all Phase 2 sections (관찰, 해석, 교훈, 생성 파일) or none (Phase 1 only)"
            ));
        }
    }
    text.push('\n');
    problems.extend(crate::validate::trial_structure_findings(&text));
    problems.extend(crate::validate::unfilled_template_findings(&text, tpl));
    if problems.is_empty() { Ok((text, filled)) } else { Err(problems) }
}

/// 헤더 `> **Handoff**:` 줄을 new_line으로 교체(끝 `\`, `\r` 보존). 줄이 없으면 None.
pub fn replace_handoff(content: &str, new_line: &str) -> Option<String> {
    let core = new_line.trim_end().trim_end_matches('\\').trim_end();
    let mut out: Vec<String> = content.split('\n').map(str::to_string).collect();
    let line = out.iter_mut().find(|l| l.starts_with(HANDOFF_PREFIX))?;
    let cr = line.ends_with('\r');
    let hard = line.trim_end_matches('\r').ends_with('\\');
    *line = format!("{core}{}{}", if hard { "\\" } else { "" }, if cr { "\r" } else { "" });
    Some(out.join("\n"))
}

/// 로그 본문의 `## t{NN}:` trial 번호 수집 (순수).
pub fn trial_numbers(content: &str) -> Vec<u32> {
    content
        .lines()
        .filter_map(|l| {
            let rest = l.trim_end_matches('\r').strip_prefix("## t")?;
            let digit_len = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            let (digits, tail) = rest.split_at(digit_len);
            if digits.is_empty() || !tail.starts_with(':') {
                return None;
            }
            digits.parse().ok()
        })
        .collect()
}

/// 다음 trial 번호 = max+1 (trial 없으면 1).
pub fn next_trial_number(content: &str) -> u32 {
    trial_numbers(content).into_iter().max().map_or(1, |m| m + 1)
}

/// trialTemplate 렌더 — `t{NN}`, `S{NNN}` 치환 + 제목(옵션).
fn render_stub(tpl: &str, trial_id: &str, session_id: &str, title: Option<&str>) -> String {
    let mut s = tpl.replace("t{NN}", trial_id).replace("S{NNN}", session_id);
    if let Some(t) = title {
        s = s.replace("[작업 제목]", t);
    }
    s
}

/// stub 삽입 위치: 구 로그(≤ v2.19 템플릿)에 `## 다음 세션 후보` 헤딩이 있으면 그 직전(그 앞 trial 구분과 동일한
/// `---` 경계 유지 — 하위호환), 없으면(v2.20 템플릿 기본) EOF에 `---` 구분과 함께 append. 기존 본문 불변(순수).
pub fn insert_trial(content: &str, stub: &str) -> String {
    let stub = stub.trim_end_matches('\n');
    // `## 다음 세션 후보` 헤딩 라인의 시작 바이트 탐색
    let mut heading_start: Option<usize> = None;
    let mut pos = 0;
    for line in content.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']).starts_with("## 다음 세션 후보") {
            heading_start = Some(pos);
            break;
        }
        pos += line.len();
    }
    match heading_start {
        // `...---\n\n` + [stub + `\n\n---\n\n`] + `## 다음 세션 후보...`
        Some(h) => format!("{}{}\n\n---\n\n{}", &content[..h], stub, &content[h..]),
        None => {
            let trimmed = content.trim_end_matches('\n');
            // 헤더만 있는 로그(v2.22 — 템플릿 t01 stub 제거): 규범 주석 뒤에 바로 t01 (구분선 중복 방지)
            if trimmed.ends_with("---") || trial_numbers(content).is_empty() {
                format!("{trimmed}\n\n{stub}\n")
            } else {
                format!("{trimmed}\n\n---\n\n{stub}\n")
            }
        }
    }
}

/// 헤더 `> **Modified**:` 값을 date로 갱신 (트레일링 `\`, `\r` 보존). 변경 없으면 None.
pub fn update_modified(content: &str, date: &str) -> Option<String> {
    let mut out: Vec<String> = content.split('\n').map(str::to_string).collect();
    let mut changed = false;
    for line in out.iter_mut() {
        let trimmed = line.strip_suffix('\r').unwrap_or(line);
        if !trimmed.starts_with("> **Modified**:") {
            continue;
        }
        let mut new = format!("> **Modified**: {date}");
        if trimmed.ends_with('\\') {
            new.push('\\');
        }
        if line.ends_with('\r') {
            new.push('\r');
        }
        if *line != new {
            *line = new;
            changed = true;
        }
        break;
    }
    if changed { Some(out.join("\n")) } else { None }
}

/// `elf trial new`: 대상 세션 결정(명시 또는 유일 활성) → 현행 trialTemplate stub append.
pub fn run_trial_new(root: &Path, opts: &TrialNewOptions) -> Result<TrialNewResult, SessionError> {
    let target = match &opts.session {
        Some(id) => {
            if session_num(id).is_none() {
                return Err(SessionError::NotFound(id.clone()));
            }
            id.clone()
        }
        None => {
            let mut open = Vec::new();
            if let Ok(entries) = fs::read_dir(root.join("2_Log")) {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if let Some(n) = log_num(&name) {
                        let st = fs::read_to_string(e.path())
                            .ok()
                            .and_then(|c| header_status(&c))
                            .unwrap_or_default();
                        if is_open_status(&st) {
                            open.push(format!("S{n:03}"));
                        }
                    }
                }
            }
            open.sort();
            match open.len() {
                0 => return Err(SessionError::NoOpenSession),
                1 => open.pop().unwrap(),
                _ => return Err(SessionError::MultipleOpen(open)),
            }
        }
    };

    let log_rel = format!("2_Log/{target}_log.md");
    let log_path = root.join(&log_rel);
    if !log_path.is_file() {
        return Err(SessionError::NotFound(target));
    }
    let content = fs::read_to_string(&log_path)?;

    let n = next_trial_number(&content);
    let trial_id = format!("t{n:02}");

    let tpl = embed::TEMPLATES
        .get_file("log/trialTemplate.md")
        .and_then(|f| f.contents_utf8())
        .expect("trialTemplate embedded (gated by tests)");
    let mut filled_from_template = Vec::new();
    let mut handoff = None;
    let stub = match &opts.body {
        None => render_stub(tpl, &trial_id, &target, opts.title.as_deref()),
        Some(raw) => {
            let Some(title) = opts.title.as_deref().filter(|t| !t.trim().is_empty()) else {
                return Err(SessionError::Rejected(vec![
                    "--from needs a trial title: elf trial new \"<title>\" --from <file>".into(),
                ]));
            };
            let (h, body) = split_handoff(raw);
            let (text, filled) = compose_trial(&body, &trial_id, &target, title).map_err(SessionError::Rejected)?;
            if h.is_some() && !content.lines().any(|l| l.starts_with(HANDOFF_PREFIX)) {
                return Err(SessionError::Rejected(vec![format!(
                    "{log_rel} has no '{HANDOFF_PREFIX}' header line to replace — remove the first line of the file or fix the header"
                )]));
            }
            filled_from_template = filled;
            handoff = h;
            text
        }
    };

    let updated = insert_trial(&content, &stub);
    let updated = match &handoff {
        Some(h) => replace_handoff(&updated, h).unwrap_or(updated),
        None => updated,
    };
    let updated = update_modified(&updated, &opts.date).unwrap_or(updated);
    fs::write(&log_path, updated)?;

    Ok(TrialNewResult { session: target, trial: trial_id, log_rel, filled_from_template, handoff_replaced: handoff.is_some() })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "# S021: T\n\n> **Created**: 2026-07-01\\\n> **Modified**: 2026-07-01\\\n> **Status**: ★ 활성\\\n> **Handoff**: -\n\n---\n\n## t01: 첫 작업\n\n### 목표 (Goal)\n- x\n\n---\n\n## 다음 세션 후보 (Next-Session Hypothesis)\n\n### 가설 후보\n- [후속 가설 1-3항]\n";

    #[test]
    fn trial_numbers_parse_only_trial_headers() {
        let c = "## t01: a\n## t2: b\n## t10: c\n## tXX: d\n### t03: not-a-trial\n## t04 no-colon\n";
        assert_eq!(trial_numbers(c), vec![1, 2, 10]);
        assert_eq!(next_trial_number(c), 11);
        assert_eq!(next_trial_number("no trials"), 1);
    }

    #[test]
    fn render_substitutes_ids_and_title() {
        let tpl = "## t{NN}: [작업 제목]\n![x](../6_Exp/64_Viz/S{NNN}/f.png)\n";
        let s = render_stub(tpl, "t05", "S021", Some("제목A"));
        assert!(s.contains("## t05: 제목A"));
        assert!(s.contains("64_Viz/S021/"));
        // 제목 생략 시 placeholder 유지
        let p = render_stub(tpl, "t05", "S021", None);
        assert!(p.contains("## t05: [작업 제목]"));
    }

    #[test]
    fn insert_before_next_session_section() {
        let out = insert_trial(LOG, "## t02: 새 작업\n\n### 목표 (Goal)\n- y\n");
        let t02 = out.find("## t02").unwrap();
        let next = out.find("## 다음 세션 후보").unwrap();
        let t01 = out.find("## t01").unwrap();
        assert!(t01 < t02 && t02 < next, "순서: t01 < t02 < 다음 세션 후보");
        // t02와 다음 세션 후보 사이 `---` 경계 존재
        assert!(out[t02..next].contains("\n---\n"));
        // 기존 본문 보존
        assert!(out.contains("## t01: 첫 작업"));
        assert!(out.contains("### 가설 후보"));
    }

    #[test]
    fn insert_appends_at_eof_without_next_section() {
        let log = "# S001: T\n\n## t01: a\n\n### 목표 (Goal)\n- x\n";
        let out = insert_trial(log, "## t02: b\n");
        assert!(out.ends_with("## t02: b\n"));
        assert!(out.contains("- x\n\n---\n\n## t02"));
        // 이미 `---`로 끝나면 중복 구분선 없음
        let log2 = "# S001: T\n\n## t01: a\n\n---\n";
        let out2 = insert_trial(log2, "## t02: b\n");
        assert!(out2.contains("---\n\n## t02: b\n"));
        assert!(!out2.contains("---\n\n---"));
    }

    const PHASE1: &str = "### 목표 (Goal)\n- g\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n";

    #[test]
    fn compose_phase1_only_gets_phase2_template() {
        let (text, filled) = compose_trial(PHASE1, "t03", "S044", "제목").unwrap();
        assert!(text.starts_with("## t03: 제목\n\n### 목표 (Goal)\n- g"));
        assert_eq!(filled, ["관찰", "해석", "교훈", "생성 파일"]);
        assert!(text.contains("### 관찰 (Observation)\n- [Phase 2") && text.contains("64_Viz/S044/"));
        assert!(text.ends_with("| Figure | `경로` |\n"));
    }

    #[test]
    fn compose_full_trial_passes_unchanged() {
        let body = format!("{PHASE1}\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n\n| 유형 | 파일 |\n|---|---|\n| (없음) | - |\n");
        let (text, filled) = compose_trial(&body, "t01", "S001", "T").unwrap();
        assert!(filled.is_empty());
        assert_eq!(text, format!("## t01: T\n\n{}\n", body.trim_matches('\n')));
    }

    #[test]
    fn compose_rejects_bad_bodies() {
        let has = |r: Result<(String, Vec<String>), Vec<String>>, s: &str| {
            let e = r.unwrap_err();
            assert!(e.iter().any(|x| x.contains(s)), "{s} not in {e:?}");
        };
        has(compose_trial(&format!("## t09: x\n{PHASE1}"), "t01", "S001", "T"), "line 1: '## t09: x'");
        has(compose_trial("### 목표 (Goal)\n- g\n", "t01", "S001", "T"), "missing '### 가설'");
        has(compose_trial(&format!("{PHASE1}\n### 관찰 (Observation)\n- o\n"), "t01", "S001", "T"), "missing '### 해석'");
        has(compose_trial(&format!("{PHASE1}\n### 비정본\n- x\n"), "t01", "S001", "T"), "non-canonical heading");
        let tpl_goal = PHASE1.replace("- g", "- [이 작업의 구체적 목표]");
        has(compose_trial(&tpl_goal, "t01", "S001", "T"), "still holds only the template text");
        // 코드 블록 안의 `## `도 거부(번호 수집이 블록을 구분하지 않음)
        assert!(compose_trial(&format!("{PHASE1}- 예:\n```\n## t01: 예시\n```\n"), "t01", "S001", "T").is_err());
    }

    #[test]
    fn handoff_split_and_replace_keep_header_form() {
        let (h, body) = split_handoff("\u{feff}\r\n> **Handoff**: 새 상태; 미완료 = X; 참조 t02\r\n\r\n### 목표 (Goal)\r\n- g\r\n");
        assert_eq!(h.as_deref(), Some("> **Handoff**: 새 상태; 미완료 = X; 참조 t02"));
        assert_eq!(body, "### 목표 (Goal)\n- g");
        assert_eq!(split_handoff("### 목표 (Goal)\n").0, None);
        let out = replace_handoff(LOG, h.as_deref().unwrap()).unwrap();
        assert!(out.contains("> **Handoff**: 새 상태; 미완료 = X; 참조 t02\n\n---"));
        assert!(!out.contains("> **Handoff**: -"));
        // 끝 `\`가 있던 줄은 유지
        let hard = LOG.replace("> **Handoff**: -\n", "> **Handoff**: -\\\n> **결론**: k\n");
        assert!(replace_handoff(&hard, "> **Handoff**: y").unwrap().contains("> **Handoff**: y\\\n> **결론**: k"));
        assert!(replace_handoff("# no header\n", "> **Handoff**: y").is_none());
    }

    #[test]
    fn modified_updated_preserving_hardbreak() {
        let out = update_modified(LOG, "2026-07-06").unwrap();
        assert!(out.contains("> **Modified**: 2026-07-06\\\n"));
        assert!(out.contains("> **Created**: 2026-07-01\\\n")); // 타 줄 불변
        assert!(update_modified(&out, "2026-07-06").is_none()); // 멱등
    }
}
