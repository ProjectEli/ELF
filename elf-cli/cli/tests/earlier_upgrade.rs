//! 어느 2.x에서든 3.x로 — 실제 릴리즈로 만든 프로젝트(시험 자료 `tests/fixtures/earlier/<버전>.tar.xz`,
//! `tools/earlier_fixtures.py`)를 `elf update`(요약과 확인 코드 → 승인)로 올림(v3.0.2, S046).
//! 시대: generator(v2.0 로그 `5_Exp/53_Analysis/Logs/`, v2.1 `7_Log/`, v2.3 `2_Log/1_Wiki`, `9_Archive`),
//! `.elf/` 없는 v2.4.0(init이 도중에 멈춤), 2.15 이전 배치(v2.4.5, v2.14.0), 안 쓰는 ELF 파일(v2.20.0), 2.27 마지막(v2.27.4).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const LEGACY: [&str; 9] = [
    "0_Meta/EliRule.md",
    "0_Meta/LogConvention.md",
    "0_Meta/AI_PARA_Framework.md",
    "0_Meta/highIFjournals.md",
    "0_Meta/LLMcliche.md",
    "0_Meta/scripts",
    "templates",
    ".claudeignore",
    ".elf/managed/templates/Archive",
];

fn git(root: &Path, args: &[&str]) {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().expect("git runs");
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn elf(root: &Path, home: &Path, args: &[&str]) -> (i32, String) {
    let o = assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .env("ELF_DISPATCH", "off")
        .env("ELF_HOME", home)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

/// ustar + xz 시험 자료를 `to`에 풂(일반 파일만).
fn extract(archive: &Path, to: &Path) {
    let data = fs::read(archive).unwrap();
    let mut tar = Vec::new();
    lzma_rust2::XzReader::new(&data[..], false).read_to_end(&mut tar).unwrap();
    let field = |b: &[u8]| String::from_utf8_lossy(&b[..b.iter().position(|c| *c == 0).unwrap_or(b.len())]).into_owned();
    let mut pos = 0;
    while pos + 512 <= tar.len() {
        let h = &tar[pos..pos + 512];
        if h.iter().all(|b| *b == 0) {
            break;
        }
        let (name, prefix) = (field(&h[0..100]), field(&h[345..500]));
        let size = usize::from_str_radix(field(&h[124..136]).trim(), 8).unwrap();
        let path = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
        pos += 512;
        if h[156] == b'0' || h[156] == 0 {
            let out = to.join(&path);
            fs::create_dir_all(out.parent().unwrap()).unwrap();
            fs::write(&out, &tar[pos..pos + size]).unwrap();
        }
        pos += size.div_ceil(512) * 512;
    }
}

fn logs_under(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            let n = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() {
                if n != ".git" && n != ".elf" {
                    stack.push(p);
                }
            } else if n.len() == 10 && n.starts_with('S') && n.ends_with("_log.md") {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

struct Upgraded {
    _tmp: TempDir,
    root: PathBuf,
    home: PathBuf,
    logs_before: Vec<String>,
    summary: String,
    applied: String,
}

/// 시험 자료를 풀고 git 커밋 → `elf update`(exit 3, 요약과 코드) → `--approve`(exit 0).
fn upgrade(tag: &str) -> Upgraded {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("proj");
    let home = tmp.path().join("elfhome");
    fs::create_dir_all(&root).unwrap();
    extract(&Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/earlier/{tag}.tar.xz")), &root);
    git(&root, &["init", "-q"]);
    git(&root, &["-c", "core.autocrlf=false", "add", "-A"]);
    git(&root, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "base"]);
    let logs_before = logs_under(&root);
    let (code, summary) = elf(&root, &home, &["update"]);
    assert_eq!(code, 3, "{tag}: the first run prints the summary only: {summary}");
    let line = cli_line();
    assert!(summary.contains(&format!("update needs approval: project line 2.27 → {line}")), "{tag}: {summary}");
    let c = summary.lines().find_map(|l| l.split("approval code: ").nth(1)).map(|s| s.trim().to_string()).unwrap_or_else(|| panic!("{tag}: {summary}"));
    let (code, applied) = elf(&root, &home, &["update", "--to", &line, "--approve", &c]);
    assert_eq!(code, 0, "{tag}: {applied}");
    Upgraded { _tmp: tmp, root, home, logs_before, summary, applied }
}

/// 공통 결과: `.elf/`(config, manifest = 이 CLI 버전), 이전 배치 파일 없음, 로그 수 보존(모두 `2_Log/S/`), 판정 = 같은 줄.
fn assert_upgraded(tag: &str, u: &Upgraded) {
    let r = &u.root;
    let m = fs::read_to_string(r.join(".elf/manifest.json")).unwrap_or_else(|_| panic!("{tag}: no manifest: {}", u.applied));
    assert!(m.contains(&format!("\"elf_version\": \"{}\"", elf_cli::embed::version())), "{tag}: {m}");
    assert!(r.join(".elf/config.json").is_file(), "{tag}");
    for l in LEGACY {
        assert!(!r.join(l).exists(), "{tag}: {l} remains\n{}", u.applied);
    }
    let after = logs_under(r);
    assert_eq!(after.len(), u.logs_before.len(), "{tag}: {:?} → {after:?}", u.logs_before);
    assert!(after.iter().all(|l| l.starts_with("2_Log/S/")), "{tag}: {after:?}");
    assert!(logs_under(r).iter().all(|l| !l.contains("Session_Registry")), "{tag}");
    for old in ["7_Log", "5_Exp/53_Analysis/Logs", "2_Log/9_Archive", "2_Log/1_Wiki", "2_Log/2_Wiki", "2_Log/Archive"] {
        assert!(!r.join(old).exists(), "{tag}: {old}/ remains");
    }
    let (_, v) = elf(r, &u.home, &["check", "version"]);
    assert!(v.contains(&format!("same line {}", cli_line())), "{tag}: {v}");
}

/// 이 CLI의 줄(MAJOR.MINOR) — 올라가는 목표 줄.
fn cli_line() -> String {
    elf_cli::pin::parse(elf_cli::embed::version()).unwrap().line().to_string()
}

#[test]
fn generator_v2_0_logs_under_exp_analysis() {
    let u = upgrade("v2.0");
    assert!(u.summary.contains("project: none recorded (ELF 2.0–2.4.0 layout, no .elf/)"), "{}", u.summary);
    assert!(u.summary.contains("would adopt: ELF 2.0–2.4.0 project without .elf/"), "{}", u.summary);
    assert_upgraded("v2.0", &u);
    // 보관 폴더(9_Archive)의 닫은 세션 = TSV의 Complete와 Key Finding
    let s2 = fs::read_to_string(u.root.join("2_Log/S/S002_log.md")).unwrap();
    assert!(s2.contains("status: complete") && s2.contains("conclusion: '측정값 확인'"), "{s2}");
    assert!(s2.contains("[S001](S001_log.md)"), "link recomputed: {s2}");
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
    let cfg = fs::read_to_string(u.root.join(".elf/config.json")).unwrap();
    assert!(cfg.contains("\"lang\": \"ko\"") && cfg.contains("\"preset\": \"full\""), "{cfg}");
}

#[test]
fn generator_v2_1_logs_in_7_log() {
    let u = upgrade("v2.1");
    assert_upgraded("v2.1", &u);
    assert!(u.applied.contains("updated (unchanged since delivered): .gitignore"), "{}", u.applied);
    assert!(fs::read_to_string(u.root.join(".gitignore")).unwrap().contains("# >>> ELF managed >>>"));
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
}

#[test]
fn generator_v2_3_with_user_edits() {
    let u = upgrade("v2.3");
    assert_upgraded("v2.3", &u);
    // 고친 EliRule = 옮긴 뒤 그대로 + 새 판은 .elf-new
    let eli = fs::read_to_string(u.root.join(".elf/managed/EliRule.md")).unwrap();
    assert!(eli.contains("사용자 추가 규칙: 측정 단위는 SI"), "edited rule kept");
    assert!(u.root.join(".elf/managed/EliRule.md.elf-new").is_file(), "{}", u.applied);
    // 고친 .gitignore(표지 없는 판) = 사용자 줄 보존 + 상단에 ELF 블록
    let gi = fs::read_to_string(u.root.join(".gitignore")).unwrap();
    assert!(gi.starts_with("# >>> ELF managed >>>") && gi.contains("my_local/"), "{gi}");
    let s2 = fs::read_to_string(u.root.join("2_Log/S/S002_log.md")).unwrap();
    assert!(s2.contains("status: complete"), "{s2}");
    // 남은 일 = .elf-new 병합(check files가 알림)
    let (_, out) = elf(&u.root, &u.home, &["check", "files"]);
    assert!(out.contains("EliRule.md"), "{out}");
}

#[test]
fn v2_4_0_without_elf_folder() {
    let u = upgrade("v2.4.0");
    assert_upgraded("v2.4.0", &u);
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
}

#[test]
fn pre_2_15_layout_rules_move_to_managed() {
    let u = upgrade("v2.4.5");
    assert!(u.summary.contains("would move (earlier layout): 0_Meta/EliRule.md → .elf/managed/EliRule.md"), "{}", u.summary);
    assert!(!u.summary.contains("v2.15.1"), "no manual detour: {}", u.summary);
    assert_upgraded("v2.4.5", &u);
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
}

#[test]
fn pre_2_15_layout_with_edited_rule() {
    let u = upgrade("v2.14.0");
    assert_upgraded("v2.14.0", &u);
    let lc = fs::read_to_string(u.root.join(".elf/managed/LogConvention.md")).unwrap();
    assert!(lc.contains("사용자 추가 규칙: trial 제목에 날짜"));
    assert!(u.root.join(".elf/managed/LogConvention.md.elf-new").is_file(), "{}", u.applied);
}

#[test]
fn unused_elf_files_removed_when_unchanged() {
    let u = upgrade("v2.20.0");
    assert!(u.applied.contains("removed (no longer used by ELF, unchanged since delivered): .claudeignore"), "{}", u.applied);
    assert_upgraded("v2.20.0", &u);
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
}

#[test]
fn last_2_27_release() {
    let u = upgrade("v2.27.4");
    assert_upgraded("v2.27.4", &u);
    let (code, out) = elf(&u.root, &u.home, &["check"]);
    assert!(code == 0, "{out}");
}
