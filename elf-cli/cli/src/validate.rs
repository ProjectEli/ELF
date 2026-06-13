//! `elf validate` — 세션/Registry/로그 정합 검사 (P012 t04). **읽기전용** — FS 무변경.
//!
//! Registry TSV 파서(session.rs)는 공용 — **파싱 불가는 escalation(exit 5)**,
//! 그 외 발견은 보고(`--check` 시 issue에 한해 exit 4). "검사 결과"와 "검사 불능"을 구분(P012 §0.1).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::session::{Escalation, RegistryRow, log_num, parse_registry, session_num};

#[derive(Debug, Default)]
pub struct ValidateReport {
    pub lines: Vec<String>,
    /// CI 게이트 대상 (정합 위반: 미등록 로그 / 유령 행 / 중복 번호 / 깨진 cross-ref)
    pub issues: usize,
    /// 비차단 경고 (번호 gap / 활성 세션 복수)
    pub warnings: usize,
}

impl ValidateReport {
    /// `--check`가 게이트하는 카운트 (warning 제외 — status와 동일 클래스)
    pub fn findings(&self) -> usize {
        self.issues
    }
    fn issue(&mut self, line: String) {
        self.issues += 1;
        self.lines.push(format!("issue: {line}"));
    }
    fn warn(&mut self, line: String) {
        self.warnings += 1;
        self.lines.push(format!("warn: {line}"));
    }
}

#[derive(Debug)]
pub enum ValidateError {
    /// Registry 파싱 불가 — 검사 진행 무의미, 상위 에이전트 위임 (exit 5)
    Escalation(Box<Escalation>),
    Io(io::Error),
}

impl From<io::Error> for ValidateError {
    fn from(e: io::Error) -> Self {
        ValidateError::Io(e)
    }
}

// ── 순수 검사 (FS 무부작용) ─────────────────────────────────────

/// 중복 번호(정렬·중복제거).
pub fn duplicate_nums(nums: &[u32]) -> Vec<u32> {
    let mut seen = BTreeSet::new();
    let mut dups = BTreeSet::new();
    for &n in nums {
        if !seen.insert(n) {
            dups.insert(n);
        }
    }
    dups.into_iter().collect()
}

/// 1..max 사이 누락 번호 (로그·Registry 합집합 기준).
pub fn numbering_gaps(known: &BTreeSet<u32>) -> Vec<u32> {
    let mut gaps = Vec::new();
    if let Some(&mx) = known.iter().max() {
        for n in 1..mx {
            if !known.contains(&n) {
                gaps.push(n);
            }
        }
    }
    gaps
}

/// 활성(Complete 아님) 세션 ID 목록 (Registry status 기준).
pub fn active_sessions(reg: &[RegistryRow]) -> Vec<String> {
    reg.iter()
        .filter(|r| !r.status.starts_with("Complete"))
        .map(|r| r.session.clone())
        .collect()
}

/// 마크다운 인라인 링크 `](target)`에서 **상대 .md 링크 대상** 추출(순수).
/// `#fragment` 제거, URL(`scheme://`·`mailto:`)·절대경로(`/`)·비 .md 제외, 제목(`"..."`)·`<...>` 처리.
pub fn extract_md_links(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = content.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b']' && b[i + 1] == b'(' {
            let rest = &content[i + 2..];
            if let Some(close) = rest.find(')') {
                i += 2 + close + 1;
                if let Some(t) = normalize_link_target(&rest[..close]) {
                    out.push(t);
                }
                continue;
            }
        }
        i += 1;
    }
    out
}

/// 링크 `()` 안 raw → 검사 대상 상대 .md 경로 (아니면 None).
fn normalize_link_target(raw: &str) -> Option<String> {
    let t = raw.trim();
    let t = if let Some(rest) = t.strip_prefix('<') {
        rest.split('>').next().unwrap_or(rest) // <...> : 공백 포함 그대로
    } else {
        t.split_whitespace().next().unwrap_or("") // 제목("...") 분리
    };
    let t = t.split('#').next().unwrap_or("").trim(); // fragment 제거
    if t.is_empty()
        || t.starts_with('/')
        || t.contains("://")
        || t.starts_with("mailto:")
        || !t.ends_with(".md")
    {
        return None;
    }
    Some(t.to_string())
}

// ── FS 실행기 ──────────────────────────────────────────────────

/// dir의 `S###_log.md` → (번호, 경로), 번호 오름차순.
fn scan_logs(dir: &Path) -> Vec<(u32, PathBuf)> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            if let Some(n) = e.file_name().to_str().and_then(log_num) {
                v.push((n, e.path()));
            }
        }
    }
    v.sort_by_key(|(n, _)| *n);
    v
}

