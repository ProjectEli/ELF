//! 버전 보관소(v3.0, S044 t51 R7, t58) — `~/.elf/versions/<vX.Y.Z>/elf[.exe]`.
//!
//! GitHub Release의 이 플랫폼 자산(`elf-cli-<target>.zip` 또는 `.tar.xz`)과 `.sha256`을 받아 대조하고 실행 파일만
//! 풀어 둠. 위임 규약(보관소 경로, Release 주소, 자산 이름)은 고정(R7). 바꿀 수 있는 것은 시험과 특수 환경용
//! 환경 변수 두 개뿐: `ELF_HOME`(보관소 상위, 기본 `~/.elf`), `ELF_RELEASES`(Release 기준 주소 또는 로컬 폴더).
//! zip과 tar는 Release 자산 형식(zip 최상위 파일, tar 한 폴더)에 맞춘 작은 직접 구현(zip crate는 매일 새 버전 —
//! S044 t58), inflate = miniz_oxide, xz = lzma-rust2.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::pin::{self, Line, Ver};

pub const REPO: &str = "ProjectEli/ELF";
pub const ENV_HOME: &str = "ELF_HOME";
pub const ENV_RELEASES: &str = "ELF_RELEASES";
/// 받는 파일 상한(자산은 3 MB 안팎)
const MAX_ASSET: u64 = 64 * 1024 * 1024;
const INSTALL_INFO: &str = "install.json";
const LIST_CACHE: &str = "releases.json";
const LATEST_CACHE: &str = "latest.json";

// ── 위치 ──────────────────────────────────────────────────────────

/// 보관소 상위(`ELF_HOME`, 없으면 `~/.elf` — 설치기가 쓰는 `~/.elf/bin`과 같은 곳).
pub fn home() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os(ENV_HOME).filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(h));
    }
    let user = if cfg!(windows) { std::env::var_os("USERPROFILE") } else { std::env::var_os("HOME") };
    user.filter(|v| !v.is_empty()).map(|h| PathBuf::from(h).join(".elf"))
}

pub fn versions_dir() -> Option<PathBuf> {
    home().map(|h| h.join("versions"))
}

/// 이 빌드의 Release 대상(target triple, 압축 형식). Release가 없는 플랫폼 = None.
pub fn target() -> Option<(&'static str, &'static str)> {
    if cfg!(all(windows, target_arch = "x86_64")) {
        Some(("x86_64-pc-windows-msvc", "zip"))
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(("x86_64-unknown-linux-musl", "tar.xz"))
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some(("aarch64-unknown-linux-musl", "tar.xz"))
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some(("x86_64-apple-darwin", "tar.xz"))
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some(("aarch64-apple-darwin", "tar.xz"))
    } else {
        None
    }
}

pub fn exe_name() -> &'static str {
    if cfg!(windows) { "elf.exe" } else { "elf" }
}

/// 이 플랫폼의 자산 이름(`elf-cli-<target>.<형식>`).
pub fn asset_name() -> Option<String> {
    target().map(|(t, e)| format!("elf-cli-{t}.{e}"))
}

/// 보관된 버전의 실행 파일 경로(있든 없든).
pub fn exe_path(v: &Ver) -> Option<PathBuf> {
    versions_dir().map(|d| d.join(v.to_string()).join(exe_name()))
}

// ── 목록 ──────────────────────────────────────────────────────────

/// 보관된 버전(새 순).
pub fn installed() -> Vec<(Ver, PathBuf)> {
    let Some(dir) = versions_dir() else { return Vec::new() };
    let mut v: Vec<(Ver, PathBuf)> = fs::read_dir(&dir)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let ver = pin::parse(name.strip_prefix('v')?)?;
                    let exe = e.path().join(exe_name());
                    exe.is_file().then_some((ver, exe))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v
}

/// 줄을 실행할 수 있는 버전인가 — 정식이고 실제 MAJOR.MINOR가 그 줄(2.28.x는 3.0 줄로 해석되지만 3.0 줄을
/// 실행하지 않음 — 2.28 실행 파일이 필요 없게, S044 t52).
fn runs_line(v: &Ver, line: Line) -> bool {
    v.pre.is_none() && (v.major, v.minor) == (line.major(), line.minor())
}

/// 줄의 가장 새 보관 버전(정식만).
pub fn newest_installed(line: Line) -> Option<(Ver, PathBuf)> {
    installed().into_iter().find(|(v, _)| runs_line(v, line))
}

/// 줄의 가장 새 정식 발행.
pub fn newest_release(line: Line, list: &[Ver]) -> Option<Ver> {
    list.iter().filter(|v| runs_line(v, line)).max().cloned()
}

fn releases_base() -> String {
    std::env::var(ENV_RELEASES)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("https://github.com/{REPO}/releases/download"))
}

fn is_remote(s: &str) -> bool {
    s.starts_with("https://") || s.starts_with("http://")
}

/// 자산 받기의 시도마다 상한(초)과 시도 수 상한(S059 t03).
const ATTEMPT_SECS: u64 = 120;
const MAX_ATTEMPTS: u32 = 20;

/// 자산과 `.sha256` 받기 — 원격은 이어받기(`fetch_resumable`), 로컬 경로는 그대로 읽음.
fn fetch(url: &str) -> Result<Vec<u8>, String> {
    if !is_remote(url) {
        return fs::read(url).map_err(|e| format!("{url}: {e}"));
    }
    fetch_resumable(url, ATTEMPT_SECS, MAX_ATTEMPTS)
}

