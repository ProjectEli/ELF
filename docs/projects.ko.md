[English](projects.md) | [한국어](projects.ko.md)

# 프로젝트 생성과 갱신

## `elf init [이름]`

ELF 프로젝트를 만듭니다. **이름을 생략하면 현재 폴더를 그 자리에서(in-place) 초기화**합니다(`git init`과 같음). 이름을 주면 `./<이름>/` 폴더를 새로 만듭니다.

| 플래그 | 기본값 | 의미 |
|--------|--------|------|
| `--here` | 꺼짐 | 현재 폴더에 만들면서 이름을 지정(`elf init --here <이름>`). 이름 없이 쓰면 원래대로 현재 폴더에 만듦 |
| `--yes` | 꺼짐 | in-place 확인 질문을 건너뜀(스크립트, CI) |
| `--dry-run` | 꺼짐 | 계획만 출력하고 아무것도 쓰지 않음 |
| `--force` | 꺼짐 | 기존 파일도 덮어씀(기본은 사용자 파일 유지) |
| `--preset <p>` | `full` | 모듈 구성: `full`(모든 모듈), `exp`(6_Exp, 7_Paper), `sw`(5_SW, 7_Paper), `minimal`(모듈 없음). 연구용 preset은 이름과 같은 주 모듈에 `paper`가 붙습니다. 2.26 이전 이름(`experimental`, `software`)도 받습니다. **`qa`**는 질문 아카이브 유형입니다(연구용 아님). `general` preset은 2.27에서 삭제되었으므로 `minimal`을 씁니다 |
| `--modules <목록>` | 없음 | 모듈 직접 지정(쉼표로 구분): `hw,fab,sw,exp,paper`. `--preset`보다 우선하며, `.elf/config.json`에 `"preset": "custom"`과 모듈 목록으로 기록 |
| `--lang <언어>` | `ko-KR` | AI 에이전트의 응답 언어, BCP-47 태그(`.elf/config.json`에 기록). 영어 태그(`en-*`)면 영어 규칙 세트로 설치(아래 참고) |

Core 폴더(`0_Meta`~`2_Log`)는 항상 만들고, 모듈 폴더(`3_HW`~`7_Paper`)는 preset이나 `--modules`에 따라 추가합니다. ELF 관리 규칙(규칙과 로그 형식 틀)은 `.elf/managed/`에 설치하고, `0_Meta/`에는 사용자 소유 파일(`ProjectRule.md`, 프로젝트별 추가 규칙)만 둡니다.

**In-place(이름 생략)** 초기화는 기존 폴더에 ELF를 **아무것도 덮어쓰지 않고** 들입니다. 없는 ELF 파일만 추가하고 사용자 파일은 그대로 두며, 이름이 겹치는 ELF 관리 파일은 `<파일>.elf-new`로 따로 만듭니다(사용자의 `.gitignore`, `README` 등은 덮어쓰지 않음). 폴더가 비어 있지 않으면 경로를 보여 주고 한 번 확인하며, 프로젝트 이름은 폴더 이름을 씁니다. 이미 ELF 프로젝트(`.elf/manifest.json`이 있음)면 exit 3으로 거부하므로 `elf update`를 씁니다.

`.elf/`는 있는데 `.elf/manifest.json`이 없는 옛 버전 ELF 폴더(`elf update`가 거부함)도 같은 방식으로 들입니다. 이때 기존 `.elf/config.json`의 키(`created` 포함)를 유지하고, `--preset`, `--lang`, 이름은 그 안의 값을 기본으로 쓰며, 이전 형식의 세션 로그는 `elf update`가 이관하도록 그대로 둡니다(출력에서 알림). `elf init`은 세션을 만들지 않으므로 첫 세션은 `elf session new`로 시작합니다. 들인 뒤 남은 일은 `elf check files`가 알려 줍니다. `0_Meta/`에 남은 옛 규칙 사본(현재 규칙은 `.elf/managed/`에 있으므로, 프로젝트에서 고친 내용은 `ProjectRule.md`로 옮긴 뒤 삭제), `@AGENTS.md` 줄이 없는 `CLAUDE.md`(Claude Code가 `AGENTS.md` 대신 그것을 읽음), ELF 블록이 없는 `.gitignore`(`elf update --force`가 맨 위에 넣고 사용자 줄은 유지)가 해당합니다. 이름을 주는 방식은 `./<이름>`이 이미 있으면 예전처럼 exit 3으로 거부합니다.

