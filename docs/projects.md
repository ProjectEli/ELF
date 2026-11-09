[English](projects.md) | [한국어](projects.ko.md)

# Creating and updating projects

## `elf init [name]`

Scaffold an ELF project. **With no name, `elf init` initializes the current folder in place** (like `git init`); with a name, it creates a fresh `./<name>/` subfolder.

| Flag | Default | Meaning |
|------|---------|---------|
| `--here` | off | Initialize the current folder under the given name (`elf init --here <name>`); with no name, `elf init` is already in place |
| `--yes` | off | Skip the in-place confirmation prompt (for scripts/CI) |
| `--dry-run` | off | Preview the plan; write nothing |
| `--force` | off | Overwrite existing files (default keeps yours) |
| `--preset <p>` | `full` | Module set: `full` (all modules) / `exp` (6_Exp + 7_Paper) / `sw` (5_SW + 7_Paper) / `minimal` (none). A research preset is the module of the same name plus `paper`; the names before 2.26 (`experimental`, `software`) are still accepted. **`qa`** = question-archive project type (not research). The `general` preset was removed in 2.27. Use `minimal` |
| `--modules <list>` | none | Custom modules (comma-separated): `hw,fab,sw,exp,paper`. Overrides `--preset`; recorded as `"preset": "custom"` plus the module list in `.elf/config.json` |
| `--lang <lang>` | `ko-KR` | AI agent response language, BCP-47 tag (written to `.elf/config.json`). For a non-Korean tag (e.g. `en-US`), English **companion** docs are also deployed (see the note below) |

Core folders (`0_Meta` to `2_Log`) are always created; module folders (`3_HW` to `7_Paper`) are added per preset or `--modules`. The managed rule payload (rules + log-format stubs) is deployed under `.elf/managed/`; `0_Meta/` holds only user-owned files (`ProjectRule.md`, data overlays).

**In-place (no name)** adopts ELF into an existing folder **without overwriting anything**: missing ELF files are added, your existing files are kept, and a colliding ELF-managed file is written alongside as `<file>.elf-new` (your `.gitignore`, `README`, etc. are never clobbered). When the folder is non-empty you get one confirmation echoing the path; the project name defaults to the folder name. If the folder is already an ELF project (`.elf/manifest.json` present) it refuses with exit 3. Use `elf update`. A folder from an old ELF version that has `.elf/` but no `.elf/manifest.json` (where `elf update` refuses) is adopted the same way: the existing `.elf/config.json` keys are kept (`created` included), `--preset`, `--lang` and the name default to the values already in it, and session logs in the earlier layout are left for `elf update` to migrate (the output says so). `elf init` creates no session; start the first one with `elf session new`. Afterwards `elf check files` lists what is left: old rule copies in `0_Meta/` (the current rules are in `.elf/managed/`; delete the copies after moving any project-specific edits into `ProjectRule.md`), a `CLAUDE.md` without `@AGENTS.md`, a `.gitignore` without the ELF block (`elf update --force` inserts it at the top and keeps your lines). The named (subfolder) form still refuses with exit 3 if `./<name>` already exists.

```bash
elf init                              # in-place: adopt ELF into the current folder
elf init --here MyTool --preset minimal   # in-place under a given name, core folders only
elf init . --yes                      # in-place, no prompt (scripts/CI)
elf init NIRS_Probe                   # subfolder: create ./NIRS_Probe/
elf init NIRS_Probe --preset exp --lang en-US
elf init NIRS_Probe --modules hw,sw
elf init my_questions --preset qa                          # Q&A bundle archive (no categories)
elf init my_questions --preset qa --categories Daily,ITGeneral   # pre-create categories
```

> **`--lang en-*` (English companions, experimental).** The operative governance docs
> stay Korean (`*.md`). The AI agent always operates from the Korean originals, so project
> behavior is identical across languages. For a non-Korean `--lang`, ELF additionally
> deploys **informative English companions** (`.elf/managed/EliRule.en.md`, etc.) for human
> reading, and the user-owned `README.md` / `ProjectRule.md` are scaffolded in English.
> Companions are non-operative (marked `NOT OPERATIVE`). Customize rules via
> `ProjectRule.md` instead of the companion. `elf update` keeps companions in sync and
> `elf check files` reports i18n status. English (`en`) is provided today; other languages fall
> back to Korean.

