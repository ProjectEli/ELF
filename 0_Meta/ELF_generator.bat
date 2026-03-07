@echo off
chcp 65001 > nul
echo ============================================
echo   ELF v2 Project Structure Generator
echo   (0~6 Hierarchy + PARA Framework)
echo ============================================
echo.

REM 1. 프로젝트 이름 입력 및 중복 검사
:INPUT_LOOP
set /p PROJECT_NAME="생성할 프로젝트 폴더 이름을 입력: "

if "%PROJECT_NAME%"=="" (
    echo [알림] 이름이 입력되지 않아 기본값 'New_ELF_Project'를 시도함.
    set PROJECT_NAME=New_ELF_Project
)

if exist "%PROJECT_NAME%" (
    echo [오류] '%PROJECT_NAME%' 폴더가 이미 존재함. 다른 이름을 입력하기 바람.
    echo.
    goto INPUT_LOOP
)

mkdir "%PROJECT_NAME%"
cd "%PROJECT_NAME%"
echo [1/6] 프로젝트 루트 '%PROJECT_NAME%' 생성 완료.

REM 2. 디렉토리 구조 생성 (ELF v2: 0~6 체계)
mkdir "0_Meta"
mkdir "1_Concept\11_Ideas"
mkdir "1_Concept\12_Literature"
mkdir "1_Concept\13_Planning"
mkdir "2_HW\21_Component\Design"
mkdir "2_HW\21_Component\Calibration"
mkdir "2_HW\22_System"
mkdir "2_HW\23_Elec"
mkdir "3_Fab\31_Recipes"
mkdir "3_Fab\32_Eval"
mkdir "4_SW\41_FW"
mkdir "4_SW\42_DAQ"
mkdir "4_SW\43_Libs"
mkdir "5_Exp\51_Sim\Scripts"
mkdir "5_Exp\51_Sim\Data"
mkdir "5_Exp\52_Empirical\Raw"
mkdir "5_Exp\52_Empirical\Processed"
mkdir "5_Exp\53_Analysis\Scripts"
mkdir "5_Exp\53_Analysis\Logs\2_Wiki"
mkdir "5_Exp\54_Viz"
mkdir "6_Paper\61_Figs\rawFig"
mkdir "6_Paper\61_Figs\processedFig"
mkdir "6_Paper\61_Figs\finalFig"
mkdir "6_Paper\62_Drafts"
mkdir "6_Paper\63_Presentations"
echo [2/6] 디렉토리 구조 생성 완료 (0_Meta ~ 6_Paper).

REM 3. 빈 폴더 Git 추적용 .gitignore 생성
for %%D in (
    "1_Concept\11_Ideas"
    "1_Concept\12_Literature"
    "1_Concept\13_Planning"
    "2_HW\21_Component\Design"
    "2_HW\21_Component\Calibration"
    "2_HW\22_System"
    "2_HW\23_Elec"
    "3_Fab\31_Recipes"
    "3_Fab\32_Eval"
    "4_SW\41_FW"
    "4_SW\42_DAQ"
    "4_SW\43_Libs"
    "5_Exp\51_Sim\Scripts"
    "5_Exp\51_Sim\Data"
    "5_Exp\52_Empirical\Processed"
    "5_Exp\53_Analysis\Scripts"
    "5_Exp\53_Analysis\Logs\2_Wiki"
    "5_Exp\54_Viz"
    "6_Paper\61_Figs\rawFig"
    "6_Paper\61_Figs\processedFig"
    "6_Paper\61_Figs\finalFig"
    "6_Paper\62_Drafts"
    "6_Paper\63_Presentations"
) do echo # Keep folder> %%~D\.gitignore

> "5_Exp\52_Empirical\Raw\.gitignore" echo *
>> "5_Exp\52_Empirical\Raw\.gitignore" echo !.gitignore

type nul > .gitattributes
type nul > LICENSE
echo [3/6] 하위 폴더 .gitignore 및 빈 파일 생성 완료.

