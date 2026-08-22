<!-- ARCHIVED TEMPLATE — 구 sessionTemplate(v2.19 이하, `## 다음 세션 후보` 절 포함). v2.20부터 비권장(off): 신규 세션 정본 = ../sessionTemplate.md(후보 절 없음). 하위호환 보존용 — 기존 로그의 절은 유효하며 `elf trial new`는 절이 있으면 그 앞에 삽입.
     Archived session template (v2.19 and earlier, with the next-session section). Not recommended since v2.20; canonical = ../sessionTemplate.md. Kept for backward compatibility — the section in existing logs stays valid. -->

# S{NNN}: [세션 제목]

> **Created**: YYYY-MM-DD\
> **Modified**: YYYY-MM-DD\
> **Status**: ★ 활성\
> **목표**: [이 세션의 핵심 목표 1-2문장]\
> **관련**: [관련 세션/문서, 예: S000, P001_xxx.md]\
> **Handoff**: -

---

<!-- ELF 규범: 형식·규칙의 정본 = .elf/managed/LogConvention.md + .elf/managed/templates/trialTemplate.md — 선례(과거 세션/trial)와 다르면 정본 우선, 선례 모방 금지.
     trial 추가 = `elf trial new`(정본 stub append; CLI 미설치 시 .elf/managed/templates/trialTemplate.md 수동 복사). 컨텍스트 재구성(compact) 후 이 헤더의 Handoff 재독. -->

## t01: [작업 제목]

### 목표 (Goal)
- [이 작업의 구체적 목표]

### 조건 (Conditions)
- [파라미터, 설정, 제약 조건]

### 가설 (Hypothesis)
- [Phase 1, 실행 전 작성 — 메커니즘·효과 의심 3-5항, 명사형]

### 예상 (Prediction)
- [Phase 1, 실행 전 작성 — 결과 수치/방향 1-3항]
- [figure 산출 trial: 예상 산출물(figure·표) 목록 — 각 항목 = 생성 즉시 embed 대상]

### 관찰 (Observation)
- [Phase 2, 실행 후 작성]

### 해석 (Interpretation)
- 가설 적중 여부: [적중 / 탈락 / 부분 적중]
- [메커니즘 해석]

### 교훈 (Lessons)
- [이 작업에서 얻은 핵심 인사이트]

### 생성 파일 (Files)

| 유형 | 파일 |
|------|------|
| Script | `경로` |
| Output | `경로` |
| Figure | `경로` |

---

## 다음 세션 후보 (Next-Session Hypothesis)

> 세션 종료 시 작성. 현 세션 결과로부터 자연스럽게 떠오른 후속 가설/예상을 명사형 1-3항으로 정리. 다음 세션 S{NNN+1} 시작 시 t01 `### 가설`·`### 예상`으로 carry-over.

### 가설 후보
- [후속 가설 1-3항]

### 예상 후보
- [위 가설의 예상 결과 1-3항]
