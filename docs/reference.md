[English](reference.md) | [한국어](reference.ko.md)

# Command reference

## Command summary

| Command | Purpose |
|---------|---------|
| `elf init [name]` | Scaffold an ELF project (in place, or in `./<name>`) |
| `elf update` | Update managed files in the current project; migrate a project made before 2.28 to the new log layout |
| `elf self update` | Update the `elf` binary itself |
| `elf self list [--available]` | List the versions kept in the version store `~/.elf/versions` (`--available`: the newest release of each line, from GitHub) |
| `elf self install <version>` | Download a release into the version store after checking its sha256: `2.27` = the newest patch of that line, `2.27.4` = that release |
| `elf self uninstall <version>`, `elf self prune` | Remove one stored version, or keep only the newest version of each line |
| `elf session new "<title>" --goal "<goal>" [--gov]` | Create the next session log in `2_Log/S/` (or `2_Log/G/`) |
| `elf session close [S###\|G###] --from <file>` | Close a session with its conclusion (status complete; the file is not moved) |
| `elf trial new "<title>" [--from <file>]` | Create the next trial (Phase 1 template, or Phase 1 from a file) |
| `elf trial write [tNN] --from <file>` | Write Phase 1 (revise until Phase 2 exists) or Phase 2 (once) from a file |
| `elf log`, `elf log show <ID> [tNN]` | List session logs; show a header and its trials, or one trial |
| `elf check [version\|setup\|files\|logs]` | Read-only checks of versions, setup, managed files, and session logs (no scope = all; exit 4 on issues) |
| `elf deprecate [target] …` | Remove content from a session log or planning document and keep the original compressed in `.elf/deprecated/`; `show`, `--restore`; no arguments = list |
| `elf gallery` | Generate the figure index `_gallery.md` from `6_Exp/64_Viz/` |
| `elf tsa [sub]` | Research-record timestamping, **opt-in**: per-commit hash manifest + RFC 3161 token (no subcommand = request today's token) |
| `elf autoread [sub]` | Governance digest for session start and after a context reconstruction: print it (any harness); Claude Code hooks inject it automatically (default-on) |

Names before 2.26 still work and are hidden from `--help`: `elf doctor`, `elf status`, `elf validate` (see `elf check`), `elf self-update`, `elf selfupdate`, `elf update --self` (see `elf self update`), `elf deprecate list`. `elf session fix-headers` and `elf deprecate purge` were removed in 2.28.

Global flags: `elf --version`, `elf --help`, `elf <command> --help`.

## Versions per project

The project manifest records the ELF version the project uses (`elf_version`, written only by `elf init` and `elf update`). The `elf` on PATH reads it and, when the project's line (MAJOR.MINOR; 2.28.x counts as 3.0) is higher than its own or in another structure generation, runs the newest stored patch of that line from the version store `~/.elf/versions` with the same arguments, output, and exit code. The first command in such a project downloads that release after checking its sha256, and once a day `elf` looks for a newer patch of the line. When the project's recorded version or line is missing from the release list fetched earlier that day, the list is fetched again (at most once an hour), and if the list still cannot settle it, the recorded version is downloaded directly. `elf self …`, `elf init`, `elf update`, and `elf check version` always run with the `elf` on PATH. A project in the 2.27 layout without a recorded version runs with the newest 2.27 release; if that cannot be downloaded, the built-in 2.27 compatibility mode runs instead. A project on a lower line of the same structure generation runs with the `elf` on PATH as is: nothing is downloaded or refused, its ELF files stay on their line, and once a day a note says that `elf update` can move it (after the user approves). When a higher line cannot be obtained, commands that write refuse (exit 3) and read-only commands run with a warning; `elf self update` resolves it. `elf check version` shows which `elf` runs the project.

| Variable | Meaning |
|----------|---------|
| `ELF_DISPATCH` | `off`: never hand over to another version (tests, CI, diagnosis). `trace`: print each decision to standard error |
| `ELF_HOME` | Folder that holds `versions/` (default `~/.elf`) |
| `ELF_RELEASES` | Base address or local folder of the release assets (testing) |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error (I/O, not an ELF project, a session or ID that does not exist, …) |
| 2 | Usage error (bad flags or arguments) |
| 3 | Refused, nothing written: a file that fails the checks (all reasons are printed), a project newer than the CLI, a write in a project whose higher ELF line cannot be obtained, a command of the new structure in a project not yet migrated, a migration that needs decisions (the decision file is written) |
| 4 | `elf check` found issues (also `--check` on the old `status` and `validate` names). A gate signal, distinct from a real error |

Exit 5 (escalation) means the TSV session registry of a project not yet migrated cannot be parsed (the 2.27 commands, which such a project keeps using). Migrated projects have no TSV registry, so exit 5 does not occur there.

---

Back to [README](../README.md)
