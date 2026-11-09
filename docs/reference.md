[English](reference.md) | [한국어](reference.ko.md)

# Command reference

## Command summary

| Command | Purpose |
|---------|---------|
| `elf init [name]` | Scaffold an ELF project (in place, or in `./<name>`) |
| `elf update` | Update managed files in the current project; migrate a project made before 2.28 to the new log layout |
| `elf self update` | Update the `elf` binary itself |
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

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error (I/O, not an ELF project, a session or ID that does not exist, …) |
| 2 | Usage error (bad flags or arguments) |
| 3 | Refused, nothing written: a file that fails the checks (all reasons are printed), a project newer than the CLI, a command of the new structure in a project not yet migrated, a migration that needs decisions (the decision file is written) |
| 4 | `elf check` found issues (also `--check` on the old `status` and `validate` names). A gate signal, distinct from a real error |

Exit 5 (escalation) means the TSV session registry of a project not yet migrated cannot be parsed (the 2.27 commands, which such a project keeps using). Migrated projects have no TSV registry, so exit 5 does not occur there.

---

Back to [README](../README.md)
