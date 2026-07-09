[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF): Base-Delta Protocol for Agile R&D

A hardware-software-experimental data integrated logging standard (Protocol) designed to support fast feedback loops (Agile) during device development and R&D validation phases. Guarantees complete data traceability while minimizing researcher documentation fatigue.

## Core Philosophy

* **Single Source of Truth:** Hardware design, analysis code, and raw data are organically connected within a single project.
* **Base-Delta Logging:** Not every variable is recorded. A Baseline is declared, and only changed variables (Deltas) are logged lightly to prevent research delays.
* **Systematic Enforcement:** Bypasses file name length limitations (Windows 260-character limit) and guarantees reproducibility through code.
* **AI Governance:** Enforces a unified logging standard for both humans and AI through `.elf/managed/LogConvention.md`, and prevents context contamination via `.elf/managed/AI_PARA_Framework.md`.

## Project Directory Structure

This project treats the folder hierarchy itself as a communication standard.

```text
Project_Root/
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF control plane (version·config·manifest — do not edit)
│   └── managed/                     # Managed rule payload: EliRule·LogConvention·AI_PARA_Framework
│       └── templates/               #   ·LLMcliche·highIFjournals + session/trial stubs
├── 0_Meta/                          # Project governance — yours (`elf update` never writes here)
│   ├── ProjectRule.md               # Project-specific rules and objectives
│   └── <name>.project.md            # Data overlays (effective rules = base ⊕ overlay)
│
├── 1_Concept/                       # Research planning, literature, ideas
│   ├── 11_Literature/               # Paper PDFs, bibliographic info, base formulas
│   ├── 12_Planning/                 # Research plans, roadmaps (multi-session)
│   │   └── Wiki/                    # Distilled planning conclusions & key rules
│   └── 13_Ideas/                    # Small snippets / naive early ideas (flat)
│
├── 2_Log/                           # Session logs (S###_log.md)
│   ├── Wiki/                      # Distilled findings & session registry
│   └── Archive/                   # Completed session logs
│
│  ─── Modules (Optional) ────────────────
│
├── 3_HW/                            # Hardware design
│   ├── 31_Component/                # Individual component specs, unit device design
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 32_System/                   # Integrated device design, housing, 3D models
│   └── 33_Elec/                     # PCB schematics, Gerber, BOM, Datasheets
│
├── 4_Fab/                           # Fabrication and processing
│   ├── 41_Recipes/                  # Process condition documentation
│   └── 42_Eval/                     # Per-module single characteristic evaluation
│
├── 5_SW/                            # Software & firmware
│   ├── 51_FW/                       # MCU/embedded firmware
│   ├── 52_DAQ/                      # PC/mobile data acquisition systems
│   └── 53_Libs/                     # Reusable shared libraries
│
├── 6_Exp/                           # Experiments: simulation + empirical + analysis
│   ├── 61_Sim/                      # Simulation
│   │   ├── Scripts/                 # Simulation code (S###_sim.m)
│   │   │   └── Archive/          # Retired scripts
│   │   └── Data/                    # Simulation results (Data/S###/)
│   ├── 62_Empirical/                # Empirical data
│   │   ├── Raw/                     # Raw sensor data (Read-Only, excluded from Git)
│   │   └── Processed/               # Primary processed data
│   ├── 63_Analysis/                 # Integrated analysis
│   │   └── Scripts/                 # Comparison/validation post-processing code
│   │       └── Archive/           # Retired scripts
│   └── 64_Viz/                      # Visualization outputs (auto-generated figures)
│
├── 7_Paper/                         # Papers & presentations
│   ├── 71_Figs/                     # Figures for papers
│   │   ├── Raw/
│   │   ├── Processed/
│   │   └── Final/
│   ├── 72_Drafts/                   # Manuscripts (Word, LaTeX)
│   │   └── Archive/               # Previous versions
│   └── 73_Presentations/            # Presentation materials (PPT, posters)
```

> For detailed usage and operational rules for each folder, refer to `.elf/managed/EliRule.md`.

## Data Logging Pipeline Specification

