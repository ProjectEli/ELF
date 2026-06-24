# AI PARA Framework & Context Management

이 문서는 프로젝트 내의 방대한 실험 로그와 기획 문서들이 AI 에이전트(예: Claude Code)의 컨텍스트 윈도우(Context Window)를 오염시키는 현상(Hallucination)을 막고, 인간-AI 협업 시 최적의 효율을 내기 위한 **AI 맞춤형 PARA (Projects, Areas, Resources, Archives) 파일 관리 규칙**을 정의함.

## 1. 파일 격리 원칙 (The Firewall Principle)

이 프로젝트의 최상단 폴더에는 `.claudeignore` 파일이 존재함.
이 파일은 AI가 파일 시스템을 자율적으로 검색하거나 읽어들일 때, 특정 이름의 폴더들을 아예 **투명 인간 취급**하여 인식하지 못하도록 차단(Firewall)함.

*   **차단 대상**: `Archive/` 및 `*Archive*` 등 과거의 폐기되거나 종료된 기록물.
*   **효과**: AI가 "현재 유효한 프로젝트의 상태"만을 바탕으로 대답할 수 있게 보장하며, 예전의 실패한 세팅값(예: 잘못된 파라미터 맵, 폐기된 논문 전개 방향안)을 현재의 사실로 오인하는 것을 완벽히 방지함.

### 1.1 정보용 companion 격리 (Informative Companion)

`*.en.md`(일반화 `*.<lang>.md`) 형식 파일은 거버넌스 문서의 **정보용 번역(read-only human companion)**임 — 국제 사용자의 *독해*용이며 operative source가 아님. operative 정본은 항상 동명 `*.md`(PROJECT_LANG 정본, 기본 한국어)임.

*   **AI 동작 규칙**: AI는 규칙·구조·지시를 **`*.md` 정본에서만** 취함. `*.en.md`는 동작 근거로 삼지 않음(읽더라도 규칙 source 아님). 정본과 companion이 충돌하면 **`*.md` 정본 우선**.
*   **커스터마이즈 경로**: 프로젝트 규칙 변경은 **`ProjectRule.md`(사용자 소유·operative·프로젝트 언어)**에 작성함 — managed 정본이나 companion을 직접 편집하지 않음(managed는 `elf update`가 교체, companion 편집은 무효).
*   **효과**: operative 버전을 정본 하나로 고정하여 번역 차이가 AI 동작을 바꾸지 못하게 차단함. 국제 사용자도 ProjectRule로 자기 언어로 override 가능.

## 2. 하이브리드 PARA 구조 (Focus-and-Filter)

인간을 위한 '인지 부하 감소(Grouping)'와 AI를 위한 '경로 평탄화(Flattening)'를 동시에 달성하기 위해, 프로젝트는 **하이브리드 구조**로 운용됨.

### 작업대 (Default Root / Active Sandbox)
*   **목적**: 지금 당장, 혹은 이번 주에 유효하게 "진행 중(In-Progress)"인 내용이 위치하는 곳임.
*   **파일 예시**: `Current_Analysis_Task.md`, `S014_log.md`
*   **운영 규칙**: 별도의 `1_Active` 폴더를 만들지 않고, **작업 중인 부모 폴더(예: `12_Planning/` 또는 `2_Log/`)의 최상단(Root)**을 바로 작업대로 사용함. 작업이 완료되어 과거의 기록이 되기 전까지는 이 공간에서 자유롭게 작성함.

### `Wiki/` (안식처 / Human Sanctuary)
*   **목적**: 작업대에서 작업이 끝난 후 얻어낸 **"변하지 않는 사실, 결론, 핵심 규칙"**만을 한두 줄로 요약해 모아두는 곳임.
*   **운영 규칙**: 인간 연구자가 "현재 제일 중요한 팩트/규칙" 만을 모아서 열람하고 싶을 때 이 폴더에 접근함. AI에게도 핵심 요약 컨텍스트를 제공하는 진실 공급원(Fact-Sheet) 역할을 함.

### `Archive/` (AI 방화벽 / The Firewall Bin)
*   **목적**: 작업이 완전히 종료(Complete)되거나 폐기(Deprecated)되어 더 이상 "현재의 관심사"가 아니지만, 나중에 참고할 일이 있을 수 있는 원본 로그 기록을 보관함.
*   **운영 규칙**: 폴더 최상단이 지저분해지면, 구형 파일을 **파일명 그대로** 이 폴더로 이동시킴(접두 태그 불요 — **폴더 위치가 곧 상태**). 이 폴더에 들어가는 순간, `.claudeignore`에 의해 AI의 자율 검색망에서 완벽하게 사라짐.

---

## 3. Scripts 폴더 관리 (코드 아카이빙)

`61_Sim/Scripts`나 `63_Analysis/Scripts` 내부에 스크립트가 무분별하게 쌓이는 것을 방지하기 위해 동일한 PARA 논리를 스크립트에도 적용함.

1.  **Active Scripts**: 현재 개발 중이거나 범용적으로 쓰이는 최신 스크립트는 Scripts 폴더의 최상단(Root)에 유지함.
2.  **Archived Scripts**: 특정 과거 세션에만 일회용으로 쓰였던 스크립트는 `Scripts/Archive/` 밑으로 이동시킴.
3.  **Wiki Tracking (Registry)**: 스크립트를 Archive로 옮길 때는, 해당 스크립트가 어떤 세션에서 무엇을 위해 쓰였는지 `Wiki/` 문서에 **경로와 함께 표기**함.

## 4. AI 접근 복원 방법 (How to bypass the firewall)

AI는 `Archive`를 스스로 뒤져볼 수 없지만, 인간 개발자가 특정 과거 기록의 복원/분석을 요구할 때는 **명시적 지시(Explicit Instruction)**를 통해 열람 가능함.

### 방법: 절대/상대 경로 강제 지정
사용자가 프롬프트 상에서 "과거의 A 파일을 열어봐"라며 정확한 경로 스니펫을 주면 AI는 정상적으로 해당 파일을 읽고 컨텍스트에 불러올 수 있음.

*   *(사용자 프롬프트 예시)*: "`2_Log/Archive/S005_log.md` 파일을 열어서 당시의 파라미터 값 추이를 요약해 줘."
*   이러한 방식을 돕기 위해, `Wiki`의 지식 문서들은 과거 데이터가 필요할 경우를 대비하여 항상 `Archive/...`로 향하는 **명시적 파일 경로 링크**를 포함해야 함.
