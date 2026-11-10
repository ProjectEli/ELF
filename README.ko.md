[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF)

**Base-Delta: 바뀐 부분만 기록으로 남기는 연구 자동화**

이제는 연구 결과가 어떻게 나왔는지도 기록합니다.\
ELF에서는 변경 하나하나가 가설, 실행, 교훈이 붙은 trial이 되고, 에이전트가 사람 대신 기록합니다.\
모든 결과에 과정이 남고, 다음 단계는 바뀌는 부분만 관리하면 됩니다.

ELF는 연구 프로젝트용 폴더 구조, AI 에이전트가 따르는 규칙, 명령행 도구 `elf`로 구성됩니다. 내 컴퓨터에 자료 원본을 두고 동작합니다.

| 할 수 있는 일 | 내용 |
|---|---|
| 기록 | 변경할 때마다 trial을 남깁니다. 실행 전에는 가설과 예상을, 실행 후에는 관찰과 교훈을 적습니다 |
| 이어서 작업 | 한 달 뒤에 다시 열어도, 대화가 새로 시작되어도 세션의 handoff와 결론을 읽고 이어 갑니다 |
| 근거 추적 | `S012 t02` 같은 번호 하나로 로그, 스크립트, 데이터, figure를 함께 찾습니다 |
| 점검 | `elf check`가 빠진 figure, 깨진 링크, Registry 불일치를 찾아 알려 줍니다 |

## session과 trial

| 단위 | 무엇 | 담는 것 |
|---|---|---|
| **trial** `t##` | 변경 1건 = 실험 1건 | phase 1(실행 전): 목표, 조건, 가설, 예상 / phase 2(실행 후): 관찰, 해석, 교훈, 생성 파일 |
| **session** `S###` | 목표 1개 아래 trial이 쌓이는 작업 단위 | 헤더(상태, 목표, handoff, 결론)와 t01, t02, …. 닫을 때 결론이 헤더와 `Registry.md`에 남습니다 |

```text
S012  목표: 파장별 SNR 비교
 ├─ t01  기준선: 3파장 sweep               가설 → 실행 → 810 nm 최적
 ├─ t02  delta: 광자 수 1e8 → 1e9           가설 → 실행 → 결론 유지
 └─ t03  delta: 피부 모델 3층 → 5층         가설 → 실행 → 735 nm 역전
 close → Registry: "810 nm 최적, 5층 모델에서는 735 nm 재검토"
```

## 빠른 시작

Claude Code, Codex 같은 AI 에이전트와 함께 씁니다. 에이전트 없이 쓰는 방법은 이 절의 마지막에 있습니다.

1. CLI를 설치합니다.

   Windows에서는 PowerShell에서 실행합니다.

   ```powershell
   powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
   ```

   Linux와 macOS에서는 터미널에서 실행합니다.

   ```bash
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
   ```

   실행 파일 하나가 `~/.elf/bin`에 설치되며 Node나 Python은 필요 없습니다. 터미널을 새로 열고 `elf --version`으로 설치를 확인합니다.

2. 프로젝트를 만듭니다.

   ```bash
   elf init MyProject --preset exp   # 6_Exp + 7_Paper. 이름 없이 실행하면 현재 폴더에 생성
   cd MyProject
   ```

   `[elf] created MyProject (ELF v3.0.1, preset: exp, lang: ko-KR)`가 출력되고, 폴더 구조와 함께 `AGENTS.md`(에이전트가 처음 읽는 규칙 요약)와 `2_Log/S/`(세션 로그를 두는 곳)가 만들어집니다. 에이전트는 `AGENTS.md`를 읽고 시작하므로 따로 설정할 것이 없습니다. preset, 언어, 만들어지는 파일은 [프로젝트 생성과 갱신](docs/projects.ko.md)에 있습니다.

3. 에이전트에서 프로젝트 폴더를 열고 세션을 시작하라고 말합니다.

   > 새 세션 생성. 목표는 파장별 SNR 비교.

   에이전트가 목표와 함께 `elf session new`를 실행해 `2_Log/S/S001_log.md`를 만들고, 세션 목록 `2_Log/S/Registry.md`를 갱신합니다.

