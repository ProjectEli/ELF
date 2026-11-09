[English](layout.md) | [한국어](layout.ko.md)

# Project layout

- File names carry only the number: `S001_t02_sweep.m`, `Data/S001/S001_t02_results.mat`, `64_Viz/S001/S001_t02_SNR_ci.png`. The conditions are recorded in the log.
- Cross-refs in logs are relative paths from `2_Log/S/` (or `2_Log/G/`): plans `../../1_Concept/12_Planning/P00x.md`, data `../../6_Exp/61_Sim/Data/S###/`, figures `../../6_Exp/64_Viz/S###/`. Logs never move, so these paths stay valid.
- Plans go in `1_Concept/12_Planning/P###_title.md`, ideas in `13_Ideas/`, and logs hold facts only. Analysis code is kept in `6_Exp/63_Analysis/Scripts/` and `61_Sim/Scripts/` (`.m` cell mode, `%%`).

Full folder structure:

```text
Project_Root/
│
├── AGENTS.md                        # Agent entry digest (shared by all agents, ELF-managed)
├── CLAUDE.md, .claude/settings.json   # Claude Code only (pointer, autoread hooks). Other agents ignore them
├── README.md, LICENSE, .gitignore, .editorconfig, .gitattributes
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF control area (config, project manifest). Do not edit
│   ├── managed/                     # Managed rule payload: EliRule, LogConvention, AI_PARA_Framework
│   │   └── templates/               #   LLMcliche, highIFjournals + session/trial templates
│   ├── deprecated/                  # Compressed originals of deprecated content (elf deprecate, tracked)
│   └── runtime/                     # Temporary tool state (not tracked)
├── 0_Meta/                          # Project governance, owned by you (`elf update` never writes here)
│   ├── ProjectRule.md               # Project-specific rules and objectives
│   ├── <name>.project.md            # Data overlays (effective rules = base ⊕ overlay)
│   └── tsa/                         # (optional) elf tsa manifests and timestamps
│
├── 1_Concept/                       # Research planning, literature, ideas
│   ├── 11_Literature/               # Paper PDFs, bibliographic info, base formulas
│   ├── 12_Planning/                 # Research plans, roadmaps (multi-session)
│   │   ├── Wiki/                    # Summarized planning conclusions & key rules
│   │   └── Archive/                 # Closed planning documents
│   └── 13_Ideas/                    # Small snippets / naive early ideas (flat)
│
├── 2_Log/                           # Session logs (status in the header; logs never move)
│   ├── S/                         # Research sessions S###_log.md + Registry.md (generated)
│   ├── G/                         # Governance sessions G###_log.md + Registry.md (generated)
│   └── Wiki/                      # Topic pages summarizing findings
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
│   │   │   └── Archive/          # One-off scripts kept after use
│   │   └── Data/                    # Simulation results (Data/S###/)
│   ├── 62_Empirical/                # Empirical data
│   │   ├── Raw/                     # Raw sensor data (Read-Only, excluded from Git)
│   │   └── Processed/               # Primary processed data
│   ├── 63_Analysis/                 # Integrated analysis
│   │   └── Scripts/                 # Comparison/validation post-processing code
│   │       └── Archive/           # One-off scripts kept after use
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

The purpose and operating rules of each folder are in `.elf/managed/EliRule.md`.

---

Back to [README](../README.md)
