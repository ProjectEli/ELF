# EliRule: Project Structure & Operational Guide

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `EliRule.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

This document defines the folder structure and operating rules of an ELF (Eli's Lab
Framework) project. Where `README.md` covers philosophy and overview, this document is
the practical, detailed specification.

---

## 1. Folder Structure

### Core (always included)

#### `.elf/` — ELF control plane
Where the `elf` CLI records project state (version, settings, list of managed files).
**Do not edit directly** — managed by `elf init` / `elf update`.

#### `0_Meta/` — project governance
A meta area defining project operating rules rather than research data.
- `ProjectRule.md`: project-specific rules and goals (**user-owned** — edit freely to fit the project)
- `EliRule.md`: this document (folder structure and operating guide)
- `LogConvention.md`: logging standard rules
- `AI_PARA_Framework.md`: state-based file-management and archiving rules that prevent AI hallucination. The most important reference document when the AI explores the project
- `highIFjournals.md`: high-IF journal whitelist for external literature search (see §4)

#### `1_Concept/` — research planning & ideas
Keeps research direction, literature review, and hypotheses separate from experimental data.
- **`11_Literature/`**: paper PDFs, bibliographic info, foundational formulas
- **`12_Planning/`**: research planning, roadmaps, figure storyboards (multi-session / roadmap unit)
  - Planning documents are numbered `P###_title.md` (e.g., `P001_wavelength_optimization.md`)
  - `Wiki/`: summaries of planning-stage conclusions and key rules
- **`13_Ideas/`**: snippets too small for a session, and early naive ideas. Flat (no Archive, never discarded). Promote to a Planning document or a session trial once mature.

#### `2_Log/` — session logs
The top-level space for logging all kinds of work (experiments, planning, software development, etc.).
- `S###_log.md`: session log file (format: see `0_Meta/LogConvention.md`)
- `Wiki/`: key-finding summaries and the Session Registry
- `Archive/`: completed session logs

