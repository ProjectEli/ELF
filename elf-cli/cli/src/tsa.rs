//! `elf tsa` — 연구 기록 저작 증명 opt-in feature (S022 — Mastication #1 역이식).
//!
//! 실소유 = manifest(무결성 기록) + RFC 3161 TSA 시점인증. GPG 커밋 서명은 git↔gpg-agent
//! 소유라 여기서 다루지 않음(enable 안내 + doctor 진단만) — 명령 이름(tsa)은 실소유 범위와 일치.
//!
//! 불변식(S022 t03): 기능 상태(config `tsa`·git hooks)는 가역 on/off — 증거(`0_Meta/tsa/`)는
//! 불가역 누적, disable도 데이터를 삭제하지 않는다. enable은 멱등.
//!
//! 해시 = 원시 바이트 sha256(정규화 없음) — 디스크 실물 그대로의 존재 증명.
//! (`hash::sha256_lf`의 CRLF 정규화는 mirror drift 판정용 — 목적 상이라 혼용 금지.)
//! 외부 전송 = manifest 파일의 sha256 해시 1건뿐(TSQ 59바이트) — 파일 내용·이름 비전송.
//! manifest 스키마·TSQ 파라미터(sha256·no-nonce·certReq)는 Mastication 실물과 동일 → 기존
//! `0_Meta/manifests/`·`timestamps/` 증거와 상호 검증 호환.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 증거 디렉토리(사용자 소유 tier — update가 건드리지 않음). manifest와 .tsr 동거(날짜 쌍 인접).
pub const TSA_DIR: &str = "0_Meta/tsa";
/// 기본 TSA(무인증 공개 엔드포인트). `.elf/config.json` `tsaUrl`로 교체 가능 — 코드에 크리덴셜 금지.
pub const DEFAULT_TSA_URL: &str = "http://timestamp.digicert.com";
/// 훅 소유 판별 마커 — 이 마커가 있는 훅 파일만 elf가 교체·삭제한다(비파괴 원칙).
const HOOK_MARKER: &str = "# elf-tsa hook";

#[derive(Debug)]
pub enum TsaError {
    Io(std::io::Error),
    Git(String),
    Http(String),
    Refuse(String),
}

impl std::fmt::Display for TsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsaError::Io(e) => write!(f, "io: {e}"),
            TsaError::Git(e) => write!(f, "git: {e}"),
            TsaError::Http(e) => write!(f, "http: {e}"),
            TsaError::Refuse(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for TsaError {
    fn from(e: std::io::Error) -> Self {
        TsaError::Io(e)
    }
}

#[derive(Debug, Default)]
pub struct TsaReport {
    pub lines: Vec<String>,
    pub warnings: usize,
}

impl TsaReport {
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
    fn warn(&mut self, s: impl Into<String>) {
        self.lines.push(format!("warn: {}", s.into()));
        self.warnings += 1;
    }
}

// ── manifest 스키마 (Mastication 호환 — 필드명·구조 동일) ──────────────────────────

#[derive(Serialize, Deserialize)]
struct TsaManifest {
    date: String,
    entries: Vec<TsaEntry>,
}

#[derive(Serialize, Deserialize)]
struct TsaEntry {
    file: String,
    sha256: String,
    timestamp: String,
}

// ── config (`.elf/config.json`) — preset(v2.16.2)과 동형 패턴 ─────────────────────

/// config `tsa` bool. 부재·파싱 실패 = false (opt-in 기본 off).
pub fn is_enabled(root: &Path) -> bool {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("tsa").and_then(|b| b.as_bool()))
        .unwrap_or(false)
}

fn read_config_url(root: &Path) -> String {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("tsaUrl").and_then(|u| u.as_str()).map(String::from))
        .unwrap_or_else(|| DEFAULT_TSA_URL.to_string())
}

