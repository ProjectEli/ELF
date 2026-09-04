# Changelog

User-facing highlights for the `elf` CLI — new features, new options, and changes
that affect your projects. (Exhaustive internal history is kept separately by the
maintainer.) The matching section is shown on each GitHub Release.

## [2.21.0] - 2026-09-04

### Removed
- **Token-economy rule removed (retroactive).** EliRule §3 no longer contains the
  "토큰 최소화 및 압축 (Token Economy)" rule (old #8) — removed from the research and
  `general` presets and their English companions, with the remaining rules renumbered
  (old 9–13 → 8–12). LogConvention's log-writing rule now asks for nominal endings only,
  dropping the token-minimization rationale and the word-centered-bullet mandate.
  Retroactive: existing projects pick the updated rule files up via `elf update`;
  logs already written need no rewrite.

## [2.20.0] - 2026-08-22

### Changed
- **Next-session section turned off.** New session logs no longer include the `## 다음 세션 후보` (next-session hypothesis) section, and `elf session close` no longer refuses when it is missing (`--force` is still accepted and has no effect). Record follow-ups in the header `Handoff`, the registry key finding, or a planning document instead.

### Added
- The previous session template is preserved as `.elf/managed/templates/Archive/sessionTemplate_v1.md` for backward compatibility; existing logs that contain the section stay valid and `elf trial new` still inserts new trials before it.

## [2.19.0] - 2026-07-16

Promotes the 2.19.0-beta autoread feature to stable, adding a full-text layer on top
of the digest: field use showed that receiving the digest alone does not reliably get
an agent to re-open the full rule documents it summarizes.

### Added
- **`autoread_fulltext` — declared canonical rules injected in full.** Projects can list
  root-relative paths (e.g. their log convention) in the `autoread_fulltext` array in
  `.elf/config.json`; after a context reconstruction the digest then carries those files
  verbatim, so the rules the digest merely summarizes are deterministically back in
  context. Opt-in: with nothing declared, behavior stays digest-only. What to declare is
  the project's call — the CLI never guesses relevance. Safety: paths that escape the
  project tree are refused, each file is capped (24k chars, truncated with a notice),
  and a missing or unsafe declaration degrades to a one-line note (fail-open, as with
  all hook paths). `elf autoread status` shows each declaration (ok/missing/unsafe) and
  `elf doctor` warns about declarations that point nowhere. The switch (`autoread`) and
  the list are separate keys, so enable/disable round-trips never touch the list.
- **Imperative closing instruction in the digest.** The injected text now ends with an
  explicit directive — apply the included full texts (never act on the compact summary
  alone), or, when nothing is declared, read the task-relevant canonical rules under
  `0_Meta/` in full before the first substantive action.

### Changed
- **AGENTS templates: the "after a context rebuild" standing duty now includes
  rule-state recovery** — re-read the task-relevant canonical rules in full (auto-included
  in the digest when declared via `autoread_fulltext`) in addition to the existing
  Handoff re-read and `elf validate` check.

## [2.19.0-beta] - 2026-07-16

Pre-release for field validation of the new autoread feature. `releases/latest`
(the installer one-liner and `elf self-update`) never picks pre-releases, so regular
installs are unaffected — install this tag explicitly to try it.

### Added
- **`elf autoread` — governance re-injection after context reconstruction (default-on).**
  When an AI coding session's context is compacted (or resumed/cleared), the summary the
  agent continues from is a lossy digest — project rules quietly drift out of view. With
  Claude Code, elf now detects the reconstruction (a `SessionStart` hook records a
  per-session marker; nothing is injected at that point) and injects a governance digest
  on the next prompt (`UserPromptSubmit` hook): the standing-duties section of AGENTS.md,
  active session headers (Handoff, truncated), and current `elf validate` counts. Until
  that prompt arrives, every `elf` command prints a one-line reminder banner as a fallback
  channel. Hook paths are strictly fail-open: any internal error exits 0 with no output
  and never blocks the session.
- **Per-project switch, on by default.** The `autoread` key in `.elf/config.json` defaults
  to on when absent; `elf autoread enable`/`disable` writes it explicitly (disable leaves
  the hook entries in place but makes them no-ops). `elf autoread` with no arguments
  prints the digest manually (works in any harness); `status` and `ack` are included.
  Hooks live in `.claude/settings.json` as two thin entries — the logic stays in the
  binary. `elf init` and `elf update` keep the entries present (the file is untracked;
  `elf update` restores it after a fresh clone), the merge preserves everything else in
  the file, and `elf doctor` reports the state.
- The managed block of `.gitignore` now excludes `.elf/runtime/` (session-scoped markers).

### Known caveat (beta)
- Do not run the installed hooks with an elf binary older than 2.19: pre-2.19 binaries
  exit with code 2 on the new subcommand, which Claude Code treats as a blocking hook
  error on prompt submission. Normal installs and updates are unaffected — only avoid
  downgrading elf after hooks are installed.

## [2.18.3] - 2026-07-15

### Added
- **One writer per session (multi-agent rule).** EliRule gains a session-ownership section
  (research §2.8, general §2.5; KO + EN): every session log has exactly one writer — an
  agent or a person. Concurrent writers silently overwrite each other in trial insertion
  and header Handoff updates, so parallel work means one session per agent, with
  relationships recorded in the header `관련:` (related) field rather than in session
  numbers. Other agents' logs are read-only; consolidation happens in a single-writer
  session. Run `elf update` to receive the new rule text.
- **`elf validate` now flags malformed session log names.** A file named like
  `S201-a_log.md` (any `*_log.md` that is not plain `S###`) used to be silently ignored by
  every scan — invisible to the registry, numbering, and validation. It is now reported as
  an issue in both `2_Log/` and `2_Log/Archive/`.

### Fixed
- **Concurrent `elf session new` no longer loses sessions or registry rows.** Two agents
  running `session new` at the same moment could compute the same number, silently
  overwrite each other's log file, and drop a registry row (the registry was rewritten
  whole). Numbers are now allocated atomically — creating the log file with an exclusive
  create is the reservation, retrying on collision — and registry rows are appended
  instead of rewritten. Verified with 8 threads starting simultaneously: all unique
  numbers, no lost rows.

