//! 구조 이관 — 이전 형식 로그와 폐기 기록을 새 구조로 (P022 §7, LogConvention §7). `elf update`가 부름.
//!
//! 대상 판정 = 파일 상태(버전 아님): 구 위치 로그(`2_Log/S###_log.md`, `Archive/`, `Deprecated/`), 인용 블록 헤더,
//! `Session_Registry.tsv`, 계획 문서 `Deprecated/`. 전제 = git 작업 트리가 깨끗함(되돌림 = git).
//! 계획을 먼저 세우고 경고가 하나라도 있으면 아무것도 쓰지 않고 결정 파일(`.elf/runtime/migration.yaml`)을 만듦 —
//! status 칸을 채우고 목록의 파일을 고친 뒤 `elf update`를 다시 실행하면 한 번에 적용하고 결정 파일을 지움.
//!
//! 하는 일: 헤더 → YAML 7키(같은 뜻 필드 대응, 그 밖의 필드 삭제와 보고), 로그 → `2_Log/S/`(링크 재계산),
//! 폐기 기록(`*.partial.md`, `Deprecated/` 전체, purge 흔적) → `.elf/deprecated/` 압축 보관과 새 표기 줄,
//! `Registry.md`와 탐색기 nesting 생성, TSV 삭제. 이관 뒤 구버전 CLI가 만든 구 위치 로그는 다음 빈 번호로 옮김.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::deprecate;
use crate::header::{self, Header, Status};
use crate::logs::{self, Series};
use crate::sections::LogLang;
use crate::session;

pub const DECISION_REL: &str = ".elf/runtime/migration.yaml";
const PLAN_BASE: &str = "1_Concept/12_Planning";
const OLD_ORIGIN: &str = "> **Deprecated** → ";
const OLD_BLOCK: &str = "> **Deprecated**: ";
/// 7키에 없는 이전 헤더 필드를 남기는 본문 줄(제목 바로 아래)
pub const KEPT_PREFIX: &str = "> 이관 전 헤더: ";
/// 영어 프로젝트의 보존 줄 머리(v3.4, S050 t02).
pub const KEPT_PREFIX_EN: &str = "> Header before migration: ";

fn kept_prefix(lang: LogLang) -> &'static str {
    match lang {
        LogLang::Ko => KEPT_PREFIX,
        LogLang::En => KEPT_PREFIX_EN,
    }
}

/// 이관 전에 지워진(purged) 흔적의 꼬리: 표기 줄 끝 + 괄호 설명.
fn purged_tail(lang: LogLang, what: &str) -> String {
    match lang {
        LogLang::Ko => format!("{}({what})", deprecate::PURGED_TAG),
        LogLang::En => format!("{} ({what})", deprecate::PURGED_TAG_EN),
    }
}

// ── 판정, 전제 ────────────────────────────────────────────────────

/// 이관이 필요한가(파일 상태로 판정).
pub fn needed(root: &Path) -> bool {
    !logs::legacy_markers(root).is_empty()
        || root.join(PLAN_BASE).join("Deprecated").is_dir()
        || logs::scan_all(root).iter().any(|l| {
            fs::read_to_string(&l.path).is_ok_and(|t| header::parse(&t).is_err() && legacy_header(&lf(&t)).is_some())
        })
}

/// 전제: git 작업 트리가 깨끗함(이 폴더 범위, 결정 파일 제외)과 커밋이 하나 이상. Err = 이유.
pub fn git_precondition(root: &Path) -> Result<(), String> {
    let git = |args: &[&str]| std::process::Command::new("git").args(args).current_dir(root).output();
    let o = match git(&["status", "--porcelain", "--untracked-files=all", "--", "."]) {
        Ok(o) if o.status.success() => o,
        _ => return Err("not a git repository — the migration is undone with git, so put the project under git and commit first".into()),
    };
    let dirty: Vec<String> = String::from_utf8_lossy(&o.stdout)
        .lines()
        .filter(|l| !l.contains(".elf/runtime/"))
        .map(|l| l.get(3..).unwrap_or(l).to_string())
        .collect();
    if !dirty.is_empty() {
        let more = if dirty.len() > 5 { " …" } else { "" };
        return Err(format!(
            "git working tree is not clean ({} path(s): {}{more}) — commit or stash first; the migration is undone with git",
            dirty.len(),
            dirty.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if !git(&["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok_and(|h| h.status.success()) {
        return Err("no git commit yet — commit first (the migration is undone with git)".into());
    }
    Ok(())
}

// ── 결정 파일 ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// 대상 경로(이관 전)
    pub log: String,
    pub kind: String,
    pub header: Option<String>,
    pub registry: Option<String>,
    pub evidence: Option<String>,
    pub detail: Option<String>,
    /// `status` 칸(값으로 정하는 경고). false = 파일을 고친 뒤 재실행
    pub fill: bool,
}

impl Warning {
    fn fix(log: &str, kind: &str, detail: String) -> Warning {
        Warning { log: log.into(), kind: kind.into(), header: None, registry: None, evidence: None, detail: Some(detail), fill: false }
    }

    pub fn line(&self) -> String {
        let mut s = format!("{}: {}", self.log, self.kind);
        if self.fill {
            s.push_str(&format!(
                " (header '{}', Registry '{}') — fill status",
                self.header.as_deref().unwrap_or("-"),
                self.registry.as_deref().unwrap_or("-")
            ));
        }
        if let Some(d) = &self.detail {
            s.push_str(&format!(" — {d}"));
        }
        s
    }
}

/// 결정 파일 → (로그 경로 → status 원문). 파일이 없으면 빈 목록.
pub fn read_decisions(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = fs::read_to_string(root.join(DECISION_REL)) else {
        return out;
    };
    let mut cur: Option<String> = None;
    for l in lf(&text).lines() {
        let t = l.trim();
        let (k, v) = match t.strip_prefix("- ").unwrap_or(t).split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        let v = if v.starts_with('\'') { header::unquote(v).unwrap_or_else(|_| v.to_string()) } else { v.to_string() };
        match k {
            "log" => cur = Some(v),
            "status" => {
                if let Some(log) = &cur
                    && !v.trim().is_empty()
                {
                    out.insert(log.clone(), v.trim().to_string());
                }
            }
            _ => {}
        }
    }
    out
}

fn decision_text(warnings: &[Warning], prior: &BTreeMap<String, String>) -> String {
    let mut s = String::from(
        "# elf update — 이관 결정 파일 (LogConvention §7). git 추적 밖(.elf/runtime/), 적용되면 elf가 지움.\n\
         # status 칸이 있는 항목: 근거(evidence)와 로그 본문을 보고 ongoing, complete, deprecated 중 하나를 사용자 확인 뒤 채움.\n\
         # status 칸이 없는 항목: detail의 파일을 고친 뒤 재실행. 모두 맞으면 `elf update`가 한 번에 적용함.\n",
    );
    for w in warnings {
        s.push_str(&format!("- log: {}\n  warning: {}\n", header::quote(&w.log), header::quote(&w.kind)));
        for (k, v) in [("header", &w.header), ("registry", &w.registry), ("evidence", &w.evidence), ("detail", &w.detail)] {
            if let Some(v) = v {
                s.push_str(&format!("  {k}: {}\n", header::quote(v)));
            }
        }
        if w.fill {
            s.push_str(&format!("  status: {}\n", header::quote(prior.get(&w.log).map(String::as_str).unwrap_or(""))));
        }
    }
    s
}

// ── 계획 ──────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Plan {
    /// (루트 기준 경로, 내용)
    pub writes: Vec<(String, Vec<u8>)>,
    pub deletes: Vec<String>,
    /// 비면 지울 폴더
    pub dirs: Vec<String>,
    pub warnings: Vec<Warning>,
    /// 보고 줄
    pub lines: Vec<String>,
    /// 다시 계산한 링크와 코드 span(이관 뒤 파일 기준 줄)
    pub changes: Vec<Change>,
    /// 로그별 헤더 값(미리보기 표)
    pub headers: Vec<HeaderRow>,
    /// 링크가 아닌 옛 경로, 지운 파일 이름 언급
    pub mentions: Vec<Mention>,
}

/// 다시 계산한 링크나 코드 span 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// 루트 기준 경로(이관 뒤)
    pub file: String,
    /// 이관 뒤 파일의 줄 번호(1부터)
    pub line: usize,
    /// `link` 또는 `span`
    pub kind: &'static str,
    pub before: String,
    pub after: String,
}

/// 로그 하나의 이관 뒤 헤더 값과 그 원천.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderRow {
    pub old: String,
    pub new: String,
    /// 정해지지 않았으면 빈 값(경고)
    pub status: String,
    pub created: String,
    pub modified: String,
    pub goal: String,
    /// `header <필드>`, `none (body: <줄>)`, `none`
    pub goal_source: String,
    pub handoff: String,
    pub conclusion: String,
    /// `header 결론`, `Registry Key Finding`, 빈 값
    pub conclusion_source: String,
    /// 제목 아래 `> 이관 전 헤더:` 줄의 내용
    pub kept: String,
    /// 확인할 점(경고 종류, conclusion 앞머리 상태어)
    pub note: String,
}

/// 링크가 아닌 옛 경로나 지운 파일 이름이 나오는 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    pub file: String,
    pub line: usize,
    /// 찾은 이름(옛 경로 또는 지운 파일 이름)
    pub name: String,
    pub text: String,
}

struct Source {
    old_rel: String,
    series: Series,
    num: u32,
    text: String,
    crlf: bool,
    /// `Deprecated/` 전체 폐기 파일의 YAML 메타(deprecated, source, reason, replaced_by, status_before)
    meta: BTreeMap<String, String>,
}

struct Legacy {
    title: String,
    fields: Vec<(String, String)>,
    body: String,
}

#[derive(Clone)]
struct Block {
    unit: String,
    date: String,
    replaced_by: String,
    reason: String,
    content: String,
    container: String,
}

pub fn lf(t: &str) -> String {
    t.trim_start_matches('\u{feff}').replace("\r\n", "\n")
}

fn dirname(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(d, _)| d)
}

fn basename(rel: &str) -> &str {
    rel.rsplit_once('/').map_or(rel, |(_, b)| b)
}

