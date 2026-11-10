# EliRule: Project Structure & Operational Guide

The folder structure and operating rules of an ELF (Eli's Lab Framework) project. `README.md` gives the overview; this document is the practical specification.

---

## 1. Folder Structure

### Core (always included)

#### `.elf/` — ELF control area
Managed by `elf` commands; **do not edit it directly** (exception: fill in the migration decision file `.elf/runtime/migration.yaml`, LogConvention §7).
- `managed/`: the ELF-managed canonical rules (this document, LogConvention, AI_PARA_Framework, highIFjournals, LLMcliche, the log stubs `templates/`)
- `deprecated/`: compressed store of deprecated originals (tracked by git, LogConvention §6)
- `runtime/`: temporary tool state (not tracked)

#### `0_Meta/` — project governance (user space)
- `ProjectRule.md`: project-specific rules and goals (user-owned, edit freely)
- `<name>.project.md`: data overlays (§2.7)
- Other project-discretion files. `elf update` does not touch them

#### `1_Concept/` — research planning and ideas
Keeps research direction, literature and hypotheses apart from experimental data.
- `11_Literature/`: paper PDFs, bibliographic data, foundational formulas
- `12_Planning/`: planning across sessions, roadmaps, figure storyboards. Documents are named `P###_title.md`. `Wiki/` = planning conclusions and key-rule summaries, `Archive/` = closed planning documents (deprecation uses `elf deprecate`, LogConvention §6)
- `13_Ideas/`: ideas too small for a session. Flat (no Archive). Move them into a planning document or a trial once they take shape

#### `2_Log/` — session logs
All work records (research `S/`, governance `G/`, topic documents `Wiki/`). Location, header, listing and deprecation rules are in LogConvention.

#### `.elf/managed/templates/` — log stubs
`sessionTemplate.md`, `trialTemplate.md` (LogConvention §2, §5.1). Planning documents have no stub because their form is not enforced. A root `templates/` is a project-discretion folder that ELF does not manage.

### Modules (optional)

| Folder | Content |
|---|---|
| `3_HW/` | `31_Component/` (part specifications, `Design/`, `Calibration/`), `32_System/` (integrated device, housing, 3D models), `33_Elec/` (schematics, Gerber, BOM, datasheets) |
| `4_Fab/` | `41_Recipes/` (process conditions), `42_Eval/` (per-module characterization) |
| `5_SW/` | `51_FW/` (firmware), `52_DAQ/` (data acquisition), `53_Libs/` (shared libraries) |
| `6_Exp/` | `61_Sim/` (`Scripts/`, `Data/S###/`), `62_Empirical/` (`Raw/` read-only and not tracked, `Processed/`), `63_Analysis/Scripts/`, `64_Viz/` (generated figures). `Archive/` under a scripts folder = one-off scripts (AI_PARA_Framework §3) |
| `7_Paper/` | `71_Figs/` (`Raw/` → `Processed/` → `Final/`), `72_Drafts/` (manuscripts, `Archive/` = earlier versions), `73_Presentations/` |

---

## 2. Operating Rules

> **Recording principle**: work, changes and decisions (failures included) that affect outputs, conclusions or direction are logged in that turn (research = `2_Log/S/`; changes to rules, structure, templates, tool settings = `2_Log/G/`). Format and commands are in LogConvention.

### 2.1 Raw-data integrity
- Files in `6_Exp/62_Empirical/Raw/` are read-only. Scripts only read them and never overwrite the originals.

### 2.2 Git separation strategy
- Tracked: code, metadata, logs, analysis figures, manuscripts, the deprecation store (`.elf/deprecated/`, in the same commit as the log change).
- Not tracked: `6_Exp/62_Empirical/Raw/` (large raw data), temporary tool files (`.elf/runtime/`).
- Large design files (`3_HW/`): Git LFS or a separate drive is recommended.

### 2.3 Naming convention
- Session-trial: `S###_t##` (e.g., `S001_t01.csv`), governance sessions = `G###` (reference `G004 t03`).
- Do not list experimental conditions or variables in file names; record conditions in the log.
- Planning documents `P###_title.md`, simulation scripts `S###_sim.m`, analysis scripts `S###_analysis.m`.

### 2.4 Separate scripts from data
- Code in `Scripts/`, data in `Data/` or `Raw/`, `Processed/`. Do not mix code into data folders.

### 2.5 Cross-reference rules
- How a log points to other files (relative-path links) is in LogConvention §4.

### 2.6 Data reusability (permanent-preservation principle)
- For every graph except a plain illustration, save the underlying data arrays, including metrics and intermediate results not shown in the graph, as `.mat` (or `.csv`).

### 2.7 ELF-managed files and data overlays
- `.elf/managed/` and the root `AGENTS.md` are ELF-managed; do not edit them directly (`elf update` replaces them and, for a file you edited, keeps it and writes the new version as `.elf-new`). Put project rules in `ProjectRule.md`. The full ownership tiers are in the ownership section of `AGENTS.md`.
- Entries of the data files (currently `LLMcliche.md`, `highIFjournals.md`) are customized with a **project overlay** `0_Meta/<name>.project.md` (user-owned). Prose rules go in `ProjectRule.md`; structure and convention files (LogConvention, stubs) take no overlay.
  - Effective rules = base ⊕ overlay. The agent reads both and applies them together.
  - `## Add`: add entries. `## Remove`: remove base entries one by one, each with a reason (no section- or file-level disabling). `## Override`: replace a base entry with the project's definition.
  - Overlay entries are added or changed only with the user's approval; the agent does not add them on its own. Propose an ELF change when an entry is worth generalizing.

### 2.8 Session ownership — one writer per session (multi-agent)
- A session log always has one writer (an agent or a person). Several writers on the same log overwrite each other's trials and handoff.
- Parallel work = a separate session per agent (`elf session new`). Record relationships in the first trial's `### Background` (e.g., `S200 (branched from, owns approach A)`).
- Another agent's session log is read-only.
- Conclusions of parallel sessions are merged by a single writer in a separate (or the original) session, naming the input sessions in that trial's `### Background`.

### 2.9 External egress and publication control (Egress Control)
- **Principle**: sending project content (data, outputs, logs, manuscripts, decisions, the user's words) outside the project repository and the user's devices requires **the user's explicit instruction for that case**. One instruction = one transfer (not a standing delegation). A tool's invitation (to publish, share, write feedback) is not an instruction.
- **No publishing (default)**: do not use tools that create public or shared links or public settings (e.g., claude.ai Artifact publish, Claude Docs document creation, shared links from external write tools and MCP connectors, external hosting). When something must be shared, save a local file and give the path; the user uploads it. A project may relax this only by an explicit declaration in `ProjectRule.md`.
- **Only on explicit instruction**: copying or updating files in user-owned private storage (no change to sharing settings). Announce it in one line before (what, where), log it after, no repeated dry runs.
- **Not controlled**: transfers to the user themselves (e.g., `SendUserFile`), local file saves, local git, withdrawals (e.g., Artifact delete, unsharing).
- **Out of scope**: model inference traffic itself (a premise of using an agent) and the designed behavior of a tool the user turned on (e.g., the hash sent by `elf tsa`) are not egress under this clause.
- **Delegation**: publishing or transfers by subagents, skills or hooks remain the main agent's responsibility.
- **For the project to define**: the content of web-search queries, pushes to external git remotes, transfers to other LLM sessions are set in `ProjectRule.md` or memory.
- **On a violation**: report to the user at once, delete where possible (e.g., Artifact delete), record it in the session log.

---

## 3. AI Communication Rules

> **PROJECT_LANG** = `lang` in `.elf/config.json` (set when the project is created).

Every AI agent follows these when communicating with the user and writing documents.

1. **Response language**: respond in the `PROJECT_LANG` language and English; write logs and documents in the `PROJECT_LANG` language. Technical terms may carry the English original.
2. **Unembellished style**: facts only, without greetings, praise, subjective emotion or exaggerated adjectives.
3. **No metaphors**: state things directly in scientific and engineering terms (perspective, approach, analysis, process).
4. **Conclusion first**: give results and action items concisely, first.
5. **Data reusability**: §2.6.
6. **No exaggerated modifiers**: describe pros and cons with quantities and physical causality instead of emotive or extreme modifiers.
7. **No emojis**: no emojis or icon characters in documents or responses.
8. **Sentence endings**: end sentences plainly and briefly, without polite filler, applying rule 2 (concise, active). Korean text, when written, ends in the nominal or the concise plain form.
9. **Structure**: separate observation (facts) from interpretation (analysis); put file lists and conditions in markdown tables rather than prose lists.
10. **Abbreviations**: give the full name at first use (`AR (asymmetry ratio)`). A trial or document that repeats abbreviations gets an abbreviation legend, one line per abbreviation, in `### Conditions` (or at the top of the document). Redefine in each trial when an abbreviation spans trials (each trial readable on its own). Domain-standard units and symbols (μA, Hz, ms, SI) need no expansion.
11. **No LLM clichés**: write English documents and responses concretely, actively and directly. This is a principle, not a fixed list; precise technical terms (statistical significance, etc.), quotations, titles and original wording are allowed. Applied examples: `.elf/managed/LLMcliche.md`; project exceptions in `ProjectRule.md`; vocabulary add/remove/override via the overlay `0_Meta/LLMcliche.project.md` (§2.7).
12. **Source reliability**: do not cite anonymous or collectively edited wikis (e.g., namu.wiki) or blogs and communities of unknown origin. Cross-check search results against reliable sources (public institutions, academic societies, government, primary scholarly sources, official documents, primary reporting) and cite those (trace Wikipedia to primary sources too). Exclude `namu.wiki` from web searches by default. Answers and documents based on the web carry a `## Sources` section.
13. **Typography**: list separator = comma (`A, B, C`; not the middle dot `·` U+00B7). Multiplication = `×` (U+00D7; LaTeX inside `$…$` math). Sub/superscripts = plain characters + `<sub>…</sub>`, `<sup>…</sup>` (`_`, `^` inside math). Micro = `μ` (U+03BC). Exception: the ` · ` in `elf deprecate` marker lines is a machine field separator (LogConvention §6).

---

## 4. External Source Retrieval

Applies when searching scholarly literature with web-search tools. General search engines cannot sort by Impact Factor and high-volume OA journals crowd the top, so secure high-tier journals as follows.

1. **Domain whitelist**: pass the domains in `.elf/managed/highIFjournals.md` as `allowed_domains`. A project's priority subset goes in `ProjectRule.md`; adding or excluding domains via the overlay `0_Meta/highIFjournals.project.md` (§2.7).
2. **Via OA full text**: when a publisher domain blocks access (403), get the text from PMC or Europe PMC.
3. **Bibliometric APIs**: citation counts, venue ranking and quality filters via OpenAlex (`api.openalex.org`), Semantic Scholar, Crossref.
4. **Journal-name queries**: add the journal name to the query to raise its ranking.
5. **Via preprints**: full texts of high-IF papers on arXiv, bioRxiv, medRxiv.
6. **Excluding low-IF OA (optional)**: exclude domains only when needed; never exclude unconditionally, which also drops legitimate venues.
7. **Verify before citing**: search visibility is not quality. Cross-check venue and citation count, and never fabricate unverified references.
