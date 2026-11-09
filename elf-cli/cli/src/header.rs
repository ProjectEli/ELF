//! 세션 로그 헤더 — YAML front matter 고정 7키 (P022 §3, S044 t12~t20).
//!
//! 형식: 파일 맨 앞 `---`, 키 7개가 정해진 순서로 한 줄씩, `---`. 날짜와 status는 따옴표 없이,
//! 자유 텍스트(goal, handoff, conclusion, user_note)는 항상 작은따옴표(안의 `'`는 `''`), 빈 값 `''`.
//! 파서는 이 모양만 받음(누락, 추가, 순서 어긋남 = 형식 오류, 줄 번호와 기대 키 보고).
//! 헤더는 elf 명령만 씀(LogConvention §2) — 렌더는 항상 같은 바이트를 냄.

/// 고정 키 순서.
pub const KEYS: [&str; 7] = ["created", "modified", "status", "goal", "handoff", "conclusion", "user_note"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ongoing,
    Complete,
    Deprecated,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ongoing => "ongoing",
            Status::Complete => "complete",
            Status::Deprecated => "deprecated",
        }
    }
    pub fn parse(s: &str) -> Option<Status> {
        match s {
            "ongoing" => Some(Status::Ongoing),
            "complete" => Some(Status::Complete),
            "deprecated" => Some(Status::Deprecated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub created: String,
    pub modified: String,
    pub status: Status,
    pub goal: String,
    pub handoff: String,
    pub conclusion: String,
    pub user_note: String,
}

impl Header {
    pub fn new(date: &str, goal: &str) -> Header {
        Header {
            created: date.to_string(),
            modified: date.to_string(),
            status: Status::Ongoing,
            goal: one_line(goal),
            handoff: String::new(),
            conclusion: String::new(),
            user_note: String::new(),
        }
    }
}

/// 헤더 형식 오류 — 줄 번호(1부터)와 기대한 것.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "header line {}: {}", self.line, self.message)
    }
}

/// 값을 한 줄로(줄바꿈 → 공백). 헤더 값은 한 줄 규칙.
pub fn one_line(v: &str) -> String {
    v.replace("\r\n", " ").replace(['\n', '\r'], " ").trim().to_string()
}

/// 자유 텍스트 → `'…'`(안의 `'`는 `''`).
pub fn quote(v: &str) -> String {
    format!("'{}'", one_line(v).replace('\'', "''"))
}

/// `'…'` → 값. 작은따옴표로 감싸지 않았거나 안의 `'`가 짝을 이루지 않으면 Err.
pub fn unquote(raw: &str) -> Result<String, String> {
    let r = raw.trim();
    let inner = r
        .strip_prefix('\'')
        .and_then(|x| x.strip_suffix('\''))
        .ok_or_else(|| format!("expected a single-quoted value, found {r}"))?;
    if inner.replace("''", "").contains('\'') {
        return Err(format!("a ' inside the value must be written '' (found {r})"));
    }
    Ok(inner.replace("''", "'"))
}

/// 헤더 렌더(`---\n` … `---\n`).
pub fn render(h: &Header) -> String {
    format!(
        "---\ncreated: {}\nmodified: {}\nstatus: {}\ngoal: {}\nhandoff: {}\nconclusion: {}\nuser_note: {}\n---\n",
        h.created,
        h.modified,
        h.status.as_str(),
        quote(&h.goal),
        quote(&h.handoff),
        quote(&h.conclusion),
        quote(&h.user_note)
    )
}

/// 내용 → (헤더, 헤더 뒤 본문의 바이트 위치). 엄격 파싱.
pub fn parse(content: &str) -> Result<(Header, usize), HeaderError> {
    let text = content.strip_prefix('\u{feff}').unwrap_or(content);
    let bom = content.len() - text.len();
    let mut pos = 0usize;
    let mut lines = Vec::new();
    for raw in text.split_inclusive('\n') {
        lines.push((pos, raw));
        pos += raw.len();
        if lines.len() > KEYS.len() + 2 {
            break;
        }
    }
    let line_of = |i: usize| -> &str { lines.get(i).map(|(_, l)| l.trim_end_matches(['\n', '\r'])).unwrap_or("") };
    if line_of(0) != "---" {
        return Err(HeaderError { line: 1, message: "expected '---' (YAML front matter with the 7 header keys)".into() });
    }
    let mut vals: Vec<String> = Vec::with_capacity(KEYS.len());
    for (k, key) in KEYS.iter().enumerate() {
        let i = k + 1;
        let l = line_of(i);
        let Some(v) = l.strip_prefix(&format!("{key}:")) else {
            return Err(HeaderError {
                line: i + 1,
                message: format!("expected '{key}: …' (keys in fixed order: {})", KEYS.join(", ")),
            });
        };
        vals.push(v.trim().to_string());
    }
    let close = KEYS.len() + 1;
    if line_of(close) != "---" {
        return Err(HeaderError { line: close + 1, message: "expected '---' after the 7 header keys".into() });
    }
    let body_at = bom + lines.get(close).map(|(p, l)| p + l.len()).unwrap_or(text.len());
    let status = Status::parse(&vals[2]).ok_or_else(|| HeaderError {
        line: 4,
        message: format!("status must be ongoing, complete or deprecated (found {})", vals[2]),
    })?;
    let mut free = Vec::new();
    for (k, key) in KEYS.iter().enumerate().skip(3) {
        free.push(unquote(&vals[k]).map_err(|m| HeaderError { line: k + 2, message: format!("{key}: {m}") })?);
    }
    let h = Header {
        created: vals[0].clone(),
        modified: vals[1].clone(),
        status,
        goal: free[0].clone(),
        handoff: free[1].clone(),
        conclusion: free[2].clone(),
        user_note: free[3].clone(),
    };
    Ok((h, body_at))
}

