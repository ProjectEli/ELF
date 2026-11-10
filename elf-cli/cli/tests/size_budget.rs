//! 배포 파일 크기 상한(S052 t01) — 상한 = Claude Code 기본 한도(올리지 않음). 넘으면 실패해 발행 관문
//! (`release.sh`의 cargo test)에서 걸림. 한도 아래에서 커지는 것은 잡지 않으므로 실측 표를 출력함
//! (`cargo test --test size_budget -- --nocapture`) — 발행 때 줄이기 검토의 기준표.
//!
//! 한도(Claude Code 공식 문서, 2026-11-10 확인):
//! - 매 세션 적재 파일(CLAUDE.md와 import한 AGENTS.md): 파일마다 200줄(memory 문서 "target under 200 lines").
//! - autoread digest: 10,000자(hooks 문서 Output limits — 넘으면 앞 2,000자만 전달, `autoread::DIGEST_CAP`).
//! - 필요할 때 읽는 관리 규칙 문서: 파일마다 40,000자(memory 문서, 필요할 때 적재하는 파일의 파일당 한도, 200K context).

use elf_cli::autoread;
use elf_cli::init::{InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const SESSION_LINES: usize = 200;
const DIGEST_CHARS: usize = 10_000;
const ON_DEMAND_CHARS: usize = 40_000;
const MANAGED_DOCS: [&str; 5] = ["EliRule", "LogConvention", "AI_PARA_Framework", "LLMcliche", "highIFjournals"];

fn project(tmp: &Path, preset: &str, lang: &str) -> PathBuf {
    let o = InitOptions {
        name: format!("{preset}_{lang}"),
        preset: preset.into(),
        modules: None,
        categories: Vec::new(),
        lang: lang.into(),
        date: "2026-11-10".into(),
    };
    run_init(tmp, &o).unwrap()
}

/// (프로젝트, 파일, 실측, 한도, 단위)
type Row = (String, String, usize, usize, &'static str);

fn report(title: &str, rows: &[Row]) -> Vec<String> {
    println!("\n{title}");
    println!("{:<12} {:<44} {:>8} {:>8} {:>6}", "project", "file", "size", "limit", "ratio");
    let mut over = Vec::new();
    for (p, f, n, limit, unit) in rows {
        println!("{p:<12} {f:<44} {n:>8} {limit:>8} {:>5}% {unit}", n * 100 / limit);
        if n > limit {
            over.push(format!(
                "{p}: {f} = {n} {unit} > {limit} (Claude Code default limit; shrink the file, do not raise the limit)"
            ));
        }
    }
    over
}

fn projects() -> (tempfile::TempDir, Vec<(String, PathBuf)>) {
    let tmp = tempdir().unwrap();
    let mut v = Vec::new();
    for preset in ["full", "qa"] {
        for lang in ["ko-KR", "en-US"] {
            let dir = tmp.path().join(format!("{preset}_{lang}"));
            fs::create_dir_all(&dir).unwrap();
            v.push((format!("{preset} {}", &lang[..2]), project(&dir, preset, lang)));
        }
    }
    (tmp, v)
}

#[test]
fn session_loaded_files_stay_within_claude_line_limit() {
    let (_tmp, ps) = projects();
    let mut rows: Vec<Row> = Vec::new();
    for (name, root) in &ps {
        for f in ["AGENTS.md", "CLAUDE.md"] {
            let t = fs::read_to_string(root.join(f)).unwrap();
            rows.push((name.clone(), f.into(), t.lines().count(), SESSION_LINES, "lines"));
            // 참고(한도 아님): 글자 수 — 한 줄이 긴 파일은 줄 수 한도가 느슨하므로 실질 관리는 이 값으로
            println!("{name:<12} {f:<44} {:>8} chars", t.chars().count());
        }
    }
    let over = report("session-loaded files (limit: 200 lines per file)", &rows);
    assert!(over.is_empty(), "{}", over.join("\n"));
}

#[test]
fn autoread_digest_stays_within_hook_output_limit() {
    let (_tmp, ps) = projects();
    let mut rows: Vec<Row> = Vec::new();
    for (name, root) in &ps {
        // 상시 의무 포함(claude_code = false) = 가장 큰 경우. 새 프로젝트라 진행 중 세션, 갱신 안내 없음
        let d = autoread::build_digest_for(root, "startup", false);
        rows.push((name.clone(), "autoread digest".into(), d.chars().count(), DIGEST_CHARS, "chars"));
    }
    assert_eq!(DIGEST_CHARS, autoread::DIGEST_CAP, "the digest cap follows the Claude Code hook output limit");
    let over = report("autoread digest (limit: 10,000 chars, hook output)", &rows);
    assert!(over.is_empty(), "{}", over.join("\n"));
}

#[test]
fn managed_rule_documents_stay_within_on_demand_limit() {
    let (_tmp, ps) = projects();
    let mut rows: Vec<Row> = Vec::new();
    for (name, root) in ps.iter().filter(|(n, _)| n.starts_with("full")) {
        for d in MANAGED_DOCS {
            let t = fs::read_to_string(root.join(format!(".elf/managed/{d}.md"))).unwrap();
            rows.push((name.clone(), format!(".elf/managed/{d}.md"), t.chars().count(), ON_DEMAND_CHARS, "chars"));
        }
    }
    let over = report("managed rule documents (limit: 40,000 chars per file, on-demand)", &rows);
    assert!(over.is_empty(), "{}", over.join("\n"));
}
