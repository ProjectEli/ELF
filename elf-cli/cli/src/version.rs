//! 버전 판정 — CLI와 프로젝트 manifest(`.elf/manifest.json`) 비교 (P022 §8, S044 t41).
//! `elf check version`이 보여 주고, 쓰기 명령(session, trial, deprecate, update)이 시작할 때 같은 판정을 씀.
//! 줄(MAJOR.MINOR, `pin::line_of` — 2.28.x = 3.0 줄)만 비교(patch 차이 허용, v3.0 S044 t56).
//! 프로젝트가 높거나 manifest schema가 높으면 쓰기 거부, 읽기는 경고 후 실행.

use std::fs;
use std::path::Path;

use crate::{embed, manifest, pin, update};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// 프로젝트 밖(`.elf/manifest.json` 없음)
    NoProject,
    /// manifest를 읽을 수 없음, 형식 오류, 버전 표기 해석 불가
    Unreadable(String),
    /// 이전 형식(schema 1) — 기록 버전 = `.elf/version`(있으면). `elf update`로 이관
    Earlier(Option<String>),
    /// `.elf/` 없는 2.0~2.4.0 프로젝트(규칙이 `0_Meta/`) — `elf update`가 `.elf/`를 만들고 이관(v3.0.2)
    EarlierLayout,
    /// MAJOR.MINOR 같음(프로젝트 기록 버전)
    Same(String),
    /// 프로젝트가 낮음 → `elf update`
    Older(String),
    /// 프로젝트가 높음 → `elf self update`, 그 전까지 쓰기 거부
    Newer(String),
    /// manifest schema가 이 CLI보다 높음(위임 규약이 바뀐 버전이 씀, R7) → `elf self update`, 그 전까지 쓰기 거부
    NewerSchema(String),
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
    if let Some(s) = pin::newer_schema(&text) {
        return Verdict::NewerSchema(s);
    }
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
    match (pin::parse(&pv).map(|v| v.line()), pin::parse(cli).map(|v| v.line())) {
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
        None if crate::earlier::find_root(cwd).is_some() => Verdict::EarlierLayout,
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
                "project manifest is the earlier format (schema 1{}) — `elf update` is recommended: it prints a summary and an approval code and migrates only after the user approves (`elf update --dry-run` to preview; LogConvention §7)",
                old.as_deref().map(|o| format!(", ELF {o}")).unwrap_or_default()
            ),
        ),
        Verdict::EarlierLayout => (
            'w',
            "ELF 2.0–2.4.0 project without .elf/ (rules in 0_Meta/) — `elf update` is recommended: it prints a summary and an approval code, then creates .elf/ and migrates only after the user approves (`elf update --dry-run` to preview)".into(),
        ),
        Verdict::Same(p) => {
            let line = pin::parse(p).map(|v| v.line().to_string()).unwrap_or_default();
            let note = if major_minor(p) == Some((2, 28)) { " (2.28.x is the 3.0 line)" } else { "" };
            ('o', format!("project {p}, CLI {cli} — same line {line}{note}"))
        }
        Verdict::Older(p) => (
            'w',
            format!(
                "project {p} is older than CLI {cli} — `elf update` is recommended: it prints a summary and an approval code and applies only after the user approves"
            ),
        ),
        Verdict::Newer(p) => (
            'i',
            format!("project {p} is newer than CLI {cli} — run `elf self update` (commands that write refuse until then)"),
        ),
        Verdict::NewerSchema(s) => (
            'i',
            format!("project manifest schema {s} is newer than CLI {cli} — run `elf self update` (commands that write refuse until then)"),
        ),
    }
}

