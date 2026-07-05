# Changelog

User-facing highlights for the `elf` CLI — new features, new options, and changes
that affect your projects. (Exhaustive internal history is kept separately by the
maintainer.) The matching section is shown on each GitHub Release.

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
