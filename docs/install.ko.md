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

`elf` 바이너리 자체를 최신 릴리즈로 갱신합니다. 인스톨러로 설치된 경우 동작(install receipt 사용), 아니면 인스톨러 명령을 안내. `elf self`만 실행하면 help를 보여 주므로 실수로 바이너리가 교체되지 않습니다. 이전 형식 `elf self-update`, `elf selfupdate`, `elf update --self`도 동작합니다.

---

[README](../README.ko.md)로 돌아가기
