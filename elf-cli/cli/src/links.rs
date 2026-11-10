//! 링크와 경로 참조(v3.7, S056): 엄격한 링크 인식, 깨진 링크, 폴더 한 단계 차이 자동 수정, 프로젝트 밖 판정.
//!
//! 판정에는 프로젝트 루트와 파일 존재 여부만 쓴다. 폴더 구조(init 뒤 사용자가 바꿀 수 있음)와 git은 쓰지
//! 않는다. 고칠 위치는 `../`를 하나 더하거나 빼서 한쪽에서만 파일이 찾아질 때만 계산한다(문서가 하위 폴더로
//! 들어가거나 나온 경우). 같은 이름 파일 찾기는 하지 않는다(다른 파일을 가리킬 수 있음).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// `.elf/config.json`의 자동 수정 스위치 키.
pub const CONFIG_KEY: &str = "fix_links";

/// config `fix_links` — 부재, 파싱 실패 = true(기본 켬, autoread와 같은 극성). 끄기는 명시적 false만.
pub fn fix_enabled(root: &Path) -> bool {
    fs::read_to_string(root.join(".elf").join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get(CONFIG_KEY).and_then(|b| b.as_bool()))
        .unwrap_or(true)
}

/// 문서 안 링크 하나. `start..end` = 내용 전체에서 대상 경로(앵커, 제목, `<>` 제외)의 바이트 범위.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub raw: String,
}

/// 마크다운 인라인 링크 `](대상)` 중 **경로 모양**인 것(순수).
/// 대상은 한 덩어리이고 뒤에는 제목(`"…"`, `'…'`)만 올 수 있음. 마지막 부분이 문자로 시작하는 확장자를
/// 갖거나 `/`로 끝나야 경로로 봄 — 한국어 문장의 `[15](27)`, `](§17.9)`, `](설명 …)` 같은 오탐 제외.
/// 제외: URL과 scheme, 앵커만, 자리표시(`{ } < > *`), 코드 블록(``` ``` ```, `~~~`)과 인라인 코드 안.
pub fn links(content: &str) -> Vec<Link> {
    let mut out = Vec::new();
    let mut fence = false;
    let mut offset = 0usize;
    for (n, line) in content.split_inclusive('\n').enumerate() {
        let base = offset;
        offset += line.len();
        let l = line.trim_end_matches(['\n', '\r']);
        let t = l.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        let code = code_mask(l);
        let b = l.as_bytes();
        let mut i = 0;
        while i + 1 < b.len() {
            if b[i] == b']'
                && b[i + 1] == b'('
                && !code[i]
                && let Some((s, e, next)) = destination(l, i + 2)
            {
                let raw = &l[s..e];
                if is_path_like(raw) {
                    out.push(Link { line: n + 1, start: base + s, end: base + e, raw: raw.to_string() });
                }
                i = next;
                continue;
            }
            i += 1;
        }
    }
    out
}

/// 줄 안 인라인 코드(같은 길이의 backtick 묶음 사이)의 바이트 표시.
fn code_mask(l: &str) -> Vec<bool> {
    let b = l.as_bytes();
    let mut m = vec![false; b.len()];
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let mut k = i;
        while k < b.len() && b[k] == b'`' {
            k += 1;
        }
        let run = k - i;
        let mut j = k;
        let mut close = None;
        while j < b.len() {
            if b[j] == b'`' {
                let mut q = j;
                while q < b.len() && b[q] == b'`' {
                    q += 1;
                }
                if q - j == run {
                    close = Some(q);
                    break;
                }
                j = q;
            } else {
                j += 1;
            }
        }
        match close {
            Some(q) => {
                for x in m.iter_mut().take(q).skip(i) {
                    *x = true;
                }
                i = q;
            }
            None => i = k,
        }
    }
    m
}

