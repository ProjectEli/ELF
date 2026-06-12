//! `elf status` — 프로젝트 ELF 파일 상태 진단 (t05). **읽기전용** — FS 무변경.
//! t02 planner(plan_update)를 그대로 재사용해 상태를 분류만 한다.
//! `--check` 게이트(발견 시 exit 4)는 main이 report 카운트로 결정.

use std::fs;
use std::path::Path;

use crate::embed;
use crate::hash;
use crate::manifest::{self, Manifest};
use crate::plan::{self, CurrentState, UpdateAction};
use crate::update::{self, UpdateError};

#[derive(Debug, Default)]
pub struct StatusReport {
    pub lines: Vec<String>,
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

    fn warn(&mut self, line: String) {
        self.warnings += 1;
        self.lines.push(format!("warn: {line}"));
    }
}

pub fn run_status(root: &Path) -> Result<StatusReport, UpdateError> {
    let stamp_path = root.join(".elf").join("manifest.json");
    if !stamp_path.is_file() {
        return Err(UpdateError::NotElfProject(root.to_path_buf()));
    }
    let stamp_text = fs::read_to_string(&stamp_path)?;
    let stamp = manifest::parse(&stamp_text).map_err(UpdateError::BadStamp)?;
    let new_m = manifest::embedded();

    let mut report = StatusReport::default();

    // 버전 줄: 프로젝트 stamp vs CLI
    let project_version = fs::read_to_string(root.join(".elf").join("version"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "(unknown)".into());
    if project_version == embed::version() {
        report.lines.push(format!("ELF {project_version} (project = CLI)"));
    } else {
        report.warnings += 1;
        report.lines.push(format!(
            "project ELF {project_version} ≠ CLI {} — run `elf update` to refresh stamp",
            embed::version()
        ));
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
        UpdateAction::NoChange { dest } => report.lines.push(format!("ok: {dest}")),
        UpdateAction::SkipSeed { dest } => report.lines.push(format!("seed: {dest}")),
        UpdateAction::Overwrite { dest } => {
            report.pending += 1;
            report.lines.push(format!("outdated: {dest} (run `elf update`)"));
        }
        UpdateAction::CreateMissing { dest } => {
            report.pending += 1;
            report.lines.push(format!("missing: {dest} (run `elf update`)"));
        }
        UpdateAction::Conflict { dest } => {
            report.conflicts += 1;
            report.lines.push(format!(
                "edited: {dest} (kept on update; `elf update --force` to overwrite)"
            ));
        }
        UpdateAction::Obsolete { dest } => {
            report.warnings += 1;
            report.lines.push(format!("obsolete: {dest} (no longer managed — left in place)"));
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
            "internal: embedded template unavailable for {dest} — reinstall (`elf self-update`)"
        ));
        return;
    };
    let Some(new_block) = update::block_of(template) else {
        report.warn(format!("internal: hybrid template missing marker block: {dest}"));
        return;
    };

    let Ok(current) = fs::read_to_string(root.join(dest)) else {
        report.conflicts += 1;
        report.lines.push(format!("unreadable (not UTF-8?): {dest}"));
        return;
    };

    match update::extract_block(&current) {
        None => {
            report.conflicts += 1;
            report.lines.push(format!(
                "marker missing: {dest} (`elf update --force` reinserts at top)"
            ));
        }
        Some((s, e)) => {
            let block = &current[s..e];
            if block == new_block {
                report.lines.push(format!("ok (block current, user area free): {dest}"));
            } else if update::read_baseline_block(root, dest).as_deref() == Some(block) {
                report.pending += 1;
                report.lines.push(format!("block outdated: {dest} (run `elf update`)"));
            } else {
                report.conflicts += 1;
                report.lines.push(format!(
                    "block edited: {dest} (kept on update; `elf update --force` to replace)"
                ));
            }
        }
    }
}
