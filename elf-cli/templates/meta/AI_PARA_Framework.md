# AI PARA Framework & Context Management

이 문서는 프로젝트의 문서(세션 로그, 기획 문서, 스크립트)가 자기 상태를 어디에 나타내는지와, AI 에이전트가 각 상태의 문서를 어떻게 읽고 써야 하는지를 정의함. 폐기, 구버전 기록이 현재의 사실로 잘못 쓰이는 것을 막고, 사람과 AI가 같은 기준으로 문서를 관리하기 위한 규칙임. 이름은 PARA(Projects, Areas, Resources, Archives) 체계에서 Archive 개념을 가져온 데서 왔으나, 본 규칙은 PARA를 몰라도 이해되도록 정의함.

## 1. 상태를 나타내는 곳

문서 종류마다 상태를 나타내는 곳이 하나씩 있음. 파일명 접두, 태그, 손으로 쓰는 별도 대장은 쓰지 않음.

| 문서 | 상태를 나타내는 곳 | 비고 |
|------|------|------|
| 세션 로그(`2_Log/S/`, `2_Log/G/`) | 헤더 `status`(`ongoing`, `complete`, `deprecated`) | 로그는 상태가 바뀌어도 옮기지 않음(경로 고정). 목록 `Registry.md`는 헤더에서 elf가 만듦 |
| 계획 문서(`1_Concept/12_Planning/`) | 폴더: 루트 = 진행 중, `Archive/` = 종료 | 위치가 곧 상태(Location = State) |
| 스크립트(`61_Sim/Scripts`, `63_Analysis/Scripts`) | 폴더: 루트 = 활성, `Archive/` = 일회용 보관 | §3 |
| 폐기(모든 문서) | 원문은 작업 트리에서 지우고 `.elf/deprecated/<ID>.md.xz`에 압축 보관, 원래 자리에는 흔적 | 폐기 폴더(`Deprecated/`)는 두지 않음 |

- 별도의 `1_Active` 폴더를 만들지 않음. 작업 중인 문서는 부모 폴더(세션 로그는 `2_Log/S/`, `2_Log/G/`)의 루트에 둠.
- 상태 폴더(`Archive/`)는 평탄함(하위 폴더 없음). 파일명은 이동해도 바뀌지 않음.
- 폐기 여부는 사용자가 결정함. 에이전트는 후보와 사유를 제시하고, 사용자의 지시나 승인이 있은 뒤 `elf deprecate`로 실행함. 형식, 절차는 `.elf/managed/LogConvention.md` §6.

### 1.1 정보용 companion (Informative Companion)

`*.en.md`(일반화 `*.<lang>.md`) 형식 파일은 거버넌스 문서의 **정보용 번역(read-only human companion)**임 — 국제 사용자의 *독해*용이며 operative source가 아님. operative 정본은 항상 동명 `*.md`(PROJECT_LANG 정본, 기본 한국어)임.

*   **AI 동작 규칙**: AI는 규칙, 구조, 지시를 **`*.md` 정본에서만** 취함. `*.en.md`는 동작 근거로 삼지 않음(읽더라도 규칙 source 아님). 정본과 companion이 충돌하면 **`*.md` 정본 우선**.
*   **커스터마이즈 방법**: 프로젝트 규칙 변경은 **`ProjectRule.md`(사용자 소유, operative, 프로젝트 언어)**에 작성함 — managed 정본이나 companion을 직접 편집하지 않음(managed는 `elf update`가 교체, companion 편집은 무효).
*   **효과**: operative 버전을 정본 하나로 고정하여 번역 차이가 AI 동작을 바꾸지 못하게 함. 국제 사용자도 ProjectRule로 자기 언어로 override 가능.

---

## 2. 문서별 상태와 바뀌는 시점

### 2.1 세션 로그

| 상태(`status`) | 내용 | AI 읽기 |
|------|------|--------|
| `ongoing` | 지금 작성, 수정 중인 세션(예: `2_Log/S/S014_log.md`) | 현재 작업의 근거 |
| `complete` | 종료된 세션. 유효한 기록이며 과거 결론, 경로를 추적할 때 참조 | 읽기 허용 |
| `deprecated` | 폐기한 세션. 헤더 7키와 제목 줄(폐기 사유와 대체)만 남음 | 흔적만 있음. 원문은 보관 파일(§4) |

