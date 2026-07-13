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
| `elf init [name]` | Scaffold an ELF project (in place, or in `./<name>`) |
| `elf update` | Update managed files in the current project |
| `elf status` | Diagnose managed-file drift (read-only) |
| `elf validate` | Check session/registry/log consistency (read-only) |
| `elf session new <title>` | Create + register the next session log |
| `elf session close [S###]` | Close the active session → archive + update registry |
| `elf session fix-headers` | Repair session-log header rendering |
| `elf trial new [title]` | Append the canonical trial stub to the active session log |
| `elf gallery` | Generate the figure index `_gallery.md` from `6_Exp/64_Viz/` |
| `elf doctor` | Aggregate environment + project health check (read-only) |
| `elf self-update` | Update the `elf` binary itself |

Global flags: `elf --version`, `elf --help`, `elf <command> --help`.

## Commands

### `elf init [name]`

Scaffold an ELF project. **With no name, `elf init` initializes the current folder in place** (like `git init`); with a name, it creates a fresh `./<name>/` subfolder.

| Flag | Default | Meaning |
|------|---------|---------|
| `--here` | off | Force in-place in the current folder even if a name is given (uses the name as the project name) |
| `--yes` | off | Skip the in-place confirmation prompt (for scripts/CI) |
| `--dry-run` | off | Preview the plan; write nothing |
| `--force` | off | Overwrite existing files (default keeps yours) |
| `--preset <p>` | `full` | Module set: `full` / `experimental` / `software` / `minimal`. Experimental project types: **`qa`** (question-archive, not research), **`general`** (goal-driven non-research) |
| `--modules <list>` | — | Custom modules (comma-separated): `hw,fab,sw,exp,paper`. Overrides `--preset` |
| `--lang <lang>` | `ko-KR` | AI agent response language, BCP-47 tag (written to `.elf/config.json`). For a non-Korean tag (e.g. `en-US`), English **companion** docs are also deployed — see note below |

Core folders (`0_Meta`–`2_Log`) are always created; module folders (`3_HW`–`7_Paper`) are added per preset or `--modules`. The managed rule payload (rules + log-format stubs) is deployed under `.elf/managed/`; `0_Meta/` holds only user-owned files (`ProjectRule.md`, data overlays).

**In-place (no name)** adopts ELF into an existing folder **without overwriting anything**: missing ELF files are added, your existing files are kept, and a colliding ELF-managed file is written alongside as `<file>.elf-new` (your `.gitignore`, `README`, etc. are never clobbered). When the folder is non-empty you get one confirmation echoing the path; the project name defaults to the folder name. If the folder is already an ELF project (`.elf/` present) it refuses with exit 3 — use `elf update`. The named (subfolder) form still refuses with exit 3 if `./<name>` already exists.

```bash
elf init                              # in-place: adopt ELF into the current folder
elf init --here --preset general      # in-place, goal-driven project type
elf init . --yes                      # in-place, no prompt (scripts/CI)
elf init NIRS_Probe                   # subfolder: create ./NIRS_Probe/
elf init NIRS_Probe --preset experimental --lang en-US
elf init NIRS_Probe --modules hw,sw
elf init my_questions --preset qa                          # experimental: Q&A bundle archive (no categories)
elf init my_questions --preset qa --categories Daily,ITGeneral   # pre-create categories
```

> **`--lang en-*` (English companions, experimental).** The operative governance docs
> stay Korean (`*.md`) — the AI agent always operates from the Korean originals, so project
> behavior is identical across languages. For a non-Korean `--lang`, ELF additionally
> deploys **informative English companions** (`.elf/managed/EliRule.en.md`, etc.) for human
> reading, and the user-owned `README.md` / `ProjectRule.md` are scaffolded in English.
> Companions are non-operative (marked `NOT OPERATIVE`) — customize rules via
> `ProjectRule.md`, not the companion. `elf update` keeps companions in sync and
> `elf doctor` reports i18n status. English (`en`) is provided today; other languages fall
> back to Korean.

