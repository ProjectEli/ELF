//! `elf init` — 프로젝트 스캐폴드 실행 (t03).
//! 순수 planner(plan_dirs/plan_init)를 소비해 FS에 반영하는 executor.
//!
//! 불변식 (S007 t02 재점검 §1): **managed/hybrid는 placeholder 치환 금지** — embed 정본
//! 바이트 그대로 배치해야 stamp 해시 비교(update 편집감지)가 성립한다. 치환은 seed에만 허용.
//!
//! 모드 (P018): `run_init`(subfolder, 기존) = `parent/<name>` 신규. `run_init_ex`(in_place) =
//! `parent`(cwd) 제자리 비파괴 — 기존 파일 미덮어쓰기(누락분만; managed/hybrid 충돌은 `<dest>.elf-new`).

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
    /// in-place 대상이 이미 ELF 프로젝트(.elf/manifest.json 존재) — refuse → `elf update` (exit 3)
    AlreadyElf(PathBuf),
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
            InitError::AlreadyElf(p) => write!(
                f,
                "refuse: {} is already an ELF project — use `elf update`",
                p.display()
            ),
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

/// init 실행 결과 — in-place 비파괴 배치의 집계(created/skipped/.elf-new) + 대상 경로.
#[derive(Debug, Default)]
pub struct InitReport {
    pub target: PathBuf,
    pub created: Vec<String>,
    /// 기존 파일이 있어 건너뛴 dest (사용자 것 유지)
    pub skipped: Vec<String>,
    /// managed/hybrid 충돌 → `<dest>.elf-new`로 병기한 dest
    pub elf_new: Vec<String>,
}

/// `parent/<name>`(subfolder) 에 ELF 프로젝트를 생성하고 경로를 반환 (기존 호출 호환).
pub fn run_init(parent: &Path, opts: &InitOptions) -> Result<PathBuf, InitError> {
    run_init_ex(parent, opts, false, false, false).map(|r| r.target)
}

