//! 저장소별 버전 고정(v3.0, S044 t51 R1, R4, R6, R7) — 고정값(pin) 읽기와 해석, 구조 세대.
//!
//! - 고정값 = 프로젝트 manifest `.elf/manifest.json`의 `elf_version`(init과 update만 씀, R1). 실행 버전 = 고정값의
//!   MAJOR.MINOR 줄(line)의 최신 patch.
//! - 2.28.x = 로그 구조 2세대를 MINOR로 낸 전환 시험판 → 3.0 줄로 해석(2.28 실행 파일이 필요 없게, S044 t52).
//! - 고정값 없는 이전 형식(manifest schema 1, `.elf/` 없이 2.27 형식 로그만 있는 폴더) = 2.27 줄로 추론(R6).
//!   1세대 프로젝트는 MINOR 사이 이관이 없으므로 1세대 마지막 줄(2.27)이 모두 다룸.
//! - 위임 규약(고정값 위치, 보관소 경로, Release 주소와 자산 이름)은 고정(R7). 바꿀 때는 manifest schema를 올리고,
//!   모르는 schema를 만난 CLI는 `elf self update`를 안내.

use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::path::Path;

/// MAJOR.MINOR 줄. `line_of`로만 만듦(2.28 → 3.0 해석을 거치게).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line {
    major: u32,
    minor: u32,
}

impl Line {
    pub fn major(self) -> u32 {
        self.major
    }

    pub fn minor(self) -> u32 {
        self.minor
    }

    /// 구조 세대(R4). 표에 없는 아주 옛 줄(2.0 이전) = 0.
    pub fn generation(self) -> u32 {
        GENERATIONS.iter().rev().find(|g| self >= g.first).map_or(0, |g| g.number)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// 2.28 → 3.0(같은 구조 세대의 정식 MAJOR), 그 밖은 그대로.
pub fn line_of(major: u32, minor: u32) -> Line {
    if (major, minor) == (2, 28) { Line { major: 3, minor: 0 } } else { Line { major, minor } }
}

/// 1세대 마지막 줄 — 고정값 없는 이전 형식의 추론 값(R6).
pub fn earlier_line() -> Line {
    line_of(2, 27)
}

/// 구조 세대 하나: 번호, 첫 줄, 설명.
pub struct Generation {
    pub number: u32,
    pub first: Line,
    pub layout: &'static str,
}

/// 구조 세대 표(R4) — 세대가 바뀌는 MAJOR의 `elf update`가 직전 세대에서 이관. 여러 세대를 건너뛸 때는 세대마다
/// 그 세대 첫 줄의 최신 patch로 차례로 `elf update`.
pub const GENERATIONS: [Generation; 2] = [
    Generation { number: 1, first: Line { major: 2, minor: 0 }, layout: "0~6 layout, quote-block log headers, closed logs in 2_Log/Archive/" },
    Generation { number: 2, first: Line { major: 3, minor: 0 }, layout: "log layout 2: 2_Log/S/, 2_Log/G/, YAML headers (2.28.x was the trial)" },
];

/// 버전 표기(`v3.0.1`, `2.28.1`, `v3.0.0-dev`, `v2.4`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// pre-release 식별자(`dev` 등)
    pub pre: Option<String>,
}

impl Ver {
    pub fn line(&self) -> Line {
        line_of(self.major, self.minor)
    }
}

impl fmt::Display for Ver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(p) = &self.pre {
            write!(f, "-{p}")?;
        }
        Ok(())
    }
}

impl PartialOrd for Ver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// semver 순서: 같은 숫자면 pre-release가 낮음.
impl Ord for Ver {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)).then_with(|| match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        })
    }
}

pub fn parse(v: &str) -> Option<Ver> {
    let v = v.trim().trim_start_matches('v');
    let (core, pre) = match v.split_once('-') {
        Some((c, p)) if !p.is_empty() => (c, Some(p.to_string())),
        Some(_) => return None,
        None => (v, None),
    };
    let mut it = core.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = match it.next() {
        Some(p) => p.parse().ok()?,
        None => 0,
    };
    if it.next().is_some() {
        return None;
    }
    Some(Ver { major, minor, patch, pre })
}

/// 고정값의 출처.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// manifest `elf_version` 원문
    Manifest(String),
    /// 이전 형식에서 추론(R6) — 이유
    Inferred(&'static str),
}

/// 폴더의 고정값과 해석한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub line: Line,
    pub source: Source,
}

/// manifest schema 번호(`elf-manifest/N`).
fn schema_number(s: &str) -> Option<u32> {
    s.strip_prefix("elf-manifest/")?.parse().ok()
}

/// 이 CLI보다 높은 manifest schema면 그 이름(R7 — 위임 규약이 바뀐 버전이 쓴 manifest).
pub fn newer_schema(manifest_text: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(manifest_text).ok()?;
    let s = v.get("schema")?.as_str()?;
    let cur = schema_number(crate::manifest::SCHEMA)?;
    (schema_number(s)? > cur).then(|| s.to_string())
}

