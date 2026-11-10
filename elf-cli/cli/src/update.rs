//! `elf update` — 프로젝트 ELF 파일 갱신 executor (t04, 1차 = 비대화형).
//!
//! 원칙: **무경고 데이터 손실 금지.**
//! - managed 편집본 → 덮어쓰지 않고 경고 + `<dest>.elf-new` 산출 (`--force`로만 덮어씀)
//! - hybrid → 마커블록만 교체, 사용자 영역 보존. 블록 편집/마커 부재 시 보수적 기본동작
//!   (interactive 질의, update_policy 저장은 2차 — S007 §4.1)
//! - user tier(사용자 소유)/instance → 절대 미접근
//! - 블록 편집 감지 기준 = `.elf/baseline/<dest>` (init/update가 기록하는 배포본 사본)

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::embed;
use crate::hash;
use crate::manifest::{self, Manifest};
use crate::plan::{self, CurrentState, UpdateAction};

pub const MARKER_START: &str = "# >>> ELF managed >>>";
pub const MARKER_END: &str = "# <<< ELF managed <<<";

pub struct UpdateOptions {
    pub dry_run: bool,
    pub force: bool,
}

#[derive(Debug)]
pub enum UpdateError {
    NotElfProject(PathBuf),
    BadStamp(String),
    /// config 선언 preset과 stamp 실체(src 시그니처)의 계보 모순 — 이종 manifest 계획 차단 (S026)
    PresetMismatch { config: String, stamp: manifest::Kind },
    /// 프로젝트 manifest가 이 CLI보다 높음 — 되돌림 차단(exit 3, P022 §8)
    NewerProject(String),
    /// 구조 이관 경고 또는 전제 미충족 — 아무것도 쓰지 않음, 보고 줄(exit 3, LogConvention §7)
    Migration(Vec<String>),
    /// 내장 데이터 불일치 등 — 손상된 설치 의심 (panic 대신 안내, t09 정책)
    Internal(String),
    Io(io::Error),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::NotElfProject(p) => write!(
                f,
                "not an ELF project: {} (no .elf/manifest.json — run inside a project)",
                p.display()
            ),
            UpdateError::BadStamp(s) => write!(f, "stamp(.elf/manifest.json): {s}"),
            UpdateError::PresetMismatch { config, stamp } => write!(
                f,
                "preset mismatch: .elf/config.json declares \"{config}\" but the stamp (.elf/manifest.json) is a {stamp} manifest — refusing to plan against the wrong template set. Fix \"preset\" in .elf/config.json (expected \"{stamp}\") and re-run"
            ),
            UpdateError::NewerProject(s) => write!(f, "{s}"),
            UpdateError::Migration(v) => write!(f, "{}", v.join("\n")),
            UpdateError::Internal(s) => write!(
                f,
                "internal: {s} — corrupted install? run `elf self update` or reinstall"
            ),
            UpdateError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl From<io::Error> for UpdateError {
    fn from(e: io::Error) -> Self {
        UpdateError::Io(e)
    }
}

/// 실행 결과 — main이 출력하고, 테스트가 검증한다.
#[derive(Debug, Default)]
pub struct UpdateReport {
    pub lines: Vec<String>,
    pub conflicts: usize,
    pub warnings: usize,
    pub changed: usize,
}

impl UpdateReport {
    fn note(&mut self, line: String) {
        self.lines.push(line);
    }
    fn warn(&mut self, line: String) {
        self.warnings += 1;
        self.lines.push(format!("warn: {line}"));
    }
}

/// cwd에서 위로 올라가며 `.elf/manifest.json` 보유 폴더(프로젝트 루트)를 찾음 (git 방식)
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|a| a.join(".elf").join("manifest.json").is_file())
        .map(Path::to_path_buf)
}

/// `.elf/config.json`의 `lang`(BCP-47). 부재, 파싱 실패 시 "" (= base만, ko 동작). (P016)
pub(crate) fn read_config_lang(root: &Path) -> String {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("lang").and_then(|l| l.as_str()).map(String::from))
        .unwrap_or_default()
}

/// `.elf/config.json`의 `preset`(init 원문). None = 미기재(pre-S026 init) → 시그니처 추론 경로.
pub(crate) fn read_config_preset(root: &Path) -> Option<String> {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("preset").and_then(|p| p.as_str()).map(String::from))
}

