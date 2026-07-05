//! `elf session new` / (t02) `session close` — 세션 수명주기 (P012 t01·t02).
//!
//! Registry TSV 파서는 t04 validate와 공용 — **위치 기반(6열) 결정론 파싱**.
//! 파싱 불가(열수·세션ID 형식)는 **자동 수정하지 않고 escalation**(exit 5)으로 상위 LLM 에이전트에 위임
//! (P012 §0.1: 결정론 도구=게이트, 판단·수리=에이전트. 무손실 원칙).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::embed;

pub const REGISTRY_REL: &str = "2_Log/Wiki/Session_Registry.tsv";
pub const REGISTRY_COLS: usize = 6;
const ACTION_FIX_REGISTRY: &str =
    "표시된 행을 스키마에 맞게 수정 후 명령 재실행 (이 도구는 Registry를 자동 수정하지 않음)";

/// 판단 개입 필요 — 상위 LLM 에이전트 위임 신호 (P012 §0.1, exit 5).
/// stderr에 자가완결 보고 + `agent-action:` 고정 마커 출력(에이전트가 패턴 인식해 수리·재실행).
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

/// Registry TSV → 행 목록. 헤더(`Session\t…`)·빈 줄 skip. 6열 아니거나 세션ID가 S### 아니면 escalation.
/// (Err은 박싱 — Escalation 구조가 커 result_large_err 회피)
pub fn parse_registry(text: &str) -> Result<Vec<RegistryRow>, Box<Escalation>> {
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let lineno = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
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

/// 다음 세션 번호 = 알려진 모든 번호의 max + 1 (순수).
pub fn next_number(nums: &[u32]) -> u32 {
    nums.iter().copied().max().map_or(1, |m| m + 1)
}

/// gap·중복·로그↔Registry 불일치 경고 (순수, 비차단). 상세 진단은 `elf validate`.
pub fn number_warnings(log_nums: &[u32], reg_nums: &[u32]) -> Vec<String> {
    let mut w = Vec::new();
    let logs: BTreeSet<u32> = log_nums.iter().copied().collect();
    let regs: BTreeSet<u32> = reg_nums.iter().copied().collect();
    if logs.len() != log_nums.len() {
        w.push("duplicate session log file numbers (run `elf validate`)".into());
    }
    for n in logs.difference(&regs) {
        w.push(format!("S{n:03} log exists but not in registry (run `elf validate`)"));
    }
    for n in regs.difference(&logs) {
        w.push(format!("S{n:03} in registry but no log file (run `elf validate`)"));
    }
    let all: BTreeSet<u32> = logs.union(&regs).copied().collect();
    if let Some(&mx) = all.iter().max() {
        for n in 1..mx {
            if !all.contains(&n) {
                w.push(format!("gap: S{n:03} missing (run `elf validate`)"));
            }
        }
    }
    w
}

pub struct SessionNewOptions {
    pub title: String,
    /// YYYY-MM-DD — 주입형(테스트 결정성; main이 오늘 날짜 주입)
    pub date: String,
}

#[derive(Debug)]
pub struct SessionNewResult {
    pub id: String,
    pub log_rel: String,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum SessionError {
    Escalation(Box<Escalation>),
    /// 계산된 로그 파일이 이미 존재 — refuse (exit 3)
    Exists(String),
    /// 제목에 탭(TSV 파탄) — refuse
    BadTitle(String),
    /// close: 닫을 활성 세션 없음
    NoOpenSession,
    /// close: 활성 세션 다수 — 명시 필요
    MultipleOpen(Vec<String>),
    /// close: 지정 세션 없음
    NotFound(String),
    /// close: '다음 세션 후보' 미작성 (refuse, --force로 우회)
    MissingNextSection(String),
    Io(io::Error),
}

impl From<io::Error> for SessionError {
    fn from(e: io::Error) -> Self {
        SessionError::Io(e)
    }
}

/// 2_Log/ + 2_Log/Archive/ 의 S###_log.md 번호 수집.
fn scan_log_numbers(root: &Path) -> Vec<u32> {
    let mut nums = Vec::new();
    for dir in ["2_Log", "2_Log/Archive"] {
        if let Ok(entries) = fs::read_dir(root.join(dir)) {
            for e in entries.flatten() {
                if let Some(n) = e.file_name().to_str().and_then(log_num) {
                    nums.push(n);
                }
            }
        }
    }
    nums
}

fn registry_header() -> &'static str {
    embed::TEMPLATES
        .get_file("log/Session_Registry.tsv")
        .and_then(|f| f.contents_utf8())
        .and_then(|s| s.lines().next())
        .unwrap_or("Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path")
}

/// `elf session new`: S### 자동 증번 → 로그 생성 + Registry 행 추가.
pub fn run_session_new(root: &Path, opts: &SessionNewOptions) -> Result<SessionNewResult, SessionError> {
    if opts.title.contains('\t') {
        return Err(SessionError::BadTitle(
            "session title must not contain a tab character (breaks the TSV registry)".into(),
        ));
    }

    let reg_path = root.join(REGISTRY_REL);
    let reg_text = fs::read_to_string(&reg_path).unwrap_or_default();
    let rows = parse_registry(&reg_text).map_err(SessionError::Escalation)?;

    let log_nums = scan_log_numbers(root);
    let reg_nums: Vec<u32> = rows.iter().filter_map(|r| session_num(&r.session)).collect();
    let warnings = number_warnings(&log_nums, &reg_nums);

    let n = next_number(&[log_nums.as_slice(), reg_nums.as_slice()].concat());
    let id = format!("S{n:03}");
    let log_rel = format!("2_Log/{id}_log.md");
    let log_path = root.join(&log_rel);
    if log_path.exists() {
        return Err(SessionError::Exists(log_rel));
    }

    // 템플릿 렌더 (managed sessionTemplate — embed 정본)
    let tpl = embed::TEMPLATES
        .get_file("log/sessionTemplate.md")
        .and_then(|f| f.contents_utf8())
        .expect("sessionTemplate embedded (gated by tests)");
    let content = tpl
        .replace("S{NNN}", &id)
        .replace("YYYY-MM-DD", &opts.date)
        .replace("[세션 제목]", &opts.title);
    fs::write(&log_path, content)?;

    // Registry 행 추가 (기존 내용 보존, 끝에 append)
    let mut out = reg_text;
    if out.is_empty() {
        out.push_str(registry_header());
        out.push('\n');
    } else if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("{id}\t{}\t{}\t★ 활성\t-\t-\n", opts.date, opts.title));
    fs::write(&reg_path, out)?;

    Ok(SessionNewResult { id, log_rel, warnings })
}

