# Eli's Lab Framework (ELF): Base-Delta Protocol for Agile R&D biolerplate

기기 개발 및 R&D 검증 단계의 빠른 피드백 루프(Agile)를 지원하기 위한 하드웨어-소프트웨어-실험 데이터 통합 기록 규격(Protocol)임. 연구자의 기록 피로도를 최소화하면서도 완벽한 데이터 추적성을 보장함.

## 📌 핵심 철학 (Philosophy)
* **단일 진실 공급원:** 하드웨어 설계, 분석 코드, 원시 데이터를 하나의 프로젝트 내에서 유기적으로 연결함.
* **Base-Delta 로깅:** 모든 변수를 기록하지 않음. 기준(Baseline)을 선언하고, 변경된 변수(Delta)만 가볍게 기록하여 연구 지연을 방지함.
* **시스템적 강제성:** 파일명 길이 제한(Windows 260자)을 우회하고, 코드를 통한 재현성을 보장함.

## 📂 프로젝트 디렉토리 규격 (Directory Structure)

본 프로젝트는 아래의 폴더 계층 구조 자체를 하나의 통신 규격으로 간주함.

```text
Project_Root/
├── .gitignore               # /rawData/, /processedData/ 등 대용량 데이터 및 임시 파일 배제
├── .gitattributes           # 바이너리 파일 손상 방지 규칙 명시
├── README.md                # 템플릿 철학 및 규격 선언문
├── 1_Hardware/              # 기구(CAD)/회로(PCB) 스펙 및 초기 Calibration 설정 보관
├── 2_Software/              # 범용 코어 알고리즘 및 로직 보관
│   └── src/                 # 재사용 가능한 Core 함수 (예: 필터링, SNR 계산)
├── 3_Experiments/           # 데이터 수집 및 후처리 파이프라인
│   ├── metadata/            # Master_Log.xlsx (세션 요약) 및 S001_log.md (러닝 로그)
│   ├── rawData/             # 센서 원시 데이터 (절대 수정 금지, Git 추적 제외)
│   ├── processedData/       # 분석 완료된 캐시 데이터 (Git 추적 제외)
│   └── scripts/             # 세션별 구동용 Cell Mode 분석 스크립트 (.m)
└── 4_Manuscript/            # 논문 작성용 원고(.tex) 및 시각화 리포트

```

## 📝 데이터 로깅 파이프라인 규격

### 1. 파일 명명 규칙 (Session-Trial Naming)

* 파일 이름에 실험 조건이나 변수 정보를 나열하는 것을 엄격히 금지함.
* **형식:** `[세션ID]_[트라이얼ID].[확장자]` (예: `S001_t1.csv`, `S001_t2.bin`)

### 2. 하이브리드 로깅 (Base-Delta 기록법)

* **Master Excel (`metadata/Experiment_Master_Log.xlsx`):** * 개별 테스트가 아닌 **세션(Session) 단위**로 한 줄만 기록함.
* 기록 항목: 날짜, 세션ID(`S001`), 공통 통제 변인(Baseline 하드웨어/소프트웨어 버전), 실험 목적.


* **러닝 로그 (`metadata/S001_log.md`):** * 즉각적인 가설-테스트-교훈을 텍스트로 기록하는 서사형 마크다운 파일임.
* 트라이얼(`t1`, `t2`...) 단위로 **의도적으로 변경한 변수(Delta)**와 관찰된 결과만 의식의 흐름대로 작성함.



### 3. 후처리 분석 규격 (Cell Mode Scripting)

* 분석 코드는 반드시 `3_Experiments/scripts/` 내에 위치해야 하며, 데이터 폴더(`rawData`) 내부에 혼재 불가함.
* 벤더 종속성(Vendor Lock-in) 방지 및 영구적 텍스트 추적을 위해 `.mlx` 대신 순수 `.m` 파일을 사용함.
* 코드 내 `%%` (Cell Mode)를 활용해 구역별로 실행하며, 도출된 인사이트는 러닝 로그에 반영함.

## ⚖️ License / 라이선스

이 프로젝트는 '구동 코드'와 '데이터 구조 규격(Protocol)'의 성격이 다르므로, 이중 라이선스(Dual License) 정책을 적용함.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
* **적용 대상:** `scripts/` 및 `2_Software/` 폴더 내의 분석/검증을 위한 모든 뼈대 소스 코드(`.m`, `.py` 등).
* **조건:** 템플릿의 코어 스크립트를 수정 및 개선하여 배포할 경우 해당 수정본은 오픈소스로 공개해야 함. 단, 사용자가 프로젝트 내에 추가한 고유 알고리즘이나 원시 데이터는 비공개(상업화) 유지가 가능함.


* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
* **적용 대상:** `README.md`, Session-Trial 폴더 계층 구조, Base-Delta 메타데이터 로깅 규칙 등 연구 방법론 전반.
* **조건:** 누구나 이 구조와 기록 방법론을 자유롭게 차용 및 변형할 수 있으나, 파생된 템플릿이나 관련 연구 결과물 발표 시 원작자 Eli (projectschnee@gmail.com) 와 본 저장소의 출처를 명시해야 함.