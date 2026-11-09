[English](sessions.md) | [한국어](sessions.ko.md)

# Sessions and trials

## Where logs live

| Folder | Holds |
|--------|-------|
| `2_Log/S/` | Research sessions `S###_log.md` and the generated `Registry.md` |
| `2_Log/G/` | Governance sessions `G###_log.md` (changes to rules, structure, templates, or tool settings that apply to all later work) and their `Registry.md` |

A log stays where it was created. Closing or deprecating a session changes the `status` in its header, not its folder, so links to a log never break. `Registry.md` lists the sessions of its folder (ongoing first, then by number, newest first) with the status and the conclusion or handoff. Every `elf` command that changes a log regenerates it; do not edit it by hand. In VS Code, closed and deprecated logs fold under `Registry.md` in the explorer (`.vscode/settings.json` file nesting, kept by `elf`).

## The header

Each log starts with a YAML header of seven keys, always in this order:

```yaml
---
created: 2026-11-09
modified: 2026-11-09
status: ongoing
goal: 'Compare SNR across wavelengths'
handoff: ''
conclusion: ''
user_note: ''
---
# S001: Wavelength comparison
```

`status` is `ongoing`, `complete`, or `deprecated`. Free text is single-quoted, with `''` for a quote inside. The header is written by `elf` commands only: `session new` writes the goal, the YAML block of a `--from` file replaces the handoff, and `session close` writes the conclusion and the user's note. `elf check logs` reports a header that was edited into a wrong shape.

## `elf session new "<title>" --goal "<goal>" [--gov]`

Create the next session log with the header, the title line, and the rule comment. `--goal` is required (one or two sentences). `--gov` creates a governance session in `2_Log/G/`. Numbers count separately per folder (S001…, G001…). `elf init` creates no session; the first one comes from this command.

```bash
elf session new "Wavelength comparison" --goal "Compare SNR across wavelengths"
# → created 2_Log/S/S001_log.md (S001) + Registry.md
```

## Trials: `elf trial new` and `elf trial write`

A trial is written in two phases: Phase 1 (목표, 조건, 가설, 예상) before running, Phase 2 (관찰, 해석, 교훈, 생성 파일) after running. Sections are written to a file with an editor or the AI agent's file-writing tool and passed with `--from`, so nothing goes through the shell.

| Command | Does |
|---------|------|
| `elf trial new "<title>"` | Creates the next trial (`t01`, `t02`, …) with the Phase 1 template |
| `elf trial new "<title>" --from <file>` | Creates the trial and writes Phase 1 from the file (same as `new`, then `write`) |
| `elf trial write [tNN] --from <file>` | Writes the sections in the file into the trial: Phase 1 sections replace the current ones (any number of times until Phase 2 exists), Phase 2 sections are written once. With no `tNN`, the last trial of the session |

`--session <S###|G###>` picks the session when several are ongoing.

```bash
elf trial new "3-wavelength sweep" --from phase1.md   # t01, Phase 1
elf trial write --from phase1.md                     # revise Phase 1 after the user's feedback
elf trial write t01 --from phase2.md                 # after running: Phase 2
```

A file holds the sections of one phase. A YAML block at the top of the file (`handoff` only) replaces the header handoff. Exact synonyms of a section name (for example `### Goal`) are corrected and reported. Everything is checked before writing; on a problem nothing is written and the command exits 3 with all the reasons:

- a missing or out-of-order section, sections of both phases in one file, or template text left in a section
- revising Phase 1 after outputs of the session appeared in `6_Exp/` (a change that reflects results is a new trial), or after Phase 2 exists
- Phase 2 while a figure created after the last Phase 1 write in `6_Exp/64_Viz/S###/` is not embedded (embed it in 관찰, or mark it `<!-- noembed: name.png -->`)

The tool keeps a comment under the trial title with the times of the Phase 1 writes and of Phase 2. Stop after Phase 1 and get the user's confirmation before running; the commands do not ask for it.

## `elf session close [S###|G###] --from <file>`

Close a session after the user has approved its conclusion. The file holds a YAML block:

```yaml
---
conclusion: '810 nm gives the highest SNR'
user_note: ''
handoff: ''
---
```

`conclusion` is required; without it nothing is written (exit 3). `user_note` is the user's own words only. The status becomes `complete`, `Registry.md` is regenerated, and the file is not moved. Close also runs the `elf check logs` checks for this session and prints the findings (non-blocking), including pending items left in the handoff. With no id, the single ongoing session is closed.

## `elf log` and `elf log show`

| Command | Shows |
|---------|-------|
| `elf log` | One line per session: id, status, last change, title, and the conclusion or handoff |
| `elf log show S012` | The header and the list of trials with their phase |
| `elf log show S012 t03` | The text of one trial |

Both are read-only.

## `elf gallery`

Scan `6_Exp/64_Viz/` and regenerate `6_Exp/64_Viz/_gallery.md`, a figure index grouped by session folder. If `6_Exp/64_Viz/` does not exist, it prints a notice and exits 0.

## Projects made before 2.28

Logs in `2_Log/`, `2_Log/Archive/`, or `2_Log/Deprecated/` with a blockquote header, and `2_Log/Wiki/Session_Registry.tsv`, are the earlier layout. Commands that write logs refuse until `elf update` migrates them; see [Creating and updating projects](projects.md#migrating-the-log-layout-228).

---

Back to [README](../README.md)
