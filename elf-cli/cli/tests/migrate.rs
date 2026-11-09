//! 구조 이관(`elf update`, P022 §7, LogConvention §7) — 결정 파일 왕복, 전제(git), 폐기 기록 변환, 구버전 로그 번호 변경,
//! 로그만 있는 폴더.

use elf_cli::header::{self, Status};
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

fn elf(root: &Path, args: &[&str]) -> (i32, String) {
    let o = assert_cmd::Command::cargo_bin("elf").unwrap().env("ELF_DISPATCH", "off").args(args).current_dir(root).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

/// 줄을 바꾸는 `elf update` = 아무것도 쓰지 않고 요약과 확인 코드(exit 3) → 사용자 승인 뒤 `--approve <코드>`로 적용
/// (S044 t60). 반환 = 적용 실행의 (종료 코드, 출력).
fn update_approved(root: &Path) -> (i32, String) {
    let (code, out) = elf(root, &["update"]);
    assert_eq!(code, 3, "the first run only prints the summary: {out}");
    assert!(out.contains("update needs approval: project line 2.27 → ") && out.contains("structure generation 1 → 2"), "{out}");
    let c = out.lines().find_map(|l| l.split("approval code: ").nth(1)).map(|s| s.trim().to_string()).unwrap_or_else(|| panic!("no code: {out}"));
    elf(root, &["update", "--approve", &c])
}

fn legacy(id: &str, status: &str, extra: &str) -> String {
    format!(
        "# {id}: 제목 {id}\n\n> **Created**: 2026-06-01\\\n> **Modified**: 2026-06-02\\\n> **Status**: {status}\\\n> **목표**: 목표 {id}\\\n> **관련**: S000\\\n> **Handoff**: -\n\n---\n\n<!-- ELF 규범: 옛 안내 -->\n\n## t01: 첫 시도\n\n### 목표 (Goal)\n- x\n{extra}"
    )
}

/// 2.27 형식 프로젝트: 루트 S003(진행), Archive S001(종료), Archive S002(헤더와 Registry 불일치), purged S004(파일 없음).
fn old_project(tmp: &Path) -> PathBuf {
    let root = run_init(
        tmp,
        &InitOptions { name: "P".into(), preset: "minimal".into(), modules: None, categories: Vec::new(), lang: "ko-KR".into(), date: "2026-06-01".into() },
    )
    .unwrap();
    fs::write(root.join(".elf/manifest.json"), elf_cli::embed::MANIFEST_JSON.replace("elf-manifest/2", "elf-manifest/1")).unwrap();
    fs::write(root.join(".elf/version"), "v2.27.4\n").unwrap();
    fs::remove_dir_all(root.join("2_Log/S")).unwrap(); // 2.27 init은 S/를 만들지 않음
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::create_dir_all(root.join("6_Exp/64_Viz/S003")).unwrap();
    fs::write(root.join("6_Exp/64_Viz/S003/f.png"), b"x").unwrap();
    fs::write(root.join("2_Log/Archive/S001_log.md"), legacy("S001", "Complete", "- 다음: [S003](../S003_log.md)\n")).unwrap();
    fs::write(root.join("2_Log/Archive/S002_log.md"), legacy("S002", "★ 활성", "")).unwrap();
    fs::write(
        root.join("2_Log/S003_log.md"),
        legacy("S003", "★ 활성", "![Fig1](../6_Exp/64_Viz/S003/f.png) [앞](Archive/S001_log.md)\n| Figure | `../6_Exp/64_Viz/S003/f.png` | `../no/such.md` | `elf check` |\n")
            .replace("> **관련**: S000\\", "> **관련**: [P001](../1_Concept/12_Planning/P001_a.md), S002\\\n> **Lecture**: W05\\"),
    )
    .unwrap();
    fs::write(root.join("2_Log/Archive/.gitkeep"), "").unwrap();
    fs::write(
        root.join("2_Log/Wiki/Session_Registry.tsv"),
        "Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\nS001\t2026-06-01\t제목 S001\tComplete\t결론 하나\tArchive/S001_log.md\nS002\t2026-06-01\t제목 S002\tComplete\t결론 둘\tArchive/S002_log.md\nS003\t2026-06-01\t제목 S003\t★ 활성\t-\t-\nS004\t2026-06-01\t지운 세션\tPurged\t-\t-\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("1_Concept/12_Planning")).unwrap();
    fs::write(root.join("1_Concept/12_Planning/P001_a.md"), "# P001\n\n근거: [S001](../../2_Log/Archive/S001_log.md)\n").unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "core.autocrlf", "false"]);
    commit(&root);
    root
}

