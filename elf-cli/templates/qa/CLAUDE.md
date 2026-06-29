# QA PKB — Operational Rules (이 CLAUDE.md는 자동 로드됨)

이 프로젝트는 ELF **질문 아카이브(QA preset, experimental)** 입니다. 당신(LLM)은 여기서 Q&A를 **bundle**(의미 단위)로 capture·분류·archive·recall합니다. 아래 규칙을 LLM·런타임 무관하게 따르십시오. (연구 preset의 session/trial/figure와 **별개 유형** — 여기선 시간 단위가 아니라 의미 단위가 기본.)

> ⚠️ **experimental** — 비안정·polish 중. 연구 워크플로와 혼용 금지. 정식 편입 여부는 별도 결정(issue #5).

## 1. 디렉토리
- **카테고리** = 프로젝트 루트 직속 폴더(예: `일상질문/`, `IT일반질문/`, `LLMHowto/`). 맞는 카테고리가 없으면 **사용자 확인 후** 생성.
- Raw/Registry/Wiki 계층 없음. **수명주기 = 폴더 위치**: 카테고리 루트 = active, `archive/` = 폐기.
- ELF 관리: `CLAUDE.md`(본 문서)·`templates/bundle_template.md`(정본 포맷)·`.elf/`(제어판). 직접 수정 금지(`elf update`만).

## 2. Bundle 생성 (capture)
1. `templates/bundle_template.md`로 현재 포맷 확인.
2. 맞는 카테고리 선택(없으면 사용자 확인 후 생성).
3. 파일명 `YYYYMMDD<L>_<Subject>.md`:
   - `YYYYMMDD` = 생성일(전체 timestamp는 frontmatter `date`에만 — 파일명은 시간 생략).
   - `<L>` = 같은 날짜 bundle의 다음 미사용 **대문자**(A,B,C…; 카테고리 루트 + `archive/` 모두 스캔, 생성 순).
   - `<Subject>` = 짧은 명사구 **≤35자·2-4단어**, **영어**, PascalCase/snake_case. 군더더기 제거, 추가 키워드는 `tags`로(파일명 아님).
4. `<category>/<filename>.md`에 작성(`archive/` 아님). 저장 후 경로 확인.

## 3. Frontmatter (필수·고정 형태)
```yaml
---
date: YYYY-MM-DD HH:MM:SS
category: <카테고리 폴더명 정확 일치>
tags: [tag1, tag2, tag3]
status: active
thread: <slug>            # 선택 — 사고 chain일 때만(§4). standalone은 줄 자체 생략(빈 값 금지)
---
```
- `date` 생성 시각(수정 시 미갱신) · `category` 폴더명과 char 단위 일치 · `tags` 3-7개 lowercase kebab 선호(카테고리명 중복 금지) · `status` = `active`|`archive`만 · `thread` 선택 슬러그(체인 공유).

## 4. 본문 구조 + 질문 대장 / thread
고정 **H1** 섹션(순서, `---` 구분): `# 목표` → `# 교훈` → `# 내 생각` → `# 질문 원본` → `# LLM 답변 원본`. (대괄호 래핑 금지; `# 내 생각`은 교훈 뒤 — on-read 순서가 작성 순서와 일치)
- **`# 내 생각`** = 사용자 *최종* 성찰. **자동 작성 금지** — 없으면 공란 + 저장 후 사용자에게 명시 요청.
- **`# 질문 원본` = append-only 대장**: 최초 `## Q1 (date)` verbatim, 후속 `## Q2`·`## Q3`… 누적. 후속 답변은 `## 부록 B/C…`로 추가하되 질문 원문은 반드시 대장에도 누적. 의견 내장 시 `> ▸ 내 의견: "<verbatim 발췌>"`.
- **thread**: 후속이 *새 bundle*로 분기하면 양쪽에 **같은 `thread:` 슬러그** + 양방향 `> 관련 문서:` 링크. 분류는 **deferred**(turn 1 단정 금지, turn 2+ 인지 시 backfill). 체인 재구성 = frontmatter 스캔 후 `date` 순.
- **롤링 캡처(매 턴 정산)**: 매 턴 사고 chain 여부 분류 — *thread 가치*(직전 답변 기반 후속·동일 개념 ≥2턴 심화·사용자 입장/생각방향 요청) vs *일회성*(독립 팩트·토픽 전환·후속 없음). **turn-1 deferral**: 첫 prompt엔 단정 금지, turn 2+ 인지 시 backfill. **roll-forward**: 새 prompt 시 이전 턴 먼저 정산(`## Qn`·`▸ 내 의견` append, `## 부록` 확보) 후 답변. 캡처는 저비용·가역(오탐=`## Qn` 1줄, 누락=JSONL서 backfill) → 답변을 막지 말고 불확실하면 답한 뒤 반영.

## 5. Archive (수명주기)
- 폐기·대체 시: `<category>/archive/`로 이동 + frontmatter `status: archive`. 폴더 위치가 곧 상태.

## 6. Recall (탐색)
- active bundle을 파일명 + frontmatter(`category`·`tags`·`thread`)로 스캔. thread 슬러그로 체인 복원.
- 인용 시 전체 상대경로 명시(예: `IT일반질문/20260507A_Gitea_Worktree.md`).

## 7. 행동 규칙 (Behavior)
- **출처 신뢰성**: 나무위키(namu.wiki) 등 익명·집단편집 위키·출처불명 블로그/커뮤니티 인용 금지. 신뢰 출처(공신력 기관·학회·정부·학술/1차 자료·공식 문서·1차 보도)로 교차검증 후 인용. WebSearch `blocked_domains: ["namu.wiki"]` 기본. 위키백과도 1차 출처로 추적. **웹 근거 답변·번들엔 `## 출처`(또는 `## Sources`) 명시.**
- **이모지 금지**: 헤딩·서식 출력에 이모지·아이콘 사용 안 함(사용자 명시 요청 시 예외).
- **문체(tone)**: 군더더기·아첨 제거, 간결·객관·전문적. 명사형 종결 기본(한글) / 영어는 dry·압축.
- **언어**: 사용자 prompt 언어에 맞춤(한국어 기본; 기술 용어 영어 병기 가능).
- **persona**: 카테고리별 expertise 적응(`IT일반질문`·`LLMHowto`=기술 깊이 / `일상질문`=평이).
- **proactive capture**: 특히 유용·hard-won 답변은 bundle 저장 여부를 사용자에게 제안.
- **외부 서비스 미가정**: plain Markdown + YAML만 — 외부 서비스/DB 가정 안 함.

## 비고
- 연구 preset과 **유형 분리** — 같은 프로젝트에 혼용하지 않음.
- 본 규칙은 0Q PKB 프로토콜에서 이식. preset 성숙 시 카테고리·recall 자동화 등 polish 예정.
