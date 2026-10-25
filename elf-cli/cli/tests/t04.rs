//! t04 FS 통합: `elf update`의 무손실 보장을 tempdir 실트리에서 검증.
//! 시나리오 구성: init으로 실프로젝트 생성 → stamp/파일/baseline 조작으로 상태 연출 → update.
//!
//! 파일명이 update.rs가 아닌 이유: 테스트 바이너리명이 `update-<hash>.exe`가 되면
//! Windows UAC installer detection(update/setup/install/patch 키워드)이 권한 상승을
//! 요구해 실행 자체가 거부됨(os error 740). t06 self-update 바이너리 작명 시에도 동일 주의.

use elf_cli::init::{run_init, InitOptions};
use elf_cli::update::{run_update, UpdateOptions, MARKER_END, MARKER_START};
use elf_cli::{embed, hash, manifest};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn new_project(tmp: &Path, name: &str) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: name.into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-06-12".into(),
        },
    )
    .unwrap()
}

fn plain() -> UpdateOptions {
    UpdateOptions { dry_run: false, force: false }
}

/// stamp의 특정 dest 항목 sha256을 가짜 구버전 해시로 바꿔 "정본이 그 후 갱신됨"을 연출
fn tamper_stamp_sha(root: &Path, dest: &str, fake_sha: &str) {
    let p = root.join(".elf/manifest.json");
    let m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    let mut m = m;
    for f in m["files"].as_array_mut().unwrap() {
        if f["dest"] == dest {
            f["sha256"] = serde_json::Value::String(fake_sha.into());
        }
    }
    fs::write(&p, serde_json::to_string_pretty(&m).unwrap()).unwrap();
}

// ── 행렬: managed ───────────────────────────────────────────

#[test]
fn fresh_project_update_is_idempotent() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let before = fs::read(root.join(".elf/managed/EliRule.md")).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.changed, 0, "fresh tree must be all up-to-date: {:?}", report.lines);
    assert_eq!(report.conflicts, 0);
    assert_eq!(fs::read(root.join(".elf/managed/EliRule.md")).unwrap(), before);
    assert!(
        !report.lines.iter().any(|l| l.contains("pre-2.15 legacy rule files remain")),
        "clean 프로젝트는 잔재 안내 없음: {:?}",
        report.lines
    );
}

// ── preset 계보 (S026 — qa/general 프로젝트를 연구 manifest로 덮어쓰던 버그의 회귀 게이트) ──

fn new_project_preset(tmp: &Path, name: &str, preset: &str) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: name.into(),
            preset: preset.into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-07-13".into(),
        },
    )
    .unwrap()
}

/// 재조작 헬퍼: config.json의 preset 값을 교체(Some) 또는 제거(None — pre-S026 init 재현)
fn set_config_preset(root: &Path, preset: Option<&str>) {
    let p = root.join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    let obj = v.as_object_mut().unwrap();
    match preset {
        Some(s) => {
            obj.insert("preset".into(), serde_json::Value::String(s.into()));
        }
        None => {
            obj.remove("preset");
        }
    }
    fs::write(&p, serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

#[test]
fn qa_project_update_keeps_qa_identity() {
    // S026 t01 피해 경로 a, b, e의 반전 검증
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "Q", "qa");
    let agents_before = fs::read(root.join("AGENTS.md")).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.changed, 0, "fresh qa tree must be no-op: {:?}", report.lines);
    assert_eq!(report.conflicts, 0);
    assert!(report.lines.iter().any(|l| l.contains("preset: qa")), "{:?}", report.lines);
    // a: AGENTS.md가 연구판으로 교체되지 않음
    assert_eq!(fs::read(root.join("AGENTS.md")).unwrap(), agents_before);
    // b: 연구 전용 파일 미창조
    assert!(!root.join(".elf/managed/EliRule.md").exists());
    assert!(!root.join(".elf/managed/LogConvention.md").exists());
    assert!(!root.join(".claudeignore").exists());
    // e: re-stamp가 qa manifest 유지 (계보 정체성 보존)
    let stamp = fs::read_to_string(root.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, embed::MANIFEST_QA_JSON);
}

