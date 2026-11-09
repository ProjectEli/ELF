[English](tsa.md) | [한국어](tsa.ko.md)

# Timestamping research records (opt-in)

## `elf tsa [subcommand]` (opt-in)

Research-record timestamping. It proves what existed and when. Once enabled, every commit's file hashes are appended to a daily manifest (`0_Meta/tsa/YYYY-MM-DD_manifest.json`, pre-commit hook), and the day's first commit requests an RFC 3161 timestamp token from a TSA (post-commit hook; `.tsr` saved next to the manifest). It is off by default. Nothing runs until you enable it, and most researchers can ignore it entirely.

**Privacy**: the only data that leaves your machine is one 32-byte sha256 digest of the manifest file. File contents and file names are never sent. **Authorship** (GPG commit signing), the optional third layer, belongs to git itself and is outside this command. `elf tsa enable` and `elf check files` print how to turn it on.

| Subcommand | Effect |
|---|---|
| *(none)* `[--all] [--quiet]` | Request timestamp tokens for unstamped manifests (default: today only; `--all` = every unstamped manifest, which fills in manifests left unstamped by offline commits). The post-commit hook calls this. A non-granted response is discarded, never saved as evidence |
| `enable` | Idempotent setup: `"tsa": true` in `.elf/config.json`, creates `0_Meta/tsa/`, installs both hooks **non-destructively** (a foreign hook is left untouched, with the exact line to add manually), prints GPG guidance, then takes a **baseline record** (records all tracked files + stamps). Retroactive proof of the past is impossible by design. The baseline proves "this existed by the day I enabled" |
| `disable` | Config off + removes only marker-owned hooks (a foreign hook survives). **Evidence under `0_Meta/tsa/` is never deleted.** Re-enabling appends to the existing evidence |
| `status` | Enabled state, hook ownership, evidence counts, unstamped manifests (read-only) |
| `record [--all] [--quiet]` | Hash staged files (default, the pre-commit hook path) or all tracked files (`--all`, baseline) into today's manifest (raw-byte sha256, duplicates skipped). The manifest itself is auto-staged so it is included in the same commit |
| `verify <file>` | Look up the file's current sha256 across all manifests to answer "when did this exact content exist?" |
| `verify --date <D>` | Lightweight manifest↔token check (granted status, messageImprint match, genTime), then prints the `openssl ts -verify` command for full signature-chain verification (rare path, for disputes/audits) |

Defaults: DigiCert's free public TSA (`http://timestamp.digicert.com`, no account or credentials; override via `"tsaUrl"` in `.elf/config.json`). Requires git. Commits are the recording unit. Hooks never block a commit (token requests run in the background; failures are filled in later with `elf tsa --all`). Everything except token requests works fully offline. `elf update` rewrites the ELF-owned tsa hooks to the current commands; when it replaces hooks written before 2.26, it prints the last manifest date and the number of commits after that day, so each project can decide whether to run `elf tsa record --all` and `elf tsa --all`. Hooks without the ELF marker (for example a hook shared with another tool) are never edited; `elf check files` and `elf tsa status` warn when such a hook still calls a removed command (`tsa stamp`, `tsa record --staged`, `--backfill`, `validate --strict`) and name the replacement (`tsa`, `tsa record`, `--all`, `check logs`).

```bash
elf tsa enable            # once per project: hooks + baseline from here on
git commit …              # recording + stamping now happen automatically
elf tsa verify paper.md   # existence history of this exact content
elf tsa --all             # after offline days
elf tsa disable           # stops recording; evidence stays
```

---

Back to [README](../README.md)
