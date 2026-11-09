# LogConvention: ELF 로깅 표준 규칙

본 문서는 ELF 프로젝트에서 사람과 AI 에이전트 모두가 따라야 할 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

읽는 법: §0은 매 trial에 적용하는 판단 규칙의 요약이며, autoread가 세션 시작과 컨텍스트 재구성 때 넣음. 각 절은 판단 규칙(에이전트와 사용자가 정하는 것)을 먼저 두고, "도구가 하는 일"(elf 명령이 쓰거나 검사하는 것)을 뒤에 둠. 도구가 하는 일은 명령 출력이 그 자리에서 알려 주므로, 명령이 가리킬 때나 형식을 확인할 때 읽음.

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

계획 문서 등 로그 외 문서의 폴더 규칙은 `.elf/managed/AI_PARA_Framework.md`를 따름.

| 항목 | 규칙 |
|------|------|
| **판정 기준** | 그 산출물에만 적용되는 작업 = S trial, 이후 작업 전체에 적용되는 변경 = G trial. G = 연구 내용 밖의 규칙 수정, 구조 변경, 템플릿, 도구 설정 등 |
| **내용 규칙** | 시뮬레이션 파라미터, 실행 결과, 에러/해결 등 **순수 Metadata (Fact-Sheet)**만을 기록함 |
| **금지 사항** | 아이디어, 기획, 방향성 논의는 `2_Log/`에 기록하지 말고 `1_Concept/`로 분리: 작은 snippet, naive 아이디어는 `13_Ideas/`, 다중 세션 계획은 `12_Planning/` |
| **Wiki** | `2_Log/Wiki/`는 여러 세션을 묶은 지식 문서(주제 페이지). 세션 종료 절차에 포함하지 않음 — 갱신은 G 세션에서 사용자 승인 뒤 |

### 도구가 하는 일

세션 로그는 상태와 무관하게 한 폴더에 두고, 상태는 헤더 `status`에만 기록함. 종료, 폐기, 복원 때 로그 파일을 옮기지 않음.

| 항목 | 규칙 |
|------|------|
| **S 계열 (연구 세션)** | 연구 세션 로그(`S{NNN}_log.md`)는 `2_Log/S/`에 둠. `elf session new "<제목>" --goal '<목표>'`가 만듦 |
| **G 계열 (거버넌스 세션)** | G 세션 로그(`G{NNN}_log.md`)는 `2_Log/G/`에 둠. `elf session new --gov "<제목>" --goal '<목표>'`가 만듦(첫 G 세션 때 폴더 생성). 생성, 진행 절차와 trial 틀은 S와 같음. 번호는 S와 별도로 G001부터 |
| **상태** | 헤더 `status` 하나로 판단: `ongoing`(진행 중), `complete`(종료), `deprecated`(폐기). 폴더 위치는 상태를 나타내지 않음 |
| **목록 (Registry.md)** | `2_Log/S/Registry.md`, `2_Log/G/Registry.md`는 헤더에서 elf가 만드는 생성 파일 — 직접 편집하지 않음. 열 = 세션(같은 폴더 로그로 가는 링크, 표시 = ID), status, 결론 또는 handoff(`ongoing` = handoff, `complete` = conclusion, `deprecated` = 폐기 사유). `ongoing`이 먼저, 이어서 번호 내림차순. 로그를 바꾸는 elf 명령이 다시 만듦 |
| **탐색기 보기** | `.vscode/settings.json`의 file nesting 설정(elf가 상태 변경 때 다시 만듦)으로 진행 중 로그만 펼쳐 보이고 나머지는 `Registry.md` 아래로 접힘. VS Code에서 프로젝트 폴더를 작업 영역으로 열 때 적용 |

---

## 2. 로그 포맷

형식 원본: trial 본문의 절 이름, 순서, 각 절에 쓸 내용 안내는 `.elf/managed/templates/trialTemplate.md` 한 곳에 둠(elf가 그 틀로 trial을 만들고 검사함). 로그 헤더와 전체 모양은 아래와 같음.

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
- **handoff**: **교체식 상태 스냅샷** — 세미콜론(`;`) 3파트 `현재 유효한 결론과 위치(누적 결과의 현재값, 예: "A 기각, B 채택"); 미완료; 참조(종합 trial 위치)`. 갱신은 직전 내용에 추가가 아니라 **전체 재작성**. **경위나 완료 타임라인 나열 금지** — 상세는 trial 본문 소관. **`;`는 파트 경계 전용(예약)** — 파트 내 나열은 쉼표(`,`), 미완료와 참조 파트는 라벨(`미완료`, `참조`)로 시작(도구는 `;` 직후 라벨 위치를 경계로 식별). 다음 세션이나 컨텍스트 재구성 때 `Read(offset=0, limit=9)`로 front matter만 읽어 작업 상태를 복원. 초기값 `''`.
- **conclusion**: 종료 때의 최종 결론(현재 유효한 결론값).
- **user_note**: 종료 때 사용자 자신의 생각(결정, 다음 행동, 남은 불확실성). **사용자 원문만** 쓰고 에이전트가 지어내거나 요약해 채우지 않음. 사용자가 건너뛰면 `''`.

