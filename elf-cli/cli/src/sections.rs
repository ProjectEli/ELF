//! 로그 절 이름과 표기 언어(P017, S050 — issue #9).
//!
//! - 절은 한국어 키(`조건` 등)를 내부 식별자로 씀. 표기는 프로젝트 언어: 영어(`en`) 프로젝트 = `### Conditions`,
//!   그 밖(한국어, 영어본이 없는 언어) = `### 조건 (Conditions)`.
//! - 읽을 때는 두 표기를 언어와 무관하게 같은 절로 인식(기존 로그, 언어를 바꾼 프로젝트도 유효). 새로 쓰는 절만
//!   프로젝트 언어 표기로 바로잡음(`elf trial new`, `elf trial write`).

use std::fs;
use std::path::Path;

/// 절(한국어 키, 영어 이름). 순서 = 정본 순서(시행착오는 위치 자유).
pub const SECTIONS: [(&str, &str); 10] = [
    ("배경", "Background"),
    ("목표", "Goal"),
    ("조건", "Conditions"),
    ("가설", "Hypothesis"),
    ("예상", "Prediction"),
    ("관찰", "Observation"),
    ("해석", "Interpretation"),
    ("교훈", "Lessons"),
    ("생성 파일", "Files"),
    ("시행착오", "Trial-and-Error"),
];

/// 해석 첫 줄 라벨: 한국어(v2.24~), 영어(v3.4~), 이전 한국어(≤ v2.23, 기존 로그 유효).
pub const INTERP_KO: &str = "예상 일치 정도";
pub const INTERP_EN: &str = "Prediction match";
pub const INTERP_LEGACY: &str = "가설 적중 여부";

/// 로그 표기 언어.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLang {
    Ko,
    En,
}

impl LogLang {
    /// `.elf/config.json`의 `lang` 주 언어가 `en`이면 영어, 그 밖(한국어, 영어본이 없는 언어, 없거나 읽지 못함)은
    /// 한국어(이전 동작). 영어 틀은 `en` variant로만 배포되므로(manifest) 배포 파일과 같은 기준.
    pub fn of(root: &Path) -> LogLang {
        let en = fs::read_to_string(root.join(".elf/config.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| v.get("lang").and_then(|l| l.as_str()).map(|l| crate::manifest::lang_primary(l) == "en"))
            .unwrap_or(false);
        if en { LogLang::En } else { LogLang::Ko }
    }

    /// 해석 첫 줄 라벨.
    pub fn interp_label(self) -> &'static str {
        match self {
            LogLang::Ko => INTERP_KO,
            LogLang::En => INTERP_EN,
        }
    }

    /// 해석 첫 줄 값의 선택지(안내 문구용).
    pub fn interp_values(self) -> &'static str {
        match self {
            LogLang::Ko => "일치/부분 일치/불일치",
            LogLang::En => "match/partial/mismatch",
        }
    }

    /// trial 제목 placeholder(`elf trial new`를 제목 없이 실행하면 남음).
    pub fn title_placeholder(self) -> &'static str {
        match self {
            LogLang::Ko => "[작업 제목]",
            LogLang::En => "[task title]",
        }
    }
}

/// 절 이름(한국어 키 또는 영어 이름, 영어는 대소문자 무관) → 한국어 키.
pub fn key_of(name: &str) -> Option<&'static str> {
    let n = name.trim();
    SECTIONS.iter().find(|(ko, en)| n == *ko || n.eq_ignore_ascii_case(en)).map(|(ko, _)| *ko)
}

/// 표기 이름: 한국어 = 키, 영어 = 영어 이름. 모르는 키는 그대로.
pub fn name(key: &str, lang: LogLang) -> &str {
    match lang {
        LogLang::Ko => key,
        LogLang::En => SECTIONS.iter().find(|(ko, _)| *ko == key).map_or(key, |(_, en)| *en),
    }
}

/// 정본 헤딩 줄: 한국어 = `### 조건 (Conditions)`, 영어 = `### Conditions`.
pub fn heading(key: &str, lang: LogLang) -> String {
    match (lang, SECTIONS.iter().find(|(ko, _)| *ko == key)) {
        (LogLang::Ko, Some((ko, en))) => format!("### {ko} ({en})"),
        (LogLang::En, Some((_, en))) => format!("### {en}"),
        (_, None) => format!("### {key}"),
    }
}

/// 키 목록의 표기 이름을 `/`로 이음(안내 문구용).
pub fn names(keys: &[&str], lang: LogLang) -> String {
    keys.iter().map(|k| name(k, lang)).collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_name_heading() {
        assert_eq!(key_of("조건"), Some("조건"));
        assert_eq!(key_of("conditions"), Some("조건"));
        assert_eq!(key_of("Files"), Some("생성 파일"));
        assert_eq!(key_of("Trial-and-Error"), Some("시행착오"));
        assert_eq!(key_of("메모"), None);
        assert_eq!(name("생성 파일", LogLang::En), "Files");
        assert_eq!(name("생성 파일", LogLang::Ko), "생성 파일");
        assert_eq!(heading("관찰", LogLang::Ko), "### 관찰 (Observation)");
        assert_eq!(heading("관찰", LogLang::En), "### Observation");
        assert_eq!(names(&["목표", "조건"], LogLang::En), "Goal/Conditions");
    }

    #[test]
    fn lang_from_config() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(LogLang::of(tmp.path()), LogLang::Ko, "no config = Korean (earlier behavior)");
        fs::create_dir_all(tmp.path().join(".elf")).unwrap();
        fs::write(tmp.path().join(".elf/config.json"), r#"{"lang": "en-US"}"#).unwrap();
        assert_eq!(LogLang::of(tmp.path()), LogLang::En);
        fs::write(tmp.path().join(".elf/config.json"), r#"{"lang": "ko"}"#).unwrap();
        assert_eq!(LogLang::of(tmp.path()), LogLang::Ko);
        fs::write(tmp.path().join(".elf/config.json"), r#"{"lang": "ja-JP"}"#).unwrap();
        assert_eq!(LogLang::of(tmp.path()), LogLang::Ko, "no English variant is deployed for ja → Korean templates");
    }
}
