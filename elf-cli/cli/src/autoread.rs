//! `elf autoread` — 컨텍스트 재구성(compact·resume·clear) 후 거버넌스 digest 자동 재주입 (S031).
//!
//! 문제(ELF_Proposal_ContextReanchor): compact 요약 = lossy 파생 digest — agent가 재구성 후
//! 정본(AGENTS 상시 의무·활성 Handoff)에 재정렬하지 않고 요약 위에서 동작하는 drift 실증(S031 t03).
//! 실패 축 분리(t04): 전달(내용이 컨텍스트에 있는가)과 발동(행동 개시점에서 실행하는가)은 별개 —
//! 본 기능은 "감지 = 하네스 훅 / 판정·출력 = 본 바이너리"로 전달을 결정화한다(발동 강제는 비채택,
//! t07: 차단은 목적[문서 재전달] 대비 과잉 — validate·close 게이트가 사후 안전망).
//!
//! 체인(t07 확정): ① SessionStart(compact|resume|clear) 훅 → 마커 기록(stdout 미사용 —
//! Claude Code #15174[compact matcher stdout 미주입]와 무관한 부수효과 경로) ② 다음
//! UserPromptSubmit 훅 → digest stdout 주입 + 마커 해제 ③ 프롬프트 미경유 갭(auto-compact 직후
//! 연속 작업)은 모든 elf 명령의 출력 선두 배너(main 진입부)가 보조 주입 채널.
//!
//! 소유 모델: 로직 = 본 바이너리 / `.claude/settings.json` = 비추적 파생물(.gitignore가 `.claude/`
//! 전체 ignore — clone 후 `elf update` 1회로 재생성) / 스위치 정본 = `.elf/config.json`(git 추적).
//! **기본 켬**: config `autoread` 키 부재 = true (tsa[부재=false·opt-in]와 반대 극성 — 사용자 확정).
//! disable = config false 기록 — 훅은 잔존하되 매 호출 config 게이트로 no-op(settings.json 왕복 회피).
//! 훅 경로는 전면 fail-open: 어떤 내부 오류도 세션을 막지 않는다(exit 0·무출력).
//! 마커 = `.elf/runtime/autoread/<session_id>` 파일 단위 — 멀티에이전트(동시 다터미널)에서
//! 세션 간 교차 오발동 없음(S030 정합). TTL 24h gc.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 마커 디렉토리 (git 비추적 — .gitignore managed 블록 `.elf/runtime/`).
pub const RUNTIME_DIR: &str = ".elf/runtime/autoread";
/// settings.json 내 elf 소유 훅 판별 문자열 (tsa HOOK_MARKER 동형 — 이 문자열을 품은 항목만 교체).
const HOOK_CMD_MARKER: &str = "elf autoread hook";
/// 마커 유효 시간 — 초과분은 각 훅 호출 시 부수 gc (죽은 세션 잔재 정리).
const MARKER_TTL_SECS: u64 = 24 * 3600;
/// digest에 싣는 활성 세션 상한 (실측: Mastication 활성 27개 — 전수 주입은 토큰 과다. S031 t01)
const ACTIVE_LIMIT: usize = 5;
/// Handoff 절단 길이(chars) (실측: S200 Handoff 700자+ — 라우팅 정보로는 선두면 충분. S031 t01)
const HANDOFF_TRUNC: usize = 200;
/// fulltext 파일당 절단 길이(chars) — 대상 정본(수~수십 KB)은 통과, 오선언 폭주만 차단 (S031 t16).
const FULLTEXT_TRUNC: usize = 24_000;
/// config 키: 재구성 후 digest에 전문 포함할 정본 경로 배열 (opt-in — 라우팅 지식은 프로젝트 소유).
const FULLTEXT_KEY: &str = "autoread_fulltext";

#[derive(Debug)]
pub enum AutoreadError {
    Io(std::io::Error),
    Refuse(String),
}

