[English](README.md) | [한국어](README.ko.md) | [日本語](docs/i18n/README.ja.md) | [中文简体](docs/i18n/README.zh-CN.md) | [中文繁體](docs/i18n/README.zh-TW.md) | [Français](docs/i18n/README.fr.md) | [Deutsch](docs/i18n/README.de.md) | [Español](docs/i18n/README.es.md) | [Italiano](docs/i18n/README.it.md) | [Português](docs/i18n/README.pt-BR.md) | [Русский](docs/i18n/README.ru.md) | [العربية](docs/i18n/README.ar.md) | [हिन्दी](docs/i18n/README.hi.md) | [Türkçe](docs/i18n/README.tr.md) | [Tiếng Việt](docs/i18n/README.vi.md) | [ภาษาไทย](docs/i18n/README.th.md) | [Nederlands](docs/i18n/README.nl.md) | [Polski](docs/i18n/README.pl.md) | [Bahasa Indonesia](docs/i18n/README.id.md)

# Eli's Lab Framework (ELF): Base-Delta Protocol for Agile R&D

기기 개발 및 R&D 검증 단계의 빠른 피드백 루프(Agile)를 지원하기 위한 하드웨어-소프트웨어-실험 데이터 통합 기록 규격(Protocol)임. 연구자의 기록 피로도를 최소화하면서도 완벽한 데이터 추적성을 보장함.

## 핵심 철학 (Philosophy)

* **단일 진실 공급원 (Single Source of Truth):** 하드웨어 설계, 분석 코드, 원시 데이터를 하나의 프로젝트 내에서 유기적으로 연결함.
* **Base-Delta 로깅:** 모든 변수를 기록하지 않음. 기준(Baseline)을 선언하고, 변경된 변수(Delta)만 가볍게 기록하여 연구 지연을 방지함.
* **시스템적 강제성:** 파일명 길이 제한(Windows 260자)을 우회하고, 코드를 통한 재현성을 보장함.
* **AI 거버넌스:** AI 에이전트의 작업 연속성을 `0_Meta/AI_Sync.md` 핸드오프 로그로 보장하고, `0_Meta/LogConvention.md`로 사람과 AI 모두 동일한 로깅 규격을 따르도록 강제함.

## 프로젝트 디렉토리 규격 (Directory Structure)

본 프로젝트는 아래의 폴더 계층 구조 자체를 하나의 통신 규격으로 간주함.

```text
Project_Root/
├── 0_Meta/                          # 프로젝트 거버넌스 & 규칙
│   ├── EliRule.md                   # 폴더 구조 및 운영 가이드
│   ├── LogConvention.md             # 로깅 표준 규칙
│   ├── AI_PARA_Framework.md         # AI 컨텍스트 관리 & 아카이빙 규칙
│   └── AI_Sync.md                   # AI 에이전트 핸드오프 로그
│
├── 1_Concept/                       # 연구 기획, 문헌, 아이디어
│   ├── 11_Ideas/                    # 러프 스케치, 가설 제안록
│   ├── 12_Literature/               # 논문 PDF, 서지 정보, 기반 공식
│   └── 13_Planning/                 # 연구 로드맵, Figure 구성 스토리보드
│
├── 2_HW/                            # 하드웨어 설계
│   ├── 21_Component/                # 개별 부품 사양서, 단위 소자 설계
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # 통합 기기 설계, 하우징, 3D 모델
│   └── 23_Elec/                     # PCB 회로도, Gerber, BOM, Datasheets
│
├── 3_Fab/                           # 제작 및 공정
│   ├── 31_Recipes/                  # 공정 조건 문서화
│   └── 32_Eval/                     # 모듈별 단일 특성 평가
│
├── 4_SW/                            # 소프트웨어 & 펌웨어
│   ├── 41_FW/                       # MCU/임베디드 펌웨어
│   ├── 42_DAQ/                      # PC/모바일 데이터 획득 시스템
│   └── 43_Libs/                     # 재사용 가능한 공용 라이브러리
│
├── 5_Exp/                           # 실험: 시뮬레이션 + 실측 + 분석
│   ├── 51_Sim/                      # 시뮬레이션
│   │   ├── Scripts/                 # 시뮬레이션 코드 (S###_sim.m)
│   │   └── Data/                    # 시뮬레이션 결과 (Data/S###/)
│   ├── 52_Empirical/                # 실측 데이터
│   │   ├── Raw/                     # 원본 센서 데이터 (Read-Only, Git 제외)
│   │   └── Processed/               # 1차 가공 데이터
│   ├── 53_Analysis/                 # 통합 분석
│   │   ├── Scripts/                 # 비교/검증 포스트프로세싱 코드
│   │   └── Logs/                    # 세션 로그 (S###_log.md)
│   └── 54_Viz/                      # 시각화 추출물 (자동 생성 Figure)
│
└── 6_Paper/                         # 논문 & 발표
    ├── 61_Figs/                     # 논문용 Figure
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # 원고 (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # 발표 자료 (PPT, 포스터)
```

> 각 폴더의 상세 용도와 운영 규칙은 `0_Meta/EliRule.md`를 참조.

## 데이터 로깅 파이프라인 규격

### 1. 파일 명명 규칙 (Session-Trial Naming)