fn write_config_flag(root: &Path, on: bool) -> Result<(), TsaError> {
    let p = root.join(".elf").join("config.json");
    let mut v: serde_json::Value = match fs::read_to_string(&p) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| TsaError::Refuse(format!(".elf/config.json malformed: {e}")))?,
        Err(_) => serde_json::json!({}),
    };
    let Some(obj) = v.as_object_mut() else {
        return Err(TsaError::Refuse(".elf/config.json is not a JSON object".into()));
    };
    obj.insert("tsa".into(), serde_json::Value::Bool(on));
    let mut out = serde_json::to_string_pretty(&v).expect("json");
    out.push('\n');
    fs::write(&p, out)?;
    Ok(())
}

// ── git 유틸 ───────────────────────────────────────────────────────────────────

fn git(root: &Path, args: &[&str]) -> Result<String, TsaError> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| TsaError::Git(format!("cannot run git: {e}")))?;
    if !out.status.success() {
        return Err(TsaError::Git(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// hooks 디렉토리 해석 — `git rev-parse --git-path hooks`(worktree-safe: 공용 gitdir로 풀림).
fn hooks_dir(root: &Path) -> Result<PathBuf, TsaError> {
    let out = git(root, &["rev-parse", "--git-path", "hooks"])?;
    let p = PathBuf::from(out.trim());
    Ok(if p.is_absolute() { p } else { root.join(p) })
}

// ── 훅 (비파괴 — S022 t03 사용자 승인) ─────────────────────────────────────────────

/// 훅 본문 — elf 호출 1줄(로직은 바이너리 소유). `$HOME/.elf/bin` 우선(훅 환경 PATH 빈약 대비),
/// 비차단(`|| true` — 커밋을 절대 막지 않음), stamp는 백그라운드(TSA 응답 지연이 커밋 UX 비침범).
fn hook_body(subcmd: &str, background: bool) -> String {
    let call = format!("\"$ELF\" tsa {subcmd} --quiet");
    let call = if background {
        format!("({call} >/dev/null 2>&1 || true) &")
    } else {
        format!("{call} || true")
    };
    format!(
        "#!/bin/sh\n{HOOK_MARKER} — managed by `elf tsa enable`; removed by `elf tsa disable`. Do not edit (replaced wholesale).\nELF=\"$HOME/.elf/bin/elf\"; {{ [ -x \"$ELF\" ] || [ -x \"$ELF.exe\" ]; }} || ELF=elf\n{call}\n"
    )
}

/// 훅 1개 설치: 부재 → 생성 / elf 마커 → 갱신(멱등) / 타 훅 → 비파괴: 손대지 않고 추가 안내.
fn install_hook(dir: &Path, name: &str, body: &str, r: &mut TsaReport) -> Result<(), TsaError> {
    let p = dir.join(name);
    match fs::read_to_string(&p) {
        Ok(existing) if !existing.contains(HOOK_MARKER) => {
            let line = body.lines().last().unwrap_or_default();
            r.warn(format!(
                "{name} hook exists and is not elf-tsa's — left untouched. Add these lines to it manually:\n         ELF=\"$HOME/.elf/bin/elf\"; {{ [ -x \"$ELF\" ] || [ -x \"$ELF.exe\" ]; }} || ELF=elf\n         {line}"
            ));
            return Ok(());
        }
        _ => {}
    }
    fs::create_dir_all(dir)?;
    fs::write(&p, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755))?;
    }
    r.say(format!("hook installed: {name} (marker-owned, replaceable)"));
    Ok(())
}

/// 훅 1개 제거 — elf 마커가 있는 파일만(타 훅 보존).
fn remove_hook(dir: &Path, name: &str, r: &mut TsaReport) {
    let p = dir.join(name);
    match fs::read_to_string(&p) {
        Ok(existing) if existing.contains(HOOK_MARKER) => {
            if fs::remove_file(&p).is_ok() {
                r.say(format!("hook removed: {name}"));
            } else {
                r.warn(format!("could not remove hook {name} — remove it manually"));
            }
        }
        Ok(_) => r.say(format!(
            "hook kept: {name} is not elf-tsa's — remove the `elf tsa` line manually if you added one"
        )),
        Err(_) => {}
    }
}

// ── enable / disable / status ─────────────────────────────────────────────────

/// `elf tsa enable` — 멱등: ① config on ② 증거 디렉토리 ③ 훅 2종(비파괴) ④ GPG 안내(비차단)
/// ⑤ baseline: git 추적 전체 record + stamp 시도(과거 소급 증명은 원리적 불가 —
/// "도입 시점까지의 존재" 기준선. 오프라인이면 stamp만 미뤄지고 `--backfill`로 보충).
pub fn run_enable(root: &Path) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    git(root, &["rev-parse", "--show-toplevel"])
        .map_err(|_| TsaError::Refuse("not a git repository — `elf tsa` needs git (commits are the recording unit)".into()))?;

    write_config_flag(root, true)?;
    r.say("config: tsa = true (.elf/config.json)");
    fs::create_dir_all(root.join(TSA_DIR))?;
    r.say(format!("evidence dir: {TSA_DIR}/ (append-only — never auto-deleted)"));

    let hooks = hooks_dir(root)?;
    install_hook(&hooks, "pre-commit", &hook_body("record --staged", false), &mut r)?;
    install_hook(&hooks, "post-commit", &hook_body("stamp", true), &mut r)?;

    // GPG는 git 소유 — 진단·안내만 (차단 없음: manifest+TSA만으로도 무결성+시점은 성립)
    let gpg_on = git(root, &["config", "--get", "commit.gpgsign"])
        .map(|v| v.trim() == "true")
        .unwrap_or(false);
    if gpg_on {
        r.say("gpg: commit.gpgsign = true (authorship layer active)");
    } else {
        r.say("gpg: commit signing off — optional authorship layer: `git config commit.gpgsign true` + `git config user.signingkey <KEY>`");
    }

    // baseline seal — 도입 시점 스냅샷 1회
    let rec = run_record(root, RecordScope::All)?;
    r.lines.extend(rec.lines);
    r.warnings += rec.warnings;
    match run_stamp(root, false) {
        Ok(st) => {
            r.lines.extend(st.lines);
            r.warnings += st.warnings;
        }
        Err(e) => r.warn(format!("baseline stamp deferred ({e}) — run `elf tsa stamp --backfill` when online")),
    }
    r.say("enabled. Every commit now records hashes (pre-commit) and stamps the day's first commit (post-commit). Only a 32-byte digest ever leaves this machine.");
    Ok(r)
}

