[English](README.md) | [한국어](README.ko.md)

# Eli's Lab Framework (ELF)

**Base-Delta: research automation that records only what changed**

Now you keep not just the result, but how it was made.\
In ELF, every change becomes a trial with a hypothesis, a run, and a lesson, and the agent records it on your behalf.\
Every result keeps its process, and for the next step you only manage what changes.

ELF consists of a folder structure for research projects, rules that AI agents follow, and a command-line tool, `elf`. It works with the original data kept on your own computer.

| What you can do | How |
|---|---|
| Record | Every change is written down as a trial: the hypothesis and prediction before the run, the observation and lesson after it |
| Resume | A month later, or in a new conversation, the agent reads the session's handoff and conclusion and continues from there |
| Trace | One number such as `S012 t02` finds the log, script, data, and figure together |
| Check | `elf check` finds missing figures, broken links, and registry mismatches |

## Sessions and trials

| Unit | What it is | What it holds |
|---|---|---|
| **trial** `t##` | one change = one experiment | phase 1 (before the run): goal, conditions, hypothesis, prediction / phase 2 (after): observation, interpretation, lesson, files |
| **session** `S###` | one goal, with trials accumulating under it | a header (status, goal, handoff, conclusion) and t01, t02, …; on close, the conclusion goes into the header and `Registry.md` |

```text
S012  goal: SNR by wavelength
 ├─ t01  baseline: 3-wavelength sweep        hypothesis → run → 810 nm best
 ├─ t02  delta: photons 1e8 → 1e9            hypothesis → run → unchanged
 └─ t03  delta: skin model 3 → 5 layers      hypothesis → run → 735 nm overtakes
 close → registry: "810 nm best; revisit 735 nm with the 5-layer model"
```

## Quickstart

ELF is used together with an AI agent such as Claude Code or Codex. To use it without an agent, see the end of this section.

1. Install the CLI.

   On Windows, run this in PowerShell:

   ```powershell
   powershell -ExecutionPolicy Bypass -c "irm https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.ps1 | iex"
   ```

   On Linux and macOS, run this in a terminal:

   ```bash
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ProjectEli/ELF/releases/latest/download/elf-cli-installer.sh | sh
   ```

   A single executable is installed to `~/.elf/bin`; Node and Python are not needed. Open a new terminal and check the install with `elf --version`.

2. Create a project.

   ```bash
   elf init MyProject --preset exp --lang en-US   # 6_Exp + 7_Paper. With no name, initializes the current folder
   cd MyProject
   ```

   This prints `[elf] created MyProject (ELF v3.5.1, preset: exp, lang: en-US)` and creates the folder structure, `AGENTS.md` (the rule summary the agent reads first), and `2_Log/S/` (where session logs go). The agent starts by reading `AGENTS.md`, so there is nothing else to set up. `--lang en-US` makes the agent answer in English and adds English copies of the rule documents for reading; the operative rules are Korean. Presets, languages, and the files created are covered in [Creating and updating projects](docs/projects.md).

3. Open the project folder in your agent and ask it to start a session.

   > Create a new session. Goal: SNR by wavelength.

   The agent runs `elf session new` with the goal, which creates `2_Log/S/S001_log.md` and updates `2_Log/S/Registry.md`, the session list.

4. Ask for the first trial.

   > t01: 3-wavelength sweep. 3-layer skin model, 1e8 photons.

   The agent writes phase 1 (goal, conditions, hypothesis, prediction) with `elf trial new`, then asks whether to run. If you ask for changes, it revises phase 1 with `elf trial write`. After you approve it, the agent runs the trial and writes phase 2 (observation, interpretation, lesson, files) with `elf trial write`.

5. Ask for the next change.

   > Raise the photon count to 1e9 and run again.

   The agent adds t02 with `elf trial new`. t02 records what changed from t01 and why, then goes through phase 1 and phase 2 like t01.

6. Close the session.

   > Close the session.

   The agent runs `elf check logs` to find missing figures or format errors, proposes the conclusion, and closes with `elf session close --from <file>` after you approve it. The log stays in `2_Log/S/`; its status becomes complete and `Registry.md` shows the conclusion.