/// 이어받기(S059 t03): GitHub 자산 CDN 속도가 시간대에 따라 20 KB/s까지 떨어져 전체 상한 하나로는 느리지만 진행 중인
/// 받기가 끊김. 시도마다 `secs` 상한, 끊기면 받은 바이트는 두고 `Range: bytes=<받은 수>-`로 이어받음. 206 = 이어 붙임,
/// 200 = 처음부터 새로(Range 무시), 416이고 받은 수 = 전체 = 끝. 한 시도에서 1바이트도 못 받은 일이 연속 2번이면 멈춤으로
/// 보고 실패, 4xx는 바로 실패. 전체 길이(`Content-Length`나 `Content-Range`의 전체)와 같아야 끝(모르면 EOF로).
pub(crate) fn fetch_resumable(url: &str, secs: u64, max_attempts: u32) -> Result<Vec<u8>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(secs)))
        .http_status_as_error(false)
        .build()
        .into();
    let mut data: Vec<u8> = Vec::new();
    let mut total: Option<usize> = None;
    let mut idle = 0u32;
    let mut last = String::from("no attempt");
    for attempt in 1..=max_attempts {
        let have = data.len();
        if attempt > 1 {
            let of = total.map(|t| format!(" of {t}")).unwrap_or_default();
            eprintln!("[elf] download interrupted at {have}{of} bytes ({last}) — resuming, attempt {attempt}");
        }
        let mut req = agent
            .get(url)
            .header("User-Agent", &format!("elf-cli/{}", crate::embed::version()))
            .header("Accept", "application/vnd.github+json, application/octet-stream");
        if have > 0 {
            req = req.header("Range", &format!("bytes={have}-"));
        }
        let (got, eof) = match req.call() {
            Err(e) => {
                last = e.to_string();
                (0, false)
            }
            Ok(mut resp) => {
                let status = resp.status().as_u16();
                let header = |k: &str| resp.headers().get(k).and_then(|v| v.to_str().ok()).map(str::to_string);
                match status {
                    200 => {
                        data.clear();
                        total = header("content-length").and_then(|v| v.trim().parse().ok());
                        read_into(&mut resp, &mut data, &mut last)
                    }
                    206 => {
                        // `bytes a-b/T` — 이어 붙일 위치가 다르면 처음부터
                        let cr = header("content-range").unwrap_or_default();
                        let start = cr.trim_start_matches("bytes ").split('-').next().and_then(|a| a.trim().parse::<usize>().ok());
                        if let Some(t) = cr.rsplit('/').next().and_then(|t| t.trim().parse().ok()) {
                            total = Some(t);
                        }
                        if start != Some(have) {
                            data.clear();
                            total = None;
                            last = format!("unexpected Content-Range `{cr}`");
                            (0, false)
                        } else {
                            read_into(&mut resp, &mut data, &mut last)
                        }
                    }
                    416 if have > 0 && total == Some(have) => return Ok(data),
                    s if (400..500).contains(&s) => return Err(format!("{url}: HTTP {s}")),
                    s => {
                        last = format!("HTTP {s}");
                        (0, false)
                    }
                }
            }
        };
        match total {
            Some(t) if data.len() == t => return Ok(data),
            Some(t) if data.len() > t => return Err(format!("{url}: received {} bytes, more than {t}", data.len())),
            None if eof => return Ok(data),
            _ => {}
        }
        if data.len() as u64 > MAX_ASSET {
            return Err(format!("{url}: larger than {MAX_ASSET} bytes"));
        }
        idle = if got == 0 { idle + 1 } else { 0 };
        if idle >= 2 {
            return Err(format!("{url}: no data in two attempts ({last}) — stopped at {} bytes", data.len()));
        }
    }
    Err(format!("{url}: incomplete after {max_attempts} attempts ({last}) — {} bytes received", data.len()))
}

/// 응답 본문을 조각으로 읽어 `data`에 붙임 → (받은 바이트, 끝까지 읽었는가). 끊기면 그때까지 받은 것은 남김.
fn read_into(resp: &mut ureq::http::Response<ureq::Body>, data: &mut Vec<u8>, last: &mut String) -> (usize, bool) {
    let mut r = resp.body_mut().with_config().limit(MAX_ASSET).reader();
    let mut buf = vec![0u8; 64 * 1024];
    let mut got = 0usize;
    loop {
        match r.read(&mut buf) {
            Ok(0) => return (got, true),
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                got += n;
            }
            Err(e) => {
                *last = e.to_string();
                return (got, false);
            }
        }
    }
}

fn fetch_with(url: &str, secs: u64) -> Result<Vec<u8>, String> {
    if !is_remote(url) {
        return fs::read(url).map_err(|e| format!("{url}: {e}"));
    }
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(secs))).build().into();
    let mut resp = agent
        .get(url)
        .header("User-Agent", &format!("elf-cli/{}", crate::embed::version()))
        .header("Accept", "application/vnd.github+json, application/octet-stream")
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    if resp.status() != 200 {
        return Err(format!("{url}: HTTP {}", resp.status()));
    }
    resp.body_mut().with_config().limit(MAX_ASSET).read_to_vec().map_err(|e| format!("{url}: {e}"))
}

/// GitHub API 형식 목록(`[{tag_name, prerelease, draft}]`) → 정식 버전.
fn parse_release_list(text: &str) -> Result<Vec<Ver>, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("release list: {e}"))?;
    let arr = v.as_array().ok_or("release list: not an array")?;
    Ok(arr
        .iter()
        .filter(|r| !r["prerelease"].as_bool().unwrap_or(false) && !r["draft"].as_bool().unwrap_or(false))
        .filter_map(|r| pin::parse(r["tag_name"].as_str()?))
        .filter(|v| v.pre.is_none())
        .collect())
}

