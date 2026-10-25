//! `elf check files`의 관리 파일 상태 진단(구 이름 `elf status` — 숨김 별칭, t05). **읽기전용** — FS 무변경.
//! t02 planner(plan_update)를 그대로 재사용해 상태를 분류만 한다.
//! `--check` 게이트(발견 시 exit 4)는 main이 report 카운트로 결정.

use std::fs;
use std::path::Path;

use crate::embed;
use crate::hash;
use crate::manifest::{self, Manifest};
use crate::plan::{self, CurrentState, UpdateAction};
use crate::update::{self, UpdateError};

/// 줄 종류 — `elf check files`가 issue/warn/ok로 다시 분류할 때 사용(줄 내용은 별칭 출력과 동일).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Ok,
    Info,
    Warn,
    /// update가 해소할 항목
    Pending,
    /// 사용자 개입 필요 항목
    Conflict,
}

#[derive(Debug, Default)]
pub struct StatusReport {
    pub lines: Vec<String>,
    /// `lines`와 같은 길이 — 각 줄의 종류
    pub kinds: Vec<LineKind>,
    /// update가 해소할 항목 (outdated/missing)
    pub pending: usize,
    /// 사용자 개입 필요 항목 (edited/마커 부재)
    pub conflicts: usize,
    pub warnings: usize,
}

impl StatusReport {
    pub fn findings(&self) -> usize {
        self.pending + self.conflicts
    }

    fn push(&mut self, kind: LineKind, line: String) {
        match kind {
            LineKind::Warn => self.warnings += 1,
            LineKind::Pending => self.pending += 1,
            LineKind::Conflict => self.conflicts += 1,
            LineKind::Ok | LineKind::Info => {}
        }
        self.kinds.push(kind);
        self.lines.push(line);
    }

    fn warn(&mut self, line: String) {
        self.push(LineKind::Warn, format!("warn: {line}"));
    }
}

pub fn run_status(root: &Path) -> Result<StatusReport, UpdateError> {
    run_status_parts(root, true)
}

