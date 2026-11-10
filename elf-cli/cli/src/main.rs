use clap::{Parser, Subcommand, ValueEnum};
use elf_cli::{autoread, check, consent, deprecate, dispatch, doctor, earlier, embed, gallery, init, legacy, logcmd, manifest, migrate, pin, selfupdate, session, status, store, trial, tsa, update, validate, version};

/// ELF (Eli's Lab Framework) 연구 프로젝트 스캐폴드, 갱신 CLI (research project scaffold & update CLI)
#[derive(Parser)]
#[command(name = "elf", version = embed::version(), arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// (주의: doc comment(`///`)는 help로 노출됨 — 내부 표기 금지, 회귀 테스트가 게이트. P011 t09)
#[derive(Subcommand)]
enum Commands {
    /// 새 ELF 프로젝트 스캐폴드 생성 / Scaffold an ELF project (생략 시 현재 폴더에 in-place)
    Init {
        /// 프로젝트 폴더 이름 (생략 시 현재 폴더 in-place) / project folder name (omit = in-place here)
        name: Option<String>,
        /// 현재 폴더에 만들면서 이름 지정 / init in the current folder under the given name (`--here <name>`)
        #[arg(long)]
        here: bool,
        /// 확인 프롬프트 생략 / skip the confirmation prompt
        #[arg(long)]
        yes: bool,
        /// 변경 없이 계획만 출력 / preview only, write nothing
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// 기존 파일도 덮어씀 / overwrite existing files
        #[arg(long)]
        force: bool,
        /// preset: full | exp | sw | minimal | qa — 연구 preset = 이름과 같은 주 모듈 + paper (full = 전체, minimal = 없음); qa = 질문 아카이브 / research presets = the module of the same name + paper; qa = question archive
        /// (생략 시 full — 기존 ELF 폴더에 제자리 실행하면 그 config의 preset / omit = full, or the existing config's preset in place)
        #[arg(long, conflicts_with = "modules")]
        preset: Option<String>,
        /// custom 모듈 선택 / custom modules (쉼표 구분: hw,fab,sw,exp,paper) — preset 대신
        #[arg(long, value_delimiter = ',')]
        modules: Option<Vec<String>>,
        /// (qa preset) 사전 생성 카테고리 / pre-create categories (쉼표 구분) — 기본 0개(수요 기반)
        #[arg(long, value_delimiter = ',')]
        categories: Option<Vec<String>>,
        /// 프로젝트 언어 / project language (AI 응답 언어 — .elf/config.json; 생략 시 ko-KR 또는 기존 config 값 / omit = ko-KR, or the existing config's value in place)
        #[arg(long)]
        lang: Option<String>,
    },
    /// 프로젝트의 ELF 관리 파일 갱신과 이전 형식 로그 이관 / Update this project's ELF-managed files and migrate
    /// earlier-layout logs (LogConvention §7; elf 실행 파일 자체는 `elf self update` / for the binary itself)
    Update {
        /// 변경 없이 계획만 출력(이관 포함) / dry-run: print the plan, including the migration, write nothing
        #[arg(long)]
        dry_run: bool,
        /// 사용자 편집 무시 강제 갱신 / force overwrite (managed 덮어쓰기, hybrid 블록 교체)
        #[arg(long)]
        force: bool,
        /// 이관 미리보기 폴더(`--dry-run`과 함께): 결과 파일, 헤더 표, 링크와 코드 span 변경, 경로 언급, 경고 /
        /// with --dry-run: write the migration's result files and tables (headers, changes, mentions, warnings) here
        #[arg(long, value_name = "FOLDER", requires = "dry_run")]
        out: Option<std::path::PathBuf>,
        /// 목표 줄(생략 = 이 elf의 줄). 줄을 바꾸는 갱신은 요약과 확인 코드를 출력하고 사용자 승인을 기다림 /
        /// target line (default: this elf's line); a change of line prints a summary and an approval code first
        #[arg(long, value_name = "LINE")]
        to: Option<String>,
        /// 사용자가 승인한 확인 코드(요약에 나온 값) / the approval code from the summary, after the user approved
        #[arg(long, value_name = "CODE", conflicts_with = "dry_run")]
        approve: Option<String>,
        /// (구 형식) `elf self update`와 같음 / same as `elf self update`
        #[arg(long = "self", hide = true, conflicts_with_all = ["dry_run", "force"])]
        update_self: bool,
    },
    /// elf 실행 파일 관리 / Manage the elf executable (update)
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        cmd: SelfCmd,
    },
    /// (구 이름) `elf self update`와 같음 / same as `elf self update`
    #[command(name = "self-update", alias = "selfupdate", hide = true)]
    SelfUpdate,
    /// 세션 수명주기 / Session lifecycle (new / close)
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },
    /// trial 만들기와 쓰기 / Create a trial and write its sections (new / write)
    Trial {
        #[command(subcommand)]
        cmd: TrialCmd,
    },
    /// 세션 로그 목록과 보기(읽기 전용) / List session logs, or show one (read-only)
    Log {
        #[command(subcommand)]
        cmd: Option<LogCmd>,
    },
    /// 점검 / Check versions, setup, ELF-managed files, and session logs (범위 생략 = 전체, 읽기전용/read-only; 문제 시 exit 4)
    Check {
        /// 범위 / scope: version (CLI와 프로젝트 manifest), setup (git, 훅, autoread), files (ELF 관리 파일), logs (세션 로그, Registry.md) — 생략 = 전체 요약
        #[arg(value_enum)]
        scope: Option<CheckScope>,
    },
    /// (구 이름) `elf check files` 상세 / managed-file status (alias)
    #[command(hide = true)]
    Status {
        /// 발견 시 exit 4 / exit 4 on findings
        #[arg(long)]
        check: bool,
    },
    /// (구 이름) `elf check logs` / session-log consistency (alias)
    #[command(hide = true)]
    Validate {
        /// issue 발견 시 exit 4 / exit 4 on issues
        #[arg(long)]
        check: bool,
    },
    /// 6_Exp/64_Viz/ → 세션별 Figure 색인 `_gallery.md` 생성 / Generate the figure gallery index
    Gallery,
    /// (구 이름) 환경 + 프로젝트 진단 / environment + project diagnosis (alias of the setup and files checks)
    #[command(hide = true)]
    Doctor,
    /// 연구 기록 시점인증 / Research-record timestamping (manifest + RFC 3161 TSA, opt-in) — 하위 명령 없음 = 오늘 manifest의 시점 토큰 요청 / no subcommand = request today's timestamp token
    #[command(args_conflicts_with_subcommands = true)]
    Tsa {
        #[command(subcommand)]
        cmd: Option<TsaCmd>,
        /// 토큰 없는 manifest 전부 / all unstamped manifests (오프라인 커밋 소급 보충)
        #[arg(long)]
        all: bool,
        /// 출력 억제 / suppress output (훅용)
        #[arg(long)]
        quiet: bool,
    },
    /// 세션 시작, compact 후 거버넌스 자동 주입 / Governance digest at session start and after context reconstruction (기본 켬/default-on) — 하위 명령 없음 = digest 출력 / no subcommand = print the digest
    Autoread {
        #[command(subcommand)]
        cmd: Option<AutoreadCmd>,
    },
    /// 폐기, 복원, 보기, 목록 / Deprecate content (removed from the document, stored compressed in .elf/deprecated/), restore, show; 인수 없음 = 목록 / no arguments = list
    #[command(args_conflicts_with_subcommands = true)]
    Deprecate {
        #[command(subcommand)]
        cmd: Option<DeprecateCmd>,
        /// 대상 / target: S### | G### | P### | project-relative path (omit = list; omit with --restore)
        target: Option<String>,
        /// trial 단위 / one trial (e.g. t03) — leaves the numbered heading and a marker line
        #[arg(long, value_name = "tNN")]
        trial: Option<String>,
        /// 절 단위 / one section of --trial (e.g. 해석 or Interpretation) — the heading goes with the body
        #[arg(long, value_name = "NAME", requires = "trial")]
        section: Option<String>,
        /// 표시 블록 / the block wrapped in <!-- deprecate:begin --> … <!-- deprecate:end -->
        #[arg(long)]
        marked: bool,
        /// 줄 범위 / lines A-B (1-based, inclusive); requires --expect
        #[arg(long, value_name = "A-B")]
        lines: Option<String>,
        /// 줄 범위 확인 / exact text of line A (guards --lines against drift)
        #[arg(long, value_name = "TEXT", requires = "lines")]
        expect: Option<String>,
        /// 대체 / what replaces the deprecated content (recorded in the marker)
        #[arg(long = "replaced-by", value_name = "TEXT")]
        replaced_by: Option<String>,
        /// 사유 / reason (recorded in the marker)
        #[arg(long, value_name = "TEXT")]
        reason: Option<String>,
        /// 변경 없이 출력만 / preview only, write nothing
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// 복원 / restore a deprecation id (S012-D01) and delete its store file
        #[arg(long, value_name = "ID", conflicts_with_all = ["target", "trial", "marked", "lines"])]
        restore: Option<String>,
    },
}

