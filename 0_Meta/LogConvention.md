# LogConvention: ELF 로깅 표준 규칙

본 문서는 ELF 프로젝트에서 사람과 AI 에이전트 모두가 따라야 할 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

---

## 1. 로그 파일 위치 및 명명

| 항목 | 규칙 |
|------|------|
| **위치** | `5_Exp/53_Analysis/Logs/S{NNN}_log.md` |
| **번호** | 3자리 zero-padding (S001, S007, S010, ...) |
| **내용** | 실험 파라미터, 실행 결과, 에러/해결 등 **순수 Metadata (Fact-Sheet)** |
| **금지** | 아이디어, 기획, 방향성 논의 → `1_Concept/`로 분리 (EliRule.md 참조) |

---

## 2. 로그 포맷

```markdown
# S{NNN}: {세션 제목}
**Date**: YYYY-MM-DD
**Status**: {Planning | In Progress | Step X 완료 | Complete}

## 요약정보
- 목표:
- 배경:
- 교훈:

## 상세내용

### t{NN}: {작업 제목}
- 목표:
- 교훈:

{실행 내용, 파라미터, 결과, 이미지 링크 등}
```

### 규칙
- **ticket 번호**: `t01`, `t02`, ... 순서대로. 중복 금지
- **이미지 경로**: `![alt text](../../51_Sim/Data/S{NNN}/filename.png)` (Logs 기준 상대경로)
- **Figure 인라인 임베딩 필수**: 분석 결과로 생성된 Figure는 반드시 로그 본문의 **해당 결과 섹션에 인라인으로 삽입**하고, alt text에 Figure 번호와 1줄 설명(축, 핵심 관찰)을 기재한다. 파일 목록 테이블에만 나열하고 본문에 임베딩하지 않는 것은 금지. 향후 사람이 로그만 읽고도 결과를 시각적으로 파악할 수 있어야 한다.
- **코드 사용법**: 코드 블록 (```lang ... ```) 으로 기재
- **파라미터 표**: 변수명, 값, 단위를 표 형태로 정리

---

## 3. 결과 파일 저장 규칙

### 3.1 스크립트

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 스크립트 | `51_Sim/Scripts/` | `S{NNN}_sim.m` |
| 후처리 스크립트 | `51_Sim/Scripts/` 또는 `53_Analysis/Scripts/` | `S{NNN}_postProcess.m` |
| 헬퍼 함수 | `51_Sim/Scripts/` | `{function_name}.m` |
| 분석 코드 | `53_Analysis/Scripts/` | `S{NNN}_analysis.m` |

### 3.2 데이터

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 결과 | `51_Sim/Data/S{NNN}/` | `S{NNN}_sim_results.mat` |
| 그래프 | `51_Sim/Data/S{NNN}/` | `S{NNN}_Fig1_*.png` |
| 실측 원본 | `52_Empirical/Raw/` | Read-Only |
| 가공 데이터 | `52_Empirical/Processed/S{NNN}/` | `S{NNN}_t##_processed.csv` 등 |

### 3.3 명명 규칙
- 시뮬 결과: `S{NNN}_{ticket}_{description}.mat`
- 그래프: `S{NNN}_{description}.png`
- 스크립트 ticket 구분 필요 시: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `5_Exp/54_Viz/` | 스크립트에서 자동 저장 |
| 논문용 Figure | `6_Paper/61_Figs/` | rawFig → processedFig → finalFig |

---

## 4. AI_Sync.md 업데이트 규칙

작업을 **완료**할 때마다 `0_Meta/AI_Sync.md`에 항목을 추가합니다.

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

| From | To | 상대경로 (Logs 기준) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/13_Planning/P00x.md` | `../../../1_Concept/13_Planning/P00x.md` |
| Logs → Sim Data | `51_Sim/Data/S{NNN}/` | `../../51_Sim/Data/S{NNN}/` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Planning으로 분리된 내용은 원본 Log에 blockquote stub으로 표시:
```markdown
> 상세 내용은 Planning 문서로 분리되었습니다.
> - [P00x_Title.md](../../../1_Concept/13_Planning/P00x_Title.md) — 섹션 N 참조
```

---

## 6. 세션 생성 시 체크리스트

AI 에이전트가 새 세션(S{NNN})을 시작할 때:

- [ ] `53_Analysis/Logs/S{NNN}_log.md` 생성 (위 포맷 준수)
- [ ] `51_Sim/Data/S{NNN}/` 디렉토리 생성
- [ ] 시뮬/실험 스크립트 `51_Sim/Scripts/S{NNN}_*.m` 작성
- [ ] 실행 후 결과 .mat + .png 저장
- [ ] Log에 ticket (t{NN}) 추가: 파라미터, 결과, 이미지 경로
- [ ] `AI_Sync.md` 업데이트
- [ ] Planning 내용이 포함된 경우 → `1_Concept/`로 분리 + cross-reference
