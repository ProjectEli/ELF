[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF): Base-Delta Protocol for Agile R&D

기기 개발 및 R&D 검증 단계의 빠른 피드백 루프(Agile)를 지원하기 위한 하드웨어-소프트웨어-실험 데이터 통합 기록 규격(Protocol)임. 연구자의 기록 피로도를 최소화하면서도 완벽한 데이터 추적성을 보장함.

## 핵심 철학 (Philosophy)

* **단일 진실 공급원 (Single Source of Truth):** 하드웨어 설계, 분석 코드, 원시 데이터를 하나의 프로젝트 내에서 유기적으로 연결함.
* **Base-Delta 로깅:** 모든 변수를 기록하지 않음. 기준(Baseline)을 선언하고, 변경된 변수(Delta)만 가볍게 기록하여 연구 지연을 방지함.
* **시스템적 강제성:** 파일명 길이 제한(Windows 260자)을 우회하고, 코드를 통한 재현성을 보장함.
* **AI 거버넌스:** `.elf/managed/LogConvention.md`로 사람과 AI 모두 동일한 로깅 규격을 따르도록 강제하고, `.elf/managed/AI_PARA_Framework.md`로 컨텍스트 오염을 방지함.

## 프로젝트 디렉토리 규격 (Directory Structure)

본 프로젝트는 아래의 폴더 계층 구조 자체를 하나의 통신 규격으로 간주함.

```text
Project_Root/
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF 제어 영역 (version·config·manifest — 직접 수정 금지)
│   └── managed/                     # 관리 규칙 payload: EliRule·LogConvention·AI_PARA_Framework
│       └── templates/               #   ·LLMcliche·highIFjournals + 세션/trial 스텁
├── 0_Meta/                          # 프로젝트 거버넌스 — 사용자 영역 (`elf update` 미접근)
│   ├── ProjectRule.md               # 프로젝트 전용 규칙 및 목표
│   └── <이름>.project.md            # Data overlay (유효 규칙 = base ⊕ overlay)
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

> 각 폴더의 상세 용도와 운영 규칙은 `.elf/managed/EliRule.md`를 참조.

## 데이터 로깅 파이프라인 규격

### 1. 파일 명명 규칙 (Session-Trial Naming)

* 파일 이름에 실험 조건이나 변수 정보를 나열하는 것을 **엄격히 금지**함.
* **형식:** `[세션ID]_[트라이얼ID].[확장자]` (예: `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta 로깅 (Hybrid Logging)

* **러닝 로그 (`2_Log/S###_log.md`):**
  * 즉각적인 가설-테스트-교훈을 텍스트로 기록하는 서사형 마크다운 파일임.
  * 트라이얼(`t1`, `t2`...) 단위로 **의도적으로 변경한 변수(Delta)**와 관찰된 결과만 의식의 흐름대로 작성함.
  * 포맷 및 상세 규칙: `.elf/managed/LogConvention.md` 참조.

### 3. Planning 문서 규칙

* 연구 로드맵, Figure 구성, 실험 전략 등은 `1_Concept/12_Planning/`에 별도 관리.
* **형식:** `P###_제목.md` (예: `P001_wavelength_optimization.md`)
* 로그에서 Planning 참조 시: `→ see 1_Concept/12_Planning/P###_xxx.md`

### 4. 후처리 분석 규격 (Cell Mode Scripting)

* 분석 코드는 `6_Exp/63_Analysis/Scripts/` 또는 `6_Exp/61_Sim/Scripts/`에 위치해야 하며, 데이터 폴더 내부에 혼재 불가함.
* 벤더 종속성(Vendor Lock-in) 방지를 위해 `.mlx` 대신 순수 `.m` 파일을 사용함.
* 코드 내 `%%` (Cell Mode)를 활용해 구역별로 실행하며, 도출된 인사이트는 러닝 로그에 반영함.
* 분석 결과물(그림, mat파일)은 `6_Exp/64_Viz/` 또는 `6_Exp/62_Empirical/Processed/S###/`에 세션별 폴더를 생성하여 저장함.

### 5. Cross-Reference 규칙

프로젝트 내 문서 간 추적성을 확보하기 위해 상호 참조 형식을 통일함.

| From → To | 형식 |
|-----------|------|
| Logs → Planning | `→ see 1_Concept/12_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ see 6_Exp/61_Sim/Data/S###/` |
| Logs → Script | `→ see 6_Exp/63_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← tracked in 2_Log/S###_log.md` |