> **`qa` preset.** Scaffolds a *question-archive* project type instead of the research hierarchy: a root `AGENTS.md` (operational rules) + `CLAUDE.md` (loader pointer), `0_Meta/ProjectRule.md` (your project rules and egress relaxations, never touched by `elf update`; it overrides `AGENTS.md` on conflict), `templates/bundle_template.md`, and `.elf/`. **No categories are pre-created by default.** Create them on demand per `AGENTS.md`, or pre-create them with `--categories a,b,c` (each gets an `archive/`). Q&A is captured as semantic **bundles** (not sessions/trials/figures). Shares the `.elf/` control area with its own manifest (`manifest.qa.json`), so `elf update` propagates the convention. Kept separate from the research preset.

> **`general` preset (removed in 2.27).** `minimal` covers the same use (core folders only). `elf init --preset general` is refused as an unknown preset. In an existing general project, `elf update` converts it to `minimal` (research lineage): the ELF-managed rule files take the research versions, your own files (`0_Meta/ProjectRule.md`, `README.md`, logs) are untouched, and the output says so (`--dry-run` shows it first).

## `elf update`

Update the ELF-managed files in the current project to the installed CLI version. **Your research data and settings are never touched.** Session logs are changed only once, when a project made before 2.28 is migrated to the new log layout ([below](#migrating-the-log-layout-228)).

| Flag | Meaning |
|------|---------|
| `--dry-run` | Show the plan (including the migration) without writing anything |
| `--force` | Overwrite user-edited managed files / replace hybrid blocks |

The project manifest `.elf/manifest.json` records the ELF version that last updated the project (`elf_version`). If it is newer than the installed CLI (by MAJOR.MINOR), `elf update` and the commands that write logs refuse (exit 3) until `elf self update`; `elf check version` shows the verdict. After a project is updated to 2.28, every PC that works on the same repository needs `elf self update` before pulling.

Behavior per file type → see [File ownership](#file-ownership).

Updates are **preset-aware**: the project type (research / `qa`) is read from the
`"preset"` field in `.elf/config.json` (recorded by `elf init`) and the matching template set
is used for planning and for rewriting the project manifest. Projects initialized before 2.16.2 have no such field.
The preset is then inferred from the project manifest (`.elf/manifest.json`) and recorded
on the first non-dry-run update. If `config.json` and the manifest contradict each other,
`elf update` refuses to run until the `"preset"` field is fixed. This protects the project
from being updated against the wrong template set. The first line shows the preset and its lineage, e.g. `preset: minimal (research lineage)`. A research project whose preset is unset or recorded as `research` is treated as `full`, and `elf update` records `full`.

```bash
elf self update          # apply a new version ① update the CLI
elf check version        # ② CLI and project versions, and what to run
elf check files          #   see what would change
elf update --dry-run     # ③ preview without writing
elf update               # ④ managed files only. An `edited:` line means your edited file was kept and the new version was written as .elf-new; diff and merge (or --force to take the canonical version)
elf check                # team/CI gate: exit 4 on issues (elf check logs for the logs only)
```

### Migrating the log layout (2.28)

From 2.28, logs live in `2_Log/S/` and `2_Log/G/` with a YAML header and stay where they are ([Sessions and trials](sessions.md)). Updating the CLI does not migrate anything: a project made earlier keeps recording with the 2.27 commands (2.28.1 and later) until you run `elf update` in it. Migrate each project when it is ready; `elf update` migrates once: blockquote headers become the 7 YAML keys, logs move from `2_Log/` and `2_Log/Archive/` to `2_Log/S/`, links are recomputed, deprecation records become `.elf/deprecated/` store files, `Registry.md` and the explorer nesting are generated, and `2_Log/Wiki/Session_Registry.tsv` is removed. The project's own AI agent runs it and handles the warnings.

| Step | What happens |
|------|--------------|
| Precondition | The git working tree must be clean (the migration is undone with git). Otherwise nothing is written |
| Plan first | `elf update` plans the whole migration. If there is any warning, it writes only the decision file `.elf/runtime/migration.yaml` (not tracked by git), prints a summary, and exits 3; the managed files wait too |
| Decide | Status warnings (header and Registry disagree, both missing, a value that is not canonical) get a `status` field to fill with `ongoing`, `complete` or `deprecated`, judged from the evidence shown and confirmed with the user. Other warnings (a link target that does not exist, two logs with one number) list a file to fix |
| Apply | Run `elf update` again. It applies everything at once and deletes the decision file. Then `elf check`, review with the user, and commit |

Header values map to the 7 keys (`Created` or `Date` → created, `목표` or `Goal` → goal, `Handoff` → handoff, `결론` or the Registry key finding → conclusion; status `★ 활성` → ongoing, `Complete` → complete, `Deprecated` or `Purged` → deprecated). Other header fields (such as `관련`) are kept as one body line under the title (`> 이관 전 헤더: …`), so their values and links stay. Links and code spans that hold a single relative path are recomputed for the new location when they point to an existing file. A log that an older CLI creates in `2_Log/` after the migration is moved to `2_Log/S/` under the next free number by the next `elf update`. A folder with `2_Log/` but no `.elf/` (logs only) is migrated the same way.

> **Pre-2.15 projects**: `elf update` does not read or migrate the legacy layout (rules in
> `0_Meta/`, stubs in a root `templates/`). When legacy rule files remain it warns, names
> them, and leaves them untouched. Upgrade path: install **v2.15.1** from the Releases
> page → `elf update` → `elf migrate` there → then return to the latest CLI. Details:
> the 2.16.0 entry in [CHANGELOG.md](../CHANGELOG.md).

## File ownership

`elf update` respects four ownership tiers:

| Tier | Files | On update |
|------|-------|-----------|
| **Managed** | `.elf/managed/` (`EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `LLMcliche.md`, `templates/*`, companions); root `.editorconfig`, `AGENTS.md` | Replaced with the new version. If you edited one, it is **kept** and the new version is written as `<file>.elf-new` (use `--force` to overwrite) |
| **Yours** | `0_Meta/` (`ProjectRule.md`, `<name>.project.md` overlays), `README.md`, all research data and logs | **Never touched** (logs: only the one-time migration above) |
| **Generated** | `2_Log/S/Registry.md`, `2_Log/G/Registry.md`, the `explorer.fileNesting` keys in `.vscode/settings.json` (other settings are yours) | Rebuilt from the log headers by every command that changes a log |
| **Hybrid** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`) is replaced; your rules outside the block are preserved |
| **Pointer** | `CLAUDE.md` | Created if missing; if present it is **never modified** (no `.elf-new` either). An existing hand-written `CLAUDE.md` stays exactly yours. Add a `@AGENTS.md` line yourself to load the ELF rules; `elf check files` checks the link the way Claude Code parses imports (an `@AGENTS.md` inside backticks, a code block, an HTML comment or quotes is not imported, and is reported as a warning) |

Customize project rules in `ProjectRule.md` (yours) instead of editing managed files, so updates never conflict. For the data files (`LLMcliche.md`, `highIFjournals.md`), add/remove/override entries via a **project overlay** `0_Meta/<name>.project.md` (user-owned; effective rules = base ⊕ overlay; removals need a stated reason, see EliRule §2.7).

## The `.elf/` directory

`elf` maintains `.elf/` (do not edit by hand, except the migration decision file):

- `config.json`: project name, language, preset (project type), creation date
- `manifest.json`: the project manifest, the record of managed files and the ELF version that last updated the project (`elf_version`); before 2.28 the version was in a separate `version` file
- `managed/`: the deployed rule payload (rules, companions, log-format stubs)
- `baseline/`: pristine copies of hybrid files, used to detect edits inside the managed block
- `deprecated/`: compressed originals of deprecated content ([Deprecating content](deprecate.md)); tracked by git
- `runtime/`: temporary tool state (autoread markers, the migration decision file); not tracked by git

---

Back to [README](../README.md)
