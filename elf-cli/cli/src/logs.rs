//! 로그 위치와 생성 파일 — `2_Log/S/`, `2_Log/G/`(경로 고정), `Registry.md`, 탐색기 nesting (P022 §2, §4, §9).
//!
//! 로그는 상태와 무관하게 계열 폴더 하나에 있고 옮기지 않음. 상태 = 헤더 `status`(header.rs).
//! `Registry.md`와 `.vscode/settings.json`의 nesting 키는 헤더에서 만드는 생성 결과 — 로그를 바꾸는
//! 명령이 `refresh`로 다시 만듦.

use std::fs;
use std::path::{Path, PathBuf};

use crate::header::{self, Header, Status};
use crate::session::SessionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Series {
    S,
    G,
}

impl Series {
    pub const ALL: [Series; 2] = [Series::S, Series::G];
    pub fn letter(self) -> char {
        match self {
            Series::S => 'S',
            Series::G => 'G',
        }
    }
    /// 루트 기준 폴더.
    pub fn dir(self) -> &'static str {
        match self {
            Series::S => "2_Log/S",
            Series::G => "2_Log/G",
        }
    }
}

/// `S007`, `G004` → (계열, 번호).
pub fn parse_id(id: &str) -> Option<(Series, u32)> {
    let mut cs = id.chars();
    let series = match cs.next()? {
        'S' => Series::S,
        'G' => Series::G,
        _ => return None,
    };
    let digits = cs.as_str();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((series, digits.parse().ok()?))
}

pub fn id_of(series: Series, n: u32) -> String {
    format!("{}{n:03}", series.letter())
}

/// 루트 기준 로그 경로 `2_Log/S/S007_log.md`.
pub fn log_rel(series: Series, n: u32) -> String {
    format!("{}/{}_log.md", series.dir(), id_of(series, n))
}

/// `S007_log.md` → (S, 7).
pub fn log_file_id(name: &str) -> Option<(Series, u32)> {
    parse_id(name.strip_suffix("_log.md")?)
}

#[derive(Debug, Clone)]
pub struct LogFile {
    pub series: Series,
    pub num: u32,
    pub id: String,
    pub path: PathBuf,
}

/// 계열 폴더의 로그(번호 오름차순).
pub fn scan(root: &Path, series: Series) -> Vec<LogFile> {
    let mut v = Vec::new();
    if let Ok(rd) = fs::read_dir(root.join(series.dir())) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some((s, n)) = log_file_id(&name)
                && s == series
            {
                v.push(LogFile { series, num: n, id: id_of(series, n), path: e.path() });
            }
        }
    }
    v.sort_by_key(|l| l.num);
    v
}

pub fn scan_all(root: &Path) -> Vec<LogFile> {
    Series::ALL.iter().flat_map(|s| scan(root, *s)).collect()
}

/// 계열 폴더의 `*_log.md` 중 `S###`/`G###` 정격이 아닌 이름(예: `S201-a_log.md`).
pub fn malformed_names(root: &Path, series: Series) -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(rd) = fs::read_dir(root.join(series.dir())) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with("_log.md") && log_file_id(&name).is_none() {
                v.push(name);
            }
        }
    }
    v.sort();
    v
}

pub fn next_number(root: &Path, series: Series) -> u32 {
    scan(root, series).iter().map(|l| l.num).max().map_or(1, |m| m + 1)
}

/// 로그 파일을 읽어 (헤더, 전체 내용).
pub fn read(log: &LogFile) -> Result<(Header, String), SessionError> {
    let content = fs::read_to_string(&log.path)?;
    let (h, _) = header::parse(&content).map_err(|e| SessionError::BadHeader(format!("{}: {e}", rel_of(log))))?;
    Ok((h, content))
}

pub fn rel_of(log: &LogFile) -> String {
    log_rel(log.series, log.num)
}