* 파일 이름에 실험 조건이나 변수 정보를 나열하는 것을 **엄격히 금지**함.
* **형식:** `[세션ID]_[트라이얼ID].[확장자]` (예: `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta 로깅 (Hybrid Logging)

* **러닝 로그 (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * 즉각적인 가설-테스트-교훈을 텍스트로 기록하는 서사형 마크다운 파일임.
  * 트라이얼(`t1`, `t2`...) 단위로 **의도적으로 변경한 변수(Delta)**와 관찰된 결과만 의식의 흐름대로 작성함.
  * 포맷 및 상세 규칙: `0_Meta/LogConvention.md` 참조.

### 3. Planning 문서 규칙

* 연구 로드맵, Figure 구성, 실험 전략 등은 `1_Concept/13_Planning/`에 별도 관리.
* **형식:** `P###_제목.md` (예: `P001_wavelength_optimization.md`)
* 로그에서 Planning 참조 시: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. 후처리 분석 규격 (Cell Mode Scripting)

* 분석 코드는 `5_Exp/53_Analysis/Scripts/` 또는 `5_Exp/51_Sim/Scripts/`에 위치해야 하며, 데이터 폴더 내부에 혼재 불가함.
* 벤더 종속성(Vendor Lock-in) 방지를 위해 `.mlx` 대신 순수 `.m` 파일을 사용함.
* 코드 내 `%%` (Cell Mode)를 활용해 구역별로 실행하며, 도출된 인사이트는 러닝 로그에 반영함.
* 분석 결과물(그림, mat파일)은 `5_Exp/54_Viz/` 또는 `5_Exp/52_Empirical/Processed/S###/`에 세션별 폴더를 생성하여 저장함.

### 5. Cross-Reference 규칙

프로젝트 내 문서 간 추적성을 확보하기 위해 상호 참조 형식을 통일함.

| From → To | 형식 |
|-----------|------|
| Logs → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ see 5_Exp/51_Sim/Data/S###/` |
| Logs → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI 거버넌스

AI 에이전트(Claude, Gemini 등)가 프로젝트에 참여할 때 다음 규칙을 따름:

1. **컨텍스트 파악**: 작업 시작 전 `0_Meta/AI_Sync.md`를 읽어 이전 작업 상태 확인.
2. **동일 규격 준수**: `0_Meta/LogConvention.md`의 로깅 규칙을 사람과 동일하게 따름.
3. **핸드오프 기록**: 작업 완료 시 `0_Meta/AI_Sync.md`에 수행 내역, 생성/수정 파일, Next Steps 기록. 최신 항목이 위에 오도록 역순 작성.
4. **아이디어 분리**: AI가 생성한 가설/아이디어는 로그가 아닌 `1_Concept/11_Ideas/`에 별도 저장.
5. **PARA 기반 컨텍스트 관리**: `9_Archive/` 폴더와 `.claudeignore`를 활용하여 AI의 컨텍스트 오염을 방지. 상세 규칙은 `0_Meta/AI_PARA_Framework.md` 참조.
6. **Communication Rules**: 객관적이고 드라이한 문체 유지. 비유/은유 금지. 결론 중심의 명확한 전달. 과장 및 감정적 수식어 금지. 상세 규칙은 `0_Meta/EliRule.md` 섹션 3 참조.
7. **Data Reusability**: 모든 Plot/Graph 생성 시 원본 Data Array를 `.mat`/`.csv`로 함께 저장. 상세 규칙은 `0_Meta/EliRule.md` 섹션 2.6 참조.

## Quick Start

새 프로젝트를 ELF v2 구조로 생성하려면 `0_Meta/ELF_generator.sh`를 실행합니다.

```bash
cd 원하는_상위_디렉토리
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> Windows에서는 Git Bash([Git for Windows](https://git-scm.com/) 포함)를 사용합니다.

프로젝트 이름을 입력하면 0~6 폴더 체계, 메타 문서, `.gitignore`가 자동 생성됩니다. Git 초기화는 Git이 설치된 경우에만 선택적으로 수행됩니다.

## 라이선스 (License)

이 프로젝트는 '구동 코드'와 '데이터 구조 규격(Protocol)'의 성격이 다르므로, 이중 라이선스(Dual License) 정책을 적용함.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **적용 대상:** `4_SW/`, `5_Exp/*/Scripts/` 폴더 내의 모든 소스 코드(`.m`, `.py` 등).
  * **조건:** 템플릿의 코어 스크립트를 수정 및 개선하여 배포할 경우 해당 수정본은 오픈소스로 공개해야 함. 단, 사용자가 프로젝트 내에 추가한 고유 알고리즘이나 원시 데이터는 비공개(상업화) 유지가 가능함.

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **적용 대상:** `README.md`, `0_Meta/` 문서, Session-Trial 폴더 계층 구조, Base-Delta 메타데이터 로깅 규칙 등 연구 방법론 전반.
  * **조건:** 누구나 이 구조와 기록 방법론을 자유롭게 차용 및 변형할 수 있으나, 파생된 템플릿이나 관련 연구 결과물 발표 시 원작자 Eli (projectschnee@gmail.com) 와 본 저장소의 출처를 명시해야 함.