/// 갱신 권장 한 줄(autoread 요약용, v3.0 S044 t60): 프로젝트 줄 < PATH의 elf 줄이면. PATH의 elf = 넘겨받은 프로세스면
/// `ELF_DISPATCHED`, 아니면 이 버전.
pub fn update_hint(root: &Path) -> Option<String> {
    let path_elf = std::env::var(crate::dispatch::ENV_DISPATCHED).ok().filter(|v| pin::parse(v).is_some()).unwrap_or_else(|| embed::version().to_string());
    let l = pin::parse(&path_elf)?.line();
    let p = pin::read(root).ok()??;
    if p.line >= l {
        return None;
    }
    let what = if l.generation() > p.line.generation() { ", with a log layout migration" } else { "" };
    Some(format!(
        "ELF update available: this project is on line {}, elf {path_elf} is installed{what}. Ask the user before running `elf update`; it prints a summary and an approval code, and applies only after the user approves",
        p.line
    ))
}

/// 실행 위임이 프로젝트 줄의 실행 파일을 구하지 못함(v3.0 S044 t59) — 이 프로세스의 쓰기 명령 거부 이유.
static UNAVAILABLE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_unavailable(msg: String) {
    let _ = UNAVAILABLE.set(msg);
}

/// 쓰기 명령 관문 — 프로젝트 줄의 실행 파일을 구하지 못했거나 프로젝트가 더 높으면 거부 문구, 그 밖은 통과
/// (이전 형식은 각 명령이 따로 안내).
pub fn gate_write(cwd: &Path) -> Result<(), String> {
    if let Some(m) = UNAVAILABLE.get() {
        return Err(m.clone());
    }
    match verdict(cwd) {
        v @ (Verdict::Newer(_) | Verdict::NewerSchema(_)) => Err(format!("{} (details: `elf check version`)", verdict_line(&v, embed::version()).1)),
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
    fn verdicts_compare_lines_only() {
        assert_eq!(verdict_with(project(&v2("v2.28.0"), None).path(), "v2.28.3"), Verdict::Same("v2.28.0".into()));
        assert_eq!(verdict_with(project(&v2("v2.27.4"), None).path(), "v2.28.0"), Verdict::Older("v2.27.4".into()));
        assert_eq!(verdict_with(project(&v2("v3.1.0"), None).path(), "v3.0.9"), Verdict::Newer("v3.1.0".into()));
        // 2.28.x = 3.0 줄(전환 시험판, S044 t52)
        assert_eq!(verdict_with(project(&v2("v2.28.1"), None).path(), "v3.0.0"), Verdict::Same("v2.28.1".into()));
        assert_eq!(verdict_with(project(&v2("v2.28.1"), None).path(), "v3.0.0-dev"), Verdict::Same("v2.28.1".into()));
        assert_eq!(verdict_with(project(&v2("v2.28.1"), None).path(), "v3.1.0"), Verdict::Older("v2.28.1".into()));
        assert_eq!(verdict_with(project(&v2("v4.0.0"), None).path(), "v3.0.0"), Verdict::Newer("v4.0.0".into()));
        let (_, same) = verdict_line(&Verdict::Same("v2.28.1".into()), "v3.0.0");
        assert!(same.contains("same line 3.0 (2.28.x is the 3.0 line)"), "{same}");
        // 더 높은 manifest schema(R7) = 판정과 쓰기 거부
        let s3 = project(r#"{ "schema": "elf-manifest/3", "elf_version": "v4.0.0", "files": [] }"#, None);
        assert_eq!(verdict_with(s3.path(), "v3.0.0"), Verdict::NewerSchema("elf-manifest/3".into()));
        assert!(gate_write(s3.path()).unwrap_err().contains("schema elf-manifest/3 is newer"));
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

    #[test]
    fn update_hint_only_for_an_older_line() {
        let me = embed::version();
        assert_eq!(update_hint(project(&v2(me), None).path()), None);
        assert_eq!(update_hint(project(&v2("v2.28.1"), None).path()), None, "2.28.x is the 3.0 line");
        assert_eq!(update_hint(project(&v2("v99.0.0"), None).path()), None);
        let h = update_hint(project(r#"{ "schema": "elf-manifest/1", "files": [] }"#, None).path()).unwrap();
        assert!(h.contains("line 2.27") && h.contains("with a log layout migration") && h.contains("Ask the user before running `elf update`"), "{h}");
    }
}
