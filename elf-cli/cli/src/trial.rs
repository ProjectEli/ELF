//! `elf trial new` / `elf trial write` — trial 만들기와 절 쓰기 (S021 t03, t07; S044 t33~t40).
//!
//! - `trial new "<제목>"`: 새 번호, 제목, 도구 주석, Phase 1 틀(정본 trialTemplate의 Phase 1 부분).
//!   `--from <파일>`은 new 다음 write와 같은 줄임 표기.
//! - `trial write [tNN] --from <파일>`: 파일에 든 `### ` 절을 trial에 씀(동작 하나). Phase 1 절은 Phase 2가
//!   생기기 전까지 여러 번(교체), Phase 2 절은 한 번(덧붙임). 한 파일에는 한 Phase의 절만.
//! - 쓰기 전 검사(관문): 문제를 모두 모아 한 번에 보고하고 아무것도 쓰지 않음. 정확한 동의어 표의 절 이름과
//!   해석 첫 줄 라벨은 바로잡고 알림. 절 순서는 고치지 않음(Phase 위반의 신호일 수 있음).
//! - 도구 주석(trial 제목 다음 줄): Phase 1 첫 쓰기와 마지막 쓰기 시각, 쓰기 횟수, Phase 2 시각.
//!   마지막 Phase 1 쓰기 뒤 생긴 그 세션 산출물 = 실행의 흔적 → Phase 1 수정 거부, 그 뒤 생긴 그림은 Phase 2 본문 필수.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use chrono::{DateTime, FixedOffset};

use crate::header;
use crate::logs::{self, LogFile};
use crate::sections::{self, INTERP_EN, INTERP_KO, INTERP_LEGACY, LogLang, SECTIONS};
use crate::session::{SessionError, refuse_legacy};
use crate::validate::{
    extract_image_targets, noembed_filenames, section_key, trial_structure_findings, trial_template,
    unfilled_template_findings,
};

/// Phase 1 절(배경은 선택). Phase 2 절(시행착오는 선택).
pub const PHASE1: [&str; 5] = ["배경", "목표", "조건", "가설", "예상"];
pub const PHASE1_REQUIRED: [&str; 4] = ["목표", "조건", "가설", "예상"];
pub const PHASE2: [&str; 5] = ["관찰", "해석", "교훈", "생성 파일", "시행착오"];
pub const PHASE2_REQUIRED: [&str; 4] = ["관찰", "해석", "교훈", "생성 파일"];

const META_PREFIX: &str = "<!-- elf-trial:";

pub struct TrialNewOptions {
    /// trial 제목 — None이면 `[작업 제목]` 유지(틀만 붙일 때)
    pub title: Option<String>,
    pub session: Option<String>,
    /// YYYY-MM-DD — 헤더 modified
    pub date: String,
    /// `--from` 파일 내용(Phase 1 절, 맨 앞 YAML 블록은 handoff만)
    pub body: Option<String>,
    pub now: DateTime<FixedOffset>,
}

pub struct TrialWriteOptions {
    /// 대상 trial `tNN` — None이면 그 세션의 마지막 trial
    pub trial: Option<String>,
    pub session: Option<String>,
    pub date: String,
    pub body: String,
    pub now: DateTime<FixedOffset>,
}

#[derive(Debug)]
pub struct TrialResult {
    pub session: String,
    pub trial: String,
    pub log_rel: String,
    /// 0 = 틀만, 1 = Phase 1, 2 = Phase 2
    pub phase: u8,
    /// Phase 1 쓰기 횟수(이번 포함)
    pub writes: u32,
    pub handoff_replaced: bool,
    /// 바로잡은 표기(절 이름, 해석 첫 줄 라벨)
    pub normalized: Vec<String>,
    pub warnings: Vec<String>,
}

// ── 도구 주석 ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrialMeta {
    pub p1_first: Option<String>,
    pub p1_last: Option<String>,
    pub writes: u32,
    pub p2: Option<String>,
}

impl TrialMeta {
    pub fn render(&self) -> String {
        let p1 = match (&self.p1_first, &self.p1_last) {
            (Some(f), Some(l)) => format!("phase1 {f} ~ {l} ({} writes)", self.writes),
            _ => "phase1 -".into(),
        };
        let p2 = self.p2.as_deref().map(|t| format!("; phase2 {t}")).unwrap_or_default();
        format!("{META_PREFIX} {p1}{p2} -->")
    }

