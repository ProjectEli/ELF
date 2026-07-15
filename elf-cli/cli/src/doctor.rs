//! `elf doctor` — 환경+프로젝트 종합 진단 (P012 t05). **읽기전용**.
//!
//! ① CLI 버전·설치 receipt ② `.elf/` 무결성(stamp·version·baseline) ③ managed 파일 상태(status)
//! ④ git 저장소·훅. 네트워크 버전 체크는 하지 않음(오프라인-safe). 프로젝트 밖에서도 환경 진단만 수행.
//! receipt 탐지(axoupdater)는 main이 주입 → `run_doctor`는 결정론적(테스트 가능).

use std::fs;
use std::path::Path;

use crate::{autoread, manifest, status, tsa, update};

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

/// install receipt 존재 여부(axoupdater, 읽기전용·오프라인). main에서 호출해 `DoctorEnv` 구성.
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
        r.add(Health::Ok, "install receipt", "found — `elf self-update` works");
    } else {
        r.add(Health::Info, "install receipt", "absent — self-update unavailable (dev/manual build)");
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
            check_elf(&root, env, &mut r);
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

fn check_elf(root: &Path, env: &DoctorEnv, r: &mut DoctorReport) {
    let elf = root.join(".elf");
    match fs::read_to_string(elf.join("manifest.json")) {
        Ok(text) => match manifest::parse(&text) {
            Ok(_) => r.add(Health::Ok, ".elf stamp", "manifest.json parses"),
            Err(e) => r.add(Health::Warn, ".elf stamp", format!("manifest.json malformed: {e}")),
        },
        Err(e) => r.add(Health::Warn, ".elf stamp", format!("manifest.json unreadable: {e}")),
    }
    match fs::read_to_string(elf.join("version")) {
        Ok(v) => {
            let v = v.trim();
            if v == env.version {
                r.add(Health::Ok, ".elf version", format!("{v} (= CLI)"));
            } else {
                r.add(
                    Health::Warn,
                    ".elf version",
                    format!("{v} ≠ CLI {} — run `elf update`", env.version),
                );
            }
        }
        Err(_) => r.add(Health::Warn, ".elf version", "version stamp missing — run `elf update`"),
    }
    if elf.join("baseline").is_dir() {
        r.add(Health::Ok, ".elf baseline", "present (hybrid block diffing enabled)");
    } else {
        r.add(Health::Info, ".elf baseline", "absent (no hybrid files or pre-baseline project)");
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
            format!("{} pending, {} conflict(s) — run `elf status`", s.pending, s.conflicts),
        ),
        Err(e) => r.add(Health::Warn, "managed files", format!("status failed: {e}")),
    }
}

/// i18n 위상 advisory (P016 M4) — 비-ko 프로젝트의 companion 상태·operative 정본 안내.
/// 누락·편집의 actionable 분류는 `check_status`가 담당(중복 경고 회피) → 여기선 Info 위주 framing.
fn check_i18n(root: &Path, r: &mut DoctorReport) {
    let lang = update::read_config_lang(root);
    let p = manifest::lang_primary(&lang);
    if p.is_empty() || p == "ko" {
        return; // base(ko) 프로젝트 — companion 비적용
    }
    let Some(stamp) = fs::read_to_string(root.join(".elf").join("manifest.json"))
        .ok()
        .and_then(|t| manifest::parse(&t).ok())
    else {
        return; // stamp 문제는 check_elf가 이미 보고
    };
    let companions: Vec<String> = stamp
        .for_lang(&lang)
        .files
        .iter()
        .filter(|e| e.role == Some(manifest::Role::Companion))
        .map(|e| e.dest.clone())
        .collect();
    if companions.is_empty() {
        r.add(
            Health::Info,
            "i18n",
            format!("lang={lang}: no companions available — KO fallback (operative source = *.md)"),
        );
        return;
    }
    let present = companions.iter().filter(|d| root.join(d).is_file()).count();
    r.add(
        Health::Info,
        "i18n",
        format!(
            "lang={lang}: {present}/{} EN companion(s) present — informative; operative source = *.md (customize via ProjectRule)",
            companions.len()
        ),
    );
}

/// L2 진입 파일 연결 진단 (S021 t04·t06) — AGENTS.md(managed 상태는 check_status 소관)와
/// CLAUDE.md 포인터의 **연결 상태**를 봄: 포인터 부재 = Claude Code가 AGENTS.md 미로드.
/// stamp에 AGENTS.md entry가 없는 구버전 프로젝트는 조용히 skip(missing은 status가 안내).
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
    match fs::read_to_string(root.join("CLAUDE.md")) {
        Err(_) => r.add(
            Health::Warn,
            "agent entry",
            "CLAUDE.md missing — Claude Code will not load AGENTS.md; run `elf update` (creates the pointer)",
        ),
        Ok(c) if !c.contains("@AGENTS.md") => r.add(
            Health::Warn,
            "agent entry",
            "CLAUDE.md lacks a `@AGENTS.md` line — Claude Code will not load AGENTS.md; add the line (your other content can stay)",
        ),
        Ok(c) => {
            let lines = c.lines().count();
            if lines > 15 {
                r.add(
                    Health::Info,
                    "agent entry",
                    format!(
                        "CLAUDE.md loads @AGENTS.md but carries {lines} lines of extra content — check for duplication with AGENTS.md"
                    ),
                );
            } else {
                r.add(Health::Ok, "agent entry", "CLAUDE.md → @AGENTS.md pointer connected");
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
        .filter(|e| e.overlayable && e.role.is_none())
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
    // base가 overlay 비허용(구조·규약 파일 등)이거나 미지 대상인 `*.project.md` — 병행 로드되지 않음을 경고
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
/// 미제출 manifest, 엄밀 검증용 openssl 존재(드문 경로 — 없어도 축적·경량 검증은 자기완결).
/// autoread — default-on 기능의 반쪽 설치(기능 켬·훅 부재) 검사 (S031 t06 — CLAUDE.md 포인터 검사와 동렬).
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
    if missing.is_empty() {
        r.add(Health::Ok, "autoread", "enabled — Claude Code hooks installed");
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
            format!("{unstamped} unstamped manifest(s) — run `elf tsa stamp --backfill` when online"),
        );
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
