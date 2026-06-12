//! `elf update` — 프로젝트 ELF 파일 갱신 executor (t04, 1차 = 비대화형).
//!
//! 원칙: **무경고 데이터 손실 금지.**
//! - managed 편집본 → 덮어쓰지 않고 경고 + `<dest>.elf-new` 산출 (`--force`로만 덮어씀)
//! - hybrid → 마커블록만 교체, 사용자 영역 보존. 블록 편집/마커 부재 시 보수적 기본동작
//!   (interactive 질의·update_policy 저장은 2차 — S007 §4.1)
//! - seed/instance → 절대 미접근
//! - 블록 편집 감지 기준 = `.elf/baseline/<dest>` (init/update가 기록하는 배포본 사본)

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

pub fn run_update(root: &Path, opts: &UpdateOptions) -> Result<UpdateReport, UpdateError> {
    let stamp_path = root.join(".elf").join("manifest.json");
    if !stamp_path.is_file() {
        return Err(UpdateError::NotElfProject(root.to_path_buf()));
    }
    let stamp_text = fs::read_to_string(&stamp_path)?;
    let stamp = manifest::parse(&stamp_text).map_err(UpdateError::BadStamp)?;
    let new_m = manifest::embedded();

    let mut report = UpdateReport::default();
    warn_if_git_dirty(root, &mut report);

    // 현재 상태 수집 (new manifest의 dest들)
    let mut current: CurrentState = CurrentState::new();
    for e in &new_m.files {
        let p = root.join(&e.dest);
        let state = match fs::read(&p) {
            Ok(bytes) => Some(hash::sha256_lf(&bytes)),
            Err(_) => None,
        };
        current.insert(e.dest.clone(), state);
    }

    let actions = plan::plan_update(&new_m, &stamp, &current);

    for a in &actions {
        apply(root, a, opts, &new_m, &mut report)?;
    }

    // re-stamp는 무조건 (t02 재점검 §2: NoChange여도 stamp 해시는 구버전)
    if !opts.dry_run {
        fs::write(&stamp_path, embed::MANIFEST_JSON)?;
        fs::write(
            root.join(".elf").join("version"),
            format!("{}\n", embed::version()),
        )?;
        report.note(format!("re-stamped .elf/ to ELF {}", embed::version()));
    }

    Ok(report)
}

fn apply(
    root: &Path,
    action: &UpdateAction,
    opts: &UpdateOptions,
    new_m: &Manifest,
    report: &mut UpdateReport,
) -> Result<(), UpdateError> {
    match action {
        UpdateAction::NoChange { dest } => {
            report.note(format!("up-to-date: {dest}"));
        }
        UpdateAction::SkipSeed { dest } => {
            report.note(format!("seed (untouched): {dest}"));
        }
        UpdateAction::Obsolete { dest } => {
            report.warn(format!(
                "obsolete (no longer managed by ELF — left in place): {dest}"
            ));
        }
        UpdateAction::Overwrite { dest } | UpdateAction::CreateMissing { dest } => {
            if !opts.dry_run {
                let bytes = template_bytes(new_m, dest);
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
                    write_file(root, dest, template_bytes(new_m, dest))?;
                }
                report.changed += 1;
                report.note(format!("force-updated (user edits discarded): {dest}"));
            } else {
                report.conflicts += 1;
                if !opts.dry_run {
                    write_file(
                        root,
                        &format!("{dest}.elf-new"),
                        template_bytes(new_m, dest),
                    )?;
                }
                report.warn(format!(
                    "edited by user — kept; new version at {dest}.elf-new (use --force to overwrite)"
                ));
            }
        }
        UpdateAction::MergeBlock { dest } => {
            merge_hybrid(root, dest, opts, new_m, report)?;
        }
    }
    Ok(())
}

fn merge_hybrid(
    root: &Path,
    dest: &str,
    opts: &UpdateOptions,
    new_m: &Manifest,
    report: &mut UpdateReport,
) -> Result<(), UpdateError> {
    let template = std::str::from_utf8(template_bytes(new_m, dest))
        .expect("hybrid templates must be UTF-8");
    let new_block = block_of(template).expect("hybrid template must contain marker block");

    let current_path = root.join(dest);
    let current_bytes = fs::read(&current_path)?;
    let Ok(current) = std::str::from_utf8(&current_bytes) else {
        report.conflicts += 1;
        report.warn(format!("not UTF-8 — skipped: {dest}"));
        return Ok(());
    };

    match extract_block(current) {
        None => {
            // 마커 부재 (사용자가 삭제) — 보수적 기본 = 미변경 경고; --force = 상단 재삽입
            if opts.force {
                if !opts.dry_run {
                    let rebuilt = format!("{new_block}\n{current}");
                    fs::write(&current_path, &rebuilt)?;
                    write_baseline(root, dest, template.as_bytes())?;
                }
                report.changed += 1;
                report.note(format!("marker reinserted at top: {dest}"));
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
                report.warn(format!(
                    "{reason} — kept; new version at {dest}.elf-new (use --force to replace block)"
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

fn template_bytes<'a>(m: &'a Manifest, dest: &str) -> &'a [u8] {
    let e = m
        .files
        .iter()
        .find(|e| e.dest == dest)
        .expect("action dest must exist in manifest");
    let rel = e.src.strip_prefix("templates/").expect("src under templates/");
    embed::TEMPLATES
        .get_file(rel)
        .expect("src embedded (gated by tests)")
        .contents()
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

/// git 작업트리가 dirty면 경고 (차단하지 않음 — git 없는 프로젝트도 지원)
fn warn_if_git_dirty(root: &Path, report: &mut UpdateReport) {
    if !root.join(".git").exists() {
        return;
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output();
    if let Ok(o) = out
        && o.status.success()
        && !o.stdout.is_empty()
    {
        report.warn(
            "git working tree is dirty — commit or stash first for easy rollback".to_string(),
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