/// 계보 해석의 출처 — 보고 문구, 자동 복구 여부 분기용.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KindSource {
    /// config `preset` 명시 (시그니처 대조 통과)
    Config,
    /// config 미기재 → stamp src 시그니처로 추론
    Inferred,
    /// 삭제된 general preset 프로젝트(config `general` 또는 stamp에 `templates/general/`) → 연구 계보 minimal로 전환
    RemovedGeneral,
}

/// 프로젝트 계보 해석 (S026 t02) — 판정 전부 결정적(no-LLM: 문자열 매핑 + enum 비교):
/// - config `preset` 있음 → `kind_from_preset` 매핑 후 stamp 시그니처와 대조. 모순 = `PresetMismatch`
///   (오선언, config 오복사 상태에서 이종 manifest로 계획하는 사고를 진입 시점에 거부).
/// - 없음(pre-S026 init) → stamp `src_signature()` 추론 (기존 프로젝트 무마이그레이션 동작).
pub(crate) fn resolve_kind(
    root: &Path,
    stamp: &Manifest,
) -> Result<(manifest::Kind, KindSource), UpdateError> {
    let sig = stamp.src_signature();
    let config = read_config_preset(root);
    if config.as_deref() == Some(manifest::REMOVED_GENERAL) || stamp.has_removed_general_src() {
        return Ok((manifest::Kind::Research, KindSource::RemovedGeneral));
    }
    match config {
        Some(p) => {
            let kind = manifest::kind_from_preset(&p);
            if kind != sig {
                return Err(UpdateError::PresetMismatch { config: p, stamp: sig });
            }
            Ok((kind, KindSource::Config))
        }
        None => Ok((sig, KindSource::Inferred)),
    }
}

/// 추론된 preset을 config.json에 영속화(자동 복구 — 다음 실행부터 config 직독).
/// 연구 계보는 미지정 연구 preset = `full`로 기록(계보 이름 `research`를 preset으로 쓰지 않음).
/// 실패는 무해(다음 실행이 재추론)라 조용히 무시. dry-run에서는 호출하지 않는다.
fn repair_config_preset(root: &Path, kind: manifest::Kind) {
    write_config_preset(root, &inferred_preset(kind));
}

/// 추론 시 기록할 preset 이름 — qa = `qa`, 연구 = `full`.
fn inferred_preset(kind: manifest::Kind) -> String {
    match kind {
        manifest::Kind::Qa => "qa".into(),
        manifest::Kind::Research => manifest::normalize_preset(None),
    }
}

fn write_config_preset(root: &Path, preset: &str) {
    let p = root.join(".elf").join("config.json");
    let Ok(text) = fs::read_to_string(&p) else { return };
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&text) else { return };
    let Some(obj) = v.as_object_mut() else { return };
    obj.insert("preset".into(), serde_json::Value::String(preset.into()));
    let Ok(mut out) = serde_json::to_string_pretty(&v) else { return };
    out.push('\n');
    let _ = fs::write(&p, out);
}

pub fn run_update(root: &Path, opts: &UpdateOptions) -> Result<UpdateReport, UpdateError> {
    run_update_with(root, opts, None)
}