## [2.18.2] - 2026-07-15

### Fixed
- **`elf session new` no longer aborts on a damaged registry.** Previously, if the session
  registry had a malformed row or a git merge-conflict marker, `session new` stopped with an
  escalation even though the session log files were intact — a problem when several agents run
  in parallel and the registry is momentarily broken. Session numbers are now taken from the
  log-file index (`S###_log.md`) as the source of truth; a broken registry only produces a
  warning ("run `elf validate` to repair") and the new row is appended as text, so nothing is
  lost. `session close` and `elf validate` still require a well-formed registry — closing or
  diagnosing against a broken registry would spread the damage.

## [2.18.1] - 2026-07-15

### Fixed
- **`elf tsa record` silently skipped files with non-ASCII paths.** With git's default
  `core.quotepath=true`, paths containing non-ASCII characters (Korean, CJK, accented
  Latin, …) are octal-escaped and quoted in `git ls-files`/`diff` output. The parser took
  the quoted string literally, so the read failed and the file was dropped from the manifest
  **with no warning** — a silent integrity gap (the tool looked like it recorded everything).
  Fixed by disabling path quoting on all git calls and reading `record` output NUL-delimited
  (`-z`), which also makes spaces and newlines in filenames safe. **If you enabled `elf tsa`
  before this release, run `elf tsa record --all` once** to capture any files that were
  previously skipped.

## [2.18.0] - 2026-07-15