/// `elf tsa disable` — config off + elf 마커 훅만 제거. 증거는 그대로(재enable 시 이어서 누적).
pub fn run_disable(root: &Path) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    write_config_flag(root, false)?;
    r.say("config: tsa = false");
    if let Ok(hooks) = hooks_dir(root) {
        remove_hook(&hooks, "pre-commit", &mut r);
        remove_hook(&hooks, "post-commit", &mut r);
    }
    r.say(format!("evidence kept: {TSA_DIR}/ untouched — re-enable resumes on top of it"));
    Ok(r)
}

/// `elf tsa status` — enabled·훅·증거 카운트·미제출. 읽기전용.
pub fn run_status(root: &Path) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    let enabled = is_enabled(root);
    r.say(format!("tsa: {}", if enabled { "enabled" } else { "disabled (opt-in — `elf tsa enable`)" }));
    if let Ok(hooks) = hooks_dir(root) {
        for name in ["pre-commit", "post-commit"] {
            let state = match fs::read_to_string(hooks.join(name)) {
                Ok(t) if t.contains(HOOK_MARKER) => "elf-tsa",
                Ok(_) => "foreign (add the elf tsa line manually — see `elf tsa enable` output)",
                Err(_) => "absent",
            };
            r.say(format!("hook {name}: {state}"));
        }
    }
    let (manifests, stamped, unstamped) = evidence_counts(root);
    r.say(format!("evidence: {manifests} manifest(s), {stamped} stamped, {unstamped} unstamped"));
    if enabled && unstamped > 0 {
        r.warn(format!("{unstamped} manifest(s) without a timestamp token — run `elf tsa stamp --backfill`"));
    }
    Ok(r)
}

