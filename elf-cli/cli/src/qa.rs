//! qa preset 번들 검사(v3.8 S057): `.elf/config.json` `preset: qa` 프로젝트에서 `elf check logs`가 번들을 검사.
//!
//! 판단 없이 계산되는 것만 봄 — frontmatter 키와 값, category와 폴더, status와 위치(카테고리 루트 = active,
//! `archive/` = archive 또는 deprecated), 파일 이름(카테고리 README가 있으면 그 카테고리는 생략), 같은 날짜 글자 중복,
//! `> 관련 문서:`/`> 대체 문서:` 줄의 경로. 결과는 모두 경고. 번들 아닌 것: 카테고리 README, `_`로 시작하는 폴더,
//! `archive/` 말고 그 밖의 하위 폴더(참고 자료). 자동 수정(직접 실행한 check, `fix_links`) = archive로 옮겨진(또는
//! archive에서 꺼낸) 번들을 가리키는 경로, 마크다운 링크의 폴더 한 단계 차이.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::links;

/// `.elf/config.json` `preset`이 `qa`인 프로젝트.
pub fn is_qa(root: &Path) -> bool {
    crate::update::read_config_preset(root).as_deref() == Some("qa")
}

struct Bundle {
    path: PathBuf,
    rel: String,
    category: String,
    archived: bool,
    name: String,
}

/// 카테고리 = 루트 직속 폴더 중 점, `_`로 시작하지 않고 `0_Meta`가 아닌 것(정렬).
fn categories(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(root)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.') && !n.starts_with('_') && n != "0_Meta")
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

fn is_readme(name: &str) -> bool {
    name.eq_ignore_ascii_case("README.md")
}

/// 카테고리 루트와 `archive/`의 md 중 README가 아닌 것.
fn bundles(root: &Path, cat: &str) -> Vec<Bundle> {
    let mut out = Vec::new();
    for (dir, archived) in [(root.join(cat), false), (root.join(cat).join("archive"), true)] {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        let mut names: Vec<String> = rd
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.to_ascii_lowercase().ends_with(".md") && !is_readme(n))
            .collect();
        names.sort();
        for name in names {
            let rel = if archived { format!("{cat}/archive/{name}") } else { format!("{cat}/{name}") };
            out.push(Bundle { path: dir.join(&name), rel, category: cat.to_string(), archived, name });
        }
    }
    out
}

/// frontmatter의 `키: 값` 목록(값의 ` #` 뒤 주석은 뺌). `---` 블록이 없으면 None.
fn frontmatter(text: &str) -> Option<Vec<(String, String)>> {
    let t = text.strip_prefix('\u{feff}').unwrap_or(text).replace("\r\n", "\n");
    let rest = t.strip_prefix("---\n")?;
    let end = if rest.starts_with("---") { 0 } else { rest.find("\n---")? + 1 };
    let mut out = Vec::new();
    for l in rest[..end].lines() {
        if l.starts_with('#') || l.starts_with(' ') || l.starts_with('\t') {
            continue;
        }
        let Some((k, v)) = l.split_once(':') else { continue };
        if k.is_empty() || !k.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            continue;
        }
        let v = match v.find(" #") {
            Some(i) => &v[..i],
            None => v,
        };
        out.push((k.to_string(), v.trim().to_string()));
    }
    Some(out)
}

/// `YYYY-MM-DD HH:MM` 또는 `YYYY-MM-DD HH:MM:SS`.
fn is_date(v: &str) -> bool {
    let b = v.as_bytes();
    let digits = |r: std::ops::Range<usize>| r.into_iter().all(|i| b[i].is_ascii_digit());
    let base = b.len() >= 16
        && digits(0..4)
        && b[4] == b'-'
        && digits(5..7)
        && b[7] == b'-'
        && digits(8..10)
        && b[10] == b' '
        && digits(11..13)
        && b[13] == b':'
        && digits(14..16);
    base && (b.len() == 16 || (b.len() == 19 && b[16] == b':' && digits(17..19)))
}