impl std::fmt::Display for AutoreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoreadError::Io(e) => write!(f, "io: {e}"),
            AutoreadError::Refuse(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for AutoreadError {
    fn from(e: std::io::Error) -> Self {
        AutoreadError::Io(e)
    }
}

#[derive(Debug, Default)]
pub struct AutoreadReport {
    pub lines: Vec<String>,
    pub warnings: usize,
}

impl AutoreadReport {
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
    fn warn(&mut self, s: impl Into<String>) {
        self.lines.push(format!("warn: {}", s.into()));
        self.warnings += 1;
    }
}

// ── config (`.elf/config.json`) — tsa와 동형 패턴, 극성만 반대(부재 = on) ─────────

/// config `autoread` bool. **부재·파싱 실패 = true (default-on)** — 끄기는 명시적 false만.
pub fn is_enabled(root: &Path) -> bool {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("autoread").and_then(|b| b.as_bool()))
        .unwrap_or(true)
}

/// config `autoread_fulltext` — 재구성 후 전문 주입할 정본 경로 목록(루트 상대).
/// 부재·형식 불일치 = 빈 목록(주입 없음 — digest만). 스위치 bool과 별도 키:
/// enable/disable(`write_config_flag`)이 목록을 건드리지 않고, pre-fulltext 바이너리와도 호환.
pub fn fulltext_list(root: &Path) -> Vec<String> {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get(FULLTEXT_KEY).and_then(|a| a.as_array()).cloned())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// 선언 경로 안전 검사 — 루트 상대·트리 내부만. 절대 경로·루트 시작(`/`·`\`)·드라이브
/// prefix(`C:` — 셋 다 join이 루트를 대체/이탈)·`..` 성분 거부.
pub(crate) fn fulltext_path_ok(rel: &str) -> bool {
    let p = Path::new(rel);
    !p.is_absolute()
        && !p.has_root()
        && !p.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
}

fn write_config_flag(root: &Path, on: bool) -> Result<(), AutoreadError> {
    let p = root.join(".elf").join("config.json");
    let mut v: serde_json::Value = match fs::read_to_string(&p) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| AutoreadError::Refuse(format!(".elf/config.json malformed: {e}")))?,
        Err(_) => serde_json::json!({}),
    };
    let Some(obj) = v.as_object_mut() else {
        return Err(AutoreadError::Refuse(".elf/config.json is not a JSON object".into()));
    };
    obj.insert("autoread".into(), serde_json::Value::Bool(on));
    let mut out = serde_json::to_string_pretty(&v).expect("json");
    out.push('\n');
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&p, out)?;
    Ok(())
}

// ── Claude Code 어댑터 — `.claude/settings.json` hooks 병합 (비파괴) ───────────────
//
// 어댑터 테이블(t09): 1차 = Claude Code. Codex(`.codex/hooks.json` — Claude 스키마 명시 호환)·
// Gemini(`.gemini/settings.json` — PreCompress/BeforeAgent 매핑)는 실사용 확인 후 행 추가.

/// (이벤트명, matcher, 훅 command) — Claude Code hooks 스키마.
fn claude_hook_rows() -> [(&'static str, Option<&'static str>, String); 2] {
    [
        // compact 외 resume·clear도 컨텍스트 경계 — SessionStart 4-matcher 중 startup 제외
        // (신규 세션은 CLAUDE.md→@AGENTS.md 체인이 fresh 로드라 재정렬 대상 아님).
        ("SessionStart", Some("compact|resume|clear"), format!("{HOOK_CMD_MARKER} session-start")),
        ("UserPromptSubmit", None, format!("{HOOK_CMD_MARKER} prompt")),
    ]
}