/// 발행 목록 받기(GitHub API 페이지, 또는 `ELF_RELEASES` 로컬 폴더의 `releases.json`).
fn fetch_list(secs: u64) -> Result<Vec<Ver>, String> {
    let base = releases_base();
    if !is_remote(&base) {
        return fetch(&format!("{base}/{LIST_CACHE}")).and_then(|b| parse_release_list(&String::from_utf8_lossy(&b)));
    }
    let mut all = Vec::new();
    for page in 1..=10 {
        let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=100&page={page}");
        let text = String::from_utf8(fetch_with(&url, secs)?).map_err(|e| e.to_string())?;
        let n = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|v| v.as_array().map(Vec::len)).unwrap_or(0);
        all.extend(parse_release_list(&text)?);
        if n < 100 {
            break;
        }
    }
    Ok(all)
}

/// 목록 캐시(`versions/releases.json`): 받은 날, 마지막으로 실패한 날, 마지막 다시 받기 시각(오래된 목록 판정 때,
/// RFC 3339), 목록.
#[derive(Default)]
struct ListCache {
    fetched: Option<String>,
    attempted: Option<String>,
    rechecked: Option<String>,
    list: Vec<Ver>,
}

fn read_cache() -> ListCache {
    let text = versions_dir().and_then(|d| fs::read_to_string(d.join(LIST_CACHE)).ok()).unwrap_or_default();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return ListCache::default() };
    ListCache {
        fetched: v["fetched"].as_str().map(str::to_string),
        attempted: v["attempted"].as_str().map(str::to_string),
        rechecked: v["rechecked"].as_str().map(str::to_string),
        list: v["versions"].as_array().map(|a| a.iter().filter_map(|x| pin::parse(x.as_str()?)).collect()).unwrap_or_default(),
    }
}

fn write_cache(c: &ListCache) {
    let Some(d) = versions_dir() else { return };
    let _ = fs::create_dir_all(&d);
    let body = serde_json::json!({ "fetched": c.fetched, "attempted": c.attempted, "rechecked": c.rechecked, "versions": c.list.iter().map(Ver::to_string).collect::<Vec<_>>() });
    let _ = fs::write(d.join(LIST_CACHE), serde_json::to_string_pretty(&body).unwrap_or_default());
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 정식 발행 목록(pre-release, draft 제외). 같은 날 받은 캐시가 있으면 그것을 쓰고(`refresh`면 다시 받음), 받지
/// 못하면 날짜와 관계없이 캐시. 반환 = (목록, 캐시를 쓴 경우 그 날짜).
pub fn releases(refresh: bool) -> Result<(Vec<Ver>, Option<String>), String> {
    let today = today();
    let cached = read_cache();
    if !refresh && cached.fetched.as_deref() == Some(today.as_str()) {
        return Ok((cached.list, None));
    }
    match fetch_list(30) {
        Ok(list) => {
            write_cache(&ListCache { fetched: Some(today), list: list.clone(), ..Default::default() });
            Ok((list, None))
        }
        Err(e) => match cached.fetched {
            Some(d) => Ok((cached.list, Some(d))),
            None => Err(format!("cannot read the release list ({e})")),
        },
    }
}

/// 실행 위임용 목록: 하루 한 번만 짧게(4초) 묻고, 실패한 날은 다시 묻지 않음(오프라인에서 명령마다 기다리지 않게).
/// 받은 적이 없으면 None.
pub fn releases_daily() -> Option<Vec<Ver>> {
    let today = today();
    let cached = read_cache();
    if cached.fetched.as_deref() == Some(today.as_str()) || cached.attempted.as_deref() == Some(today.as_str()) {
        return cached.fetched.is_some().then_some(cached.list);
    }
    match fetch_list(4) {
        Ok(list) => {
            write_cache(&ListCache { fetched: Some(today), rechecked: cached.rechecked, list: list.clone(), ..Default::default() });
            Some(list)
        }
        Err(_) => {
            let had = cached.fetched.is_some();
            let list = cached.list.clone();
            write_cache(&ListCache { attempted: Some(today), ..cached });
            had.then_some(list)
        }
    }
}

/// 다시 받기 간격(초) — 오래된 목록으로 판정해 같은 날 다시 받는 것은 이 간격에 한 번(받지 못하는 환경에서 명령마다
/// 기다리지 않게).
pub const RECHECK_SECS: i64 = 3600;

/// 목록이 오래됐을 때(프로젝트가 고정한 판이나 필요한 줄이 목록에 없음) 같은 날이라도 다시 받음(S053). 마지막 다시
/// 받기가 `RECHECK_SECS` 안이면 받지 않음. 반환 = 새로 받은 목록(받지 않았거나 실패하면 None).
pub fn releases_recheck() -> Option<Vec<Ver>> {
    let cached = read_cache();
    let now = chrono::Local::now().fixed_offset();
    let recent = cached
        .rechecked
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .is_some_and(|t| (now - t).num_seconds() < RECHECK_SECS);
    if recent {
        return None;
    }
    match fetch_list(4) {
        Ok(list) => {
            write_cache(&ListCache { fetched: Some(today()), rechecked: Some(now.to_rfc3339()), list: list.clone(), ..Default::default() });
            Some(list)
        }
        Err(_) => {
            write_cache(&ListCache { rechecked: Some(now.to_rfc3339()), ..cached });
            None
        }
    }
}

// ── 설치 ──────────────────────────────────────────────────────────

fn sha256_hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// `.sha256` 파일(`<hex> *<이름>`)의 첫 64자리 16진수.
fn expected_sha(text: &str) -> Option<String> {
    let h = text.split_whitespace().next()?.to_ascii_lowercase();
    (h.len() == 64 && h.bytes().all(|c| c.is_ascii_hexdigit())).then_some(h)
}

fn le16(b: &[u8], at: usize) -> Option<usize> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]) as usize)
}