/// `elf validate`: Registry↔로그 정합 · 번호 중복/gap · 활성 복수 · cross-ref 검사. 읽기전용.
pub fn run_validate(root: &Path) -> Result<ValidateReport, ValidateError> {
    let reg_text = fs::read_to_string(root.join(crate::session::REGISTRY_REL)).unwrap_or_default();
    let rows = parse_registry(&reg_text).map_err(ValidateError::Escalation)?;

    let live = scan_logs(&root.join("2_Log"));
    let archive = scan_logs(&root.join("2_Log/Archive"));

    let log_set: BTreeSet<u32> = live.iter().chain(archive.iter()).map(|(n, _)| *n).collect();
    let reg_set: BTreeSet<u32> = rows.iter().filter_map(|r| session_num(&r.session)).collect();

    let mut report = ValidateReport::default();

    // ① Registry ↔ 로그 정합
    for n in log_set.difference(&reg_set) {
        report.issue(format!("S{n:03} log exists but is not in the registry"));
    }
    for n in reg_set.difference(&log_set) {
        report.issue(format!(
            "S{n:03} is in the registry but has no log file (2_Log/ or Archive/)"
        ));
    }

    // ② 번호 중복 + gap
    let all_log_nums: Vec<u32> = live.iter().chain(archive.iter()).map(|(n, _)| *n).collect();
    for n in duplicate_nums(&all_log_nums) {
        report.issue(format!("duplicate log number S{n:03} across 2_Log/ and Archive/"));
    }
    let known: BTreeSet<u32> = log_set.union(&reg_set).copied().collect();
    for n in numbering_gaps(&known) {
        report.warn(format!("gap in session numbering: S{n:03} missing"));
    }

    // ④ 활성 세션 복수
    let active = active_sessions(&rows);
    if active.len() > 1 {
        report.warn(format!(
            "multiple active sessions ({}) — close all but the current one",
            active.join(", ")
        ));
    }

    // ③ cross-ref (상대 .md 링크 대상 부재)
    for (_, path) in live.iter().chain(archive.iter()) {
        let content = fs::read_to_string(path)?;
        let dir = path.parent().unwrap_or(root);
        let fname = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        for target in extract_md_links(&content) {
            if !dir.join(&target).exists() {
                report.issue(format!("{fname}: broken cross-ref → {target}"));
            }
        }
    }

    if report.lines.is_empty() {
        report
            .lines
            .push("ok: registry, logs, numbering, and cross-refs are consistent".into());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(session: &str, status: &str) -> RegistryRow {
        RegistryRow {
            line: 0,
            session: session.into(),
            date: "2026-01-01".into(),
            title: "t".into(),
            status: status.into(),
            key_finding: "-".into(),
            archive_path: "-".into(),
        }
    }

    #[test]
    fn duplicates_found_sorted() {
        assert_eq!(duplicate_nums(&[1, 2, 2, 3, 3, 3]), vec![2, 3]);
        assert!(duplicate_nums(&[1, 2, 3]).is_empty());
    }

    #[test]
    fn gaps_between_one_and_max() {
        let s: BTreeSet<u32> = [1u32, 4].into_iter().collect();
        assert_eq!(numbering_gaps(&s), vec![2, 3]);
        let consec: BTreeSet<u32> = [1u32, 2, 3].into_iter().collect();
        assert!(numbering_gaps(&consec).is_empty());
        assert!(numbering_gaps(&BTreeSet::new()).is_empty());
    }

    #[test]
    fn active_excludes_complete() {
        let rows = vec![
            row("S001", "★ 활성"),
            row("S002", "Complete"),
            row("S003", "Complete (archived)"),
            row("S004", "In Progress"),
        ];
        assert_eq!(active_sessions(&rows), vec!["S001", "S004"]);
    }

    #[test]
    fn links_extracted_and_filtered() {
        let md = "[a](rel/x.md) [b](../y.md#sec) [img](pic.png) \
                  [ext](https://e.com/z.md) [abs](/z.md) [t](title.md \"caption\") [ang](<ang.md>)";
        let links = extract_md_links(md);
        assert!(links.contains(&"rel/x.md".to_string()));
        assert!(links.contains(&"../y.md".to_string())); // fragment 제거
        assert!(links.contains(&"title.md".to_string())); // 제목 분리
        assert!(links.contains(&"ang.md".to_string())); // <...> 처리
        assert!(!links.iter().any(|l| l.ends_with("pic.png"))); // 비 .md 제외
        assert!(!links.iter().any(|l| l.contains("://"))); // URL 제외
        assert!(!links.contains(&"/z.md".to_string())); // 절대경로 제외
    }

    #[test]
    fn bracket_without_paren_is_not_a_link() {
        // 템플릿 `> **관련**: [관련 세션/문서, 예: S000, P001_xxx.md]` 형태(파렌 없음) → 추출 안 됨
        assert!(extract_md_links("> **관련**: [예: S000, P001_xxx.md]\\\n").is_empty());
    }
}