/// hooks 병합 — 기존 설정 보존: `hooks.<event>` 배열에서 elf 소유 항목(command에
/// `elf autoread hook` 포함)만 최신 정의로 교체, 없으면 추가. 그 외 항목·키 불변(멱등).
/// 반환 = 파일 변경 여부. 파싱 실패 = Refuse(사용자 파일 훼손 금지 — 침묵 금지, S029).
pub fn install_hooks(root: &Path) -> Result<bool, AutoreadError> {
    let dir = root.join(".claude");
    let p = dir.join("settings.json");
    let mut v: serde_json::Value = match fs::read_to_string(&p) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| {
            AutoreadError::Refuse(format!(
                ".claude/settings.json malformed: {e} — fix it first (autoread hooks not installed)"
            ))
        })?,
        Err(_) => serde_json::json!({}),
    };
    let before = v.clone();
    let Some(obj) = v.as_object_mut() else {
        return Err(AutoreadError::Refuse(".claude/settings.json is not a JSON object".into()));
    };
    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let Some(hooks_obj) = hooks.as_object_mut() else {
        return Err(AutoreadError::Refuse(".claude/settings.json `hooks` is not an object".into()));
    };

    for (event, matcher, cmd) in claude_hook_rows() {
        let entry = serde_json::json!({
            "matcher": matcher,
            "hooks": [{ "type": "command", "command": cmd }],
        });
        // matcher가 None이면 필드 자체를 생략 (Claude Code: matcher 생략 = 전체 매칭)
        let entry = match matcher {
            Some(_) => entry,
            None => serde_json::json!({ "hooks": [{ "type": "command", "command": cmd }] }),
        };
        let arr = hooks_obj
            .entry(event)
            .or_insert_with(|| serde_json::json!([]));
        let Some(list) = arr.as_array_mut() else {
            return Err(AutoreadError::Refuse(format!(
                ".claude/settings.json `hooks.{event}` is not an array"
            )));
        };
        match list.iter_mut().find(|item| is_ours(item)) {
            Some(existing) => *existing = entry,
            None => list.push(entry),
        }
    }

    if v == before {
        return Ok(false);
    }
    fs::create_dir_all(&dir)?;
    let mut out = serde_json::to_string_pretty(&v).expect("json");
    out.push('\n');
    fs::write(&p, out)?;
    Ok(true)
}

/// 항목 소유 판별 — 내부 hooks 배열의 command 중 하나라도 elf autoread 호출이면 elf 소유.
fn is_ours(item: &serde_json::Value) -> bool {
    item.get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.contains(HOOK_CMD_MARKER))
            })
        })
        .unwrap_or(false)
}

/// 설치 상태 — (이벤트명, 설치 여부) 목록. status·doctor 공용.
pub fn hook_states(root: &Path) -> Vec<(String, bool)> {
    let text = fs::read_to_string(root.join(".claude").join("settings.json")).unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));
    claude_hook_rows()
        .iter()
        .map(|(event, _, _)| {
            let installed = v
                .get("hooks")
                .and_then(|h| h.get(*event))
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().any(is_ours))
                .unwrap_or(false);
            ((*event).to_string(), installed)
        })
        .collect()
}

// ── enable / disable / status / ack ───────────────────────────────────────────

/// `elf autoread enable` — 멱등: ① config `"autoread": true`(.elf에 기록 — 스위치 정본)
/// ② Claude Code 훅 병합 ③ 안내. 프로젝트 단위(root 기준 — 프로젝트별 독립).
pub fn run_enable(root: &Path) -> Result<AutoreadReport, AutoreadError> {
    let mut r = AutoreadReport::default();
    write_config_flag(root, true)?;
    r.say("config: autoread = true (.elf/config.json — per-project switch)");
    match install_hooks(root) {
        Ok(true) => r.say("hooks: .claude/settings.json updated (SessionStart[compact|resume|clear] + UserPromptSubmit)"),
        Ok(false) => r.say("hooks: already installed (no change)"),
        Err(AutoreadError::Refuse(e)) => r.warn(e),
        Err(e) => return Err(e),
    }
    r.say("enabled. After a context reconstruction (compact/resume/clear), the governance digest is injected on the next prompt; every elf command also shows a one-line banner until then.");
    Ok(r)
}