### Added
- **`elf tsa` — opt-in research-record timestamping.** Once enabled, every commit's file
  hashes are appended to a daily manifest (`0_Meta/tsa/`, pre-commit hook), and the day's
  first commit requests an RFC 3161 timestamp token from a TSA (post-commit hook, saved
  next to the manifest) — verifiable proof that this exact content existed by that day.
  Off by default; nothing runs until `elf tsa enable`.
  - `enable` is idempotent and **non-destructive**: an existing foreign hook is left
    untouched (the exact line to add is printed instead), and it finishes with a baseline
    seal of all tracked files — retroactive proof of the past is impossible by design, so
    the baseline anchors "existed by the day I enabled".
  - `disable` removes only elf-owned hooks and **never deletes evidence** under
    `0_Meta/tsa/` — re-enabling resumes on top of it.
  - `record --staged|--all`, `stamp [--backfill]` (catches up after offline days),
    `verify <file>` (existence history of the exact content), `verify --date <D>`
    (manifest↔token check + the `openssl ts -verify` command for full signature-chain
    verification). Hooks never block a commit; stamping runs in the background.
  - Privacy: the only bytes that ever leave your machine are one 32-byte manifest digest
    per day — no file contents, no file names. Default TSA is DigiCert's free public
    endpoint (no account; override with `"tsaUrl"` in `.elf/config.json`). Requests are
    built in pure Rust — `openssl` is only needed for the rare full chain verification.
  - GPG commit signing (the optional authorship layer) stays git-owned; `elf tsa enable`
    and `elf doctor` print how to turn it on.
- `elf status` shows a tsa summary line and `elf doctor` gains tsa checks when the
  feature is enabled (both stay silent otherwise). See CLI.md for the full reference.

## [2.17.2] - 2026-07-14

### Changed
- **LLMcliche — `provenance` is now excluded outright.** The §4 data-lineage exception
  introduced in 2.17.1 is removed: use *source/origin* or a concrete term such as
  *data lineage* in every context, including data-lineage writing. The §1 entry now says
  "no exception". meta and general presets, plus the EN companion. Run `elf update`.

## [2.17.1] - 2026-07-14

### Added
- **LLMcliche — three cliché nouns added**: the §1 list gains `provenance` (inflated
  nominalization of "source" — use *source/origin*; the data-lineage standard use — data
  provenance, W3C PROV, reproducibility records — stays allowed via a matching §4
  exception), `dossier` (non-academic elevation of a compilation — use *record/summary*),
  and `caveat` (cliché hedging noun — use *limitation/note*), with a before/after example.
  Promoted from a project overlay (`0_Meta/LLMcliche.project.md`) — the first field use of
  the overlay-to-core promotion path. meta and general presets, plus the EN companion. Run
  `elf update`.

### Fixed
- The LogConvention initiator rule itself used `provenance` — reworded in all four
  deployed variants (research/general × KO/EN) so the canon does not use a listed word.

## [2.17.0] - 2026-07-13

### Added
- **Figure-embed discipline is now injected at the point of action.** A standing rule in
  the loaded context guarantees presence, not activation at the right moment — so the rule
  now travels with the workflow:
  - `elf trial new` output reminds you to list expected figures in Phase 1 and embed each
    one into `### 관찰 (Observation)` the moment it is created (sub-agent outputs included).
  - `elf session close` runs `elf validate` automatically before archiving and reports the
    findings **scoped to the session being closed** (missing embeds, trial-structure
    warnings) — non-blocking, but it is the last check before the log leaves the
    active-structure scope by moving to `Archive/`.