/// `run_update` + 이관 미리보기 폴더(`migrate::prepare_preview`로 준비, dry-run에서만 씀).
pub fn run_update_with(root: &Path, opts: &UpdateOptions, preview: Option<&Path>) -> Result<UpdateReport, UpdateError> {
    let stamp_path = root.join(".elf").join("manifest.json");
    // `.elf/` 없는 2.0~2.4.0 프로젝트 = 있는 ELF 파일로 만든 합성 stamp(원본 판정 = 지문, v3.0.2 S046)
    let adopted = !stamp_path.is_file() && crate::earlier::footprint(root);
    if !stamp_path.is_file() && !adopted {
        return Err(UpdateError::NotElfProject(root.to_path_buf()));
    }
    let stamp_text = if adopted { crate::earlier::synthetic_stamp(root) } else { fs::read_to_string(&stamp_path)? };
    // 프로젝트 언어로 stamp, new 둘 다 해석 — companion/variant가 lang에 맞게 일관 필터됨 (P016 §9).
    // stamp는 init이 전체 JSON으로 찍으므로, 같은 lang으로 필터해야 obsolete 오탐 방지.
    let lang = if adopted { crate::earlier::guessed_lang(root) } else { read_config_lang(root) };
    // 프로젝트나 manifest schema가 더 높으면 거부(되돌림 차단, R5, R7) — schema 해석 오류보다 먼저
    if let v @ (crate::version::Verdict::Newer(_) | crate::version::Verdict::NewerSchema(_)) = crate::version::verdict_with(root, embed::version()) {
        return Err(UpdateError::NewerProject(format!(
            "{} — updating would write older ELF files over it (details: `elf check version`)",
            crate::version::verdict_line(&v, embed::version()).1
        )));
    }
    let stamp_full = manifest::parse(&stamp_text).map_err(UpdateError::BadStamp)?;
    // 계보 해석 — preset별 정본 세트 선택 + 모순 시 거부 (S026: 이종 manifest 계획 금지)
    let (kind, kind_source) = resolve_kind(root, &stamp_full)?;
    let mut stamp = stamp_full.for_lang(&lang);
    let new_m = manifest::embedded_kind(kind).for_lang(&lang);

    let mut report = UpdateReport::default();
    // 구조 이관(이전 형식 로그) — 경고나 전제 미충족이면 update 전체를 쓰지 않음(P022 §7)
    let mut migrated = false;
    if crate::migrate::needed(root) {
        use crate::migrate::Outcome;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let lines = match crate::migrate::run_with(root, opts.dry_run, &today, preview)? {
            Outcome::Blocked(lines) => return Err(UpdateError::Migration(lines)),
            Outcome::Applied(lines) => {
                migrated = true;
                lines
            }
            Outcome::Planned(lines) => lines,
        };
        for l in lines {
            match l.strip_prefix("warn: ") {
                Some(w) => report.warn(w.to_string()),
                None => report.note(l),
            }
        }
    }
    // 로그가 없어 이관이 돌지 않은 프로젝트에 남는 이전 배치의 빈 폴더(`.gitkeep`만) — 예: `2_Log/Archive/`
    for d in crate::logs::OLD_EMPTY_DIRS {
        let full = root.join(d);
        if full.is_dir() && fs::read_dir(&full).is_ok_and(|rd| rd.flatten().all(|e| e.file_name() == ".gitkeep")) {
            if !opts.dry_run {
                let _ = fs::remove_file(full.join(".gitkeep"));
                let _ = fs::remove_dir(&full);
            }
            report.note(format!("{} empty folder of the earlier layout: {d}/", if opts.dry_run { "would remove" } else { "removed" }));
        }
    }
    if adopted {
        let what = "ELF 2.0–2.4.0 project without .elf/ — creates .elf/ (config, project manifest); ELF files still as delivered are replaced, edited ones are kept with a .elf-new beside them";
        if opts.dry_run {
            report.note(format!("would adopt: {what}"));
        } else {
            write_adopted_config(root, &lang)?;
            report.note(format!("adopted: {what}"));
        }
    }
    match kind_source {
        KindSource::Config => {
            let cfg = read_config_preset(root);
            if kind == manifest::Kind::Qa {
                report.note("preset: qa".into());
            } else {
                let eff = manifest::normalize_preset(cfg.as_deref());
                if cfg.as_deref() != Some(eff.as_str()) {
                    let was = cfg.unwrap_or_default();
                    if opts.dry_run {
                        report.note(format!("preset: {eff} (research lineage; .elf/config.json says `{was}` — would record `{eff}`)"));
                    } else {
                        write_config_preset(root, &eff);
                        report.note(format!("preset: {eff} (research lineage; recorded `{eff}` to .elf/config.json — was `{was}`)"));
                    }
                } else {
                    report.note(format!("preset: {eff} (research lineage)"));
                }
            }
        }
        KindSource::RemovedGeneral => {
            if opts.dry_run {
                report.note("preset: general was removed in ELF 2.27 — this update would convert the project to `minimal` (research lineage)".into());
            } else {
                write_config_preset(root, "minimal");
                report.note("preset: general was removed in ELF 2.27 — converted this project to `minimal` (research lineage; recorded to .elf/config.json)".into());
            }
            report.note("note: ELF-managed rule files (.elf/managed/, AGENTS.md) take the research versions below; files you own (0_Meta/ProjectRule.md, README.md, logs) are untouched — review ProjectRule for rules that relied on the general versions".into());
        }
        KindSource::Inferred => {
            if opts.dry_run {
                report.note(format!(
                    "preset: {} (inferred from stamp — would record to .elf/config.json)",
                    inferred_preset(kind)
                ));
            } else {
                repair_config_preset(root, kind);
                report.note(format!(
                    "preset: {} (inferred from stamp — recorded to .elf/config.json)",
                    inferred_preset(kind)
                ));
            }
        }
    }
    // 이관을 적용했으면 전제에서 깨끗함을 확인했고 지금의 변경은 이관 자신의 것 — 경고하지 않음
    if !migrated {
        warn_if_git_dirty(root, &mut report);
    }
    // 2.15 이전 배치(규칙 `0_Meta/`, 틀 `templates/`) → 현재 위치(`.elf/managed/`)로 옮김. 옮긴 파일은 원래 stamp sha로
    // 판정 → 원본이면 새 판으로 교체, 고쳤으면 그대로 두고 `.elf-new`(v3.0.2 S046, 2.15.1 `elf migrate`와 같은 이동)
    let new_dests: BTreeSet<&str> = new_m.files.iter().map(|e| e.dest.as_str()).collect();
    // stamp에 없는 구 규칙 사본(`0_Meta/`의 ELF 규칙 이름)도 대상 — 원본 판정 = 지문, 모르면 고친 것으로(지우지 않음)
    for rel in legacy_rule_copies(root) {
        if !stamp.files.iter().any(|e| e.dest == rel) {
            let bytes = fs::read(root.join(&rel))?;
            let sha256 = if crate::earlier::pristine(&rel, &bytes).is_some() { hash::sha256_lf(&bytes) } else { crate::earlier::EDITED.into() };
            stamp.files.push(manifest::Entry { src: format!("earlier/{rel}"), dest: rel, tier: manifest::Tier::Managed, sha256, lang: None, role: None, overlayable: false });
        }
    }
    let mut moves: Vec<(String, String)> = Vec::new();
    for e in &stamp.files {
        if e.tier != manifest::Tier::Managed || new_dests.contains(e.dest.as_str()) {
            continue;
        }
        let Some(to) = crate::earlier::relocated(&e.dest) else { continue };
        if new_dests.contains(to.as_str()) && root.join(&e.dest).is_file() && !root.join(&to).exists() && !moves.iter().any(|(_, t)| *t == to) {
            moves.push((e.dest.clone(), to));
        }
    }
    for e in stamp.files.iter_mut() {
        if let Some((_, to)) = moves.iter().find(|(from, _)| *from == e.dest) {
            e.dest = to.clone();
        }
    }
    warn_if_legacy_leftovers(root, &moves, &mut report);

    // 현재 상태 수집 (new manifest의 dest들 — 옮길 파일은 옮기기 전 위치에서)
    let mut current: CurrentState = CurrentState::new();
    for e in &new_m.files {
        let from = moves.iter().find(|(_, to)| *to == e.dest).map(|(f, _)| f.as_str()).unwrap_or(&e.dest);
        let state = match fs::read(root.join(from)) {
            Ok(bytes) => Some(hash::sha256_lf(&bytes)),
            Err(_) => None,
        };
        current.insert(e.dest.clone(), state);
    }
    for (from, to) in &moves {
        if !opts.dry_run {
            move_file(root, from, to)?;
            let pending = format!("{from}.elf-new");
            if root.join(&pending).is_file() {
                move_file(root, &pending, &format!("{to}.elf-new"))?;
            }
            crate::earlier::prune_empty_parents(root, from);
        }
        report.note(format!("{} (earlier layout): {from} → {to}", if opts.dry_run { "would move" } else { "moved" }));
    }
    let stamp_sha: BTreeMap<String, String> = stamp.files.iter().map(|e| (e.dest.clone(), e.sha256.clone())).collect();

    let actions = plan::plan_update(&new_m, &stamp, &current);

    for a in &actions {
        apply(root, a, opts, &new_m, &stamp_sha, &mut report)?;
    }

    // re-stamp는 무조건 (t02 재점검 §2: NoChange여도 stamp 해시는 구버전) — 반드시 같은 계보로 (S026)
    if !opts.dry_run {
        fs::write(&stamp_path, manifest::project_json(kind))?;
        report.note(format!("project manifest: .elf/manifest.json → ELF {} (schema 2)", embed::version()));
        // 이전 형식의 버전 파일 — 버전은 manifest의 elf_version으로 옮김(P022 §8)
        if root.join(".elf").join("version").is_file() {
            fs::remove_file(root.join(".elf").join("version"))?;
            report.note("removed .elf/version (the version is now elf_version in .elf/manifest.json)".into());
        }
    }

    // autoread 훅 보장 (기본 켬의 "설치 자동화" 지점 — S031 t06): config on(부재=on)이면
    // `.claude/settings.json`에 elf 소유 훅을 멱등 병합. settings.json은 비추적 파생물
    // (.gitignore가 `.claude/` ignore)이라 clone 직후에는 이 단계가 유일한 복원 경로.
    // 실패는 update 전체를 막지 않음(warn 강등 — settings.json malformed 등).
    if crate::autoread::is_enabled(root) {
        if opts.dry_run {
            report.note("autoread: would ensure Claude Code hooks (.claude/settings.json)".into());
        } else {
            match crate::autoread::install_hooks(root) {
                Ok(true) => report.note(
                    "autoread: Claude Code hooks installed/refreshed (.claude/settings.json)".into(),
                ),
                Ok(false) => {}
                Err(e) => report.warn(format!("autoread hooks not installed: {e}")),
            }
        }
    }

    // tsa 훅 갱신 — 켠 프로젝트만, elf 소유(마커) 훅만 현행 본문으로 교체. git hooks는 비추적이라
    // 명령 이름이 바뀌면 이 단계가 하류 프로젝트의 유일한 갱신 경로. 구 본문이면 누락 가능 안내(판단은 프로젝트 몫).
    if crate::tsa::is_enabled(root) {
        for line in crate::tsa::refresh_hooks(root, opts.dry_run) {
            match line.strip_prefix("warn: ") {
                Some(w) => report.warn(w.to_string()),
                None => report.note(line),
            }
        }
    }

    Ok(report)
}

