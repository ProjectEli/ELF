# LogConvention: ELF 로깅 표준 규칙

본 문서는 ELF 프로젝트에서 사람과 AI 에이전트 모두가 따라야 할 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

---

## 1. 로그 파일 위치 및 명명 (PARA 워크플로우 적용)

모든 연구 및 기획 진행은 PARA(Projects, Areas, Resources, Archives) 관리 체계와 `.claudeignore` 룰을 따릅니다. 자세한 원칙은 `.elf/managed/AI_PARA_Framework.md`를 참조하십시오.

| 항목 | 규칙 |
|------|------|
| **진행 중 (Active Sandbox)** | 새로운 세션(S{NNN})을 시작할 때는 무조건 **`2_Log/`의 최상단(Root)**에 로그(`S{NNN}_log.md`)를 생성하여 작성합니다. |
| **결론 요약 (Wiki)** | 세션이 완료되면 핵심 교훈이나 산출된 파라미터를 `Wiki/` 폴더 내의 지식 문서에 한두 줄로 요약합니다. 이 때 반드시 원본 Archive 로그 파일의 절대/상대 경로 링크를 포함합니다. |
| **보관 (Archive)** | 세션이 종료되면 전체 원본 로그 파일은 **반드시** `Archive/` 폴더로 **파일명 그대로** 이동시킵니다. (예: `Archive/S{NNN}_log.md` — 접두 태그 불요, 폴더 위치가 곧 상태) |
| **내용 규칙** | 시뮬레이션 파라미터, 실행 결과, 에러/해결 등 **순수 Metadata (Fact-Sheet)**만을 기록합니다. |
| **금지 사항** | 아이디어·기획·방향성 논의는 `2_Log/`에 기록하지 말고 `1_Concept/`로 분리: 작은 snippet·naive 아이디어는 `13_Ideas/`, 다중 세션 계획은 `12_Planning/`. |

---

## 2. 로그 포맷

```markdown
# S{NNN}: {세션 제목}

> **Created**: YYYY-MM-DD\
> **Modified**: YYYY-MM-DD\
> **Status**: {★ 활성 | In Progress | Complete}\
> **목표**: {세션의 핵심 목표 1-2문장}\
> **관련**: {관련 세션/문서 링크}\
> **Handoff**: {현재 상태; 미완료 작업; 참조 파일}

---

## t{NN}: {작업 제목}

### 배경 (Background)  (선택: 맥락이 목표를 초과할 때만. 선행 종합·다중 입력 등; 일반 trial은 생략)
- {trial 진입 의도·맥락}
- **발의**: {사용자 제기 / AI 발상}
- {진입 시 기각한 경로 1줄 — 있을 때만; Phase1 사후편집 금지}

### 목표 (Goal)
- {구체적 작업 목표 명시}

### 조건 (Conditions)
- {파라미터, 설정, 제약 등을 표나 짧은 리스트로 기술}

### 가설 (Hypothesis)
- {Phase 1, 실행 전 — 메커니즘/효과 의심 3-5항, 명사형}

### 예상 (Prediction)
- {Phase 1, 실행 전 — 결과 수치/방향 1-3항, 정량 가능 시 정량}

### 관찰 (Observation)
- {Phase 2, 실행 후 — 현상, 팩트 중심 결과 나열}
- {Figure 인라인 삽입}
- {가설/예상 대조 표 권장}

### 해석 (Interpretation)
- 가설 적중 여부: {적중 / 탈락 / 부분 적중}
- {관찰된 팩트에 대한 물리적/논리적 해석}

### 교훈 (Lessons)
- {핵심 인사이트, 향후 유의점}

### 생성 파일 (Files)

| 유형 | 파일 |
|------|------|
| Script | `경로` |
| Output | `경로` |
| Figure | `경로` |
```

