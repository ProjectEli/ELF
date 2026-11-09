//! [동결] ELF v2.27.4 `main.rs`의 로그 명령 처리(출력과 종료 코드 그대로) — 이전 형식 저장소 전용.
//! 반환 = 종료 코드. 쓰기 명령이 성공하면 끝에 이관 안내 1줄(v2.28.1 추가).

use std::path::Path;

use super::{NOTICE, deprecate, session, trial, validate};

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn notice() {
    println!("[elf] note: {NOTICE}");
}

pub fn session_new(root: &Path, title: String) -> i32 {
    let opts = session::SessionNewOptions { title, date: today() };
    match session::run_session_new(root, &opts) {
        Ok(res) => {
            for w in &res.warnings {
                println!("[elf] warn: {w}");
            }
            println!("[elf] created {} ({}) + registry row", res.log_rel, res.id);
            println!("[elf] next: elf trial new \"<title>\" → t01 — the log holds the header only; fill Phase 1, then stop before execution (LogConvention §5.1)");
            notice();
            0
        }
        Err(session::SessionError::Escalation(e)) => {
            eprintln!("[elf] {e}");
            5
        }
        Err(session::SessionError::Exists(p)) => {
            eprintln!("[elf] refuse: {p} already exists");
            3
        }
        Err(session::SessionError::BadTitle(m)) => {
            eprintln!("[elf] error: {m}");
            1
        }
        Err(session::SessionError::Io(e)) => {
            eprintln!("[elf] io error: {e}");
            1
        }
        Err(e) => {
            eprintln!("[elf] error: {e:?}");
            1
        }
    }
}

pub fn session_close(root: &Path, id: Option<String>, force: bool) -> i32 {
    match session::run_session_close(root, &session::CloseOptions { id, force }) {
        Ok(r) => {
            for w in &r.warnings {
                println!("[elf] warn: {w}");
            }
            println!("[elf] closed {} → {} (Status: Complete, registry updated)", r.id, r.archived_to);
            println!(
                "[elf] note: rewrite the registry key finding as the session's final conclusion, replacing the previous text (LogConvention §5.2)"
            );
            notice();
            0
        }
        Err(session::SessionError::Escalation(e)) => {
            eprintln!("[elf] {e}");
            5
        }
        Err(session::SessionError::NoOpenSession) => {
            eprintln!("[elf] nothing to close: no open session in 2_Log/ (all Complete)");
            1
        }
        Err(session::SessionError::MultipleOpen(ids)) => {
            eprintln!("[elf] multiple open sessions ({}) — specify one: elf session close <S###>", ids.join(", "));
            1
        }
        Err(session::SessionError::NotFound(id)) => {
            eprintln!("[elf] session not found: {id}");
            1
        }
        Err(session::SessionError::Exists(p)) => {
            eprintln!("[elf] refuse: {p} already exists");
            3
        }
        Err(session::SessionError::BadTitle(m)) => {
            eprintln!("[elf] error: {m}");
            1
        }
        Err(session::SessionError::Io(e)) => {
            eprintln!("[elf] io error: {e}");
            1
        }
        Err(e) => {
            eprintln!("[elf] error: {e:?}");
            1
        }
    }
}

pub fn fix_headers(root: &Path, dry_run: bool) -> i32 {
    match session::run_fix_headers(root, dry_run) {
        Ok(files) => {
            for f in &files {
                println!("[elf] {} {}", if dry_run { "would fix" } else { "fixed" }, f.path);
            }
            println!("[elf] {} file(s) {}", files.len(), if dry_run { "would change (dry-run)" } else { "changed" });
            0
        }
        Err(e) => {
            eprintln!("[elf] io error: {e}");
            1
        }
    }
}

pub fn trial_new(root: &Path, title: Option<String>, sess: Option<String>, body: Option<String>) -> i32 {
    let from_file = body.is_some();
    let opts = trial::TrialNewOptions { title, session: sess, date: today(), body };
    match trial::run_trial_new(root, &opts) {
        Ok(r) => {
            println!("[elf] appended {} to {} ({}){}", r.trial, r.log_rel, r.session, if from_file { ", from file" } else { "" });
            if r.handoff_replaced {
                println!("[elf] header Handoff replaced with the first line of the file");
            }
            if !from_file {
                println!("[elf] next: Phase 1 — fill 목표/조건/가설/예상, then stop before execution (LogConvention §5.1)");
            } else if !r.filled_from_template.is_empty() {
                println!(
                    "[elf] next: Phase 2 sections ({}) hold the template text — stop before execution, fill them afterwards (LogConvention §5.1)",
                    r.filled_from_template.join(", ")
                );
            }
            println!(
                "[elf] note: list expected figures in 예상, and embed each into `### 관찰` the moment it is created — a table path is not an embed; sub-agent outputs included (LogConvention §2)"
            );
            println!("[elf] note: keep the header Handoff a replace-style summary (state; pending; refs) — do not append history");
            notice();
            0
        }
        Err(session::SessionError::NoOpenSession) => {
            eprintln!("[elf] no open session in 2_Log/ — start one with `elf session new \"<title>\"`");
            1
        }
        Err(session::SessionError::MultipleOpen(ids)) => {
            eprintln!("[elf] multiple open sessions ({}) — specify one: elf trial new --session <S###>", ids.join(", "));
            1
        }
        Err(session::SessionError::NotFound(id)) => {
            eprintln!("[elf] session not found: {id}");
            1
        }
        Err(session::SessionError::Rejected(reasons)) => {
            eprintln!("[elf] refuse: trial not appended (the log is unchanged):");
            for x in reasons {
                eprintln!("[elf]   {x}");
            }
            3
        }
        Err(session::SessionError::Io(e)) => {
            eprintln!("[elf] io error: {e}");
            1
        }
        Err(e) => {
            eprintln!("[elf] error: {e:?}");
            1
        }
    }
}

