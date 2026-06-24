# PLACEHOLDER_PROJECT_NAME

A goal-oriented, non-research project created with the ELF **general preset
(experimental)**. (Created: PLACEHOLDER_DATE)

Accumulates a multi-session project with a clear goal (building a tool, preparing a
proposal, a focused learning project, building something, etc.) as base-delta session
logs. A type separated from academic research (simulation, figures, papers).

## Structure
- `0_Meta/` — governance (`EliRule`, `LogConvention`, `AI_PARA_Framework`, `ProjectRule`)
- `1_Concept/12_Planning/` — goals, roadmap, plans / `13_Ideas/` — small ideas
- `2_Log/` — session logs (`S###_log.md`), `Wiki/` (summaries, Registry), `Archive/` (completed)
- `templates/` — session and trial stubs
- `.elf/` — ELF control plane (version, manifest; update rules with `elf update`). Do not edit directly.
- Domain work folders (`src/`, `docs/`, etc.) are added by the project itself.

## Use
1. Write goals and plans in `1_Concept/12_Planning/`.
2. Accumulate work as base-delta sessions (`S###`) and trials (`t##`) — Phase 1 (hypothesis, prediction) stop point, then Phase 2 (execution, observation, interpretation). Rules: `0_Meta/LogConvention.md`.
3. The trial form defaults to 5-section. If the project needs an adjustment, state it in `0_Meta/ProjectRule.md`.

## Project Rules
Project-specific rules and goals: `0_Meta/ProjectRule.md`. Structure and operating detail: `0_Meta/EliRule.md`. Logging rules: `0_Meta/LogConvention.md`. AI context management: `0_Meta/AI_PARA_Framework.md`.

> Warning: experimental preset — not stable. For academic research projects, use `elf init <name> --preset full` (the default).
