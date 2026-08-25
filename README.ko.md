[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF)

**Base-Delta: 모든 변경은 실험이고, 기록은 에이전트가 남긴다.**

이제는 연구결과가 어떻게 나왔는지도 기록합니다.\
ELF에서는 변경 하나하나가 가설·실행·교훈이 붙은 trial이 되고, 에이전트가 그 자리에서 씁니다.\
모든 결과에 과정이 남고, 다음 단계는 마지막 교훈에서 시작합니다.

## Session과 trial

| 단위 | 무엇 | 담는 것 |
|---|---|---|
| **trial** `t##` | 변경 1건 = 실험 1건 | 실행 전: 목표·조건·가설·예상 / 실행 후: 관찰·해석·교훈·생성 파일 |
| **session** `S###` | 목표 1개 아래 trial이 쌓이는 작업 단위 | 헤더(목표·관련·Handoff) + t01, t02, … → 닫을 때 결론 1줄이 Registry에 남습니다 |

```text
S012  목표: 파장별 SNR 비교
 ├─ t01  기준선: 3파장 sweep               가설 → 실행 → 810 nm 최적
 ├─ t02  delta: 광자 수 1e8 → 1e9           가설 → 실행 → 결론 유지
 └─ t03  delta: 피부 모델 3층 → 5층         가설 → 실행 → 735 nm 역전
 close → Registry: "810 nm 최적 — 5층 모델에서는 735 nm 재검토"
```

- 기록은 `S012 t02` 같은 번호로 정형화됩니다. 로그·스크립트·데이터·figure가 같은 번호를 쓰기 때문에, 나중에 논문·계획·대화 어디서든 번호 하나로 인용하고 찾아갑니다.
- 다음 trial을 정하는 것은 사람입니다. 에이전트는 직전 결과로부터 다음 변경과 가설·예상을 적어 두고, 사람이 검토해 실행을 결정합니다.

| 용어 | 뜻 |
|---|---|
| `S###` · `t##` | 세션·trial 번호. 로그·스크립트·데이터·figure가 같은 번호를 씁니다 |
| delta | 직전 trial에서 바꾼 것 + 바꾼 이유 |
| Handoff | 세션 헤더의 현재 상태 1줄 — "유효 결론; 미완료; 참조"를 덮어쓰기로 갱신 |
| Registry | `2_Log/Wiki/Session_Registry.tsv` — 세션마다 1행(상태·결론 1줄) |
| Archive | 마감된 로그가 가는 곳(`2_Log/Archive/`). 에이전트의 자율 탐색에서 제외 |
| validate | `elf validate` — 로그·Registry·번호·cross-ref·figure embed·절 구조의 정합 검사 |

## 핵심 원칙

| 원칙 | 한 줄 |
|---|---|
| **Base-Delta** | 기준선은 한 번, 이후엔 바뀐 것만 기록합니다. 변경 하나가 trial, 목표 하나가 session — trial이 session으로, session이 프로젝트의 기록으로 쌓입니다. |
| **가설이 먼저** | 실행 전에 가설·예상을 적고 멈춥니다. 그래서 변경 하나가 실험 하나가 됩니다. |
| **사람은 판단, 에이전트는 기록, 도구는 검증** | 같은 형식으로 쓰기 때문에 서로의 작업을 이어받고, `elf validate`가 빠진 것을 잡습니다. |
| **전부 로컬에** | 마크다운과 폴더뿐입니다. 서버도 계정도 없이 git으로 관리하고, ELF 없이도 읽을 수 있습니다. |

## 실사용 장면

| 장면 | 말하는 것 | 기록되는 것 |
|---|---|---|
| 시작 | "새 세션 생성. 목표는 파장별 SNR 비교." | `elf session new` → 헤더·Registry 행. t01 가설·예상 작성 후 **멈춤** — 사람이 확인하고 실행 |
| 변경 | "광자 수 1e9로 올려서 다시." | t02: base = t01, delta + 이유 → 가설 → 실행 → figure가 관찰 절에 → 적중 판정 |
| 재개 | (한 달 뒤, 또는 컨텍스트 초기화 뒤) "SNR 비교 어디까지 했지?" | Handoff와 Registry 결론에서 이어감 — 기억이 아니라 기록에서 |
| 추적 | "Fig 3의 810 nm 결론, 근거가 뭐지?" | S012 t01→t03 연쇄 + 스크립트·데이터 경로 그대로 제시 |
| 논문 | "지금까지 결론으로 원고 outline." | Registry 결론 → `7_Paper/72_Drafts` — 누적 → 논문 |

