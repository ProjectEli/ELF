use clap::{Parser, Subcommand};
use elf_cli::{
    doctor, embed, gallery, init, manifest, selfupdate, session, status, trial, update, validate,
};

/// ELF (Eli's Lab Framework) 연구 프로젝트 스캐폴드·갱신 CLI (research project scaffold & update CLI)
#[derive(Parser)]
#[command(name = "elf", version = embed::version(), arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 새 ELF 프로젝트 스캐폴드 생성 · Scaffold an ELF project (생략 시 현재 폴더에 in-place)
    Init {
        /// 프로젝트 폴더 이름 (생략 시 현재 폴더 in-place) · project folder name (omit = in-place here)
        name: Option<String>,
        /// 현재 폴더에 in-place 강제 · force in-place in the current directory
        #[arg(long)]
        here: bool,
        /// 확인 프롬프트 생략 · skip the confirmation prompt
        #[arg(long)]
        yes: bool,
        /// 변경 없이 계획만 출력 · preview only, write nothing
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// 기존 파일도 덮어씀 · overwrite existing files
        #[arg(long)]
        force: bool,
        /// 모듈 preset · module preset (full | experimental | software | minimal | qa[exp] | general[exp])
        #[arg(long, default_value = "full", conflicts_with = "modules")]
        preset: String,
        /// custom 모듈 선택 · custom modules (쉼표 구분: hw,fab,sw,exp,paper) — preset 대신
        #[arg(long, value_delimiter = ',')]
        modules: Option<Vec<String>>,
        /// (qa preset) 사전 생성 카테고리 · pre-create categories (쉼표 구분) — 기본 0개(수요 기반)
        #[arg(long, value_delimiter = ',')]
        categories: Option<Vec<String>>,
        /// 프로젝트 언어 · project language (AI 응답 언어 — .elf/config.json)
        #[arg(long, default_value = "ko-KR")]
        lang: String,
    },
    /// 프로젝트의 ELF managed/hybrid 파일을 현 CLI 버전으로 갱신 · Update this project's ELF-managed files
    /// (CLI 자체 갱신은 `elf self-update` 또는 `elf update --self`)
    Update {
        /// 변경 없이 작업 목록만 출력 · dry-run (preview only)
        #[arg(long)]
        dry_run: bool,
        /// 사용자 편집 무시 강제 갱신 · force overwrite (managed 덮어쓰기·hybrid 블록 교체)
        #[arg(long)]
        force: bool,
        /// elf 바이너리 자체를 갱신 · update the elf binary (= self-update alias)
        #[arg(long = "self", conflicts_with_all = ["dry_run", "force"])]
        update_self: bool,
    },
    /// elf 바이너리 자체를 최신 릴리즈로 갱신 · Update the elf binary itself (프로젝트 파일은 `elf update`)
    #[command(name = "self-update", alias = "selfupdate")]
    SelfUpdate,
    /// 세션 수명주기 · Session lifecycle (new / close / fix-headers)
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },
    /// trial 스캐폴드 · Trial scaffold (활성 로그에 정본 stub 추가 / append the canonical stub)
    Trial {
        #[command(subcommand)]
        cmd: TrialCmd,
    },
    // (주의: doc comment(`///`)는 help로 노출됨 — 내부 표기 금지, 회귀 테스트가 게이트. P011 t09)
    /// 프로젝트 ELF 파일 상태 진단 · Diagnose managed-file drift (편집/누락 — 읽기전용/read-only)
    Status {
        /// 발견 시 exit 4 · exit 4 on findings — pre-commit/CI 게이트
        #[arg(long)]
        check: bool,
    },
    /// 세션/Registry/로그 정합 검사 · Validate session/registry/log consistency (읽기전용/read-only)
    Validate {
        /// issue 발견 시 exit 4 · exit 4 on issues — pre-commit/CI 게이트
        #[arg(long)]
        check: bool,
        /// figure-embed 누락을 issue로 승격 · promote missing figure-embeds to issues
        #[arg(long)]
        strict: bool,
    },
    /// 6_Exp/64_Viz/ → 세션별 Figure 색인 `_gallery.md` 생성 · Generate the figure gallery index
    Gallery,
    /// 환경+프로젝트 종합 진단 · Environment + project health check (읽기전용/read-only)
    Doctor,
}

