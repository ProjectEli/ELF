//! `elf session new` / `session close` — 세션 수명주기 (P012 t01, t02; 구조 개편 P022 §6, S044).
//!
//! 로그는 `2_Log/S/`(연구), `2_Log/G/`(거버넌스)에 고정되고 옮기지 않음. 상태 = YAML 헤더 `status`.
//! 헤더는 elf 명령만 씀: `session new --goal`(goal), `session close --from`(conclusion, user_note, handoff).
//! 로그를 바꾼 뒤 `logs::refresh`가 `Registry.md`와 탐색기 nesting을 다시 만듦.
//!
//! 이 파일 아래쪽의 "이전 형식" 절(TSV Registry 파서, 인용 블록 헤더 읽기)은 `elf update` 이관(migrate.rs)이 씀.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::embed;
use crate::header::{self, Header, Status};
use crate::logs::{self, Series};

#[derive(Debug)]
pub enum SessionError {
    /// 계산된 로그 파일이 이미 존재 — refuse (exit 3)
    Exists(String),
    /// 제목, goal 등 입력 오류 — refuse
    BadTitle(String),
    /// 닫을(쓸) 진행 중 세션 없음
    NoOpenSession,
    /// 진행 중 세션 여럿 — 명시 필요
    MultipleOpen(Vec<String>),
    /// 지정 세션 없음
    NotFound(String),
    /// 입력 파일 검사 실패 — 쓰지 않은 이유 목록 (refuse, exit 3)
    Rejected(Vec<String>),
    /// 로그 헤더(YAML 7키)를 읽을 수 없음 — 형식 오류 위치 (refuse, exit 3)
    BadHeader(String),
    /// 이전 형식 로그가 남아 있음 — `elf update`로 이관 먼저 (refuse, exit 3)
    Legacy(Vec<String>),
    Io(io::Error),
}

impl From<io::Error> for SessionError {
    fn from(e: io::Error) -> Self {
        SessionError::Io(e)
    }
}

/// 이전 형식 흔적이 있으면 쓰기 명령을 막음(두 형식이 섞이지 않게).
pub fn refuse_legacy(root: &Path) -> Result<(), SessionError> {
    let m = logs::legacy_markers(root);
    if m.is_empty() { Ok(()) } else { Err(SessionError::Legacy(m)) }
}

/// 세션 기록이 이미 있는가 — 새 구조 로그 또는 이전 형식 흔적(제자리 init 판정).
pub fn has_session_records(root: &Path) -> bool {
    !logs::scan_all(root).is_empty() || !logs::legacy_markers(root).is_empty()
}

/// 내장 sessionTemplate 원문.
pub fn template_text() -> &'static str {
    embed::TEMPLATES
        .get_file("log/sessionTemplate.md")
        .and_then(|f| f.contents_utf8())
        .expect("sessionTemplate embedded (gated by tests)")
}

/// 내장 sessionTemplate의 헤더 뒤 본문(제목 줄과 규범 주석).
fn template_body() -> &'static str {
    let tpl = template_text();
    match header::parse(tpl) {
        Ok((_, at)) => &tpl[at..],
        Err(_) => tpl,
    }
}

pub struct SessionNewOptions {
    pub title: String,
    /// YYYY-MM-DD — 주입형(테스트 결정성; main이 오늘 날짜 주입)
    pub date: String,
    /// 세션 목표 1-2문장 — 필수(헤더는 명령만 쓰므로 만들 때 정함)
    pub goal: String,
    /// 거버넌스 세션(`2_Log/G/`)
    pub gov: bool,
}

#[derive(Debug)]
pub struct SessionNewResult {
    pub id: String,
    pub log_rel: String,
    pub warnings: Vec<String>,
}