/// `](` 다음 위치 `p`에서 대상 범위(앵커 제외)와 다음 탐색 위치. 링크 형식이 아니면 None.
fn destination(l: &str, p: usize) -> Option<(usize, usize, usize)> {
    let rest = &l[p..];
    if let Some(r) = rest.strip_prefix('<') {
        let gt = r.find('>')?;
        let after = &r[gt + 1..];
        let close = after.find(')')?;
        let title = after[..close].trim();
        if !(title.is_empty() || is_title(title)) {
            return None;
        }
        let s = p + 1;
        let e = s + gt;
        return Some((s, cut_anchor(l, s, e), s + gt + 1 + close + 1));
    }
    let close = rest.find(')')?;
    let inner = &rest[..close];
    let (dest, title) = match inner.find(char::is_whitespace) {
        Some(w) => (&inner[..w], inner[w..].trim()),
        None => (inner, ""),
    };
    if dest.is_empty() || !(title.is_empty() || is_title(title)) {
        return None;
    }
    Some((p, cut_anchor(l, p, p + dest.len()), p + close + 1))
}

fn is_title(t: &str) -> bool {
    t.len() >= 2 && ((t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')))
}

fn cut_anchor(l: &str, s: usize, e: usize) -> usize {
    match l[s..e].find('#') {
        Some(h) => s + h,
        None => e,
    }
}

/// URL scheme(`https:`, `mailto:`, `doi:` 등 두 글자 이상) — 드라이브 문자(`C:`)는 제외.
fn has_scheme(raw: &str) -> bool {
    match raw.find(':') {
        Some(c) if c >= 2 => {
            let s = &raw[..c];
            s.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic())
                && s.chars().all(|ch| ch.is_ascii_alphanumeric() || "+.-".contains(ch))
        }
        _ => false,
    }
}

fn is_path_like(raw: &str) -> bool {
    if raw.is_empty() || raw.contains("://") || has_scheme(raw) || raw.contains(['{', '}', '<', '>', '*']) {
        return false;
    }
    if raw.ends_with('/') {
        return true;
    }
    let last = raw.rsplit(['/', '\\']).next().unwrap_or(raw);
    match last.rfind('.') {
        Some(d) => {
            let ext = &last[d + 1..];
            !ext.is_empty()
                && ext.len() <= 12
                && ext.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
        }
        None => false,
    }
}

/// 퍼센트 표기(`%20` 등)를 풂. 잘못된 표기는 그대로 둠.
pub fn decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = |c: u8| (c as char).to_digit(16);
            if let (Some(x), Some(y)) = (h(b[i + 1]), h(b[i + 2])) {
                out.push((x * 16 + y) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 경로를 글자 그대로 정리(`.` 제거, `..` 적용). 파일 시스템은 보지 않음.
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// `p`가 `root` 아래인지(글자 기준). Windows는 대소문자와 구분자를 가리지 않고 `\\?\` 접두를 뺌.
pub fn inside(root: &Path, p: &Path) -> bool {
    let key = |x: &Path| {
        let s = normalize(x).to_string_lossy().replace('\\', "/");
        let s = s.strip_prefix("//?/").map(str::to_string).unwrap_or(s);
        let s = s.trim_end_matches('/').to_string();
        if cfg!(windows) { s.to_lowercase() } else { s }
    };
    let (r, q) = (key(root), key(p));
    q == r || q.starts_with(&format!("{r}/"))
}

/// 프로젝트 밖 판정과 자동 수정의 경계(S056 t02): 프로젝트 루트부터 위로 올라가며 `.git`(폴더, 또는 worktree의
/// 파일)이 있는 첫 폴더, 없으면 프로젝트 루트. 같은 저장소의 파일은 기록과 함께 보존되므로 안으로 봄 — 저장소의
/// 하위 폴더 프로젝트(예: 기록 폴더가 제품 코드를 인용)도 같게. git 프로그램은 쓰지 않고 존재만 확인.
pub fn boundary(root: &Path) -> PathBuf {
    let mut d = Some(root);
    while let Some(x) = d {
        if x.join(".git").exists() {
            return x.to_path_buf();
        }
        d = x.parent();
    }
    root.to_path_buf()
}

/// 환경 변수(`%VAR%`, `$VAR`)나 홈(`~/`)으로 시작 — 이 기계에만 있는 경로.
pub fn is_env_or_home(s: &str) -> bool {
    s.starts_with('%') || s.starts_with('$') || s == "~" || s.starts_with("~/") || s.starts_with("~\\")
}

/// 절대 경로(`/…`, `\…`, `C:/…`, `C:\…`).
pub fn is_absolute(s: &str) -> bool {
    let b = s.as_bytes();
    s.starts_with('/') || s.starts_with('\\') || (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'/' || b[2] == b'\\'))
}

/// 경계(`boundary`) 밖을 가리키는지: 환경 변수와 홈, 경계 밖 절대 경로, 어느 기준 폴더에서 풀어도 경계를 벗어나는
/// 상대 경로.
pub fn outside(bound: &Path, bases: &[&Path], s: &str) -> bool {
    if is_env_or_home(s) {
        return true;
    }
    let d = decode(s);
    if is_absolute(&d) {
        return !inside(bound, Path::new(&d));
    }
    bases.iter().all(|b| !inside(bound, &b.join(&d)))
}

/// `base` 기준으로 풀었을 때 파일(또는 폴더)이 있는지.
pub fn exists_from(base: &Path, s: &str) -> bool {
    let d = decode(s);
    if is_absolute(&d) { Path::new(&d).exists() } else { base.join(&d).exists() }
}

/// 폴더 한 단계 차이 보정: `../`를 하나 더하거나 하나 빼서 한쪽에서만 경계 안 파일이 찾아지면 그 경로.
/// 양쪽 모두 찾아지거나(어느 쪽인지 판단 불가) 둘 다 없으면 None.
pub fn one_level_fix(dir: &Path, raw: &str, bound: &Path) -> Option<String> {
    if is_env_or_home(raw) || is_absolute(&decode(raw)) {
        return None;
    }
    let ok = |c: &str| {
        let p = dir.join(decode(c));
        inside(bound, &p) && p.exists()
    };
    let up = format!("../{raw}");
    let down = raw.strip_prefix("../").map(str::to_string);
    let u = ok(&up);
    let w = down.as_deref().is_some_and(ok);
    match (u, w) {
        (true, false) => Some(up),
        (false, true) => down,
        _ => None,
    }
}

/// 고칠 자리 하나(내용 전체 기준 바이트 범위, 이전 값, 새 값).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub start: usize,
    pub end: usize,
    pub old: String,
    pub new: String,
}

/// 문서 하나의 링크 판정.
#[derive(Debug, Default)]
pub struct Scan {
    pub fixes: Vec<Fix>,
    pub broken: Vec<String>,
    pub outside: Vec<String>,
}

/// 문서 내용의 링크를 판정. `bound` = `boundary(프로젝트 루트)`, `check_outside` = 경계 밖 링크를 따로 모음(진행 중 로그).
pub fn scan(content: &str, dir: &Path, bound: &Path, check_outside: bool) -> Scan {
    let mut r = Scan::default();
    for l in links(content) {
        if check_outside && outside(bound, &[dir], &l.raw) {
            r.outside.push(l.raw);
            continue;
        }
        if exists_from(dir, &l.raw) {
            continue;
        }
        match one_level_fix(dir, &l.raw, bound) {
            Some(new) => r.fixes.push(Fix { start: l.start, end: l.end, old: l.raw, new }),
            None => r.broken.push(l.raw),
        }
    }
    r
}

/// 고칠 자리를 내용에 반영(뒤에서부터 — 앞쪽 범위가 밀리지 않음). 경로 글자만 바뀌고 나머지는 그대로.
pub fn apply(content: &str, fixes: &[Fix]) -> String {
    let mut sorted: Vec<&Fix> = fixes.iter().collect();
    sorted.sort_by_key(|f| f.start);
    let mut s = content.to_string();
    for f in sorted.iter().rev() {
        s.replace_range(f.start..f.end, &f.new);
    }
    s
}

/// 프로젝트의 md 문서(정렬). 점으로 시작하는 폴더(`.git`, `.elf`, `.claude` 등 도구 영역)는 들어가지 않고,
/// `skip`(세션 로그, Registry)은 뺌.
pub fn project_docs(root: &Path, skip: &BTreeSet<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if !name.starts_with('.') {
                    stack.push(e.path());
                }
            } else if ft.is_file() && name.to_ascii_lowercase().ends_with(".md") {
                let p = e.path();
                if !skip.contains(&p) {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}

/// 경고에 싣는 대상 목록(앞 3개 + 나머지 수).
pub fn sample(items: &[String]) -> String {
    let mut s = items.iter().take(3).map(|t| format!("`{t}`")).collect::<Vec<_>>().join(", ");
    if items.len() > 3 {
        s.push_str(&format!(" (+{} more)", items.len() - 3));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raws(md: &str) -> Vec<String> {
        links(md).into_iter().map(|l| l.raw).collect()
    }

    #[test]
    fn recognizes_path_links_only() {
        let md = "[a](rel/x.md) [b](../y.md#sec) ![i](../../6_Exp/f.png \"cap\") [ang](<a b.md>) [dir](../Archive/)\n";
        assert_eq!(raws(md), vec!["rel/x.md", "../y.md", "../../6_Exp/f.png", "a b.md", "../Archive/"]);
    }

    #[test]
    fn skips_prose_urls_placeholders_and_code() {
        let md = "Supp [6](supp_v3 2026-10-29 저장본), [15](27), [Yan](L608–611), [x](§17.9), [y](+in/across맥락)\n\
                  [u](https://e.com/a.md) [m](mailto:a@b.c) [d](doi:10.1/x.pdf) [h](#sec) [p](S{NNN}/f.png) [q](<file>.png)\n\
                  `[c](code.md)` and\n```\n[k](fence.md)\n```\n~~~\n[k2](tilde.md)\n~~~\n";
        assert!(raws(md).is_empty(), "{:?}", raws(md));
    }

    #[test]
    fn byte_ranges_point_at_the_path() {
        let md = "앞 줄\n그림 ![f](../6_Exp/a.png#x \"t\") 끝\n";
        let l = &links(md)[0];
        assert_eq!(&md[l.start..l.end], "../6_Exp/a.png");
        assert_eq!(l.line, 2);
    }

    #[test]
    fn decode_percent() {
        assert_eq!(decode("a%20b.md"), "a b.md");
        assert_eq!(decode("%E2%80%94.md"), "—.md");
        assert_eq!(decode("100%.md"), "100%.md");
        assert_eq!(decode("%zz.md"), "%zz.md");
    }

    #[test]
    fn inside_and_outside() {
        let root = Path::new("/p/proj");
        assert!(inside(root, Path::new("/p/proj/a/b")));
        assert!(inside(root, Path::new("/p/proj/a/../b")));
        assert!(!inside(root, Path::new("/p/proj/../other")));
        assert!(!inside(root, Path::new("/p/project2")));
        let logs = Path::new("/p/proj/2_Log/S");
        assert!(outside(root, &[root, logs], "%TEMP%/x.py"));
        assert!(outside(root, &[root, logs], "$HOME/x.py"));
        assert!(outside(root, &[root, logs], "~/x.py"));
        assert!(outside(root, &[root, logs], "../../../elf-cli/src/a.rs"));
        assert!(!outside(root, &[root, logs], "../../6_Exp/a.png"));
        assert!(!outside(root, &[root, logs], "6_Exp/a.png"));
    }

    #[test]
    fn boundary_is_the_nearest_folder_with_git() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let proj = repo.join("sub/proj");
        fs::create_dir_all(&proj).unwrap();
        assert_eq!(boundary(&proj), proj, "no .git above: the project root");
        fs::write(repo.join(".git"), "gitdir: elsewhere\n").unwrap(); // worktree = 파일
        assert_eq!(boundary(&proj), repo);
        fs::create_dir_all(proj.join(".git")).unwrap();
        assert_eq!(boundary(&proj), proj, "the nearest one wins");
    }

    #[test]
    fn apply_replaces_only_paths() {
        let md = "a ![f](../6_Exp/a.png) b [g](../x.md \"t\")\r\nc\r\n";
        let ls = links(md);
        let fixes: Vec<Fix> = ls
            .iter()
            .map(|l| Fix { start: l.start, end: l.end, old: l.raw.clone(), new: format!("../{}", l.raw) })
            .collect();
        assert_eq!(apply(md, &fixes), "a ![f](../../6_Exp/a.png) b [g](../../x.md \"t\")\r\nc\r\n");
    }

    #[test]
    fn sample_lists_three() {
        let v: Vec<String> = ["a", "b", "c", "d", "e"].iter().map(|s| s.to_string()).collect();
        assert_eq!(sample(&v), "`a`, `b`, `c` (+2 more)");
    }
}