/// 대상 세션: 지정하면 그 로그, 아니면 진행 중(`ongoing`) 세션이 하나일 때 그것.
pub fn resolve(root: &Path, id: Option<&str>) -> Result<LogFile, SessionError> {
    if let Some(id) = id {
        let (series, n) = parse_id(id).ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        let path = root.join(log_rel(series, n));
        if !path.is_file() {
            return Err(SessionError::NotFound(id.to_string()));
        }
        return Ok(LogFile { series, num: n, id: id_of(series, n), path });
    }
    let mut open = Vec::new();
    for l in scan_all(root) {
        let ongoing = fs::read_to_string(&l.path)
            .ok()
            .and_then(|c| header::parse(&c).ok())
            .is_some_and(|(h, _)| h.status == Status::Ongoing);
        if ongoing {
            open.push(l);
        }
    }
    match open.len() {
        0 => Err(SessionError::NoOpenSession),
        1 => Ok(open.pop().unwrap()),
        _ => Err(SessionError::MultipleOpen(open.into_iter().map(|l| l.id).collect())),
    }
}

/// 이전 버전의 로그 폴더(이관 대상). 2.4~2.27 = `2_Log/`와 `2_Log/Archive/`, 2.2~2.3 보관 = `2_Log/9_Archive/`,
/// 2.1 = `7_Log/`, 2.0 = `5_Exp/53_Analysis/Logs/`(그 뒤 번호 = `6_Exp/63_Analysis/Logs/`). `Deprecated/`는 따로(v3.0.2 S046).
pub const OLD_LOG_DIRS: [&str; 10] = [
    "2_Log",
    "2_Log/Archive",
    "2_Log/9_Archive",
    "7_Log",
    "7_Log/9_Archive",
    "5_Exp/53_Analysis/Logs",
    "5_Exp/53_Analysis/Logs/9_Archive",
    "6_Exp/63_Analysis/Logs",
    "6_Exp/63_Analysis/Logs/9_Archive",
    "2_Log/Deprecated",
];

/// 이전 버전의 Registry TSV 위치(2.4~2.27 = `2_Log/Wiki/`, 그 전 = `1_Wiki/`, `2_Wiki/`).
pub const OLD_REGISTRY_RELS: [&str; 6] = [
    "2_Log/Wiki/Session_Registry.tsv",
    "2_Log/1_Wiki/Session_Registry.tsv",
    "2_Log/2_Wiki/Session_Registry.tsv",
    "7_Log/2_Wiki/Session_Registry.tsv",
    "5_Exp/53_Analysis/Logs/2_Wiki/Session_Registry.tsv",
    "6_Exp/63_Analysis/Logs/2_Wiki/Session_Registry.tsv",
];

/// 이관 뒤 비면(`.gitkeep`만 남아도) 지우는 이전 버전 폴더 — 자식 먼저.
pub const OLD_EMPTY_DIRS: [&str; 14] = [
    "2_Log/Archive",
    "2_Log/Deprecated",
    "2_Log/9_Archive",
    "2_Log/1_Wiki",
    "2_Log/2_Wiki",
    "7_Log/9_Archive",
    "7_Log/2_Wiki",
    "7_Log",
    "5_Exp/53_Analysis/Logs/9_Archive",
    "5_Exp/53_Analysis/Logs/2_Wiki",
    "5_Exp/53_Analysis/Logs",
    "6_Exp/63_Analysis/Logs/9_Archive",
    "6_Exp/63_Analysis/Logs/2_Wiki",
    "6_Exp/63_Analysis/Logs",
];