> **`qa` preset (experimental).** Scaffolds a *question-archive* project type instead of the research hierarchy: a root `AGENTS.md` (operational rules) + `CLAUDE.md` (loader pointer), `templates/bundle_template.md`, and `.elf/`. **No categories are pre-created by default** — create them on demand per `CLAUDE.md`, or pre-create with `--categories a,b,c` (each gets an `archive/`). Q&A is captured as semantic **bundles** (not sessions/trials/figures). Shares the `.elf/` control plane with its own manifest (`manifest.qa.json`), so `elf update` propagates the convention. Isolated from the research preset; subject to change while it is polished.

> **`general` preset (experimental).** Scaffolds a *goal-driven non-research* project (tool development, proposal, learning, build) — session/trial base-delta logging like the research preset, minus the academic layer (no `6_Exp`/`7_Paper`/figures/sim/literature). Shares the neutral managed files with the research preset and adds general-specific `EliRule`/`LogConvention` via its own manifest (`manifest.general.json`). Trial format defaults to the 5-section template; override per project in `ProjectRule.md`. Isolated from research; subject to change while polished.

### `elf update`

Update the ELF-managed files in the current project to the installed CLI version. **Your research data, logs, and settings are never touched.**

| Flag | Meaning |
|------|---------|
| `--dry-run` | Show the action list without writing anything |
| `--force` | Overwrite user-edited managed files / replace hybrid blocks |
| `--self` | Update the `elf` binary instead (alias of `self-update`) |

