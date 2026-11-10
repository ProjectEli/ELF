//! 이전 배치의 ELF 파일 판정 — `elf update`가 어느 2.x 프로젝트든 3.x로 올리는 데 씀(v3.0.2, S046).
//!
//! - `.elf/` 없는 2.0~2.4.0 프로젝트: generator(2.0~2.3)가 만들었거나 2.4.0 `init`이 도중에 멈춤. 흔적 =
//!   `.elf/manifest.json`이 없고 `0_Meta/`에 ELF 규칙 파일이 있음. `elf update`는 있는 ELF 파일로 합성 stamp를 만들어
//!   보통 갱신 경로로 처리함(원본 판정 = `earlier_files` 지문).
//! - 2.15 이전 배치: 규칙이 `0_Meta/`, 틀이 `templates/`. 현재 위치(`.elf/managed/`)로 옮기는 대응표.

use std::fs;
use std::path::{Path, PathBuf};

use crate::earlier_files::{KNOWN, Print};
use crate::hash;

/// 합성 stamp에서 "사용자가 고침"을 뜻하는 sha 자리 값(어떤 판의 sha와도 맞지 않음).
pub const EDITED: &str = "edited";

/// `0_Meta/`의 ELF 규칙 파일(이 중 하나라도 있으면 ELF 2.x 프로젝트의 흔적).
const RULE_MARKS: [&str; 3] = ["0_Meta/EliRule.md", "0_Meta/LogConvention.md", "0_Meta/AI_PARA_Framework.md"];

/// `.elf/` 없는 2.0~2.4.0 프로젝트의 흔적: `.elf/manifest.json`이 없고 `0_Meta/`에 ELF 규칙 파일이 있음.
pub fn footprint(root: &Path) -> bool {
    !root.join(".elf").join("manifest.json").is_file() && RULE_MARKS.iter().any(|m| root.join(m).is_file())
}

/// cwd에서 위로 올라가며 흔적이 있는 폴더(가장 가까운 것). `.elf/manifest.json` 프로젝트 안이면 None.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    for a in start.ancestors() {
        if a.join(".elf").join("manifest.json").is_file() {
            return None;
        }
        if footprint(a) {
            return Some(a.to_path_buf());
        }
    }
    None
}

fn lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn sha(bytes: &[u8]) -> String {
    hash::sha256_lf(bytes)
}

/// 이전 배치에서 ELF가 넣던 위치인가(지문 표의 위치, `0_Meta/`의 ELF 규칙 이름). `elf update`가 옮기거나 바꾸거나 지움.
pub fn delivered_dest(rel: &str) -> bool {
    KNOWN.iter().any(|k| k.dest == rel)
        || rel.strip_prefix("0_Meta/").is_some_and(|n| {
            ["EliRule", "LogConvention", "AI_PARA_Framework", "highIFjournals", "LLMcliche"]
                .iter()
                .any(|b| n == format!("{b}.md") || n == format!("{b}.en.md"))
        })
}

/// 이전 버전이 넣은 원본 그대로면 Some((처음 나온 버전, 언어 한 줄 — EliRule만)).
pub fn pristine(dest: &str, bytes: &[u8]) -> Option<(&'static str, Option<String>)> {
    let norm = lf(bytes);
    let whole = sha(&norm);
    let body: &[u8] = norm.strip_prefix(b"\xef\xbb\xbf".as_slice()).unwrap_or(&norm);
    for k in KNOWN.iter().filter(|k| k.dest == dest) {
        match &k.print {
            Print::Exact(s) if *s == whole => return Some((k.since, None)),
            Print::Around { head_len, head, tail_len, tail } => {
                let (h, t) = (*head_len, *tail_len);
                if body.len() < h + t || sha(&body[..h]) != *head || sha(&body[body.len() - t..]) != *tail {
                    continue;
                }
                let mid = &body[h..body.len() - t];
                if mid.len() <= 64 && !mid.contains(&b'\n') {
                    return Some((k.since, Some(String::from_utf8_lossy(mid).into_owned())));
                }
            }
            Print::Exact(_) => {}
        }
    }
    None
}

/// 합성 stamp(schema 1 JSON) — 지문 표에 있는 ELF 파일 가운데 있는 것마다 한 항목. sha = 원본 그대로면 현재 파일의
/// sha(= 고치지 않음), 고쳤으면 `edited`(어떤 판과도 맞지 않음 → 기존 충돌 처리).
pub fn synthetic_stamp(root: &Path) -> String {
    let mut dests: Vec<(&str, &str)> = Vec::new();
    for k in KNOWN {
        if !dests.iter().any(|(d, _)| *d == k.dest) {
            dests.push((k.dest, k.tier));
        }
    }
    let mut files = Vec::new();
    for (dest, tier) in dests {
        let Ok(bytes) = fs::read(root.join(dest)) else { continue };
        let sha256 = if pristine(dest, &bytes).is_some() { hash::sha256_lf(&bytes) } else { EDITED.to_string() };
        files.push(serde_json::json!({ "src": format!("earlier/{dest}"), "dest": dest, "tier": tier, "sha256": sha256 }));
    }
    serde_json::json!({ "schema": crate::manifest::SCHEMA_V1, "generated": "", "note": "synthetic: ELF 2.0–2.4.0 project without .elf/", "files": files }).to_string()
}