### 규칙
- **작성 언어**: 로그는 `.elf/managed/EliRule.md`의 `PROJECT_LANG` 설정에 따른 언어로 작성하되, **토큰 최소화**를 위해 반드시 명사형 종결어미('-음/함/임')와 단어 중심 개조식을 사용한다. 기술 용어인 경우 영어 병기 가능.
- **Status**: `★ 활성` (현재 작업 중), `In Progress` (중간 단계), `Complete` (완료, 아카이빙 전)
- **Handoff**: **교체식 상태 스냅샷** — 세미콜론(`;`) 3파트 `현재 유효한 결론·위치(누적의 fold, 예: "A 기각·B 채택"); 미완료; 참조(종합 trial 좌표)`. 갱신은 직전 문안에 추가가 아니라 **전체 재작성**. **경위·완료 타임라인 나열 금지** — 세션 개요는 Registry key finding, 상세는 trial 본문 소관. 1줄 유지. **`;`는 파트 경계 전용(예약)** — 파트 내 나열은 `·`/`,` 사용, 미완료·참조 파트는 라벨(`미완료`·`참조`)로 시작(도구는 `;` 직후 라벨 위치를 경계로 식별). 다음 세션·컨텍스트 재구성 시 `Read(offset=0, limit=9)`로 이 필드만 읽어 재정렬. 미완료를 남긴 채 종료하면 `elf session close`가 경고. 초기값 `-`
- **Registry key finding**: Handoff와 동일한 fold 원칙 — 경위 누적이 아니라 **현재 유효한 최종 결론값**으로 교체 갱신하고, 세션 종료 시 최종 fold로 재작성(§5.2).
- **헤더 줄바꿈**: 인용구(`>`) 헤더 각 줄 끝의 `\`는 CommonMark **hard break** — Discord 미리보기 등 strict 렌더러에서 줄 분리를 보존(없으면 6개 항목이 한 줄로 합쳐짐). **삭제 금지**. 마지막 줄(Handoff)은 `\` 없음(블록 종료). 헤더는 6줄 유지 → `limit=9` 빠른읽기 불변.
- **trial 번호**: `t01`, `t02`, ... 순서대로. 중복 금지
- **이미지 경로**: `![alt text](../6_Exp/64_Viz/S{NNN}/filename.png)` (2_Log 기준 상대경로)
- **Figure 인라인 임베딩 필수 (plot = trial 산출물)**: 모든 plot은 특정 trial(`t{NN}`)의 산출물 — trial 추가(base delta)로 figure 생성 시 **그 작업 turn에 즉시** 해당 trial `### 관찰` 절에 인라인 임베딩(`![alt](경로)`)하고, alt text에 Figure 번호 + 1줄 설명(축·핵심 관찰)을 기재한다. **금지**: ① 파일 목록 테이블에 경로만 기재·본문 미임베딩, ② 확정·피드백 대기·중간(v1) 버전을 이유로 미룸(v1도 즉시 embed 후 갱신), ③ 외부 표시(채팅/뷰어 전송)로 로그 임베딩 대체. `elf validate`가 64_Viz 그림↔본문 임베딩 누락을 검출(`--strict`는 issue 승격; 의도적 제외는 `<!-- noembed: file.png -->`).
- **예상 산출물 사전등록 (Phase 1)**: figure 산출이 예상되는 trial은 `### 예상`에 예상 산출물(figure·표) 목록을 명시 — 각 항목 = Phase 2에서 **생성 즉시 embed 대상**. **"전부 그린 후 일괄 정리" 계획 금지**(계획 단위 = figure 1건 생성 즉시 embed — 지연 배치는 중단·컨텍스트 재구성에서 유실됨). 예상에 없던 figure도 생성 즉시 embed하고 예상 대비 차이를 `### 관찰`에 명시.
- **서브에이전트(위임 작업자) 산출 figure 동일 적용**: 위임 작업자가 생성한 figure도 본 규칙의 대상 — **회수(merge) 시점에 main 에이전트가** 해당 trial `### 관찰`에 embed. **위임은 embed 의무를 면제하지 않음(의무 귀속 = main)**. 회수 기록의 산출물이 figure면 경로 기재가 아니라 embed.
- **행동 시점 리마인더 훅 (권장)**: 상시 규칙은 존재를 보장할 뿐 발동 시점을 보장하지 않음 — harness(에이전트 실행 환경)가 도구 훅을 제공하면 **figure 생성 직후 embed 리마인더 훅 설치를 권장**(원칙 = harness 무관, 구현 = harness별). Claude Code 예시: `.claude/settings.json`에 PostToolUse 훅 등록 — 셸 도구 실행 후 `6_Exp/64_Viz/`의 신규 이미지를 감지하면 "생성된 figure를 해당 trial `### 관찰`에 즉시 embed (LogConvention §2)" 리마인더를 반환. 등록 문법은 사용 harness 문서 참조.
- **시행착오 — base-delta trial 전개**: 같은 형태 plot/분석을 **반복 개선**할 때는 각 수정 시도를 독립 trial(`t{NN}`)로 전개. 각 trial은 직전을 base로 `### 조건 (Conditions)`에 **delta(변경점) + 이유(직전 시도 문제)**, `### 관찰`에 그 버전 figure embed. plot script가 새로 생성돼도 **버전별 모두 보존**(§3.3), 중간 figure도 각 trial에 남김 → 시행착오가 trial chain으로 재현 가능 보존. **생존 편향 차단**: "조용히 고치고 최종만 보고" 금지(AI 포함) — 각 버전을 그 turn에 trial로 기록(덮어쓰기·사후 회상 금지). **트리거(내재화)**: *작동하는 산출물(figure·표·데이터)을 base로 파라미터·스타일을 바꿔 재생성하는 순간 = 무조건 다음 trial(delta)* + 버전 보존(§3.3). 작동-전 디버그(산출물 생성 실패→성공시키기)만 같은 trial `### 시행착오` 표 — 즉 **완성본 수정 = delta-trial / 작동 디버그 = 시행착오 표**. **소규모 예외 엄격화**: 표로 끝낼 수 있는 건 **산출물 외형이 바뀌지 않는** 수정(오타·경로·주석·변수명)뿐 — **figure/표 외형이 바뀌는 변경(폰트·색·크기·축·스타일·파라미터)은 1회라도 반드시 delta-trial**. 전개 비용은 §3.3 core+wrapper로 해소.
- **생성 파일 테이블**: 각 task에서 생성된 자료를 테이블로 정리해 가독성 및 토큰 효율 극대화. 스크립트, 데이터, Figure를 테이블로 정리. 유형(`Script`, `Output`, `Figure`, `Config` 등)과 프로젝트 루트 기준 상대경로를 명시.
- **코드 사용법**: 코드 블록 (```lang ... ```) 으로 기재
- **파라미터 표**: 변수명, 값, 단위를 표 형태로 정리
- **가설/예상 절 (Phase 1)**: trial 실행 **전** 작성. 명사형 list. 가설 5항 초과·사고 chain 5단계 초과 시 `1_Concept/12_Planning`으로 escape + cross-ref stub 1줄 남김. 단락(paragraph) 서술 금지. 정성적 예상만 반복 금지 — 가능한 정량화.
- **관찰 절 (Phase 2)**: trial 실행 **후** 작성. 가설/예상과의 대조 표 권장 (2열: `예상` vs `관찰`).
- **해석 절 1줄 규칙**: 첫 줄은 `가설 적중 여부: 적중 / 탈락 / 부분 적중` 중 택1 명시. 둘째 줄부터 메커니즘 해석.
- **`## 다음 세션 후보` 절 — 비권장(off, v2.20~)**: 활용도가 낮아 작성 권장을 중단(구 규칙: close 직전 가설·예상 후보를 적고 다음 세션 t01로 carry-over). 후속 과제·가설의 기록처 = 세션 활성 중 **Handoff 미완료 파트** · close 시 **Registry key finding(fold)** · 다중 세션에 걸치면 `1_Concept/12_Planning/`. 하위호환: 기존 로그의 절은 유효(`elf trial new`는 절이 있으면 그 앞에 삽입), `elf session close`는 절을 요구하지 않음(`--force` 불요). 구 템플릿 보존 = `.elf/managed/templates/Archive/sessionTemplate_v1.md`.
- **배경 절 (선택·조건부)**: `### 목표` 위 `### 배경 (Background)`은 **맥락이 목표를 초과할 때만** 분리(선행 세션 종합·다중 입력 등). 일반 trial은 목표가 흡수 — 상시 heading·빈 배경 금지. 경량(1-2줄).
- **발의주체**: 작업·전환의 발의 주체를 배경(없으면 목표)에 1단어 명시(`사용자 제기` / `AI 발상`) — 사고흐름의 발의 출처 보존. 원본 쿼리 verbatim은 세션 JSONL이 보존하므로 로그엔 비병기(SSOT=trial 본문 흡수).
- **버린 대안 위치(시점 기준)**: 진입 시 기각 경로 → `### 배경`(1줄, Phase1 사후편집 금지) / 범위 제외 → `### 조건` / 결과 기반 기각 → `### 해석`·`### 교훈`. 일괄 배경 삽입 금지(시점 역행·사후합리화 방지).
- **정본 우선(선례≠규범)**: 과거 세션/trial 로그는 참고 자료일 뿐 형식·규칙의 규범이 아님 — 규범 정본은 본 문서와 `.elf/managed/templates/`(sessionTemplate·trialTemplate). 선례가 정본과 다르면 정본을 따르고 그 일탈을 모방하지 말 것(발견 시 사용자에게 보고). trial 추가는 `elf trial new`(현행 정본 stub을 활성 로그에 append; CLI 미설치 시 trialTemplate 수동 복사) 권장.
- **소급 정책**: 본 규칙 시행 시점 이전 trial/session은 backfill 강제 X. **현시점 이후 신규 작성분부터 적용**. archive 진입 직전 trial은 회고 시 1줄 채움 권장.

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
- 스크립트 trial 구분 필요 시: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`
- **반복 개선(시행착오) 버전 보존**: script·figure 덮어쓰지 말고 **버전 접미사** 보존 — `S{NNN}_A06a_*`→`_A06b_*`… 또는 `_v1`/`_v2`. 각 버전 ↔ delta trial 1:1. 덮어쓰면 중간본(실패본) 유실 → 재현 불가.
- **core + wrapper 패턴(전개 비용 해소)**: 버전마다 full script 복제 대신 **공용 core + 버전별 얇은 wrapper**(delta=파라미터) — `S{NNN}_A06_core.m`(변형을 `normMode`·`isoLevels`·`clim` 등 파라미터로 수용) + `S{NNN}_A06{a..}_plot.m`(각 struct 1개 호출 = 그 버전 delta). 코드 diff가 곧 delta라 가독·보존 동시 충족.

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `6_Exp/64_Viz/` | 스크립트에서 자동 저장. (Figure 색인 `_gallery.md`는 `elf gallery` 명령으로 세션별 자동 생성.) |
| 논문용 Figure | `7_Paper/71_Figs/` | rawFig → processedFig → finalFig |

---

## 4. Cross-reference 규칙

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

## 5. 세션 생성 시 체크리스트

AI 에이전트가 새 세션(S{NNN})을 시작할 때:

- [ ] `2_Log/S{NNN}_log.md` 생성 (위 포맷 준수)
- [ ] 이어받을 선행 세션이 있으면 그 Handoff·Registry key finding(fold) 확인 → 신규 trial t01 `### 배경`/`### 가설`에 반영하고 `관련:`에 세션 ID 기재(구 `## 다음 세션 후보` carry-over 대체)
- [ ] `6_Exp/61_Sim/Data/S{NNN}/` 디렉토리 생성
- [ ] 시뮬/실험 스크립트 `6_Exp/61_Sim/Scripts/S{NNN}_*.m` 작성
- [ ] 실행 후 결과 .mat + .png 저장
- [ ] Log에 trial (t{NN}) 추가: 파라미터, 결과, 이미지 경로
- [ ] 작업 완료 시: 얻은 지식을 `2_Log/Wiki/`에 요약하고, `S{NNN}_log.md`는 `2_Log/Archive/`로 이동
- [ ] 스크립트 완료 시: 1회용 스크립트는 `Scripts/Archive/`로 이동, 범용은 Root 유지
- [ ] `2_Log/Wiki/Session_Registry.tsv`에 해당 세션 항목 추가 업데이트
- [ ] Planning 내용이 포함된 경우 → `1_Concept/`로 분리 + cross-reference

