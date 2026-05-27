# ProjectRule: ELF (Eli's Lab Framework)

전역 규칙(`0_Meta/EliRule.md`)을 기반으로, 이 프로젝트에만 적용되는 맥락·환경·오버라이드를 정의합니다.
AI 에이전트는 EliRule.md와 함께 이 문서를 읽어 프로젝트 컨텍스트를 파악합니다.

---

## 1. 프로젝트 개요

| 항목 | 내용 |
|------|------|
| **연구 목표** | ELF v2 프로젝트 구조 템플릿 유지보수 및 개선 |
| **연구 기간** | 2026-01-01 ~ |
| **담당 연구자** | ProjectEli |
| **현재 단계** | 템플릿 구조 정립 및 generator 개선 |

---

## 2. 핵심 도메인 & 용어 정의

| 용어 / 약어 | 정의 |
|-------------|------|
| ELF | Eli's Lab Framework. R&D 프로젝트 구조 표준 템플릿 |
| Generator | `0_Meta/ELF_generator.ps1` (Windows) 또는 `ELF_generator.sh` (Linux/macOS). 새 ELF 프로젝트를 자동 생성하는 스크립트 |
| Session | `S###_log.md` 단위의 실험/작업 기록 묶음 |
| Trial | 세션 내 개별 작업 단위 (`t01`, `t02`, ...) |
| PARA | Projects / Areas / Resources / Archives. 파일 관리 체계 |

---

## 3. 실험 환경 (Baseline)

### 하드웨어
- 해당 없음 (소프트웨어/문서 프로젝트)

### 소프트웨어
- **편집기**: VS Code (Markdown 미리보기 + outline 활용)
- **버전 관리**: Git (main 브랜치 단일 운용)
- **스크립트 환경**: PowerShell 5.1+ (Windows), Bash (Linux/macOS)

### 운용 조건
- 해당 없음

---

## 4. 데이터 파이프라인 특이사항

> EliRule.md 기본 파이프라인과 다른 점만 기재합니다.

- 이 프로젝트는 연구 데이터가 없는 메타 프로젝트임. `6_Exp/` 폴더는 generator 출력 예시 용도로만 사용.
- `2_Log/S001_log.md`: generator가 생성하는 세션 로그의 예시 파일로 유지.

---

## 5. 네이밍 오버라이드

> EliRule.md 기본값과 다른 점만 기재합니다.

- (없음)

---

## 6. 폴더 사용 계획

| 구분 | 폴더 | 사용 여부 | 비고 |
|------|------|-----------|------|
| Core | `0_Meta/` | ✅ | 핵심. generator, 규칙 문서 |
| Core | `1_Concept/` | ✅ | Planning 문서 |
| Core | `2_Log/` | ✅ | 세션 로그 (S001_log.md 예시 파일) |
| — | `templates/` | ✅ | meta, log, root, scripts 템플릿 |
| Module | 3~7 | 미포함 | 프레임워크 repo이므로 Module 폴더 불포함. Generator가 생성 |

---

## 7. 진행 현황 *(수시 업데이트)*

- **현재 활성 세션**: 없음 (유지보수 작업 단위로 진행)
- **완료된 주요 마일스톤**:
  - ELF v2 디렉토리 구조 확정 (0~6 계층)
  - generator PS1/SH 이중화
  - templates 폴더 분리 (Single Source of Truth)
  - 다국어 README → 영어/한국어 축소
- **다음 목표**: generator 동작 검증
- **현재 주요 미해결 이슈**: (없음)

---

## 8. AI 에이전트 추가 지시사항

> EliRule.md Section 3에 더해, 이 프로젝트 한정 지시사항입니다.

- 이 프로젝트는 **ELF 자체가 대상**이므로, `0_Meta/` 내 문서를 수정할 때 generator(PS1/SH)와 templates 폴더의 **동기화 여부를 항상 확인**할 것.
- generator 내 인라인 콘텐츠는 제거되었음. 템플릿 내용 변경은 `templates/*.md`만 수정하면 됨.