/// 헤더를 바꾼 전체 내용(본문 그대로).
pub fn replace(content: &str, h: &Header) -> Result<String, HeaderError> {
    let (_, at) = parse(content)?;
    Ok(format!("{}{}", render(h), &content[at..]))
}

/// `--from` 파일의 맨 앞 YAML 블록 → (키와 값 목록, 나머지 본문). 블록이 없으면 빈 목록.
/// 값은 작은따옴표(`''` escape) 또는 따옴표 없는 한 줄. 같은 키가 두 번이면 Err.
pub fn split_block(text: &str) -> Result<(Vec<(String, String)>, String), String> {
    let text = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let mut it = text.lines().skip_while(|l| l.trim().is_empty()).peekable();
    if it.peek().map(|l| l.trim_end()) != Some("---") {
        let body: Vec<&str> = text.lines().collect();
        return Ok((Vec::new(), body.join("\n").trim_matches('\n').to_string()));
    }
    it.next();
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut closed = false;
    for l in it.by_ref() {
        let t = l.trim_end();
        if t == "---" {
            closed = true;
            break;
        }
        if t.trim().is_empty() {
            continue;
        }
        let Some((k, v)) = t.split_once(':') else {
            return Err(format!("YAML block line is not 'key: value': {t}"));
        };
        let k = k.trim().to_string();
        let v = v.trim();
        let val = if v.starts_with('\'') { unquote(v)? } else { v.to_string() };
        if pairs.iter().any(|(x, _)| *x == k) {
            return Err(format!("YAML block key '{k}' appears twice"));
        }
        pairs.push((k, val));
    }
    if !closed {
        return Err("YAML block at the top of the file is not closed with '---'".into());
    }
    let rest: Vec<&str> = it.collect();
    Ok((pairs, rest.join("\n").trim_matches('\n').to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Header {
        Header {
            created: "2026-11-09".into(),
            modified: "2026-11-10".into(),
            status: Status::Ongoing,
            goal: "파장별 SNR 비교".into(),
            handoff: "A 기각; 미완료 = X; 참조 t01".into(),
            conclusion: String::new(),
            user_note: "it's ok".into(),
        }
    }

    #[test]
    fn render_parse_roundtrip_and_quote_escape() {
        let h = sample();
        let text = format!("{}# S001: 제목\n\n본문\n", render(&h));
        assert!(text.contains("user_note: 'it''s ok'"));
        assert!(text.contains("conclusion: ''"));
        let (p, at) = parse(&text).unwrap();
        assert_eq!(p, h);
        assert_eq!(&text[at..], "# S001: 제목\n\n본문\n");
    }

    #[test]
    fn parse_rejects_missing_reordered_or_bad_values() {
        let good = render(&sample());
        let swapped = good.replace("goal:", "XX:").replace("handoff:", "goal:").replace("XX:", "handoff:");
        let e = parse(&swapped).unwrap_err();
        assert_eq!(e.line, 5);
        assert!(e.message.contains("goal"));
        assert!(parse(&good.replace("status: ongoing", "status: Active")).unwrap_err().message.contains("status"));
        assert!(parse(&good.replace("goal: '파장별 SNR 비교'", "goal: 파장별")).is_err());
        assert!(parse(&good.replace("user_note: 'it''s ok'", "user_note: 'it's ok'")).is_err());
        assert_eq!(parse("# title\n").unwrap_err().line, 1);
        let missing_close = good.replacen("---\n", "", 2).replacen("", "---\n", 1);
        assert!(parse(&missing_close).is_err());
    }

    #[test]
    fn replace_keeps_body_and_one_line_values() {
        let text = format!("{}# S001: t\nbody\n", render(&sample()));
        let mut h = sample();
        h.status = Status::Complete;
        h.conclusion = "결론\n두 줄".into();
        let out = replace(&text, &h).unwrap();
        assert!(out.contains("status: complete\n"));
        assert!(out.contains("conclusion: '결론 두 줄'\n"));
        assert!(out.ends_with("# S001: t\nbody\n"));
    }

    #[test]
    fn split_block_reads_yaml_and_body() {
        let (kv, body) = split_block("---\nhandoff: '상태 = x; 미완료 = y'\n---\n### 목표 (Goal)\n- a\n").unwrap();
        assert_eq!(kv, vec![("handoff".to_string(), "상태 = x; 미완료 = y".to_string())]);
        assert_eq!(body, "### 목표 (Goal)\n- a");
        let (kv, body) = split_block("\u{feff}### 목표 (Goal)\r\n- a\r\n").unwrap();
        assert!(kv.is_empty());
        assert_eq!(body, "### 목표 (Goal)\n- a");
        assert!(split_block("---\nhandoff: 'x'\n### 목표\n").is_err());
        assert!(split_block("---\na: 1\na: 2\n---\n").is_err());
    }
}
