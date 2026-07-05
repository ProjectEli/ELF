# AGENTS.md — ELF Agent Entry Rules (digest + canonical pointers)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `AGENTS.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `0_Meta/ProjectRule.md` (not this file).

This project follows **ELF (Eli's Lab Framework)** governance. `AGENTS.md` is the agent-entry **digest** — the canonical rules live in `0_Meta/`. When the digest and a canonical document differ, **the canonical document wins**. (ELF-managed file — do not edit directly; `elf update` replaces it.)

## Canonical rules (required reading)

| File | Role |
|---|---|
| `0_Meta/EliRule.md` | Global rules — folder structure, §3 AI communication (language, style, bans), literature search |
| `0_Meta/LogConvention.md` | **Session-log / trial writing rules (mandatory)** — format, phase procedure, figure embedding |
| `0_Meta/ProjectRule.md` | Project-specific rules (user-owned) — **customize here** |
| `0_Meta/AI_PARA_Framework.md` | PARA file isolation, Archive firewall, hallucination guards |
| `templates/sessionTemplate.md` · `templates/trialTemplate.md` | Canonical log-format stubs |

## Standing duties (digest)

- **Record**: any work that affects outputs, conclusions, or direction is logged as a trial (`t##`) in `2_Log/S###_log.md` in the same turn.
- **Add trials with `elf trial new [title]`** — appends the current canonical stub to the active log. Without the CLI, copy `templates/trialTemplate.md` manually.
- **Precedent ≠ norm**: past sessions/trials are reference only — follow the canonical format and rules; when a precedent deviates from the canon, do not imitate it and report it to the user.
- **Phase separation**: `### 가설 (Hypothesis)` / `### 예상 (Prediction)` (Phase 1) are written **before** execution, then stop → execute → `### 관찰 (Observation)` through `### 교훈 (Lessons)` (Phase 2). (LogConvention §5.1)
- **Embed figures immediately**: in the turn a plot is produced, embed it inline in that trial's `### 관찰 (Observation)` — a path in a table is not an embed. (LogConvention §2)
- **Session lifecycle**: start with `elf session new "<title>"` → before closing run `elf validate` (resolve warnings) → `elf session close`.
- **After a context rebuild** (compaction, session restart): re-read the active log header (`Handoff`) and continue from there.

## Ownership & precedence

- ELF-managed files (`0_Meta/` EliRule, LogConvention, AI_PARA_Framework, highIFjournals, LLMcliche; `templates/*`; `.claudeignore`; this file) — **do not edit directly**; `elf update` replaces them (edits are preserved as `.elf-new` siblings). Customizations and exceptions go in `0_Meta/ProjectRule.md`.
- Rule precedence on conflict: `0_Meta/ProjectRule.md` (project-specific) > canonical general rules (`0_Meta/*`) > this digest > parent-directory/global agent rules.
- Never explore `Archive/` folders autonomously (`.claudeignore` firewall) — open them only when the user names a path.
