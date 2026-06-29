//! §3 AI Communication Rules drift gate.
//! research(meta) EliRule와 general EliRule의 **보편 tone 규칙**이 byte-identical로
//! 유지되는지 검증 — 한쪽만 수정되어 조용히 갈라지는 drift를 cargo test에서 적발.
//! 도메인 특화 규칙(4 엔지니어링 팩트 / 5 Data Reusability↔Reproducibility /
//! 6 물리적 인과 / 10 실험 관찰↔중립 관찰)은 **의도적 분기**라 동일성 미강제.
//! 신규 §3 규칙은 UNIVERSAL/DIVERGENT 중 하나로 분류해야 통과(미분류 시 실패) — 자기유지.

use elf_cli::embed;
use std::collections::BTreeMap;

/// `## 3. ...` 섹션(다음 `## ` 헤더 또는 EOF까지)에서 `N. 본문` 번호 항목을 {번호: 본문}으로 추출.
fn section3_rules(md: &str) -> BTreeMap<u32, String> {
    let mut in_s3 = false;
    let mut out = BTreeMap::new();
    for line in md.lines() {
        if line.starts_with("## 3.") {
            in_s3 = true;
            continue;
        }
        if in_s3 && line.starts_with("## ") {
            break;
        }
        if in_s3 {
            if let Some((num, rest)) = line.split_once(". ") {
                if let Ok(n) = num.parse::<u32>() {
                    out.insert(n, rest.trim().to_string());
                }
            }
        }
    }
    out
}

fn elirule(path: &str) -> String {
    embed::TEMPLATES
        .get_file(path)
        .unwrap_or_else(|| panic!("missing embed: {path}"))
        .contents_utf8()
        .expect("utf8")
        .to_string()
}

#[test]
fn section3_universal_rules_match_between_research_and_general() {
    let research = section3_rules(&elirule("meta/EliRule.md"));
    let general = section3_rules(&elirule("general/EliRule.md"));

    assert!(!research.is_empty(), "research §3 규칙 파싱 실패 — 헤더 포맷 변경?");
    assert!(!general.is_empty(), "general §3 규칙 파싱 실패 — 헤더 포맷 변경?");

    // 보편 규칙: 두 유형에서 byte-identical 유지 (silent drift 차단)
    const UNIVERSAL: [u32; 9] = [1, 2, 3, 7, 8, 9, 11, 12, 13];
    // 의도적 분기(도메인 특화) — 동일성 미강제
    const DIVERGENT: [u32; 4] = [4, 5, 6, 10];

    // 1) 규칙 번호 집합 일치 — 한쪽에만 규칙 추가/삭제 시 적발
    let rkeys: Vec<u32> = research.keys().copied().collect();
    let gkeys: Vec<u32> = general.keys().copied().collect();
    assert_eq!(
        rkeys, gkeys,
        "§3 규칙 번호 집합이 research/general 간 불일치 — 한쪽 규칙 추가/삭제 의심"
    );

    // 2) 모든 규칙은 UNIVERSAL/DIVERGENT 중 하나로 분류돼야 함 — 신규 규칙 강제 분류(자기유지)
    for &n in research.keys() {
        assert!(
            UNIVERSAL.contains(&n) || DIVERGENT.contains(&n),
            "§3 rule {n} 미분류 — 본 테스트 UNIVERSAL/DIVERGENT에 추가하여 의도 명시 필요"
        );
    }

    // 3) 보편 규칙 동일성 — drift 차단
    for n in UNIVERSAL {
        assert_eq!(
            research.get(&n),
            general.get(&n),
            "§3 rule {n} drift — research/general EliRule 보편 규칙은 동일해야 함(한쪽만 수정됨)"
        );
    }
}