/// 이관 전(구 형식) 흔적: 옛 로그 폴더의 루트 로그, 보관과 폐기 폴더, TSV Registry. 없으면 빈 목록.
pub fn legacy_markers(root: &Path) -> Vec<String> {
    let mut v = Vec::new();
    for d in OLD_LOG_DIRS {
        let Ok(rd) = fs::read_dir(root.join(d)) else { continue };
        let names: Vec<String> = rd
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        if !d.contains("Archive") && !d.ends_with("Deprecated") {
            // 루트 로그 폴더 = 로그 파일마다
            v.extend(names.iter().filter(|n| n.ends_with("_log.md")).map(|n| format!("{d}/{n}")));
        } else if names.iter().any(|n| n.ends_with("_log.md") || n.ends_with(".partial.md")) {
            // 보관, 폐기 폴더 = 로그(`*_log.md`)나 부분 폐기 파일(`*.partial.md`)이 있을 때만 — 이관 뒤 남은 다른 파일은 이전 형식이 아님
            v.push(format!("{d}/"));
        }
    }
    v.extend(OLD_REGISTRY_RELS.iter().filter(|r| root.join(r).is_file()).map(|r| r.to_string()));
    v.sort();
    v
}

// ── Registry.md ───────────────────────────────────────────────────

const REGISTRY_MARK: &str = "<!-- generated by elf from the log headers; do not edit (regenerated by every elf command that changes a log) -->";

fn korean(root: &Path) -> bool {
    fs::read_to_string(root.join(".elf/config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("lang").and_then(|l| l.as_str()).map(|l| l.to_ascii_lowercase().starts_with("ko")))
        .unwrap_or(true)
}

fn cell(v: &str) -> String {
    let v = header::one_line(v).replace('|', "\\|");
    if v.is_empty() { "-".into() } else { v }
}

/// 계열의 Registry.md 내용(순수 — 로그 목록과 헤더에서).
pub fn render_registry(series: Series, rows: &[(LogFile, Header)], ko: bool) -> String {
    let mut sorted: Vec<&(LogFile, Header)> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        let ka = (a.1.status != Status::Ongoing, std::cmp::Reverse(a.0.num));
        let kb = (b.1.status != Status::Ongoing, std::cmp::Reverse(b.0.num));
        ka.cmp(&kb)
    });
    let title = match (series, ko) {
        (Series::S, true) => "# 연구 세션 목록",
        (Series::G, true) => "# 거버넌스 세션 목록",
        (Series::S, false) => "# Research sessions",
        (Series::G, false) => "# Governance sessions",
    };
    let head = if ko { "| 세션 | status | 결론 또는 handoff |" } else { "| Session | status | conclusion or handoff |" };
    let mut out = format!("{REGISTRY_MARK}\n{title}\n\n{head}\n|---|---|---|\n");
    for (l, h) in sorted {
        let note = match h.status {
            Status::Complete => &h.conclusion,
            Status::Ongoing | Status::Deprecated => &h.handoff,
        };
        out.push_str(&format!(
            "| [{id}]({id}_log.md) | {} | {} |\n",
            h.status.as_str(),
            cell(note),
            id = l.id
        ));
    }
    out
}

/// 헤더를 읽을 수 있는 로그만 모아 (로그, 헤더). 읽을 수 없는 로그 이름은 두 번째 값.
pub fn headers(root: &Path, series: Series) -> (Vec<(LogFile, Header)>, Vec<String>) {
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for l in scan(root, series) {
        match fs::read_to_string(&l.path).ok().and_then(|c| header::parse(&c).ok()) {
            Some((h, _)) => ok.push((l, h)),
            None => bad.push(l.id.clone()),
        }
    }
    (ok, bad)
}

pub fn registry_rel(series: Series) -> String {
    format!("{}/Registry.md", series.dir())
}

/// 지금 헤더로 만든 Registry.md 내용(계열 폴더가 없으면 None).
pub fn expected_registry(root: &Path, series: Series) -> Option<String> {
    if !root.join(series.dir()).is_dir() {
        return None;
    }
    let (rows, _) = headers(root, series);
    Some(render_registry(series, &rows, korean(root)))
}

// ── 탐색기 nesting (.vscode/settings.json) ─────────────────────────

const NEST_ENABLED: &str = "explorer.fileNesting.enabled";
const NEST_PATTERNS: &str = "explorer.fileNesting.patterns";