fn rel_of(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

fn field(l: &str) -> Option<(String, String)> {
    let r = l.trim_end_matches('\r').strip_prefix("> **")?;
    let (k, v) = r.split_once("**:")?;
    Some((k.trim().to_string(), v.trim().trim_end_matches('\\').trim().to_string()))
}

/// 인용 블록 헤더(`# 제목` 뒤 `> **Key**: value\` 줄들, 이어지는 `---` 구분선) → 제목, 필드, 본문.
fn legacy_header(text: &str) -> Option<Legacy> {
    let lines: Vec<&str> = text.split('\n').collect();
    let ti = lines.iter().take(10).position(|l| l.starts_with("# "))?;
    let mut i = ti + 1;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    let mut fields = Vec::new();
    while i < lines.len() {
        match field(lines[i]) {
            Some(f) => fields.push(f),
            None => break,
        }
        i += 1;
    }
    if fields.is_empty() {
        return None;
    }
    let mut j = i;
    while j < lines.len() && lines[j].trim().is_empty() {
        j += 1;
    }
    if j < lines.len() && lines[j].trim() == "---" {
        j += 1;
        while j < lines.len() && lines[j].trim().is_empty() {
            j += 1;
        }
    }
    Some(Legacy { title: lines[ti].to_string(), fields, body: lines[j..].join("\n") })
}

/// 맨 앞 YAML(`---` … `---`) → (키와 값, 나머지). 큰따옴표 값은 풂.
fn split_meta(text: &str) -> (BTreeMap<String, String>, String) {
    let mut m = BTreeMap::new();
    let Some(rest) = text.strip_prefix("---\n") else {
        return (m, text.to_string());
    };
    let Some(end) = rest.find("\n---\n") else {
        return (m, text.to_string());
    };
    for l in rest[..end].lines() {
        if let Some((k, v)) = l.split_once(':') {
            let v = v.trim();
            let v = if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
                v[1..v.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\")
            } else {
                v.to_string()
            };
            m.insert(k.trim().to_string(), v);
        }
    }
    (m, rest[end + 5..].trim_start_matches('\n').to_string())
}

/// 상태 신호: 없음, 정식 값(상태, purged 여부), 정식 값 아님.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Sig {
    Absent,
    Canon(Status, bool),
    Odd(String),
}

fn sig(raw: Option<&str>) -> Sig {
    let Some(r) = raw.map(str::trim).filter(|r| !r.is_empty() && *r != "-") else {
        return Sig::Absent;
    };
    match r {
        "★ 활성" | "ongoing" => Sig::Canon(Status::Ongoing, false),
        "Complete" | "complete" => Sig::Canon(Status::Complete, false),
        "Deprecated" | "deprecated" => Sig::Canon(Status::Deprecated, false),
        "Purged" => Sig::Canon(Status::Deprecated, true),
        _ => Sig::Odd(r.to_string()),
    }
}

/// 두 신호 → 상태 또는 경고 종류.
fn decide(h: &Sig, r: &Sig) -> Result<(Status, bool), &'static str> {
    match (h, r) {
        (Sig::Canon(a, pa), Sig::Canon(b, pb)) if a == b => Ok((*a, *pa || *pb)),
        (Sig::Canon(a, p), Sig::Absent) | (Sig::Absent, Sig::Canon(a, p)) => Ok((*a, *p)),
        (Sig::Absent, Sig::Absent) => Err("no status in the header or the Registry"),
        (Sig::Odd(_), _) | (_, Sig::Odd(_)) => Err("status is not a canonical value"),
        _ => Err("header Status and Registry status disagree"),
    }
}

fn sig_text(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim).filter(|r| !r.is_empty()).map(str::to_string)
}

fn date_of(v: &str) -> Option<String> {
    let d = v.trim().get(..10)?;
    let b = d.as_bytes();
    (b.len() == 10 && b[4] == b'-' && b[7] == b'-' && d.bytes().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()))
        .then(|| d.to_string())
}

fn clean_value(v: &str) -> String {
    let v = header::one_line(v);
    if v.trim() == "-" { String::new() } else { v.trim().to_string() }
}

fn git_last_date(root: &Path, rel: &str) -> Option<String> {
    let o = std::process::Command::new("git").args(["log", "-1", "--format=%cs", "--", rel]).current_dir(root).output().ok()?;
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    date_of(&s)
}

fn evidence(body: &str, has_conclusion: bool) -> String {
    let last = body.lines().rfind(|l| l.starts_with("## t")).map(|l| l.trim_start_matches("## ").trim().to_string());
    let concl = has_conclusion
        || body.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("## 결론") || t.starts_with("### 결론") || t.starts_with("**결론**") || t.starts_with("- **결론**")
        });
    let dep = body.contains("> **Deprecated**");
    let yn = |b: bool| if b { "yes" } else { "no" };
    format!("last trial: {}; conclusion line: {}; deprecation marker: {}", last.unwrap_or_else(|| "-".into()), yn(concl), yn(dep))
}

/// 규범 주석의 시작 표지(한국어 틀, 영어 틀 — P017).
const NORM_MARKS: [&str; 2] = ["<!-- ELF 규범", "<!-- ELF norms"];

/// 내장 sessionTemplate(프로젝트 언어)의 규범 주석(`<!-- ELF 규범 … -->`, `<!-- ELF norms … -->`).
fn norm_comment(lang: LogLang) -> String {
    let t = session::template_text(lang);
    let Some(a) = NORM_MARKS.iter().find_map(|m| t.find(m)) else { return String::new() };
    let Some(e) = t[a..].find("-->") else { return String::new() };
    t[a..a + e + 3].to_string()
}

/// 본문 앞쪽의 옛 규범 주석을 현행 주석으로 바꿈. 바꿨으면 true.
fn refresh_norm(body: &str, current: &str) -> (String, bool) {
    if current.is_empty() {
        return (body.to_string(), false);
    }
    let lines: Vec<&str> = body.split('\n').collect();
    let Some(a) = lines.iter().take(30).position(|l| NORM_MARKS.iter().any(|m| l.trim_start().starts_with(m))) else {
        return (body.to_string(), false);
    };
    let Some(b) = (a..lines.len()).find(|&i| lines[i].contains("-->")) else {
        return (body.to_string(), false);
    };
    let old = lines[a..=b].join("\n");
    if old.trim() == current.trim() {
        return (body.to_string(), false);
    }
    let mut out: Vec<String> = lines[..a].iter().map(|s| s.to_string()).collect();
    out.push(current.to_string());
    out.extend(lines[b + 1..].iter().map(|s| s.to_string()));
    (out.join("\n"), true)
}

// ── 링크 ──────────────────────────────────────────────────────────

/// `a/b/../c` → `a/c`. 루트 밖으로 나가면 None.
/// 루트 밖은 앞의 `..`로 남김(예: 프레임워크 `_dev`의 `../../elf-cli/…` — 파일 시스템에서 확인 가능).
fn normalize(p: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for c in p.split('/') {
        match c {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|l| *l != "..") {
                    parts.pop();
                } else {
                    parts.push("..");
                }
            }
            x => parts.push(x),
        }
    }
    Some(parts.join("/"))
}

fn join(dir: &str, rel: &str) -> Option<String> {
    normalize(&if dir.is_empty() { rel.to_string() } else { format!("{dir}/{rel}") })
}

/// `target`(루트 기준)을 `from_dir`(루트 기준) 기준 상대 경로로.
fn relpath(target: &str, from_dir: &str) -> String {
    let t: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let f: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let common = t.iter().zip(&f).take_while(|(a, b)| a == b).count();
    let mut out: Vec<&str> = vec![".."; f.len() - common];
    out.extend(&t[common..]);
    out.join("/")
}

/// 다시 쓴 결과.
struct Rewritten {
    text: String,
    /// 바뀐 링크 수
    links: usize,
    /// 바뀐 코드 span 경로 수
    spans: usize,
    /// 없는 .md 대상(이관 전 기준) `줄: 대상`
    broken: Vec<String>,
    /// (입력 기준 줄, `link`|`span`, 전, 후)
    changes: Vec<(usize, &'static str, String, String)>,
}

/// 본문의 상대 링크(`](대상)`, 코드 밖)와 상대 경로 하나뿐인 코드 span(`` `../x/y.md` ``, 사용자 결정 2026-11-10)을
/// `old_dir` 기준에서 `new_dir` 기준으로 다시 계산하고, 옮겨지는 대상은 `moves`로 바꿈. 대상이 실제로 있거나 옮겨지는
/// 파일일 때만, 표기가 달라질 때만 바꿈.
fn rewrite_links(root: &Path, text: &str, old_dir: &str, new_dir: &str, moves: &BTreeMap<String, String>) -> Rewritten {
    let mut r = Rewritten { text: String::with_capacity(text.len()), links: 0, spans: 0, broken: Vec::new(), changes: Vec::new() };
    let mut fence = false;
    for (ln, line) in text.split_inclusive('\n').enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
            r.text.push_str(line);
            continue;
        }
        if fence || (!line.contains("](") && !line.contains('`')) {
            r.text.push_str(line);
            continue;
        }
        let b = line.as_bytes();
        let mut i = 0;
        let mut last = 0;
        while i + 1 < b.len() {
            if b[i] == b'`' {
                // 코드 span = 한 단위(안의 링크 모양은 링크가 아님). 상대 경로 하나뿐이면 다시 계산
                let Some(close) = line[i + 1..].find('`') else { break };
                let span = &line[i + 1..i + 1 + close];
                if is_rel_path(span) {
                    let mut ignore = Vec::new();
                    if let Some(new) = recompute(root, span, old_dir, new_dir, moves, &mut ignore, ln + 1) {
                        r.text.push_str(&line[last..i + 1]);
                        r.text.push_str(&new);
                        last = i + 1 + close;
                        r.spans += 1;
                        r.changes.push((ln + 1, "span", span.to_string(), new));
                    }
                }
                i += close + 2;
                continue;
            }
            if b[i] == b']' && b[i + 1] == b'(' {
                let start = i + 2;
                let Some(close) = line[start..].find(')') else { break };
                let raw = &line[start..start + close];
                if let Some((path_part, tail)) = split_target(raw)
                    && let Some(new_target) = recompute(root, path_part, old_dir, new_dir, moves, &mut r.broken, ln + 1)
                {
                    r.text.push_str(&line[last..start]);
                    r.text.push_str(&new_target);
                    r.text.push_str(tail);
                    last = start + close;
                    r.links += 1;
                    r.changes.push((ln + 1, "link", path_part.trim().to_string(), new_target.trim().to_string()));
                }
                i = start + close + 1;
                continue;
            }
            i += 1;
        }
        r.text.push_str(&line[last..]);
    }
    r
}

/// 코드 span이 상대 경로 하나인가(`../`나 `./`로 시작, 공백 없음).
fn is_rel_path(s: &str) -> bool {
    (s.starts_with("../") || s.starts_with("./")) && !s.contains(char::is_whitespace) && s.len() > 3
}

/// 링크 괄호 안 → (경로, 나머지: `#조각`, 제목). 링크가 아니면 None — `<…>` 형식, 공백 뒤가 제목(`"`, `'`, `(`)이
/// 아닌 경우(CommonMark: 목적지에 공백 불가 — 예: 산문 `[DOC](CHANGELOG, CLI.md)`).
fn split_target(raw: &str) -> Option<(&str, &str)> {
    let lead = raw.len() - raw.trim_start().len();
    let r = &raw[lead..];
    if r.starts_with('<') {
        return None;
    }
    let ws = r.find(char::is_whitespace).unwrap_or(r.len());
    let after = r[ws..].trim_start();
    if !after.is_empty() && !after.starts_with(['"', '\'', '(']) {
        return None;
    }
    let end = r[..ws].find(['#', '?']).unwrap_or(ws);
    Some((&raw[..lead + end], &raw[lead + end..]))
}