/// `elf autoread disable` — config false 기록. 훅은 잔존(매 호출 config 게이트로 no-op) —
/// settings.json 반복 편집 회피. 완전 제거가 필요하면 사용자가 해당 항목을 직접 삭제.
pub fn run_disable(root: &Path) -> Result<AutoreadReport, AutoreadError> {
    let mut r = AutoreadReport::default();
    write_config_flag(root, false)?;
    r.say("config: autoread = false (.elf/config.json)");
    r.say("hooks kept in .claude/settings.json but now no-op (config gate) — re-enable with `elf autoread enable`");
    let cleared = clear_markers(root);
    if cleared > 0 {
        r.say(format!("cleared {cleared} pending marker(s)"));
    }
    Ok(r)
}

/// `elf autoread status` — config·훅·대기 마커. 읽기전용.
pub fn run_status(root: &Path) -> Result<AutoreadReport, AutoreadError> {
    let mut r = AutoreadReport::default();
    let explicit = fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("autoread").and_then(|b| b.as_bool()));
    let state = match explicit {
        Some(true) => "enabled (config: true)",
        Some(false) => "disabled (config: false — `elf autoread enable` to turn on)",
        None => "enabled (default-on — no config key; `elf autoread disable` to turn off)",
    };
    r.say(format!("autoread: {state}"));
    for (event, installed) in hook_states(root) {
        r.say(format!(
            "hook {event}: {}",
            if installed { "installed" } else { "absent — run `elf update` or `elf autoread enable`" }
        ));
    }
    let declared = fulltext_list(root);
    if declared.is_empty() {
        r.say(format!(
            "fulltext: none declared — add root-relative paths to `{FULLTEXT_KEY}` in .elf/config.json to inject canonical rules in full after a reconstruction"
        ));
    } else {
        for rel in &declared {
            let state = if !fulltext_path_ok(rel) {
                "UNSAFE PATH (skipped)"
            } else if root.join(rel).is_file() {
                "ok"
            } else {
                "MISSING"
            };
            r.say(format!("fulltext: {rel} — {state}"));
        }
    }
    let pending = list_markers(root).len();
    r.say(format!("pending markers: {pending}"));
    Ok(r)
}

/// `elf autoread ack` — 대기 마커 전체 해제(수동).
pub fn run_ack(root: &Path) -> Result<AutoreadReport, AutoreadError> {
    let mut r = AutoreadReport::default();
    let n = clear_markers(root);
    r.say(format!("cleared {n} pending marker(s)"));
    Ok(r)
}

// ── 마커 (`.elf/runtime/autoread/<session_id>`) ────────────────────────────────