/// v2.27 general preset 삭제: 기존 general 프로젝트는 update가 연구 계보 minimal로 전환하고 안내.
#[test]
fn removed_general_project_is_converted_to_minimal_on_update() {
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "G", "minimal");
    // v2.26 이전 general 프로젝트 재현: config preset general + stamp src `templates/general/`,
    // EliRule = general판 내용(stamp 해시와 일치 = 사용자 미편집)
    let general_eli = "# EliRule (general)\n";
    fs::write(root.join(".elf/managed/EliRule.md"), general_eli).unwrap();
    let stamp_path = root.join(".elf/manifest.json");
    let mut stamp: serde_json::Value = serde_json::from_str(&fs::read_to_string(&stamp_path).unwrap()).unwrap();
    for e in stamp["files"].as_array_mut().unwrap() {
        if e["dest"] == ".elf/managed/EliRule.md" {
            e["src"] = "templates/general/EliRule.md".into();
            e["sha256"] = elf_cli::hash::sha256_lf(general_eli.as_bytes()).into();
        }
    }
    fs::write(&stamp_path, serde_json::to_string_pretty(&stamp).unwrap()).unwrap();
    set_config_preset(&root, Some("general"));
    let config_preset = |root: &Path| -> String {
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
        v["preset"].as_str().unwrap_or("").to_string()
    };

    // status: 경고 + 연구 계보로 진단(EliRule = 갱신 대기)
    let s = elf_cli::status::run_status(&root).unwrap();
    assert!(s.lines.iter().any(|l| l.contains("general was removed")), "{:?}", s.lines);
    assert!(s.pending >= 1, "{:?}", s.lines);

    // dry-run: 전환 예고만, config 불변
    let r = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("would convert the project to `minimal`")), "{:?}", r.lines);
    assert_eq!(config_preset(&root), "general");

    // 실행: config = minimal, EliRule = 연구판, stamp = 연구 manifest, 사용자 소유 파일 불변
    let pr_before = fs::read(root.join("0_Meta/ProjectRule.md")).unwrap();
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("converted this project to `minimal`")), "{:?}", r.lines);
    assert_eq!(config_preset(&root), "minimal");
    let eli = fs::read_to_string(root.join(".elf/managed/EliRule.md")).unwrap();
    assert_ne!(eli, general_eli);
    assert_eq!(fs::read_to_string(&stamp_path).unwrap(), embed::MANIFEST_JSON);
    assert_eq!(fs::read(root.join("0_Meta/ProjectRule.md")).unwrap(), pr_before);

    // 두 번째 update: 전환 안내 없음
    let r = run_update(&root, &plain()).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("general was removed")), "{:?}", r.lines);
}

/// config에 preset이 없어도 stamp의 `templates/general/` src로 전환 대상을 판정.
#[test]
fn removed_general_detected_from_stamp_without_config_preset() {
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "G2", "minimal");
    let stamp_path = root.join(".elf/manifest.json");
    let mut stamp: serde_json::Value = serde_json::from_str(&fs::read_to_string(&stamp_path).unwrap()).unwrap();
    stamp["files"][0]["src"] = "templates/general/AGENTS.md".into();
    fs::write(&stamp_path, serde_json::to_string_pretty(&stamp).unwrap()).unwrap();
    set_config_preset(&root, None);
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("converted this project to `minimal`")), "{:?}", r.lines);
}

#[test]
fn legacy_config_without_preset_is_inferred_and_healed() {
    // pre-S026 init 재현: config에 preset 키 없음 → stamp src 시그니처로 추론 + 자동 복구
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "Q", "qa");
    set_config_preset(&root, None);
    let agents_before = fs::read(root.join("AGENTS.md")).unwrap();

    // dry-run: 추론 안내만, config 무변경 (읽기전용 예고)
    let dry = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert!(dry.lines.iter().any(|l| l.contains("inferred") && l.contains("would record")), "{:?}", dry.lines);
    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert!(cfg.get("preset").is_none(), "dry-run must not heal config");

    // 본실행: 추론 + 자동 복구 + qa 정체성 유지
    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l.contains("inferred") && l.contains("recorded")), "{:?}", report.lines);
    assert_eq!(report.changed, 0);
    let cfg: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(cfg["preset"], "qa", "automatic repair records the inferred kind");
    assert_eq!(cfg["lang"], "ko-KR", "existing config fields preserved");
    assert_eq!(fs::read(root.join("AGENTS.md")).unwrap(), agents_before);
    assert_eq!(
        fs::read_to_string(root.join(".elf/manifest.json")).unwrap(),
        embed::MANIFEST_QA_JSON
    );
}

