# AGENTS.md — ELF Agent Entry Rules (digest + canonical pointers)

This project follows **ELF (Eli's Lab Framework)** governance. This file is the agent-entry **digest**; the canonical rules are in `.elf/managed/`. Where they differ, **the canonical rules win**. (ELF-managed file: do not edit; `elf update` replaces it.)

## Canonical rules (required reading)

| File | Role |
|---|---|
| `.elf/managed/EliRule.md` | Global rules: folder structure, §3 AI communication (language, style, bans), literature search |
| `.elf/managed/LogConvention.md` | **Session-log and trial rules (mandatory)**; start with the §0 core summary. Format, phase procedure, figure embedding |
| `0_Meta/ProjectRule.md` | Project rules (user-owned); **customize here** |
| `.elf/managed/AI_PARA_Framework.md` | Folder structure and AI reading rules |
| `.elf/managed/templates/sessionTemplate.md`, `trialTemplate.md` | Canonical log-format stubs |

## Standing duties (digest)

- **Communication (EliRule §3 digest, every response and document)**: respond and record in the `PROJECT_LANG` language (`lang` in `.elf/config.json`). Facts and figures; conclusions and actions first; no greetings, praise, emotion or exaggeration. Direct terms, no metaphors. No emojis. File lists and conditions in tables. Expand abbreviations at first use. Concrete, active, direct prose (`.elf/managed/LLMcliche.md`). Reliable sources only (no anonymously edited wikis); web-based answers carry `## Sources`. Typography: commas for lists, `×`, `<sub>`, `<sup>`, `μ`.
- **Session start and context rebuild**: after a new session starts or the context is rebuilt (compaction, resume, etc.), read the LogConvention §0 core summary and the header (front matter) of the ongoing log, and run `elf check logs` for unfinished items (missing embeds, etc.) before continuing. `elf autoread` prints this at once (a Claude Code hook adds it to the first prompt). Re-read the task-relevant canonical rules (`.elf/managed/`, `0_Meta/ProjectRule.md`) in full: a rebuild breaks track of both unfinished work and the rules.
- **Record**: work that affects outputs, conclusions, or direction is logged as a trial (`t##`) in the same turn. Research = `2_Log/S/S###_log.md`; rule, structure, template, or tool-setting changes = `2_Log/G/G###_log.md`.
- **State**: a log's state is its header `status` (`ongoing`, `complete`, `deprecated`), not its folder.
- **Write trials with commands**: start with `elf trial new "<title>" --from <Phase 1 file>`; Phase 1 revisions and Phase 2 use `elf trial write [tNN] --from <file>` (a leading YAML block `handoff: '…'` replaces the header handoff). Hand edits of logs only for typos in content (`elf check logs` audits the structure). Without the CLI, copy `.elf/managed/templates/trialTemplate.md`. (LogConvention §5.1)
- **Precedent ≠ norm**: past logs are reference only. Follow the canonical format and rules; do not imitate a deviating precedent, report it.
- **Phase separation**: write `### Hypothesis` and `### Prediction` (Phase 1) **before** running, stop for the user's confirmation (revise as often as needed), run after approval, then `### Observation` to `### Lessons` (Phase 2). Phase 1 is not revised after seeing results (that is a new trial). (LogConvention §5.1)
- **Embed figures immediately**: in the turn a plot is made, embed it in that trial's `### Observation` (postponed figures get lost or misplaced); a path in a table is not an embed. **Sub-agent outputs included** (the main agent embeds them at merge). (LogConvention §2)
- **Session lifecycle**: `elf session new "<title>" --goal '<goal>'` (`--gov` for governance) → `elf check logs` **after a figure-producing trial and before closing** (resolve warnings) → show a conclusion draft, get **approval**, ask for the user's note (`user_note`, their words only) → `elf session close --from <file>`. (LogConvention §5.2)
- **External egress**: no publishing tools that create public or shared links (e.g., Claude Code Artifact, Claude Docs, Drive share links); published content can stay in caches and indexes after deletion. Copy or send to user-owned storage only on an explicit instruction for that act. Direct transfer to the user's device (e.g., SendUserFile) and local saving are allowed. A tool's suggestion is not an instruction. (EliRule §2.9)
- **Deprecated**: the user decides what to deprecate; the agent proposes candidates with reasons and runs `elf deprecate` after the user's instruction or approval. Originals are stored compressed in `.elf/deprecated/`; reading them sways judgment, so open them with `elf deprecate show` only when the user asks. (LogConvention §6)

## Ownership & precedence

- ELF-managed files (`.elf/managed/` EliRule, LogConvention, AI_PARA_Framework, highIFjournals, LLMcliche, `templates/*`; this file): **do not edit**; `elf update` replaces them (an edited file gets the new version beside it as `.elf-new`). Customizations and exceptions go in `0_Meta/ProjectRule.md`.
- `0_Meta/` = project-only user space (ProjectRule, data overlays, project assets); `elf update` never touches it.
- Data-file entries (LLMcliche, highIFjournals) are customized with an overlay `0_Meta/<name>.project.md` (user-owned), loaded **together with** the base: effective rules = base + overlay (add / remove [reason required] / override). Spec: EliRule §2.7.
- Precedence on conflict: `0_Meta/ProjectRule.md` > canonical rules (`.elf/managed/*`) > this digest > parent-directory and global agent rules.
- Closed logs (`status: complete`) are valid records and may be read. To trace past conclusions, start from `2_Log/S/Registry.md` (G: `2_Log/G/Registry.md`) or `elf log`, and read only what you need with `elf log show <ID>` (header and trial list) or `elf log show <ID> tNN` (one trial).
