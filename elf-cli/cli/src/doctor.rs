//! 환경, 프로젝트 진단 검사 묶음 (P012 t05). **읽기전용**.
//! `elf check setup` = `setup_checks`, `elf check files` = `files_checks` + 관리 파일 상태(status).
//! 구 이름 `elf doctor`(숨김 별칭)는 `run_doctor` — 두 묶음과 status 요약을 기존 순서로 출력.
//!
//! ① CLI 버전, 설치 receipt ② `.elf/` 무결성(stamp, version, baseline) ③ managed 파일 상태(status)
//! ④ git 저장소, 훅. 네트워크 버전 체크는 하지 않음(오프라인-safe). 프로젝트 밖에서도 환경 진단만 수행.
//! receipt 탐지(axoupdater)는 main이 주입 → `run_doctor`는 결정론적(테스트 가능).

use std::fs;
use std::path::Path;

use crate::{autoread, deprecate, manifest, status, tsa, update};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Ok,
    Warn,
    Info,
}

#[derive(Debug)]
pub struct Check {
    pub health: Health,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct DoctorReport {
    pub checks: Vec<Check>,
}

impl DoctorReport {
    pub fn warnings(&self) -> usize {
        self.checks.iter().filter(|c| c.health == Health::Warn).count()
    }
    fn add(&mut self, health: Health, label: &str, detail: impl Into<String>) {
        self.checks.push(Check { health, label: label.into(), detail: detail.into() });
    }
}

/// 환경 정보(주입형). receipt 탐지는 axoupdater라 main이 채워 넣고, run_doctor는 FS만 본다.
pub struct DoctorEnv {
    pub version: String,
    pub receipt_present: bool,
}

/// install receipt 존재 여부(axoupdater, 읽기전용, 오프라인). main에서 호출해 `DoctorEnv` 구성.
pub fn probe_receipt() -> bool {
    let mut u = axoupdater::AxoUpdater::new_for("elf-cli");
    u.load_receipt().is_ok()
}

/// `elf doctor`: cwd 기준 종합 진단(env 주입). 프로젝트 밖이면 환경 검사만.
pub fn run_doctor(cwd: &Path, env: &DoctorEnv) -> DoctorReport {
    let mut r = DoctorReport::default();

    // ── 환경 ──
    r.add(Health::Info, "elf version", env.version.clone());
    if env.receipt_present {
        r.add(Health::Ok, "install receipt", "found — `elf self update` works");
    } else {
        r.add(Health::Info, "install receipt", "absent — `elf self update` unavailable (dev/manual build)");
    }

    // ── 프로젝트(.elf/) ──
    match update::find_project_root(cwd) {
        None => r.add(
            Health::Info,
            "ELF project",
            "not inside an ELF project (.elf/ not found) — environment checks only",
        ),
        Some(root) => {
            r.add(Health::Ok, "ELF project", format!("root: {}", root.display()));
            check_manifest_version(&root, env, &mut r);
            check_elf(&root, &mut r);
            check_claudeignore(&root, &mut r);
            check_experimental(&root, &mut r);
            check_status(&root, &mut r);
            check_i18n(&root, &mut r);
            check_overlay(&root, &mut r);
            check_l2(&root, &mut r);
            check_tsa(&root, &mut r);
            check_autoread(&root, &mut r);
        }
    }

    // ── git ──
    check_git(cwd, &mut r);

    r
}

/// `elf check setup` — git과 훅, (프로젝트 안이면) autoread, experimental. CLI와 프로젝트 버전은 `check version`.
pub fn setup_checks(cwd: &Path, _env: &DoctorEnv) -> DoctorReport {
    let mut r = DoctorReport::default();
    match update::find_project_root(cwd) {
        None => r.add(Health::Info, "ELF project", "not inside an ELF project (.elf/ not found) — project settings skipped"),
        Some(root) => {
            r.add(Health::Ok, "ELF project", format!("root: {}", root.display()));
            check_experimental(&root, &mut r);
            check_autoread(&root, &mut r);
        }
    }
    check_git(cwd, &mut r);
    r
}

/// `elf check files`의 진단 묶음 — `.elf` 기록, `.claudeignore`, 언어 파일, overlay, AGENTS 연결, tsa.
/// 관리 파일 상태(status)는 호출 측이 이어 붙임.
pub fn files_checks(root: &Path, _env: &DoctorEnv) -> DoctorReport {
    let mut r = DoctorReport::default();
    check_elf(root, &mut r);
    check_claudeignore(root, &mut r);
    check_i18n(root, &mut r);
    check_overlay(root, &mut r);
    check_l2(root, &mut r);
    check_tsa(root, &mut r);
    let leftovers = update::legacy_rule_copies(root);
    if !leftovers.is_empty() {
        r.add(Health::Warn, "0_Meta", update::legacy_copies_message(root, &leftovers));
    }
    r
}

/// 구 이름 `elf doctor` 전용 — `check version`과 같은 판정을 한 항목으로.
fn check_manifest_version(root: &Path, env: &DoctorEnv, r: &mut DoctorReport) {
    let v = crate::version::verdict_with(root, &env.version);
    let (k, line) = crate::version::verdict_line(&v, &env.version);
    r.add(if k == 'o' { Health::Ok } else { Health::Warn }, "project manifest", line);
}

fn check_elf(root: &Path, r: &mut DoctorReport) {
    let elf = root.join(".elf");
    if elf.join("baseline").is_dir() {
        r.add(Health::Ok, ".elf baseline", "present (hybrid block diffing enabled)");
    } else {
        r.add(Health::Info, ".elf baseline", "absent (no hybrid files or pre-baseline project)");
    }
}

/// 루트 `.claudeignore` — Claude Code에서 효과가 없고 v2.24부터 ELF가 배포, 관리하지 않음(S037). 있으면 안내만(자동 삭제 없음).
fn check_claudeignore(root: &Path, r: &mut DoctorReport) {
    if root.join(".claudeignore").is_file() {
        r.add(
            Health::Info,
            ".claudeignore",
            "present — it has no effect in Claude Code and is not ELF-managed since v2.24; safe to delete (use your harness's own access rules if you still need exclusions)",
        );
    }
}

/// `.elf/config.json` `experimental` 배열 — 켜진 experimental 기능(예: deprecate_purge)을 Info로 노출.
fn check_experimental(root: &Path, r: &mut DoctorReport) {
    let list = deprecate::experimental_features(root);
    if !list.is_empty() {
        r.add(Health::Info, "experimental", format!("enabled in .elf/config.json: {}", list.join(", ")));
    }
}

fn check_status(root: &Path, r: &mut DoctorReport) {
    match status::run_status(root) {
        Ok(s) if s.findings() == 0 => {
            r.add(Health::Ok, "managed files", format!("up to date ({} warning(s))", s.warnings))
        }
        Ok(s) => r.add(
            Health::Warn,
            "managed files",
            format!("{} pending, {} conflict(s) — run `elf check files`", s.pending, s.conflicts),
        ),
        Err(e) => r.add(Health::Warn, "managed files", format!("status failed: {e}")),
    }
}

/// i18n 위상 advisory — 비-ko 프로젝트의 규칙 세트 언어(v3.4, S050 t02: 영어 프로젝트는 영어 규칙 문서가 같은 경로의
/// 정본, 다른 언어는 한국어 정본). 3.4 이전 영어 companion(`*.en.md`)이 남아 있으면 안내.
/// 누락, 편집의 actionable 분류는 `check_status`가 담당(중복 경고 회피) → 여기선 Info.
fn check_i18n(root: &Path, r: &mut DoctorReport) {
    let lang = update::read_config_lang(root);
    let p = manifest::lang_primary(&lang);
    if p.is_empty() || p == "ko" {
        return; // 한국어 프로젝트 — 해당 없음
    }
    let Some(stamp) = fs::read_to_string(root.join(".elf").join("manifest.json"))
        .ok()
        .and_then(|t| manifest::parse(&t).ok())
    else {
        return; // stamp 문제는 check_elf가 이미 보고
    };
    let view = stamp.for_lang(&lang);
    let variants = view.files.iter().filter(|e| e.role == Some(manifest::Role::Variant) && e.tier != manifest::Tier::User).count();
    let leftovers: Vec<String> = view
        .files
        .iter()
        .filter(|e| e.role == Some(manifest::Role::Companion))
        .map(|e| e.dest.clone())
        .chain(["AGENTS.en.md".to_string()])
        .chain(
            fs::read_dir(root.join(".elf/managed"))
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| format!(".elf/managed/{}", e.file_name().to_string_lossy()))
                .filter(|n| n.ends_with(".en.md")),
        )
        .filter(|d| root.join(d).is_file())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if variants == 0 {
        r.add(
            Health::Info,
            "i18n",
            format!("lang={lang}: no rule set in this language — the Korean rule documents apply (customize via ProjectRule)"),
        );
    } else {
        r.add(
            Health::Info,
            "i18n",
            format!("lang={lang}: {variants} managed file(s) in this language (rule documents, templates, comments) — canonical in this project (customize via ProjectRule)"),
        );
    }
    if !leftovers.is_empty() {
        r.add(
            Health::Info,
            "i18n",
            format!(
                "translation copies from before 3.4 are no longer used: {} — delete them after moving any edits into ProjectRule",
                leftovers.join(", ")
            ),
        );
    }
}

