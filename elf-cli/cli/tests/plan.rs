//! t02 통합: embed된 정본 manifest의 자기검증.
//! 이 테스트가 곧 src 측 drift 게이트 — manifest 해시가 stale이면 cargo test가 실패한다.

use elf_cli::{embed, hash, manifest};

#[test]
fn embedded_manifest_parses_with_valid_schema() {
    let m = manifest::embedded();
    assert!(!m.files.is_empty());
}

#[test]
fn every_src_exists_in_embedded_templates() {
    for e in &manifest::embedded().files {
        let rel = e
            .src
            .strip_prefix("templates/")
            .unwrap_or_else(|| panic!("src must start with templates/: {}", e.src));
        assert!(
            embed::TEMPLATES.get_file(rel).is_some(),
            "manifest src not embedded: {}",
            e.src
        );
    }
}

/// manifest의 sha256 == embed된 정본 파일의 실제 해시 (stale 즉시 검출)
#[test]
fn manifest_hashes_match_embedded_templates() {
    for e in &manifest::embedded().files {
        let rel = e.src.strip_prefix("templates/").unwrap();
        let file = embed::TEMPLATES.get_file(rel).unwrap();
        let actual = hash::sha256_lf(file.contents());
        assert_eq!(
            actual, e.sha256,
            "manifest sha256 stale for {} — 재산출 필요",
            e.src
        );
    }
}

/// 역방향: embed된 모든 템플릿이 manifest에 등재되어야 함 (orphan = 배포 누락 의심)
#[test]
fn every_embedded_template_is_in_manifest() {
    use include_dir::Dir;
    fn collect<'a>(dir: &'a Dir, out: &mut Vec<&'a str>) {
        for f in dir.files() {
            out.push(f.path().to_str().unwrap());
        }
        for d in dir.dirs() {
            collect(d, out);
        }
    }
    let mut embedded = Vec::new();
    collect(&embed::TEMPLATES, &mut embedded);

    let manifest_srcs: std::collections::BTreeSet<String> = manifest::embedded()
        .files
        .iter()
        .map(|e| e.src.strip_prefix("templates/").unwrap().replace('\\', "/"))
        .collect();

    for path in embedded {
        let norm = path.replace('\\', "/");
        assert!(
            manifest_srcs.contains(&norm),
            "embedded template not in manifest (orphan): templates/{norm}"
        );
    }
}

#[test]
fn dests_and_srcs_are_unique() {
    let m = manifest::embedded();
    let mut dests: Vec<_> = m.files.iter().map(|e| &e.dest).collect();
    let mut srcs: Vec<_> = m.files.iter().map(|e| &e.src).collect();
    dests.sort();
    srcs.sort();
    let (dn, sn) = (dests.len(), srcs.len());
    dests.dedup();
    srcs.dedup();
    assert_eq!(dn, dests.len(), "duplicate dest in manifest");
    assert_eq!(sn, srcs.len(), "duplicate src in manifest");
}

/// dirs 데이터 정합: preset이 참조하는 모듈 키 실존 + raw_data ⊆ 전체 폴더 + core 비어있지 않음
#[test]
fn embedded_dirs_data_is_consistent() {
    let m = manifest::embedded();
    assert!(!m.dirs.core.is_empty(), "dirs.core must not be empty");
    assert!(!m.dirs.presets.is_empty(), "dirs.presets must not be empty");
    for (preset, mods) in &m.dirs.presets {
        for key in mods {
            assert!(
                m.dirs.modules.contains_key(key),
                "preset {preset} references unknown module: {key}"
            );
        }
    }
    let all_dirs: std::collections::BTreeSet<&str> = m
        .dirs
        .core
        .iter()
        .chain(m.dirs.modules.values().flatten())
        .map(String::as_str)
        .collect();
    for r in &m.dirs.raw_data {
        assert!(all_dirs.contains(r.as_str()), "raw_data not in any dir list: {r}");
    }
    // 생성 파일의 dest 부모 폴더가 scaffold에 존재하는지 (templates/ 등)
    let full = elf_cli::plan::plan_dirs(&m, "full").unwrap();
    assert!(full.iter().any(|d| d.path == "templates"));
    assert!(full.iter().any(|d| d.path == "6_Exp/62_Empirical/Raw"));
}

/// hybrid 정본은 마커블록 규약(S007 §4.1)을 포함해야 한다
#[test]
fn hybrid_templates_contain_marker_block() {
    for e in &manifest::embedded().files {
        if e.tier == manifest::Tier::Hybrid {
            let rel = e.src.strip_prefix("templates/").unwrap();
            let s = embed::TEMPLATES.get_file(rel).unwrap().contents_utf8().unwrap();
            assert!(
                s.contains("# >>> ELF managed >>>") && s.contains("# <<< ELF managed <<<"),
                "hybrid template missing marker block: {}",
                e.src
            );
        }
    }
}
