[English](autoread.md) | [한국어](autoread.ko.md)

# 대화가 다시 구성된 뒤 규칙 다시 읽기(autoread)

## `elf autoread [서브명령]`

AI 에이전트의 대화가 압축(compaction)되거나, 재개되거나, 초기화되면 에이전트는 내용이 빠진 압축 요약을 보고 작업을 이어 가고, 프로젝트 규칙은 컨텍스트에서 빠집니다. `elf autoread`는 이를 바로잡는 요약(digest)을 출력합니다. 요약에는 `AGENTS.md`의 상시 의무 절, 로그 형식 규칙(`.elf/managed/LogConvention.md`)의 §0 핵심 요약, 진행 중인 세션의 handoff(앞부분만, 거버넌스 세션 먼저), 현재 `elf check logs` 집계, 압축 요약 대신 규칙 원문을 따르라는 마지막 지시가 들어갑니다. 새 세션을 시작할 때도 같은 요약을 받습니다. 서브명령 없이 실행하면 요약을 표준 출력으로 내보내므로 어떤 에이전트에서도 쓸 수 있습니다(다음 프롬프트에 붙여 넣거나 파이프로 전달).

**Claude Code 연동(자동).** `elf init`, `elf update`가 `.claude/settings.json`에 훅 항목 2개를 유지합니다. 이 파일은 git으로 추적하지 않고, 병합할 때 다른 설정은 그대로 둡니다. `SessionStart` 훅은 새 세션 시작과 대화 재구성(압축, 재개, 초기화) 때 `.elf/runtime/`에 세션별 마커만 기록하고, `UserPromptSubmit` 훅이 다음 프롬프트에 요약을 넣습니다. 이전 버전이 설치한 훅은 `elf update`가 새 정의로 바꾸고, 바뀌기 전에는 `elf check setup`이 경고합니다. 그 프롬프트가 오기 전까지는 그 세션에서 실행한 모든 `elf` 명령이 1줄 알림을 출력합니다(대체 경로). 세션은 Claude Code가 Bash 명령과 훅에 설정하는 `CLAUDE_CODE_SESSION_ID`로 구분하므로, 같은 프로젝트의 다른 세션에는 영향이 없습니다. 훅은 어떤 경우에도 세션을 막지 않습니다. 내부 오류가 나면 아무것도 출력하지 않고 exit 0으로 끝납니다. `.gitignore`의 관리 블록이 `.elf/runtime/`을 제외합니다. 루트 `CLAUDE.md`가 `AGENTS.md`를 import하면 Claude Code가 대화가 다시 구성된 뒤 그것을 다시 읽으므로, 요약에는 상시 의무 절 대신 1줄 안내만 넣습니다. 그 밖의 경우(`CLAUDE.md`가 없는 프로젝트 포함)에는 그 절을 전부 넣습니다.

| 서브명령 | 효과 |
|---|---|
| *(없음)* | 요약 출력(모든 에이전트). 현재 Claude Code 세션의 대기 마커 해제(Claude Code 밖에서는 전체 해제) |
| `enable` | `.elf/config.json`에 `"autoread": true`를 쓰고 Claude Code 훅 항목을 설치. 여러 번 실행해도 결과가 같고, 프로젝트 단위로 적용. 키가 없으면 켜진 것으로 봄 |
| `disable` | `"autoread": false`. 훅 항목은 남지만 아무 일도 하지 않음 |
| `status` | 설정, 훅 설치 상태, `autoread_fulltext` 선언(ok, missing, unsafe), 대기 마커를 보여 줌(읽기만 함) |

**`autoread_fulltext`(선택).** `.elf/config.json`의 `autoread_fulltext` 배열에 루트 기준 상대 경로(예: 로그 형식 규칙)를 적으면, 대화가 다시 구성된 뒤 요약에 그 파일들의 원문이 함께 들어갑니다. 프로젝트 폴더 밖을 가리키는 경로는 거부합니다. 파일이 없거나 안전하지 않은 항목은 1줄 안내로 대신합니다. 요약 전체는 10,000자를 넘지 않게 합니다. Claude Code는 훅 출력이 10,000자를 넘으면 앞 2,000자만 에이전트에게 전달하기 때문입니다. 그래서 선언한 파일은 남은 자리만큼만 넣고, 넘으면 안내와 함께 자르거나 뺍니다. 로그 형식 규칙의 §0 핵심 요약은 기본으로 들어가므로 `LogConvention.md` 전문을 선언할 필요는 없습니다. 켜고 끄는 키(`autoread`)와 파일 목록 키는 따로입니다. `elf check setup`이 autoread 상태를 보고하고, 없는 파일을 가리키는 선언과 10,000자 안에 들어가지 않는 선언을 경고합니다.

```bash
elf autoread              # 지금 요약 출력
elf autoread status       # 설정, 훅, fulltext 선언, 대기 마커
elf autoread disable      # 이 프로젝트에서 끔(훅은 남지만 동작하지 않음)
```

> 훅을 설치한 뒤에는 `elf`를 2.19 미만으로 낮추지 마세요. 2.19 미만은 알 수 없는 서브명령에 exit 2를 반환하고, Claude Code는 이를 프롬프트 제출을 막는 훅 오류로 처리합니다. 보통의 설치와 갱신에는 영향이 없습니다.

---

[README](../README.ko.md)로 돌아가기