// ── 헤더 hard-break 보정 (codemod) ──────────────────────────────
//
// 세션 로그 헤더(첫 h1 뒤 첫 `>` 블록)의 각 줄(마지막 제외) 끝에 CommonMark hard break(`\`).
// Discord 미리보기 등 strict 렌더러에서 6개 메타 항목이 한 줄로 합쳐지는 것 방지(LogConvention §2).
// 멱등 — 이미 `\`면 건드리지 않음. 다른 `>` 블록·본문 불변.

/// 변경 시 보정된 전체 문자열, 변경 없으면 None.
pub fn fix_session_header(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let title = lines.iter().position(|l| l.starts_with("# "))?;

    let mut start = title + 1;
    while start < lines.len() && lines[start].trim().is_empty() {
        start += 1;
    }
    if start >= lines.len() || !lines[start].starts_with('>') {
        return None;
    }
    let mut end = start;
    while end < lines.len() && lines[end].starts_with('>') {
        end += 1;
    }
    // end-1 = 블록 마지막 줄(Handoff) → `\` 부여 제외

    let mut out: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let mut changed = false;
    for line in out.iter_mut().take(end.saturating_sub(1)).skip(start) {
        // CRLF-safe: split('\n')이 남긴 줄 끝 '\r'을 분리해 '\'를 그 앞(=줄 내용 끝)에 삽입
        let had_cr = line.ends_with('\r');
        let core_len = line.len() - usize::from(had_cr);
        {
            let core = &line[..core_len];
            let body = core.trim_start_matches('>').trim();
            if body.is_empty() || core.ends_with('\\') {
                continue; // bare '>' 또는 이미 hard break
            }
        }
        line.insert(core_len, '\\');
        changed = true;
    }
    if changed { Some(out.join("\n")) } else { None }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FixedFile {
    pub path: String,
    pub changed: bool,
}

/// `base/2_Log/*.md` + `base/2_Log/Archive/*.md`에 헤더 보정 적용(또는 dry-run 집계).
pub fn run_fix_headers(base: &Path, dry_run: bool) -> io::Result<Vec<FixedFile>> {
    let mut out = Vec::new();
    for dir in ["2_Log", "2_Log/Archive"] {
        let Ok(entries) = fs::read_dir(base.join(dir)) else {
            continue;
        };
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .collect();
        paths.sort();
        for p in paths {
            let content = fs::read_to_string(&p)?;
            if let Some(fixed) = fix_session_header(&content) {
                if !dry_run {
                    fs::write(&p, fixed)?;
                }
                out.push(FixedFile {
                    path: format!("{dir}/{}", p.file_name().unwrap_or_default().to_string_lossy()),
                    changed: true,
                });
            }
        }
    }
    Ok(out)
}

// ── session close (t02) + 공용 루트 탐지 ────────────────────────
//
// 루트 탐지 = `2_Log/` 기반(`.elf/` 불요). session new/close·fix-headers 공용 →
// 생성 프로젝트뿐 아니라 프레임워크 `_dev/`도 자기사용 가능(S011 도그푸딩 발견 반영).

/// cwd에서 위로 올라가며 `2_Log/` 보유 디렉토리(세션 루트)를 찾음.
pub fn find_log_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|a| a.join("2_Log").is_dir())
        .map(Path::to_path_buf)
}

