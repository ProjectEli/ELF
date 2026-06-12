use std::path::Path;

use clap::{Parser, Subcommand};
use elf_cli::{embed, init, status, update};

/// ELF (Eli's Lab Framework) scaffold & update CLI
#[derive(Parser)]
#[command(name = "elf", version = embed::version(), about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 새 ELF 프로젝트 스캐폴드 생성
    Init {
        /// 프로젝트 폴더 이름
        name: String,
        /// 모듈 preset (full | experimental | software | minimal)
        #[arg(long, default_value = "full", conflicts_with = "modules")]
        preset: String,
        /// custom 모듈 선택 (쉼표 구분: hw,fab,sw,exp,paper) — preset 대신 사용
        #[arg(long, value_delimiter = ',')]
        modules: Option<Vec<String>>,
        /// 프로젝트 언어 (AI 에이전트 응답 언어 — .elf/config.json에 기록)
        #[arg(long, default_value = "한국어")]
        lang: String,
    },
    /// 프로젝트의 ELF managed/hybrid 파일을 현 CLI 버전으로 갱신
    /// (주의: CLI 자체 갱신은 self-update — t06)
    Update {
        /// 변경 없이 수행될 작업 목록만 출력
        #[arg(long)]
        dry_run: bool,
        /// 사용자 편집 보호를 무시하고 강제 갱신 (managed 덮어쓰기·hybrid 블록 교체·마커 재삽입)
        #[arg(long)]
        force: bool,
    },
    /// 프로젝트 ELF 파일 상태 진단 (drift/편집/누락 — 읽기전용)
    Status {
        /// 발견(outdated/missing/edited) 시 exit 4 — pre-commit/CI 게이트용
        #[arg(long)]
        check: bool,
    },
    // self-update: t06에서 추가
}

// exit code 규약 (clap 표준 수용, 2026-06-12): 0=성공, 1=실행 오류, 2=usage(clap 기본), 3=refuse.
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            name,
            preset,
            modules,
            lang,
        } => {
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let label = match &modules {
                Some(keys) => format!("custom({})", keys.join(",")),
                None => preset.clone(),
            };
            let opts = init::InitOptions {
                name,
                preset,
                modules,
                lang,
                date,
            };
            match init::run_init(Path::new("."), &opts) {
                Ok(target) => {
                    println!(
                        "[elf] created {} (ELF {}, preset: {}, lang: {})",
                        target.display(),
                        embed::version(),
                        label,
                        opts.lang
                    );
                }
                Err(e @ init::InitError::TargetExists(_)) => {
                    eprintln!("[elf] {e}");
                    std::process::exit(3);
                }
                Err(e) => {
                    eprintln!("[elf] error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Update { dry_run, force } => {
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
    }
}