/// ELF 프로젝트 scaffold.
/// - `in_place=false`: `parent/<name>` 신규 생성(기존 존재 시 거부) — 기존 동작.
/// - `in_place=true` : `parent`(=cwd) 제자리. `.elf/manifest.json` 존재 시 거부(→`update`).
///   기존 파일은 **절대 덮어쓰지 않음**(누락분만 추가; managed/hybrid 충돌은 `<dest>.elf-new`).
/// - `dry_run`: FS 미기록(plan만 산출). `force`: 기존 파일도 덮어씀.
pub fn run_init_ex(
    parent: &Path,
    opts: &InitOptions,
    in_place: bool,
    dry_run: bool,
    force: bool,
) -> Result<InitReport, InitError> {
    let target = if in_place {
        parent.to_path_buf()
    } else {
        parent.join(&opts.name)
    };

    if in_place {
        if target.join(".elf").join("manifest.json").is_file() {
            return Err(InitError::AlreadyElf(target));
        }
    } else if target.exists() {
        return Err(InitError::TargetExists(target));
    }

    // preset에 따라 계보 manifest 선택 — qa=질문 아카이브, general=목표지향 비연구, 그 외=연구
    let kind = manifest::kind_from_preset(&opts.preset);
    let base = manifest::embedded_kind(kind);
    // 프로젝트 언어로 배포 entry 해석 (P016 §9)
    let m = base.for_lang(&opts.lang);

    let mut report = InitReport {
        target: target.clone(),
        ..Default::default()
    };

    // 1. 폴더 scaffold — keepfile은 **빈 dir에만**(in-place 기존 콘텐츠 보호; fresh dir는 항상 빈)
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
        if !dry_run {
            fs::create_dir_all(&p)?;
        }
        // fresh init(=!in_place)은 무조건(기존 동작 보존); in-place는 실제 빈 dir만
        let writable = !in_place
            || fs::read_dir(&p)
                .map(|mut r| r.next().is_none())
                .unwrap_or(true);
        if writable {
            match d.keep {
                KeepFile::GitKeep => {
                    if !dry_run {
                        fs::write(p.join(".gitkeep"), b"")?;
                    }
                }
                KeepFile::RawGitignore => {
                    let gi = p.join(".gitignore");
                    if !gi.exists() && !dry_run {
                        fs::write(&gi, b"*\n!.gitignore\n")?;
                    }
                }
            }
        }
    }

    // (qa) 사용자 지정 카테고리 폴더 — `<cat>/` + `<cat>/archive/` (각 .gitkeep). 경로 탈출 방지.
    if opts.preset == "qa" {
        for cat in &opts.categories {
            if cat.is_empty() || cat.contains('/') || cat.contains('\\') || cat.contains("..") {
                return Err(InitError::Plan(format!(
                    "invalid category name (no empty / slash / `..`): {cat:?}"
                )));
            }
            if !dry_run {
                let base = target.join(cat);
                fs::create_dir_all(base.join("archive"))?;
                fs::write(base.join(".gitkeep"), b"")?;
                fs::write(base.join("archive").join(".gitkeep"), b"")?;
            }
        }
    }

    // 2. 빈 루트 파일 (generator parity) — 부재 시만(기존 LICENSE 등 보호). 연구 preset만.
    if opts.preset != "qa" {
        place_empty(&target, ".gitattributes", force, dry_run, &mut report)?;
        place_empty(&target, "LICENSE", force, dry_run, &mut report)?;
    }

    // 3. manifest 파일 배치 — 하이브리드 충돌(seed skip / managed·hybrid `.elf-new`)
    for a in plan::plan_init(&m) {
        let rel = a
            .src
            .strip_prefix("templates/")
            .ok_or_else(|| InitError::Internal(format!("invalid template path: {}", a.src)))?;
        let file = embed::TEMPLATES
            .get_file(rel)
            .ok_or_else(|| InitError::Internal(format!("embedded template missing: {}", a.src)))?;
        let dest = target.join(&a.dest);

        if dest.exists() && !force {
            match a.tier {
                // seed·pointer: 기존 파일 유지(불변경). pointer는 `.elf-new` 병기도 없음 (t06)
                Tier::Seed | Tier::Pointer => report.skipped.push(a.dest.clone()),
                Tier::Managed | Tier::Hybrid => {
                    if !dry_run {
                        let newp = elf_new_path(&dest);
                        if let Some(dir) = newp.parent() {
                            fs::create_dir_all(dir)?;
                        }
                        fs::write(&newp, file.contents())?;
                    }
                    report.elf_new.push(format!("{}.elf-new", a.dest));
                }
            }
            continue;
        }

        if !dry_run {
            if let Some(dir) = dest.parent() {
                fs::create_dir_all(dir)?;
            }
            match a.tier {
                // pointer도 정본 바이트 그대로 (placeholder 없음 — sha 비교 성립 유지)
                Tier::Managed | Tier::Hybrid | Tier::Pointer => fs::write(&dest, file.contents())?,
                Tier::Seed => {
                    let text = file.contents_utf8().expect("seed templates must be UTF-8");
                    fs::write(&dest, substitute_seed(&a.dest, text, opts))?;
                }
            }
        }
        report.created.push(a.dest.clone());
    }

    // 4. 파생 instance: 2_Log/S001_log.md — 연구 preset 전용, 부재 시만
    if opts.preset != "qa" {
        let rel = "2_Log/S001_log.md";
        let dest = target.join(rel);
        if dest.exists() && !force {
            report.skipped.push(rel.into());
        } else {
            if !dry_run {
                let session_tpl = embed::TEMPLATES
                    .get_file("log/sessionTemplate.md")
                    .and_then(|f| f.contents_utf8())
                    .ok_or_else(|| {
                        InitError::Internal("embedded template missing: sessionTemplate.md".into())
                    })?;
                let s001 = session_tpl
                    .replace("S{NNN}", "S001")
                    .replace("YYYY-MM-DD", &opts.date);
                fs::write(&dest, s001)?;
            }
            report.created.push(rel.into());
        }
    }

    // 5. .elf/ control plane: config + version stamp + manifest stamp + hybrid baseline (marker — 항상)
    if !dry_run {
        let elf_dir = target.join(".elf");
        fs::create_dir_all(&elf_dir)?;
        // preset = 계보 정체성 영속화 — update/status가 이 값으로 정본 세트를 선택 (S026).
        let config = serde_json::json!({
            "name": opts.name,
            "lang": opts.lang,
            "preset": opts.preset,
            "created": opts.date,
        });
        let mut config_text = serde_json::to_string_pretty(&config).expect("config serializes");
        config_text.push('\n');
        fs::write(elf_dir.join("config.json"), config_text)?;
        fs::write(elf_dir.join("version"), format!("{}\n", embed::version()))?;
        fs::write(elf_dir.join("manifest.json"), manifest::kind_json(kind))?;

        // hybrid 배포본 baseline (블록 내 편집 감지의 비교 기준 — update가 사용)
        for e in &m.files {
            if e.tier == Tier::Hybrid {
                let rel = e.src.strip_prefix("templates/").ok_or_else(|| {
                    InitError::Internal(format!("invalid template path: {}", e.src))
                })?;
                let file = embed::TEMPLATES.get_file(rel).ok_or_else(|| {
                    InitError::Internal(format!("embedded template missing: {}", e.src))
                })?;
                let baseline = elf_dir.join("baseline").join(&e.dest);
                if let Some(dir) = baseline.parent() {
                    fs::create_dir_all(dir)?;
                }
                fs::write(baseline, file.contents())?;
            }
        }

        // autoread 훅 (기본 켬 — S031 t06): 신규 scaffold는 config 키 미기재(부재=on)로 켜지고,
        // 훅은 여기서 최초 배치. in-place에서 기존 settings.json이 malformed면 skip 기록
        // (비파괴 — init을 막지 않고 `elf update`/`elf autoread enable`이 재시도 경로).
        match crate::autoread::install_hooks(&target) {
            Ok(true) => report.created.push(".claude/settings.json (autoread hooks)".into()),
            Ok(false) => {}
            Err(crate::autoread::AutoreadError::Refuse(e)) => {
                report.skipped.push(format!(".claude/settings.json — {e}"));
            }
            Err(crate::autoread::AutoreadError::Io(e)) => return Err(InitError::Io(e)),
        }
    }

    Ok(report)
}

/// 빈 파일을 부재 시만 배치(force면 덮어씀). 존재 시 skip 기록(사용자 것 유지).
fn place_empty(
    target: &Path,
    rel: &str,
    force: bool,
    dry_run: bool,
    report: &mut InitReport,
) -> Result<(), InitError> {
    let dest = target.join(rel);
    if dest.exists() && !force {
        report.skipped.push(rel.into());
    } else {
        if !dry_run {
            fs::write(&dest, b"")?;
        }
        report.created.push(rel.into());
    }
    Ok(())
}

/// `<dest>` → `<dest>.elf-new` (sibling, 무손실 병기).
fn elf_new_path(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(".elf-new");
    PathBuf::from(s)
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
