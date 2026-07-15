# EliRule: Project Structure & Operational Guide (general)

본 문서는 ELF `general` 프로젝트(목표지향 비연구 프로젝트 — 도구 개발·제안서 준비·학습·구축 등)의 폴더 구조와 운영 규칙을 정의합니다. README.md가 철학·개요를, 이 문서가 실무 규격을 담당합니다.

> `general` = 학술연구가 아닌 **명확한 목표를 가진 다세션 프로젝트**. 연구 preset(`6_Exp`·`7_Paper`·figure·시뮬)과 분리됩니다. 도메인별 세부 규약은 `ProjectRule.md`에 작성합니다.

---

## 1. 폴더 구조

### Core (항상 포함)

#### `.elf/` — ELF 제어 영역
`elf` CLI가 프로젝트 상태(버전·설정·관리 파일 목록)를 기록하고, ELF 관리 규칙 정본 payload를 `.elf/managed/`에 배치합니다. **직접 수정 금지** — `elf init`/`elf update`가 관리합니다.
- `.elf/managed/EliRule.md`: 이 문서 (구조·운영 가이드)
- `.elf/managed/LogConvention.md`: 로깅 표준 규칙
- `.elf/managed/AI_PARA_Framework.md`: 상태 기반 파일 관리·아카이빙 규칙. AI가 프로젝트를 탐색할 때 가장 중요한 기준 문서
- `.elf/managed/templates/`: 마크다운 스텁 (아래)

#### `0_Meta/` — 프로젝트 거버넌스 (사용자 영역)
프로젝트 전용 규칙과 커스터마이즈를 두는 메타 구역입니다. `elf update`가 접근하지 않습니다.
- `ProjectRule.md`: 프로젝트 전용 규칙·목표 (**사용자 소유** — 프로젝트에 맞게 자유 수정)
- `<이름>.project.md`: data overlay (§2.4 — LLMcliche 항목 커스터마이즈)
- 그 외 프로젝트 재량(스크립트·설정 등)

#### `1_Concept/` — 기획 & 아이디어
- **`12_Planning/`**: 프로젝트 목표·로드맵·계획 (`P###_제목.md` 넘버링). `Wiki/`: 기획 단계 결론 요약.
- **`13_Ideas/`**: 세션화하기엔 작은 snippet·초기 아이디어. flat(Archive 없음, 폐기하지 않음). 성숙 시 Planning 문서나 세션 trial로 승격.

#### `2_Log/` — 세션 로그
모든 작업을 기록하는 세션 로그의 최상위 공간입니다.
- `S###_log.md`: 세션 로그 (포맷: `.elf/managed/LogConvention.md` 참조)
- `Wiki/`: 핵심 발견 요약 + Session Registry
- `Archive/`: 완료된 세션 로그 보관