/// `elf session new`: 계열 폴더에 다음 번호로 로그 생성(YAML 헤더 + 제목 + 규범 주석) → Registry.md 갱신.
pub fn run_session_new(root: &Path, opts: &SessionNewOptions) -> Result<SessionNewResult, SessionError> {
    if opts.title.trim().is_empty() || opts.title.contains(['\n', '\r']) {
        return Err(SessionError::BadTitle("session title must be one non-empty line".into()));
    }
    if opts.goal.trim().is_empty() {
        return Err(SessionError::BadTitle(
            "--goal is required: the session goal in 1-2 sentences (the header is written only by elf commands)".into(),
        ));
    }
    refuse_legacy(root)?;
    let series = if opts.gov { Series::G } else { Series::S };
    fs::create_dir_all(root.join(series.dir()))?;

    // 원자적 번호 할당: create_new 성공 = 그 번호의 유일 소유(TOCTOU 없음). 선점 시 재스캔.
    let (mut file, id, log_rel) = 'alloc: {
        for _ in 0..32 {
            let n = logs::next_number(root, series);
            let rel = logs::log_rel(series, n);
            match fs::OpenOptions::new().write(true).create_new(true).open(root.join(&rel)) {
                Ok(f) => break 'alloc (f, logs::id_of(series, n), rel),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        return Err(SessionError::Io(io::Error::other(
            "could not allocate a session number in 32 attempts (unexpected contention)",
        )));
    };
    let h = Header::new(&opts.date, &opts.goal);
    let body = template_body().replace("S{NNN}", &id).replace("[세션 제목]", opts.title.trim());
    file.write_all(format!("{}{}", header::render(&h), body).as_bytes())?;
    drop(file);
    let warnings = logs::refresh(root)?;
    Ok(SessionNewResult { id, log_rel, warnings })
}

pub struct CloseOptions {
    pub id: Option<String>,
    /// `--from` 파일 내용 — YAML 블록(conclusion 필수, user_note, handoff)
    pub body: Option<String>,
    pub date: String,
}

#[derive(Debug)]
pub struct CloseResult {
    pub id: String,
    pub log_rel: String,
    /// 비차단 경고(handoff 미완료 잔존, 닫는 세션의 점검 결과)
    pub warnings: Vec<String>,
}

/// 종료 파일 예(거부 메시지에 함께 출력).
pub const CLOSE_FILE_EXAMPLE: &str = "---\nconclusion: '현재 유효한 최종 결론'\nuser_note: '사용자 원문(건너뛰면 빈 값)'\nhandoff: ''\n---";

/// `elf session close --from <파일>`: status complete, conclusion(필수), user_note, handoff 기록. 파일은 옮기지 않음.
pub fn run_session_close(root: &Path, opts: &CloseOptions) -> Result<CloseResult, SessionError> {
    refuse_legacy(root)?;
    let target = logs::resolve(root, opts.id.as_deref())?;
    let (mut h, content) = logs::read(&target)?;
    let log_rel = logs::rel_of(&target);
    if h.status != Status::Ongoing {
        return Err(SessionError::Rejected(vec![format!(
            "{} is already {} — only an ongoing session can be closed",
            target.id,
            h.status.as_str()
        )]));
    }
    let approval = "show the user a conclusion draft, get approval, and ask for the user's own note (user_note, their words only) before closing (LogConvention §5.2)";
    let Some(raw) = opts.body.as_deref() else {
        return Err(SessionError::Rejected(vec![
            "a conclusion is required: elf session close --from <file>".into(),
            approval.into(),
            format!("file example:\n{CLOSE_FILE_EXAMPLE}"),
        ]));
    };
    let (pairs, rest) = header::split_block(raw).map_err(|e| SessionError::Rejected(vec![e]))?;
    let mut problems = Vec::new();
    if !rest.trim().is_empty() {
        problems.push("the close file holds only the YAML block (conclusion, user_note, handoff)".into());
    }
    for (k, _) in &pairs {
        if !["conclusion", "user_note", "handoff"].contains(&k.as_str()) {
            problems.push(format!("key '{k}' is not allowed — allowed: conclusion (required), user_note, handoff"));
        }
    }
    let get = |k: &str| pairs.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone());
    let conclusion = get("conclusion").unwrap_or_default();
    if conclusion.trim().is_empty() {
        problems.push("conclusion is missing or empty — the session is not closed without a conclusion".into());
        problems.push(approval.into());
    }
    if !problems.is_empty() {
        return Err(SessionError::Rejected(problems));
    }
    h.status = Status::Complete;
    h.conclusion = header::one_line(&conclusion);
    if let Some(u) = get("user_note") {
        h.user_note = header::one_line(&u);
    }
    if let Some(x) = get("handoff") {
        h.handoff = header::one_line(&x);
    }
    h.modified = opts.date.clone();

    let mut warnings = Vec::new();
    if let Some(pending) = handoff_pending(&h.handoff) {
        warnings.push(format!(
            "handoff still lists pending items: \"{pending}\" — finish them or move them to a planning doc (LogConvention §5.2)"
        ));
    }
    // 닫는 세션 범위의 점검 결과(비차단) — 종료 전 마지막 확인
    if let Ok(v) = crate::validate::run_validate(root) {
        let by_file = format!("{}_log.md", target.id);
        let by_id = format!("{}:", target.id);
        for l in &v.lines {
            let body = l.strip_prefix("issue: ").or_else(|| l.strip_prefix("warn: ")).unwrap_or(l);
            if body.starts_with(&by_file) || body.starts_with(&by_id) {
                warnings.push(format!("check ({}): {l}", target.id));
            }
        }
    }
    let updated = header::replace(&content, &h).map_err(|e| SessionError::BadHeader(format!("{log_rel}: {e}")))?;
    fs::write(&target.path, updated)?;
    warnings.extend(logs::refresh(root)?);
    Ok(CloseResult { id: target.id, log_rel, warnings })
}

