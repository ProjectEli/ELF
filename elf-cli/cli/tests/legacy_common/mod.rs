//! 이전 형식(ELF 2.27) 저장소 fixture — 2.28 init 뒤 2.27.4 init 결과로 되돌림(로그 구조, TSV, S001, manifest schema 1).

use elf_cli::init::{InitError, InitOptions, run_init};
use std::fs;
use std::path::{Path, PathBuf};

fn tpl(name: &str) -> &'static str {
    elf_cli::legacy::embed::TEMPLATES.get_file(name).and_then(|f| f.contents_utf8()).expect("legacy template")
}

/// v2.27.4 `elf init`과 같은 로그 구조: `2_Log/S001_log.md`(인용 블록 헤더), `2_Log/Wiki/Session_Registry.tsv`, `2_Log/Archive/`.
#[allow(dead_code)]
pub fn legacy_init(tmp: &Path, o: &InitOptions) -> Result<PathBuf, InitError> {
    let root = run_init(tmp, o)?;
    if o.preset == "qa" {
        return Ok(root);
    }
    let _ = fs::remove_dir_all(root.join("2_Log/S"));
    fs::create_dir_all(root.join("2_Log/Wiki")).unwrap();
    fs::create_dir_all(root.join("2_Log/Archive")).unwrap();
    fs::write(root.join("2_Log/Wiki/Session_Registry.tsv"), tpl("log/Session_Registry.tsv").replace("YYYY-MM-DD", &o.date)).unwrap();
    fs::write(root.join("2_Log/S001_log.md"), tpl("log/sessionTemplate.md").replace("S{NNN}", "S001").replace("YYYY-MM-DD", &o.date)).unwrap();
    fs::write(root.join(".elf/manifest.json"), elf_cli::embed::MANIFEST_JSON.replace("elf-manifest/2", "elf-manifest/1")).unwrap();
    fs::write(root.join(".elf/version"), "v2.27.4\n").unwrap();
    Ok(root)
}