/// `YYYYMMDD<L>_<Subject>.md` — 같은 날짜 글자 키(`YYYYMMDDL`)를 돌려줌.
fn standard_name(name: &str) -> Option<String> {
    let b = name.as_bytes();
    let ok = b.len() > 13
        && b[..8].iter().all(|c| c.is_ascii_digit())
        && b[8].is_ascii_uppercase()
        && b[9] == b'_'
        && name.to_ascii_lowercase().ends_with(".md")
        && !name[10..name.len() - 3].trim().is_empty();
    ok.then(|| name[..9].to_string())
}

/// 관련 문서와 대체 문서 줄(한영).
fn is_ref_line(l: &str) -> bool {
    let t = l.trim_start();
    ["> 관련 문서", "> 대체 문서", "> Related", "> Replaced by"].iter().any(|p| t.starts_with(p))
}

/// 참조 줄의 경로(내용 전체 기준 바이트 범위): 백틱 안 `.md` 경로와 마크다운 링크 대상.
fn ref_paths(content: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in content.split_inclusive('\n') {
        let base = offset;
        offset += line.len();
        let l = line.trim_end_matches(['\n', '\r']);
        if !is_ref_line(l) {
            continue;
        }
        let mut pos = 0usize;
        for (i, span) in l.split('`').enumerate() {
            let start = pos;
            pos += span.len() + 1;
            let p = span.trim();
            if i % 2 == 1 && p.to_ascii_lowercase().ends_with(".md") && !p.contains(['{', '}', '<', '>', '*', ' ']) {
                let s = base + start + (span.len() - span.trim_start().len());
                out.push((s, s + p.len(), p.to_string()));
            }
        }
        for lk in links::links(l) {
            if lk.raw.to_ascii_lowercase().ends_with(".md") {
                out.push((base + lk.start, base + lk.end, lk.raw));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// archive 이동 보정: 경로 글자에서 마지막 파일 앞의 `archive/`를 빼거나 넣음.
fn toggle_archive(p: &str) -> String {
    match p.rsplit_once('/') {
        Some(("archive", file)) => file.to_string(),
        Some((dir, file)) if dir.ends_with("/archive") => format!("{}/{file}", &dir[..dir.len() - "/archive".len()]),
        Some((dir, file)) => format!("{dir}/archive/{file}"),
        None => format!("archive/{p}"),
    }
}

/// 참조 경로 판정: 있음 → None, 고칠 경로 → Some(Some(new)), 깨짐 → Some(None).
fn judge_ref(p: &str, root: &Path, dir: &Path, bound: &Path) -> Option<Option<String>> {
    let found = |s: &str| links::exists_from(root, s) || links::exists_from(dir, s);
    if found(p) {
        return None;
    }
    if links::is_env_or_home(p) || links::is_absolute(&links::decode(p)) {
        return Some(None);
    }
    let alt = toggle_archive(p);
    let inside = |s: &str| {
        [root, dir]
            .iter()
            .any(|b| links::inside(bound, &b.join(links::decode(s))) && b.join(links::decode(s)).exists())
    };
    if inside(&alt) {
        return Some(Some(alt));
    }
    Some(links::one_level_fix(dir, p, bound))
}

/// qa 검사 줄(`warn: `, `fixed: `, `info: `, `ok: `). `fix` = 고쳐 씀(직접 실행한 check, `fix_links`).
pub fn run_check(root: &Path, fix: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut warns = 0usize;
    let mut warn = |lines: &mut Vec<String>, s: String| {
        warns += 1;
        lines.push(format!("warn: {s}"));
    };
    let bound = links::boundary(root);
    let cats = categories(root);
    let mut fixed = (0usize, 0usize);
    let mut bundle_paths: BTreeSet<PathBuf> = BTreeSet::new();
    let mut count = 0usize;
    for cat in &cats {
        let has_readme = fs::read_dir(root.join(cat))
            .map(|rd| rd.flatten().any(|e| is_readme(&e.file_name().to_string_lossy())))
            .unwrap_or(false);
        let mut letters: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for b in bundles(root, cat) {
            count += 1;
            bundle_paths.insert(b.path.clone());
            let Ok(bytes) = fs::read(&b.path) else { continue };
            let Ok(text) = String::from_utf8(bytes) else {
                warn(&mut lines, format!("{}: not UTF-8 text", b.rel));
                continue;
            };
            match standard_name(&b.name) {
                Some(key) => letters.entry(key).or_default().push(b.rel.clone()),
                None if !has_readme => warn(
                    &mut lines,
                    format!("{}: the name is not YYYYMMDD<L>_<Subject>.md (a category README can set its own naming)", b.rel),
                ),
                None => {}
            }
            match frontmatter(&text) {
                None => warn(&mut lines, format!("{}: no frontmatter — start the bundle with the `---` block (AGENTS.md §3)", b.rel)),
                Some(fm) => {
                    let get = |k: &str| fm.iter().find(|(x, _)| x == k).map(|(_, v)| v.as_str());
                    let missing: Vec<&str> =
                        ["date", "category", "tags", "status"].into_iter().filter(|k| get(k).is_none()).collect();
                    if !missing.is_empty() {
                        warn(&mut lines, format!("{}: the frontmatter lacks {}", b.rel, missing.join(", ")));
                    }
                    if let Some(d) = get("date")
                        && !is_date(d)
                    {
                        warn(&mut lines, format!("{}: date `{d}` is not YYYY-MM-DD HH:MM:SS", b.rel));
                    }
                    if let Some(c) = get("category")
                        && c != b.category
                    {
                        warn(&mut lines, format!("{}: category `{c}` differs from its folder `{}`", b.rel, b.category));
                    }
                    if let Some(t) = get("tags")
                        && !(t.starts_with('[') && t.ends_with(']') && !t[1..t.len() - 1].trim().is_empty())
                    {
                        warn(&mut lines, format!("{}: tags is empty or not a [list]", b.rel));
                    }
                    if let Some(s) = get("status") {
                        if !["active", "archive", "deprecated"].contains(&s) {
                            warn(&mut lines, format!("{}: status `{s}` is not active, archive or deprecated", b.rel));
                        } else if b.archived && s == "active" {
                            warn(&mut lines, format!(
                                "{}: status active, but the bundle is in archive/ — the folder decides: set status archive (or deprecated)",
                                b.rel
                            ));
                        } else if !b.archived && s != "active" {
                            warn(&mut lines, format!(
                                "{}: status {s}, but the bundle is at the category root — move it to archive/ or set status active",
                                b.rel
                            ));
                        }
                    }
                    if get("thread").is_some_and(str::is_empty) {
                        warn(&mut lines, format!("{}: thread is empty — omit the line for a standalone bundle", b.rel));
                    }
                }
            }
            // 참조 경로(관련 문서, 대체 문서)와 그 밖의 마크다운 링크의 폴더 한 단계 차이
            let dir = b.path.parent().unwrap_or(root);
            let mut fixes: Vec<links::Fix> = Vec::new();
            let mut broken: Vec<String> = Vec::new();
            for (s, e, p) in ref_paths(&text) {
                match judge_ref(&p, root, dir, &bound) {
                    None => {}
                    Some(Some(new)) => fixes.push(links::Fix { start: s, end: e, old: p, new }),
                    Some(None) => broken.push(p),
                }
            }
            if let Some(f) = fixes.first() {
                if fix {
                    if fs::write(&b.path, links::apply(&text, &fixes)).is_ok() {
                        lines.push(format!(
                            "fixed: {}: {} path(s) to bundles moved to or from archive/ (e.g. `{}` → `{}`)",
                            b.rel,
                            fixes.len(),
                            f.old,
                            f.new
                        ));
                        fixed.0 += fixes.len();
                        fixed.1 += 1;
                    }
                } else {
                    warn(&mut lines, format!(
                        "{}: {} path(s) point to bundles moved to or from archive/ (e.g. `{}` → `{}`) — `elf check logs` fixes them",
                        b.rel,
                        fixes.len(),
                        f.old,
                        f.new
                    ));
                }
            }
            if !broken.is_empty() {
                warn(&mut lines, format!(
                    "{}: {} related or replaced-by path(s) do not exist → {}",
                    b.rel,
                    broken.len(),
                    links::sample(&broken)
                ));
            }
        }
        for (key, rels) in letters.iter().filter(|(_, v)| v.len() > 1) {
            warn(&mut lines, format!(
                "{cat}: {key} is used by {} bundles (a letter is unique per date across the root and archive/): {}",
                rels.len(),
                links::sample(rels)
            ));
        }
    }
    // 번들 아닌 문서(README, 참고 자료, 0_Meta): 링크 검사(v3.7과 같음). `_`로 시작하는 폴더 아래는 제외.
    for doc in links::project_docs(root, &bundle_paths) {
        let rel = doc.strip_prefix(root).unwrap_or(&doc).to_string_lossy().replace('\\', "/");
        if rel.split('/').any(|seg| seg.starts_with('_')) {
            continue;
        }
        let Ok(bytes) = fs::read(&doc) else { continue };
        let Ok(text) = String::from_utf8(bytes) else { continue };
        let dir = doc.parent().unwrap_or(root);
        let ls = links::scan(&text, dir, &bound, false);
        if let Some(f) = ls.fixes.first() {
            if fix {
                if fs::write(&doc, links::apply(&text, &ls.fixes)).is_ok() {
                    lines.push(format!(
                        "fixed: {rel}: {} link(s) one folder level (e.g. `{}` → `{}`)",
                        ls.fixes.len(),
                        f.old,
                        f.new
                    ));
                    fixed.0 += ls.fixes.len();
                    fixed.1 += 1;
                }
            } else {
                warn(&mut lines, format!(
                    "{rel}: {} broken link(s) are one folder level off (e.g. `{}` → `{}`) — `elf check logs` fixes them",
                    ls.fixes.len(),
                    f.old,
                    f.new
                ));
            }
        }
        if !ls.broken.is_empty() {
            warn(&mut lines, format!("{rel}: {} broken link(s) → {}", ls.broken.len(), links::sample(&ls.broken)));
        }
    }
    if fixed.0 > 0 {
        lines.push(format!(
            "fixed: {} path(s) in {} file(s) — to only report them, set \"{}\": false in .elf/config.json",
            fixed.0,
            fixed.1,
            links::CONFIG_KEY
        ));
    }
    lines.insert(0, format!("info: qa bundles: {count} in {} categories", cats.len()));
    if warns == 0 {
        lines.push("ok: bundle frontmatter, locations, names and related paths are consistent".into());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_keys_and_comments() {
        let fm = frontmatter("---\r\ndate: 2026-11-11 10:00:00\r\nstatus: active   # active | archive\r\n# thread: x\r\n---\r\n# 목표\n").unwrap();
        assert_eq!(fm, vec![("date".into(), "2026-11-11 10:00:00".into()), ("status".into(), "active".into())]);
        assert!(frontmatter("# no\n").is_none());
        assert!(frontmatter("---\ndate: x\n").is_none(), "unclosed");
    }

    #[test]
    fn dates_and_names() {
        assert!(is_date("2026-11-11 10:00:00") && is_date("2026-11-11 10:00"));
        assert!(!is_date("2026-11-11") && !is_date("2026/11/11 10:00:00"));
        assert_eq!(standard_name("20261111A_Gitea_Setup.md").as_deref(), Some("20261111A"));
        assert!(standard_name("Y001_Talk.md").is_none() && standard_name("20261111a_x.md").is_none());
    }

    #[test]
    fn toggles_archive_segment() {
        assert_eq!(toggle_archive("IT/20260101A_X.md"), "IT/archive/20260101A_X.md");
        assert_eq!(toggle_archive("IT/archive/20260101A_X.md"), "IT/20260101A_X.md");
        assert_eq!(toggle_archive("20260101A_X.md"), "archive/20260101A_X.md");
        assert_eq!(toggle_archive("archive/20260101A_X.md"), "20260101A_X.md");
        assert_eq!(toggle_archive("../IT/20260101A_X.md"), "../IT/archive/20260101A_X.md");
    }

    #[test]
    fn ref_paths_only_on_reference_lines() {
        let md = "답변 `src/main.md` 언급\n> 관련 문서: `IT/20260101A_X.md`, [y](../Daily/20260102A_Y.md)\n> Replaced by: `IT/archive/20260103A_Z.md`\n";
        let r: Vec<String> = ref_paths(md).into_iter().map(|(_, _, p)| p).collect();
        assert_eq!(r, vec!["IT/20260101A_X.md", "../Daily/20260102A_Y.md", "IT/archive/20260103A_Z.md"]);
        for (s, e, p) in ref_paths(md) {
            assert_eq!(&md[s..e], p);
        }
    }
}
