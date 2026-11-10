[English](check.md) | [한국어](check.ko.md)

# Checks

## `elf check [version|setup|files|logs]`

Read-only checks in four scopes. With no scope, all four run in the order version, setup, files, logs (a version mismatch can make the other results wrong, so it comes first) and only problems are printed: one summary line per scope, its issue and warning lines, and a total. With a scope, every item of that scope is printed as `ok` / `info` / `warn` / `issue`. It does not use the network.

| Scope | Checks |
|---|---|
| `version` | The installed `elf` with its line (MAJOR.MINOR) and structure generation, its install receipt (whether `elf self update` works), the project manifest `.elf/manifest.json` (format and `elf_version`), the project's pin (`elf_version`, the line it resolves to, and the structure generation; 2.28.x counts as the 3.0 line; an earlier-format project, a project of 2.0 to 2.4.0 without `.elf/`, or a log-only folder in the 2.27 layout is read as the 2.27 line), the version store `~/.elf/versions` (location, how many versions, and whether the project's line is run by this CLI, by a stored version, or is not stored yet), and a one-line verdict: same line = ok, project older = run `elf update`, project newer or a newer manifest schema = run `elf self update` (commands that write refuse until then), earlier format = run `elf update` to migrate. Outside a project, the CLI items only |
| `setup` | git repository and `pre-commit` hook, autoread state and `autoread_fulltext` declarations, experimental features enabled in `.elf/config.json` |
| `files` | `.elf/` baseline; each managed file as `ok` / `outdated` / `missing` / `edited`, after the project's `preset:` line (a `config.json`/manifest contradiction is a warning and the diagnosis follows the manifest); a leftover `.claudeignore` (no effect in Claude Code, not ELF-managed since v2.24, safe to delete); language companions; data overlays (`0_Meta/<name>.project.md`: removal entries missing a reason, overlays without an overlayable base); the `CLAUDE.md` → `@AGENTS.md` pointer, excess pointer content and pending `*.elf-new` files; tsa when enabled (unstamped manifests, hook state, GPG signing, `openssl`) |
| `logs` | Session logs, below |

**`logs` in detail** (`2_Log/S/` and `2_Log/G/`, each series counted separately):

| Level | Finding |
|---|---|
| issue | Logs left in the earlier layout in a migrated project (one line naming them; run `elf update`), a header that does not have the 7 keys in order or has a wrong value (with the line number), a log name that is not `S###`/`G###`, a broken relative `.md` link, a deprecation marker without its store file in `.elf/deprecated/` |
| warn | A gap in numbering, more than one ongoing session in a series, `goal` empty while ongoing, `conclusion` empty while complete, pending items left in the handoff while complete, a figure in `6_Exp/64_Viz/S###/` that is not embedded in the log (a table path is not an embed), trial structure in ongoing logs (section names and order, the `예상 일치 정도` first line of 해석, Phase 1 sections missing while 관찰 exists), template text left in a section, `Registry.md` missing or different from the headers, a store file without a marker line |
| info | A store file not yet tracked by git (commit it together with the log change) |

- In a project not yet migrated, `logs` runs the 2.27 checks (registry and log files, numbering, links, figure embeds, trial structure) and adds an info line about the migration; the earlier layout itself is not an issue.
- Exit code: **4** if there is any issue (use it as a pre-commit hook or CI gate), **0** with warnings only.
- A scope that does not apply is skipped: without `.elf/` (outside an ELF project, or a folder that only keeps `2_Log/`), `files` is skipped and `logs` still runs. Naming a scope that does not apply exits **1**.
- Exclude an intentional non-embed (SI, or a figure kept out of the body on purpose) with a `<!-- noembed: filename.png -->` comment in the log.
- `elf check` reports and never edits. Logs are written by the `elf` commands, which check before writing ([Sessions and trials](sessions.md)).
- Names before 2.26 still work, hidden from `--help`: `elf doctor` (setup and files items with a project manifest line), `elf status [--check]` (managed files), `elf validate [--check]` (logs).

```bash
elf check          # all scopes, problems only
elf check version  # which of elf update / elf self update to run
elf check logs     # session logs in detail (after a figure-producing trial and before closing)
elf check files    # managed files in detail: what `elf update` would change
```

---

Back to [README](../README.md)