#[test]
fn preset_mismatch_refuses_update_untouched() {
    // config 오선언(연구 preset)인데 stamp는 qa — 이종 manifest 계획을 진입 시점에 거부, 트리 무변경
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "Q", "qa");
    set_config_preset(&root, Some("full"));
    let agents_before = fs::read(root.join("AGENTS.md")).unwrap();
    let stamp_before = fs::read(root.join(".elf/manifest.json")).unwrap();

    let err = run_update(&root, &plain()).unwrap_err();
    assert!(
        matches!(err, elf_cli::update::UpdateError::PresetMismatch { .. }),
        "expected PresetMismatch, got {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("\"full\"") && msg.contains("qa"), "{msg}");
    // 거부 = 완전 무변경 (파일, stamp, version 전부)
    assert_eq!(fs::read(root.join("AGENTS.md")).unwrap(), agents_before);
    assert_eq!(fs::read(root.join(".elf/manifest.json")).unwrap(), stamp_before);
    assert!(!root.join(".elf/managed/EliRule.md").exists());
}

// ── pre-2.15 잔재 안내 (S024 t09 — 감지+안내만, 비접촉) ──────

#[test]
fn pre215_leftovers_warn_but_are_left_untouched() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 현행 배치(.elf/managed/에 정본) + 0_Meta에 구 규칙 사본 잔존 — 구버전 폴더를 제자리 도입한 뒤의 상태
    fs::write(root.join("0_Meta/EliRule.md"), b"legacy leftover
").unwrap();
    fs::write(root.join("0_Meta/LLMcliche.md"), b"legacy leftover
").unwrap();

    let report = run_update(&root, &plain()).unwrap();
    let warn = report
        .lines
        .iter()
        .find(|l| l.contains("old rule copies remain"))
        .expect("구 사본 경고 출력");
    assert!(warn.contains("0_Meta/EliRule.md") && warn.contains("0_Meta/LLMcliche.md"));
    // v2.27.1: 현행 배치에서는 v2.15.1 경로가 아니라 검토 후 삭제 안내
    assert!(warn.contains("current rules are in .elf/managed/") && warn.contains("then delete them"), "{warn}");
    assert!(!warn.contains("v2.15.1"), "{warn}");
    assert!(report.warnings >= 1);
    // 비접촉 보장: 잔재 무변경, .elf-new 병기 없음, update는 정상 진행
    assert_eq!(fs::read(root.join("0_Meta/EliRule.md")).unwrap(), b"legacy leftover
");
    assert!(!root.join("0_Meta/EliRule.md.elf-new").exists());
    assert_eq!(report.conflicts, 0, "{:?}", report.lines);
    // dry-run에도 동일 안내, check files에도 경고
    let dry = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert!(dry.lines.iter().any(|l| l.contains("old rule copies remain")));
    let env = elf_cli::doctor::DoctorEnv { version: embed::version().to_string(), receipt_present: false };
    let files = elf_cli::check::run_scope(&root, &env, elf_cli::check::Scope::Files);
    assert!(files.items.iter().any(|i| i.text.starts_with("0_Meta: old rule copies remain")), "{:?}", files.items);
}

/// 구 배치(.elf/managed/ 정본 없음)에서는 종전 안내(v2.15.1 경로) 유지.
#[test]
fn legacy_layout_keeps_two_step_upgrade_message() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join("0_Meta/EliRule.md"), b"legacy
").unwrap();
    fs::remove_file(root.join(".elf/managed/EliRule.md")).unwrap();
    let dry = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    let warn = dry.lines.iter().find(|l| l.contains("pre-2.15 legacy rule files remain")).expect("구 배치 안내");
    assert!(warn.contains("v2.15.1") && warn.contains("elf migrate"), "{warn}");
}

#[test]
fn project_rule_in_0meta_is_not_flagged_as_leftover() {
    // 0_Meta/ProjectRule.md는 user tier(정당 상주) — 잔재 감지 대상 아님
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    assert!(root.join("0_Meta/ProjectRule.md").is_file(), "user tier 전제");
    let report = run_update(&root, &plain()).unwrap();
    assert!(!report.lines.iter().any(|l| l.contains("pre-2.15 legacy rule files remain")), "{:?}", report.lines);
}

#[test]
fn managed_unedited_with_newer_template_is_overwritten() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 연출: 배포 시점엔 "old content"였고 stamp도 그 해시였다고 가정
    let dest = ".elf/managed/EliRule.md";
    fs::write(root.join(dest), b"old content\n").unwrap();
    tamper_stamp_sha(&root, dest, &hash::sha256_lf(b"old content\n"));

    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l == &format!("updated: {dest}")));
    // 새 정본 바이트로 교체됨
    let expected = embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents();
    assert_eq!(fs::read(root.join(dest)).unwrap(), expected);
}

