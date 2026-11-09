//! `elf deprecate` — 폐기(S038, S039; 1단계 압축 보관 S044 t21~t23, P022 §6).
//!
//! 폐기 = 원문을 작업 트리의 문서에서 지우고 `.elf/deprecated/<ID>.md.xz`(xz, git 추적)에 보관, 원래 자리에는
//! 중립 흔적(표기 줄). 에이전트가 폐기한 내용을 읽지 않게 하는 것이 목적(읽는 순간 판단이 영향을 받음).
//! 단위: 문서 전체(세션 로그 = 헤더 status `deprecated`, 계획 문서 = 제목 줄과 표기 줄만), trial, 절, 표시 블록, 줄 범위.
//! `show <ID>`(풀어서 출력, 사용자 요청 때만), `--restore <ID>`(되돌리고 보관 파일 삭제), `list`, 인수 없음 = 목록.
//! 판단이 필요한 남는 참조는 고치지 않고 `review:` 줄로 출력. 표기 줄 ↔ 보관 파일 짝은 `elf check`가 검사.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::header::{self, Status};
use crate::logs;
use crate::validate;

pub const PLAN_BASE: &str = "1_Concept/12_Planning";
pub const STORE_DIR: &str = ".elf/deprecated";
const MARK_BEGIN: &str = "<!-- deprecate:begin -->";
const MARK_END: &str = "<!-- deprecate:end -->";
/// 흔적 표기 줄: `> **Deprecated**: ID · 날짜 · 단위 · 사유: … · 대체: …`
pub const MARKER_PREFIX: &str = "> **Deprecated**: ";
/// 보관 파일이 없는 흔적(이관 전 purge) — 표기 줄 끝에 붙음, 짝 검사에서 제외
pub const PURGED_TAG: &str = " · 복원 불가";

#[derive(Debug)]
pub enum DeprecateError {
    /// 거부 — 파일 무변경 (exit 3)
    Refuse(String),
    /// 대상, ID 없음 (exit 1)
    NotFound(String),
    Io(io::Error),
}

impl From<io::Error> for DeprecateError {
    fn from(e: io::Error) -> Self {
        DeprecateError::Io(e)
    }
}

impl std::fmt::Display for DeprecateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeprecateError::Refuse(s) => write!(f, "refuse: {s}"),
            DeprecateError::NotFound(s) => write!(f, "not found: {s}"),
            DeprecateError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

type Res<T> = Result<T, DeprecateError>;

fn refuse<T>(msg: impl Into<String>) -> Res<T> {
    Err(DeprecateError::Refuse(msg.into()))
}

#[derive(Debug, Default)]
pub struct Report {
    /// 출력 줄(접두 `[elf] `는 호출자가 붙임)
    pub lines: Vec<String>,
    /// 파일 변경 여부(dry-run이면 false)
    pub changed: bool,
}

#[derive(Debug, Default, Clone)]
pub struct DeprecateOptions {
    /// `S###`, `G###`, `P###`, 프로젝트 기준 경로
    pub target: String,
    pub trial: Option<String>,
    pub section: Option<String>,
    pub marked: bool,
    /// 1-based inclusive
    pub lines: Option<(usize, usize)>,
    pub expect: Option<String>,
    pub replaced_by: Option<String>,
    pub reason: Option<String>,
    pub dry_run: bool,
    /// YYYY-MM-DD (주입형)
    pub date: String,
}

// ── 압축 보관 ─────────────────────────────────────────────────────

pub fn store_rel(id: &str) -> String {
    format!("{STORE_DIR}/{id}.md.xz")
}

fn compress(text: &str) -> io::Result<Vec<u8>> {
    use io::Write;
    let mut w = lzma_rust2::XzWriter::new(Vec::new(), lzma_rust2::XzOptions::with_preset(6))?;
    w.write_all(text.as_bytes())?;
    w.finish()
}

pub(crate) fn decompress(bytes: &[u8]) -> io::Result<String> {
    use io::Read;
    let mut out = Vec::new();
    lzma_rust2::XzReader::new(bytes, false).read_to_end(&mut out).map_err(|e| io::Error::other(format!("xz: {e}")))?;
    String::from_utf8(out).map_err(|e| io::Error::other(e.to_string()))
}

/// 보관 파일 → (메타 키와 값, 원문).
fn read_store(root: &Path, id: &str) -> Res<(Vec<(String, String)>, String)> {
    let p = root.join(store_rel(id));
    let mut bytes = Vec::new();
    fs::File::open(&p).map_err(|_| DeprecateError::NotFound(format!("{id} (no {})", store_rel(id))))?.read_to_end(&mut bytes)?;
    let text = decompress(&bytes)?;
    // 메타 블록 다음의 원문은 앞뒤 공백까지 그대로 — split_block을 쓰지 않고 직접 자름
    let rest = text.strip_prefix("---\n").ok_or_else(|| DeprecateError::Refuse(format!("{} has no metadata block", store_rel(id))))?;
    let end = rest.find("\n---\n").ok_or_else(|| DeprecateError::Refuse(format!("{} metadata block not closed", store_rel(id))))?;
    let (meta, body) = (&rest[..end], &rest[end + 5..]);
    let mut kv = Vec::new();
    for l in meta.lines() {
        if let Some((k, v)) = l.split_once(':') {
            let v = v.trim();
            let v = if v.starts_with('\'') { header::unquote(v).unwrap_or_else(|_| v.to_string()) } else { v.to_string() };
            kv.push((k.trim().to_string(), v));
        }
    }
    Ok((kv, body.to_string()))
}