/// CLAUDE.md의 `@AGENTS.md` import 상태 — check files와 autoread digest 공용 (S043).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsImport {
    /// 본문에 유효한 `@AGENTS.md`(또는 `@./AGENTS.md`)
    Imported,
    /// 언급은 있으나 import되지 않는 자리(백틱, 코드 블록, 블록 HTML 주석, 따옴표, 붙은 문장부호)
    NotImported,
    /// 언급 없음
    Absent,
}

const AGENTS_IMPORTS: [&str; 2] = ["@AGENTS.md", "@./AGENTS.md"];

/// Claude Code import 파싱 규칙 기준 판정(공식 문서 memory: code span과 fenced block 안은 import하지
/// 않음, 경로는 공백에서 끝남, 따옴표로 감싼 경로는 import하지 않음). 블록 HTML 주석은 하네스가
/// 제거하므로 import 안 됨으로 봄(문서 미기재 — 보수적 판정). 유효 = 앞이 줄 시작이나 공백, 뒤가 공백이나 줄 끝.
pub fn agents_import(text: &str) -> AgentsImport {
    let mut fence: Option<(u8, usize)> = None;
    let mut in_comment = false;
    let mut mentioned = false;
    for line in text.lines() {
        mentioned |= AGENTS_IMPORTS.iter().any(|tok| line.contains(tok));
        let t = line.trim_start();
        let indent = line.len() - t.len();
        let run = |c: u8| t.bytes().take_while(|&b| b == c).count();
        if let Some((c, n)) = fence {
            if indent <= 3 && run(c) >= n && t[run(c)..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if in_comment {
            in_comment = !line.contains("-->");
            continue;
        }
        if indent <= 3 {
            // 백틱 fence의 info 문자열에는 백틱 불가(CommonMark) — "```x```"는 code span
            if let Some(c) = [b'`', b'~'].into_iter().find(|&c| run(c) >= 3 && (c == b'~' || !t[run(c)..].contains('`'))) {
                fence = Some((c, run(c)));
                continue;
            }
            if let Some(rest) = t.strip_prefix("<!--") {
                in_comment = !rest.contains("-->");
                continue;
            }
        }
        if has_import(&strip_code_spans(line)) {
            return AgentsImport::Imported;
        }
    }
    if mentioned { AgentsImport::NotImported } else { AgentsImport::Absent }
}

/// 백틱 code span을 import 불가 표시 문자(U+0001)로 바꿈 — 짝 없는 백틱은 글자 그대로(CommonMark).
fn strip_code_spans(line: &str) -> String {
    let b = line.as_bytes();
    let run_at = |i: usize| b[i..].iter().take_while(|&&c| c == b'`').count();
    let mut out = String::new();
    let mut i = 0;
    let mut lit = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let n = run_at(i);
        let mut j = i + n;
        let close = loop {
            match b[j..].iter().position(|&c| c == b'`') {
                None => break None,
                Some(k) => {
                    let m = run_at(j + k);
                    if m == n {
                        break Some(j + k);
                    }
                    j += k + m;
                }
            }
        };
        match close {
            Some(k) => {
                out.push_str(&line[lit..i]);
                out.push('\u{1}');
                i = k + n;
                lit = i;
            }
            None => i += n,
        }
    }
    out.push_str(&line[lit..]);
    out
}

fn has_import(s: &str) -> bool {
    AGENTS_IMPORTS.iter().any(|tok| {
        s.match_indices(tok).any(|(p, _)| {
            s[..p].chars().last().is_none_or(char::is_whitespace)
                && s[p + tok.len()..].chars().next().is_none_or(char::is_whitespace)
        })
    })
}

/// 루트 CLAUDE.md의 import 상태 — 파일이 없으면 None.
pub fn claude_md_agents_import(root: &Path) -> Option<AgentsImport> {
    fs::read_to_string(root.join("CLAUDE.md")).ok().map(|c| agents_import(&c))
}

/// L2 진입 파일 연결 진단 (S021 t04, t06) — AGENTS.md(managed 상태는 check_status 소관)와
/// 에이전트 진입 파일(S054, 3.6): ELF는 `AGENTS.md`만 배포함. Claude Code(v2.1.277+)는 `CLAUDE.md`와
/// `CLAUDE.local.md`가 없으면 `AGENTS.md`를 직접 읽고, 둘 중 하나라도 있으면 그것만 읽음 — 그때는 그 안의
/// `@AGENTS.md` import가 있어야 규칙이 실림. stamp에 AGENTS.md entry가 없는 구버전 프로젝트는 조용히 skip.
fn check_l2(root: &Path, r: &mut DoctorReport) {
    let Some(stamp) = fs::read_to_string(root.join(".elf").join("manifest.json"))
        .ok()
        .and_then(|t| manifest::parse(&t).ok())
    else {
        return; // stamp 문제는 check_elf가 이미 보고
    };
    if !stamp.files.iter().any(|e| e.dest == "AGENTS.md") {
        return; // pre-L2 stamp — `elf update` 후 활성화
    }
    let present: Vec<(&str, String)> = ["CLAUDE.md", "CLAUDE.local.md"]
        .into_iter()
        .filter_map(|n| fs::read_to_string(root.join(n)).ok().map(|c| (n, c)))
        .collect();
    if present.is_empty() {
        r.add(
            Health::Ok,
            "agent entry",
            "AGENTS.md is the entry file — Claude Code reads it directly (2.1.277 or later; with an older version, or a session that cannot read AGENTS.md, add a CLAUDE.md with the line `@AGENTS.md`)",
        );
    }
    for (name, c) in &present {
        match agents_import(c) {
            AgentsImport::Absent => r.add(
                Health::Warn,
                "agent entry",
                format!("{name} exists without a `@AGENTS.md` line — Claude Code then reads {name} instead of AGENTS.md; add the line (your other content can stay) or delete the file (AGENTS.md alone is enough on Claude Code 2.1.277 or later)"),
            ),
            AgentsImport::NotImported => r.add(
                Health::Warn,
                "agent entry",
                format!("{name} mentions @AGENTS.md only where Claude Code does not import it (inside backticks, a code block, an HTML comment or quotes, or with a character attached) — AGENTS.md is not loaded; put `@AGENTS.md` on a line of plain text"),
            ),
            AgentsImport::Imported => {
                let lines = c.lines().count();
                if lines > 15 {
                    r.add(
                        Health::Info,
                        "agent entry",
                        format!("{name} loads @AGENTS.md but contains {lines} lines of extra content — check for duplication with AGENTS.md"),
                    );
                } else {
                    r.add(
                        Health::Ok,
                        "agent entry",
                        format!("{name} → @AGENTS.md connected (no longer required: Claude Code 2.1.277 or later reads AGENTS.md directly; ELF 3.6+ does not create {name})"),
                    );
                }
            }
        }
    }
    for f in ["AGENTS.md.elf-new", "CLAUDE.md.elf-new"] {
        if root.join(f).is_file() {
            r.add(
                Health::Warn,
                "agent entry",
                format!("{f} pending — review and merge or delete"),
            );
        }
    }
}

/// data overlay 진단 (EliRule §2.7) — overlayable 대상의 overlay 인지 + 형식(제외 사유 필수) +
/// 비허용 대상 overlay 경고. overlay 위치 = `0_Meta/<이름>.project.md`(사용자 소유 — 레이아웃 무관 고정).
/// overlay 부재는 보고하지 않음(선택 사항).
fn check_overlay(root: &Path, r: &mut DoctorReport) {
    let Some(stamp) = fs::read_to_string(root.join(".elf").join("manifest.json"))
        .ok()
        .and_then(|t| manifest::parse(&t).ok())
    else {
        return; // stamp 문제는 check_elf가 이미 보고
    };
    let lang = update::read_config_lang(root);
    let mut allowed = std::collections::BTreeSet::new();
    for e in stamp
        .for_lang(&lang)
        .files
        .iter()
        .filter(|e| e.overlayable && e.role != Some(manifest::Role::Companion))
    {
        let Some(ov) = overlay_path(&e.dest) else { continue };
        allowed.insert(ov.clone());
        let p = root.join(&ov);
        if !p.is_file() {
            continue;
        }
        match fs::read_to_string(&p) {
            Ok(text) => {
                let missing = removals_missing_reason(&text);
                if missing == 0 {
                    r.add(Health::Ok, "overlay", format!("{ov} active (base ⊕ overlay)"));
                } else {
                    r.add(
                        Health::Warn,
                        "overlay",
                        format!(
                            "{ov}: {missing} removal entr{} without a reason — each `## 제외 (remove)` entry needs one (EliRule §2.7)",
                            if missing == 1 { "y" } else { "ies" }
                        ),
                    );
                }
            }
            Err(_) => r.add(Health::Warn, "overlay", format!("{ov} unreadable (not UTF-8?)")),
        }
    }
    // base가 overlay 비허용(구조, 규약 파일 등)이거나 미지 대상인 `*.project.md` — 병행 로드되지 않음을 경고
    if let Ok(rd) = fs::read_dir(root.join("0_Meta")) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".project.md") {
                continue;
            }
            let rel = format!("0_Meta/{name}");
            if !allowed.contains(&rel) {
                r.add(
                    Health::Warn,
                    "overlay",
                    format!(
                        "{rel} has no overlayable base — only manifest-marked data files accept overlays (EliRule §2.7)"
                    ),
                );
            }
        }
    }
}

