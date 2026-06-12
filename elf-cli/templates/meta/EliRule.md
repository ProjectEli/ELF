# EliRule: Project Structure & Operational Guide

본 문서는 ELF(Eli's Lab Framework) 프로젝트의 폴더 구조와 운영 규칙을 정의합니다.
README.md가 철학과 개요를 담당한다면, 이 문서는 실무 레벨의 상세 규격입니다.

---

## 1. 폴더 구조 상세

### Core (항상 포함)

#### `0_Meta/` — 프로젝트 거버넌스
연구 데이터가 아닌 프로젝트 운영 규칙을 정의하는 메타 구역입니다.
- `EliRule.md`: 이 문서 (폴더 구조 및 운영 가이드)
- `LogConvention.md`: 로깅 표준 규칙
- `AI_PARA_Framework.md`: AI의 환각을 방지하기 위한 상태 기반 파일 관리 및 아카이빙 규칙. AI가 프로젝트를 탐색할 때 가장 중요한 기준 문서
- `AI_Sync.md`: AI 에이전트 핸드오프 로그

#### `1_Concept/` — 연구 기획 & 아이디어
연구 방향성, 문헌 고찰, 가설 설정을 실험 데이터와 분리하여 보관합니다.
- **`11_Literature/`**: 논문 PDF, 서지 정보, 기반 공식 정리
- **`12_Planning/`**: 연구 기획, 로드맵, Figure 구성 스토리보드 (다중 세션·roadmap 단위)
  - Planning 문서는 `P###_제목.md` 형식으로 넘버링 (예: `P001_wavelength_optimization.md`)
  - `Wiki/`: 기획 단계 결론 및 핵심 규칙 요약
- **`13_Ideas/`**: 세션화하기엔 작은 snippet·초기 naive 아이디어. flat(Archive 없음, 폐기하지 않음). 성숙 시 Planning 문서나 세션 trial로 승격.

#### `2_Log/` — 세션 로그
모든 종류의 작업(실험, 기획, SW 개발 등)을 기록하는 세션 로그의 최상위 공간입니다.
- `S###_log.md`: 세션 로그 파일 (포맷: `0_Meta/LogConvention.md` 참조)
- `Wiki/`: 핵심 발견 요약 및 Session Registry
- `Archive/`: 완료된 세션 로그 보관

### Modules (선택적 포함)

> Generator 실행 시 preset 선택으로 필요한 모듈만 포함 가능.

#### `3_HW/` — 하드웨어 설계
장치의 물리적 설계를 컴포넌트와 통합 시스템으로 분리합니다.
- **`31_Component/`**: 개별 부품 사양서, 단위 소자 설계
  - `Design/`: 설계 파일
  - `Calibration/`: 교정 데이터 및 설정
- **`32_System/`**: 통합 기기 설계, 하우징, 3D 모델 (`.stl`, `.step`)
- **`33_Elec/`**: PCB 회로도, Gerber, BOM, Datasheets

#### `4_Fab/` — 제작 & 공정
부품/기기 제작 공정 기록 및 특성 평가를 관리합니다.
- **`41_Recipes/`**: 공정 조건 문서화
- **`42_Eval/`**: 모듈별 단일 특성 평가 데이터

#### `5_SW/` — 소프트웨어 & 펌웨어
- **`51_FW/`**: MCU/임베디드 펌웨어 소스
- **`52_DAQ/`**: PC/모바일 데이터 획득 시스템
- **`53_Libs/`**: 재사용 가능한 공용 라이브러리 (필터, SNR 계산 등)

#### `6_Exp/` — 실험 (Sim + Empirical + Analysis)
시뮬레이션과 실측 데이터를 1:1 비교 검증할 수 있는 구조입니다.
- **`61_Sim/`**: 시뮬레이션
  - `Scripts/`: 시뮬레이션 코드 (`S###_sim.m` 등), `Archive/`: 폐기 스크립트 보관
  - `Data/`: 시뮬레이션 결과 (`Data/S###/`)
- **`62_Empirical/`**: 실측 데이터
  - `Raw/`: 원본 센서 데이터 (**Read-Only, Git 추적 제외**)
  - `Processed/`: 1차 가공 데이터
- **`63_Analysis/`**: 통합 분석
  - `Scripts/`: 비교/검증 포스트프로세싱 코드, `Archive/`: 폐기 스크립트 보관
- **`64_Viz/`**: 자동 생성된 시각화 추출물 (Figure PNG 등)

#### `7_Paper/` — 논문 & 발표
- **`71_Figs/`**: 논문용 Figure
  - `Raw/` → `Processed/` → `Final/` (3단계 파이프라인)
- **`72_Drafts/`**: 원고 (Word, LaTeX), `Archive/`: 이전 버전 보관
- **`73_Presentations/`**: 발표 자료 (PPT, 포스터)

---

## 2. 운영 규칙

### 2.1 Raw Data 무결성
- `6_Exp/62_Empirical/Raw/`에 저장된 파일은 **읽기 전용(Read-Only)**입니다.
- 스크립트에서 읽기만 수행하며, 원본을 절대 덮어쓰지 않습니다.

### 2.2 Git 분리 전략
- **Git 추적 대상**: 코드, 메타데이터, 로그, 분석 Figure, 원고 등 프로젝트 산출물 전반
- **Git 추적 제외**: `6_Exp/62_Empirical/Raw/` (대용량 원본 센서 데이터), 도구 임시 파일
- 대용량 설계 파일(`3_HW/`)은 Git LFS 또는 별도 드라이브 관리를 권장합니다.
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
- 로그에서 Planning 문서 참조: `→ see 1_Concept/12_Planning/P001_xxx.md`
- 로그에서 시뮬레이션 데이터 참조: `→ see 6_Exp/61_Sim/Data/S###/`
- 로그에서 분석 스크립트 참조: `→ see 6_Exp/63_Analysis/Scripts/S###_analysis.m`

### 2.6 Data Reusability (데이터 영구 보존 원칙)
- 단순 Illustration(시각적 도해)을 제외한 모든 Plot/Graph 생성 시, 그래프에 표면적으로 드러나지 않는 메트릭이나 중간 연산 결과일지라도 **향후 재사용이 가능하도록 반드시 `.mat` 파일(또는 `.csv`) 형태로 원본 Data Array를 함께 저장(Export)**하는 것을 원칙으로 합니다.

---

## 3. AI Communication Rules

> **PROJECT_LANG**: `.elf/config.json` 의 `lang` 필드 참조 (프로젝트 생성 시 설정). AI 에이전트는 해당 값에 따라 응답 언어를 결정함.
> 본 EliRule은 프로젝트 무관 공통 규칙(ELF-managed)이므로 언어 값을 본문에 두지 않음 — 값은 `.elf/config.json`에 분리.

프로젝트 내 모든 AI Agent는 사용자와 소통하고 문서를 작성할 때 다음 원칙을 준수합니다:

1. **응답 언어 (Response Language)**: AI 에이전트는 `PROJECT_LANG`에 지정된 언어와 English 두 가지로 응답합니다. 로그, 문서 작성 시에도 동일하게 `PROJECT_LANG` 언어를 사용합니다. 기술 용어는 English 원문을 병기할 수 있습니다.
2. **객관적이고 드라이한 문체 유지**: 불필요한 인삿말, 과도한 칭찬, 주관적 감정 표현, 과장된 형용사 사용을 금지합니다.
3. **비유 금지**: 비유나 은유를 금지하고, 직관적이고 객관적인 학술/엔지니어링 용어로만 사실을 전달합니다.
4. **결론 중심의 명확한 전달**: 분석 결과와 Action Item을 간결하고 명확하게 제시하며, 논리적이고 정교한 엔지니어링 팩트만을 다룹니다.
5. **Data Reusability**: 위 2.6 항목을 엄격히 준수합니다.
6. **과장 및 감정적 수식어 금지 (No Embellishment)**: '압도적(Overwhelming)', '무기(Weapon)', '치명적', '파급력' 등 감정을 자극하거나 극단적인 수식어의 사용을 전면 배제합니다. 오직 정량적 수치와 물리적 인과관계로만 장단점을 서술합니다.
7. **이모지 사용 금지 (No Emojis)**: 아이콘이나 이모지(Emoji)를 어떠한 문서나 응답에도 사용하지 않습니다.
8. **토큰 최소화 및 압축 (Token Economy)**: AI 통신 및 로깅의 기본 철학은 '토큰의 최소화'입니다. 완전한 문장형 서술보다는 단어 중심의 압축된 개조식(Bullet points)을 사용하며, 불필요한 조사와 접속사는 생략합니다.
9. **문장 종결 방식 (Formatting)**: 한글 기록 시 '~입니다/습니다'나 해요체 등은 일절 배제하며, 서술이 필요한 경우에도 반드시 명사형 종결어미('-음', '-함', '-임')나 간결한 '-다'로 끝을 맺습니다.
10. **구조화된 로깅 (Structured Logging)**: 실험 관찰(관찰)과 물리적 분석(해석)을 명확히 분리하여 서술하며, 생성된 파일 목록이나 파라미터 조건 등은 나열식 서술 대신 반드시 마크다운 표(Table) 형식으로 압축하여 정보 밀도를 극대화합니다.

---

## 4. 외부 문헌 검색 (External Source Retrieval)

웹 검색 도구로 학술 문헌 조사 시 적용. high-tier(고-IF) 저널 우선 확보 목표.

- **배경**: 웹 검색 도구는 일반 검색 엔진 기반 (JCR/Scopus/WoS 아님) → Impact Factor 정렬·필터 미지원. OA 대량 발행 저널(MDPI·Frontiers·Hindawi 등)이 SEO·발행량으로 검색 listing 점령. high-IF flagship(Nature·Science·Cell·구독형 Elsevier/Wiley)은 paywall+anti-bot로 본문 fetch 차단(403). → high-tier 조사 의도 시 결과가 저-IF OA로 쏠림. 원인은 "엔진이 high-IF를 못 봄"이 아니라 "IF 기준 정렬 부재 + OA 색인 편향"이며 access 차단은 부분 요인.
- **규칙**:
  1. 도메인 화이트리스트: 검색 시 `0_Meta/highIFjournals.md` 의 도메인 목록을 `allowed_domains` 에 지정. 프로젝트별 우선 target subset은 `ProjectRule.md` 정의.
  2. OA full-text 경유: target 저널이 OA면 출판사 도메인 403 시 PMC·Europe PMC URL로 본문 확보.
  3. bibliometric API: 인용수·venue 정렬·품질 필터는 OpenAlex(`api.openalex.org`)·Semantic Scholar·Crossref 사용.
  4. 저널명 명시 쿼리: 쿼리에 저널명 토큰 추가하여 랭킹 상향.
  5. preprint 경유: high-IF 게재작 full text는 arXiv·bioRxiv·medRxiv.
  6. 저-IF OA 배제(선택): tier 강제 필요 시 검색 도구 도메인 배제 기능 사용. 단 정당 venue 동반 배제 주의 — 무조건 배제 금지.
  7. 인용 전 검증: "검색 노출 ≠ 고품질". venue+인용수 cross-check. 미확인 ref 날조 금지 (`AI_PARA_Framework.md` hallucination 방지 정합).
- **Reference**: 도메인 화이트리스트·OA 미러·bibliometric API 전체 목록 = `0_Meta/highIFjournals.md` (목록 갱신은 이 파일만 수정; rule 문서는 참조).
