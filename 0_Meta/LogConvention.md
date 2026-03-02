# LogConvention: ELF 로깅 표준 규칙

본 문서는 ELF 프로젝트의 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

---

## 1. 로그 파일 위치 및 명명

| 항목 | 규칙 |
|------|------|
| 위치 | `5_Exp/53_Analysis/Logs/` |
| 파일명 | `S{NNN}_log.md` (예: `S001_log.md`) |
| 생성 시점 | 새 세션(Session) 시작 시 |

---

## 2. 로그 포맷

```markdown
Goal: [세션 목표 — 한 줄 요약]
Base: [Baseline 조건 — 고정 변수 나열]

t01: [변경 변수(Delta) 설명]
- 결과: [관찰 사실]
- 교훈: [해석 및 다음 액션]
→ see [Cross-reference 경로] (선택)

t02: [변경 변수(Delta) 설명]
- 결과: ...
- 교훈: ...
```

### 규칙
- **Goal/Base**: 세션 시작 시 한 번만 선언
- **t## (Ticket/Trial)**: 의도적으로 변경한 변수(Delta)와 관찰 결과만 기록
- **교훈**: 사실 기반 해석. 아이디어/가설은 `1_Concept/11_Ideas/`로 분리
- **Cross-reference**: 관련 Planning, 시뮬레이션, 스크립트 경로를 `→ see` 형식으로 링크

---

## 3. 결과 파일 저장 규칙

### 3.1 스크립트
| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬레이션 코드 | `5_Exp/51_Sim/Scripts/` | `S###_sim.m` |
| 분석 코드 | `5_Exp/53_Analysis/Scripts/` | `S###_analysis.m` |

### 3.2 데이터
| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬레이션 결과 | `5_Exp/51_Sim/Data/S###/` | `S###_t##.mat` 등 |
| 실측 원본 | `5_Exp/52_Empirical/Raw/S###/` | `S###_t##.csv` 등 |
| 가공 데이터 | `5_Exp/52_Empirical/Processed/S###/` | `S###_t##_processed.csv` 등 |

### 3.3 시각화
| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `5_Exp/54_Viz/` | 스크립트에서 자동 저장 |
| 논문용 Figure | `6_Paper/61_Figs/` | rawFig → processedFig → finalFig |

---

## 4. AI_Sync.md 업데이트 규칙

AI 에이전트가 작업을 수행할 때마다 `0_Meta/AI_Sync.md`에 핸드오프 엔트리를 추가합니다.

### 업데이트 시점
- 세션 작업 완료 시
- 에이전트 전환(Claude → Gemini 등) 시
- 중요한 마일스톤 달성 시

### 엔트리 포맷
```markdown
## [날짜] [세션ID] — [요약]
- Agent: [에이전트명]
- 수행한 작업: ...
- 생성/수정 파일: ...
- Next Steps: ...
```

---

## 5. Cross-Reference 규칙

로그와 다른 문서 간 추적성을 확보하기 위한 상호 참조 규칙입니다.

| From | To | 형식 |
|------|----|------|
| Logs → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` | |
| Logs → Sim Data | `→ see 5_Exp/51_Sim/Data/S###/` | |
| Logs → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` | |
| Planning → Logs | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` | |

---

## 6. 새 세션 시작 체크리스트

1. `5_Exp/53_Analysis/Logs/`에 `S{NNN}_log.md` 생성
2. Goal/Base 섹션 작성
3. 필요 시 `1_Concept/13_Planning/`에 Planning 문서 생성
4. 작업 완료 후 `0_Meta/AI_Sync.md` 업데이트 (AI 에이전트 사용 시)