#[test]
fn managed_user_edit_is_preserved_with_elf_new() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = ".elf/managed/EliRule.md";
    fs::write(root.join(dest), b"my custom rules\n").unwrap(); // stamp는 정본 해시 그대로

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    // 원본 보존
    assert_eq!(fs::read(root.join(dest)).unwrap(), b"my custom rules\n");
    // 새 버전은 .elf-new로
    let elf_new = fs::read(root.join(format!("{dest}.elf-new"))).unwrap();
    assert_eq!(elf_new, embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents());
}

#[test]
fn managed_user_edit_force_overwrites() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = ".elf/managed/EliRule.md";
    fs::write(root.join(dest), b"my custom rules\n").unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert_eq!(report.conflicts, 0);
    assert_eq!(
        fs::read(root.join(dest)).unwrap(),
        embed::TEMPLATES.get_file("meta/EliRule.md").unwrap().contents()
    );
    assert!(report.lines.iter().any(|l| l.contains("force-updated")));
}

#[test]
fn managed_missing_is_restored() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = ".elf/managed/highIFjournals.md";
    fs::remove_file(root.join(dest)).unwrap();

    run_update(&root, &plain()).unwrap();
    assert_eq!(
        fs::read(root.join(dest)).unwrap(),
        embed::TEMPLATES.get_file("meta/highIFjournals.md").unwrap().contents()
    );
}

// ── 행렬: user / instance ───────────────────────────────────

#[test]
fn user_tier_and_instance_are_never_touched() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join("0_Meta/ProjectRule.md"), b"customized by user\n").unwrap();
    fs::write(root.join("2_Log/S001_log.md"), b"my session notes\n").unwrap();

    run_update(&root, &plain()).unwrap();
    assert_eq!(fs::read(root.join("0_Meta/ProjectRule.md")).unwrap(), b"customized by user\n");
    assert_eq!(fs::read(root.join("2_Log/S001_log.md")).unwrap(), b"my session notes\n");
    assert!(!root.join("0_Meta/ProjectRule.md.elf-new").exists());
}

// ── 행렬: hybrid (마커블록) ─────────────────────────────────

#[test]
fn hybrid_clean_block_is_replaced_and_user_area_preserved() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 연출: 배포본이 구버전 블록이었다고 가정 — baseline, 현재 파일 모두 구블록 + 사용자 규칙
    let old_template = format!("{MARKER_START}\nold-rule/\n{MARKER_END}\n");
    let current = format!("{old_template}\n# user rules\nmy_data/\n");
    fs::write(root.join(".gitignore"), &current).unwrap();
    fs::write(root.join(".elf/baseline/.gitignore"), &old_template).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l.contains("merged block")), "{:?}", report.lines);
    let merged = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(!merged.contains("old-rule/"), "old block must be replaced");
    assert!(merged.contains("*.asv"), "new block content present");
    assert!(merged.contains("my_data/"), "user area preserved");
    // baseline 갱신 → 다음 update에서 편집 오탐 없음
    let report2 = run_update(&root, &plain()).unwrap();
    assert_eq!(report2.conflicts, 0);
}

#[test]
fn hybrid_edited_block_is_kept_with_elf_new() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    // 사용자가 블록 안을 직접 수정 (baseline은 init이 기록한 정본)
    let current = fs::read_to_string(root.join(".gitignore")).unwrap();
    let hacked = current.replace("*.asv", "*.asv\nuser-inserted-inside-block/");
    fs::write(root.join(".gitignore"), &hacked).unwrap();

    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    assert!(report.lines.iter().any(|l| l.contains("block edited")), "{:?}", report.lines);
    // 파일 보존 + .elf-new 산출
    assert_eq!(fs::read_to_string(root.join(".gitignore")).unwrap(), hacked);
    assert!(root.join(".gitignore.elf-new").exists());
}

#[test]
fn hybrid_edited_block_force_replaces() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let current = fs::read_to_string(root.join(".gitignore")).unwrap();
    fs::write(root.join(".gitignore"), current.replace("*.asv", "hacked/")).unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert_eq!(report.conflicts, 0);
    let merged = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(merged.contains("*.asv"));
    assert!(!merged.contains("hacked/"));
}