#[derive(Serialize, Deserialize)]
struct Marker {
    source: String,
    ts: u64,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// session_id → 마커 파일 경로. 영숫자·`-`·`_` 외 문자는 `_`로 치환(경로 안전).
fn marker_path(root: &Path, session_id: &str) -> PathBuf {
    let safe: String = session_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let safe = if safe.is_empty() { "unknown".to_string() } else { safe };
    root.join(RUNTIME_DIR).join(safe)
}

fn write_marker(root: &Path, session_id: &str, source: &str) -> Result<(), AutoreadError> {
    let p = marker_path(root, session_id);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    let m = Marker { source: source.to_string(), ts: now_secs() };
    fs::write(&p, serde_json::to_string(&m).expect("marker json"))?;
    Ok(())
}

fn read_marker(root: &Path, session_id: &str) -> Option<Marker> {
    let text = fs::read_to_string(marker_path(root, session_id)).ok()?;
    serde_json::from_str(&text).ok()
}

fn list_markers(root: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(rd) = fs::read_dir(root.join(RUNTIME_DIR)) {
        for e in rd.flatten() {
            if e.path().is_file() {
                v.push(e.path());
            }
        }
    }
    v
}

fn clear_markers(root: &Path) -> usize {
    let mut n = 0;
    for p in list_markers(root) {
        if fs::remove_file(&p).is_ok() {
            n += 1;
        }
    }
    n
}

/// TTL 초과·판독 불가 마커 정리 — 각 훅 호출 시 부수 실행(죽은 세션 잔재).
fn gc_markers(root: &Path) {
    let now = now_secs();
    for p in list_markers(root) {
        let stale = fs::read_to_string(&p)
            .ok()
            .and_then(|t| serde_json::from_str::<Marker>(&t).ok())
            .map(|m| now.saturating_sub(m.ts) > MARKER_TTL_SECS)
            .unwrap_or(true); // 판독 불가 = stale
        if stale {
            let _ = fs::remove_file(&p);
        }
    }
}

// ── 훅 진입점 (fail-open — 훅 경로는 절대 세션을 막지 않는다) ────────────────────

/// `elf autoread hook <event>` 본체. stdin JSON(session_id·source…)을 받아
/// session-start = 마커 기록(무출력) / prompt = 마커 있으면 digest 반환(stdout 주입)+해제.
/// 모든 실패는 None(무출력·exit 0) — fail-open.
pub fn run_hook(root: &Path, event: &str, stdin_json: &str) -> Option<String> {
    if !is_enabled(root) {
        return None;
    }
    gc_markers(root);
    let v: serde_json::Value = serde_json::from_str(stdin_json).unwrap_or(serde_json::json!({}));
    let session_id = v.get("session_id").and_then(|s| s.as_str()).unwrap_or("unknown");
    match event {
        "session-start" => {
            let source = v.get("source").and_then(|s| s.as_str()).unwrap_or("");
            // matcher가 이미 거르지만 방어적 재검사 (설정 드리프트·수동 호출 대비)
            if matches!(source, "compact" | "resume" | "clear") {
                let _ = write_marker(root, session_id, source);
            }
            None
        }
        "prompt" => {
            let m = read_marker(root, session_id)?;
            let digest = build_digest(root, &m.source);
            let _ = fs::remove_file(marker_path(root, session_id));
            Some(digest)
        }
        _ => None,
    }
}

/// 배너 — 마커가 하나라도 대기 중이면 1행 반환. main이 (autoread hook·quiet 경로 제외)
/// 모든 명령 출력 선두에 부착: 프롬프트 미경유 경로(auto-compact 직후 연속 작업)의 보조 주입 채널.
pub fn pending_banner(cwd: &Path) -> Option<String> {
    let root = crate::update::find_project_root(cwd)?;
    if !is_enabled(&root) {
        return None;
    }
    let n = list_markers(&root).len();
    if n == 0 {
        return None;
    }
    Some(
        "autoread: context was reconstructed — run `elf autoread` for the governance digest (re-read the active session Handoff before substantive work)"
            .to_string(),
    )
}

// ── digest (정본 실행 시점 추출 — 하드코딩 없음) ──────────────────────────────────

/// 거버넌스 digest 조립. 원천: 루트 `AGENTS.md` 상시 의무 절 + 활성 세션 Handoff(2_Log 스캔) +
/// validate 카운트. 총량 목표 ≤ ~1.5k tok — compact 직후 컨텍스트 압박 시점(전량 정본 미주입).
pub fn build_digest(root: &Path, source: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "[elf autoread] context was reconstructed ({source}) — the summary you were given is a lossy digest, not the canonical rules. Re-align before substantive work.\n"
    ));

    if let Some(duties) = agents_duties(root) {
        out.push_str("\n--- AGENTS.md standing duties (canonical digest) ---\n");
        out.push_str(duties.trim_end());
        out.push('\n');
    } else {
        out.push_str("(AGENTS.md standing-duties section not found — read AGENTS.md directly)\n");
    }

    let sessions = active_session_handoffs(root);
    if !sessions.is_empty() {
        out.push_str("\n--- active sessions (newest first; Handoff truncated) ---\n");
        let shown = sessions.len().min(ACTIVE_LIMIT);
        for (id, handoff) in sessions.iter().take(ACTIVE_LIMIT) {
            let h: String = handoff.chars().take(HANDOFF_TRUNC).collect();
            let ellipsis = if handoff.chars().count() > HANDOFF_TRUNC { "…" } else { "" };
            out.push_str(&format!("{id}: {h}{ellipsis}\n"));
        }
        if sessions.len() > shown {
            out.push_str(&format!("(+{} more active)\n", sessions.len() - shown));
        }
    }

    if let Ok(rep) = crate::validate::run_validate(root) {
        out.push_str(&format!(
            "\nvalidate: {} issue(s), {} warning(s){}\n",
            rep.issues,
            rep.warnings,
            if rep.issues > 0 { " — run `elf validate` and resolve" } else { "" }
        ));
    }

    // 전문(full-text) layer (S031 t16, ContextReanchor 개정판): 프로젝트가 config에 선언한
    // 정본만 전문 주입 — 라우팅 지식은 프로젝트 소유, core는 전달만. 미선언 = digest만(현행).
    let injected_fulltext = append_fulltext(root, &mut out);

    // 말미 지시 — 명령형(산 증거: soft "re-align"만으로는 전문 재독 미달).
    if injected_fulltext {
        out.push_str(
            "\nnext: the full text of the declared canonical rules is included above — apply them; do not act on the compact summary alone. Before your first substantive action, also read the full header (Handoff) of the session log you are working in; keep Phase discipline (LogConvention §5.1) and embed figures immediately (§2).\n",
        );
    } else {
        out.push_str(
            "\nnext: before your first substantive action (1) read the full header (Handoff) of the session log you are working in, and (2) Read in full the task-relevant canonical rule documents under `0_Meta/` (follow the project's routing rule if it defines one); keep Phase discipline (LogConvention §5.1) and embed figures immediately (§2).\n",
        );
    }
    out
}