#### `templates/` — markdown stubs
- `sessionTemplate.md`: copy to `2_Log/S###_log.md` when starting a new session
- `trialTemplate.md`: paste into the body when adding a trial (t##) to an in-progress session
- **Planning documents (`P###`) are intentionally template-free** — trials are disciplined by templates for reproducibility, but planning is the researcher's free exploration, so no format is imposed.

### Modules (optional)

> Include only the modules you need via `elf init`'s `--preset` (full/experimental/software/minimal) or `--modules`.

#### `3_HW/` — hardware design
Separates the device's physical design into components and the integrated system.
- **`31_Component/`**: individual part specs, unit-device design
  - `Design/`: design files
  - `Calibration/`: calibration data and settings
- **`32_System/`**: integrated instrument design, housing, 3D models (`.stl`, `.step`)
- **`33_Elec/`**: PCB schematics, Gerber, BOM, datasheets

#### `4_Fab/` — fabrication & process
Manages part/device fabrication-process records and characterization.
- **`41_Recipes/`**: process-condition documentation
- **`42_Eval/`**: per-module single-characteristic evaluation data

#### `5_SW/` — software & firmware
- **`51_FW/`**: MCU/embedded firmware source
- **`52_DAQ/`**: PC/mobile data-acquisition systems
- **`53_Libs/`**: reusable shared libraries (filters, SNR computation, etc.)

#### `6_Exp/` — experiments (Sim + Empirical + Analysis)
A structure for 1:1 comparison and validation of simulation against measured data.
- **`61_Sim/`**: simulation
  - `Scripts/`: simulation code (`S###_sim.m`, etc.); `Archive/`: discarded scripts
  - `Data/`: simulation results (`Data/S###/`)
- **`62_Empirical/`**: measured data
  - `Raw/`: original sensor data (**read-only, excluded from Git**)
  - `Processed/`: first-pass processed data
- **`63_Analysis/`**: integrated analysis
  - `Scripts/`: comparison/validation post-processing code; `Archive/`: discarded scripts
- **`64_Viz/`**: auto-generated visualization outputs (figure PNGs, etc.)

#### `7_Paper/` — papers & presentations
- **`71_Figs/`**: paper figures
  - `Raw/` → `Processed/` → `Final/` (three-stage pipeline)
- **`72_Drafts/`**: manuscripts (Word, LaTeX); `Archive/`: previous versions
- **`73_Presentations/`**: presentation materials (PPT, posters)

---

## 2. Operating Rules

> **Traceability principle**: As a rule, **record all work, changes, and decisions in the session log (`2_Log/S###`)**. Meaningful code/data/document changes and attempts/results (including failures) are left in the work turn to secure reproducibility and traceability — trivial fixes (typos, paths) are not forced, but changes affecting outputs, conclusions, or direction are not omitted. For detailed format and base-delta rules, see `LogConvention.md`.

### 2.1 Raw-data integrity
- Files stored in `6_Exp/62_Empirical/Raw/` are **read-only**.
- Scripts only read them; never overwrite the originals.

### 2.2 Git separation strategy
- **Tracked by Git**: code, metadata, logs, analysis figures, manuscripts — project outputs in general
- **Excluded from Git**: `6_Exp/62_Empirical/Raw/` (large original sensor data), tool temp files
- Large design files (`3_HW/`) are best managed with Git LFS or a separate drive.
- Analysis figures (`.png`, etc.) and manuscripts (`.docx`, etc.) are tracked in Git for version control.

### 2.3 Naming convention
- **Session-Trial**: `S###_t##` (e.g., `S001_t1.csv`)
- **Do not** list experimental conditions/variables in file names — record all conditions in the log
- Planning documents: `P###_title.md` (e.g., `P001_experiment_roadmap.md`)
- Simulation scripts: `S###_sim.m`
- Analysis scripts: `S###_analysis.m`

### 2.4 Separate scripts from data
- Analysis code goes in `Scripts/`, data in `Data/` or `Raw/`/`Processed/`
- Do not mix code inside data folders.

### 2.5 Cross-reference rules
- Reference a Planning document from a log: `→ see 1_Concept/12_Planning/P001_xxx.md`
- Reference simulation data from a log: `→ see 6_Exp/61_Sim/Data/S###/`
- Reference an analysis script from a log: `→ see 6_Exp/63_Analysis/Scripts/S###_analysis.m`

### 2.6 Data reusability (permanent-preservation principle)
- For every plot/graph except simple illustrations, **also export the original data array as a `.mat` (or `.csv`) file so it can be reused later** — including metrics or intermediate results not visible on the graph surface.

### 2.7 ELF-managed files and updates (`elf update`)

Project files fall into three ownership classes, which `elf update` strictly respects:

| Class | Files | `elf update` behavior |
|------|------|-------------------|
| **ELF-managed** | `EliRule.md`, `LogConvention.md`, `AI_PARA_Framework.md`, `highIFjournals.md`, `LLMcliche.md`, `templates/*`, `.claudeignore` | Replaced with the new version. **If you edited it, it is not overwritten**; the new version is written as `<file>.elf-new` (merging is up to you; `--force` replaces) |
| **User-owned** | `ProjectRule.md`, `Session_Registry.tsv`, `README.md`, all research data and logs | **Never touched** |
| **Partially managed** | `.gitignore` | Only the marker block (`# >>> ELF managed >>>` ~ `# <<< ELF managed <<<`) is replaced; user rules outside the block are preserved |

- For project-rule customization, prefer **writing in `ProjectRule.md`** instead of editing ELF-managed files (no update conflicts).
- Check status: `elf status` (diagnoses without changes; `--check` is for CI/hook gates).

---

## 3. AI Communication Rules

> **PROJECT_LANG**: see the `lang` field in `.elf/config.json` (set at project creation). The AI agent decides its response language from this value.
> This EliRule is a project-independent common rule (ELF-managed), so the language value is not kept in the body — it lives separately in `.elf/config.json`.

Every AI agent in the project follows these principles when communicating with the user and writing documents:

1. **Response language**: respond in both the language set in `PROJECT_LANG` and English. Use the `PROJECT_LANG` language for logs and documents as well. Technical terms may carry the English original alongside.
2. **Objective, dry style**: no unnecessary greetings, excessive praise, subjective emotional expression, or exaggerated adjectives.
3. **No metaphor**: avoid metaphor and simile; convey facts only in plain, objective academic/engineering terms. In particular, drop the LLM cliché metaphors that recast a viewpoint/approach as a "lens" or analysis/exploration as "navigate" / "deep dive" / "journey," and use direct terms (viewpoint, approach, analysis, process).
4. **Conclusion-first clarity**: present analysis results and action items concisely and clearly, covering only logical, precise engineering facts.
5. **Data reusability**: strictly follow §2.6 above.
6. **No embellishment**: fully avoid emotionally charged or extreme modifiers — "overwhelming," "weapon," "fatal," "impactful." Describe pros and cons only with quantitative figures and physical cause and effect.
7. **No emojis**: do not use icons or emojis in any document or response.
8. **Token economy**: the base philosophy of AI communication and logging is token minimization. Prefer compact, word-centered bullet points over full sentences, and omit unnecessary particles and conjunctions.
9. **Sentence-ending form**: in Korean writing, fully avoid "~입니다/습니다" and the 해요 style; even when prose is needed, end with a nominal ending ('-음', '-함', '-임') or a concise '-다'. This ending form does not apply to non-Korean (e.g., English) output; for that, apply the equivalent intent of §§2 and 8 (dry, compact style — concise, active, no filler).
10. **Structured logging**: clearly separate experimental observation (observation) from physical analysis (interpretation); compress file lists, parameter conditions, etc. into Markdown tables rather than prose to maximize information density.
11. **Abbreviations**: define an abbreviation in full on first use (`AR (asymmetry ratio)`). In trials/documents where an abbreviation recurs, list an abbreviation legend as a ul list under `### Conditions` (or at the top of the document), with **each abbreviation on its own row** — keeping a trial atom independently readable. If the same abbreviation spans multiple trials, **redefine it in each trial** (atom independence > DRY; a single session-wide definition is not enough). Exception: domain-standard units/symbols (µA, Hz, ms, SI) need no gloss.
12. **LLM cliché ban**: when writing English documents/communication, drop LLM-characteristic cliché vocabulary/register (signature verbs, vague adjectives, cliché nouns, overused connectors, formulaic constructions) and replace with **concrete, active, direct** statements. Apply as a *principle*, not a fixed list — drop vague filler and overused register, but allow terms with a **precise technical meaning** (e.g., statistical significance, robustness). For connectors, drop only overuse and sentence-initial runs (a justified single use is fine); quotations, titles, and source wording are exempt; Korean dry logs are unaffected (the rule applies when English is mixed in). Declare project-specific technical homonym exceptions in `ProjectRule.md`. Exclusions, exceptions, and before/after examples (non-exhaustive reference): `0_Meta/LLMcliche.md`.
13. **Source Reliability**: do not cite namu.wiki (나무위키) or other anonymous/collectively-edited wikis, or unattributed blogs/community posts, as the source of an answer or document. Even if they surface in search, do not trust them as-is — cross-check against **reliable sources** (authoritative institutions, academic societies, governments, scholarly/primary sources, official docs, primary reporting) and cite *that* source. Set `blocked_domains: ["namu.wiki"]` by default in WebSearch. Wikipedia is only a starting point — trace to the primary source. **For any web-grounded answer or document, list the verified reliable sources under `## 출처` (or `## Sources`).**

---

## 4. External Source Retrieval

Applies when surveying academic literature with web-search tools. Goal: secure high-tier (high-IF) journals first.

- **Background**: web-search tools run on general search engines (not JCR/Scopus/WoS) → no Impact-Factor sorting/filtering. High-volume OA publishers (MDPI, Frontiers, Hindawi, etc.) occupy search listings through SEO and publication volume. High-IF flagships (Nature, Science, Cell, subscription Elsevier/Wiley) block full-text fetch with paywalls and anti-bot (403). → when the intent is a high-tier survey, results skew toward low-IF OA. The cause is "no IF-based sorting + OA-indexing bias," not "the engine cannot see high-IF"; access blocking is a partial factor.
- **Rules**:
  1. Domain whitelist: set the domain list in `0_Meta/highIFjournals.md` as `allowed_domains` when searching. Define the per-project priority target subset in `ProjectRule.md`.
  2. OA full-text route: if a target journal is OA and the publisher domain returns 403, get the full text via a PMC / Europe PMC URL.
  3. Bibliometric APIs: use OpenAlex (`api.openalex.org`), Semantic Scholar, and Crossref for citation counts, venue sorting, and quality filtering.
  4. Journal-name queries: add journal-name tokens to the query to raise ranking.
  5. Preprint route: get full text of high-IF publications from arXiv, bioRxiv, medRxiv.
  6. Exclude low-IF OA (optional): use the search tool's domain-exclusion feature when tier enforcement is needed. Beware of excluding legitimate venues alongside — do not exclude unconditionally.
  7. Verify before citing: "search exposure ≠ high quality." Cross-check venue + citation count. Do not fabricate unverified references (consistent with `AI_PARA_Framework.md` hallucination prevention).
- **Reference**: the full list of domain whitelist, OA mirrors, and bibliometric APIs is in `0_Meta/highIFjournals.md` (update the list only in that file; rule documents reference it).
