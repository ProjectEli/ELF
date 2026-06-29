# LogConvention: ELF Logging Standard

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `LogConvention.md` (Korean). The AI agent operates from the
> Korean original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

This document defines the rules every human and AI agent in an ELF project follows for
writing experiment logs, saving result files, and AI handoff.

---

## 1. Log File Location & Naming (PARA workflow)

All research and planning work follows the PARA (Projects, Areas, Resources, Archives)
scheme and the `.claudeignore` rules. For the detailed principles, see
`0_Meta/AI_PARA_Framework.md`.

| Item | Rule |
|------|------|
| **In progress (Active Sandbox)** | When starting a new session (S{NNN}), always create and write the log (`S{NNN}_log.md`) at the **top (root) of `2_Log/`**. |
| **Conclusion summary (Wiki)** | When a session completes, summarize the key lessons or produced parameters in one or two lines in a knowledge document under `Wiki/`. Always include an absolute/relative path link to the original archived log file. |
| **Archive** | When a session ends, **always** move the full original log file to `Archive/` **under the same file name** (e.g., `Archive/S{NNN}_log.md` — no prefix tag; folder location is the state). |
| **Content rule** | Record only **pure metadata (a fact sheet)** — simulation parameters, run results, errors/fixes. |
| **Prohibited** | Do not record ideas, planning, or direction discussion in `2_Log/`; separate them into `1_Concept/`: small snippets and naive ideas in `13_Ideas/`, multi-session plans in `12_Planning/`. |

---

## 2. Log Format

```markdown
# S{NNN}: {session title}

> **Created**: YYYY-MM-DD\
> **Modified**: YYYY-MM-DD\
> **Status**: {★ 활성 | In Progress | Complete}\
> **목표 (Goal)**: {1-2 sentences on the session's core goal}\
> **관련 (Related)**: {related session/document links}\
> **Handoff**: {current state; unfinished work; reference files}

---

## t{NN}: {task title}

### 배경 (Background)  (optional: only when context exceeds the goal — prior-session synthesis, multiple inputs; omit for ordinary trials)
- {trial entry intent/context}
- **발의 (Initiator)**: {user-raised / AI-originated}
- {one line on a path rejected at entry — only if any; no Phase-1 after-the-fact editing}

### 목표 (Goal)
- {state the concrete task goal}

### 조건 (Conditions)
- {parameters, settings, constraints as a table or short list}

### 가설 (Hypothesis)
- {Phase 1, before execution — 3-5 suspected mechanisms/effects, nominal form}

### 예상 (Prediction)
- {Phase 1, before execution — 1-3 result figures/directions, quantified where possible}

### 관찰 (Observation)
- {Phase 2, after execution — phenomena, fact-centered results}
- {inline figure embed}
- {hypothesis/prediction comparison table recommended}

### 해석 (Interpretation)
- Hypothesis hit: {hit / miss / partial}
- {physical/logical interpretation of the observed facts}

### 교훈 (Lessons)
- {key insight, future caution}

### 생성 파일 (Files)

| Type | File |
|------|------|
| Script | `path` |
| Output | `path` |
| Figure | `path` |
```

### End of session body (at session close)

```markdown
---

## 다음 세션 후보 (Next-Session Hypothesis)

### 가설 후보
- {1-3 follow-up hypotheses, nominal form}

### 예상 후보
- {1-3 predicted results for the above hypotheses}
```

> Note: the section headers above (`### 목표 (Goal)`, etc.) are the literal operative
> format defined by the Korean original; they are kept verbatim here. The bilingual
> labels let an English reader follow real logs.

