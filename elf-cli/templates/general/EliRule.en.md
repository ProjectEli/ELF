# EliRule: Project Structure & Operational Guide (general)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `EliRule.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

This document defines the folder structure and operating rules of an ELF `general`
project (a goal-oriented, non-research project — building a tool, preparing a proposal,
a focused learning project, building something, etc.). Where `README.md` covers
philosophy and overview, this document is the practical specification.

> `general` = a **multi-session project with a clear goal**, not academic research. It is
> separated from the research preset (`6_Exp`, `7_Paper`, figures, simulation). Write
> domain-specific detailed rules in `ProjectRule.md`.

---

## 1. Folder Structure

### Core (always included)

#### `.elf/` — ELF control plane
Where the `elf` CLI records project state (version, settings, list of managed files).
**Do not edit directly** — managed by `elf init` / `elf update`.

#### `0_Meta/` — project governance
- `ProjectRule.md`: project-specific rules and goals (**user-owned** — edit freely to fit the project)
- `EliRule.md`: this document (structure and operating guide)
- `LogConvention.md`: logging standard rules
- `AI_PARA_Framework.md`: state-based file-management and archiving rules. The most important reference document when the AI explores the project

#### `1_Concept/` — planning & ideas
- **`12_Planning/`**: project goals, roadmaps, plans (`P###_title.md` numbering). `Wiki/`: planning-stage conclusion summaries.
- **`13_Ideas/`**: snippets too small for a session, and early ideas. Flat (no Archive, never discarded). Promote to a Planning document or a session trial once mature.

#### `2_Log/` — session logs
The top-level space for logging all work.
- `S###_log.md`: session log (format: see `0_Meta/LogConvention.md`)
- `Wiki/`: key-finding summaries + the Session Registry
- `Archive/`: completed session logs