REM 4. 메타 문서 및 설정 파일 생성 (PowerShell 사용)
for /f "tokens=1 delims=:" %%n in ('findstr /n "#__PS_START__" "%~f0"') do set "psline=%%n"
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "Get-Content -Path '%~f0' -Encoding UTF8 | Select-Object -Skip %psline% | Set-Content -Path '%TEMP%\_elfgen.ps1' -Encoding UTF8"
powershell -NoProfile -ExecutionPolicy Bypass -File "%TEMP%\_elfgen.ps1"
del "%TEMP%\_elfgen.ps1" 2>nul
echo [4/6] 메타 문서 및 설정 파일 생성 완료.

REM 5. Git 초기화 및 첫 커밋
echo.
git init
git add .
git commit -m "chore: Initialize ELF v2 project structure"
echo [5/6] Git 초기화 완료.

echo.
echo ============================================
echo   [%PROJECT_NAME%] ELF v2 프로젝트 구조 생성 완료!
echo ============================================
echo [6/6] 완료!
pause
exit /b

#__PS_START__
$utf8 = [System.Text.UTF8Encoding]::new($false)
$projName = $env:PROJECT_NAME
$dateStr = Get-Date -Format "yyyy-MM-dd"

# ================================================================
# .gitignore
# ================================================================
[IO.File]::WriteAllText(".gitignore", @'
# === Claude Code & AI Toolkit ===
.claude/
.ecc/
CLAUDE.md
.claudeignore

# === MATLAB Autosave files ===
*.asv
*.m~
*.mlapp~

# === Compiled binaries / MEX files ===
*.mex*
*.mexa64
*.mexw64
*.mexw32
*.mexmaci64

# === Simulink generated files and caches ===
slprj/
sfprj/
*.slxc

# === MATLAB specific temporary files ===
*.prj~
*.log

# === Office autosave files ===
~$*.xls*
~$*.ppt*
~$*.doc*

# === Raw Data ===
5_Exp/52_Empirical/Raw/

# === Optional: large data files ===
# *.mat
# *.fig
'@, $utf8)

# ================================================================
# .claudeignore
# ================================================================
[IO.File]::WriteAllText(".claudeignore", @'
# Ignore archive folders from Claude Code context
**/9_Archive/
**/*Archive*/
'@, $utf8)

# ================================================================
# 0_Meta\EliRule.md
# ================================================================
[IO.File]::WriteAllText("0_Meta\EliRule.md", @'
# EliRule: Project Structure & Operational Guide

ELF(Eli's Lab Framework) 프로젝트의 폴더 구조와 운영 규칙을 정의합니다.
README.md가 철학과 개요를 담당한다면, 이 문서는 실무 레벨의 상세 규격입니다.

---

## 1. 폴더 구조 상세

### `0_Meta/` — 프로젝트 거버넌스
연구 데이터가 아닌 프로젝트 운영 규칙을 정의하는 메타 구역입니다.
- `EliRule.md`: 이 문서 (폴더 구조 및 운영 가이드)
- `LogConvention.md`: 로깅 표준 규칙
- `AI_PARA_Framework.md`: AI 환각 방지를 위한 상태 기반 파일 관리 및 아카이빙 규칙. AI가 프로젝트를 탐색할 때 가장 중요한 기준 문서
- `AI_Sync.md`: AI 에이전트 핸드오프 로그

### `1_Concept/` — 연구 기획 & 아이디어
연구 방향성, 문헌 고찰, 가설 설정을 실험 데이터와 분리하여 보관합니다.
- **`11_Ideas/`**: 러프 스케치, 가설 제안록, 브레인스토밍 메모
- **`12_Literature/`**: 논문 PDF, 서지 정보, 기반 공식 정리
- **`13_Planning/`**: 연구 로드맵, Figure 구성 스토리보드, 실험 계획서
  - Planning 문서는 `P###_제목.md` 형식으로 넘버링 (예: `P001_wavelength_optimization.md`)

### `2_HW/` — 하드웨어 설계
장치의 물리적 설계를 컴포넌트와 통합 시스템으로 분리합니다.
- **`21_Component/`**: 개별 부품 사양서, 단위 소자 설계
  - `Design/`: 설계 파일
  - `Calibration/`: 교정 데이터 및 설정
- **`22_System/`**: 통합 기기 설계, 하우징, 3D 모델 (`.stl`, `.step`)
- **`23_Elec/`**: PCB 회로도, Gerber, BOM, Datasheets

### `3_Fab/` — 제작 & 공정
부품/기기 제작 공정 기록 및 특성 평가를 관리합니다.
- **`31_Recipes/`**: 공정 조건 문서화
- **`32_Eval/`**: 모듈별 단일 특성 평가 데이터

### `4_SW/` — 소프트웨어 & 펌웨어
- **`41_FW/`**: MCU/임베디드 펌웨어 소스
- **`42_DAQ/`**: PC/모바일 데이터 획득 시스템
- **`43_Libs/`**: 재사용 가능한 공용 라이브러리 (필터, SNR 계산 등)

### `5_Exp/` — 실험 (Sim + Empirical + Analysis)
시뮬레이션과 실측 데이터를 1:1 비교 검증할 수 있는 구조입니다.
- **`51_Sim/`**: 시뮬레이션
  - `Scripts/`: 시뮬레이션 코드 (`S###_sim.m` 등)
  - `Data/`: 시뮬레이션 결과 (`Data/S###/`)
- **`52_Empirical/`**: 실측 데이터
  - `Raw/`: 원본 센서 데이터 (**Read-Only, Git 추적 제외**)
  - `Processed/`: 1차 가공 데이터
- **`53_Analysis/`**: 통합 분석
  - `Scripts/`: 비교/검증 포스트프로세싱 코드
  - `Logs/`: 세션 로그 (`S###_log.md`), `2_Wiki/` 요약, `9_Archive/` 보관
- **`54_Viz/`**: 자동 생성된 시각화 추출물 (Figure PNG 등)

### `6_Paper/` — 논문 & 발표
- **`61_Figs/`**: 논문용 Figure
  - `rawFig/` → `processedFig/` → `finalFig/` (3단계 파이프라인)
- **`62_Drafts/`**: 원고 (Word, LaTeX)
- **`63_Presentations/`**: 발표 자료 (PPT, 포스터)

---

## 2. 운영 규칙

### 2.1 Raw Data 무결성
- `5_Exp/52_Empirical/Raw/`에 저장된 파일은 **읽기 전용(Read-Only)**입니다.
- 스크립트에서 읽기만 수행하며, 원본을 절대 덮어쓰지 않습니다.

### 2.2 Git 분리 전략
- **Git 추적 대상**: 코드, 메타데이터, 로그, 분석 Figure, 원고 등 프로젝트 산출물 전반
- **Git 추적 제외**: `5_Exp/52_Empirical/Raw/` (대용량 원본 센서 데이터), 도구 임시 파일
- 대용량 설계 파일(`2_HW/`)은 Git LFS 또는 별도 드라이브 관리를 권장합니다.
- 분석 Figure(`.png` 등)와 원고(`.docx` 등)는 Git으로 추적하여 버전 관리합니다.

### 2.3 Naming Convention
- **Session-Trial**: `S###_t##` (예: `S001_t1.csv`)
- 파일 이름에 실험 조건/변수 정보 나열 **금지** — 모든 조건은 로그에 기록
- Planning 문서: `P###_제목.md` (예: `P001_experiment_roadmap.md`)
- 시뮬레이션 스크립트: `S###_sim.m`
- 분석 스크립트: `S###_analysis.m`

### 2.4 스크립트와 데이터 분리
- 분석 코드는 `Scripts/` 폴더에, 데이터는 `Data/` 또는 `Raw/`/`Processed/` 폴더에 위치
- 데이터 폴더 내부에 코드를 혼재하지 않습니다.

### 2.5 Cross-Reference 규칙
- 로그에서 Planning 문서 참조: `→ see 1_Concept/13_Planning/P001_xxx.md`
- 로그에서 시뮬레이션 데이터 참조: `→ see 5_Exp/51_Sim/Data/S###/`
- 로그에서 분석 스크립트 참조: `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m`

### 2.6 Data Reusability (데이터 영구 보존 원칙)
- 단순 Illustration(시각적 도해)을 제외한 모든 Plot/Graph 생성 시, 그래프에 표면적으로 드러나지 않는 메트릭이나 중간 연산 결과일지라도 **향후 재사용이 가능하도록 반드시 `.mat` 파일(또는 `.csv`) 형태로 원본 Data Array를 함께 저장(Export)**하는 것을 원칙으로 합니다.

---

## 3. AI Communication Rules

프로젝트 내 모든 AI Agent는 사용자와 소통하고 문서를 작성할 때 다음 원칙을 준수합니다:

1. **객관적이고 드라이한 문체 유지**: 불필요한 인삿말, 과도한 칭찬, 주관적 감정 표현, 과장된 형용사 사용을 금지합니다.
2. **비유 금지**: 비유나 은유를 금지하고, 직관적이고 객관적인 학술/엔지니어링 용어로만 사실을 전달합니다.
3. **결론 중심의 명확한 전달**: 분석 결과와 Action Item을 간결하고 명확하게 제시하며, 논리적이고 정교한 엔지니어링 팩트만을 다룹니다.
4. **Data Reusability**: 위 2.6 항목을 엄격히 준수합니다.
5. **과장 및 감정적 수식어 금지 (No Embellishment)**: 정량적 수치와 물리적 인과관계로만 장단점을 서술합니다.
'@, $utf8)

# ================================================================
# 0_Meta\LogConvention.md
# ================================================================
[IO.File]::WriteAllText("0_Meta\LogConvention.md", @'
# LogConvention: ELF 로깅 표준 규칙

사람과 AI 에이전트 모두가 따라야 할 실험 로그 작성, 결과 파일 저장, AI 핸드오프 규칙을 정의합니다.

---

## 1. 로그 파일 위치 및 명명 (PARA 워크플로우 적용)

모든 연구 및 기획 진행은 PARA(Projects, Areas, Resources, Archives) 관리 체계와 `.claudeignore` 룰을 따릅니다. 자세한 원칙은 `0_Meta/AI_PARA_Framework.md`를 참조하십시오.

| 항목 | 규칙 |
|------|------|
| **진행 중 (Active Sandbox)** | 새로운 세션(S{NNN})을 시작할 때는 무조건 **현재 폴더(예: `53_Analysis/Logs/`)의 최상단(Root)**에 로그(`S{NNN}_log.md`)를 생성하여 작성합니다. |
| **결론 요약 (Wiki)** | 세션이 완료되면 핵심 교훈이나 산출된 파라미터를 `2_Wiki/` 폴더 내의 지식 문서에 한두 줄로 요약합니다. 이 때 반드시 원본 Archive 로그 파일의 절대/상대 경로 링크를 포함합니다. |
| **보관 (Archive)** | 세션이 종료되면 전체 원본 로그 파일은 **반드시** `9_Archive/` 폴더로 이동시킵니다. (예: `9_Archive/[Archived]_S{NNN}_log.md`) |
| **내용 규칙** | 시뮬레이션 파라미터, 실행 결과, 에러/해결 등 **순수 Metadata (Fact-Sheet)**만을 기록합니다. |
| **금지 사항** | 아이디어, 논문 기획, 방향성 논의는 `5_Exp/`에 기록하지 말고 `1_Concept/13_Planning/`의 기획 문서로 분리합니다. |

---

## 2. 로그 포맷

```markdown
# S{NNN}: {세션 제목}
**Date**: YYYY-MM-DD
**Status**: {Planning | In Progress | Step X 완료 | Complete}

## 요약정보
- 목표:
- 배경:
- 교훈:

## 상세내용

### t{NN}: {작업 제목}
- 목표:
- 교훈:

{실행 내용, 파라미터, 결과, 이미지 링크 등}
```

### 규칙
- **ticket 번호**: `t01`, `t02`, ... 순서대로. 중복 금지
- **이미지 경로**: `![alt text](../../51_Sim/Data/S{NNN}/filename.png)` (Logs 기준 상대경로)
- **Figure 인라인 임베딩 필수**: 분석 결과로 생성된 Figure는 반드시 로그 본문의 **해당 결과 섹션에 인라인으로 삽입**하고, alt text에 Figure 번호와 1줄 설명(축, 핵심 관찰)을 기재한다. 파일 목록 테이블에만 나열하고 본문에 임베딩하지 않는 것은 금지. 향후 사람이 로그만 읽고도 결과를 시각적으로 파악할 수 있어야 한다.
- **코드 사용법**: 코드 블록 (```lang ... ```) 으로 기재
- **파라미터 표**: 변수명, 값, 단위를 표 형태로 정리

---

## 3. 결과 파일 저장 규칙

### 3.1 스크립트

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 스크립트 | `51_Sim/Scripts/` | `S{NNN}_sim.m` |
| 후처리 스크립트 | `51_Sim/Scripts/` 또는 `53_Analysis/Scripts/` | `S{NNN}_postProcess.m` |
| 헬퍼 함수 | `51_Sim/Scripts/` | `{function_name}.m` |
| 분석 코드 | `53_Analysis/Scripts/` | `S{NNN}_analysis.m` |

### 3.2 데이터

| 종류 | 위치 | 명명 |
|------|------|------|
| 시뮬 결과 | `51_Sim/Data/S{NNN}/` | `S{NNN}_sim_results.mat` |
| 그래프 | `51_Sim/Data/S{NNN}/` | `S{NNN}_Fig1_*.png` |
| 실측 원본 | `52_Empirical/Raw/` | Read-Only |
| 가공 데이터 | `52_Empirical/Processed/S{NNN}/` | `S{NNN}_t##_processed.csv` 등 |

### 3.3 명명 규칙
- 시뮬 결과: `S{NNN}_{ticket}_{description}.mat`
- 그래프: `S{NNN}_{description}.png`
- 스크립트 ticket 구분 필요 시: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`

### 3.4 시각화

| 종류 | 위치 | 비고 |
|------|------|------|
| 자동 생성 Figure | `5_Exp/54_Viz/` | 스크립트에서 자동 저장 |
| 논문용 Figure | `6_Paper/61_Figs/` | rawFig → processedFig → finalFig |

---

## 4. AI_Sync.md 업데이트 규칙

작업을 **완료**할 때마다 `0_Meta/AI_Sync.md`에 항목을 추가합니다.

### 포맷
```markdown
## [YYYY-MM-DD / {Agent Name}] {작업 제목}

### 수행한 작업
- 생성/수정/삭제한 파일 목록 (표 형태)
- 핵심 결과 요약

### Next Steps
- 다음 에이전트가 이어서 수행할 구체적 Task
- 필요한 파일 경로, 함수명 등 명시
```

### 규칙
- **최신 항목이 위**에 오도록 작성 (역순)
- 파일 경로는 프로젝트 루트 기준 상대경로 사용
- Next Steps에는 구체적인 파일명, 함수 시그니처, 예상 입출력을 포함

---

## 5. Cross-reference 규칙

| From | To | 상대경로 (Logs 기준) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/13_Planning/P00x.md` | `../../../1_Concept/13_Planning/P00x.md` |
| Logs → Sim Data | `51_Sim/Data/S{NNN}/` | `../../51_Sim/Data/S{NNN}/` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Planning으로 분리된 내용은 원본 Log에 blockquote stub으로 표시:
```markdown
> 상세 내용은 Planning 문서로 분리되었습니다.
> - [P00x_Title.md](../../../1_Concept/13_Planning/P00x_Title.md) — 섹션 N 참조
```

---

## 6. 세션 생성 시 체크리스트

AI 에이전트가 새 세션(S{NNN})을 시작할 때:

- [ ] `53_Analysis/Logs/S{NNN}_log.md` 생성 (위 포맷 준수, 최상단에 생성)
- [ ] `51_Sim/Data/S{NNN}/` 디렉토리 생성
- [ ] 시뮬/실험 스크립트 `51_Sim/Scripts/S{NNN}_*.m` 작성
- [ ] 실행 후 결과 .mat + .png 저장
- [ ] Log에 ticket (t{NN}) 추가: 파라미터, 결과, 이미지 경로
- [ ] 작업 완료 시: 얻은 지식을 `2_Wiki`에 요약하고, `S{NNN}_log.md`는 `9_Archive/`로 이동
- [ ] 스크립트 완료 시: 1회용 스크립트는 `Scripts/9_Archive/`로 이동, 범용은 Root 유지
- [ ] `53_Analysis/Logs/2_Wiki/Session_Registry.tsv`에 해당 세션 항목 추가 업데이트
- [ ] `AI_Sync.md` 업데이트
- [ ] Planning 내용이 포함된 경우 → `1_Concept/`로 분리 + cross-reference
'@, $utf8)

# ================================================================
# 0_Meta\AI_PARA_Framework.md
# ================================================================
[IO.File]::WriteAllText("0_Meta\AI_PARA_Framework.md", @'
# AI PARA Framework & Context Management

이 문서는 프로젝트 내의 방대한 실험 로그와 기획 문서들이 AI 에이전트(Claude Code, Gemini 등)의 컨텍스트 윈도우(Context Window)를 오염시키는 현상(Hallucination)을 막고, 인간-AI 협업 시 최적의 효율을 내기 위한 **AI 맞춤형 PARA (Projects, Areas, Resources, Archives) 파일 관리 규칙**을 정의함.

## 1. 파일 격리 원칙 (The Firewall Principle)

이 프로젝트의 최상단 폴더에는 `.claudeignore` 파일이 존재함.
이 파일은 AI가 파일 시스템을 자율적으로 검색하거나 읽어들일 때, 특정 이름의 폴더들을 아예 **투명 인간 취급**하여 인식하지 못하도록 차단(Firewall)함.

*   **차단 대상**: `9_Archive/` 및 `*Archive*` 등 과거의 폐기되거나 종료된 기록물.
*   **효과**: AI가 "현재 유효한 프로젝트의 상태"만을 바탕으로 대답할 수 있게 보장하며, 예전의 실패한 세팅값(예: 잘못된 파라미터 맵, 폐기된 논문 전개 방향안)을 현재의 사실로 오인하는 것을 완벽히 방지함.

## 2. 하이브리드 PARA 구조 (Focus-and-Filter)

인간을 위한 '인지 부하 감소(Grouping)'와 AI를 위한 '경로 평탄화(Flattening)'를 동시에 달성하기 위해, 프로젝트는 **하이브리드 구조**로 운용됨.

### 작업대 (Default Root / Active Sandbox)
*   **목적**: 지금 당장, 혹은 이번 주에 유효하게 "진행 중(In-Progress)"인 내용이 위치하는 곳임.
*   **파일 예시**: `Current_Analysis_Task.md`, `S014_log.md`
*   **운영 규칙**: 별도의 `1_Active` 폴더를 만들지 않고, **작업 중인 부모 폴더(예: `13_Planning/` 또는 `53_Analysis/Logs/`)의 최상단(Root)**을 바로 작업대로 사용함. 작업이 완료되어 과거의 기록이 되기 전까지는 이 공간에서 자유롭게 작성함.

### `2_Wiki/` (안식처 / Human Sanctuary)
*   **목적**: 작업대에서 작업이 끝난 후 얻어낸 **"변하지 않는 사실, 결론, 핵심 규칙"**만을 한두 줄로 요약해 모아두는 곳임.
*   **운영 규칙**: 인간 연구자가 "현재 제일 중요한 팩트/규칙" 만을 모아서 열람하고 싶을 때 이 폴더에 접근함. AI에게도 핵심 요약 컨텍스트를 제공하는 진실 공급원(Fact-Sheet) 역할을 함.

### `9_Archive/` (AI 방화벽 / The Firewall Bin)
*   **목적**: 작업이 완전히 종료(Complete)되거나 폐기(Deprecated)되어 더 이상 "현재의 관심사"가 아니지만, 나중에 참고할 일이 있을 수 있는 원본 로그 기록을 보관함.
*   **운영 규칙**: 폴더 최상단이 지저분해지면, 구형 파일명 앞에 `[Archived]_`나 `[Deprecated]_`라는 태그를 붙여 이 폴더로 밀어 넣음. 이 폴더에 들어가는 순간, `.claudeignore`에 의해 AI의 자율 검색망에서 완벽하게 사라짐.

---

## 3. Scripts 폴더 관리 (코드 아카이빙)

`51_Sim/Scripts`나 `53_Analysis/Scripts` 내부에 스크립트가 무분별하게 쌓이는 것을 방지하기 위해 동일한 PARA 논리를 스크립트에도 적용함.

1.  **Active Scripts**: 현재 개발 중이거나 범용적으로 쓰이는 최신 스크립트는 Scripts 폴더의 최상단(Root)에 유지함.
2.  **Archived Scripts**: 특정 과거 세션에만 일회용으로 쓰였던 스크립트는 `Scripts/9_Archive/` 밑으로 이동시킴.
3.  **Wiki Tracking (Registry)**: 스크립트를 Archive로 옮길 때는, 해당 스크립트가 어떤 세션에서 무엇을 위해 쓰였는지 `2_Wiki/` 문서에 **경로와 함께 표기**함.

## 4. AI 접근 복원 방법 (How to bypass the firewall)

AI는 `9_Archive`를 스스로 뒤져볼 수 없지만, 인간 개발자가 특정 과거 기록의 복원/분석을 요구할 때는 **명시적 지시(Explicit Instruction)**를 통해 열람 가능함.

### 방법: 절대/상대 경로 강제 지정
사용자가 프롬프트 상에서 "과거의 A 파일을 열어봐"라며 정확한 경로 스니펫을 주면 AI는 정상적으로 해당 파일을 읽고 컨텍스트에 불러올 수 있음.

*   *(사용자 프롬프트 예시)*: "`5_Exp/53_Analysis/Logs/9_Archive/S005_log.md` 파일을 열어서 당시의 파라미터 값 추이를 요약해 줘."
*   이러한 방식을 돕기 위해, `2_Wiki`의 지식 문서들은 과거 데이터가 필요할 경우를 대비하여 항상 `9_Archive/...`로 향하는 **명시적 파일 경로 링크**를 포함해야 함.
'@, $utf8)

# ================================================================
# 0_Meta\AI_Sync.md
# ================================================================
[IO.File]::WriteAllText("0_Meta\AI_Sync.md", @'
# AI_Sync: Agent Handoff Log

최신 항목이 위에 오도록 역순으로 작성합니다.
포맷 및 상세 규칙: `LogConvention.md` 섹션 4 참조.

---

(아직 핸드오프 기록이 없습니다.)
'@, $utf8)

# ================================================================
# README.md (변수 삽입 필요 — 단독 처리)
# ================================================================
$readmeContent = @'
# {0}

## 프로젝트 개요
- **연구 목표:** [여기에 테스트 목표 및 가설 작성]
- **연구 기간:** {1} ~
- **담당 연구자:** [이름 작성]

## 하드웨어 및 소프트웨어 베이스라인 (Baseline)
- **HW Version:** [기구/회로/펌웨어의 기본 상태 명시]
- **SW Version:** [분석 스크립트 및 툴 환경 명시]

## 데이터 파이프라인 규격 (Protocol)
본 프로젝트는 Agile R&D Boilerplate 규격(ELF v2)을 준수함.
- **rawData:** 센서 원시 데이터 보관 (읽기 전용)
- **metaData:** 세션-트라이얼 (Session-Trial) 기반 Base-Delta 마크다운 로깅
- **scripts:** 부분 실행 (Cell Mode)을 위한 후처리 스크립트 관리

## 프로젝트 규칙
폴더 구조 및 운영 상세 규칙은 `0_Meta/EliRule.md`를 참조.
AI 에이전트 로깅 규칙은 `0_Meta/LogConvention.md`를 참조.
AI 컨텍스트 관리 규칙은 `0_Meta/AI_PARA_Framework.md`를 참조.
'@
[IO.File]::WriteAllText("README.md", ($readmeContent -f $projName, $dateStr), $utf8)

# ================================================================
# 5_Exp\53_Analysis\Logs\S001_log.md
# ================================================================
$logContent = @'
# S001: [세션 제목]
**Date**: {0}
**Status**: Planning

## 요약정보
- 목표:
- 배경:
- 교훈:

## 상세내용

### t01: [작업 제목]
- 목표:
- 교훈:
'@
[IO.File]::WriteAllText("5_Exp\53_Analysis\Logs\S001_log.md", ($logContent -f $dateStr), $utf8)

# ================================================================
# 5_Exp\53_Analysis\Logs\2_Wiki\Session_Registry.tsv
# ================================================================
$tsvContent = "Session`tDate`tTitle`tStatus`tKey Finding`tArchive Path`r`n"
$tsvContent += "S001`t$dateStr`t[세션 제목]`tPlanning`t-`t-`r`n"
[IO.File]::WriteAllText("5_Exp\53_Analysis\Logs\2_Wiki\Session_Registry.tsv", $tsvContent, $utf8)

Write-Host "[PS] 메타 문서 7개 생성 완료."