4. 첫 trial을 요청합니다.

   > t01: 3파장 sweep. 3층 피부 모델, 광자 1e8.

   에이전트는 `elf trial new`로 phase 1(목표, 조건, 가설, 예상)을 적고 실행할지 묻습니다. 수정을 요청하면 `elf trial write`로 phase 1을 고칩니다. 승인하면 실행한 뒤 `elf trial write`로 phase 2(관찰, 해석, 교훈, 생성 파일)를 적습니다.

5. 다음 변경을 요청합니다.

   > 광자 수 1e9로 올려서 다시.

   에이전트가 `elf trial new`로 t02를 추가합니다. t02에는 t01을 기준으로 무엇을 왜 바꿨는지가 적히고, t01과 같이 phase 1, phase 2 순서로 진행됩니다.

6. 세션을 마감합니다.

   > 세션 마감.

   에이전트가 `elf check logs`로 빠진 figure나 형식 오류를 확인하고 결론을 제안합니다. 승인하면 `elf session close --from <파일>`로 마감합니다. 로그는 `2_Log/S/`에 그대로 있고, 상태가 complete로 바뀌며 `Registry.md`에 결론이 나옵니다.

에이전트 없이 쓸 때도 같은 명령을 씁니다. 절이나 결론을 파일에 적고 `--from`으로 넘깁니다.

## trial 예시

위 5단계에서 에이전트가 남기는 t02는 다음과 같습니다.

```markdown
## t02: 광자 수 1e9

### 목표 (Goal)
- t01의 광자 수를 1e8 → 1e9로 올려 810 nm 우위가 표본 잡음이 아닌지 확인

### 조건 (Conditions)
- base = t01, delta = 광자 수 1e8 → 1e9 (이유: t01에서 810, 940 nm 신뢰구간이 겹침)

### 가설 (Hypothesis)
- 잡음 감소로 신뢰구간이 좁아지고 순위는 유지

### 예상 (Prediction)
- 810 nm 우위 유지, 신뢰구간 폭 1/3 이하
- 산출물: `S001_t02_SNR_ci.png`

### 관찰 (Observation)
- 810 nm 우위 유지, 신뢰구간 폭 0.28배
- ![S001_t02: 파장별 SNR, 1e9 광자](../../6_Exp/64_Viz/S001/S001_t02_SNR_ci.png)

### 해석 (Interpretation)
- 예상 일치 정도: 일치
- 1e8에서의 겹침은 표본 잡음

### 교훈 (Lessons)
- 순위 판정에는 1e9 이상 필요

### 생성 파일 (Files)

| 유형 | 파일 |
|------|------|
| Script | `6_Exp/61_Sim/Scripts/S001_t02_sweep.m` |
| Output | `6_Exp/61_Sim/Data/S001/S001_t02_results.mat` |
| Figure | `6_Exp/64_Viz/S001/S001_t02_SNR_ci.png` |
```

## 작동 원리

### 핵심 원칙

| 원칙 | 설명 |
|---|---|
| **Base-Delta** | 완료 내용은 한 번만 적고, 이후에는 바뀐 것만 기록합니다. 변경 하나가 trial, 목표 하나가 session이며, trial이 모여 session이 되고 session이 모여 프로젝트 기록이 됩니다. |
| **가설이 먼저** | 한 trial에서 가설과 예상을 적는 phase 1, 이를 실행하는 phase 2로 나눕니다. phase 1에서 수정 후 승인하면 phase 2로 진행합니다. |
| **사람은 판단, 에이전트는 기록, 도구는 검증** | 같은 형식으로 쓰기 때문에 서로의 작업을 이어받고, `elf check`가 빠진 것을 찾아냅니다. |
| **전부 로컬에** | 내 컴퓨터에 자료 원본을 두고 동작합니다. git으로 관리하고, ELF 없이도 읽을 수 있습니다. 보안을 위해 에이전트의 공개 링크 게시는 원칙적으로 금지됩니다. |