fn apply(
    root: &Path,
    action: &UpdateAction,
    opts: &UpdateOptions,
    new_m: &Manifest,
    stamp_sha: &BTreeMap<String, String>,
    report: &mut UpdateReport,
) -> Result<(), UpdateError> {
    match action {
        UpdateAction::NoChange { dest } => {
            report.note(format!("up-to-date: {dest}"));
        }
        UpdateAction::SkipUser { dest } => {
            report.note(format!("user (untouched): {dest}"));
        }
        UpdateAction::KeepPointer { dest } => {
            // t06: 기존 파일 절대 불변경 + 공지 — `.elf-new` 병기도 하지 않음(1줄 병합은 사용자 몫)
            report.note(format!(
                "pointer (kept as-is): {dest} — ensure it loads AGENTS.md (`@AGENTS.md` line); `elf check files` checks"
            ));
        }
        UpdateAction::Obsolete { dest } => {
            // 없으면 말하지 않음. 전달된 그대로면 지움(빈 상위 폴더도), 고쳤으면 남기고 경고(v3.0.2 S046)
            let Ok(bytes) = fs::read(root.join(dest)) else { return Ok(()) };
            if stamp_sha.get(dest).is_some_and(|s| *s == hash::sha256_lf(&bytes)) {
                if !opts.dry_run {
                    fs::remove_file(root.join(dest))?;
                    crate::earlier::prune_empty_parents(root, dest);
                }
                report.changed += 1;
                report.note(format!(
                    "{} (no longer used by ELF, unchanged since delivered): {dest}",
                    if opts.dry_run { "would remove" } else { "removed" }
                ));
            } else {
                report.warn(format!(
                    "obsolete (no longer managed by ELF; edited, so left in place — delete it once you no longer need your edits): {dest}"
                ));
            }
        }
        UpdateAction::Overwrite { dest } | UpdateAction::CreateMissing { dest } => {
            if !opts.dry_run {
                let bytes = template_bytes(new_m, dest)?;
                write_file(root, dest, bytes)?;
                if entry_is_hybrid(new_m, dest) {
                    write_baseline(root, dest, bytes)?;
                }
            }
            report.changed += 1;
            let verb = if matches!(action, UpdateAction::Overwrite { .. }) {
                "updated"
            } else {
                "created"
            };
            report.note(format!("{verb}: {dest}"));
        }
        UpdateAction::Conflict { dest } => {
            if opts.force {
                if !opts.dry_run {
                    write_file(root, dest, template_bytes(new_m, dest)?)?;
                }
                report.changed += 1;
                report.note(format!("force-updated (user edits discarded): {dest}"));
            } else {
                report.conflicts += 1;
                if !opts.dry_run {
                    write_file(
                        root,
                        &format!("{dest}.elf-new"),
                        template_bytes(new_m, dest)?,
                    )?;
                }
                report.warn(format!(
                    "edited by user — kept; new version at {dest}.elf-new (use --force to overwrite)"
                ));
            }
        }
        UpdateAction::MergeBlock { dest } => {
            merge_hybrid(root, dest, opts, new_m, stamp_sha.get(dest).map(String::as_str), report)?;
        }
    }
    Ok(())
}