/// config `autoread_fulltext` 선언 정본의 전문 블록을 out에 부착. 반환 = 실제 전문을 1건이라도
/// 실었는지. 선언 오류(트리 이탈 경로·판독 불가)는 1행 표기 후 계속 — fail-open.
fn append_fulltext(root: &Path, out: &mut String) -> bool {
    let declared = fulltext_list(root);
    let mut injected = false;
    for rel in &declared {
        if !fulltext_path_ok(rel) {
            out.push_str(&format!("\n(fulltext skipped — path escapes the project tree: {rel})\n"));
            continue;
        }
        match fs::read_to_string(root.join(rel)) {
            Ok(text) => {
                out.push_str(&format!("\n--- full text: {rel} (declared in .elf/config.json {FULLTEXT_KEY}) ---\n"));
                let total = text.chars().count();
                if total > FULLTEXT_TRUNC {
                    let cut: String = text.chars().take(FULLTEXT_TRUNC).collect();
                    out.push_str(cut.trim_end());
                    out.push_str(&format!(
                        "\n…[truncated at {FULLTEXT_TRUNC} chars ({total} total) — Read the file directly for the remainder]\n"
                    ));
                } else {
                    out.push_str(text.trim_end());
                    out.push('\n');
                }
                injected = true;
            }
            Err(_) => {
                out.push_str(&format!("\n(fulltext declared but unreadable: {rel} — fix {FULLTEXT_KEY} in .elf/config.json)\n"));
            }
        }
    }
    injected
}

/// 루트 `AGENTS.md`의 상시 의무 절(`## 상시 의무`(KO) / `## Standing duties`(EN)) 추출 —
/// 헤딩 라인 다음부터 다음 `## ` 전까지. 관리 파일이라 구조 안정(root/general 동일 헤딩).
fn agents_duties(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("AGENTS.md")).ok()?;
    let mut in_section = false;
    let mut body = String::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            if in_section {
                break;
            }
            if line.contains("상시 의무") || line.contains("Standing duties") {
                in_section = true;
            }
            continue;
        }
        if in_section {
            body.push_str(line);
            body.push('\n');
        }
    }
    let trimmed = body.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

