//! 실행 위임(v3.0 S044 t59): 프로젝트 줄 ≠ 이 CLI → 보관소 버전으로 넘김(인수, 출력, 종료 코드), 런처 전용 명령,
//! 재귀 방지, 첫 사용 설치와 새 patch 받기, 구할 수 없을 때(쓰기 거부, 2.27 줄은 내장 이전 형식 모드).
//! 보관소의 "다른 버전" = 이 시험 빌드의 복사본(같은 코드가 `ELF_DISPATCHED`로 이 프로세스에서 실행).
//! `ELF_HOME`, `ELF_RELEASES` = 임시 폴더 — 네트워크와 실제 `~/.elf`를 쓰지 않음.

use elf_cli::init::{InitOptions, run_init};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

struct Env {
    home: PathBuf,
    rel: PathBuf,
}

fn run(dir: &Path, env: &Env, extra: &[(&str, &str)], args: &[&str]) -> (i32, String, String) {
    let mut c = assert_cmd::Command::cargo_bin("elf").unwrap();
    c.args(args).current_dir(dir).env("ELF_HOME", &env.home).env("ELF_RELEASES", &env.rel).env_remove("ELF_DISPATCHED").env_remove("ELF_DISPATCH");
    for (k, v) in extra {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

const TRACE: (&str, &str) = ("ELF_DISPATCH", "trace");
const OFF: (&str, &str) = ("ELF_DISPATCH", "off");

fn exe() -> &'static str {
    if cfg!(windows) { "elf.exe" } else { "elf" }
}

fn this_binary() -> Vec<u8> {
    fs::read(assert_cmd::cargo::cargo_bin("elf")).unwrap()
}

fn env(tmp: &Path) -> Env {
    let e = Env { home: tmp.join("home"), rel: tmp.join("rel") };
    fs::create_dir_all(&e.rel).unwrap();
    e
}

/// 보관소에 직접 넣음(받기 생략).
fn store_put(e: &Env, tag: &str, bin: &[u8]) {
    let d = e.home.join("versions").join(tag);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join(exe()), bin).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(d.join(exe()), fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn zip(name: &str, body: &[u8]) -> Vec<u8> {
    let mut out = vec![0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 0, 0];
    out.extend([0u8; 8]);
    out.extend((body.len() as u32).to_le_bytes());
    out.extend((body.len() as u32).to_le_bytes());
    out.extend((name.len() as u16).to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out.extend(name.as_bytes());
    out.extend(body);
    let at = out.len() as u32;
    let mut ch = vec![0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0];
    ch.extend([0u8; 8]);
    ch.extend((body.len() as u32).to_le_bytes());
    ch.extend((body.len() as u32).to_le_bytes());
    ch.extend((name.len() as u16).to_le_bytes());
    ch.extend([0u8; 12]);
    ch.extend(0u32.to_le_bytes());
    ch.extend(name.as_bytes());
    out.extend(&ch);
    out.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0]);
    out.extend((ch.len() as u32).to_le_bytes());
    out.extend(at.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

fn tar_xz(path: &str, body: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    let mut h = [0u8; 512];
    h[..path.len()].copy_from_slice(path.as_bytes());
    h[124..136].copy_from_slice(format!("{:011o}\0", body.len()).as_bytes());
    h[156] = b'0';
    h[257..263].copy_from_slice(b"ustar\0");
    t.extend(h);
    t.extend(body);
    t.extend(vec![0u8; body.len().div_ceil(512) * 512 - body.len() + 1024]);
    let mut w = lzma_rust2::XzWriter::new(Vec::new(), lzma_rust2::XzOptions::with_preset(0)).unwrap();
    w.write_all(&t).unwrap();
    w.finish().unwrap()
}

/// 가짜 Release 자산(이 플랫폼 형식)과 `.sha256`.
fn publish(e: &Env, tag: &str, bin: &[u8]) {
    let name = elf_cli::store::asset_name().unwrap();
    let data = if name.ends_with(".zip") { zip(exe(), bin) } else { tar_xz(&format!("{}/{}", name.trim_end_matches(".tar.xz"), exe()), bin) };
    let d = e.rel.join(tag);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join(&name), &data).unwrap();
    let sha: String = Sha256::digest(&data).iter().map(|b| format!("{b:02x}")).collect();
    fs::write(d.join(format!("{name}.sha256")), format!("{sha} *{name}\n")).unwrap();
}

fn list(e: &Env, tags: &[&str]) {
    let rows: Vec<String> = tags.iter().map(|t| format!(r#"{{"tag_name":"{t}","prerelease":false,"draft":false}}"#)).collect();
    fs::write(e.rel.join("releases.json"), format!("[{}]", rows.join(","))).unwrap();
}

/// 프로젝트(이 CLI로 init)의 고정값을 바꿈.
fn project(tmp: &Path, pin: &str) -> PathBuf {
    let root = run_init(
        &tmp.join("p"),
        &InitOptions { name: "P".into(), preset: "minimal".into(), modules: None, categories: Vec::new(), lang: "ko-KR".into(), date: "2026-11-10".into() },
    )
    .unwrap();
    let mp = root.join(".elf/manifest.json");
    let cli = elf_cli::embed::version();
    let m = fs::read_to_string(&mp).unwrap();
    fs::write(&mp, m.replace(&format!("\"elf_version\": \"{cli}\""), &format!("\"elf_version\": \"{pin}\""))).unwrap();
    root
}

#[test]
fn dispatches_to_the_stored_line_and_forwards_output_and_exit_code() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let root = project(tmp.path(), "v9.9.0");
    store_put(&e, "v9.9.1", &this_binary());
    list(&e, &["v9.9.1"]);

    // 넘김: 결정 한 줄(재귀 없음 — 넘겨받은 쪽은 다시 넘기지 않음), 출력과 종료 코드 = 직접 실행과 같음
    for args in [&["log"][..], &["deprecate", "show", "NOPE"], &["check", "logs"]] {
        let (code, out, err) = run(&root, &e, &[TRACE], args);
        assert_eq!(err.matches("dispatch: project line 9.9 → v9.9.1").count(), 1, "{args:?}: {err}");
        let (code_off, out_off, _) = run(&root, &e, &[OFF], args);
        assert_eq!((code, &out), (code_off, &out_off), "{args:?}");
    }
    assert_ne!(run(&root, &e, &[OFF], &["deprecate", "show", "NOPE"]).0, 0, "a failing exit code is forwarded too");
    // 런처 전용 명령, 재귀 방지, 끔
    for args in [&["check", "version"][..], &["--version"], &["self", "list"], &["update", "--dry-run"]] {
        let (_, _, err) = run(&root, &e, &[TRACE], args);
        assert!(!err.contains("dispatch:"), "{args:?}: {err}");
    }
    let (_, _, err) = run(&root, &e, &[TRACE, ("ELF_DISPATCHED", "v0.0.1")], &["log"]);
    assert!(!err.contains("dispatch:"), "{err}");
    // 이 CLI의 줄(고정값 = 이 버전) = 넘기지 않음
    let same = project(&tmp.path().join("same"), elf_cli::embed::version());
    let (_, _, err) = run(&same, &e, &[TRACE], &["log"]);
    assert!(!err.contains("dispatch:"), "{err}");
}

#[test]
fn first_use_installs_the_newest_patch_and_picks_up_a_newer_one_daily() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let root = project(tmp.path(), "v9.8.0");
    publish(&e, "v9.8.3", &this_binary());
    list(&e, &["v9.8.3", "v9.9.1"]);

    let (code, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(err.contains("line 9.8: installing v9.8.3 into the version store") && err.contains("dispatch: project line 9.8 → v9.8.3"), "{err}");
    assert_eq!(code, run(&root, &e, &[OFF], &["log"]).0);
    assert!(e.home.join("versions/v9.8.3").join(exe()).is_file());
    let (_, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(!err.contains("installing") && err.contains("→ v9.8.3"), "stored now: {err}");

    // 새 patch: 목록은 하루 한 번 — 캐시 날짜를 옛날로 돌려 다음 실행이 다시 묻게 함
    publish(&e, "v9.8.4", b"not an executable");
    list(&e, &["v9.8.4", "v9.8.3"]);
    let cache = e.home.join("versions/releases.json");
    let c = fs::read_to_string(&cache).unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    fs::write(&cache, c.replace(&today, "2000-01-01")).unwrap();
    let (code, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(err.contains("installing v9.8.4") && err.contains("→ v9.8.4"), "{err}");
    assert!(code != 0 && err.contains("cannot run"), "the fake 9.8.4 cannot start: {err}");
}

#[test]
fn unavailable_line_refuses_writes_and_the_2_27_line_falls_back_to_the_compat_mode() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    list(&e, &["v9.9.1"]);
    // 줄을 구할 수 없음: 읽기 = 경고 후 이 CLI, 쓰기 = 거부(exit 3)
    let root = project(tmp.path(), "v9.7.0");
    let (_, _, err) = run(&root, &e, &[], &["log"]);
    assert!(err.contains("project line 9.7 needs elf 9.7.x, which is not available"), "{err}");
    let (code, out, err) = run(&root, &e, &[], &["session", "new", "다음", "--goal", "g"]);
    assert_eq!(code, 3, "{out}{err}");
    assert!(err.contains("`elf self install 9.7`") && !root.join("2_Log/S/S001_log.md").exists(), "{err}");

    // 2.27 형식 로그만 있는 폴더(1세대), 2.27을 구할 수 없음 = 내장 이전 형식 모드로 실행
    let old = tmp.path().join("old");
    fs::create_dir_all(old.join("2_Log")).unwrap();
    fs::write(old.join("2_Log/S001_log.md"), "# S001: a\n\n> **Created**: 2026-06-01\\\n> **Status**: ★ 활성\\\n> **Handoff**: -\n").unwrap();
    let (code, out, err) = run(&old, &e, &[TRACE], &["check", "logs"]);
    assert!(err.contains("built-in 2.27 compatibility mode") && !err.contains("dispatch:"), "{err}");
    assert_eq!(code, run(&old, &e, &[OFF], &["check", "logs"]).0, "{out}");
}

// ── 오래된 목록, 고정 판 직접 받기, 같은 세대 지금 CLI로 실행(S053) ─────────────

/// 보관소 목록 캐시를 직접 씀(같은 날 받은 것으로 표시).
fn cache(e: &Env, tags: &[&str], rechecked: Option<&str>) {
    let d = e.home.join("versions");
    fs::create_dir_all(&d).unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let body = serde_json::json!({ "fetched": today, "attempted": null, "rechecked": rechecked, "versions": tags });
    fs::write(d.join("releases.json"), body.to_string()).unwrap();
}

#[test]
fn stale_same_day_list_is_rechecked_when_the_pinned_version_is_missing() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let root = project(tmp.path(), "v9.6.0");
    // 같은 날 받은 캐시에는 9.6이 없음(그 뒤에 발행됨) — 발행처에는 있음
    cache(&e, &["v9.5.0"], None);
    publish(&e, "v9.6.0", &this_binary());
    list(&e, &["v9.6.0", "v9.5.0"]);
    let (code, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(err.contains("installing v9.6.0") && err.contains("dispatch: project line 9.6 → v9.6.0"), "{err}");
    assert_eq!(code, run(&root, &e, &[OFF], &["log"]).0);
}

#[test]
fn the_pinned_version_is_installed_directly_when_the_list_cannot_be_read() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let root = project(tmp.path(), "v9.5.0");
    publish(&e, "v9.5.0", &this_binary()); // 목록(`releases.json`)은 없음
    let (_, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(err.contains("installing v9.5.0") && err.contains("→ v9.5.0"), "{err}");
}

#[test]
fn recheck_waits_an_hour_and_messages_name_the_real_cause() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    // 개발 판 고정: 직접 받을 수 없음 — 원인 = 발행되지 않은 판
    let root = project(tmp.path(), "v9.4.0-dev");
    cache(&e, &["v9.3.0"], None);
    list(&e, &["v9.3.0"]);
    let (_, _, err) = run(&root, &e, &[], &["log"]);
    assert!(err.contains("written by a pre-release (v9.4.0-dev), which is not published"), "{err}");
    // 9.4.0이 발행돼도 한 시간 안에는 다시 받지 않음(오프라인에서 명령마다 기다리지 않게)
    publish(&e, "v9.4.0", &this_binary());
    list(&e, &["v9.4.0", "v9.3.0"]);
    let (_, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(!err.contains("installing") && err.contains("not available"), "{err}");
    // 간격이 지나면 다시 받아 넘김
    let c = e.home.join("versions/releases.json");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&c).unwrap()).unwrap();
    let old = (chrono::Local::now() - chrono::Duration::hours(2)).fixed_offset().to_rfc3339();
    let mut v = v;
    v["rechecked"] = serde_json::json!(old);
    fs::write(&c, v.to_string()).unwrap();
    let (_, _, err) = run(&root, &e, &[TRACE], &["log"]);
    assert!(err.contains("installing v9.4.0") && err.contains("→ v9.4.0"), "{err}");
    // 받기 실패 = 받지 못한 판과 이유
    let root2 = project(&tmp.path().join("b"), "v9.2.0");
    let (_, _, err) = run(&root2, &e, &[], &["log"]);
    assert!(err.contains("v9.2.0 could not be downloaded"), "{err}");
}

// ── 판별 폴더와 실행기(v3.9 S059): 이전 줄도 자기 판, 실행기의 첫 실행 이관, self update, 정리 ──────────

/// 이 CLI보다 한 줄 낮은 같은 세대 줄(없으면 None — x.0 판).
fn older_line() -> Option<String> {
    let me = elf_cli::pin::parse(elf_cli::embed::version()).unwrap();
    if me.minor == 0 {
        return None;
    }
    let older = format!("v{}.{}.0", me.major, me.minor - 1);
    (elf_cli::pin::parse(&older).unwrap().line().generation() == me.line().generation()).then_some(older)
}

#[test]
fn an_older_line_runs_its_own_stored_version_with_a_daily_note() {
    let Some(older) = older_line() else { return };
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    store_put(&e, &older, &this_binary());
    let root = project(tmp.path(), &older);
    let (code, out, err) = run(&root, &e, &[TRACE], &["session", "new", "다음", "--goal", "g"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains(&format!("→ {older}")), "handed to its own version: {err}");
    assert!(err.contains("(its own version)") && err.contains("after the user approves"), "{err}");
    assert!(root.join("2_Log/S/S001_log.md").is_file());
    let m = fs::read_to_string(root.join(".elf/manifest.json")).unwrap();
    assert!(m.contains(&format!("\"elf_version\": \"{older}\"")), "the project line is not moved: {m}");
    assert!(e.home.join("versions").join(&older).join("last_used").is_file(), "last use is recorded");
    // 안내는 프로젝트마다 하루 한 번
    let (code, _, err) = run(&root, &e, &[], &["log"]);
    assert!(code == 0 && !err.contains("after the user approves"), "{err}");
}

#[test]
fn an_older_line_that_cannot_be_obtained_runs_with_the_default_and_a_note() {
    let Some(older) = older_line() else { return };
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    // 보관소에 없고 발행 목록도 읽을 수 없음 = 기본 판(이 CLI)으로 실행, 막지 않음
    let root = project(tmp.path(), &older);
    let (code, out, err) = run(&root, &e, &[TRACE], &["session", "new", "다음", "--goal", "g"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(err.contains("is not available") && err.contains("runs it as is"), "{err}");
    assert!(root.join("2_Log/S/S001_log.md").is_file());
}

/// 실행기 자리(`<ELF_HOME>/bin/`)에 이 시험 빌드의 사본을 둠.
fn put_launcher(e: &Env) -> PathBuf {
    let bin = e.home.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let l = bin.join(exe());
    fs::write(&l, this_binary()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&l, fs::Permissions::from_mode(0o755)).unwrap();
    }
    l
}

fn run_exe(exe: &Path, dir: &Path, env: &Env, extra: &[(&str, &str)], args: &[&str]) -> (i32, String, String) {
    let mut c = std::process::Command::new(exe);
    c.args(args).current_dir(dir).env("ELF_HOME", &env.home).env("ELF_RELEASES", &env.rel).env_remove("ELF_DISPATCHED").env_remove("ELF_DISPATCH");
    for (k, v) in extra {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

#[test]
fn the_launcher_moves_itself_into_the_store_and_hands_every_command_to_a_version() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let launcher = put_launcher(&e);
    let me = elf_cli::embed::version();
    // 첫 실행: 자기 사본을 보관소에 넣고 current를 씀, 그 뒤 프로젝트 밖 명령도 보관소 판이 실행
    let (code, out, err) = run_exe(&launcher, tmp.path(), &e, &[TRACE], &["--version"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(e.home.join("versions").join(me).join(exe()).is_file(), "copied into the store");
    assert_eq!(fs::read_to_string(e.home.join("current")).unwrap().trim(), me);
    assert!(err.contains(&format!("→ {me}")), "handed to the stored copy: {err}");
    // 같은 줄 프로젝트도 보관소 판으로
    let root = project(tmp.path(), me);
    let (code, _, err) = run_exe(&launcher, &root, &e, &[TRACE], &["log"]);
    assert!(code == 0 && err.contains(&format!("→ {me}")), "{err}");
    assert!(e.home.join("versions").join(me).join("last_used").is_file());
}

#[test]
fn self_update_adds_a_version_switches_current_and_cleans_the_store() {
    let tmp = tempdir().unwrap();
    let e = env(tmp.path());
    let launcher = put_launcher(&e);
    let me = elf_cli::embed::version();
    run_exe(&launcher, tmp.path(), &e, &[], &["--version"]); // 첫 실행(이관)
    // 보관소: 같은 줄 옛 patch(98.1.0), 오래 안 쓴 줄(97.0.0), 최근 쓴 줄(96.0.0)
    for t in ["v98.1.0", "v98.1.2", "v97.0.0", "v96.0.0"] {
        store_put(&e, t, b"stored");
    }
    fs::write(e.home.join("versions/v97.0.0/last_used"), "2000-01-01\n").unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    fs::write(e.home.join("versions/v96.0.0/last_used"), format!("{today}\n")).unwrap();
    fs::write(e.home.join("versions/v98.1.2/last_used"), format!("{today}\n")).unwrap();
    publish(&e, "v99.0.0", &this_binary());
    list(&e, &["v99.0.0", "v98.1.2"]);
    let launcher_before = fs::read(&launcher).unwrap();

    let (code, out, err) = run_exe(&launcher, tmp.path(), &e, &[], &["self", "update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains(&format!("updated to v99.0.0 — was {me}")), "{out}");
    assert_eq!(fs::read_to_string(e.home.join("current")).unwrap().trim(), "v99.0.0");
    assert!(e.home.join("versions/v99.0.0").join(exe()).is_file());
    assert!(out.contains("removed v98.1.0 (a newer patch of the line is kept)") && out.contains("removed v97.0.0 (not used for more than 30 days)"), "{out}");
    for kept in ["v98.1.2", "v96.0.0", me] {
        assert!(e.home.join("versions").join(kept).is_dir(), "{kept} kept: {out}");
    }
    assert_eq!(fs::read(&launcher).unwrap(), launcher_before, "same launcher protocol = the launcher file is not touched");
    // 이제 프로젝트 밖 명령은 새 기본 판이 실행
    let (_, _, err) = run_exe(&launcher, tmp.path(), &e, &[TRACE], &["--version"]);
    assert!(err.contains("→ v99.0.0"), "{err}");
    // 다시 실행 = 이미 최신
    let (code, out, _) = run_exe(&launcher, tmp.path(), &e, &[], &["self", "update"]);
    assert!(code == 0 && out.contains("already up to date (v99.0.0)"), "{out}");
}
