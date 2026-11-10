[English](check.md) | [한국어](check.ko.md)

# Checks

## `elf check [version|setup|files|logs]`

Checks in four scopes. They only read, except that `logs` fixes links that are one folder level off ([Fixing links](#fixing-links)). With no scope, all four run in the order version, setup, files, logs (a version mismatch can make the other results wrong, so it comes first) and only problems and fixes are printed: one summary line per scope, its issue, warning and `fixed` lines, and a total. With a scope, every item of that scope is printed as `ok` / `info` / `warn` / `issue` / `fixed`. It does not use the network.

| Scope | Checks |
|---|---|
| `version` | The installed `elf` with its line (MAJOR.MINOR) and structure generation, its install receipt (whether `elf self update` works), the project manifest `.elf/manifest.json` (format and `elf_version`), the project's pin (`elf_version`, the line it resolves to, and the structure generation; 2.28.x counts as the 3.0 line; an earlier-format project, a project of 2.0 to 2.4.0 without `.elf/`, or a log-only folder in the 2.27 layout is read as the 2.27 line), the version store `~/.elf/versions` (location, how many versions, and whether the project's line is run by this CLI, by a stored version, or is not stored yet), and a one-line verdict: same line = ok, project older = run `elf update`, project newer or a newer manifest schema = run `elf self update` (commands that write refuse until then), earlier format = run `elf update` to migrate. Outside a project, the CLI items only |
| `setup` | git repository and `pre-commit` hook, autoread state and `autoread_fulltext` declarations, experimental features enabled in `.elf/config.json`, whether link fixing is on |
| `files` | `.elf/` baseline; each managed file as `ok` / `outdated` / `missing` / `edited`, after the project's `preset:` line (a `config.json`/manifest contradiction is a warning and the diagnosis follows the manifest); a leftover `.claudeignore` (no effect in Claude Code, not ELF-managed since v2.24, safe to delete); the rule set language of a non-Korean project and English copies (`*.en.md`) left from before 3.4; data overlays (`0_Meta/<name>.project.md`: removal entries missing a reason, overlays without an overlayable base); the agent entry (`AGENTS.md` alone is enough for Claude Code 2.1.277 or later; an existing `CLAUDE.md` or `CLAUDE.local.md` must import `@AGENTS.md`, because Claude Code then reads that file instead), excess content in it and pending `*.elf-new` files; tsa when enabled (unstamped manifests, hook state, GPG signing, `openssl`) |
| `logs` | Session logs, below |

**`logs` in detail** (`2_Log/S/` and `2_Log/G/`, each series counted separately, and the links of the other Markdown documents of the project):

| Level | Finding |
|---|---|
| issue | Logs left in the earlier layout in a migrated project (one line naming them; run `elf update`), a header that does not have the 7 keys in order or has a wrong value (with the line number), a log name that is not `S###`/`G###`, a broken relative `.md` link in a log, a deprecation marker without its store file in `.elf/deprecated/` |
| warn | A gap in numbering, `goal` empty while ongoing, a figure in `6_Exp/64_Viz/S###/` that is not embedded in the log (a table path is not an embed), trial structure in ongoing logs (section names and order, the first line of the interpretation (`예상 일치 정도` or `Prediction match`), Phase 1 sections missing while an observation exists; Korean-first `### 조건 (Conditions)` and English `### Conditions` headings are the same section), template text left in a section, `Registry.md` missing or different from the headers, a store file without a marker line, a broken link to any other file in a log, a broken link in another Markdown document of the project (`Archive/` included; folders whose names start with `.` are skipped), one line per document; in ongoing logs, a link or a `### Files` path outside the project (an absolute path outside it, `%VAR%`, `$VAR`, `~/`, or `../` beyond it; inside a git repository the limit is the repository root, the nearest folder above that has `.git`, so a project in a subfolder can cite the other files of the repository) and a `### Files` path that does not exist |
| fixed | Links and `### Files` paths fixed because they were one folder level off ([Fixing links](#fixing-links)), one line per file and a total |
| info | The number of ongoing sessions in a series when there are several (parallel sessions are allowed), a store file not yet tracked by git (commit it together with the log change) |

- In a project not yet migrated, `logs` runs the 2.27 checks (registry and log files, numbering, links, figure embeds, trial structure) and adds an info line about the migration; the earlier layout itself is not an issue.
- Exit code: **4** if there is any issue (use it as a pre-commit hook or CI gate), **0** with warnings only.
- A scope that does not apply is skipped: without `.elf/` (outside an ELF project, or a folder that only keeps `2_Log/`), `files` is skipped and `logs` still runs. Naming a scope that does not apply exits **1**.
- Exclude an intentional non-embed (SI, or a figure kept out of the body on purpose) with a `<!-- noembed: filename.png -->` comment in the log.
- Apart from fixing links, `elf check` reports and never edits. Logs are written by the `elf` commands, which check before writing ([Sessions and trials](sessions.md)).
- Names before 2.26 still work, hidden from `--help`: `elf doctor` (setup and files items with a project manifest line), `elf status [--check]` (managed files), `elf validate [--check]` (logs).

### Fixing links

A document moved into a subfolder (for example `Archive/`) keeps relative links that were written for its old folder. `elf check` and `elf check logs` fix such a link when adding one `../` or removing one finds a file inside the project (or its git repository) and only one of the two does. Only the path changes; the anchor, the title, the line endings and the rest of the file stay as they are. `### Files` paths in ongoing logs that start with `../` are fixed the same way.

- Not fixed, only reported: both directions find a file, the fixed path would leave the project, or no file is found (a deleted or renamed target). A file with the same name elsewhere is not looked for, because it may be a different file.
- `elf trial write`, `elf session close` and the autoread hooks do not fix; they say that `elf check logs` can.
- To only report, set `"fix_links": false` in `.elf/config.json`. Without the key, fixing is on.
- A link is a relative path in `](…)` that ends in a file extension or `/`, with nothing after it except a title in quotes. Text in brackets followed by a remark in parentheses, URLs, anchors, placeholders (`{ } < > *`) and code are not links.
- In `### Files`, a range such as `fig_p1~p5.png` and a folder name alone such as `supp/` are not checked, like a bare file name.

```bash
elf check          # all scopes, problems only
elf check version  # which of elf update / elf self update to run
elf check logs     # session logs in detail (after a figure-producing trial and before closing)
elf check files    # managed files in detail: what `elf update` would change
```

---

Back to [README](../README.md)