### Rules
- **Writing language**: write logs in the language set by `PROJECT_LANG` in `0_Meta/EliRule.md`, but for **token economy** always use nominal endings ('-음/함/임') and word-centered bullet points. English may be added for technical terms.
- **Status**: `★ 활성 (active)` (currently working), `In Progress` (intermediate stage), `Complete` (done, before archiving).
- **Handoff**: three parts separated by semicolons (`;`) — `current state; unfinished work; reference files`. Update continuously during the session so the next session can pick up context by reading just this field with a `Read(offset=0, limit=9)` pattern. Initial value `-`.
- **Header line breaks**: the trailing `\` on each blockquote (`>`) header line is a CommonMark **hard break** — it preserves line separation in strict renderers (e.g., Discord preview) (without it, the six items collapse into one line). **Do not delete.** The last line (Handoff) has no `\` (block end). Keep the header at 6 lines → the `limit=9` quick-read stays valid.
- **Trial numbers**: `t01`, `t02`, … in order. No duplicates.
- **Image paths**: `![alt text](../6_Exp/64_Viz/S{NNN}/filename.png)` (relative to `2_Log`).
- **Inline figure embedding required (plot = trial output)**: every plot is the output of a specific trial (`t{NN}`) — when a figure is produced by adding a trial (base delta), embed it inline (`![alt](path)`) in that trial's `### 관찰 (Observation)` section **in the same work turn**, with the figure number plus a one-line description (axes, key observation) in the alt text. **Prohibited**: ① recording only a path in a file-list table without embedding in the body, ② deferring for being final/awaiting-feedback/intermediate (v1) (embed v1 immediately, then update), ③ replacing the log embed with external display (sending to a chat/viewer). `elf validate` detects 64_Viz figures missing a body embed (`--strict` promotes to an issue; intentional exclusion: `<!-- noembed: file.png -->`).
- **Trial and error — base-delta trials**: when **iteratively improving** the same kind of plot/analysis, expand each revision as an independent trial (`t{NN}`). Each trial takes the previous one as base, with the **delta (what changed) + reason (problem with the previous attempt)** in `### 조건 (Conditions)` and that version's figure embedded in `### 관찰 (Observation)`. Even when the plot script is regenerated, **keep every version** (§3.3), and keep intermediate figures in each trial → trial-and-error is preserved as a reproducible trial chain. **Block survivorship bias**: no "quietly fix and report only the final" (AI included) — record each version as a trial in that turn (no overwriting, no after-the-fact recall). **Trigger (internalize)**: *the moment you regenerate a working output (figure/table/data) with changed parameters/style = always the next trial (delta)* + version preservation (§3.3). Only pre-working debugging (making a failed output succeed) stays in the same trial's `### 시행착오 (Trial-and-Error)` table — i.e., **editing a finished output = delta-trial / debugging to working = trial-and-error table**. **Tighten the small-change exception**: only changes that **do not alter the output's appearance** (typo, path, comment, variable name) can end in a table — **any change to a figure/table's appearance (font, color, size, axes, style, parameters) is a delta-trial even once**. Expansion cost is handled by the §3.3 core+wrapper pattern.
- **Files table**: organize the assets produced in each task as a table to maximize readability and token efficiency. Tabulate scripts, data, and figures. State the type (`Script`, `Output`, `Figure`, `Config`, etc.) and the path relative to the project root.
- **Code usage**: record in code blocks (```lang ... ```).
- **Parameter table**: organize variable name, value, and unit as a table.
- **Hypothesis/Prediction sections (Phase 1)**: write **before** running the trial. Nominal-form list. If hypotheses exceed 5 items or a reasoning chain exceeds 5 steps, escape to `1_Concept/12_Planning` and leave a one-line cross-ref stub. No paragraph prose. Do not just repeat qualitative predictions — quantify where possible.
- **Observation section (Phase 2)**: write **after** running the trial. A comparison table against hypothesis/prediction is recommended (2 columns: `예상 (Prediction)` vs `관찰 (Observation)`).
- **Interpretation one-line rule**: the first line states one of `Hypothesis hit: hit / miss / partial`. Mechanism interpretation from the second line.
- **Next-Session Hypothesis**: just before closing the session (Status → Complete), write a `## 다음 세션 후보 (Next-Session Hypothesis)` section at the end of the body. 1-3 hypothesis candidates + 1-3 prediction candidates. At the next session S{NNN+1}, carry them over into t01's `### 가설 (Hypothesis)` / `### 예상 (Prediction)` to avoid breaking the hypothesis chain across sessions.
- **Background section (optional, conditional)**: separate `### 배경 (Background)` above `### 목표` **only when context exceeds the goal** (prior-session synthesis, multiple inputs). Ordinary trials let the goal absorb it — no standing heading, no empty background. Keep it light (1-2 lines).
- **Initiator**: name the initiator of the work/pivot in the background (or goal) in one word (`user-raised` / `AI-originated`) — preserves the provenance of the thinking flow. The verbatim original query is preserved by the session JSONL, so it is not duplicated in the log (SSOT = absorbed into the trial body).
- **Rejected-alternative placement (by timing)**: path rejected at entry → `### 배경` (one line, no Phase-1 after-the-fact editing) / scope exclusion → `### 조건` / result-based rejection → `### 해석`·`### 교훈`. Do not dump all into the background (timing inversion / post-hoc rationalization).
- **Retroactive policy**: trials/sessions before this rule took effect are not force-backfilled. **Applies to new writing from now on.** For a trial about to enter the archive, filling one line in retrospect is recommended.

