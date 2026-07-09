# PLACEHOLDER_PROJECT_NAME

A goal-oriented, non-research project created with the ELF **general preset
(experimental)**. (Created: PLACEHOLDER_DATE)

Accumulates a multi-session project with a clear goal (building a tool, preparing a
proposal, a focused learning project, building something, etc.) as base-delta session
logs. A type separated from academic research (simulation, figures, papers).

## Structure
- `.elf/` — ELF control plane (version, manifest) + managed rule payload `managed/` (`EliRule`, `LogConvention`, `AI_PARA_Framework`, `LLMcliche`, session/trial stubs `templates/`). Updated by `elf update` — do not edit directly.
- `0_Meta/` — project governance (`ProjectRule`, data overlays `<name>.project.md`) — user space
- `1_Concept/12_Planning/` — goals, roadmap, plans / `13_Ideas/` — small ideas
- `2_Log/` — session logs (`S###_log.md`), `Wiki/` (summaries, Registry), `Archive/` (completed)
- Domain work folders (`src/`, `docs/`, etc.) are added by the project itself.

## Use
1. Write goals and plans in `1_Concept/12_Planning/`.
2. Accumulate work as base-delta sessions (`S###`) and trials (`t##`) — Phase 1 (hypothesis, prediction) stop point, then Phase 2 (execution, observation, interpretation). Rules: `.elf/managed/LogConvention.md`.
3. The trial form defaults to 5-section. If the project needs an adjustment, state it in `0_Meta/ProjectRule.md`.

## Project Rules
Project-specific rules and goals: `0_Meta/ProjectRule.md`. Structure and operating detail: `.elf/managed/EliRule.md`. Logging rules: `.elf/managed/LogConvention.md`. AI context management: `.elf/managed/AI_PARA_Framework.md`.

> Warning: experimental preset — not stable. For academic research projects, use `elf init <name> --preset full` (the default).
