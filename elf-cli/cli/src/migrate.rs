//! `elf migrate` — legacy(`0_Meta/`·`templates/`) → managed(`.elf/managed/`) 레이아웃 이전 (S024/B).
//!
//! **opt-in 전용**: `elf update`가 자동 수행하지 않는다 — 사용자 가시 파일의 이동은 명시 호출로만.
//! 원칙: 이동 전 전체 계획 수립·충돌 검증(하나라도 충돌 시 무이동 중단), git 추적 변경 존재 시
//! 중단(이전만 선별 롤백 불가 — update dirty 경고와 동일 근거의 차단판), 사용자 소유물 내
//! 구경로 참조는 **보고만**(재작성하지 않음 — 사용자 파일 미접근 원칙).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::manifest::{self, Layout};
use crate::update;

pub struct MigrateOptions {
    pub dry_run: bool,
}

#[derive(Debug)]
pub enum MigrateError {
    NotElfProject(PathBuf),
    /// 이미 managed 레이아웃 — 할 일 없음 (정상 종료용 신호)
    AlreadyManaged,
    /// git 추적 파일에 미커밋 변경 — 이전분만 선별 롤백 불가하므로 중단
    DirtyTree,
    BadStamp(String),
    /// 이동 대상과 목적지가 동시에 존재 — 수동 해소 필요
    TargetExists(String),
    Io(io::Error),
}

impl std::fmt::Display for MigrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrateError::NotElfProject(p) => write!(
                f,
                "not an ELF project: {} (no .elf/manifest.json — run inside a project)",
                p.display()
            ),
            MigrateError::AlreadyManaged => {
                write!(f, "already on the managed layout (.elf/managed/) — nothing to do")
            }
            MigrateError::DirtyTree => write!(
                f,
                "git working tree has uncommitted tracked changes — commit or stash first, so the migration alone can be rolled back cleanly"
            ),
            MigrateError::BadStamp(s) => write!(f, "stamp(.elf/manifest.json): {s}"),
            MigrateError::TargetExists(d) => write!(
                f,
                "both old and new locations exist for {d} — resolve manually (keep one), then re-run"
            ),
            MigrateError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl From<io::Error> for MigrateError {
    fn from(e: io::Error) -> Self {
        MigrateError::Io(e)
    }
}

#[derive(Debug, Default)]
pub struct MigrateReport {
    pub lines: Vec<String>,
    pub moved: usize,
    /// legacy 파일 부재로 건너뜀 (`elf update`가 신경로에 재생성)
    pub skipped: usize,
    /// 사용자 소유물 내 구경로 참조 (보고만 — 재작성은 사용자 소관)
    pub refs: Vec<String>,
}