fn merge_hybrid(
    root: &Path,
    dest: &str,
    opts: &UpdateOptions,
    new_m: &Manifest,
    stamp_sha: Option<&str>,
    report: &mut UpdateReport,
) -> Result<(), UpdateError> {
    let template = std::str::from_utf8(template_bytes(new_m, dest)?)
        .map_err(|_| UpdateError::Internal(format!("hybrid template not UTF-8: {dest}")))?;
    let new_block = block_of(template)
        .ok_or_else(|| UpdateError::Internal(format!("hybrid template missing marker block: {dest}")))?;

    let current_path = root.join(dest);
    let current_bytes = fs::read(&current_path)?;
    // 파일 전체가 전달된 그대로(이전 판, 표지 없는 generator 판 포함) — 사용자 내용이 없으므로 통째로 새 판(v3.0.2 S046)
    if stamp_sha == Some(hash::sha256_lf(&current_bytes).as_str()) {
        if !opts.dry_run {
            fs::write(&current_path, template.as_bytes())?;
            write_baseline(root, dest, template.as_bytes())?;
        }
        report.changed += 1;
        report.note(format!("updated (unchanged since delivered): {dest}"));
        return Ok(());
    }
    let Ok(current) = std::str::from_utf8(&current_bytes) else {
        report.conflicts += 1;
        report.warn(format!("not UTF-8 — skipped: {dest}"));
        return Ok(());
    };

    match extract_block(current) {
        None => {
            // 마커 부재 (사용자가 삭제) — 보수적 기본 = 미변경 경고; --force = 상단 재삽입.
            // 표지가 생기기 전 판(2.0~2.4.0, 합성 stamp에서 고친 것으로 판정)은 지운 것이 아니므로 상단에 넣음(v3.0.2 S046)
            let before_markers = stamp_sha == Some(crate::earlier::EDITED);
            if opts.force || before_markers {
                if !opts.dry_run {
                    let rebuilt = format!("{new_block}\n{current}");
                    fs::write(&current_path, &rebuilt)?;
                    write_baseline(root, dest, template.as_bytes())?;
                }
                report.changed += 1;
                if before_markers {
                    report.note(format!("ELF block added at the top (earlier file without markers; your lines kept below): {dest}"));
                } else {
                    report.note(format!("marker reinserted at top: {dest}"));
                }
            } else {
                report.conflicts += 1;
                report.warn(format!(
                    "ELF marker block missing — left unchanged: {dest} (use --force to reinsert at top)"
                ));
            }
        }
        Some((s, e)) => {
            let current_block = &current[s..e];
            let baseline_block = read_baseline_block(root, dest);
            let edited = match &baseline_block {
                Some(b) => current_block != b,
                None => true, // baseline 부재 = 검증 불가 → 보수적으로 '편집됨' 취급
            };
            if edited && !opts.force {
                report.conflicts += 1;
                if !opts.dry_run {
                    write_file(root, &format!("{dest}.elf-new"), template.as_bytes())?;
                }
                let reason = if baseline_block.is_some() {
                    "managed block edited by user"
                } else {
                    "no baseline to verify block"
                };
                let (a, b) = block_lines(current, s, e);
                report.warn(format!(
                    "{reason} (lines {a}-{b}; user rules go below the end marker) — kept; new version at {dest}.elf-new (use --force to replace block)"
                ));
            } else {
                let merged = format!("{}{}{}", &current[..s], new_block, &current[e..]);
                if merged == current {
                    // 블록이 이미 최신 (사용자 영역만 정본과 다른 경우) — no-op
                    if !opts.dry_run {
                        write_baseline(root, dest, template.as_bytes())?;
                    }
                    report.note(format!("up-to-date (block current): {dest}"));
                } else {
                    if !opts.dry_run {
                        fs::write(&current_path, merged)?;
                        write_baseline(root, dest, template.as_bytes())?;
                    }
                    report.changed += 1;
                    report.note(format!("merged block (user area preserved): {dest}"));
                }
            }
        }
    }
    Ok(())
}

