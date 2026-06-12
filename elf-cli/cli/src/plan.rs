//! 순수 planner — FS 무부작용. 입력(manifest·stamp·현재 해시) → action 목록만 산출.
//! 데이터-손실 위험 로직을 전부 여기에 모아 전수 테스트한다 (S007 t02·§4 행렬).
//!
//! 실행(FS 반영)은 init.rs(t03)/update.rs(t04)가 담당. 이 모듈은 std::fs를 import하지 않는다.

use std::collections::{BTreeMap, BTreeSet};

use crate::manifest::{Manifest, Tier};

/// `elf init` 계획: manifest 전 항목을 배치 (대상 디렉토리 신규 전제 — 존재 거부는 executor 책임).
#[derive(Debug, PartialEq, Eq)]
pub struct InitAction {
    pub src: String,
    pub dest: String,
    pub tier: Tier,
}

pub fn plan_init(m: &Manifest) -> Vec<InitAction> {
    m.files
        .iter()
        .map(|e| InitAction {
            src: e.src.clone(),
            dest: e.dest.clone(),
            tier: e.tier,
        })
        .collect()
}

/// 빈 폴더 보존 방식: 일반 폴더 = .gitkeep, 원본 데이터 폴더 = `*`+`!.gitignore` (git 추적 제외)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeepFile {
    GitKeep,
    RawGitignore,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DirPlan {
    pub path: String,
    pub keep: KeepFile,
}

