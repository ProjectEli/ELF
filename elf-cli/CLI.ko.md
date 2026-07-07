[English](CLI.md) | [한국어](CLI.ko.md)

# elf CLI 레퍼런스

`elf`는 ELF(Eli's Lab Framework)의 명령행 도구입니다 — 연구 프로젝트 스캐폴드 생성, 프레임워크 파일 갱신, drift 진단을 수행합니다. Node·Python 런타임이 필요 없는 자기완결 단일 바이너리로 배포됩니다.

> 프레임워크 철학·폴더 구조: [README.ko.md](../README.ko.md). 본 문서는 명령 레퍼런스입니다.

## 설치

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

바이너리가 `~/.elf/bin`에 설치되고 PATH에 등록됩니다. 새 셸을 열어 `elf --version`으로 확인하세요. 도구 자체 갱신은 `elf self-update`(또는 인스톨러 재실행).

## 명령 요약

| 명령 | 용도 |
|------|------|
| `elf init [이름]` | ELF 프로젝트 생성 (현재 폴더 또는 `./<이름>`) |
| `elf update` | 현재 프로젝트의 managed 파일 갱신 |
| `elf migrate` | managed payload를 `.elf/managed/`로 이전(opt-in, legacy 프로젝트) |
| `elf status` | managed 파일 drift 진단(읽기전용) |
| `elf validate` | 세션/Registry/로그 정합 검사(읽기전용) |
| `elf session new <제목>` | 다음 세션 로그 생성 + 등록 |
| `elf session close [S###]` | 활성 세션 종료 → Archive 이동 + Registry 갱신 |
| `elf session fix-headers` | 세션 로그 헤더 렌더링 보정 |
| `elf trial new [제목]` | 활성 세션 로그에 정본 trial stub 추가 |
| `elf gallery` | `6_Exp/64_Viz/`에서 Figure 색인 `_gallery.md` 생성 |
| `elf doctor` | 환경+프로젝트 종합 건강검진(읽기전용) |
| `elf self-update` | `elf` 바이너리 자체 갱신 |

전역 플래그: `elf --version`, `elf --help`, `elf <명령> --help`.

## 명령

### `elf init [이름]`

ELF 프로젝트를 생성합니다. **이름을 생략하면 `elf init`은 현재 폴더를 제자리(in-place)로 초기화**합니다(`git init`처럼). 이름을 주면 새 `./<이름>/` 하위폴더를 만듭니다.

| 플래그 | 기본값 | 의미 |
|--------|--------|------|
| `--here` | off | 이름을 줘도 현재 폴더에 in-place 강제(그 이름을 프로젝트명으로 사용) |
| `--yes` | off | in-place 확인 프롬프트 생략(스크립트/CI) |
| `--dry-run` | off | 계획만 출력, 미기록 |
| `--force` | off | 기존 파일도 덮어씀(기본은 사용자 것 유지) |
| `--preset <p>` | `full` | 모듈 세트: `full` / `experimental` / `software` / `minimal`. 실험적 유형: **`qa`**(질문 아카이브, 연구 아님), **`general`**(목표지향 비연구) |
| `--modules <목록>` | — | custom 모듈(쉼표): `hw,fab,sw,exp,paper`. `--preset`보다 우선 |
| `--lang <언어>` | `ko-KR` | AI 에이전트 응답 언어, BCP-47 태그(`.elf/config.json`에 기록). 비한국어 태그(예: `en-US`)면 영어 **companion** 문서도 함께 배포 — 아래 note 참조 |

Core 폴더(`0_Meta`–`2_Log`)는 항상 생성, 모듈 폴더(`3_HW`–`7_Paper`)는 preset/`--modules`에 따라 추가. ELF 관리 규칙 payload(규칙+로그 형식 stub)는 `.elf/managed/`에 배포되고, `0_Meta/`는 사용자 소유 파일(`ProjectRule.md`·data overlay)만 둡니다.

**In-place(이름 생략)**는 기존 폴더에 ELF를 **아무것도 덮어쓰지 않고** 도입합니다: 누락된 ELF 파일만 추가하고 사용자 파일은 유지하며, 충돌하는 ELF managed 파일은 `<파일>.elf-new`로 병기합니다(사용자 `.gitignore`·`README` 등은 절대 미클로버). 폴더가 비어있지 않으면 경로를 echo하는 확인 1회를 거치고, 프로젝트명은 폴더명으로 기본 설정됩니다. 이미 ELF 프로젝트(`.elf/` 존재)면 exit 3로 거부 — `elf update`를 사용하세요. 이름(하위폴더) 형식은 `./<이름>`이 이미 있으면 종전대로 exit 3 거부.

```bash
elf init                              # in-place: 현재 폴더에 ELF 도입
elf init --here --preset general      # in-place, 목표지향 유형
elf init . --yes                      # in-place, 프롬프트 없이(스크립트/CI)
elf init NIRS_Probe                   # 하위폴더: ./NIRS_Probe/ 생성
elf init NIRS_Probe --preset experimental --lang en-US
elf init NIRS_Probe --modules hw,sw
elf init my_questions --preset qa                          # experimental: Q&A bundle 아카이브 (카테고리 0개)
elf init my_questions --preset qa --categories 일상질문,IT일반질문   # 카테고리 사전 생성
```

> **`--lang en-*` (영어 companion, experimental).** operative 거버넌스 문서는 한국어(`*.md`)
> 유지 — AI는 항상 한국어 정본으로 동작하므로 언어와 무관하게 프로젝트 동작이 동일함. 비한국어
> `--lang`이면 ELF가 영어 **정보용 companion**(`0_Meta/EliRule.en.md` 등)을 인간 독해용으로 추가
> 배포하고, 사용자 소유 `README.md`·`ProjectRule.md`는 영어로 scaffold함. companion은 비operative
> (`NOT OPERATIVE` 표기) — 규칙 커스터마이즈는 companion이 아니라 `ProjectRule.md`에 작성. `elf update`가
> companion을 동기화, `elf doctor`가 i18n 상태 보고. 현재 영어(`en`)만 제공, 그 외 언어는 한국어 fallback.

> **`qa` preset (experimental).** 연구 계층 대신 *질문 아카이브* 유형을 스캐폴드: 루트 `AGENTS.md`(운영 규칙 정본) + `CLAUDE.md`(로드용 포인터) + `templates/bundle_template.md` + `.elf/`. **기본은 카테고리 0개** — `CLAUDE.md` 규약대로 수요 기반 생성, 또는 `--categories a,b,c`로 사전 생성(각 `archive/` 동반). Q&A를 의미 단위 **bundle**로 기록(session/trial/figure 아님). `.elf/` 제어판 공유 + 자체 manifest(`manifest.qa.json`) → `elf update`로 규칙 전파. 연구 preset과 격리, polish 중이라 변경 가능.

> **`general` preset (experimental).** *목표지향 비연구* 프로젝트(도구 개발·제안서·학습·구축)를 스캐폴드 — 연구 preset처럼 session/trial base-delta 로깅, 단 학술 레이어 제외(`6_Exp`/`7_Paper`/figure/sim/문헌 없음). 중립 managed 파일은 연구 preset과 공유하고 general 전용 `EliRule`/`LogConvention`을 자체 manifest(`manifest.general.json`)로 추가. trial 형식 기본 5-section, 프로젝트별 `ProjectRule.md`에서 override. 연구와 격리, polish 중이라 변경 가능.

### `elf update`

현재 프로젝트의 ELF 관리 파일을 설치된 CLI 버전으로 갱신합니다. **연구 데이터·로그·설정은 절대 건드리지 않습니다.**

| 플래그 | 의미 |
|--------|------|
| `--dry-run` | 변경 없이 작업 목록만 출력 |
| `--force` | 사용자 편집 managed 덮어쓰기 / hybrid 블록 교체 |
| `--self` | 프로젝트가 아니라 `elf` 바이너리 자체 갱신(self-update의 alias) |

파일 종류별 동작 → [파일 소유권](#파일-소유권) 참조.

```bash
elf status            # 무엇이 바뀔지 확인
elf update --dry-run  # 작업 미리보기
elf update            # 적용(기본 안전)
```

### `elf migrate [--dry-run]`

**legacy 레이아웃** 프로젝트의 관리 규칙 payload(`0_Meta/`·`templates/`)를 `.elf/managed/`로 이전하고 `.elf/config.json`에 `layout: managed`를 기록합니다. **opt-in 전용** — `elf update`는 절대 자동 수행하지 않으며, legacy 프로젝트는 이전 없이도 구 경로에서 계속 동작합니다.

| 플래그 | 의미 |
|--------|------|
| `--dry-run` | 이동 계획만 출력, 무이동 |

동작:

- 이동 전 전체 계획 수립·충돌 전수 검증 — 구·신 위치에 파일이 동시에 존재하면 무이동 거부(exit 3).
- git 작업트리에 미커밋 **추적** 변경이 있으면 거부(exit 3) — 이전만 선별 되돌릴 수 있도록 먼저 커밋.
- 미병합 `<파일>.elf-new` 병기본도 base와 함께 이동.
- 사용자 소유물(`0_Meta/ProjectRule.md`·로그·데이터)은 그대로 두고, `.md` 파일 내 구경로 참조는 **보고만**(내용 재작성 없음).
- 멱등: 이미 managed 레이아웃이면 "nothing to do" 출력 후 exit 0.

```bash
elf migrate --dry-run   # 이전 미리보기
elf migrate             # 이전 후 `git status` 확인·커밋
```

### `elf status [--check]`

관리 파일 상태 진단(읽기전용). 각 파일을 `ok` / `outdated` / `missing` / `edited`로 보고하고, 버전 불일치·obsolete 항목도 표시.

- `--check`: 발견 시 exit **4** — pre-commit 훅·CI 게이트로 사용.

### `elf validate [--check] [--strict]`

세션 장부 정합을 검사(읽기전용): Registry ↔ 로그 파일(미등록 로그 / 유령 행), 세션 번호(중복 / gap), 활성 세션 복수, 로그 내 깨진 상대 `.md` cross-ref, **figure-embed 누락**(`6_Exp/64_Viz/S###/`에 그림이 있으나 해당 세션 로그 본문에 인라인 임베딩 안 됨 — 표에 경로만 기재한 것은 embed 아님), **trial 구조**(활성 로그의 비정본 `###` 헤딩·절 순서·`### 해석` 첫 줄 `가설 적중 여부` 규칙·`### 관찰` 존재 시 Phase 1 절 누락). 구조 검사는 `Archive/` 제외(소급 정책 — 신규 작성분부터) + 형식의 안정 코어만 대상(내용 품질은 기계 검사 안 함).

- **issue**(Registry/로그 불일치·번호 중복·깨진 링크) vs **warning**(번호 gap·활성 복수·figure-embed 누락·trial 구조) — issue만 게이트.
- `--check`: issue 발견 시 exit **4** — pre-commit/CI 게이트.
- `--strict`: figure-embed 누락·trial 구조 발견을 warning→issue로 승격(→ `--check`가 게이트).
- 의도적 비임베딩(SI/폐기 figure)은 로그에 `<!-- noembed: filename.png -->` 주석으로 제외.
- Registry 자체가 파싱 불가면 exit **5**(escalation) — "문제 발견"과 "검사 불능"을 구분.

```bash
elf validate                   # 보고 (figure-embed 누락은 warning)
elf validate --check            # issue 시 exit 4 (CI 게이트)
elf validate --check --strict   # figure-embed 누락도 게이트
```

### `elf session new <제목>`

다음 세션 로그(`2_Log/` + `2_Log/Archive/` + Registry에서 `S###` 자동 증번)를 템플릿에서 생성하고 `2_Log/Wiki/Session_Registry.tsv`에 등록합니다.

```bash
elf session new "Wavelength Optimization"
# → 2_Log/S002_log.md 생성 + Registry 행 추가
```

제목에 탭 문자는 불가(TSV Registry 파탄). Registry를 파싱할 수 없으면 기록하지 않고 exit **5**(escalation — 아래 참조).

### `elf session close [S###]`

세션을 종료합니다: 헤더 `Status`를 `Complete`로, 로그를 `2_Log/Archive/`로 이동(파일명 그대로 — 폴더 위치가 곧 상태), Registry 행 갱신. id 생략 시 유일한 활성 세션을 자동 선택하며, 활성 세션이 여럿이면 목록을 보여주고 하나를 지정하도록 요구합니다.

| 플래그 | 의미 |
|------|------|
| `--force` | `## 다음 세션 후보` 섹션이 비어 있어도 강제 종료 |

```bash
elf session close             # 유일한 활성 세션 종료
elf session close S007        # 특정 세션 종료
```

다음 세션 후보 섹션이 미작성이면 exit **3**으로 거부(작성하거나 `--force`). 헤더 `Handoff`에 미완료 항목이 남아 있으면 **비차단 경고** 출력 — 해소하거나 다음 세션 후보로 이관; 종료 후에는 Registry key finding을 최종 결론(fold)으로 재작성하라는 안내가 출력됩니다. Registry 파싱 불가 시 exit **5**(escalation).

### `elf session fix-headers [--dry-run]`

기존 세션 로그의 인용구 헤더에 CommonMark hard break(`\`)를 부여해, strict Markdown 렌더러(예: Discord 미리보기)에서 메타데이터가 줄 단위로 표시되게 합니다. 멱등·CRLF-safe. `2_Log/` + `2_Log/Archive/` 대상.

### `elf trial new [제목]`

**현행 정본 trial stub**(`templates/trialTemplate.md`, CLI에 embed)을 활성 세션 로그에 추가합니다: 다음 `t##` 자동 증번, `S{NNN}` 경로 치환, 헤더 `Modified` 날짜 갱신, `## 다음 세션 후보` 섹션 앞에 삽입. 제목 생략 시 `[작업 제목]` placeholder 유지.

| 플래그 | 의미 |
|--------|------|
| `--session <S###>` | 활성 세션이 여럿일 때 대상 지정 |

```bash
elf trial new "파장 스윕 v2"      # 유일 활성 세션에 t## 추가
elf trial new --session S007     # 활성 복수 — 대상 지정
```

명령이 존재하는 이유: 에이전트(와 사람)는 직전 trial의 모양을 모방합니다. `elf trial new`는 모방 대상을 정본으로 유지합니다 — stub이 항상 *설치된* 템플릿 버전에서 나오므로, drift된 선례가 전파되지 않습니다. 오류: 활성 세션 없음 → exit 1(`elf session new`로 시작); `--session` 없이 활성 복수 → exit 1 + 목록 안내.

### `elf gallery`

`6_Exp/64_Viz/`를 스캔해 `6_Exp/64_Viz/_gallery.md`(세션 하위 디렉토리별 Figure 색인)를 재생성합니다. 각 `.png`/`.jpg`/`.svg`가 이미지 링크로 삽입되고, 이미지 없는 세션은 건너뜁니다. `6_Exp/64_Viz/`가 없으면 안내 후 exit **0**(할 일 없음).

```bash
elf gallery
# → wrote 6_Exp/64_Viz/_gallery.md (3 session(s), 12 image(s))
```

### `elf self-update`

`elf` 바이너리 자체를 최신 릴리즈로 갱신합니다. 인스톨러로 설치된 경우 동작(install receipt 사용), 아니면 인스톨러 명령을 안내. `elf update --self`로도 호출 가능.

### `elf doctor`

종합 건강검진(읽기전용). 각 항목을 `OK` / `WARN` / `INFO`로 보고:

- **환경** — `elf` 버전, install receipt 유무(self-update 가능 여부)
- **프로젝트**(프로젝트 내일 때) — `.elf/` stamp 파싱·version이 CLI와 일치·baseline 존재·레이아웃(managed / legacy면 `elf migrate` 안내)
- **managed 파일** — `elf status` 요약(pending / conflict)
- **overlay** — 활성 data overlay(`0_Meta/<이름>.project.md`)·제외 사유 누락·비허용 대상 overlay
- **agent entry** — `CLAUDE.md`가 `@AGENTS.md`를 로드하는지(포인터 줄 부재 시 경고 — Claude Code가 규칙을 로드하지 못함), 포인터에 과다 콘텐츠·`AGENTS.md.elf-new`/`CLAUDE.md.elf-new` 대기 여부
- **git** — 저장소·`pre-commit` 훅 존재

프로젝트 밖에서도 동작(환경 검사만). 네트워크를 쓰지 않으며 항상 exit **0**.

```bash
elf doctor
```

## Exit code

| 코드 | 의미 |
|------|------|
| 0 | 성공 |
| 1 | 실행 오류(I/O, ELF 프로젝트 아님 등) |
| 2 | usage 오류(잘못된 플래그·인자) |
| 3 | 거부(예: 대상이 이미 존재) |
| 4 | `--check` 발견 있음(게이트 신호, 실제 오류와 구분) |
| 5 | **escalation** — 사람/에이전트 판단 필요(아래 참조) |

## Escalation (exit 5)

결정론적 작업이 자동 수정해선 안 되는 상황(예: 손상된 `Session_Registry.tsv`)을 만나면, `elf`는 추측하지 않습니다. stderr에 구조화 보고를 출력하고 exit 5로 종료합니다:

```
[elf] escalation: 2_Log/Wiki/Session_Registry.tsv (line 7)
  expected: 6 tab-separated columns (Session/Date/Title/Status/Key Finding/Archive Path)
  found:    4 columns
  raw:      <문제의 행>
agent-action: fix the line to match the schema, then re-run (this tool will not auto-edit)
```

`agent-action:` 줄은 고정 마커입니다. `elf`를 구동하는 LLM 에이전트가 이를 인식해 파일을 수리하고 재실행할 수 있습니다. exit 5는 exit 1(일반 오류)과 의도적으로 구분되어, 자동화가 "파일 수정으로 풀림"과 "재시도 무의미"를 분기할 수 있습니다.

## 파일 소유권

`elf update`는 4가지 소유권 tier를 지킵니다:

| Tier | 파일 | update 동작 |
|------|------|-------------|
| **Managed** | `.elf/managed/`(`EliRule.md`·`LogConvention.md`·`AI_PARA_Framework.md`·`highIFjournals.md`·`LLMcliche.md`·`templates/*`·companion), 루트 `.claudeignore`·`.editorconfig`·`AGENTS.md` | 새 버전으로 교체. 편집한 경우 **보존**하고 새 버전을 `<파일>.elf-new`로 생성(`--force`로 덮어쓰기) |
| **사용자 소유** | `0_Meta/`(`ProjectRule.md`·`<이름>.project.md` overlay), `Session_Registry.tsv`, `README.md`, 모든 연구 데이터·로그 | **절대 미접근** |
| **Hybrid** | `.gitignore` | 마커 블록(`# >>> ELF managed >>>` … `# <<< ELF managed <<<`)만 교체, 블록 밖 사용자 규칙은 보존 |
| **Pointer** | `CLAUDE.md` | 없으면 생성, 있으면 **절대 불변경**(`.elf-new` 병기도 없음) — 수제 `CLAUDE.md`는 온전히 사용자 것. ELF 규칙 로드는 `@AGENTS.md` 1줄을 직접 추가; 연결 여부는 `elf doctor`가 점검 |

legacy 레이아웃(`elf migrate` 미실행 프로젝트)에서는 managed 파일이 종전대로 `0_Meta/`·`templates/`에 있으며, update도 이전 전까지 그 경로를 유지합니다.

프로젝트 규칙은 managed 파일을 고치는 대신 `ProjectRule.md`(사용자 소유)에 작성하세요 — 그러면 update 충돌이 없습니다. data 파일(`LLMcliche.md`·`highIFjournals.md`)의 항목 추가·제외·재정의는 **project overlay** `0_Meta/<이름>.project.md`로 선언합니다(사용자 소유; 유효 규칙 = base ⊕ overlay; 제외는 사유 필수 — EliRule §2.7).

## `.elf/` 디렉토리

`elf init`/`update`가 관리합니다(직접 수정 금지):

- `config.json` — 프로젝트 이름·언어·생성일·레이아웃(`managed` = payload가 `.elf/managed/`; 부재 = legacy)
- `version` — 프로젝트를 마지막으로 건드린 ELF 버전
- `manifest.json` — `update`/`status`가 쓰는 관리 파일 기록
- `managed/` — 배포된 규칙 payload(규칙·companion·로그 형식 stub, managed 레이아웃)
- `baseline/` — hybrid 파일의 원본 사본(블록 내 편집 감지용)