Without an agent, run the same commands yourself: write the sections or the conclusion to a file and pass it with `--from`.

## Example trial

The t02 the agent writes in step 5:

```markdown
## t02: photons 1e9

### Goal
- Raise t01's photon count 1e8 → 1e9 to check that the 810 nm lead is not sampling noise

### Conditions
- base = t01, delta = photons 1e8 → 1e9 (reason: 810 and 940 nm confidence intervals overlap in t01)

### Hypothesis
- Less noise narrows the intervals; the ranking holds

### Prediction
- 810 nm keeps the lead; interval width ≤ 1/3
- Output: `S001_t02_SNR_ci.png`

### Observation
- 810 nm keeps the lead; interval width 0.28×
- ![S001_t02: SNR by wavelength, 1e9 photons](../../6_Exp/64_Viz/S001/S001_t02_SNR_ci.png)

### Interpretation
- Prediction match: match
- The overlap at 1e8 was sampling noise

### Lessons
- Ranking calls need 1e9 photons or more

### Files

| Type | File |
|------|------|
| Script | `6_Exp/61_Sim/Scripts/S001_t02_sweep.m` |
| Output | `6_Exp/61_Sim/Data/S001/S001_t02_results.mat` |
| Figure | `6_Exp/64_Viz/S001/S001_t02_SNR_ci.png` |
```

This is an English project (`--lang en-US`). A Korean project writes the same sections with Korean-first headings (`### 목표 (Goal)`).

## How it works

### Principles

| Principle | Description |
|---|---|
| **Base-Delta** | Write completed work once; after that, record only what changed. One change is a trial and one goal is a session; trials add up to sessions, and sessions add up to the project's record. |
| **Hypothesis first** | Each trial is split into phase 1, which writes down the hypothesis and prediction, and phase 2, which runs them. Once phase 1 is revised and approved, the trial moves on to phase 2. |
| **People judge, the agent writes, the tool checks** | Both write in the same format, so each can continue the other's work, and `elf check` finds what is missing. |
| **Everything stays local** | It works with the original data kept on your own computer. Manage it with git and read it without ELF. For security, the agent is not allowed to publish public links as a rule. |

- Every record gets a number such as `S012 t02`. The log, script, data, and figure share it, so you can cite and find any trial with one number in a paper, a plan, or a prompt.
- You decide the next trial. The agent writes down the next change with its hypothesis and prediction based on the previous result; you review it and decide whether it runs.

### Usage examples

| Situation | Your input | Action |
|---|---|---|
| Starting a session | "Create a new session. Goal: SNR by wavelength." | Creates the log and a registry row with `elf session new`, writes phase 1 of t01, and runs phase 2 after you approve |
| Changing a condition | "Raise the photon count to 1e9 and run again." | Records in t02 the change from t01, its reason, and the hypothesis. After the run, puts the figure in Observation and compares it with the prediction |
| Resuming work | (a month later, or in a new conversation) "Where were we on the SNR comparison?" | Reads the handoff and `Registry.md` and continues |
| Tracing evidence | "Fig 3 says 810 nm. What is that based on?" | Shows the S012 t01 to t03 records with their script and data paths |
| Writing a paper | "Outline the manuscript from the conclusions so far." | Collects the session conclusions into `7_Paper/72_Drafts` |

## If you already have an idea or existing work

The Quickstart opens a session in an empty project. If you already have an idea or existing work, bring it into the project first.

| Case | Your input | Action |
|---|---|---|
| You have an idea | "Sort out my thinking on the wavelength choice for a wearable optical sensor, and write a plan to test it as a plan document." | Writes the idea in `1_Concept/13_Ideas/` and the plan in `1_Concept/12_Planning/P001_*.md`. After you approve the plan, opens a session with `관련: P001` (related) in the header |
| You have existing work | (after copying the measured raw CSV into `6_Exp/62_Empirical/Raw/`) "Start a session comparing SNR by wavelength from this data. t01 is data cleaning." | Keeps the originals read-only, saves the cleaned data in `62_Empirical/Processed/` in t01, and analyzes from t02 |