/// cwd에서 위로 올라가며 `2_Log/` 보유 디렉토리(세션 루트)를 찾음.
pub fn find_log_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|a| a.join("2_Log").is_dir()).map(Path::to_path_buf)
}

/// handoff 3파트 중 미완료 파트의 잔존 내용 (S021 t13 #3, t17 경계 강화).
/// 파트 경계 = **`;` 직후(공백 허용) 라벨**(`미완료`/`pending`, `참조`/`refs`)이 오는 위치만.
/// 값 맨 앞의 라벨도 경계로 봄(첫 파트를 생략한 handoff). 라벨 경계 부재 = 판정 보류(None).
pub fn handoff_pending(handoff: &str) -> Option<String> {
    let owned = format!(";{handoff}");
    let handoff = owned.as_str();
    let boundary_after = |labels: &[&str], from: usize| -> Option<(usize, usize)> {
        let mut i = from;
        while let Some(off) = handoff[i..].find(';') {
            let semi = i + off;
            let rest = &handoff[semi + 1..];
            let content_at = semi + 1 + (rest.len() - rest.trim_start().len());
            for l in labels {
                if handoff[content_at..].starts_with(l) {
                    return Some((semi, content_at + l.len()));
                }
            }
            i = semi + 1;
        }
        None
    };
    let (_, pending_start) = boundary_after(&["미완료", "pending", "Pending"], 0)?;
    let end = boundary_after(&["참조", "refs", "Refs", "references"], pending_start)
        .map(|(semi, _)| semi)
        .unwrap_or(handoff.len());
    let mut p = handoff[pending_start..end].trim();
    p = p.strip_prefix(['=', ':']).unwrap_or(p).trim();
    let p = p.trim_start_matches(['—', '–', '-', ' ']).trim();
    let none = p.is_empty()
        || p == "-"
        || p.starts_with("없음")
        || p.get(..4).is_some_and(|x| x.eq_ignore_ascii_case("none"));
    if none {
        None
    } else {
        Some(p.to_string())
    }
}

// ── 이전 형식(이관 전용) ───────────────────────────────────────────
//
// `2_Log/Wiki/Session_Registry.tsv` 6열 파서와 인용 블록 헤더(`> **Status**:` 등) 읽기.
// 새 구조에서는 쓰지 않고, `elf update` 이관이 두 신호(헤더 Status, Registry)를 읽을 때만 씀.

pub const REGISTRY_REL: &str = "2_Log/Wiki/Session_Registry.tsv";
pub const REGISTRY_COLS: usize = 6;
const ACTION_FIX_REGISTRY: &str =
    "표시된 행을 스키마에 맞게 수정 후 명령 재실행 (이 도구는 Registry를 자동 수정하지 않음)";

/// 판단 개입 필요 — 상위 LLM 에이전트 위임 신호 (P012 §0.1, exit 5).
/// stderr에 자가완결 보고 + `agent-action:` 고정 마커 출력(에이전트가 패턴 인식해 수리, 재실행).
#[derive(Debug, PartialEq, Eq)]
pub struct Escalation {
    pub file: String,
    pub line: usize, // 1-based; 0 = 해당 없음
    pub expected: String,
    pub found: String,
    pub raw: String,
    pub action: String,
}

