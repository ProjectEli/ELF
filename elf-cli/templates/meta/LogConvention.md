# LogConvention: ELF 로깅 표준 규칙

ELF 프로젝트에서 사람과 AI 에이전트가 따르는 실험 로그 작성, 결과 파일 저장, 핸드오프 규칙.

읽는 법: §0은 매 trial에 적용하는 판단 규칙의 요약이며, autoread가 세션 시작과 컨텍스트 재구성 때 넣음. 각 절은 판단 규칙(에이전트와 사용자가 정하는 것)을 먼저 두고, "도구가 하는 일"(elf 명령이 쓰거나 검사하는 것)을 뒤에 둠. 명령의 세부 동작은 명령 출력과 `elf <명령> --help`가 알려 줌.

---

## 0. 핵심 요약

매 trial에 적용하는 판단 규칙만 모은 요약. `AGENTS.md` 상시 의무에 있는 항목(Phase 분리, figure 즉시 embed, 세션 수명주기, 폐기, 로그 손 편집 자제 등)과 도구가 쓰거나 검사하는 형식은 빼고 적음. 줄 끝 §는 자세한 규칙의 위치. 요약과 본문이 다르면 본문을 따름.

- **S와 G 구분**: 그 산출물에만 적용되는 작업 = S trial, 이후 작업 전체에 적용되는 규칙, 구조, 템플릿, 도구 설정 변경 = G trial (§1)
- **로그에 쓰는 것**: 사실(파라미터, 결과, 오류와 해결)만. 아이디어는 `1_Concept/13_Ideas/`, 여러 세션에 걸친 계획은 `1_Concept/12_Planning/` (§1)
- **goal, handoff, conclusion, user_note**: goal = 세션 목표 1-2문장. handoff = `현재 유효한 결론; 미완료 …; 참조 …`, 전체 교체, 경위 나열 금지. conclusion = 종료 때 현재 유효한 최종 결론값. user_note = 사용자 원문만 (§2)
- **문체**: 명사형 종결, 목록, 단락 서술 금지. 가설 5항 또는 사고 5단계를 넘으면 `1_Concept/12_Planning/`으로 분리 (§2)
- **figure 사전 등록**: figure가 나올 trial은 예상 절에 산출물 목록 (§2)
- **반복 개선**: 작동하는 산출물을 파라미터나 외형을 바꿔 다시 만들면 다음 trial(조건에 delta와 이유), 스크립트와 figure는 버전 접미사로 보존 (§2, §3.3)
- **발의와 버린 대안**: 배경(없으면 목표)에 `사용자 제기` 또는 `AI 발상`. 진입 때 버린 경로 = 배경, 범위 제외 = 조건, 결과로 버린 것 = 해석과 교훈 (§2)
- **규칙 변경 참조**: 규칙이 바뀌면 G trial에 쓰고, 영향받은 S trial 조건에는 `- 규칙 변경: G004 t03` 1줄만 (§2)
- **파일 이름**: 산출물은 `S{NNN}_…`, trial 구분이 필요하면 `S{NNN}_t{NN}_…` (§3)

---

## 1. 로그 파일 위치 및 명명

로그 외 문서(계획 등)의 폴더 규칙은 `.elf/managed/AI_PARA_Framework.md`를 따름.

| 항목 | 규칙 |
|------|------|
| **판정 기준** | 그 산출물에만 적용되는 작업 = S trial, 이후 작업 전체에 적용되는 변경 = G trial. G = 연구 내용 밖의 규칙 수정, 구조 변경, 템플릿, 도구 설정 등 |
| **내용 규칙** | 시뮬레이션 파라미터, 실행 결과, 오류와 해결 등 사실(fact-sheet)만 기록 |
| **금지 사항** | 아이디어, 기획, 방향성 논의는 `2_Log/`가 아니라 `1_Concept/`에: 작은 아이디어 = `13_Ideas/`, 여러 세션 계획 = `12_Planning/` |
| **Wiki** | `2_Log/Wiki/` = 여러 세션을 묶은 지식 문서(주제 페이지). 세션 종료 절차에 포함하지 않고, 갱신은 G 세션에서 사용자 승인 뒤 |

