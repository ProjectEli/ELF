//! 줄을 바꾸는 `elf update`의 목표(`--to`)와 사용자 동의(v3.0 S044 t60 C안): 요약과 확인 코드(아무것도 쓰지 않음),
//! 코드가 계획에 묶임, 같은 줄은 바로 적용, 낮은 줄 거부, 다른 줄은 목표 버전으로 넘김(3.0 이전에는 `--to` 빼고).

use elf_cli::init::{InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn git(root: &Path, args: &[&str]) {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().expect("git runs");
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn commit(root: &Path) {
    git(root, &["add", "-A"]);
    git(root, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-qm", "x"]);
}

fn elf_env(root: &Path, env: &[(&str, &str)], args: &[&str]) -> (i32, String, String) {
    let mut c = assert_cmd::Command::cargo_bin("elf").unwrap();
    c.args(args).current_dir(root).env_remove("ELF_DISPATCHED");
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

fn elf(root: &Path, args: &[&str]) -> (i32, String, String) {
    elf_env(root, &[("ELF_DISPATCH", "off")], args)
}

fn code_of(out: &str) -> String {
    out.lines().find_map(|l| l.split("approval code: ").nth(1)).map(|s| s.trim().to_string()).unwrap_or_else(|| panic!("no code: {out}"))
}

fn new_project(dir: &Path) -> PathBuf {
    run_init(dir, &InitOptions { name: "P".into(), preset: "minimal".into(), modules: None, categories: Vec::new(), lang: "ko-KR".into(), date: "2026-11-10".into() })
        .unwrap()
}

/// 2.27 형식 프로젝트(manifest schema 1, 루트 로그 1개), git 커밋됨.
fn old_project(dir: &Path) -> PathBuf {
    let root = new_project(dir);
    fs::write(root.join(".elf/manifest.json"), elf_cli::embed::MANIFEST_JSON.replace("elf-manifest/2", "elf-manifest/1")).unwrap();
    fs::write(root.join(".elf/version"), "v2.27.4\n").unwrap();
    fs::remove_dir_all(root.join("2_Log/S")).unwrap();
    fs::write(
        root.join("2_Log/S001_log.md"),
        "# S001: 제목\n\n> **Created**: 2026-06-01\\\n> **Status**: ★ 활성\\\n> **목표**: g\\\n> **Handoff**: -\n\n---\n\n## t01: a\n",
    )
    .unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "core.autocrlf", "false"]);
    commit(&root);
    root
}

#[test]
fn line_change_prints_a_summary_and_applies_only_with_the_matching_code() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    // dry-run = 계획과 동의 안내, 쓰지 않음
    let (code, out, _) = elf(&root, &["update", "--dry-run"]);
    assert!(code == 0 && out.contains("needs the user's approval"), "{out}");
    // 요약과 코드, 아무것도 쓰지 않음
    let (code, out, _) = elf(&root, &["update"]);
    assert_eq!(code, 3, "{out}");
    let l = elf_cli::pin::parse(elf_cli::embed::version()).unwrap().line().to_string(); // 이 CLI의 줄
    assert!(out.contains(&format!("update needs approval: project line 2.27 → {l} (structure generation 1 → 2: the log layout is migrated)")), "{out}");
    assert!(out.contains("project: none recorded (earlier format, manifest schema 1)") && out.contains("logs: 1 moved"), "{out}");
    assert!(out.contains("AI agent: show this summary to the user") && out.contains(&format!("--to {l} --approve ")), "{out}");
    let c1 = code_of(&out);
    assert_eq!(c1.len(), 6);
    assert!(!root.join("2_Log/S").exists() && root.join("2_Log/S001_log.md").is_file(), "nothing written");
    assert_eq!(code_of(&elf(&root, &["update"]).1), c1, "same plan, same code");
    // 틀린 코드 = 거부와 새 요약
    let (code, out, err) = elf(&root, &["update", "--approve", "000000"]);
    assert!(code == 3 && err.contains("does not match the current plan") && out.contains(&c1), "{out}{err}");
    // 그사이 무엇이 바뀌면 코드가 바뀜 → 옛 코드 거부
    fs::write(root.join("note.md"), "x\n").unwrap();
    commit(&root);
    let c2 = code_of(&elf(&root, &["update"]).1);
    assert_ne!(c1, c2);
    let (code, _, err) = elf(&root, &["update", "--approve", &c1]);
    assert!(code == 3 && err.contains("does not match"), "{err}");
    assert!(!root.join("2_Log/S").exists());
    // 맞는 코드 = 적용(대문자도 같음)
    let (code, out, err) = elf(&root, &["update", "--to", &l, "--approve", &c2.to_uppercase()]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains(&format!("approved update (code {c2}): line 2.27 → {l}")) && root.join("2_Log/S/S001_log.md").is_file(), "{out}");
    // 이제 같은 줄 = 동의 없이 바로
    commit(&root);
    let (code, out, _) = elf(&root, &["update"]);
    assert!(code == 0 && !out.contains("approval"), "{out}");
    // --approve와 --dry-run은 함께 쓸 수 없음
    assert_eq!(elf(&root, &["update", "--dry-run", "--approve", "x"]).0, 2);
}

#[test]
fn lower_lines_are_refused_and_other_lines_are_run_by_that_version() {
    let tmp = tempdir().unwrap();
    // 이 CLI로 만든 프로젝트: 낮은 줄 거부, 잘못된 표기
    let root = new_project(&tmp.path().join("a"));
    let (code, _, err) = elf(&root, &["update", "--to", "2.27"]);
    assert!(code == 3 && err.contains("is lower than the project's line"), "{err}");
    assert_eq!(elf(&root, &["update", "--to", "latest"]).0, 2);
    // 프로젝트가 이 CLI보다 높음(인수 없음) = self update 안내
    let mp = root.join(".elf/manifest.json");
    let cli = elf_cli::embed::version();
    fs::write(&mp, fs::read_to_string(&mp).unwrap().replace(&format!("\"elf_version\": \"{cli}\""), "\"elf_version\": \"v9.9.0\"")).unwrap();
    let (code, _, err) = elf(&root, &["update"]);
    assert!(code == 3 && err.contains("newer than this elf") && err.contains("elf self update"), "{err}");

    // 1세대 프로젝트에서 같은 줄(2.27) 갱신 = v2.27.x가 실행: 넘기기를 끄면 거부, 구할 수 없으면 오류
    let old = old_project(&tmp.path().join("b"));
    let (code, _, err) = elf(&old, &["update", "--to", "2.27"]);
    assert!(code == 3 && err.contains("handing over is off"), "{err}");
    let home = tmp.path().join("home");
    let rel = tmp.path().join("rel");
    fs::create_dir_all(&rel).unwrap();
    fs::write(rel.join("releases.json"), "[]").unwrap();
    let env = [("ELF_HOME", home.to_str().unwrap()), ("ELF_RELEASES", rel.to_str().unwrap()), ("ELF_DISPATCH", "trace")];
    let (code, _, err) = elf_env(&old, &env, &["update", "--to", "2.27"]);
    assert!(code == 1 && err.contains("cannot obtain elf 2.27.x"), "{err}");
    // 보관소의 v2.27.9(이 시험 빌드 복사본)로 넘김 — 3.0 이전 버전에는 `--to`를 빼고 넘기므로, 받은 쪽은 인수 없는
    // `elf update`로 실행(복사본은 3.0 코드라 동의 요약을 출력)
    let d = home.join("versions/v2.27.9");
    fs::create_dir_all(&d).unwrap();
    let exe = if cfg!(windows) { "elf.exe" } else { "elf" };
    fs::copy(assert_cmd::cargo::cargo_bin("elf"), d.join(exe)).unwrap();
    let (code, out, err) = elf_env(&old, &env, &["update", "--to", "2.27"]);
    assert!(err.contains("dispatch: update to line 2.27 → v2.27.9"), "{err}");
    assert!(code == 3 && out.contains("update needs approval") && !err.contains("handing over is off"), "--to was not forwarded: {out}{err}");
    assert!(!old.join("2_Log/S").exists());
}
