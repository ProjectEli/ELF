//! tsa: `elf tsa` — opt-in 연구 기록 시점인증 (S022 — Mastication #1 역이식).
//! 불변식 검증 중심: 멱등 enable, 비파괴 훅(타 훅 보존), disable 후 증거 보존 ,
//! 원시 바이트 해시, disabled 게이트(quiet 침묵). 네트워크 비의존(offline graceful만 검증).

use assert_cmd::Command as AssertCommand;
use elf_cli::init::{InitOptions, run_init};
use elf_cli::tsa::{self, RecordScope};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(root).output().expect("run git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn git_stdout(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(root).output().expect("run git");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// minimal 프로젝트 + git init (커밋 불요 — record는 index 기준).
fn new_project(tmp: &Path) -> PathBuf {
    let root = run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: Vec::new(),
            lang: "ko-KR".into(),
            date: "2026-07-15".into(),
        },
    )
    .unwrap();
    git(&root, &["init", "-q"]);
    root
}

#[test]
fn enable_is_idempotent_and_sets_config_dir_hooks() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert!(!tsa::is_enabled(&root));

    let r1 = tsa::run_enable(&root).unwrap();
    assert!(tsa::is_enabled(&root));
    assert!(root.join("0_Meta/tsa").is_dir());
    for name in ["pre-commit", "post-commit"] {
        let hook = fs::read_to_string(root.join(".git/hooks").join(name)).unwrap();
        assert!(hook.contains("# elf-tsa hook"), "{name} not marker-owned:\n{hook}");
        assert!(hook.contains("|| true"), "{name} must never block a commit");
    }
    // 훅 환경 PATH 빈약 대비 — $HOME/.elf/bin 우선 폴백이 본문에 존재
    let pre = fs::read_to_string(root.join(".git/hooks/pre-commit")).unwrap();
    assert!(pre.contains("$HOME/.elf/bin/elf"));

    // 재실행 = 멱등 (오류, 중복 없음)
    let r2 = tsa::run_enable(&root).unwrap();
    assert!(tsa::is_enabled(&root));
    assert!(r1.lines.iter().any(|l| l.contains("config: tsa = true")));
    assert!(r2.lines.iter().any(|l| l.contains("config: tsa = true")));
}

#[test]
fn enable_leaves_foreign_hook_untouched_with_guidance() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let hooks = root.join(".git/hooks");
    fs::create_dir_all(&hooks).unwrap();
    let foreign = "#!/usr/bin/env python\nprint('my own hook')\n";
    fs::write(hooks.join("pre-commit"), foreign).unwrap();

    let r = tsa::run_enable(&root).unwrap();
    // 타 훅 원문 그대로 (비파괴 — S022 t03 승인안; Mastication python 훅이 반례 원형)
    assert_eq!(fs::read_to_string(hooks.join("pre-commit")).unwrap(), foreign);
    // 수동 추가 안내 경고 존재
    assert!(
        r.lines.iter().any(|l| l.contains("pre-commit hook exists") && l.contains("manually")),
        "{:?}",
        r.lines
    );
    // post-commit은 부재였으므로 정상 설치
    assert!(fs::read_to_string(hooks.join("post-commit")).unwrap().contains("# elf-tsa hook"));
}

#[test]
fn disable_removes_only_marker_hooks_and_keeps_evidence() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();

    // 증거 생성(record) 후 disable
    fs::write(root.join("data.txt"), b"payload-1").unwrap();
    git(&root, &["add", "data.txt"]);
    tsa::run_record(&root, RecordScope::Staged).unwrap();
    let (manifests, _, _) = tsa::evidence_counts(&root);
    assert_eq!(manifests, 1);

    // post-commit을 타 훅으로 교체(사용자가 직접 관리로 전환한 상황)
    fs::write(root.join(".git/hooks/post-commit"), "#!/bin/sh\n# mine\n").unwrap();

    let r = tsa::run_disable(&root).unwrap();
    assert!(!tsa::is_enabled(&root));
    assert!(!root.join(".git/hooks/pre-commit").exists(), "marker hook must be removed");
    assert!(root.join(".git/hooks/post-commit").exists(), "foreign hook must survive disable");
    // 증거 불변식: disable은 0_Meta/tsa/를 건드리지 않음
    let (after, _, _) = tsa::evidence_counts(&root);
    assert_eq!(after, 1);
    assert!(r.lines.iter().any(|l| l.contains("evidence kept")));
}