#[test]
fn e2e_decision_file_roundtrip_then_apply() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    // 이관 전: 새 구조 전용 명령만 거부(2.27 명령은 동작 — tests/legacy.rs)
    assert_eq!(elf(&root, &["log"]).0, 3);
    assert_eq!(elf(&root, &["trial", "write", "--from", "x.md"]).0, 3);
    // 1차: 경고(S002 불일치) → 결정 파일만, 나머지 무변경
    let (code, out) = elf(&root, &["update"]);
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("S002_log.md: header Status and Registry status disagree"), "{out}");
    let dec = fs::read_to_string(root.join(".elf/runtime/migration.yaml")).unwrap();
    assert!(dec.contains("evidence: 'last trial: t01: 첫 시도") && dec.contains("status: ''"), "{dec}");
    assert!(root.join("2_Log/S003_log.md").is_file() && !root.join("2_Log/S").exists());
    assert!(fs::read_to_string(root.join(".elf/manifest.json")).unwrap().contains("elf-manifest/1"), "manifest untouched");
    // 2차: 값을 채우고 재실행 → 적용
    fs::write(root.join(".elf/runtime/migration.yaml"), dec.replace("status: ''", "status: 'complete'")).unwrap();
    let (code, out) = update_approved(&root);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("logs: 3 moved") && out.contains("header field kept as a body line (`> 이관 전 헤더:`): 관련 — 3 log(s)"), "{out}");
    assert!(out.contains("code-span paths: 1 recomputed"), "{out}");
    assert!(!root.join(".elf/runtime/migration.yaml").exists());
    assert!(!root.join("2_Log/Archive").exists() && !root.join("2_Log/Wiki/Session_Registry.tsv").exists());
    assert!(!root.join(".elf/version").exists());
    assert!(fs::read_to_string(root.join(".elf/manifest.json")).unwrap().contains("\"elf_version\""));
    // 헤더 값
    let s1 = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    let (h, at) = header::parse(&s1).unwrap();
    assert_eq!((h.status, h.conclusion.as_str(), h.goal.as_str(), h.handoff.as_str()), (Status::Complete, "결론 하나", "목표 S001", ""));
    assert_eq!((h.created.as_str(), h.modified.as_str()), ("2026-06-01", "2026-06-02"));
    assert!(s1[at..].starts_with("# S001: 제목 S001\n\n> 이관 전 헤더: 관련 = S000\n\n<!-- ELF 규범:") && !s1.contains("옛 안내"), "{s1}");
    assert!(s1.contains("[S003](S003_log.md)"), "link to the moved root log: {s1}");
    let s2 = header::parse(&fs::read_to_string(root.join("2_Log/S/S002_log.md")).unwrap()).unwrap().0;
    assert_eq!((s2.status, s2.conclusion.as_str()), (Status::Complete, "결론 둘"));
    // 루트 로그: 한 단계 깊어진 링크
    let s3 = fs::read_to_string(root.join("2_Log/S/S003_log.md")).unwrap();
    assert!(s3.contains("![Fig1](../../6_Exp/64_Viz/S003/f.png) [앞](S001_log.md)"), "{s3}");
    // 코드 span: 해석되는 상대 경로만 다시 계산(사용자 결정 2026-11-10), 없는 대상과 명령은 그대로
    assert!(s3.contains("| Figure | `../../6_Exp/64_Viz/S003/f.png` | `../no/such.md` | `elf check` |"), "{s3}");
    // 7키에 없는 필드 = 제목 아래 한 줄(링크도 새 위치 기준)
    assert!(s3.contains("# S003: 제목 S003\n\n> 이관 전 헤더: 관련 = [P001](../../1_Concept/12_Planning/P001_a.md), S002; Lecture = W05\n\n<!-- ELF 규범:"), "{s3}");
    assert_eq!(header::parse(&s3).unwrap().0.status, Status::Ongoing);
    // purged = 헤더와 흔적만(번호 보존, 보관 파일 없음)
    let s4 = fs::read_to_string(root.join("2_Log/S/S004_log.md")).unwrap();
    assert!(s4.contains("status: deprecated") && s4.contains("# S004: 지운 세션") && s4.contains("복원 불가"), "{s4}");
    // 밖에서 들어오는 링크
    assert!(fs::read_to_string(root.join("1_Concept/12_Planning/P001_a.md")).unwrap().contains("(../../2_Log/S/S001_log.md)"));
    // 생성 파일, 점검
    assert!(fs::read_to_string(root.join("2_Log/S/Registry.md")).unwrap().contains("[S003](S003_log.md) | ongoing"));
    let (code, out) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "{out}");
    let (code, out) = elf(&root, &["check", "version"]);
    assert!(code == 0 && out.contains("ok: verdict"), "{out}");
    // 이관 뒤 쓰기 명령 동작
    assert_eq!(elf(&root, &["session", "new", "다음", "--goal", "g"]).0, 0);
    assert!(root.join("2_Log/S/S005_log.md").is_file());
}

