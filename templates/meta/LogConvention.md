# LogConvention: ELF 로깅 표준 규칙

본 문서는 ELF 프로젝트에서 사람과 AI 에이전트 모두가 따라야 할 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

---

## 1. 로그 파일 위치 및 명명 (PARA 워크플로우 적용)

모든 연구 및 기획 진행은 PARA(Projects, Areas, Resources, Archives) 관리 체계와 `.claudeignore` 룰을 따릅니다. 자세한 원칙은 `0_Meta/AI_PARA_Framework.md`를 참조하십시오.

| 항목 | 규칙 |
|------|------|
| **진행 중 (Active Sandbox)** | 새로운 세션(S{NNN})을 시작할 때는 무조건 **`2_Log/`의 최상단(Root)**에 로그(`S{NNN}_log.md`)를 생성하여 작성합니다. |
| **결론 요약 (Wiki)** | 세션이 완료되면 핵심 교훈이나 산출된 파라미터를 `1_Wiki/` 폴더 내의 지식 문서에 한두 줄로 요약합니다. 이 때 반드시 원본 Archive 로그 파일의 절대/상대 경로 링크를 포함합니다. |
| **보관 (Archive)** | 세션이 종료되면 전체 원본 로그 파일은 **반드시** `9_Archive/` 폴더로 이동시킵니다. (예: `9_Archive/[Archived]_S{NNN}_log.md`) |
| **내용 규칙** | 시뮬레이션 파라미터, 실행 결과, 에러/해결 등 **순수 Metadata (Fact-Sheet)**만을 기록합니다. |
| **금지 사항** | 아이디어, 논문 기획, 방향성 논의는 `2_Log/`에 기록하지 말고 `1_Concept/12_Planning/`의 기획 문서로 분리합니다. |

---

## 2. 로그 포맷

```markdown
# S{NNN}: {세션 제목}

> **Created**: YYYY-MM-DD
> **Modified**: YYYY-MM-DD
> **Status**: {★ 활성 | In Progress | Complete}
> **목표**: {세션의 핵심 목표 1-2문장}
> **관련**: {관련 세션/문서 링크}
> **Handoff**: {현재 상태; 미완료 작업; 참조 파일}

---

## t{NN}: {작업 제목}

### 목표
- {구체적 작업 목표 명시}

### 조건
- {파라미터, 설정, 제약 등을 표나 짧은 리스트로 기술}

### 관찰 (Observation)
- {현상, 팩트 중심 결과 나열}
- {Figure 인라인 삽입}

### 해석 (Interpretation)
- {관찰된 팩트에 대한 물리적/논리적 해석}

### 교훈
- {핵심 인사이트, 향후 유의점}

### 생성 파일

| 유형 | 파일 |
|------|------|
| Script | `경로` |
| Output | `경로` |
| Figure | `경로` |
```

### 규칙
- **작성 언어**: 로그는 `0_Meta/EliRule.md`의 `PROJECT_LANG` 설정에 따른 언어로 작성하되, **토큰 최소화**를 위해 반드시 명사형 종결어미('-음/함/임')와 단어 중심 개조식을 사용한다. 기술 용어인 경우 영어 병기 가능.
- **Status**: `★ 활성` (현재 작업 중), `In Progress` (중간 단계), `Complete` (완료, 아카이빙 전)
- **Handoff**: 세미콜론(`;`)으로 3파트 구분 — `현재 상태; 미완료 작업; 참조 파일`. 세션 진행 중 수시 갱신. 다음 세션이 `Read(offset=0, limit=9)` 패턴으로 이 필드만 읽어 맥락을 이어받을 수 있도록 함. 초기값 `-`
- **ticket 번호**: `t01`, `t02`, ... 순서대로. 중복 금지
- **이미지 경로**: `![alt text](../6_Exp/64_Viz/S{NNN}/filename.png)` (2_Log 기준 상대경로)
- **Figure 인라인 임베딩 필수**: 분석 결과로 생성된 Figure는 반드시 로그 본문의 **해당 결과 섹션에 인라인으로 삽입**하고, alt text에 Figure 번호와 1줄 설명(축, 핵심 관찰)을 기재한다. 파일 목록 테이블에만 나열하고 본문에 임베딩하지 않는 것은 금지.
- **생성 파일 테이블**: 각 task에서 생성된 자료를 테이블로 정리해 가독성 및 토큰 효율 극대화. 스크립트, 데이터, Figure를 테이블로 정리. 유형(`Script`, `Output`, `Figure`, `Config` 등)과 프로젝트 루트 기준 상대경로를 명시.
- **코드 사용법**: 코드 블록 (```lang ... ```) 으로 기재
- **파라미터 표**: 변수명, 값, 단위를 표 형태로 정리