    pub fn parse(line: &str) -> Option<TrialMeta> {
        let inner = line.trim().strip_prefix(META_PREFIX)?.strip_suffix("-->")?.trim();
        let mut m = TrialMeta::default();
        for part in inner.split("; ") {
            if let Some(r) = part.strip_prefix("phase1 ") {
                if let Some((range, writes)) = r.split_once(" (") {
                    if let Some((f, l)) = range.split_once(" ~ ") {
                        m.p1_first = Some(f.trim().to_string());
                        m.p1_last = Some(l.trim().to_string());
                    }
                    m.writes = writes.trim_end_matches(" writes)").trim().parse().unwrap_or(0);
                }
            } else if let Some(t) = part.strip_prefix("phase2 ") {
                m.p2 = Some(t.trim().to_string());
            }
        }
        Some(m)
    }
}

fn epoch(rfc3339: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(rfc3339).ok().map(|d| d.timestamp_millis())
}

fn mtime(p: &Path) -> Option<i64> {
    fs::metadata(p).ok()?.modified().ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as i64)
}

// ── 템플릿 ────────────────────────────────────────────────────────

/// 틀의 Phase 1 부분(첫 Phase 2 절 앞까지, 제목 줄 제외)과 Phase 2 부분.
fn template_parts(lang: LogLang) -> (String, String) {
    let tpl = trial_template(lang);
    let mut lines = tpl.lines();
    lines.next(); // `## t{NN}: [작업 제목]`
    let rest: Vec<&str> = lines.collect();
    let cut = rest.iter().position(|l| section_key(l).is_some_and(|k| PHASE2.contains(&k))).unwrap_or(rest.len());
    let p1 = rest[..cut].join("\n").trim_matches('\n').to_string();
    let p2 = rest[cut..].join("\n").trim_matches('\n').to_string();
    (p1, p2)
}

// ── 바로잡기(정확한 동의어 표만) ───────────────────────────────────

/// 절 제목의 정확한 동의어(`### 목표`, `### Goal`, `### Goal (목표)`, `### 목표 (Goal)`) → 프로젝트 언어 표기
/// (영어 프로젝트 `### Goal`, 그 밖 `### 목표 (Goal)`, P017); 해석 첫 줄의 다른 라벨 → 프로젝트 언어 라벨.
pub fn normalize(body: &str, lang: LogLang) -> (String, Vec<String>) {
    let mut notes = Vec::new();
    let mut out = Vec::new();
    let mut interp_pending = false;
    let target = lang.interp_label();
    for line in body.lines() {
        let l = line.trim_end_matches('\r');
        if let Some(rest) = l.strip_prefix("### ") {
            let rest = rest.trim();
            let hit = SECTIONS.iter().find(|(ko, en)| {
                rest == *ko
                    || rest.eq_ignore_ascii_case(en)
                    || rest == format!("{en} ({ko})")
                    || rest == format!("{ko} ({en})")
            });
            let fixed = match hit {
                Some((ko, _)) => {
                    let canon = sections::heading(ko, lang);
                    if l != canon {
                        notes.push(format!("heading '{l}' → '{canon}'"));
                    }
                    canon
                }
                None => l.to_string(),
            };
            interp_pending = section_key(&fixed) == Some("해석");
            out.push(fixed);
            continue;
        }
        if interp_pending && !l.trim().is_empty() {
            interp_pending = false;
            let t = l.trim_start();
            let lead = &l[..l.len() - t.len()];
            let mut fixed = None;
            for old in [INTERP_LEGACY, INTERP_KO, INTERP_EN].into_iter().filter(|x| *x != target) {
                let old = format!("- {old}:");
                if let Some(v) = t.strip_prefix(&old) {
                    notes.push(format!("interpretation label '{old}' → '- {target}:'"));
                    fixed = Some(format!("{lead}- {target}:{v}"));
                    break;
                }
            }
            out.push(fixed.unwrap_or_else(|| l.to_string()));
            continue;
        }
        out.push(l.to_string());
    }
    (out.join("\n"), notes)
}

// ── trial 위치 ────────────────────────────────────────────────────

