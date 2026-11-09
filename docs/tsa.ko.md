[English](tsa.md) | [한국어](tsa.ko.md)

# 연구 기록 시점 인증(선택 기능)

## `elf tsa [서브명령]` (opt-in)

연구 기록의 시점을 인증합니다. "이 내용이 이 시점에 있었음"을 증명하는 기능입니다. 켜 두면 커밋할 때마다 파일 해시가 날짜별 manifest(`0_Meta/tsa/YYYY-MM-DD_manifest.json`)에 기록되고(pre-commit 훅), 그날 첫 커밋 때 TSA에 RFC 3161 시점 토큰을 요청해 `.tsr` 파일로 저장합니다(post-commit 훅). 기본은 꺼져 있고 `enable` 전에는 아무것도 실행하지 않으므로, 필요 없으면 무시해도 됩니다.

**개인정보**: 외부로 나가는 것은 manifest 파일의 sha256 해시(32바이트) 하나뿐이며, 파일 내용과 파일 이름은 보내지 않습니다. **작성자 증명**(GPG 커밋 서명)은 이 명령이 아니라 git 설정으로 하는 선택 사항입니다. `elf tsa enable`과 `elf check files`가 설정 방법을 안내합니다.

| 서브명령 | 효과 |
|---|---|
| *(없음)* `[--all] [--quiet]` | 아직 토큰이 없는 manifest에 시점 토큰을 요청합니다. 기본은 그날 것만, `--all`은 토큰이 없는 manifest 전부(오프라인에서 한 커밋을 나중에 보충). post-commit 훅이 실행합니다. granted가 아닌 응답은 증거로 저장하지 않습니다 |
| `enable` | 여러 번 실행해도 결과가 같습니다. `.elf/config.json`에 `"tsa": true`를 쓰고, `0_Meta/tsa/`를 만들고, 훅 2개를 기존 훅을 지우지 않고 설치합니다(다른 도구의 훅은 건드리지 않고 직접 추가할 줄을 안내). GPG 안내를 출력한 뒤 **baseline을 기록**합니다(추적 중인 파일 전체를 기록하고 stamp). 과거 시점은 원리상 소급해 증명할 수 없으므로, baseline이 "도입 시점까지 있었음"의 기준이 됩니다 |
| `disable` | 설정을 끄고 ELF 표식이 있는 훅만 지웁니다(다른 훅은 유지). **`0_Meta/tsa/`의 증거는 지우지 않으며**, 다시 켜면 이어서 쌓입니다 |
| `status` | 켜짐 여부, 훅 소유, 증거 수, 토큰이 없는 manifest를 보여 줍니다(읽기만 함) |
| `record [--all] [--quiet]` | staged 파일(기본, pre-commit 훅이 쓰는 경로)이나 추적 중인 파일 전체(`--all`, baseline)의 해시를 그날 manifest에 기록합니다. 원래 바이트의 sha256이며 이미 있는 것은 건너뜁니다. manifest 자신은 자동으로 stage되어 같은 커밋에 들어갑니다 |
| `verify <파일>` | 파일의 현재 sha256을 모든 manifest에서 찾아 "이 내용이 언제 있었는지"를 보여 줍니다 |
| `verify --date <D>` | manifest와 토큰을 간단히 검증(granted, messageImprint 대조, genTime)하고, 서명 체인까지 엄밀히 검증할 `openssl ts -verify` 명령을 안내합니다(분쟁이나 감사처럼 드문 경우용) |

기본 TSA는 DigiCert의 무료 공개 TSA(`http://timestamp.digicert.com`)이며 계정이나 인증 정보가 필요 없습니다. `.elf/config.json`의 `"tsaUrl"`로 바꿀 수 있습니다. git이 필요하며 커밋이 기록 단위입니다. 훅은 커밋을 막지 않습니다(토큰 요청은 백그라운드에서 하고, 실패한 것은 나중에 `elf tsa --all`로 보충). 토큰 요청 말고는 모두 오프라인으로 동작합니다.

`elf update`는 ELF가 설치한 tsa 훅을 현재 명령으로 다시 씁니다. 2.26 이전에 설치된 훅을 바꿀 때는 마지막 manifest 날짜와 그 뒤의 커밋 수를 출력하므로, 각 프로젝트에서 `elf tsa record --all`, `elf tsa --all`을 실행할지 판단할 수 있습니다. ELF 표식이 없는 훅(예: 다른 도구와 함께 쓰는 훅)은 고치지 않습니다. 그런 훅이 삭제된 명령(`tsa stamp`, `tsa record --staged`, `--backfill`, `validate --strict`)을 부르면 `elf check files`와 `elf tsa status`가 경고와 함께 대체 명령(`tsa`, `tsa record`, `--all`, `check logs`)을 보여 줍니다.

```bash
elf tsa enable            # 프로젝트마다 한 번. 이후 훅과 baseline
git commit …              # 기록과 시점 인증이 자동으로 됨
elf tsa verify paper.md   # 이 내용이 있었던 이력
elf tsa --all             # 오프라인 기간 뒤 보충
elf tsa disable           # 기록 중지. 증거는 남음
```

---

[README](../README.ko.md)로 돌아가기