/// 활성 세션 (id, Handoff) — 2_Log/*.md 헤더에서 Status가 Complete가 아닌 것, 번호 내림차순.
fn active_session_handoffs(root: &Path) -> Vec<(String, String)> {
    let dir = root.join("2_Log");
    let mut rows: Vec<(u32, String, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(num) = crate::session::log_num(&name) else { continue };
            let Ok(content) = fs::read_to_string(e.path()) else { continue };
            let Some(status) = crate::session::header_status(&content) else { continue };
            if !crate::session::is_open_status(&status) {
                continue;
            }
            let handoff = crate::session::header_handoff(&content).unwrap_or_else(|| "-".into());
            rows.push((num, format!("S{num:03}"), handoff));
        }
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.0));
    rows.into_iter().map(|(_, id, h)| (id, h)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_rows_cover_reconstruction_boundaries_not_startup() {
        let rows = claude_hook_rows();
        assert_eq!(rows[0].0, "SessionStart");
        let m = rows[0].1.expect("matcher");
        for s in ["compact", "resume", "clear"] {
            assert!(m.contains(s));
        }
        assert!(!m.contains("startup"));
        assert_eq!(rows[1].0, "UserPromptSubmit");
        assert!(rows[1].1.is_none()); // 전체 매칭 — matcher 생략
    }

    #[test]
    fn marker_path_sanitizes_session_id() {
        let root = Path::new("/proj");
        let p = marker_path(root, "abc-123_X");
        assert!(p.ends_with(PathBuf::from(RUNTIME_DIR).join("abc-123_X")));
        let evil = marker_path(root, "../../etc/passwd");
        let name = evil.file_name().unwrap().to_string_lossy().into_owned();
        assert!(!name.contains('/') && !name.contains('.') && !name.contains('\\'));
        assert!(marker_path(root, "").file_name().is_some());
    }

    #[test]
    fn fulltext_path_rejects_tree_escape() {
        for bad in ["../secret.md", "a/../../b.md", "/etc/passwd", "\\root.md", "C:\\x.md", "C:x.md"] {
            assert!(!fulltext_path_ok(bad), "should reject: {bad}");
        }
        for good in ["0_Meta/LogConvention.md", "AGENTS.md", "a/b/c.md", "a/./b.md"] {
            assert!(fulltext_path_ok(good), "should accept: {good}");
        }
    }

    #[test]
    fn fulltext_truncates_over_cap_and_keeps_small_files_whole() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".elf")).unwrap();
        fs::write(
            tmp.path().join(".elf/config.json"),
            format!(r#"{{"{FULLTEXT_KEY}": ["small.md", "big.md"]}}"#),
        )
        .unwrap();
        fs::write(tmp.path().join("small.md"), "tiny rule body").unwrap();
        fs::write(tmp.path().join("big.md"), "x".repeat(FULLTEXT_TRUNC + 100)).unwrap();
        let mut out = String::new();
        assert!(append_fulltext(tmp.path(), &mut out));
        assert!(out.contains("full text: small.md") && out.contains("tiny rule body"));
        assert!(out.contains("truncated at") && out.contains("Read the file directly"));
        // 절단본 길이 확인 — cap + 마커 여유 내
        assert!(out.chars().count() < FULLTEXT_TRUNC + 1000);
    }

    #[test]
    fn agents_duties_extracts_between_headings() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("AGENTS.md"),
            "# AGENTS\n\nintro\n\n## 상시 의무 (요약)\n\n- duty A\n- duty B\n\n## 소유권\n\n- other\n",
        )
        .unwrap();
        let d = agents_duties(tmp.path()).unwrap();
        assert!(d.contains("duty A") && d.contains("duty B"));
        assert!(!d.contains("other") && !d.contains("intro"));
        // EN 헤딩도 동일 경로
        fs::write(
            tmp.path().join("AGENTS.md"),
            "# t\n## Standing duties (digest)\n- duty EN\n## Ownership\n- x\n",
        )
        .unwrap();
        assert_eq!(agents_duties(tmp.path()).unwrap(), "- duty EN");
    }
}