/// legacy → managed 이전. dry_run이면 계획만 출력하고 무변경.
pub fn run_migrate(root: &Path, opts: &MigrateOptions) -> Result<MigrateReport, MigrateError> {
    let stamp_path = root.join(".elf").join("manifest.json");
    if !stamp_path.is_file() {
        return Err(MigrateError::NotElfProject(root.to_path_buf()));
    }
    if update::read_config_layout(root) == Layout::Managed {
        return Err(MigrateError::AlreadyManaged);
    }
    if !opts.dry_run && git_tracked_dirty(root) {
        return Err(MigrateError::DirtyTree);
    }

    let lang = update::read_config_lang(root);
    let stamp = manifest::parse(&fs::read_to_string(&stamp_path)?)
        .map_err(MigrateError::BadStamp)?
        .for_lang(&lang);

    // 1) 이동 계획 — managed tier 중 레이아웃 간 dest가 달라지는 항목만. 충돌은 이동 전 전수 검증.
    let mut moves: Vec<(String, String)> = Vec::new();
    let mut report = MigrateReport::default();
    for e in &stamp.files {
        let canon = manifest::dest_to_managed(&e.dest, e.tier);
        let legacy = manifest::dest_to_legacy(&canon);
        if canon == legacy {
            continue; // root 파일·seed 등 — 이동 무관
        }
        let from = root.join(&legacy);
        let to = root.join(&canon);
        match (from.is_file(), to.is_file()) {
            (true, true) => return Err(MigrateError::TargetExists(canon)),
            (true, false) => moves.push((legacy.clone(), canon.clone())),
            (false, true) => report.lines.push(format!("already moved: {canon}")),
            (false, false) => {
                report.skipped += 1;
                report.lines.push(format!(
                    "skipped (missing): {legacy} — `elf update` will create {canon}"
                ));
            }
        }
        // 미병합 `.elf-new` 병기본도 함께 이동 (분리 시 짝 소실)
        let elf_new = format!("{legacy}.elf-new");
        if root.join(&elf_new).is_file() {
            moves.push((elf_new, format!("{canon}.elf-new")));
        }
    }

    for (from, to) in &moves {
        report
            .lines
            .push(format!("{}: {from} -> {to}", if opts.dry_run { "would move" } else { "move" }));
    }

    if opts.dry_run {
        report.lines.push("dry-run — nothing moved".into());
        return Ok(report);
    }

    // 2) 실행 — 계획 검증 후 일괄 이동
    for (from, to) in &moves {
        let to_path = root.join(to);
        if let Some(dir) = to_path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::rename(root.join(from), to_path)?;
        report.moved += 1;
    }

    // 3) config layout 기록 (기존 필드 보존)
    set_config_layout(root)?;
    report.lines.push("layout: managed (.elf/config.json)".into());

    // 4) 사용자 소유물 내 구경로 참조 스캔 — 보고만. `.elf/`·`.git/`·Archive 제외.
    let needles: Vec<&str> = moves
        .iter()
        .map(|(from, _)| from.as_str())
        .filter(|f| !f.ends_with(".elf-new"))
        .collect();
    if !needles.is_empty() {
        scan_refs(root, root, &needles, &mut report.refs);
        report.refs.truncate(50);
        for r in &report.refs {
            report
                .lines
                .push(format!("old-path reference (left as-is, update manually): {r}"));
        }
    }

    Ok(report)
}

fn set_config_layout(root: &Path) -> io::Result<()> {
    let p = root.join(".elf").join("config.json");
    let mut v: serde_json::Value = fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    v["layout"] = serde_json::Value::String("managed".into());
    let mut text = serde_json::to_string_pretty(&v).expect("config serializes");
    text.push('\n');
    fs::write(p, text)
}

/// git 추적 파일 변경 여부 (porcelain 비-`??` 줄). git 부재·오류 = false(차단하지 않음).
fn git_tracked_dirty(root: &Path) -> bool {
    if !root.join(".git").exists() {
        return false;
    }
    let Ok(o) = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
    else {
        return false;
    };
    o.status.success()
        && String::from_utf8_lossy(&o.stdout)
            .lines()
            .any(|l| !l.starts_with("??"))
}

/// `.md` 파일에서 구경로 문자열 검색 (재귀). 제외: `.elf`·`.git`·`Archive`·`target`·`node_modules`.
fn scan_refs(root: &Path, dir: &Path, needles: &[&str], out: &mut Vec<String>) {
    if out.len() >= 50 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if matches!(name.as_str(), ".elf" | ".git" | "target" | "node_modules")
                || name.contains("Archive")
            {
                continue;
            }
            scan_refs(root, &p, needles, out);
        } else if name.ends_with(".md") {
            let Ok(text) = fs::read_to_string(&p) else { continue };
            for n in needles {
                if text.contains(n) {
                    let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                    out.push(format!("{}: {n}", rel.replace('\\', "/")));
                    if out.len() >= 50 {
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_refs_finds_and_excludes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log")).unwrap();
        fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
        fs::create_dir_all(root.join(".elf")).unwrap();
        fs::write(root.join("2_Log/S001_log.md"), "see 0_Meta/EliRule.md").unwrap();
        fs::write(root.join("2_Log/Archive/S000_log.md"), "see 0_Meta/EliRule.md").unwrap();
        fs::write(root.join(".elf/x.md"), "see 0_Meta/EliRule.md").unwrap();
        fs::write(root.join("note.txt"), "see 0_Meta/EliRule.md").unwrap(); // .md 아님
        let mut out = Vec::new();
        scan_refs(root, root, &["0_Meta/EliRule.md"], &mut out);
        assert_eq!(out, vec!["2_Log/S001_log.md: 0_Meta/EliRule.md".to_string()]);
    }
}