---

## 3. 결과 파일 저장 규칙

### 3.1 스크립트

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 스크립트 | `61_Sim/Scripts/` | `S{NNN}_sim.m` |
| 후처리 스크립트 | `61_Sim/Scripts/` 또는 `63_Analysis/Scripts/` | `S{NNN}_postProcess.m` |
| 헬퍼 함수 | `61_Sim/Scripts/` | `{function_name}.m` |
| 분석 코드 | `63_Analysis/Scripts/` | `S{NNN}_analysis.m` |

### 3.2 데이터

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 결과 | `61_Sim/Data/S{NNN}/` | `S{NNN}_sim_results.mat` |
| 그래프 | `61_Sim/Data/S{NNN}/` | `S{NNN}_Fig1_*.png` |
| 실측 원본 | `62_Empirical/Raw/` | Read-Only |
| 가공 데이터 | `62_Empirical/Processed/S{NNN}/` | `S{NNN}_t##_processed.csv` 등 |

### 3.3 명명 규칙
- 시뮬 결과: `S{NNN}_{ticket}_{description}.mat`
- 그래프: `S{NNN}_{description}.png`
- 스크립트 ticket 구분 필요 시: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `6_Exp/64_Viz/` | 스크립트에서 자동 저장. 산출 후 `0_Meta/scripts/elf_gallery` 실행하여 `_gallery.md` 자동 갱신 권장. |
| 논문용 Figure | `7_Paper/71_Figs/` | rawFig → processedFig → finalFig |

---

## 4. AI_Sync.md 업데이트 규칙

AI 에이전트는 매 작업(쿼리) 종료 시마다 자동으로 업데이트하지 않고, **사용자의 명시적 업데이트 요청(Explicit Request)**이 있을 때에 한하여 과거 대화 내역을 취합해 `0_Meta/AI_Sync.md`에 항목을 추가함.

### 포맷
```markdown
## [YYYY-MM-DD / {Agent Name}] {작업 제목}

### 수행한 작업
- 생성/수정/삭제한 파일 목록 (표 형태)
- 핵심 결과 요약

### Next Steps
- 다음 에이전트가 이어서 수행할 구체적 Task
- 필요한 파일 경로, 함수명 등 명시
```

### 규칙
- **최신 항목이 위**에 오도록 작성 (역순)
- 파일 경로는 프로젝트 루트 기준 상대경로 사용
- Next Steps에는 구체적인 파일명, 함수 시그니처, 예상 입출력을 포함

---

## 5. Cross-reference 규칙

| From | To | 상대경로 (2_Log 기준) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/12_Planning/P00x.md` | `../1_Concept/12_Planning/P00x.md` |
| Logs → Sim Data | `6_Exp/61_Sim/Data/S{NNN}/` | `../6_Exp/61_Sim/Data/S{NNN}/` |
| Logs → Viz | `6_Exp/64_Viz/S{NNN}/` | `../6_Exp/64_Viz/S{NNN}/` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Planning으로 분리된 내용은 원본 Log에 blockquote stub으로 표시:
```markdown
> 상세 내용은 Planning 문서로 분리되었습니다.
> - [P00x_Title.md](../1_Concept/12_Planning/P00x_Title.md) — 섹션 N 참조
```

---

## 6. 세션 생성 시 체크리스트

AI 에이전트가 새 세션(S{NNN})을 시작할 때:

- [ ] `2_Log/S{NNN}_log.md` 생성 (위 포맷 준수)
- [ ] `6_Exp/61_Sim/Data/S{NNN}/` 디렉토리 생성
- [ ] 시뮬/실험 스크립트 `6_Exp/61_Sim/Scripts/S{NNN}_*.m` 작성
- [ ] 실행 후 결과 .mat + .png 저장
- [ ] Log에 ticket (t{NN}) 추가: 파라미터, 결과, 이미지 경로
- [ ] 작업 완료 시: 얻은 지식을 `2_Log/1_Wiki/`에 요약하고, `S{NNN}_log.md`는 `2_Log/9_Archive/`로 이동
- [ ] 스크립트 완료 시: 1회용 스크립트는 `Scripts/9_Archive/`로 이동, 범용은 Root 유지
- [ ] `2_Log/1_Wiki/Session_Registry.tsv`에 해당 세션 항목 추가 업데이트
- [ ] Planning 내용이 포함된 경우 → `1_Concept/`로 분리 + cross-reference
