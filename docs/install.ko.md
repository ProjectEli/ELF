[English](install.md) | [한국어](install.ko.md)

# elf 설치

## 설치

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

바이너리가 `~/.elf/bin`에 설치되고 PATH에 등록됩니다. 새 셸을 열어 `elf --version`으로 확인하세요. 도구 자체 갱신은 `elf self update`(또는 인스톨러 재실행).

## CLI 갱신

`elf self update`는 최신 발행본을 버전 보관소의 자기 폴더(`~/.elf/versions/vX.Y.Z/`)에 받아 sha256과 실행 여부를 확인한 뒤에야 기본 버전(`~/.elf/current`)으로 바꿉니다. 프로젝트는 자기 버전으로 계속 실행하고, 실행 중인 프로젝트가 쓰는 파일은 바꾸지 않으므로 열려 있는 다른 프로젝트가 끊기지 않습니다. PATH의 실행기는 버전 고르는 방식이 바뀐 발행본에서만, 받아 둔 파일로 한 번에 바꿉니다. 느리거나 끊긴 받기는 멈춘 곳부터 이어받습니다(시도마다 2분까지, 시도는 20번까지, 연속 두 번 아무것도 받지 못하면 멈춤). 발행 목록을 읽을 수 없으면 수동 재설치용 인스톨러 명령을 안내합니다.

**정리**는 `elf self update`와 `elf self install` 때만 합니다. 줄마다 최신 patch만 남기고, 다른 줄은 기본 줄이거나 최근 30일 안에 쓴 것만 남기며, 사용 중인 파일은 건너뛰고 다음에 다시 시도합니다. 지운 줄은 프로젝트가 필요할 때 다시 받습니다. `elf self prune`, `elf self uninstall <버전>`으로 직접 지울 수도 있습니다. `elf self`만 실행하면 help를 보여 주므로 실수로 바이너리가 교체되지 않습니다. 이전 형식 `elf self-update`, `elf selfupdate`, `elf update --self`도 동작합니다.

---

[README](../README.ko.md)로 돌아가기
