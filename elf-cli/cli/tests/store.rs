//! 버전 보관소(v3.0 S044 t58): `elf self list|install|uninstall|prune`, `check version` 보관소 줄.
//! 가짜 Release 폴더(`ELF_RELEASES`)와 임시 보관소(`ELF_HOME`) — 네트워크와 실제 `~/.elf`를 쓰지 않음.

use elf_cli::init::{InitOptions, run_init};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn elf(dir: &Path, home: &Path, rel: &Path, args: &[&str]) -> (i32, String) {
    let o = assert_cmd::Command::cargo_bin("elf")
        .unwrap()
        .env("ELF_DISPATCH", "off")
        .args(args)
        .current_dir(dir)
        .env("ELF_HOME", home)
        .env("ELF_RELEASES", rel)
        .output()
        .unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn exe() -> &'static str {
    if cfg!(windows) { "elf.exe" } else { "elf" }
}

/// zip(stored) 최소 작성 — 최상위에 실행 파일과 README.
fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    let (mut out, mut cd) = (Vec::new(), Vec::new());
    for (name, body) in files {
        let off = out.len() as u32;
        let mut lh = vec![0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 0, 0];
        lh.extend([0u8; 8]);
        lh.extend((body.len() as u32).to_le_bytes());
        lh.extend((body.len() as u32).to_le_bytes());
        lh.extend((name.len() as u16).to_le_bytes());
        lh.extend(0u16.to_le_bytes());
        lh.extend(name.as_bytes());
        out.extend(lh);
        out.extend(*body);
        let mut ch = vec![0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0];
        ch.extend([0u8; 8]);
        ch.extend((body.len() as u32).to_le_bytes());
        ch.extend((body.len() as u32).to_le_bytes());
        ch.extend((name.len() as u16).to_le_bytes());
        ch.extend([0u8; 12]);
        ch.extend(off.to_le_bytes());
        ch.extend(name.as_bytes());
        cd.extend(ch);
    }
    let at = out.len() as u32;
    out.extend(&cd);
    out.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((cd.len() as u32).to_le_bytes());
    out.extend(at.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

/// tar.xz — `elf-cli-<target>/` 아래 파일(cargo-dist 형식).
fn tar_xz(dir: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut t = Vec::new();
    for (name, body) in files {
        let path = format!("{dir}/{name}");
        let mut h = [0u8; 512];
        h[..path.len()].copy_from_slice(path.as_bytes());
        h[124..136].copy_from_slice(format!("{:011o}\0", body.len()).as_bytes());
        h[156] = b'0';
        h[257..263].copy_from_slice(b"ustar\0");
        t.extend(h);
        t.extend(*body);
        t.extend(vec![0u8; body.len().div_ceil(512) * 512 - body.len()]);
    }
    t.extend([0u8; 1024]);
    let mut w = lzma_rust2::XzWriter::new(Vec::new(), lzma_rust2::XzOptions::with_preset(1)).unwrap();
    w.write_all(&t).unwrap();
    w.finish().unwrap()
}

/// 가짜 Release: 버전마다 이 플랫폼 자산과 `.sha256`(bad_sha면 틀린 값), 발행 목록.
fn releases(root: &Path) -> PathBuf {
    let rel = root.join("rel");
    let name = elf_cli::store::asset_name().expect("platform with a release asset");
    for (tag, bad_sha) in [("v9.9.0", false), ("v9.9.1", false), ("v9.8.3", false), ("v9.7.0", true)] {
        let bin = format!("fake elf {tag}").into_bytes();
        let data = if name.ends_with(".zip") {
            zip(&[("README.md", b"readme"), (exe(), &bin)])
        } else {
            tar_xz(name.trim_end_matches(".tar.xz"), &[("README.md", b"readme"), (exe(), &bin)])
        };
        let d = rel.join(tag);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(&name), &data).unwrap();
        let sha: String = Sha256::digest(if bad_sha { b"other".as_slice() } else { &data }).iter().map(|b| format!("{b:02x}")).collect();
        fs::write(d.join(format!("{name}.sha256")), format!("{sha} *{name}\n")).unwrap();
    }
    fs::write(
        rel.join("releases.json"),
        r#"[{"tag_name":"v9.9.2-rc","prerelease":true,"draft":false},{"tag_name":"v9.9.1","prerelease":false,"draft":false},
            {"tag_name":"v9.9.0","prerelease":false,"draft":false},{"tag_name":"v9.8.3","prerelease":false,"draft":false},
            {"tag_name":"v9.7.0","prerelease":false,"draft":false}]"#,
    )
    .unwrap();
    rel
}