/// `elf deprecate` 인수(2.27.4 표면).
pub struct DeprecateArgs {
    /// `purge <ID> [--dry-run]`
    pub purge: Option<(String, bool)>,
    /// `list`(숨김 별칭) 또는 인수 없음
    pub list: bool,
    pub target: Option<String>,
    pub trial: Option<String>,
    pub section: Option<String>,
    pub marked: bool,
    pub lines: Option<String>,
    pub expect: Option<String>,
    pub replaced_by: Option<String>,
    pub reason: Option<String>,
    pub dry_run: bool,
    pub restore: Option<String>,
}

pub fn deprecate(root: &Path, a: DeprecateArgs) -> i32 {
    let purge_dry = matches!(&a.purge, Some((_, true)));
    let dry_run = a.dry_run;
    let result = if let Some((id, dr)) = a.purge {
        deprecate::run_purge(root, &id, dr, &today())
    } else if a.list {
        deprecate::run_list(root)
    } else if let Some(id) = a.restore {
        deprecate::run_restore(root, &id)
    } else {
        let Some(target) = a.target else {
            let bare = a.trial.is_none()
                && a.section.is_none()
                && !a.marked
                && a.lines.is_none()
                && a.expect.is_none()
                && a.replaced_by.is_none()
                && a.reason.is_none()
                && !a.dry_run;
            if !bare {
                eprintln!("[elf] error: a target is required (S### | P### | path), or use --restore <ID>");
                return 2;
            }
            return match deprecate::run_list(root) {
                Ok(rep) => {
                    for l in &rep.lines {
                        println!("[elf] {l}");
                    }
                    0
                }
                Err(e) => {
                    eprintln!("[elf] error: {e:?}");
                    1
                }
            };
        };
        let lines = match a.lines.as_deref() {
            None => None,
            Some(s) => match s.split_once('-').and_then(|(x, y)| Some((x.trim().parse::<usize>().ok()?, y.trim().parse::<usize>().ok()?))) {
                Some(r) => Some(r),
                None => {
                    eprintln!("[elf] error: --lines expects A-B (1-based line numbers), got \"{s}\"");
                    return 2;
                }
            },
        };
        deprecate::run_deprecate(
            root,
            &deprecate::DeprecateOptions {
                target,
                trial: a.trial,
                section: a.section,
                marked: a.marked,
                lines,
                expect: a.expect,
                replaced_by: a.replaced_by,
                reason: a.reason,
                dry_run,
                date: today(),
            },
        )
    };
    match result {
        Ok(rep) => {
            for l in &rep.lines {
                println!("[elf] {l}");
            }
            if (dry_run || purge_dry) && !rep.changed {
                println!("[elf] dry-run — nothing written");
            }
            if rep.changed {
                notice();
            }
            0
        }
        Err(deprecate::DeprecateError::Refuse(m)) => {
            eprintln!("[elf] refuse: {m}");
            3
        }
        Err(deprecate::DeprecateError::NotFound(m)) => {
            eprintln!("[elf] not found: {m}");
            1
        }
        Err(deprecate::DeprecateError::Escalation(e)) => {
            eprintln!("[elf] {e}");
            5
        }
        Err(deprecate::DeprecateError::Io(e)) => {
            eprintln!("[elf] io error: {e}");
            1
        }
    }
}

/// 구 이름 `elf validate [--check]`.
pub fn validate(root: &Path, check: bool) -> i32 {
    match validate::run_validate(root) {
        Ok(report) => {
            for line in &report.lines {
                println!("[elf] {line}");
            }
            println!("[elf] validate: {} issue(s), {} warning(s)", report.issues, report.warnings);
            if check && report.findings() > 0 { 4 } else { 0 }
        }
        Err(validate::ValidateError::Escalation(e)) => {
            eprintln!("[elf] {e}");
            5
        }
        Err(validate::ValidateError::Io(e)) => {
            eprintln!("[elf] io error: {e}");
            1
        }
    }
}

/// `elf check logs`용 — 2.27.4 검사 줄(접두 `issue: `/`warn: `/`ok: `/`hint: `)과 이관 안내 info.
/// Registry 파싱 불가(2.27의 exit 5)는 issue 줄로 바꿔 넣음(check 범위는 issue = exit 4).
pub fn check_logs_lines(root: &Path) -> Vec<String> {
    let mut out = match validate::run_validate(root) {
        Ok(r) => r.lines,
        Err(validate::ValidateError::Escalation(e)) => vec![format!("issue: {}", e.to_string().replace('\n', " "))],
        Err(validate::ValidateError::Io(e)) => vec![format!("issue: io error: {e}")],
    };
    out.push(format!("info: {NOTICE}"));
    out
}

/// 새 구조 전용 명령을 이전 형식 저장소에서 부를 때(exit 3).
pub fn after_migration(what: &str) -> i32 {
    eprintln!(
        "[elf] refuse: `{what}` is available after this project is migrated to the 2.28 log layout — run `elf update --dry-run` to preview the migration (LogConvention §7); until then the 2.27 commands keep working"
    );
    3
}