## 따라 하기

AI 코딩 에이전트와 함께하는 첫 세션입니다. `AGENTS.md`를 읽는 에이전트면 어느 것이든 됩니다. 앞 절의 S012는 세션이 쌓인 뒤의 모습이고, 여기서는 S001부터 시작합니다. 시작점은 둘입니다 — 데이터가 아직 없으면 **A**, 이미 있으면 **B**. 둘은 같은 연구의 두 시점으로, A는 파장별 SNR을 시뮬레이션으로, B는 시제품 광학 센서로 측정한 데이터로 봅니다. 에이전트 없이 쓰는 방법은 마지막 줄에 있습니다.

**1. 설치 · 프로젝트 생성**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

단일 바이너리가 `~/.elf/bin`에 설치됩니다(Node·Python 불필요). 새 셸에서 `elf --version`으로 확인합니다.

```bash
elf init MyProject --preset experimental   # 6_Exp + 7_Paper. 이름 없이 실행하면 현재 폴더에 생성
cd MyProject
```

`[elf] created MyProject (ELF v2.20.0, preset: experimental, lang: ko-KR)` — 폴더 구조와 함께 `AGENTS.md`(에이전트 공통 진입 규칙 요약)·`2_Log/S001_log.md`(첫 세션 stub)가 생깁니다. 에이전트는 `AGENTS.md`를 읽고 시작하므로 별도 설정이 없습니다. Claude Code용 파일(`CLAUDE.md` 포인터·훅 설정)도 함께 생성되며, 다른 에이전트에서는 무시해도 됩니다. 프로젝트 고유 규칙은 `0_Meta/ProjectRule.md`에 적습니다. 영어 응답·영어 규칙 문서가 필요하면 `--lang en-US`(규칙 정본은 한국어, 영어본은 읽기용 companion).

**2-A. 처음부터 — 아이디어 스케치부터 시작**

| 단계 | 말하는 것 | 기록되는 것 | 사람이 하는 일 |
|---|---|---|---|
| 아이디어 | "웨어러블 광학 센서의 파장 선택에 대해 생각 정리가 필요. 현재 생각은 피부 투과와 헤모글로빈 흡수를 같이 보면 810 nm 근처가 유리하다는 것. 근거와 반례를 함께 정리." | `1_Concept/13_Ideas/wavelength_choice.md`(로그에는 적지 않음) | 방향 판단 |
| 계획 | "이 아이디어를 검증할 구체적인 계획을 plan 문서로 작성. 시뮬레이션으로 파장별 SNR을 비교한 뒤, 시제품 센서 측정으로 확인하는 순서." | `1_Concept/12_Planning/P001_wavelength_optimization.md` | 단계 승인 |
| 세션 | "P001의 1단계부터 시작. S001 목표는 파장별 SNR 비교 — 735/810/940 nm, Monte Carlo 시뮬레이션." | S001 헤더 `관련: P001` + Registry 행 | 목표 확정 |
| t01 | "t01: 3파장 sweep. 기준 조건은 P001에 적은 대로 — 3층 피부 모델, 광자 1e8." | t01 실행 전 절 → **정지** → 실행 → 관찰·해석·교훈 · `61_Sim/Scripts/S001_t01_*.m` · `61_Sim/Data/S001/` | 가설·예상 검토 → 실행 지시 |

근거 논문은 `1_Concept/11_Literature/`에 둡니다 — 출처 신뢰 규칙과 저널 도메인 목록이 규칙에 들어 있습니다.

**2-B. 데이터가 있을 때 — 분석부터 시작**

| 단계 | 명령 / 말하는 것 | 기록되는 것 | 사람이 하는 일 |
|---|---|---|---|
| 데이터 투입 | 측정 raw CSV를 `6_Exp/62_Empirical/Raw/`에 복사 | 기록 없음 — 원본 보존(읽기 전용·git 제외), 기록은 t01부터 | 원본 확인 |
| 세션 | "시제품 광학 센서로 3파장 반사광을 측정한 CSV가 있음. S001 목표는 이 데이터의 파장별 SNR 비교." | S001 헤더 + Registry 행 | 목표 확정 |
| t01 정제 | "t01: 데이터 개요와 정제. 포화 구간과 움직임 잡음 구간은 제외." | `62_Empirical/Processed/S001/` · `63_Analysis/Scripts/S001_t01_*.m`(조건에 범위·필터) | 정제 기준 승인 |
| t02 분석 | "t02: 정제 데이터로 파장별 SNR 비교. 시뮬레이션과 같은 정의로." | t02 실행 전 절 → **정지** → figure `64_Viz/S001/` + 데이터 배열 `.mat`/`.csv` → 해석·교훈 | 가설·예상 검토 → 실행 지시 |

