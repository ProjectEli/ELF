[English](deprecate.md) | [한국어](deprecate.ko.md)

# 폐기

## `elf deprecate [대상] [단위] [옵션]`, `show <ID>`, `--restore <ID>`

폐기는 더 이상 유효하지 않은 내용을 작업 트리에서 지워, AI 에이전트가 그 내용을 읽고 판단에 쓰지 않게 합니다. 원문은 `.elf/deprecated/<ID>.md.xz`(표준 xz, git 추적)에 압축해 두므로 일반 읽기와 검색에는 드러나지 않고, 원래 자리에는 중립 표기 줄이 남습니다. 폐기 여부는 사용자가 정하고, 에이전트는 사용자의 지시나 승인을 받은 뒤 실행하며 보관 원문은 사용자가 요청할 때만 보여 줍니다(`AGENTS.md`). 산출물(스크립트, 데이터, 그림)은 그대로 둡니다.

| 대상, 단위 | 문서에 남는 것 | 보관 파일로 가는 것 |
|---|---|---|
| `S###` / `G###`(세션 로그 전체) | 7키 헤더와 제목 줄. `status: deprecated`, handoff에 사유와 대체, `conclusion`과 `user_note`는 비움 | 본문과 원래 status, handoff, conclusion, user_note |
| `P###` 또는 프로젝트 기준 경로(계획 문서 전체) | 제목 줄과 표기 줄 | 전문 |
| `--trial t03` | `## t03: (폐기 ID)`(번호 유지)와 표기 줄 | trial 제목과 본문 |
| `--trial t03 --section 해석` | 표기 줄 | 제목을 포함한 절 |
| `--marked` | 표기 줄 | `<!-- deprecate:begin -->` … `<!-- deprecate:end -->`로 감싼 블록 |
| `--lines A-B --expect "<A번 줄 내용>"` | 표기 줄 | 줄 범위(1부터). A번 줄이 다르면 거부 |

표기 줄: `> **Deprecated**: S012-D01 · 2026-11-09 · t03 · 사유: 측정 조건 오류 · 대체: S015 t02`. ID는 문서별 연번(`S012-D01`, `P007-D02`)이고, `--reason`과 `--replaced-by`가 마지막 두 칸을 채웁니다.

| 명령 | 하는 일 |
|---|---|
| `elf deprecate`(인수 없음) | 목록: 보관 항목마다 한 줄(ID, 날짜, 단위, 원래 위치, 사유, 대체)과 합계 |
| `elf deprecate show <ID>` | 보관 원문 출력(사용자가 요청할 때만) |
| `elf deprecate --restore <ID>` | 원문을 원래 자리에 되돌리고(세션 전체는 원래 헤더 값으로) 보관 파일을 지움. 되돌린 내용의 링크 대상이 없으면 경고 |
| `--dry-run` | 출력만 하고 쓰지 않음 |

- 문서 변경과 보관 파일은 함께 커밋합니다. `elf check logs`가 짝을 봅니다. 보관 파일 없는 표기 줄은 issue, 표기 줄 없는 보관 파일은 warning, git이 추적하지 않는 보관 파일은 info.
- 거부(exit 3, 아무것도 쓰지 않음): 대상 범위 밖의 경로(세션 로그와 계획 문서만), 이미 폐기한 문서, trial 제목이나 문서 제목이 들어간 블록, 짝이 맞지 않는 표시, `--expect` 불일치, 세션 로그에 새 trial 구조 문제가 생기는 변경(예: `관찰`이 남아 있는데 `가설` 절만 폐기), 이전 로그 형식이 남은 프로젝트. 대상이나 ID가 없으면 exit 1, 사용법 오류는 exit 2.
- 출력: 명령이 고치지 않는 항목(원래 로그 헤더의 handoff, 다른 문서의 `S012 t03` 같은 참조, 지운 내용 안의 그림 embed)은 `review:` 줄로 보여 주며, 사용자와 함께 처리합니다.
- 2.28 이전에는 폐기한 내용을 `Deprecated/` 폴더(`*.partial.md` 파일)로 옮기고 purge할 수 있었습니다. `elf update`가 그 기록을 한 번 보관 파일과 표기 줄로 바꿉니다([이관](projects.ko.md#로그-구조-이관-228)). 그 전에 purge한 기록은 끝에 `복원 불가`가 붙은 표기 줄만 남고 보관 파일은 없습니다.

```bash
elf deprecate S012 --trial t03 --replaced-by "S015 t02" --reason "측정 조건 오류"
# → deprecated S012-D01 (t03) → .elf/deprecated/S012-D01.md.xz
# → review: handoff: 2_Log/S/S012_log.md header mentions t03
elf deprecate S014 --reason "설정 오류"            # 세션 전체: 헤더 status deprecated
elf deprecate P007 --marked --reason "범위 축소"    # 계획 문서의 표시 블록
elf deprecate show S012-D01                        # 사용자가 요청할 때만
elf deprecate --restore S012-D01                   # trial 복원
elf deprecate                                      # 목록
```

---

[README](../README.ko.md)로 돌아가기