```bash
elf init                              # in-place: 현재 폴더에 ELF 도입
elf init --here MyTool --preset minimal   # in-place, 이름 지정, core 폴더만
elf init . --yes                      # in-place, 확인 없이(스크립트, CI)
elf init NIRS_Probe                   # 하위 폴더: ./NIRS_Probe/ 생성
elf init NIRS_Probe --preset exp --lang en-US
elf init NIRS_Probe --modules hw,sw
elf init my_questions --preset qa                          # Q&A bundle 아카이브 (카테고리 0개)
elf init my_questions --preset qa --categories 일상질문,IT일반질문   # 카테고리 미리 생성
```

> **`--lang en-*`(영어 프로젝트).** 영어 프로젝트는 모든 것을 영어로 받습니다. 규칙 문서(`.elf/managed/EliRule.md`, `LogConvention.md` 등과 루트 `AGENTS.md`)는 같은 경로의 영어판이며, 그 프로젝트에서 AI 에이전트가 따르는 정본입니다. 세션과 trial 틀은 영어 절 제목(`### Conditions`, `Prediction match:`)을 쓰고, `.gitignore` 블록, `.editorconfig`, 사용자 소유인 `README.md`, `ProjectRule.md`도 영어이며, elf가 쓰는 폐기 표기도 영어입니다. 새 영어 프로젝트에는 한국어가 남지 않습니다. elf는 이전의 한국어 우선 절 제목(`### 조건 (Conditions)`)도 같은 절로 읽으므로 이전 로그도 그대로 유효합니다. 기존 영어 프로젝트는 `elf update` 승인 뒤 이 세트로 옮겨집니다. 고치지 않은 규칙 파일은 교체하고, 고친 파일은 새 버전을 `.elf-new`로 옆에 두며, 이전 영어 사본(`*.en.md`)은 고치지 않았으면 지웁니다. 규칙을 바꿀 때는 `ProjectRule.md`에 씁니다. 다른 언어는 한국어 세트를 씁니다.

> **`qa` preset.** 연구용 폴더 구조 대신 *질문 아카이브* 유형을 만듭니다. 구성은 루트 `AGENTS.md`(운영 규칙 원본), `0_Meta/ProjectRule.md`(이 프로젝트만의 규칙과 외부 반출 완화 선언. `elf update`가 바꾸지 않고, `AGENTS.md`와 충돌하면 이쪽이 우선), `.elf/managed/templates/bundle_template.md`, `.elf/`입니다. **카테고리는 기본 0개**이며, `AGENTS.md` 규칙대로 필요할 때 만들거나 `--categories a,b,c`로 미리 만듭니다(각각 `archive/` 포함). Q&A는 session, trial, figure가 아닌 의미 단위 **bundle**로 기록합니다. 더 이어갈 일이 없는 bundle은 `archive/`로 옮기고 recall 대상으로 남으며, 틀렸거나 대체된 bundle은 `status: deprecated`로 표시합니다. 카테고리마다 `<카테고리>/README.md`에 그 카테고리 규칙을, `0_Meta/templates/<카테고리>.md`에 사용자 틀을 둘 수 있고, `_`로 시작하는 폴더는 도구, 작업 영역입니다. `elf check logs`가 bundle을 점검합니다([점검](check.ko.md)). `.elf/` 제어 영역은 함께 쓰고 manifest는 따로(`manifest.qa.json`) 두며, `elf update`로 규칙을 갱신합니다. 연구용 preset과는 별개입니다.

> **`general` preset(2.27에서 삭제).** 같은 용도에는 `minimal`(core 폴더만)을 씁니다. `elf init --preset general`은 알 수 없는 preset으로 거부됩니다. 기존 general 프로젝트에서 `elf update`를 실행하면 `minimal`(연구 계열)로 바뀝니다. ELF 관리 규칙 파일은 연구용으로 바뀌고, 사용자 소유 파일(`0_Meta/ProjectRule.md`, `README.md`, 로그)은 그대로이며, 바뀐 사실을 출력으로 알립니다(`--dry-run`으로 먼저 확인 가능).