fn recompute(
    root: &Path,
    path_part: &str,
    old_dir: &str,
    new_dir: &str,
    moves: &BTreeMap<String, String>,
    broken: &mut Vec<String>,
    line_no: usize,
) -> Option<String> {
    let lead = &path_part[..path_part.len() - path_part.trim_start().len()];
    let p = path_part.trim();
    if p.is_empty() || p.starts_with('/') || p.contains("://") || p.starts_with("mailto:") || p.starts_with('#') {
        return None;
    }
    let abs = join(old_dir, p)?;
    let decoded = abs.replace("%20", " ");
    let target = moves.get(&abs).or_else(|| moves.get(&decoded)).cloned().unwrap_or_else(|| abs.clone());
    // 대상이 없으면(옮겨지는 파일도 아니면) 바꾸지 않음 — 산문 속 `[REL](main)` 같은 링크 모양 텍스트 보호.
    // .md 대상은 깨진 링크로 알림(check logs와 같은 기준)
    if !moves.contains_key(&abs) && !moves.contains_key(&decoded) && !root.join(&abs).exists() && !root.join(&decoded).exists() {
        if p.ends_with(".md") {
            broken.push(format!("line {line_no}: {p}"));
        }
        return None;
    }
    // 새 위치에서도 같은 대상을 가리키면 표기를 바꾸지 않음(`./x.md`, 끝 `/` 같은 원래 표기 유지)
    if join(new_dir, p).as_deref() == Some(target.as_str()) || join(new_dir, &p.replace("%20", " ")).as_deref() == Some(target.as_str()) {
        return None;
    }
    let mut new = relpath(&target, new_dir);
    if new.is_empty() {
        new.push('.');
    }
    // 폴더 표기의 끝 `/` 유지(`../../docs/` → `../../../docs/`, 3.0.1)
    if p.ends_with('/') && !new.ends_with('/') {
        new.push('/');
    }
    (new != p).then(|| format!("{lead}{new}"))
}

/// 변경 목록의 줄 번호를 결과 파일(`text`) 기준으로. 예상 위치 = 입력 줄 + `delta`(결과와 입력의 줄 수 차이 — 줄 수가
/// 바뀌는 곳은 본문 앞의 헤더와 규범 주석뿐). 차례를 지키며 `after`가 든 줄 가운데 예상 위치에 가장 가까운 줄을 고름 —
/// 같은 문자열이 앞 줄에 바뀌지 않은 채 있어도 그 줄을 보고하지 않게(3.0.1). 못 찾으면 예상 위치.
fn locate(file: &str, text: &str, raw: Vec<(usize, &'static str, String, String)>, delta: isize, out: &mut Vec<Change>) {
    let lines: Vec<&str> = text.lines().collect();
    // 다음 탐색 시작 줄: 같은 입력 줄의 변경이면 그 줄부터, 다음 입력 줄이면 그 다음 줄부터
    let mut prev: Option<(usize, usize)> = None; // (입력 줄, 찾은 결과 줄 index)
    for (ln, kind, before, after) in raw {
        let needle = if kind == "span" { format!("`{after}`") } else { format!("]({after}") };
        let start = match prev {
            Some((pl, pi)) if pl == ln => pi,
            Some((_, pi)) => pi + 1,
            None => 0,
        };
        let expect = (ln as isize - 1 + delta).max(0) as usize;
        let found = (start.min(lines.len())..lines.len()).filter(|&i| lines[i].contains(&needle)).min_by_key(|&i| i.abs_diff(expect));
        let line = match found {
            Some(i) => {
                prev = Some((ln, i));
                i + 1
            }
            None => (expect + 1).min(lines.len().max(1)),
        };
        out.push(Change { file: file.to_string(), line, kind, before, after });
    }
}

/// conclusion 앞머리의 상태어(Registry TSV 결론 열에 상태를 함께 적던 관행). 단어 뒤가 끝이나 글자 아닌 문자일 때만.
fn status_word_prefix(v: &str) -> Option<&'static str> {
    const WORDS: [&str; 18] = [
        "진행 중", "진행중", "진행", "완료", "보관", "종료", "폐기", "중단", "활성", "★", "Completed", "Complete", "Done", "Closed", "Archived",
        "Deprecated", "Ongoing", "Active",
    ];
    let t = v.trim_start().trim_start_matches(['[', '(', '【', '<', '*']).trim_start();
    WORDS.into_iter().find(|w| {
        t.get(..w.len()).is_some_and(|h| h.eq_ignore_ascii_case(w)) && t[w.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
    })
}

/// 첫 trial 앞 본문의 목표 줄(`**목표**: …`, `목표: …`, `## 목표` 등) — 헤더에 목표가 없을 때 원천 안내용.
fn body_goal_line(body: &str) -> Option<String> {
    for l in body.lines() {
        if l.starts_with("## t") {
            break;
        }
        let t = l.trim_start_matches(['#', '>', '-', '*', ' ']).trim_start();
        let lower = t.to_lowercase();
        if t.starts_with("목표") || lower.starts_with("goal") || t.starts_with("목적") || lower.starts_with("objective") {
            let s = header::one_line(l.trim());
            return Some(if s.chars().count() > 80 { format!("{}…", s.chars().take(80).collect::<String>()) } else { s });
        }
    }
    None
}

/// git 추적 파일(루트 기준 경로, `/` 구분). git이 아니면 None.
fn tracked(root: &Path) -> Option<Vec<String>> {
    let o = std::process::Command::new("git").args(["ls-files", "-z"]).current_dir(root).output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&o.stdout).split('\0').filter(|s| !s.is_empty()).map(str::to_string).collect())
}

/// 링크 재계산 대상 문서(`.md`) — 추적 파일만(git이 아니면 폴더 전체). 점 폴더, `node_modules`, `target` 제외.
fn link_docs(root: &Path, tracked: Option<&[String]>) -> Vec<PathBuf> {
    match tracked {
        Some(t) => t
            .iter()
            .filter(|r| r.ends_with(".md") && !r.split('/').any(|c| c.starts_with('.') || c == "node_modules" || c == "target"))
            .map(|r| root.join(r))
            .collect(),
        None => {
            let mut d = Vec::new();
            walk_md(root, &mut d);
            d
        }
    }
}

// ── 폐기 기록 ─────────────────────────────────────────────────────

/// `.partial.md` → (원래 문서 경로, ID → 블록).
fn parse_container(rel: &str, text: &str) -> (Option<String>, BTreeMap<String, Block>) {
    let (meta, body) = split_meta(text);
    let mut blocks = BTreeMap::new();
    let mut pending: Option<(String, String, String, String, String)> = None; // id, unit, date, rb, reason
    let mut cur: Option<(String, Vec<&str>)> = None;
    for l in body.split('\n') {
        let c = l.trim_end_matches('\r');
        if let Some((id, lines)) = &mut cur {
            if c.trim() == format!("<!-- deprecated:end {id} -->") {
                let (pid, unit, date, rb, reason) = pending.take().filter(|p| &p.0 == id).unwrap_or_default();
                let _ = pid;
                blocks.insert(
                    id.clone(),
                    Block { unit, date, replaced_by: rb, reason, content: lines.join("\n"), container: rel.to_string() },
                );
                cur = None;
            } else {
                lines.push(l);
            }
            continue;
        }
        if let Some(rest) = c.strip_prefix(OLD_BLOCK) {
            let f: Vec<&str> = rest.split(" · ").collect();
            if f.len() >= 3 {
                let mut rb = String::new();
                let mut reason = String::new();
                for x in &f[3..] {
                    if let Some(v) = x.strip_prefix("replaced_by: ") {
                        rb = v.to_string();
                    } else if let Some(v) = x.strip_prefix("reason: ") {
                        reason = v.to_string();
                    }
                }
                pending = Some((f[1].trim().to_string(), f[2].trim().to_string(), f[0].trim().to_string(), rb, reason));
            }
        } else if let Some(id) = c.trim().strip_prefix("<!-- deprecated:begin ").and_then(|r| r.strip_suffix(" -->")) {
            cur = Some((id.trim().to_string(), Vec::new()));
        }
    }
    (meta.get("source").cloned(), blocks)
}

/// 옛 흔적 줄 `> **Deprecated** → [..](..) (ID · 단위 · 날짜[ · replaced_by: X])`, `→ purged (… · purged: D)` →
/// (ID, 단위, 날짜, 대체, purged 날짜).
fn parse_origin(line: &str) -> Option<(String, String, String, String, Option<String>)> {
    let rest = line.trim_end_matches('\r').strip_prefix(OLD_ORIGIN)?;
    let open = rest.rfind(" (").map(|i| i + 2).or_else(|| rest.strip_prefix("purged (").map(|_| "purged (".len()))?;
    let inner = rest[open..].strip_suffix(')')?;
    let f: Vec<&str> = inner.split(" · ").collect();
    if f.len() < 3 {
        return None;
    }
    let mut rb = String::new();
    let mut purged = None;
    for x in &f[3..] {
        if let Some(v) = x.strip_prefix("replaced_by: ") {
            rb = v.to_string();
        } else if let Some(v) = x.strip_prefix("purged: ") {
            purged = Some(v.to_string());
        }
    }
    Some((f[0].trim().to_string(), f[1].trim().to_string(), f[2].trim().to_string(), rb, purged))
}

fn is_trial_unit(u: &str) -> bool {
    u.len() >= 2 && u.starts_with('t') && u[1..].bytes().all(|b| b.is_ascii_digit())
}

struct Stores<'a> {
    root: &'a Path,
    used: BTreeSet<String>,
    out: Vec<(String, String)>, // (ID, 보관 원문)
}

impl Stores<'_> {
    fn alloc(&mut self, doc: &str) -> String {
        let mut n = 1;
        loop {
            let id = format!("{doc}-D{n:02}");
            if !self.used.contains(&id) && !self.root.join(deprecate::store_rel(&id)).exists() {
                self.used.insert(id.clone());
                return id;
            }
            n += 1;
        }
    }
}

