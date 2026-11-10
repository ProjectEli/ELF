//! `elf check logs` — 로그 점검(구 이름 `elf validate` — 숨김 별칭, P012 t04). **읽기전용** — FS 무변경.
//!
//! 구조 개편(P022): `2_Log/S/`, `2_Log/G/`의 YAML 헤더, status별 값, 번호, 링크, figure embed, trial 구조,
//! 생성 파일(Registry.md)과 헤더의 일치, 이전 형식 흔적. 손 편집과 구버전 기록의 감사가 주 용도
//! (새로 쓰는 내용은 `trial new`/`trial write`/`session close`가 쓰기 전에 검사, S044 t33, t34).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use crate::header::{self, Status};
use crate::logs::{self, Series};

#[derive(Debug, Default)]
pub struct ValidateReport {
    pub lines: Vec<String>,
    /// CI 게이트 대상 (정합 위반: 미등록 로그 / 로그 없는 행 / 중복 번호 / 깨진 cross-ref)
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
    Io(io::Error),
}

impl From<io::Error> for ValidateError {
    fn from(e: io::Error) -> Self {
        ValidateError::Io(e)
    }
}

// ── 순수 검사 (FS 무부작용) ─────────────────────────────────────

/// 중복 번호(정렬, 중복제거).
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

/// 1..max 사이 누락 번호 (로그, Registry 합집합 기준).
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

/// 마크다운 인라인 링크 `](target)`에서 **상대 .md 링크 대상** 추출(순수).
/// `#fragment` 제거, URL(`scheme://`, `mailto:`), 절대경로(`/`), 비 .md 제외, 제목(`"..."`), `<...>` 처리.
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

