[English](reference.md) | [한국어](reference.ko.md)

# Command reference

## Command summary

| Command | Purpose |
|---------|---------|
| `elf init [name]` | Scaffold an ELF project (in place, or in `./<name>`) |
| `elf update` | Update managed files in the current project; migrate a project made before 2.28 to the new log layout |
| `elf self update` | Add the newest release to the version store and make it the default (projects keep their own versions); then clean the store |
| `elf self list [--available]` | List the versions kept in the version store `~/.elf/versions` (`--available`: the newest release of each line, from GitHub) |
| `elf self install <version>` | Download a release into the version store after checking its sha256: `2.27` = the newest patch of that line, `2.27.4` = that release; then clean the store (the version just installed is kept) |
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

Every installed version has its own folder, `~/.elf/versions/vX.Y.Z/`. The `elf` on PATH (`~/.elf/bin/elf`) is a launcher: it picks a version and runs that version's executable with the same arguments, output, and exit code, and does no work itself. So updating `elf` adds a folder and changes the default version, and never touches the files a project in another window is running.

- **Inside a project**, the version comes from the project manifest (`elf_version`, written only by `elf init` and `elf update`): the newest stored patch of the project's line (MAJOR.MINOR; 2.28.x counts as 3.0), whether that line is older, the same, or newer than the default. The first command in such a project downloads that release after checking its sha256. Once a day `elf` asks GitHub which release is the latest (the `releases/latest` redirect, a few kilobytes); when it is a newer patch of the project's line, it is downloaded. Releases follow one line at a time, so an older line gets no new patches and its stored version keeps running. The full release list is read only when an older line that is not stored is needed for the first time; when the project's recorded version or line is missing from the list fetched earlier that day, the list is fetched again (at most once an hour), and if the list still cannot settle it, the recorded version is downloaded directly.
- **Outside a project**, and for `elf self …`, `elf init`, `elf update`, `elf check version`, `--help` and `--version`, the default version runs (`~/.elf/current`).
- A project on an older line of the same structure generation gets a note once a day that `elf update` can move it to the default line after the user approves. When its version cannot be obtained, the default version runs it as is (nothing is refused) and the note says so.
- A project in the 2.27 layout without a recorded version runs with the newest 2.27 release; if that cannot be downloaded, the built-in 2.27 compatibility mode runs instead. When a newer line than the default cannot be obtained, commands that write refuse (exit 3) and read-only commands run with a warning; `elf self update` resolves it.
- The first time the launcher runs (for example right after updating from 3.8), it copies itself into the version store and writes `~/.elf/current`.
- `elf check version` shows the default version, the launcher, and which version runs the project.

| Variable | Meaning |
|----------|---------|
| `ELF_DISPATCH` | `off`: never hand over to another version (tests, CI, diagnosis). `trace`: print each decision to standard error |
| `ELF_HOME` | Folder that holds `bin/`, `versions/` and `current` (default `~/.elf`) |
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