/// 탐색기 nesting 항목(키, 값). 종료, 폐기 로그를 계열마다 번호순 사슬로 이음: `Registry.md` → 첫 종료 로그,
/// 앞 종료 로그 → 다음 종료 로그. VS Code는 같은 파일에 맞는 키의 자식을 합치고 자식의 자식을 맨 위 부모 아래로
/// 펼쳐 보이므로(`explorerFileNestingTrie`) 모두 `Registry.md` 바로 아래에 보이고, 한 줄에 로그 하나라 세션을
/// 닫을 때 `settings.json` diff가 1~2줄(SOMG 제안 D3, S047). 진행 중 로그는 사슬에 없어 그대로 보임.
pub fn nesting_entries(root: &Path) -> Vec<(String, String)> {
    let mut firsts = Vec::new();
    let mut links = Vec::new();
    for s in Series::ALL {
        let (rows, _) = headers(root, s);
        let closed: Vec<String> = rows.iter().filter(|(_, h)| h.status != Status::Ongoing).map(|(l, _)| format!("{}_log.md", l.id)).collect();
        if let Some(f) = closed.first() {
            firsts.push(f.clone());
        }
        links.extend(closed.windows(2).map(|w| (w[0].clone(), w[1].clone())));
    }
    let mut out = Vec::new();
    if !firsts.is_empty() {
        out.push(("Registry.md".to_string(), firsts.join(", ")));
    }
    out.extend(links);
    out
}

/// ELF가 만드는 nesting 키(`Registry.md`, 로그 파일 이름) — 그 밖의 키는 사용자 것.
fn elf_nest_key(k: &str) -> bool {
    k == "Registry.md" || log_file_id(k).is_some()
}

/// settings.json에 nesting 두 키만 반영(텍스트에서 그 부분만 바꿈 — 다른 설정의 순서, 들여쓰기, 바이트는 그대로).
/// 바뀌었으면 true. JSON으로 읽을 수 없으면(주석 등) 손대지 않고 Err.
fn write_nesting(root: &Path, entries: &[(String, String)]) -> Result<bool, String> {
    let p = root.join(".vscode/settings.json");
    // 접을 로그가 없고 설정 파일도 없으면 만들지 않음(새 프로젝트에 빈 설정 파일을 늘리지 않음)
    if entries.is_empty() && !p.exists() {
        return Ok(false);
    }
    let raw = fs::read_to_string(&p).unwrap_or_default();
    let text = if raw.trim().is_empty() { "{}\n".to_string() } else { raw.clone() };
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        format!(".vscode/settings.json is not plain JSON ({e}) — explorer nesting not updated; set \"{NEST_PATTERNS}\" by hand or remove comments")
    })?;
    if !v.is_object() {
        return Err(".vscode/settings.json is not a JSON object — explorer nesting not updated".into());
    }
    if v.get(NEST_PATTERNS).is_some_and(|x| !x.is_object()) {
        return Err(format!(".vscode/settings.json \"{NEST_PATTERNS}\" is not an object — explorer nesting not updated"));
    }
    let new = nesting_text(&text, entries).ok_or(".vscode/settings.json could not be edited in place — explorer nesting not updated")?;
    if new == raw {
        return Ok(false);
    }
    fs::create_dir_all(root.join(".vscode")).map_err(|e| e.to_string())?;
    fs::write(&p, new).map_err(|e| e.to_string())?;
    Ok(true)
}