/// 마크다운 **이미지 임베드** `![alt](target)`의 target 목록(순수).
/// 표/산문 속 경로 *언급*(앞에 `![` 없음)은 제외 — "표에 경로만 기재 ≠ 임베딩"(G1/G4) 판별의 핵심.
pub fn extract_image_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = content.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'!' && b[i + 1] == b'[' {
            // 이미지 패턴: `![` … `](` … `)`
            if let Some(mid) = content[i + 2..].find("](") {
                let after = i + 2 + mid + 2;
                if let Some(close) = content[after..].find(')') {
                    let target = content[after..after + close]
                        .split_whitespace()
                        .next()
                        .unwrap_or("");
                    if !target.is_empty() {
                        out.push(target.to_string());
                    }
                    i = after + close + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// 이미지 확장자 여부(case-insensitive) — gallery와 동일 집합(png/jpg/svg).
pub(crate) fn is_image(name: &str) -> bool {
    let l = name.to_ascii_lowercase();
    l.ends_with(".png") || l.ends_with(".jpg") || l.ends_with(".svg")
}

/// image target 목록에 basename이 `fname`인 임베드가 있으면 true(경로 깊이 무관, 파일명 일치).
fn embedded_basename(targets: &[String], fname: &str) -> bool {
    targets
        .iter()
        .any(|t| Path::new(t).file_name().and_then(|n| n.to_str()) == Some(fname))
}

/// `<!-- noembed: a.png, b.svg -->` 주석에서 의도적 제외 파일명 집합(순수). SI, 본문 미삽입 figure 용.
pub fn noembed_filenames(content: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for (idx, _) in content.match_indices("noembed:") {
        let rest = &content[idx + "noembed:".len()..];
        let end = rest.find("-->").unwrap_or(rest.len());
        for tok in rest[..end].split(|c: char| c == ',' || c.is_whitespace()) {
            if is_image(tok) {
                set.insert(tok.to_string());
            }
        }
    }
    set
}

// ── trial 구조 검사 (안정 코어 — S021 t05/t07) ─────────────────
//
// 검사 대상 = LogConvention §2의 **기계 판정 가능, 개정 빈도 낮은 코어**만:
// ① 비정본 `###` 헤딩 ② 코어 절 순서 ③ 해석 첫 줄 규칙 ④ Phase 절 존재(관찰 있는데 가설/예상 없음).
// 문체, 내용 수준 규칙은 검사하지 않음(기계 판정 불가, 정본↔검사기 동기 부채 방지).
// 활성 로그 전용 — Archive 제외(소급 정책 §2: 신규 작성분부터 적용).

/// 해석 절 첫 줄 라벨(v2.24) — `예상 일치 정도: 일치 / 부분 일치 / 불일치`.
pub const INTERP_LABEL: &str = "예상 일치 정도";
/// 이전 라벨(≤ v2.23) — 기존 로그는 계속 유효(소급 정책).
pub const INTERP_LABEL_LEGACY: &str = "가설 적중 여부";

/// 코어 절의 정본 순서 (배경=0 … 생성 파일=8). `시행착오`는 위치 자유(순서 검사 제외).
pub(crate) const TRIAL_SECTIONS: [&str; 9] =
    ["배경", "목표", "조건", "가설", "예상", "관찰", "해석", "교훈", "생성 파일"];

/// `### 헤딩 (Gloss)` → 한국어 키("생성 파일" 등 공백 포함). `### ` 아닌 줄은 None.
pub(crate) fn section_key(line: &str) -> Option<&str> {
    let rest = line.trim_end_matches('\r').strip_prefix("### ")?;
    let key = match rest.find(" (") {
        Some(i) => &rest[..i],
        None => rest,
    };
    Some(key.trim())
}

/// 로그 본문의 trial 구조 발견 사항(순수). 메시지는 trial id 포함, 파일명은 호출자가 접두.
pub fn trial_structure_findings(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    // trial 블록 분할: `## tNN:` ~ 다음 `## `
    let mut current: Option<(String, Vec<String>)> = None; // (trial id, 헤딩 키 순서)
    let mut interp_pending = false; // 해석 헤딩 직후 첫 내용 줄 검사 대기

    let flush = |trial: &str, keys: &[String], out: &mut Vec<String>| {
        // ② 코어 절 순서 (등장한 것들끼리 단조 증가 — 시행착오 등 비코어는 제외)
        let mut last: Option<usize> = None;
        for k in keys {
            let Some(idx) = TRIAL_SECTIONS.iter().position(|s| s == k) else { continue };
            if let Some(prev) = last
                && idx < prev
            {
                out.push(format!(
                    "{trial}: '### {k}' out of canonical order (after '### {}') — spec: LogConvention §2",
                    TRIAL_SECTIONS[prev]
                ));
            }
            last = Some(last.map_or(idx, |p| p.max(idx)));
        }
        // ④ Phase 절 존재: 관찰 있는데 가설/예상 없음
        let has = |k: &str| keys.iter().any(|x| x == k);
        if has("관찰") && (!has("가설") || !has("예상")) {
            out.push(format!(
                "{trial}: has '### 관찰' but missing '### 가설'/'### 예상' — Phase 1 sections are written before execution (LogConvention §5.1)"
            ));
        }
    };

    for line in content.lines() {
        let l = line.trim_end_matches('\r');
        if let Some(rest) = l.strip_prefix("## ") {
            // 새 trial 헤더 또는 타 섹션 — 진행 중 trial 마감
            if let Some((trial, keys)) = current.take() {
                flush(&trial, &keys, &mut out);
            }
            interp_pending = false;
            let is_trial = rest.starts_with('t')
                && rest[1..].chars().take_while(|c| c.is_ascii_digit()).count() > 0
                && rest[1..].trim_start_matches(|c: char| c.is_ascii_digit()).starts_with(':');
            if is_trial {
                let id: String = rest.chars().take_while(|c| *c != ':').collect();
                current = Some((id, Vec::new()));
            }
            continue;
        }
        let Some((trial, keys)) = current.as_mut() else { continue };
        if let Some(key) = section_key(l) {
            // ① 비정본 헤딩
            if !TRIAL_SECTIONS.contains(&key) && key != "시행착오" {
                out.push(format!(
                    "{trial}: non-canonical heading '### {key}' — canonical: 배경/목표/조건/가설/예상/관찰/해석/교훈/생성 파일/시행착오 (LogConvention §2)"
                ));
            }
            interp_pending = key == "해석";
            keys.push(key.to_string());
            continue;
        }
        // ③ 해석 첫 내용 줄 = `예상 일치 정도: …` (v2.24 — 이전 라벨 `가설 적중 여부`도 인식: 기존 로그 유효)
        if interp_pending && !l.trim().is_empty() {
            if !l.contains(INTERP_LABEL) && !l.contains(INTERP_LABEL_LEGACY) {
                out.push(format!(
                    "{trial}: '### 해석' must start with '{INTERP_LABEL}: 일치/부분 일치/불일치' (LogConvention §2; the older '{INTERP_LABEL_LEGACY}' line is still accepted)"
                ));
            }
            interp_pending = false;
        }
    }
    if let Some((trial, keys)) = current.take() {
        flush(&trial, &keys, &mut out);
    }
    out
}

/// 내장 정본 trialTemplate(빌드 시 embed — 테스트가 존재 보증).
pub(crate) fn trial_template() -> &'static str {
    crate::embed::TEMPLATES
        .get_file("log/trialTemplate.md")
        .and_then(|f| f.contents_utf8())
        .unwrap_or("")
}

/// Phase 1 절 — 실행 전에 채워야 하는 절(Phase 2 절은 Phase 2 대기 trial에 템플릿 문구가 정상적으로 남음).
pub(crate) const PHASE1_SECTIONS: [&str; 4] = ["목표", "조건", "가설", "예상"];
/// trialTemplate 제목 placeholder (`elf trial new`를 제목 없이 실행하면 남음).
const TITLE_PLACEHOLDER: &str = "[작업 제목]";

/// 절 키 → 그 절의 비어 있지 않은 줄들(trim). 줄 단위 상태 기계 — `## `에서 trial 경계.
fn section_lines(body: &[&str]) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for line in body {
        let l = line.trim_end_matches('\r');
        if let Some(key) = section_key(l) {
            out.push((key.to_string(), Vec::new()));
        } else if let Some((_, lines)) = out.last_mut()
            && !l.trim().is_empty()
        {
            lines.push(l.trim().to_string());
        }
    }
    out
}

/// 빈 틀 발견 사항(순수) — 활성 로그의 trial 중 (1) 제목이 `[작업 제목]` (2) Phase 1 절의 내용이
/// 정본 trialTemplate의 같은 절 문구로만 이루어진 경우. 판정 = 템플릿 줄과 정확히 일치(결정적).
/// 템플릿 문구가 일부 남아도 사용자가 쓴 줄이 있으면 채운 것으로 봄(안내 줄 잔존은 경고 대상 아님).
pub fn unfilled_template_findings(content: &str, template: &str) -> Vec<String> {
    let tpl_lines: Vec<&str> = template.lines().collect();
    let tpl: Vec<(String, Vec<String>)> = section_lines(&tpl_lines);
    let tpl_of = |key: &str| -> Option<&Vec<String>> { tpl.iter().find(|(k, _)| k == key).map(|(_, v)| v) };

    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i].trim_end_matches('\r');
        let Some(rest) = l.strip_prefix("## ") else {
            i += 1;
            continue;
        };
        let is_trial = rest.starts_with('t')
            && rest[1..].chars().take_while(|c| c.is_ascii_digit()).count() > 0
            && rest[1..].trim_start_matches(|c: char| c.is_ascii_digit()).starts_with(':');
        let start = i + 1;
        let mut end = start;
        while end < lines.len() && !lines[end].starts_with("## ") {
            end += 1;
        }
        if is_trial {
            let id: String = rest.chars().take_while(|c| *c != ':').collect();
            let title = rest.split_once(':').map(|(_, x)| x.trim()).unwrap_or("");
            if title == TITLE_PLACEHOLDER {
                out.push(format!("{id}: title is still the template placeholder '{TITLE_PLACEHOLDER}' — give the trial a title"));
            }
            for (key, body) in section_lines(&lines[start..end]) {
                if !PHASE1_SECTIONS.contains(&key.as_str()) || body.is_empty() {
                    continue;
                }
                if let Some(tl) = tpl_of(&key)
                    && body.iter().all(|x| tl.contains(x))
                {
                    out.push(format!(
                        "{id}: '### {key}' still holds only the template text — fill it in before execution (LogConvention §5.1)"
                    ));
                }
            }
        }
        i = end;
    }
    out
}

