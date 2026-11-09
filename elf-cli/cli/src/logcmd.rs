//! `elf log` — 세션 로그 목록과 보기(읽기 전용, P022 §6, S044 t18). 관련 계산 없음.
//!
//! `elf log` = 목록(진행 중 먼저, 이어서 번호 내림차순; 상태와 결론 또는 handoff 한 줄).
//! `elf log show <ID>` = 헤더와 trial 목록(제목, Phase 기록), `elf log show <ID> <tNN>` = 그 trial 본문.

use std::fs;
use std::path::Path;

use crate::header::{self, Header, Status};
use crate::logs::{self, LogFile, Series};
use crate::session::SessionError;
use crate::trial::TrialMeta;

const CELL: usize = 100;

fn cut(s: &str, n: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}

fn title_of(body: &str) -> String {
    body.lines().find(|l| l.starts_with("# ")).map(|l| l.trim_start_matches("# ").to_string()).unwrap_or_default()
}

fn summary(h: &Header) -> &str {
    match h.status {
        Status::Ongoing => &h.handoff,
        _ if h.conclusion.is_empty() => &h.handoff,
        _ => &h.conclusion,
    }
}

/// `elf log`: 목록 줄(머리 없이). 이전 형식이면 SessionError::Legacy.
pub fn run_list(root: &Path) -> Result<Vec<String>, SessionError> {
    crate::session::refuse_legacy(root)?;
    let mut rows: Vec<(LogFile, Header, String)> = Vec::new();
    let mut bad = Vec::new();
    for l in logs::scan_all(root) {
        let text = fs::read_to_string(&l.path)?;
        match header::parse(&text) {
            Ok((h, at)) => {
                let t = title_of(&text[at..]);
                rows.push((l, h, t));
            }
            Err(e) => bad.push(format!("{}: header line {}: {} — run `elf check logs`", logs::rel_of(&l), e.line, e.message)),
        }
    }
    rows.sort_by(|a, b| {
        let oa = a.1.status != Status::Ongoing;
        let ob = b.1.status != Status::Ongoing;
        (oa, a.0.series == Series::S, std::cmp::Reverse(a.0.num)).cmp(&(ob, b.0.series == Series::S, std::cmp::Reverse(b.0.num)))
    });
    let mut out: Vec<String> = rows
        .iter()
        .map(|(l, h, t)| {
            let s = summary(h);
            let tail = if s.is_empty() { String::new() } else { format!(" — {}", cut(s, CELL)) };
            format!("{} · {} · {} · {}{tail}", l.id, h.status.as_str(), h.modified, cut(t.split_once(": ").map_or(t.as_str(), |x| x.1), 60))
        })
        .collect();
    out.extend(bad.into_iter().map(|b| format!("warn: {b}")));
    if out.is_empty() {
        out.push("no session logs yet — start one with `elf session new \"<title>\" --goal '<goal>'`".into());
    }
    Ok(out)
}