fn le32(b: &[u8], at: usize) -> Option<usize> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize)
}

fn base_name(p: &str) -> &str {
    p.rsplit(['/', '\\']).next().unwrap_or(p)
}

/// zip에서 경로 마지막 부분이 `name`인 항목 — 중앙 디렉터리 기준, stored(0)와 deflate(8)만, zip64 아님.
pub(crate) fn unzip_one(data: &[u8], name: &str) -> Result<Vec<u8>, String> {
    let bad = || "zip: damaged archive".to_string();
    let from = data.len().saturating_sub(65_557);
    let eocd = (from..data.len().saturating_sub(21)).rev().find(|&i| data[i..i + 4] == [0x50, 0x4b, 0x05, 0x06]).ok_or_else(bad)?;
    let count = le16(data, eocd + 10).ok_or_else(bad)?;
    let mut p = le32(data, eocd + 16).ok_or_else(bad)?;
    for _ in 0..count {
        if data.get(p..p + 4) != Some(&[0x50, 0x4b, 0x01, 0x02][..]) {
            return Err(bad());
        }
        let method = le16(data, p + 10).ok_or_else(bad)?;
        let csize = le32(data, p + 20).ok_or_else(bad)?;
        let usize_ = le32(data, p + 24).ok_or_else(bad)?;
        let (nl, el, cl) = (le16(data, p + 28).ok_or_else(bad)?, le16(data, p + 30).ok_or_else(bad)?, le16(data, p + 32).ok_or_else(bad)?);
        let local = le32(data, p + 42).ok_or_else(bad)?;
        let entry = String::from_utf8_lossy(data.get(p + 46..p + 46 + nl).ok_or_else(bad)?).into_owned();
        p += 46 + nl + el + cl;
        if base_name(&entry) != name || entry.ends_with('/') {
            continue;
        }
        if csize == 0xFFFF_FFFF || usize_ == 0xFFFF_FFFF {
            return Err("zip: zip64 entries are not supported".into());
        }
        if data.get(local..local + 4) != Some(&[0x50, 0x4b, 0x03, 0x04][..]) {
            return Err(bad());
        }
        let start = local + 30 + le16(data, local + 26).ok_or_else(bad)? + le16(data, local + 28).ok_or_else(bad)?;
        let raw = data.get(start..start + csize).ok_or_else(bad)?;
        let out = match method {
            0 => raw.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, usize_.max(1)).map_err(|e| format!("zip: inflate failed ({e:?})"))?,
            m => return Err(format!("zip: compression method {m} is not supported")),
        };
        if out.len() != usize_ {
            return Err(bad());
        }
        return Ok(out);
    }
    Err(format!("zip: no {name} in the archive"))
}

fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

fn octal(b: &[u8]) -> Option<usize> {
    let s = cstr(b);
    let s = s.trim();
    if s.is_empty() { Some(0) } else { usize::from_str_radix(s, 8).ok() }
}

/// tar에서 경로 마지막 부분이 `name`인 일반 파일 — ustar, GNU 긴 이름(`L`), pax `path`.
pub(crate) fn untar_one(data: &[u8], name: &str) -> Result<Vec<u8>, String> {
    let bad = || "tar: damaged archive".to_string();
    let mut pos = 0;
    let mut long: Option<String> = None;
    while pos + 512 <= data.len() {
        let h = &data[pos..pos + 512];
        if h.iter().all(|&c| c == 0) {
            break;
        }
        let size = octal(&h[124..136]).ok_or_else(bad)?;
        let body = data.get(pos + 512..pos + 512 + size).ok_or_else(bad)?;
        let mut path = cstr(&h[0..100]);
        if &h[257..262] == b"ustar" {
            let prefix = cstr(&h[345..500]);
            if !prefix.is_empty() {
                path = format!("{prefix}/{path}");
            }
        }
        match h[156] {
            b'L' => long = Some(cstr(body)),
            b'x' => {
                long = String::from_utf8_lossy(body).lines().find_map(|l| l.split_once(" path=").map(|(_, p)| p.to_string()));
            }
            b'0' | 0 => {
                let full = long.take().unwrap_or(path);
                if base_name(&full) == name {
                    return Ok(body.to_vec());
                }
            }
            _ => long = None,
        }
        pos += 512 + size.div_ceil(512) * 512;
    }
    Err(format!("tar: no {name} in the archive"))
}

fn unxz(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    lzma_rust2::XzReader::new(data, false).take(MAX_ASSET * 4).read_to_end(&mut out).map_err(|e| format!("xz: {e}"))?;
    Ok(out)
}

/// 자산(압축 파일) → 실행 파일 내용.
pub(crate) fn extract_exe(asset: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    if asset.ends_with(".zip") { unzip_one(data, exe_name()) } else { untar_one(&unxz(data)?, exe_name()) }
}

/// 설치 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    pub version: Ver,
    pub exe: PathBuf,
    /// 이번에 새로 받았는가(false = 이미 있었음)
    pub fresh: bool,
}