- 모든 기록에는 `S012 t02` 같은 번호가 붙습니다. 로그, 스크립트, 데이터, figure가 같은 번호를 쓰기 때문에 논문, 계획, 대화 어디서든 번호 하나로 인용하고 찾아갈 수 있습니다.
- 다음 trial은 사람이 정합니다. 에이전트는 직전 결과를 보고 다음 변경과 가설, 예상을 적어 두고, 사람이 검토해 실행 여부를 결정합니다.

### 사용 예시

| 상황 | 사용자 입력 | 동작 |
|---|---|---|
| 세션 시작 | "새 세션 생성. 목표는 파장별 SNR 비교." | `elf session new`로 로그와 Registry 행 생성. t01의 phase 1을 적고, 사용자가 승인하면 phase 2 실행 |
| 조건 변경 | "광자 수 1e9로 올려서 다시." | t02에 t01 기준의 변경과 이유, 가설 기록. 실행 후 figure를 관찰 절에 넣고 예상과 비교 |
| 작업 재개 | (한 달 뒤, 또는 대화를 새로 시작한 뒤) "SNR 비교 어디까지 했지?" | handoff와 `Registry.md`를 읽고 이어 감 |
| 근거 추적 | "Fig 3의 810 nm 결론, 근거가 뭐지?" | S012 t01~t03의 기록과 스크립트, 데이터 경로를 보여 줌 |
| 논문 작성 | "지금까지 결론으로 원고 개요 작성." | Registry의 결론을 모아 `7_Paper/72_Drafts`에 작성 |

## 이미 아이디어나 진행된 내용이 있는 경우

빠른 시작은 빈 프로젝트에서 바로 세션을 엽니다. 아이디어나 이미 진행된 내용이 있으면 먼저 프로젝트에 넣고 시작합니다.

| 경우 | 사용자 입력 | 동작 |
|---|---|---|
| 아이디어가 있음 | "웨어러블 광학 센서의 파장 선택에 대한 생각을 정리하고, 검증 계획을 plan 문서로 작성." | 아이디어는 `1_Concept/13_Ideas/`, 계획은 `1_Concept/12_Planning/P001_*.md`에 작성. 사용자가 계획을 승인하면 헤더에 `관련: P001`을 적은 세션을 엶 |
| 진행된 내용이 있음 | (측정 raw CSV를 `6_Exp/62_Empirical/Raw/`에 복사한 뒤) "이 데이터로 파장별 SNR 비교 세션 시작. t01은 데이터 정제." | 원본은 읽기 전용으로 두고, t01에서 정제한 데이터를 `62_Empirical/Processed/`에 저장한 뒤 t02부터 분석 |

단계별 예는 [시작 예시](docs/examples.ko.md)에 있습니다.

## 용어

| 용어 | 뜻 |
|---|---|
| `S###`, `G###`, `t##` | 연구 세션, 거버넌스 세션, trial 번호. 로그, 스크립트, 데이터, figure가 같은 번호를 씁니다 |
| delta | 직전 trial에서 바꾼 것과 바꾼 이유 |
| handoff | 세션 헤더에 적는 현재 상태 한 줄. "유효한 결론; 남은 일; 참조" 형식이며 고칠 때는 줄 전체를 새로 씁니다 |
| status | 로그 헤더의 `ongoing`, `complete`, `deprecated`. 로그는 옮기지 않으므로 기록이 유효한지는 status로 봅니다 |
| Registry | `2_Log/S/`, `2_Log/G/`의 `Registry.md`. 로그 헤더에서 만드는 생성 파일이며 세션마다 한 줄(상태, 결론 또는 handoff) |
| Deprecated | 사용자가 폐기하기로 정한 내용. 문서에서 지우고 원문은 `.elf/deprecated/`에 압축 보관하며 표기 줄이 남습니다. 현재 판단의 근거로 쓰지 않습니다 |

## 구성 요소

규칙 문서는 프로젝트의 `.elf/managed/`에 있습니다.