---

## 3. Result-File Saving Rules

### 3.1 Scripts

| Kind | Location | Naming |
|------|------|------|
| Simulation script | `61_Sim/Scripts/` | `S{NNN}_sim.m` |
| Post-processing script | `61_Sim/Scripts/` or `63_Analysis/Scripts/` | `S{NNN}_postProcess.m` |
| Helper function | `61_Sim/Scripts/` | `{function_name}.m` |
| Analysis code | `63_Analysis/Scripts/` | `S{NNN}_analysis.m` |

### 3.2 Data

| Kind | Location | Naming |
|------|------|------|
| Simulation result | `61_Sim/Data/S{NNN}/` | `S{NNN}_sim_results.mat` |
| Graph | `61_Sim/Data/S{NNN}/` | `S{NNN}_Fig1_*.png` |
| Measured original | `62_Empirical/Raw/` | Read-Only |
| Processed data | `62_Empirical/Processed/S{NNN}/` | e.g., `S{NNN}_t##_processed.csv` |

### 3.3 Naming Rules
- Simulation result: `S{NNN}_{trial}_{description}.mat`
- Graph: `S{NNN}_{description}.png`
- When script trials must be distinguished: `S{NNN}_t{NN}.m`, `S{NNN}_t{NN}_postProcess.m`
- **Preserve iterative-improvement (trial-and-error) versions**: do not overwrite scripts/figures; preserve a **version suffix** — `S{NNN}_A06a_*`→`_A06b_*`… or `_v1`/`_v2`. Each version ↔ a delta trial 1:1. Overwriting loses the intermediate (failed) versions → not reproducible.
- **core + wrapper pattern (handles expansion cost)**: instead of cloning a full script per version, use a **shared core + a thin per-version wrapper** (delta = parameters) — `S{NNN}_A06_core.m` (absorbs variants via parameters like `normMode`, `isoLevels`, `clim`) + `S{NNN}_A06{a..}_plot.m` (one struct call each = that version's delta). The code diff is the delta, satisfying readability and preservation at once.

### 3.4 Visualization

| Kind | Location | Note |
|------|------|------|
| Auto-generated figure | `6_Exp/64_Viz/` | Auto-saved by scripts. (The figure index `_gallery.md` is auto-generated per session by `elf gallery`.) |
| Paper figure | `7_Paper/71_Figs/` | rawFig → processedFig → finalFig |

---

## 4. Cross-reference Rules

| From | To | Relative path (from `2_Log`) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/12_Planning/P00x.md` | `../1_Concept/12_Planning/P00x.md` |
| Logs → Sim Data | `6_Exp/61_Sim/Data/S{NNN}/` | `../6_Exp/61_Sim/Data/S{NNN}/` |
| Logs → Viz | `6_Exp/64_Viz/S{NNN}/` | `../6_Exp/64_Viz/S{NNN}/` |
| Scripts → Data | `../Data/S{NNN}/` | `../Data/S{NNN}/` |

Content separated into Planning is marked in the original log with a blockquote stub:
```markdown
> Details have been separated into a Planning document.
> - [P00x_Title.md](../1_Concept/12_Planning/P00x_Title.md) — see section N
```

---

## 5. New-session Checklist

When an AI agent starts a new session (S{NNN}):

- [ ] Create `2_Log/S{NNN}_log.md` (follow the format above)
- [ ] Check the previous session S{NNN-1}'s `## 다음 세션 후보 (Next-Session Hypothesis)` section → carry over into the new trial t01's `### 가설 (Hypothesis)` / `### 예상 (Prediction)`
- [ ] Create the `6_Exp/61_Sim/Data/S{NNN}/` directory
- [ ] Write the simulation/experiment script `6_Exp/61_Sim/Scripts/S{NNN}_*.m`
- [ ] After running, save the result .mat + .png
- [ ] Add a trial (t{NN}) to the log: parameters, results, image path
- [ ] When work is done: summarize the knowledge gained in `2_Log/Wiki/`, and move `S{NNN}_log.md` to `2_Log/Archive/`
- [ ] When a script is done: move one-off scripts to `Scripts/Archive/`, keep general-purpose ones at the root
- [ ] Update `2_Log/Wiki/Session_Registry.tsv` with the session entry
- [ ] If Planning content is included → separate into `1_Concept/` + cross-reference

### 5.1 Trial Standard Operating Procedure (Phase 1 / Phase 2 split)

Write each trial (t{NN}) in two phases. **A stop point between Phase 1 and Phase 2 is required** — it prevents ad-hoc execution and forces hypothesis-observation cycle closure.

**Phase 1 — before execution (pre-execution, stop point)**
- [ ] Write the `## t{NN}: [task title]` header
- [ ] (optional) `### 배경 (Background)` — only when context exceeds the goal: entry intent + **initiator** (user-raised/AI-originated) + (one line on a rejected entry path)
- [ ] Write `### 목표 (Goal)` (the what of the task)
- [ ] Write `### 조건 (Conditions)` (fix parameters/constraints)
- [ ] Write `### 가설 (Hypothesis)` (3-5 suspected mechanisms/effects, nominal form)
- [ ] Write `### 예상 (Prediction)` (1-3 concrete result predictions, quantified where possible)
- [ ] **Stop** — after confirming hypothesis/prediction, enter Phase 2 (an auto-mode AI also honors this stop point)

**Phase 2 — after execution (post-execution)**
- [ ] Run the trial (script/simulation/survey, etc.)
- [ ] **Check the figure right after running the code**: as soon as MATLAB (etc.) finishes, check whether the figure this trial produced is in `6_Exp/64_Viz/S{NNN}/` (before leaving the run turn). If so, embed it without omission per the step below — do not defer for finality/feedback/external display (sending to a chat/viewer).
- [ ] Write `### 관찰 (Observation)` (facts + hypothesis/prediction comparison table)
- [ ] Embed the produced figure inline in `### 관찰 (Observation)` (`![alt](path)`; a path-only table entry is not an embed — `elf validate` detects it)
- [ ] **Iterative-improvement decision (trigger)**: if you regenerate a working output with changed parameters/style → expand as the next trial (delta) (preserve script versions §3.3 + figure embed + delta/reason in `### 조건 (Conditions)`). **An appearance change (font, color, size, style) is a delta-trial even once**; only appearance-preserving fixes (typo, path) go in a `### 시행착오 (Trial-and-Error)` table.
- [ ] Write `### 해석 (Interpretation)` (first line: state hypothesis hit + mechanism interpretation)
- [ ] Write `### 교훈 (Lessons)`
- [ ] Write `### 생성 파일 (Files)`

### 5.2 Session-close Standard Procedure

Just before switching Status to `Complete`:
- [ ] Write a `## 다음 세션 후보 (Next-Session Hypothesis)` section at the end of the body
- [ ] State 1-3 hypothesis candidates + 1-3 prediction candidates (nominal form, one-line list)
- [ ] Add a key-conclusion summary + Archive path link in `Wiki/`
- [ ] Move `S{NNN}_log.md` → `Archive/` under the same file name
- [ ] Update the status in `Session_Registry.tsv`
