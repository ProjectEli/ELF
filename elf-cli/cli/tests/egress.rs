//! v2.23 Egress Control — 규칙 존재 게이트(S036 t02): research EliRule §2 절 + 2계보(research, qa) AGENTS 상시 의무 1줄.
//! 한쪽 계보만 수정되거나 `elf update`로 배포되지 않는 drift를 cargo test에서 적발.

use elf_cli::embed;

fn tpl(path: &str) -> String {
    embed::TEMPLATES
        .get_file(path)
        .unwrap_or_else(|| panic!("missing embed: {path}"))
        .contents_utf8()
        .expect("utf8")
        .to_string()
}

#[test]
fn egress_clause_present_in_elirule_both_lineages() {
    for (p, num) in [("meta/EliRule.md", "2.9")] {
        let t = tpl(p);
        assert!(t.contains(&format!("### {num} 외부 반출과 공개 통제 (Egress Control)")), "{p}: clause heading");
        assert!(t.contains("명시 선언을 둘 때만"), "{p}: ProjectRule relaxation clause");
        assert!(t.contains("본 조항의 반출이 아닙니다"), "{p}: explicit exclusion (inference, opt-in tools)");
    }
    for (p, num) in [("meta/EliRule.en.md", "2.9")] {
        assert!(tpl(p).contains(&format!("### {num} External egress and publication control (Egress Control)")), "{p}");
    }
}

#[test]
fn egress_line_present_in_agents_all_lineages() {
    for p in ["root/AGENTS.md", "qa/AGENTS.md"] {
        assert!(tpl(p).contains("- **외부 반출**:"), "{p}: standing-duty line");
    }
    for p in ["root/AGENTS.en.md"] {
        assert!(tpl(p).contains("- **External egress**:"), "{p}: standing-duty line (EN)");
    }
    // 계보별 참조 번호 — research §2.9 (qa는 EliRule 없음 → 자기완결, 참조 없음, 완화 선언처 = 0_Meta/ProjectRule.md)
    assert!(tpl("root/AGENTS.md").contains("(EliRule §2.9)"));
    assert!(tpl("root/AGENTS.en.md").contains("(EliRule §2.9)"));
    assert!(!tpl("qa/AGENTS.md").contains("EliRule §"));
    assert!(tpl("qa/AGENTS.md").contains("완화는 `0_Meta/ProjectRule.md`에 상시 허용 대상을 선언한 경우만"));
}

#[test]
fn deprecated_line_is_own_bullet_in_agents() {
    // v2.24.0 결함(S039 t02): 폐기 줄이 외부 반출 줄 끝에 개행 없이 붙어 독립 항목이 아니었음 — 줄머리 bullet 강제.
    for p in ["root/AGENTS.md"] {
        assert!(tpl(p).lines().any(|l| l.starts_with("- **폐기(Deprecated)**:")), "{p}: deprecated duty must be its own bullet");
    }
    for p in ["root/AGENTS.en.md"] {
        assert!(tpl(p).lines().any(|l| l.starts_with("- **Deprecated**:")), "{p}: deprecated duty must be its own bullet (EN)");
    }
}