### 5.1 Trial 작성 표준 절차 (Standard Operating Procedure — Phase 1 / Phase 2 분리)

각 trial(t{NN}) 작성 시 두 단계로 분리하여 진행. **Phase 1과 Phase 2 사이에 멈춤점 필수** — 즉흥 실행 방지, 가설-관찰 사이클 closure 강제.

**Phase 1 — 실행 전 (pre-execution, 멈춤점)**
- [ ] trial stub 생성: `elf trial new [제목]` 권장(현행 trialTemplate을 활성 로그에 append; CLI 미설치 시 `.elf/managed/templates/trialTemplate.md` 수동 복사 또는 `## t{NN}: [작업 제목]` 헤더 수동 작성)
- [ ] (선택) `### 배경 (Background)` — 맥락이 목표를 초과할 때만: 진입 의도 + **발의주체**(사용자 제기/AI 발상) + (진입 기각 경로 1줄)
- [ ] `### 목표 (Goal)` 작성 (task의 what)
- [ ] `### 조건 (Conditions)` 작성 (parameter·제약 확정)
- [ ] `### 가설 (Hypothesis)` 작성 (메커니즘/효과 의심 3-5항, 명사형)
- [ ] `### 예상 (Prediction)` 작성 (구체 결과 예측 1-3항, 정량 가능 시 정량) — (figure 산출 trial) **예상 산출물(figure·표) 목록 포함**(각 항목 = 생성 즉시 embed 대상, §2)
- [ ] **멈춤** — 가설·예상 확인 후 Phase 2 진입 (자동 모드 AI도 본 멈춤점 준수)