### 도구가 하는 일
- 로그는 `2_Log/S/`(연구), `2_Log/G/`(거버넌스)에 두고 옮기지 않음. 상태는 헤더 `status`(`ongoing`, `complete`, `deprecated`)로만 나타냄.
- `Registry.md`(세션 목록)와 `.vscode/settings.json`의 탐색기 설정은 elf가 헤더에서 만드는 생성 파일 — 직접 편집하지 않음.

---

## 2. 로그 포맷

trial 본문의 절 이름, 순서, 절마다 쓸 내용은 `.elf/managed/templates/trialTemplate.md` 한 곳에 둠(elf가 그 틀로 trial을 만들고 검사함). 로그 전체 모양:

```markdown
---
created: YYYY-MM-DD
modified: YYYY-MM-DD
status: ongoing
goal: '{세션의 핵심 목표 1-2문장}'
handoff: '{현재 상태; 미완료 작업; 참조}'
conclusion: ''
user_note: ''
---
# S{NNN}: {세션 제목}

<!-- ELF 규범 주석(elf가 넣음) -->

## t{NN}: {작업 제목}
(본문 = trialTemplate: 선택 절 배경, Phase 1 목표, 조건, 가설, 예상, Phase 2 관찰, 해석, 교훈, 생성 파일)
```

### 헤더 규칙
- **goal**: 세션의 핵심 목표 1-2문장.
- **handoff**: 교체식 상태 스냅샷. `;`로 나눈 3파트 `현재 유효한 결론과 위치(예: "A 기각, B 채택"); 미완료; 참조(종합 trial 위치)`. 갱신은 전체 재작성, 경위와 완료 타임라인 나열 금지(상세는 trial 본문). `;`는 파트 경계 전용 — 파트 안 나열은 쉼표, 미완료와 참조 파트는 라벨(`미완료`, `참조`)로 시작. 다음 세션이나 컨텍스트 재구성 때 front matter(`Read(offset=0, limit=9)`)로 상태를 복원.
- **conclusion**: 종료 때의 최종 결론(현재 유효한 결론값).
- **user_note**: 종료 때 사용자 자신의 생각(결정, 다음 행동, 남은 불확실성). **사용자 원문만** 쓰고 에이전트가 지어내거나 요약하지 않음(사용자 판단과 에이전트 해석을 구분). 건너뛰면 `''`.

#### 도구가 하는 일
- 헤더(YAML 7키)는 elf 명령만 씀: goal = `elf session new --goal`, handoff = `elf trial new`, `elf trial write`, `elf session close`의 `--from` 파일 맨 앞 YAML 블록, conclusion과 user_note = `elf session close --from`(§5.1, §5.2). `--from` 파일의 값은 작은따옴표로 감싸고, 값 안의 `'`는 `''`로 씀.
- status별 값(`elf check logs`가 검사): `ongoing` = conclusion, user_note `''` / `complete` = conclusion 필수 / `deprecated` = handoff에 폐기 사유와 대체(§6).