#[test]
fn dirty_tree_refuses_whole_update_and_dry_run_only_reports() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    fs::write(root.join("README.md"), "edited\n").unwrap();
    let before = fs::read_to_string(root.join(".elf/manifest.json")).unwrap();
    let (code, out) = elf(&root, &["update"]);
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("git working tree is not clean") && out.contains("README.md"), "{out}");
    assert_eq!(fs::read_to_string(root.join(".elf/manifest.json")).unwrap(), before, "managed update waits too");
    assert!(!root.join(".elf/runtime/migration.yaml").exists());
    let (code, out) = elf(&root, &["update", "--dry-run"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("warn: migration: git working tree is not clean") && out.contains("would write .elf/runtime/migration.yaml"), "{out}");
}

#[test]
fn partial_and_purged_deprecations_become_store_and_markers() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    // S001에 옛 부분 폐기(t02, 블록 보관) + purge 흔적(t03), S002 불일치는 Registry 쪽을 고쳐 경고 없앰
    let s1 = legacy(
        "S001",
        "Complete",
        "\n> **Deprecated** → [Deprecated/S001_log.partial.md](../Deprecated/S001_log.partial.md) (S001-D01 · t02 · 2026-06-03 · replaced_by: S001 t04)\n\n> **Deprecated** → purged (S001-D02 · t03 · 2026-06-03 · purged: 2026-06-05)\n",
    );
    fs::write(root.join("2_Log/Archive/S001_log.md"), s1).unwrap();
    fs::create_dir_all(root.join("2_Log/Deprecated")).unwrap();
    fs::write(
        root.join("2_Log/Deprecated/S001_log.partial.md"),
        "---\ndeprecated: partial\nsource: 2_Log/Archive/S001_log.md\n---\n\n# S001: 제목 S001\n> **Deprecated**: 2026-06-03 · S001-D01 · t02 · replaced_by: S001 t04 · reason: 조건 오류\n\n<!-- deprecated:begin S001-D01 -->\n## t02: 틀린 시도\n\n### 목표 (Goal)\n- 틀린 가설 [S003](../S003_log.md)\n<!-- deprecated:end S001-D01 -->\n",
    )
    .unwrap();
    let tsv = fs::read_to_string(root.join("2_Log/Wiki/Session_Registry.tsv")).unwrap().replace("S002\t2026-06-01\t제목 S002\tComplete", "S002\t2026-06-01\t제목 S002\t★ 활성");
    fs::write(root.join("2_Log/Wiki/Session_Registry.tsv"), tsv).unwrap();
    commit(&root);
    let (code, out) = update_approved(&root);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("deprecation records: 2 converted"), "{out}");
    let s1 = fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap();
    assert!(s1.contains("## t02: (폐기 S001-D01)\n> **Deprecated**: S001-D01 · 2026-06-03 · t02 · 사유: 조건 오류 · 대체: S001 t04"), "{s1}");
    assert!(s1.contains("## t03: (폐기 S001-D02)\n> **Deprecated**: S001-D02 · 2026-06-03 · t03 · 복원 불가(purged 2026-06-05)"), "{s1}");
    assert!(!root.join("2_Log/Deprecated").exists());
    let raw = fs::read(root.join(".elf/deprecated/S001-D01.md.xz")).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("틀린 가설"));
    let (_, shown) = elf(&root, &["deprecate", "show", "S001-D01"]);
    assert!(shown.contains("## t02: 틀린 시도") && shown.contains("[S003](S003_log.md)"), "links rebased to the log: {shown}");
    // purge 흔적은 보관 파일 없이도 짝 검사 통과, 블록은 되돌릴 수 있음
    commit(&root);
    let (code, out) = elf(&root, &["check", "logs"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(elf(&root, &["deprecate", "--restore", "S001-D01"]).0, 0);
    assert!(fs::read_to_string(root.join("2_Log/S/S001_log.md")).unwrap().contains("## t02: 틀린 시도"));
}

#[test]
fn log_made_by_older_cli_after_migration_is_renumbered() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    let tsv = fs::read_to_string(root.join("2_Log/Wiki/Session_Registry.tsv")).unwrap().replace("S002\t2026-06-01\t제목 S002\tComplete", "S002\t2026-06-01\t제목 S002\t★ 활성");
    fs::write(root.join("2_Log/Wiki/Session_Registry.tsv"), tsv).unwrap();
    commit(&root);
    assert_eq!(update_approved(&root).0, 0);
    commit(&root);
    // 구버전 CLI가 루트에 S002를 다시 만듦(이관 뒤 번호 계산 근거가 사라짐)
    fs::write(root.join("2_Log/S002_log.md"), legacy("S002", "★ 활성", "").replace("제목 S002", "다른 세션")).unwrap();
    assert!(elf(&root, &["check", "logs"]).1.contains("earlier layout"));
    commit(&root);
    let (code, out) = elf(&root, &["update"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("renumbered S002 → S005"), "{out}");
    assert!(fs::read_to_string(root.join("2_Log/S/S005_log.md")).unwrap().contains("# S005: 다른 세션"));
    assert!(fs::read_to_string(root.join("2_Log/S/S002_log.md")).unwrap().contains("# S002: 제목 S002"), "the migrated S002 is kept");
}

#[test]
fn log_only_folder_migrates_logs_without_elf() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::write(root.join("2_Log/Archive/S001_log.md"), legacy("S001", "Complete", "")).unwrap();
    fs::write(root.join("2_Log/S002_log.md"), legacy("S002", "★ 활성", "")).unwrap();
    git(root, &["init", "-q"]);
    commit(root);
    let (code, out) = update_approved(root);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("log-only folder"), "{out}");
    assert!(root.join("2_Log/S/S001_log.md").is_file() && root.join("2_Log/S/S002_log.md").is_file());
    assert!(!root.join(".elf").exists(), "no .elf/ left behind");
    assert_eq!(elf(root, &["check", "logs"]).0, 0);
}

