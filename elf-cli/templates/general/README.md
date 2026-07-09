# PLACEHOLDER_PROJECT_NAME

ELF **general preset (experimental)** 로 생성된 목표지향 비연구 프로젝트. (생성: PLACEHOLDER_DATE)

명확한 목표를 가진 다세션 프로젝트(도구 개발·제안서 준비·학습·구축 등)를 base-delta 세션 로그로 누적합니다. 학술연구(시뮬·figure·논문)와 분리된 유형입니다.

## 구조
- `.elf/` — ELF 제어판(버전·manifest) + 관리 규칙 payload `managed/`(`EliRule`·`LogConvention`·`AI_PARA_Framework`·`LLMcliche`·세션/trial 스텁 `templates/`). `elf update`로 갱신 — 직접 수정 금지.
- `0_Meta/` — 프로젝트 거버넌스(`ProjectRule`·data overlay `<이름>.project.md`) — 사용자 영역
- `1_Concept/12_Planning/` — 목표·로드맵·계획 / `13_Ideas/` — 작은 아이디어
- `2_Log/` — 세션 로그(`S###_log.md`), `Wiki/`(요약·Registry), `Archive/`(완료)
- 도메인 작업 폴더(`src/`·`docs/` 등)는 프로젝트가 직접 추가.

## 사용
1. 목표·계획을 `1_Concept/12_Planning/`에 작성.
2. 작업은 세션(`S###`)·trial(`t##`)로 base-delta 누적 — Phase 1(가설·예상) 멈춤점 후 Phase 2(실행·관찰·해석). 규칙은 `.elf/managed/LogConvention.md`.
3. trial 형식은 기본 5-section. 프로젝트 성격상 조정이 필요하면 `0_Meta/ProjectRule.md`에 명시.

## 프로젝트 규칙
프로젝트 전용 규칙·목표는 `0_Meta/ProjectRule.md`. 구조·운영 상세는 `.elf/managed/EliRule.md`. 로깅 규칙은 `.elf/managed/LogConvention.md`. AI 컨텍스트 관리는 `.elf/managed/AI_PARA_Framework.md`.

> 주의: 실험적 preset — 비안정. 학술연구 프로젝트는 `elf init <name> --preset full`(기본)을 사용하세요.
