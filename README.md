[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF): Base-Delta Protocol for Agile R&D

A hardware-software-experimental data integrated logging standard (Protocol) designed to support fast feedback loops (Agile) during device development and R&D validation phases. Guarantees complete data traceability while minimizing researcher documentation fatigue.

## Core Philosophy

* **Single Source of Truth:** Hardware design, analysis code, and raw data are organically connected within a single project.
* **Base-Delta Logging:** Not every variable is recorded. A Baseline is declared, and only changed variables (Deltas) are logged lightly to prevent research delays.
* **Systematic Enforcement:** Bypasses file name length limitations (Windows 260-character limit) and guarantees reproducibility through code.
* **AI Governance:** Ensures AI agent work continuity via the `0_Meta/AI_Sync.md` handoff log, and enforces a unified logging standard for both humans and AI through `0_Meta/LogConvention.md`.

## Project Directory Structure

This project treats the folder hierarchy itself as a communication standard.

```text
Project_Root/
├── 0_Meta/                          # Project governance & rules
│   ├── ProjectRule.md               # Project-specific rules and objectives
│   ├── EliRule.md                   # Folder structure and operational guide
│   ├── LogConvention.md             # Logging standard rules
│   ├── AI_PARA_Framework.md         # AI context management & archiving rules
│   └── AI_Sync.md                   # AI agent handoff log
│
├── 1_Concept/                       # Research planning, literature, ideas
│   ├── 11_Ideas/                    # Rough sketches, hypothesis proposals
│   ├── 12_Literature/               # Paper PDFs, bibliographic info, base formulas
│   └── 13_Planning/                 # Research roadmaps, figure composition storyboards
│       └── 2_Wiki/                  # Distilled planning conclusions & key rules
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
│   │   │   └── 9_Archive/          # Retired scripts
│   │   └── Data/                    # Simulation results (Data/S###/)
│   ├── 62_Empirical/                # Empirical data
│   │   ├── Raw/                     # Raw sensor data (Read-Only, excluded from Git)
│   │   └── Processed/               # Primary processed data
│   ├── 63_Analysis/                 # Integrated analysis
│   │   └── Scripts/                 # Comparison/validation post-processing code
│   │       └── 9_Archive/           # Retired scripts
│   └── 64_Viz/                      # Visualization outputs (auto-generated figures)
│
├── 7_Paper/                         # Papers & presentations
│   ├── 71_Figs/                     # Figures for papers
│   │   ├── Raw/
│   │   ├── Processed/
│   │   └── Final/
│   ├── 72_Drafts/                   # Manuscripts (Word, LaTeX)
│   │   └── 9_Archive/               # Previous versions
│   └── 73_Presentations/            # Presentation materials (PPT, posters)
│
└── 2_Log/                           # Session logs (S###_log.md)
    ├── 2_Wiki/                      # Distilled findings & session registry
    └── 9_Archive/                   # Completed session logs
```

> For detailed usage and operational rules for each folder, refer to `0_Meta/EliRule.md`.

## Data Logging Pipeline Specification

### 1. File Naming Convention (Session-Trial Naming)

