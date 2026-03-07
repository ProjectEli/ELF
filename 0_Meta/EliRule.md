# EliRule: Project Structure & Operational Guide

본 문서는 ELF(Eli's Lab Framework) 프로젝트의 폴더 구조와 운영 규칙을 정의합니다.
README.md가 철학과 개요를 담당한다면, 이 문서는 실무 레벨의 상세 규격입니다.

---

## 1. 폴더 구조 상세

### `0_Meta/` — 프로젝트 거버넌스
연구 데이터가 아닌 프로젝트 운영 규칙을 정의하는 메타 구역입니다.
- `EliRule.md`: 이 문서 (폴더 구조 및 운영 가이드)
- `LogConvention.md`: 로깅 표준 규칙
- `AI_PARA_Framework.md`: AI의 환각을 방지하기 위한 상태 기반 파일 관리 및 아카이빙 규칙. AI가 프로젝트를 탐색할 때 가장 중요한 기준 문서
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
5. **과장 및 감정적 수식어 금지 (No Embellishment)**: '압도적(Overwhelming)', '무기(Weapon)', '치명적', '파급력' 등 감정을 자극하거나 극단적인 수식어의 사용을 전면 배제합니다. 오직 정량적 수치와 물리적 인과관계로만 장단점을 서술합니다.