/// settings.json 텍스트(유효한 JSON 객체)에서 `explorer.fileNesting.enabled` = true와 `explorer.fileNesting.patterns`만
/// 바꾼 결과. patterns = 사용자 키(원문, 원래 순서) + ELF 항목, 한 줄에 하나. 그 밖의 바이트는 그대로.
fn nesting_text(text: &str, entries: &[(String, String)]) -> Option<String> {
    let b = text.as_bytes();
    let open = jt::skip_ws(b, 0);
    if b.get(open) != Some(&b'{') {
        return None;
    }
    let (members, close) = jt::members(text, open)?;
    let ind = members.first().map(|m| jt::indent_at(text, m.start)).filter(|s| !s.is_empty()).unwrap_or_else(|| "  ".into());
    let quote = |s: &str| serde_json::to_string(s).expect("string");
    let pat = members.iter().find(|m| m.key == NEST_PATTERNS);
    let mut lines: Vec<String> = Vec::new();
    if let Some(m) = pat {
        let (inner, _) = jt::members(text, m.value.0)?;
        lines.extend(inner.iter().filter(|u| !elf_nest_key(&u.key)).map(|u| text[u.start..u.value.1].to_string()));
    }
    lines.extend(entries.iter().map(|(k, v)| format!("{}: {}", quote(k), quote(v))));
    let mind = pat.map(|m| jt::indent_at(text, m.start)).filter(|s| !s.is_empty()).unwrap_or_else(|| ind.clone());
    let obj = if lines.is_empty() {
        "{}".to_string()
    } else {
        format!("{{\n{}\n{mind}}}", lines.iter().map(|l| format!("{mind}{ind}{l}")).collect::<Vec<_>>().join(",\n"))
    };
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut inserts: Vec<String> = Vec::new();
    match members.iter().find(|m| m.key == NEST_ENABLED) {
        Some(m) if &text[m.value.0..m.value.1] == "true" => {}
        Some(m) => edits.push((m.value.0, m.value.1, "true".into())),
        None => inserts.push(format!("{}: true", quote(NEST_ENABLED))),
    }
    match pat {
        Some(m) => edits.push((m.value.0, m.value.1, obj)),
        None => inserts.push(format!("{}: {obj}", quote(NEST_PATTERNS))),
    }
    if !inserts.is_empty() {
        match members.last() {
            Some(last) => edits.push((last.value.1, last.value.1, inserts.iter().map(|s| format!(",\n{ind}{s}")).collect())),
            None => edits.push((open + 1, close, format!("\n{}\n", inserts.iter().map(|s| format!("{ind}{s}")).collect::<Vec<_>>().join(",\n")))),
        }
    }
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = text.to_string();
    for (s, e, r) in edits {
        out.replace_range(s..e, &r);
    }
    Some(out)
}

/// 유효한 JSON 텍스트에서 객체 멤버의 위치를 찾는 최소 스캐너(값 해석은 serde_json에 맡김).
mod jt {
    pub struct Member {
        pub key: String,
        /// 키 따옴표 위치
        pub start: usize,
        /// 값 [시작, 끝)
        pub value: (usize, usize),
    }