// ── 마커블록 헬퍼 (순수 — unit test 대상) ───────────────────────

/// 텍스트에서 마커블록의 [시작, 끝) 바이트 범위 (마커 라인 포함, 끝 개행 포함)
pub fn extract_block(text: &str) -> Option<(usize, usize)> {
    let s_idx = text.find(MARKER_START)?;
    // 블록 시작 = 마커가 있는 라인의 첫 바이트
    let start = text[..s_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let e_idx = text[s_idx..].find(MARKER_END)? + s_idx;
    let end = text[e_idx..]
        .find('\n')
        .map(|i| e_idx + i + 1)
        .unwrap_or(text.len());
    Some((start, end))
}

/// 블록 범위(`extract_block`의 바이트 위치) → 줄 번호(1부터, 시작과 끝 표지 줄 포함).
pub fn block_lines(text: &str, s: usize, e: usize) -> (usize, usize) {
    let first = text[..s].matches('\n').count() + 1;
    let last = text[..e].trim_end_matches(['\n', '\r']).matches('\n').count() + 1;
    (first, last)
}

/// 템플릿에서 블록 부분 문자열 추출
pub fn block_of(template: &str) -> Option<&str> {
    extract_block(template).map(|(s, e)| &template[s..e])
}

// ── FS 헬퍼 ─────────────────────────────────────────────────────

fn entry_is_hybrid(m: &Manifest, dest: &str) -> bool {
    m.files
        .iter()
        .any(|e| e.dest == dest && e.tier == manifest::Tier::Hybrid)
}

fn template_bytes<'a>(m: &'a Manifest, dest: &str) -> Result<&'a [u8], UpdateError> {
    let e = m
        .files
        .iter()
        .find(|e| e.dest == dest)
        .ok_or_else(|| UpdateError::Internal(format!("not in manifest: {dest}")))?;
    let rel = e
        .src
        .strip_prefix("templates/")
        .ok_or_else(|| UpdateError::Internal(format!("invalid template path: {}", e.src)))?;
    Ok(embed::TEMPLATES
        .get_file(rel)
        .ok_or_else(|| UpdateError::Internal(format!("embedded template missing: {}", e.src)))?
        .contents())
}