/// 본문에서 trial 하나(`## tNN:`부터 다음 `## ` 앞까지)만 떼어 냄.
fn trial_block(text: &str, trial_id: &str) -> String {
    let head = format!("## {trial_id}:");
    let mut out = Vec::new();
    let mut inside = false;
    for l in text.lines() {
        if l.starts_with("## ") {
            if inside {
                break;
            }
            inside = l.starts_with(&head);
        }
        if inside {
            out.push(l);
        }
    }
    out.join("\n")
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

pub fn next_trial_number(content: &str) -> u32 {
    trial_numbers(content).into_iter().max().map_or(1, |m| m + 1)
}

/// trial 범위(줄 인덱스): (제목 줄, 내용 끝 — 뒤쪽 빈 줄과 구분선 `---` 제외).
fn trial_span(lines: &[&str], trial_id: &str) -> Option<(usize, usize)> {
    let head = format!("## {trial_id}:");
    let h = lines.iter().position(|l| l.trim_end_matches('\r').starts_with(&head))?;
    let mut end = lines[h + 1..].iter().position(|l| l.starts_with("## ")).map_or(lines.len(), |i| h + 1 + i);
    while end > h + 1 {
        let t = lines[end - 1].trim();
        if t.is_empty() || t == "---" {
            end -= 1;
        } else {
            break;
        }
    }
    Some((h, end))
}

/// stub 삽입 위치: 구 로그의 `## 다음 세션 후보` 헤딩 직전, 없으면 EOF(`---` 구분과 함께). 기존 본문 불변.
pub fn insert_trial(content: &str, stub: &str) -> String {
    let stub = stub.trim_end_matches('\n');
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
        Some(h) => format!("{}{}\n\n---\n\n{}", &content[..h], stub, &content[h..]),
        None => {
            let trimmed = content.trim_end_matches('\n');
            if trimmed.ends_with("---") || trial_numbers(content).is_empty() {
                format!("{trimmed}\n\n{stub}\n")
            } else {
                format!("{trimmed}\n\n---\n\n{stub}\n")
            }
        }
    }
}

// ── 쓰기 전 검사 ──────────────────────────────────────────────────

/// 파일 내용 → (handoff, 본문). YAML 블록 키는 handoff만.
fn split_input(raw: &str) -> Result<(Option<String>, String), Vec<String>> {
    let (pairs, body) = header::split_block(raw).map_err(|e| vec![e])?;
    let mut problems = Vec::new();
    let mut handoff = None;
    for (k, v) in pairs {
        if k == "handoff" {
            handoff = Some(v);
        } else {
            problems.push(format!("YAML block key '{k}' is not allowed here — only handoff (LogConvention §5.1)"));
        }
    }
    if body.lines().any(|l| l.trim_start().starts_with("> **Handoff**:")) {
        problems.push("the old '> **Handoff**:' line is not used — put `handoff: '…'` in a YAML block at the top of the file".into());
    }
    if problems.is_empty() { Ok((handoff, body)) } else { Err(problems) }
}

fn keys_of(body: &str) -> Vec<String> {
    body.lines().filter_map(section_key).map(str::to_string).collect()
}

/// 본문(바로잡은 뒤) 공통 검사: 제목 줄 금지, 정본 절 이름, Phase 혼합. 반환 = 이 본문의 Phase(1 또는 2).
fn common_checks(body: &str, lang: LogLang, problems: &mut Vec<String>) -> Option<u8> {
    for (n, l) in body.lines().enumerate() {
        if l.starts_with("# ") || l.starts_with("## ") {
            problems.push(format!(
                "line {}: '{}' — the body holds only '### ' sections; the trial heading comes from the title",
                n + 1,
                l.chars().take(40).collect::<String>()
            ));
        }
    }
    let keys = keys_of(body);
    for k in &keys {
        if !PHASE1.contains(&k.as_str()) && !PHASE2.contains(&k.as_str()) {
            problems.push(format!(
                "non-canonical heading '### {k}' — canonical: {} (Phase 1), {} (Phase 2)",
                sections::names(&PHASE1, lang),
                sections::names(&PHASE2, lang)
            ));
        }
    }
    let p1 = keys.iter().any(|k| PHASE1.contains(&k.as_str()));
    let p2 = keys.iter().any(|k| PHASE2.contains(&k.as_str()));
    match (p1, p2) {
        (true, true) => {
            problems.push("one file holds the sections of one phase only — write Phase 1, stop for the user's confirmation, run, then write Phase 2 with `elf trial write` (LogConvention §5.1)".into());
            None
        }
        (true, false) => Some(1),
        (false, true) => Some(2),
        (false, false) => {
            problems.push("no '### ' sections found in the file".into());
            None
        }
    }
}

