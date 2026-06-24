//! `elf init` — 프로젝트 스캐폴드 실행 (t03).
//! 순수 planner(plan_dirs/plan_init)를 소비해 FS에 반영하는 executor.
//!
//! 불변식 (S007 t02 재점검 §1): **managed/hybrid는 placeholder 치환 금지** — embed 정본
//! 바이트 그대로 배치해야 stamp 해시 비교(update 편집감지)가 성립한다. 치환은 seed에만 허용.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::embed;
use crate::manifest::{self, Tier};
use crate::plan::{self, KeepFile};

pub struct InitOptions {
    pub name: String,
    /// preset 이름 (full/experimental/software/minimal). modules 지정 시 무시.
    pub preset: String,
    /// custom 모듈 선택 (Some이면 preset 대신 사용)
    pub modules: Option<Vec<String>>,
    /// (qa preset 전용) 사전 생성할 카테고리 폴더 — 비면 0개(수요 기반 생성). 그 외 preset은 무시.
    pub categories: Vec<String>,
    pub lang: String,
    /// YYYY-MM-DD — 주입형(테스트 결정성; 프로덕션은 main이 오늘 날짜 주입)
    pub date: String,
}

#[derive(Debug)]
pub enum InitError {
    /// 대상 폴더가 이미 존재 — refuse (exit 3)
    TargetExists(PathBuf),
    /// 미지 preset/module 등 계획 오류
    Plan(String),
    /// 내장 데이터 불일치 등 — 손상된 설치 의심 (panic 대신 안내, t09 정책)
    Internal(String),
    Io(io::Error),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::TargetExists(p) => write!(f, "refuse: {} already exists", p.display()),
            InitError::Plan(s) => write!(f, "plan: {s}"),
            InitError::Internal(s) => write!(
                f,
                "internal: {s} — corrupted install? run `elf self-update` or reinstall"
            ),
            InitError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl From<io::Error> for InitError {
    fn from(e: io::Error) -> Self {
        InitError::Io(e)
    }
}

/// `parent/<name>` 에 ELF 프로젝트를 생성하고 생성된 경로를 반환.
pub fn run_init(parent: &Path, opts: &InitOptions) -> Result<PathBuf, InitError> {
    let target = parent.join(&opts.name);
    if target.exists() {
        return Err(InitError::TargetExists(target));
    }

    // preset에 따라 유형 manifest 선택 — qa=질문 아카이브, general=목표지향 비연구, 그 외=연구
    let base = if opts.preset == "qa" {
        manifest::embedded_qa()
    } else if opts.preset == "general" {
        manifest::embedded_general()
    } else {
        manifest::embedded()
    };
    // 프로젝트 언어로 배포 entry 해석 — ko/미지정=base만(현행), en=companion·variant 포함 (P016 §9)
    let m = base.for_lang(&opts.lang);

    // 1. 폴더 scaffold (+ .gitkeep / raw .gitignore) — 데이터 출처 = manifest.dirs
    let dir_plan = match &opts.modules {
        Some(keys) => {
            let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
            plan::plan_dirs_from_modules(&m, &refs)
        }
        None => plan::plan_dirs(&m, &opts.preset),
    }
    .map_err(InitError::Plan)?;

    for d in &dir_plan {
        let p = target.join(&d.path);
        fs::create_dir_all(&p)?;
        match d.keep {
            KeepFile::GitKeep => fs::write(p.join(".gitkeep"), b"")?,
            KeepFile::RawGitignore => fs::write(p.join(".gitignore"), b"*\n!.gitignore\n")?,
        }
    }

    // (qa) 사용자 지정 카테고리 폴더 — `<cat>/` + `<cat>/archive/` (각 .gitkeep).
    //      비면 0개 = 수요 기반 생성(CLAUDE.md 규약). 경로 탈출 방지 검증.
    if opts.preset == "qa" {
        for cat in &opts.categories {
            if cat.is_empty() || cat.contains('/') || cat.contains('\\') || cat.contains("..") {
                return Err(InitError::Plan(format!(
                    "invalid category name (no empty / slash / `..`): {cat:?}"
                )));
            }
            let base = target.join(cat);
            fs::create_dir_all(base.join("archive"))?;
            fs::write(base.join(".gitkeep"), b"")?;
            fs::write(base.join("archive").join(".gitkeep"), b"")?;
        }
    }

    // 2. 빈 루트 파일 (generator parity) — 연구 preset만. qa는 불필요(cruft) 생략.
    if opts.preset != "qa" {
        fs::write(target.join(".gitattributes"), b"")?;
        fs::write(target.join("LICENSE"), b"")?;
    }

    // 3. manifest 파일 배치 — managed/hybrid = 정본 그대로, seed = placeholder 치환
    for a in plan::plan_init(&m) {
        let rel = a
            .src
            .strip_prefix("templates/")
            .ok_or_else(|| InitError::Internal(format!("invalid template path: {}", a.src)))?;
        let file = embed::TEMPLATES
            .get_file(rel)
            .ok_or_else(|| InitError::Internal(format!("embedded template missing: {}", a.src)))?;
        let dest = target.join(&a.dest);
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        match a.tier {
            Tier::Managed | Tier::Hybrid => fs::write(&dest, file.contents())?,
            Tier::Seed => {
                let text = file.contents_utf8().expect("seed templates must be UTF-8");
                fs::write(&dest, substitute_seed(&a.dest, text, opts))?;
            }
        }
    }

    // 4. 파생 instance: 2_Log/S001_log.md — 연구 preset 전용 (qa는 세션/trial 미사용 유형)
    if opts.preset != "qa" {
        let session_tpl = embed::TEMPLATES
            .get_file("log/sessionTemplate.md")
            .and_then(|f| f.contents_utf8())
            .ok_or_else(|| {
                InitError::Internal("embedded template missing: sessionTemplate.md".into())
            })?;
        let s001 = session_tpl
            .replace("S{NNN}", "S001")
            .replace("YYYY-MM-DD", &opts.date);
        fs::write(target.join("2_Log/S001_log.md"), s001)?;
    }

    // 5. .elf/ control plane: config + version stamp + manifest stamp(배포 시점 사본)
    let elf_dir = target.join(".elf");
    fs::create_dir_all(&elf_dir)?;
    let config = serde_json::json!({
        "name": opts.name,
        "lang": opts.lang,
        "created": opts.date,
    });
    let mut config_text = serde_json::to_string_pretty(&config).expect("config serializes");
    config_text.push('\n');
    fs::write(elf_dir.join("config.json"), config_text)?;
    fs::write(elf_dir.join("version"), format!("{}\n", embed::version()))?;
    let manifest_json = if opts.preset == "qa" {
        embed::MANIFEST_QA_JSON
    } else if opts.preset == "general" {
        embed::MANIFEST_GENERAL_JSON
    } else {
        embed::MANIFEST_JSON
    };
    fs::write(elf_dir.join("manifest.json"), manifest_json)?;

    // hybrid 배포본 baseline (블록 내 편집 감지의 비교 기준 — t04 update가 사용)
    for e in &m.files {
        if e.tier == Tier::Hybrid {
            let rel = e
                .src
                .strip_prefix("templates/")
                .ok_or_else(|| InitError::Internal(format!("invalid template path: {}", e.src)))?;
            let file = embed::TEMPLATES
                .get_file(rel)
                .ok_or_else(|| InitError::Internal(format!("embedded template missing: {}", e.src)))?;
            let baseline = elf_dir.join("baseline").join(&e.dest);
            if let Some(dir) = baseline.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(baseline, file.contents())?;
        }
    }

    Ok(target)
}

/// seed 전용 placeholder 치환 (dest 기준 — 토큰이 파일별 역사적 상이).
/// 토큰 통일·데이터화는 manifest schema 확장 시 재검토 (S007 §5).
fn substitute_seed(dest: &str, text: &str, o: &InitOptions) -> String {
    match dest {
        "0_Meta/ProjectRule.md" => text
            .replace("[프로젝트명]", &o.name)
            .replace("[Project Name]", &o.name)
            .replace("YYYY-MM-DD", &o.date),
        "README.md" => text
            .replace("PLACEHOLDER_PROJECT_NAME", &o.name)
            .replace("PLACEHOLDER_DATE", &o.date),
        "2_Log/Wiki/Session_Registry.tsv" => text.replace("YYYY-MM-DD", &o.date),
        _ => text.to_string(), // 그 외 seed: 치환 없음
    }
}
