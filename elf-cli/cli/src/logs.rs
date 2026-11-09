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

/// 이관 전(구 형식) 흔적: 루트 로그, `Archive/`, `Deprecated/`, TSV Registry. 없으면 빈 목록.
pub fn legacy_markers(root: &Path) -> Vec<String> {
    let mut v = Vec::new();
    let log = root.join("2_Log");
    if let Ok(rd) = fs::read_dir(&log) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.path().is_file() && name.ends_with("_log.md") {
                v.push(format!("2_Log/{name}"));
            }
        }
    }
    // Archive/, Deprecated/ = 로그(`*_log.md`)나 부분 폐기 파일(`*.partial.md`)이 있을 때만 — 이관 뒤 남은 다른 파일은 이전 형식이 아님
    for d in ["Archive", "Deprecated"] {
        let has_md = fs::read_dir(log.join(d)).is_ok_and(|rd| {
            rd.flatten().any(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                e.path().is_file() && (n.ends_with("_log.md") || n.ends_with(".partial.md"))
            })
        });
        if has_md {
            v.push(format!("2_Log/{d}/"));
        }
    }
    if log.join("Wiki/Session_Registry.tsv").is_file() {
        v.push("2_Log/Wiki/Session_Registry.tsv".into());
    }
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

/// 종료, 폐기 로그 파일 이름(쉼표 목록) — `Registry.md` 아래로 접힘.
pub fn nesting_value(root: &Path) -> String {
    let mut names = Vec::new();
    for s in Series::ALL {
        let (rows, _) = headers(root, s);
        for (l, h) in rows {
            if h.status != Status::Ongoing {
                names.push(format!("{}_log.md", l.id));
            }
        }
    }
    names.join(", ")
}

/// settings.json에 nesting 키만 병합. 바뀌었으면 true. JSON으로 읽을 수 없으면(주석 등) 손대지 않고 Err.
fn write_nesting(root: &Path, value: &str) -> Result<bool, String> {
    let p = root.join(".vscode/settings.json");
    // 접을 로그가 없고 설정 파일도 없으면 만들지 않음(새 프로젝트에 빈 설정 파일을 늘리지 않음)
    if value.is_empty() && !p.exists() {
        return Ok(false);
    }
    let mut v: serde_json::Value = match fs::read_to_string(&p) {
        Ok(t) if t.trim().is_empty() => serde_json::json!({}),
        Ok(t) => serde_json::from_str(&t).map_err(|e| {
            format!(".vscode/settings.json is not plain JSON ({e}) — explorer nesting not updated; set \"{NEST_PATTERNS}\" by hand or remove comments")
        })?,
        Err(_) => serde_json::json!({}),
    };
    let before = v.clone();
    let Some(obj) = v.as_object_mut() else {
        return Err(".vscode/settings.json is not a JSON object — explorer nesting not updated".into());
    };
    obj.insert(NEST_ENABLED.into(), serde_json::Value::Bool(true));
    let pats = obj.entry(NEST_PATTERNS).or_insert_with(|| serde_json::json!({}));
    let Some(pobj) = pats.as_object_mut() else {
        return Err(format!(".vscode/settings.json \"{NEST_PATTERNS}\" is not an object — explorer nesting not updated"));
    };
    pobj.insert("Registry.md".into(), serde_json::Value::String(value.to_string()));
    if v == before {
        return Ok(false);
    }
    fs::create_dir_all(root.join(".vscode")).map_err(|e| e.to_string())?;
    let mut out = serde_json::to_string_pretty(&v).expect("json");
    out.push('\n');
    fs::write(&p, out).map_err(|e| e.to_string())?;
    Ok(true)
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
    if let Err(w) = write_nesting(root, &nesting_value(root)) {
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