#[derive(Subcommand)]
enum SessionCmd {
    /// 새 세션 로그 생성 + Registry 등록 · Create + register a new session log (S### 자동 증번)
    New {
        /// 세션 제목 · session title (Registry 기록 — 탭 불가)
        title: String,
    },
    /// 활성 세션 종료 · Close the active session (→ Complete + Archive + registry, cross-ref 보정)
    Close {
        /// 닫을 세션 ID · session id (생략 시 유일 활성 자동 선택)
        id: Option<String>,
        /// '다음 세션 후보' 미작성도 강제 종료 · force close
        #[arg(long)]
        force: bool,
    },
    /// 세션 로그 헤더 hard break(`\`) 보정 · Repair session-log header hard breaks
    #[command(name = "fix-headers")]
    FixHeaders {
        /// 변경 없이 대상 파일만 출력 · dry-run (list targets only)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum TrialCmd {
    /// 활성 세션 로그에 현행 trialTemplate stub 추가 · Append the current canonical trial stub to the active session log
    New {
        /// trial 제목 · trial title (생략 시 placeholder 유지 / omit = keep placeholder)
        title: Option<String>,
        /// 대상 세션 · target session (S### — 생략 시 유일 활성 자동 선택)
        #[arg(long)]
        session: Option<String>,
    },
}

// exit code 규약: 0=성공, 1=실행 오류, 2=usage(clap 기본), 3=refuse, 4=check 발견, 5=escalation(상위 에이전트 위임).
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            name,
            here,
            yes,
            dry_run,
            force,
            preset,
            modules,
            categories,
            lang,
        } => {
            let cwd = std::env::current_dir().expect("cwd");
            // mode: 무명 · `--here` · `.` → 현재 폴더 in-place (P018 name-presence)
            let dot = name.as_deref() == Some(".");
            let in_place = here || name.is_none() || dot;
            // project name: in-place는 cwd basename(또는 `--here <name>`), 그 외는 인자명
            let proj_name = if in_place {
                match name.clone().filter(|n| n != ".") {
                    Some(n) => n,
                    None => cwd
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("project")
                        .to_string(),
                }
            } else {
                name.clone().expect("subfolder requires a name")
            };
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let label = match &modules {
                Some(keys) => format!("custom({})", keys.join(",")),
                None => preset.clone(),
            };
            let opts = init::InitOptions {
                name: proj_name,
                preset,
                modules,
                categories: categories.unwrap_or_default(),
                lang,
                date,
            };

            // graduated confirm (P018 §4): in-place + cwd 비어있지않음 + !yes + !dry-run → 위치 echo
            if in_place && !yes && !dry_run {
                let non_empty = std::fs::read_dir(&cwd)
                    .map(|mut r| r.next().is_some())
                    .unwrap_or(false);
                if non_empty {
                    use std::io::Write;
                    print!(
                        "[elf] Initialize ELF in {}? Existing files are left untouched. [Y/n] ",
                        cwd.display()
                    );
                    let _ = std::io::stdout().flush();
                    let mut line = String::new();
                    let _ = std::io::stdin().read_line(&mut line);
                    let ans = line.trim();
                    if !(ans.is_empty() || ans.eq_ignore_ascii_case("y") || ans.eq_ignore_ascii_case("yes"))
                    {
                        eprintln!("[elf] aborted.");
                        std::process::exit(0);
                    }
                }
            }

            match init::run_init_ex(&cwd, &opts, in_place, dry_run, force) {
                Ok(report) => {
                    if dry_run {
                        println!("[elf] dry-run — nothing written");
                    }
                    println!(
                        "[elf] {} {} (ELF {}, preset: {}, lang: {})",
                        if dry_run { "would init" } else { "created" },
                        report.target.display(),
                        embed::version(),
                        label,
                        opts.lang
                    );
                    if in_place {
                        println!(
                            "[elf]   added {}, skipped {} (kept), .elf-new {}",
                            report.created.len(),
                            report.skipped.len(),
                            report.elf_new.len()
                        );
                        for s in &report.skipped {
                            println!("[elf]     kept your {s}");
                        }
                        for n in &report.elf_new {
                            println!("[elf]     ELF version at {n}");
                        }
                        println!("[elf]   existing files left untouched");
                    }
                    // 진입 안내는 계보별 실재 파일로 — qa는 0_Meta 없음 (S026 부수 수정)
                    let next = match manifest::kind_from_preset(&opts.preset) {
                        manifest::Kind::Qa => "README.md",
                        _ => "0_Meta/ProjectRule.md",
                    };
                    println!("[elf] next: open {next}");
                }
                Err(e @ (init::InitError::TargetExists(_) | init::InitError::AlreadyElf(_))) => {
                    eprintln!("[elf] {e}");
                    std::process::exit(3);
                }
                Err(e) => {
                    eprintln!("[elf] error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::SelfUpdate => do_self_update(),
        Commands::Update { update_self: true, .. } => do_self_update(),
        Commands::Update { dry_run, force, update_self: false } => {
            let cwd = std::env::current_dir().expect("cwd");
            let Some(root) = update::find_project_root(&cwd) else {
                eprintln!("[elf] error: not an ELF project (no .elf/manifest.json upward from {})", cwd.display());
                std::process::exit(1);
            };
            let opts = update::UpdateOptions { dry_run, force };
            match update::run_update(&root, &opts) {
                Ok(report) => {
                    for line in &report.lines {
                        println!("[elf] {line}");
                    }
                    println!(
                        "[elf] done: {} changed, {} conflicts, {} warnings{}",
                        report.changed,
                        report.conflicts,
                        report.warnings,
                        if dry_run { " (dry-run — nothing written)" } else { "" }
                    );
                }
                Err(e) => {
                    eprintln!("[elf] error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Status { check } => {
            let cwd = std::env::current_dir().expect("cwd");
            let Some(root) = update::find_project_root(&cwd) else {
                eprintln!("[elf] error: not an ELF project (no .elf/manifest.json upward from {})", cwd.display());
                std::process::exit(1);
            };
            match status::run_status(&root) {
                Ok(report) => {
                    for line in &report.lines {
                        println!("[elf] {line}");
                    }
                    println!(
                        "[elf] status: {} pending, {} conflicts, {} warnings",
                        report.pending, report.conflicts, report.warnings
                    );
                    if check && report.findings() > 0 {
                        std::process::exit(4);
                    }
                }
                Err(e) => {
                    eprintln!("[elf] error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Doctor => {
            let cwd = std::env::current_dir().expect("cwd");
            let env = doctor::DoctorEnv {
                version: embed::version().to_string(),
                receipt_present: doctor::probe_receipt(),
            };
            let report = doctor::run_doctor(&cwd, &env);
            for c in &report.checks {
                let mark = match c.health {
                    doctor::Health::Ok => "OK  ",
                    doctor::Health::Warn => "WARN",
                    doctor::Health::Info => "INFO",
                };
                println!("[elf] [{mark}] {}: {}", c.label, c.detail);
            }
            println!("[elf] doctor: {} warning(s)", report.warnings());
        }
        Commands::Gallery => {
            let root = log_root_or_exit();
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
            match gallery::run_gallery(&root, &now) {
                Ok(r) if !r.viz_present => {
                    println!("[elf] 6_Exp/64_Viz/ not found — nothing to do");
                }
                Ok(r) => {
                    println!(
                        "[elf] wrote {} ({} session(s), {} image(s))",
                        r.output_rel, r.sessions, r.images
                    );
                }
                Err(e) => {
                    eprintln!("[elf] io error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate { check, strict } => {
            let root = log_root_or_exit();
            match validate::run_validate_opts(&root, strict) {
                Ok(report) => {
                    for line in &report.lines {
                        println!("[elf] {line}");
                    }
                    println!(
                        "[elf] validate: {} issue(s), {} warning(s)",
                        report.issues, report.warnings
                    );
                    if check && report.findings() > 0 {
                        std::process::exit(4);
                    }
                }
                Err(validate::ValidateError::Escalation(e)) => {
                    eprintln!("[elf] {e}");
                    std::process::exit(5);
                }
                Err(validate::ValidateError::Io(e)) => {
                    eprintln!("[elf] io error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Session { cmd } => match cmd {
            SessionCmd::New { title } => {
                let root = log_root_or_exit();
                let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let opts = session::SessionNewOptions { title, date };
                match session::run_session_new(&root, &opts) {
                    Ok(res) => {
                        for w in &res.warnings {
                            println!("[elf] warn: {w}");
                        }
                        println!("[elf] created {} ({}) + registry row", res.log_rel, res.id);
                    }
                    Err(session::SessionError::Escalation(e)) => {
                        eprintln!("[elf] {e}");
                        std::process::exit(5);
                    }
                    Err(session::SessionError::Exists(p)) => {
                        eprintln!("[elf] refuse: {p} already exists");
                        std::process::exit(3);
                    }
                    Err(session::SessionError::BadTitle(m)) => {
                        eprintln!("[elf] error: {m}");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::Io(e)) => {
                        eprintln!("[elf] io error: {e}");
                        std::process::exit(1);
                    }
                    // close 전용 변형은 new 경로에서 발생 불가 (공용 enum 망라용)
                    Err(
                        e @ (session::SessionError::NoOpenSession
                        | session::SessionError::MultipleOpen(_)
                        | session::SessionError::NotFound(_)
                        | session::SessionError::MissingNextSection(_)),
                    ) => unreachable!("session new cannot yield close-only error: {e:?}"),
                }
            }
            SessionCmd::Close { id, force } => {
                let root = log_root_or_exit();
                match session::run_session_close(&root, &session::CloseOptions { id, force }) {
                    Ok(r) => {
                        for w in &r.warnings {
                            println!("[elf] warn: {w}");
                        }
                        println!(
                            "[elf] closed {} → {} (Status: Complete, registry updated)",
                            r.id, r.archived_to
                        );
                        println!(
                            "[elf] note: rewrite the registry key finding as the session's final conclusion (fold — LogConvention §5.2)"
                        );
                    }
                    Err(session::SessionError::Escalation(e)) => {
                        eprintln!("[elf] {e}");
                        std::process::exit(5);
                    }
                    Err(session::SessionError::NoOpenSession) => {
                        eprintln!("[elf] nothing to close: no open session in 2_Log/ (all Complete)");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::MultipleOpen(ids)) => {
                        eprintln!(
                            "[elf] multiple open sessions ({}) — specify one: elf session close <S###>",
                            ids.join(", ")
                        );
                        std::process::exit(1);
                    }
                    Err(session::SessionError::NotFound(id)) => {
                        eprintln!("[elf] session not found: {id}");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::MissingNextSection(id)) => {
                        eprintln!(
                            "[elf] refuse: {id} has no filled '다음 세션 후보' section (LogConvention §5.2) — fill it or use --force"
                        );
                        std::process::exit(3);
                    }
                    Err(session::SessionError::Exists(p)) => {
                        eprintln!("[elf] refuse: {p} already exists");
                        std::process::exit(3);
                    }
                    Err(session::SessionError::BadTitle(m)) => {
                        eprintln!("[elf] error: {m}");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::Io(e)) => {
                        eprintln!("[elf] io error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            SessionCmd::FixHeaders { dry_run } => {
                let root = log_root_or_exit();
                match session::run_fix_headers(&root, dry_run) {
                    Ok(files) => {
                        for f in &files {
                            println!("[elf] {} {}", if dry_run { "would fix" } else { "fixed" }, f.path);
                        }
                        println!(
                            "[elf] {} file(s) {}",
                            files.len(),
                            if dry_run { "would change (dry-run)" } else { "changed" }
                        );
                    }
                    Err(e) => {
                        eprintln!("[elf] io error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Trial { cmd } => match cmd {
            TrialCmd::New { title, session: sess } => {
                let root = log_root_or_exit();
                let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let opts = trial::TrialNewOptions { title, session: sess, date };
                match trial::run_trial_new(&root, &opts) {
                    Ok(r) => {
                        println!("[elf] appended {} to {} ({})", r.trial, r.log_rel, r.session);
                        println!(
                            "[elf] next: Phase 1 — fill 목표/조건/가설/예상, then stop before execution (LogConvention §5.1)"
                        );
                        // 행동 시점 주입 (S027): 상시 문서의 embed 규칙은 발동 시점을 보장하지 못함 —
                        // trial 시작(= figure 작성 이전이 보장되는 유일한 ELF 소유 시점)에 반복 주입
                        println!(
                            "[elf] note: list expected figures in 예상, and embed each into `### 관찰` the moment it is created — a table path is not an embed; sub-agent outputs included (LogConvention §2)"
                        );
                        println!(
                            "[elf] note: keep the header Handoff a replace-style fold (state; pending; refs) — do not append history"
                        );
                    }
                    Err(session::SessionError::NoOpenSession) => {
                        eprintln!("[elf] no open session in 2_Log/ — start one with `elf session new \"<title>\"`");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::MultipleOpen(ids)) => {
                        eprintln!(
                            "[elf] multiple open sessions ({}) — specify one: elf trial new --session <S###>",
                            ids.join(", ")
                        );
                        std::process::exit(1);
                    }
                    Err(session::SessionError::NotFound(id)) => {
                        eprintln!("[elf] session not found: {id}");
                        std::process::exit(1);
                    }
                    Err(session::SessionError::Io(e)) => {
                        eprintln!("[elf] io error: {e}");
                        std::process::exit(1);
                    }
                    Err(e) => {
                        eprintln!("[elf] error: {e:?}");
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}

// session 명령 공용 — `2_Log/` 보유 디렉토리 탐지(`.elf/` 불요; framework _dev도 대상).
fn log_root_or_exit() -> std::path::PathBuf {
    let cwd = std::env::current_dir().expect("cwd");
    match session::find_log_root(&cwd) {
        Some(r) => r,
        None => {
            eprintln!("[elf] error: no 2_Log/ directory found from {}", cwd.display());
            std::process::exit(1);
        }
    }
}

fn do_self_update() {
    match selfupdate::run_self_update() {
        Ok(msg) => println!("[elf] {msg}"),
        Err(e) => {
            eprintln!("[elf] {e}");
            std::process::exit(1);
        }
    }
}
