[English](README.md) | [한국어](README.ko.md) | [日本語](docs/i18n/README.ja.md) | [中文简体](docs/i18n/README.zh-CN.md) | [中文繁體](docs/i18n/README.zh-TW.md) | [Français](docs/i18n/README.fr.md) | [Deutsch](docs/i18n/README.de.md) | [Español](docs/i18n/README.es.md) | [Italiano](docs/i18n/README.it.md) | [Português](docs/i18n/README.pt-BR.md) | [Русский](docs/i18n/README.ru.md) | [العربية](docs/i18n/README.ar.md) | [हिन्दी](docs/i18n/README.hi.md) | [Türkçe](docs/i18n/README.tr.md) | [Tiếng Việt](docs/i18n/README.vi.md) | [ภาษาไทย](docs/i18n/README.th.md) | [Nederlands](docs/i18n/README.nl.md) | [Polski](docs/i18n/README.pl.md) | [Bahasa Indonesia](docs/i18n/README.id.md)

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
│   ├── EliRule.md                   # Folder structure and operational guide
│   ├── LogConvention.md             # Logging standard rules
│   ├── AI_PARA_Framework.md         # AI context management & archiving rules
│   └── AI_Sync.md                   # AI agent handoff log
│
├── 1_Concept/                       # Research planning, literature, ideas
│   ├── 11_Ideas/                    # Rough sketches, hypothesis proposals
│   ├── 12_Literature/               # Paper PDFs, bibliographic info, base formulas
│   └── 13_Planning/                 # Research roadmaps, figure composition storyboards
│
├── 2_HW/                            # Hardware design
│   ├── 21_Component/                # Individual component specs, unit device design
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Integrated device design, housing, 3D models
│   └── 23_Elec/                     # PCB schematics, Gerber, BOM, Datasheets
│
├── 3_Fab/                           # Fabrication and processing
│   ├── 31_Recipes/                  # Process condition documentation
│   └── 32_Eval/                     # Per-module single characteristic evaluation
│
├── 4_SW/                            # Software & firmware
│   ├── 41_FW/                       # MCU/embedded firmware
│   ├── 42_DAQ/                      # PC/mobile data acquisition systems
│   └── 43_Libs/                     # Reusable shared libraries
│
├── 5_Exp/                           # Experiments: simulation + empirical + analysis
│   ├── 51_Sim/                      # Simulation
│   │   ├── Scripts/                 # Simulation code (S###_sim.m)
│   │   └── Data/                    # Simulation results (Data/S###/)
│   ├── 52_Empirical/                # Empirical data
│   │   ├── Raw/                     # Raw sensor data (Read-Only, excluded from Git)
│   │   └── Processed/               # Primary processed data
│   ├── 53_Analysis/                 # Integrated analysis
│   │   ├── Scripts/                 # Comparison/validation post-processing code
│   │   └── Logs/                    # Session logs (S###_log.md)
│   └── 54_Viz/                      # Visualization outputs (auto-generated figures)
│
└── 6_Paper/                         # Papers & presentations
    ├── 61_Figs/                     # Figures for papers
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manuscripts (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Presentation materials (PPT, posters)
```

> For detailed usage and operational rules for each folder, refer to `0_Meta/EliRule.md`.

## Data Logging Pipeline Specification

### 1. File Naming Convention (Session-Trial Naming)

* Listing experimental conditions or variable information in file names is **strictly prohibited**.
* **Format:** `[SessionID]_[TrialID].[extension]` (e.g., `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Logging (Hybrid Logging)

* **Running Log (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * A narrative markdown file that records immediate hypothesis-test-lesson cycles in text.
  * Written per trial (`t1`, `t2`...) in a stream-of-consciousness style, recording only the **intentionally changed variables (Delta)** and observed results.
  * Format and detailed rules: refer to `0_Meta/LogConvention.md`.

### 3. Planning Document Rules

* Research roadmaps, figure compositions, experimental strategies, etc. are managed separately in `1_Concept/13_Planning/`.
* **Format:** `P###_title.md` (e.g., `P001_wavelength_optimization.md`)
* When referencing Planning from a log: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Post-Processing Analysis Specification (Cell Mode Scripting)

* Analysis code must be located in `5_Exp/53_Analysis/Scripts/` or `5_Exp/51_Sim/Scripts/` and must not be mixed inside data folders.
* Pure `.m` files are used instead of `.mlx` to prevent vendor lock-in.
* Code is executed section by section using `%%` (Cell Mode), and derived insights are reflected in the running log.
* Analysis outputs (figures, mat files) are saved in `5_Exp/54_Viz/` or `5_Exp/52_Empirical/Processed/S###/` within per-session folders.

### 5. Cross-Reference Rules

Cross-reference formats are unified to ensure traceability between project documents.

| From → To | Format |
|-----------|--------|
| Logs → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ see 5_Exp/51_Sim/Data/S###/` |
| Logs → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

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

To create a new project with the ELF v2 structure, run `0_Meta/ELF_generator.ps1`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.ps1
```

Enter a project name and the 0–6 folder structure, meta documents, `.gitignore`, and Git initialization will all be completed automatically.

## License

This project applies a Dual License policy because the nature of "executable code" and "data structure specification (Protocol)" differs.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Applies to:** All source code (`.m`, `.py`, etc.) within the `4_SW/` and `5_Exp/*/Scripts/` folders.
  * **Condition:** If template core scripts are modified and improved for redistribution, those modifications must be released as open source. However, unique algorithms or raw data added by the user within the project may remain private (commercialized).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Applies to:** `README.md`, `0_Meta/` documents, the Session-Trial folder hierarchy, Base-Delta metadata logging rules, and the overall research methodology.
  * **Condition:** Anyone may freely adopt and adapt this structure and recording methodology, but when publishing derived templates or related research outputs, the original author Eli (projectschnee@gmail.com) and the source repository must be credited.
