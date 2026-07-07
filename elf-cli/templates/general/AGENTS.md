# AGENTS.md — ELF 에이전트 진입 규칙 (요약 + 정본 포인터, general)

이 프로젝트는 **ELF(Eli's Lab Framework) general preset**(목표지향 비연구) 거버넌스를 따름. 본 파일은 AI 에이전트 진입용 **요약(digest)** — 규칙 정본은 `.elf/managed/`. 요약과 정본이 다르면 **정본 우선**. (ELF 관리 파일 — 직접 수정 금지, `elf update`가 교체.)

## 규칙 정본 (필독)

| 파일 | 역할 |
|---|---|
| `.elf/managed/EliRule.md` | 전역 규칙 — 폴더 구조, §3 AI 소통(언어·문체·금지) |
| `.elf/managed/LogConvention.md` | **세션 로그·trial 작성 규칙(필수 준수)** — 포맷·Phase 절차 |
| `0_Meta/ProjectRule.md` | 프로젝트 전용 규칙(사용자 소유) — **커스터마이즈는 여기** |
| `.elf/managed/AI_PARA_Framework.md` | PARA 파일 격리·Archive 방화벽·환각 방지 |
| `.elf/managed/templates/sessionTemplate.md` · `trialTemplate.md` | 로그 형식 정본 stub |

> legacy 레이아웃(마이그레이션 전) 프로젝트는 위 관리 정본이 `0_Meta/`·`templates/`에 위치 — `elf migrate`로 이전(opt-in).

## 상시 의무 (요약)

- **기록**: 산출물·결론·방향에 영향 주는 작업은 그 turn에 `2_Log/S###_log.md`에 trial(`t##`)로 기록.
- **trial 추가 = `elf trial new [제목]`** — 현행 정본 stub을 활성 로그에 append. CLI 미설치 시 `.elf/managed/templates/trialTemplate.md` 수동 복사.
- **선례 ≠ 규범**: 과거 세션/trial 로그는 참고일 뿐 규범이 아님 — 형식·규칙은 정본을 따르고, 선례가 정본과 다르면 모방하지 말고 사용자에게 보고.
- **Phase 분리**: `### 가설`·`### 예상`(Phase 1) = 실행 **전** 작성 후 멈춤 → 실행 → `### 관찰`~`### 교훈`(Phase 2). (LogConvention §5)
- **산출물 버전 보존**: 반복 개선 시 덮어쓰지 말고 버전 접미사(`_v1`/`_v2`)로 보존 — 각 버전 ↔ delta trial 1:1. (LogConvention §3)
- **세션 수명주기**: 시작 `elf session new "<제목>"` → 종료 전 `elf validate`(경고 해소) → `elf session close`.
- **컨텍스트 재구성 후 재정렬**: compact·세션 재시작 등으로 컨텍스트가 재구성되면 활성 로그 헤더(`Handoff`)를 다시 읽고 이어감.

## 소유권·우선순위

- ELF 관리 파일(`.elf/managed/`의 EliRule·LogConvention·AI_PARA_Framework·LLMcliche·`templates/*`와 companion, 루트 `.claudeignore`, 본 파일) **직접 수정 금지** — `elf update`가 교체(편집 시 `.elf-new` 병기). 커스터마이즈·예외 선언은 `0_Meta/ProjectRule.md`.
- `0_Meta/` = 프로젝트 전용(사용자 영역): ProjectRule·data overlay·프로젝트 재량 — `elf update` 미접근.
- data 파일(LLMcliche)의 항목 커스터마이즈 = project overlay `0_Meta/LLMcliche.project.md`(사용자 소유) — base와 **병행 로드**, 유효 규칙 = base ⊕ overlay(추가 / 제외[사유 필수] / 재정의). 규약: EliRule §2.4.
- 규칙 충돌 시 우선순위: `0_Meta/ProjectRule.md`(프로젝트 특화) > 정본 일반 규칙(`.elf/managed/*`) > 본 요약 > 상위 디렉토리·전역 에이전트 규칙.
- `Archive/` 폴더는 자율 탐색 금지(`.claudeignore` 방화벽) — 사용자가 경로를 명시할 때만 열람.