/// `elf log show <ID> [tNN]`.
pub fn run_show(root: &Path, id: &str, trial: Option<&str>) -> Result<Vec<String>, SessionError> {
    crate::session::refuse_legacy(root)?;
    let log = logs::resolve(root, Some(id))?;
    let (h, text) = logs::read(&log)?;
    let (_, at) = header::parse(&text).map_err(|e| SessionError::BadHeader(e.to_string()))?;
    let body = &text[at..];
    let lines: Vec<&str> = body.split('\n').collect();
    let heads: Vec<usize> = (0..lines.len()).filter(|&i| lines[i].starts_with("## t")).collect();
    if let Some(t) = trial {
        let n: u32 = t.trim_start_matches('t').parse().map_err(|_| SessionError::NotFound(format!("{id} {t}")))?;
        let want = format!("## t{n:02}:");
        let Some(k) = heads.iter().position(|&i| lines[i].starts_with(&want)) else {
            return Err(SessionError::NotFound(format!("{id} t{n:02}")));
        };
        let end = heads.get(k + 1).copied().unwrap_or(lines.len());
        let mut out: Vec<String> = lines[heads[k]..end].iter().map(|s| s.to_string()).collect();
        while out.last().is_some_and(|l| l.trim().is_empty() || l.trim() == "---") {
            out.pop();
        }
        return Ok(out);
    }
    let mut out = vec![
        format!("{} · {}", log.id, title_of(body).split_once(": ").map_or(String::new(), |x| x.1.to_string())),
        format!("path: {}", logs::rel_of(&log)),
        format!("status: {} · created {} · modified {}", h.status.as_str(), h.created, h.modified),
        format!("goal: {}", h.goal),
    ];
    for (k, v) in [("handoff", &h.handoff), ("conclusion", &h.conclusion), ("user_note", &h.user_note)] {
        if !v.is_empty() {
            out.push(format!("{k}: {v}"));
        }
    }
    out.push(format!("trials: {}", heads.len()));
    for &i in &heads {
        let title = lines[i].trim_start_matches("## ").trim();
        let meta = lines.get(i + 1).and_then(|l| TrialMeta::parse(l));
        let phase = match &meta {
            Some(m) if m.p2.is_some() => format!(" · Phase 2 ({} Phase 1 write(s))", m.writes),
            Some(m) if m.writes > 0 => format!(" · Phase 1 ({} write(s))", m.writes),
            Some(_) => " · Phase 1 template".to_string(),
            None if lines.get(i + 1).is_some_and(|l| l.starts_with(crate::deprecate::MARKER_PREFIX)) => " · deprecated".to_string(),
            None => String::new(),
        };
        out.push(format!("  {}{phase}", cut(title, 90)));
    }
    out.push(format!("body: `elf log show {} tNN`", log.id));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionNewOptions, run_session_new};

    #[test]
    fn list_orders_ongoing_first_and_show_lists_trials() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("2_Log")).unwrap();
        for (t, gov) in [("첫", false), ("둘", false), ("규칙", true)] {
            run_session_new(root, &SessionNewOptions { title: t.into(), date: "2026-11-09".into(), goal: format!("{t} 목표"), gov }).unwrap();
        }
        let p = root.join("2_Log/S/S001_log.md");
        let c = fs::read_to_string(&p).unwrap().replace("status: ongoing", "status: complete").replace("conclusion: ''", "conclusion: '결론 1'");
        fs::write(&p, format!("{c}\n## t01: 시도\n<!-- elf-trial: phase1 2026-11-09T10:00:00+09:00 ~ 2026-11-09T10:00:00+09:00 (1 writes); phase2 2026-11-09T11:00:00+09:00 -->\n\n### 목표 (Goal)\n- x\n\n---\n\n## t02: (폐기 S001-D01)\n> **Deprecated**: S001-D01 · 2026-11-09 · t02\n")).unwrap();
        let l = run_list(root).unwrap();
        assert!(l[0].starts_with("G001 · ongoing") && l[1].starts_with("S002 · ongoing") && l[2].starts_with("S001 · complete"), "{l:?}");
        assert!(l[2].ends_with("— 결론 1"), "{l:?}");
        let s = run_show(root, "S001", None).unwrap();
        assert!(s.iter().any(|x| x == "trials: 2"), "{s:?}");
        assert!(s.iter().any(|x| x == "  t01: 시도 · Phase 2 (1 Phase 1 write(s))"), "{s:?}");
        assert!(s.iter().any(|x| x == "  t02: (폐기 S001-D01) · deprecated"), "{s:?}");
        let t = run_show(root, "S001", Some("t1")).unwrap();
        assert_eq!(t.first().map(String::as_str), Some("## t01: 시도"));
        assert_eq!(t.last().map(String::as_str), Some("- x"));
        assert!(matches!(run_show(root, "S001", Some("t09")), Err(SessionError::NotFound(_))));
    }
}