pub(crate) fn compress_text(text: &str) -> io::Result<Vec<u8>> {
    compress(text)
}

pub(crate) fn store_text(meta: &[(&str, String)], body: &str) -> String {
    let mut s = String::from("---\n");
    for (k, v) in meta {
        s.push_str(&format!("{k}: {}\n", header::quote(v)));
    }
    s.push_str("---\n");
    s.push_str(body);
    s
}

fn get<'a>(kv: &'a [(String, String)], k: &str) -> Option<&'a str> {
    kv.iter().find(|(x, _)| x == k).map(|(_, v)| v.as_str())
}

// ── 문서 식별 ─────────────────────────────────────────────────────

fn dirname(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(d, _)| d)
}

fn basename(rel: &str) -> &str {
    rel.rsplit_once('/').map_or(rel, |(_, b)| b)
}

fn is_plan_id(s: &str) -> bool {
    s.len() >= 2 && s.starts_with('P') && s[1..].bytes().all(|b| b.is_ascii_digit())
}

/// 문서 ID: 세션 로그 = `S012`/`G004`, 계획 문서 = 파일명 앞의 `P###`, 그 밖 = 파일명(확장자 제외).
pub fn doc_id(rel: &str) -> String {
    let b = basename(rel);
    if let Some((s, n)) = logs::log_file_id(b) {
        return logs::id_of(s, n);
    }
    let stem = b.strip_suffix(".md").unwrap_or(b);
    let head = stem.split('_').next().unwrap_or(stem);
    if is_plan_id(head) { head.to_string() } else { stem.to_string() }
}

fn is_session_log(rel: &str) -> bool {
    logs::log_file_id(basename(rel)).is_some() && (dirname(rel) == "2_Log/S" || dirname(rel) == "2_Log/G")
}

/// `S###`, `G###`, `P###`, 경로 → 프로젝트 기준 경로.
fn resolve_doc(root: &Path, target: &str) -> Res<String> {
    if let Some((s, n)) = logs::parse_id(target) {
        let rel = logs::log_rel(s, n);
        return if root.join(&rel).is_file() { Ok(rel) } else { Err(DeprecateError::NotFound(format!("session {target}"))) };
    }
    if is_plan_id(target) {
        for d in [PLAN_BASE.to_string(), format!("{PLAN_BASE}/Archive")] {
            let mut hits: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(root.join(&d)) {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if name.starts_with(&format!("{target}_")) && name.ends_with(".md") {
                        hits.push(name);
                    }
                }
            }
            hits.sort();
            if hits.len() == 1 {
                return Ok(format!("{d}/{}", hits[0]));
            }
            if hits.len() > 1 {
                return refuse(format!("ambiguous planning doc {target}: {}", hits.join(", ")));
            }
        }
        return Err(DeprecateError::NotFound(format!("planning doc {target}")));
    }
    let rel = target.replace('\\', "/");
    let rel = rel.trim_start_matches("./").trim_end_matches('/').to_string();
    if root.join(&rel).is_file() { Ok(rel) } else { Err(DeprecateError::NotFound(target.to_string())) }
}

fn check_scope(rel: &str) -> Res<()> {
    let in_plan = rel.starts_with(&format!("{PLAN_BASE}/")) && !rel[PLAN_BASE.len() + 1..].contains("Wiki/");
    if !is_session_log(rel) && !in_plan {
        return refuse(format!(
            "out of scope: {rel} — elf deprecate handles session logs (2_Log/S/, 2_Log/G/) and planning docs (1_Concept/12_Planning/)"
        ));
    }
    Ok(())
}

// ── 줄 범위 탐지 (코드 펜스 안의 줄은 헤딩, 표시로 보지 않음) ─────────────

fn fence_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(lines.len());
    let mut fence = false;
    for l in lines {
        if l.trim_end_matches('\r').starts_with("```") {
            mask.push(true);
            fence = !fence;
            continue;
        }
        mask.push(fence);
    }
    mask
}

fn clean(line: &str) -> &str {
    line.trim_end_matches('\r')
}

/// `## tNN:` 헤딩이면 trial id.
fn trial_heading_id(line: &str) -> Option<String> {
    let rest = clean(line).strip_prefix("## t")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || !rest[digits.len()..].starts_with(':') {
        return None;
    }
    Some(format!("t{digits}"))
}