/// 버전 하나를 보관소에 설치(이미 있으면 그대로). 받기 → `.sha256` 대조 → 실행 파일만 풀기 → 원자적 이름 바꾸기.
pub fn install(v: &Ver) -> Result<Installed, String> {
    let dir = versions_dir().ok_or("no home folder for the version store (set ELF_HOME)")?;
    let dest = dir.join(v.to_string());
    let exe = dest.join(exe_name());
    if exe.is_file() {
        return Ok(Installed { version: v.clone(), exe, fresh: false });
    }
    let asset = asset_name().ok_or("this platform has no ELF release asset — build from source")?;
    let base = releases_base();
    let url = format!("{base}/{v}/{asset}");
    let data = fetch(&url)?;
    let sums = fetch(&format!("{url}.sha256"))?;
    let want = expected_sha(&String::from_utf8_lossy(&sums)).ok_or_else(|| format!("{url}.sha256: no sha256 value"))?;
    let got = sha256_hex(&data);
    if got != want {
        return Err(format!("{asset} for {v}: sha256 mismatch (expected {want}, got {got}) — nothing installed"));
    }
    let bin = extract_exe(&asset, &data)?;
    let tmp = dir.join(format!(".tmp-{}-{}", std::process::id(), v));
    let _ = fs::remove_dir_all(&tmp);
    let staged = tmp.join(v.to_string());
    fs::create_dir_all(&staged).map_err(|e| format!("{}: {e}", staged.display()))?;
    fs::write(staged.join(exe_name()), &bin).map_err(|e| format!("{}: {e}", staged.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(staged.join(exe_name()), fs::Permissions::from_mode(0o755));
    }
    let info = serde_json::json!({
        "version": v.to_string(),
        "target": target().map(|t| t.0).unwrap_or_default(),
        "asset": asset,
        "sha256": got,
        "source": url,
        "installed": chrono::Local::now().format("%Y-%m-%d").to_string(),
    });
    fs::write(staged.join(INSTALL_INFO), serde_json::to_string_pretty(&info).unwrap_or_default()).map_err(|e| e.to_string())?;
    let moved = fs::rename(&staged, &dest);
    let _ = fs::remove_dir_all(&tmp);
    match moved {
        Ok(()) => Ok(Installed { version: v.clone(), exe, fresh: true }),
        // 다른 실행이 먼저 설치함
        Err(_) if exe.is_file() => Ok(Installed { version: v.clone(), exe, fresh: false }),
        Err(e) => Err(format!("{}: {e}", dest.display())),
    }
}

/// 최신 정식 판(S060): 원격 = `releases/latest` 리다이렉트의 `Location`(발행 목록 2 MB 대신 수백 바이트, GitHub의
/// latest는 초안과 pre-release 제외), 로컬 폴더(`ELF_RELEASES`, 시험) = 그 폴더 `releases.json`의 최신 정식 판.
/// 찾지 못하면 Err — 옛 목록으로 대신 판단하지 않음.
pub fn latest_release() -> Result<Ver, String> {
    latest_with(30).or_else(|_| latest_with(30))
}

/// 최신 정식 판 한 번 묻기(`secs` 상한): 원격 = `releases/latest` 리다이렉트, 로컬 폴더 = `releases.json`의 최신.
fn latest_with(secs: u64) -> Result<Ver, String> {
    let base = releases_base();
    if !is_remote(&base) {
        let list = fetch(&format!("{base}/{LIST_CACHE}")).and_then(|b| parse_release_list(&String::from_utf8_lossy(&b)))?;
        return list.into_iter().filter(|v| v.pre.is_none()).max().ok_or_else(|| "the release list has no published release".into());
    }
    latest_from(&latest_url(&base), secs)
}

/// 실행 위임의 하루 한 번 patch 확인용 최신 정식 판(S061): 발행이 한 줄로 이어지므로(새 줄이 나온 뒤 이전 줄 patch를
/// 낸 적 없음) latest와 같은 줄의 최신 patch = latest, 이전 줄은 새 patch 없음. 2 MB 발행 목록 대신 수 KB 리다이렉트를
/// 4초 상한으로 하루 한 번, 결과는 `versions/latest.json`(확인한 날, 실패한 날, 판). 실패한 날은 다시 묻지 않고,
/// 받은 적이 있으면 마지막으로 안 판(날짜와 무관).
pub fn latest_daily() -> Option<Ver> {
    let dir = versions_dir()?;
    let path = dir.join(LATEST_CACHE);
    let today = today();
    let cached: serde_json::Value =
        fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| serde_json::json!({}));
    let known = cached["version"].as_str().and_then(pin::parse);
    if cached["checked"].as_str() == Some(today.as_str()) || cached["attempted"].as_str() == Some(today.as_str()) {
        return known;
    }
    let fresh = latest_with(4);
    let body = match &fresh {
        Ok(v) => serde_json::json!({ "checked": today, "version": v.to_string() }),
        Err(_) => serde_json::json!({ "checked": cached["checked"], "attempted": today, "version": cached["version"] }),
    };
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(&path, serde_json::to_string_pretty(&body).unwrap_or_default());
    fresh.ok().or(known)
}

/// `v`가 줄 `line`을 실행하는 정식 판인가(2.28.x는 3.0 줄로 해석되지만 3.0 줄을 실행하지 않음).
pub fn is_of_line(v: &Ver, line: Line) -> bool {
    runs_line(v, line)
}

