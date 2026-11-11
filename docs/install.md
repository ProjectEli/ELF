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

`elf self update` downloads the latest release into its own folder in the version store (`~/.elf/versions/vX.Y.Z/`), checks its sha256 and that it runs, and only then makes it the default (`~/.elf/current`). Projects keep running their own versions, and nothing a running project uses is replaced, so another open project is not interrupted. The launcher on PATH is replaced only by a release that changes how versions are picked, and then from the downloaded file in one quick step. When the release list cannot be read, it prints the installer command for a manual reinstall.

**Cleanup** runs only during `elf self update` and `elf self install`: each line keeps only its newest patch; other lines are kept when they are the default line or were used in the last 30 days; a file in use is skipped and retried next time. A removed line is downloaded again when a project needs it. `elf self prune` and `elf self uninstall <version>` remove versions by hand. `elf self` alone prints help, so the binary is never replaced by accident. The older forms `elf self-update`, `elf selfupdate` and `elf update --self` still work.

---

Back to [README](../README.md)