### 본문 규칙
- **작성 언어**: `PROJECT_LANG`(EliRule §3)의 언어로 쓰고 명사형 종결('-음/함/임'). 기술 용어는 영어 병기 가능.
- **절 제목 표기**: 배포된 trialTemplate을 따름(영어(`en`) 프로젝트 = `### Conditions`, 그 밖 = `### 조건 (Conditions)`). 본 문서는 절을 한국어 이름(`### 관찰` 등)으로 가리킴. elf는 두 표기를 같은 절로 읽고, 새로 쓰는 절은 프로젝트 언어 표기로 바로잡음.
- **Figure 인라인 embed (plot = trial 산출물)**: figure를 만든 turn에 그 trial `### 관찰`에 `![alt](경로)`로 embed하고, alt에 Figure 번호와 1줄 설명(축, 핵심 관찰)을 씀. 미루면 빠지거나 다른 trial과 어긋남. 금지: 표에 경로만 적기, 확정이나 피드백을 이유로 미루기(v1도 즉시 embed 뒤 갱신), 채팅이나 뷰어 표시로 대신하기. 위임 작업자(서브에이전트)가 만든 figure도 회수 때 main 에이전트가 embed함(위임은 의무를 면제하지 않음).
- **예상 산출물 사전 등록 (Phase 1)**: figure가 나올 trial은 `### 예상`에 산출물(figure, 표) 목록을 둠 — 각 항목은 생성 즉시 embed. "다 그린 뒤 일괄 정리" 계획 금지(컨텍스트 재구성에서 유실됨). 예상에 없던 figure도 즉시 embed하고 차이를 `### 관찰`에 씀.
- **리마인더 훅 (권장)**: harness(에이전트 실행 환경)가 도구 훅을 제공하면 figure 생성 직후 embed를 알리는 훅을 둠. 예: Claude Code PostToolUse 훅이 셸 실행 뒤 `6_Exp/64_Viz/`의 새 이미지를 감지해 알림.
- **반복 개선 (base-delta trial)**: 작동하는 산출물(figure, 표, 데이터)을 파라미터나 외형(폰트, 색, 크기, 축, 스타일)을 바꿔 다시 만들면 1회라도 다음 trial로 씀 — `### 조건`에 base = 직전 trial, delta와 이유(직전 시도의 문제), `### 관찰`에 그 버전 figure. 스크립트와 figure는 버전별로 보존(§3.3). 조용히 고치고 최종만 보고하는 것 금지(생존 편향 방지: 실패한 시도도 재현할 수 있게 각 버전을 그 turn에 기록). 같은 trial의 `### 시행착오` 표에 쓰는 것은 산출물이 나오기 전의 디버그와 외형이 바뀌지 않는 수정(오타, 경로, 주석, 변수명)뿐.
- **생성 파일 표**: 유형(`Script`, `Output`, `Figure`, `Config` 등)과 프로젝트 루트 기준 상대 경로.
- **코드와 파라미터**: 코드는 코드 블록, 파라미터는 변수명, 값, 단위 표.
- **가설과 예상 (Phase 1)**: 실행 전에 씀(결과를 본 뒤 맞춘 가설은 검증이 안 됨). 명사형 목록, 단락 서술 금지, 가능하면 정량. 가설 5항 또는 사고 5단계를 넘으면 `1_Concept/12_Planning/`으로 분리하고 cross-ref 1줄.
- **관찰 (Phase 2)**: 실행 뒤에 씀. 예상 대 관찰 대조 표 권장.
- **해석 첫 줄**: `예상 일치 정도: 일치 / 부분 일치 / 불일치` 중 하나(영어 프로젝트 = `Prediction match: match / partial / mismatch`, 예전 라벨 `가설 적중 여부`도 유효). 예상과 결과를 한 줄로 대조하고, 둘째 줄부터 메커니즘 해석.
- **후속 과제 위치**: `## 다음 세션 후보` 절은 쓰지 않음(기존 로그의 절은 유효). 진행 중 = handoff 미완료 파트, 종료 때 = conclusion, 여러 세션에 걸치면 `1_Concept/12_Planning/`.
- **배경 절 (선택)**: 맥락이 목표를 넘을 때만(선행 세션 종합, 여러 입력) `### 목표` 위에 1-2줄. 빈 배경 금지.
- **발의주체**: 작업, 전환의 발의 주체를 배경(없으면 목표)에 1단어(`사용자 제기` / `AI 발상`). 사용자 발언이 작업의 근거면 그 발언을 배경에 인용할 수 있음(진행 중 의견). 종료 때의 사용자 생각은 헤더 `user_note`.
- **규칙 변경 참조 (S trial)**: 연구 중 규칙이나 구조가 바뀌면 그 변경은 G trial에 쓰고, 영향받은 S trial의 `### 조건`에는 `- 규칙 변경: G004 t03` 1줄만(사유는 G에만).
- **버린 대안 위치(시점 기준)**: 진입 때 버린 경로 = `### 배경`(1줄, Phase 1 사후 편집 금지), 범위 제외 = `### 조건`, 결과로 버린 것 = `### 해석`, `### 교훈`. 나중에 배경에 몰아넣지 않음(사후 합리화 방지).
- **정본 우선(선례 ≠ 규범)**: 과거 세션과 trial 로그는 참고 자료일 뿐 규범이 아님. 규범 = 본 문서와 `.elf/managed/templates/`(sessionTemplate, trialTemplate). 선례가 정본과 다르면 정본을 따르고 사용자에게 보고(선례의 일탈이 복제되는 것을 막음). CLI가 없으면 trialTemplate을 복사해 본문을 씀(헤더와 Registry.md는 elf가 필요).
- **소급 정책**: 규칙은 시행 뒤 새로 쓰는 것부터 적용하고 이전 기록은 고쳐 쓰지 않음(그때의 판단 근거를 보존). 헤더 형식과 위치는 `elf update`의 이관(§7)이 바꿈.