/// 프로젝트 언어(BCP-47) — generator가 EliRule에 쓴 언어 한 줄에서. 모르면 "".
pub fn guessed_lang(root: &Path) -> String {
    let Ok(bytes) = fs::read(root.join("0_Meta/EliRule.md")) else { return String::new() };
    match pristine("0_Meta/EliRule.md", &bytes).and_then(|(_, l)| l) {
        Some(l) if l.trim() == "English" => "en".into(),
        Some(l) if l.trim() == "한국어" => "ko".into(),
        _ => String::new(),
    }
}

/// 2.15 이전 배치의 위치 → 현재 위치(`0_Meta/X.md` → `.elf/managed/X.md`, `templates/X` → `.elf/managed/templates/X`).
/// 실제로 옮길지는 현재 manifest에 그 위치가 있는지로 정함(호출하는 쪽).
pub fn relocated(dest: &str) -> Option<String> {
    if let Some(name) = dest.strip_prefix("0_Meta/") {
        return (!name.contains('/') && name.ends_with(".md")).then(|| format!(".elf/managed/{name}"));
    }
    dest.strip_prefix("templates/").filter(|n| !n.contains('/')).map(|n| format!(".elf/managed/templates/{n}"))
}

/// 지운 파일 위의 빈 폴더를 지움 — 루트 바로 아래의 숫자 폴더(`0_Meta` 등)와 `.elf`, `.git`은 남김.
pub fn prune_empty_parents(root: &Path, dest: &str) {
    let mut cur = Path::new(dest).parent();
    while let Some(dir) = cur {
        let s = dir.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            break;
        }
        let top = !s.contains('/');
        if top && (s.starts_with(|c: char| c.is_ascii_digit()) || s.starts_with('.')) {
            break;
        }
        if s == ".elf" || s == ".elf/managed" {
            break;
        }
        // 남은 것이 `.gitkeep`뿐이면 함께 지움(빈 폴더 표시용 — 이관과 같은 규칙)
        let full = root.join(dir);
        if fs::read_dir(&full).is_ok_and(|rd| rd.flatten().all(|e| e.file_name() == ".gitkeep")) {
            let _ = fs::remove_file(full.join(".gitkeep"));
        }
        if fs::remove_dir(&full).is_err() {
            break;
        }
        cur = dir.parent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocation_table() {
        assert_eq!(relocated("0_Meta/EliRule.md").as_deref(), Some(".elf/managed/EliRule.md"));
        assert_eq!(relocated("0_Meta/EliRule.en.md").as_deref(), Some(".elf/managed/EliRule.en.md"));
        assert_eq!(relocated("templates/sessionTemplate.md").as_deref(), Some(".elf/managed/templates/sessionTemplate.md"));
        assert_eq!(relocated("0_Meta/scripts/elf_gallery.ps1"), None);
        assert_eq!(relocated(".gitignore"), None);
    }

    #[test]
    fn unknown_content_is_not_pristine() {
        assert_eq!(pristine("0_Meta/EliRule.md", b"# my own rules\n"), None);
        assert_eq!(pristine("0_Meta/nothing.md", b""), None);
    }

    #[test]
    fn footprint_needs_rules_and_no_manifest() {
        let t = tempfile::tempdir().unwrap();
        assert!(!footprint(t.path()));
        fs::create_dir_all(t.path().join("0_Meta")).unwrap();
        fs::write(t.path().join("0_Meta/LogConvention.md"), "x").unwrap();
        assert!(footprint(t.path()));
        fs::create_dir_all(t.path().join("sub/deeper")).unwrap();
        assert_eq!(find_root(&t.path().join("sub/deeper")).as_deref(), Some(t.path()));
        fs::create_dir_all(t.path().join(".elf")).unwrap();
        fs::write(t.path().join(".elf/manifest.json"), "{}").unwrap();
        assert!(!footprint(t.path()));
        assert_eq!(find_root(&t.path().join("sub")), None);
    }

    #[test]
    fn prune_stops_at_numbered_top_folders() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join("0_Meta/scripts")).unwrap();
        fs::create_dir_all(t.path().join("templates")).unwrap();
        fs::write(t.path().join("templates/.gitkeep"), "").unwrap();
        prune_empty_parents(t.path(), "0_Meta/scripts/a.ps1");
        assert!(!t.path().join("0_Meta/scripts").exists() && t.path().join("0_Meta").is_dir());
        prune_empty_parents(t.path(), "templates/sessionTemplate.md");
        assert!(!t.path().join("templates").exists());
    }
}
