//! autoread: `elf autoread` — 컨텍스트 재구성 후 거버넌스 digest 재주입 (S031).
//! 불변식 검증 중심: 기본 켬(config 부재=on) · 비파괴 병합(타 설정 보존·malformed refuse) ·
//! 마커 수명주기(session_id 단위·1회 주입·TTL gc) · digest 정본 추출(AGENTS 절·활성 상한·절단) ·
//! 훅 fail-open(disabled 침묵) · update 훅 보장.

use elf_cli::autoread;
use elf_cli::init::{run_init, InitOptions};
use elf_cli::update::{run_update, UpdateOptions};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn new_project(tmp: &Path) -> PathBuf {
    run_init(
        tmp,
        &InitOptions {
            name: "P".into(),
            preset: "minimal".into(),
            modules: None,
            categories: vec![],
            lang: "ko-KR".into(),
            date: "2026-07-16".into(),
        },
    )
    .expect("init")
}

fn settings_path(root: &Path) -> PathBuf {
    root.join(".claude").join("settings.json")
}

/// 활성 세션 로그를 직접 배치 (헤더 형식 = sessionTemplate 동형 — 파서 대상 필드만).
fn write_active_log(root: &Path, num: u32, handoff: &str) {
    let id = format!("S{num:03}");
    let content = format!(
        "# {id}: test session {num}\n\n> **Created**: 2026-07-16\\\n> **Modified**: 2026-07-16\\\n> **Status**: ★ 활성\\\n> **목표**: t\\\n> **관련**: -\\\n> **Handoff**: {handoff}\n\n---\n"
    );
    fs::write(root.join("2_Log").join(format!("{id}_log.md")), content).expect("write log");
}

fn session_start_stdin(session_id: &str, source: &str) -> String {
    format!(r#"{{"session_id":"{session_id}","hook_event_name":"SessionStart","source":"{source}"}}"#)
}

fn prompt_stdin(session_id: &str) -> String {
    format!(r#"{{"session_id":"{session_id}","hook_event_name":"UserPromptSubmit","prompt":"go"}}"#)
}

// ── config 극성 (기본 켬) ─────────────────────────────────────────────────────

#[test]
fn config_absent_means_enabled_and_toggle_roundtrip() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // init은 config에 autoread 키를 쓰지 않음 — 부재 = 켬 (default-on)
    let cfg = fs::read_to_string(root.join(".elf/config.json")).unwrap();
    assert!(!cfg.contains("autoread"));
    assert!(autoread::is_enabled(&root));

    // turn off → .elf에 false 기록, turn on → true 기록 (프로젝트 단위 스위치)
    autoread::run_disable(&root).unwrap();
    assert!(!autoread::is_enabled(&root));
    assert!(fs::read_to_string(root.join(".elf/config.json")).unwrap().contains("\"autoread\": false"));
    autoread::run_enable(&root).unwrap();
    assert!(autoread::is_enabled(&root));
    assert!(fs::read_to_string(root.join(".elf/config.json")).unwrap().contains("\"autoread\": true"));
}

// ── settings.json 병합 (비파괴) ───────────────────────────────────────────────

#[test]
fn init_installs_hooks_and_install_is_idempotent() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // init이 훅을 배치 (기본 켬의 설치 지점)
    let text = fs::read_to_string(settings_path(&root)).unwrap();
    assert!(text.contains("elf autoread hook session-start"));
    assert!(text.contains("elf autoread hook prompt"));
    assert!(text.contains("compact|resume|clear"));
    for (_, installed) in autoread::hook_states(&root) {
        assert!(installed);
    }
    // 멱등: 재실행 = 무변경
    assert!(!autoread::install_hooks(&root).unwrap());
    assert_eq!(text, fs::read_to_string(settings_path(&root)).unwrap());
}

#[test]
fn install_preserves_foreign_settings_and_hooks() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 사용자 설정 + 타 훅이 이미 존재
    fs::write(
        settings_path(&root),
        r#"{
  "permissions": { "allow": ["Bash(ls:*)"] },
  "hooks": {
    "SessionStart": [
      { "matcher": "startup", "hooks": [{ "type": "command", "command": "echo hello" }] }
    ]
  }
}"#,
    )
    .unwrap();
    assert!(autoread::install_hooks(&root).unwrap());
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(settings_path(&root)).unwrap()).unwrap();
    // 사용자 키·타 훅 보존
    assert_eq!(v["permissions"]["allow"][0], "Bash(ls:*)");
    let ss = v["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(ss.len(), 2, "foreign entry kept + elf entry added");
    assert!(ss.iter().any(|e| e["hooks"][0]["command"] == "echo hello"));
    // elf 항목 재실행 시 중복 증식 없음
    autoread::install_hooks(&root).ok();
    let v2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(settings_path(&root)).unwrap()).unwrap();
    assert_eq!(v2["hooks"]["SessionStart"].as_array().unwrap().len(), 2);
}

