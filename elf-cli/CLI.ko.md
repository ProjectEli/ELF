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
| `elf init <이름>` | 새 ELF 프로젝트 생성 |
| `elf update` | 현재 프로젝트의 managed 파일 갱신 |
| `elf status` | managed 파일 drift 진단(읽기전용) |
| `elf validate` | 세션/Registry/로그 정합 검사(읽기전용) |
| `elf session new <제목>` | 다음 세션 로그 생성 + 등록 |
| `elf session close [S###]` | 활성 세션 종료 → Archive 이동 + Registry 갱신 |
| `elf session fix-headers` | 세션 로그 헤더 렌더링 보정 |
| `elf gallery` | `6_Exp/64_Viz/`에서 Figure 색인 `_gallery.md` 생성 |
| `elf doctor` | 환경+프로젝트 종합 건강검진(읽기전용) |
| `elf self-update` | `elf` 바이너리 자체 갱신 |

전역 플래그: `elf --version`, `elf --help`, `elf <명령> --help`.

## 명령

### `elf init <이름>`

`./<이름>`에 새 ELF 프로젝트를 생성합니다.

| 플래그 | 기본값 | 의미 |
|--------|--------|------|
| `--preset <p>` | `full` | 모듈 세트: `full` / `experimental` / `software` / `minimal` |
| `--modules <목록>` | — | custom 모듈(쉼표): `hw,fab,sw,exp,paper`. `--preset`보다 우선 |
| `--lang <언어>` | `한국어` | AI 에이전트 응답 언어(`.elf/config.json`에 기록) |

Core 폴더(`0_Meta`–`2_Log` + `templates`)는 항상 생성, 모듈 폴더(`3_HW`–`7_Paper`)는 preset/`--modules`에 따라 추가. `<이름>`이 이미 있으면 exit 3로 거부.

```bash
elf init NIRS_Probe
elf init NIRS_Probe --preset experimental --lang English
elf init NIRS_Probe --modules hw,sw
```

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

### `elf status [--check]`

관리 파일 상태 진단(읽기전용). 각 파일을 `ok` / `outdated` / `missing` / `edited`로 보고하고, 버전 불일치·obsolete 항목도 표시.

- `--check`: 발견 시 exit **4** — pre-commit 훅·CI 게이트로 사용.

### `elf validate [--check]`

세션 장부 정합을 검사(읽기전용): Registry ↔ 로그 파일(미등록 로그 / 유령 행), 세션 번호(중복 / gap), 활성 세션 복수, 로그 내 깨진 상대 `.md` cross-ref.

- **issue**(Registry/로그 불일치·번호 중복·깨진 링크) vs **warning**(번호 gap·활성 복수) — issue만 게이트.
- `--check`: issue 발견 시 exit **4** — pre-commit/CI 게이트.
- Registry 자체가 파싱 불가면 exit **5**(escalation) — "문제 발견"과 "검사 불능"을 구분.

```bash
elf validate          # 보고
elf validate --check   # issue 시 exit 4 (CI 게이트)
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

다음 세션 후보 섹션이 미작성이면 exit **3**으로 거부(작성하거나 `--force`). Registry 파싱 불가 시 exit **5**(escalation).

### `elf session fix-headers [--dry-run]`

기존 세션 로그의 인용구 헤더에 CommonMark hard break(`\`)를 부여해, strict Markdown 렌더러(예: Discord 미리보기)에서 메타데이터가 줄 단위로 표시되게 합니다. 멱등·CRLF-safe. `2_Log/` + `2_Log/Archive/` 대상.

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
- **프로젝트**(프로젝트 내일 때) — `.elf/` stamp 파싱·version이 CLI와 일치·baseline 존재
- **managed 파일** — `elf status` 요약(pending / conflict)
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

`elf update`는 3가지 소유권 tier를 지킵니다:

| Tier | 파일 | update 동작 |
|------|------|-------------|
| **Managed** | `EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `templates/*`, `.claudeignore`, `.editorconfig` | 새 버전으로 교체. 편집한 경우 **보존**하고 새 버전을 `<파일>.elf-new`로 생성(`--force`로 덮어쓰기) |
| **사용자 소유** | `ProjectRule.md`, `AI_Sync.md`, `Session_Registry.tsv`, `README.md`, 모든 연구 데이터·로그 | **절대 미접근** |
| **Hybrid** | `.gitignore` | 마커 블록(`# >>> ELF managed >>>` … `# <<< ELF managed <<<`)만 교체, 블록 밖 사용자 규칙은 보존 |

프로젝트 규칙은 managed 파일을 고치는 대신 `ProjectRule.md`(사용자 소유)에 작성하세요 — 그러면 update 충돌이 없습니다.

## `.elf/` 디렉토리

`elf init`/`update`가 관리합니다(직접 수정 금지):

- `config.json` — 프로젝트 이름·언어·생성일
- `version` — 프로젝트를 마지막으로 건드린 ELF 버전
- `manifest.json` — `update`/`status`가 쓰는 관리 파일 기록
- `baseline/` — hybrid 파일의 원본 사본(블록 내 편집 감지용)