/// (manifest 수, tsr 짝 있는 수, 없는 수) — status·doctor 공용.
pub fn evidence_counts(root: &Path) -> (usize, usize, usize) {
    let dir = root.join(TSA_DIR);
    let mut manifests = 0usize;
    let mut stamped = 0usize;
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(date) = name.strip_suffix("_manifest.json") {
                manifests += 1;
                if dir.join(format!("{date}.tsr")).is_file() {
                    stamped += 1;
                }
            }
        }
    }
    (manifests, stamped, manifests - stamped)
}

// ── record ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordScope {
    /// pre-commit: staged 파일만
    Staged,
    /// baseline: git 추적 전체
    All,
}

/// 당일 manifest에 파일 해시 append (원시 바이트 sha256). (file, sha256) 중복은 skip(멱등).
/// manifest 자신은 git add(커밋에 봉인 동승 — Mastication pre-commit 동작 승계).
pub fn run_record(root: &Path, scope: RecordScope) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    let listing = match scope {
        RecordScope::Staged => git(root, &["diff", "--cached", "--name-only", "--diff-filter=ACMR"])?,
        RecordScope::All => git(root, &["ls-files"])?,
    };
    let files: Vec<&str> = listing.lines().filter(|l| !l.trim().is_empty()).collect();
    if files.is_empty() {
        r.say("nothing to record");
        return Ok(r);
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let now = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
    let dir = root.join(TSA_DIR);
    fs::create_dir_all(&dir)?;
    let mpath = dir.join(format!("{today}_manifest.json"));
    let mut m: TsaManifest = match fs::read_to_string(&mpath) {
        Ok(t) => serde_json::from_str(&t)
            .map_err(|e| TsaError::Refuse(format!("{} malformed: {e} — fix or move it aside", mpath.display())))?,
        Err(_) => TsaManifest { date: today.clone(), entries: Vec::new() },
    };

    let self_rel = format!("{TSA_DIR}/{today}_manifest.json");
    let mut added = 0usize;
    for rel in files {
        if rel == self_rel {
            continue; // 자기 참조 배제 — 기록하는 순간 자신이 변해 즉시 낡는 해시(전날 manifest는 체인이라 허용)
        }
        let p = root.join(rel);
        let Ok(bytes) = fs::read(&p) else { continue }; // 삭제 예정·비파일은 조용히 skip
        let mut h = Sha256::new();
        h.update(&bytes);
        let sha = format!("{:x}", h.finalize());
        if m.entries.iter().any(|e| e.file == rel && e.sha256 == sha) {
            continue;
        }
        m.entries.push(TsaEntry { file: rel.to_string(), sha256: sha, timestamp: now.clone() });
        added += 1;
    }
    if added > 0 {
        let mut out = serde_json::to_string_pretty(&m).expect("json");
        out.push('\n');
        fs::write(&mpath, out)?;
        // manifest 자신을 stage — 실패는 무해(다음 커밋에 포함)
        let rel = format!("{TSA_DIR}/{today}_manifest.json");
        let _ = git(root, &["add", &rel]);
        r.say(format!("recorded {added} file(s) → {rel}"));
    } else {
        r.say("recorded 0 file(s) (all already in today's manifest)");
    }
    Ok(r)
}

// ── stamp (RFC 3161) ──────────────────────────────────────────────────────────

