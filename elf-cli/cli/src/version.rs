//! 버전 판정 — CLI와 프로젝트 manifest(`.elf/manifest.json`) 비교 (P022 §8, S044 t41).
//! `elf check version`이 보여 주고, 쓰기 명령(session, trial, deprecate, update)이 시작할 때 같은 판정을 씀.
//! MAJOR.MINOR만 비교(patch 차이 허용). 프로젝트가 높으면 쓰기 거부, 읽기는 경고 후 실행.

use std::fs;
use std::path::Path;

use crate::{embed, manifest, update};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// 프로젝트 밖(`.elf/manifest.json` 없음)
    NoProject,
    /// manifest를 읽을 수 없음, 형식 오류, 버전 표기 해석 불가
    Unreadable(String),
    /// 이전 형식(schema 1) — 기록 버전 = `.elf/version`(있으면). `elf update`로 이관
    Earlier(Option<String>),
    /// MAJOR.MINOR 같음(프로젝트 기록 버전)
    Same(String),
    /// 프로젝트가 낮음 → `elf update`
    Older(String),
    /// 프로젝트가 높음 → `elf self update`, 그 전까지 쓰기 거부
    Newer(String),
}

/// `v2.28.0`, `2.28.1-dev` → (2, 28).
pub fn major_minor(v: &str) -> Option<(u32, u32)> {
    let mut it = v.trim().trim_start_matches('v').split(['.', '-']);
    Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
}

/// 프로젝트 루트(manifest가 있는 폴더) 기준 판정. `cli` = 비교할 CLI 버전(주입형, 테스트용).
pub fn verdict_with(root: &Path, cli: &str) -> Verdict {
    let p = root.join(".elf").join("manifest.json");
    let Ok(text) = fs::read_to_string(&p) else {
        return Verdict::NoProject;
    };
    let m = match manifest::parse(&text) {
        Ok(m) => m,
        Err(e) => return Verdict::Unreadable(e),
    };
    if m.schema == manifest::SCHEMA_V1 {
        let old = fs::read_to_string(root.join(".elf").join("version")).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        return Verdict::Earlier(old);
    }
    let Some(pv) = m.elf_version.clone() else {
        return Verdict::Unreadable("elf_version missing".into());
    };
    match (major_minor(&pv), major_minor(cli)) {
        (Some(a), Some(b)) if a == b => Verdict::Same(pv),
        (Some(a), Some(b)) if a < b => Verdict::Older(pv),
        (Some(_), Some(_)) => Verdict::Newer(pv),
        _ => Verdict::Unreadable(format!("elf_version \"{pv}\" is not a version")),
    }
}

/// cwd에서 위로 프로젝트를 찾아 판정.
pub fn verdict(cwd: &Path) -> Verdict {
    match update::find_project_root(cwd) {
        Some(root) => verdict_with(&root, embed::version()),
        None => Verdict::NoProject,
    }
}

/// 판정 한 줄과 수준 — ('o' ok | 'w' warn | 'i' issue | 'n' info, 내용).
pub fn verdict_line(v: &Verdict, cli: &str) -> (char, String) {
    match v {
        Verdict::NoProject => ('n', "not inside an ELF project — CLI only".into()),
        Verdict::Unreadable(e) => ('i', format!("project manifest unreadable ({e}) — run `elf update`, or restore .elf/manifest.json from git")),
        Verdict::Earlier(old) => (
            'w',
            format!(
                "project manifest is the earlier format (schema 1{}) — run `elf update` to migrate (LogConvention §7)",
                old.as_deref().map(|o| format!(", ELF {o}")).unwrap_or_default()
            ),
        ),
        Verdict::Same(p) => ('o', format!("project {p}, CLI {cli} — same MAJOR.MINOR")),
        Verdict::Older(p) => ('w', format!("project {p} is older than CLI {cli} — run `elf update`")),
        Verdict::Newer(p) => (
            'i',
            format!("project {p} is newer than CLI {cli} — run `elf self update` (commands that write refuse until then)"),
        ),
    }
}

/// 쓰기 명령 관문 — 프로젝트가 더 높으면 거부 문구, 그 밖은 통과(이전 형식은 각 명령이 따로 안내).
pub fn gate_write(cwd: &Path) -> Result<(), String> {
    match verdict(cwd) {
        v @ Verdict::Newer(_) => Err(format!("{} (details: `elf check version`)", verdict_line(&v, embed::version()).1)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(json: &str, version_file: Option<&str>) -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join(".elf")).unwrap();
        fs::write(t.path().join(".elf/manifest.json"), json).unwrap();
        if let Some(v) = version_file {
            fs::write(t.path().join(".elf/version"), v).unwrap();
        }
        t
    }

    fn v2(ver: &str) -> String {
        format!(r#"{{ "schema": "elf-manifest/2", "elf_version": "{ver}", "files": [] }}"#)
    }

    #[test]
    fn major_minor_parses_tags() {
        assert_eq!(major_minor("v2.28.0"), Some((2, 28)));
        assert_eq!(major_minor("2.28.1-dev"), Some((2, 28)));
        assert_eq!(major_minor("v2.4-dev"), Some((2, 4)));
        assert_eq!(major_minor("x"), None);
    }

    #[test]
    fn verdicts_compare_major_minor_only() {
        assert_eq!(verdict_with(project(&v2("v2.28.0"), None).path(), "v2.28.3"), Verdict::Same("v2.28.0".into()));
        assert_eq!(verdict_with(project(&v2("v2.27.4"), None).path(), "v2.28.0"), Verdict::Older("v2.27.4".into()));
        assert_eq!(verdict_with(project(&v2("v2.29.0"), None).path(), "v2.28.9"), Verdict::Newer("v2.29.0".into()));
        assert_eq!(verdict_with(project(&v2("v3.0.0"), None).path(), "v2.28.0"), Verdict::Newer("v3.0.0".into()));
        let old = project(r#"{ "schema": "elf-manifest/1", "files": [] }"#, Some("v2.27.4\n"));
        assert_eq!(verdict_with(old.path(), "v2.28.0"), Verdict::Earlier(Some("v2.27.4".into())));
        assert!(matches!(verdict_with(project("{", None).path(), "v2.28.0"), Verdict::Unreadable(_)));
        assert!(matches!(verdict_with(project(&v2("dev"), None).path(), "v2.28.0"), Verdict::Unreadable(_)));
        assert_eq!(verdict_with(tempfile::tempdir().unwrap().path(), "v2.28.0"), Verdict::NoProject);
    }

    #[test]
    fn gate_refuses_only_newer_project() {
        let newer = project(&v2("v99.0.0"), None);
        let e = gate_write(newer.path()).unwrap_err();
        assert!(e.contains("elf self update") && e.contains("elf check version"), "{e}");
        assert!(gate_write(project(&v2("v0.1.0"), None).path()).is_ok());
        assert!(gate_write(project(r#"{ "schema": "elf-manifest/1", "files": [] }"#, None).path()).is_ok());
    }
}