/// Release 기준 주소(`…/releases/download`) → `…/releases/latest`.
fn latest_url(base: &str) -> String {
    let b = base.trim_end_matches('/');
    match b.strip_suffix("/download") {
        Some(root) => format!("{root}/latest"),
        None => format!("{b}/latest"),
    }
}

/// 리다이렉트를 따르지 않고 `url`을 물어 `Location`의 판을 읽음.
pub(crate) fn latest_from(url: &str, secs: u64) -> Result<Ver, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(secs)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into();
    let resp = agent
        .get(url)
        .header("User-Agent", &format!("elf-cli/{}", crate::embed::version()))
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let status = resp.status().as_u16();
    let loc = resp.headers().get("location").and_then(|v| v.to_str().ok()).map(str::to_string);
    match (status, loc) {
        (300..=399, Some(l)) => tag_from_location(&l).ok_or_else(|| format!("{url}: unexpected redirect to `{l}`")),
        (s, _) => Err(format!("{url}: HTTP {s} without a redirect to the latest release")),
    }
}

/// `…/releases/tag/vX.Y.Z` → 정식 판(pre-release, 다른 경로 = None).
pub fn tag_from_location(loc: &str) -> Option<Ver> {
    let path = loc.split(['?', '#']).next()?.trim_end_matches('/');
    let (head, tag) = path.rsplit_once('/')?;
    if !head.ends_with("/releases/tag") {
        return None;
    }
    pin::parse(tag).filter(|v| v.pre.is_none())
}

/// 줄의 최신 정식 patch를 설치(발행 목록 필요).
pub fn install_line(line: Line, refresh: bool) -> Result<Installed, String> {
    let (mut list, _) = releases(refresh)?;
    // 같은 날 받은 목록에 그 줄이 없으면 목록이 오래된 것일 수 있음 — 한 번 다시 받음(S053)
    if !refresh && newest_release(line, &list).is_none() {
        list = releases(true)?.0;
    }
    let v = newest_release(line, &list).ok_or_else(|| format!("no published release in line {line}"))?;
    install(&v)
}

/// 보관된 버전 하나 삭제.
pub fn uninstall(v: &Ver) -> Result<(), String> {
    let dest = versions_dir().ok_or("no home folder for the version store")?.join(v.to_string());
    if !dest.is_dir() {
        return Err(format!("{v} is not in the version store"));
    }
    fs::remove_dir_all(&dest).map_err(|e| format!("{}: {e} (is it running?)", dest.display()))
}

/// 줄마다 가장 새 버전만 남기고 지움(고정값은 줄의 최신 patch로 실행되므로 이전 patch는 쓰이지 않음). 반환 = 지운 버전.
pub fn prune() -> Result<Vec<Ver>, String> {
    let mut seen: Vec<Line> = Vec::new();
    let mut removed = Vec::new();
    for (v, _) in installed() {
        if seen.contains(&v.line()) {
            uninstall(&v)?;
            removed.push(v);
        } else {
            seen.push(v.line());
        }
    }
    Ok(removed)
}

/// `2.27` = 그 줄의 최신 정식 patch, `2.27.4`(또는 `v2.27.4`, `v3.0.0-rc`) = 그 버전.
pub fn install_spec(spec: &str) -> Result<Installed, String> {
    let s = spec.trim().trim_start_matches('v');
    let v = pin::parse(s).ok_or_else(|| format!("not a version or a line: {spec} (e.g. 2.27 or 2.27.4)"))?;
    let core = s.split('-').next().unwrap_or(s);
    if core.matches('.').count() == 1 && v.pre.is_none() { install_line(v.line(), false) } else { install(&v) }
}

/// `elf self list [--available]` 출력 줄.
pub fn list_lines(available: bool) -> Result<Vec<String>, String> {
    let cli = crate::embed::version();
    let mut out = vec![format!(
        "this CLI: {cli}{}",
        pin::parse(cli).map(|v| format!(" (line {}, structure generation {})", v.line(), v.line().generation())).unwrap_or_default()
    )];
    let dir = versions_dir().map(|d| shown(&d)).unwrap_or_else(|| "(no home folder — set ELF_HOME)".into());
    let inst = installed();
    out.push(format!("version store: {dir} — {} version(s)", inst.len()));
    for (v, _) in &inst {
        out.push(format!("  {v} (line {}, structure generation {})", v.line(), v.line().generation()));
    }
    if available {
        let (list, cached) = releases(true)?;
        if let Some(d) = cached {
            out.push(format!("release list: offline — using the list saved on {d}"));
        }
        let mut lines: Vec<Line> = list.iter().map(Ver::line).collect();
        lines.sort();
        lines.dedup();
        out.push("releases (newest patch of each line):".into());
        for l in lines.into_iter().rev() {
            if let Some(v) = newest_release(l, &list) {
                let mark = if inst.iter().any(|(x, _)| *x == v) { " — stored" } else { "" };
                out.push(format!("  {v} (line {l}, structure generation {}){mark}", l.generation()));
            }
        }
    }
    Ok(out)
}

/// 보관 기록(`install.json`)의 sha256(있으면).
pub fn recorded_sha(v: &Ver) -> Option<String> {
    let p = versions_dir()?.join(v.to_string()).join(INSTALL_INFO);
    let j: serde_json::Value = serde_json::from_str(&fs::read_to_string(p).ok()?).ok()?;
    j["sha256"].as_str().map(str::to_string)
}