#[derive(Subcommand)]
enum SelfCmd {
    /// elf 실행 파일을 최신 릴리즈로 갱신 / Update the elf binary to the latest release (프로젝트 파일은 `elf update`)
    Update,
    /// 버전 보관소(~/.elf/versions) 목록 / List the elf versions kept in the version store (~/.elf/versions)
    List {
        /// 줄마다 최신 발행도 보임(네트워크) / also list the newest release of each line (network)
        #[arg(long)]
        available: bool,
    },
    /// 보관소에 설치: `2.27` = 그 줄의 최신 patch, `2.27.4` = 그 버전 / Install into the version store:
    /// `2.27` = the newest patch of that line, `2.27.4` = that exact release
    Install {
        #[arg(value_name = "VERSION")]
        version: String,
    },
    /// 보관된 버전 하나 삭제 / Remove one version from the version store
    Uninstall {
        #[arg(value_name = "VERSION")]
        version: String,
    },
    /// 줄마다 가장 새 버전만 남김 / Keep only the newest version of each line in the version store
    Prune,
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckScope {
    /// 버전 / CLI and project manifest versions, one-line verdict (elf update or elf self update)
    Version,
    /// 설정 / git and hooks, autoread, experimental
    Setup,
    /// ELF 관리 파일 / managed files, .elf record, overlay, AGENTS link, tsa
    Files,
    /// 세션 로그, Registry / session logs, registry, figure embeds
    Logs,
}

#[derive(Subcommand)]
enum DeprecateCmd {
    /// (구 이름) 목록 = 인수 없는 `elf deprecate` / list (same as `elf deprecate` with no arguments)
    #[command(hide = true)]
    List,
    /// 보관 원문 보기(사용자가 요청할 때만) / Print the stored original (only when the user asks)
    Show {
        /// 폐기 ID / deprecation id (e.g. S012-D01)
        id: String,
    },
    /// (2.27 형식 저장소 전용, experimental) 영구 삭제 / earlier layout only: purge (experimental)
    #[command(hide = true)]
    Purge {
        id: String,
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum AutoreadCmd {
    /// 켬 / Enable (config `"autoread": true` + Claude Code hooks — 멱등/idempotent, 프로젝트 단위)
    Enable,
    /// 끔 / Disable (config `"autoread": false` — 훅은 잔존, no-op/hooks stay but no-op)
    Disable,
    /// 상태 / Status (config, hooks, pending markers — 읽기전용/read-only)
    Status,
    /// (내부) 하네스 훅 진입점 / harness hook entrypoint (stdin JSON)
    #[command(hide = true)]
    Hook {
        /// 이벤트 / event (session-start | prompt)
        event: String,
    },
}

#[derive(Subcommand)]
enum TsaCmd {
    /// 기능 도입 / Enable (config + hooks + baseline record — 멱등/idempotent, 기존 훅 비파괴)
    Enable,
    /// 기능 해제 / Disable (config off + elf 훅만 제거 — 증거 `0_Meta/tsa/`는 보존/kept)
    Disable,
    /// 상태 / Status (enabled, hooks, evidence counts — 읽기전용/read-only)
    Status,
    /// 해시 기록 / Record hashes of staged files into today's manifest (pre-commit 훅이 호출)
    Record {
        /// git 추적 전체 / all tracked files (baseline)
        #[arg(long)]
        all: bool,
        /// 출력 억제 / suppress output (훅용)
        #[arg(long)]
        quiet: bool,
    },
    /// 검증 / Verify a file against manifests, or a day's manifest↔token pair (읽기전용)
    Verify {
        /// 검증할 파일 / file to look up (sha256 → manifest 이력)
        file: Option<String>,
        /// 날짜 쌍 검증 / verify manifest↔token for a date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
    },
}

#[derive(Subcommand)]
enum SessionCmd {
    /// 새 세션 로그 / Create a session log (2_Log/S/S###, --gov = 2_Log/G/G###) with a YAML header
    New {
        /// 세션 제목 / session title
        title: String,
        /// 세션 목표 1-2문장(필수 — 이전 형식 저장소에서는 쓰지 않음) / the session goal in 1-2 sentences (required; not used in a project still in the 2.27 layout)
        #[arg(long)]
        goal: Option<String>,
        /// 거버넌스 세션 / governance session (rules, structure, templates, tool settings) → 2_Log/G/
        #[arg(long)]
        gov: bool,
    },
    /// 세션 종료 / Close a session: status complete + conclusion (required) from a file
    Close {
        /// 닫을 세션 ID / session id (S### or G###; 생략 시 진행 중 세션이 하나일 때 그것)
        id: Option<String>,
        /// 종료 파일 / close file: YAML block with conclusion (required), user_note, handoff
        #[arg(long, value_name = "FILE")]
        from: Option<std::path::PathBuf>,
        /// 호환용(무효과) / accepted for compatibility, no effect
        #[arg(long, hide = true)]
        force: bool,
    },
    /// (2.27 형식 저장소 전용) 헤더 줄바꿈 보정 / earlier layout only: repair blockquote header hard breaks
    #[command(name = "fix-headers", hide = true)]
    FixHeaders {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum LogCmd {
    /// 헤더와 trial 목록, 또는 한 trial 본문 / Header and trial list, or one trial's text
    Show {
        /// S### | G###
        id: String,
        /// tNN — 그 trial 본문 / print this trial's text
        trial: Option<String>,
    },
}

#[derive(Subcommand)]
enum TrialCmd {
    /// 새 trial / Create the next trial (title, Phase 1 template); --from = then write Phase 1 from a file
    New {
        /// trial 제목 / trial title (생략 시 placeholder 유지 / omit = keep placeholder)
        title: Option<String>,
        /// 대상 세션 / target session (S### or G### — 생략 시 진행 중 세션이 하나일 때 그것)
        #[arg(long)]
        session: Option<String>,
        /// Phase 1 파일 / Phase 1 sections file (top YAML block: handoff only)
        #[arg(long, value_name = "FILE")]
        from: Option<std::path::PathBuf>,
    },
    /// trial 절 쓰기 / Write the sections in a file into a trial: Phase 1 (replace, until Phase 2 exists) or Phase 2 (once)
    Write {
        /// 대상 trial / target trial (tNN — 생략 시 그 세션의 마지막 trial)
        trial: Option<String>,
        /// 대상 세션 / target session (S### or G###)
        #[arg(long)]
        session: Option<String>,
        /// 절 파일 / sections file (one phase; top YAML block: handoff only)
        #[arg(long, value_name = "FILE")]
        from: std::path::PathBuf,
    },
}

// exit code 규약: 0=성공, 1=실행 오류, 2=usage(clap 기본), 3=refuse(이관 경고 포함), 4=check 발견. (5 escalation은 Registry TSV 폐지로 없어짐, P022)
fn main() {
    // 실행 위임(v3.0): 프로젝트 줄이 이 CLI와 다르면 보관소 버전으로 넘기고 그 종료 코드로 끝남
    dispatch::maybe_dispatch();
    let cli = Cli::parse();
    // autoread 배너 — 재구성 마커 대기 중이면 모든 명령 출력 선두 1행(보조 주입 채널: auto-compact
    // 직후 프롬프트 미경유 경로 커버). 제외 = autoread 자신(훅 출력 계약, digest 중복), tsa quiet(훅 경로).
    let banner_suppressed = matches!(cli.command, Commands::Autoread { .. })
        || matches!(
            cli.command,
            Commands::Tsa { cmd: Some(TsaCmd::Record { quiet: true, .. }), .. } | Commands::Tsa { cmd: None, quiet: true, .. }
        );
    if !banner_suppressed {
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(b) = autoread::pending_banner(&cwd) {
                println!("[elf] {b}");
            }
        }
    }
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
            // mode: 무명, `--here`, `.` → 현재 폴더 in-place (P018 name-presence)
            let dot = name.as_deref() == Some(".");
            let in_place = here || name.is_none() || dot;
            // 제자리 도입: 기존 `.elf/config.json`(manifest 없는 구버전 포함)의 name, preset, lang을 기본값으로 (S042 t04)
            let existing: Option<serde_json::Value> = if in_place {
                std::fs::read_to_string(cwd.join(".elf").join("config.json"))
                    .ok()
                    .and_then(|t| serde_json::from_str(&t).ok())
            } else {
                None
            };
            let existing_str = |k: &str| -> Option<String> {
                existing.as_ref().and_then(|v| v.get(k)).and_then(|x| x.as_str()).map(String::from)
            };
            // project name: in-place는 `--here <name>` > 기존 config name > cwd basename, 그 외는 인자명
            let proj_name = if in_place {
                match name.clone().filter(|n| n != ".") {
                    Some(n) => n,
                    None => existing_str("name").unwrap_or_else(|| {
                        cwd.file_name().and_then(|s| s.to_str()).unwrap_or("project").to_string()
                    }),
                }
            } else {
                name.clone().expect("subfolder requires a name")
            };
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            // preset: 명시 > 기존 config(삭제된 general → minimal, custom은 modules와 함께만 의미) > full.
            // 구 이름(experimental, software)과 계보 이름(research)은 현행 이름으로(미지정 연구 preset = full).
            let preset = match preset {
                Some(p) => manifest::normalize_preset(Some(&p)),
                None => match existing_str("preset").as_deref() {
                    Some(manifest::REMOVED_GENERAL) => "minimal".to_string(),
                    Some("custom") | None => "full".to_string(),
                    Some(p) => manifest::normalize_preset(Some(p)),
                },
            };
            let lang = lang.or_else(|| existing_str("lang")).unwrap_or_else(|| "ko-KR".to_string());
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
                        for n in &report.notes {
                            println!("[elf]     {n}");
                        }
                        for n in &report.elf_new {
                            println!("[elf]     ELF version at {n}");
                        }
                        println!("[elf]   existing files left untouched");
                    }
                    // 진입 안내는 계보별 실재 파일로 — qa는 0_Meta 없음 (S026 부수 수정)
                    let kind = manifest::kind_from_preset(&opts.preset);
                    let next = match kind {
                        manifest::Kind::Qa => "README.md",
                        _ => "0_Meta/ProjectRule.md",
                    };
                    println!("[elf] next: open {next}");
                    if kind != manifest::Kind::Qa && report.created.iter().any(|c| c == "2_Log/S001_log.md") {
                        // v2.22: S001 로그는 헤더만(t01 stub 제거) — 첫 trial은 `elf trial new`
                        println!("[elf] next: elf trial new \"<title>\" → S001 t01 (the session log holds the header only)");
                    }
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
        Commands::SelfCmd { cmd: SelfCmd::Update } => do_self_update(),
        Commands::SelfCmd { cmd } => do_store(cmd),
        Commands::SelfUpdate => do_self_update(),
        Commands::Update { update_self: true, .. } => do_self_update(),
        Commands::Update { dry_run, force, out, to, approve, update_self: false } => {
            let cwd = std::env::current_dir().expect("cwd");
            // 목표와 동의 관문(v3.0 S044 t60): 낮은 줄 거부, 세대 건너뛰기 거부, 다른 줄 = 목표 버전으로 넘김,
            // 줄을 바꾸는 갱신 = 요약과 확인 코드(승인 전 아무것도 쓰지 않음)
            let gate = |root: &std::path::Path, log_only: bool| match consent::gate_update(root, log_only, to.as_deref(), approve.as_deref(), dry_run) {
                consent::Gate::Proceed(lines) => {
                    for l in lines {
                        println!("[elf] {l}");
                    }
                }
                consent::Gate::Dispatch { exe, args, line, version } => {
                    if std::env::var(dispatch::ENV_DISPATCH).is_ok_and(|v| v == "trace") {
                        eprintln!("[elf] dispatch: update to line {line} → {version} ({})", store::shown(&exe));
                    }
                    std::process::exit(dispatch::run(&exe, &args, embed::version()));
                }
                consent::Gate::Stop(code, lines) => {
                    for l in lines {
                        if code == 3 && !l.starts_with("refuse") && !l.starts_with("error") {
                            println!("[elf] {l}");
                        } else {
                            eprintln!("[elf] {l}");
                        }
                    }
                    std::process::exit(code);
                }
            };
            // 이관 미리보기 폴더 준비(이관할 것이 있을 때만) — 잘못된 위치는 usage 오류
            let preview_for = |root: &std::path::Path| -> Option<std::path::PathBuf> {
                let o = out.as_deref()?;
                if !migrate::needed(root) {
                    println!("[elf] --out: nothing to migrate here — no preview written");
                    return None;
                }
                match migrate::prepare_preview(root, o) {
                    Ok(p) => Some(p),
                    Err(m) => {
                        eprintln!("[elf] error: {m}");
                        std::process::exit(2);
                    }
                }
            };
            // `.elf/` 없는 2.0~2.4.0 프로젝트(규칙이 `0_Meta/`)도 프로젝트로 갱신 — 합성 stamp(v3.0.2 S046)
            let Some(root) = update::find_project_root(&cwd).or_else(|| earlier::find_root(&cwd)) else {
                // 로그만 있는 폴더(`.elf/` 없음, 예: 프레임워크 `_dev`) — 구조 이관만
                if let Some(lr) = session::find_log_root(&cwd)
                    && migrate::needed(&lr)
                {
                    gate(&lr, true);
                    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                    let preview = preview_for(&lr);
                    match migrate::run_with(&lr, dry_run, &today, preview.as_deref()) {
                        Ok(migrate::Outcome::Blocked(lines)) => {
                            for l in &lines {
                                println!("[elf] {l}");
                            }
                            std::process::exit(3);
                        }
                        Ok(migrate::Outcome::Applied(lines) | migrate::Outcome::Planned(lines)) => {
                            for l in &lines {
                                println!("[elf] {l}");
                            }
                            if dry_run {
                                println!("[elf] log-only folder (no .elf/) — the migration covers the logs only (dry-run — nothing written)");
                            } else {
                                println!("[elf] log-only folder (no .elf/) — migrated the logs only");
                            }
                        }
                        Err(e) => {
                            eprintln!("[elf] io error: {e}");
                            std::process::exit(1);
                        }
                    }
                    return;
                }
                eprintln!("[elf] error: not an ELF project (no .elf/manifest.json upward from {})", cwd.display());
                std::process::exit(1);
            };
            gate(&root, false);
            let preview = preview_for(&root);
            let opts = update::UpdateOptions { dry_run, force };
            match update::run_update_with(&root, &opts, preview.as_deref()) {
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
                Err(update::UpdateError::NewerProject(m)) => {
                    eprintln!("[elf] refuse: {m}");
                    std::process::exit(3);
                }
                Err(update::UpdateError::Migration(lines)) => {
                    for l in &lines {
                        println!("[elf] {l}");
                    }
                    std::process::exit(3);
                }
                Err(e) => {
                    eprintln!("[elf] error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Check { scope } => {
            let cwd = std::env::current_dir().expect("cwd");
            let env = doctor::DoctorEnv {
                version: embed::version().to_string(),
                receipt_present: doctor::probe_receipt(),
            };
            let code = match scope {
                None => print_check_all(&cwd, &env),
                Some(s) => print_check_scope(&cwd, &env, scope_of(s)),
            };
            if code != 0 {
                std::process::exit(code);
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
        Commands::Tsa { cmd, all, quiet } => {
            let cwd = std::env::current_dir().expect("cwd");
            let Some(root) = update::find_project_root(&cwd) else {
                eprintln!("[elf] error: not an ELF project (no .elf/manifest.json upward from {})", cwd.display());
                std::process::exit(1);
            };
            let quiet = matches!(cmd, Some(TsaCmd::Record { quiet: true, .. })) || (cmd.is_none() && quiet);
            // enabled 게이트: disable 후 타 훅에 수동 잔존한 호출 라인의 무해화 —
            // 훅 경로(quiet)는 침묵 성공, 직접 실행은 refuse 안내.
            if matches!(cmd, None | Some(TsaCmd::Record { .. })) && !tsa::is_enabled(&root) {
                if quiet {
                    return;
                }
                eprintln!("[elf] tsa is disabled — run `elf tsa enable` first");
                std::process::exit(3);
            }
            let result = match cmd {
                // 하위 명령 없음 = 시점 토큰 요청(기본 = 오늘 manifest, --all = 토큰 없는 manifest 전부)
                None => tsa::run_stamp(&root, all),
                Some(TsaCmd::Enable) => tsa::run_enable(&root),
                Some(TsaCmd::Disable) => tsa::run_disable(&root),
                Some(TsaCmd::Status) => tsa::run_status(&root),
                Some(TsaCmd::Record { all, .. }) => {
                    let scope = if all { tsa::RecordScope::All } else { tsa::RecordScope::Staged };
                    tsa::run_record(&root, scope)
                }
                Some(TsaCmd::Verify { file, date }) => {
                    tsa::run_verify(&root, file.as_deref(), date.as_deref())
                }
            };
            match result {
                Ok(report) => {
                    if !quiet {
                        for line in &report.lines {
                            println!("[elf] {line}");
                        }
                    }
                }
                Err(tsa::TsaError::Refuse(e)) => {
                    eprintln!("[elf] {e}");
                    std::process::exit(3);
                }
                Err(e) => {
                    if !quiet {
                        eprintln!("[elf] error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Autoread { cmd } => {
            let cwd = std::env::current_dir().expect("cwd");
            match cmd {
                // 훅 경로 — 전면 fail-open: 프로젝트 미탐지, stdin 오류, 내부 실패 전부 exit 0 무출력.
                Some(AutoreadCmd::Hook { event }) => {
                    let Some(root) = update::find_project_root(&cwd) else { return };
                    let mut input = String::new();
                    use std::io::Read as _;
                    let _ = std::io::stdin().read_to_string(&mut input);
                    if let Some(digest) = autoread::run_hook(&root, &event, &input) {
                        println!("{digest}");
                    }
                }
                cmd => {
                    let Some(root) = update::find_project_root(&cwd) else {
                        eprintln!(
                            "[elf] error: not an ELF project (no .elf/manifest.json upward from {})",
                            cwd.display()
                        );
                        std::process::exit(1);
                    };
                    // 인수 없음 = 수동 digest (하네스 무관 fallback — 훅 없는 환경, 즉시 재정렬용) + 대기 마커 해제
                    let Some(cmd) = cmd else {
                        print!("{}", autoread::run_manual(&root));
                        return;
                    };
                    let result = match cmd {
                        AutoreadCmd::Enable => autoread::run_enable(&root),
                        AutoreadCmd::Disable => autoread::run_disable(&root),
                        AutoreadCmd::Status => autoread::run_status(&root),
                        AutoreadCmd::Hook { .. } => unreachable!("hook handled above"),
                    };
                    match result {
                        Ok(report) => {
                            for line in &report.lines {
                                println!("[elf] {line}");
                            }
                        }
                        Err(autoread::AutoreadError::Refuse(e)) => {
                            eprintln!("[elf] {e}");
                            std::process::exit(3);
                        }
                        Err(e) => {
                            eprintln!("[elf] error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Deprecate { cmd, target, trial, section, marked, lines, expect, replaced_by, reason, dry_run, restore } => {
            let root = log_root_or_exit();
            if cmd.is_none() && (restore.is_some() || target.is_some()) && !dry_run {
                gate_write_or_exit();
            }
            if legacy::active(&root) {
                let code = match cmd {
                    Some(DeprecateCmd::Show { .. }) => legacy::cli::after_migration("elf deprecate show"),
                    Some(DeprecateCmd::Purge { id, dry_run: dr }) => legacy::cli::deprecate(
                        &root,
                        legacy::cli::DeprecateArgs {
                            purge: Some((id, dr)),
                            list: false,
                            target: None,
                            trial: None,
                            section: None,
                            marked: false,
                            lines: None,
                            expect: None,
                            replaced_by: None,
                            reason: None,
                            dry_run: false,
                            restore: None,
                        },
                    ),
                    Some(DeprecateCmd::List) | None => legacy::cli::deprecate(
                        &root,
                        legacy::cli::DeprecateArgs {
                            purge: None,
                            list: matches!(cmd, Some(DeprecateCmd::List)),
                            target,
                            trial,
                            section,
                            marked,
                            lines,
                            expect,
                            replaced_by,
                            reason,
                            dry_run,
                            restore,
                        },
                    ),
                };
                std::process::exit(code);
            }
            if matches!(cmd, Some(DeprecateCmd::Purge { .. })) {
                eprintln!("[elf] error: `elf deprecate purge` was removed in 2.28 — deprecated content is kept compressed in .elf/deprecated/ (docs/deprecate.md)");
                std::process::exit(2);
            }
            let result = match cmd {
                Some(DeprecateCmd::List) => deprecate::run_list(&root),
                Some(DeprecateCmd::Show { id }) => deprecate::run_show(&root, &id),
                Some(DeprecateCmd::Purge { .. }) => unreachable!("handled above"),
                None => {
                    if let Some(id) = restore {
                        deprecate::run_restore(&root, &id)
                    } else if let Some(target) = target {
                        let lines = match lines.as_deref() {
                            None => None,
                            Some(s) => match s
                                .split_once('-')
                                .and_then(|(a, b)| Some((a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?)))
                            {
                                Some(r) => Some(r),
                                None => {
                                    eprintln!("[elf] error: --lines expects A-B (1-based line numbers), got \"{s}\"");
                                    std::process::exit(2);
                                }
                            },
                        };
                        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                        deprecate::run_deprecate(
                            &root,
                            &deprecate::DeprecateOptions { target, trial, section, marked, lines, expect, replaced_by, reason, dry_run, date },
                        )
                    } else {
                        // 인수 없음 = 목록(읽기전용). 대상 없이 단위 옵션만 준 경우는 usage 오류.
                        let bare = trial.is_none()
                            && section.is_none()
                            && !marked
                            && lines.is_none()
                            && expect.is_none()
                            && replaced_by.is_none()
                            && reason.is_none()
                            && !dry_run;
                        if !bare {
                            eprintln!("[elf] error: a target is required (S### | G### | P### | path), or use --restore <ID>");
                            std::process::exit(2);
                        }
                        deprecate::run_list(&root)
                    }
                }
            };
            match result {
                Ok(rep) => {
                    for l in &rep.lines {
                        println!("[elf] {l}");
                    }
                    if dry_run && !rep.changed {
                        println!("[elf] dry-run — nothing written");
                    } else if rep.changed {
                        println!("[elf] note: commit the document change and its .elf/deprecated/ store file together (LogConvention §6)");
                    }
                }
                Err(deprecate::DeprecateError::Refuse(m)) => {
                    eprintln!("[elf] refuse: {m}");
                    std::process::exit(3);
                }
                Err(deprecate::DeprecateError::NotFound(m)) => {
                    eprintln!("[elf] not found: {m}");
                    std::process::exit(1);
                }
                Err(deprecate::DeprecateError::Io(e)) => {
                    eprintln!("[elf] io error: {e}");
                    std::process::exit(1);
                }
            }
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
        Commands::Validate { check } => {
            let root = log_root_or_exit();
            if legacy::active(&root) {
                std::process::exit(legacy::cli::validate(&root, check));
            }
            match validate::run_validate(&root) {
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
                Err(validate::ValidateError::Io(e)) => {
                    eprintln!("[elf] io error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Session { cmd } => match cmd {
            SessionCmd::New { title, goal, gov } => {
                let root = log_root_or_exit();
                gate_write_or_exit();
                if legacy::active(&root) {
                    if gov {
                        std::process::exit(legacy::cli::after_migration("elf session new --gov"));
                    }
                    std::process::exit(legacy::cli::session_new(&root, title));
                }
                let Some(goal) = goal else {
                    eprintln!("[elf] error: --goal is required: the session goal in 1-2 sentences (elf session new \"<title>\" --goal \"<goal>\")");
                    std::process::exit(2);
                };
                let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let opts = session::SessionNewOptions { title, date, goal, gov };
                match session::run_session_new(&root, &opts) {
                    Ok(res) => {
                        for w in &res.warnings {
                            println!("[elf] warn: {w}");
                        }
                        println!("[elf] created {} ({}) + Registry.md", res.log_rel, res.id);
                        println!(
                            "[elf] note: S = work on its own output; G = a change to rules, structure, templates, or tool settings for all later work (LogConvention §1)"
                        );
                        println!(
                            "[elf] next: write Phase 1 to a file, then `elf trial new \"<title>\" --from <file>` → t01; stop for the user's confirmation before running (LogConvention §5.1)"
                        );
                    }
                    Err(e) => session_exit(e, "elf session new"),
                }
            }
            SessionCmd::Close { id, from, force } => {
                let root = log_root_or_exit();
                gate_write_or_exit();
                if legacy::active(&root) {
                    if from.is_some() {
                        std::process::exit(legacy::cli::after_migration("elf session close --from"));
                    }
                    std::process::exit(legacy::cli::session_close(&root, id, force));
                }
                let date = chrono::Local::now().format("%Y-%m-%d").to_string();
                let body = from.as_ref().map(|p| read_input(p));
                match session::run_session_close(&root, &session::CloseOptions { id, body, date }) {
                    Ok(r) => {
                        for w in &r.warnings {
                            println!("[elf] warn: {w}");
                        }
                        println!("[elf] closed {} ({}) — status complete, Registry.md updated (the file is not moved)", r.id, r.log_rel);
                    }
                    Err(e) => session_exit(e, "elf session close <S###|G###>"),
                }
            }
            SessionCmd::FixHeaders { dry_run } => {
                let root = log_root_or_exit();
                if !legacy::active(&root) {
                    eprintln!("[elf] error: `elf session fix-headers` was removed in 2.28 — log headers are YAML now");
                    std::process::exit(2);
                }
                std::process::exit(legacy::cli::fix_headers(&root, dry_run));
            }
        },
        Commands::Log { cmd } => {
            let root = log_root_or_exit();
            if legacy::active(&root) {
                std::process::exit(legacy::cli::after_migration("elf log"));
            }
            let result = match cmd {
                None => logcmd::run_list(&root),
                Some(LogCmd::Show { id, trial }) => logcmd::run_show(&root, &id, trial.as_deref()),
            };
            match result {
                Ok(lines) => {
                    for l in lines {
                        println!("[elf] {l}");
                    }
                }
                Err(e) => session_exit(e, "elf log show <S###|G###> [tNN]"),
            }
        }
        Commands::Trial { cmd } => {
            let root = log_root_or_exit();
            gate_write_or_exit();
            if legacy::active(&root) {
                let code = match cmd {
                    TrialCmd::New { title, session: sess, from } => {
                        let body = from.as_ref().map(|p| read_input(p));
                        legacy::cli::trial_new(&root, title, sess, body)
                    }
                    TrialCmd::Write { .. } => legacy::cli::after_migration("elf trial write"),
                };
                std::process::exit(code);
            }
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let now = chrono::Local::now().fixed_offset();
            let result = match cmd {
                TrialCmd::New { title, session: sess, from } => {
                    let body = from.as_ref().map(|p| read_input(p));
                    trial::run_trial_new(&root, &trial::TrialNewOptions { title, session: sess, date, body, now })
                }
                TrialCmd::Write { trial: t, session: sess, from } => {
                    let body = read_input(&from);
                    trial::run_trial_write(&root, &trial::TrialWriteOptions { trial: t, session: sess, date, body, now })
                }
            };
            let lang = elf_cli::sections::LogLang::of(&root);
            let sec = |k: &str| elf_cli::sections::name(k, lang).to_string();
            match result {
                Ok(r) => {
                    for w in &r.warnings {
                        println!("[elf] warn: {w}");
                    }
                    for n in &r.normalized {
                        println!("[elf] corrected: {n}");
                    }
                    match r.phase {
                        0 => {
                            println!("[elf] appended {} to {} ({}) — Phase 1 template", r.trial, r.log_rel, r.session);
                            println!(
                                "[elf] next: fill Phase 1 ({}), or write it to a file and run `elf trial write --from <file>`; stop before running (LogConvention §5.1)",
                                elf_cli::sections::names(&trial::PHASE1_REQUIRED, lang)
                            );
                        }
                        1 => {
                            println!(
                                "[elf] {} Phase 1 written to {} ({}) — write {}",
                                r.trial, r.log_rel, r.session, r.writes
                            );
                            println!("[elf] next: stop — show Phase 1 to the user and get confirmation; revise with `elf trial write --from <file>`; run only after approval (LogConvention §5.1)");
                            println!(
                                "[elf] note: list expected figures in {}, and embed each into `### {}` the moment it is created — a table path is not an embed; sub-agent outputs included (LogConvention §2)",
                                sec("예상"),
                                sec("관찰")
                            );
                        }
                        _ => {
                            println!("[elf] {} Phase 2 written to {} ({})", r.trial, r.log_rel, r.session);
                            println!("[elf] note: a rerun or a change that reflects these results is a new trial (LogConvention §2)");
                        }
                    }
                    if r.handoff_replaced {
                        println!("[elf] header handoff replaced from the file's YAML block");
                    }
                }
                Err(e) => session_exit(e, "elf trial new|write --session <S###|G###>"),
            }
        }
    }
}

/// `--from` 파일 읽기(실패 = exit 1).
fn read_input(p: &std::path::Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| {
        eprintln!("[elf] cannot read {}: {e}", p.display());
        std::process::exit(1);
    })
}

/// session, trial, log 명령 공용 오류 출력과 종료 코드(1 = 실행 오류, 3 = 거부).
fn session_exit(e: session::SessionError, usage: &str) -> ! {
    use session::SessionError as E;
    match e {
        E::Exists(p) => {
            eprintln!("[elf] refuse: {p} already exists");
            std::process::exit(3);
        }
        E::BadTitle(m) => {
            eprintln!("[elf] error: {m}");
            std::process::exit(1);
        }
        E::NoOpenSession => {
            eprintln!("[elf] no ongoing session — start one with `elf session new \"<title>\" --goal '<goal>'`");
            std::process::exit(1);
        }
        E::MultipleOpen(ids) => {
            eprintln!("[elf] multiple ongoing sessions ({}) — specify one: {usage}", ids.join(", "));
            std::process::exit(1);
        }
        E::NotFound(id) => {
            eprintln!("[elf] not found: {id}");
            std::process::exit(1);
        }
        E::Rejected(reasons) => {
            eprintln!("[elf] refuse: nothing written (fix all of these, then run again):");
            for x in reasons {
                for (i, l) in x.lines().enumerate() {
                    eprintln!("[elf] {}{l}", if i == 0 { "  " } else { "    " });
                }
            }
            std::process::exit(3);
        }
        E::BadHeader(m) => {
            eprintln!("[elf] refuse: {m} — the header is written only by elf commands; run `elf check logs`");
            std::process::exit(3);
        }
        E::Legacy(m) => {
            eprintln!(
                "[elf] refuse: this project still has the earlier log layout ({}) — run `elf update` to migrate first (LogConvention §7)",
                m.join(", ")
            );
            std::process::exit(3);
        }
        E::Io(x) => {
            eprintln!("[elf] io error: {x}");
            std::process::exit(1);
        }
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

/// 쓰기 명령 관문 — 프로젝트 manifest가 이 CLI보다 높으면 거부(exit 3, P022 §8). 프로젝트 밖(`_dev` 등)은 통과.
fn gate_write_or_exit() {
    let cwd = std::env::current_dir().expect("cwd");
    if let Err(m) = version::gate_write(&cwd) {
        eprintln!("[elf] refuse: {m}");
        std::process::exit(3);
    }
}

/// `elf self list|install|uninstall|prune` — 버전 보관소(v3.0 S044 t58).
fn do_store(cmd: SelfCmd) {
    let res: Result<Vec<String>, String> = match cmd {
        SelfCmd::Update => unreachable!("handled by do_self_update"),
        SelfCmd::List { available } => store::list_lines(available),
        SelfCmd::Install { version } => store::install_spec(&version).map(|i| {
            vec![format!("{} {} → {}", if i.fresh { "installed" } else { "already in the version store:" }, i.version, store::shown(&i.exe))]
        }),
        SelfCmd::Uninstall { version } => match pin::parse(&version) {
            Some(v) => store::uninstall(&v).map(|_| vec![format!("removed {v} from the version store")]),
            None => Err(format!("not a version: {version}")),
        },
        SelfCmd::Prune => store::prune().map(|r| {
            if r.is_empty() {
                vec!["nothing to prune — one version per line".into()]
            } else {
                r.iter().map(|v| format!("removed {v} (a newer patch of line {} is kept)", v.line())).collect()
            }
        }),
    };
    match res {
        Ok(lines) => {
            for l in lines {
                println!("[elf] {l}");
            }
        }
        Err(e) => {
            eprintln!("[elf] error: {e}");
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

fn scope_of(s: CheckScope) -> check::Scope {
    match s {
        CheckScope::Version => check::Scope::Version,
        CheckScope::Setup => check::Scope::Setup,
        CheckScope::Files => check::Scope::Files,
        CheckScope::Logs => check::Scope::Logs,
    }
}

/// `elf check` — 네 범위 요약: 범위별 요약 1줄 + 문제와 경고 줄, 마지막에 합계. 반환 = 종료 코드.
fn print_check_all(cwd: &std::path::Path, env: &doctor::DoctorEnv) -> i32 {
    let (mut issues, mut warnings) = (0, 0);
    // 버전이 맞지 않으면 다른 범위 결과가 틀릴 수 있어 version을 먼저(S044 t41)
    for scope in [check::Scope::Version, check::Scope::Setup, check::Scope::Files, check::Scope::Logs] {
        let r = check::run_scope(cwd, env, scope);
        if let Some(why) = &r.skipped {
            println!("[elf] check {}: skipped — {why}", scope.name());
            continue;
        }
        println!("[elf] check {}: {} issue(s), {} warning(s)", scope.name(), r.issues(), r.warnings());
        for item in r.items.iter().filter(|i| i.is_finding()) {
            println!("[elf]   {}", item.render());
        }
        issues += r.issues();
        warnings += r.warnings();
    }
    println!("[elf] check: {issues} issue(s), {warnings} warning(s)");
    if issues > 0 { 4 } else { 0 }
}

/// `elf check <scope>` — 그 범위의 전체 상세. 적용 불가 범위 = 오류(종료 1).
fn print_check_scope(cwd: &std::path::Path, env: &doctor::DoctorEnv, scope: check::Scope) -> i32 {
    let r = check::run_scope(cwd, env, scope);
    if let Some(why) = &r.skipped {
        eprintln!("[elf] error: check {}: {why}", scope.name());
        return 1;
    }
    for item in &r.items {
        println!("[elf] {}", item.render());
    }
    println!("[elf] check {}: {} issue(s), {} warning(s)", scope.name(), r.issues(), r.warnings());
    if r.issues() > 0 {
        4
    } else {
        0
    }
}