/// managed dest → overlay 경로. dest의 레이아웃(`0_Meta/` 또는 `.elf/managed/`)과 무관하게
/// overlay는 사용자 영역 `0_Meta/<stem>.project.md` 고정.
fn overlay_path(dest: &str) -> Option<String> {
    let base = dest.rsplit('/').next().unwrap_or(dest);
    let stem = base.strip_suffix(".md")?;
    Some(format!("0_Meta/{stem}.project.md"))
}

/// `## 제외 (remove)` 절의 bullet 항목 중 사유 미기재 개수 (KO `사유` / EN `reason`, 대소문자 무시).
fn removals_missing_reason(text: &str) -> usize {
    let mut in_remove = false;
    let mut missing = 0;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("## ") {
            let low = t.to_lowercase();
            in_remove = low.contains("제외") || low.contains("remove");
            continue;
        }
        if in_remove && (t.starts_with("- ") || t.starts_with("* ")) {
            let low = t.to_lowercase();
            if !(low.contains("사유") || low.contains("reason")) {
                missing += 1;
            }
        }
    }
    missing
}

/// tsa 진단 (S022) — opt-in이라 disabled면 침묵. enabled 시: GPG 저작자 계층(선택),
/// 미제출 manifest, 엄밀 검증용 openssl 존재(드문 경로 — 없어도 축적, 경량 검증은 자기완결).
/// autoread — default-on 기능의 반쪽 설치(기능 켬, 훅 부재) 검사 (S031 t06 — CLAUDE.md 포인터 검사와 동렬).
fn check_autoread(root: &Path, r: &mut DoctorReport) {
    if !autoread::is_enabled(root) {
        r.add(Health::Info, "autoread", "disabled (config) — `elf autoread enable` to turn on");
        return;
    }
    let states = autoread::hook_states(root);
    let missing: Vec<&str> = states
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(e, _)| e.as_str())
        .collect();
    let outdated = autoread::hooks_outdated(root);
    if missing.is_empty() && outdated.is_empty() {
        r.add(Health::Ok, "autoread", "enabled — Claude Code hooks installed");
    } else if missing.is_empty() {
        // 설치는 되어 있으나 정의가 예전 것(예: SessionStart matcher에 startup 없음 — S044 t29)
        r.add(
            Health::Warn,
            "autoread",
            format!(
                "hook(s) outdated: {} — run `elf update` to refresh them",
                outdated.join(", ")
            ),
        );
    } else {
        r.add(
            Health::Warn,
            "autoread",
            format!(
                "enabled but hook(s) missing: {} — run `elf update` (or `elf autoread enable`)",
                missing.join(", ")
            ),
        );
    }
    // fulltext 선언 실효성 — 부재, 트리 이탈 경로는 주입 시 fail-open 스킵되므로 여기서만 드러남 (S031 t16).
    let declared = autoread::fulltext_list(root);
    let broken: Vec<String> = declared
        .iter()
        .filter(|rel| !autoread::fulltext_path_ok(rel) || !root.join(rel).is_file())
        .cloned()
        .collect();
    if !declared.is_empty() {
        // digest 상한(Claude Code 훅 출력 상한) 안에 들어가는지 — 넘치는 fulltext는 잘리거나 빠짐 (S044 t28)
        let digest = autoread::build_digest_for(root, "check", false);
        if digest.contains("autoread limit") {
            r.add(
                Health::Warn,
                "autoread fulltext",
                format!(
                    "declared file(s) do not fit the autoread limit of {} chars (Claude Code hook output cap): trimmed or left out — shorten `autoread_fulltext`",
                    autoread::DIGEST_CAP
                ),
            );
        }
        if declared.iter().any(|p| p.ends_with("LogConvention.md")) {
            r.add(
                Health::Info,
                "autoread fulltext",
                "the LogConvention §0 core summary is included by default — declaring the full LogConvention.md is no longer needed",
            );
        }
    }
    if !broken.is_empty() {
        r.add(
            Health::Warn,
            "autoread fulltext",
            format!(
                "declared file(s) not found: {} — fix `autoread_fulltext` in .elf/config.json",
                broken.join(", ")
            ),
        );
    }
}

