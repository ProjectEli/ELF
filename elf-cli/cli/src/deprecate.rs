//! `elf deprecate` — 세션 로그·계획 문서의 폐기 이동·복원·목록 (S038 t02 규격 B).
//!
//! 위치가 곧 상태: 폐기 문서는 기준 폴더의 `Deprecated/`로 옮긴다 — 전체는 같은 파일명,
//! 부분(trial·절·표시 블록·줄 범위)은 `<이름>.partial.md`에 블록으로 쌓고 원 위치에는 이동 표기
//! 1줄만 남긴다. 옮긴 내용의 상대 링크는 폴더 기준으로 다시 계산한다(복원 시 역방향).
//! 세션 로그의 부분 폐기는 조작 전후 구조 검사(validate)로 새 위반이 생기면 거부한다(파일 무변경).
//! 복원은 거부하지 않고 복원 뒤 문서 범위 validate·링크 확인 결과를 경고로 돌려준다.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::session::{self, Escalation, REGISTRY_REL};
use crate::validate;

pub const LOG_BASE: &str = "2_Log";
pub const PLAN_BASE: &str = "1_Concept/12_Planning";
/// 기준 폴더 아래 같은 레벨의 상태 폴더(진행 중 = 폴더 루트).
pub const STATE_DIRS: [&str; 3] = ["Wiki", "Archive", "Deprecated"];
const MARK_BEGIN: &str = "<!-- deprecate:begin -->";
const MARK_END: &str = "<!-- deprecate:end -->";
const ORIGIN_PREFIX: &str = "> **Deprecated** → ";
const BLOCK_PREFIX: &str = "> **Deprecated**: ";

#[derive(Debug)]
pub enum DeprecateError {
    /// 거부 — 파일 무변경 (exit 3)
    Refuse(String),
    /// 대상·ID 없음 (exit 1)
    NotFound(String),
    /// Registry 파싱 불가 (exit 5)
    Escalation(Box<Escalation>),
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
            DeprecateError::Escalation(e) => write!(f, "{e}"),
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
    /// `S###` · `P###` · 프로젝트 기준 경로
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

// ── 경로 계산 (프로젝트 기준, `/` 구분, 사전식) ──────────────────────────

fn norm_parts<'a>(parts: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in parts {
        match s {
            "" | "." => {}
            ".." => {
                if out.last().is_some_and(|l| l != "..") {
                    out.pop();
                } else {
                    out.push("..".into());
                }
            }
            _ => out.push(s.to_string()),
        }
    }
    out
}

fn rel_parts(target: &[String], base: &[String]) -> Vec<String> {
    let mut i = 0;
    while i < target.len() && i < base.len() && target[i] == base[i] && target[i] != ".." {
        i += 1;
    }
    let mut out: Vec<String> = vec!["..".to_string(); base.len() - i];
    out.extend(target[i..].iter().cloned());
    out
}

/// 프로젝트 기준 경로 `target_rel`을 `from_dir` 기준 상대 경로로.
pub fn rel_path(target_rel: &str, from_dir: &str) -> String {
    let r = rel_parts(&norm_parts(target_rel.split('/')), &norm_parts(from_dir.split('/')));
    if r.is_empty() { ".".into() } else { r.join("/") }
}

/// `from_dir` 기준 상대 링크 대상 `tok`을 `to_dir` 기준으로 재계산(`#fragment`·끝 `/` 보존).
pub fn rebase_token(tok: &str, from_dir: &str, to_dir: &str) -> String {
    let (path, frag) = match tok.find('#') {
        Some(i) => (&tok[..i], &tok[i..]),
        None => (tok, ""),
    };
    if path.is_empty() {
        return tok.to_string();
    }
    let trailing = path.ends_with('/');
    let tgt = norm_parts(from_dir.split('/').chain(path.split('/')));
    let rel = rel_parts(&tgt, &norm_parts(to_dir.split('/')));
    let mut new = if rel.is_empty() { ".".to_string() } else { rel.join("/") };
    if trailing && !new.ends_with('/') {
        new.push('/');
    }
    new + frag
}

/// 재계산 대상인 상대 링크 토큰인지(URL·절대경로·앵커·`<…>`·비경로 산문 제외).
fn is_relative_token(tok: &str) -> bool {
    !tok.is_empty()
        && !tok.starts_with(['#', '/', '<'])
        && !tok.contains("://")
        && !tok.starts_with("mailto:")
        && tok.chars().any(|c| c.is_alphanumeric())
}

/// 본문의 `](대상)` 상대 링크(이미지 포함)를 `from_dir` → `to_dir` 기준으로 재계산. (새 본문, 바뀐 수)
pub fn rebase_links(text: &str, from_dir: &str, to_dir: &str) -> (String, usize) {
    if from_dir == to_dir {
        return (text.to_string(), 0);
    }
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 64);
    let mut i = 0;
    let mut last = 0;
    let mut n = 0;
    while i + 1 < b.len() {
        if b[i] == b']'
            && b[i + 1] == b'('
            && let Some(close) = text[i + 2..].find(')')
        {
            let raw = &text[i + 2..i + 2 + close];
            let stripped = raw.trim_start();
            let lead = &raw[..raw.len() - stripped.len()];
            let tok = stripped.split_whitespace().next().unwrap_or("");
            let rest = &stripped[tok.len()..];
            if is_relative_token(tok) {
                let new = rebase_token(tok, from_dir, to_dir);
                if new != tok {
                    out.push_str(&text[last..i + 2]);
                    out.push_str(lead);
                    out.push_str(&new);
                    out.push_str(rest);
                    last = i + 2 + close;
                    n += 1;
                }
            }
            i = i + 2 + close + 1;
            continue;
        }
        i += 1;
    }
    out.push_str(&text[last..]);
    (out, n)
}

/// 본문의 상대 링크 토큰 목록(이미지 포함).
fn link_tokens(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b']'
            && b[i + 1] == b'('
            && let Some(close) = text[i + 2..].find(')')
        {
            let tok = text[i + 2..i + 2 + close].split_whitespace().next().unwrap_or("");
            if is_relative_token(tok) {
                out.push(tok.to_string());
            }
            i = i + 2 + close + 1;
            continue;
        }
        i += 1;
    }
    out
}

/// `dir` 기준 상대 토큰이 가리키는 파일·폴더가 root 아래에 있는지.
fn token_exists(root: &Path, dir: &str, tok: &str) -> bool {
    let path = tok.split('#').next().unwrap_or("");
    let parts = norm_parts(dir.split('/').chain(path.split('/')));
    if parts.first().is_some_and(|p| p == "..") {
        return false;
    }
    root.join(parts.join("/")).exists()
}

