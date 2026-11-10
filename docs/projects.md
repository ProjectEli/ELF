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
| `--lang <lang>` | `ko-KR` | AI agent response language, BCP-47 tag (written to `.elf/config.json`). With an English tag (`en-*`) the project gets the English rule set (see the note below) |

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

> **`--lang en-*` (English projects).** An English project gets everything in English. The rule
> documents (`.elf/managed/EliRule.md`, `LogConvention.md`, etc., and the root `AGENTS.md`) are the English
> versions at the same paths, and they are the canonical rules the AI agent follows in that project. The session
> and trial templates write English headings (`### Conditions`, `Prediction match:`); the `.gitignore` block,
> `.editorconfig`, `CLAUDE.md` and the user-owned `README.md` / `ProjectRule.md` are in English; elf writes
> deprecation markers in English. A new English project contains no Korean text. elf still reads the older
> Korean-first headings (`### 조건 (Conditions)`) as the same sections, so earlier logs stay valid. `elf update`
> moves an existing English project to this set after approval: unchanged rule files are replaced, an edited one
> gets the new version beside it as `.elf-new`, and the earlier English copies (`*.en.md`) are removed when
> unchanged. An existing `CLAUDE.md` is never changed. Customize rules in `ProjectRule.md`. Other languages use
> the Korean set.

> **`qa` preset.** Scaffolds a *question-archive* project type instead of the research hierarchy: a root `AGENTS.md` (operational rules) + `CLAUDE.md` (loader pointer), `0_Meta/ProjectRule.md` (your project rules and egress relaxations, never touched by `elf update`; it overrides `AGENTS.md` on conflict), `templates/bundle_template.md`, and `.elf/`. **No categories are pre-created by default.** Create them on demand per `AGENTS.md`, or pre-create them with `--categories a,b,c` (each gets an `archive/`). Q&A is captured as semantic **bundles** (not sessions/trials/figures). Shares the `.elf/` control area with its own manifest (`manifest.qa.json`), so `elf update` propagates the convention. Kept separate from the research preset.

> **`general` preset (removed in 2.27).** `minimal` covers the same use (core folders only). `elf init --preset general` is refused as an unknown preset. In an existing general project, `elf update` converts it to `minimal` (research lineage): the ELF-managed rule files take the research versions, your own files (`0_Meta/ProjectRule.md`, `README.md`, logs) are untouched, and the output says so (`--dry-run` shows it first).

## `elf update`

