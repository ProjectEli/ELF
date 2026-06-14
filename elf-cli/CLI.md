[English](CLI.md) | [한국어](CLI.ko.md)

# elf CLI Reference

`elf` is the command-line tool for ELF (Eli's Lab Framework): it scaffolds research projects, keeps framework files up to date, and diagnoses drift. It ships as a self-contained single binary — no Node or Python runtime required.

> Framework philosophy and folder structure: [README.md](../README.md). This document is the command reference.

## Install

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

Installs the binary to `~/.elf/bin` and adds it to PATH. Open a new shell and verify with `elf --version`. Update the tool itself with `elf self-update` (or re-run the installer).

## Command summary

| Command | Purpose |
|---------|---------|
| `elf init <name>` | Scaffold a new ELF project |
| `elf update` | Update managed files in the current project |
| `elf status` | Diagnose managed-file drift (read-only) |
| `elf validate` | Check session/registry/log consistency (read-only) |
| `elf session new <title>` | Create + register the next session log |
| `elf session close [S###]` | Close the active session → archive + update registry |
| `elf session fix-headers` | Repair session-log header rendering |
| `elf gallery` | Generate the figure index `_gallery.md` from `6_Exp/64_Viz/` |
| `elf doctor` | Aggregate environment + project health check (read-only) |
| `elf self-update` | Update the `elf` binary itself |

Global flags: `elf --version`, `elf --help`, `elf <command> --help`.

## Commands

### `elf init <name>`

Scaffold a new ELF project in `./<name>`.

| Flag | Default | Meaning |
|------|---------|---------|
| `--preset <p>` | `full` | Module set: `full` / `experimental` / `software` / `minimal` |
| `--modules <list>` | — | Custom modules (comma-separated): `hw,fab,sw,exp,paper`. Overrides `--preset` |
| `--lang <lang>` | `한국어` | AI agent response language (written to `.elf/config.json`) |

Core folders (`0_Meta`–`2_Log` + `templates`) are always created; module folders (`3_HW`–`7_Paper`) are added per preset or `--modules`. Refuses with exit 3 if `<name>` already exists.

```bash
elf init NIRS_Probe
elf init NIRS_Probe --preset experimental --lang English
elf init NIRS_Probe --modules hw,sw
```

### `elf update`

Update the ELF-managed files in the current project to the installed CLI version. **Your research data, logs, and settings are never touched.**

| Flag | Meaning |
|------|---------|
| `--dry-run` | Show the action list without writing anything |
| `--force` | Overwrite user-edited managed files / replace hybrid blocks |
| `--self` | Update the `elf` binary instead (alias of `self-update`) |

Behavior per file type → see [File ownership](#file-ownership).

```bash
elf status            # see what would change
elf update --dry-run  # preview the actions
elf update            # apply (safe by default)
```

### `elf status [--check]`

Diagnose managed-file state (read-only). Reports each file as `ok` / `outdated` / `missing` / `edited`, plus version mismatch and obsolete entries.

- `--check`: exit **4** if there are any findings — use it as a pre-commit hook or CI gate.

### `elf validate [--check] [--strict]`

Check session bookkeeping consistency (read-only): registry ↔ log files (unregistered logs / phantom rows), session numbering (duplicates / gaps), multiple active sessions, broken relative `.md` cross-references inside logs, and **figure-embed gaps** (a figure exists in `6_Exp/64_Viz/S###/` but is not inline-embedded in that session's log body — a table path is *not* an embed).

- **issues** (registry/log mismatch, duplicate number, broken cross-ref) vs **warnings** (numbering gap, multiple active, figure-embed gap) — only issues are gated.
- `--check`: exit **4** if there are any issues — pre-commit/CI gate.
- `--strict`: promote figure-embed gaps from warnings to issues (so `--check` gates on them).
- Exclude an intentional non-embed (SI/deprecated figure) with a `<!-- noembed: filename.png -->` comment in the log.
- If the registry itself cannot be parsed, `elf` exits **5** (escalation) — it distinguishes "found problems" from "cannot check".

```bash
elf validate                   # report (figure-embed gaps as warnings)
elf validate --check            # exit 4 on issues (CI gate)
elf validate --check --strict   # also gate on figure-embed gaps
```

### `elf session new <title>`

Create the next session log (auto-incremented `S###`, derived from logs in `2_Log/` + `2_Log/Archive/` and the registry) from the template, and register it in `2_Log/Wiki/Session_Registry.tsv`.

```bash
elf session new "Wavelength Optimization"
# → creates 2_Log/S002_log.md and appends a registry row
```

The title must not contain a tab character (it would break the TSV registry). If the registry cannot be parsed, `elf` exits **5** (escalation — see below) instead of writing.

### `elf session close [S###]`

Close a session: set its header `Status` to `Complete`, move the log to `2_Log/Archive/` (filename unchanged — the folder *is* the status), and update its registry row. With no id, the single active session is chosen automatically; if several are open, `elf` lists them and asks you to name one.

| Flag | Meaning |
|------|---------|
| `--force` | Close even when the `## 다음 세션 후보` (next-session) section is still empty |

```bash
elf session close             # close the one active session
elf session close S007        # close a specific session
```

Refuses with exit **3** if the next-session section is unfilled (fill it or pass `--force`). A parse-broken registry exits **5** (escalation).

### `elf session fix-headers [--dry-run]`

Add CommonMark hard breaks (`\`) to the blockquote headers of existing session logs so the metadata renders on separate lines in strict Markdown renderers (e.g. Discord preview). Idempotent and CRLF-safe. Operates on `2_Log/` + `2_Log/Archive/`.

### `elf gallery`

Scan `6_Exp/64_Viz/` and regenerate `6_Exp/64_Viz/_gallery.md` — a figure index grouped by session subdirectory. Each `.png` / `.jpg` / `.svg` becomes an embedded image link. Sessions with no images are skipped. If `6_Exp/64_Viz/` does not exist, it prints a notice and exits **0** (nothing to do).

```bash
elf gallery
# → wrote 6_Exp/64_Viz/_gallery.md (3 session(s), 12 image(s))
```

### `elf self-update`

Update the `elf` binary itself to the latest release. Works on installer-based installs (it reads the install receipt); otherwise it prints the installer command for manual update. Also reachable as `elf update --self`.

### `elf doctor`

Aggregate health check (read-only). Reports each item as `OK` / `WARN` / `INFO`:

- **environment** — `elf` version, install receipt presence (self-update availability)
- **project** (if inside one) — `.elf/` stamp parses, version matches the CLI, baseline present
- **managed files** — a `elf status` summary (pending / conflicts)
- **git** — repository and `pre-commit` hook presence

Works outside an ELF project too (environment checks only). Does not hit the network. Always exits **0**.

```bash
elf doctor
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error (I/O, not an ELF project, …) |
| 2 | Usage error (bad flags or arguments) |
| 3 | Refused (e.g. the target already exists) |
| 4 | `--check` found issues (gate signal, distinct from a real error) |
| 5 | **Escalation** — needs human/agent judgment (see below) |

## Escalation (exit 5)

When a deterministic operation hits something it must not auto-repair (for example, a malformed `Session_Registry.tsv`), `elf` does **not** guess. It writes a structured report to stderr and exits 5:

```
[elf] escalation: 2_Log/Wiki/Session_Registry.tsv (line 7)
  expected: 6 tab-separated columns (Session/Date/Title/Status/Key Finding/Archive Path)
  found:    4 columns
  raw:      <the offending line>
agent-action: fix the line to match the schema, then re-run (this tool will not auto-edit)
```

The `agent-action:` line is a stable marker. An LLM agent driving `elf` can detect it, repair the file, and retry. Exit 5 is deliberately distinct from exit 1 so automation can branch on "fixable by editing a file" versus "retrying is pointless".

## File ownership

`elf update` respects three ownership tiers:

| Tier | Files | On update |
|------|-------|-----------|
| **Managed** | `EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `templates/*`, `.claudeignore`, `.editorconfig` | Replaced with the new version. If you edited one, it is **kept** and the new version is written as `<file>.elf-new` (use `--force` to overwrite) |
| **Yours** | `ProjectRule.md`, `Session_Registry.tsv`, `README.md`, all research data and logs | **Never touched** |
| **Hybrid** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`) is replaced; your rules outside the block are preserved |

Customize project rules in `ProjectRule.md` (yours) rather than editing managed files — that way updates never conflict.

## The `.elf/` directory

`elf init` and `elf update` maintain `.elf/` (do not edit by hand):

- `config.json` — project name, language, creation date
- `version` — the ELF version that last touched the project
- `manifest.json` — the record of managed files used by `update`/`status`
- `baseline/` — pristine copies of hybrid files, used to detect edits inside the managed block