/// TimeStampReq DER — 고정 59바이트(sha256·no-nonce·certReq TRUE). openssl
/// `ts -query -sha256 -no_nonce -cert`와 동일 구조(Mastication 파라미터 승계 — 증거 동질).
pub fn build_tsq(hash: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(59);
    v.extend_from_slice(&[0x30, 0x39]); // TimeStampReq SEQUENCE (57)
    v.extend_from_slice(&[0x02, 0x01, 0x01]); // version INTEGER 1
    v.extend_from_slice(&[0x30, 0x31]); // messageImprint SEQUENCE (49)
    v.extend_from_slice(&[0x30, 0x0d]); // AlgorithmIdentifier SEQUENCE (13)
    v.extend_from_slice(&[0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01]); // OID sha256
    v.extend_from_slice(&[0x05, 0x00]); // parameters NULL
    v.extend_from_slice(&[0x04, 0x20]); // hashedMessage OCTET STRING (32)
    v.extend_from_slice(hash);
    v.extend_from_slice(&[0x01, 0x01, 0xff]); // certReq BOOLEAN TRUE
    v
}

/// TSR 최소 판독: PKIStatus granted(0)/grantedWithMods(1) 여부.
/// TSR = SEQUENCE { status PKIStatusInfo SEQUENCE { INTEGER status … } … } — 선두 고정 구조만 파싱.
fn tsr_granted(tsr: &[u8]) -> bool {
    // 0x30 <len…> 0x30 <len…> 0x02 0x01 <status> — DER 길이는 1~n바이트: 최소 스캔으로 status INTEGER 탐색
    let mut i = 0usize;
    let mut depth = 0;
    while i + 2 < tsr.len() && depth < 2 {
        if tsr[i] != 0x30 {
            return false;
        }
        // 길이 필드 건너뛰기
        i += 1;
        if tsr[i] & 0x80 != 0 {
            i += 1 + (tsr[i] & 0x7f) as usize;
        } else {
            i += 1;
        }
        depth += 1;
    }
    tsr.len() >= i + 3 && tsr[i] == 0x02 && tsr[i + 1] == 0x01 && (tsr[i + 2] == 0 || tsr[i + 2] == 1)
}

/// TSR에서 genTime(GeneralizedTime `YYYYMMDDHHMMSSZ`) 추출 — 태그 0x18 스캔(경량).
fn tsr_gen_time(tsr: &[u8]) -> Option<String> {
    for i in 0..tsr.len().saturating_sub(17) {
        if tsr[i] == 0x18 && tsr[i + 1] == 0x0f {
            let s = &tsr[i + 2..i + 17];
            if s.iter().all(|b| b.is_ascii_digit() || *b == b'Z') && s[14] == b'Z' {
                let t = std::str::from_utf8(s).ok()?;
                return Some(format!(
                    "{}-{}-{} {}:{}:{} UTC",
                    &t[0..4], &t[4..6], &t[6..8], &t[8..10], &t[10..12], &t[12..14]
                ));
            }
        }
    }
    None
}

fn sha256_file(p: &Path) -> Result<[u8; 32], TsaError> {
    let bytes = fs::read(p)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(h.finalize().into())
}

/// `elf tsa stamp [--backfill]` — .tsr 없는 manifest에 TSA 시점토큰 요청·저장.
/// 기본 = 당일 manifest만(훅 경로), backfill = 전체(오프라인 누락 소급 — Mastication 알려진 제약 해소).
/// 저장 전 granted 판독 — 깨진 응답을 증거로 남기지 않음.
pub fn run_stamp(root: &Path, backfill: bool) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    let dir = root.join(TSA_DIR);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut targets: Vec<String> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(date) = name.strip_suffix("_manifest.json") {
                if !dir.join(format!("{date}.tsr")).is_file() && (backfill || date == today) {
                    targets.push(date.to_string());
                }
            }
        }
    }
    if targets.is_empty() {
        r.say("nothing to stamp (no unstamped manifest in scope)");
        return Ok(r);
    }
    targets.sort();
    let url = read_config_url(root);
    for date in targets {
        let mpath = dir.join(format!("{date}_manifest.json"));
        let hash = sha256_file(&mpath)?;
        let tsq = build_tsq(&hash);
        match post_tsq(&url, &tsq) {
            Ok(tsr) if tsr_granted(&tsr) => {
                let tpath = dir.join(format!("{date}.tsr"));
                fs::write(&tpath, &tsr)?;
                let when = tsr_gen_time(&tsr).unwrap_or_else(|| "time unparsed".into());
                r.say(format!("stamped {date} ({when}) → {TSA_DIR}/{date}.tsr"));
                let _ = git(root, &["add", &format!("{TSA_DIR}/{date}.tsr")]);
            }
            Ok(_) => r.warn(format!("{date}: TSA response not granted — token discarded, retry later")),
            Err(e) => r.warn(format!("{date}: {e} — retry with `elf tsa stamp --backfill` when online")),
        }
    }
    Ok(r)
}

