[English](reference.md) | [한국어](reference.ko.md)

# 명령 레퍼런스

## 명령 요약

| 명령 | 용도 |
|------|------|
| `elf init [이름]` | ELF 프로젝트 생성(현재 폴더, 또는 `./<이름>`) |
| `elf update` | 현재 프로젝트의 관리 파일 갱신, 2.28 이전 프로젝트의 로그 구조 이관 |
| `elf self update` | 최신 발행본을 버전 보관소에 받아 기본 버전으로 함(프로젝트는 자기 버전 유지), 이어서 보관소 정리 |
| `elf self list [--available]` | 버전 보관소 `~/.elf/versions`에 둔 버전 목록(`--available`: GitHub에서 줄마다 최신 발행) |
| `elf self install <버전>` | 발행본을 받아 sha256을 확인한 뒤 버전 보관소에 둠. `2.27` = 그 줄의 최신 patch, `2.27.4` = 그 발행본. 이어서 보관소 정리(방금 받은 버전은 남김) |
| `elf self uninstall <버전>`, `elf self prune` | 보관된 버전 하나 삭제, 줄마다 가장 새 버전만 남김 |
| `elf session new "<제목>" --goal "<목표>" [--gov]` | `2_Log/S/`(또는 `2_Log/G/`)에 다음 세션 로그 생성 |
| `elf session close [S###\|G###] --from <파일>` | 결론과 함께 세션 종료(status complete, 파일은 옮기지 않음) |
| `elf trial new "<제목>" [--from <파일>]` | 다음 trial 생성(Phase 1 틀, 또는 파일의 Phase 1) |
| `elf trial write [tNN] --from <파일>` | 파일로 Phase 1(Phase 2 전까지 수정) 또는 Phase 2(한 번) 쓰기 |
| `elf log`, `elf log show <ID> [tNN]` | 세션 로그 목록, 헤더와 trial 목록 또는 trial 하나 보기 |
| `elf check [version\|setup\|files\|logs]` | 버전, 설정, 관리 파일, 세션 로그 점검(읽기 전용, 범위 생략 = 전체, issue가 있으면 exit 4) |
| `elf deprecate [대상] …` | 세션 로그나 계획 문서에서 내용을 지우고 원문을 `.elf/deprecated/`에 압축 보관, `show`, `--restore`, 인수 없음 = 목록 |
| `elf gallery` | `6_Exp/64_Viz/`에서 figure 색인 `_gallery.md` 생성 |
| `elf tsa [하위]` | 연구 기록 타임스탬프, **opt-in**: 커밋별 해시 manifest와 RFC 3161 토큰(하위 명령 없음 = 오늘 토큰 요청) |
| `elf autoread [하위]` | 세션 시작과 컨텍스트 재구성 뒤의 규칙 요약 출력(모든 하네스). Claude Code에서는 훅이 자동으로 넣음(기본 켬) |

2.26 이전 이름도 `--help`에는 보이지 않지만 동작합니다: `elf doctor`, `elf status`, `elf validate`(`elf check` 참고), `elf self-update`, `elf selfupdate`, `elf update --self`(`elf self update` 참고), `elf deprecate list`. `elf session fix-headers`와 `elf deprecate purge`는 2.28에서 없앴습니다.

공통 플래그: `elf --version`, `elf --help`, `elf <명령> --help`.

## 프로젝트별 버전

설치된 버전은 모두 자기 폴더 `~/.elf/versions/vX.Y.Z/`에 있습니다. PATH의 `elf`(`~/.elf/bin/elf`)는 실행기입니다. 버전을 골라 그 버전의 실행 파일로 같은 인수, 출력, 종료 코드를 넘겨 실행하고, 작업은 직접 하지 않습니다. 그래서 `elf`를 갱신하면 폴더가 하나 늘고 기본 버전이 바뀔 뿐이며, 다른 창의 프로젝트가 실행 중인 파일은 건드리지 않습니다.

- **프로젝트 안**에서는 프로젝트 manifest의 버전(`elf_version`, `elf init`과 `elf update`만 씀)을 따릅니다. 프로젝트의 줄(MAJOR.MINOR, 2.28.x는 3.0으로 봄)이 기본보다 낮든 같든 높든 그 줄의 보관된 최신 patch가 실행합니다. 그런 프로젝트의 첫 명령이 그 발행본을 받아 sha256을 확인한 뒤 보관하고, 하루 한 번 그 줄의 새 patch가 있는지 봅니다. 그날 앞서 받은 발행 목록에 프로젝트의 기록 버전이나 줄이 없으면 목록을 다시 받고(1시간에 한 번까지), 그래도 정하지 못하면 기록 버전을 바로 받습니다.
- **프로젝트 밖**과 `elf self …`, `elf init`, `elf update`, `elf check version`, `--help`, `--version`은 기본 버전(`~/.elf/current`)이 실행합니다.
- 같은 구조 세대에서 더 낮은 줄의 프로젝트에는 `elf update`로 기본 줄로 옮길 수 있다는 안내(사용자 승인 뒤 적용)를 하루 한 번 보여 줍니다. 그 버전을 구할 수 없으면 기본 버전이 그대로 실행하고(거부하지 않음) 안내에 그 사실을 적습니다.
- 버전 기록이 없는 2.27 형식 프로젝트는 2.27의 최신 발행본으로 실행하고, 받을 수 없으면 내장 2.27 호환 모드로 실행합니다. 기본보다 높은 줄을 구할 수 없으면 쓰는 명령은 거부하고(exit 3) 읽는 명령은 경고와 함께 실행하며, `elf self update`로 해결합니다.
- 실행기가 처음 실행될 때(예: 3.8에서 갱신한 직후) 자기 자신을 버전 보관소에 넣고 `~/.elf/current`를 씁니다.
- 기본 버전, 실행기, 프로젝트를 실행하는 버전은 `elf check version`이 보여 줍니다.

| 변수 | 의미 |
|------|------|
| `ELF_DISPATCH` | `off`: 다른 버전으로 넘기지 않음(시험, CI, 진단). `trace`: 결정을 표준 오류에 출력 |
| `ELF_HOME` | `bin/`, `versions/`, `current`를 둘 폴더(기본 `~/.elf`) |
| `ELF_RELEASES` | 발행 자산의 기준 주소 또는 로컬 폴더(시험용) |

## 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 성공 |
| 1 | 실행 오류(I/O, ELF 프로젝트가 아님, 없는 세션이나 ID 등) |
| 2 | 사용법 오류(잘못된 플래그나 인수) |
| 3 | 거부, 아무것도 쓰지 않음: 검사를 통과하지 못한 파일(이유를 모두 출력), CLI보다 높은 프로젝트, 더 높은 ELF 줄을 구할 수 없는 프로젝트에서 쓰기, 이관하지 않은 프로젝트에서 새 구조 명령, 결정이 필요한 이관(결정 파일을 씀) |
| 4 | `elf check`가 issue를 찾음(옛 이름 `status`, `validate`의 `--check` 포함). 실제 오류와 구분되는 통과 여부 신호 |

exit 5(escalation)는 이관하지 않은 프로젝트(2.27 명령을 계속 씀)의 TSV 세션 Registry를 읽을 수 없을 때입니다. 이관한 프로젝트에는 TSV Registry가 없으므로 exit 5가 나오지 않습니다.

---

[README](../README.ko.md)로 돌아가기