/// 폴더(프로젝트 루트 또는 `.elf/` 없이 로그만 있는 폴더)의 고정값.
/// Ok(None) = 고정값 없음(ELF 폴더 아님, 또는 `.elf/` 없는 새 구조 로그 폴더 — 실행 중인 CLI로 동작).
/// Err = manifest를 읽을 수 없거나 버전 표기를 해석할 수 없음, 또는 더 높은 schema.
pub fn read(root: &Path) -> Result<Option<Pin>, String> {
    match fs::read_to_string(root.join(".elf").join("manifest.json")) {
        Ok(text) => {
            if let Some(s) = newer_schema(&text) {
                return Err(format!("manifest schema {s} is newer than this CLI — run `elf self update`"));
            }
            let m = crate::manifest::parse(&text)?;
            if m.schema == crate::manifest::SCHEMA_V1 {
                return Ok(Some(Pin { line: earlier_line(), source: Source::Inferred("earlier format, manifest schema 1") }));
            }
            let raw = m.elf_version.ok_or_else(|| "elf_version missing".to_string())?;
            let v = parse(&raw).ok_or_else(|| format!("elf_version \"{raw}\" is not a version"))?;
            Ok(Some(Pin { line: v.line(), source: Source::Manifest(raw) }))
        }
        Err(_) if crate::earlier::footprint(root) => {
            Ok(Some(Pin { line: earlier_line(), source: Source::Inferred("ELF 2.0–2.4.0 layout, no .elf/") }))
        }
        Err(_) if crate::legacy::active(root) => {
            Ok(Some(Pin { line: earlier_line(), source: Source::Inferred("earlier log layout, no .elf/") }))
        }
        Err(_) => Ok(None),
    }
}

/// 고정값 한 줄 설명(`elf check version`).
pub fn describe(p: &Pin) -> String {
    let g = p.line.generation();
    match &p.source {
        Source::Manifest(raw) => {
            let note = if parse(raw).is_some_and(|v| (v.major, v.minor) == (2, 28)) { "; 2.28.x is the 3.0 line" } else { "" };
            format!("pin: {raw} → line {} (structure generation {g}{note}) — changed only by `elf update`", p.line)
        }
        Source::Inferred(why) => format!(
            "pin: none recorded ({why}) → line {} inferred (structure generation {g}) — `elf update` migrates to generation {}",
            p.line,
            GENERATIONS[GENERATIONS.len() - 1].number
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_order() {
        assert_eq!(parse("v3.0.1"), Some(Ver { major: 3, minor: 0, patch: 1, pre: None }));
        assert_eq!(parse("2.28.1").unwrap().to_string(), "v2.28.1");
        assert_eq!(parse("v2.4-dev").unwrap(), Ver { major: 2, minor: 4, patch: 0, pre: Some("dev".into()) });
        for bad in ["", "v", "x", "3", "3.x", "1.2.3.4", "3.0.0-"] {
            assert_eq!(parse(bad), None, "{bad}");
        }
        assert!(parse("v3.0.0-dev").unwrap() < parse("v3.0.0").unwrap());
        assert!(parse("v3.0.0").unwrap() < parse("v3.0.1").unwrap());
        assert!(parse("v2.28.1").unwrap() < parse("v3.0.0").unwrap());
    }

    #[test]
    fn lines_and_generations() {
        assert_eq!(parse("v2.28.1").unwrap().line(), line_of(3, 0), "2.28.x = 3.0 line");
        assert_eq!(line_of(3, 0).to_string(), "3.0");
        assert_eq!(earlier_line().to_string(), "2.27");
        assert_eq!(earlier_line().generation(), 1);
        assert_eq!(line_of(2, 4).generation(), 1);
        assert_eq!(line_of(2, 28).generation(), 2);
        assert_eq!(line_of(3, 1).generation(), 2);
        assert_eq!(line_of(1, 9).generation(), 0);
        assert!(earlier_line() < line_of(2, 28));
    }

    fn folder(manifest: Option<&str>, logs: &[(&str, &str)]) -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        if let Some(m) = manifest {
            fs::create_dir_all(t.path().join(".elf")).unwrap();
            fs::write(t.path().join(".elf/manifest.json"), m).unwrap();
        }
        for (rel, body) in logs {
            let p = t.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, body).unwrap();
        }
        t
    }

    #[test]
    fn read_pins_from_manifest_or_layout() {
        let m = |v: &str| format!(r#"{{ "schema": "elf-manifest/2", "elf_version": "{v}", "files": [] }}"#);
        let p = read(folder(Some(&m("v2.28.1")), &[]).path()).unwrap().unwrap();
        assert_eq!((p.line, &p.source), (line_of(3, 0), &Source::Manifest("v2.28.1".into())));
        assert!(describe(&p).contains("line 3.0 (structure generation 2; 2.28.x is the 3.0 line)"), "{}", describe(&p));
        let p = read(folder(Some(r#"{ "schema": "elf-manifest/1", "files": [] }"#), &[]).path()).unwrap().unwrap();
        assert_eq!(p.line, earlier_line());
        assert!(describe(&p).contains("line 2.27 inferred (structure generation 1) — `elf update` migrates to generation 2"), "{}", describe(&p));
        // .elf/ 없는 폴더: 2.27 형식 로그 = 추론, 새 구조 로그 = 고정값 없음
        let old = folder(None, &[("2_Log/S001_log.md", "# S001: a\n\n> **Status**: ★ 활성\n")]);
        assert_eq!(read(old.path()).unwrap().unwrap().source, Source::Inferred("earlier log layout, no .elf/"));
        assert_eq!(read(folder(None, &[("2_Log/S/S001_log.md", "---\n")]).path()).unwrap(), None);
        assert_eq!(read(folder(None, &[]).path()).unwrap(), None);
        // 오류: 버전 아님, 빠짐, 더 높은 schema
        assert!(read(folder(Some(&m("dev")), &[]).path()).unwrap_err().contains("not a version"));
        assert!(read(folder(Some(r#"{ "schema": "elf-manifest/2", "files": [] }"#), &[]).path()).unwrap_err().contains("missing"));
        let e = read(folder(Some(r#"{ "schema": "elf-manifest/3", "elf_version": "v4.0.0", "files": [] }"#), &[]).path()).unwrap_err();
        assert!(e.contains("elf-manifest/3") && e.contains("elf self update"), "{e}");
    }
}