/// TSQ HTTP POST — RFC 3161 media type. 토큰 자체가 서명이라 채널 신뢰 불요(http 허용),
/// https TSA도 config 교체로 동작(ureq/rustls).
fn post_tsq(url: &str, tsq: &[u8]) -> Result<Vec<u8>, TsaError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(20)))
        .build()
        .into();
    let mut resp = agent
        .post(url)
        .header("Content-Type", "application/timestamp-query")
        .send(tsq)
        .map_err(|e| TsaError::Http(format!("TSA unreachable ({e})")))?;
    if resp.status() != 200 {
        return Err(TsaError::Http(format!("TSA returned HTTP {}", resp.status())));
    }
    resp.body_mut()
        .read_to_vec()
        .map_err(|e| TsaError::Http(format!("TSA response read failed ({e})")))
}

// ── verify ────────────────────────────────────────────────────────────────────

/// `elf tsa verify <file>` — 파일 원시 sha256을 전체 manifest에서 탐색(존재 이력 출력).
/// `elf tsa verify --date <D>` — manifest↔tsr 경량 검증(granted·messageImprint 대조·genTime)
/// + 엄밀 서명 체인 검증은 openssl 위임 안내(드문 경로 — S022 t04 단계 구성).
pub fn run_verify(root: &Path, file: Option<&str>, date: Option<&str>) -> Result<TsaReport, TsaError> {
    let mut r = TsaReport::default();
    let dir = root.join(TSA_DIR);
    match (file, date) {
        (Some(f), None) => {
            let target = Path::new(f);
            let target = if target.is_absolute() { target.to_path_buf() } else { root.join(target) };
            let hash = sha256_file(&target)?;
            let hex = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
            r.say(format!("sha256: {hex}"));
            let mut found = 0usize;
            let mut names: Vec<String> = Vec::new();
            if let Ok(rd) = fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if n.ends_with("_manifest.json") {
                        names.push(n);
                    }
                }
            }
            names.sort();
            for n in names {
                let Ok(text) = fs::read_to_string(dir.join(&n)) else { continue };
                let Ok(m) = serde_json::from_str::<TsaManifest>(&text) else { continue };
                for e in m.entries.iter().filter(|e| e.sha256 == hex) {
                    let date = n.strip_suffix("_manifest.json").unwrap_or(&n);
                    let stamped = if dir.join(format!("{date}.tsr")).is_file() { "stamped" } else { "unstamped" };
                    r.say(format!("found: {} @ {} ({} — {stamped})", e.file, e.timestamp, n));
                    found += 1;
                }
            }
            if found == 0 {
                r.warn("not found in any manifest — current content was never recorded (edited since, or recorded before enable)");
            }
        }
        (None, Some(d)) => {
            let mpath = dir.join(format!("{d}_manifest.json"));
            let tpath = dir.join(format!("{d}.tsr"));
            if !mpath.is_file() {
                return Err(TsaError::Refuse(format!("no manifest for {d}")));
            }
            if !tpath.is_file() {
                return Err(TsaError::Refuse(format!("no timestamp token for {d} — run `elf tsa stamp --backfill`")));
            }
            let hash = sha256_file(&mpath)?;
            let tsr = fs::read(&tpath)?;
            if !tsr_granted(&tsr) {
                r.warn("token status: not granted — token may be corrupt");
            } else {
                r.say("token status: granted");
            }
            if tsr.windows(32).any(|w| w == hash) {
                r.say("messageImprint: matches manifest sha256 (token is for this exact file)");
            } else {
                r.warn("messageImprint mismatch — manifest edited after stamping, or token belongs to another file");
            }
            if let Some(t) = tsr_gen_time(&tsr) {
                r.say(format!("genTime: {t}"));
            }
            r.say(format!(
                "lightweight check done. Full signature-chain verification (rare path — disputes/audits):\n       openssl ts -verify -data {} -in {} -CAfile <DigiCert-TSA-CA.pem>",
                mpath.display(),
                tpath.display()
            ));
        }
        _ => {
            return Err(TsaError::Refuse(
                "usage: `elf tsa verify <file>` or `elf tsa verify --date YYYY-MM-DD`".into(),
            ));
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 3161 TimeStampReq 골든 바이트 — openssl `ts -query -sha256 -no_nonce -cert`와 동일 구조.
    #[test]
    fn tsq_der_is_fixed_59_bytes_with_expected_frame() {
        let hash = [0xABu8; 32];
        let tsq = build_tsq(&hash);
        assert_eq!(tsq.len(), 59);
        assert_eq!(&tsq[..2], &[0x30, 0x39]); // outer SEQUENCE, len 57
        assert_eq!(&tsq[2..5], &[0x02, 0x01, 0x01]); // version 1
        assert_eq!(&tsq[5..7], &[0x30, 0x31]); // messageImprint, len 49
        assert_eq!(&tsq[7..9], &[0x30, 0x0d]); // AlgorithmIdentifier, len 13
        assert_eq!(&tsq[22..24], &[0x04, 0x20]); // OCTET STRING 32
        assert_eq!(&tsq[24..56], &[0xABu8; 32]); // hash payload
        assert_eq!(&tsq[56..], &[0x01, 0x01, 0xff]); // certReq TRUE
    }

    #[test]
    fn tsr_granted_parses_short_and_long_length_forms() {
        // 짧은 길이형: 30 06 30 04 02 01 00 (granted)
        assert!(tsr_granted(&[0x30, 0x06, 0x30, 0x04, 0x02, 0x01, 0x00]));
        // grantedWithMods(1)도 허용
        assert!(tsr_granted(&[0x30, 0x06, 0x30, 0x04, 0x02, 0x01, 0x01]));
        // rejection(2) 거부
        assert!(!tsr_granted(&[0x30, 0x06, 0x30, 0x04, 0x02, 0x01, 0x02]));
        // 긴 길이형(0x82 xx xx): 30 82 01 00 30 82 00 10 02 01 00
        assert!(tsr_granted(&[0x30, 0x82, 0x01, 0x00, 0x30, 0x82, 0x00, 0x10, 0x02, 0x01, 0x00]));
        // SEQUENCE 아님
        assert!(!tsr_granted(&[0x31, 0x06, 0x30, 0x04, 0x02, 0x01, 0x00]));
    }

    #[test]
    fn tsr_gen_time_extracts_generalized_time() {
        let mut blob = vec![0u8; 8];
        blob.extend_from_slice(&[0x18, 0x0f]);
        blob.extend_from_slice(b"20260715083000Z");
        blob.extend_from_slice(&[0x00, 0x00]);
        assert_eq!(tsr_gen_time(&blob).as_deref(), Some("2026-07-15 08:30:00 UTC"));
        assert_eq!(tsr_gen_time(&[0u8; 10]), None);
    }

    #[test]
    fn hook_body_is_marker_owned_nonblocking() {
        let b = hook_body("record --staged", false);
        assert!(b.contains(HOOK_MARKER));
        assert!(b.contains("|| true")); // 커밋 비차단
        assert!(b.starts_with("#!/bin/sh\n"));
        let bg = hook_body("stamp", true);
        assert!(bg.contains(") &")); // TSA 지연이 커밋 UX 비침범
    }
}