fn missing_link_warnings(root: &Path, dir: &str, text: &str, where_: &str) -> Vec<String> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for tok in link_tokens(text) {
        if seen.contains(&tok) {
            continue;
        }
        if !token_exists(root, dir, &tok) {
            out.push(format!("warn: link target not found: {tok} ({where_})"));
        }
        seen.push(tok);
    }
    out
}

// ── 문서 식별 ───────────────────────────────────────────────────────

fn dirname(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(d, _)| d)
}

fn basename(rel: &str) -> &str {
    rel.rsplit_once('/').map_or(rel, |(_, b)| b)
}

/// 문서의 기준 폴더 — 상태 폴더(`Wiki/`·`Archive/`·`Deprecated/`) 안이면 그 상위.
pub fn base_dir(rel: &str) -> String {
    let d = dirname(rel);
    if STATE_DIRS.contains(&basename(d)) { dirname(d).to_string() } else { d.to_string() }
}

fn is_plan_id(s: &str) -> bool {
    s.strip_prefix('P').is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

/// 문서 ID — `S###`(세션 로그) · `P###`(계획 문서 접두) · 그 외 파일명(확장자 제외).
pub fn doc_id(rel: &str) -> String {
    let name = basename(rel);
    if let Some(stem) = name.strip_suffix("_log.md")
        && session::session_num(stem).is_some()
    {
        return stem.to_string();
    }
    if let Some((head, _)) = name.split_once('_')
        && is_plan_id(head)
    {
        return head.to_string();
    }
    name.strip_suffix(".md").unwrap_or(name).to_string()
}

fn container_rel(rel: &str) -> String {
    let name = basename(rel);
    format!("{}/Deprecated/{}.partial.md", base_dir(rel), name.strip_suffix(".md").unwrap_or(name))
}

fn whole_rel(rel: &str) -> String {
    format!("{}/Deprecated/{}", base_dir(rel), basename(rel))
}

fn is_session_log(rel: &str) -> bool {
    base_dir(rel) == LOG_BASE && session::log_num(basename(rel)).is_some()
}

fn state_variants(base: &str) -> [String; 3] {
    [base.to_string(), format!("{base}/Archive"), format!("{base}/Deprecated")]
}

/// `S###` · `P###` · 경로 → 프로젝트 기준 경로.
fn resolve_doc(root: &Path, target: &str) -> Res<String> {
    if session::session_num(target).is_some() {
        for d in state_variants(LOG_BASE) {
            let rel = format!("{d}/{target}_log.md");
            if root.join(&rel).is_file() {
                return Ok(rel);
            }
        }
        return Err(DeprecateError::NotFound(format!("session {target}")));
    }
    if is_plan_id(target) {
        for d in state_variants(PLAN_BASE) {
            let mut hits: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(root.join(&d)) {
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if name.starts_with(&format!("{target}_"))
                        && name.ends_with(".md")
                        && !name.ends_with(".partial.md")
                    {
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
    let b = base_dir(rel);
    if b != LOG_BASE && b != PLAN_BASE {
        return refuse(format!(
            "out of scope: {rel} — elf deprecate handles session logs (2_Log/) and planning docs (1_Concept/12_Planning/)"
        ));
    }
    if rel.ends_with(".partial.md") {
        return refuse(format!("{rel} is a deprecated-blocks file, not a document"));
    }
    Ok(())
}

// ── 줄 범위 탐지 (코드 펜스 안의 줄은 헤딩·표시로 보지 않음) ─────────────

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

/// `### 헤딩 (Gloss)` → 키(validate와 같은 규칙).
fn section_key(line: &str) -> Option<&str> {
    let rest = clean(line).strip_prefix("### ")?;
    let key = match rest.find(" (") {
        Some(i) => &rest[..i],
        None => rest,
    };
    Some(key.trim())
}

/// trial 범위 — (헤딩 줄, 본문 시작, 본문 끝[배타]). 본문 앞뒤 빈 줄과 끝 구분선(`---`) 제외.
fn trial_bounds(lines: &[&str], tid: &str) -> Res<(usize, usize, usize)> {
    let mask = fence_mask(lines);
    let h = lines
        .iter()
        .enumerate()
        .position(|(i, l)| !mask[i] && trial_heading_id(l).as_deref() == Some(tid))
        .ok_or_else(|| DeprecateError::NotFound(format!("trial {tid}")))?;
    let mut nxt = lines.len();
    for i in h + 1..lines.len() {
        if !mask[i] && clean(lines[i]).starts_with("## ") {
            nxt = i;
            break;
        }
    }
    let mut e = nxt;
    while e > h + 1 && clean(lines[e - 1]).trim().is_empty() {
        e -= 1;
    }
    if e > h + 1 && clean(lines[e - 1]).trim() == "---" {
        e -= 1;
        while e > h + 1 && clean(lines[e - 1]).trim().is_empty() {
            e -= 1;
        }
    }
    let mut s = h + 1;
    while s < e && clean(lines[s]).trim().is_empty() {
        s += 1;
    }
    if s >= e {
        return refuse(format!("{tid} has no body to deprecate"));
    }
    Ok((h, s, e))
}

/// 절 범위 — (trial 헤딩 줄, 절 헤딩 줄, 끝[배타]). 절 헤딩 포함, 끝 빈 줄 제외.
fn section_bounds(lines: &[&str], tid: &str, key: &str) -> Res<(usize, usize, usize)> {
    let (h, s, e) = trial_bounds(lines, tid)?;
    let mask = fence_mask(lines);
    let a = (s..e)
        .find(|&i| !mask[i] && section_key(lines[i]) == Some(key))
        .ok_or_else(|| DeprecateError::NotFound(format!("section {tid} {key}")))?;
    let mut b = e;
    for i in a + 1..e {
        if !mask[i] && clean(lines[i]).starts_with("### ") {
            b = i;
            break;
        }
    }
    while b > a + 1 && clean(lines[b - 1]).trim().is_empty() {
        b -= 1;
    }
    Ok((h, a, b))
}

/// idx 위쪽의 가장 가까운 trial 헤딩(같은 trial 안일 때) — (줄, id).
fn enclosing_trial(lines: &[&str], idx: usize) -> Option<(usize, String)> {
    let mask = fence_mask(lines);
    for i in (0..=idx.min(lines.len().saturating_sub(1))).rev() {
        if mask[i] {
            continue;
        }
        if let Some(tid) = trial_heading_id(lines[i]) {
            return Some((i, tid));
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
            return section_key(l).map(str::to_string);
        }
        if l.starts_with("## ") || l.starts_with("# ") {
            return None;
        }
    }
    None
}

/// idx 위쪽의 가장 가까운 2단계 이상 헤딩 줄(맥락 헤딩 사본용).
fn nearest_heading(lines: &[&str], idx: usize) -> Option<usize> {
    let mask = fence_mask(lines);
    (0..=idx.min(lines.len().saturating_sub(1))).rev().find(|&i| {
        !mask[i] && {
            let l = clean(lines[i]);
            l.starts_with("##") && l.trim_start_matches('#').starts_with(' ')
        }
    })
}

// ── ID·표기 줄·YAML ──────────────────────────────────────────────────

/// `<문서 ID>-D<NN>` — 원 문서와 부분 파일에 남아 있는 최대 번호 + 1.
fn next_block_id(doc: &str, origin: &str, container: Option<&str>) -> String {
    let prefix = format!("{doc}-D");
    let mut max = 0u32;
    for text in [Some(origin), container].into_iter().flatten() {
        for (idx, _) in text.match_indices(&prefix) {
            let digits: String =
                text[idx + prefix.len()..].chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                max = max.max(n);
            }
        }
    }
    format!("{doc}-D{:02}", max + 1)
}

fn is_block_id(s: &str) -> bool {
    s.rsplit_once("-D")
        .is_some_and(|(doc, n)| !doc.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn origin_marker(origin_rel: &str, crel: &str, id: &str, unit: &str, date: &str, replaced_by: Option<&str>) -> String {
    let link = rel_path(crel, dirname(origin_rel));
    let rb = replaced_by.map(|r| format!(" · replaced_by: {r}")).unwrap_or_default();
    format!("{ORIGIN_PREFIX}[Deprecated/{}]({link}) ({id} · {unit} · {date}{rb})", basename(crel))
}

fn block_marker(date: &str, id: &str, unit: &str, replaced_by: Option<&str>, reason: Option<&str>) -> String {
    let mut s = format!("{BLOCK_PREFIX}{date} · {id} · {unit}");
    if let Some(r) = replaced_by {
        s.push_str(&format!(" · replaced_by: {r}"));
    }
    if let Some(r) = reason {
        s.push_str(&format!(" · reason: {r}"));
    }
    s
}

fn first_title(text: &str) -> Option<&str> {
    text.lines().map(clean).find(|l| l.starts_with("# "))
}

fn yaml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn yaml_unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\")
    } else {
        t.to_string()
    }
}

/// 맨 앞 YAML header(`---` … `---`) → (키·값, 닫는 `---` 줄 index). 없으면 None.
fn parse_yaml_header(lines: &[&str]) -> Option<(BTreeMap<String, String>, usize)> {
    if clean(lines.first()?) != "---" {
        return None;
    }
    let end = (1..lines.len()).find(|&i| clean(lines[i]) == "---")?;
    let mut map = BTreeMap::new();
    for l in &lines[1..end] {
        if let Some((k, v)) = clean(l).split_once(':') {
            map.insert(k.trim().to_string(), yaml_unquote(v));
        }
    }
    Some((map, end))
}

/// 헤더 `> **Status**:` 값을 바꿈(끝 `\`·`\r` 보존). 없으면 None.
fn set_header_status(content: &str, value: &str) -> Option<String> {
    let mut out: Vec<String> = content.split('\n').map(str::to_string).collect();
    for line in out.iter_mut() {
        let trimmed = line.strip_suffix('\r').unwrap_or(line);
        if !trimmed.starts_with("> **Status**:") {
            continue;
        }
        let mut new = format!("> **Status**: {value}");
        if trimmed.ends_with('\\') {
            new.push('\\');
        }
        if line.ends_with('\r') {
            new.push('\r');
        }
        *line = new;
        return Some(out.join("\n"));
    }
    None
}

fn eol_of(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

// ── Registry (전체 폐기·복원) ──────────────────────────────────────────

/// 행의 Status·경로 열 갱신(타 행·EOL 보존). 반환 = 변경 전 (status, path).
fn registry_set(reg_text: &str, sid: &str, status: &str, path: &str) -> (String, Option<(String, String)>) {
    let mut out = String::new();
    let mut old = None;
    for piece in reg_text.split_inclusive('\n') {
        let (body, nl) = match piece.strip_suffix('\n') {
            Some(b) => (b, "\n"),
            None => (piece, ""),
        };
        let core = body.strip_suffix('\r').unwrap_or(body);
        let cr = if body.ends_with('\r') { "\r" } else { "" };
        let cols: Vec<&str> = core.split('\t').collect();
        if cols.len() == session::REGISTRY_COLS && cols[0] == sid {
            old = Some((cols[3].to_string(), cols[5].to_string()));
            out.push_str(&format!("{}\t{}\t{}\t{status}\t{}\t{path}{cr}{nl}", cols[0], cols[1], cols[2], cols[4]));
        } else {
            out.push_str(piece);
        }
    }
    (out, old)
}

fn load_registry(root: &Path) -> Res<String> {
    let text = fs::read_to_string(root.join(REGISTRY_REL)).unwrap_or_default();
    session::parse_registry(&text).map_err(DeprecateError::Escalation)?;
    Ok(text)
}

// ── 부분 폐기 (순수) ───────────────────────────────────────────────────

struct BlockSpec {
    /// 원 문서 lines[a..b)를 옮김
    a: usize,
    b: usize,
    unit: String,
    ctx_heading: Option<String>,
}

struct MoveOut {
    origin: String,
    container: String,
    id: String,
    unit: String,
    /// 옮긴 원문(재계산 전)
    stored: String,
    /// 부분 파일에 기록된 본문(재계산 후)
    rebased: String,
}

/// lines[a..b)를 이동 표기 1줄로 바꾸고 부분 파일에 블록을 추가한 결과(파일 무변경).
#[allow(clippy::too_many_arguments)]
fn move_block_pure(
    origin_text: &str,
    container_text: Option<&str>,
    origin_rel: &str,
    crel: &str,
    spec: &BlockSpec,
    date: &str,
    replaced_by: Option<&str>,
    reason: Option<&str>,
) -> MoveOut {
    let eol = eol_of(origin_text);
    let lines: Vec<&str> = origin_text.split('\n').collect();
    let doc = doc_id(origin_rel);
    let id = next_block_id(&doc, origin_text, container_text);
    let stored = lines[spec.a..spec.b].join("\n");
    let (rebased, _) = rebase_links(&stored, dirname(origin_rel), dirname(crel));

    let mut cont = match container_text {
        Some(c) => c.to_string(),
        None => {
            let mut c = format!("---\ndeprecated: partial\nsource: {origin_rel}\n---\n\n");
            if let Some(t) = first_title(origin_text) {
                c.push_str(t);
                c.push('\n');
            }
            c
        }
    };
    if !cont.ends_with('\n') {
        cont.push('\n');
    }
    cont.push('\n');
    if let Some(h) = &spec.ctx_heading {
        cont.push_str(clean(h));
        cont.push('\n');
    }
    cont.push_str(&block_marker(date, &id, &spec.unit, replaced_by, reason));
    cont.push_str("\n\n");
    cont.push_str(&format!("<!-- deprecated:begin {id} -->\n{rebased}\n<!-- deprecated:end {id} -->\n"));

    let mut marker = origin_marker(origin_rel, crel, &id, &spec.unit, date, replaced_by);
    if eol == "\r\n" {
        marker.push('\r');
    }
    let mut new_lines: Vec<&str> = Vec::with_capacity(lines.len());
    new_lines.extend_from_slice(&lines[..spec.a]);
    new_lines.push(&marker);
    new_lines.extend_from_slice(&lines[spec.b..]);
    MoveOut { origin: new_lines.join("\n"), container: cont, id, unit: spec.unit.clone(), stored, rebased }
}

fn guard_no_trial_heading(lines: &[&str], a: usize, b: usize) -> Res<()> {
    let mask = fence_mask(lines);
    for i in a..b {
        if !mask[i] && trial_heading_id(lines[i]).is_some() {
            return refuse(format!("block contains a trial heading ({}) — use --trial", clean(lines[i])));
        }
        if !mask[i] && clean(lines[i]).starts_with("# ") {
            return refuse("block contains the document title line");
        }
    }
    Ok(())
}

fn block_unit_ctx(lines: &[&str], a: usize) -> (String, Option<String>) {
    if let Some((hi, tid)) = enclosing_trial(lines, a) {
        let unit = match enclosing_section(lines, a) {
            Some(sec) => format!("{tid} {sec} part"),
            None => format!("{tid} part"),
        };
        return (unit, Some(lines[hi].to_string()));
    }
    let ctx = nearest_heading(lines, a).map(|i| lines[i].to_string());
    ("part".to_string(), ctx)
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

/// 표시 줄(`<!-- deprecate:begin/end -->`)만 뺀 본문 — 조작 전 구조 검사의 기준.
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

// ── review: 판단 필요 항목 수집 (수정하지 않음) ──────────────────────────

fn walk_docs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if name.starts_with('.') || name == "Deprecated" || name == "target" || name == "node_modules" {
                continue;
            }
            walk_docs(&p, out);
        } else if name.ends_with(".md") || name.ends_with(".tsv") {
            out.push(p);
        }
    }
}

/// `doc`(+ 선택적 `tid`)에 대한 언급이 줄에 있는지 — `S012 t03`·`S012·t03`·`S012_t03` 등.
fn mentions(line: &str, doc: &str, tid: Option<&str>) -> bool {
    for (idx, _) in line.match_indices(doc) {
        let before_ok = idx == 0 || !line[..idx].chars().next_back().is_some_and(|c| c.is_alphanumeric());
        let after = &line[idx + doc.len()..];
        let after_ok = !after.chars().next().is_some_and(|c| c.is_alphanumeric());
        if !(before_ok && after_ok) {
            continue;
        }
        match tid {
            None => return true,
            Some(t) => {
                let rest = after.trim_start_matches([' ', '·', '_', '-', '/']);
                if rest.starts_with(t) && !rest[t.len()..].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
    }
    false
}

/// 판단이 필요한 항목(출력만): 원 문서의 Handoff·같은 문서 안의 다른 trial에서의 언급, Registry key finding,
/// 다른 문서의 참조(`scan_refs`), 옮긴 내용 속 figure embed·생성 파일 경로. 원 문서 본문은 조작 후 상태(`origin_text`)로 본다.
fn review_items(
    root: &Path,
    origin_rel: &str,
    origin_text: Option<&str>,
    doc: &str,
    tid: Option<&str>,
    stored: &str,
    scan_refs: bool,
) -> Vec<String> {
    let mut items = Vec::new();
    if let (Some(text), Some(t)) = (origin_text, tid) {
        let lines: Vec<&str> = text.split('\n').collect();
        // 해당 trial 자신의 범위(헤딩 ~ 다음 `## `)는 제외 — 자기 산출물 이름(S002_t02.m 등) 언급은 참조가 아님
        let mask = fence_mask(&lines);
        let own = lines
            .iter()
            .enumerate()
            .position(|(i, l)| !mask[i] && trial_heading_id(l).as_deref() == Some(t))
            .map(|h| {
                let nxt = (h + 1..lines.len()).find(|&i| !mask[i] && clean(lines[i]).starts_with("## ")).unwrap_or(lines.len());
                (h, nxt)
            });
        for (i, l) in lines.iter().enumerate() {
            let l = clean(l);
            if own.is_some_and(|(h, nxt)| i >= h && i < nxt) || l.starts_with(ORIGIN_PREFIX) {
                continue;
            }
            if l.starts_with("> **Handoff**:") {
                if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == t) {
                    items.push(format!("review: handoff: {origin_rel}:{} mentions {t}", i + 1));
                }
            } else if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == t) {
                items.push(format!("review: ref: {origin_rel}:{} mentions {t}", i + 1));
            }
        }
    }
    if session::session_num(doc).is_some()
        && let Ok(reg) = fs::read_to_string(root.join(REGISTRY_REL))
    {
        for (i, l) in reg.lines().enumerate() {
            let cols: Vec<&str> = clean(l).split('\t').collect();
            if cols.len() == session::REGISTRY_COLS && cols[0] == doc && cols[4] != "-" && !cols[4].is_empty() {
                items.push(format!(
                    "review: registry: {REGISTRY_REL}:{} key finding of {doc} — check whether it relies on the deprecated content",
                    i + 1
                ));
            }
        }
    }
    if scan_refs {
        let mut docs = Vec::new();
        walk_docs(root, &mut docs);
        for p in docs {
            let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            if rel == origin_rel || rel == REGISTRY_REL {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else { continue };
            for (i, l) in text.lines().enumerate() {
                if mentions(clean(l), doc, tid) {
                    let what = match tid {
                        Some(t) => format!("{doc} {t}"),
                        None => doc.to_string(),
                    };
                    items.push(format!("review: ref: {rel}:{} refers to {what}", i + 1));
                }
            }
        }
    }
    for tok in link_tokens(stored) {
        let lower = tok.to_ascii_lowercase();
        if stored.contains(&format!("]({tok}"))
            && stored.contains("![")
            && (lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".svg"))
        {
            items.push(format!("review: figure: {tok} is embedded in the deprecated content (file left in place)"));
        }
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

fn scoped_validate_warnings(root: &Path, sid: &str) -> Vec<String> {
    let Ok(v) = validate::run_validate_opts(root, false) else { return Vec::new() };
    let by_file = format!("{sid}_log.md");
    let by_id = format!("{sid}:");
    v.lines
        .iter()
        .filter(|l| {
            let body = l.strip_prefix("issue: ").or_else(|| l.strip_prefix("warn: ")).unwrap_or(l);
            body.starts_with(&by_file) || body.starts_with(&by_id)
        })
        .map(|l| format!("warn: validate ({sid}): {l} — resolve or state an intentional exception"))
        .collect()
}

// ── 실행기: 폐기 ──────────────────────────────────────────────────────

pub fn run_deprecate(root: &Path, o: &DeprecateOptions) -> Res<Report> {
    let rel = resolve_doc(root, &o.target)?;
    check_scope(&rel)?;
    if basename(dirname(&rel)) == "Deprecated" {
        return refuse(format!("{rel} is already deprecated"));
    }
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
    if units == 0 {
        return deprecate_whole(root, &rel, o);
    }
    if o.trial.is_some() && !is_session_log(&rel) {
        return refuse("--trial/--section apply to session logs only (planning docs: --marked or --lines)");
    }
    deprecate_blocks(root, &rel, o)
}

fn deprecate_blocks(root: &Path, rel: &str, o: &DeprecateOptions) -> Res<Report> {
    let crel = container_rel(rel);
    let original = fs::read_to_string(root.join(rel))?;
    let cpath = root.join(&crel);
    let mut container: Option<String> = if cpath.is_file() { Some(fs::read_to_string(&cpath)?) } else { None };
    let mut text = original.clone();
    let mut moved: Vec<MoveOut> = Vec::new();
    let mut tids: Vec<Option<String>> = Vec::new();

    loop {
        let lines: Vec<&str> = text.split('\n').collect();
        let (spec, tid) = if let Some(tid) = &o.trial {
            if !moved.is_empty() {
                break;
            }
            if let Some(key) = &o.section {
                let (h, a, b) = section_bounds(&lines, tid, key)?;
                (BlockSpec { a, b, unit: format!("{tid} {key}"), ctx_heading: Some(lines[h].to_string()) }, Some(tid.clone()))
            } else {
                let (h, s, e) = trial_bounds(&lines, tid)?;
                (BlockSpec { a: s, b: e, unit: tid.clone(), ctx_heading: Some(lines[h].to_string()) }, Some(tid.clone()))
            }
        } else if o.marked {
            let Some((a, b)) = first_marked_block(&lines)? else {
                if moved.is_empty() {
                    return refuse(format!("no {MARK_BEGIN} … {MARK_END} block in {rel}"));
                }
                break;
            };
            guard_no_trial_heading(&lines, a + 1, b - 1)?;
            let (unit, ctx) = block_unit_ctx(&lines, a);
            let tid = enclosing_trial(&lines, a).map(|(_, t)| t);
            // 표시 줄 2개는 버리고 그 사이만 저장 — 범위는 표시 줄을 포함해 교체
            let inner = lines[a + 1..b - 1].to_vec();
            if inner.is_empty() {
                return refuse("empty marked block");
            }
            (BlockSpec { a, b, unit, ctx_heading: ctx }, tid)
        } else {
            let (a1, b1) = o.lines.unwrap();
            if !moved.is_empty() {
                break;
            }
            if a1 == 0 || b1 < a1 || b1 > lines.len() {
                return refuse(format!("--lines {a1}-{b1} out of range (1..{})", lines.len()));
            }
            let expect = o.expect.as_deref().unwrap_or("");
            if clean(lines[a1 - 1]).trim() != expect.trim() {
                return refuse(format!("--expect mismatch at line {a1}: found \"{}\"", clean(lines[a1 - 1])));
            }
            guard_no_trial_heading(&lines, a1 - 1, b1)?;
            let (unit, ctx) = block_unit_ctx(&lines, a1 - 1);
            let tid = enclosing_trial(&lines, a1 - 1).map(|(_, t)| t);
            (BlockSpec { a: a1 - 1, b: b1, unit, ctx_heading: ctx }, tid)
        };

        let mut out = move_block_pure(
            &text,
            container.as_deref(),
            rel,
            &crel,
            &spec,
            &o.date,
            o.replaced_by.as_deref(),
            o.reason.as_deref(),
        );
        if o.marked {
            // 표시 블록: 저장 원문 = 표시 줄 사이(이미 lines[a+1..b-1]) — move_block_pure는 a..b 전체를 저장하므로 보정
            let inner: Vec<&str> = text.split('\n').collect::<Vec<_>>()[spec.a + 1..spec.b - 1].to_vec();
            let stored = inner.join("\n");
            let (rebased, _) = rebase_links(&stored, dirname(rel), dirname(&crel));
            out.container = out.container.replace(&format!("-->\n{}\n<!-- deprecated:end", out.rebased), &format!("-->\n{rebased}\n<!-- deprecated:end"));
            out.stored = stored;
            out.rebased = rebased;
        }
        text = out.origin.clone();
        container = Some(out.container.clone());
        moved.push(out);
        tids.push(tid);
    }

    // 조작 후 구조 검사 — 세션 로그에서 새 위반이 생기면 거부(파일 무변경)
    if is_session_log(rel) {
        let new = new_structure_findings(&strip_marks(&original), &text);
        if !new.is_empty() {
            return refuse(format!(
                "the move would introduce structure findings in {rel} — choose another unit (e.g. the whole trial): {}",
                new.join(" / ")
            ));
        }
    }

    let mut rep = Report::default();
    let doc = doc_id(rel);
    for m in &moved {
        rep.lines.push(format!("deprecated {} ({}) → {crel}", m.id, m.unit));
    }
    if !o.dry_run {
        if let Some(dir) = cpath.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&cpath, container.as_deref().unwrap_or(""))?;
        fs::write(root.join(rel), &text)?;
        rep.changed = true;
    }
    // review (출력만) — dry-run은 조작 전 상태에서 수집
    let mut review = Vec::new();
    for (m, tid) in moved.iter().zip(tids.iter()) {
        for it in review_items(root, rel, Some(&text), &doc, tid.as_deref(), &m.stored, tid.is_some()) {
            if !review.contains(&it) {
                review.push(it);
            }
        }
    }
    let n = review.len();
    rep.lines.extend(review);
    if n > 0 {
        rep.lines.push(format!("review: {n} item(s) — not modified by elf; resolve each or confirm no change is needed"));
    }
    for m in &moved {
        rep.lines.extend(missing_link_warnings(root, dirname(&crel), &m.rebased, &crel));
    }
    Ok(rep)
}

fn deprecate_whole(root: &Path, rel: &str, o: &DeprecateOptions) -> Res<Report> {
    let dst = whole_rel(rel);
    if root.join(&dst).exists() {
        return refuse(format!("{dst} already exists"));
    }
    let text = fs::read_to_string(root.join(rel))?;
    let status_before = session::header_status(&text);
    let body = match &status_before {
        Some(_) => set_header_status(&text, "Deprecated").unwrap_or_else(|| text.clone()),
        None => text.clone(),
    };
    let (body, _) = rebase_links(&body, dirname(rel), dirname(&dst));
    let mut y = format!("---\ndeprecated: {}\nsource: {rel}\n", o.date);
    if let Some(r) = &o.replaced_by {
        y.push_str(&format!("replaced_by: {}\n", yaml_quote(r)));
    }
    if let Some(r) = &o.reason {
        y.push_str(&format!("reason: {}\n", yaml_quote(r)));
    }
    if let Some(s) = &status_before {
        y.push_str(&format!("status_before: {}\n", yaml_quote(s)));
    }
    y.push_str("---\n\n");

    let is_log = is_session_log(rel);
    let sid = doc_id(rel);
    let reg_text = if is_log { Some(load_registry(root)?) } else { None };

    let mut rep = Report::default();
    rep.lines.push(format!("deprecated {sid} (whole) → {dst}"));
    if !o.dry_run {
        if let Some(dir) = root.join(&dst).parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(root.join(&dst), format!("{y}{body}"))?;
        fs::remove_file(root.join(rel))?;
        if let Some(reg) = reg_text {
            let (new_reg, _) = registry_set(&reg, &sid, "Deprecated", &format!("Deprecated/{}", basename(rel)));
            fs::write(root.join(REGISTRY_REL), new_reg)?;
            rep.lines.push(format!("registry: {sid} → Deprecated, Deprecated/{}", basename(rel)));
        }
        if status_before.is_some() {
            rep.lines.push(format!("header: Status → Deprecated (status_before kept in {dst})"));
        }
        rep.changed = true;
    }
    let review = review_items(root, rel, None, &sid, None, &text, true);
    let n = review.len();
    rep.lines.extend(review);
    if n > 0 {
        rep.lines.push(format!("review: {n} item(s) — not modified by elf; resolve each or confirm no change is needed"));
    }
    rep.lines.extend(missing_link_warnings(root, dirname(&dst), &body, &dst));
    Ok(rep)
}

// ── 실행기: 복원 ──────────────────────────────────────────────────────

pub fn run_restore(root: &Path, id: &str) -> Res<Report> {
    if is_block_id(id) {
        return restore_block(root, id);
    }
    let rel = resolve_doc(root, id)?;
    if basename(dirname(&rel)) != "Deprecated" {
        return refuse(format!("{rel} is not deprecated"));
    }
    restore_whole(root, &rel)
}

fn deprecated_dirs(root: &Path) -> Vec<(String, PathBuf)> {
    [LOG_BASE, PLAN_BASE]
        .into_iter()
        .map(|b| (format!("{b}/Deprecated"), root.join(b).join("Deprecated")))
        .filter(|(_, p)| p.is_dir())
        .collect()
}

fn find_origin(root: &Path, crel: &str) -> Res<String> {
    let base = base_dir(crel);
    let name = basename(crel).replace(".partial.md", ".md");
    for d in state_variants(&base) {
        let rel = format!("{d}/{name}");
        if root.join(&rel).is_file() {
            return Ok(rel);
        }
    }
    Err(DeprecateError::NotFound(format!("origin document for {crel}")))
}

fn restore_block(root: &Path, id: &str) -> Res<Report> {
    let begin = format!("<!-- deprecated:begin {id} -->");
    let end = format!("<!-- deprecated:end {id} -->");
    let mut hit: Option<(String, String)> = None;
    for (drel, dpath) in deprecated_dirs(root) {
        let Ok(entries) = fs::read_dir(&dpath) else { continue };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".partial.md"))
            .collect();
        names.sort();
        for n in names {
            let text = fs::read_to_string(dpath.join(&n))?;
            if text.contains(&begin) {
                hit = Some((format!("{drel}/{n}"), text));
            }
        }
    }
    let Some((crel, ctext)) = hit else {
        return Err(DeprecateError::NotFound(format!("block {id}")));
    };
    let clines: Vec<&str> = ctext.split('\n').collect();
    let b = clines.iter().position(|l| clean(l) == begin).unwrap();
    let Some(e) = clines.iter().position(|l| clean(l) == end) else {
        return refuse(format!("{crel}: {begin} without matching end marker"));
    };
    if b < 2 || !clean(clines[b - 1]).is_empty() || !clean(clines[b - 2]).starts_with(BLOCK_PREFIX) {
        return refuse(format!("{crel}: unexpected layout around {id} (expected marker line, blank line, begin)"));
    }
    let m = b - 2;
    let mut start = if m > 0 && clean(clines[m - 1]).starts_with("##") { m - 1 } else { m };
    if start > 0 && clean(clines[start - 1]).is_empty() {
        start -= 1;
    }
    let unit = clean(clines[m])[BLOCK_PREFIX.len()..].split(" · ").nth(2).unwrap_or("").to_string();
    let stored = clines[b + 1..e].join("\n");

    let origin_rel = find_origin(root, &crel)?;
    let otext = fs::read_to_string(root.join(&origin_rel))?;
    let olines: Vec<&str> = otext.split('\n').collect();
    let needle = format!("({id} ·");
    let ks: Vec<usize> = olines
        .iter()
        .enumerate()
        .filter(|(_, l)| clean(l).starts_with(ORIGIN_PREFIX) && l.contains(&needle))
        .map(|(i, _)| i)
        .collect();
    if ks.len() != 1 {
        return refuse(format!("move marker for {id} not found exactly once in {origin_rel} ({} found)", ks.len()));
    }
    let (block, _) = rebase_links(&stored, dirname(&crel), dirname(&origin_rel));
    let mut new_lines: Vec<&str> = Vec::with_capacity(olines.len() + 8);
    new_lines.extend_from_slice(&olines[..ks[0]]);
    new_lines.extend(block.split('\n'));
    new_lines.extend_from_slice(&olines[ks[0] + 1..]);
    fs::write(root.join(&origin_rel), new_lines.join("\n"))?;

    let mut rest: Vec<&str> = Vec::new();
    rest.extend_from_slice(&clines[..start]);
    rest.extend_from_slice(&clines[e + 1..]);
    let cpath = root.join(&crel);
    if rest.iter().any(|l| clean(l).starts_with("<!-- deprecated:begin ")) {
        fs::write(&cpath, rest.join("\n"))?;
    } else {
        fs::remove_file(&cpath)?;
    }

    let mut rep = Report { lines: vec![format!("restored {id} → {origin_rel} ({unit})")], changed: true };
    rep.lines.extend(missing_link_warnings(root, dirname(&origin_rel), &block, &origin_rel));
    if is_session_log(&origin_rel) {
        rep.lines.extend(scoped_validate_warnings(root, &doc_id(&origin_rel)));
    }
    Ok(rep)
}

fn restore_whole(root: &Path, rel: &str) -> Res<Report> {
    let text = fs::read_to_string(root.join(rel))?;
    let lines: Vec<&str> = text.split('\n').collect();
    let Some((meta, end)) = parse_yaml_header(&lines) else {
        return refuse(format!("{rel} has no deprecated YAML header"));
    };
    let Some(src) = meta.get("source") else {
        return refuse(format!("{rel}: YAML header has no `source`"));
    };
    if root.join(src).exists() {
        return refuse(format!("{src} already exists — move it away before restoring"));
    }
    let mut body_start = end + 1;
    if body_start < lines.len() && clean(lines[body_start]).is_empty() {
        body_start += 1;
    }
    let body = lines[body_start..].join("\n");
    let (body, _) = rebase_links(&body, dirname(rel), dirname(src));
    let body = match meta.get("status_before") {
        Some(s) => set_header_status(&body, s).unwrap_or(body),
        None => body,
    };
    let is_log = is_session_log(src);
    let reg_text = if is_log { Some(load_registry(root)?) } else { None };
    if let Some(dir) = root.join(src).parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(root.join(src), &body)?;
    fs::remove_file(root.join(rel))?;
    let sid = doc_id(src);
    let mut rep = Report { lines: vec![format!("restored {sid} → {src} (whole)")], changed: true };
    if let Some(reg) = reg_text {
        let in_archive = basename(dirname(src)) == "Archive";
        let status = session::header_status(&body)
            .unwrap_or_else(|| if in_archive { "Complete".into() } else { "★ 활성".into() });
        let path = if in_archive { format!("Archive/{}", basename(src)) } else { "-".to_string() };
        let (new_reg, _) = registry_set(&reg, &sid, &status, &path);
        fs::write(root.join(REGISTRY_REL), new_reg)?;
        rep.lines.push(format!("registry: {sid} → {status}, {path}"));
    }
    rep.lines.extend(missing_link_warnings(root, dirname(src), &body, src));
    if is_log {
        rep.lines.extend(scoped_validate_warnings(root, &sid));
    }
    Ok(rep)
}

// ── 실행기: list (Deprecated/ 폴더 순회 + 표기↔블록 대조) ─────────────────

pub fn run_list(root: &Path) -> Res<Report> {
    let mut rep = Report::default();
    let mut n_docs = 0;
    let mut n_blocks = 0;
    let mut warns = Vec::new();
    for (drel, dpath) in deprecated_dirs(root) {
        let Ok(entries) = fs::read_dir(&dpath) else { continue };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".md"))
            .collect();
        names.sort();
        for n in names {
            let rel = format!("{drel}/{n}");
            let text = fs::read_to_string(dpath.join(&n))?;
            let lines: Vec<&str> = text.split('\n').collect();
            let Some((meta, _)) = parse_yaml_header(&lines) else {
                warns.push(format!("warn: {rel}: no deprecated YAML header"));
                continue;
            };
            if meta.get("deprecated").map(String::as_str) == Some("partial") {
                let mut ids = Vec::new();
                for l in &lines {
                    if let Some(rest) = clean(l).strip_prefix(BLOCK_PREFIX) {
                        let f: Vec<&str> = rest.split(" · ").collect();
                        if f.len() >= 3 {
                            rep.lines.push(format!("{} · {} · {} · {rel}", f[1], f[0], f[2..].join(" · ")));
                            ids.push(f[1].to_string());
                            n_blocks += 1;
                        }
                    }
                }
                match find_origin(root, &rel) {
                    Ok(orel) => {
                        let otext = fs::read_to_string(root.join(&orel)).unwrap_or_default();
                        for id in &ids {
                            if !otext.contains(&format!("({id} ·")) {
                                warns.push(format!("warn: {id}: no move marker in {orel}"));
                            }
                        }
                        for l in otext.lines() {
                            if let Some(rest) = clean(l).strip_prefix(ORIGIN_PREFIX)
                                && let Some(p) = rest.find(" (")
                            {
                                let id = rest[p + 2..].split(" · ").next().unwrap_or("").to_string();
                                if !id.is_empty() && !ids.contains(&id) {
                                    warns.push(format!("warn: {id}: move marker in {orel} has no block in {rel}"));
                                }
                            }
                        }
                    }
                    Err(e) => warns.push(format!("warn: {rel}: {e}")),
                }
            } else {
                n_docs += 1;
                let mut line = format!(
                    "{} · {} · whole · source: {}",
                    doc_id(&rel),
                    meta.get("deprecated").map(String::as_str).unwrap_or("?"),
                    meta.get("source").map(String::as_str).unwrap_or("?")
                );
                for k in ["replaced_by", "reason"] {
                    if let Some(v) = meta.get(k) {
                        line.push_str(&format!(" · {k}: {v}"));
                    }
                }
                line.push_str(&format!(" · {rel}"));
                rep.lines.push(line);
            }
        }
    }
    rep.lines.push(format!("deprecated: {n_docs} document(s), {n_blocks} block(s)"));
    rep.lines.extend(warns);
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebase_token_root_to_deprecated_and_back() {
        assert_eq!(rebase_token("../6_Exp/64_Viz/S002/f.png", "2_Log", "2_Log/Deprecated"), "../../6_Exp/64_Viz/S002/f.png");
        assert_eq!(rebase_token("S011_log.md", "2_Log", "2_Log/Deprecated"), "../S011_log.md");
        assert_eq!(rebase_token("Archive/S001_log.md", "2_Log", "2_Log/Deprecated"), "../Archive/S001_log.md");
        assert_eq!(rebase_token("Wiki/a.md#sec", "2_Log", "2_Log/Deprecated"), "../Wiki/a.md#sec");
        // 역방향(복원)
        assert_eq!(rebase_token("../../6_Exp/x.png", "2_Log/Deprecated", "2_Log"), "../6_Exp/x.png");
        assert_eq!(rebase_token("../Archive/S001_log.md", "2_Log/Deprecated", "2_Log/Archive"), "S001_log.md");
        // 같은 깊이(Archive → Deprecated): `../` 링크 불변, 같은 폴더 링크는 `../Archive/`
        assert_eq!(rebase_token("../../1_Concept/p.md", "2_Log/Archive", "2_Log/Deprecated"), "../../1_Concept/p.md");
        assert_eq!(rebase_token("S001_log.md", "2_Log/Archive", "2_Log/Deprecated"), "../Archive/S001_log.md");
        // 끝 `/` 보존
        assert_eq!(rebase_token("../6_Exp/61_Sim/Data/S002/", "2_Log", "2_Log/Deprecated"), "../../6_Exp/61_Sim/Data/S002/");
    }

    #[test]
    fn rebase_links_skips_urls_anchors_and_prose() {
        let t = "[a](../x.md) ![f](../6_Exp/f.png) [u](https://e.com/a.md) [h](#sec) [p](/abs.md) `](...)` [t](../q.md \"제목\")";
        let (o, n) = rebase_links(t, "2_Log", "2_Log/Deprecated");
        assert_eq!(n, 3, "{o}");
        assert!(o.contains("[a](../../x.md)"));
        assert!(o.contains("![f](../../6_Exp/f.png)"));
        assert!(o.contains("[t](../../q.md \"제목\")"));
        assert!(o.contains("(https://e.com/a.md)") && o.contains("(#sec)") && o.contains("(/abs.md)") && o.contains("`](...)`"));
        assert_eq!(rebase_links(t, "2_Log", "2_Log").1, 0);
    }

    #[test]
    fn doc_id_and_paths() {
        assert_eq!(doc_id("2_Log/S012_log.md"), "S012");
        assert_eq!(doc_id("2_Log/Archive/S012_log.md"), "S012");
        assert_eq!(doc_id("1_Concept/12_Planning/P007_Plan.md"), "P007");
        assert_eq!(doc_id("1_Concept/12_Planning/Notes.md"), "Notes");
        assert_eq!(base_dir("2_Log/Archive/S012_log.md"), "2_Log");
        assert_eq!(base_dir("2_Log/S012_log.md"), "2_Log");
        assert_eq!(container_rel("2_Log/Archive/S012_log.md"), "2_Log/Deprecated/S012_log.partial.md");
        assert_eq!(whole_rel("1_Concept/12_Planning/P007_Plan.md"), "1_Concept/12_Planning/Deprecated/P007_Plan.md");
        assert_eq!(rel_path("2_Log/Deprecated/S012_log.partial.md", "2_Log"), "Deprecated/S012_log.partial.md");
        assert_eq!(rel_path("2_Log/Deprecated/S012_log.partial.md", "2_Log/Archive"), "../Deprecated/S012_log.partial.md");
    }

    #[test]
    fn trial_and_section_bounds_skip_fences_and_separators() {
        let log = "# S1: T\n\n## t01: a\n\n### 목표 (Goal)\n- x\n\n```\n## t09: not a trial\n### 해석 (x)\n```\n\n### 해석 (Interpretation)\n- 가설 적중 여부: 적중\n\n---\n\n## t02: b\n\n### 목표 (Goal)\n- y\n";
        let lines: Vec<&str> = log.split('\n').collect();
        let (h, s, e) = trial_bounds(&lines, "t01").unwrap();
        assert_eq!(lines[h], "## t01: a");
        assert_eq!(lines[s], "### 목표 (Goal)");
        assert_eq!(lines[e - 1], "- 가설 적중 여부: 적중");
        let (_, a, b) = section_bounds(&lines, "t01", "해석").unwrap();
        assert_eq!(lines[a], "### 해석 (Interpretation)");
        assert_eq!(b, e);
        assert!(trial_bounds(&lines, "t09").is_err());
        let (_, s2, e2) = trial_bounds(&lines, "t02").unwrap();
        assert_eq!(&lines[s2..e2], &["### 목표 (Goal)", "- y"]);
    }

    #[test]
    fn block_ids_increment_from_both_sides() {
        assert_eq!(next_block_id("S012", "no ids", None), "S012-D01");
        assert_eq!(next_block_id("S012", "> **Deprecated** → [x](y) (S012-D03 · t01 · d)", Some("<!-- deprecated:begin S012-D01 -->")), "S012-D04");
        assert!(is_block_id("S012-D01") && is_block_id("P007-D12") && !is_block_id("S012") && !is_block_id("x-D"));
    }

    #[test]
    fn marker_and_yaml_roundtrip() {
        let m = origin_marker("2_Log/Archive/S012_log.md", "2_Log/Deprecated/S012_log.partial.md", "S012-D01", "t03 해석", "2026-10-05", Some("S015 t02"));
        assert_eq!(m, "> **Deprecated** → [Deprecated/S012_log.partial.md](../Deprecated/S012_log.partial.md) (S012-D01 · t03 해석 · 2026-10-05 · replaced_by: S015 t02)");
        let q = yaml_quote("a \"b\" \\ c");
        assert_eq!(yaml_unquote(&q), "a \"b\" \\ c");
        let lines = vec!["---", "deprecated: 2026-10-05", "source: 2_Log/S003_log.md", "reason: \"x: y\"", "---", "", "# S003"];
        let (meta, end) = parse_yaml_header(&lines).unwrap();
        assert_eq!(end, 4);
        assert_eq!(meta["reason"], "x: y");
        assert_eq!(meta["source"], "2_Log/S003_log.md");
    }

    #[test]
    fn header_status_set_preserves_hardbreak() {
        let c = "# T\n\n> **Created**: x\\\n> **Status**: ★ 활성\\\n> **Handoff**: -\n";
        let o = set_header_status(c, "Deprecated").unwrap();
        assert!(o.contains("> **Status**: Deprecated\\\n"));
        assert_eq!(set_header_status(&o, "★ 활성").unwrap(), c);
    }

    #[test]
    fn mentions_matches_session_and_trial_forms() {
        assert!(mentions("S002 t02의 결과", "S002", Some("t02")));
        assert!(mentions("S002·t02", "S002", Some("t02")));
        assert!(!mentions("S002 t020", "S002", Some("t02")));
        assert!(!mentions("S0021 t02", "S002", Some("t02")));
        assert!(mentions("see S002", "S002", None));
    }
}