#### 도구가 하는 일
- trial 번호는 `elf trial new`가 붙임(G도 같음, 참조 표기 `G004 t03`). 이미지 경로 = `../../6_Exp/64_Viz/S{NNN}/파일.png`(로그 폴더 기준, 로그를 옮기지 않으므로 고정).
- `elf trial new --from`, `elf trial write`가 쓰기 전에 절 이름과 순서, 필수 절, 해석 첫 줄, 틀 문구, figure embed 누락을 검사하고, `elf check logs`가 손 편집분을 같은 기준으로 감사함. 의도적 제외 = `<!-- noembed: file.png -->`.

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
- 시뮬 결과: `S{NNN}_{trial}_{description}.mat`
- 그래프: `S{NNN}_{description}.png`
- 스크립트 trial 구분이 필요하면: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`
- **반복 개선 버전 보존**: 스크립트와 figure를 덮어쓰지 않고 버전 접미사로 보존(`S{NNN}_A06a_*` → `_A06b_*`, 또는 `_v1`, `_v2`). 버전과 delta trial은 1:1. 덮어쓰면 중간본이 사라져 재현할 수 없음.
- **core + wrapper**: 버전마다 스크립트 전체를 복제하지 않고 공용 core(변형을 파라미터로 받음)와 버전별 얇은 wrapper(그 버전의 delta 파라미터)로 나눔. 예: `S{NNN}_A06_core.m` + `S{NNN}_A06{a..}_plot.m`.

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `6_Exp/64_Viz/` | 스크립트에서 자동 저장. Figure 색인 `_gallery.md`는 `elf gallery`가 세션별로 만듦 |
| 논문용 Figure | `7_Paper/71_Figs/` | rawFig → processedFig → finalFig |

---

## 4. Cross-reference 규칙

로그는 `2_Log/S/`(G는 `2_Log/G/`)에 고정되므로 상대 경로 깊이가 바뀌지 않음.

| From | To | 상대경로 (`2_Log/S/` 기준) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/12_Planning/P00x.md` | `../../1_Concept/12_Planning/P00x.md` |
| Logs → Sim Data | `6_Exp/61_Sim/Data/S{NNN}/` | `../../6_Exp/61_Sim/Data/S{NNN}/` |
| Logs → Viz | `6_Exp/64_Viz/S{NNN}/` | `../../6_Exp/64_Viz/S{NNN}/` |
| Logs → 같은 계열 로그 | `2_Log/S/S{NNN}_log.md` | `S{NNN}_log.md` |
| Logs → 다른 계열 로그 | `2_Log/G/G{NNN}_log.md` | `../G/G{NNN}_log.md` |
| Logs → Wiki | `2_Log/Wiki/x.md` | `../Wiki/x.md` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Planning으로 분리한 내용은 원래 로그에 blockquote stub으로 표시:
```markdown
> 상세 내용은 Planning 문서로 분리되었습니다.
> - [P00x_Title.md](../../1_Concept/12_Planning/P00x_Title.md) — 섹션 N 참조
```

