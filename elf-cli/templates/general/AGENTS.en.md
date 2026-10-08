# AGENTS.md — ELF Agent Entry Rules (digest + canonical pointers, general)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `AGENTS.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `0_Meta/ProjectRule.md` (not this file).

This project follows **ELF (Eli's Lab Framework) general preset** (goal-oriented, non-research) governance. `AGENTS.md` is the agent-entry **digest** — the canonical rules live in `.elf/managed/`. When the digest and a canonical document differ, **the canonical document takes precedence**. (ELF-managed file — do not edit directly; `elf update` replaces it.)

## Canonical rules (required reading)

| File | Role |
|---|---|
| `.elf/managed/EliRule.md` | Global rules — folder structure, §3 AI communication (language, style, bans) |
| `.elf/managed/LogConvention.md` | **Session-log / trial writing rules (mandatory)** — format, phase procedure |
| `0_Meta/ProjectRule.md` | Project-specific rules (user-owned) — **customize here** |
| `.elf/managed/AI_PARA_Framework.md` | Folder location = state (in progress, Wiki, Archive, Deprecated) and the AI reading rules |
| `.elf/managed/templates/sessionTemplate.md` · `trialTemplate.md` | Canonical log-format stubs |

## Standing duties (digest)

- **Record**: any work that affects outputs, conclusions, or direction is logged as a trial (`t##`) in `2_Log/S###_log.md` in the same turn.
- **Add trials with `elf trial new [title]`** — appends the current canonical stub to the active log. Without the CLI, copy `.elf/managed/templates/trialTemplate.md` manually.
- **Precedent ≠ norm**: past sessions/trials are reference only — follow the canonical format and rules; when a precedent deviates from the canonical rules, do not imitate it and report it to the user.
- **Phase separation**: `### 가설 (Hypothesis)` / `### 예상 (Prediction)` (Phase 1) are written **before** execution, then stop → execute → `### 관찰 (Observation)` through `### 교훈 (Lessons)` (Phase 2). (LogConvention §5)
- **Preserve output versions**: when iterating, do not overwrite — keep version suffixes (`_v1`/`_v2`); each version maps 1:1 to a delta trial. (LogConvention §3)
- **Session lifecycle**: start with `elf session new "<title>"` → before closing run `elf validate` (resolve warnings) → `elf session close`.
- **After a context rebuild** (compaction, session restart): re-read the active log header (`Handoff`), **run `elf validate` to list unfinished items**, and **re-read the task-relevant canonical rules under `0_Meta/` in full** (auto-included in the digest when declared via `autoread_fulltext` in `.elf/config.json`), then continue — a rebuild is where both unfinished-state and rule-awareness tracking break.
- **External egress**: do not use publishing tools that create public or shared links (e.g., Claude Code Artifact, Claude Docs, Drive share links) · copy or transfer into user-owned storage only on an explicit instruction for that act · direct transfer to the user's machine (e.g., SendUserFile) and local saving are allowed. A tool's own suggestion is not an instruction. (EliRule §2.6)- **Deprecated**: documents under `Deprecated/` are past attempts that were withdrawn. They may be read but are not a basis for current decisions. The user decides what to deprecate; the agent proposes candidates with reasons and runs `elf deprecate` only after the user's instruction or approval. (AI_PARA_Framework §1 · LogConvention §6)

## Ownership & precedence

- ELF-managed files (`.elf/managed/` EliRule, LogConvention, AI_PARA_Framework, LLMcliche, `templates/*` and companions; this file) — **do not edit directly**; `elf update` replaces them (edits are preserved as `.elf-new` siblings). Customizations and exceptions go in `0_Meta/ProjectRule.md`.
- `0_Meta/` = project-only (user space): ProjectRule, data overlays, project assets — `elf update` never modifies it.
- Data-file entry customization (LLMcliche) = project overlay `0_Meta/LLMcliche.project.md` (user-owned) — loaded **together with** the base; effective rules = base ⊕ overlay (add / remove [reason required] / override). Spec: EliRule §2.4.
- Rule precedence on conflict: `0_Meta/ProjectRule.md` (project-specific) > canonical general rules (`.elf/managed/*`) > this digest > parent-directory/global agent rules.
- `Archive/` holds closed, still-valid records — reading is allowed; refer to it when tracing past conclusions and paths.
