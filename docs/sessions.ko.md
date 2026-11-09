[English](sessions.md) | [한국어](sessions.ko.md)

# 세션과 trial

## 로그 위치

| 폴더 | 담는 것 |
|------|---------|
| `2_Log/S/` | 연구 세션 `S###_log.md`와 생성 파일 `Registry.md` |
| `2_Log/G/` | 거버넌스 세션 `G###_log.md`(이후 작업 전체에 적용되는 규칙, 구조, 템플릿, 도구 설정 변경)와 `Registry.md` |

로그는 만든 자리에 그대로 있습니다. 세션을 종료하거나 폐기해도 폴더가 아니라 헤더의 `status`가 바뀌므로 로그로 가는 링크가 깨지지 않습니다. `Registry.md`는 그 폴더의 세션 목록(진행 중 먼저, 이어서 번호 내림차순)과 상태, 결론 또는 handoff를 보여 줍니다. 로그를 바꾸는 `elf` 명령이 매번 다시 만드므로 직접 고치지 않습니다. VS Code 탐색기에서는 종료, 폐기된 로그가 `Registry.md` 아래로 접힙니다(`.vscode/settings.json` 파일 중첩 설정, `elf`가 관리).

## 헤더

로그는 7개 키의 YAML 헤더로 시작하고, 키 순서는 항상 같습니다.

```yaml
---
created: 2026-11-09
modified: 2026-11-09
status: ongoing
goal: '파장별 SNR 비교'
handoff: ''
conclusion: ''
user_note: ''
---
# S001: 파장 비교
```

`status`는 `ongoing`, `complete`, `deprecated` 중 하나입니다. 자유 텍스트는 작은따옴표로 감싸고, 안의 따옴표는 `''`로 씁니다. 헤더는 `elf` 명령만 씁니다. `session new`가 goal을, `--from` 파일 맨 앞의 YAML 블록이 handoff를, `session close`가 결론과 사용자 메모를 씁니다. 형식이 어긋난 헤더는 `elf check logs`가 알려 줍니다.

## `elf session new "<제목>" --goal "<목표>" [--gov]`

다음 번호의 세션 로그를 헤더, 제목 줄, 규칙 주석과 함께 만듭니다. `--goal`은 필수입니다(1~2문장). `--gov`는 `2_Log/G/`에 거버넌스 세션을 만듭니다. 번호는 폴더마다 따로 셉니다(S001…, G001…). `elf init`은 세션을 만들지 않으므로 첫 세션도 이 명령으로 만듭니다.

```bash
elf session new "파장 비교" --goal "파장별 SNR 비교"
# → created 2_Log/S/S001_log.md (S001) + Registry.md
```

## trial: `elf trial new`와 `elf trial write`

trial은 두 단계로 씁니다. 실행 전 Phase 1(목표, 조건, 가설, 예상), 실행 후 Phase 2(관찰, 해석, 교훈, 생성 파일)입니다. 절은 편집기나 AI 에이전트의 파일 쓰기 도구로 파일에 쓰고 `--from`으로 넘기므로 셸을 거치지 않습니다.

| 명령 | 하는 일 |
|------|---------|
| `elf trial new "<제목>"` | 다음 trial(`t01`, `t02`, …)을 Phase 1 틀과 함께 만듦 |
| `elf trial new "<제목>" --from <파일>` | trial을 만들고 파일의 Phase 1을 씀(`new` 다음 `write`와 같음) |
| `elf trial write [tNN] --from <파일>` | 파일의 절을 trial에 씀. Phase 1 절은 지금 내용을 바꿈(Phase 2가 생기기 전까지 여러 번), Phase 2 절은 한 번. `tNN`을 생략하면 그 세션의 마지막 trial |

진행 중인 세션이 여럿이면 `--session <S###|G###>`로 고릅니다.

```bash
elf trial new "3파장 스윕" --from phase1.md   # t01, Phase 1
elf trial write --from phase1.md             # 사용자 의견을 반영해 Phase 1 수정
elf trial write t01 --from phase2.md         # 실행 후 Phase 2
```