- **Convention updates** (research preset unless noted):
  - **Pre-register expected outputs**: a figure-producing trial lists its expected figures/
    tables in the Phase-1 `### 예상 (Prediction)` section — each item is an
    embed-on-creation target, and "draw everything, tidy up later" batching is prohibited
    (deferred batches get lost on interruption or a context rebuild).
  - **Sub-agent figures**: figures produced by a delegated worker must be embedded by the
    **main** agent at retrieval — delegation does not waive the embed duty.
  - **Point-of-action reminder hook**: a harness-agnostic principle with a Claude Code
    PostToolUse example — when your agent runtime provides tool hooks, install one that
    reminds you to embed right after a plot lands in `6_Exp/64_Viz/`.
  - The session-close checklists now include a validate step ("before close is the last
    verification point"), and the context-rebuild step says to run `elf validate` to
    surface unfinished items (both presets).

## [2.16.2] - 2026-07-13

### Fixed
- **`elf update` / `elf status` now respect the project's preset.** On `qa` and `general`
  projects both commands planned against the *research* template set: `elf status`
  mis-reported unedited files (e.g. `AGENTS.md`) as `outdated` and research-only rules as
  `missing`, and running `elf update` would then replace those files with their research
  versions, create the research rule set, and re-stamp the project as a research project —
  silently losing its type. Both commands now resolve the preset first and plan against the
  matching template set. (User edits were never at risk — they are conflict-protected — but
  unedited managed files were not.)
- `elf init --preset qa` ended with `next: open 0_Meta/ProjectRule.md`, which does not exist
  in a qa project — the hint now points at `README.md`.

### Added
- **Preset identity is persistent now.** `elf init` records a `"preset"` field in
  `.elf/config.json`; `elf update` and `elf status` read it and print a `preset:` line.
  Projects initialized with an older CLI have no such field — the preset is inferred from the
  project's own stamp (`.elf/manifest.json`) and recorded on the first non-dry-run
  `elf update`, so existing projects need no manual migration. If `config.json` and the stamp
  ever contradict each other, `elf update` refuses to touch the project until the field is
  fixed; `elf status` warns and diagnoses by the stamp.

## [2.16.1] - 2026-07-09

### Changed
- **Documentation-only release.** The README and `CLI.md` now carry a standing
  "Upgrading a project created before v2.15" section — the two-step path via v2.15.1
  (install v2.15.1 → `elf update` → `elf migrate` → return to the latest CLI) — so the
  guidance is visible on the repository page itself, not only in the 2.16.0 release
  notes and the `elf update` leftover warning. No functional changes to the CLI; the
  binaries are identical to 2.16.0 apart from the version string.

## [2.16.0] - 2026-07-09

### Added
- **`elf update` warns about pre-2.15 leftovers** — when legacy rule files are still
  present in `0_Meta/` (e.g. `0_Meta/EliRule.md`), the update output includes a warning
  that names them and prints the two-step upgrade path below. Detection and guidance
  only — the leftover files are never touched.

### Removed
- **Legacy layout support (breaking).** The pre-2.15 layout — rules in `0_Meta/`, stubs
  in a root `templates/` — is no longer read or updated. The `elf migrate` command and
  the layout notes in `elf update` / `elf status` / `elf doctor` output are gone with it;
  the `.elf/managed/` layout is now the only layout.
  **Upgrading a pre-2.15 project**: do **not** run this version's `elf update` on it
  directly — nothing would be lost, but the rules would be deployed a second time under
  `.elf/managed/` while the old copies remain in place as unmanaged leftovers. Instead,
  take the two-step path: install **v2.15.1** from the Releases page → `elf update` →
  `elf migrate` → then update the CLI to the latest version.
- **Bootstrap generator scripts** (`ELF_generator.ps1` / `ELF_generator.sh`) — they
  scaffolded the legacy layout and duplicated what `elf init` does; the self-contained
  CLI is the single supported path. An obsolete runtime proof-of-concept folder was
  removed alongside.

### Fixed
- Newly scaffolded projects' `README.md` / `ProjectRule.md`, the repository README, and
  `CLI.md` now consistently point to the `.elf/managed/` rule paths — several references
  still named the pre-2.15 locations, which do not exist in newly created projects.

## [2.15.1] - 2026-07-09

### Changed
- **`elf update` / `elf status` now state the layout** — on a legacy-layout project every
  run (including `--dry-run` and runs with nothing to change) ends with a one-line note:
  `layout: legacy (kept — intended; relocation to .elf/managed/ is opt-in: elf migrate,
  preview with --dry-run)`. It is a note, not a warning — the legacy layout stays fully
  supported and the line never affects `--check` gates; it disappears once you migrate.
  Added after real-use confusion where an up-to-date project gave no hint that the
  missing `.elf/managed/` was intentional.

## [2.15.0] - 2026-07-07

### Added
- **Project overlays for data files** — customize `LLMcliche.md` / `highIFjournals.md`
  without editing the managed base: write a user-owned `0_Meta/<name>.project.md`;
  effective rules = base ⊕ overlay — `## 추가 (add)` / `## 제외 (remove)` (per entry,
  each removal states a reason) / `## 재정의 (override)`. `elf update` never touches
  overlays, so data customization no longer round-trips through `.elf-new` merges.
  `elf doctor` reports active overlays, removal entries missing a reason, and overlays
  without an overlayable base. Spec: EliRule §2.7.
- **Managed payload relocated to `.elf/managed/` + new `elf migrate`** — ownership is
  now visible in the layout: `.elf/managed/` holds the framework rule payload (rules,
  EN companions, log-format stubs) and `0_Meta/` is purely yours (`ProjectRule.md`,
  overlays, anything else — `elf update` never writes there). New projects scaffold
  this layout by default. **Existing projects keep working on the old paths
  indefinitely** — relocate only when you choose, with the opt-in `elf migrate`:
  plans every move first, refuses on conflicts or uncommitted tracked changes, moves
  pending `.elf-new` siblings along, reports (never rewrites) old-path references in
  your notes, supports `--dry-run`, and is idempotent.

### Changed
- The root `templates/` folder is no longer scaffolded or managed — the canonical
  stubs live in `.elf/managed/templates/`; the root folder is yours for project
  templates.
- `AGENTS.md`, `EliRule.md`, `LogConvention.md`, and the session template now point
  to the `.elf/managed/` paths (with a legacy-layout note), and `CLI.md` documents
  `elf migrate`.

Run `elf update` (keeps your current layout), then `elf migrate` when ready.

## [2.14.0] - 2026-07-07

### Changed
- **LogConvention — the header `Handoff` is a replace-style snapshot (fold)** — the field
  is now specified as three parts: `currently valid conclusion (the fold of what
  accumulated); pending; references (where the synthesis lives)`, rewritten as a whole on
  each update. Do not accumulate history or completed timelines — the session overview
  belongs to the registry key finding (same fold principle), details to the trial bodies.
  The session-close checklist gains "clean the Handoff" and "rewrite the key finding as
  the final conclusion", and `elf trial new` prints a one-line reminder of the fold style.
  Research + general presets, EN companions. Run `elf update`.
- **`elf session close` — pending-handoff warning** — closing a session whose `Handoff`
  still lists pending items prints a **non-blocking** warning quoting the leftovers
  (resolve them or carry them into the next-session section); a note after closing
  reminds you to rewrite the registry key finding as the session's final conclusion.
  Part boundaries are anchored as a semicolon **immediately followed by a label**
  (`미완료`/`pending`, `참조`/`refs`), so a semicolon or a label word inside the state
  text cannot confuse the warning; logs without labeled boundaries are left alone
  (no format enforcement). Semicolons are reserved for part boundaries — use `·`/`,`
  inside a part.
- **`elf update` — smarter dirty-tree warning** — the warning now fires only when
  **tracked** files have uncommitted changes, and explains the actual risk: "your
  uncommitted changes and this update's changes would mix in one tree, so the update
  alone could not be rolled back cleanly". Untracked-only working trees no longer warn
  (the update remains cleanly revertible); a repository with no commits yet gets a
  dedicated "no baseline to roll back to — commit first" warning instead.

## [2.13.0] - 2026-07-06

### Added
- **`AGENTS.md` agent entry file (all presets)** — every project now ships a root
  `AGENTS.md`: a governance digest (logging duties, `elf trial new`, canon-over-precedent,
  re-read the Handoff after a context rebuild, customization pointers) that vendor-neutral
  coding agents load, plus a `CLAUDE.md` loader pointer (`@AGENTS.md`) for Claude Code.
  `CLAUDE.md` uses a new **pointer tier**: created if missing, **never modified if
  present** — an existing hand-written `CLAUDE.md` stays untouched (no `.elf-new` either);
  `elf doctor` checks the link. The `qa` preset's operational rules move from `CLAUDE.md`
  into `AGENTS.md` (an unedited `CLAUDE.md` converts to the pointer automatically on
  `elf update`). English companions (`AGENTS.en.md`) deploy for `--lang en-*` projects.
  Run `elf update`.
- **`elf trial new [title]`** — appends the current canonical trial stub to the active
  session log: auto-numbered `t##`, `S###` paths substituted, header `Modified` refreshed,
  inserted before the next-session section (`--session S###` when several are open). Keeps
  the imitation target canonical, so a drifted precedent never propagates.
- **`elf validate` — trial structure checks** — active logs are checked for non-canonical
  `###` headings, section order, the `가설 적중 여부` first line of `### 해석`, and missing
  Phase-1 sections when `### 관찰` exists. Warnings by default (promoted to issues with
  `--strict`); archived logs are skipped — the convention applies to new writing, not
  backfill.
- **`elf doctor` — agent-entry check** — warns when `CLAUDE.md` lacks the `@AGENTS.md`
  line (Claude Code would not load the rules), and flags heavy extra content in the
  pointer and pending `AGENTS.md.elf-new` / `CLAUDE.md.elf-new` files.

### Changed
- **LogConvention — canon over precedent** — past session/trial logs are reference, not
  the norm: follow the canonical templates, do not imitate a drifted precedent (report it),
  and prefer `elf trial new` for new trials. The session template now carries a two-line
  reminder comment (canonical sources + how to add trials + re-read the Handoff after a
  context rebuild). Research + general presets, EN companions. Run `elf update`.
- **`.gitignore` template — `CLAUDE.md` no longer ignored** — the pointer is project
  infrastructure (it must survive a clone so agents load `AGENTS.md`), not personal AI
  config. Unedited managed blocks pick this up on `elf update`.
- **LLMcliche — 7 clichés added** — verbs `constitute` / `intensify` / `tie (A to B)`,
  nouns `readout` / `X family`, connector `Nonetheless`, phrase `"in parallel (with)"`,
  with precise-use exceptions (§4) and a before/after example (from a Mastication
  manuscript-editing proposal). Run `elf update`.

## [2.12.1] - 2026-07-02

### Changed
- **LLMcliche — `verdict` added** — the §1 cliché-noun list gains `verdict` (elevates an
  evaluation or check result into courtroom register — use *result* / *pass-fail*), with a
  matching before/after example. meta and general presets, plus the EN companion. Run
  `elf update`.

## [2.12.0] - 2026-06-29

### Added
- **Source-reliability rule in governance** — `EliRule.md` gains a Source Reliability rule:
  do not cite namu.wiki or other anonymous/collectively-edited wikis, or unattributed
  blogs/community posts, as sources; cross-check against reliable sources (institutions,
  academic societies, governments, primary/official materials) and cite those; set
  `blocked_domains: ["namu.wiki"]` in WebSearch; and list verified sources under `## 출처`
  (or `## Sources`) for any web-grounded answer or document. Applies to research and
  `general` projects (and the `qa` preset via its new §7). Run `elf update`.
- **More high-IF journal domains** — `highIFjournals.md` adds `pubs.rsc.org` (Lab on a Chip,
  Chemical Science, Chem Soc Rev, …), `link.springer.com` / `*.springeropen.com` (eLight,
  PhotoniX, Nano-Micro Letters), `iopscience.iop.org`, and `journals.aps.org` (Reviews of
  Modern Physics, Physical Review X). Run `elf update` (research projects).
- **`qa` preset — behavior rules & rolling capture** — the `qa` preset `CLAUDE.md` gains a
  §7 (source reliability, no-emoji, tone, language, persona, proactive capture) and a
  rolling-capture protocol (per-turn thread classification, turn-1 deferral, roll-forward).
  `qa` stays experimental.

### Changed
- **`qa` bundle — `# 내 생각` moved** — the reflection section now sits after `# 교훈`
  (was at the top), so the on-read order matches the writing order.

### Fixed
- **LLMcliche — `alleviate` restored** — the cliché verb `alleviate`, missing from the §1
  verb list, is restored. Run `elf update`.

## [2.11.0] - 2026-06-29

### New
- **Optional `### 배경 (Background)` section in trial logs** — research and `general`
  projects can add a `### 배경 (Background)` section above `### 목표 (Goal)` in a trial to
  preserve the *thinking flow* behind it (entry intent, **initiator**, a rejected entry
  path) without duplicating the verbatim query — the session transcript already keeps the
  raw query. It is **conditional** (only when context exceeds the goal, e.g. synthesizing a
  prior session; ordinary trials let the goal absorb it), kept light (1-2 lines), and never
  enforced by `elf validate`. Run `elf update`.

### Changed
- **LogConvention — initiator & rejected-alternative placement** — trials name the
  **initiator** of a task/pivot in one word (user-raised / AI-originated), and rejected
  alternatives have a defined home by timing (entry → `### 배경`, scope → `### 조건`,
  result-based → `### 해석`·`### 교훈`). Applies to both research and `general` presets
  (plus English companions). Run `elf update`.

## [2.10.0] - 2026-06-26

### New
- **`elf init` in place** — running `elf init` with **no name now initializes the current
  folder in place** (like `git init`); `elf init <name>` still creates a fresh `./<name>/`
  subfolder. In place, ELF is added to an existing folder **without overwriting anything**:
  missing ELF files are created, your files are kept, and a colliding ELF-managed file is
  written alongside as `<file>.elf-new` (your `.gitignore`, `README`, etc. are never
  clobbered; user-owned seed files are skipped). A non-empty folder gets one confirmation
  echoing the path; an already-initialized project is refused (use `elf update`). New flags:
  `--here`, `--yes`, `--dry-run`, `--force`. See `CLI.md`.

### Changed
- **`elf init` defaults to the current folder** — the no-name form used to error (a name was
  required); it now initializes in place. `elf init <name>` is unchanged, so existing usage
  stays backward-compatible.

## [2.9.1] - 2026-06-24

### Fixed
- **English companions — section names now glossed** — in the `LogConvention.en.md` companions,
  bare Korean section tokens referenced in prose (`### 가설`, `### 관찰`, etc.) now carry their
  English gloss (`### 가설 (Hypothesis)`, …), matching the actual bilingual log headers, so the
  English companion reads without untranslated Korean. Run `elf update` (English projects).

## [2.9.0] - 2026-06-24

### New
- **English companion docs (experimental)** — `elf init <name> --lang en-US` now also deploys
  *informative English companions* of the governance docs (`0_Meta/EliRule.en.md`,
  `LogConvention.en.md`, etc.) for human reading. The operative rules stay Korean (`*.md`) and
  the AI agent always operates from them, so project behavior is identical across languages;
  companions are marked `NOT OPERATIVE`. The user-owned `README.md` and `ProjectRule.md` are
  scaffolded in English. `elf update` keeps companions in sync; `elf doctor` reports i18n
  status. English only for now — other languages fall back to Korean. Korean projects are
  unchanged. See `CLI.md`.

### Changed
- **EliRule / AI_PARA_Framework** — a companion-firewall rule (English `*.en.md` files are
  informative, not operative — the AI operates from `*.md`; customize via `ProjectRule.md`) and
  a note that the Korean sentence-ending rule does not apply to English output. Run
  `elf update`.

## [2.8.0] - 2026-06-23

### New
- **`general` preset (experimental)** — `elf init <name> --preset general` scaffolds a
  *goal-oriented, non-research* project (building a tool, preparing a grant
  proposal, a focused learning project) instead of the research layout. It keeps the
  session/trial logging spine but drops the academic parts (experiments, paper,
  literature). See `elf-cli/CLI.md`.

### Changed
- **`--lang` takes a BCP-47 tag** — the default is now `ko-KR` (was `한국어`); the
  no-install generator lists languages as `code (native)`, e.g. `ko-KR (한국어)`.
- **Logging convention — "trial" wording, spelled-out SOP, English labels** — logs use
  "trial" (not "ticket") throughout; the trial/session procedures are written out as a
  Standard Operating Procedure; section labels carry English glosses
  (목표(Goal) / 조건(Conditions) / 교훈(Lessons) / 생성 파일(Files)). Run `elf update`.
- **EliRule — two governance rules** — a *traceability* principle (record meaningful
  work/changes/decisions in the session log) and an *LLM cliché ban* (avoid
  LLM-characteristic filler in English writing, with a non-binding reference list in the
  new `0_Meta/LLMcliche.md`). Run `elf update` for the new `EliRule.md`.

## [2.7.3] - 2026-06-20

### Changed
- **Wording — "archetype" → "project type"**: the `qa` preset is described as a
  *project type* (instead of "archetype") in the CLI reference and the qa project's
  `CLAUDE.md`, for clarity. Run `elf update` in qa projects for the new `CLAUDE.md`.

## [2.7.2] - 2026-06-18

### Changed
- **Logging convention — no-metaphor rule sharpened**: the "no metaphor" rule now
  lists concrete examples of common LLM metaphors to avoid (lens / navigate /
  deep dive / journey) and points to plain terms instead. Run `elf update` for the
  new `EliRule.md`.

## [2.7.1] - 2026-06-16

### Changed
- **Logging convention — abbreviations**: define an abbreviation on first use
  (`AR (asymmetry ratio)`) and give an abbreviation legend per trial, so a single
  trial stays self-readable. Run `elf update` for the new `EliRule.md`.

## [2.7.0] - 2026-06-16

### New
- **`qa` preset (experimental)** — `elf init <name> --preset qa` scaffolds a
  *question-archive* project (Q&A "bundles") instead of the research layout.
  Add `--categories a,b,c` to pre-create category folders (default: none —
  created on demand). See `elf-cli/CLI.md`.

## [2.6.2] - 2026-06-15

### Changed
- **Trial-and-error trigger sharpened** — any change to a working figure's
  appearance (color/axes/font/params) becomes its own delta-trial (even once);
  only non-appearance fixes / pre-working debugging stay in a `### 시행착오` table.
  Run `elf update` for the new `LogConvention.md`.

## [2.6.1] - 2026-06-15

### Changed
- **Logging convention — trial-and-error as base-delta trials**: when iteratively
  improving a plot/analysis, expand each attempt as its own trial and keep every
  script/figure version (reproducible, no survivorship bias). Run `elf update` for
  the new `LogConvention.md`.

## [2.6.0] - 2026-06-14

### New
- **`elf validate` flags missing figure embeds** — a figure in `6_Exp/64_Viz/S###/`
  that isn't inline-embedded in its session log is reported (a table path is not an
  embed). Warning by default; `--strict` makes it a gating issue. Exclude with
  `<!-- noembed: file.png -->`.

