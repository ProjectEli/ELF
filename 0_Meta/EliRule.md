# EliRule: Project Structure & Operational Guide

본 문서는 ELF(Eli's Lab Framework) 프로젝트의 폴더 구조와 운영 규칙을 정의합니다.
README.md가 철학과 개요를 담당한다면, 이 문서는 실무 레벨의 상세 규격입니다.

---

## 1. 폴더 구조 상세

### `0_Meta/` — 프로젝트 거버넌스
연구 데이터가 아닌 프로젝트 운영 규칙을 정의하는 메타 구역입니다.
- `EliRule.md`: 이 문서 (폴더 구조 및 운영 가이드)
- `LogConvention.md`: 로깅 표준 규칙
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
  - `Logs/`: 세션 로그 (`S###_log.md`)
- **`54_Viz/`**: 자동 생성된 시각화 추출물 (Figure PNG 등)

### `6_Paper/` — 논문 & 발표
- **`61_Figs/`**: 논문용 Figure
  - `rawFig/` → `processedFig/` → `finalFig/` (3단계 파이프라인)
- **`62_Drafts/`**: 원고 (Word, LaTeX)
  - `archive/`: 이전 버전 백업
- **`63_Presentations/`**: 발표 자료 (PPT, 포스터)

---

## 2. 운영 규칙

### 2.1 Raw Data 무결성
- `5_Exp/52_Empirical/Raw/`에 저장된 파일은 **읽기 전용(Read-Only)**입니다.
- 스크립트에서 읽기만 수행하며, 원본을 절대 덮어쓰지 않습니다.

### 2.2 Git 분리 전략
- **Git 추적 대상**: 코드(`4_SW/`, `5_Exp/*/Scripts/`), 메타데이터(`0_Meta/`, `5_Exp/53_Analysis/Logs/`), 문서(`README.md`)
- **Git 추적 제외**: 대용량 바이너리(Raw 데이터, 이미지, 문서 파일), `5_Exp/52_Empirical/Raw/`
- 대용량 설계 파일(`2_HW/`)은 Git LFS 또는 별도 드라이브 관리를 권장합니다.

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