## AI 거버넌스

AI 에이전트(Claude 등)가 프로젝트에 참여할 때 다음 규칙을 따름:

1. **컨텍스트 파악**: 작업 시작 전 `2_Log/`의 활성 세션 로그와 `2_Log/Wiki/Session_Registry.tsv`를 읽어 이전 작업 상태 확인.
2. **동일 규격 준수**: `.elf/managed/LogConvention.md`의 로깅 규칙을 사람과 동일하게 따름.
3. **핸드오프 기록**: 작업 완료 시 세션 로그(`2_Log/S###_log.md`)에 수행 내역, 생성/수정 파일, Next Steps 기록 — 로그 헤더의 `Handoff` 필드 사용.
4. **아이디어 분리**: AI가 생성한 가설/아이디어는 로그가 아닌 `1_Concept/`에 별도 저장 (작은 아이디어 → `13_Ideas/`, 계획 → `12_Planning/`).
5. **PARA 기반 컨텍스트 관리**: `Archive/` 폴더와 `.claudeignore`를 활용하여 AI의 컨텍스트 오염을 방지. 상세 규칙은 `.elf/managed/AI_PARA_Framework.md` 참조.
6. **Communication Rules**: 객관적이고 드라이한 문체 유지. 비유/은유 금지. 결론 중심의 명확한 전달. 과장 및 감정적 수식어 금지. 상세 규칙은 `.elf/managed/EliRule.md` 섹션 3 참조.
7. **Data Reusability**: 모든 Plot/Graph 생성 시 원본 Data Array를 `.mat`/`.csv`로 함께 저장. 상세 규칙은 `.elf/managed/EliRule.md` 섹션 2.6 참조.
8. **Data 파일 커스터마이즈**: 어휘·검색 도메인의 추가/제외/재정의는 project overlay `0_Meta/<이름>.project.md`(사용자 소유; 유효 규칙 = base ⊕ overlay; 제외는 사유 기재)로 선언. 규약: `.elf/managed/EliRule.md` §2.7.

## Quick Start

### 1. `elf` CLI 설치 (권장)

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

자기완결 단일 실행파일이 `~/.elf/bin`에 설치되고 PATH에 등록됩니다 — Node/Python 등 런타임 불필요. 새 셸에서 `elf --version`으로 확인하세요.

### 2. 프로젝트 생성

```bash
cd /원하는/상위/디렉토리
elf init MyProject                                  # 기본: full preset, 한국어
elf init MyProject --preset experimental            # 6_Exp + 7_Paper만
elf init MyProject --modules hw,sw --lang English   # custom 모듈 선택
```

Core 폴더(0~2)는 항상 생성되며, Module 폴더(3~7)는 preset(`full`/`experimental`/`software`/`minimal`) 또는 `--modules` 선택에 따라 포함됩니다.


## CLI 명령 & 사용 시나리오

> 전체 명령 레퍼런스(모든 플래그·exit code·escalation·소유권): **[elf-cli/CLI.ko.md](elf-cli/CLI.ko.md)**