fn write_file(root: &Path, dest: &str, bytes: &[u8]) -> io::Result<()> {
    let p = root.join(dest);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(p, bytes)
}

fn write_baseline(root: &Path, dest: &str, bytes: &[u8]) -> io::Result<()> {
    let p = root.join(".elf").join("baseline").join(dest);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(p, bytes)
}

pub(crate) fn read_baseline_block(root: &Path, dest: &str) -> Option<String> {
    let p = root.join(".elf").join("baseline").join(dest);
    let text = fs::read_to_string(p).ok()?;
    block_of(&text).map(str::to_string)
}

/// pre-v2.15 legacy 잔재 감지 — **안내만**(비이전, 비접촉, 차단하지 않음). (S024 t09)
/// v2.16에서 legacy 레이아웃 지원이 제거되어, 미이전 프로젝트에 update를 실행하면
/// `.elf/managed/`에 새 사본이 생기고 구 파일은 참조 없는 파일로 남는다. managed 규칙명이
/// `0_Meta/`에 남아 있으면 그 상태이므로 2단계 업그레이드 경로를 경고로 안내한다.
/// (감지 대상 = 규칙 5종 고정명 — `0_Meta/`는 사용자 영역이라 동명 파일은 잔재 외 비개연.)
/// `0_Meta/`에 남은 구 규칙 사본(pre-2.15 배치) — update 경고와 `check files` 공용.
pub(crate) fn legacy_rule_copies(root: &Path) -> Vec<String> {
    const LEGACY_RULE_NAMES: [&str; 5] = [
        "EliRule.md",
        "LogConvention.md",
        "AI_PARA_Framework.md",
        "highIFjournals.md",
        "LLMcliche.md",
    ];
    LEGACY_RULE_NAMES
        .iter()
        .filter(|n| root.join("0_Meta").join(n).is_file())
        .map(|n| format!("0_Meta/{n}"))
        .collect()
}

/// `0_Meta/`에 남은 구 규칙 사본 안내 — `elf update`가 옮기지 못한 경우(같은 이름이 `.elf/managed/`에도 있음)만 남음.
pub(crate) fn legacy_copies_message(root: &Path, leftovers: &[String]) -> String {
    if root.join(".elf").join("managed").join("EliRule.md").is_file() {
        format!(
            "old rule copies remain ({}) — the current rules are in .elf/managed/ and these copies are not read; move any project-specific edits into 0_Meta/ProjectRule.md, then delete them",
            leftovers.join(", ")
        )
    } else {
        format!(
            "rule files of the earlier layout remain ({}) — `elf update` moves them to .elf/managed/ (as delivered = replaced by the current version, edited = kept with a .elf-new beside them)",
            leftovers.join(", ")
        )
    }
}