fn check_tsa(root: &Path, r: &mut DoctorReport) {
    if !tsa::is_enabled(root) {
        return;
    }
    let (manifests, stamped, unstamped) = tsa::evidence_counts(root);
    if unstamped == 0 {
        r.add(Health::Ok, "tsa", format!("enabled — {manifests} manifest(s), {stamped} stamped"));
    } else {
        r.add(
            Health::Warn,
            "tsa",
            format!("{unstamped} unstamped manifest(s) — run `elf tsa --all` when online"),
        );
    }
    for (name, state) in tsa::hook_states(root) {
        match state {
            tsa::HookState::Current => {}
            tsa::HookState::Outdated => r.add(
                Health::Warn,
                "tsa hooks",
                format!("{name} hook calls an old command — run `elf update`"),
            ),
            tsa::HookState::Absent => r.add(
                Health::Warn,
                "tsa hooks",
                format!("{name} hook absent — commits are not recorded; run `elf tsa enable`"),
            ),
            tsa::HookState::Foreign => match tsa::hook_removed_calls(root, name) {
                calls if !calls.is_empty() => r.add(
                    Health::Warn,
                    "tsa hooks",
                    format!(
                        "{name} hook is not elf-tsa's and calls a removed command — replace {} (elf update does not edit this hook)",
                        tsa::replacements_text(&calls)
                    ),
                ),
                _ => r.add(
                    Health::Info,
                    "tsa hooks",
                    format!("{name} hook is not elf-tsa's — make sure it calls `elf tsa` (see `elf tsa enable` output)"),
                ),
            },
        }
    }
    let gpg_on = std::process::Command::new("git")
        .args(["config", "--get", "commit.gpgsign"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);
    if gpg_on {
        r.add(Health::Ok, "tsa gpg", "commit.gpgsign = true (authorship layer active)");
    } else {
        r.add(
            Health::Info,
            "tsa gpg",
            "commit signing off — optional authorship layer (`git config commit.gpgsign true`)",
        );
    }
    let openssl = std::process::Command::new("openssl")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if openssl {
        r.add(Health::Ok, "tsa openssl", "available (full signature-chain verify possible)");
    } else {
        r.add(
            Health::Info,
            "tsa openssl",
            "not found — recording/stamping/lightweight verify are self-contained; openssl only needed for full chain verification",
        );
    }
}

fn check_git(cwd: &Path, r: &mut DoctorReport) {
    let git = cwd.ancestors().map(|a| a.join(".git")).find(|g| g.exists());
    match git {
        None => r.add(Health::Info, "git", "no git repository"),
        Some(g) => {
            r.add(Health::Ok, "git", "repository present");
            if g.is_dir() {
                if g.join("hooks").join("pre-commit").is_file() {
                    r.add(Health::Ok, "git hooks", "pre-commit installed");
                } else {
                    r.add(Health::Info, "git hooks", "no pre-commit hook");
                }
            } else {
                // worktree: .git은 파일 → 훅은 공용 common dir, 깊은 해석 생략(존재만 보고)
                r.add(Health::Info, "git hooks", "worktree (.git file) — hooks in shared dir, not checked");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_path_is_layout_independent() {
        assert_eq!(
            overlay_path("0_Meta/LLMcliche.md").as_deref(),
            Some("0_Meta/LLMcliche.project.md")
        );
        assert_eq!(
            overlay_path(".elf/managed/LLMcliche.md").as_deref(),
            Some("0_Meta/LLMcliche.project.md")
        );
        assert_eq!(overlay_path(".claudeignore"), None); // .md 외 = overlay 비대상
    }

    #[test]
    fn removals_require_reason_ko_or_en() {
        let ok = "# o\n## 추가 (add)\n- foo\n## 제외 (remove)\n- bar — 사유: 도메인 용어\n- baz — reason: domain term\n";
        assert_eq!(removals_missing_reason(ok), 0);
        let bad = "## 제외 (remove)\n- bar\n- baz — 사유: ok\n";
        assert_eq!(removals_missing_reason(bad), 1);
        // 제외 절 밖의 bullet은 사유 불요
        let outside = "## 추가 (add)\n- new-word\n";
        assert_eq!(removals_missing_reason(outside), 0);
    }

    #[test]
    fn agents_import_follows_claude_code_parsing() {
        use AgentsImport::*;
        for ok in [
            "@AGENTS.md\n",
            "@./AGENTS.md",
            "  @AGENTS.md  \n",
            "규칙은 @AGENTS.md 참고\n",
            "`elf` 안내\n@AGENTS.md\n",
            "```\ncode\n```\n@AGENTS.md\n",
            "<!-- note -->\n@AGENTS.md\n",
            "`a` @AGENTS.md `b`\n",
        ] {
            assert_eq!(agents_import(ok), Imported, "{ok:?}");
        }
        for quoted in [
            "`@AGENTS.md`\n",
            "``x `@AGENTS.md` y``\n",
            "```\n@AGENTS.md\n```\n",
            "~~~~md\n@AGENTS.md\n~~~\n~~~~\n",
            "<!--\n@AGENTS.md\n-->\n",
            "<!-- @AGENTS.md -->\n",
            "\"@AGENTS.md\"\n",
            "see @AGENTS.md.\n",
            "x@AGENTS.md\n",
            "`a`@AGENTS.md\n",
            "```@AGENTS.md```\n",
        ] {
            assert_eq!(agents_import(quoted), NotImported, "{quoted:?}");
        }
        // 짝 없는 백틱은 글자 그대로 — 뒤의 import 유효
        assert_eq!(agents_import("`` @AGENTS.md\n"), Imported);
        // `@` 없는 파일 이름 언급(ELF 템플릿 설명 줄) = 언급 없음
        assert_eq!(agents_import("> 규칙은 `AGENTS.md`(요약)\n"), Absent);
        assert_eq!(agents_import(""), Absent);
    }

    #[test]
    fn warnings_counts_only_warn() {
        let mut r = DoctorReport::default();
        r.add(Health::Ok, "a", "x");
        r.add(Health::Info, "b", "y");
        r.add(Health::Warn, "c", "z");
        r.add(Health::Warn, "d", "z");
        assert_eq!(r.warnings(), 2);
        assert_eq!(r.checks.len(), 4);
    }
}