fn required(body: &str, req: &[&str], phase: u8, lang: LogLang, problems: &mut Vec<String>) {
    let keys = keys_of(body);
    for k in req {
        if !keys.iter().any(|x| x == k) {
            problems.push(format!(
                "missing '### {}' — Phase {phase} needs {} (LogConvention §5.1)",
                sections::name(k, lang),
                req.iter().map(|x| sections::name(x, lang)).collect::<Vec<_>>().join(", ")
            ));
        }
    }
}

/// `6_Exp/` 아래 이름이 `id`인 폴더의 파일 중 `since`(epoch ms)보다 새 파일(최대 n개, 루트 기준 경로).
fn newer_outputs(root: &Path, id: &str, since: i64, n: usize) -> Vec<String> {
    #[allow(clippy::too_many_arguments)]
    fn walk(dir: &Path, root: &Path, id: &str, since: i64, inside: bool, depth: u8, out: &mut Vec<String>, n: usize) {
        if depth > 6 || out.len() >= n {
            return;
        }
        let Ok(rd) = fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let here = inside || e.file_name().to_string_lossy() == id;
            if p.is_dir() {
                walk(&p, root, id, since, here, depth + 1, out, n);
            } else if inside && mtime(&p).is_some_and(|t| t > since) {
                out.push(p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/"));
                if out.len() >= n {
                    return;
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("6_Exp"), root, id, since, false, 0, &mut out, n);
    out
}

// ── 실행기 ────────────────────────────────────────────────────────

fn read_ongoing(root: &Path, session: Option<&str>) -> Result<(LogFile, header::Header, String), SessionError> {
    refuse_legacy(root)?;
    let target = logs::resolve(root, session)?;
    let (h, content) = logs::read(&target)?;
    if h.status != header::Status::Ongoing {
        return Err(SessionError::Rejected(vec![format!(
            "{} is {} — trials are written only to an ongoing session",
            target.id,
            h.status.as_str()
        )]));
    }
    Ok((target, h, content))
}

fn finish(
    root: &Path,
    target: &LogFile,
    mut h: header::Header,
    content: &str,
    date: &str,
    handoff: &Option<String>,
) -> Result<Vec<String>, SessionError> {
    h.modified = date.to_string();
    if let Some(x) = handoff {
        h.handoff = header::one_line(x);
    }
    let rel = logs::rel_of(target);
    let out = header::replace(content, &h).map_err(|e| SessionError::BadHeader(format!("{rel}: {e}")))?;
    fs::write(&target.path, out)?;
    Ok(logs::refresh(root)?)
}

/// `elf trial new`: 새 trial(제목, 도구 주석, Phase 1 틀). `--from`이면 이어서 Phase 1을 씀.
pub fn run_trial_new(root: &Path, opts: &TrialNewOptions) -> Result<TrialResult, SessionError> {
    let (target, h, content) = read_ongoing(root, opts.session.as_deref())?;
    let n = next_trial_number(&content);
    let trial_id = format!("t{n:02}");
    let title = opts.title.as_deref().map(str::trim).filter(|t| !t.is_empty());
    if opts.body.is_some() && title.is_none() {
        return Err(SessionError::Rejected(vec!["--from needs a trial title: elf trial new \"<title>\" --from <file>".into()]));
    }
    let lang = LogLang::of(root);
    let title = title.unwrap_or(lang.title_placeholder());
    let (p1_tpl, _) = template_parts(lang);
    let p1_tpl = p1_tpl.replace("S{NNN}", &target.id);
    let stub = format!("## {trial_id}: {title}\n{}\n\n{p1_tpl}\n", TrialMeta::default().render());
    match &opts.body {
        None => {
            let updated = insert_trial(&content, &stub);
            let warnings = finish(root, &target, h, &updated, &opts.date, &None)?;
            Ok(TrialResult {
                session: target.id.clone(),
                trial: trial_id,
                log_rel: logs::rel_of(&target),
                phase: 0,
                writes: 0,
                handoff_replaced: false,
                normalized: Vec::new(),
                warnings,
            })
        }
        Some(raw) => {
            // new 다음 write와 같음 — 검사를 먼저 해서, 거부되면 trial도 만들지 않음
            let (handoff, body) = split_input(raw).map_err(SessionError::Rejected)?;
            let (body, normalized) = normalize(&body, lang);
            let mut problems = Vec::new();
            let phase = common_checks(&body, lang, &mut problems);
            if phase == Some(2) {
                problems.push("`elf trial new --from` takes Phase 1 sections; Phase 2 goes in `elf trial write` after the run".into());
            }
            required(&body, &PHASE1_REQUIRED, 1, lang, &mut problems);
            let text = format!("## {trial_id}: {title}\n\n{body}\n");
            problems.extend(trial_structure_findings(&text, lang));
            problems.extend(unfilled_template_findings(&text, lang));
            if !problems.is_empty() {
                return Err(SessionError::Rejected(problems));
            }
            let now = opts.now.to_rfc3339();
            let meta = TrialMeta { p1_first: Some(now.clone()), p1_last: Some(now), writes: 1, p2: None };
            let trial = format!("## {trial_id}: {title}\n{}\n\n{body}\n", meta.render());
            let updated = insert_trial(&content, &trial);
            let warnings = finish(root, &target, h, &updated, &opts.date, &handoff)?;
            Ok(TrialResult {
                session: target.id.clone(),
                trial: trial_id,
                log_rel: logs::rel_of(&target),
                phase: 1,
                writes: 1,
                handoff_replaced: handoff.is_some(),
                normalized,
                warnings,
            })
        }
    }
}

/// `elf trial write [tNN] --from <파일>`: Phase 1 절(교체, Phase 2 전까지) 또는 Phase 2 절(덧붙임, 한 번).
pub fn run_trial_write(root: &Path, opts: &TrialWriteOptions) -> Result<TrialResult, SessionError> {
    let (target, h, content) = read_ongoing(root, opts.session.as_deref())?;
    let nums = trial_numbers(&content);
    let trial_id = match &opts.trial {
        Some(t) => {
            let n: u32 = t.strip_prefix('t').and_then(|d| d.parse().ok()).ok_or_else(|| SessionError::NotFound(t.clone()))?;
            if !nums.contains(&n) {
                return Err(SessionError::NotFound(format!("{} {t}", target.id)));
            }
            format!("t{n:02}")
        }
        None => match nums.iter().max() {
            Some(n) => format!("t{n:02}"),
            None => return Err(SessionError::Rejected(vec![format!("{} has no trial yet — start one with `elf trial new \"<title>\" --from <file>`", target.id)])),
        },
    };
    let lang = LogLang::of(root);
    let (handoff, body) = split_input(&opts.body).map_err(SessionError::Rejected)?;
    let (body, normalized) = normalize(&body, lang);

    let lines: Vec<&str> = content.split('\n').collect();
    let (hi, end) = trial_span(&lines, &trial_id).ok_or_else(|| SessionError::NotFound(format!("{} {trial_id}", target.id)))?;
    let title_line = lines[hi].trim_end_matches('\r');
    let meta_idx = (hi + 1..end).find(|&i| lines[i].trim().starts_with(META_PREFIX));
    let mut meta = meta_idx.and_then(|i| TrialMeta::parse(lines[i])).unwrap_or_default();
    let existing: Vec<&str> = lines[meta_idx.map_or(hi + 1, |i| i + 1)..end].to_vec();
    let existing_text = existing.join("\n");
    let has_p2 = keys_of(&existing_text).iter().any(|k| PHASE2.contains(&k.as_str()));
    let since = meta.p1_last.as_deref().and_then(epoch);

    let mut problems = Vec::new();
    let phase = common_checks(&body, lang, &mut problems);
    let now = opts.now.to_rfc3339();
    let new_block: String;
    match phase {
        Some(1) => {
            if has_p2 {
                return Err(SessionError::Rejected(vec![format!(
                    "{trial_id} already has Phase 2 — Phase 1 is fixed after the run; a change that reflects results is a new trial"
                )]));
            }
            required(&body, &PHASE1_REQUIRED, 1, lang, &mut problems);
            if let Some(s) = since {
                let newer = newer_outputs(root, &target.id, s, 5);
                if !newer.is_empty() {
                    problems.push(format!(
                        "outputs of {} appeared after the last Phase 1 write ({}) — Phase 1 is not revised after seeing results; start a new trial for a change that reflects them",
                        target.id,
                        newer.join(", ")
                    ));
                }
            }
            let text = format!("{title_line}\n\n{body}\n");
            problems.extend(trial_structure_findings(&text, lang));
            problems.extend(unfilled_template_findings(&text, lang));
            meta.p1_first.get_or_insert_with(|| now.clone());
            meta.p1_last = Some(now.clone());
            meta.writes += 1;
            new_block = format!("{title_line}\n{}\n\n{body}", meta.render());
        }
        Some(2) => {
            if has_p2 {
                // 덧붙이면 생길 순서 지적은 소음 — 원인 하나만 알림
                return Err(SessionError::Rejected(vec![format!(
                    "{trial_id} already has Phase 2 — a rerun is a new trial (LogConvention §2 trial and error)"
                )]));
            }
            required(&body, &PHASE2_REQUIRED, 2, lang, &mut problems);
            let p1_keys = keys_of(&existing_text);
            for k in PHASE1_REQUIRED {
                if !p1_keys.iter().any(|x| x == k) {
                    problems.push(format!("{trial_id} has no '### {}' — write Phase 1 first", sections::name(k, lang)));
                }
            }
            let full = format!("{title_line}\n\n{}\n\n{body}\n", existing_text.trim_matches('\n'));
            problems.extend(unfilled_template_findings(&full, lang).into_iter().map(|f| format!("{f} (Phase 1 is still the template)")));
            problems.extend(trial_structure_findings(&full, lang));
            if let Some(s) = since {
                let viz = root.join(format!("6_Exp/64_Viz/{}", target.id));
                let mut targets = extract_image_targets(&full);
                targets.extend(extract_image_targets(&content));
                let skip = noembed_filenames(&full);
                if let Ok(rd) = fs::read_dir(&viz) {
                    let mut missing: Vec<String> = rd
                        .flatten()
                        .filter(|e| mtime(&e.path()).is_some_and(|t| t > s))
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                        .filter(|n| crate::validate::is_image(n))
                        .filter(|n| !skip.contains(n) && !targets.iter().any(|t| t.rsplit('/').next() == Some(n.as_str())))
                        .collect();
                    missing.sort();
                    if !missing.is_empty() {
                        problems.push(format!(
                            "figures created after Phase 1 are not embedded: {} — embed each in '### {}' with a Fig number and a one-line alt (or `<!-- noembed: name -->`)",
                            missing.join(", "),
                            sections::name("관찰", lang)
                        ));
                    }
                }
            }
            meta.p2 = Some(now.clone());
            new_block = format!("{title_line}\n{}\n\n{}\n\n{body}", meta.render(), existing_text.trim_matches('\n'));
        }
        _ => new_block = String::new(),
    }
    if !problems.is_empty() {
        return Err(SessionError::Rejected(problems));
    }
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 8);
    out.extend(lines[..hi].iter().map(|s| s.to_string()));
    out.push(new_block);
    out.extend(lines[end..].iter().map(|s| s.to_string()));
    let updated = out.join("\n");
    let mut warnings = finish(root, &target, h, &updated, &opts.date, &handoff)?;
    // Phase 2: 생성 파일 표 경로와 링크 검사(경고만, S054 · v3.7 S056 — 프로젝트 밖, 없음, 폴더 한 단계 차이)
    if phase == Some(2) {
        let dir = target.path.parent().unwrap_or(root);
        let only_this = trial_block(&updated, &trial_id);
        warnings.extend(crate::validate::files_table_findings(&only_this, root, dir, lang));
        warnings.extend(
            crate::validate::link_findings(&only_this, root, dir)
                .into_iter()
                .map(|m| format!("{trial_id}: {m}")),
        );
    }
    Ok(TrialResult {
        session: target.id.clone(),
        trial: trial_id,
        log_rel: logs::rel_of(&target),
        phase: phase.unwrap_or(0),
        writes: meta.writes,
        handoff_replaced: handoff.is_some(),
        normalized,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionNewOptions, run_session_new};

    fn now(h: u32) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(&format!("2026-11-09T{h:02}:00:00+09:00")).unwrap()
    }

    const P1: &str = "### 목표 (Goal)\n- 파장별 SNR\n\n### 조건 (Conditions)\n- 광자 1e8\n\n### 가설 (Hypothesis)\n- 810 우위\n\n### 예상 (Prediction)\n- 810 최대\n";
    const P2: &str = "### 관찰 (Observation)\n- 810 최대\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n\n### 교훈 (Lessons)\n- ok\n\n### 생성 파일 (Files)\n- 없음\n";

    fn project() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("2_Log")).unwrap();
        run_session_new(tmp.path(), &SessionNewOptions { title: "t".into(), date: "2026-11-09".into(), goal: "g".into(), gov: false }).unwrap();
        tmp
    }

    fn new(root: &Path, title: Option<&str>, body: Option<&str>) -> Result<TrialResult, SessionError> {
        run_trial_new(root, &TrialNewOptions { title: title.map(str::to_string), session: None, date: "2026-11-10".into(), body: body.map(str::to_string), now: now(10) })
    }

    fn write(root: &Path, body: &str, h: u32) -> Result<TrialResult, SessionError> {
        run_trial_write(root, &TrialWriteOptions { trial: None, session: None, date: "2026-11-10".into(), body: body.into(), now: now(h) })
    }

    fn log(root: &Path) -> String {
        fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap()
    }

    #[test]
    fn meta_roundtrip() {
        let m = TrialMeta { p1_first: Some("a".into()), p1_last: Some("b".into()), writes: 3, p2: Some("c".into()) };
        assert_eq!(TrialMeta::parse(&m.render()), Some(m));
        assert_eq!(TrialMeta::parse(&TrialMeta::default().render()), Some(TrialMeta::default()));
    }

    #[test]
    fn new_without_from_appends_phase1_template_only() {
        let tmp = project();
        let r = new(tmp.path(), None, None).unwrap();
        assert_eq!((r.trial.as_str(), r.phase), ("t01", 0));
        let text = log(tmp.path());
        assert!(text.contains("## t01: [작업 제목]\n<!-- elf-trial: phase1 - -->"));
        assert!(text.contains("### 예상 (Prediction)"));
        assert!(!text.contains("### 관찰"), "Phase 2 template is not appended");
    }

    #[test]
    fn new_from_writes_phase1_and_write_revises_then_phase2() {
        let tmp = project();
        let root = tmp.path();
        let r = new(root, Some("sweep"), Some(&format!("---\nhandoff: 'h1; 미완료 = 확인 대기'\n---\n{P1}"))).unwrap();
        assert_eq!((r.phase, r.writes, r.handoff_replaced), (1, 1, true));
        assert!(log(root).contains("handoff: 'h1; 미완료 = 확인 대기'"));
        // Phase 1 수정(교체) — 횟수 증가, 내용 교체
        let r2 = write(root, &P1.replace("광자 1e8", "광자 1e9"), 11).unwrap();
        assert_eq!((r2.phase, r2.writes), (1, 2));
        let text = log(root);
        assert!(text.contains("광자 1e9") && !text.contains("광자 1e8"));
        assert!(text.contains("(2 writes)"));
        // Phase 2
        let r3 = write(root, P2, 12).unwrap();
        assert_eq!(r3.phase, 2);
        let text = log(root);
        assert!(text.contains("; phase2 2026-11-09T12:00:00+09:00"));
        assert!(text.find("### 예상").unwrap() < text.find("### 관찰").unwrap());
        // Phase 2 이후 Phase 1 수정 거부, Phase 2 재쓰기 거부
        assert!(matches!(write(root, P1, 13), Err(SessionError::Rejected(v)) if v.iter().any(|x| x.contains("already has Phase 2"))));
        assert!(matches!(write(root, P2, 13), Err(SessionError::Rejected(_))));
    }

    #[test]
    fn gate_reports_all_problems_and_writes_nothing() {
        let tmp = project();
        let root = tmp.path();
        let before = log(root);
        let bad = format!("## t09: x\n{P1}{P2}### 메모\n- y\n");
        let Err(SessionError::Rejected(v)) = new(root, Some("x"), Some(&bad)) else { panic!() };
        assert!(v.iter().any(|x| x.contains("heading comes from the title")));
        assert!(v.iter().any(|x| x.contains("one phase only")));
        assert!(v.iter().any(|x| x.contains("non-canonical heading '### 메모'")));
        assert_eq!(log(root), before, "nothing written");
        assert!(matches!(new(root, Some("x"), Some("---\ngoal: 'x'\n---\n### 목표\n- a\n")), Err(SessionError::Rejected(v)) if v[0].contains("only handoff")));
        assert!(matches!(new(root, Some("x"), Some(&format!("> **Handoff**: old\n{P1}"))), Err(SessionError::Rejected(v)) if v.iter().any(|x| x.contains("YAML block"))));
    }

    #[test]
    fn normalizes_exact_synonyms_only() {
        let tmp = project();
        let root = tmp.path();
        let body = "### Goal\n- a\n\n### 조건\n- b\n\n### Hypothesis (가설)\n- c\n\n### 예상 (Prediction)\n- d\n";
        let r = new(root, Some("x"), Some(body)).unwrap();
        assert_eq!(r.normalized.len(), 3);
        let text = log(root);
        assert!(text.contains("### 목표 (Goal)\n- a") && text.contains("### 조건 (Conditions)") && text.contains("### 가설 (Hypothesis)"));
        let p2 = P2.replace("- 예상 일치 정도: 일치", "- 가설 적중 여부: 일치");
        let r2 = write(root, &p2, 12).unwrap();
        assert!(r2.normalized.iter().any(|n| n.contains("interpretation label")));
        assert!(log(root).contains("- 예상 일치 정도: 일치"));
    }

    #[test]
    fn normalize_follows_project_language() {
        // P017: 영어 프로젝트 = `### Goal`, 해석 라벨 `Prediction match`; 모르는 괄호 표기는 그대로
        let body = "### 목표 (Goal)\n- a\n\n### 조건\n- b\n\n### hypothesis\n- c\n\n### 예상 (메모)\n- d\n\n### 해석 (Interpretation)\n- 예상 일치 정도: 일치\n";
        let (out, notes) = normalize(body, LogLang::En);
        assert!(out.contains("### Goal\n- a") && out.contains("### Conditions\n") && out.contains("### Hypothesis\n"), "{out}");
        assert!(out.contains("### 예상 (메모)"), "{out}");
        assert!(out.contains("### Interpretation\n- Prediction match: 일치"), "{out}");
        assert_eq!(notes.len(), 5, "{notes:?}");
        let (out, notes) = normalize("### Goal\n- a\n\n### 목표 (Goal)\n- b\n", LogLang::Ko);
        assert_eq!(out, "### 목표 (Goal)\n- a\n\n### 목표 (Goal)\n- b");
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn phase1_revision_refused_after_outputs_and_phase2_needs_new_figures() {
        let tmp = project();
        let root = tmp.path();
        new(root, Some("x"), Some(P1)).unwrap();
        // 마지막 Phase 1 쓰기 뒤(과거 시각 기준) 생긴 산출물과 그림
        fs::create_dir_all(root.join("6_Exp/64_Viz/S001")).unwrap();
        fs::write(root.join("6_Exp/64_Viz/S001/fig1.png"), b"x").unwrap();
        // 도구 주석 시각을 과거로(파일 mtime이 더 새로움)
        let text = log(root).replace("2026-11-09T10:00:00+09:00 ~ 2026-11-09T10:00:00+09:00", "2000-01-01T00:00:00+09:00 ~ 2000-01-01T00:00:00+09:00");
        fs::write(root.join("2_Log/S/S001_log.md"), text).unwrap();
        assert!(matches!(write(root, P1, 11), Err(SessionError::Rejected(v)) if v.iter().any(|x| x.contains("appeared after the last Phase 1 write"))));
        assert!(matches!(write(root, P2, 12), Err(SessionError::Rejected(v)) if v.iter().any(|x| x.contains("fig1.png"))));
        let p2 = P2.replace("- 810 최대\n\n### 해석", "- 810 최대\n- ![Fig1: SNR](../../6_Exp/64_Viz/S001/fig1.png)\n\n### 해석");
        assert_eq!(write(root, &p2, 12).unwrap().phase, 2);
    }

    #[test]
    fn insert_trial_positions() {
        let with_next = "# S\n\n## t01: a\n\n---\n\n## 다음 세션 후보\n- x\n";
        let out = insert_trial(with_next, "## t02: b\n");
        assert!(out.contains("## t02: b\n\n---\n\n## 다음 세션 후보"));
        let plain = "# S\n\n<!-- c -->\n";
        assert_eq!(insert_trial(plain, "## t01: a\n"), "# S\n\n<!-- c -->\n\n## t01: a\n");
        let two = "# S\n\n## t01: a\n- x\n";
        assert_eq!(insert_trial(two, "## t02: b"), "# S\n\n## t01: a\n- x\n\n---\n\n## t02: b\n");
        assert_eq!(trial_numbers("## t01: a\n## t1x: b\n## t03: c\n"), vec![1, 3]);
    }
}