impl std::fmt::Display for Escalation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "escalation: {}", self.file)?;
        if self.line > 0 {
            write!(f, " (line {})", self.line)?;
        }
        writeln!(f)?;
        writeln!(f, "  expected: {}", self.expected)?;
        writeln!(f, "  found:    {}", self.found)?;
        if !self.raw.is_empty() {
            writeln!(f, "  raw:      {}", self.raw)?;
        }
        // 'agent-action:' 은 column 0 고정 마커 — grep/패턴 계약
        write!(f, "agent-action: {}", self.action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryRow {
    pub line: usize,
    pub session: String,
    pub date: String,
    pub title: String,
    pub status: String,
    pub key_finding: String,
    pub archive_path: String,
}

/// Registry TSV → 행 목록. 헤더(`Session\t…`), 빈 줄 skip. 6열 아니거나 세션ID가 S### 아니면 escalation.
pub fn parse_registry(text: &str) -> Result<Vec<RegistryRow>, Box<Escalation>> {
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let lineno = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        if cols.first() == Some(&"Session") {
            continue; // 헤더
        }
        if cols.len() != REGISTRY_COLS {
            return Err(Box::new(Escalation {
                file: REGISTRY_REL.into(),
                line: lineno,
                expected: format!(
                    "{REGISTRY_COLS} tab-separated columns (Session/Date/Title/Status/Key Finding/Archive Path)"
                ),
                found: format!("{} columns", cols.len()),
                raw: line.to_string(),
                action: ACTION_FIX_REGISTRY.into(),
            }));
        }
        if session_num(cols[0]).is_none() {
            return Err(Box::new(Escalation {
                file: REGISTRY_REL.into(),
                line: lineno,
                expected: "session id like S### in column 1".into(),
                found: format!("\"{}\"", cols[0]),
                raw: line.to_string(),
                action: ACTION_FIX_REGISTRY.into(),
            }));
        }
        rows.push(RegistryRow {
            line: lineno,
            session: cols[0].into(),
            date: cols[1].into(),
            title: cols[2].into(),
            status: cols[3].into(),
            key_finding: cols[4].into(),
            archive_path: cols[5].into(),
        });
    }
    Ok(rows)
}