#[test]
fn install_refuses_malformed_settings_without_touching_file() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    fs::write(settings_path(&root), "{ not json").unwrap();
    let err = autoread::install_hooks(&root).unwrap_err();
    assert!(matches!(err, autoread::AutoreadError::Refuse(_)));
    assert_eq!(fs::read_to_string(settings_path(&root)).unwrap(), "{ not json");
}

// ── 마커 수명주기 · 훅 진입점 ─────────────────────────────────────────────────

#[test]
fn session_start_marks_and_prompt_injects_exactly_once() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    write_active_log(&root, 2, "state; 미완료 = X; 참조 t01");

    // compact 감지 → 마커 (stdout 없음 — #15174 우회: 기록만)
    assert!(autoread::run_hook(&root, "session-start", &session_start_stdin("sid-1", "compact")).is_none());
    // 다음 프롬프트 → digest 주입 1회
    let digest = autoread::run_hook(&root, "prompt", &prompt_stdin("sid-1")).expect("digest");
    assert!(digest.contains("context was reconstructed (compact)"));
    assert!(digest.contains("AGENTS.md standing duties")); // 절 구분자
    assert!(digest.contains("컨텍스트 재구성 후 재정렬")); // AGENTS 정본 본문(L23) 실추출 검증
    assert!(digest.contains("S002")); // 활성 세션 Handoff
    assert!(digest.contains("validate:")); // 카운트
    // 두 번째 프롬프트 = 침묵 (마커 소모됨)
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("sid-1")).is_none());
}

#[test]
fn markers_are_per_session_no_cross_fire() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    autoread::run_hook(&root, "session-start", &session_start_stdin("agent-A", "compact"));
    // 다른 세션(agent-B)의 프롬프트에는 미발동 (멀티에이전트 교차 오발동 방지 — S030 정합)
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("agent-B")).is_none());
    // 본인(agent-A)에는 발동
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("agent-A")).is_some());
}

#[test]
fn startup_source_and_disabled_config_stay_silent() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // startup = 신규 세션(CLAUDE.md 체인 fresh 로드) — 재정렬 대상 아님
    autoread::run_hook(&root, "session-start", &session_start_stdin("s", "startup"));
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("s")).is_none());
    // disabled → 훅 전체 no-op (훅 잔존·config 게이트)
    autoread::run_disable(&root).unwrap();
    autoread::run_hook(&root, "session-start", &session_start_stdin("s2", "compact"));
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("s2")).is_none());
}

#[test]
fn stale_and_unreadable_markers_are_gced() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let dir = root.join(autoread::RUNTIME_DIR);
    fs::create_dir_all(&dir).unwrap();
    // TTL(24h) 초과 마커 + 판독 불가 마커
    fs::write(dir.join("old-session"), r#"{"source":"compact","ts":1}"#).unwrap();
    fs::write(dir.join("garbage"), "not json").unwrap();
    // 아무 훅 호출이 부수 gc 수행
    autoread::run_hook(&root, "session-start", &session_start_stdin("fresh", "compact"));
    assert!(!dir.join("old-session").exists());
    assert!(!dir.join("garbage").exists());
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("old-session")).is_none());
}

// ── digest 구성 (정본 추출·상한·절단) ─────────────────────────────────────────

#[test]
fn digest_caps_active_sessions_and_truncates_handoff() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    let long_handoff = "가".repeat(300);
    for n in 2..=8 {
        write_active_log(&root, n, &long_handoff);
    }
    let d = autoread::build_digest(&root, "manual");
    // 최근(번호 큰 순) 5개 + "+N more" (실측 근거: 활성 27개 프로젝트 — S031 t01)
    for shown in ["S008", "S007", "S006", "S005", "S004"] {
        assert!(d.contains(shown), "missing {shown}");
    }
    assert!(!d.contains("S003:"));
    assert!(d.contains("more active"));
    // Handoff 200자 절단 + ellipsis (300자 원문 미포함)
    assert!(d.contains(&"가".repeat(200)));
    assert!(!d.contains(&"가".repeat(201)));
    assert!(d.contains('…'));
}