* Listing experimental conditions or variable information in file names is **strictly prohibited**.
* **Format:** `[SessionID]_[TrialID].[extension]` (e.g., `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Logging (Hybrid Logging)

* **Running Log (`2_Log/S###_log.md`):**
  * A narrative markdown file that records immediate hypothesis-test-lesson cycles in text.
  * Written per trial (`t1`, `t2`...) in a stream-of-consciousness style, recording only the **intentionally changed variables (Delta)** and observed results.
  * Format and detailed rules: refer to `0_Meta/LogConvention.md`.

### 3. Planning Document Rules

* Research roadmaps, figure compositions, experimental strategies, etc. are managed separately in `1_Concept/13_Planning/`.
* **Format:** `P###_title.md` (e.g., `P001_wavelength_optimization.md`)
* When referencing Planning from a log: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Post-Processing Analysis Specification (Cell Mode Scripting)

* Analysis code must be located in `6_Exp/63_Analysis/Scripts/` or `6_Exp/61_Sim/Scripts/` and must not be mixed inside data folders.
* Pure `.m` files are used instead of `.mlx` to prevent vendor lock-in.
* Code is executed section by section using `%%` (Cell Mode), and derived insights are reflected in the running log.
* Analysis outputs (figures, mat files) are saved in `6_Exp/64_Viz/` or `6_Exp/62_Empirical/Processed/S###/` within per-session folders.

### 5. Cross-Reference Rules

Cross-reference formats are unified to ensure traceability between project documents.

| From → To | Format |
|-----------|--------|
| Logs → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ see 6_Exp/61_Sim/Data/S###/` |
| Logs → Script | `→ see 6_Exp/63_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← tracked in 2_Log/S###_log.md` |

## AI Governance

When AI agents (Claude, Gemini, etc.) participate in the project, the following rules apply:

1. **Context Acquisition:** Before starting work, read `0_Meta/AI_Sync.md` to confirm the state of previous work.
2. **Unified Standard Compliance:** Follow the logging rules in `0_Meta/LogConvention.md` in the same way as a human researcher.
3. **Handoff Recording:** Upon task completion, record performed actions, created/modified files, and Next Steps in `0_Meta/AI_Sync.md`. Write in reverse chronological order with the most recent entry at the top.
4. **Idea Separation:** Hypotheses and ideas generated by AI are stored separately in `1_Concept/11_Ideas/`, not in logs.
5. **PARA-Based Context Management:** Use the `9_Archive/` folder and `.claudeignore` to prevent AI context contamination. For detailed rules, refer to `0_Meta/AI_PARA_Framework.md`.
6. **Communication Rules:** Maintain an objective and dry writing style. No analogies or metaphors. Deliver conclusions clearly and directly. No exaggeration or emotional modifiers. For detailed rules, refer to section 3 of `0_Meta/EliRule.md`.
7. **Data Reusability:** When generating any Plot/Graph, save the original Data Array alongside as `.mat`/`.csv`. For detailed rules, refer to section 2.6 of `0_Meta/EliRule.md`.

## Quick Start

**Windows (PowerShell 5.1 or later):**

```powershell
cd C:\desired\parent\directory
powershell -ExecutionPolicy Bypass -File "C:\path\to\ELF\0_Meta\ELF_generator.ps1"
```

**Linux / macOS (Bash):**

```bash
cd /desired/parent/directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

Enter a project name and the 0–6 folder structure, meta documents, and `.gitkeep` will be created automatically. Git initialization is optional and prompted only when Git is available.

## Usage

### 0. Templates

The `templates/` folder provides ready-to-use stubs:

| File | When to use |
|------|-------------|
| `sessionTemplate.md` | Copy to `2_Log/` when starting a new session (rename to `S###_log.md`) |
| `trialTemplate.md` | Paste into an active session log when adding a new trial (`t02`, `t03`, ...) |

> **Note**: `ProjectRule.md` is auto-generated and placed in `0_Meta/` during initialization. Edit Sections 1–8 of `0_Meta/ProjectRule.md` directly to fit your project.

### 1. Read the Rules First

Before starting research, read the two governance documents generated in `0_Meta/`:

| Document | Purpose |
|----------|---------|
| `EliRule.md` | Folder structure spec, naming convention, operational rules (Section 1-2), AI communication rules (Section 3) |
| `LogConvention.md` | Session log format, file naming, archiving workflow, cross-reference rules |

### 2. Start a New Session

Create a log file in `2_Log/`:

```markdown
# S002: Wavelength Optimization Simulation

> **Created**: 2026-04-01
> **Modified**: 2026-04-01
> **Status**: ★ Active
> **Goal**: Compare SNR across 735/810/940 nm wavelengths via Monte Carlo simulation
> **Related**: P001_wavelength_optimization.md
```

- Session numbers (`S001`, `S002`, ...) increment sequentially — no gaps, no duplicates.
- File naming: `S002_WavelengthOpt.md` (session number + short descriptor).
- The `S001_log.md` template is auto-generated with the correct format.

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
2. **Summarize to Wiki**: Add a 1-2 line summary to `2_Log/2_Wiki/` knowledge documents with a link to the archived log.
3. **Update Session Registry**: Add a row to `2_Log/2_Wiki/Session_Registry.tsv`:
   ```
   S002	2026-04-01	Wavelength Optimization	Complete	810 nm optimal	9_Archive/S002_WavelengthOpt.md
   ```
4. **Archive the log**: Move the log file to `2_Log/9_Archive/`.
5. **Archive scripts** (if one-time): Move to `Scripts/9_Archive/`.

### 5. AI Agent Handoff (Optional)

If using AI agents, update `0_Meta/AI_Sync.md` upon task completion with: performed actions, modified files, and next steps. See `LogConvention.md` Section 4 for format.

## License

This project applies a Dual License policy because the nature of "executable code" and "data structure specification (Protocol)" differs.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Applies to:** All source code (`.m`, `.py`, etc.) within the `5_SW/` and `6_Exp/*/Scripts/` folders.
  * **Condition:** If template core scripts are modified and improved for redistribution, those modifications must be released as open source. However, unique algorithms or raw data added by the user within the project may remain private (commercialized).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Applies to:** `README.md`, `0_Meta/` documents, the Session-Trial folder hierarchy, Base-Delta metadata logging rules, and the overall research methodology.
  * **Condition:** Anyone may freely adopt and adapt this structure and recording methodology, but when publishing derived templates or related research outputs, the original author Eli (projectschnee@gmail.com) and the source repository must be credited.