Behavior per file type → see [File ownership](#file-ownership).

Updates are **preset-aware**: the project type (research / `qa` / `general`) is read from the
`"preset"` field in `.elf/config.json` (recorded by `elf init`) and the matching template set
is used for planning and re-stamping. Projects initialized before 2.16.2 have no such field —
the preset is then inferred from the project's own stamp (`.elf/manifest.json`) and recorded
on the first non-dry-run update. If `config.json` and the stamp contradict each other,
`elf update` refuses to run until the `"preset"` field is fixed — this protects the project
from being updated against the wrong template set.

```bash
elf status            # see what would change
elf update --dry-run  # preview the actions
elf update            # apply (safe by default)
```

> **Pre-2.15 projects**: `elf update` does not read or migrate the legacy layout (rules in
> `0_Meta/`, stubs in a root `templates/`). When leftovers are detected it warns, names
> them, and leaves them untouched. Upgrade path: install **v2.15.1** from the Releases
> page → `elf update` → `elf migrate` there → then return to the latest CLI. Details:
> the 2.16.0 entry in [CHANGELOG.md](../CHANGELOG.md).

### `elf status [--check]`

Diagnose managed-file state (read-only). Reports each file as `ok` / `outdated` / `missing` / `edited`, plus version mismatch and obsolete entries. The report starts with the project's `preset:` line (research / qa / general — same resolution as `elf update`); a `config.json`/stamp contradiction is reported as a warning and the diagnosis follows the stamp.

- `--check`: exit **4** if there are any findings — use it as a pre-commit hook or CI gate.

### `elf validate [--check] [--strict]`

Check session bookkeeping consistency (read-only): registry ↔ log files (unregistered logs / phantom rows), session numbering (duplicates / gaps), multiple active sessions, broken relative `.md` cross-references inside logs, **figure-embed gaps** (a figure exists in `6_Exp/64_Viz/S###/` but is not inline-embedded in that session's log body — a table path is *not* an embed), and **trial structure** in active logs (non-canonical `###` headings, section order, the `가설 적중 여부` first line of `### 해석`, and Phase-1 sections missing while `### 관찰` exists). Structure checks skip `Archive/` (the convention applies to new writing, not backfill) and only cover the stable core of the format — content quality is not machine-checked.

- **issues** (registry/log mismatch, duplicate number, broken cross-ref) vs **warnings** (numbering gap, multiple active, figure-embed gap, trial structure) — only issues are gated.
- `--check`: exit **4** if there are any issues — pre-commit/CI gate.
- `--strict`: promote figure-embed gaps and trial-structure findings from warnings to issues (so `--check` gates on them).
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

Before archiving, close also runs `elf validate` and reports the findings **scoped to the session being closed** (missing figure embeds, trial-structure warnings) — non-blocking, but this is the last check before the log leaves the active-structure scope by moving to `Archive/`.

| Flag | Meaning |
|------|---------|
| `--force` | Close even when the `## 다음 세션 후보` (next-session) section is still empty |

```bash
elf session close             # close the one active session
elf session close S007        # close a specific session
```

Refuses with exit **3** if the next-session section is unfilled (fill it or pass `--force`). If the header `Handoff` still lists pending items, a **non-blocking warning** is printed — resolve them or carry them into the next-session section; a closing note also reminds you to rewrite the registry key finding as the session's final conclusion (fold). A parse-broken registry exits **5** (escalation).

### `elf session fix-headers [--dry-run]`

Add CommonMark hard breaks (`\`) to the blockquote headers of existing session logs so the metadata renders on separate lines in strict Markdown renderers (e.g. Discord preview). Idempotent and CRLF-safe. Operates on `2_Log/` + `2_Log/Archive/`.

### `elf trial new [title]`

Append the **current canonical trial stub** (embedded in the CLI; deployed reference copy at `.elf/managed/templates/trialTemplate.md`) to the active session log: the next `t##` is auto-numbered, `S{NNN}` paths are substituted, the header `Modified` date is refreshed, and the stub is inserted before the `## 다음 세션 후보` section. With no title the `[작업 제목]` placeholder is kept. The output also reminds you of the figure discipline at the point of action — list expected figures in Phase 1 and embed each one into `### 관찰 (Observation)` the moment it is created (sub-agent outputs included).

| Flag | Meaning |
|------|---------|
| `--session <S###>` | Target a specific session when several are open |

```bash
elf trial new "Wavelength sweep v2"   # append t## to the single active session
elf trial new --session S007          # several sessions open — name one
```

Why a command: agents (and humans) imitate whatever the previous trial looked like. `elf trial new` keeps the imitation target canonical — the stub always comes from the *installed* template version, so a drifted precedent never propagates. Errors: no open session → exit 1 (start one with `elf session new`); several open without `--session` → exit 1 with the list.

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
- **overlay** — active data overlays (`0_Meta/<name>.project.md`), removal entries missing a reason, overlays without an overlayable base
- **agent entry** — `CLAUDE.md` loads `@AGENTS.md` (warns when the pointer line is missing — Claude Code would not load the rules), flags heavy extra content in the pointer and pending `AGENTS.md.elf-new`/`CLAUDE.md.elf-new` files
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

`elf update` respects four ownership tiers:

| Tier | Files | On update |
|------|-------|-----------|
| **Managed** | `.elf/managed/` (`EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `LLMcliche.md`, `templates/*`, companions); root `.claudeignore`, `.editorconfig`, `AGENTS.md` | Replaced with the new version. If you edited one, it is **kept** and the new version is written as `<file>.elf-new` (use `--force` to overwrite) |
| **Yours** | `0_Meta/` (`ProjectRule.md`, `<name>.project.md` overlays), `Session_Registry.tsv`, `README.md`, all research data and logs | **Never touched** |
| **Hybrid** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`) is replaced; your rules outside the block are preserved |
| **Pointer** | `CLAUDE.md` | Created if missing; if present it is **never modified** (no `.elf-new` either) — an existing hand-written `CLAUDE.md` stays exactly yours. Add a `@AGENTS.md` line yourself to load the ELF rules; `elf doctor` checks the link |

Customize project rules in `ProjectRule.md` (yours) rather than editing managed files — that way updates never conflict. For the data files (`LLMcliche.md`, `highIFjournals.md`), add/remove/override entries via a **project overlay** `0_Meta/<name>.project.md` (user-owned; effective rules = base ⊕ overlay; removals need a stated reason — see EliRule §2.7).

## The `.elf/` directory

`elf init` and `elf update` maintain `.elf/` (do not edit by hand):

- `config.json` — project name, language, preset (project type), creation date
- `version` — the ELF version that last touched the project
- `manifest.json` — the record of managed files used by `update`/`status`
- `managed/` — the deployed rule payload (rules, companions, log-format stubs)
- `baseline/` — pristine copies of hybrid files, used to detect edits inside the managed block