파일 하나에는 한 Phase의 절만 넣습니다. 파일 맨 앞의 YAML 블록(`handoff`만)은 헤더 handoff를 바꿉니다. 절 이름의 정확한 동의어(예: `### Goal`)는 고친 뒤 알려 줍니다. 쓰기 전에 모두 검사하고, 문제가 있으면 아무것도 쓰지 않고 이유를 모두 출력한 뒤 exit 3으로 끝납니다.

- 빠지거나 순서가 어긋난 절, 두 Phase의 절이 섞인 파일, 틀 문구가 남은 절
- 그 세션의 산출물이 `6_Exp/`에 생긴 뒤의 Phase 1 수정(결과를 반영한 변경은 새 trial), Phase 2가 있는 trial의 Phase 1 수정
- 마지막 Phase 1 쓰기 뒤 `6_Exp/64_Viz/S###/`에 생긴 그림이 본문에 없는 Phase 2(관찰에 넣거나 `<!-- noembed: name.png -->`로 표시)

도구는 trial 제목 아래 주석에 Phase 1 쓰기와 Phase 2의 시각을 남깁니다. Phase 1을 쓴 뒤에는 멈추고 사용자 확인을 받은 다음 실행합니다. 명령이 확인을 대신 묻지는 않습니다.

## `elf session close [S###|G###] --from <파일>`

사용자가 결론을 승인한 뒤 세션을 닫습니다. 파일에는 YAML 블록을 씁니다.

```yaml
---
conclusion: '810 nm에서 SNR 최대'
user_note: ''
handoff: ''
---
```

`conclusion`은 필수이고, 없으면 아무것도 쓰지 않습니다(exit 3). `user_note`에는 사용자의 말만 씁니다. 상태가 `complete`가 되고 `Registry.md`를 다시 만들며, 파일은 옮기지 않습니다. 종료할 때 이 세션 범위의 `elf check logs` 결과(handoff에 남은 미완료 포함)를 알려 주지만 종료를 막지는 않습니다. 세션 번호를 생략하면 진행 중인 세션이 하나일 때 그 세션을 닫습니다.

## `elf log`와 `elf log show`

| 명령 | 보여 주는 것 |
|------|--------------|
| `elf log` | 세션마다 한 줄: 번호, 상태, 마지막 변경일, 제목, 결론 또는 handoff |
| `elf log show S012` | 헤더와 trial 목록(Phase 진행 상태) |
| `elf log show S012 t03` | trial 하나의 본문 |

둘 다 읽기만 합니다.

## `elf gallery`

`6_Exp/64_Viz/`를 훑어 `6_Exp/64_Viz/_gallery.md`(세션 폴더별 figure 색인)를 다시 만듭니다. `6_Exp/64_Viz/`가 없으면 안내만 하고 exit 0으로 끝납니다.

## 2.28 이전에 만든 프로젝트

`2_Log/`, `2_Log/Archive/`, `2_Log/Deprecated/`에 있는 인용 블록 헤더 로그와 `2_Log/Wiki/Session_Registry.tsv`는 이전 형식입니다. CLI를 갱신해도 이런 프로젝트는 바뀌지 않습니다. 3.0부터는 PATH의 `elf`가 이런 프로젝트를 elf 2.27.x로 실행하므로(처음 쓸 때 받음) 로그 명령이 2.27과 같습니다(`--goal` 없는 `elf session new "<제목>"`, 로그를 `Archive/`로 옮기는 `elf session close`, 첫 줄 `> **Handoff**:`를 쓰는 `elf trial new --from`, 내용을 `Deprecated/`로 옮기는 `elf deprecate`). elf 2.27.x를 받을 수 없으면 내장 호환 모드가 같은 명령을 실행하고, 기록할 때마다 안내 한 줄이 나옵니다. 새 구조의 명령(`elf trial write`, `elf log`, `elf deprecate show`, `--gov`, `session close --from`)은 `elf update`로 이관한 뒤 쓸 수 있습니다. [프로젝트 생성과 갱신](projects.ko.md#로그-구조-이관-228)을 참고하세요.

---

[README](../README.ko.md)로 돌아가기
