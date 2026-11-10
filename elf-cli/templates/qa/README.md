# PLACEHOLDER_PROJECT_NAME

ELF **QA preset**으로 생성된 질문 아카이브 프로젝트. (생성: PLACEHOLDER_DATE)

Q&A를 **bundle**(의미 단위)로 capture, 분류, archive, recall합니다. 운영 규칙 정본은 루트 `AGENTS.md`, 이 프로젝트만의 규칙은 `0_Meta/ProjectRule.md`, 정본 포맷은 `.elf/managed/templates/bundle_template.md` 참조.

## 구조
- `<카테고리>/` — 진행 중 bundle(`YYYYMMDD<L>_<Subject>.md`), `<카테고리>/archive/` — 완결 보관과 폐기(`status: deprecated`)
- `<카테고리>/README.md` — 그 카테고리만의 규칙(있을 때), `_`로 시작하는 하위 폴더 — 도구, 작업 영역
- `0_Meta/templates/<카테고리>.md` — 그 카테고리의 사용자 틀(있을 때, 사용자 소유)
- `AGENTS.md` — 운영 규칙 정본(루트, ELF 관리)
- `0_Meta/ProjectRule.md` — 이 프로젝트만의 규칙, 외부 반출 완화 선언(사용자 소유, `elf update` 미접근)
- `.elf/managed/templates/bundle_template.md` — bundle 정본 포맷
- `.elf/` — ELF 제어 영역(버전, manifest; `elf update`로 규칙 갱신). 직접 수정 금지.

## 사용
1. 질문이 생기면 맞는 카테고리에 bundle 작성(템플릿 복사 → frontmatter + 5개 섹션 채움).
2. 사고가 이어지면 `thread:` 슬러그로 묶고 질문 대장(append-only)에 누적.
3. 더 이어갈 일이 없으면 `archive/`로 이동(완결 보관, recall 대상). 틀렸거나 대체된 bundle은 `status: deprecated`.
4. `elf check logs`로 frontmatter, 위치, 이름, 관련 문서 경로를 점검(archive로 옮겨진 경로는 고침).

> 실험적 preset — 비안정. 연구용 프로젝트는 `elf init <name> --preset full`(기본)을 사용하세요.