#### `.elf/managed/templates/` — 마크다운 스텁
- `sessionTemplate.md`: 새 세션 시작 시 `2_Log/S###_log.md`로 복사
- `trialTemplate.md`: 진행 중 세션에 trial(t##) 추가 시 본문에 붙여넣기
- **Planning 문서(`P###`)는 의도적 무템플릿** — trial은 재현성 위해 템플릿으로 규율하나, planning은 연구자 자유 탐색이라 형식을 강제하지 않음.
- 루트 `templates/`는 ELF가 생성·관리하지 않는 프로젝트 재량 폴더입니다.

### 도메인 폴더 (사용자 추가)
프로젝트 성격에 맞는 작업 폴더(예: `src/`·`docs/`·`assets/`)는 **사용자가 직접 추가**합니다. ELF는 spine(`.elf` 제어판·세션 로그·규약)을 제공하고, 도메인 산출물 구조는 프로젝트가 정의합니다.

---

## 2. 운영 규칙

> **기록 원칙 (Traceability)**: 가급적 **모든 작업·변경·결정은 세션 로그(`2_Log/S###`)에 기록**함을 원칙으로 한다. 의미 있는 코드·데이터·문서 변경과 시도·결과(실패 포함)는 그 작업 turn에 남겨 재현·추적성을 확보한다 — 사소한 수정(오타·경로)까지 강제하지는 않으나 산출물·결론·방향에 영향을 주는 변경은 누락하지 않는다. 상세 포맷·base-delta 규칙은 `LogConvention.md` 참조.

### 2.1 Git 분리 전략
- **Git 추적 대상**: 코드, 메타데이터, 로그, 산출물 등 프로젝트 산출물 전반.
- **Git 추적 제외**: 대용량 바이너리, 도구 임시 파일. 대용량 자산은 Git LFS 또는 별도 관리 권장.

### 2.2 Naming Convention
- **Session-Trial**: `S###_t##` (예: `S001_t1`).
- 파일 이름에 조건·변수 정보 나열 **금지** — 모든 조건은 로그에 기록.
- Planning 문서: `P###_제목.md`.

### 2.3 Cross-Reference 규칙
- 로그에서 Planning 문서 참조: `→ see 1_Concept/12_Planning/P###_xxx.md`.

### 2.4 ELF 관리 파일과 갱신 (`elf update`)

프로젝트 파일은 소유권에 따라 세 가지로 나뉘며, `elf update`는 이 구분을 엄격히 지킵니다:

| 구분 | 파일 | `elf update` 동작 |
|------|------|-------------------|
| **ELF 관리** | `.elf/managed/`의 `EliRule.md`·`LogConvention.md`·`AI_PARA_Framework.md`·`LLMcliche.md`·`templates/*`(companion 포함), 루트 `.claudeignore`·`AGENTS.md` | 새 버전으로 교체. **직접 수정한 경우 덮어쓰지 않고** `<파일>.elf-new`로 생성(병합은 사용자 몫, `--force`로 강제 교체) |
| **사용자 소유** | `ProjectRule.md`, `Session_Registry.tsv`, `README.md`, 모든 작업 데이터·로그 | **절대 미접근** |
| **포인터(생성만)** | `CLAUDE.md` | 없으면 생성, 있으면 **절대 불변경**(내용 무관 — 기존 파일 소유권 존중). `@AGENTS.md` 로드 여부는 `elf doctor`가 점검 |
| **부분 관리** | `.gitignore` | 마커블록(`# >>> ELF managed >>>` ~ `# <<< ELF managed <<<`) 안쪽만 교체, 블록 밖 사용자 규칙 보존 |

- 프로젝트 규칙 커스터마이즈는 ELF 관리 파일을 고치는 대신 **`ProjectRule.md`에 작성**하는 것을 권장합니다(갱신 충돌 없음).
- 상태 확인: `elf status` (변경 없이 진단만, `--check`는 CI/훅 게이트용).

#### Data overlay — `<이름>.project.md` (data 파일 커스터마이즈)

data 성격의 ELF 관리 파일(manifest에 `overlayable`로 명시 — 현행: `LLMcliche.md`)은 직접 수정 대신 **project overlay**로 커스터마이즈합니다. prose 규칙 커스터마이즈는 종전대로 `ProjectRule.md`이며, 구조·규약 파일(LogConvention·템플릿 등)은 overlay 대상이 아닙니다.

- **파일**: `0_Meta/<이름>.project.md`(예: `0_Meta/LLMcliche.project.md`) — **사용자 소유**, `elf update` 미접근(갱신 충돌·`.elf-new` 없음).
- **유효 규칙 = base ⊕ overlay**: AI 에이전트는 base(ELF 관리 정본)와 overlay를 병행 로드하여 병합 적용합니다.
  - `## 추가 (add)`: 항목 추가 — 유효 목록 = base ∪ add.
  - `## 제외 (remove)`: base 항목의 **항목 단위** 제외 — 각 항목에 **사유 필수 기재**. 절·파일 단위 무력화 금지.
  - `## 재정의 (override)`: base 항목을 프로젝트 정의로 교체.
- **등록 규율**: overlay 항목의 등록·수정은 사용자 승인 경유 — AI가 자율 등록하지 않습니다. 일반화 가치가 있는 항목은 ELF 코어 반영을 제안합니다.
- 점검: `elf doctor`가 overlay 인지·제외 사유 유무·비허용 대상 overlay를 진단합니다.

### 2.5 세션 소유권 — 1세션 1작성자 (멀티에이전트)

- **세션 로그 1개의 작성자는 항상 1명**(에이전트 또는 사람). 같은 세션 로그를 복수 에이전트·터미널이 동시에 작성하지 않습니다 — 세션 로그는 단일 사고흐름의 선형 기록이며, 동시 작성은 trial 추가·헤더 Handoff 갱신에서 조용한 상호 덮어쓰기(lost update)를 일으킵니다.
- 병렬 작업은 **에이전트별 세션 분리**: 각자 `elf session new`로 번호를 받습니다. 관계는 번호가 아니라 헤더 `관련:` 필드가 담당합니다(참조 사유 1구 병기, 예: `S200(분기 원본 — 접근 A 담당)`).
- 타 에이전트의 세션 로그는 **읽기 전용** 참조 — 남의 로그·Handoff를 수정하지 않습니다.
- 병렬 세션들의 결론 통합은 별도 세션(또는 원본 세션)에서 **단일 작성자**로 수행하고, 입력이 된 세션들을 `관련:`에 명시합니다.

---

## 3. AI Communication Rules

> **PROJECT_LANG**: `.elf/config.json` 의 `lang` 필드(BCP-47 태그, 예: `ko-KR`) 참조. AI 에이전트는 해당 값에 따라 응답 언어를 결정함.

프로젝트 내 모든 AI Agent는 사용자와 소통하고 문서를 작성할 때 다음 원칙을 준수합니다:

1. **응답 언어 (Response Language)**: AI 에이전트는 `PROJECT_LANG`에 지정된 언어와 English 두 가지로 응답합니다. 로그, 문서 작성 시에도 동일하게 `PROJECT_LANG` 언어를 사용합니다. 기술 용어는 English 원문을 병기할 수 있습니다.
2. **객관적이고 드라이한 문체 유지**: 불필요한 인삿말, 과도한 칭찬, 주관적 감정 표현, 과장된 형용사 사용을 금지합니다.
3. **비유 금지**: 비유나 은유를 금지하고, 직관적이고 객관적인 학술/엔지니어링 용어로만 사실을 전달합니다. 특히 관점·접근을 '렌즈(lens)'로, 분석·탐색을 '항해(navigate)'·'깊이 파다(deep dive)'·'여정(journey)'으로 치환하는 LLM 상투 은유를 배제하고 직접 용어(관점·접근·분석·과정)를 사용합니다.
4. **결론 중심의 명확한 전달**: 분석 결과와 Action Item을 간결하고 명확하게 제시하며, 논리적이고 정교한 팩트만을 다룹니다.
5. **산출물 재현성 (Reproducibility)**: 작업 산출물(코드·문서·데이터)과 중요한 중간 결과는 **재현 가능하도록 적절한 형식·위치에 보존**합니다. 덮어쓰기로 이전 버전을 유실하지 않습니다.
6. **과장 및 감정적 수식어 금지 (No Embellishment)**: '압도적(Overwhelming)', '무기(Weapon)', '치명적', '파급력' 등 감정을 자극하거나 극단적인 수식어의 사용을 전면 배제합니다. 오직 정량적 수치와 인과관계로만 장단점을 서술합니다.
7. **이모지 사용 금지 (No Emojis)**: 아이콘이나 이모지(Emoji)를 어떠한 문서나 응답에도 사용하지 않습니다.
8. **토큰 최소화 및 압축 (Token Economy)**: AI 통신 및 로깅의 기본 철학은 '토큰의 최소화'입니다. 완전한 문장형 서술보다는 단어 중심의 압축된 개조식(Bullet points)을 사용하며, 불필요한 조사와 접속사는 생략합니다.
9. **문장 종결 방식 (Formatting)**: 한글 기록 시 '~입니다/습니다'나 해요체 등은 일절 배제하며, 서술이 필요한 경우에도 반드시 명사형 종결어미('-음', '-함', '-임')나 간결한 '-다'로 끝을 맺습니다. 비한글(예: 영어) 출력에는 본 종결 형식이 적용되지 않으며, 2·8항의 드라이·압축 원칙(간결·능동·filler 없는 문체)을 등가 의도로 적용합니다.
10. **구조화된 로깅 (Structured Logging)**: 관찰(사실)과 분석(해석)을 명확히 분리하여 서술하며, 생성된 파일 목록이나 조건 등은 나열식 서술 대신 반드시 마크다운 표(Table) 형식으로 압축하여 정보 밀도를 극대화합니다.
11. **약어 표기 (Abbreviations)**: 약어는 **첫 사용 시 full name 병기**(`AR (asymmetry ratio)`). 약어가 반복되는 trial·문서는 `### 조건`(또는 문서 상단)에 **약어 legend를 ul list**로 명시하되 **각 약어를 별도 row**로 — trial atom 단독 열람성 확보. 동일 약어가 여러 trial에 걸치면 **각 trial에서 재정의**(atom 독립성 > DRY; 세션 1회 정의로 갈음 금지). 예외: 도메인 표준 단위·기호(µA·Hz·ms·SI)는 병기 불요.
12. **LLM 상투 표현 배제 (LLM Cliché Ban)**: 영어 문서·communication 작성 시 LLM 특유의 상투 어휘·register(특징적 동사·막연 형용사·상투 명사·과용 연결어·정형 구문)를 배제하고 **구체·능동·직접 서술**로 대체합니다. 고정 목록이 아닌 *원칙* 적용 — 막연한 filler·과용 register는 배제하되 **정확한 기술적 의미**의 용어는 허용합니다(예: 통계 significance, robustness). 연결어는 과용·문두 연속만 배제(정당한 단일 사용 허용), 인용·제목·원문 표현은 면제하며, 한국어 dry 로그는 무관합니다(영어 혼용 시 적용). 프로젝트별 기술 동음어 예외는 `ProjectRule.md`에, 어휘 항목의 추가·제외·재정의는 data overlay `0_Meta/LLMcliche.project.md`(base ⊕ overlay)에 선언합니다. 배제·예외·전후 예시(비망라 참고): `.elf/managed/LLMcliche.md`.
13. **출처 신뢰성 (Source Reliability)**: 나무위키(namu.wiki) 등 익명·집단 편집 위키 및 출처 불명 블로그·커뮤니티를 답변·문서의 출처로 인용 금지. 검색 결과에 떠도 그대로 신뢰하지 말고 **신뢰 출처**(공신력 기관·학회·정부, 학술/1차 자료, 공식 문서, 1차 보도)로 교차검증 후 *그 신뢰 출처*를 인용. WebSearch 시 `blocked_domains: ["namu.wiki"]` 기본 배제. 위키백과도 출발점일 뿐 1차 출처로 추적·확인. **웹을 근거로 한 답변·문서에는 검증한 신뢰 출처를 `## 출처`(또는 `## Sources`)로 명시.**