/// 헤더 블록 `> **Status**:` 값 (트레일링 `\`·`\r` 제거).
pub fn header_status(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.trim_end_matches('\r')
            .strip_prefix("> **Status**:")
            .map(|v| v.trim_end_matches('\\').trim().to_string())
    })
}

/// Complete가 아니면 열린(닫을 수 있는) 세션. (trial.rs 활성 세션 탐지와 공용)
pub(crate) fn is_open_status(s: &str) -> bool {
    !s.starts_with("Complete")
}

/// `## 다음 세션 후보` 섹션에 placeholder(`- [...]`) 아닌 실제 bullet이 1개 이상이면 true.
/// (LogConvention §5.2 — 섹션 존재 + 작성 완료 게이트)
pub fn next_section_filled(content: &str) -> bool {
    let mut in_section = false;
    for raw in content.lines() {
        let l = raw.trim_end_matches('\r');
        if l.starts_with("## 다음 세션 후보") {
            in_section = true;
            continue;
        }
        if in_section && l.starts_with("## ") {
            break;
        }
        if in_section
            && let Some(rest) = l.trim().strip_prefix("- ")
        {
            let rest = rest.trim();
            if !rest.is_empty() && !rest.starts_with('[') {
                return true;
            }
        }
    }
    false
}

/// 헤더 Status를 Complete로 (트레일링 `\`·`\r` 보존). 변경 없으면 None.
pub fn mark_status_complete(content: &str) -> Option<String> {
    let mut out: Vec<String> = content.split('\n').map(str::to_string).collect();
    let mut changed = false;
    for line in out.iter_mut() {
        let trimmed = line.strip_suffix('\r').unwrap_or(line);
        if !trimmed.starts_with("> **Status**:") {
            continue;
        }
        let mut new = String::from("> **Status**: Complete");
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

/// close 시 Registry 행 갱신: status→Complete, archive_path→Archive/<id>_log.md. 타 행·EOL 보존.
fn registry_mark_closed(reg_text: &str, target: &str) -> String {
    let archive = format!("Archive/{target}_log.md");
    let mut out = String::new();
    for piece in reg_text.split_inclusive('\n') {
        let (body, nl) = match piece.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (piece, ""),
        };
        let core = body.strip_suffix('\r').unwrap_or(body);
        let cr = if body.ends_with('\r') { "\r" } else { "" };
        let cols: Vec<&str> = core.split('\t').collect();
        if cols.len() == REGISTRY_COLS && cols[0] == target {
            out.push_str(&format!(
                "{}\t{}\t{}\tComplete\t{}\t{archive}{cr}{nl}",
                cols[0], cols[1], cols[2], cols[4]
            ));
        } else {
            out.push_str(piece);
        }
    }
    out
}

/// 로그가 `2_Log/` → `2_Log/Archive/`로 **한 단계 깊어질** 때, **`../`로 시작하는**(= 2_Log 밖
/// 고정 대상을 가리키는) 상대 링크에만 `../`를 1개 prepend해 동일 대상을 유지. 변경 없으면 None.
/// bare same-dir 링크(`](S###_log.md)` 형제 로그 — 함께 이동)·산문 속 `](...)` 오탐은 자동 회피.
/// (prefix폐지 "단순 이동"이 누락한 상대링크 깊이 보정 — close 전용.)
pub fn deepen_relative_links(content: &str) -> Option<String> {
    let b = content.as_bytes();
    let mut out = String::with_capacity(content.len() + 64);
    let mut i = 0;
    let mut last = 0;
    let mut changed = false;
    while i + 1 < b.len() {
        if b[i] == b']'
            && b[i + 1] == b'('
            && let Some(close) = content[i + 2..].find(')')
        {
            let raw = &content[i + 2..i + 2 + close];
            if let Some(adj) = deepen_link_target(raw) {
                out.push_str(&content[last..i + 2]); // "...]("
                out.push_str(&adj);
                last = i + 2 + close; // ')' 위치
                changed = true;
            }
            i = i + 2 + close + 1;
            continue;
        }
        i += 1;
    }
    if changed {
        out.push_str(&content[last..]);
        Some(out)
    } else {
        None
    }
}

/// `../`로 시작하는 상대 링크 대상에만 `../` prepend한 새 문자열, 아니면 None.
fn deepen_link_target(raw: &str) -> Option<String> {
    let token = raw.split_whitespace().next().unwrap_or("");
    if token.starts_with("../") {
        Some(format!("../{raw}"))
    } else {
        None
    }
}

pub struct CloseOptions {
    pub id: Option<String>,
    pub force: bool,
}

#[derive(Debug)]
pub struct CloseResult {
    pub id: String,
    pub archived_to: String,
}

/// `elf session close [id]`: Status→Complete + Archive 이동(파일명 그대로) + Registry 갱신.
pub fn run_session_close(root: &Path, opts: &CloseOptions) -> Result<CloseResult, SessionError> {
    let reg_path = root.join(REGISTRY_REL);
    let reg_text = fs::read_to_string(&reg_path).unwrap_or_default();
    parse_registry(&reg_text).map_err(SessionError::Escalation)?; // 손상 시 escalation

    let target = match &opts.id {
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

    let log_path = root.join(format!("2_Log/{target}_log.md"));
    if !log_path.is_file() {
        return Err(SessionError::NotFound(target));
    }
    let content = fs::read_to_string(&log_path)?;

    if !opts.force && !next_section_filled(&content) {
        return Err(SessionError::MissingNextSection(target));
    }

    let archive_rel = format!("2_Log/Archive/{target}_log.md");
    let archive_path = root.join(&archive_rel);
    if archive_path.exists() {
        return Err(SessionError::Exists(archive_rel));
    }

    let updated = mark_status_complete(&content).unwrap_or(content);
    // 2_Log → 2_Log/Archive 한 단계 깊어짐 → 상대 cross-ref에 ../ 1개 보정
    let updated = deepen_relative_links(&updated).unwrap_or(updated);
    if let Some(dir) = archive_path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&archive_path, updated)?;
    fs::remove_file(&log_path)?;
    fs::write(&reg_path, registry_mark_closed(&reg_text, &target))?;

    Ok(CloseResult { id: target, archived_to: archive_rel })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path";

    #[test]
    fn header_status_extracts_value() {
        assert_eq!(
            header_status("# T\n\n> **Status**: ★ 활성 (x)\\\n").as_deref(),
            Some("★ 활성 (x)")
        );
        assert_eq!(header_status("# T\n\n> **Status**: Complete\n").as_deref(), Some("Complete"));
        assert_eq!(header_status("no status here\n"), None);
    }

    #[test]
    fn is_open_distinguishes_complete() {
        assert!(is_open_status("★ 활성"));
        assert!(is_open_status("In Progress"));
        assert!(!is_open_status("Complete"));
        assert!(!is_open_status("Complete (archived)"));
    }

    #[test]
    fn mark_status_complete_preserves_hardbreak_and_idempotent() {
        let input = "# T\n\n> **Created**: x\\\n> **Status**: ★ 활성 (y)\\\n> **Handoff**: -\n";
        let out = mark_status_complete(input).unwrap();
        assert!(out.contains("> **Status**: Complete\\\n"));
        assert!(out.contains("> **Created**: x\\\n")); // 타 줄 불변
        assert!(mark_status_complete(&out).is_none());
    }

    #[test]
    fn next_section_filled_detects_real_bullets() {
        assert!(!next_section_filled("## 다음 세션 후보\n\n### 가설 후보\n- [후속 가설 1-3항]\n"));
        assert!(next_section_filled("## 다음 세션 후보\n\n### 가설 후보\n- 실제 가설\n"));
        assert!(!next_section_filled("no section\n- 실제 가설\n")); // 섹션 밖 bullet 무시
    }

    #[test]
    fn deepen_prepends_dotdot_to_updir_links_only() {
        let input = "[P](../1_Concept/P.md) [img](../../x/y.png) [ext](https://e.com/z) [abs](/a.md) [a](#sec) [t](../q.md \"제목\") [sib](S011_log.md)";
        let out = deepen_relative_links(input).unwrap();
        assert!(out.contains("[P](../../1_Concept/P.md)"));
        assert!(out.contains("[img](../../../x/y.png)"));
        assert!(out.contains("[t](../../q.md \"제목\")")); // 제목 보존
        assert!(out.contains("[ext](https://e.com/z)")); // URL 불변
        assert!(out.contains("[abs](/a.md)")); // 절대 불변
        assert!(out.contains("[a](#sec)")); // 앵커 불변
        assert!(out.contains("[sib](S011_log.md)")); // bare 형제 링크 불변(함께 이동)
    }

    #[test]
    fn deepen_noop_without_updir_links() {
        // bare same-dir 링크·산문 속 `](...)`·URL·앵커는 보정 안 함
        assert!(deepen_relative_links("`](` 없는 비링크 설명(AC) [sib](S005_log.md) [x](https://a.b)\n").is_none());
    }

    #[test]
    fn registry_mark_closed_updates_only_target() {
        let reg = format!("{HEADER}\nS001\t2026-01-01\tA\t★ 활성\t-\t-\nS002\t2026-01-02\tB\t★ 활성\tkey\t-\n");
        let out = registry_mark_closed(&reg, "S001");
        assert!(out.contains("S001\t2026-01-01\tA\tComplete\t-\tArchive/S001_log.md\n"));
        assert!(out.contains("S002\t2026-01-02\tB\t★ 활성\tkey\t-\n")); // 불변
    }

    #[test]
    fn fix_header_adds_hardbreaks_except_last() {
        let input = "# S001: Title\n\n> **Created**: 2026-01-01\n> **Status**: active\n> **Handoff**: -\n\n---\nbody\n";
        let out = fix_session_header(input).unwrap();
        assert!(out.contains("> **Created**: 2026-01-01\\\n"));
        assert!(out.contains("> **Status**: active\\\n"));
        assert!(out.contains("> **Handoff**: -\n")); // 마지막: `\` 없음
        assert!(out.ends_with("---\nbody\n")); // 본문 불변
        assert!(fix_session_header(&out).is_none()); // 멱등
    }

    #[test]
    fn fix_header_only_first_block() {
        let input = "# S001: T\n\n> **Created**: x\n> **Handoff**: -\n\n> separate note\n\n## t01\n";
        let out = fix_session_header(input).unwrap();
        assert!(out.contains("> **Created**: x\\\n"));
        assert!(out.contains("> separate note\n")); // 별도 블록 불변
    }

    #[test]
    fn fix_header_crlf_safe() {
        let input = "# S1: T\r\n\r\n> **Created**: x\r\n> **Handoff**: -\r\n\r\n---\r\n";
        let out = fix_session_header(input).unwrap();
        assert!(out.contains("> **Created**: x\\\r\n")); // `\`는 \r 앞 (줄 끝)
        assert!(out.contains("> **Handoff**: -\r\n")); // 마지막: `\` 없음
        assert!(!out.contains("\r\\")); // \r 뒤 backslash(corruption) 없음
        assert!(fix_session_header(&out).is_none()); // 멱등
    }

    #[test]
    fn fix_header_noop_cases() {
        assert!(fix_session_header("no title\njust text\n").is_none());
        assert!(fix_session_header("# Title\n\nbody, no blockquote\n").is_none());
        // 이미 보정됨
        assert!(fix_session_header("# T\n\n> **a**: 1\\\n> **b**: 2\n").is_none());
    }

    #[test]
    fn parse_valid_registry_skips_header() {
        let text = format!("{HEADER}\nS001\t2026-01-01\tFoo\t★ 활성\t-\t-\nS002\t2026-01-02\tBar\tComplete\tkey\tArchive/S002_log.md\n");
        let rows = parse_registry(&text).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].session, "S001");
        assert_eq!(rows[1].title, "Bar");
        assert_eq!(rows[1].line, 3);
    }

    #[test]
    fn parse_blank_lines_ignored() {
        let text = format!("{HEADER}\n\nS001\t2026-01-01\tFoo\t★ 활성\t-\t-\n\n");
        assert_eq!(parse_registry(&text).unwrap().len(), 1);
    }

    #[test]
    fn parse_wrong_column_count_escalates_with_line() {
        let text = format!("{HEADER}\nS001\t2026-01-01\tFoo\t★ 활성\n"); // 4 cols
        let e = parse_registry(&text).unwrap_err();
        assert_eq!(e.line, 2);
        assert!(e.found.contains("4 columns"));
        assert_eq!(e.file, REGISTRY_REL);
        assert!(e.to_string().contains("agent-action:"));
    }

    #[test]
    fn parse_bad_session_id_escalates() {
        let text = format!("{HEADER}\nX001\t2026-01-01\tFoo\t★ 활성\t-\t-\n");
        let e = parse_registry(&text).unwrap_err();
        assert!(e.expected.contains("S###"));
        assert!(e.found.contains("X001"));
    }

    #[test]
    fn empty_registry_is_ok() {
        assert!(parse_registry("").unwrap().is_empty());
        assert!(parse_registry(HEADER).unwrap().is_empty());
    }

    #[test]
    fn session_and_log_num() {
        assert_eq!(session_num("S007"), Some(7));
        assert_eq!(session_num("S000"), Some(0));
        assert_eq!(session_num("S12"), Some(12));
        assert_eq!(session_num("X1"), None);
        assert_eq!(session_num("S"), None);
        assert_eq!(session_num("S1a"), None);
        assert_eq!(log_num("S007_log.md"), Some(7));
        assert_eq!(log_num("notes.md"), None);
    }

    #[test]
    fn next_number_is_max_plus_one() {
        assert_eq!(next_number(&[]), 1);
        assert_eq!(next_number(&[1]), 2);
        assert_eq!(next_number(&[1, 2, 5]), 6);
        assert_eq!(next_number(&[3, 1, 2]), 4);
    }

    #[test]
    fn warnings_detect_mismatch_and_gap() {
        // 로그엔 1,3 / Registry엔 1,2 → 3은 미등록, 2는 로그없음, gap 없음(2 존재)
        let w = number_warnings(&[1, 3], &[1, 2]);
        assert!(w.iter().any(|s| s.contains("S003") && s.contains("not in registry")));
        assert!(w.iter().any(|s| s.contains("S002") && s.contains("no log")));
        // gap: 로그 1,4 / Registry 1,4 → 2,3 누락
        let g = number_warnings(&[1, 4], &[1, 4]);
        assert!(g.iter().any(|s| s.contains("S002 missing")));
        assert!(g.iter().any(|s| s.contains("S003 missing")));
        // clean
        assert!(number_warnings(&[1, 2], &[1, 2]).is_empty());
    }

    #[test]
    fn escalation_format_has_action_marker_at_col0() {
        let e = Escalation {
            file: "x.tsv".into(),
            line: 7,
            expected: "6 cols".into(),
            found: "4 cols".into(),
            raw: "a\tb".into(),
            action: "fix it".into(),
        };
        let s = e.to_string();
        assert!(s.contains("escalation: x.tsv (line 7)"));
        assert!(s.lines().any(|l| l.starts_with("agent-action: fix it")));
    }
}
