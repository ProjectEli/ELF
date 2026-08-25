[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF)

**Base-Delta: every change is an experiment — and the agent writes it down.**

Now you keep not just the result, but how it was made.\
In ELF, every change becomes a trial — hypothesis, run, lesson — written by the agent on the spot.\
Every result keeps its process, and the next step starts from the last lesson.

## Sessions and trials

| Unit | What it is | What it holds |
|---|---|---|
| **trial** `t##` | one change = one experiment | before the run: goal, conditions, hypothesis, prediction / after: observation, interpretation, lesson, files |
| **session** `S###` | one goal, with trials accumulating under it | header (goal, related, Handoff) + t01, t02, … → on close, a one-line conclusion goes to the registry |

```text
S012  goal: SNR by wavelength
 ├─ t01  baseline: 3-wavelength sweep        hypothesis → run → 810 nm best
 ├─ t02  delta: photons 1e8 → 1e9            hypothesis → run → unchanged
 └─ t03  delta: skin model 3 → 5 layers      hypothesis → run → 735 nm overtakes
 close → registry: "810 nm best — revisit 735 nm with the 5-layer model"
```

- Every record is standardized by number — `S012 t02`. The log, script, data, and figure share it, so later you cite and find any trial with one number: in a paper, a plan, or a prompt.
- You decide the next trial. The agent writes down the next change with its hypothesis and prediction from the previous result; you review it and decide whether it runs.

| Term | Meaning |
|---|---|
| `S###` · `t##` | Session and trial numbers. The log, script, data, and figure share them |
| delta | What changed from the previous trial, and why |
| Handoff | One line of current state in the session header — "valid conclusion; pending; references", rewritten as a whole |
| Registry | `2_Log/Wiki/Session_Registry.tsv` — one row per session (status, one-line conclusion) |
| Archive | Where closed logs go (`2_Log/Archive/`). Excluded from the agent's autonomous exploration |
| validate | `elf validate` — consistency check of logs, registry, numbering, cross-refs, figure embeds, and trial sections |

## Principles

| Principle | One line |
|---|---|
| **Base-Delta** | Declare the baseline once; after that, record only what changed. One change is a trial, one goal is a session — trials accumulate into sessions, sessions into the project's record. |
| **Hypothesis first** | Write the hypothesis and prediction, stop, then run. That is what makes one change one experiment. |
| **People judge, the agent writes, the tool checks** | One format for both, so each picks up where the other left off — and `elf validate` catches what is missing. |
| **Everything stays local** | Markdown and folders, nothing else — no server, no account. Manage it with git, read it without ELF. |

## In practice

| Scene | What you say | What gets recorded |
|---|---|---|
| Start | "Create a new session. Goal: SNR by wavelength." | `elf session new` → header + registry row. t01 hypothesis and prediction, then a **stop** — you confirm, then it runs |
| Change | "Raise the photon count to 1e9 and run again." | t02: base = t01, delta + reason → hypothesis → run → the figure lands in Observation → hit or miss |
| Resume | (a month later, or after a context reset) "Where were we on the SNR comparison?" | Continues from the Handoff and the registry conclusion — from the record, not from memory |
| Trace | "Fig 3 says 810 nm — what is that based on?" | The S012 t01→t03 chain with script and data paths, as recorded |
| Paper | "Outline the manuscript from the conclusions so far." | Registry conclusions → `7_Paper/72_Drafts` — accumulation becomes the paper |

## Walkthrough

Your first session with an AI coding agent — any agent that reads `AGENTS.md`. The S012 above is what a session looks like after it has accumulated; here we start from S001. There are two starting points — **A** if you have no data yet, **B** if you already do. They are the same study at two moments: A simulates SNR by wavelength, B analyzes data measured with a prototype optical sensor. Using ELF without an agent is covered in the last line.

**1. Install and create a project**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
```

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
```

A single binary is installed to `~/.elf/bin` (no Node or Python). Open a new shell and check with `elf --version`.