/// `with_version_tsa`=false → 버전 줄과 tsa 줄 생략(`elf check files`는 같은 항목을 `.elf` 기록, tsa 검사로 따로 보고).
pub fn run_status_parts(root: &Path, with_version_tsa: bool) -> Result<StatusReport, UpdateError> {
    let stamp_path = root.join(".elf").join("manifest.json");
    if !stamp_path.is_file() {
        return Err(UpdateError::NotElfProject(root.to_path_buf()));
    }
    let stamp_text = fs::read_to_string(&stamp_path)?;
    // 프로젝트 언어로 stamp, new 해석 — companion/variant를 lang에 맞게 필터 (P016 §9, update와 동일)
    let lang = update::read_config_lang(root);
    let stamp_full = manifest::parse(&stamp_text).map_err(UpdateError::BadStamp)?;

    let mut report = StatusReport::default();

    // 계보 해석 (S026) — update와 동일 규칙. 단 status는 읽기전용: 자동 복구 없음,
    // 모순도 중단 대신 경고 + stamp 시그니처(실물과 정합한 쪽) 기준으로 진단 계속.
    let kind = match update::resolve_kind(root, &stamp_full) {
        Ok((kind, update::KindSource::Config)) => {
            report.push(LineKind::Info, format!("preset: {kind}"));
            kind
        }
        Ok((kind, update::KindSource::RemovedGeneral)) => {
            report.warn(
                "preset: general was removed in ELF 2.27 — run `elf update` to convert this project to `minimal` (research lineage); diagnosing as research".into(),
            );
            kind
        }
        Ok((kind, update::KindSource::Inferred)) => {
            report.push(LineKind::Info, format!(
                "preset: {kind} (inferred from stamp — `elf update` records it to .elf/config.json)"
            ));
            kind
        }
        Err(UpdateError::PresetMismatch { config, stamp: sig }) => {
            report.warn(format!(
                "preset mismatch: .elf/config.json declares \"{config}\" but the stamp is a {sig} manifest — diagnosing as {sig}; fix \"preset\" in .elf/config.json (`elf update` refuses until then)"
            ));
            sig
        }
        Err(e) => return Err(e),
    };

    let stamp = stamp_full.for_lang(&lang);
    let new_m = manifest::embedded_kind(kind).for_lang(&lang);

    // 버전 줄: 프로젝트 stamp vs CLI
    let project_version = fs::read_to_string(root.join(".elf").join("version"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "(unknown)".into());
    if !with_version_tsa {
        // check files: 버전은 `.elf` 기록 검사가 보고
    } else if project_version == embed::version() {
        report.push(LineKind::Info, format!("ELF {project_version} (project = CLI)"));
    } else {
        report.push(
            LineKind::Warn,
            format!(
                "project ELF {project_version} ≠ CLI {} — run `elf update` to refresh stamp",
                embed::version()
            ),
        );
    }

    // tsa (opt-in — enabled일 때만 1줄; 상세는 `elf tsa status`/`elf check files`)
    if with_version_tsa && crate::tsa::is_enabled(root) {
        let (manifests, stamped, unstamped) = crate::tsa::evidence_counts(root);
        if unstamped == 0 {
            report.push(LineKind::Info, format!("tsa: enabled ({manifests} manifest(s), {stamped} stamped)"));
        } else {
            report.warn(format!(
                "tsa: {unstamped} unstamped manifest(s) — run `elf tsa --all` when online"
            ));
        }
    }

    let mut current: CurrentState = CurrentState::new();
    for e in &new_m.files {
        let state = fs::read(root.join(&e.dest)).ok().map(|b| hash::sha256_lf(&b));
        current.insert(e.dest.clone(), state);
    }

    for action in plan::plan_update(&new_m, &stamp, &current) {
        classify(root, &action, &new_m, &mut report);
    }

    Ok(report)
}

fn classify(root: &Path, action: &UpdateAction, new_m: &Manifest, report: &mut StatusReport) {
    match action {
        UpdateAction::NoChange { dest } => report.push(LineKind::Ok, format!("ok: {dest}")),
        UpdateAction::SkipUser { dest } => report.push(LineKind::Info, format!("user: {dest}")),
        UpdateAction::KeepPointer { dest } => report.push(
            LineKind::Info,
            format!("pointer (user content, kept): {dest} (`elf check files` checks @AGENTS.md)"),
        ),
        UpdateAction::Overwrite { dest } => {
            report.push(LineKind::Pending, format!("outdated: {dest} (run `elf update`)"));
        }
        UpdateAction::CreateMissing { dest } => {
            report.push(LineKind::Pending, format!("missing: {dest} (run `elf update`)"));
        }
        UpdateAction::Conflict { dest } => {
            report.push(
                LineKind::Conflict,
                format!("edited: {dest} (kept on update; `elf update --force` to overwrite)"),
            );
        }
        UpdateAction::Obsolete { dest } => {
            report.push(LineKind::Warn, format!("obsolete: {dest} (no longer managed — left in place)"));
        }
        UpdateAction::MergeBlock { dest } => classify_hybrid(root, dest, new_m, report),
    }
}

/// hybrid는 블록 단위로 세분: 블록==새 정본→ok / ==baseline→outdated / 그 외→edited / 마커 부재→edited
fn classify_hybrid(root: &Path, dest: &str, new_m: &Manifest, report: &mut StatusReport) {
    // 내장 데이터 불일치 = 손상 의심 — 진단 도구이므로 panic 대신 경고 후 계속 (t09 정책)
    let template = new_m
        .files
        .iter()
        .find(|e| e.dest == *dest)
        .and_then(|e| e.src.strip_prefix("templates/"))
        .and_then(|rel| embed::TEMPLATES.get_file(rel))
        .and_then(|f| f.contents_utf8());
    let Some(template) = template else {
        report.warn(format!(
            "internal: embedded template unavailable for {dest} — reinstall (`elf self update`)"
        ));
        return;
    };
    let Some(new_block) = update::block_of(template) else {
        report.warn(format!("internal: hybrid template missing marker block: {dest}"));
        return;
    };

    let Ok(current) = fs::read_to_string(root.join(dest)) else {
        report.push(LineKind::Conflict, format!("unreadable (not UTF-8?): {dest}"));
        return;
    };

    match update::extract_block(&current) {
        None => {
            report.push(
                LineKind::Conflict,
                format!("marker missing: {dest} (`elf update --force` reinserts at top)"),
            );
        }
        Some((s, e)) => {
            let block = &current[s..e];
            if block == new_block {
                report.push(LineKind::Ok, format!("ok (block current, user area free): {dest}"));
            } else if update::read_baseline_block(root, dest).as_deref() == Some(block) {
                report.push(LineKind::Pending, format!("block outdated: {dest} (run `elf update`)"));
            } else {
                report.push(
                    LineKind::Conflict,
                    format!("block edited: {dest} (kept on update; `elf update --force` to replace)"),
                );
            }
        }
    }
}