Step-by-step examples are in [Starting examples](docs/examples.md).

## Terms

| Term | Meaning |
|---|---|
| `S###`, `G###`, `t##` | Research session, governance session, and trial numbers. The log, script, data, and figure share them |
| delta | What changed from the previous trial, and why |
| handoff | One line of current state in the session header, "valid conclusion; pending; references", rewritten as a whole |
| status | `ongoing`, `complete`, or `deprecated` in the log header. Logs stay in place, so the status shows whether a record is current |
| Registry | `Registry.md` in `2_Log/S/` and `2_Log/G/`, generated from the log headers: one row per session (status, conclusion or handoff) |
| Deprecated | Content you decided to withdraw. It is removed from the document and kept compressed in `.elf/deprecated/`; a marker line stays. Not a basis for current decisions |

## Components

The rule documents are in `.elf/managed/` in your project.

| Component | Role | Details |
|---|---|---|
| `AGENTS.md` | The rule summary the agent reads first. Project-specific rules go in `0_Meta/ProjectRule.md` | [Creating and updating projects](docs/projects.md) |
| `elf check` | Checks the install and settings, the managed files, and the logs. `elf session close` also checks the logs before closing | [Checks](docs/check.md) |
| `elf autoread` | Makes the agent read the rule summary and the handoff again after the conversation is compacted or restarted. In Claude Code, hooks run it automatically | [Reading the rules again](docs/autoread.md) |
| `elf deprecate` | Removes content you decided to withdraw and keeps the original compressed | [Deprecating content](docs/deprecate.md) |
| Approval before the run | The agent waits for your approval after phase 1, even in autonomous mode | `LogConvention.md` §5.1 |
| One writer | Each session log has one writer, an agent or a person. When several agents work in parallel, each opens its own session | `EliRule.md` |
| Project overlays | Vocabulary and search domains are customized in `0_Meta/<name>.project.md`. `elf update` never modifies it | [Creating and updating projects](docs/projects.md) |

## Reference

| Topic | Doc |
|---|---|
| Installing and updating the CLI | [docs/install.md](docs/install.md) |
| Creating and updating projects (presets, languages, file ownership, `.elf/`, log layout migration) | [docs/projects.md](docs/projects.md) |
| Step-by-step examples starting from an idea or existing work | [docs/examples.md](docs/examples.md) |
| Sessions and trials (`session new`, `session close`, `trial new`, `trial write`, `elf log`, `gallery`) | [docs/sessions.md](docs/sessions.md) |
| Checks (`elf check` scopes, issues and warnings) | [docs/check.md](docs/check.md) |
| Deprecating content (`elf deprecate`) | [docs/deprecate.md](docs/deprecate.md) |
| Rules after the conversation is rebuilt (`elf autoread`, Claude Code hooks) | [docs/autoread.md](docs/autoread.md) |
| Timestamping research records (optional, `elf tsa`) | [docs/tsa.md](docs/tsa.md) |
| Every folder in a project | [docs/layout.md](docs/layout.md) |
| Command summary, exit codes | [docs/reference.md](docs/reference.md) |
| Log format rules | `.elf/managed/LogConvention.md` in your project |
| Changes by version | [CHANGELOG.md](CHANGELOG.md) |

## License

This project applies a Dual License policy because the nature of "executable code" and "data structure specification (Protocol)" differs.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Applies to:** All source code (`.m`, `.py`, etc.) within the `5_SW/` and `6_Exp/*/Scripts/` folders.
  * **Condition:** If template core scripts are modified and improved for redistribution, those modifications must be released as open source. However, unique algorithms or raw data added by the user within the project may remain private (commercialized).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Applies to:** `README.md`, the governance documents (`.elf/managed/`, `0_Meta/`), the Session-Trial folder hierarchy, Base-Delta metadata logging rules, and the overall research methodology.
  * **Condition:** Anyone may freely adopt and adapt this structure and recording methodology, but when publishing derived templates or related research outputs, the original author Eli (projectschnee@gmail.com) and the source repository must be credited.