// ── fulltext (config 선언식 전문 주입 — ContextReanchor 개정판, S031 t16) ──────

/// config에 `autoread_fulltext` 배열 기록 (스위치 bool과 별도 키 — 기존 config 병합).
fn declare_fulltext(root: &Path, paths: &[&str]) {
    let p = root.join(".elf/config.json");
    let mut v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap_or_else(|_| "{}".into())).unwrap();
    v["autoread_fulltext"] = serde_json::json!(paths);
    fs::write(&p, serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

#[test]
fn fulltext_declared_files_injected_in_full_with_imperative_footer() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // 미선언 = digest만 + 명령형 전문 재독 지시 (경로 특정은 프로젝트 라우팅 소관)
    let d0 = autoread::build_digest(&root, "manual");
    assert!(!d0.contains("--- full text:"));
    assert!(d0.contains("before your first substantive action"));
    assert!(d0.contains("Read in full the task-relevant canonical rule documents"));

    fs::create_dir_all(root.join("0_Meta")).ok();
    fs::write(root.join("0_Meta/LocalRule.md"), "# LocalRule\n\nfigure embed 3단 규칙 본문").unwrap();
    declare_fulltext(&root, &["0_Meta/LocalRule.md"]);
    let d = autoread::build_digest(&root, "compact");
    // 전문 블록 + 출처 표기 + 명령형 적용 지시(fulltext 분기)
    assert!(d.contains("--- full text: 0_Meta/LocalRule.md"));
    assert!(d.contains("figure embed 3단 규칙 본문"));
    assert!(d.contains("declared canonical rules is included above"));
    assert!(d.contains("do not act on the compact summary alone"));
}

#[test]
fn fulltext_unsafe_and_missing_paths_fail_open() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    declare_fulltext(&root, &["../outside.md", "0_Meta/absent.md"]);
    // 트리 이탈·부재 — digest는 정상 생성(fail-open), 각 1행 표기, 전문 블록 없음
    let d = autoread::build_digest(&root, "compact");
    assert!(d.contains("path escapes the project tree: ../outside.md"));
    assert!(d.contains("declared but unreadable: 0_Meta/absent.md"));
    assert!(!d.contains("--- full text:"));
    // 훅 경로도 동일 (주입 자체는 성공)
    autoread::run_hook(&root, "session-start", &session_start_stdin("s", "compact"));
    assert!(autoread::run_hook(&root, "prompt", &prompt_stdin("s")).is_some());
    // status가 선언 상태 표면화
    let st = autoread::run_status(&root).unwrap();
    let joined = st.lines.join("\n");
    assert!(joined.contains("../outside.md — UNSAFE PATH"));
    assert!(joined.contains("0_Meta/absent.md — MISSING"));
}

#[test]
fn fulltext_declaration_survives_toggle_roundtrip() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    declare_fulltext(&root, &["0_Meta/LocalRule.md"]);
    // disable/enable이 스위치 bool만 조작 — 선언 목록 비파괴 (별도 키 설계 근거)
    autoread::run_disable(&root).unwrap();
    autoread::run_enable(&root).unwrap();
    assert_eq!(autoread::fulltext_list(&root), vec!["0_Meta/LocalRule.md".to_string()]);
}

// ── 배너 (보조 주입 채널) ─────────────────────────────────────────────────────

#[test]
fn banner_pending_then_cleared_by_ack() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    assert!(autoread::pending_banner(&root).is_none());
    autoread::run_hook(&root, "session-start", &session_start_stdin("s", "resume"));
    let b = autoread::pending_banner(&root).expect("banner");
    assert!(b.contains("elf autoread"));
    autoread::run_ack(&root).unwrap();
    assert!(autoread::pending_banner(&root).is_none());
}

// ── update 통합 (설치 자동화 — clone 복원 경로) ───────────────────────────────

#[test]
fn update_restores_hooks_unless_disabled() {
    let tmp = tempdir().unwrap();
    let root = new_project(tmp.path());
    // clone 직후 상황 재현: settings.json 부재 (.claude/는 git 비추적)
    fs::remove_dir_all(root.join(".claude")).unwrap();
    run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert!(settings_path(&root).is_file(), "update must restore hooks");
    // disabled면 update가 훅을 만들지 않음
    fs::remove_dir_all(root.join(".claude")).unwrap();
    autoread::run_disable(&root).unwrap();
    run_update(&root, &UpdateOptions { dry_run: false, force: false }).unwrap();
    assert!(!settings_path(&root).exists());
}