```bash
elf init MyProject --preset experimental --lang en-US   # 6_Exp + 7_Paper. With no name, initializes the current folder
cd MyProject
```

`[elf] created MyProject (ELF v2.20.0, preset: experimental, lang: en-US)` — along with the folder structure you get `AGENTS.md` (the shared entry digest for agents) and `2_Log/S001_log.md` (the first session stub). The agent starts by reading `AGENTS.md`, so nothing else needs to be set up. Claude Code files (`CLAUDE.md` pointer, hook settings) are created too; other agents can ignore them. Project-specific rules go in `0_Meta/ProjectRule.md`. `--lang en-US` sets the agent's response language and deploys English companions of the rule documents (the operative rules are Korean; the English copies are for reading, and log section headings are bilingual, e.g. `### 목표 (Goal)`).

**2-A. From scratch — start from an idea sketch**

| Step | What you say | What gets recorded | What you do |
|---|---|---|---|
| Idea | "I need to sort out my thinking on the wavelength choice for a wearable optical sensor. Current view: taking skin penetration and hemoglobin absorption together, around 810 nm looks favorable. Note the evidence and the counter-cases." | `1_Concept/13_Ideas/wavelength_choice.md` (not in the log) | Judge the direction |
| Plan | "Write a concrete plan to test this as a plan document: compare SNR by wavelength in simulation first, then confirm with measurements from the prototype sensor." | `1_Concept/12_Planning/P001_wavelength_optimization.md` | Approve the steps |
| Session | "Start with step 1 of P001. S001 goal: SNR by wavelength — 735/810/940 nm, Monte Carlo simulation." | S001 header `관련: P001` + registry row | Confirm the goal |
| t01 | "t01: 3-wavelength sweep. Baseline conditions as written in P001 — 3-layer skin model, 1e8 photons." | t01 pre-run sections → **stop** → run → observation, interpretation, lesson · `61_Sim/Scripts/S001_t01_*.m` · `61_Sim/Data/S001/` | Review hypothesis and prediction → say run |

Reference papers go in `1_Concept/11_Literature/` — the rules include a source-reliability rule and a journal domain list.

**2-B. With existing data — start from analysis**

| Step | Command / what you say | What gets recorded | What you do |
|---|---|---|---|
| Data in | Copy the measured raw CSV into `6_Exp/62_Empirical/Raw/` | Nothing yet — originals kept read-only and out of git; recording starts at t01 | Check the originals |
| Session | "I have CSVs of 3-wavelength reflectance measured with the prototype optical sensor. S001 goal: SNR by wavelength from this data." | S001 header + registry row | Confirm the goal |
| t01 cleaning | "t01: data overview and cleaning. Drop the saturated stretches and the motion-noise stretches." | `62_Empirical/Processed/S001/` · `63_Analysis/Scripts/S001_t01_*.m` (range and filters in Conditions) | Approve the cleaning criteria |
| t02 analysis | "t02: SNR by wavelength on the cleaned data, with the same definition as the simulation." | t02 pre-run sections → **stop** → figure in `64_Viz/S001/` + data array `.mat`/`.csv` → interpretation, lesson | Review hypothesis and prediction → say run |

**3. Shared — next change → close**

| Step | Command / what you say | What gets recorded | What you do |
|---|---|---|---|
| Next change | "Raise the photon count to 1e9 and run again." (A) → `elf trial new "photons 1e9"` | `[elf] appended t02 to 2_Log/S001_log.md (S001)` — t02: base = t01, delta + reason → stop → run → record. Repeat from the last result | Decide whether it runs |
| Close | `elf validate` → `elf session close` | `[elf] ok: registry, logs, numbering, and cross-refs are consistent` → `[elf] closed S001 → 2_Log/Archive/S001_log.md (Status: Complete, registry updated)` | Check the one-line conclusion in the registry |

`elf validate` reports missing figures, non-canonical sections, and registry mismatches as warnings or issues; `elf session close` runs validate once more, moves the log to `Archive/`, and fixes its relative cross-refs. If the Handoff still lists pending items, it warns.