### Changed
- LogConvention: embed figures inline in the same turn they're generated.

## [2.5.0] - 2026-06-14

### Changed (heads-up)
- **`0_Meta/AI_Sync.md` deprecated** — new `elf init` no longer creates it; AI-handoff
  moves to the session log's `Handoff` field + `Session_Registry.tsv`. Existing files
  in your projects are left untouched (`elf update` won't delete them).

## [2.4.5] - 2026-06-14

### Changed
- **Bilingual CLI help** — `elf --help` now shows `한국어 · English`.
- The no-install generator also writes `.elf/manifest.json`, so generated projects
  are recognized by `elf update`/`status`/`doctor`.

## [2.4.4] - 2026-06-13

### Fixed
- `elf session close` now fixes a log's relative cross-references (`](../…)`) for its
  new archive depth.

## [2.4.3] - 2026-06-13

### New
- **`elf session close [S###]`** — close a session (Status → Complete, move to
  `2_Log/Archive/`, update the registry).
- **`elf validate [--check]`** — check session/registry/log consistency (read-only;
  `--check` exits 4 for CI).
- **`elf gallery`** — build a per-session figure index (`_gallery.md`) from `6_Exp/64_Viz/`.
- **`elf doctor`** — environment + project health check (read-only).

## [2.4.2] - 2026-06-12

### New
- **`elf self-update`** — update the `elf` binary in place.
- **`elf session new <title>`** — create + register the next session log.
- **`elf session fix-headers`** — retrofit session-header line breaks (idempotent).
- Projects now receive a `.editorconfig` (LF / UTF-8) and a CLI reference (`CLI.md`).

### Fixed
- Cleaner `--help` (internal labels removed); embed/data errors fail gracefully
  instead of panicking.

## [2.4.1] - 2026-06-12

### Fixed
- `elf init` no longer panics on a missing embedded template (affected the v2.4.0
  binary). Use v2.4.1+.

## [2.4.0] - 2026-06-12

### New
- First Rust **`elf` CLI** — `elf init` (scaffold a project; `--preset`/`--modules`/`--lang`),
  `elf update` (refresh ELF-managed files; `--dry-run`/`--force`), `elf status`
  (drift check; `--check` for CI). Single static binary, no Node/npm.
- Prebuilt installers for Windows/macOS/Linux (x64/arm64).
