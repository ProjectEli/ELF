[English](check.md) | [한국어](check.ko.md)

# 점검

## `elf check [version|setup|files|logs]`

네 범위를 점검합니다. 읽기만 하며, `logs`가 폴더 한 단계 어긋난 링크를 고치는 것만 예외입니다([링크 고치기](#링크-고치기)). 네트워크는 쓰지 않습니다. 범위를 생략하면 version, setup, files, logs 순서로 모두 점검하고(버전이 맞지 않으면 다른 결과가 틀릴 수 있어 version이 먼저) 문제와 고친 것만 출력합니다. 범위마다 요약 1줄, 그 아래에 issue, warning, `fixed` 줄, 마지막에 합계가 나옵니다. 범위를 지정하면 그 범위의 모든 항목을 `ok`, `info`, `warn`, `issue`, `fixed`로 출력합니다.

| 범위 | 점검 내용 |
|---|---|
| `version` | 설치된 `elf`와 그 줄(MAJOR.MINOR), 구조 세대, 설치 기록(`elf self update` 가능 여부), 프로젝트 manifest `.elf/manifest.json`(형식과 `elf_version`), 프로젝트 고정값(`elf_version`, 해석한 줄, 구조 세대. 2.28.x는 3.0 줄로 봄. 이전 형식 프로젝트, `.elf/` 없는 2.0~2.4.0 프로젝트, 2.27 형식 로그만 있는 폴더는 2.27 줄로 봄), 버전 보관소 `~/.elf/versions`(위치, 버전 수, 프로젝트 줄을 이 CLI, 보관된 버전 중 무엇이 실행하는지 또는 아직 없는지), 판정 한 줄. 줄이 같으면 ok, 프로젝트가 낮으면 `elf update`, 프로젝트나 manifest schema가 높으면 `elf self update`(그 전까지 쓰기 명령 거부), 이전 형식이면 `elf update`로 이관. 프로젝트 밖에서는 CLI 항목만 |
| `setup` | git 저장소와 `pre-commit` 훅, autoread 상태와 `autoread_fulltext` 선언, `.elf/config.json`에서 켠 실험 기능, 링크 고치기 켬 여부 |
| `files` | `.elf/` baseline. 관리 파일별 상태 `ok`, `outdated`, `missing`, `edited`(프로젝트의 `preset:` 줄 다음에 표시. `config.json`과 manifest가 어긋나면 경고하고, 진단은 manifest 기준). 루트에 남은 `.claudeignore`(Claude Code에서 효과가 없고 v2.24부터 ELF가 관리하지 않으므로 지워도 됨). 한국어가 아닌 프로젝트의 규칙 세트 언어와 3.4 이전에 남은 영어 사본(`*.en.md`). 프로젝트별 추가 규칙(`0_Meta/<이름>.project.md`)의 제외 사유 누락과 허용되지 않는 대상. 에이전트 진입 파일(Claude Code 2.1.277 이상은 `AGENTS.md`만으로 충분. `CLAUDE.md`나 `CLAUDE.local.md`가 있으면 Claude Code가 그 파일을 대신 읽으므로 `@AGENTS.md` import가 있어야 함), 그 파일의 지나친 내용, 대기 중인 `*.elf-new` 파일. tsa를 켠 경우 토큰이 없는 manifest, 훅 상태, GPG 서명, `openssl` |
| `logs` | 세션 로그. 아래 |

**`logs` 상세**(`2_Log/S/`, `2_Log/G/`, 계열마다 따로 셈. 프로젝트의 다른 마크다운 문서의 링크도 점검):

| 수준 | 내용 |
|---|---|
| issue | 이관한 프로젝트에 남은 이전 형식 로그(한 줄로 알림, `elf update` 실행), 7키가 순서대로 있지 않거나 값이 틀린 헤더(줄 번호 표시), `S###`/`G###`가 아닌 로그 이름, 로그의 깨진 상대 `.md` 링크, `.elf/deprecated/`에 보관 파일이 없는 폐기 표기 줄 |
| warn | 번호 빈칸, 진행 중인데 `goal`이 빔, `6_Exp/64_Viz/S###/`의 그림이 로그에 embed되지 않음(표에 경로만 적은 것은 embed가 아님), 진행 중 로그의 trial 구조(절 이름과 순서, 해석 첫 줄 `예상 일치 정도` 또는 `Prediction match`, 관찰이 있는데 Phase 1 절이 없음. 한국어 우선 `### 조건 (Conditions)`과 영어 `### Conditions`는 같은 절), 틀 문구가 남은 절, `Registry.md`가 없거나 헤더와 다름, 표기 줄이 없는 보관 파일, 로그에서 그 밖의 파일로 가는 깨진 링크, 프로젝트의 다른 마크다운 문서의 깨진 링크(`Archive/` 포함, 이름이 `.`으로 시작하는 폴더는 제외, 문서마다 한 줄). 진행 중 로그에서 프로젝트 밖을 가리키는 링크나 `### 생성 파일` 경로(프로젝트 밖 절대 경로, `%VAR%`, `$VAR`, `~/`, 밖으로 벗어나는 `../`. git 저장소 안이면 경계는 `.git`이 있는 가장 가까운 상위 폴더인 저장소 루트라서, 하위 폴더 프로젝트는 저장소의 다른 파일을 인용할 수 있음), 없는 `### 생성 파일` 경로 |
| fixed | 폴더 한 단계 어긋나서 고친 링크와 `### 생성 파일` 경로([링크 고치기](#링크-고치기)). 파일마다 한 줄과 합계 |
| info | 한 계열에 진행 중 세션이 여럿일 때 그 수(병렬 세션 허용), git이 아직 추적하지 않는 보관 파일(로그 변경과 함께 커밋) |

- 이관하지 않은 프로젝트에서 `logs`는 2.27 검사(Registry와 로그 파일, 번호, 링크, 그림 embed, trial 구조)를 하고 이관 안내를 info로 덧붙입니다. 이전 형식 자체는 issue가 아닙니다.
- 종료 코드: issue가 하나라도 있으면 **4**(pre-commit 훅이나 CI 통과 조건으로 사용), warning만 있으면 **0**.
- 해당하지 않는 범위는 건너뜁니다. `.elf/`가 없으면(ELF 프로젝트 밖, 또는 `2_Log/`만 있는 폴더) `files`는 건너뛰고 `logs`는 점검합니다. 해당하지 않는 범위를 직접 지정하면 exit **1**.
- 일부러 본문에 넣지 않는 그림(SI 등)은 로그에 `<!-- noembed: filename.png -->` 주석을 달아 제외합니다.
- 링크 고치기 말고는 `elf check`는 알려 주기만 하고 고치지 않습니다. 로그는 쓰기 전에 검사하는 `elf` 명령으로 씁니다([세션과 trial](sessions.ko.md)).
- 2.26 이전 이름도 `--help`에는 보이지 않지만 동작합니다: `elf doctor`(setup, files 항목과 프로젝트 manifest 줄), `elf status [--check]`(관리 파일), `elf validate [--check]`(로그).

### 링크 고치기

하위 폴더(예: `Archive/`)로 옮긴 문서에는 원래 폴더 기준으로 쓴 상대 링크가 남습니다. `elf check`와 `elf check logs`는 `../`를 하나 더하거나 하나 빼서 프로젝트(또는 그 git 저장소) 안에서 파일이 찾아지고, 그것이 한쪽뿐이면 그 링크를 고칩니다. 경로만 바뀌고 앵커, 제목, 줄바꿈, 파일의 나머지는 그대로입니다. 진행 중 로그의 `../`로 시작하는 `### 생성 파일` 경로도 같은 방식으로 고칩니다.

- 고치지 않고 알리기만 하는 경우: 양쪽 모두 파일이 찾아짐, 고친 경로가 프로젝트 밖, 파일이 찾아지지 않음(지워졌거나 이름이 바뀐 대상). 다른 곳의 같은 이름 파일은 다른 파일일 수 있어 찾지 않습니다.
- `elf trial write`, `elf session close`, autoread 훅은 고치지 않고 `elf check logs`로 고칠 수 있다고 알립니다.
- 알리기만 하려면 `.elf/config.json`에 `"fix_links": false`를 둡니다. 키가 없으면 고치기가 켜져 있습니다.
- 링크로 보는 것은 `](…)` 안의 상대 경로로, 파일 확장자나 `/`로 끝나고 뒤에는 따옴표 제목만 오는 것입니다. 대괄호 글 뒤의 괄호 설명, URL, 앵커, 자리표시(`{ } < > *`), 코드는 링크로 보지 않습니다.
- `### 생성 파일`에서 `fig_p1~p5.png` 같은 범위 표기와 `supp/` 같은 폴더 이름만 쓴 칸은 파일 이름만 쓴 칸처럼 점검하지 않습니다.

```bash
elf check          # 모든 범위, 문제만
elf check version  # elf update와 elf self update 중 무엇을 실행할지
elf check logs     # 세션 로그 상세(그림을 만든 trial 뒤, 종료 전)
elf check files    # 관리 파일 상세: elf update가 바꿀 것
```

---

[README](../README.ko.md)로 돌아가기