/// trial 범위 — (헤딩 줄, 끝[배타]). 끝 빈 줄과 구분선(`---`) 제외.
fn trial_bounds(lines: &[&str], tid: &str) -> Res<(usize, usize)> {
    let mask = fence_mask(lines);
    let h = lines
        .iter()
        .enumerate()
        .position(|(i, l)| !mask[i] && trial_heading_id(l).as_deref() == Some(tid))
        .ok_or_else(|| DeprecateError::NotFound(format!("trial {tid}")))?;
    let mut e = (h + 1..lines.len()).find(|&i| !mask[i] && clean(lines[i]).starts_with("## ")).unwrap_or(lines.len());
    loop {
        // 끝의 빈 줄과 trial 구분선(`---`)은 다음 trial 몫으로 남김
        let t = if e > h + 1 { clean(lines[e - 1]).trim() } else { "x" };
        if t.is_empty() || t == "---" {
            e -= 1;
        } else {
            break;
        }
    }
    if e <= h + 1 {
        return refuse(format!("{tid} has no body to deprecate"));
    }
    Ok((h, e))
}

/// 절 범위 — (절 헤딩 줄, 끝[배타]). 끝 빈 줄 제외.
fn section_bounds(lines: &[&str], tid: &str, key: &str) -> Res<(usize, usize)> {
    let (h, e) = trial_bounds(lines, tid)?;
    let mask = fence_mask(lines);
    let a = (h + 1..e)
        .find(|&i| !mask[i] && validate::section_key(lines[i]) == Some(key))
        .ok_or_else(|| DeprecateError::NotFound(format!("section {tid} {key}")))?;
    let mut b = (a + 1..e).find(|&i| !mask[i] && clean(lines[i]).starts_with("### ")).unwrap_or(e);
    while b > a + 1 && clean(lines[b - 1]).trim().is_empty() {
        b -= 1;
    }
    Ok((a, b))
}

fn enclosing_trial(lines: &[&str], idx: usize) -> Option<String> {
    let mask = fence_mask(lines);
    for i in (0..=idx.min(lines.len().saturating_sub(1))).rev() {
        if mask[i] {
            continue;
        }
        if let Some(tid) = trial_heading_id(lines[i]) {
            return Some(tid);
        }
        let l = clean(lines[i]);
        if l.starts_with("## ") || l.starts_with("# ") {
            return None;
        }
    }
    None
}

fn enclosing_section(lines: &[&str], idx: usize) -> Option<String> {
    let mask = fence_mask(lines);
    for i in (0..=idx.min(lines.len().saturating_sub(1))).rev() {
        if mask[i] {
            continue;
        }
        let l = clean(lines[i]);
        if l.starts_with("### ") {
            return validate::section_key(l).map(str::to_string);
        }
        if l.starts_with("## ") || l.starts_with("# ") {
            return None;
        }
    }
    None
}

fn block_unit(lines: &[&str], a: usize) -> String {
    match (enclosing_trial(lines, a), enclosing_section(lines, a)) {
        (Some(t), Some(s)) => format!("{t} {s} part"),
        (Some(t), None) => format!("{t} part"),
        _ => "part".into(),
    }
}

fn guard_block(lines: &[&str], a: usize, b: usize) -> Res<()> {
    let mask = fence_mask(lines);
    for i in a..b {
        if !mask[i] && trial_heading_id(lines[i]).is_some() {
            return refuse(format!("block contains a trial heading ({}) — use --trial", clean(lines[i])));
        }
        if !mask[i] && clean(lines[i]).starts_with("# ") {
            return refuse("block contains the document title line");
        }
        if !mask[i] && clean(lines[i]).starts_with(MARKER_PREFIX) {
            return refuse("block contains a deprecation marker line — restore that block first");
        }
    }
    Ok(())
}

/// 표시 블록(`<!-- deprecate:begin -->` … `<!-- deprecate:end -->`) 첫 쌍 → 범위(표시 줄 포함).
fn first_marked_block(lines: &[&str]) -> Res<Option<(usize, usize)>> {
    let mask = fence_mask(lines);
    let Some(a) = (0..lines.len()).find(|&i| !mask[i] && clean(lines[i]).trim() == MARK_BEGIN) else {
        return Ok(None);
    };
    for i in a + 1..lines.len() {
        if mask[i] {
            continue;
        }
        let l = clean(lines[i]).trim();
        if l == MARK_BEGIN {
            return refuse("nested <!-- deprecate:begin --> markers");
        }
        if l == MARK_END {
            return Ok(Some((a, i + 1)));
        }
    }
    refuse("<!-- deprecate:begin --> without <!-- deprecate:end -->")
}