**3. 공통 — 다음 변경 → 마감**

| 단계 | 명령 / 말하는 것 | 기록되는 것 | 사람이 하는 일 |
|---|---|---|---|
| 다음 변경 | "광자 수 1e9로 올려서 다시."(A) → `elf trial new "광자 수 1e9"` | `[elf] appended t02 to 2_Log/S001_log.md (S001)` — t02: base = t01, delta + 이유 → 정지 → 실행 → 기록. 직전 결과를 보고 반복합니다 | 실행 결정 |
| 마감 | `elf validate` → `elf session close` | `[elf] ok: registry, logs, numbering, and cross-refs are consistent` → `[elf] closed S001 → 2_Log/Archive/S001_log.md (Status: Complete, registry updated)` | Registry 결론 1줄 확인 |

`elf validate`는 figure 누락·비정본 절·Registry 불일치를 warning/issue로 보고하고, `elf session close`는 validate를 한 번 더 돌린 뒤 Archive로 옮기고 cross-ref 상대경로를 보정합니다. Handoff에 미완료가 남아 있으면 경고합니다.

에이전트가 남기는 trial은 이런 모습입니다(A의 t02):

```markdown
## t02: 광자 수 1e9

### 목표 (Goal)
- t01의 광자 수를 1e8 → 1e9로 올려 810 nm 우위가 표본 잡음이 아닌지 확인

### 조건 (Conditions)
- base = t01, delta = 광자 수 1e8 → 1e9 (이유: t01에서 810·940 nm 신뢰구간이 겹침)

### 가설 (Hypothesis)
- 잡음 감소로 신뢰구간이 좁아지고 순위는 유지

### 예상 (Prediction)
- 810 nm 우위 유지, 신뢰구간 폭 1/3 이하
- 산출물: `S001_t02_SNR_ci.png`

### 관찰 (Observation)
- 810 nm 우위 유지, 신뢰구간 폭 0.28배
- ![S001_t02: 파장별 SNR, 1e9 광자](../6_Exp/64_Viz/S001/S001_t02_SNR_ci.png)

### 해석 (Interpretation)
- 가설 적중 여부: 적중
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

에이전트 없이: `.elf/managed/templates/trialTemplate.md`를 로그에 붙여 직접 쓰고, `elf trial new`·`elf validate`·`elf session close`는 그대로 씁니다.

## 작동 원리

| 장치 | 역할 |
|---|---|
| `AGENTS.md` | 에이전트 진입 규칙 요약 — 같은 turn에 기록, 정본 우선(선례 모방 금지), 컨텍스트 재구성 후 Handoff 재독. 정본은 `.elf/managed/`(EliRule·LogConvention·AI_PARA_Framework), 프로젝트 규칙은 `0_Meta/ProjectRule.md` |
| 실행 전 정지 | 가설·예상은 실행 전에 확정합니다(LogConvention §5.1). 에이전트는 자동 실행 모드에서도 이 지점에서 멈춥니다 |
| `elf validate` | Registry↔로그·번호·cross-ref·figure embed 누락·trial 절 구조 검사. `session close`가 자동 실행 |
| `elf autoread` | `elf autoread`(인수 없이) = 규칙 요약·활성 세션 Handoff·validate 결과를 digest로 출력 — 컨텍스트가 재구성된 뒤 어느 에이전트에게든 다시 읽힙니다. Claude Code에서는 훅(`.claude/settings.json`)이 compaction·재시작 뒤 첫 프롬프트에 자동 주입합니다(기본 on, `autoread_fulltext`로 규칙 전문 포함 가능) |
| Handoff · Registry | Handoff = "유효 결론; 미완료; 참조" 1줄을 덮어쓰기로 유지, Registry key finding = 세션 결론 1줄 — 재개할 때 읽는 곳 |
| one writer | 세션 로그 1개의 작성자는 하나(에이전트 또는 사람). 병렬 작업은 에이전트마다 세션을 열고, 관계는 헤더 `관련:`에 적습니다 |
| Archive 방화벽 | `Archive/`는 에이전트의 자율 탐색 대상에서 제외됩니다 — `AGENTS.md` 규칙(경로를 지목할 때만 열람). Claude Code에서는 `.claudeignore`가 검색에서도 차단합니다(AI_PARA_Framework) |
| overlay | 어휘·검색 도메인 커스터마이즈는 `0_Meta/<이름>.project.md`(유효 규칙 = base ⊕ overlay). `elf update`가 건드리지 않습니다 |

## CLI

> 전체 레퍼런스(플래그·exit code·파일 소유권): **[elf-cli/CLI.ko.md](elf-cli/CLI.ko.md)**

| 명령 | 역할 |
|---|---|
| `elf init [name] [--preset …] [--modules …] [--lang …]` | 프로젝트 생성. 이름 없으면 현재 폴더에 in-place. preset `full`/`experimental`/`software`/`minimal`(실험적: `general`·`qa`) |
| `elf session new <title>` / `close [S###]` / `fix-headers` | 세션 생성·마감(validate → Archive 이동 → Registry·cross-ref 보정)·헤더 줄바꿈 수리 |
| `elf trial new [title]` | 활성 로그에 정본 trial stub 추가(`t##` 자동 번호) |
| `elf validate [--check] [--strict]` | 정합 검사(읽기 전용). `--check` = issue 시 exit 4, `--strict` = embed 누락·절 구조 경고도 issue로 |
| `elf gallery` | `6_Exp/64_Viz/`의 figure 색인 `_gallery.md` 생성 |
| `elf autoread [enable\|disable\|status]` | 컨텍스트 재구성 후 규칙 재주입. 인수 없이 실행하면 digest 출력(모든 에이전트 공통); 훅 자동 주입은 Claude Code(기본 on) |
| `elf update [--dry-run] [--force]` | 관리 파일을 설치된 CLI 버전으로 갱신 — 연구 데이터·로그·설정은 건드리지 않음 |
| `elf status [--check]` | 관리 파일 상태 진단(읽기 전용). `--check` = 발견 시 exit 4 |
| `elf doctor` | 환경·프로젝트 종합 점검(읽기 전용) |
| `elf tsa <sub>` | 선택: 커밋마다 파일 해시 manifest + RFC 3161 타임스탬프로 존재 시점을 증명. 기본 off, 외부로 나가는 것은 manifest digest 32바이트뿐 |
| `elf self-update` | `elf` 바이너리 갱신 |

운영 시나리오:

```bash
elf self-update          # 새 버전 반영 ① CLI 갱신
elf status               # ② 무엇이 바뀌는지 진단
elf update --dry-run     # ③ 쓰기 없이 미리 보기
elf update               # ④ 관리 파일만 교체 — "edited: … — kept; new version at ….elf-new"면 내 편집 보존, diff 후 병합(또는 --force로 정본 채택)
elf status --check       # 팀/CI 게이트 — 발견 시 exit 4 (elf validate --check도 같은 방식)
```

`.gitignore`는 마커 블록(`# >>> ELF managed >>>` … `# <<< ELF managed <<<`)만 관리하고, 그 밖의 규칙은 보존됩니다.

## 프로젝트 구조

- 파일 이름은 번호만 씁니다 — `S001_t02_sweep.m` · `Data/S001/S001_t02_results.mat` · `64_Viz/S001/S001_t02_SNR_ci.png`. 조건은 로그에 있습니다.
- 로그의 cross-ref는 상대경로입니다 — 계획 `../1_Concept/12_Planning/P00x.md`, 데이터 `../6_Exp/61_Sim/Data/S###/`, figure `../6_Exp/64_Viz/S###/`. `session close`가 Archive 깊이에 맞춰 보정합니다.
- 계획은 `1_Concept/12_Planning/P###_제목.md`, 아이디어는 `13_Ideas/`, 로그에는 사실만 적습니다. 분석 코드는 `6_Exp/63_Analysis/Scripts/`·`61_Sim/Scripts/`(`.m` 셀 모드 `%%`).

전체 폴더 구조:

```text
Project_Root/
│
├── AGENTS.md                        # 에이전트 진입 규칙 요약 (모든 에이전트 공통, ELF 관리)
├── CLAUDE.md · .claude/settings.json · .claudeignore   # Claude Code 전용 (포인터·autoread 훅·Archive 제외) — 다른 에이전트는 무시
├── README.md · LICENSE · .gitignore · .editorconfig · .gitattributes
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF 제어 영역 (version·config·manifest — 직접 수정 금지)
│   └── managed/                     # 관리 규칙 payload: EliRule·LogConvention·AI_PARA_Framework
│       └── templates/               #   ·LLMcliche·highIFjournals + 세션/trial 스텁 (Archive/ = 구 템플릿 보존)
├── 0_Meta/                          # 프로젝트 거버넌스 — 사용자 영역 (`elf update` 미접근)
│   ├── ProjectRule.md               # 프로젝트 전용 규칙 및 목표
│   ├── <이름>.project.md            # Data overlay (유효 규칙 = base ⊕ overlay)
│   └── tsa/                         # (선택) elf tsa manifest·타임스탬프
│
├── 1_Concept/                       # 연구 기획, 문헌, 아이디어
│   ├── 11_Literature/               # 논문 PDF, 서지 정보, 기반 공식
│   ├── 12_Planning/                 # 연구 기획, 로드맵 (다중 세션)
│   │   └── Wiki/                    # 기획 단계 결론 및 핵심 규칙 요약
│   └── 13_Ideas/                    # 작은 snippet / 초기 naive 아이디어 (flat)
│
├── 2_Log/                           # 세션 로그 (S###_log.md)
│   ├── Wiki/                      # 핵심 발견 요약 및 세션 레지스트리
│   └── Archive/                   # 완료된 세션 로그
│
│  ─── Modules (Optional) ────────────────
│
├── 3_HW/                            # 하드웨어 설계
│   ├── 31_Component/                # 개별 부품 사양서, 단위 소자 설계
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 32_System/                   # 통합 기기 설계, 하우징, 3D 모델
│   └── 33_Elec/                     # PCB 회로도, Gerber, BOM, Datasheets
│
├── 4_Fab/                           # 제작 및 공정
│   ├── 41_Recipes/                  # 공정 조건 문서화
│   └── 42_Eval/                     # 모듈별 단일 특성 평가
│
├── 5_SW/                            # 소프트웨어 & 펌웨어
│   ├── 51_FW/                       # MCU/임베디드 펌웨어
│   ├── 52_DAQ/                      # PC/모바일 데이터 획득 시스템
│   └── 53_Libs/                     # 재사용 가능한 공용 라이브러리
│
├── 6_Exp/                           # 실험: 시뮬레이션 + 실측 + 분석
│   ├── 61_Sim/                      # 시뮬레이션
│   │   ├── Scripts/                 # 시뮬레이션 코드 (S###_sim.m)
│   │   │   └── Archive/          # 폐기 스크립트
│   │   └── Data/                    # 시뮬레이션 결과 (Data/S###/)
│   ├── 62_Empirical/                # 실측 데이터
│   │   ├── Raw/                     # 원본 센서 데이터 (Read-Only, Git 제외)
│   │   └── Processed/               # 1차 가공 데이터
│   ├── 63_Analysis/                 # 통합 분석
│   │   └── Scripts/                 # 비교/검증 포스트프로세싱 코드
│   │       └── Archive/           # 폐기 스크립트
│   └── 64_Viz/                      # 시각화 추출물 (자동 생성 Figure)
│
├── 7_Paper/                         # 논문 & 발표
│   ├── 71_Figs/                     # 논문용 Figure
│   │   ├── Raw/
│   │   ├── Processed/
│   │   └── Final/
│   ├── 72_Drafts/                   # 원고 (Word, LaTeX)
│   │   └── Archive/               # 이전 버전 보관
│   └── 73_Presentations/            # 발표 자료 (PPT, 포스터)
```

각 폴더의 용도와 운영 규칙은 `.elf/managed/EliRule.md`에 있습니다.

## 라이선스 (License)

이 프로젝트는 '구동 코드'와 '데이터 구조 규격(Protocol)'의 성격이 다르므로, 이중 라이선스(Dual License) 정책을 적용함.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **적용 대상:** `5_SW/`, `6_Exp/*/Scripts/` 폴더 내의 모든 소스 코드(`.m`, `.py` 등).
  * **조건:** 템플릿의 코어 스크립트를 수정 및 개선하여 배포할 경우 해당 수정본은 오픈소스로 공개해야 함. 단, 사용자가 프로젝트 내에 추가한 고유 알고리즘이나 원시 데이터는 비공개(상업화) 유지가 가능함.

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **적용 대상:** `README.md`, 거버넌스 문서(`.elf/managed/`·`0_Meta/`), Session-Trial 폴더 계층 구조, Base-Delta 메타데이터 로깅 규칙 등 연구 방법론 전반.
  * **조건:** 누구나 이 구조와 기록 방법론을 자유롭게 차용 및 변형할 수 있으나, 파생된 템플릿이나 관련 연구 결과물 발표 시 원작자 Eli (projectschnee@gmail.com) 와 본 저장소의 출처를 명시해야 함.