#### 도구가 하는 일
- **쓰는 주체**: 헤더는 elf 명령만 씀(손으로 고치지 않음). goal = `elf session new --goal`, handoff = `elf trial new --from`, `elf trial write --from`, `elf session close --from` 파일의 맨 앞 YAML 블록(§5.1, §5.2), conclusion과 user_note = `elf session close --from`으로만(§5.2).
- **형식**: 파일 맨 앞 YAML front matter. 키 7개를 항상 이 순서로 둠: `created`, `modified`, `status`, `goal`, `handoff`, `conclusion`, `user_note`. 키를 빼거나 더하거나 순서를 바꾸지 않음. S와 G 공통. front matter 다음 줄이 `# S{NNN}: {제목}`(G는 `# G{NNN}: {제목}`)이며 제목은 front matter에 넣지 않음.
- **값**: 한 줄. 날짜와 status는 따옴표 없이 씀. 자유 텍스트(goal, handoff, conclusion, user_note)는 항상 작은따옴표로 감싸고, 값 안의 `'`는 `''`로 씀(`"`, `\`는 그대로). 빈 값은 `''`.
- **status**: `ongoing`(진행 중), `complete`(종료), `deprecated`(폐기) 셋만 씀. 상태 변경은 elf 명령(`session close`, `deprecate`, `deprecate --restore`)이 함.
- **modified**: 로그를 바꾸는 elf 명령이 갱신함.
- **검사**: `ongoing`인데 goal이 비어 있으면, `complete`인데 conclusion이 비어 있으면 `elf check logs`가 경고.
- **status별 값**: `ongoing` = conclusion, user_note `''` / `complete` = conclusion 필수, handoff의 미완료가 남으면 경고 / `deprecated` = handoff에 폐기 사유와 대체, conclusion과 user_note `''`(§6).

### 본문 규칙
- **작성 언어**: 로그는 `.elf/managed/EliRule.md`의 `PROJECT_LANG` 설정에 따른 언어로 작성하되, 반드시 명사형 종결어미('-음/함/임')를 사용한다(EliRule §3 문장 종결 방식). 기술 용어인 경우 영어 병기 가능.
- **Figure 인라인 임베딩 필수 (plot = trial 산출물)**: 모든 plot은 특정 trial(`t{NN}`)의 산출물 — trial 추가(base delta)로 figure 생성 시 **그 작업 turn에 즉시** 해당 trial `### 관찰` 절에 인라인 임베딩(`![alt](경로)`)하고, alt text에 Figure 번호 + 1줄 설명(축, 핵심 관찰)을 기재한다. **금지**: ① 파일 목록 테이블에 경로만 기재, 본문 미임베딩, ② 확정, 피드백 대기, 중간(v1) 버전을 이유로 미룸(v1도 즉시 embed 후 갱신), ③ 외부 표시(채팅/뷰어 전송)로 로그 임베딩 대체.
- **예상 산출물 사전등록 (Phase 1)**: figure 산출이 예상되는 trial은 `### 예상`에 예상 산출물(figure, 표) 목록을 명시 — 각 항목 = Phase 2에서 **생성 즉시 embed 대상**. **"전부 그린 후 일괄 정리" 계획 금지**(계획 단위 = figure 1건 생성 즉시 embed — 지연 배치는 중단, 컨텍스트 재구성에서 유실됨). 예상에 없던 figure도 생성 즉시 embed하고 예상 대비 차이를 `### 관찰`에 명시.
- **서브에이전트(위임 작업자) 산출 figure 동일 적용**: 위임 작업자가 생성한 figure도 본 규칙의 대상 — **회수(merge) 시점에 main 에이전트가** 해당 trial `### 관찰`에 embed. **위임은 embed 의무를 면제하지 않음(의무 귀속 = main)**. 회수 기록의 산출물이 figure면 경로 기재가 아니라 embed.
- **행동 시점 리마인더 훅 (권장)**: 상시 규칙은 존재를 보장할 뿐 발동 시점을 보장하지 않음 — harness(에이전트 실행 환경)가 도구 훅을 제공하면 **figure 생성 직후 embed 리마인더 훅 설치를 권장**(원칙 = harness 무관, 구현 = harness별). Claude Code 예시: `.claude/settings.json`에 PostToolUse 훅 등록 — 셸 도구 실행 후 `6_Exp/64_Viz/`의 신규 이미지를 감지하면 "생성된 figure를 해당 trial `### 관찰`에 즉시 embed (LogConvention §2)" 리마인더를 반환. 등록 문법은 사용 harness 문서 참조.
- **시행착오 — base-delta trial 전개**: 같은 형태 plot/분석을 **반복 개선**할 때는 각 수정 시도를 독립 trial(`t{NN}`)로 전개. 각 trial은 직전을 base로 `### 조건 (Conditions)`에 **delta(변경점) + 이유(직전 시도 문제)**, `### 관찰`에 그 버전 figure embed. plot script가 새로 생성돼도 **버전별 모두 보존**(§3.3), 중간 figure도 각 trial에 남김 → 시행착오가 trial chain으로 재현 가능 보존. **생존 편향 차단**: "조용히 고치고 최종만 보고" 금지(AI 포함) — 각 버전을 그 turn에 trial로 기록(덮어쓰기, 사후 회상 금지). **트리거(내재화)**: *작동하는 산출물(figure, 표, 데이터)을 base로 파라미터, 스타일을 바꿔 재생성하는 순간 = 무조건 다음 trial(delta)* + 버전 보존(§3.3). 작동-전 디버그(산출물 생성 실패→성공시키기)만 같은 trial `### 시행착오` 표 — 즉 **완성본 수정 = delta-trial / 작동 디버그 = 시행착오 표**. **소규모 예외 엄격화**: 표로 끝낼 수 있는 건 **산출물 외형이 바뀌지 않는** 수정(오타, 경로, 주석, 변수명)뿐 — **figure/표 외형이 바뀌는 변경(폰트, 색, 크기, 축, 스타일, 파라미터)은 1회라도 반드시 delta-trial**. 전개 비용은 §3.3 core+wrapper로 해소.
- **생성 파일 테이블**: 각 task에서 생성된 자료를 테이블로 정리해 가독성 극대화. 스크립트, 데이터, Figure를 테이블로 정리. 유형(`Script`, `Output`, `Figure`, `Config` 등)과 프로젝트 루트 기준 상대경로를 명시.
- **코드 사용법**: 코드 블록 (```lang ... ```) 으로 기재
- **파라미터 표**: 변수명, 값, 단위를 표 형태로 정리
- **가설/예상 절 (Phase 1)**: trial 실행 **전** 작성. 명사형 list. 가설 5항 초과, 사고 chain 5단계 초과 시 `1_Concept/12_Planning`으로 분리 이관 + cross-ref stub 1줄 남김. 단락(paragraph) 서술 금지. 정성적 예상만 반복 금지 — 가능한 정량화.
- **관찰 절 (Phase 2)**: trial 실행 **후** 작성. 가설/예상과의 대조 표 권장 (2열: `예상` vs `관찰`).
- **해석 절 1줄 규칙**: 첫 줄은 `예상 일치 정도: 일치 / 부분 일치 / 불일치` 중 택1 명시(v2.23 이전 로그의 `가설 적중 여부` 줄도 유효 — 소급 정책). 둘째 줄부터 메커니즘 해석.
- **`## 다음 세션 후보` 절**: 쓰지 않음. 후속 과제와 가설의 기록처 = 세션 활성 중 **handoff 미완료 파트**, 종료 때 **conclusion**, 다중 세션에 걸치면 `1_Concept/12_Planning/`. 기존 로그의 절은 유효하며 `elf trial new`는 절이 있으면 그 앞에 삽입하고 `elf session close`는 절을 요구하지 않음.
- **배경 절 (선택, 조건부)**: `### 목표` 위 `### 배경 (Background)`은 **맥락이 목표를 초과할 때만** 분리(선행 세션 종합, 다중 입력 등). 일반 trial은 목표 절에 포함 — 상시 heading, 빈 배경 금지. 경량(1-2줄).
- **발의주체**: 작업, 전환의 발의 주체를 배경(없으면 목표)에 1단어 명시(`사용자 제기` / `AI 발상`) — 작업 과정의 발의 출처 보존. 사용자 발언이 작업의 근거면 그 발언을 배경에 인용할 수 있음(진행 중 의견). 종료 때의 사용자 생각은 헤더 `user_note`.
- **규칙 변경 참조 (S trial)**: 연구 도중 규칙이나 구조가 바뀌면 그 변경은 G trial에 쓰고, 영향을 받은 S trial의 `### 조건`에는 1줄 `- 규칙 변경: G004 t03`만 남김(내용 요약 없이 ID만 — 사유는 G에만).
- **버린 대안 위치(시점 기준)**: 진입 시 기각 경로 → `### 배경`(1줄, Phase1 사후편집 금지) / 범위 제외 → `### 조건` / 결과 기반 기각 → `### 해석`, `### 교훈`. 일괄 배경 삽입 금지(시점 역행, 사후합리화 방지).
- **정본 우선(선례≠규범)**: 과거 세션/trial 로그는 참고 자료일 뿐 형식, 규칙의 규범이 아님 — 규범 정본은 본 문서와 `.elf/managed/templates/`(sessionTemplate, trialTemplate). 선례가 정본과 다르면 정본을 따르고 그 일탈을 모방하지 말 것(발견 시 사용자에게 보고). trial은 `elf trial new`와 `elf trial write`로 씀(§5.1). CLI 미설치 시 trialTemplate을 수동 복사해 본문을 씀(헤더와 Registry.md는 elf가 필요).
- **소급 정책**: 본 규칙 시행 시점 이전 trial/session은 backfill 강제 X. **현시점 이후 신규 작성분부터 적용**. 헤더 형식과 위치는 `elf update`의 이관(§7)이 바꿈.

#### 도구가 하는 일
- **trial 번호**: `elf trial new`가 붙임. `t01`, `t02`, ... 순서대로. 중복 금지. G 계열도 같음(참조 표기 `G004 t03`).
- **이미지 경로**: `![alt text](../../6_Exp/64_Viz/S{NNN}/filename.png)` (로그 폴더 `2_Log/S/` 기준 상대경로 — 로그를 옮기지 않으므로 깊이 고정)
- **쓰기 전 검사**: `elf trial new --from`, `elf trial write`가 절 이름과 순서, Phase 1 필수 절, 해석 첫 줄, 틀 문구 그대로인 절을 검사(§5.1). `elf check logs`는 손 편집분을 같은 기준으로 감사.
- **figure 누락 검출**: `elf trial write`(Phase 2)가 마지막 Phase 1 쓰기 뒤 생긴 그 세션 그림 중 본문에 없는 것을 거부, `elf check logs`가 64_Viz 그림↔본문 임베딩 누락을 검출(의도적 제외는 `<!-- noembed: file.png -->`).

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
- **반복 개선(시행착오) 버전 보존**: script, figure 덮어쓰지 말고 **버전 접미사** 보존 — `S{NNN}_A06a_*`→`_A06b_*`… 또는 `_v1`/`_v2`. 각 버전 ↔ delta trial 1:1. 덮어쓰면 중간본(실패본) 유실 → 재현 불가.
- **core + wrapper 패턴(전개 비용 해소)**: 버전마다 full script 복제 대신 **공용 core + 버전별 얇은 wrapper**(delta=파라미터) — `S{NNN}_A06_core.m`(변형을 `normMode`, `isoLevels`, `clim` 등 파라미터로 수용) + `S{NNN}_A06{a..}_plot.m`(각 struct 1개 호출 = 그 버전 delta). 코드 diff가 곧 delta라 가독, 보존 동시 충족.

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `6_Exp/64_Viz/` | 스크립트에서 자동 저장. (Figure 색인 `_gallery.md`는 `elf gallery` 명령으로 세션별 자동 생성.) |
| 논문용 Figure | `7_Paper/71_Figs/` | rawFig → processedFig → finalFig |

---

## 4. Cross-reference 규칙

로그는 `2_Log/S/`(G는 `2_Log/G/`)에 고정되므로 상대경로 깊이가 바뀌지 않음.

| From | To | 상대경로 (`2_Log/S/` 기준) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/12_Planning/P00x.md` | `../../1_Concept/12_Planning/P00x.md` |
| Logs → Sim Data | `6_Exp/61_Sim/Data/S{NNN}/` | `../../6_Exp/61_Sim/Data/S{NNN}/` |
| Logs → Viz | `6_Exp/64_Viz/S{NNN}/` | `../../6_Exp/64_Viz/S{NNN}/` |
| Logs → 같은 계열 로그 | `2_Log/S/S{NNN}_log.md` | `S{NNN}_log.md` |
| Logs → 다른 계열 로그 | `2_Log/G/G{NNN}_log.md` | `../G/G{NNN}_log.md` |
| Logs → Wiki | `2_Log/Wiki/x.md` | `../Wiki/x.md` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Planning으로 분리된 내용은 원본 Log에 blockquote stub으로 표시:
```markdown
> 상세 내용은 Planning 문서로 분리되었습니다.
> - [P00x_Title.md](../../1_Concept/12_Planning/P00x_Title.md) — 섹션 N 참조
```

### 도구가 하는 일
- `elf check logs`가 로그 안의 깨진 상대 `.md` 링크를 검출함. 링크는 도구가 파일을 옮긴 경우(이관, 구 위치 로그 수습)에만 도구가 다시 계산하고, 그 밖의 깨진 링크는 사람과 에이전트가 고침.

---

## 5. 세션 생성 시 체크리스트

AI 에이전트가 새 세션을 시작할 때:

- [ ] 연구 = `elf session new "<제목>" --goal '<목표>'`(`2_Log/S/`), 거버넌스(규칙, 구조, 템플릿, 도구 설정 변경) = `elf session new --gov "<제목>" --goal '<목표>'`(`2_Log/G/`). 계열마다 진행 중 세션은 1개(S 1개와 G 1개는 동시에 둘 수 있음). goal은 세션의 핵심 목표 1-2문장(§2)
- [ ] 이어받을 선행 세션이 있으면 그 handoff, conclusion 확인 → 신규 trial t01 `### 배경`/`### 가설`에 반영(세션 ID는 배경에 씀)
- [ ] `6_Exp/61_Sim/Data/S{NNN}/` 디렉토리 생성
- [ ] 시뮬/실험 스크립트 `6_Exp/61_Sim/Scripts/S{NNN}_*.m` 작성
- [ ] 실행 후 결과 .mat + .png 저장
- [ ] Log에 trial (t{NN}) 추가: 파라미터, 결과, 이미지 경로(§5.1)
- [ ] 스크립트 완료 시: 1회용 스크립트는 `Scripts/Archive/`로 이동, 범용은 Root 유지
- [ ] Planning 내용이 포함된 경우 → `1_Concept/`로 분리 + cross-reference

#### 도구가 하는 일
- `elf session new`가 번호(S와 G 따로)를 붙이고, YAML 7키 헤더를 쓰고(goal 포함), `Registry.md`와 탐색기 설정을 갱신하고, S와 G 판정 기준을 출력함. G 폴더는 첫 G 세션 때 만듦.

### 5.1 Trial 작성 표준 절차 (Standard Operating Procedure — Phase 1 / Phase 2 분리)

각 trial(t{NN}) 작성 시 두 단계로 분리하여 진행. **Phase 1과 Phase 2 사이에 멈춤점 필수** — 즉흥 실행 방지, 가설-관찰 사이클 완결 강제.

**명령**: 로그는 아래 명령으로만 씀. 손 편집은 내용 절의 오탈자 정도로 자제(구조는 `elf check logs`가 감사).

| 단계 | 명령 |
|---|---|
| 시작(Phase 1 첫 작성) | `elf trial new "<제목>" --from <Phase 1 파일>` = `elf trial new "<제목>"` 다음 `elf trial write --from <파일>`. `--from` 없이 `elf trial new [제목]`으로 실행하면 Phase 1 틀만 붙임(사람이 편집기에서 쓸 때, 제목을 생략하면 `[작업 제목]`이 남음). G 세션은 `--session G{NNN}` |
| Phase 1 수정(사용자 검토 중, 여러 번) | `elf trial write [tNN] --from <Phase 1 파일>` |
| 사용자 승인 뒤 실행, 결과 | `elf trial write [tNN] --from <Phase 2 파일>` |

본문을 파일로 넘기므로 셸 escape가 필요 없음. tNN을 생략하면 진행 중 세션의 마지막 trial. 파일 맨 앞에 YAML 블록을 두면 헤더 handoff를 교체함(허용 키 = `handoff`만, 아래 예). 한 파일에는 한 Phase의 절만 넣음. CLI 미설치 시 `.elf/managed/templates/trialTemplate.md` 수동 복사 또는 `## t{NN}: [작업 제목]` 헤더 수동 작성.

**Phase 1 — 실행 전 (pre-execution, 멈춤점)**
- [ ] (선택) `### 배경 (Background)` — 맥락이 목표를 초과할 때만: 진입 의도 + **발의주체**(사용자 제기/AI 발상) + (진입 기각 경로 1줄)
- [ ] `### 목표 (Goal)` 작성 (task의 what)
- [ ] `### 조건 (Conditions)` 작성 (parameter, 제약 확정)
- [ ] `### 가설 (Hypothesis)` 작성 (메커니즘/효과 의심 3-5항, 명사형)
- [ ] `### 예상 (Prediction)` 작성 (구체 결과 예측 1-3항, 정량 가능 시 정량) — (figure 산출 trial) **예상 산출물(figure, 표) 목록 포함**(각 항목 = 생성 즉시 embed 대상, §2)
- [ ] Phase 1 파일로 `elf trial new "<제목>" --from <파일>` 실행
- [ ] **멈춤** — Phase 1을 사용자에게 보이고 확인을 받음. handoff 미완료에 "t{NN} Phase 1 사용자 확인 대기". 수정 요청은 `elf trial write --from`으로 반영(여러 번). 가설, 예상 확인 후 Phase 2 진입 (자동 모드 AI도 본 멈춤점 준수)

`--from` 파일 예(맨 앞 YAML 블록은 선택):
```markdown
---
handoff: '상태 = …; 미완료 = …; 참조 t03'
---
### 목표 (Goal)
- …
```

**Phase 2 — 실행 후 (post-execution)**
- [ ] trial 실행 (스크립트/시뮬/조사 등) — 사용자 승인 뒤
- [ ] **코드 실행 직후 figure 확인**: MATLAB 등 실행 종료 즉시 `6_Exp/64_Viz/S{NNN}/`에 이번 trial이 산출한 figure가 있는지 확인(실행 turn 이탈 전). 있으면 누락 없이 아래 embed 단계 수행 — 확정 대기, 피드백, 외부표시(채팅/뷰어 전송)를 이유로 미루지 않음.
- [ ] `### 관찰 (Observation)` 작성 (팩트 + 가설/예상 대조 표)
- [ ] 생성한 figure를 `### 관찰`에 인라인 embed (§2 Figure 인라인 임베딩)
- [ ] **반복 개선 판정(트리거)**: 작동하는 산출물을 파라미터, 스타일 바꿔 재생성하면 → 다음 trial(delta)로 전개(§2 시행착오, §3.3)
- [ ] `### 해석 (Interpretation)` 작성 (첫 줄: 예상 일치 정도 명시 + 메커니즘 해석)
- [ ] `### 교훈 (Lessons)` 작성
- [ ] `### 생성 파일 (Files)` 작성
- [ ] Phase 2 파일로 `elf trial write [tNN] --from <파일>` 실행

#### 도구가 하는 일
- **쓰기 전 검사**: 실패하면 문제를 모두 한 번에 보고하고 아무것도 쓰지 않음(exit 3). 한 파일에 Phase 1 절과 Phase 2 절이 섞이면 거부. 본문 안의 `#`, `## ` 제목 거부.
- **Phase 1 절**: 필수 절(목표, 조건, 가설, 예상), 틀 문구 그대로인 절 거부. Phase 2가 이미 있거나, 마지막 Phase 1 쓰기 뒤 그 세션 산출물이 생겼으면 거부(결과를 본 뒤 Phase 1 수정 금지 — 결과를 반영한 변경은 새 trial).
- **Phase 2 절**: 관찰, 해석, 교훈, 생성 파일 전부, 해석 첫 줄, Phase 1 완성 여부 검사. Phase 2가 이미 있으면 거부(다시 실행 = 새 trial). 마지막 Phase 1 쓰기 뒤 생긴 그 세션 그림이 본문에 없으면 거부.
- **바로잡기**: 정확한 동의어 표에 있는 절 이름과 해석 첫 줄 라벨은 거부하지 않고 바로잡고 알림. 절 순서는 고치지 않음(Phase 위반의 신호일 수 있음).
- **기록**: trial 제목 아래 도구 주석에 Phase 1 첫 쓰기와 마지막 쓰기 시각, 쓰기 횟수, Phase 2 쓰기 시각. 헤더 `modified`와 `Registry.md` 갱신.

### 5.2 Session 종료 표준 절차

- [ ] **`elf check logs` 실행 — 경고 해소**(미embed, 구조 위반) 또는 의도적 예외 명시(`<!-- noembed: file.png -->`) 후 진행
- [ ] handoff 정리: 미완료 파트는 완료하거나 `12_Planning/` 문서로 옮김 — 잔존 시 `elf session close`가 경고
- [ ] 에이전트가 conclusion 초안 작성(현재 유효한 최종 결론값, 경위 나열 금지)
- [ ] 사용자에게 초안을 보여주고 **승인**을 받음. 이때 사용자 생각(`user_note`)을 요청함 — 사용자 원문만 받고, 건너뛰면 빈 값. 에이전트가 지어내거나 요약해 채우지 않음
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
- `elf session close --from`: 허용 키 = `conclusion`(필수), `user_note`, `handoff`. conclusion이 없거나 `''`이면 종료하지 않음.
- elf가 status를 `complete`로 바꾸고 `Registry.md`와 탐색기 설정을 다시 만듦(파일은 옮기지 않음).

---

## 6. 폐기 (Deprecated)

폐기 = 과거에 시도했으나 더 이상 현재 판단의 근거로 쓰지 않는 기록. 에이전트가 폐기한 내용을 읽으면 그 순간 판단이 영향을 받으므로("근거로 쓰지 말라"는 지시로는 막지 못함), 폐기하면 원문을 작업 트리의 로그에서 지우고 `.elf/deprecated/`에 압축해 보관함. 원문은 필요할 때 다시 볼 수 있고 되돌릴 수 있음. 폐기 여부는 사용자가 결정하고, 에이전트는 후보와 사유를 제시한 뒤 지시, 승인이 있을 때 `elf deprecate`로 실행함(사용법: `elf deprecate --help`, CLI 문서).

- 에이전트 읽기: **보관 내용을 풀어 보는 것(`elf deprecate show`)은 사용자가 요청할 때만**.
- 흔적의 goal과 제목은 세션을 알아보도록 남김. 둘에 폐기한 주장이 담겨 있으면 폐기 때 사용자와 상의해 중립 표현으로 바꿈.
- 남는 참조: Handoff, conclusion, 다른 문서에서 폐기 ID를 가리키는 곳은 도구가 고치지 않고 `review:` 줄로 출력함 — 에이전트가 사용자와 처리.
- 일괄 치환 주의: 프로젝트 md 전역에서 가운뎃점(`·`)을 쉼표로 치환할 때 `> **Deprecated**:` 행은 제외합니다. 이 줄의 ` · `는 나열이 아니라 `elf deprecate list`/`--restore`가 해석하는 필드 구분(기계 형식)입니다(EliRule §3 표기 규칙의 예외).

### 도구가 하는 일

| 단위 | 로그에 남는 것 | 보관 파일로 가는 것 |
|------|----------------|---------------------|
| 세션 로그 전체 | 헤더 7키와 제목 줄. status `deprecated`, handoff = 폐기 사유와 대체, conclusion과 user_note `''` | 본문, 원래 status, conclusion, user_note |
| trial | `## tNN: (폐기 ID)` 헤딩(번호 보존) + 표기 줄 1개 | 원래 제목과 본문 |
| 절 | 표기 줄 1개(헤딩 없음) | `### 절` 헤딩 포함 내용 |
| 표시 블록, 줄 범위 | 표기 줄 1개 | `<!-- deprecate:begin -->` … `<!-- deprecate:end -->` 사이 / 지정한 줄 |
| 계획 문서 | 파일 전체 / 표시 블록 / 줄 범위 — 로그와 같은 방식 | 같음 |

- 보관 파일: `.elf/deprecated/<ID>.md.xz`(xz 압축, ID마다 1개, git 추적). 맨 앞 YAML(원래 위치, 단위, 날짜, 사유, 대체, 원래 값) + 원문. ID = `S###-D01`처럼 문서별 일련번호(재사용 없음).
- 표기 줄: `> **Deprecated**: ID · 날짜 · 단위 · 사유 … · 대체 …`.
- 보관 파일은 압축이라 읽기 도구와 검색에 내용이 드러나지 않음.
- `Registry.md`의 폐기 행은 status와 사유만 보여줌.
- 산출물(스크립트, 데이터, figure)은 제자리.
- 열람, 복원, 목록: `elf deprecate show <ID>`(풀어서 출력), `elf deprecate --restore <ID>`(원래 자리에 되돌리고 보관 파일 삭제, 세션 전체는 원래 status와 값으로), `elf deprecate list`.
- 커밋: 폐기와 복원은 보관 파일로 동작하므로 커밋 시점과 무관함. 커밋은 몰아서 해도 되나 **로그 변경과 보관 파일은 같은 커밋**에 넣음. `elf check`가 짝을 검사함 — 표기 줄은 있는데 보관 파일이 없음 = issue, 보관 파일만 있음 = 경고, git이 추적하지 않는 보관 파일 = info.
- `elf check logs`는 폐기 흔적을 번호, 헤더 검사에 포함함. 폐기 블록에만 embed됐던 figure는 embed 합산에서 빠지므로 경고가 날 수 있음(`<!-- noembed: … -->` 또는 파일 정리는 사용자 결정).

---

## 7. 구조 이관 (`elf update`)

이전 형식(인용 블록 헤더, `2_Log/` 루트와 `Archive/`, `Deprecated/`의 로그, `Session_Registry.tsv`)의 프로젝트는 `elf update`가 1회 이관함. 이관은 각 프로젝트에서 그 프로젝트의 에이전트가 실행하고 경고를 처리함.

경고가 있으면 `elf update`는 아무것도 쓰지 않고 이관 결정 파일 `.elf/runtime/migration.yaml`을 만듦. 에이전트는 각 항목의 근거를 보고 아래 기준으로 값을 정해, 사용자 확인 뒤 채움. `.elf/`는 직접 수정하지 않는 영역이지만 이 결정 파일은 예외로 채움.

| 경고 | 판단 기준 | 고치는 방법 |
|------|-----------|-------------|
| 헤더 Status와 Registry 상태 불일치 | 로그 본문으로 실제 상태를 판단(종료 trial과 결론이 있으면 종료, 진행 중 작업이 있으면 진행 중, 폐기 표기가 있으면 폐기) | 결정 파일의 `status` 칸에 판단한 값(`ongoing`, `complete`, `deprecated` 중 하나) |
| 둘 다 없음 | 위와 같음 | 결정 파일의 `status` 칸 |
| 정식 값이 아님(예: `Active (t01 계획)`, `In Progress`) | 위와 같음 | 결정 파일의 `status` 칸 |
| 링크 대상 없음 | 이관 전부터 깨진 링크인지 확인 | 링크를 고치거나 지움(결정 파일에는 목록만) |
| 옮길 자리에 같은 이름 파일 | 어느 쪽이 정본인지 판단 | 중복 파일을 정리(결정 파일에는 목록만) |
| git 작업 트리 미정리 | — | 사용자가 커밋하거나 변경을 치운 뒤 다시 실행 |

### 도구가 하는 일
- **전제**: git 작업 트리가 깨끗해야 함(아니면 update 전체를 적용하지 않음). 결정 파일은 `.elf/runtime/`(gitignore)에 있어 트리를 더럽히지 않음. 되돌릴 때는 git으로 원상복구.
- **순서**: `elf update` → 경고가 없으면 적용 / 있으면 아무것도 쓰지 않고 결정 파일 생성과 화면 요약 → 결정 파일을 채우고 목록의 파일을 고침 → `elf update` 재실행(빈칸이나 잘못된 값이 있으면 그 목록과 함께 거부, 모두 맞으면 적용하고 결정 파일 삭제) → `elf check` → 사용자 확인 뒤 커밋. `elf update --dry-run`은 쓰지 않고 계획과 경고만 보여 줌.
- **결정 파일 형식**: YAML 목록. 항목마다 로그 경로, 헤더 값, Registry 값, 근거(마지막 trial 제목, 결론 줄 유무, 폐기 표기 유무), 경고 종류, 채울 칸 `status: ''`(값으로 정할 수 없는 경고는 칸 없음).
- **이관이 하는 일**: 헤더를 YAML 7키로 변환(status 값 변환 `★ 활성` → `ongoing`, `Complete` → `complete`, `Deprecated`와 `Purged` → `deprecated`, `Date` → created 등 같은 뜻의 필드 대응, Registry Key Finding → conclusion). 7키에 없는 헤더 필드(`관련` 등)와 진행 중 로그의 결론은 제목 아래 본문 한 줄 `> 이관 전 헤더: …`로 남김(값과 링크 유지). 로그를 `2_Log/S/`로 옮기고, 링크와 상대 경로 하나뿐인 코드 span은 실제 파일을 가리킬 때만 새 위치 기준으로 다시 계산. 폐기 기록(`Deprecated/`, `*.partial.md`)은 `.elf/deprecated/` 보관 파일과 표기 줄로 바꿈(이관 전 purge 항목은 보관 파일 없이 `복원 불가` 표기). `Registry.md`와 탐색기 설정 생성, `Session_Registry.tsv` 삭제, manifest를 새 형식(`elf-manifest/2`, `elf_version`)으로. 링크가 아닌 경로 언급은 이관 보고에 목록으로 나옴. `.elf/` 없이 `2_Log/`만 있는 폴더는 로그만 이관함.
- **버전 확인**: `elf check version`이 CLI 버전, 프로젝트 manifest(형식, `elf_version`), 판정(같음, 프로젝트가 낮음 → `elf update`, 프로젝트가 높음 → `elf self update`, 이전 형식 → `elf update`로 이관)을 한 줄로 보여 줌. 쓰기 명령은 시작할 때 같은 판정을 먼저 실행함.
- **CLI 갱신과 이관은 별개**: CLI를 2.28 이상으로 올려도 저장소는 이관되지 않음. 이관 전 저장소에서는 2.27 명령이 그대로 동작(2.28.1부터, 기록마다 안내 1줄)하고, 새 구조 명령(`trial write`, `elf log` 등)은 이관 뒤 사용. 저장소마다 준비됐을 때 `elf update`.
- **구버전 CLI**: 이관 뒤에는 같은 저장소를 쓰는 모든 PC에서 `elf self update`가 필요함. 구버전이 만든 구 위치의 로그는 새 CLI가 다음 빈 번호로 바꿔 `2_Log/S/`로 옮기고 보고함.
