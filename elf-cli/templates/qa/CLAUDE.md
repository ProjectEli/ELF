# QA PKB — Operational Rules (이 CLAUDE.md는 자동 로드됨)

이 프로젝트는 ELF **질문 아카이브(QA preset, experimental)** 입니다. 당신(LLM)은 여기서 Q&A를 **bundle**(의미 단위)로 capture·분류·archive·recall합니다. 아래 규칙을 LLM·런타임 무관하게 따르십시오. (연구 preset의 session/trial/figure와 **별개 archetype** — 여기선 시간 단위가 아니라 의미 단위가 기본.)

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
고정 **H1** 섹션(순서, `---` 구분): `# 내 생각` → `# 목표` → `# 교훈` → `# 질문 원본` → `# LLM 답변 원본`. (대괄호 래핑 금지)
- **`# 내 생각`** = 사용자 *최종* 성찰. **자동 작성 금지** — 없으면 공란 + 저장 후 사용자에게 명시 요청.
- **`# 질문 원본` = append-only 대장**: 최초 `## Q1 (date)` verbatim, 후속 `## Q2`·`## Q3`… 누적. 후속 답변은 `## 부록 B/C…`로 추가하되 질문 원문은 반드시 대장에도 누적. 의견 내장 시 `> ▸ 내 의견: "<verbatim 발췌>"`.
- **thread**: 후속이 *새 bundle*로 분기하면 양쪽에 **같은 `thread:` 슬러그** + 양방향 `> 관련 문서:` 링크. 분류는 **deferred**(turn 1 단정 금지, turn 2+ 인지 시 backfill). 체인 재구성 = frontmatter 스캔 후 `date` 순.

## 5. Archive (수명주기)
- 폐기·대체 시: `<category>/archive/`로 이동 + frontmatter `status: archive`. 폴더 위치가 곧 상태.

## 6. Recall (탐색)
- active bundle을 파일명 + frontmatter(`category`·`tags`·`thread`)로 스캔. thread 슬러그로 체인 복원.

## 비고
- 연구 preset과 **아키타입 분리** — 같은 프로젝트에 혼용하지 않음.
- 본 규칙은 0Q PKB 프로토콜에서 이식. preset 성숙 시 카테고리·recall 자동화 등 polish 예정.