// ── FS 실행기 ──────────────────────────────────────────────────

/// `6_Exp/64_Viz/<id>/`의 이미지 파일명(정렬). 디렉토리 부재 시 빈 벡터.
fn scan_viz_images(viz: &Path) -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(viz) {
        for e in entries.flatten() {
            if let Some(name) = e.file_name().to_str()
                && is_image(name)
            {
                v.push(name.to_string());
            }
        }
    }
    v.sort();
    v
}

/// `elf check logs`: 헤더 형식과 status별 값, 번호, 진행 중 세션 수, 링크, figure embed, trial 구조,
/// Registry.md와 헤더의 일치, 폐기 표기와 보관 파일의 짝, 이전 형식 흔적. 읽기전용.
pub fn run_validate(root: &Path) -> Result<ValidateReport, ValidateError> {
    let mut report = ValidateReport::default();
    let legacy = logs::legacy_markers(root);
    let migrated = Series::ALL.iter().any(|s| root.join(s.dir()).is_dir());
    if !legacy.is_empty() {
        if migrated {
            report.issue(format!(
                "logs in the earlier layout remain ({}) — run `elf update` to move them (a log made by an older CLI is renumbered)",
                legacy.join(", ")
            ));
        } else {
            report.issue(format!(
                "this project uses the earlier log layout ({}) — run `elf update` to migrate (LogConvention §7)",
                legacy.join(", ")
            ));
            return Ok(report);
        }
    }

    let mut structure_hit = false;
    for s in Series::ALL {
        let logs = logs::scan(root, s);
        for name in logs::malformed_names(root, s) {
            report.issue(format!(
                "malformed session log name: {}/{name} — session ids are plain S###/G### (no suffix/variant); give each parallel session its own number",
                s.dir()
            ));
        }
        let known: BTreeSet<u32> = logs.iter().map(|l| l.num).collect();
        for n in numbering_gaps(&known) {
            report.warn(format!("gap in session numbering: {} missing", logs::id_of(s, n)));
        }
        let mut ongoing = Vec::new();
        for l in &logs {
            let content = fs::read_to_string(&l.path)?;
            let fname = format!("{}_log.md", l.id);
            let parsed = header::parse(&content);
            match &parsed {
                Err(e) => report.issue(format!("{fname}: {e} (LogConvention §2)")),
                Ok((h, _)) => match h.status {
                    Status::Ongoing => {
                        ongoing.push(l.id.clone());
                        if h.goal.trim().is_empty() {
                            report.warn(format!("{fname}: goal is empty while ongoing"));
                        }
                    }
                    // 완료 로그의 헤더 값(conclusion 빔, handoff 미완료)은 닫는 때만 검사(`session close` — conclusion 필수,
                    // handoff 미완료 경고). 닫힌 뒤에는 고칠 명령이 없는 지난 기록이고, 이관 전 기록인지는 결정론적으로
                    // 가릴 수 없음(v3.2 S048, SOMG 제안 E2)
                    Status::Complete => {}
                    Status::Deprecated => {
                        if !h.conclusion.is_empty() || !h.user_note.is_empty() {
                            report.warn(format!(
                                "{fname}: a deprecated log keeps conclusion and user_note empty (LogConvention §6)"
                            ));
                        }
                    }
                },
            }
            let dir = l.path.parent().unwrap_or(root);
            for target in extract_md_links(&content) {
                if !dir.join(&target).exists() {
                    report.issue(format!("{fname}: broken cross-ref → {target}"));
                }
            }
            let images = scan_viz_images(&root.join(format!("6_Exp/64_Viz/{}", l.id)));
            if !images.is_empty() {
                let targets = extract_image_targets(&content);
                let skip = noembed_filenames(&content);
                for img in images {
                    if skip.contains(&img) || embedded_basename(&targets, &img) {
                        continue;
                    }
                    report.warn(format!(
                        "{}: figure '{img}' exists in 64_Viz/ but is not embedded in the log body \
                         (table path ≠ embed; add ![..](..) or `<!-- noembed: {img} -->`)",
                        l.id
                    ));
                }
            }
            if matches!(&parsed, Ok((h, _)) if h.status == Status::Ongoing) {
                for f in trial_structure_findings(&content) {
                    structure_hit = true;
                    report.warn(format!("{fname}: {f}"));
                }
                for f in unfilled_template_findings(&content, trial_template()) {
                    report.warn(format!("{fname}: {f}"));
                }
            }
        }
        if ongoing.len() > 1 {
            // 병렬 세션 허용(v3.2 S048, SOMG 제안 E1) — 개수는 운영 정보일 뿐 결함이 아님
            report.lines.push(format!("info: {} ongoing sessions ({})", ongoing.len(), ongoing.join(", ")));
        }
        if let Some(expected) = logs::expected_registry(root, s) {
            let rel = logs::registry_rel(s);
            match fs::read_to_string(root.join(&rel)) {
                Err(_) => report.warn(format!("{rel} is missing — any elf command that changes a log regenerates it")),
                Ok(actual) if actual != expected => report.warn(format!(
                    "{rel} differs from the log headers (edited by hand?) — it is generated; any elf command that changes a log regenerates it"
                )),
                Ok(_) => {}
            }
        }
    }
    for (kind, msg) in crate::deprecate::pairing_findings(root) {
        match kind {
            'i' => report.issue(msg),
            'w' => report.warn(msg),
            _ => report.lines.push(format!("info: {msg}")),
        }
    }
    if structure_hit {
        report.lines.push(
            "hint: write trials with `elf trial new --from` and `elf trial write` (checked before writing) — spec: LogConvention.md §2, §5.1".into(),
        );
    }
    if report.lines.is_empty() {
        report.lines.push("ok: log headers, numbering, cross-refs, and Registry.md are consistent".into());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn image_targets_only_from_image_syntax() {
        // 표/산문 속 경로 언급(`![` 없음)은 제외, 이미지 임베드만 추출 — G1/G4 판별 핵심.
        let md = "| Figure | `../6_Exp/64_Viz/S156/foo.png` |\n\n\
                  ![Fig1: 축 설명](../6_Exp/64_Viz/S156/foo.png)\n[plan](../1_Concept/p.md)";
        let t = extract_image_targets(md);
        assert_eq!(t.len(), 1, "{t:?}");
        assert!(t[0].ends_with("foo.png"));
        assert!(embedded_basename(&t, "foo.png"));
        assert!(!embedded_basename(&t, "bar.png"));
    }

    #[test]
    fn noembed_parses_image_filenames() {
        let s = noembed_filenames("본문 <!-- noembed: a.png, b.svg --> 그리고 noembed: c.jpg -->");
        assert!(s.contains("a.png"));
        assert!(s.contains("b.svg"));
        assert!(s.contains("c.jpg"));
        assert!(!s.contains("noembed")); // 비이미지 토큰 제외
    }

    // ── trial 구조 검사 ────────────────────────────────

    #[test]
    fn canonical_trial_passes_structure_check() {
        let log = "# S001: T\n\n## t01: 작업\n\n### 배경 (Background)\n- 맥락\n\n### 목표 (Goal)\n- x\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n- 가설 적중 여부: 적중\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n\n| 유형 | 파일 |\n|---|---|\n\n## 다음 세션 후보\n- 후보\n";
        assert!(trial_structure_findings(log).is_empty());
    }

    #[test]
    fn phase1_only_trial_is_clean() {
        // 진행 중 trial(멈춤점 상태) — 관찰 이후 절 부재는 정상
        let log = "## t02: 진행중\n\n### 목표 (Goal)\n- x\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n";
        assert!(trial_structure_findings(log).is_empty());
    }

    #[test]
    fn header_only_log_has_no_structure_findings() {
        // v2.22: session new/init 직후(헤더 + 규범 주석만, trial 0) — 구조 검사 findings 없음
        let log = "# S001: T\n\n> **Created**: 2026-09-07\\\n> **Modified**: 2026-09-07\\\n> **Status**: ★ 활성\\\n> **목표**: x\\\n> **관련**: -\\\n> **Handoff**: -\n\n---\n\n<!-- 규범 -->\n";
        assert!(trial_structure_findings(log).is_empty());
    }

    #[test]
    fn detects_unknown_heading_and_order() {
        let log = "## t01: 작업\n\n### 결과 (Results)\n- r\n\n### 가설 (Hypothesis)\n- h\n\n### 목표 (Goal)\n- x\n";
        let f = trial_structure_findings(log);
        assert!(f.iter().any(|m| m.contains("non-canonical heading '### 결과'")), "{f:?}");
        assert!(f.iter().any(|m| m.contains("'### 목표' out of canonical order")), "{f:?}");
    }

    #[test]
    fn detects_missing_phase1_and_interpretation_rule() {
        let log = "## t03: 작업\n\n### 목표 (Goal)\n- x\n\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n- 그냥 해석\n";
        let f = trial_structure_findings(log);
        assert!(f.iter().any(|m| m.contains("missing '### 가설'/'### 예상'")), "{f:?}");
        assert!(f.iter().any(|m| m.contains("예상 일치 정도")), "{f:?}");
    }

    #[test]
    fn interpretation_accepts_current_and_legacy_labels() {
        let base = "## t01: 작업\n\n### 목표 (Goal)\n- x\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n\n### 관찰 (Observation)\n- o\n\n### 해석 (Interpretation)\n";
        assert!(trial_structure_findings(&format!("{base}- 예상 일치 정도: 부분 일치\n")).is_empty());
        assert!(trial_structure_findings(&format!("{base}- 가설 적중 여부: 적중\n")).is_empty()); // 이전 라벨 유효
        let f = trial_structure_findings(&format!("{base}- 해석만\n"));
        assert!(f.iter().any(|m| m.contains("예상 일치 정도") && m.contains("가설 적중 여부")), "{f:?}");
    }

    #[test]
    fn sihaengchago_is_allowed_anywhere() {
        let log = "## t01: 작업\n\n### 목표 (Goal)\n- x\n\n### 조건 (Conditions)\n- c\n\n### 가설 (Hypothesis)\n- h\n\n### 예상 (Prediction)\n- p\n\n### 관찰 (Observation)\n- o\n\n### 시행착오\n\n| 시도 | 결과 |\n|---|---|\n\n### 해석 (Interpretation)\n- 가설 적중 여부: 탈락\n\n### 교훈 (Lessons)\n- l\n\n### 생성 파일 (Files)\n- f\n";
        assert!(trial_structure_findings(log).is_empty());
    }

    #[test]
    fn non_trial_sections_are_ignored() {
        // trial 밖(세션 헤더, 다음 세션 후보)의 `###`는 검사 대상 아님
        let log = "# S001: T\n\n### 임의 절\n\n## 다음 세션 후보\n\n### 가설 후보\n- x\n";
        assert!(trial_structure_findings(log).is_empty());
    }
}