#[test]
fn record_staged_writes_raw_sha256_stages_manifest_and_dedupes() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();

    // CRLF 내용 — 원시 바이트 해시 검증(정규화 금지: hash::sha256_lf와 목적 상이)
    let bytes = b"line1\r\nline2\r\n";
    fs::write(root.join("data.txt"), bytes).unwrap();
    git(&root, &["add", "data.txt"]);
    tsa::run_record(&root, RecordScope::Staged).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mpath = root.join(format!("0_Meta/tsa/{today}_manifest.json"));
    let m: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mpath).unwrap()).unwrap();
    let entry = m["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["file"] == "data.txt")
        .expect("data.txt recorded");
    // 원시 sha256(CRLF 그대로) — sha2로 재계산 대조
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    assert_eq!(entry["sha256"].as_str().unwrap(), format!("{:x}", h.finalize()));

    // manifest 자신이 staged(커밋 동승 — Mastication pre-commit 동작 승계)
    let staged = git_stdout(&root, &["diff", "--cached", "--name-only"]);
    assert!(staged.contains("0_Meta/tsa/"), "manifest must be auto-staged: {staged}");

    // 동일 내용 재기록 = 중복 없음(멱등)
    let before = m["entries"].as_array().unwrap().len();
    tsa::run_record(&root, RecordScope::Staged).unwrap();
    let m2: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mpath).unwrap()).unwrap();
    assert_eq!(m2["entries"].as_array().unwrap().len(), before);
}

#[test]
fn record_all_covers_tracked_files_as_baseline() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();
    fs::write(root.join("a.txt"), b"a").unwrap();
    fs::write(root.join("b.txt"), b"b").unwrap();
    git(&root, &["add", "."]);
    tsa::run_record(&root, RecordScope::All).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let m: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("0_Meta/tsa/{today}_manifest.json"))).unwrap(),
    )
    .unwrap();
    let files: Vec<&str> =
        m["entries"].as_array().unwrap().iter().filter_map(|e| e["file"].as_str()).collect();
    assert!(files.contains(&"a.txt") && files.contains(&"b.txt"));
    // init 산출물(추적 시)도 포함 — baseline은 git 추적 전체
    assert!(files.iter().any(|f| f.starts_with("0_Meta/") || f.ends_with(".md")), "{files:?}");
}

#[test]
fn verify_finds_recorded_file_and_flags_edited_one() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();
    fs::write(root.join("data.txt"), b"original").unwrap();
    git(&root, &["add", "data.txt"]);
    tsa::run_record(&root, RecordScope::Staged).unwrap();

    let ok = tsa::run_verify(&root, Some("data.txt"), None).unwrap();
    assert!(ok.lines.iter().any(|l| l.starts_with("found: data.txt")), "{:?}", ok.lines);
    assert_eq!(ok.warnings, 0);

    // 기록 후 수정 → 현재 내용은 어느 manifest에도 없음(경고)
    fs::write(root.join("data.txt"), b"edited").unwrap();
    let miss = tsa::run_verify(&root, Some("data.txt"), None).unwrap();
    assert!(miss.warnings > 0, "{:?}", miss.lines);
}

#[test]
fn stamp_offline_is_graceful_and_leaves_no_token() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();
    // TSA를 닫힌 로컬 포트로 — 네트워크 비의존 실패 경로
    let cfg = root.join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    v.as_object_mut().unwrap().insert("tsaUrl".into(), "http://127.0.0.1:9".into());
    fs::write(&cfg, serde_json::to_string_pretty(&v).unwrap()).unwrap();

    fs::write(root.join("data.txt"), b"x").unwrap();
    git(&root, &["add", "data.txt"]);
    tsa::run_record(&root, RecordScope::Staged).unwrap();

    let r = tsa::run_stamp(&root, true).unwrap(); // Err 아닌 Ok+경고 (훅 경로 비차단)
    assert!(r.warnings > 0, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.contains("elf tsa --all")), "재시도 안내 필요: {:?}", r.lines);
    let (_, stamped, unstamped) = tsa::evidence_counts(&root);
    assert_eq!((stamped, unstamped), (0, 1)); // 깨진 토큰 미저장
}

