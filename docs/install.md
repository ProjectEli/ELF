[English](install.md) | [한국어](install.ko.md)

# Installing elf

## Install

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

Installs the binary to `~/.elf/bin` and adds it to PATH. Open a new shell and verify with `elf --version`. Update the tool itself with `elf self update` (or re-run the installer).

## Update the CLI

Update the `elf` binary itself to the latest release. Works on installer-based installs (it reads the install receipt); otherwise it prints the installer command for manual update. `elf self` alone prints help, so the binary is never replaced by accident. The older forms `elf self-update`, `elf selfupdate` and `elf update --self` still work.

---

Back to [README](../README.md)