fn strip_marks(text: &str) -> String {
    text.split('\n')
        .filter(|l| {
            let s = clean(l).trim();
            s != MARK_BEGIN && s != MARK_END
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 조작 전후 trial 구조 검사 — 조작 후에만 나타나는 항목(거부 근거).
fn new_structure_findings(before: &str, after: &str) -> Vec<String> {
    let mut old = validate::trial_structure_findings(before);
    let mut out = Vec::new();
    for f in validate::trial_structure_findings(after) {
        if let Some(pos) = old.iter().position(|o| *o == f) {
            old.remove(pos);
        } else {
            out.push(f);
        }
    }
    out
}

// ── ID, 표기 줄 ───────────────────────────────────────────────────

/// `<문서 ID>-D<NN>` — 문서의 표기 줄과 보관 파일에 쓰인 최대 번호 + 1(재사용 없음).
fn next_id(root: &Path, doc: &str, text: &str) -> String {
    let prefix = format!("{doc}-D");
    let mut max = 0u32;
    let mut scan = |s: &str| {
        for (idx, _) in s.match_indices(&prefix) {
            let digits: String = s[idx + prefix.len()..].chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                max = max.max(n);
            }
        }
    };
    scan(text);
    if let Ok(rd) = fs::read_dir(root.join(STORE_DIR)) {
        for e in rd.flatten() {
            scan(&e.file_name().to_string_lossy());
        }
    }
    format!("{doc}-D{:02}", max + 1)
}

fn is_block_id(s: &str) -> bool {
    s.rsplit_once("-D").is_some_and(|(doc, n)| !doc.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn marker(id: &str, date: &str, unit: &str, reason: Option<&str>, replaced_by: Option<&str>) -> String {
    let mut m = format!("{MARKER_PREFIX}{id} · {date} · {unit}");
    if let Some(r) = reason.filter(|r| !r.trim().is_empty()) {
        m.push_str(&format!(" · 사유: {}", header::one_line(r)));
    }
    if let Some(r) = replaced_by.filter(|r| !r.trim().is_empty()) {
        m.push_str(&format!(" · 대체: {}", header::one_line(r)));
    }
    m
}

/// 표기 줄 내용(이관도 같은 형식을 씀).
pub(crate) fn marker_line(id: &str, date: &str, unit: &str, reason: Option<&str>, replaced_by: Option<&str>) -> String {
    marker(id, date, unit, reason, replaced_by)
}

/// 표기 줄 → ID.
pub fn marker_id(line: &str) -> Option<String> {
    let rest = clean(line).strip_prefix(MARKER_PREFIX)?;
    let id = rest.split(" · ").next()?.trim();
    is_block_id(id).then(|| id.to_string())
}

fn first_title(text: &str) -> Option<&str> {
    let body = header::parse(text).map(|(_, at)| &text[at..]).unwrap_or(text);
    body.lines().find(|l| l.starts_with("# "))
}

// ── review: 판단 필요 항목 수집(수정하지 않음) ────────────────────────

fn walk_docs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            walk_docs(&p, out);
        } else if name.ends_with(".md") {
            out.push(p);
        }
    }
}

/// `doc`(+ 선택적 `tid`)에 대한 언급이 줄에 있는지 — `S012 t03`, `S012, t03`, `S012_t03` 등.
fn mentions(line: &str, doc: &str, tid: Option<&str>) -> bool {
    for (idx, _) in line.match_indices(doc) {
        let before_ok = idx == 0 || !line[..idx].chars().next_back().is_some_and(|c| c.is_alphanumeric());
        let after = &line[idx + doc.len()..];
        let after_ok = !after.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '-');
        if !(before_ok && after_ok) {
            continue;
        }
        match tid {
            None => return true,
            Some(t) => {
                let rest = after.trim_start_matches([' ', ',', '·', '_', '-', '/']);
                if rest.starts_with(t) && !rest[t.len()..].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
    }
    false
}

fn review_items(root: &Path, rel: &str, after_text: &str, doc: &str, tid: Option<&str>, stored: &str) -> Vec<String> {
    let mut items = Vec::new();
    if let Ok((h, _)) = header::parse(after_text) {
        let word = tid.unwrap_or(doc);
        for (k, v) in [("handoff", &h.handoff), ("conclusion", &h.conclusion)] {
            if v.split(|c: char| !c.is_alphanumeric()).any(|w| w == word) {
                items.push(format!("review: {k}: {rel} header mentions {word}"));
            }
        }
    }
    if let Some(t) = tid {
        for (i, l) in after_text.lines().enumerate() {
            let l = clean(l);
            if l.starts_with(MARKER_PREFIX) || l.starts_with("handoff:") || l.starts_with("conclusion:") {
                continue;
            }
            if trial_heading_id(l).is_none() && l.split(|c: char| !c.is_alphanumeric()).any(|w| w == t) {
                items.push(format!("review: ref: {rel}:{} mentions {t}", i + 1));
            }
        }
    }
    let mut docs = Vec::new();
    walk_docs(root, &mut docs);
    for p in docs {
        let r = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        if r == rel || r.ends_with("/Registry.md") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else { continue };
        for (i, l) in text.lines().enumerate() {
            if mentions(clean(l), doc, tid) {
                let what = tid.map_or(doc.to_string(), |t| format!("{doc} {t}"));
                items.push(format!("review: ref: {r}:{} refers to {what}", i + 1));
            }
        }
    }
    for t in validate::extract_image_targets(stored) {
        items.push(format!("review: figure: {t} is embedded in the deprecated content (file left in place)"));
    }
    for l in stored.lines() {
        let l = clean(l);
        if !l.starts_with('|') {
            continue;
        }
        let mut rest = l;
        while let Some(s) = rest.find('`') {
            let after = &rest[s + 1..];
            let Some(e) = after.find('`') else { break };
            let code = &after[..e];
            if code.contains('/') && !code.contains(' ') {
                items.push(format!("review: file: {code} is listed in the deprecated content (file left in place)"));
            }
            rest = &after[e + 1..];
        }
    }
    items
}

// ── 실행기: 폐기 ──────────────────────────────────────────────────

pub fn run_deprecate(root: &Path, o: &DeprecateOptions) -> Res<Report> {
    if let Err(crate::session::SessionError::Legacy(m)) = crate::session::refuse_legacy(root) {
        return refuse(format!("this project still has the earlier log layout ({}) — run `elf update` to migrate first", m.join(", ")));
    }
    let rel = resolve_doc(root, &o.target)?;
    check_scope(&rel)?;
    let units = usize::from(o.trial.is_some()) + usize::from(o.marked) + usize::from(o.lines.is_some());
    if units > 1 {
        return refuse("choose one unit: --trial [--section] | --marked | --lines");
    }
    if o.section.is_some() && o.trial.is_none() {
        return refuse("--section requires --trial");
    }
    if o.lines.is_some() && o.expect.is_none() {
        return refuse("--lines requires --expect \"<first line>\"");
    }
    if o.trial.is_some() && !is_session_log(&rel) {
        return refuse("--trial/--section apply to session logs only (planning docs: --marked or --lines)");
    }
    let text = fs::read_to_string(root.join(&rel))?;
    if units == 0 { deprecate_whole(root, &rel, &text, o) } else { deprecate_block(root, &rel, &text, o) }
}

fn deprecate_whole(root: &Path, rel: &str, text: &str, o: &DeprecateOptions) -> Res<Report> {
    let doc = doc_id(rel);
    let id = next_id(root, &doc, text);
    let unit_label = if is_session_log(rel) { "세션 전체" } else { "문서 전체" };
    let mark = marker(&id, &o.date, unit_label, o.reason.as_deref(), o.replaced_by.as_deref());
    let mut meta: Vec<(&str, String)> = vec![
        ("id", id.clone()),
        ("source", rel.to_string()),
        ("unit", "whole".into()),
        ("date", o.date.clone()),
        ("reason", o.reason.clone().unwrap_or_default()),
        ("replaced_by", o.replaced_by.clone().unwrap_or_default()),
    ];
    let (new_text, stored) = if is_session_log(rel) {
        let (mut h, at) = header::parse(text).map_err(|e| DeprecateError::Refuse(format!("{rel}: {e}")))?;
        if h.status == Status::Deprecated {
            return refuse(format!("{rel} is already deprecated"));
        }
        meta.push(("status", h.status.as_str().to_string()));
        meta.push(("handoff", h.handoff.clone()));
        meta.push(("conclusion", h.conclusion.clone()));
        meta.push(("user_note", h.user_note.clone()));
        let body = &text[at..];
        let title = body.lines().find(|l| l.starts_with("# ")).unwrap_or("").to_string();
        h.status = Status::Deprecated;
        let mut ho = format!("폐기 {id}");
        if let Some(r) = o.reason.as_deref().filter(|r| !r.trim().is_empty()) {
            ho.push_str(&format!(": {}", header::one_line(r)));
        }
        if let Some(r) = o.replaced_by.as_deref().filter(|r| !r.trim().is_empty()) {
            ho.push_str(&format!(", 대체 = {}", header::one_line(r)));
        }
        h.handoff = ho;
        h.conclusion.clear();
        h.user_note.clear();
        h.modified = o.date.clone();
        (format!("{}{title}\n\n{mark}\n", header::render(&h)), body.to_string())
    } else {
        if text.lines().next().is_some_and(|l| l.starts_with(MARKER_PREFIX))
            || text.lines().nth(2).is_some_and(|l| clean(l).starts_with(MARKER_PREFIX) && text.lines().count() <= 4)
        {
            return refuse(format!("{rel} is already deprecated"));
        }
        let title = first_title(text).unwrap_or("").to_string();
        (format!("{title}\n\n{mark}\n"), text.to_string())
    };
    let mut rep = Report::default();
    rep.lines.push(format!("deprecated {id} (whole document) → {}", store_rel(&id)));
    if !o.dry_run {
        write_store(root, &id, &store_text(&meta, &stored))?;
        fs::write(root.join(rel), &new_text)?;
        rep.changed = true;
        rep.lines.extend(logs::refresh(root)?.into_iter().map(|w| format!("warn: {w}")));
    }
    push_review(&mut rep, review_items(root, rel, &new_text, &doc, None, &stored));
    Ok(rep)
}

fn write_store(root: &Path, id: &str, text: &str) -> Res<()> {
    let p = root.join(store_rel(id));
    if p.exists() {
        return refuse(format!("{} already exists", store_rel(id)));
    }
    fs::create_dir_all(root.join(STORE_DIR))?;
    fs::write(&p, compress(text)?)?;
    Ok(())
}

fn push_review(rep: &mut Report, items: Vec<String>) {
    let mut seen = BTreeSet::new();
    let items: Vec<String> = items.into_iter().filter(|i| seen.insert(i.clone())).collect();
    let n = items.len();
    rep.lines.extend(items);
    if n > 0 {
        rep.lines.push(format!("review: {n} item(s) — not modified by elf; resolve each with the user or confirm no change is needed"));
    }
}

fn deprecate_block(root: &Path, rel: &str, text: &str, o: &DeprecateOptions) -> Res<Report> {
    if is_session_log(rel)
        && let Ok((h, _)) = header::parse(text)
        && h.status == Status::Deprecated
    {
        return refuse(format!("{rel} is already deprecated as a whole"));
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let doc = doc_id(rel);
    let id = next_id(root, &doc, text);
    // (교체 범위 a..b, 저장 원문, 단위, trial id, 남길 줄들)
    let (a, b, stored, unit, tid, keep): (usize, usize, String, String, Option<String>, Vec<String>) = if let Some(t) = &o.trial {
        if let Some(key) = &o.section {
            let (a, b) = section_bounds(&lines, t, key)?;
            (a, b, lines[a..b].join("\n"), format!("{t} {key}"), Some(t.clone()), Vec::new())
        } else {
            let (h, e) = trial_bounds(&lines, t)?;
            let num = t.trim_start_matches('t');
            (h, e, lines[h..e].join("\n"), t.clone(), Some(t.clone()), vec![format!("## t{num}: (폐기 {id})")])
        }
    } else if o.marked {
        let Some((a, b)) = first_marked_block(&lines)? else {
            return refuse(format!("no {MARK_BEGIN} … {MARK_END} block in {rel}"));
        };
        guard_block(&lines, a + 1, b - 1)?;
        if a + 1 >= b - 1 {
            return refuse("empty marked block");
        }
        (a, b, lines[a + 1..b - 1].join("\n"), block_unit(&lines, a), enclosing_trial(&lines, a), Vec::new())
    } else {
        let (a1, b1) = o.lines.unwrap();
        if a1 == 0 || b1 < a1 || b1 > lines.len() {
            return refuse(format!("--lines {a1}-{b1} out of range (1..{})", lines.len()));
        }
        let expect = o.expect.as_deref().unwrap_or("");
        if clean(lines[a1 - 1]).trim() != expect.trim() {
            return refuse(format!("--expect mismatch at line {a1}: found \"{}\"", clean(lines[a1 - 1])));
        }
        guard_block(&lines, a1 - 1, b1)?;
        (a1 - 1, b1, lines[a1 - 1..b1].join("\n"), block_unit(&lines, a1 - 1), enclosing_trial(&lines, a1 - 1), Vec::new())
    };
    let mark = marker(&id, &o.date, &unit, o.reason.as_deref(), o.replaced_by.as_deref());
    let mut out: Vec<String> = lines[..a].iter().map(|s| s.to_string()).collect();
    out.extend(keep);
    out.push(mark);
    out.extend(lines[b..].iter().map(|s| s.to_string()));
    let mut new_text = out.join("\n");
    if is_session_log(rel) {
        let found = new_structure_findings(&strip_marks(text), &new_text);
        if !found.is_empty() {
            return refuse(format!(
                "the change would introduce structure findings in {rel} — choose another unit (e.g. the whole trial): {}",
                found.join(" / ")
            ));
        }
        if let Ok((mut h, _)) = header::parse(&new_text) {
            h.modified = o.date.clone();
            new_text = header::replace(&new_text, &h).unwrap_or(new_text);
        }
    }
    let meta: Vec<(&str, String)> = vec![
        ("id", id.clone()),
        ("source", rel.to_string()),
        ("unit", unit.clone()),
        ("date", o.date.clone()),
        ("reason", o.reason.clone().unwrap_or_default()),
        ("replaced_by", o.replaced_by.clone().unwrap_or_default()),
    ];
    let mut rep = Report::default();
    rep.lines.push(format!("deprecated {id} ({unit}) → {}", store_rel(&id)));
    if !o.dry_run {
        write_store(root, &id, &store_text(&meta, &stored))?;
        fs::write(root.join(rel), &new_text)?;
        rep.changed = true;
        rep.lines.extend(logs::refresh(root)?.into_iter().map(|w| format!("warn: {w}")));
    }
    push_review(&mut rep, review_items(root, rel, &new_text, &doc, tid.as_deref(), &stored));
    Ok(rep)
}

// ── 실행기: show, 복원, 목록 ───────────────────────────────────────

/// `elf deprecate show <ID>` — 보관 원문을 풀어 출력(사용자 요청 때만, LogConvention §6).
pub fn run_show(root: &Path, id: &str) -> Res<Report> {
    let (kv, body) = read_store(root, id)?;
    let mut rep = Report::default();
    rep.lines.push(format!(
        "{id}: source {} · unit {} · date {}{}{}",
        get(&kv, "source").unwrap_or("?"),
        get(&kv, "unit").unwrap_or("?"),
        get(&kv, "date").unwrap_or("?"),
        get(&kv, "reason").filter(|r| !r.is_empty()).map(|r| format!(" · 사유: {r}")).unwrap_or_default(),
        get(&kv, "replaced_by").filter(|r| !r.is_empty()).map(|r| format!(" · 대체: {r}")).unwrap_or_default(),
    ));
    rep.lines.push("--- deprecated content (not a basis for current decisions) ---".into());
    rep.lines.extend(body.lines().map(str::to_string));
    Ok(rep)
}

pub fn run_restore(root: &Path, id: &str) -> Res<Report> {
    if !is_block_id(id) {
        return Err(DeprecateError::NotFound(format!("{id} is not a deprecation id (e.g. S012-D01)")));
    }
    let (kv, body) = read_store(root, id)?;
    let rel = get(&kv, "source").ok_or_else(|| DeprecateError::Refuse(format!("{} has no source", store_rel(id))))?.to_string();
    let path = root.join(&rel);
    let text = fs::read_to_string(&path).map_err(|_| DeprecateError::NotFound(format!("{rel} (source of {id})")))?;
    let lines: Vec<&str> = text.split('\n').collect();
    let mi = lines
        .iter()
        .position(|l| marker_id(l).as_deref() == Some(id))
        .ok_or_else(|| DeprecateError::Refuse(format!("{rel} has no marker line for {id} — cannot place it back")))?;
    let restored = if get(&kv, "unit") == Some("whole") {
        if is_session_log(&rel) {
            let (mut h, _) = header::parse(&text).map_err(|e| DeprecateError::Refuse(format!("{rel}: {e}")))?;
            h.status = get(&kv, "status").and_then(Status::parse).unwrap_or(Status::Ongoing);
            h.handoff = get(&kv, "handoff").unwrap_or("").to_string();
            h.conclusion = get(&kv, "conclusion").unwrap_or("").to_string();
            h.user_note = get(&kv, "user_note").unwrap_or("").to_string();
            format!("{}{body}", header::render(&h))
        } else {
            body
        }
    } else {
        // trial 단위는 흔적 제목 줄(`## tNN: (폐기 ID)`)도 함께 바꿈
        let start = if mi > 0 && clean(lines[mi - 1]).ends_with(&format!("(폐기 {id})")) { mi - 1 } else { mi };
        let mut out: Vec<String> = lines[..start].iter().map(|s| s.to_string()).collect();
        out.push(body);
        out.extend(lines[mi + 1..].iter().map(|s| s.to_string()));
        out.join("\n")
    };
    fs::write(&path, restored)?;
    fs::remove_file(root.join(store_rel(id)))?;
    let mut rep = Report { changed: true, ..Default::default() };
    rep.lines.push(format!("restored {id} → {rel} ({} removed)", store_rel(id)));
    rep.lines.extend(logs::refresh(root)?.into_iter().map(|w| format!("warn: {w}")));
    let dir = Path::new(&rel).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let now = fs::read_to_string(&path).unwrap_or_default();
    for t in validate::extract_md_links(&now) {
        if !root.join(&dir).join(&t).exists() {
            rep.lines.push(format!("warn: {rel}: restored content links to a missing file → {t}"));
        }
    }
    Ok(rep)
}

/// 문서들(세션 로그, 계획 문서)의 표기 줄 → (ID, 문서 경로).
fn markers_in_docs(root: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> = logs::scan_all(root).into_iter().map(|l| l.path).collect();
    walk_docs(&root.join(PLAN_BASE), &mut files);
    for p in files {
        let Ok(text) = fs::read_to_string(&p) else { continue };
        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        for l in text.lines() {
            if let Some(id) = marker_id(l)
                && !l.contains(PURGED_TAG)
            {
                out.push((id, rel.clone()));
            }
        }
    }
    out
}

fn store_ids(root: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(root.join(STORE_DIR))
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".md.xz").map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// `elf check` 짝 검사: ('i' issue | 'w' warn | 'n' info, 메시지).
pub fn pairing_findings(root: &Path) -> Vec<(char, String)> {
    let markers = markers_in_docs(root);
    let stores: BTreeSet<String> = store_ids(root).into_iter().collect();
    let marked: BTreeSet<String> = markers.iter().map(|(i, _)| i.clone()).collect();
    let mut out = Vec::new();
    for (id, rel) in &markers {
        if !stores.contains(id) {
            out.push(('i', format!("{rel}: marker {id} has no store file {} (LogConvention §6)", store_rel(id))));
        }
    }
    for id in stores.difference(&marked) {
        out.push(('w', format!("{} has no marker line in any document (LogConvention §6)", store_rel(id))));
    }
    if !stores.is_empty()
        && let Ok(o) = std::process::Command::new("git").args(["ls-files", STORE_DIR]).current_dir(root).output()
        && o.status.success()
    {
        let tracked = String::from_utf8_lossy(&o.stdout);
        for id in &stores {
            if !tracked.lines().any(|l| l.trim() == store_rel(id)) {
                out.push(('n', format!("{} is not tracked by git — commit it together with the log change", store_rel(id))));
            }
        }
    }
    out
}

/// 목록 — 보관 파일마다 1줄, 짝 경고.
pub fn run_list(root: &Path) -> Res<Report> {
    let mut rep = Report::default();
    let ids = store_ids(root);
    for id in &ids {
        match read_store(root, id) {
            Ok((kv, _)) => rep.lines.push(format!(
                "{id} · {} · {} · {}{}{}",
                get(&kv, "date").unwrap_or("?"),
                get(&kv, "unit").unwrap_or("?"),
                get(&kv, "source").unwrap_or("?"),
                get(&kv, "reason").filter(|r| !r.is_empty()).map(|r| format!(" · 사유: {r}")).unwrap_or_default(),
                get(&kv, "replaced_by").filter(|r| !r.is_empty()).map(|r| format!(" · 대체: {r}")).unwrap_or_default(),
            )),
            Err(e) => rep.lines.push(format!("warn: {id}: {e}")),
        }
    }
    for (kind, msg) in pairing_findings(root) {
        if kind != 'n' {
            rep.lines.push(format!("warn: {msg}"));
        }
    }
    rep.lines.push(format!("{} deprecated item(s)", ids.len()));
    Ok(rep)
}

// ── experimental 게이트 (.elf/config.json `experimental` 배열) ────────────

/// `.elf/config.json`의 `experimental` 배열(기능 이름 목록). 부재, 형식 불일치 = 빈 목록.
pub fn experimental_features(root: &Path) -> Vec<String> {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("experimental").and_then(|a| a.as_array()).cloned())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionNewOptions, run_session_new};

    fn project() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("2_Log")).unwrap();
        run_session_new(tmp.path(), &SessionNewOptions { title: "t".into(), date: "2026-11-09".into(), goal: "g".into(), gov: false }).unwrap();
        let p = tmp.path().join("2_Log/S/S001_log.md");
        let mut text = fs::read_to_string(&p).unwrap();
        text.push_str("\n## t01: a\n\n### 목표 (Goal)\n- x\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n- 비밀 해석\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n- f\n\n---\n\n## t02: b\n\n### 목표 (Goal)\n- y\n");
        fs::write(&p, text).unwrap();
        tmp
    }

    fn opts(target: &str) -> DeprecateOptions {
        DeprecateOptions { target: target.into(), date: "2026-11-10".into(), reason: Some("조건 오류".into()), ..Default::default() }
    }

    #[test]
    fn xz_roundtrip() {
        let t = "가나다 abc\n줄2\n";
        assert_eq!(decompress(&compress(t).unwrap()).unwrap(), t);
    }

    #[test]
    fn trial_deprecate_hides_text_and_restores() {
        let tmp = project();
        let root = tmp.path();
        let before = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
        let mut o = opts("S001");
        o.trial = Some("t01".into());
        let r = run_deprecate(root, &o).unwrap();
        assert!(r.lines[0].starts_with("deprecated S001-D01 (t01)"));
        let after = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
        assert!(!after.contains("비밀 해석"), "content removed from the working tree");
        assert!(after.contains("## t01: (폐기 S001-D01)\n> **Deprecated**: S001-D01 · 2026-11-10 · t01 · 사유: 조건 오류"));
        assert!(root.join(".elf/deprecated/S001-D01.md.xz").is_file());
        assert!(run_show(root, "S001-D01").unwrap().lines.iter().any(|l| l.contains("비밀 해석")));
        assert!(pairing_findings(root).iter().all(|(k, _)| *k == 'n'));
        run_restore(root, "S001-D01").unwrap();
        let back = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
        let strip = |s: &str| s.lines().filter(|l| !l.starts_with("modified:")).collect::<Vec<_>>().join("\n");
        assert_eq!(strip(&back), strip(&before));
        assert!(!root.join(".elf/deprecated/S001-D01.md.xz").exists());
    }

    #[test]
    fn whole_session_and_section_and_pairing() {
        let tmp = project();
        let root = tmp.path();
        let mut o = opts("S001");
        o.trial = Some("t01".into());
        o.section = Some("가설".into());
        assert!(matches!(run_deprecate(root, &o), Err(DeprecateError::Refuse(m)) if m.contains("structure findings")));
        let mut w = opts("S001");
        w.replaced_by = Some("S002".into());
        run_deprecate(root, &w).unwrap();
        let text = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
        let (h, _) = header::parse(&text).unwrap();
        assert_eq!(h.status, Status::Deprecated);
        assert!(h.handoff.starts_with("폐기 S001-D01: 조건 오류"));
        assert!(!text.contains("### 목표"));
        assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("| deprecated |"));
        assert!(matches!(run_deprecate(root, &opts("S001")), Err(DeprecateError::Refuse(_))));
        // 보관 파일을 지우면 짝 검사 issue
        fs::remove_file(root.join(".elf/deprecated/S001-D01.md.xz")).unwrap();
        assert!(pairing_findings(root).iter().any(|(k, m)| *k == 'i' && m.contains("S001-D01")));
        run_list(root).unwrap();
    }

    #[test]
    fn planning_marked_block() {
        let tmp = project();
        let root = tmp.path();
        fs::create_dir_all(root.join(PLAN_BASE)).unwrap();
        fs::write(root.join(format!("{PLAN_BASE}/P007_plan.md")), "# P007: plan\n\n## 1\n<!-- deprecate:begin -->\n옛 계획\n<!-- deprecate:end -->\n- 남김\n").unwrap();
        let mut o = opts("P007");
        o.marked = true;
        run_deprecate(root, &o).unwrap();
        let t = fs::read_to_string(root.join(format!("{PLAN_BASE}/P007_plan.md"))).unwrap();
        assert!(!t.contains("옛 계획") && t.contains("> **Deprecated**: P007-D01") && t.contains("- 남김"));
        run_restore(root, "P007-D01").unwrap();
        assert!(fs::read_to_string(root.join(format!("{PLAN_BASE}/P007_plan.md"))).unwrap().contains("옛 계획"));
    }
}