#[test]
fn install_list_prune_uninstall_with_sha_check() {
    let tmp = tempdir().unwrap();
    let (home, rel) = (tmp.path().join("home"), releases(tmp.path()));
    let cwd = tmp.path();
    let versions = home.join("versions");

    // 줄 = 최신 정식 patch(pre-release 제외)
    let (code, out) = elf(cwd, &home, &rel, &["self", "install", "9.9"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("installed v9.9.1 → "), "{out}");
    assert_eq!(fs::read(versions.join("v9.9.1").join(exe())).unwrap(), b"fake elf v9.9.1");
    let info = fs::read_to_string(versions.join("v9.9.1/install.json")).unwrap();
    assert!(info.contains("\"version\": \"v9.9.1\"") && info.contains("\"sha256\""), "{info}");
    let (code, out) = elf(cwd, &home, &rel, &["self", "install", "v9.9.1"]);
    assert!(code == 0 && out.contains("already in the version store: v9.9.1"), "{out}");
    assert_eq!(elf(cwd, &home, &rel, &["self", "install", "9.9.0"]).0, 0);

    // sha256 불일치 = 거부, 아무것도 남기지 않음
    let (code, out) = elf(cwd, &home, &rel, &["self", "install", "9.7.0"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("sha256 mismatch") && out.contains("nothing installed"), "{out}");
    assert!(!versions.join("v9.7.0").exists());
    assert!(fs::read_dir(&versions).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().starts_with(".tmp")));
    // 없는 줄, 잘못된 표기
    assert!(elf(cwd, &home, &rel, &["self", "install", "9.6"]).1.contains("no release in line 9.6"));
    assert!(elf(cwd, &home, &rel, &["self", "install", "latest"]).1.contains("not a version or a line"));

    // 목록
    let (code, out) = elf(cwd, &home, &rel, &["self", "list"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("version store: ") && out.contains("— 2 version(s)"), "{out}");
    assert!(out.find("v9.9.1 (line 9.9, structure generation 2)").unwrap() < out.find("v9.9.0").unwrap(), "newest first: {out}");
    let (_, out) = elf(cwd, &home, &rel, &["self", "list", "--available"]);
    assert!(out.contains("v9.9.1 (line 9.9, structure generation 2) — stored") && out.contains("v9.8.3 (line 9.8"), "{out}");
    assert!(!out.contains("9.9.2-rc"), "pre-releases are not offered: {out}");

    // check version: 프로젝트 줄의 실행 파일
    let proj = run_init(
        &tmp.path().join("p"),
        &InitOptions { name: "P".into(), preset: "minimal".into(), modules: None, categories: Vec::new(), lang: "ko-KR".into(), date: "2026-11-10".into() },
    )
    .unwrap();
    let mp = proj.join(".elf/manifest.json");
    let cli = elf_cli::embed::version();
    let (_, out) = elf(&proj, &home, &rel, &["check", "version"]);
    assert!(out.contains("— 2 version(s)") && out.contains(&format!("ok: line {}: this CLI", elf_cli::pin::parse(cli).unwrap().line())), "{out}");
    let m = fs::read_to_string(&mp).unwrap();
    fs::write(&mp, m.replace(&format!("\"elf_version\": \"{cli}\""), "\"elf_version\": \"v9.9.0\"")).unwrap();
    let (_, out) = elf(&proj, &home, &rel, &["check", "version"]);
    assert!(out.contains("ok: line 9.9: v9.9.1 in the version store"), "{out}");
    fs::write(&mp, m.replace(&format!("\"elf_version\": \"{cli}\""), "\"elf_version\": \"v9.8.0\"")).unwrap();
    let (_, out) = elf(&proj, &home, &rel, &["check", "version"]);
    assert!(out.contains("line 9.8: not in the version store — the first command here installs it (or `elf self install 9.8`)"), "{out}");

    // 정리와 삭제
    let (code, out) = elf(cwd, &home, &rel, &["self", "prune"]);
    assert!(code == 0 && out.contains("removed v9.9.0 (a newer patch of line 9.9 is kept)"), "{out}");
    assert!(elf(cwd, &home, &rel, &["self", "prune"]).1.contains("nothing to prune"));
    let (code, out) = elf(cwd, &home, &rel, &["self", "uninstall", "v9.9.1"]);
    assert!(code == 0 && out.contains("removed v9.9.1"), "{out}");
    let (code, out) = elf(cwd, &home, &rel, &["self", "uninstall", "v9.9.1"]);
    assert!(code == 1 && out.contains("is not in the version store"), "{out}");
    assert!(elf(cwd, &home, &rel, &["self", "list"]).1.contains("— 0 version(s)"));
}