## `elf update`

현재 프로젝트의 ELF 관리 파일을 설치된 CLI 버전에 맞춰 갱신합니다. **연구 데이터와 설정은 건드리지 않습니다.** 세션 로그는 2.28 이전에 만든 프로젝트를 새 로그 구조로 이관할 때 한 번만 바꿉니다([아래](#로그-구조-이관-228)).

| 플래그 | 의미 |
|--------|------|
| `--dry-run` | 바꾸지 않고 계획(이관 포함)만 출력 |
| `--out <폴더>` | `--dry-run`과 함께: 이관 결과 파일과 표(헤더, 링크와 코드 span 변경, 경로 언급, 경고)를 이 폴더에 씀. 프로젝트 안은 `.elf/runtime/` 아래만 |
| `--to <줄>` | 목표 줄(예: `3.0`, 생략하면 PATH `elf`의 줄). 다른 줄이면 버전 보관소의 그 버전이 실행 |
| `--approve <코드>` | 줄을 바꾸는 갱신을 요약에 나온 확인 코드로 적용(사용자 승인 뒤) |
| `--force` | 사용자가 고친 관리 파일도 덮어쓰고, hybrid 블록을 교체 |

프로젝트 manifest `.elf/manifest.json`에는 그 프로젝트가 쓰는 ELF 버전(`elf_version`)이 들어 있고, PATH의 `elf`는 프로젝트의 모든 명령을 그 줄로 실행합니다([프로젝트별 버전](reference.ko.md#프로젝트별-버전)). 이 값은 `elf update`만 바꿉니다. 같은 줄 안에서는 `elf update`가 관리 파일을 바로 갱신합니다.

**줄을 바꾸는 갱신은 사용자 승인이 필요합니다.** 높은 줄로 옮기는 갱신(MINOR 이동, 또는 로그 이관이 따르는 구조 세대 변경)은 처음에 아무것도 쓰지 않습니다. `elf update`가 요약(현재와 목표 버전, 이관과 관리 파일 계획, 다른 PC 영향)과 6자리 확인 코드를 출력하고 exit 3으로 끝납니다. AI 에이전트는 요약을 사용자에게 보이고, 사용자가 승인한 뒤에만 `elf update --to <줄> --approve <코드>`를 실행하며, 승인 문장을 세션 로그에 남기고 갱신만 따로 커밋합니다. 그사이 무엇이 바뀌면 코드가 맞지 않아 새 요약이 출력됩니다. 터미널에서 실행하면 [y/N]을 묻습니다. 낮은 줄로 옮기는 것은 거부하고(되돌리기는 git), 이관은 구조 세대 하나씩 합니다. 아직 2.27 형식인 프로젝트에서 `elf update --to 2.27`은 이관하지 않고 elf 2.27.x로 관리 파일만 갱신합니다. elf 2.28.1 이하가 설치된 PC에는 런처가 없으므로, 3.0 이상으로 옮긴 프로젝트를 pull하기 전에 `elf self update`가 필요합니다. 판정은 `elf check version`이 보여 줍니다.

파일 종류별 동작은 [파일 소유권](#파일-소유권)을 참고합니다.

갱신은 **preset에 따라** 합니다. 프로젝트 유형(연구용, `qa`)을 `.elf/config.json`의 `"preset"` 필드(`elf init`이 기록)에서 읽고, 그 유형의 정식 파일 묶음으로 갱신 계획을 세우고 프로젝트 manifest를 다시 씁니다. 2.16.2 이전에 만든 프로젝트에는 이 필드가 없으므로, 프로젝트 manifest(`.elf/manifest.json`)에서 유형을 추정해 처음으로 실제 갱신할 때 기록합니다. `config.json`과 manifest가 서로 맞지 않으면 `"preset"` 필드를 고칠 때까지 `elf update`가 실행을 거부합니다. 잘못된 파일 묶음으로 갱신되는 것을 막기 위해서입니다. 출력 첫 줄에 preset과 계열을 함께 보여 줍니다(예: `preset: minimal (research lineage)`). 연구 계열에서 preset이 없거나 `research`로 적힌 프로젝트는 `full`로 보고, `elf update`가 `full`을 기록합니다.

```bash
elf self update          # ① CLI를 새 버전으로 갱신
elf check version        # ② CLI와 프로젝트 버전, 할 일 확인
elf check files          #   무엇이 바뀌는지 확인
elf update --dry-run     # ③ 쓰지 않고 미리 보기
elf update               # ④ 관리 파일만 교체. `edited:` 줄은 내가 고친 파일을 그대로 두고 새 버전을 .elf-new로 만들었다는 뜻이므로 diff를 보고 병합(또는 --force로 새 버전 채택)
elf check                # 팀이나 CI의 통과 조건. issue가 있으면 exit 4 (로그만 볼 때는 elf check logs)
```

### 로그 구조 이관 (2.28)

2.28부터 로그는 YAML 헤더를 달고 `2_Log/S/`, `2_Log/G/`에 고정됩니다([세션과 trial](sessions.ko.md)). CLI를 갱신해도 이관되지 않습니다. 3.0부터는 PATH의 `elf`가 이관 전 프로젝트를 elf 2.27.x로 실행하므로(처음 쓸 때 받음, 받을 수 없으면 내장 2.27 호환 모드), 그 프로젝트는 `elf update`로 이관하기 전까지 2.27 명령으로 계속 기록합니다. 프로젝트마다 준비됐을 때 이관하면 되고, `elf update`가 한 번 이관합니다. 인용 블록 헤더를 YAML 7키로 바꾸고, 로그를 `2_Log/`와 `2_Log/Archive/`에서 `2_Log/S/`로 옮기고, 링크를 다시 계산하고, 폐기 기록을 `.elf/deprecated/` 보관 파일로 바꾸고, `Registry.md`와 탐색기 중첩 설정을 만들고, `2_Log/Wiki/Session_Registry.tsv`를 지웁니다. 이관은 그 프로젝트의 AI 에이전트가 실행하고 경고를 처리합니다.

| 단계 | 내용 |
|------|------|
| 전제 | git 작업 트리가 깨끗해야 함(되돌릴 때는 git). 아니면 아무것도 쓰지 않음 |
| 미리 보기 | `elf update --dry-run --out <폴더>`가 결과 파일(`files/`), 로그별 7키 값과 goal, conclusion의 원천 표(`headers.tsv`), 다시 계산한 링크와 코드 span(`changes.tsv`), 링크가 아닌 옛 경로와 지운 파일 이름 언급(스크립트 포함, `mentions.tsv`), 경고(`warnings.tsv`)를 씀. 프로젝트에는 쓰지 않으므로 사본에 적용해 보지 않아도 됨 |
| 계획 먼저 | `elf update`가 이관 전체를 계획함. 경고가 하나라도 있으면 결정 파일 `.elf/runtime/migration.yaml`(git 추적 밖)만 쓰고 요약을 출력한 뒤 exit 3. 관리 파일 갱신도 기다림 |
| 결정 | 상태 경고(헤더와 Registry 불일치, 둘 다 없음, 정식 값이 아님)에는 `status` 칸이 있어, 보여 주는 근거로 `ongoing`, `complete`, `deprecated` 중 하나를 정하고 사용자 확인 뒤 채움. 그 밖의 경고(없는 링크 대상, 같은 번호의 로그 두 개)는 고칠 파일을 알려 줌 |
| 승인과 적용 | 결정이 끝나면 `elf update`가 요약과 확인 코드를 출력함. 사용자 승인 뒤 요약에 나온 대로 `elf update --to <줄> --approve <코드>`(예: `--to 3.1`)가 한 번에 적용하고 결정 파일을 지움. 이어서 `elf check`, 사용자 검토, 커밋 |

#### 이관 가이드라인

가이드라인은 방향만 정합니다. 세부 판단은 프로젝트마다 다르므로 그 프로젝트의 AI 에이전트와 사용자가 정합니다. 이관 전에 불규칙한 헤더를 어떻게 정리할지, 7키 밖의 헤더 필드를 어떻게 처리할지, 이력 문장이 당시 경로를 유지해야 하는지, 어떤 프로젝트 스크립트를 고칠지, 이관 작업 자체를 어떻게 기록할지가 여기에 해당합니다.

| 방향 | 내용 |
|------|------|
| 시점 | 저장소마다 준비됐을 때. 서두를 필요 없음. 이관 전까지 그 프로젝트는 elf 2.27.x로 그대로 동작함 |
| 진행 | 그 프로젝트의 AI 에이전트가 사용자와 함께 진행. 무엇이든 쓰기 전에 사용자가 요약을 승인함 |
| 순서 | 진행 중 작업 커밋, `elf update --dry-run --out <폴더>`로 미리 보고 사용자와 검토, 경고 정리, 요약 승인과 적용, `elf check`와 차이 검토(`git diff -M`) |
| 커밋 | 이관만 따로 커밋해 `git revert`로 되돌릴 수 있게 함. 다른 되돌리기 방법은 없음 |
| 이관이 남기는 것 | 링크가 아닌 옛 로그 경로와 지운 파일 언급, 프로젝트 스크립트, 옛 절차를 다시 적은 프로젝트 규칙 문서. 출력과 `mentions.tsv`를 보고 프로젝트에서 정리함. 프로젝트 규칙 문서에는 관리 규칙과 다른 점만 남김 |
| 다른 PC | 이관한 프로젝트를 pull하는 PC에는 elf 3.0 이상이 필요(`elf self update`) |

헤더 값은 7키로 옮깁니다(`Created`나 `Date` → created, `목표`나 `Goal` → goal, `Handoff` → handoff, `결론`이나 Registry key finding → conclusion, 상태 `★ 활성` → ongoing, `Complete` → complete, `Deprecated`나 `Purged` → deprecated). 그 밖의 헤더 필드(`관련` 등)는 제목 아래 본문 한 줄(`> 이관 전 헤더: …`)로 남기므로 값과 링크가 사라지지 않습니다. 링크와, 상대 경로 하나만 담은 코드 span은 실제 파일을 가리킬 때 새 위치 기준으로 다시 계산합니다. 링크 재계산과 언급 검사는 git이 추적하는 파일만 대상으로 합니다(추적 밖 사본, 임시 폴더는 그대로). 바뀐 코드 span은 출력에 파일과 줄로 나오므로, 당시 경로를 적은 이력 문장이면 원래 표기로 되돌립니다. conclusion 앞머리에 상태어(완료, 보관 등)가 남은 로그와, 이관이 고치지 않는 옛 로그 경로나 지운 파일(`Session_Registry.tsv` 등) 언급이 남은 파일(스크립트 먼저)도 출력에 나옵니다. 이관 뒤 구버전 CLI가 `2_Log/`에 만든 로그는 다음 `elf update`가 다음 빈 번호로 `2_Log/S/`에 옮깁니다. `.elf/` 없이 `2_Log/`만 있는 폴더도 같은 방식으로 이관합니다.

#### 2.x 어느 버전에서든

3.0.2부터 `elf update`는 2.0부터 2.28.x까지 어느 버전으로 만든 프로젝트든 같은 순서(요약, 확인 코드, 승인)로 PATH의 `elf`의 줄(3.0 이상)로 올립니다. 버전마다 따로 할 일은 없습니다.

| 시작 버전 | `elf update`가 더 하는 일 |
|---|---|
| 2.0~2.3(generator), 2.4.0 | `.elf/`가 없어도 `0_Meta/`의 ELF 규칙 파일로 ELF 프로젝트임을 알아보고 `.elf/`(설정, 프로젝트 manifest)를 만듭니다. 옛 로그 위치(`5_Exp/53_Analysis/Logs/`, `7_Log/`)와 폴더 이름(`9_Archive`, `1_Wiki`, `2_Wiki`)의 로그와 Registry도 이관합니다 |
| 2.14 이하 | 옛 위치의 규칙(`0_Meta/`)과 틀(루트 `templates/`)을 `.elf/managed/`로 옮깁니다 |
| 모든 버전 | 지금은 쓰지 않는 ELF 파일(`.claudeignore`, 옛 gallery 스크립트, 옛 틀 사본 등)을 지웁니다 |

옮기거나 지우는 ELF 파일은 그 버전이 넣은 그대로일 때만 새 판으로 바꾸거나 지웁니다. 사용자가 고친 파일은 남기고, 옮긴 규칙이면 새 판을 `<파일>.elf-new`로 둡니다(위 Managed와 같은 처리). `.gitignore`는 표시 블록이 없는 옛 판이면 사용자 줄을 그대로 두고 맨 위에 블록을 넣습니다. `0_Meta/ProjectRule.md`, `README.md`, `AI_Sync.md`와 연구 폴더(옛 번호 포함)는 사용자 파일이라 그대로 둡니다. 판정은 `elf check version`이 보여 줍니다.

## 파일 소유권

`elf update`는 파일 소유권을 4가지로 나눠 지킵니다.

| 구분 | 파일 | update 동작 |
|------|------|-------------|
| **Managed** | `.elf/managed/`(`EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `LLMcliche.md`, `templates/*`), 루트 `.editorconfig`, `AGENTS.md` | 새 버전으로 교체. 사용자가 고친 파일은 **그대로 두고** 새 버전을 `<파일>.elf-new`로 만듦(`--force`면 덮어씀) |
| **사용자 소유** | `0_Meta/`(`ProjectRule.md`, 프로젝트별 추가 규칙 `<이름>.project.md`), `README.md`, 모든 연구 데이터, 로그 | **건드리지 않음**(로그는 위 1회 이관만) |
| **생성 파일** | `2_Log/S/Registry.md`, `2_Log/G/Registry.md`, `.vscode/settings.json`의 `explorer.fileNesting` 키(그 밖의 설정과 직접 둔 nesting 패턴은 사용자 소유) | 로그를 바꾸는 명령이 헤더에서 매번 다시 만듦. nesting은 종료된 로그를 한 줄에 하나씩 이어 적고(`"S001_log.md": "S002_log.md"`), 그 키 부분만 바꿔 씀. 세션을 닫을 때 diff는 1~2줄 |
| **Hybrid** | `.gitignore` | 표시 블록(`# >>> ELF managed >>>`부터 `# <<< ELF managed <<<`까지)만 교체하고, 블록 밖의 사용자 규칙은 유지 |
| **배포 안 함** | `CLAUDE.md` | 3.6부터 ELF가 만들지 않습니다. Claude Code 2.1.277 이상은 `CLAUDE.md`가 없으면 `AGENTS.md`를 직접 읽습니다. 기존 `CLAUDE.md`는 바꾸거나 지우지 않습니다. 남겨 두거나(또는 오래된 Claude Code, `AGENTS.md`를 읽지 못하는 세션) 쓸 때는 `@AGENTS.md` 1줄을 넣습니다. Claude Code가 그때는 `AGENTS.md` 대신 `CLAUDE.md`를 읽기 때문입니다. 연결 여부는 `elf check files`가 Claude Code의 import 해석 방식대로 점검합니다(백틱, 코드 블록, HTML 주석, 따옴표 안의 `@AGENTS.md`는 import되지 않으므로 warning) |

프로젝트 규칙은 관리 파일을 고치지 말고 `ProjectRule.md`(사용자 소유)에 씁니다. 그러면 갱신할 때 충돌하지 않습니다. 데이터 파일(`LLMcliche.md`, `highIFjournals.md`)의 항목 추가, 제외, 재정의는 **프로젝트별 추가 규칙** `0_Meta/<이름>.project.md`에 적습니다(사용자 소유. 실제 규칙 = 기본 규칙 ⊕ 추가 규칙. 제외할 때는 사유 필수, EliRule §2.7).

## `.elf/` 디렉토리

`elf`가 관리합니다(직접 수정 금지, 이관 결정 파일만 예외).

- `config.json`: 프로젝트 이름, 언어, preset(프로젝트 유형), 생성일
- `manifest.json`: 프로젝트 manifest. 관리 파일 기록과 프로젝트를 마지막으로 갱신한 ELF 버전(`elf_version`). 2.28 이전에는 버전이 별도 `version` 파일에 있었음
- `managed/`: 설치된 규칙(규칙, 로그 형식 틀)
- `baseline/`: hybrid 파일의 원본 사본(블록 안 편집 감지용)
- `deprecated/`: 폐기한 내용의 압축 원문([내용 폐기](deprecate.ko.md)). git으로 추적
- `runtime/`: 도구 임시 상태(autoread 표시, 이관 결정 파일). git 추적 밖

---

[README](../README.ko.md)로 돌아가기
