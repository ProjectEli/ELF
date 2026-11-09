[English](layout.md) | [한국어](layout.ko.md)

# 프로젝트 구조

- 파일 이름에는 번호만 씁니다. 예: `S001_t02_sweep.m`, `Data/S001/S001_t02_results.mat`, `64_Viz/S001/S001_t02_SNR_ci.png`. 조건은 로그에 적습니다.
- 로그 안의 링크는 `2_Log/S/`(또는 `2_Log/G/`) 기준 상대 경로로 씁니다. 예: 계획 `../../1_Concept/12_Planning/P00x.md`, 데이터 `../../6_Exp/61_Sim/Data/S###/`, figure `../../6_Exp/64_Viz/S###/`. 로그는 옮기지 않으므로 경로가 그대로 유지됩니다.
- 계획은 `1_Concept/12_Planning/P###_제목.md`, 아이디어는 `13_Ideas/`에 두고, 로그에는 사실만 적습니다. 분석 코드는 `6_Exp/63_Analysis/Scripts/`, `61_Sim/Scripts/`에 둡니다(`.m`은 `%%` 셀 모드).

전체 폴더 구조:

```text
Project_Root/
│
├── AGENTS.md                        # 에이전트 진입 규칙 요약 (모든 에이전트 공통, ELF 관리)
├── CLAUDE.md, .claude/settings.json   # Claude Code 전용 (포인터, autoread 훅). 다른 에이전트는 무시
├── README.md, LICENSE, .gitignore, .editorconfig, .gitattributes
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF 제어 영역 (config, 프로젝트 manifest. 직접 수정 금지)
│   ├── managed/                     # 관리 규칙: EliRule, LogConvention, AI_PARA_Framework,
│   │   └── templates/               #   LLMcliche, highIFjournals, 세션과 trial 틀
│   ├── deprecated/                  # 폐기한 내용의 압축 원문 (elf deprecate, git 추적)
│   └── runtime/                     # 도구 임시 상태 (git 추적 밖)
├── 0_Meta/                          # 프로젝트 거버넌스. 사용자 영역 (`elf update`가 건드리지 않음)
│   ├── ProjectRule.md               # 프로젝트 전용 규칙 및 목표
│   ├── <이름>.project.md            # 프로젝트별 추가 규칙 (실제 규칙 = 기본 규칙 ⊕ 추가 규칙)
│   └── tsa/                         # (선택) elf tsa manifest, 타임스탬프
│
├── 1_Concept/                       # 연구 기획, 문헌, 아이디어
│   ├── 11_Literature/               # 논문 PDF, 서지 정보, 기반 공식
│   ├── 12_Planning/                 # 연구 기획, 로드맵 (다중 세션)
│   │   ├── Wiki/                    # 기획 단계 결론 및 핵심 규칙 요약
│   │   └── Archive/                 # 종료된 계획 문서
│   └── 13_Ideas/                    # 짧은 메모, 초기 아이디어 (하위 폴더 없음)
│
├── 2_Log/                           # 세션 로그 (상태는 헤더에, 로그는 옮기지 않음)
│   ├── S/                         # 연구 세션 S###_log.md + Registry.md (생성 파일)
│   ├── G/                         # 거버넌스 세션 G###_log.md + Registry.md (생성 파일)
│   └── Wiki/                      # 발견 요약 주제 문서
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
├── 5_SW/                            # 소프트웨어, 펌웨어
│   ├── 51_FW/                       # MCU/임베디드 펌웨어
│   ├── 52_DAQ/                      # PC/모바일 데이터 획득 시스템
│   └── 53_Libs/                     # 재사용 가능한 공용 라이브러리
│
├── 6_Exp/                           # 실험: 시뮬레이션 + 실측 + 분석
│   ├── 61_Sim/                      # 시뮬레이션
│   │   ├── Scripts/                 # 시뮬레이션 코드 (S###_sim.m)
│   │   │   └── Archive/          # 일회용 스크립트 보관
│   │   └── Data/                    # 시뮬레이션 결과 (Data/S###/)
│   ├── 62_Empirical/                # 실측 데이터
│   │   ├── Raw/                     # 원본 센서 데이터 (Read-Only, Git 제외)
│   │   └── Processed/               # 1차 가공 데이터
│   ├── 63_Analysis/                 # 통합 분석
│   │   └── Scripts/                 # 비교, 검증용 후처리 코드
│   │       └── Archive/           # 일회용 스크립트 보관
│   └── 64_Viz/                      # 시각화 추출물 (자동 생성 Figure)
│
├── 7_Paper/                         # 논문, 발표
│   ├── 71_Figs/                     # 논문용 Figure
│   │   ├── Raw/
│   │   ├── Processed/
│   │   └── Final/
│   ├── 72_Drafts/                   # 원고 (Word, LaTeX)
│   │   └── Archive/               # 이전 버전 보관
│   └── 73_Presentations/            # 발표 자료 (PPT, 포스터)
```

각 폴더의 용도와 운영 규칙은 `.elf/managed/EliRule.md`에 있습니다.

---

[README](../README.ko.md)로 돌아가기