/// preset 이름 → 폴더 계획 (core + preset의 모듈들). 데이터 출처 = manifest.dirs (단일소스).
pub fn plan_dirs(m: &Manifest, preset: &str) -> Result<Vec<DirPlan>, String> {
    let module_keys = m.dirs.presets.get(preset).ok_or_else(|| {
        format!(
            "unknown preset: {preset} (valid: {})",
            m.dirs.presets.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    })?;
    plan_dirs_from_modules(m, &module_keys.iter().map(String::as_str).collect::<Vec<_>>())
}

/// 모듈 키 목록 → 폴더 계획 (custom 선택용; preset도 내부적으로 이 경로 사용)
pub fn plan_dirs_from_modules(m: &Manifest, modules: &[&str]) -> Result<Vec<DirPlan>, String> {
    let mut paths: Vec<&String> = m.dirs.core.iter().collect();
    for key in modules {
        let dirs = m
            .dirs
            .modules
            .get(*key)
            .ok_or_else(|| {
                format!(
                    "unknown module: {key} (valid: {})",
                    m.dirs.modules.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            })?;
        paths.extend(dirs.iter());
    }
    let raw: BTreeSet<&str> = m.dirs.raw_data.iter().map(String::as_str).collect();
    Ok(paths
        .into_iter()
        .map(|p| DirPlan {
            path: p.clone(),
            keep: if raw.contains(p.as_str()) {
                KeepFile::RawGitignore
            } else {
                KeepFile::GitKeep
            },
        })
        .collect())
}

/// dest(프로젝트 루트 상대) → 현재 파일의 sha256_lf. 항목 부재 또는 None = 파일 없음.
pub type CurrentState = BTreeMap<String, Option<String>>;

/// `elf update` 계획 — tier × 상태 행렬 (S007 §4).
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateAction {
    /// 이미 최신 (current == new 정본)
    NoChange { dest: String },
    /// managed 미편집(current == stamp) + 정본 갱신 → 덮어쓰기
    Overwrite { dest: String },
    /// dest 부재 → 정본에서 재생성 (managed/hybrid)
    CreateMissing { dest: String },
    /// managed 사용자 편집(current ∉ {stamp, new}) → 무경고 덮어쓰기 금지: 경고 + .elf-new (t04)
    Conflict { dest: String },
    /// seed tier — update 절대 미접근 (부재여도 미접근)
    SkipSeed { dest: String },
    /// hybrid — 마커블록만 교체(마커 부재/블록 편집의 세부 분기는 t04가 내용 기반 처리)
    MergeBlock { dest: String },
    /// 구버전 stamp에만 있고 새 manifest에 없음 — 자동 삭제하지 않고 경고만
    Obsolete { dest: String },
}

pub fn plan_update(new_m: &Manifest, stamp: &Manifest, current: &CurrentState) -> Vec<UpdateAction> {
    let stamp_sha_by_dest: BTreeMap<&str, &str> = stamp
        .files
        .iter()
        .map(|e| (e.dest.as_str(), e.sha256.as_str()))
        .collect();

    let mut out = Vec::new();

    for e in &new_m.files {
        let cur = current.get(e.dest.as_str()).cloned().flatten();
        let action = match e.tier {
            Tier::Seed => UpdateAction::SkipSeed { dest: e.dest.clone() },
            Tier::Hybrid => match cur {
                None => UpdateAction::CreateMissing { dest: e.dest.clone() },
                // 파일 전체가 새 정본과 동일 = 블록 최신 + 사용자 영역 무변경 → 머지 불필요
                Some(c) if c == e.sha256 => UpdateAction::NoChange { dest: e.dest.clone() },
                Some(_) => UpdateAction::MergeBlock { dest: e.dest.clone() },
            },
            Tier::Managed => {
                let stamp_sha = stamp_sha_by_dest.get(e.dest.as_str()).copied();
                match cur {
                    // 부재 → 재생성
                    None => UpdateAction::CreateMissing { dest: e.dest.clone() },
                    // 이미 새 정본과 동일(사용자가 선반영했어도) → 변경 없음
                    Some(c) if c == e.sha256 => UpdateAction::NoChange { dest: e.dest.clone() },
                    // 배포 시점 그대로(미편집) → 안전한 덮어쓰기
                    Some(c) if Some(c.as_str()) == stamp_sha => {
                        UpdateAction::Overwrite { dest: e.dest.clone() }
                    }
                    // 그 외 = 사용자 편집본 → 보호
                    Some(_) => UpdateAction::Conflict { dest: e.dest.clone() },
                }
            }
        };
        out.push(action);
    }

    // 새 manifest에서 사라진 항목: 삭제는 자동 수행하지 않음 (P009 §7 삭제/원자성).
    // seed는 제외 — 사용자 소유 콘텐츠라 "obsolete" 경고가 사용자 콘텐츠 삭제를 유도할 위험
    // (ELF가 seeding을 중단했을 뿐, 사용자가 할 일 없음).
    let new_dests: BTreeSet<&str> = new_m.files.iter().map(|e| e.dest.as_str()).collect();
    for e in &stamp.files {
        if e.tier != Tier::Seed && !new_dests.contains(e.dest.as_str()) {
            out.push(UpdateAction::Obsolete { dest: e.dest.clone() });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Entry, Manifest};

    fn entry(src: &str, dest: &str, tier: Tier, sha: &str) -> Entry {
        Entry {
            src: src.into(),
            dest: dest.into(),
            tier,
            sha256: sha.into(),
        }
    }

    fn manifest(files: Vec<Entry>) -> Manifest {
        Manifest {
            schema: crate::manifest::SCHEMA.into(),
            generated: String::new(),
            note: String::new(),
            files,
            dirs: Default::default(),
        }
    }

    fn cur(pairs: &[(&str, Option<&str>)]) -> CurrentState {
        pairs
            .iter()
            .map(|(d, h)| ((*d).to_string(), h.map(|s| s.to_string())))
            .collect()
    }

    /// 표준 시나리오: stamp(구버전 NEW=old) → new(정본 NEW=new)
    fn managed_pair() -> (Manifest, Manifest) {
        let new_m = manifest(vec![entry("templates/m.md", "0_Meta/m.md", Tier::Managed, "new")]);
        let stamp = manifest(vec![entry("templates/m.md", "0_Meta/m.md", Tier::Managed, "old")]);
        (new_m, stamp)
    }

    // ── managed × 상태 행렬 ─────────────────────────────

    #[test]
    fn managed_unedited_and_template_updated_overwrites() {
        let (new_m, stamp) = managed_pair();
        let c = cur(&[("0_Meta/m.md", Some("old"))]); // current == stamp
        assert_eq!(
            plan_update(&new_m, &stamp, &c),
            vec![UpdateAction::Overwrite { dest: "0_Meta/m.md".into() }]
        );
    }

    #[test]
    fn managed_already_latest_is_nochange() {
        let (new_m, stamp) = managed_pair();
        let c = cur(&[("0_Meta/m.md", Some("new"))]); // current == new 정본
        assert_eq!(
            plan_update(&new_m, &stamp, &c),
            vec![UpdateAction::NoChange { dest: "0_Meta/m.md".into() }]
        );
    }

    #[test]
    fn managed_user_edited_is_conflict() {
        let (new_m, stamp) = managed_pair();
        let c = cur(&[("0_Meta/m.md", Some("user-edit"))]); // ∉ {old, new}
        assert_eq!(
            plan_update(&new_m, &stamp, &c),
            vec![UpdateAction::Conflict { dest: "0_Meta/m.md".into() }]
        );
    }

    #[test]
    fn managed_missing_is_create() {
        let (new_m, stamp) = managed_pair();
        for c in [cur(&[("0_Meta/m.md", None)]), cur(&[])] {
            assert_eq!(
                plan_update(&new_m, &stamp, &c),
                vec![UpdateAction::CreateMissing { dest: "0_Meta/m.md".into() }]
            );
        }
    }

    #[test]
    fn managed_unedited_and_template_unchanged_is_nochange() {
        // stamp == new == current → NoChange (current==new 분기가 우선)
        let new_m = manifest(vec![entry("templates/m.md", "0_Meta/m.md", Tier::Managed, "same")]);
        let stamp = manifest(vec![entry("templates/m.md", "0_Meta/m.md", Tier::Managed, "same")]);
        let c = cur(&[("0_Meta/m.md", Some("same"))]);
        assert_eq!(
            plan_update(&new_m, &stamp, &c),
            vec![UpdateAction::NoChange { dest: "0_Meta/m.md".into() }]
        );
    }

    // ── manifest 신규 항목 (stamp에 없음) ───────────────

    #[test]
    fn managed_new_in_manifest_missing_creates() {
        let new_m = manifest(vec![entry("templates/n.md", "0_Meta/n.md", Tier::Managed, "new")]);
        let stamp = manifest(vec![]);
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[])),
            vec![UpdateAction::CreateMissing { dest: "0_Meta/n.md".into() }]
        );
    }

    #[test]
    fn managed_new_in_manifest_but_user_file_exists_is_conflict() {
        // 사용자 파일이 선점한 경로에 ELF가 새 managed 도입 → 보수적으로 Conflict
        let new_m = manifest(vec![entry("templates/n.md", "0_Meta/n.md", Tier::Managed, "new")]);
        let stamp = manifest(vec![]);
        let c = cur(&[("0_Meta/n.md", Some("user-file"))]);
        assert_eq!(
            plan_update(&new_m, &stamp, &c),
            vec![UpdateAction::Conflict { dest: "0_Meta/n.md".into() }]
        );
    }

    // ── seed / hybrid ──────────────────────────────────

    #[test]
    fn seed_is_skipped_in_all_states() {
        let new_m = manifest(vec![entry("templates/s.md", "0_Meta/s.md", Tier::Seed, "new")]);
        let stamp = manifest(vec![entry("templates/s.md", "0_Meta/s.md", Tier::Seed, "old")]);
        for c in [
            cur(&[("0_Meta/s.md", Some("old"))]),
            cur(&[("0_Meta/s.md", Some("user"))]),
            cur(&[("0_Meta/s.md", None)]), // 부재여도 미접근 (의도적 삭제 존중)
        ] {
            assert_eq!(
                plan_update(&new_m, &stamp, &c),
                vec![UpdateAction::SkipSeed { dest: "0_Meta/s.md".into() }]
            );
        }
    }

    #[test]
    fn hybrid_existing_merges_missing_creates_identical_skips() {
        let new_m = manifest(vec![entry("templates/root/.gitignore", ".gitignore", Tier::Hybrid, "new")]);
        let stamp = manifest(vec![entry("templates/root/.gitignore", ".gitignore", Tier::Hybrid, "old")]);
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[(".gitignore", Some("whatever"))])),
            vec![UpdateAction::MergeBlock { dest: ".gitignore".into() }]
        );
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[])),
            vec![UpdateAction::CreateMissing { dest: ".gitignore".into() }]
        );
        // 정본과 바이트 동일 → 머지 불필요 (idempotency)
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[(".gitignore", Some("new"))])),
            vec![UpdateAction::NoChange { dest: ".gitignore".into() }]
        );
    }

    // ── obsolete ───────────────────────────────────────

    #[test]
    fn stamp_only_entry_is_obsolete_not_deleted() {
        let new_m = manifest(vec![]);
        let stamp = manifest(vec![entry("templates/gone.md", "0_Meta/gone.md", Tier::Managed, "old")]);
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[("0_Meta/gone.md", Some("old"))])),
            vec![UpdateAction::Obsolete { dest: "0_Meta/gone.md".into() }]
        );
    }

    #[test]
    fn stamp_only_seed_is_not_flagged_obsolete() {
        // seed = 사용자 소유 — ELF가 seeding을 멈춰도 사용자에게 경고할 일이 아님
        let new_m = manifest(vec![]);
        let stamp = manifest(vec![entry("templates/p.md", "0_Meta/ProjectRule.md", Tier::Seed, "old")]);
        assert_eq!(
            plan_update(&new_m, &stamp, &cur(&[("0_Meta/ProjectRule.md", Some("user"))])),
            vec![]
        );
    }

    // ── init ───────────────────────────────────────────

    #[test]
    fn plan_init_covers_all_files_in_manifest_order() {
        let m = manifest(vec![
            entry("templates/a.md", "0_Meta/a.md", Tier::Managed, "x"),
            entry("templates/b.md", "0_Meta/b.md", Tier::Seed, "y"),
        ]);
        let plan = plan_init(&m);
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].dest, "0_Meta/a.md");
        assert_eq!(plan[1].tier, Tier::Seed);
    }

    // ── dirs (scaffold 데이터화) ───────────────────────

    fn manifest_with_dirs() -> Manifest {
        let mut m = manifest(vec![]);
        m.dirs.core = vec!["0_Meta".into(), "2_Log".into()];
        m.dirs.modules.insert("exp".into(), vec!["6_Exp/Raw".into(), "6_Exp/Viz".into()]);
        m.dirs.modules.insert("sw".into(), vec!["5_SW".into()]);
        m.dirs.presets.insert("full".into(), vec!["exp".into(), "sw".into()]);
        m.dirs.presets.insert("minimal".into(), vec![]);
        m.dirs.raw_data = vec!["6_Exp/Raw".into()];
        m
    }

    #[test]
    fn preset_resolves_core_plus_modules_with_raw_marking() {
        let m = manifest_with_dirs();
        let plan = plan_dirs(&m, "full").unwrap();
        let paths: Vec<&str> = plan.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(paths, vec!["0_Meta", "2_Log", "6_Exp/Raw", "6_Exp/Viz", "5_SW"]);
        assert_eq!(
            plan.iter().find(|d| d.path == "6_Exp/Raw").unwrap().keep,
            KeepFile::RawGitignore
        );
        assert_eq!(
            plan.iter().find(|d| d.path == "0_Meta").unwrap().keep,
            KeepFile::GitKeep
        );
    }

    #[test]
    fn minimal_preset_is_core_only() {
        let plan = plan_dirs(&manifest_with_dirs(), "minimal").unwrap();
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn unknown_preset_and_module_error_with_valid_list() {
        let m = manifest_with_dirs();
        let e = plan_dirs(&m, "wild").unwrap_err();
        assert!(e.contains("full") && e.contains("minimal"));
        let e2 = plan_dirs_from_modules(&m, &["nope"]).unwrap_err();
        assert!(e2.contains("exp") && e2.contains("sw"));
    }

    #[test]
    fn custom_module_subset_works() {
        let plan = plan_dirs_from_modules(&manifest_with_dirs(), &["sw"]).unwrap();
        let paths: Vec<&str> = plan.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(paths, vec!["0_Meta", "2_Log", "5_SW"]);
    }

    /// instance(=manifest 미등재) 파일은 planner 출력에 절대 등장하지 않음 — 미접근 보장
    #[test]
    fn unmanaged_files_never_appear_in_plan() {
        let (new_m, stamp) = managed_pair();
        let c = cur(&[("0_Meta/m.md", Some("old")), ("2_Log/S001_log.md", Some("zzz"))]);
        let plan = plan_update(&new_m, &stamp, &c);
        assert!(plan.iter().all(|a| !format!("{a:?}").contains("S001")));
        assert_eq!(plan.len(), 1);
    }
}