Update the ELF-managed files in the current project to the installed CLI version. **Your research data and settings are never touched.** Session logs are changed only once, when a project made before 2.28 is migrated to the new log layout ([below](#migrating-the-log-layout-228)).

| Flag | Meaning |
|------|---------|
| `--dry-run` | Show the plan (including the migration) without writing anything |
| `--out <folder>` | With `--dry-run`: write the migration's result files and tables (headers, link and code span changes, path mentions, warnings) to this folder. Inside the project, only under `.elf/runtime/` |
| `--to <line>` | Target line, such as `3.0` (default: the line of the `elf` on PATH). Another line is run by that version from the version store |
| `--approve <code>` | Apply a change of line with the approval code from the summary, after the user approved it |
| `--force` | Overwrite user-edited managed files / replace hybrid blocks |

The project manifest `.elf/manifest.json` records the ELF version the project uses (`elf_version`), and the `elf` on PATH runs every command in the project with that line ([Versions per project](reference.md#versions-per-project)). Only `elf update` changes it. Within the same line, `elf update` refreshes the managed files at once.

**Changing the line needs the user's approval.** A move to a higher line (a MINOR step, or a new structure generation with a log migration) writes nothing at first: `elf update` prints a summary (the current and target versions, the migration and managed-file plan, the effect on other PCs) and a six-character approval code, and exits 3. An AI agent shows the summary to the user and, only after the user approves, runs `elf update --to <line> --approve <code>`, quotes the approval in the session log, and commits the update by itself. If anything changed in between, the code no longer matches and a new summary is printed. Run in a terminal, `elf update` asks [y/N] instead. Moving to a lower line is refused (undo an update with git), and a migration goes one structure generation at a time. In a project still in the 2.27 layout, `elf update --to 2.27` refreshes the managed files with elf 2.27.x without migrating. A PC with elf 2.28.1 or older has no launcher and needs `elf self update` before pulling a project that was moved to 3.0 or later; `elf check version` shows the verdict.

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

From 2.28, logs live in `2_Log/S/` and `2_Log/G/` with a YAML header and stay where they are ([Sessions and trials](sessions.md)). Updating the CLI does not migrate anything. From 3.0, the `elf` on PATH runs a project that is not migrated yet with elf 2.27.x (downloaded on first use; if it cannot be downloaded, the built-in 2.27 compatibility mode runs instead), so the project keeps recording with the 2.27 commands until you run `elf update` in it. Migrate each project when it is ready; `elf update` migrates once: blockquote headers become the 7 YAML keys, logs move from `2_Log/` and `2_Log/Archive/` to `2_Log/S/`, links are recomputed, deprecation records become `.elf/deprecated/` store files, `Registry.md` and the explorer nesting are generated, and `2_Log/Wiki/Session_Registry.tsv` is removed. The project's own AI agent runs it and handles the warnings.

| Step | What happens |
|------|--------------|
| Precondition | The git working tree must be clean (the migration is undone with git). Otherwise nothing is written |
| Preview | `elf update --dry-run --out <folder>` writes the result files (`files/`), a table of the 7 header values per log with where goal and conclusion came from (`headers.tsv`), the recomputed links and code spans (`changes.tsv`), mentions of old paths and removed files that are not links, scripts included (`mentions.tsv`), and the warnings (`warnings.tsv`). Nothing is written to the project, so there is no need to rehearse on a copy |
| Plan first | `elf update` plans the whole migration. If there is any warning, it writes only the decision file `.elf/runtime/migration.yaml` (not tracked by git), prints a summary, and exits 3; the managed files wait too |
| Decide | Status warnings (header and Registry disagree, both missing, a value that is not canonical) get a `status` field to fill with `ongoing`, `complete` or `deprecated`, judged from the evidence shown and confirmed with the user. Other warnings (a link target that does not exist, two logs with one number) list a file to fix |
| Approve and apply | With the decisions done, `elf update` prints the summary and an approval code. After the user approves, `elf update --to <line> --approve <code>` as the summary shows (for example `--to 3.1`) applies everything at once and deletes the decision file. Then `elf check`, review with the user, and commit |

#### Migration guideline

The guideline gives the direction only. The details differ from project to project and are left to the project's AI agent and the user: how to tidy irregular headers before the migration, what to do with header fields outside the 7 keys, whether a history note should keep the path it recorded, which project scripts to fix, and how to record the migration work itself.

| Direction | Content |
|-----------|---------|
| When | Each project when it is ready. There is no hurry: until the migration the project keeps working with elf 2.27.x |
| Who | The project's AI agent, with the user. The user approves the summary before anything is written |
| Order | Commit the work in progress, preview with `elf update --dry-run --out <folder>` and review it with the user, settle the warnings, approve the summary and apply, then `elf check` and a review of the diff (`git diff -M`) |
| Commit | The migration as a commit of its own, so that `git revert` undoes it. There is no other way back |
| What the migration leaves | Mentions of old log paths and removed files outside links, project scripts, and project rule documents that repeat the old procedures. Use the output and `mentions.tsv` to clean them up in the project; a project rule document should keep only what differs from the managed rules |
| Other PCs | A PC that pulls the migrated project needs elf 3.0 or later (`elf self update`) |

Header values map to the 7 keys (`Created` or `Date` → created, `목표` or `Goal` → goal, `Handoff` → handoff, `결론` or the Registry key finding → conclusion; status `★ 활성` → ongoing, `Complete` → complete, `Deprecated` or `Purged` → deprecated). Other header fields (such as `관련`) are kept as one body line under the title (`> 이관 전 헤더: …`), so their values and links stay. Links and code spans that hold a single relative path are recomputed for the new location when they point to an existing file. Link recomputation and the mention check cover only files tracked by git (untracked copies and scratch folders are left alone). The output lists each changed code span with its file and line: if a span recorded the path as it was at the time (a history note), put the old notation back. The output also lists logs whose conclusion starts with a status word (such as 완료 or 보관), and the files that still mention old log paths or removed files such as `Session_Registry.tsv` (scripts first), which the migration does not edit. A log that an older CLI creates in `2_Log/` after the migration is moved to `2_Log/S/` under the next free number by the next `elf update`. A folder with `2_Log/` but no `.elf/` (logs only) is migrated the same way.

#### From any 2.x version

From 3.0.2, `elf update` brings a project made with any version from 2.0 to 2.28.x to the line of the `elf` on PATH (3.0 or later) in the same steps (summary, approval code, approval). Nothing else is needed per version.

| Made with | What `elf update` also does |
|---|---|
| 2.0 to 2.3 (generator), 2.4.0 | Recognizes the project from the ELF rule files in `0_Meta/` even without `.elf/`, and creates `.elf/` (settings, project manifest). Logs and the Registry in the old log locations (`5_Exp/53_Analysis/Logs/`, `7_Log/`) and old folder names (`9_Archive`, `1_Wiki`, `2_Wiki`) are migrated too |
| 2.14 or earlier | Moves the rules in the old location (`0_Meta/`) and the stubs in a root `templates/` to `.elf/managed/` |
| Any version | Removes ELF files no longer used (`.claudeignore`, the old gallery scripts, old stub copies, and so on) |

An ELF file is replaced or removed only when it is still exactly as that version delivered it. A file you edited is kept; for a moved rule the new version is written as `<file>.elf-new` (the same as Managed below). A `.gitignore` from before the marker block keeps your lines and gets the block at the top. `0_Meta/ProjectRule.md`, `README.md`, `AI_Sync.md` and research folders (including old numbering) are yours and are left as they are. `elf check version` shows the verdict.

## File ownership

`elf update` respects four ownership tiers:

| Tier | Files | On update |
|------|-------|-----------|
| **Managed** | `.elf/managed/` (`EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `LLMcliche.md`, `templates/*`); root `.editorconfig`, `AGENTS.md` | Replaced with the new version. If you edited one, it is **kept** and the new version is written as `<file>.elf-new` (use `--force` to overwrite) |
| **Yours** | `0_Meta/` (`ProjectRule.md`, `<name>.project.md` overlays), `README.md`, all research data and logs | **Never touched** (logs: only the one-time migration above) |
| **Generated** | `2_Log/S/Registry.md`, `2_Log/G/Registry.md`, the `explorer.fileNesting` keys in `.vscode/settings.json` (other settings and nesting patterns you add are yours) | Rebuilt from the log headers by every command that changes a log. The nesting lists closed logs one per line, each linked to the previous one (`"S001_log.md": "S002_log.md"`), and only those keys are rewritten, so closing a session changes one or two lines in the diff |
| **Hybrid** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`) is replaced; your rules outside the block are preserved |
| **Pointer** | `CLAUDE.md` | Created if missing; if present it is **never modified** (no `.elf-new` either). An existing hand-written `CLAUDE.md` stays exactly yours. Add a `@AGENTS.md` line yourself to load the ELF rules; `elf check files` checks the link the way Claude Code parses imports (an `@AGENTS.md` inside backticks, a code block, an HTML comment or quotes is not imported, and is reported as a warning) |

Customize project rules in `ProjectRule.md` (yours) instead of editing managed files, so updates never conflict. For the data files (`LLMcliche.md`, `highIFjournals.md`), add/remove/override entries via a **project overlay** `0_Meta/<name>.project.md` (user-owned; effective rules = base ⊕ overlay; removals need a stated reason, see EliRule §2.7).

## The `.elf/` directory

`elf` maintains `.elf/` (do not edit by hand, except the migration decision file):

- `config.json`: project name, language, preset (project type), creation date
- `manifest.json`: the project manifest, the record of managed files and the ELF version that last updated the project (`elf_version`); before 2.28 the version was in a separate `version` file
- `managed/`: the deployed rule payload (rules, log-format stubs)
- `baseline/`: pristine copies of hybrid files, used to detect edits inside the managed block
- `deprecated/`: compressed originals of deprecated content ([Deprecating content](deprecate.md)); tracked by git
- `runtime/`: temporary tool state (autoread markers, the migration decision file); not tracked by git

---

Back to [README](../README.md)