#### `templates/` — markdown stubs
- `sessionTemplate.md`: copy to `2_Log/S###_log.md` when starting a new session
- `trialTemplate.md`: paste into the body when adding a trial (t##) to an in-progress session
- **Planning documents (`P###`) are intentionally template-free** — trials are disciplined by templates for reproducibility, but planning is the researcher's free exploration, so no format is imposed.

### Domain folders (user-added)
Work folders that fit the project (e.g., `src/`, `docs/`, `assets/`) are **added by the
user**. ELF provides the spine (`.elf` control plane, session logs, conventions), and the
project defines the structure of domain outputs.

---

## 2. Operating Rules

> **Traceability principle**: As a rule, **record all work, changes, and decisions in the session log (`2_Log/S###`)**. Meaningful code/data/document changes and attempts/results (including failures) are left in the work turn to secure reproducibility and traceability — trivial fixes (typos, paths) are not forced, but changes affecting outputs, conclusions, or direction are not omitted. For detailed format and base-delta rules, see `LogConvention.md`.

### 2.1 Git separation strategy
- **Tracked by Git**: code, metadata, logs, outputs — project outputs in general.
- **Excluded from Git**: large binaries, tool temp files. Large assets are best managed with Git LFS or separately.

### 2.2 Naming convention
- **Session-Trial**: `S###_t##` (e.g., `S001_t1`).
- **Do not** list condition/variable info in file names — record all conditions in the log.
- Planning documents: `P###_title.md`.

### 2.3 Cross-reference rules
- Reference a Planning document from a log: `→ see 1_Concept/12_Planning/P###_xxx.md`.

### 2.4 ELF-managed files and updates (`elf update`)

Project files fall into three ownership classes, which `elf update` strictly respects:

| Class | Files | `elf update` behavior |
|------|------|-------------------|
| **ELF-managed** | `EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `LLMcliche.md`, `templates/*`, `.claudeignore`, `AGENTS.md` | Replaced with the new version. **If you edited it, it is not overwritten**; the new version is written as `<file>.elf-new` (merging is up to you; `--force` replaces) |
| **User-owned** | `ProjectRule.md`, `Session_Registry.tsv`, `README.md`, all work data and logs | **Never touched** |
| **Pointer (create-only)** | `CLAUDE.md` | Created if missing; if present it is **never modified** (regardless of content — existing-file ownership is respected). `elf doctor` checks whether it loads `@AGENTS.md` |
| **Partially managed** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` ~ `# <<< ELF managed <<<`) is replaced; user rules outside the block are preserved |

- For project-rule customization, prefer **writing in `ProjectRule.md`** instead of editing ELF-managed files (no update conflicts).
- Check status: `elf status` (diagnoses without changes; `--check` is for CI/hook gates).

---

## 3. AI Communication Rules

> **PROJECT_LANG**: see the `lang` field (a BCP-47 tag, e.g., `ko-KR`) in `.elf/config.json`. The AI agent decides its response language from this value.

Every AI agent in the project follows these principles when communicating with the user and writing documents:

1. **Response language**: respond in both the language set in `PROJECT_LANG` and English. Use the `PROJECT_LANG` language for logs and documents as well. Technical terms may carry the English original alongside.
2. **Objective, dry style**: no unnecessary greetings, excessive praise, subjective emotional expression, or exaggerated adjectives.
3. **No metaphor**: avoid metaphor and simile; convey facts only in plain, objective academic/engineering terms. In particular, drop the LLM cliché metaphors that recast a viewpoint/approach as a "lens" or analysis/exploration as "navigate" / "deep dive" / "journey," and use direct terms (viewpoint, approach, analysis, process).
4. **Conclusion-first clarity**: present analysis results and action items concisely and clearly, covering only logical, precise facts.
5. **Reproducibility**: preserve work outputs (code, documents, data) and important intermediate results **in an appropriate format/location so they are reproducible**. Do not lose previous versions by overwriting.
6. **No embellishment**: fully avoid emotionally charged or extreme modifiers — "overwhelming," "weapon," "fatal," "impactful." Describe pros and cons only with quantitative figures and cause and effect.
7. **No emojis**: do not use icons or emojis in any document or response.
8. **Token economy**: the base philosophy of AI communication and logging is token minimization. Prefer compact, word-centered bullet points over full sentences, and omit unnecessary particles and conjunctions.
9. **Sentence-ending form**: in Korean writing, fully avoid "~입니다/습니다" and the 해요 style; even when prose is needed, end with a nominal ending ('-음', '-함', '-임') or a concise '-다'. This ending form does not apply to non-Korean (e.g., English) output; for that, apply the equivalent intent of §§2 and 8 (dry, compact style — concise, active, no filler).
10. **Structured logging**: clearly separate observation (fact) from analysis (interpretation); compress file lists, conditions, etc. into Markdown tables rather than prose to maximize information density.
11. **Abbreviations**: define an abbreviation in full on first use (`AR (asymmetry ratio)`). In trials/documents where an abbreviation recurs, list an abbreviation legend as a ul list under `### Conditions` (or at the top of the document), with **each abbreviation on its own row** — keeping a trial atom independently readable. If the same abbreviation spans multiple trials, **redefine it in each trial** (atom independence > DRY; a single session-wide definition is not enough). Exception: domain-standard units/symbols (µA, Hz, ms, SI) need no gloss.
12. **LLM cliché ban**: when writing English documents/communication, drop LLM-characteristic cliché vocabulary/register (signature verbs, vague adjectives, cliché nouns, overused connectors, formulaic constructions) and replace with **concrete, active, direct** statements. Apply as a *principle*, not a fixed list — drop vague filler and overused register, but allow terms with a **precise technical meaning** (e.g., statistical significance, robustness). For connectors, drop only overuse and sentence-initial runs (a justified single use is fine); quotations, titles, and source wording are exempt; Korean dry logs are unaffected (the rule applies when English is mixed in). Declare project-specific technical homonym exceptions in `ProjectRule.md`. Exclusions, exceptions, and before/after examples (non-exhaustive reference): `0_Meta/LLMcliche.md`.
13. **Source Reliability**: do not cite namu.wiki (나무위키) or other anonymous/collectively-edited wikis, or unattributed blogs/community posts, as the source of an answer or document. Even if they surface in search, do not trust them as-is — cross-check against **reliable sources** (authoritative institutions, academic societies, governments, scholarly/primary sources, official docs, primary reporting) and cite *that* source. Set `blocked_domains: ["namu.wiki"]` by default in WebSearch. Wikipedia is only a starting point — trace to the primary source. **For any web-grounded answer or document, list the verified reliable sources under `## 출처` (or `## Sources`).**