| 명령 | 역할 |
|------|------|
| `elf init <이름> [--preset …] [--modules …] [--lang …]` | 새 프로젝트 스캐폴드 생성 |
| `elf update [--dry-run] [--force]` | 프로젝트의 ELF 관리 파일을 현재 CLI 버전으로 갱신 — **사용자 파일 무손실** |
| `elf status [--check]` | 관리 파일 상태 진단 (읽기전용). `--check`는 발견 시 exit 4 |
| `elf validate [--check]` | 세션/Registry/로그 정합 검사 (읽기전용). `--check`는 issue 시 exit 4 |
| `elf session new <제목>` | 다음 세션 로그 생성 + 등록 (S### 자동 증번) |
| `elf session close [S###]` | 활성 세션 종료 → Archive 이동 + Registry 갱신 (cross-ref 보정) |
| `elf session fix-headers` | 세션 로그 헤더 hard break(`\`) 보정 |
| `elf gallery` | `6_Exp/64_Viz/`에서 Figure 색인 `_gallery.md` 생성 |
| `elf doctor` | 환경+프로젝트 종합 건강검진 (읽기전용) |
| `elf self-update` (= `elf update --self`) | `elf` 바이너리 자체를 최신 릴리즈로 갱신 |

### 시나리오 A — 새 프로젝트 시작

```bash
elf init NIRS_Probe --preset experimental
cd NIRS_Probe
# .elf/managed/EliRule.md · LogConvention.md 읽기 → 2_Log/S001_log.md에서 연구 시작
```

### 시나리오 B — ELF 새 버전을 기존 프로젝트에 반영

```bash
elf self-update          # ① CLI 자체를 최신으로
cd MyProject
elf status               # ② 무엇이 바뀔지 진단 (outdated / edited / missing)
elf update --dry-run     # ③ 변경 없이 작업 목록 미리보기
elf update               # ④ 갱신 — ELF 관리 파일만 교체, 연구 데이터·로그·설정은 절대 미접근
```

> **v2.15 이전에 생성된 프로젝트 업그레이드**(규칙이 `0_Meta/`·스텁이 루트 `templates/`에 있는 경우):
> 현행 CLI는 그 레이아웃을 읽지도 이전하지도 않습니다. 2단계 경로를 사용하세요 —
> [Releases 페이지](https://github.com/ProjectEli/ELF/releases/tag/v2.15.1)에서 **v2.15.1**을 설치해
> 그 버전으로 `elf update` 후 `elf migrate`를 실행하고, 그 다음에 CLI를 최신으로 갱신(`elf self-update`)합니다.
> 최신 `elf update`를 그런 프로젝트에 바로 실행하면 규칙이 `.elf/managed/`에 이중 배치되고 구 파일은
> 비관리 잔재로 남습니다 — 경고로 파일명을 안내하며, 삭제·이전은 하지 않습니다.

### 시나리오 C — 내가 수정한 관리 파일과 충돌할 때

```bash
elf update
# → "edited: .elf/managed/LogConvention.md — kept; new version at ….elf-new"
#   내 편집본은 보존되고, 새 버전이 <파일>.elf-new 로 생성됨 → diff 후 직접 병합
elf update --force       # 또는: 내 편집을 버리고 정본으로 교체
```

`.gitignore`는 마커블록(`# >>> ELF managed >>>` ~ `# <<< ELF managed <<<`) 안쪽만 ELF가 관리하며, 블록 밖에 추가한 사용자 규칙은 항상 보존됩니다.

### 시나리오 D — 팀/CI에서 drift 차단

```bash
elf status --check       # 발견 시 exit 4 → pre-commit 훅·CI 게이트로 사용
```

## 사용법 (Usage)

> **Tip — 세션 수명주기는 CLI로:** 아래 워크플로는 자동화 가능 — `elf session new`(시작), `elf gallery`(Figure 색인), `elf validate`(정합 검사), `elf session close`(완료). 수동 절차도 그대로 유효하니 CLI는 점진적으로 도입하면 됩니다.

### 0. 템플릿 (Templates)

`.elf/managed/templates/` 폴더에는 즉시 사용 가능한 마크다운 스텁이 포함되어 있습니다:

| 파일 | 사용 시점 |
|------|----------|
| `sessionTemplate.md` | 새 세션 시작 시 `2_Log/`에 복사 후 `S###_log.md`로 이름 변경 |
| `trialTemplate.md` | 진행 중인 세션 로그에 trial 추가 시 (`t02`, `t03`, ...) 본문에 붙여넣기 |

> **Note**: `ProjectRule.md`는 프로젝트 생성 시 자동으로 `0_Meta/` 내부에 배치됩니다. 프로젝트 특성에 맞게 `0_Meta/ProjectRule.md`의 섹션 1~8을 직접 수정하여 사용하세요.

### 1. 규칙 문서 읽기

연구 시작 전, `.elf/managed/`에 생성된 두 거버넌스 문서를 읽습니다:

| 문서 | 목적 |
|------|------|
| `EliRule.md` | 폴더 구조 규격, 네이밍 컨벤션, 운영 규칙 (섹션 1-2), AI 커뮤니케이션 규칙 (섹션 3) |
| `LogConvention.md` | 세션 로그 포맷, 파일 명명, 아카이빙 워크플로우, Cross-reference 규칙 |

### 2. 새 세션 시작

`2_Log/`에 로그 파일을 생성합니다:

```markdown
# S002: 파장 최적화 시뮬레이션

> **Created**: 2026-04-01\
> **Modified**: 2026-04-01\
> **Status**: ★ 활성\
> **목표**: Monte Carlo 시뮬레이션으로 735/810/940 nm 파장별 SNR 비교\
> **관련**: P001_wavelength_optimization.md\
> **Handoff**: -
```

- 세션 번호(`S001`, `S002`, ...)는 순차 증가 — 빈 번호, 중복 금지.
- 파일명: `S###_log.md` (예: `S002_log.md`).
- `elf session new "<제목>"`이 템플릿에서 로그 생성 + 자동 등록(S### 자동 증번); 또는 `.elf/managed/templates/sessionTemplate.md` 수동 복사.

### 3. 작업 전개 (t01, t02, ...)

각 세션 내에서 작업을 순차 task로 분리합니다:

```markdown
## t01: MCX Forward Simulation — 3파장 sweep

### 목표
- λ = {735, 810, 940} nm에서 SDS = 20 mm 조건으로 MCX 시뮬레이션 실행

### 조건
- Tissue model: 3-layer (epidermis/dermis/subcutaneous)
- Photon count: 1e8 per wavelength
- fmel = 0.10 (Fitzpatrick III)

### 결과
- 940 nm이 최고 감도 (ΔR/Δh = 0.12 mm⁻¹)
- 735 nm은 noise floor 최저이나 h > 15 mm에서 포화

![S002_t01: SNR 비교](../6_Exp/64_Viz/S002/S002_t01_SNR_comparison.png)

### 교훈
- 810 nm이 감도와 dynamic range 간 최적 절충점

### 생성 파일

| 유형 | 파일 |
|------|------|
| Script | `61_Sim/Scripts/S002_t01_wavelength_sweep.m` |
| Output | `61_Sim/Data/S002/S002_t01_results.mat` |
| Figure | `64_Viz/S002/S002_t01_SNR_comparison.png` |
```

- Task는 순차적으로 발전: `t01` → `t02` → `t03`.
- 각 task 구성: **목표** (goal), **조건** (conditions), **결과** (results), **교훈** (lesson), **생성 파일** (generated files).
- Figure는 결과 섹션에 인라인 임베딩 필수 — 파일 목록만 나열하고 본문에 임베딩하지 않는 것은 금지.

### 4. 세션 완료

세션 종료 시 다음을 수행합니다:

1. **Status 변경**: 로그 헤더의 `★ 활성`을 `Complete`로 변경.
2. **Wiki 요약**: `2_Log/Wiki/`의 지식 문서에 1-2줄 요약 추가. 아카이브된 로그 경로 링크 포함.
3. **Session Registry 업데이트**: `2_Log/Wiki/Session_Registry.tsv`에 행 추가:
   ```
   S002	2026-04-01	파장 최적화	Complete	810 nm 최적	Archive/S002_log.md
   ```
4. **로그 아카이빙**: 로그 파일을 `2_Log/Archive/`로 이동 (파일명 그대로).
5. **스크립트 아카이빙** (1회용인 경우): `Scripts/Archive/`로 이동.

> `elf session close [S###]`이 1·3(Status)·4단계를 자동 수행 — 아카이브 깊이에 맞춰 로그의 상대 cross-ref도 보정. 2·5단계는 수동.

### 5. AI 에이전트 핸드오프 (선택)

AI 에이전트를 사용하는 경우, 작업 완료 시 세션 로그의 `Handoff` 필드에 수행 내역, 수정 파일, Next Steps를 기록합니다.

## 라이선스 (License)

이 프로젝트는 '구동 코드'와 '데이터 구조 규격(Protocol)'의 성격이 다르므로, 이중 라이선스(Dual License) 정책을 적용함.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **적용 대상:** `5_SW/`, `6_Exp/*/Scripts/` 폴더 내의 모든 소스 코드(`.m`, `.py` 등).
  * **조건:** 템플릿의 코어 스크립트를 수정 및 개선하여 배포할 경우 해당 수정본은 오픈소스로 공개해야 함. 단, 사용자가 프로젝트 내에 추가한 고유 알고리즘이나 원시 데이터는 비공개(상업화) 유지가 가능함.

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **적용 대상:** `README.md`, 거버넌스 문서(`.elf/managed/`·`0_Meta/`), Session-Trial 폴더 계층 구조, Base-Delta 메타데이터 로깅 규칙 등 연구 방법론 전반.
  * **조건:** 누구나 이 구조와 기록 방법론을 자유롭게 차용 및 변형할 수 있으나, 파생된 템플릿이나 관련 연구 결과물 발표 시 원작자 Eli (projectschnee@gmail.com) 와 본 저장소의 출처를 명시해야 함.