| 구성 요소 | 역할 | 자세히 |
|---|---|---|
| `AGENTS.md` | 에이전트가 처음 읽는 규칙 요약. 프로젝트 고유 규칙은 `0_Meta/ProjectRule.md`에 적습니다 | [프로젝트 생성과 갱신](docs/projects.ko.md) |
| `elf check` | 설치와 설정, 관리 파일, 로그를 점검합니다. `elf session close`도 마감 전에 로그를 점검합니다 | [점검](docs/check.ko.md) |
| `elf autoread` | 대화가 압축되거나 새로 시작된 뒤 에이전트가 규칙 요약과 handoff를 다시 읽게 합니다. Claude Code에서는 훅이 자동으로 실행합니다 | [규칙 다시 읽기](docs/autoread.ko.md) |
| `elf deprecate` | 사용자가 폐기하기로 정한 내용을 지우고 원문을 압축 보관합니다 | [폐기](docs/deprecate.ko.md) |
| 실행 전 승인 | 에이전트는 자동 실행 모드에서도 phase 1이 끝나면 사용자 승인을 받습니다 | `LogConvention.md` §5.1 |
| 작성자 한 명 | 세션 로그 하나는 한 작성자(에이전트 또는 사용자)만 씁니다. 여러 에이전트가 함께 일하면 에이전트마다 세션을 따로 엽니다 | `EliRule.md` |
| 프로젝트별 추가 규칙 | 어휘와 검색 도메인은 `0_Meta/<이름>.project.md`로 바꿉니다. `elf update`는 이 파일을 건드리지 않습니다 | [프로젝트 생성과 갱신](docs/projects.ko.md) |

## 참고 문서

| 주제 | 문서 |
|---|---|
| CLI 설치와 갱신 | [docs/install.ko.md](docs/install.ko.md) |
| 프로젝트 생성과 갱신(preset, 언어, 파일 소유권, `.elf/`, 로그 구조 이관) | [docs/projects.ko.md](docs/projects.ko.md) |
| 아이디어나 진행된 내용에서 시작하는 단계별 예 | [docs/examples.ko.md](docs/examples.ko.md) |
| 세션과 trial(`session new`, `session close`, `trial new`, `trial write`, `elf log`, `gallery`) | [docs/sessions.ko.md](docs/sessions.ko.md) |
| 점검(`elf check` 범위, issue와 warning) | [docs/check.ko.md](docs/check.ko.md) |
| 폐기(`elf deprecate`) | [docs/deprecate.ko.md](docs/deprecate.ko.md) |
| 대화가 다시 구성된 뒤 규칙 다시 읽기(`elf autoread`, Claude Code 훅) | [docs/autoread.ko.md](docs/autoread.ko.md) |
| 연구 기록 시점 인증(선택 기능, `elf tsa`) | [docs/tsa.ko.md](docs/tsa.ko.md) |
| 전체 폴더 구조 | [docs/layout.ko.md](docs/layout.ko.md) |
| 명령 요약, exit code | [docs/reference.ko.md](docs/reference.ko.md) |
| 로그 형식 규칙 | 프로젝트의 `.elf/managed/LogConvention.md` |
| 버전별 변경 | [CHANGELOG.md](CHANGELOG.md) |

## 라이선스 (License)

이 프로젝트는 '구동 코드'와 '데이터 구조 규격(Protocol)'의 성격이 다르므로, 이중 라이선스(Dual License) 정책을 적용함.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **적용 대상:** `5_SW/`, `6_Exp/*/Scripts/` 폴더 내의 모든 소스 코드(`.m`, `.py` 등).
  * **조건:** 템플릿의 코어 스크립트를 수정 및 개선하여 배포할 경우 해당 수정본은 오픈소스로 공개해야 함. 단, 사용자가 프로젝트 내에 추가한 고유 알고리즘이나 원시 데이터는 비공개(상업화) 유지가 가능함.

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **적용 대상:** `README.md`, 거버넌스 문서(`.elf/managed/`, `0_Meta/`), Session-Trial 폴더 계층 구조, Base-Delta 메타데이터 로깅 규칙 등 연구 방법론 전반.
  * **조건:** 누구나 이 구조와 기록 방법론을 자유롭게 차용 및 변형할 수 있으나, 파생된 템플릿이나 관련 연구 결과물 발표 시 원작자 Eli (projectschnee@gmail.com) 와 본 저장소의 출처를 명시해야 함.