/// 이번 갱신이 옮기지 않는 구 규칙 사본만 경고(옮길 것은 이동 줄이 알림).
fn warn_if_legacy_leftovers(root: &Path, moves: &[(String, String)], report: &mut UpdateReport) {
    let leftovers: Vec<String> = legacy_rule_copies(root).into_iter().filter(|l| !moves.iter().any(|(from, _)| from == l)).collect();
    if !leftovers.is_empty() {
        let msg = legacy_copies_message(root, &leftovers);
        report.warn(msg);
    }
}

fn move_file(root: &Path, from: &str, to: &str) -> io::Result<()> {
    let dst = root.join(to);
    if let Some(dir) = dst.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::rename(root.join(from), dst)
}

/// `.elf/` 없는 2.0~2.4.0 프로젝트에 `.elf/config.json`을 만듦(`elf init`과 같은 키: name, lang, preset, created).
/// 연구 계열 기본 preset = full(그때 generator 기본 = 모든 모듈).
fn write_adopted_config(root: &Path, lang: &str) -> io::Result<()> {
    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let created = chrono::Local::now().format("%Y-%m-%d").to_string();
    let v = serde_json::json!({ "name": name, "lang": lang, "preset": manifest::normalize_preset(None), "created": created });
    let mut text = serde_json::to_string_pretty(&v).expect("config serializes");
    text.push('\n');
    fs::create_dir_all(root.join(".elf"))?;
    fs::write(root.join(".elf").join("config.json"), text)
}

/// git 작업트리가 dirty면 경고 (차단하지 않음 — git 없는 프로젝트도 지원).
/// 발화 기준(S021 t15): **추적 파일 변경**(porcelain 비-`??` 줄)이 있을 때만 — 사용자
/// 미커밋 변경과 update 변경이 한 tree에 섞여 update만 선별 롤백할 수 없게 되는 실위험.
/// untracked-only는 제외(추적 managed 파일의 롤백이 사용자 파일과 분리 가능 — 과경고 방지).
/// 예외: 커밋이 전무한 unborn HEAD는 롤백 기준 자체가 없어 경고 유지("commit first"가 정확한 조언).
fn warn_if_git_dirty(root: &Path, report: &mut UpdateReport) {
    if !root.join(".git").exists() {
        return;
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output();
    let Ok(o) = out else { return };
    if !o.status.success() || o.stdout.is_empty() {
        return;
    }
    let stdout = String::from_utf8_lossy(&o.stdout);
    let tracked_changes = stdout.lines().any(|l| !l.starts_with("??"));
    // untracked-only일 때만 평가(지연) — 커밋 0개면 rev-parse HEAD 실패 = unborn
    let unborn_head = || {
        !std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "--verify", "--quiet", "HEAD"])
            .output()
            .map(|h| h.status.success())
            .unwrap_or(false)
    };
    if tracked_changes {
        report.warn(
            "git working tree is dirty — your uncommitted changes and this update's changes would mix in one tree, so the update alone could not be rolled back cleanly; commit or stash first"
                .to_string(),
        );
    } else if unborn_head() {
        report.warn(
            "no git commit yet — there is no baseline to roll this update back to; commit first"
                .to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "# >>> ELF managed >>>\nrule1\n# <<< ELF managed <<<\n";

    #[test]
    fn extract_block_at_top() {
        let text = format!("{BLOCK}\nuser/\n");
        let (s, e) = extract_block(&text).unwrap();
        assert_eq!(&text[s..e], BLOCK);
    }

    #[test]
    fn extract_block_mid_file_includes_full_lines() {
        let text = format!("user-top/\n{BLOCK}user-bottom/\n");
        let (s, e) = extract_block(&text).unwrap();
        assert_eq!(&text[s..e], BLOCK);
        assert_eq!(&text[..s], "user-top/\n");
        assert_eq!(&text[e..], "user-bottom/\n");
    }

    #[test]
    fn extract_block_missing_markers_is_none() {
        assert!(extract_block("just\nuser\nrules\n").is_none());
        assert!(extract_block(&format!("{MARKER_START}\nno end")).is_none());
    }

    #[test]
    fn extract_block_end_marker_without_trailing_newline() {
        let text = format!("# >>> ELF managed >>>\nx\n{MARKER_END}");
        let (s, e) = extract_block(&text).unwrap();
        assert_eq!(s, 0);
        assert_eq!(e, text.len());
    }
}