| 시점 | 동작 | 도구 |
|------|------|------|
| 세션 시작 | `2_Log/S/S###_log.md`(거버넌스는 `2_Log/G/G###_log.md`) 생성, status `ongoing`, goal 기록 | `elf session new --goal`(`--gov`) |
| trial 쓰기 | Phase 1과 Phase 2를 파일로 받아 씀. 로그 손 편집은 내용 절의 오탈자 정도로 자제 | `elf trial new`, `elf trial write`(LogConvention §5.1) |
| 세션 종료 | status `complete`, conclusion과 user_note 기록(사용자 승인 뒤), `Registry.md`와 탐색기 설정 갱신. 파일은 옮기지 않으므로 상대 링크도 그대로 | `elf session close --from` |
| 폐기 | 문서 전체, trial, 절, 표시 블록, 줄 범위의 원문을 지우고 압축 보관, 원래 자리에 흔적(표기 줄) | `elf deprecate` |
| 복원 | 보관 파일의 원문과 원래 값으로 되돌림 | `elf deprecate --restore` |

- 목록: `2_Log/S/Registry.md`, `2_Log/G/Registry.md`는 헤더에서 elf가 만드는 생성 파일(세션, status, 결론 또는 handoff). 세션 상태를 볼 때 사람이 가장 먼저 보는 곳이며, 직접 편집하지 않음. 예전의 Session Registry(`2_Log/Wiki/Session_Registry.tsv`)를 대신함.
- 탐색기: VS Code file nesting 설정으로 진행 중 로그만 펼쳐 보이고 나머지는 `Registry.md` 아래로 접힘(LogConvention §1).

### 2.2 계획 문서

| 위치 | 상태 | 내용 | AI 읽기 |
|------|------|------|--------|
| `1_Concept/12_Planning/` 루트 | 진행 중 | 지금 작성, 수정 중인 계획 문서 | 현재 작업의 근거 |
| `Archive/` | 종료 보관 | 종료된 계획 문서 원본(파일명 그대로 이동). 유효한 기록 | 읽기 허용 |

- 폐기는 로그와 같은 방식(원문 압축 보관과 흔적). 단위는 파일 전체, 표시 블록, 줄 범위(계획 문서는 trial, 절 단위가 없음).

### 2.3 Wiki

| 위치 | 내용 | 갱신 |
|------|------|------|
| `2_Log/Wiki/` | 여러 세션을 묶은 주제 문서. 작업이 끝난 뒤 남는 확정 결론, 파라미터, 규칙을 한두 줄로 요약 | G 세션에서 사용자 승인 뒤(세션 종료 절차에 포함하지 않음) |
| `1_Concept/12_Planning/Wiki/` | 기획 단계 결론과 핵심 규칙 요약 | 계획 문서 작업 때 |

- `Wiki/`의 요약 문서는 원본 로그로 향하는 경로 링크를 포함함(로그는 `2_Log/S/`에 고정되므로 링크가 바뀌지 않음) — 과거 데이터가 필요할 때 그 링크로 원본을 읽음.
- Wiki는 현재 사실의 요약이며, 주제를 찾을 때 사람이 먼저 보는 곳.

---

## 3. Scripts 폴더 관리

`61_Sim/Scripts`, `63_Analysis/Scripts` 안의 스크립트에는 위치 = 상태 규칙을 적용함.

1.  **Active Scripts**: 현재 개발 중이거나 범용으로 쓰이는 최신 스크립트는 Scripts 폴더의 루트에 둠.
2.  **Archived Scripts**: 특정 과거 세션에만 일회용으로 쓰인 스크립트는 `Scripts/Archive/`로 이동함(삭제하지 않음 — 세션 재현에 필요).
3.  **Wiki Tracking**: 스크립트를 Archive로 옮길 때 어떤 세션에서 무엇을 위해 쓰였는지 `Wiki/` 문서에 **경로와 함께 표기**함.

## 4. 읽기 규칙 요약

| 문서 | 현재 판단의 근거로 사용 | 비고 |
|----------|----------------------|------|
| `ongoing` 로그, 계획 문서 루트, `Wiki/`, `Registry.md` | 사용 | 현재 상태, 확정 결론 |
| `complete` 로그, 계획 문서 `Archive/` | 사용(종료된 유효 기록) | 결론, 경로 추적, 과거 파라미터 확인 |
| 폐기 흔적(`deprecated` 로그, 표기 줄) | 사용하지 않음 | 무엇을 시도했다가 폐기했는지는 흔적의 사유와 대체로 앎 |
| 폐기 원문(`.elf/deprecated/`) | 사용하지 않음 | 압축이라 읽기 도구와 검색에 드러나지 않음. 풀어 보는 것(`elf deprecate show`)은 사용자가 요청할 때만. 복원은 사용자 결정 |

- 사용자가 경로를 지정해 "이 파일을 열어 요약해 줘"라고 요청하면 위치와 무관하게 그 파일을 읽음(폐기 원문은 `elf deprecate show`).
- 폐기 내용을 다시 쓰려면 `elf deprecate --restore`로 복원한 뒤 사용함(보관된 원문을 풀어 복사해 쓰지 않음).