#[test]
fn disabled_gate_refuses_direct_run_but_quiet_hook_path_is_silent() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // enable 없이 record, 토큰 요청(인수 없는 `elf tsa`) 직접 실행 → refuse(exit 3)
    for args in [vec!["tsa", "record"], vec!["tsa"], vec!["tsa", "--all"]] {
        AssertCommand::cargo_bin("elf").unwrap().args(&args).current_dir(&root).assert().code(3);
    }
    // 훅 경로(--quiet) → 침묵 성공(disable 후 잔존 수동 훅 라인의 무해화)
    for args in [vec!["tsa", "record", "--quiet"], vec!["tsa", "--quiet"]] {
        AssertCommand::cargo_bin("elf")
            .unwrap()
            .args(&args)
            .current_dir(&root)
            .assert()
            .success()
            .stdout("");
    }
}

/// v2.26: `tsa stamp`, `--backfill`, `record --staged` 삭제(별칭 없음) — usage 오류.
#[test]
fn removed_tsa_forms_are_usage_errors() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    for args in [
        vec!["tsa", "stamp"],
        vec!["tsa", "stamp", "--backfill"],
        vec!["tsa", "--backfill"],
        vec!["tsa", "record", "--staged"],
    ] {
        AssertCommand::cargo_bin("elf").unwrap().args(&args).current_dir(&root).assert().code(2);
    }
}

/// 네트워크 비의존 enable — TSA를 닫힌 로컬 포트로 지정한 뒤 enable(기준선 토큰 요청은 경고로 실패).
/// 추적 파일 1개를 먼저 두어 기준선 manifest가 생기게 함.
fn enable_offline(root: &Path) {
    fs::write(root.join("seed.txt"), b"seed").unwrap();
    git(root, &["add", "seed.txt"]);
    let cfg = root.join(".elf/config.json");
    let mut v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    v.as_object_mut().unwrap().insert("tsaUrl".into(), "http://127.0.0.1:9".into());
    fs::write(&cfg, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    tsa::run_enable(root).unwrap();
}

const LEGACY_HEAD: &str = "#!/bin/sh\n# elf-tsa hook — managed by `elf tsa enable`; removed by `elf tsa disable`. Do not edit (replaced in full on enable).\nELF=\"$HOME/.elf/bin/elf\"; { [ -x \"$ELF\" ] || [ -x \"$ELF.exe\" ]; } || ELF=elf\n";

/// v2.25 이전 훅 본문(구 명령 호출) 재현.
fn write_legacy_hooks(root: &Path) {
    let hooks = root.join(".git/hooks");
    fs::write(hooks.join("pre-commit"), format!("{LEGACY_HEAD}\"$ELF\" tsa record --staged --quiet || true\n")).unwrap();
    fs::write(
        hooks.join("post-commit"),
        format!("{LEGACY_HEAD}(\"$ELF\" tsa stamp --quiet >/dev/null 2>&1 || true) &\n"),
    )
    .unwrap();
}

#[test]
fn update_replaces_legacy_hooks_and_reports_once() {
    use elf_cli::update::{UpdateOptions, run_update};
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    enable_offline(&root);
    assert!(tsa::hook_states(&root).iter().all(|(_, s)| *s == tsa::HookState::Current));
    write_legacy_hooks(&root);
    assert!(tsa::hook_states(&root).iter().all(|(_, s)| *s == tsa::HookState::Outdated));

    // dry-run: 안내만, 훅 불변
    let r = run_update(&root, &UpdateOptions { dry_run: true, force: false }).unwrap();
    assert!(r.lines.iter().any(|l| l.contains("tsa: would update hooks")), "{:?}", r.lines);
    assert!(tsa::hook_states(&root).iter().all(|(_, s)| *s == tsa::HookState::Outdated));

    // 실행: 교체 + 누락 가능 안내(바뀐 내용, 마지막 manifest, 보충 명령)
    let r = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    let text = r.lines.join("\n");
    assert!(text.contains("tsa: updated hooks"), "{text}");
    assert!(text.contains("no longer exist"), "{text}");
    assert!(text.contains("last manifest"), "{text}");
    assert!(text.contains("elf tsa record --all") && text.contains("elf tsa --all"), "{text}");
    assert!(tsa::hook_states(&root).iter().all(|(_, s)| *s == tsa::HookState::Current));
    let pre = fs::read_to_string(root.join(".git/hooks/pre-commit")).unwrap();
    assert!(pre.contains("tsa record --quiet") && !pre.contains("--staged"), "{pre}");
    let post = fs::read_to_string(root.join(".git/hooks/post-commit")).unwrap();
    assert!(post.contains("\"$ELF\" tsa --quiet") && !post.contains("stamp"), "{post}");

    // 두 번째 update: 현행 본문이므로 tsa 안내 없음(첫 update에서 1회)
    let r = run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert!(!r.lines.iter().any(|l| l.contains("tsa:") || l.contains("no longer exist")), "{:?}", r.lines);
}

#[test]
fn update_leaves_foreign_hooks_and_disabled_projects_alone() {
    use elf_cli::update::{UpdateOptions, run_update};
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // tsa 끔: 구 본문이 있어도 손대지 않음
    fs::create_dir_all(root.join(".git/hooks")).unwrap();
    write_legacy_hooks(&root);
    run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    let pre = fs::read_to_string(root.join(".git/hooks/pre-commit")).unwrap();
    assert!(pre.contains("--staged"));
    // tsa 켬 + 타 도구 훅: 비접근
    enable_offline(&root);
    fs::write(root.join(".git/hooks/pre-commit"), "#!/bin/sh\necho other\n").unwrap();
    run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert_eq!(fs::read_to_string(root.join(".git/hooks/pre-commit")).unwrap(), "#!/bin/sh\necho other\n");
}

#[test]
fn check_files_warns_on_outdated_and_absent_hooks() {
    use elf_cli::check::{self, Level, Scope};
    use elf_cli::doctor::DoctorEnv;
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    enable_offline(&root);
    let env = DoctorEnv { version: elf_cli::embed::version().to_string(), receipt_present: false };
    let warns = |root: &Path| -> Vec<String> {
        check::run_scope(root, &env, Scope::Files)
            .items
            .into_iter()
            .filter(|i| i.level == Level::Warn && i.text.starts_with("tsa hooks"))
            .map(|i| i.text)
            .collect()
    };
    assert!(warns(&root).is_empty(), "{:?}", warns(&root));
    write_legacy_hooks(&root);
    let w = warns(&root);
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().all(|x| x.contains("run `elf update`")), "{w:?}");
    fs::remove_file(root.join(".git/hooks/post-commit")).unwrap();
    assert!(warns(&root).iter().any(|x| x.contains("post-commit hook absent")));
}