/// 이관 시행착오 제안(S044 t55): `--dry-run --out` 미리보기, git 추적 밖 폴더 제외, 스크립트의 지운 파일 언급,
/// conclusion 앞머리 상태어, 코드 span 변경 목록(이관 뒤 파일 기준 줄).
#[test]
fn dry_run_out_preview_and_review_lists() {
    let tmp = tempdir().unwrap();
    let root = old_project(tmp.path());
    let tsv = root.join("2_Log/Wiki/Session_Registry.tsv");
    let t = fs::read_to_string(&tsv)
        .unwrap()
        .replace("Complete\t결론 하나", "Complete\t완료: 결론 하나")
        .replace("S002\t2026-06-01\t제목 S002\tComplete", "S002\t2026-06-01\t제목 S002\t★ 활성");
    fs::write(&tsv, t).unwrap();
    fs::create_dir_all(root.join("tools")).unwrap();
    fs::write(root.join("tools/read.py"), "REG = '2_Log/Wiki/Session_Registry.tsv'\nOLD = '2_Log/Archive/S001_log.md'\n").unwrap();
    let gi = fs::read_to_string(root.join(".gitignore")).unwrap();
    fs::write(root.join(".gitignore"), format!("{gi}\nscratch/\n")).unwrap();
    fs::create_dir_all(root.join("scratch")).unwrap();
    let scratch = "# copy\n\n[S001](../2_Log/Archive/S001_log.md) Archive/S001_log.md\n";
    fs::write(root.join("scratch/copy.md"), scratch).unwrap();
    commit(&root);

    // 위치 검사: 프로젝트 안(.elf/runtime 밖), --dry-run 없음, 표지 없는 비지 않은 폴더 = usage 오류
    let (code, out) = elf(&root, &["update", "--dry-run", "--out", "preview"]);
    assert!(code == 2 && out.contains("inside the project"), "{out}");
    assert!(!root.join("preview").exists());
    assert_eq!(elf(&root, &["update", "--out", "x"]).0, 2);
    let other = tmp.path().join("other");
    fs::create_dir_all(&other).unwrap();
    fs::write(other.join("a.txt"), "x").unwrap();
    assert_eq!(elf(&root, &["update", "--dry-run", "--out", other.to_str().unwrap()]).0, 2);

    // 미리보기
    let pv = tmp.path().join("pv");
    let (code, out) = elf(&root, &["update", "--dry-run", "--out", pv.to_str().unwrap()]);
    assert!(code == 0 && out.contains("preview written to"), "{out}");
    assert!(pv.join("ELF_MIGRATION_PREVIEW.txt").is_file() && pv.join("report.txt").is_file() && pv.join("warnings.tsv").is_file());
    assert!(!root.join("2_Log/S").exists(), "dry-run writes nothing in the project");
    let s3_preview = fs::read_to_string(pv.join("files/2_Log/S/S003_log.md")).unwrap();
    let headers = fs::read_to_string(pv.join("headers.tsv")).unwrap();
    assert!(
        headers.lines().any(|l| l.starts_with("2_Log/Archive/S001_log.md\t2_Log/S/S001_log.md\tcomplete\t2026-06-01\t2026-06-02\t목표 S001\theader 목표\t")
            && l.contains("\t완료: 결론 하나\tRegistry Key Finding\t")
            && l.ends_with("conclusion starts with a status word (완료)")),
        "{headers}"
    );
    assert!(headers.lines().any(|l| l.starts_with("2_Log/S003_log.md\t2_Log/S/S003_log.md\tongoing\t") && l.contains("Lecture = W05")), "{headers}");
    let changes = fs::read_to_string(pv.join("changes.tsv")).unwrap();
    let span: Vec<&str> = changes.lines().find(|l| l.contains("\tspan\t")).unwrap().split('\t').collect();
    let n: usize = span[1].parse().unwrap();
    assert_eq!((span[0], span[3], span[4]), ("2_Log/S/S003_log.md", "../6_Exp/64_Viz/S003/f.png", "../../6_Exp/64_Viz/S003/f.png"));
    assert!(s3_preview.lines().nth(n - 1).unwrap().contains("`../../6_Exp/64_Viz/S003/f.png`"), "line {n}\n{s3_preview}");
    assert!(changes.contains("1_Concept/12_Planning/P001_a.md\t3\tlink\t../../2_Log/Archive/S001_log.md\t../../2_Log/S/S001_log.md"), "{changes}");
    let mentions = fs::read_to_string(pv.join("mentions.tsv")).unwrap();
    assert!(mentions.contains("tools/read.py\t1\tSession_Registry.tsv\t") && mentions.contains("tools/read.py\t2\t"), "{mentions}");
    assert!(!mentions.contains("scratch/") && !changes.contains("scratch/"), "untracked folder left out:\n{mentions}\n{changes}");
    // 다시 실행 = 표지가 있는 폴더는 비우고 다시 씀, .elf/runtime 아래는 허용
    fs::write(pv.join("stale.txt"), "x").unwrap();
    assert_eq!(elf(&root, &["update", "--dry-run", "--out", pv.to_str().unwrap()]).0, 0);
    assert!(!pv.join("stale.txt").exists());
    assert_eq!(elf(&root, &["update", "--dry-run", "--out", ".elf/runtime/pv"]).0, 0);
    assert!(root.join(".elf/runtime/pv/headers.tsv").is_file());

    // 적용: 출력의 코드 span 목록과 줄, 상태어, 스크립트 언급 먼저, 추적 밖 사본 그대로, 결과 = 미리보기
    let (code, out) = update_approved(&root);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("code span changed: 2_Log/S/S003_log.md:{n} `../6_Exp/64_Viz/S003/f.png` → `../../6_Exp/64_Viz/S003/f.png`")), "{out}");
    assert!(out.contains("conclusion starts with a status word") && out.contains("1 log(s): S001 (완료)"), "{out}");
    assert!(out.contains("review scripts and rules: ") && out.contains("file(s): tools/read.py (2 lines)"), "{out}");
    assert_eq!(fs::read_to_string(root.join("scratch/copy.md")).unwrap(), scratch, "untracked copy untouched");
    assert_eq!(fs::read_to_string(root.join("2_Log/S/S003_log.md")).unwrap(), s3_preview, "preview = applied result");
}