### 도구가 하는 일
- `elf check logs`가 깨진 상대 `.md` 링크를 검출함. 도구는 자기가 파일을 옮긴 경우(이관)에만 링크를 다시 계산하고, 그 밖의 깨진 링크는 사람과 에이전트가 고침.

---

## 5. 세션 생성 시 체크리스트

AI 에이전트가 새 세션을 시작할 때:

- [ ] 연구 = `elf session new "<제목>" --goal '<목표>'`(`2_Log/S/`), 거버넌스(규칙, 구조, 템플릿, 도구 설정 변경) = `elf session new --gov "<제목>" --goal '<목표>'`(`2_Log/G/`). 진행 중 세션은 여러 개 둘 수 있음(병렬 작업은 작업마다 세션 하나, 여럿이면 trial 명령에 `--session`). goal은 세션의 핵심 목표 1-2문장(§2)
- [ ] 이어받을 선행 세션이 있으면 그 handoff, conclusion을 확인해 새 trial t01의 `### 배경`/`### 가설`에 반영(세션 ID는 배경에 씀)
- [ ] `6_Exp/61_Sim/Data/S{NNN}/` 디렉토리 생성, 스크립트 `6_Exp/61_Sim/Scripts/S{NNN}_*.m` 작성, 실행 뒤 결과 .mat와 .png 저장
- [ ] 로그에 trial(t{NN}) 추가: 파라미터, 결과, 이미지 경로(§5.1)
- [ ] 스크립트를 마치면 1회용은 `Scripts/Archive/`로, 범용은 Root에 유지
- [ ] Planning 내용이 섞이면 `1_Concept/`로 분리하고 cross-reference

### 5.1 Trial 작성 표준 절차 (Standard Operating Procedure — Phase 1 / Phase 2 분리)

trial(t{NN})은 두 단계로 씀. **Phase 1과 Phase 2 사이에 멈춤점 필수** — 즉흥 실행을 막고 가설-관찰 사이클을 완결함.

로그는 아래 명령으로만 씀. 손 편집은 내용 절의 오탈자 정도로 자제(구조는 `elf check logs`가 감사).

| 단계 | 명령 |
|---|---|
| 시작(Phase 1 첫 작성) | `elf trial new "<제목>" --from <Phase 1 파일>`(G 세션은 `--session G{NNN}`). `--from` 없이 실행하면 Phase 1 틀만 붙임(편집기에서 쓸 때) |
| Phase 1 수정(사용자 검토 중, 여러 번) | `elf trial write [tNN] --from <Phase 1 파일>` |
| 사용자 승인 뒤 실행, 결과 | `elf trial write [tNN] --from <Phase 2 파일>` |

tNN을 생략하면 진행 중 세션의 마지막 trial. 파일 맨 앞 YAML 블록(허용 키 = `handoff`만)은 헤더 handoff를 교체함. 한 파일에는 한 Phase의 절만 넣음.

**Phase 1 — 실행 전 (멈춤점)**
- [ ] (선택) `### 배경 (Background)` — 맥락이 목표를 넘을 때만: 진입 의도, **발의주체**, 진입 때 버린 경로 1줄
- [ ] `### 목표 (Goal)`(task의 what), `### 조건 (Conditions)`(파라미터, 제약), `### 가설 (Hypothesis)`(메커니즘이나 효과 3-5항, 명사형), `### 예상 (Prediction)`(구체 결과 1-3항, 가능하면 정량, figure 산출 trial은 예상 산출물 목록 — §2)
- [ ] `elf trial new "<제목>" --from <파일>` 실행
- [ ] **멈춤** — Phase 1을 사용자에게 보이고 확인을 받음. handoff 미완료에 "t{NN} Phase 1 사용자 확인 대기". 수정 요청은 `elf trial write --from`으로 반영. 자동 모드 AI도 이 멈춤점을 지킴