/// 상대 경로 표시용(Windows `\\?\` 접두 제거).
pub fn shown(p: &Path) -> String {
    let s = p.display().to_string();
    s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// stored 또는 deflate 항목 zip(시험용 최소 작성기).
    fn zip(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, body, deflate) in entries {
            let data = if *deflate { miniz_oxide::deflate::compress_to_vec(body, 6) } else { body.to_vec() };
            let off = out.len() as u32;
            let method: u16 = if *deflate { 8 } else { 0 };
            let mut lh = vec![0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0];
            lh.extend(method.to_le_bytes());
            lh.extend([0u8; 8]); // 시각, crc(읽기에서 보지 않음)
            lh.extend((data.len() as u32).to_le_bytes());
            lh.extend((body.len() as u32).to_le_bytes());
            lh.extend((name.len() as u16).to_le_bytes());
            lh.extend(0u16.to_le_bytes());
            lh.extend(name.as_bytes());
            out.extend(&lh);
            out.extend(&data);
            let mut ch = vec![0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0];
            ch.extend(method.to_le_bytes());
            ch.extend([0u8; 8]);
            ch.extend((data.len() as u32).to_le_bytes());
            ch.extend((body.len() as u32).to_le_bytes());
            ch.extend((name.len() as u16).to_le_bytes());
            ch.extend([0u8; 12]); // extra, comment, disk, attrs
            ch.extend(off.to_le_bytes());
            ch.extend(name.as_bytes());
            central.extend(ch);
        }
        let cd_off = out.len() as u32;
        out.extend(&central);
        out.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
        out.extend((entries.len() as u16).to_le_bytes());
        out.extend((entries.len() as u16).to_le_bytes());
        out.extend((central.len() as u32).to_le_bytes());
        out.extend(cd_off.to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out
    }

    fn tar(entries: &[(&str, &[u8], u8)]) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, body, kind) in entries {
            let mut h = [0u8; 512];
            h[..name.len().min(100)].copy_from_slice(&name.as_bytes()[..name.len().min(100)]);
            let size = format!("{:011o}\0", body.len());
            h[124..136].copy_from_slice(size.as_bytes());
            h[156] = *kind;
            h[257..263].copy_from_slice(b"ustar\0");
            out.extend(h);
            out.extend(*body);
            out.extend(vec![0u8; body.len().div_ceil(512) * 512 - body.len()]);
        }
        out.extend([0u8; 1024]);
        out
    }

    #[test]
    fn unzip_finds_the_exe_stored_or_deflated() {
        let bin: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
        for deflate in [false, true] {
            let z = zip(&[("CHANGELOG.md", b"# log", deflate), ("elf.exe", &bin, deflate), ("dir/", b"", false)]);
            assert_eq!(unzip_one(&z, "elf.exe").unwrap(), bin, "deflate {deflate}");
            assert!(unzip_one(&z, "nope").unwrap_err().contains("no nope"));
        }
        assert!(unzip_one(b"not a zip at all, definitely not", "elf.exe").unwrap_err().contains("damaged"));
    }

    #[test]
    fn untar_reads_ustar_and_gnu_long_names() {
        let long = format!("{}/elf", "d".repeat(120));
        let t = tar(&[
            ("elf-cli-x/", b"", b'5'),
            ("elf-cli-x/README.md", b"readme", b'0'),
            ("././@LongLink", long.as_bytes(), b'L'),
            ("truncated", b"real-binary", b'0'),
        ]);
        assert_eq!(untar_one(&t, "elf").unwrap(), b"real-binary");
        let t = tar(&[("elf-cli-x/elf", b"bin", b'0')]);
        assert_eq!(untar_one(&t, "elf").unwrap(), b"bin");
        assert!(untar_one(&t, "elf.exe").is_err());
    }

    #[test]
    fn sha_values_and_release_lists() {
        assert_eq!(expected_sha(&format!("{} *elf-cli.zip\n", "A".repeat(64))), Some("a".repeat(64)));
        assert_eq!(expected_sha("abc *x"), None);
        let list = parse_release_list(
            r#"[{"tag_name":"v2.28.1","prerelease":true,"draft":false},{"tag_name":"v2.27.4","prerelease":false,"draft":false},
                {"tag_name":"v2.27.3","prerelease":false,"draft":false},{"tag_name":"v3.0.0","prerelease":false,"draft":true},
                {"tag_name":"v3.0.1","prerelease":false,"draft":false},{"tag_name":"v3.0.2-rc","prerelease":false,"draft":false}]"#,
        )
        .unwrap();
        assert_eq!(list.iter().map(Ver::to_string).collect::<Vec<_>>(), ["v2.27.4", "v2.27.3", "v3.0.1"]);
        assert_eq!(newest_release(pin::earlier_line(), &list).unwrap().to_string(), "v2.27.4");
        assert_eq!(newest_release(pin::line_of(2, 28), &list).unwrap().to_string(), "v3.0.1", "2.28 = 3.0 line");
        assert_eq!(newest_release(pin::line_of(2, 26), &list), None);
        // 2.28.x는 3.0 줄로 해석되지만 3.0 줄을 실행하는 버전으로 고르지 않음
        let only28 = vec![pin::parse("v2.28.5").unwrap()];
        assert_eq!(newest_release(pin::line_of(3, 0), &only28), None);
    }

    // ── 이어받기(S059 t03): 시험 안 로컬 HTTP 서버, 연결마다 정해 둔 응답 ───────────────

    type Step = Box<dyn FnOnce(&str, &mut std::net::TcpStream) + Send>;

    /// 연결을 차례로 받아 각 단계로 응답하는 서버 → 기준 주소. 요청 머리(헤더)를 단계에 넘김.
    fn serve(steps: Vec<Step>) -> String {
        use std::io::{BufRead, BufReader, Write};
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            for step in steps {
                let Ok((mut s, _)) = l.accept() else { return };
                let mut req = String::new();
                let mut rd = BufReader::new(s.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    if rd.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    req.push_str(&line);
                }
                step(&req, &mut s);
                let _ = s.flush();
            }
        });
        format!("http://{addr}/asset.zip")
    }

    fn body(n: usize) -> Vec<u8> {
        (0..n).map(|i| (i % 251) as u8).collect()
    }

    fn head(s: &mut std::net::TcpStream, status: &str, extra: &str) {
        use std::io::Write;
        let _ = write!(s, "HTTP/1.1 {status}\r\nConnection: close\r\n{extra}\r\n");
    }

    #[test]
    fn resumes_after_a_cut_with_a_range_request() {
        use std::io::Write;
        let full = body(100_000);
        let (a, b) = (full.clone(), full.clone());
        let url = serve(vec![
            Box::new(move |_, s| {
                head(s, "200 OK", "Content-Length: 100000\r\n");
                let _ = s.write_all(&a[..30_000]); // 끊김
            }),
            Box::new(move |req, s| {
                assert!(req.to_ascii_lowercase().contains("range: bytes=30000-"), "{req}");
                head(s, "206 Partial Content", "Content-Length: 70000\r\nContent-Range: bytes 30000-99999/100000\r\n");
                let _ = s.write_all(&b[30_000..]);
            }),
        ]);
        assert_eq!(fetch_resumable(&url, 10, 5).unwrap(), full);
    }

    #[test]
    fn a_server_ignoring_range_starts_over_without_duplicates() {
        use std::io::Write;
        let full = body(50_000);
        let (a, b) = (full.clone(), full.clone());
        let url = serve(vec![
            Box::new(move |_, s| {
                head(s, "200 OK", "Content-Length: 50000\r\n");
                let _ = s.write_all(&a[..10_000]);
            }),
            Box::new(move |_, s| {
                head(s, "200 OK", "Content-Length: 50000\r\n");
                let _ = s.write_all(&b);
            }),
        ]);
        assert_eq!(fetch_resumable(&url, 10, 5).unwrap(), full);
    }

    #[test]
    fn one_shot_and_client_errors() {
        use std::io::Write;
        let full = body(5_000);
        let a = full.clone();
        let url = serve(vec![Box::new(move |_, s| {
            head(s, "200 OK", &format!("Content-Length: {}\r\n", a.len()));
            let _ = s.write_all(&a);
        })]);
        assert_eq!(fetch_resumable(&url, 10, 5).unwrap(), full);
        let url = serve(vec![Box::new(|_, s| head(s, "404 Not Found", "Content-Length: 0\r\n"))]);
        let e = fetch_resumable(&url, 10, 5).unwrap_err();
        assert!(e.contains("HTTP 404"), "{e}");
    }

    #[test]
    fn two_attempts_without_data_stop() {
        // 머리만 보내고 본문 없이 붙잡음 → 시도마다 1초 상한으로 끊김, 연속 2번 0바이트 = 멈춤
        let hold = |_: &str, s: &mut std::net::TcpStream| {
            head(s, "200 OK", "Content-Length: 1000\r\n");
            std::thread::sleep(std::time::Duration::from_millis(2500));
        };
        let url = serve(vec![Box::new(hold), Box::new(hold), Box::new(hold)]);
        let e = fetch_resumable(&url, 1, 5).unwrap_err();
        assert!(e.contains("no data in two attempts"), "{e}");
    }


    // ── 최신 판(S060): releases/latest 리다이렉트 ─────────────────────────────

    #[test]
    fn latest_tag_comes_from_the_redirect_location() {
        let v = |s: &str| pin::parse(s).unwrap();
        assert_eq!(tag_from_location("https://github.com/ProjectEli/ELF/releases/tag/v3.9.1"), Some(v("v3.9.1")));
        assert_eq!(tag_from_location("https://github.com/o/r/releases/tag/v3.9.1/?x=1"), Some(v("v3.9.1")));
        assert_eq!(tag_from_location("https://github.com/o/r/releases/tag/v3.10.0-rc"), None, "pre-release");
        assert_eq!(tag_from_location("https://github.com/o/r/releases"), None);
        assert_eq!(latest_url("https://github.com/o/r/releases/download"), "https://github.com/o/r/releases/latest");
        assert_eq!(latest_url("http://127.0.0.1:9/x/"), "http://127.0.0.1:9/x/latest");
    }

    #[test]
    fn latest_from_reads_a_302_and_rejects_a_page_without_one() {
        let url = serve(vec![Box::new(|req, s| {
            assert!(req.starts_with("GET /asset.zip"), "{req}");
            head(s, "302 Found", "Location: https://github.com/o/r/releases/tag/v9.9.9\r\nContent-Length: 0\r\n");
        })]);
        assert_eq!(latest_from(&url, 10).unwrap(), pin::parse("v9.9.9").unwrap());
        let url = serve(vec![Box::new(|_, s| head(s, "200 OK", "Content-Length: 0\r\n"))]);
        assert!(latest_from(&url, 10).unwrap_err().contains("HTTP 200 without a redirect"));
        let url = serve(vec![Box::new(|_, s| head(s, "302 Found", "Location: https://github.com/o/r/releases\r\nContent-Length: 0\r\n"))]);
        assert!(latest_from(&url, 10).unwrap_err().contains("unexpected redirect"));
    }

}