    pub fn skip_ws(b: &[u8], mut i: usize) -> usize {
        while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
        }
        i
    }

    /// `i` = 여는 따옴표 → 닫는 따옴표 다음 위치.
    fn string_end(b: &[u8], mut i: usize) -> Option<usize> {
        i += 1;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                b'"' => return Some(i + 1),
                _ => i += 1,
            }
        }
        None
    }

    /// 값의 끝(다음 위치): 객체와 배열 = 짝 괄호(문자열 안 무시), 문자열, 그 밖 = 구분자나 공백 전까지.
    fn value_end(b: &[u8], i: usize) -> Option<usize> {
        match *b.get(i)? {
            b'"' => string_end(b, i),
            b'{' | b'[' => {
                let mut depth = 0usize;
                let mut j = i;
                while j < b.len() {
                    match b[j] {
                        b'"' => {
                            j = string_end(b, j)?;
                            continue;
                        }
                        b'{' | b'[' => depth += 1,
                        b'}' | b']' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(j + 1);
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                None
            }
            _ => {
                let mut j = i;
                while j < b.len() && !matches!(b[j], b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n') {
                    j += 1;
                }
                Some(j)
            }
        }
    }

    /// `open` = `{` 위치 → (멤버들, 닫는 `}` 위치).
    pub fn members(text: &str, open: usize) -> Option<(Vec<Member>, usize)> {
        let b = text.as_bytes();
        let mut v = Vec::new();
        let mut i = skip_ws(b, open + 1);
        if b.get(i) == Some(&b'}') {
            return Some((v, i));
        }
        loop {
            if b.get(i) != Some(&b'"') {
                return None;
            }
            let ke = string_end(b, i)?;
            let key: String = serde_json::from_str(&text[i..ke]).ok()?;
            let mut j = skip_ws(b, ke);
            if b.get(j) != Some(&b':') {
                return None;
            }
            j = skip_ws(b, j + 1);
            let ve = value_end(b, j)?;
            v.push(Member { key, start: i, value: (j, ve) });
            let k = skip_ws(b, ve);
            match *b.get(k)? {
                b',' => i = skip_ws(b, k + 1),
                b'}' => return Some((v, k)),
                _ => return None,
            }
        }
    }

    /// `pos`가 있는 줄의 앞 공백.
    pub fn indent_at(text: &str, pos: usize) -> String {
        let line_start = text[..pos].rfind('\n').map(|n| n + 1).unwrap_or(0);
        text[line_start..pos].chars().take_while(|c| *c == ' ' || *c == '\t').collect()
    }
}

/// 로그를 바꾼 뒤 생성 결과를 다시 만듦: 계열별 Registry.md(폴더가 있을 때), 탐색기 nesting.
/// 반환 = 경고(헤더를 읽을 수 없는 로그, settings.json 병합 실패). 쓰기 오류는 Err.
pub fn refresh(root: &Path) -> std::io::Result<Vec<String>> {
    let mut warns = Vec::new();
    for s in Series::ALL {
        if !root.join(s.dir()).is_dir() {
            continue;
        }
        let (rows, bad) = headers(root, s);
        for id in bad {
            warns.push(format!("{id}: header unreadable — left out of {} (run `elf check logs`)", registry_rel(s)));
        }
        let text = render_registry(s, &rows, korean(root));
        let p = root.join(registry_rel(s));
        if fs::read_to_string(&p).ok().as_deref() != Some(text.as_str()) {
            fs::write(&p, text)?;
        }
    }
    if let Err(w) = write_nesting(root, &nesting_entries(root)) {
        warns.push(w);
    }
    Ok(warns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(status: Status, handoff: &str, conclusion: &str) -> Header {
        let mut x = Header::new("2026-11-09", "g");
        x.status = status;
        x.handoff = handoff.into();
        x.conclusion = conclusion.into();
        x
    }

    fn lf(n: u32) -> LogFile {
        LogFile { series: Series::S, num: n, id: id_of(Series::S, n), path: PathBuf::new() }
    }

    #[test]
    fn ids_and_paths() {
        assert_eq!(parse_id("S007"), Some((Series::S, 7)));
        assert_eq!(parse_id("G012"), Some((Series::G, 12)));
        assert_eq!(parse_id("P001"), None);
        assert_eq!(parse_id("S"), None);
        assert_eq!(log_rel(Series::G, 4), "2_Log/G/G004_log.md");
        assert_eq!(log_file_id("S201-a_log.md"), None);
    }

    #[test]
    fn registry_orders_ongoing_first_then_desc_and_picks_column() {
        let rows = vec![
            (lf(1), h(Status::Complete, "-", "결론 A | B")),
            (lf(3), h(Status::Deprecated, "폐기 사유", "")),
            (lf(2), h(Status::Ongoing, "상태; 미완료 = x", "")),
        ];
        let r = render_registry(Series::S, &rows, true);
        let body: Vec<&str> = r.lines().filter(|l| l.starts_with("| [")).collect();
        assert_eq!(body[0], "| [S002](S002_log.md) | ongoing | 상태; 미완료 = x |");
        assert_eq!(body[1], "| [S003](S003_log.md) | deprecated | 폐기 사유 |");
        assert_eq!(body[2], "| [S001](S001_log.md) | complete | 결론 A \\| B |");
        assert!(r.starts_with("<!-- generated by elf"));
    }

    #[test]
    fn refresh_writes_registry_and_merges_nesting() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log/S")).unwrap();
        for (n, st) in [(1, Status::Complete), (2, Status::Ongoing)] {
            let mut x = h(st, "", "c");
            x.goal = format!("goal {n}");
            fs::write(root.join(log_rel(Series::S, n)), format!("{}# S00{n}: t\n", header::render(&x))).unwrap();
        }
        fs::create_dir_all(root.join(".vscode")).unwrap();
        fs::write(root.join(".vscode/settings.json"), "{\"editor.tabSize\": 2}").unwrap();
        assert!(refresh(root).unwrap().is_empty());
        let reg = fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap();
        assert!(reg.contains("[S002](S002_log.md) | ongoing"));
        assert!(!root.join("2_Log/G/Registry.md").exists(), "no G folder → no G registry");
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(root.join(".vscode/settings.json")).unwrap()).unwrap();
        assert_eq!(v["editor.tabSize"], 2);
        assert_eq!(v[NEST_PATTERNS]["Registry.md"], "S001_log.md");
        // 주석이 든 settings.json은 건드리지 않고 경고
        fs::write(root.join(".vscode/settings.json"), "{ // c\n}").unwrap();
        let w = refresh(root).unwrap();
        assert!(w.iter().any(|x| x.contains("not plain JSON")));
        assert_eq!(fs::read_to_string(root.join(".vscode/settings.json")).unwrap(), "{ // c\n}");
    }

    fn closed_logs(root: &Path, series: Series, items: &[(u32, Status)]) {
        fs::create_dir_all(root.join(series.dir())).unwrap();
        for (n, st) in items {
            let mut x = h(*st, "", "c");
            x.goal = format!("goal {n}");
            fs::write(root.join(log_rel(series, *n)), format!("{}# {}: t\n", header::render(&x), id_of(series, *n))).unwrap();
        }
    }

    /// 줄 단위 차이(바뀐 줄 수 = 빠진 줄 + 더한 줄, 같은 줄은 순서대로 맞춤).
    fn changed_lines(a: &str, b: &str) -> usize {
        let (x, y): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
        let mut lcs = vec![vec![0usize; y.len() + 1]; x.len() + 1];
        for i in (0..x.len()).rev() {
            for j in (0..y.len()).rev() {
                lcs[i][j] = if x[i] == y[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
            }
        }
        x.len() + y.len() - 2 * lcs[0][0]
    }

    #[test]
    fn nesting_is_a_chain_of_closed_logs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        closed_logs(root, Series::S, &[(1, Status::Complete), (2, Status::Ongoing), (3, Status::Deprecated), (4, Status::Complete)]);
        closed_logs(root, Series::G, &[(1, Status::Complete), (2, Status::Complete)]);
        let e = nesting_entries(root);
        let pairs: Vec<(&str, &str)> = e.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        assert_eq!(
            pairs,
            vec![
                ("Registry.md", "S001_log.md, G001_log.md"),
                ("S001_log.md", "S003_log.md"),
                ("S003_log.md", "S004_log.md"),
                ("G001_log.md", "G002_log.md"),
            ],
            "진행 중 S002는 사슬에 없음"
        );
    }

    #[test]
    fn nesting_edit_keeps_other_settings_and_user_patterns() {
        let before = "{\n\t\"editor.tabSize\": 4,\n\t\"explorer.fileNesting.patterns\": {\n\t\t\"*.ts\": \"${capture}.js\",\n\t\t\"Registry.md\": \"S001_log.md, S002_log.md\"\n\t},\n\t\"files.eol\": \"\\n\"\n}\n";
        let e = vec![("Registry.md".to_string(), "S001_log.md".to_string()), ("S001_log.md".to_string(), "S002_log.md".to_string())];
        let after = nesting_text(before, &e).unwrap();
        assert_eq!(
            after,
            "{\n\t\"editor.tabSize\": 4,\n\t\"explorer.fileNesting.patterns\": {\n\t\t\"*.ts\": \"${capture}.js\",\n\t\t\"Registry.md\": \"S001_log.md\",\n\t\t\"S001_log.md\": \"S002_log.md\"\n\t},\n\t\"files.eol\": \"\\n\",\n\t\"explorer.fileNesting.enabled\": true\n}\n"
        );
        assert_eq!(nesting_text(&after, &e).unwrap(), after, "같은 항목이면 그대로");
        // 새 파일과 빈 객체
        assert_eq!(
            nesting_text("{}\n", &e[..1]).unwrap(),
            "{\n  \"explorer.fileNesting.enabled\": true,\n  \"explorer.fileNesting.patterns\": {\n    \"Registry.md\": \"S001_log.md\"\n  }\n}\n"
        );
        // 접을 로그가 없어지면 ELF 항목만 빠지고 사용자 키는 남음
        let none = nesting_text(&after, &[]).unwrap();
        assert!(none.contains("\"*.ts\": \"${capture}.js\"") && !none.contains("Registry.md"), "{none}");
    }

    #[test]
    fn closing_a_session_changes_one_or_two_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut items: Vec<(u32, Status)> = (1..=40).map(|n| (n, if n == 7 || n >= 39 { Status::Ongoing } else { Status::Complete })).collect();
        closed_logs(root, Series::S, &items);
        assert!(refresh(root).unwrap().is_empty());
        let p = root.join(".vscode/settings.json");
        let a = fs::read_to_string(&p).unwrap();
        // 가장 최근 번호(S039)를 닫음 → 새 줄 1 + 앞 줄 쉼표 1
        items[38].1 = Status::Complete;
        closed_logs(root, Series::S, &items);
        refresh(root).unwrap();
        let b = fs::read_to_string(&p).unwrap();
        assert_eq!(changed_lines(&a, &b), 3, "-1 +2:\n{a}\n---\n{b}");
        // 오래 열어 둔 S007을 닫음 → 앞 줄 1 수정 + 새 줄 1
        items[6].1 = Status::Complete;
        closed_logs(root, Series::S, &items);
        refresh(root).unwrap();
        let c = fs::read_to_string(&p).unwrap();
        assert_eq!(changed_lines(&b, &c), 3, "-1 +2:\n{b}\n---\n{c}");
        assert!(c.contains("\"S006_log.md\": \"S007_log.md\"") && c.contains("\"S007_log.md\": \"S008_log.md\""), "{c}");
        // 새 세션(S041, 진행 중)은 바꾸지 않음
        items.push((41, Status::Ongoing));
        closed_logs(root, Series::S, &items);
        refresh(root).unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), c);
    }

    #[test]
    fn resolve_picks_unique_ongoing_or_reports() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(matches!(resolve(root, None), Err(SessionError::NoOpenSession)));
        fs::create_dir_all(root.join("2_Log/S")).unwrap();
        fs::create_dir_all(root.join("2_Log/G")).unwrap();
        fs::write(root.join("2_Log/S/S001_log.md"), format!("{}# S001: t\n", header::render(&h(Status::Ongoing, "", "")))).unwrap();
        assert_eq!(resolve(root, None).unwrap().id, "S001");
        fs::write(root.join("2_Log/G/G001_log.md"), format!("{}# G001: t\n", header::render(&h(Status::Ongoing, "", "")))).unwrap();
        assert!(matches!(resolve(root, None), Err(SessionError::MultipleOpen(v)) if v.len() == 2));
        assert_eq!(resolve(root, Some("G001")).unwrap().series, Series::G);
        assert!(matches!(resolve(root, Some("S009")), Err(SessionError::NotFound(_))));
    }
}