`--from` 파일 예(맨 앞 YAML 블록은 선택):
```markdown
---
handoff: '상태 = …; 미완료 = …; 참조 t03'
---
### 목표 (Goal)
- …
```

**Phase 2 — 실행 후**
- [ ] 사용자 승인 뒤 trial 실행(스크립트, 시뮬, 조사 등)
- [ ] 실행 직후 `6_Exp/64_Viz/S{NNN}/`에 이번 trial의 figure가 있는지 확인하고, 실행 turn을 벗어나기 전에 `### 관찰`에 embed(§2)
- [ ] `### 관찰 (Observation)`(사실과 예상 대조 표), `### 해석 (Interpretation)`(첫 줄 예상 일치 정도 + 메커니즘), `### 교훈 (Lessons)`, `### 생성 파일 (Files)`
- [ ] 작동하는 산출물을 파라미터나 외형을 바꿔 다시 만들었으면 다음 trial(delta)로 전개(§2, §3.3)
- [ ] `elf trial write [tNN] --from <파일>` 실행

#### 도구가 하는 일
- 쓰기 전 검사에 걸리면 문제를 한 번에 모두 보고하고 아무것도 쓰지 않음. Phase 2가 생긴 뒤의 Phase 1 수정, 마지막 Phase 1 쓰기 뒤 산출물이 생긴 뒤의 Phase 1 수정은 거부(결과를 반영한 변경 = 새 trial). Phase 2 다시 쓰기도 거부(다시 실행 = 새 trial).
- 정확한 동의어인 절 이름과 해석 첫 줄 라벨은 바로잡고 알림. trial 제목 아래 도구 주석에 Phase 1, Phase 2 쓰기 시각과 횟수를 남김.

### 5.2 Session 종료 표준 절차

- [ ] **`elf check logs` 실행 — 경고 해소**(미embed, 구조 위반) 또는 의도적 예외 명시(`<!-- noembed: file.png -->`)
- [ ] handoff 정리: 미완료 파트는 완료하거나 `12_Planning/` 문서로 옮김(남으면 `elf session close`가 경고)
- [ ] 에이전트가 conclusion 초안 작성(현재 유효한 최종 결론값, 경위 나열 금지)
- [ ] 사용자에게 초안을 보여 **승인**을 받고, 사용자 생각(`user_note`)을 요청함 — 사용자 원문만, 건너뛰면 빈 값
- [ ] 종료 파일을 써서 `elf session close --from <파일>` 실행

종료 파일 예:
```markdown
---
conclusion: '…'
user_note: '…'
handoff: ''
---
```

#### 도구가 하는 일
- `elf session close --from`: 허용 키 = `conclusion`(필수), `user_note`, `handoff`. status를 `complete`로 바꾸고 `Registry.md`와 탐색기 설정을 다시 만듦(파일은 옮기지 않음).

---

## 6. 폐기 (Deprecated)

폐기 = 과거에 시도했으나 더 이상 현재 판단의 근거로 쓰지 않는 기록. 에이전트가 폐기한 내용을 읽으면 그 순간 판단이 영향을 받으므로("근거로 쓰지 말라"는 지시로는 막지 못함), 폐기하면 원문을 작업 트리의 로그에서 지우고 `.elf/deprecated/`에 압축해 보관함(다시 보고 되돌릴 수 있음). 폐기 여부는 사용자가 결정하고, 에이전트는 후보와 사유를 제시한 뒤 지시나 승인이 있을 때 `elf deprecate`로 실행함.