**Phase 2 — 실행 후 (post-execution)**
- [ ] trial 실행 (스크립트/시뮬/조사 등)
- [ ] **코드 실행 직후 figure 확인**: MATLAB 등 실행 종료 즉시 `6_Exp/64_Viz/S{NNN}/`에 이번 trial이 산출한 figure가 있는지 확인(실행 turn 이탈 전). 있으면 누락 없이 아래 embed 단계 수행 — 확정 대기·피드백·외부표시(채팅/뷰어 전송)를 이유로 미루지 않음.
- [ ] `### 관찰 (Observation)` 작성 (팩트 + 가설/예상 대조 표)
- [ ] 생성한 figure를 `### 관찰`에 인라인 embed (`![alt](경로)`; 경로만 표 기재는 embed 아님 — `elf validate`가 검출)
- [ ] **반복 개선 판정(트리거)**: 작동하는 산출물을 파라미터·스타일 바꿔 재생성하면 → 다음 trial(delta)로 전개(script 버전 보존 §3.3 + figure embed + `### 조건 (Conditions)`에 delta·이유). **외형 변경(폰트·색·크기·스타일)은 1회라도 delta-trial**; 외형 무변경(오타·경로) 수정만 `### 시행착오` 절 표.
- [ ] `### 해석 (Interpretation)` 작성 (첫 줄: 가설 적중 여부 명시 + 메커니즘 해석)
- [ ] `### 교훈 (Lessons)` 작성
- [ ] `### 생성 파일 (Files)` 작성

### 5.2 Session 종료 표준 절차

세션 Status를 `Complete`로 전환 직전:
- [ ] **`elf validate` 실행 — 경고 해소**(미embed·구조 위반) 또는 의도적 예외 명시(`<!-- noembed: file.png -->`) 후 진행: Archive 이동 후에는 구조 검사 범위에서 빠지므로 **close 전이 마지막 검증 기회**
- [ ] Handoff 정리: 미완료 파트 소거(완료, 또는 Registry key finding·`12_Planning/` 문서로 이관) — 잔존 시 `elf session close`가 경고
- [ ] `Wiki/`에 핵심 결론 요약 + Archive 경로 링크 추가
- [ ] `S{NNN}_log.md` → `Archive/`로 파일명 그대로 이동
- [ ] `Session_Registry.tsv` 상태 갱신 + key finding을 최종 결론(fold)으로 재작성
