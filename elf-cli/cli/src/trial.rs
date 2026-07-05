//! `elf trial new` — 활성 세션 로그에 현행 정본 trialTemplate stub을 append (S021 t03·t07).
//!
//! 스캐폴드 원리: 규칙 텍스트를 복제하지 않고 **정본 구조를 인스턴스화** — 에이전트의
//! 선례 모방 행동이 참조할 대상을 항상 현행 정본으로 유지한다(모방 채널 자기치유).
//! CLI 바이너리가 현행 템플릿을 embed하므로 생성 stub은 개정 시점과 무관하게 최신.

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
}

#[derive(Debug)]
pub struct TrialNewResult {
    pub session: String,
    /// 생성된 trial id (예: "t09")
    pub trial: String,
    pub log_rel: String,
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

/// trialTemplate 렌더 — `t{NN}`·`S{NNN}` 치환 + 제목(옵션).
fn render_stub(tpl: &str, trial_id: &str, session_id: &str, title: Option<&str>) -> String {
    let mut s = tpl.replace("t{NN}", trial_id).replace("S{NNN}", session_id);
    if let Some(t) = title {
        s = s.replace("[작업 제목]", t);
    }
    s
}

/// stub을 `## 다음 세션 후보` 헤딩 직전(그 앞 trial 구분과 동일한 `---` 경계 유지)에 삽입.
/// 섹션 부재 시 EOF에 `---` 구분과 함께 append. 기존 본문 불변(순수).
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
            if trimmed.ends_with("---") {
                format!("{trimmed}\n\n{stub}\n")
            } else {
                format!("{trimmed}\n\n---\n\n{stub}\n")
            }
        }
    }
}

/// 헤더 `> **Modified**:` 값을 date로 갱신 (트레일링 `\`·`\r` 보존). 변경 없으면 None.
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
    let stub = render_stub(tpl, &trial_id, &target, opts.title.as_deref());

    let updated = insert_trial(&content, &stub);
    let updated = update_modified(&updated, &opts.date).unwrap_or(updated);
    fs::write(&log_path, updated)?;

    Ok(TrialNewResult { session: target, trial: trial_id, log_rel })
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

    #[test]
    fn modified_updated_preserving_hardbreak() {
        let out = update_modified(LOG, "2026-07-06").unwrap();
        assert!(out.contains("> **Modified**: 2026-07-06\\\n"));
        assert!(out.contains("> **Created**: 2026-07-01\\\n")); // 타 줄 불변
        assert!(update_modified(&out, "2026-07-06").is_none()); // 멱등
    }
}