#[test]
fn hybrid_missing_markers_kept_unless_force() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".gitignore"), b"only-user-rules/\n").unwrap();

    // 기본: 미변경 + 경고
    let report = run_update(&root, &plain()).unwrap();
    assert_eq!(report.conflicts, 1);
    assert_eq!(fs::read(root.join(".gitignore")).unwrap(), b"only-user-rules/\n");

    // --force: 상단 재삽입 + 기존 내용 보존
    let report2 = run_update(&root, &UpdateOptions { dry_run: false, force: true }).unwrap();
    assert!(report2.lines.iter().any(|l| l.contains("reinserted")));
    let rebuilt = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(rebuilt.starts_with(MARKER_START));
    assert!(rebuilt.contains("only-user-rules/"));
}

// ── dry-run / re-stamp / 프로젝트 인식 ──────────────────────

#[test]
fn dry_run_writes_nothing() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let dest = ".elf/managed/EliRule.md";
    fs::write(root.join(dest), b"edited\n").unwrap();
    fs::write(root.join(".elf/version"), "v0.0\n").unwrap();

    let report = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert_eq!(report.conflicts, 1); // 보고는 됨
    assert!(!root.join(format!("{dest}.elf-new")).exists(), "dry-run must not write .elf-new");
    assert_eq!(fs::read_to_string(root.join(".elf/version")).unwrap(), "v0.0\n", "dry-run must not re-stamp");
}

#[test]
fn update_always_restamps_manifest_and_version() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    fs::write(root.join(".elf/version"), "v0.0\n").unwrap();
    tamper_stamp_sha(&root, ".elf/managed/EliRule.md", "deadbeef");

    run_update(&root, &plain()).unwrap();
    assert_eq!(
        fs::read_to_string(root.join(".elf/version")).unwrap(),
        format!("{}\n", embed::version())
    );
    // stamp가 embed 정본으로 복원
    let stamp = fs::read_to_string(root.join(".elf/manifest.json")).unwrap();
    assert_eq!(stamp, embed::MANIFEST_JSON);
    assert!(manifest::parse(&stamp).is_ok());
}

#[test]
fn non_elf_dir_errors_and_root_is_found_from_subdir() {
    let tmp = tempdir().unwrap();
    assert!(elf_cli::update::find_project_root(tmp.path()).is_none());

    let root = new_project(tmp.path(), "P");
    let sub = root.join("2_Log/Wiki");
    assert_eq!(elf_cli::update::find_project_root(&sub).unwrap(), root);
}

#[test]
fn git_unborn_repo_warns_missing_baseline() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path(), "P");
    let git_ok = std::process::Command::new("git")
        .arg("-C").arg(&root).arg("init").output()
        .map(|o| o.status.success()).unwrap_or(false);
    if !git_ok {
        eprintln!("git unavailable — skipping");
        return;
    }
    // git init 직후 = 커밋 0(unborn HEAD) — 롤백 기준 부재 경고 (S021 t15).
    // untracked-only 무경고, 추적 변경 혼합 경고의 전체 분기는 tests/trial.rs 참조.
    let report = run_update(&root, &plain()).unwrap();
    assert!(report.lines.iter().any(|l| l.contains("no git commit yet")), "{:?}", report.lines);
}

// ── preset 표시와 정규화 (v2.27.1 — 미지정 연구 preset = full) ───────────────

#[test]
fn research_lineage_name_in_config_is_recorded_as_full() {
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "R", "minimal");
    set_config_preset(&root, Some("research")); // 구 update가 추론 결과로 기록하던 값
    let s = elf_cli::status::run_status(&root).unwrap();
    assert!(s.lines.iter().any(|l| l.contains("preset: full (research lineage; .elf/config.json says `research`")), "{:?}", s.lines);
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("preset: full (research lineage; recorded `full`")), "{:?}", r.lines);
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(v["preset"], "full");
    // 다음 update는 그대로 표시만
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l == "preset: full (research lineage)"), "{:?}", r.lines);
}

#[test]
fn inferred_research_preset_is_recorded_as_full() {
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "R2", "minimal");
    set_config_preset(&root, None);
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("preset: full (inferred from stamp — recorded")), "{:?}", r.lines);
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".elf/config.json")).unwrap()).unwrap();
    assert_eq!(v["preset"], "full");
}

#[test]
fn config_preset_is_shown_with_lineage() {
    let tmp = tempdir().unwrap();
    let root = new_project_preset(tmp.path(), "R3", "minimal");
    let r = run_update(&root, &plain()).unwrap();
    assert!(r.lines.iter().any(|l| l == "preset: minimal (research lineage)"), "{:?}", r.lines);
}