- 에이전트 읽기: **보관 내용을 풀어 보는 것(`elf deprecate show`)은 사용자가 요청할 때만**.
- 흔적의 goal과 제목은 세션을 알아보도록 남김. 둘에 폐기한 주장이 담겨 있으면 폐기 때 사용자와 상의해 중립 표현으로 바꿈.
- 남는 참조: handoff, conclusion, 다른 문서에서 폐기 ID를 가리키는 곳은 도구가 고치지 않고 `review:` 줄로 출력함 — 에이전트가 사용자와 처리.
- 일괄 치환 주의: 가운뎃점(`·`)을 쉼표로 일괄 치환할 때 `> **Deprecated**:` 줄은 제외함. 이 줄의 ` · `는 elf가 읽는 필드 구분(EliRule §3 표기 규칙의 예외).

### 도구가 하는 일
- 단위 = 세션 전체, trial, 절, 표시 블록(`<!-- deprecate:begin -->` … `<!-- deprecate:end -->`), 줄 범위(계획 문서도 같음). 로그에는 표기 줄 `> **Deprecated**: ID · 날짜 · 단위 · 사유 … · 대체 …`(trial은 번호를 지킨 제목 줄도)만 남고, 원문은 `.elf/deprecated/<ID>.md.xz`로 감. 산출물(스크립트, 데이터, figure)은 제자리.
- 열람, 복원, 목록 = `elf deprecate show`, `--restore`, `list`(사용법: `elf deprecate --help`). **로그 변경과 보관 파일은 같은 커밋**에 넣음 — `elf check`가 짝을 검사함.

---

## 7. 구조 이관 (`elf update`)

이전 형식(인용 블록 헤더, `2_Log/` 루트와 `Archive/`, `Deprecated/`의 로그, `Session_Registry.tsv`)의 프로젝트는 `elf update`가 1회 이관함. 이관은 각 프로젝트에서 그 프로젝트의 에이전트가 실행하고 경고를 처리함.

경고가 있으면 `elf update`는 아무것도 쓰지 않고 이관 결정 파일 `.elf/runtime/migration.yaml`을 만듦. 에이전트는 각 항목의 근거를 보고 아래 기준으로 값을 정해 사용자 확인 뒤 채움(`.elf/` 안에서 손으로 채우는 유일한 예외).

| 경고 | 판단 기준 | 고치는 방법 |
|------|-----------|-------------|
| 헤더 Status와 Registry 상태 불일치 | 로그 본문으로 실제 상태를 판단(종료 trial과 결론이 있으면 종료, 진행 중 작업이 있으면 진행 중, 폐기 표기가 있으면 폐기) | 결정 파일의 `status` 칸에 판단한 값(`ongoing`, `complete`, `deprecated` 중 하나) |
| 둘 다 없음 | 위와 같음 | 결정 파일의 `status` 칸 |
| 정식 값이 아님(예: `Active (t01 계획)`, `In Progress`) | 위와 같음 | 결정 파일의 `status` 칸 |
| 링크 대상 없음 | 이관 전부터 깨진 링크인지 확인 | 링크를 고치거나 지움(결정 파일에는 목록만) |
| 옮길 자리에 같은 이름 파일 | 어느 쪽이 정본인지 판단 | 중복 파일을 정리(결정 파일에는 목록만) |
| git 작업 트리 미정리 | — | 사용자가 커밋하거나 변경을 치운 뒤 다시 실행 |

### 도구가 하는 일
- 결정 파일을 채우고 `elf update`를 다시 실행하면, 값이 모두 맞을 때 이관을 적용하고 결정 파일을 지움. `elf update --dry-run`은 쓰지 않고 계획과 경고만 보여 줌. 이관이 바꾸는 것과 결정 파일 형식은 `elf update` 출력과 CLI 문서(projects)가 알려 줌.
- 이관 뒤에는 같은 저장소를 쓰는 모든 PC에서 `elf self update`가 필요함.