/// "S007" → 7. 형식 불일치 시 None.
pub fn session_num(id: &str) -> Option<u32> {
    let digits = id.strip_prefix('S')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// "S007_log.md" → 7.
pub fn log_num(filename: &str) -> Option<u32> {
    session_num(filename.strip_suffix("_log.md")?)
}

/// 이전 형식 헤더 블록의 `> **<label>**:` 값(트레일링 `\`, `\r` 제거).
pub fn legacy_field(content: &str, label: &str) -> Option<String> {
    let prefix = format!("> **{label}**:");
    content.lines().find_map(|line| {
        line.trim_end_matches('\r').strip_prefix(&prefix).map(|v| v.trim_end_matches('\\').trim().to_string())
    })
}

/// 이전 형식 헤더 `> **Status**:` 값.
pub fn header_status(content: &str) -> Option<String> {
    legacy_field(content, "Status")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path";

    fn opts(title: &str, goal: &str, gov: bool) -> SessionNewOptions {
        SessionNewOptions { title: title.into(), date: "2026-11-09".into(), goal: goal.into(), gov }
    }

    #[test]
    fn new_creates_yaml_log_in_series_folder_and_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log")).unwrap();
        let r = run_session_new(root, &opts("SNR 비교", "파장별 SNR 비교", false)).unwrap();
        assert_eq!(r.id, "S001");
        assert_eq!(r.log_rel, "2_Log/S/S001_log.md");
        let text = fs::read_to_string(root.join(&r.log_rel)).unwrap();
        let (h, at) = header::parse(&text).unwrap();
        assert_eq!(h.status, Status::Ongoing);
        assert_eq!(h.goal, "파장별 SNR 비교");
        assert!(text[at..].starts_with("# S001: SNR 비교\n"));
        assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("[S001](S001_log.md) | ongoing"));
        let g = run_session_new(root, &opts("규칙", "규칙 개정", true)).unwrap();
        assert_eq!(g.log_rel, "2_Log/G/G001_log.md");
        assert!(fs::read_to_string(root.join(&g.log_rel)).unwrap().contains("# G001: 규칙"));
        assert_eq!(run_session_new(root, &opts("둘째", "x", false)).unwrap().id, "S002");
    }

    #[test]
    fn new_refuses_without_goal_or_with_legacy_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log")).unwrap();
        assert!(matches!(run_session_new(root, &opts("t", " ", false)), Err(SessionError::BadTitle(_))));
        fs::write(root.join("2_Log/S003_log.md"), "# S003: old\n").unwrap();
        assert!(matches!(run_session_new(root, &opts("t", "g", false)), Err(SessionError::Legacy(v)) if v[0] == "2_Log/S003_log.md"));
    }

    #[test]
    fn close_requires_conclusion_and_writes_header() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log")).unwrap();
        run_session_new(root, &opts("t", "g", false)).unwrap();
        let close = |body: Option<&str>| {
            run_session_close(root, &CloseOptions { id: None, body: body.map(str::to_string), date: "2026-11-10".into() })
        };
        assert!(matches!(close(None), Err(SessionError::Rejected(v)) if v[0].contains("conclusion is required")));
        assert!(matches!(close(Some("---\nconclusion: ''\n---\n")), Err(SessionError::Rejected(_))));
        assert!(matches!(close(Some("---\nconclusion: 'x'\nstatus: done\n---\n")), Err(SessionError::Rejected(v)) if v[0].contains("not allowed")));
        let r = close(Some("---\nconclusion: '810 nm 최적'\nuser_note: '다음은 5층'\nhandoff: '결론; 미완료 = 735 재검토; 참조 t03'\n---\n")).unwrap();
        assert!(r.warnings.iter().any(|w| w.contains("pending")));
        let text = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
        let (h, _) = header::parse(&text).unwrap();
        assert_eq!((h.status, h.conclusion.as_str(), h.user_note.as_str(), h.modified.as_str()), (Status::Complete, "810 nm 최적", "다음은 5층", "2026-11-10"));
        assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("| complete | 810 nm 최적 |"));
        // 닫힌 세션은 다시 닫지 않음
        assert!(matches!(run_session_close(root, &CloseOptions { id: Some("S001".into()), body: Some("---\nconclusion: 'y'\n---".into()), date: "d".into() }), Err(SessionError::Rejected(_))));
    }

    #[test]
    fn template_has_yaml_header_and_body() {
        let tpl = embed::TEMPLATES.get_file("log/sessionTemplate.md").unwrap().contents_utf8().unwrap();
        assert!(header::parse(tpl).is_ok());
        assert!(template_body().starts_with("# S{NNN}: [세션 제목]"));
    }

    #[test]
    fn legacy_field_extracts_value() {
        assert_eq!(header_status("# T\n\n> **Status**: ★ 활성 (x)\\\n").as_deref(), Some("★ 활성 (x)"));
        assert_eq!(legacy_field("# T\n> **Handoff**: a; 미완료 = b\n", "Handoff").as_deref(), Some("a; 미완료 = b"));
        assert_eq!(header_status("no status here\n"), None);
    }

    #[test]
    fn handoff_pending_detection() {
        assert_eq!(handoff_pending("A 기각, B 채택; 미완료 = X, Y 반영; 참조 t07").as_deref(), Some("X, Y 반영"));
        assert_eq!(handoff_pending("state; pending: fix docs; refs t02").as_deref(), Some("fix docs"));
        assert_eq!(handoff_pending("상태; 미완료 = X").as_deref(), Some("X"));
        assert_eq!(handoff_pending("상태; 미완료 -; 참조 t01"), None);
        assert_eq!(handoff_pending("상태; 미완료 없음; 참조"), None);
        assert_eq!(handoff_pending("state; pending: none; refs"), None);
        assert_eq!(handoff_pending("-"), None);
        assert_eq!(handoff_pending("상태 한 덩어리 서술"), None);
        assert_eq!(handoff_pending("상태; 미착수 작업들; 파일"), None);
        assert_eq!(handoff_pending("A 확정; B 결정; 미완료 = X; 참조 t1").as_deref(), Some("X"));
        assert_eq!(handoff_pending("Handoff 미완료 소거 규약 반영 완료; -; 참조 t14"), None);
        assert_eq!(handoff_pending("미완료 = t01 Phase 1 확인 대기").as_deref(), Some("t01 Phase 1 확인 대기"));
        assert_eq!(handoff_pending("  미완료 없음; 참조 t01"), None);
        assert_eq!(handoff_pending("완료; 미완료 = 없음(다음 세션 후보로 이관); 참조 t02"), None);
        assert_eq!(handoff_pending("완료; 미완료 — 없음"), None);
    }

    #[test]
    fn parse_registry_rows_and_escalations() {
        let text = format!("{HEADER}\nS001\t2026-01-01\tFoo\t★ 활성\t-\t-\n\nS002\t2026-01-02\tBar\tComplete\tkey\tArchive/S002_log.md\n");
        let rows = parse_registry(&text).unwrap();
        assert_eq!((rows.len(), rows[1].title.as_str(), rows[1].line), (2, "Bar", 4));
        let e = parse_registry(&format!("{HEADER}\nS001\t2026-01-01\tFoo\t★ 활성\n")).unwrap_err();
        assert_eq!(e.line, 2);
        assert!(e.to_string().lines().any(|l| l.starts_with("agent-action:")));
        assert!(parse_registry(&format!("{HEADER}\nX001\ta\tb\tc\td\te\n")).unwrap_err().found.contains("X001"));
        assert!(parse_registry("").unwrap().is_empty());
        assert_eq!(session_num("S007"), Some(7));
        assert_eq!(log_num("S007_log.md"), Some(7));
        assert_eq!(log_num("notes.md"), None);
    }
}