/// 문서 본문의 옛 흔적 줄을 새 표기로 바꾸고 블록을 보관 목록에 넣음.
#[allow(clippy::too_many_arguments)]
fn convert_origins(
    root: &Path,
    text: &str,
    doc_rel_old: &str,
    doc_rel_new: &str,
    blocks: &mut BTreeMap<String, Block>,
    stores: &mut Stores,
    moves: &BTreeMap<String, String>,
    warnings: &mut Vec<Warning>,
) -> (String, usize) {
    if !text.contains(OLD_ORIGIN) {
        return (text.to_string(), 0);
    }
    let lang = LogLang::of(root);
    let mut n = 0;
    let mut out: Vec<String> = Vec::new();
    for l in text.split('\n') {
        let Some((id, unit, date, rb, purged)) = parse_origin(l) else {
            out.push(l.to_string());
            continue;
        };
        let w = deprecate::words(lang);
        let heading = if is_trial_unit(&unit) { Some(format!("## {unit}: ({} {id})", w.deprecated)) } else { None };
        let mark = if let Some(d) = purged {
            format!("{}{}", deprecate::marker_line(&id, &date, &unit, None, Some(&rb), lang), purged_tail(lang, &format!("purged {d}")))
        } else if let Some(b) = blocks.remove(&id) {
            let content = rewrite_links(root, &b.content, dirname(&b.container), dirname(doc_rel_new), moves).text;
            let meta: Vec<(&str, String)> = vec![
                ("id", id.clone()),
                ("source", doc_rel_new.to_string()),
                ("unit", unit.clone()),
                ("date", date.clone()),
                ("reason", b.reason.clone()),
                ("replaced_by", if rb.is_empty() { b.replaced_by.clone() } else { rb.clone() }),
            ];
            stores.used.insert(id.clone());
            stores.out.push((id.clone(), deprecate::store_text(&meta, &content)));
            deprecate::marker_line(&id, &date, &unit, Some(&b.reason), Some(&rb), lang)
        } else {
            warnings.push(Warning::fix(
                doc_rel_old,
                "deprecation marker without its block",
                format!("{id}: no `<!-- deprecated:begin {id} -->` block in the .partial.md file — restore it from git or delete the marker line"),
            ));
            out.push(l.to_string());
            continue;
        };
        if let Some(h) = heading {
            out.push(h);
        }
        out.push(mark);
        n += 1;
    }
    (out.join("\n"), n)
}

// ── 계획 세우기 ───────────────────────────────────────────────────

fn read_dir_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    v.sort();
    v
}

fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) {
    for p in read_dir_sorted(dir) {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if p.is_dir() {
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                walk_md(&p, out);
            }
        } else if name.ends_with(".md") {
            out.push(p);
        }
    }
}

fn collect_sources(root: &Path, p: &mut Plan) -> Vec<Source> {
    let mut v = Vec::new();
    for d in logs::OLD_LOG_DIRS {
        for path in read_dir_sorted(&root.join(d)) {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if !path.is_file() || !name.ends_with("_log.md") || name.ends_with(".partial.md") {
                continue;
            }
            let rel = format!("{d}/{name}");
            let Some((series, num)) = logs::log_file_id(&name) else {
                p.warnings.push(Warning::fix(&rel, "log name is not S### or G###", "rename it (e.g. S012_log.md) or move it out of 2_Log/".into()));
                continue;
            };
            let Ok(raw) = fs::read_to_string(&path) else { continue };
            let crlf = raw.contains("\r\n");
            let text = lf(&raw);
            let (meta, text) = if d == "2_Log/Deprecated" { split_meta(&text) } else { (BTreeMap::new(), text) };
            v.push(Source { old_rel: rel, series, num, text, crlf, meta });
        }
    }
    for l in logs::scan_all(root) {
        let Ok(raw) = fs::read_to_string(&l.path) else { continue };
        let text = lf(&raw);
        if header::parse(&text).is_err() && legacy_header(&text).is_some() {
            v.push(Source { old_rel: logs::rel_of(&l), series: l.series, num: l.num, crlf: raw.contains("\r\n"), text, meta: BTreeMap::new() });
        }
    }
    v.sort_by(|a, b| (a.series, a.num, &a.old_rel).cmp(&(b.series, b.num, &b.old_rel)));
    v
}