### 1. File Naming Convention (Session-Trial Naming)

* Listing experimental conditions or variable information in file names is **strictly prohibited**.
* **Format:** `[SessionID]_[TrialID].[extension]` (e.g., `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Logging (Hybrid Logging)

* **Running Log (`2_Log/S###_log.md`):**
  * A narrative markdown file that records immediate hypothesis-test-lesson cycles in text.
  * Written per trial (`t1`, `t2`...) in a stream-of-consciousness style, recording only the **intentionally changed variables (Delta)** and observed results.
  * Format and detailed rules: refer to `.elf/managed/LogConvention.md`.

### 3. Planning Document Rules

* Research roadmaps, figure compositions, experimental strategies, etc. are managed separately in `1_Concept/12_Planning/`.
* **Format:** `P###_title.md` (e.g., `P001_wavelength_optimization.md`)
* When referencing Planning from a log: `→ see 1_Concept/12_Planning/P###_xxx.md`

### 4. Post-Processing Analysis Specification (Cell Mode Scripting)

* Analysis code must be located in `6_Exp/63_Analysis/Scripts/` or `6_Exp/61_Sim/Scripts/` and must not be mixed inside data folders.
* Pure `.m` files are used instead of `.mlx` to prevent vendor lock-in.
* Code is executed section by section using `%%` (Cell Mode), and derived insights are reflected in the running log.
* Analysis outputs (figures, mat files) are saved in `6_Exp/64_Viz/` or `6_Exp/62_Empirical/Processed/S###/` within per-session folders.

### 5. Cross-Reference Rules

Cross-reference formats are unified to ensure traceability between project documents.

| From → To | Format |
|-----------|--------|
| Logs → Planning | `→ see 1_Concept/12_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ see 6_Exp/61_Sim/Data/S###/` |
| Logs → Script | `→ see 6_Exp/63_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← tracked in 2_Log/S###_log.md` |

## AI Governance

When AI agents (Claude, etc.) participate in the project, the following rules apply:

1. **Context Acquisition:** Before starting work, read the active session log in `2_Log/` and `2_Log/Wiki/Session_Registry.tsv` to confirm the state of previous work.
2. **Unified Standard Compliance:** Follow the logging rules in `.elf/managed/LogConvention.md` in the same way as a human researcher.
3. **Handoff Recording:** Upon task completion, record performed actions, created/modified files, and Next Steps in the session log (`2_Log/S###_log.md`) — use the log header's `Handoff` field.
4. **Idea Separation:** Hypotheses and ideas generated by AI are stored separately in `1_Concept/` (small ideas → `13_Ideas/`, plans → `12_Planning/`), not in logs.
5. **PARA-Based Context Management:** Use the `Archive/` folder and `.claudeignore` to prevent AI context contamination. For detailed rules, refer to `.elf/managed/AI_PARA_Framework.md`.
6. **Communication Rules:** Maintain an objective and dry writing style. No analogies or metaphors. Deliver conclusions clearly and directly. No exaggeration or emotional modifiers. For detailed rules, refer to section 3 of `.elf/managed/EliRule.md`.
7. **Data Reusability:** When generating any Plot/Graph, save the original Data Array alongside as `.mat`/`.csv`. For detailed rules, refer to section 2.6 of `.elf/managed/EliRule.md`.
8. **Data-File Customization:** Add/remove/override vocabulary or search domains via a project overlay `0_Meta/<name>.project.md` (user-owned; effective rules = base ⊕ overlay; each removal states a reason). Spec: `.elf/managed/EliRule.md` §2.7.

## Quick Start

### 1. Install the `elf` CLI (recommended)

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

A self-contained single binary is installed to `~/.elf/bin` and added to PATH — no Node/Python runtime required. Verify with `elf --version` in a new shell.

### 2. Create a project

```bash
cd /desired/parent/directory
elf init MyProject --lang English                   # default: full preset
elf init MyProject --preset experimental --lang English   # 6_Exp + 7_Paper only
elf init MyProject --modules hw,sw --lang English   # custom module selection
```

Core folders (0~2) are always created; module folders (3~7) are included per preset (`full`/`experimental`/`software`/`minimal`) or `--modules` selection.


## CLI Commands & Usage Scenarios

> Full command reference (every flag, exit codes, escalation, file ownership): **[elf-cli/CLI.md](elf-cli/CLI.md)**

| Command | Role |
|---------|------|
| `elf init <name> [--preset …] [--modules …] [--lang …]` | Scaffold a new project |
| `elf update [--dry-run] [--force]` | Update ELF-managed files in a project to the current CLI version — **never touches your files** |
| `elf status [--check]` | Diagnose managed-file state (read-only). `--check` exits 4 on findings |
| `elf validate [--check]` | Check session/registry/log consistency (read-only). `--check` exits 4 on issues |
| `elf session new <title>` | Create + register the next session log (auto-increments `S###`) |
| `elf session close [S###]` | Close the active session → archive + update registry (fixes cross-refs) |
| `elf session fix-headers` | Repair session-log header hard breaks (`\`) |
| `elf gallery` | Generate the figure index `_gallery.md` from `6_Exp/64_Viz/` |
| `elf doctor` | Aggregate environment + project health check (read-only) |
| `elf self-update` (= `elf update --self`) | Update the `elf` binary itself to the latest release |

### Scenario A — Start a new project

```bash
elf init NIRS_Probe --preset experimental --lang English
cd NIRS_Probe
# Read .elf/managed/EliRule.md · LogConvention.md → start research in 2_Log/S001_log.md
```

### Scenario B — Apply a new ELF version to an existing project

```bash
elf self-update          # ① update the CLI itself
cd MyProject
elf status               # ② diagnose what would change (outdated / edited / missing)
elf update --dry-run     # ③ preview the action list without writing
elf update               # ④ update — replaces ELF-managed files only; research data, logs, and your settings are never touched
```

> **Upgrading a project created before v2.15** (rules still in `0_Meta/`, stubs in a root
> `templates/`): the current CLI does not read or migrate that layout. Take the two-step
> path — install **v2.15.1** from the [Releases page](https://github.com/ProjectEli/ELF/releases/tag/v2.15.1),
> run `elf update` and then `elf migrate` there, and only then update the CLI to the
> latest (`elf self-update`). Running the latest `elf update` directly on such a project
> deploys a second copy of the rules under `.elf/managed/` and leaves the old files as
> unmanaged leftovers — it warns and names them, but nothing is deleted or migrated.

### Scenario C — When an ELF-managed file you edited conflicts

```bash
elf update
# → "edited: .elf/managed/LogConvention.md — kept; new version at ….elf-new"
#   Your edit is preserved; the new version lands as <file>.elf-new → diff and merge manually
elf update --force       # or: discard your edits and replace with the canonical version
```

For `.gitignore`, ELF manages only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`); user rules outside the block are always preserved.

### Scenario D — Gate drift in teams/CI

```bash
elf status --check       # exits 4 on findings → use as a pre-commit hook / CI gate
```

## Usage

> **Tip — drive the session lifecycle with the CLI:** the workflow below can be automated — `elf session new` (start), `elf gallery` (figure index), `elf validate` (consistency check), `elf session close` (complete). The manual steps remain valid for any workflow, so adopt the CLI incrementally.

### 0. Templates

The `.elf/managed/templates/` folder provides ready-to-use stubs:

| File | When to use |
|------|-------------|
| `sessionTemplate.md` | Copy to `2_Log/` when starting a new session (rename to `S###_log.md`) |
| `trialTemplate.md` | Paste into an active session log when adding a new trial (`t02`, `t03`, ...) |

> **Note**: `ProjectRule.md` is auto-generated and placed in `0_Meta/` during initialization. Edit Sections 1–8 of `0_Meta/ProjectRule.md` directly to fit your project.

### 1. Read the Rules First

Before starting research, read the two governance documents generated in `.elf/managed/`:

| Document | Purpose |
|----------|---------|
| `EliRule.md` | Folder structure spec, naming convention, operational rules (Section 1-2), AI communication rules (Section 3) |
| `LogConvention.md` | Session log format, file naming, archiving workflow, cross-reference rules |

### 2. Start a New Session

Create a log file in `2_Log/`:

```markdown
# S002: Wavelength Optimization Simulation

> **Created**: 2026-04-01\
> **Modified**: 2026-04-01\
> **Status**: ★ Active\
> **Goal**: Compare SNR across 735/810/940 nm wavelengths via Monte Carlo simulation\
> **Related**: P001_wavelength_optimization.md\
> **Handoff**: -
```

- Session numbers (`S001`, `S002`, ...) increment sequentially — no gaps, no duplicates.
- File naming: `S###_log.md` (e.g., `S002_log.md`).
- `elf session new "<title>"` creates the log from the template and registers it automatically (auto-increments `S###`); or copy `.elf/managed/templates/sessionTemplate.md` manually.

### 3. Develop Tasks (t01, t02, ...)

Within each session, break work into sequential tasks:

```markdown
## t01: MCX Forward Simulation — 3-wavelength sweep

### Goal
- Run MCX simulation for λ = {735, 810, 940} nm at SDS = 20 mm

### Conditions
- Tissue model: 3-layer (epidermis/dermis/subcutaneous)
- Photon count: 1e8 per wavelength
- fmel = 0.10 (Fitzpatrick III)

### Results
- 940 nm shows highest sensitivity (ΔR/Δh = 0.12 mm⁻¹)
- 735 nm has lowest noise floor but saturates at h > 15 mm

![S002_t01: SNR comparison](../6_Exp/64_Viz/S002/S002_t01_SNR_comparison.png)

### Lesson
- 810 nm is the best compromise between sensitivity and dynamic range

### Generated Files

| Type | File |
|------|------|
| Script | `61_Sim/Scripts/S002_t01_wavelength_sweep.m` |
| Output | `61_Sim/Data/S002/S002_t01_results.mat` |
| Figure | `64_Viz/S002/S002_t01_SNR_comparison.png` |
```

- Tasks build on each other: `t01` → `t02` → `t03`.
- Each task has: **Goal**, **Conditions**, **Results**, **Lesson**, **Generated Files**.
- Embed figures inline in the results section — never list file paths without visual embedding.

### 4. Complete a Session

When a session is done:

1. **Update Status**: Change `★ Active` to `Complete` in the log header.
2. **Summarize to Wiki**: Add a 1-2 line summary to `2_Log/Wiki/` knowledge documents with a link to the archived log.
3. **Update Session Registry**: Add a row to `2_Log/Wiki/Session_Registry.tsv`:
   ```
   S002	2026-04-01	Wavelength Optimization	Complete	810 nm optimal	Archive/S002_log.md
   ```
4. **Archive the log**: Move the log file to `2_Log/Archive/` (filename unchanged).
5. **Archive scripts** (if one-time): Move to `Scripts/Archive/`.

> `elf session close [S###]` automates steps 1, 3 (status), and 4 — including fixing the log's relative cross-references for its new depth. Steps 2 and 5 remain manual.

### 5. AI Agent Handoff (Optional)

If using AI agents, record performed actions, modified files, and next steps in the session log's `Handoff` field upon task completion.

## License

This project applies a Dual License policy because the nature of "executable code" and "data structure specification (Protocol)" differs.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Applies to:** All source code (`.m`, `.py`, etc.) within the `5_SW/` and `6_Exp/*/Scripts/` folders.
  * **Condition:** If template core scripts are modified and improved for redistribution, those modifications must be released as open source. However, unique algorithms or raw data added by the user within the project may remain private (commercialized).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Applies to:** `README.md`, the governance documents (`.elf/managed/`·`0_Meta/`), the Session-Trial folder hierarchy, Base-Delta metadata logging rules, and the overall research methodology.
  * **Condition:** Anyone may freely adopt and adapt this structure and recording methodology, but when publishing derived templates or related research outputs, the original author Eli (projectschnee@gmail.com) and the source repository must be credited.
