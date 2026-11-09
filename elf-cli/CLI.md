[English](CLI.md) | [한국어](CLI.ko.md)

# elf CLI Reference

`elf` is the command-line tool for ELF (Eli's Lab Framework): it scaffolds research projects, keeps framework files up to date, and diagnoses drift. It ships as a self-contained single binary, with no Node or Python runtime required.

The reference is split by topic. `elf --help` and `elf <command> --help` print the same commands in the terminal.

| Topic | Doc |
|---|---|
| Installing and updating the CLI | [docs/install.md](../docs/install.md) |
| `elf init`, `elf update`: presets, languages, file ownership, `.elf/`, log layout migration, target line (`--to`) and the user's approval for a change of line | [docs/projects.md](../docs/projects.md) |
| `elf session`, `elf trial`, `elf log`, `elf gallery` | [docs/sessions.md](../docs/sessions.md) |
| `elf check` | [docs/check.md](../docs/check.md) |
| `elf deprecate` | [docs/deprecate.md](../docs/deprecate.md) |
| `elf autoread` | [docs/autoread.md](../docs/autoread.md) |
| `elf tsa` (opt-in) | [docs/tsa.md](../docs/tsa.md) |
| Command summary, exit codes | [docs/reference.md](../docs/reference.md) |

Framework overview and quickstart: [README.md](../README.md).