#[test]
fn e2e_tsa_all_retries_unstamped_manifests_offline() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    enable_offline(&root); // 기준선 manifest는 토큰 없이 남음
    let out = AssertCommand::cargo_bin("elf").unwrap().args(["tsa", "--all"]).current_dir(&root).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("retry with `elf tsa --all`"), "{text}");
}

#[test]
fn status_reports_tsa_line_only_when_enabled() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // disabled: elf status에 tsa 줄 없음 (opt-in 침묵)
    let s = elf_cli::status::run_status(&root).unwrap();
    assert!(!s.lines.iter().any(|l| l.contains("tsa")), "{:?}", s.lines);

    tsa::run_enable(&root).unwrap();
    let s = elf_cli::status::run_status(&root).unwrap();
    assert!(s.lines.iter().any(|l| l.contains("tsa: enabled")), "{:?}", s.lines);
}

#[test]
fn record_captures_non_ascii_filenames_despite_quotepath() {
    // S029 회귀: git 기본 core.quotepath=true는 non-ASCII 경로를 octal-escape+인용 출력 →
    // 구현이 인용 문자열을 그대로 join하면 fs::read 실패로 조용히 skip(무결성 gap).
    // Mastication 실측: baseline record가 한글파일 32건 미기록.
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    tsa::run_enable(&root).unwrap();
    // 최악 조건 강제: repo-local quotepath=true (코드의 -c false, -z가 이를 이겨야 함)
    git(&root, &["config", "core.quotepath", "true"]);
    fs::write(root.join("한글파일.txt"), b"korean").unwrap();
    fs::write(root.join("café.md"), b"accent").unwrap();
    fs::write(root.join("plain.txt"), b"ascii").unwrap();
    git(&root, &["add", "."]);
    tsa::run_record(&root, RecordScope::Staged).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let m: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("0_Meta/tsa/{today}_manifest.json"))).unwrap(),
    )
    .unwrap();
    let files: Vec<&str> =
        m["entries"].as_array().unwrap().iter().filter_map(|e| e["file"].as_str()).collect();
    assert!(files.contains(&"한글파일.txt"), "Korean filename skipped: {files:?}");
    assert!(files.contains(&"café.md"), "diacritic filename skipped: {files:?}");
    assert!(files.contains(&"plain.txt"), "ascii control missing: {files:?}");
    // verify도 non-ASCII 파일을 이력에서 찾을 수 있어야(round-trip)
    let v = tsa::run_verify(&root, Some("한글파일.txt"), None).unwrap();
    assert!(v.lines.iter().any(|l| l.starts_with("found:")), "{:?}", v.lines);
}
