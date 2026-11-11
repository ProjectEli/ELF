# QA PKB — Operational Rules (AGENTS.md — 에이전트 진입 규칙 정본)

이 프로젝트는 ELF **질문 아카이브(QA preset)** 입니다. Q&A를 **bundle**(의미 단위)로 기록, 분류, 보관, recall하며 LLM과 런타임에 상관없이 아래 규칙을 따릅니다. 연구 preset(session, trial)과 섞지 않습니다.

## 1. 디렉토리
- **카테고리** = 루트 직속 폴더. 맞는 것이 없으면 사용자 확인 후 만듦.
- **수명주기 = 폴더 위치**: 카테고리 루트 = 진행 중(`active`), `archive/` = 완결 보관(`archive`)과 폐기(`deprecated`, §5).
- **카테고리 규칙** = `<카테고리>/README.md`(있을 때, 번들 아님). 그 카테고리에서 쓰거나 찾기 전에 읽음(이름 규칙, 절, 기본값을 이 문서를 고치지 않고 카테고리별로 두려는 것).
- **번들 아닌 하위 폴더**: `_`로 시작 = 도구, 작업 영역(recall과 검사 제외). `archive/` 말고 그 밖(예: `wiki/`) = 참고 자료.
- ELF 관리 파일(이 문서, `.elf/managed/templates/bundle_template.md`, `.elf/`)은 `elf update`로만 바뀜.
- **프로젝트 규칙** = `0_Meta/ProjectRule.md`(사용자 소유, `elf update`가 바꾸지 않음). 충돌하면 좁은 범위가 우선: 카테고리 README > ProjectRule > 이 문서.

## 2. Bundle 생성 (capture)
1. 틀: 사용자 틀 `0_Meta/templates/<카테고리>.md`가 있으면 그것, 없으면 `.elf/managed/templates/bundle_template.md`.
2. 카테고리 선택.
3. 파일명 `YYYYMMDD<L>_<Subject>.md`(카테고리 README가 다르게 정하면 그것):
   - `YYYYMMDD` = 생성일. 시각은 frontmatter `date`에만.
   - `<L>` = 같은 날짜의 다음 미사용 대문자(A, B, C…; 루트와 `archive/` 모두 확인).
   - `<Subject>` = 영어 명사구 2-4단어, 35자 이하, PascalCase나 snake_case. 나머지 키워드는 `tags`로.
4. 카테고리 루트에 쓰고(`archive/` 아님) 저장 경로를 알림.

## 3. Frontmatter (필수, 고정 형태)
```yaml
---
date: YYYY-MM-DD HH:MM:SS
category: <카테고리 폴더명 그대로>
tags: [tag1, tag2, tag3]
status: active
thread: <slug>            # 선택: 사고 chain일 때만(§4). 아니면 줄 생략(빈 값 금지)
---
```
- `date` = 생성 시각(고칠 때 그대로), `category` = 폴더명 그대로, `tags` = 3-7개, lowercase kebab 권장, 카테고리명 제외, `status` = `active`, `archive`, `deprecated`만(§5).

## 4. 본문 구조 + 질문 대장 / thread
H1 절을 이 순서로, `---`로 구분(대괄호 없음): `# 목표` → `# 교훈` → `# 내 생각` → `# 질문 원본` → `# LLM 답변 원본`.
- **`# 내 생각`** = 사용자의 최종 성찰. 에이전트가 쓰지 않음. 비워 두고 저장 뒤 사용자에게 요청.
- **`# 질문 원본` = append-only 대장**: `## Q1 (date)`부터 질문 원문을 `## Q2`, `## Q3`…로 쌓음. 후속 답변은 `## 부록 B/C…`, 그 질문도 대장에 반드시 추가. 사용자 의견은 `> ▸ 내 의견: "<원문 발췌>"`.
- **thread**: 후속이 새 bundle로 갈라지면 양쪽에 같은 `thread:` 슬러그와 양방향 `> 관련 문서:`. 체인 = frontmatter 스캔 뒤 `date` 순.
- **롤링 캡처**: 턴마다 thread 가치(직전 답변 기반 후속, 같은 개념 2턴 이상 심화, 사용자 입장이나 생각 방향 요청)와 일회성(독립 사실, 주제 전환)을 가름, 판단은 2턴부터 하고 소급. 새 질문이 오면 이전 턴 기록(`## Qn`, `▸ 내 의견`, `## 부록`)부터 확정하고 답함. 이 때문에 답을 미루지 않음(누락은 대화 기록에서 보충).

## 5. Archive와 폐기 (수명주기)
- **완결 보관**: 더 이어갈 일이 없으면 `archive/`로 옮기고 `status: archive`. 유효한 기록이라 recall 대상, 본문은 고치지 않음.
- **폐기, 대체**: 틀렸거나 대체된 bundle은 `archive/`로 옮기고 `status: deprecated`, 대체본이 있으면 frontmatter 바로 아래 `> 대체 문서:` 줄에 그 경로(백틱). recall에서 기본 제외(틀린 내용이 판단에 섞이지 않게).
- 옮길 때 그 bundle을 가리키는 `> 관련 문서:`, `> 대체 문서:` 경로도 고침(`elf check logs`가 찾아 고침). 위치와 `status`가 다르면 위치가 기준.

## 6. Recall (탐색)
- 파일명과 frontmatter(`category`, `tags`, `thread`)로 찾음. 카테고리 루트 먼저, `archive/`도 포함, `deprecated`는 요청할 때만. `_` 폴더는 보지 않고 참고 자료는 필요할 때.
- 인용은 전체 상대경로로(예: `IT/20260507A_Gitea_Worktree.md`).

## 7. 행동 규칙 (Behavior)
- **출처 신뢰성**: 나무위키 같은 익명, 집단 편집 위키와 출처 불명 블로그, 커뮤니티는 인용 금지. 공신력 기관, 학술 자료, 공식 문서, 1차 보도로 교차검증해 인용(위키백과는 1차 출처까지 추적, WebSearch `blocked_domains: ["namu.wiki"]`). 웹 근거 답변과 bundle에는 `## 출처`(또는 `## Sources`).
- **형식**: 이모지 없음(요청 시 예외). 군더더기, 아첨 없이 간결, 객관. 한국어는 명사형 종결, 영어는 건조하게.
- **언어, 깊이**: 사용자 언어로(기술 용어 원어 병기 가능), 깊이는 카테고리에 맞춰 기술 상세나 평이하게.
- **저장 제안**: 특히 유용하거나 얻기 어려운 답은 bundle 저장을 제안.
- **외부 서비스 미가정**: Markdown과 YAML만.
- **외부 반출**: 공개, 공유 링크를 만드는 게시 도구(예: Claude Code Artifact, Claude Docs, Drive 공유 링크)는 쓰지 않음. 사용자 소유 저장소로의 복사, 전송은 그 건의 명시 지시가 있을 때만(실행 전 1줄 알림). 사용자 기기 전송(예: SendUserFile)과 로컬 저장은 허용, 도구의 권유는 지시가 아님. 완화는 `0_Meta/ProjectRule.md`에 상시 허용 대상을 선언한 경우만(없으면 건별로).