/// 이관 계획. `decisions` = 결정 파일의 status 값(로그 경로 → 값), `today` = 대체 날짜.
pub fn plan(root: &Path, decisions: &BTreeMap<String, String>, today: &str) -> io::Result<Plan> {
    let mut p = Plan::default();

    // Registry TSV(2.4~2.27 = `2_Log/Wiki/`, 그 전 = 옛 로그 폴더의 `1_Wiki/`, `2_Wiki/` — 먼저 나온 행 우선)
    let mut rows: BTreeMap<String, session::RegistryRow> = BTreeMap::new();
    let tsvs: Vec<&str> = logs::OLD_REGISTRY_RELS.into_iter().filter(|r| root.join(r).is_file()).collect();
    for rel in &tsvs {
        let Ok(t) = fs::read_to_string(root.join(rel)) else { continue };
        match session::parse_registry(&t) {
            Ok(v) => {
                for r in v {
                    rows.entry(r.session.clone()).or_insert(r);
                }
            }
            Err(e) => p.warnings.push(Warning::fix(
                rel,
                "Registry row cannot be read",
                format!("line {}: expected {}, found {} — fix the row", e.line, e.expected, e.found),
            )),
        }
    }
    let registry_rel = tsvs.first().copied().unwrap_or(session::REGISTRY_REL);

    let sources = collect_sources(root, &mut p);

    // 같은 번호가 두 곳에
    let mut by_id: BTreeMap<(Series, u32), Vec<String>> = BTreeMap::new();
    for s in &sources {
        by_id.entry((s.series, s.num)).or_default().push(s.old_rel.clone());
    }
    let dup: BTreeSet<(Series, u32)> = by_id.iter().filter(|(_, v)| v.len() > 1).map(|(k, _)| *k).collect();
    for k in &dup {
        let v = &by_id[k];
        p.warnings.push(Warning::fix(&v[0], "same session number in two places", format!("{} — keep one (move or delete the other), then rerun", v.join(", "))));
    }

    // 번호 배정: 새 구조에 같은 번호의 다른 로그가 있으면(이관 뒤 구버전 CLI가 만든 로그) 다음 빈 번호
    let mut taken: BTreeMap<Series, BTreeSet<u32>> = BTreeMap::new();
    let mut existing: BTreeMap<String, String> = BTreeMap::new(); // 새 경로 → 내용(YAML 로그만)
    for l in logs::scan_all(root) {
        let Ok(t) = fs::read_to_string(&l.path) else { continue };
        if header::parse(&t).is_ok() {
            taken.entry(l.series).or_default().insert(l.num);
            existing.insert(logs::rel_of(&l), lf(&t));
        }
    }
    for s in &sources {
        taken.entry(s.series).or_default().insert(s.num);
    }
    let mut moves: BTreeMap<String, String> = BTreeMap::new();
    let mut targets: Vec<(u32, bool)> = Vec::new(); // (번호, 이어하기 후보)
    for s in &sources {
        let rel = logs::log_rel(s.series, s.num);
        let mut num = s.num;
        let mut resume = false;
        if rel != s.old_rel
            && let Some(cur) = existing.get(&rel)
        {
            let title = legacy_header(&s.text).map(|l| l.title).unwrap_or_default();
            if cur.lines().any(|l| l == title) {
                resume = true;
            } else {
                num = taken[&s.series].iter().max().copied().unwrap_or(0) + 1;
                taken.entry(s.series).or_default().insert(num);
            }
        }
        moves.insert(s.old_rel.clone(), logs::log_rel(s.series, num));
        targets.push((num, resume));
    }

    // 폐기 기록: 부분 파일, 계획 문서 전체 폐기
    let mut blocks: BTreeMap<String, Block> = BTreeMap::new();
    let mut containers: Vec<(String, Option<String>)> = Vec::new();
    let mut plan_wholes: Vec<(String, String, BTreeMap<String, String>, String)> = Vec::new(); // (옛 경로, 새 경로, 메타, 본문)
    for d in ["2_Log/Deprecated", "1_Concept/12_Planning/Deprecated"] {
        for path in read_dir_sorted(&root.join(d)) {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let rel = format!("{d}/{name}");
            if name.ends_with(".partial.md") {
                let text = lf(&fs::read_to_string(&path)?);
                let (src, b) = parse_container(&rel, &text);
                blocks.extend(b);
                containers.push((rel, src));
            } else if d.starts_with(PLAN_BASE) && name.ends_with(".md") {
                let text = lf(&fs::read_to_string(&path)?);
                let (meta, body) = split_meta(&text);
                let target = meta.get("source").cloned().unwrap_or_else(|| format!("{PLAN_BASE}/{name}"));
                if root.join(&target).exists() {
                    p.warnings.push(Warning::fix(&rel, "a file with the same name exists where it would go back", format!("{target} — keep one, then rerun")));
                    continue;
                }
                moves.insert(rel.clone(), target.clone());
                plan_wholes.push((rel, target, meta, body));
            }
        }
    }
    let mut stores = Stores { root, used: BTreeSet::new(), out: Vec::new() };
    for id in blocks.keys() {
        stores.used.insert(id.clone());
    }
    for s in &sources {
        for l in s.text.lines() {
            if let Some((id, ..)) = parse_origin(l) {
                stores.used.insert(id);
            }
        }
    }

    let lang = LogLang::of(root);
    let norm = norm_comment(lang);
    let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut moved = Vec::new();
    let mut renumbered = Vec::new();
    let mut norm_refreshed = 0;
    let mut deprecations = 0;
    let mut links_changed = 0;
    let mut spans_changed = 0;
    let mut status_words: Vec<String> = Vec::new();

    for (s, (num, resume)) in sources.iter().zip(&targets) {
        if dup.contains(&(s.series, s.num)) {
            continue;
        }
        let new_rel = logs::log_rel(s.series, *num);
        let new_id = logs::id_of(s.series, *num);
        let Some(lg) = legacy_header(&s.text) else {
            p.warnings.push(Warning::fix(&s.old_rel, "no header found", "expected `# S###: title` then `> **Status**: …` lines — fix the header by hand, then rerun".into()));
            p.headers.push(HeaderRow { old: s.old_rel.clone(), new: new_rel.clone(), note: "warning: no header found".into(), ..Default::default() });
            continue;
        };
        let get = |labels: &[&str]| -> String {
            labels.iter().find_map(|k| lg.fields.iter().find(|(f, _)| f == k).map(|(_, v)| clean_value(v))).filter(|v| !v.is_empty()).unwrap_or_default()
        };
        let row = rows.get(&logs::id_of(s.series, s.num)).filter(|_| s.series == Series::S);

        // 헤더 값(상태와 무관 — 경고가 난 로그도 미리보기 표에 남김)
        let created = [get(&["Created"]), get(&["Date"]), row.map(|r| r.date.clone()).unwrap_or_default()]
            .iter()
            .find_map(|v| date_of(v))
            .unwrap_or_else(|| today.to_string());
        let modified = date_of(&get(&["Modified"])).or_else(|| git_last_date(root, &s.old_rel)).unwrap_or_else(|| created.clone());
        const GOAL_KEYS: [&str; 4] = ["목표", "Goal", "목적", "Objective"];
        let goal = get(&GOAL_KEYS);
        let goal_source = match GOAL_KEYS.iter().find(|k| !get(&[**k]).is_empty()) {
            Some(k) => format!("header {k}"),
            None => body_goal_line(&lg.body).map(|l| format!("none (body: {l})")).unwrap_or_else(|| "none".into()),
        };
        let handoff = get(&["Handoff"]);
        let mut conclusion = get(&["결론"]);
        let mut conclusion_source = if conclusion.is_empty() { String::new() } else { "header 결론".to_string() };
        if conclusion.is_empty() {
            conclusion = row.map(|r| clean_value(&r.key_finding)).unwrap_or_default();
            if !conclusion.is_empty() {
                conclusion_source = "Registry Key Finding".into();
            }
        }
        let mut hrow = HeaderRow {
            old: s.old_rel.clone(),
            new: new_rel.clone(),
            created: created.clone(),
            modified: modified.clone(),
            goal: goal.clone(),
            goal_source,
            handoff: handoff.clone(),
            conclusion: conclusion.clone(),
            conclusion_source,
            ..Default::default()
        };
        let mut notes: Vec<String> = Vec::new();
        if let Some(w) = status_word_prefix(&conclusion) {
            notes.push(format!("conclusion starts with a status word ({w})"));
            status_words.push(format!("{} ({w})", logs::id_of(s.series, s.num)));
        }

        let hs = lg.fields.iter().find(|(f, _)| f == "Status").map(|(_, v)| v.as_str());
        let rs = row.map(|r| r.status.as_str());
        let (status, purged) = match decide(&sig(hs), &sig(rs)) {
            Ok(x) => x,
            Err(kind) => match decisions.get(&s.old_rel).map(|v| v.trim()) {
                Some(v) if Status::parse(v).is_some() => (Status::parse(v).unwrap(), false),
                other => {
                    let concl = !get(&["결론"]).is_empty() || row.is_some_and(|r| !clean_value(&r.key_finding).is_empty());
                    let kind: String = if other.is_some() { "decision file status is not ongoing, complete or deprecated".into() } else { kind.into() };
                    notes.insert(0, format!("warning: {kind} — fill status"));
                    p.warnings.push(Warning {
                        log: s.old_rel.clone(),
                        kind,
                        header: sig_text(hs),
                        registry: sig_text(rs),
                        evidence: Some(evidence(&lg.body, concl)),
                        detail: None,
                        fill: true,
                    });
                    hrow.note = notes.join("; ");
                    p.headers.push(hrow);
                    continue;
                }
            },
        };
        // 7키에 없는 필드와 진행 중 로그의 결론 = 본문 한 줄로 보존(사용자 결정 2026-11-10 — 링크와 값 손실 없음)
        let known = ["Created", "Date", "Modified", "Status", "목표", "Goal", "목적", "Objective", "Handoff", "결론"];
        let mut kept: Vec<String> = Vec::new();
        for (f, v) in &lg.fields {
            let v = clean_value(v);
            if !known.contains(&f.as_str()) && !v.is_empty() {
                kept.push(format!("{f} = {v}"));
                dropped.entry(f.clone()).or_default().push(logs::id_of(s.series, s.num));
            }
        }
        if status == Status::Ongoing && !conclusion.is_empty() {
            kept.push(format!("{} = {conclusion}", if lang == LogLang::Ko { "결론" } else { "conclusion" }));
            dropped.entry("결론(status ongoing)".into()).or_default().push(logs::id_of(s.series, s.num));
        }
        let pre = if kept.is_empty() { lg.body.clone() } else { format!("{}{}\n\n{}", kept_prefix(lang), kept.join("; "), lg.body) };
        hrow.kept = kept.join("; ");

        // 본문: 흔적 변환 → 링크 → 규범 주석
        let (body, nd) = convert_origins(root, &pre, &s.old_rel, &new_rel, &mut blocks, &mut stores, &moves, &mut p.warnings);
        deprecations += nd;
        let rw = rewrite_links(root, &body, dirname(&s.old_rel), dirname(&new_rel), &moves);
        let body = rw.text;
        let body_lines = body.lines().count() as isize;
        let raw_changes = rw.changes;
        links_changed += rw.links;
        spans_changed += rw.spans;
        for b in rw.broken {
            p.warnings.push(Warning::fix(&s.old_rel, "link target does not exist", format!("{b} — fix or remove the link (it was broken before the migration)")));
        }
        let (body, refreshed) = refresh_norm(&body, &norm);
        norm_refreshed += refreshed as usize;
        let mut title = lg.title.clone();
        if *num != s.num {
            let old_id = logs::id_of(s.series, s.num);
            title = title.replacen(&format!("# {old_id}"), &format!("# {new_id}"), 1);
            renumbered.push(format!("{old_id} → {new_id} ({})", s.old_rel));
        }

        let mut h = Header { created, modified, status, goal, handoff, conclusion, user_note: String::new() };
        if h.status == Status::Ongoing {
            h.conclusion.clear(); // 값은 위 보존 줄에
        }
        let stored_body = format!("{title}\n\n{}", body.trim_start_matches('\n'));
        let text = if status == Status::Deprecated {
            let id = stores.alloc(&new_id);
            let date = s.meta.get("deprecated").and_then(|d| date_of(d)).unwrap_or_else(|| h.modified.clone());
            let reason = s.meta.get("reason").cloned().unwrap_or_default();
            let rb = s.meta.get("replaced_by").cloned().unwrap_or_default();
            let before = s.meta.get("status_before").map(|v| sig(Some(v)));
            let orig_status = match before {
                Some(Sig::Canon(st, _)) if st != Status::Deprecated => st,
                _ => Status::Complete,
            };
            let w = deprecate::words(lang);
            let mut mark = deprecate::marker_line(&id, &date, w.whole_session, Some(&reason), Some(&rb), lang);
            if purged {
                mark.push_str(&purged_tail(lang, if lang == LogLang::Ko { "이관 전 purged" } else { "purged before migration" }));
            } else {
                let meta: Vec<(&str, String)> = vec![
                    ("id", id.clone()),
                    ("source", new_rel.clone()),
                    ("unit", "whole".into()),
                    ("date", date.clone()),
                    ("reason", reason.clone()),
                    ("replaced_by", rb.clone()),
                    ("status", orig_status.as_str().into()),
                    ("handoff", h.handoff.clone()),
                    ("conclusion", if orig_status == Status::Complete { h.conclusion.clone() } else { String::new() }),
                    ("user_note", String::new()),
                ];
                stores.out.push((id.clone(), deprecate::store_text(&meta, &stored_body)));
            }
            let mut ho = format!("{} {id}", w.deprecated);
            if !reason.is_empty() {
                ho.push_str(&format!(": {}", header::one_line(&reason)));
            }
            if !rb.is_empty() {
                ho.push_str(&format!(", {} {}", w.replaced_eq, header::one_line(&rb)));
            }
            h.handoff = ho;
            h.conclusion.clear();
            deprecations += 1;
            format!("{}{title}\n\n{mark}\n", header::render(&h))
        } else {
            format!("{}{stored_body}", header::render(&h))
        };
        let text = if text.ends_with('\n') { text } else { format!("{text}\n") };
        hrow.status = h.status.as_str().into();
        hrow.handoff = h.handoff.clone();
        hrow.conclusion = h.conclusion.clone();

        if *resume {
            if existing.get(&new_rel) == Some(&text) {
                p.deletes.push(s.old_rel.clone());
                p.lines.push(format!("{} already migrated to {new_rel} — removing the old file", s.old_rel));
                notes.push("already migrated — the old file is removed".into());
            } else {
                p.warnings.push(Warning::fix(&s.old_rel, "the new location already has this session", format!("{new_rel} has the same title but different content — keep one, then rerun")));
                notes.insert(0, "warning: the new location already has this session".into());
            }
            hrow.note = notes.join("; ");
            p.headers.push(hrow);
            continue;
        }
        hrow.note = notes.join("; ");
        p.headers.push(hrow);
        if status != Status::Deprecated {
            let delta = text.lines().count() as isize - body_lines;
            locate(&new_rel, &text, raw_changes, delta, &mut p.changes);
        }
        let bytes = if s.crlf { text.replace('\n', "\r\n") } else { text };
        p.writes.push((new_rel.clone(), bytes.into_bytes()));
        if s.old_rel != new_rel {
            p.deletes.push(s.old_rel.clone());
            moved.push(s.old_rel.clone());
        }
    }

    // Registry 행만 있는 세션: purged = 흔적 로그, 그 밖 = 경고
    let present: BTreeSet<String> = sources.iter().map(|s| logs::id_of(s.series, s.num)).chain(logs::scan(root, Series::S).into_iter().map(|l| l.id)).collect();
    for (id, r) in &rows {
        if present.contains(id) {
            continue;
        }
        match sig(Some(&r.status)) {
            Sig::Canon(Status::Deprecated, true) => {
                let Some((series, n)) = logs::parse_id(id) else { continue };
                let sid = stores.alloc(id);
                let date = date_of(&r.date).unwrap_or_else(|| today.to_string());
                let h = Header {
                    created: date.clone(),
                    modified: date.clone(),
                    status: Status::Deprecated,
                    goal: String::new(),
                    handoff: match lang {
                        LogLang::Ko => format!("폐기 {sid}: 이관 전 purged, 원문 없음"),
                        LogLang::En => format!("deprecated {sid}: purged before migration, no original"),
                    },
                    conclusion: String::new(),
                    user_note: String::new(),
                };
                let mark = format!(
                    "{}{}",
                    deprecate::marker_line(&sid, &date, deprecate::words(lang).whole_session, None, None, lang),
                    purged_tail(lang, if lang == LogLang::Ko { "이관 전 purged" } else { "purged before migration" })
                );
                let text = format!("{}# {id}: {}\n\n{mark}\n", header::render(&h), r.title.trim());
                p.writes.push((logs::log_rel(series, n), text.into_bytes()));
                p.lines.push(format!("{id}: purged before the migration — wrote a header-only log (number kept, no store)"));
            }
            _ => p.warnings.push(Warning::fix(
                registry_rel,
                "Registry row without a log file",
                format!("{id} ({}) — restore the log from git, or delete the row", r.status),
            )),
        }
    }

    // 계획 문서: 전체 폐기 → 제목과 표기 줄 + 보관, 흔적 줄 변환, 들어오는 링크
    for (old, target, meta, body) in &plan_wholes {
        let doc = deprecate::doc_id(target);
        let id = stores.alloc(&doc);
        let date = meta.get("deprecated").and_then(|d| date_of(d)).unwrap_or_else(|| today.to_string());
        let reason = meta.get("reason").cloned().unwrap_or_default();
        let rb = meta.get("replaced_by").cloned().unwrap_or_default();
        let content = rewrite_links(root, body, dirname(old), dirname(target), &moves).text;
        let m: Vec<(&str, String)> = vec![
            ("id", id.clone()),
            ("source", target.clone()),
            ("unit", "whole".into()),
            ("date", date.clone()),
            ("reason", reason.clone()),
            ("replaced_by", rb.clone()),
        ];
        stores.out.push((id.clone(), deprecate::store_text(&m, &content)));
        let title = content.lines().find(|l| l.starts_with("# ")).unwrap_or("").to_string();
        let mark = deprecate::marker_line(&id, &date, deprecate::words(lang).whole_doc, Some(&reason), Some(&rb), lang);
        p.writes.push((target.clone(), format!("{title}\n\n{mark}\n").into_bytes()));
        p.deletes.push(old.clone());
        deprecations += 1;
    }
    let source_set: BTreeSet<&str> = sources.iter().map(|s| s.old_rel.as_str()).collect();
    let skip: BTreeSet<String> = containers
        .iter()
        .map(|(c, _)| c.clone())
        .chain(plan_wholes.iter().map(|w| w.0.clone()))
        .chain(p.writes.iter().map(|w| w.0.clone()))
        .collect();
    // 다른 문서: git 추적 파일만(추적 밖 사본, 임시 폴더는 건드리지 않음)
    let tracked = tracked(root);
    let mut touched = 0;
    for path in link_docs(root, tracked.as_deref()) {
        let rel = rel_of(root, &path);
        if source_set.contains(rel.as_str()) || skip.contains(&rel) {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else { continue };
        let text = lf(&raw);
        let (t1, nd) = if rel.starts_with(PLAN_BASE) {
            convert_origins(root, &text, &rel, &rel, &mut blocks, &mut stores, &moves, &mut p.warnings)
        } else {
            (text.clone(), 0)
        };
        deprecations += nd;
        let rw = rewrite_links(root, &t1, dirname(&rel), dirname(&rel), &moves);
        let t2 = rw.text;
        if t2 != text {
            links_changed += rw.links;
            spans_changed += rw.spans;
            touched += 1;
            locate(&rel, &t2, rw.changes, 0, &mut p.changes);
            let out = if raw.contains("\r\n") { t2.replace('\n', "\r\n") } else { t2 };
            p.writes.push((rel, out.into_bytes()));
        }
    }

    // 흔적 줄 없는 블록 — 잃지 않게 보관(이관 뒤 `elf check`가 표기 줄 없는 보관 파일로 경고)
    for (id, b) in std::mem::take(&mut blocks) {
        let src_old = containers.iter().find(|(c, _)| *c == b.container).and_then(|(_, s)| s.clone()).unwrap_or_default();
        let src = moves.get(&src_old).cloned().unwrap_or(src_old);
        let content = rewrite_links(root, &b.content, dirname(&b.container), dirname(&src), &moves).text;
        let meta: Vec<(&str, String)> = vec![
            ("id", id.clone()),
            ("source", src.clone()),
            ("unit", b.unit.clone()),
            ("date", b.date.clone()),
            ("reason", b.reason.clone()),
            ("replaced_by", b.replaced_by.clone()),
        ];
        stores.out.push((id.clone(), deprecate::store_text(&meta, &content)));
        p.lines.push(format!("{id}: block in {} had no marker line in {src} — stored anyway; `elf check` will list it", b.container));
    }
    for (id, text) in &stores.out {
        p.writes.push((deprecate::store_rel(id), deprecate::compress_text(text)?));
    }
    for (c, _) in &containers {
        p.deletes.push(c.clone());
    }
    for rel in &tsvs {
        p.deletes.push(rel.to_string());
    }
    for d in logs::OLD_EMPTY_DIRS.into_iter().chain(["1_Concept/12_Planning/Deprecated"]) {
        if root.join(d).is_dir() {
            p.dirs.push(d.into());
        }
    }

    // 보고
    if !moved.is_empty() {
        p.lines.push(format!("logs: {} moved to 2_Log/S/ (or 2_Log/G/), header → YAML 7 keys", moved.len()));
    }
    for r in &renumbered {
        p.lines.push(format!("renumbered {r} — a log made by an older CLI after the migration"));
    }
    for (f, ids) in &dropped {
        p.lines.push(format!("header field kept as a body line (`{}`): {f} — {} log(s): {}", KEPT_PREFIX.trim(), ids.len(), ids.join(", ")));
    }
    if deprecations > 0 {
        p.lines.push(format!("deprecation records: {deprecations} converted (store: .elf/deprecated/)"));
    }
    if links_changed > 0 {
        p.lines.push(format!("links: {links_changed} recomputed ({touched} other file(s) changed)"));
    }
    if spans_changed > 0 {
        p.lines.push(format!("code-span paths: {spans_changed} recomputed (a single relative path that resolves to a file) — review: history notes may have meant the old path"));
        let spans: Vec<&Change> = p.changes.iter().filter(|c| c.kind == "span").collect();
        for c in spans.iter().take(SHOW) {
            p.lines.push(format!("code span changed: {}:{} `{}` → `{}`", c.file, c.line, c.before, c.after));
        }
        if spans.len() > SHOW {
            p.lines.push(format!("code span changed: … {} more (full list: `elf update --dry-run --out <folder>`, changes.tsv)", spans.len() - SHOW));
        }
    }
    if norm_refreshed > 0 {
        p.lines.push(format!("ELF norm comment refreshed in {norm_refreshed} log(s)"));
    }
    for rel in &tsvs {
        p.lines.push(format!("{rel} → removed (Key Finding moved to conclusion)"));
    }
    if !status_words.is_empty() {
        p.lines.push(format!(
            "conclusion starts with a status word — review (the Registry Key Finding column often held the status too): {} log(s): {}",
            status_words.len(),
            status_words.join(", ")
        ));
    }
    p.mentions = path_mentions(root, &p, &moves, tracked.as_deref());
    if !p.mentions.is_empty() {
        let mut by_file: Vec<(String, usize)> = Vec::new();
        for m in &p.mentions {
            match by_file.iter_mut().find(|(f, _)| *f == m.file) {
                Some(e) => e.1 += 1,
                None => by_file.push((m.file.clone(), 1)),
            }
        }
        // 스크립트 등 `.md` 아닌 파일 먼저(이관이 고치지 않고 실행이 깨질 수 있음), 이어서 언급 많은 순
        by_file.sort_by(|a, b| a.0.ends_with(".md").cmp(&b.0.ends_with(".md")).then(b.1.cmp(&a.1)).then(a.0.cmp(&b.0)));
        let shown: Vec<String> = by_file.iter().take(8).map(|(f, n)| format!("{f} ({n} line{})", if *n == 1 { "" } else { "s" })).collect();
        let more = if by_file.len() > 8 {
            format!("; … {} more file(s) (full list: `elf update --dry-run --out <folder>`, mentions.tsv)", by_file.len() - 8)
        } else {
            String::new()
        };
        p.lines.push(format!(
            "mentions (not links) of moved or removed files — left as written; review scripts and rules: {} file(s): {}{more}",
            by_file.len(),
            shown.join(", ")
        ));
    }
    Ok(p)
}

/// 출력에 보이는 변경 목록 수(나머지는 미리보기 파일)
const SHOW: usize = 20;

/// 링크가 아닌 옛 경로(`Archive/S012_log.md` 등)와 지운 파일 이름(`Session_Registry.tsv` 등) 언급 줄.
/// 대상 = 이관이 쓰는 `.md`(새 내용)와 git 추적 텍스트 파일 전체(스크립트, `.claude/` 포함, 모든 깊이의 `.elf/` 제외 — 관리 파일).
/// git이 아니면 폴더의 `.md`만.
fn path_mentions(root: &Path, p: &Plan, moves: &BTreeMap<String, String>, tracked: Option<&[String]>) -> Vec<Mention> {
    let mut needles: Vec<String> = moves
        .keys()
        .flat_map(|k| {
            let b = basename(k).to_string();
            let d = basename(dirname(k)).to_string();
            vec![k.clone(), format!("{d}/{b}")]
        })
        .filter(|n| !n.starts_with("2_Log/S/") && !n.starts_with("S/"))
        .collect();
    // 옮기지 않고 지우는 파일(Registry TSV, 부분 폐기 파일) = 이름만으로도 깨진 참조
    for d in &p.deletes {
        if !moves.contains_key(d) {
            needles.push(basename(d).to_string());
        }
    }
    needles.sort();
    needles.dedup();
    let written: BTreeMap<&str, &Vec<u8>> = p.writes.iter().map(|(r, b)| (r.as_str(), b)).collect();
    let mut out = Vec::new();
    let mut scan = |rel: &str, text: &str| {
        for (i, l) in text.lines().enumerate() {
            if let Some(n) = needles.iter().find(|x| l.contains(x.as_str())) {
                let t = l.trim();
                let t = if t.chars().count() > 200 { format!("{}…", t.chars().take(200).collect::<String>()) } else { t.to_string() };
                out.push(Mention { file: rel.to_string(), line: i + 1, name: n.clone(), text: t });
            }
        }
    };
    for (rel, b) in &written {
        if rel.ends_with(".md") && !rel.starts_with(".elf/") {
            scan(rel, &String::from_utf8_lossy(b));
        }
    }
    let files: Vec<String> = match tracked {
        Some(t) => t.iter().filter(|r| !r.split('/').any(|c| c == ".elf" || c == ".git")).cloned().collect(),
        None => {
            let mut d = Vec::new();
            walk_md(root, &mut d);
            d.iter().map(|x| rel_of(root, x)).collect()
        }
    };
    for rel in files {
        // ELF가 넣은 이전 배치 파일(구 규칙, 틀, gallery 스크립트)은 같은 갱신이 옮기거나 바꾸거나 지움 — 언급 보고 제외
        if p.deletes.contains(&rel) || written.contains_key(rel.as_str()) || crate::earlier::delivered_dest(&rel) {
            continue;
        }
        let path = root.join(&rel);
        if fs::metadata(&path).map(|m| m.len() > 2_000_000).unwrap_or(true) {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else { continue };
        if bytes.iter().take(8000).any(|b| *b == 0) {
            continue;
        }
        let Ok(t) = String::from_utf8(bytes) else { continue };
        scan(&rel, &t);
    }
    out
}

// ── 미리보기(`elf update --dry-run --out <폴더>`) ─────────────────

/// 미리보기 폴더 표지 — 이 파일이 있는 폴더만 비우고 다시 씀.
pub const PREVIEW_MARK: &str = "ELF_MIGRATION_PREVIEW.txt";

/// 미리보기 폴더 준비. 프로젝트 안은 `.elf/runtime/` 아래만(작업 트리를 더럽히지 않게), 비어 있지 않으면 표지가 있을 때만
/// 비움. 반환 = 절대 경로. Err = 사용자에게 보일 이유.
pub fn prepare_preview(root: &Path, out: &Path) -> Result<PathBuf, String> {
    let abs = std::path::absolute(out).map_err(|e| format!("--out {}: {e}", out.display()))?;
    let mut anc = abs.clone();
    let mut rest = Vec::new();
    while !anc.exists() {
        match (anc.file_name().map(|n| n.to_owned()), anc.parent().map(Path::to_path_buf)) {
            (Some(n), Some(pa)) => {
                rest.push(n);
                anc = pa;
            }
            _ => break,
        }
    }
    let mut full = fs::canonicalize(&anc).map_err(|e| format!("--out {}: {e}", out.display()))?;
    for n in rest.iter().rev() {
        full.push(n);
    }
    if let Ok(root_c) = fs::canonicalize(root)
        && let Ok(r) = full.strip_prefix(&root_c)
    {
        let r = r.to_string_lossy().replace('\\', "/");
        if !r.starts_with(".elf/runtime/") || r.trim_end_matches('/') == DECISION_REL {
            return Err(format!(
                "--out {} is inside the project — use a folder outside it, or under .elf/runtime/ (git-ignored), so the working tree stays clean for the migration",
                out.display()
            ));
        }
    }
    if full.exists() {
        if !full.is_dir() {
            return Err(format!("--out {} is a file — give a folder", out.display()));
        }
        let empty = fs::read_dir(&full).map(|mut d| d.next().is_none()).unwrap_or(false);
        if !empty {
            if !full.join(PREVIEW_MARK).is_file() {
                return Err(format!(
                    "--out {} is not empty and is not an earlier preview (no {PREVIEW_MARK}) — give a new or empty folder",
                    out.display()
                ));
            }
            fs::remove_dir_all(&full).map_err(|e| format!("--out {}: {e}", out.display()))?;
        }
    }
    fs::create_dir_all(&full).map_err(|e| format!("--out {}: {e}", out.display()))?;
    Ok(full)
}

fn preview_line(dir: &Path) -> String {
    let d = dir.display().to_string();
    format!(
        "migration: preview written to {} — files/, headers.tsv, changes.tsv, mentions.tsv, warnings.tsv, report.txt (the project is untouched)",
        d.strip_prefix(r"\\?\").unwrap_or(&d)
    )
}

fn tsv_cell(v: &str) -> String {
    v.replace(['\t', '\n', '\r'], " ")
}

fn write_tsv(path: &Path, head: &[&str], rows: impl Iterator<Item = Vec<String>>) -> io::Result<()> {
    let mut s = head.join("\t");
    s.push('\n');
    for r in rows {
        s.push_str(&r.iter().map(|c| tsv_cell(c)).collect::<Vec<_>>().join("\t"));
        s.push('\n');
    }
    fs::write(path, s)
}

/// 계획을 미리보기 폴더에 씀: 결과 파일, 지울 목록, 헤더 표, 변경, 언급, 경고, 출력.
fn write_preview(dir: &Path, p: &Plan, report: &[String]) -> io::Result<()> {
    fs::write(
        dir.join(PREVIEW_MARK),
        "elf update --dry-run --out — migration preview (nothing is written to the project, LogConvention §7)\n\
         files/        the content of the files the migration would write (paths from the root; deprecation stores as unpacked .md)\n\
         deletes.txt   files to delete, and folders to delete when they become empty\n\
         headers.tsv   per log: the 7 YAML key values after the migration, the source of goal and conclusion, kept lines, points to check\n\
         changes.tsv   recomputed links and code spans (line in the migrated file, before, after)\n\
         mentions.tsv  old paths that are not links, mentions of deleted file names (scripts included; the migration does not change them)\n\
         warnings.tsv  warnings that block applying (decision file items)\n\
         report.txt    the whole output\n\
         compare with the original: git diff --no-index <original file> files/<same path>\n",
    )?;
    let files = dir.join("files");
    for (rel, bytes) in &p.writes {
        let (rel, data) = match rel.strip_suffix(".xz").filter(|_| rel.starts_with(".elf/deprecated/")) {
            Some(r) => (r.to_string(), deprecate::decompress(bytes)?.into_bytes()),
            None => (rel.clone(), bytes.clone()),
        };
        let path = files.join(&rel);
        if let Some(d) = path.parent() {
            fs::create_dir_all(d)?;
        }
        fs::write(path, data)?;
    }
    let mut del: Vec<String> = p.deletes.clone();
    del.extend(p.dirs.iter().map(|d| format!("{d}/ (folder, removed if empty)")));
    fs::write(dir.join("deletes.txt"), del.join("\n") + "\n")?;
    write_tsv(
        &dir.join("headers.tsv"),
        &["old", "new", "status", "created", "modified", "goal", "goal_source", "handoff", "conclusion", "conclusion_source", "kept_line", "note"],
        p.headers.iter().map(|h| {
            vec![
                h.old.clone(),
                h.new.clone(),
                h.status.clone(),
                h.created.clone(),
                h.modified.clone(),
                h.goal.clone(),
                h.goal_source.clone(),
                h.handoff.clone(),
                h.conclusion.clone(),
                h.conclusion_source.clone(),
                h.kept.clone(),
                h.note.clone(),
            ]
        }),
    )?;
    write_tsv(
        &dir.join("changes.tsv"),
        &["file", "line", "kind", "before", "after"],
        p.changes.iter().map(|c| vec![c.file.clone(), c.line.to_string(), c.kind.to_string(), c.before.clone(), c.after.clone()]),
    )?;
    write_tsv(
        &dir.join("mentions.tsv"),
        &["file", "line", "name", "text"],
        p.mentions.iter().map(|m| vec![m.file.clone(), m.line.to_string(), m.name.clone(), m.text.clone()]),
    )?;
    write_tsv(
        &dir.join("warnings.tsv"),
        &["log", "warning", "header", "registry", "evidence", "detail", "fill_status"],
        p.warnings.iter().map(|w| {
            vec![
                w.log.clone(),
                w.kind.clone(),
                w.header.clone().unwrap_or_default(),
                w.registry.clone().unwrap_or_default(),
                w.evidence.clone().unwrap_or_default(),
                w.detail.clone().unwrap_or_default(),
                if w.fill { "yes".into() } else { String::new() },
            ]
        }),
    )?;
    fs::write(dir.join("report.txt"), report.join("\n") + "\n")
}

/// 계획 적용: 새 파일 먼저 쓰고 옛 파일 삭제(중단되면 재실행으로 이어감), 빈 폴더 삭제, Registry.md와 nesting 생성.
pub fn apply(root: &Path, p: &Plan) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    for (rel, bytes) in &p.writes {
        let path = root.join(rel);
        if let Some(d) = path.parent() {
            fs::create_dir_all(d)?;
        }
        fs::write(path, bytes)?;
    }
    for rel in &p.deletes {
        let path = root.join(rel);
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    for d in &p.dirs {
        let path = root.join(d);
        // 남은 것이 `.gitkeep`뿐이면 함께 지움(빈 폴더 표시용 파일)
        let only_keep = fs::read_dir(&path).is_ok_and(|rd| rd.flatten().all(|e| e.file_name() == ".gitkeep"));
        if only_keep {
            let _ = fs::remove_file(path.join(".gitkeep"));
        }
        if path.is_dir() && fs::remove_dir(&path).is_err() {
            lines.push(format!("{d}/ kept — it still holds files that are not logs; review and move them"));
        }
    }
    lines.extend(logs::refresh(root)?.into_iter().map(|w| format!("warn: {w}")));
    let _ = fs::remove_file(root.join(DECISION_REL));
    // 로그만 있는 폴더(`.elf/` 없음)에 결정 파일 때문에 생긴 빈 폴더 정리
    let _ = fs::remove_dir(root.join(".elf/runtime"));
    if !root.join(".elf/manifest.json").exists() {
        let _ = fs::remove_dir(root.join(".elf"));
    }
    Ok(lines)
}

/// 이관 실행 결과.
#[derive(Debug)]
pub enum Outcome {
    /// 적용함 — 보고 줄
    Applied(Vec<String>),
    /// dry-run — 계획과 경고(쓰지 않음)
    Planned(Vec<String>),
    /// 경고 또는 전제 미충족 — 아무것도 쓰지 않음(결정 파일만), 보고 줄
    Blocked(Vec<String>),
}

/// `elf update`의 이관 단계. `today` = YYYY-MM-DD.
pub fn run(root: &Path, dry_run: bool, today: &str) -> io::Result<Outcome> {
    run_with(root, dry_run, today, None)
}

/// `run` + 미리보기 폴더(`prepare_preview`로 준비한 경로, dry-run에서만 씀).
pub fn run_with(root: &Path, dry_run: bool, today: &str, preview: Option<&Path>) -> io::Result<Outcome> {
    run_core(root, dry_run, today, preview, true)
}

/// 동의 전 점검(v3.0 S044 t60): 전제와 경고만 처리 — 전제 미충족이나 경고면 `run`과 같이 Blocked(경고면 결정 파일을
/// 씀), 문제가 없으면 적용하지 않고 Planned.
pub fn check_ready(root: &Path, today: &str) -> io::Result<Outcome> {
    run_core(root, false, today, None, false)
}

fn run_core(root: &Path, dry_run: bool, today: &str, preview: Option<&Path>, apply_now: bool) -> io::Result<Outcome> {
    let preview = preview.filter(|_| dry_run);
    let mut out = Vec::new();
    let pre = git_precondition(root);
    if let Err(m) = &pre {
        if !dry_run {
            return Ok(Outcome::Blocked(vec![
                format!("warn: migration: {m}"),
                "migration: nothing written (the whole update waits for the migration — LogConvention §7)".into(),
            ]));
        }
        out.push(format!("warn: migration: {m}"));
    }
    let decisions = read_decisions(root);
    let p = plan(root, &decisions, today)?;
    if !p.warnings.is_empty() {
        for w in &p.warnings {
            out.push(format!("warn: migration: {}", w.line()));
        }
        if dry_run {
            out.push(format!("migration: {} warning(s) — `elf update` would write {DECISION_REL} and nothing else", p.warnings.len()));
            if let Some(dir) = preview {
                let mut rep = out.clone();
                rep.extend(p.lines.iter().map(|l| format!("migration (planned, waits for the warnings above): {l}")));
                write_preview(dir, &p, &rep)?;
                out.push(preview_line(dir));
            }
            return Ok(Outcome::Planned(out));
        }
        let path = root.join(DECISION_REL);
        if let Some(d) = path.parent() {
            fs::create_dir_all(d)?;
        }
        fs::write(&path, decision_text(&p.warnings, &decisions))?;
        out.push(format!(
            "migration: {} warning(s) — nothing written; fill the status fields and fix the listed files in {DECISION_REL}, then run `elf update` again (LogConvention §7)",
            p.warnings.len()
        ));
        return Ok(Outcome::Blocked(out));
    }
    out.extend(p.lines.iter().map(|l| format!("migration: {l}")));
    if dry_run {
        out.push(format!("migration: would write {} file(s) and remove {}", p.writes.len(), p.deletes.len()));
        if let Some(dir) = preview {
            write_preview(dir, &p, &out)?;
            out.push(preview_line(dir));
        }
        return Ok(Outcome::Planned(out));
    }
    if !apply_now {
        return Ok(Outcome::Planned(out));
    }
    out.extend(apply(root, &p)?.into_iter().map(|l| format!("migration: {l}")));
    out.push("migration: done — run `elf check`, review with the user, then commit (undo = git)".into());
    Ok(Outcome::Applied(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_comment_follows_project_language() {
        // P017: 영어 프로젝트는 영어 규범 주석, 두 표지 모두 갱신 대상
        let ko = norm_comment(LogLang::Ko);
        let en = norm_comment(LogLang::En);
        assert!(ko.starts_with("<!-- ELF 규범") && ko.ends_with("-->"), "{ko}");
        assert!(en.starts_with("<!-- ELF norms") && en.ends_with("-->"), "{en}");
        let (out, changed) = refresh_norm("<!-- ELF 규범: old -->\n\n## t01: a", &en);
        assert!(changed && out.starts_with(&en) && out.ends_with("## t01: a"), "{out}");
        let (out, changed) = refresh_norm(&format!("{en}\n\n## t01: a"), &en);
        assert!(!changed && out.starts_with(&en));
    }

    #[test]
    fn legacy_header_parses_fields_and_body() {
        let t = "# S012: 제목\n\n> **Created**: 2026-06-01\\\n> **Status**: Complete\\\n> **목표**: g\\\n> **관련**: S011\\\n> **Handoff**: -\n\n---\n\n<!-- ELF 규범: old -->\n\n## t01: a\n";
        let l = legacy_header(t).unwrap();
        assert_eq!(l.title, "# S012: 제목");
        assert_eq!(l.fields.len(), 5);
        assert_eq!(l.fields[1], ("Status".into(), "Complete".into()));
        assert!(l.body.starts_with("<!-- ELF 규범: old -->"));
        assert!(legacy_header("# x\n\nno header\n").is_none());
    }

    #[test]
    fn status_signals() {
        assert_eq!(decide(&sig(Some("★ 활성")), &sig(Some("★ 활성"))), Ok((Status::Ongoing, false)));
        assert_eq!(decide(&sig(Some("Complete")), &sig(None)), Ok((Status::Complete, false)));
        assert_eq!(decide(&sig(None), &sig(Some("Purged"))), Ok((Status::Deprecated, true)));
        assert!(decide(&sig(Some("★ 활성")), &sig(Some("Complete"))).unwrap_err().contains("disagree"));
        assert!(decide(&sig(Some("☑ 완료")), &sig(Some("Complete"))).unwrap_err().contains("canonical"));
        assert!(decide(&sig(Some("-")), &sig(None)).unwrap_err().contains("no status"));
    }

    #[test]
    fn relpath_and_link_rewrite() {
        assert_eq!(relpath("6_Exp/64_Viz/S001/a.png", "2_Log/S"), "../../6_Exp/64_Viz/S001/a.png");
        assert_eq!(relpath("2_Log/S/S002_log.md", "2_Log/S"), "S002_log.md");
        assert_eq!(relpath("2_Log/S/S002_log.md", "2_Log/Wiki"), "../S/S002_log.md");
        assert_eq!(join("2_Log", "../../elf-cli/x.md").as_deref(), Some("../elf-cli/x.md"));
        assert_eq!(relpath("../elf-cli/x.md", "2_Log/S"), "../../../elf-cli/x.md");
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("6_Exp")).unwrap();
        fs::write(tmp.path().join("6_Exp/a.png"), b"x").unwrap();
        let mut moves = BTreeMap::new();
        moves.insert("2_Log/Archive/S002_log.md".to_string(), "2_Log/S/S002_log.md".to_string());
        let text = "![f](../6_Exp/a.png) [s](Archive/S002_log.md#t01) `[c](x.md)` [u](https://x/y.md) [REL](main) [DOC](CHANGELOG, x.md)\n```\n[k](a.md)\n```\n[gone](nope.md \"t\")\n";
        let rw = rewrite_links(tmp.path(), text, "2_Log", "2_Log/S", &moves);
        let (out, n, broken) = (rw.text, rw.links, rw.broken);
        assert_eq!(rw.spans, 0, "`[c](x.md)` is not a path span");
        assert!(out.contains("![f](../../6_Exp/a.png)") && out.contains("[s](S002_log.md#t01)"), "{out}");
        assert!(out.contains("`[c](x.md)`") && out.contains("[k](a.md)") && out.contains("(https://x/y.md)"), "{out}");
        assert!(out.contains("[gone](nope.md \"t\")"), "missing target: left as written: {out}");
        assert!(out.contains("[REL](main) [DOC](CHANGELOG, x.md)"), "prose that looks like a link: {out}");
        assert_eq!(n, 2);
        assert_eq!(broken, vec!["line 5: nope.md".to_string()]);
    }

    #[test]
    fn old_markers_and_containers_parse() {
        let o = "> **Deprecated** → [Deprecated/S012_log.partial.md](../Deprecated/S012_log.partial.md) (S012-D01 · t03 · 2026-10-05 · replaced_by: S015 t02)";
        assert_eq!(parse_origin(o), Some(("S012-D01".into(), "t03".into(), "2026-10-05".into(), "S015 t02".into(), None)));
        let pg = "> **Deprecated** → purged (S012-D02 · t04 해석 · 2026-10-05 · purged: 2026-10-10)";
        assert_eq!(parse_origin(pg).unwrap().4.as_deref(), Some("2026-10-10"));
        let c = "---\ndeprecated: partial\nsource: 2_Log/Archive/S012_log.md\n---\n\n# S012: x\n> **Deprecated**: 2026-10-05 · S012-D01 · t03 · replaced_by: S015 t02 · reason: 조건 오류\n\n<!-- deprecated:begin S012-D01 -->\n## t03: 틀린 시도\n- [a](../Archive/S011_log.md)\n<!-- deprecated:end S012-D01 -->\n";
        let (src, b) = parse_container("2_Log/Deprecated/S012_log.partial.md", c);
        assert_eq!(src.as_deref(), Some("2_Log/Archive/S012_log.md"));
        let blk = &b["S012-D01"];
        assert_eq!((blk.unit.as_str(), blk.reason.as_str()), ("t03", "조건 오류"));
        assert!(blk.content.starts_with("## t03: 틀린 시도"));
    }

    #[test]
    fn decision_file_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let w = Warning {
            log: "2_Log/Archive/S030_log.md".into(),
            kind: "header Status and Registry status disagree".into(),
            header: Some("★ 활성".into()),
            registry: Some("Complete".into()),
            evidence: Some("last trial: t05 정리; conclusion line: yes; deprecation marker: no".into()),
            detail: None,
            fill: true,
        };
        let mut prior = BTreeMap::new();
        prior.insert("2_Log/Archive/S030_log.md".to_string(), "complete".to_string());
        fs::create_dir_all(tmp.path().join(".elf/runtime")).unwrap();
        fs::write(tmp.path().join(DECISION_REL), decision_text(std::slice::from_ref(&w), &prior)).unwrap();
        assert_eq!(read_decisions(tmp.path()).get("2_Log/Archive/S030_log.md").map(String::as_str), Some("complete"));
        fs::write(tmp.path().join(DECISION_REL), decision_text(&[w], &BTreeMap::new())).unwrap();
        assert!(read_decisions(tmp.path()).is_empty(), "empty status is not a decision");
    }

    #[test]
    fn status_word_prefix_needs_a_word_boundary() {
        for (v, w) in [
            ("완료: 결론", Some("완료")),
            ("[보관] 결론", Some("보관")),
            ("진행 중 — 미정", Some("진행 중")),
            ("Complete. found x", Some("Complete")),
            ("completed - x", Some("Completed")),
            ("★ 활성", Some("★")),
            ("완료된 실험의 재현", None),
            ("Doneness 측정", None),
            ("결론: 완료", None),
            ("", None),
        ] {
            assert_eq!(status_word_prefix(v), w, "{v}");
        }
    }

    #[test]
    fn body_goal_line_only_before_the_first_trial() {
        assert_eq!(body_goal_line("> 이관 전 헤더: x\n\n**목표**: 측정 범위 확정\n\n## t01: a\n### 목표 (Goal)\n"), Some("**목표**: 측정 범위 확정".into()));
        assert_eq!(body_goal_line("소개\n\n## t01: a\n\n### 목표 (Goal)\n- x\n"), None, "trial goals are not the session goal");
        assert_eq!(body_goal_line("- Goal: find it\n"), Some("- Goal: find it".into()));
    }

    #[test]
    fn locate_maps_changes_to_result_lines_in_order() {
        let text = "---\nstatus: complete\n---\n# S001: t\n\n[a](../x.md) `../y.md`\n\n[b](../x.md)\n";
        let raw = vec![
            (1, "link", "x.md".to_string(), "../x.md".to_string()),
            (1, "span", "y.md".to_string(), "../y.md".to_string()),
            (3, "link", "x.md".to_string(), "../x.md".to_string()),
            (9, "link", "z.md".to_string(), "../gone.md".to_string()),
        ];
        let mut out = Vec::new();
        locate("2_Log/S/S001_log.md", text, raw, 5, &mut out);
        assert_eq!(out.iter().map(|c| c.line).collect::<Vec<_>>(), vec![6, 6, 8, 8], "not found = expected line, at most the last line");
    }

    #[test]
    fn locate_skips_an_unchanged_earlier_occurrence() {
        // 3.0.1: 바뀐 span과 같은 문자열이 앞 줄에 바뀌지 않은 채 있음(실제 _dev S038) — 앞 줄이 아니라 바뀐 줄을 보고
        let text = "---\nstatus: complete\n---\n# S038: t\n\n`Archive/S001_log.md` → `S001_log.md`\n\n접두 추가(`S001_log.md`)\n";
        let raw = vec![(6, "span", "../Archive/S001_log.md".to_string(), "S001_log.md".to_string())];
        let mut out = Vec::new();
        locate("2_Log/S/S038_log.md", text, raw, 2, &mut out);
        assert_eq!(out[0].line, 8);
    }

    #[test]
    fn span_recompute_keeps_a_trailing_slash() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("2_Log")).unwrap();
        fs::create_dir_all(tmp.path().join("docs")).unwrap();
        let text = "`../docs/` `../docs` `../2_Log/`\n";
        let rw = rewrite_links(tmp.path(), text, "2_Log", "2_Log/S", &BTreeMap::new());
        assert_eq!(rw.text, "`../../docs/` `../../docs` `../`\n");
        assert_eq!(rw.spans, 3);
    }
}
