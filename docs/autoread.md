[English](autoread.md) | [한국어](autoread.ko.md)

# Rules after a context rebuild (autoread)

## `elf autoread [subcommand]`

Governance re-injection after a context reconstruction. When an AI agent's context is compacted, resumed, or cleared, the agent continues from a lossy summary and the project rules are no longer in the context. `elf autoread` prints a **digest** that puts them back: the standing-duties section of `AGENTS.md`, the §0 core summary of the log format rules (`.elf/managed/LogConvention.md`), the handoff of each ongoing session (truncated; governance sessions first), the current `elf check logs` counts, and a closing instruction to act on the full rule texts rather than the summary. A new session receives the same digest at its start. With no subcommand it prints the digest to stdout, which works with any agent harness (paste or pipe it into the next prompt).

**Claude Code integration (automatic).** `elf init` and `elf update` keep two minimal hook entries in `.claude/settings.json` (the file is untracked; the merge preserves everything else in it): a `SessionStart` hook records a per-session marker under `.elf/runtime/` when a session starts or the context is rebuilt (compaction, resume, clear; nothing is injected at that point), and a `UserPromptSubmit` hook injects the digest on the next prompt. `elf update` replaces hooks installed by an earlier version with the current definition; until then, `elf check setup` warns about them. Until that prompt arrives, every `elf` command run in that session prints a one-line reminder banner as a fallback channel (the session is identified by `CLAUDE_CODE_SESSION_ID`, which Claude Code sets for Bash commands and hooks; other sessions in the same project are not affected). Hook paths are strictly fail-open: any internal error exits 0 with no output and never blocks the session. The managed block of `.gitignore` excludes `.elf/runtime/`. Claude Code reloads the root `CLAUDE.md` after a reconstruction, so when it imports `AGENTS.md` the digest Claude Code receives replaces the standing-duties section with a one-line note instead of a second copy; without a working import the section is included in full.

| Subcommand | Effect |
|---|---|
| *(none)* | Print the digest (any harness) and clear the pending reconstruction marker of the current Claude Code session (outside Claude Code, all pending markers) |
| `enable` | `"autoread": true` in `.elf/config.json` + Claude Code hook entries. Idempotent, per project. An absent key means on |
| `disable` | `"autoread": false`. Hook entries stay in place but become no-ops |
| `status` | Config, hook installation, `autoread_fulltext` declarations (ok / missing / unsafe), pending markers (read-only) |

**`autoread_fulltext` (opt-in).** List root-relative paths (e.g. your log convention) in the `autoread_fulltext` array of `.elf/config.json`; after a reconstruction the digest then carries those files verbatim, so the rules it summarizes are deterministically back in context. Paths that escape the project tree are refused, and a missing or unsafe entry degrades to a one-line note. The whole digest stays under 10,000 characters, because Claude Code passes only the first 2,000 characters of a hook output that exceeds 10,000. Declared files therefore get only the room that is left; a file that does not fit is truncated or left out with a notice. The §0 core summary of the log format rules is always included, so declaring the full `LogConvention.md` is not needed. The switch (`autoread`) and the list are separate keys. `elf check setup` reports the autoread state and warns about declarations that point nowhere or do not fit within 10,000 characters.

```bash
elf autoread              # print the digest now
elf autoread status       # config, hooks, fulltext declarations, pending markers
elf autoread disable      # per-project off (hooks remain as no-ops)
```

> Do not run the installed hooks with an `elf` binary older than 2.19: it exits with code 2 on the unknown subcommand, which Claude Code treats as a blocking hook error on prompt submission. Normal installs and updates are unaffected. Only avoid downgrading `elf` after hooks are installed.

---

Back to [README](../README.md)