A trial the agent leaves behind looks like this (A's t02):

```markdown
## t02: photons 1e9

### 목표 (Goal)
- Raise t01's photon count 1e8 → 1e9 to check that the 810 nm lead is not sampling noise

### 조건 (Conditions)
- base = t01, delta = photons 1e8 → 1e9 (reason: 810 and 940 nm confidence intervals overlap in t01)

### 가설 (Hypothesis)
- Less noise narrows the intervals; the ranking holds

### 예상 (Prediction)
- 810 nm keeps the lead; interval width ≤ 1/3
- Output: `S001_t02_SNR_ci.png`

### 관찰 (Observation)
- 810 nm keeps the lead; interval width 0.28×
- ![S001_t02: SNR by wavelength, 1e9 photons](../6_Exp/64_Viz/S001/S001_t02_SNR_ci.png)

### 해석 (Interpretation)
- 가설 적중 여부: hit
- The overlap at 1e8 was sampling noise

### 교훈 (Lessons)
- Ranking calls need 1e9 photons or more

### 생성 파일 (Files)

| Type | File |
|------|------|
| Script | `6_Exp/61_Sim/Scripts/S001_t02_sweep.m` |
| Output | `6_Exp/61_Sim/Data/S001/S001_t02_results.mat` |
| Figure | `6_Exp/64_Viz/S001/S001_t02_SNR_ci.png` |
```

Without an agent: paste `.elf/managed/templates/trialTemplate.md` into the log and write it yourself; `elf trial new`, `elf validate`, and `elf session close` work the same.

## How it works

| Mechanism | Role |
|---|---|
| `AGENTS.md` | The agent's entry digest — record in the same turn, canon over precedent (no imitating drifted logs), re-read the Handoff after a context rebuild. The canonical rules live in `.elf/managed/` (EliRule, LogConvention, AI_PARA_Framework); project rules in `0_Meta/ProjectRule.md` |
| Stop before the run | Hypothesis and prediction are fixed before execution (LogConvention §5.1). The agent honors this stop even in autonomous mode |
| `elf validate` | Checks registry ↔ logs, numbering, cross-refs, missing figure embeds, and trial section structure. `session close` runs it automatically |
| `elf autoread` | `elf autoread` (no arguments) prints a digest — rule summary, active session Handoff, validate counts — for any agent to re-read after a context rebuild. In Claude Code, hooks (`.claude/settings.json`) inject it automatically on the first prompt after a compaction or restart (on by default; `autoread_fulltext` adds full rule texts) |
| Handoff · Registry | Handoff = one line "valid conclusion; pending; references", rewritten as a whole; registry key finding = the session's one-line conclusion — where you resume from |
| One writer | One session log has one writer (an agent or a person). Parallel work means one session per agent, with relationships noted in the header `관련:` (related) field |
| Archive firewall | `Archive/` is off-limits to autonomous exploration — an `AGENTS.md` rule (open only when you name a path). In Claude Code, `.claudeignore` also blocks it from search (AI_PARA_Framework) |
| Overlays | Customize vocabulary and search domains in `0_Meta/<name>.project.md` (effective rules = base ⊕ overlay). `elf update` never touches it |

## CLI

> Full reference (every flag, exit codes, file ownership): **[elf-cli/CLI.md](elf-cli/CLI.md)**

| Command | Role |
|---|---|
| `elf init [name] [--preset …] [--modules …] [--lang …]` | Create a project. With no name, in place in the current folder. Presets `full`/`experimental`/`software`/`minimal` (experimental: `general`, `qa`) |
| `elf session new <title>` / `close [S###]` / `fix-headers` | Create a session · close it (validate → move to Archive → registry and cross-ref fix-up) · repair header line breaks |
| `elf trial new [title]` | Append the canonical trial stub to the active log (auto-numbered `t##`) |
| `elf validate [--check] [--strict]` | Consistency check (read-only). `--check` exits 4 on issues; `--strict` promotes missing embeds and structure warnings to issues |
| `elf gallery` | Build the figure index `_gallery.md` from `6_Exp/64_Viz/` |
| `elf autoread [enable\|disable\|status]` | Rule re-injection after a context rebuild. With no argument, prints the digest (any agent); automatic hook injection is Claude Code only (on by default) |
| `elf update [--dry-run] [--force]` | Update managed files to the installed CLI version — research data, logs, and settings are never touched |
| `elf status [--check]` | Diagnose managed-file state (read-only). `--check` exits 4 on findings |
| `elf doctor` | Environment and project health check (read-only) |
| `elf tsa <sub>` | Optional: per-commit file-hash manifest + RFC 3161 timestamp — proof of what existed when. Off by default; the only thing that leaves your machine is a 32-byte manifest digest |
| `elf self-update` | Update the `elf` binary |

Operations:

```bash
elf self-update          # apply a new version ① update the CLI
elf status               # ② see what would change
elf update --dry-run     # ③ preview without writing
elf update               # ④ managed files only — on "edited: … — kept; new version at ….elf-new" your edit stays; diff and merge (or --force to take the canonical version)
elf status --check       # team/CI gate — exit 4 on findings (elf validate --check works the same way)
```

In `.gitignore`, ELF manages only the marker block (`# >>> ELF managed >>>` … `# <<< ELF managed <<<`); everything else is preserved.

## Project layout

- File names carry only the number — `S001_t02_sweep.m` · `Data/S001/S001_t02_results.mat` · `64_Viz/S001/S001_t02_SNR_ci.png`. The conditions live in the log.
- Cross-refs in logs are relative paths — plans `../1_Concept/12_Planning/P00x.md`, data `../6_Exp/61_Sim/Data/S###/`, figures `../6_Exp/64_Viz/S###/`. `session close` fixes them for the Archive depth.
- Plans go in `1_Concept/12_Planning/P###_title.md`, ideas in `13_Ideas/`, and logs hold facts only. Analysis code lives in `6_Exp/63_Analysis/Scripts/` and `61_Sim/Scripts/` (`.m` cell mode, `%%`).

Full folder structure:

```text
Project_Root/
│
├── AGENTS.md                        # Agent entry digest (shared by all agents, ELF-managed)
├── CLAUDE.md · .claude/settings.json · .claudeignore   # Claude Code only (pointer · autoread hooks · Archive exclusion) — other agents ignore them
├── README.md · LICENSE · .gitignore · .editorconfig · .gitattributes
│
│  ─── Core ───────────────────────────────
│
├── .elf/                            # ELF control plane (version·config·manifest — do not edit)
│   └── managed/                     # Managed rule payload: EliRule·LogConvention·AI_PARA_Framework
│       └── templates/               #   ·LLMcliche·highIFjournals + session/trial stubs (Archive/ = previous templates)
├── 0_Meta/                          # Project governance — yours (`elf update` never writes here)
│   ├── ProjectRule.md               # Project-specific rules and objectives
│   ├── <name>.project.md            # Data overlays (effective rules = base ⊕ overlay)
│   └── tsa/                         # (optional) elf tsa manifests and timestamps
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

The purpose and operating rules of each folder are in `.elf/managed/EliRule.md`.

## License

This project applies a Dual License policy because the nature of "executable code" and "data structure specification (Protocol)" differs.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Applies to:** All source code (`.m`, `.py`, etc.) within the `5_SW/` and `6_Exp/*/Scripts/` folders.
  * **Condition:** If template core scripts are modified and improved for redistribution, those modifications must be released as open source. However, unique algorithms or raw data added by the user within the project may remain private (commercialized).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Applies to:** `README.md`, the governance documents (`.elf/managed/`·`0_Meta/`), the Session-Trial folder hierarchy, Base-Delta metadata logging rules, and the overall research methodology.
  * **Condition:** Anyone may freely adopt and adapt this structure and recording methodology, but when publishing derived templates or related research outputs, the original author Eli (projectschnee@gmail.com) and the source repository must be credited.
