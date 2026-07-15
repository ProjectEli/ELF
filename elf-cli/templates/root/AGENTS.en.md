# AGENTS.md — ELF Agent Entry Rules (digest + canonical pointers)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `AGENTS.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `0_Meta/ProjectRule.md` (not this file).

This project follows **ELF (Eli's Lab Framework)** governance. `AGENTS.md` is the agent-entry **digest** — the canonical rules live in `.elf/managed/`. When the digest and a canonical document differ, **the canonical document wins**. (ELF-managed file — do not edit directly; `elf update` replaces it.)

## Canonical rules (required reading)

| File | Role |
|---|---|
| `.elf/managed/EliRule.md` | Global rules — folder structure, §3 AI communication (language, style, bans), literature search |
| `.elf/managed/LogConvention.md` | **Session-log / trial writing rules (mandatory)** — format, phase procedure, figure embedding |
| `0_Meta/ProjectRule.md` | Project-specific rules (user-owned) — **customize here** |
| `.elf/managed/AI_PARA_Framework.md` | PARA file isolation, Archive firewall, hallucination guards |
| `.elf/managed/templates/sessionTemplate.md` · `trialTemplate.md` | Canonical log-format stubs |

## Standing duties (digest)

- **Record**: any work that affects outputs, conclusions, or direction is logged as a trial (`t##`) in `2_Log/S###_log.md` in the same turn.
- **Add trials with `elf trial new [title]`** — appends the current canonical stub to the active log. Without the CLI, copy `.elf/managed/templates/trialTemplate.md` manually.
- **Precedent ≠ norm**: past sessions/trials are reference only — follow the canonical format and rules; when a precedent deviates from the canon, do not imitate it and report it to the user.
- **Phase separation**: `### 가설 (Hypothesis)` / `### 예상 (Prediction)` (Phase 1) are written **before** execution, then stop → execute → `### 관찰 (Observation)` through `### 교훈 (Lessons)` (Phase 2). (LogConvention §5.1)
- **Embed figures immediately**: in the turn a plot is produced, embed it inline in that trial's `### 관찰 (Observation)` — a path in a table is not an embed. **Sub-agent outputs included** (main embeds them at retrieval). (LogConvention §2)
- **Session lifecycle**: start with `elf session new "<title>"` → run `elf validate` **right after a figure-producing trial and before closing** (resolve warnings) → `elf session close`.
- **After a context rebuild** (compaction, session restart): re-read the active log header (`Handoff`), **run `elf validate` to surface unfinished items (missing embeds, etc.)**, and **re-read the task-relevant canonical rules under `0_Meta/` in full** (auto-included in the digest when declared via `autoread_fulltext` in `.elf/config.json`), then continue — a rebuild is where both unfinished-state and rule-awareness tracking break.

## Ownership & precedence

- ELF-managed files (`.elf/managed/` EliRule, LogConvention, AI_PARA_Framework, highIFjournals, LLMcliche, `templates/*` and companions; root `.claudeignore`; this file) — **do not edit directly**; `elf update` replaces them (edits are preserved as `.elf-new` siblings). Customizations and exceptions go in `0_Meta/ProjectRule.md`.
- `0_Meta/` = project-only (user space): ProjectRule, data overlays, project assets — `elf update` never touches it.
- Data-file entry customization (LLMcliche, highIFjournals) = project overlay `0_Meta/<name>.project.md` (user-owned) — loaded **together with** the base; effective rules = base ⊕ overlay (add / remove [reason required] / override). Spec: EliRule §2.7.
- Rule precedence on conflict: `0_Meta/ProjectRule.md` (project-specific) > canonical general rules (`.elf/managed/*`) > this digest > parent-directory/global agent rules.
- Never explore `Archive/` folders autonomously (`.claudeignore` firewall) — open them only when the user names a path.
