# PLACEHOLDER_PROJECT_NAME

ELF **QA preset (experimental)** 로 생성된 질문 아카이브 프로젝트. (생성: PLACEHOLDER_DATE)

Q&A를 **bundle**(의미 단위)로 capture·분류·archive·recall합니다. 운영 규칙은 루트 `CLAUDE.md`(LLM 자동 로드), 정본 포맷은 `templates/bundle_template.md` 참조.

## 구조
- `<카테고리>/` — active bundle(`YYYYMMDD<L>_<Subject>.md`) · `<카테고리>/archive/` — 폐기·대체본
- `CLAUDE.md` — 운영 규칙(루트·LLM 자동 로드, ELF 관리)
- `templates/bundle_template.md` — bundle 정본 포맷
- `.elf/` — ELF 제어판(버전·manifest; `elf update`로 규칙 갱신). 직접 수정 금지.

## 사용
1. 질문이 생기면 맞는 카테고리에 bundle 작성(템플릿 복사 → frontmatter + 5개 섹션 채움).
2. 사고가 이어지면 `thread:` 슬러그로 묶고 질문 대장(append-only)에 누적.
3. 폐기 시 `archive/`로 이동.

> ⚠️ 실험적 preset — 비안정. 연구용 프로젝트는 `elf init <name> --preset full`(기본)을 사용하세요.
