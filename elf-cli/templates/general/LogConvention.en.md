# LogConvention: ELF Logging Standard (general)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `LogConvention.md` (Korean). The AI agent operates from the
> Korean original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

This document defines the rules every human and AI agent in an ELF `general` project
follows for writing session logs, saving outputs, and AI handoff.

> **Default trial form = 5-section** (`### 가설 (Hypothesis)`, `### 예상 (Prediction)`, `### 관찰 (Observation)`, `### 해석 (Interpretation)`),
> on the premise that every intentional change of work has a latent hypothesis and
> prediction. If a different form fits the project, state the trial form in
> `0_Meta/ProjectRule.md` to override (a one-time per-project setting — not decided per
> trial).

---

## 1. Log File Location & Naming (PARA workflow)

All work and planning follows the "location is the state" rule — folder root = in progress, `Wiki/` = key summaries, `Archive/` = closed records (reading allowed), `Deprecated/` = withdrawn (reading allowed, not a basis for current decisions). For the detailed principles, see `.elf/managed/AI_PARA_Framework.md`.

| Item | Rule |
|------|------|
| **In progress (folder root)** | When starting a new session (S{NNN}), create and write `S{NNN}_log.md` at the **top (root) of `2_Log/`**. |
| **Conclusion summary (Wiki)** | When a session completes, summarize the key lessons/results in one or two lines in a `Wiki/` knowledge document. Always include a path link to the original archived log. |
| **Archive** | When a session ends, move the original log file to **`Archive/` under the same file name** (no prefix tag; folder location is the state). Closed records remain valid and may be read. |
| **Deprecated** | Withdrawn session logs, trials, sections, or blocks are moved with `elf deprecate` to `Deprecated/`, leaving a one-line move marker at the original position (§6). They may be read but are not a basis for current decisions. The user decides what to deprecate. |
| **Prohibited** | Do not record ideas, planning, or direction discussion in `2_Log/`; separate them into `1_Concept/`: small snippets in `13_Ideas/`, multi-session plans in `12_Planning/`. |

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
- {concrete task goal}

### 조건 (Conditions)
- {parameters, settings, constraints as a table or short list}

### 가설 (Hypothesis)
- {Phase 1, before execution — 3-5 suspected intentions/mechanisms/effects, nominal form}

### 예상 (Prediction)
- {Phase 1, before execution — 1-3 result predictions, quantified where possible}

### 관찰 (Observation)
- {Phase 2, after execution — fact-centered results}
- {hypothesis/prediction comparison table recommended}

### 해석 (Interpretation)
- Prediction match: {match / partial / mismatch}
- {interpretation of the observed facts}

### 교훈 (Lessons)
- {key insight, future caution}

### 생성 파일 (Files)

| Type | File |
|------|------|
| {type} | `path` |
```

> Note: the section headers above (`### 목표 (Goal)`, etc.) are the literal operative
> format defined by the Korean original; they are kept verbatim here.

### Rules
- **Writing language**: write logs in the language set by `PROJECT_LANG` in `.elf/managed/EliRule.md`, and use nominal endings ('-음/함/임') (EliRule §3, sentence-ending form). English may be added for technical terms.
- **Status**: `★ 활성 (active)` (currently working), `In Progress` (intermediate stage), `Complete` (done, before archiving).
- **Handoff**: a **replace-style state snapshot** — three `;` parts: `currently valid conclusion/position (the current net result of what accumulated, e.g. "A rejected · B adopted"); pending; references (location of the synthesis trial)`. Update by **rewriting the whole line**, not by appending. **Do not list history / completed timelines** — the session overview belongs to the registry key finding, details to the trial bodies. Keep it one line. **`;` is reserved for part boundaries** — use `·`/`,` inside a part, and start the pending/reference parts with their labels (`미완료`/`pending`, `참조`/`refs`) — tools identify a boundary as `;` immediately followed by a label. On session handover or a context rebuild, `Read(offset=0, limit=9)` reads just this field to restore the working state. Closing with pending items left triggers an `elf session close` warning. Initial value `-`.
- **Registry key finding**: the same replace-update principle as Handoff — replace-update it with the currently valid final conclusion (not an accumulating narrative), and rewrite it as the final conclusion at session close (§5.1).
- **Header line breaks**: the trailing `\` on each blockquote (`>`) header line is a CommonMark **hard break** — it preserves line separation in strict renderers. **Do not delete.** The last line (Handoff) has no `\`. Keep the header at 6 lines → the `limit=9` quick-read stays valid.
- **Trial numbers**: `t01`, `t02`, … in order. No duplicates.
- **Base-delta trial expansion**: when regenerating/improving a working output by changing it (parameters, structure, content, style), expand each attempt as an **independent trial (`t{NN}`)**. Each trial takes the previous one as base, recording the **delta (what changed) + reason (problem with the previous attempt)** in `### 조건 (Conditions)`. **Version preservation**: do not overwrite outputs/scripts; preserve versions (losing intermediates = not reproducible). **Block survivorship bias**: no "quietly fix and report only the final" (AI included) — record each version as a trial in that turn (no after-the-fact recall). Only pre-working debugging (a failed output → success) stays in the same trial's `### 시행착오 (Trial-and-Error)` table.
- **Hypothesis/Prediction sections (Phase 1)**: write **before** running the trial. Nominal-form list. If hypotheses exceed 5 items or a reasoning chain exceeds 5 steps, move it to `1_Concept/12_Planning` and leave a one-line cross-ref stub. No paragraph prose. Quantify where possible.
- **Observation section (Phase 2)**: write **after** running the trial. A comparison table against hypothesis/prediction is recommended (2 columns: `예상 (Prediction)` vs `관찰 (Observation)`).
- **Interpretation one-line rule**: the first line states one of `Prediction match: match / partial / mismatch` (logs written before v2.24 may keep the older `가설 적중 여부` line — retroactive policy). Interpretation from the second line.
- **`## 다음 세션 후보` (Next-Session Hypothesis) section — not recommended (off since v2.20)**: low utilization; record follow-ups in the **Handoff pending part** while active, in the **registry key finding (fold)** at close, and in `1_Concept/12_Planning/` when they span sessions. Backward compatible: the section in existing logs stays valid (`elf trial new` inserts before it) and `elf session close` no longer requires it. Old template = `.elf/managed/templates/Archive/sessionTemplate_v1.md`.
- **Background section (optional, conditional)**: separate `### 배경 (Background)` above `### 목표` **only when context exceeds the goal** (prior-session synthesis, multiple inputs). Ordinary trials include it in the goal section — no standing heading, no empty background. Keep it light (1-2 lines).
- **Initiator**: name the initiator of the work/pivot in the background (or goal) in one word (`user-raised` / `AI-originated`) — preserves the origin of the work. The verbatim original query is preserved by the session JSONL, so it is not duplicated in the log (SSOT = included in the trial body).
- **Rejected-alternative placement (by timing)**: path rejected at entry → `### 배경` (one line, no Phase-1 after-the-fact editing) / scope exclusion → `### 조건` / result-based rejection → `### 해석`·`### 교훈`. Do not place all of them in the background (timing inversion / post-hoc rationalization).
- **Canonical rules over precedent (precedent ≠ norm)**: past session/trial logs are reference material only, not the norm for format or rules — the canonical sources are this document and `.elf/managed/templates/` (sessionTemplate, trialTemplate). When a precedent deviates from the canonical rules, follow the canonical rules and do not imitate the deviation (report it to the user). Adding a trial via `elf trial new` (appends the current canonical stub to the active log; without the CLI, copy trialTemplate manually) is recommended.
- **Retroactive policy**: trials/sessions before this rule took effect are not force-backfilled. Applies to new writing from now on.

---

## 3. Output-Saving Rules
- Save work outputs (code, documents, data, assets) in the domain folders the project defines, and record the type + project-root-relative path as a table in the session log's `### 생성 파일 (Files)`.
- **Version preservation**: on iterative improvement, do not overwrite; preserve with a version suffix (`_v1`/`_v2`, etc.) — each version ↔ a delta trial 1:1.
- Record code usage in code blocks (```lang … ```); record parameters as a table of variable name, value, unit.

---

## 4. Cross-reference Rules

| From | To | Relative path (from `2_Log`) |
|------|----|----------------------|
| Logs → Planning | `1_Concept/12_Planning/P##.md` | `../1_Concept/12_Planning/P##.md` |

Content separated into Planning is marked in the original log with a blockquote stub:
```markdown
> Details have been separated into a Planning document.
> - [P##_Title.md](../1_Concept/12_Planning/P##_Title.md) — see section N
```

---

## 5. Session Standard Operating Procedure (Phase 1 / Phase 2 split)

Write each trial (t{NN}) in two phases. **A stop point between Phase 1 and Phase 2 is required** — it prevents ad-hoc execution and forces completion of the hypothesis-observation cycle.

**Phase 1 — before execution (pre-execution, stop point)**
- [ ] Create the trial stub: `elf trial new [title]` recommended (appends the current trialTemplate; without the CLI, copy `.elf/managed/templates/trialTemplate.md` manually) → (optional) `### 배경 (Background)` (when context exceeds the goal: intent + initiator) + `### 목표 (Goal)` + `### 조건 (Conditions)` (fix parameters/constraints)
- [ ] `### 가설 (Hypothesis)` (3-5 suspected intentions/mechanisms/effects, nominal form)
- [ ] `### 예상 (Prediction)` (1-3 concrete result predictions, quantified where possible)
- [ ] **Stop** — after confirming hypothesis/prediction, enter Phase 2 (an auto-mode AI also observes this stop point)

**Phase 2 — after execution (post-execution)**
- [ ] Run the trial
- [ ] `### 관찰 (Observation)` (facts + hypothesis/prediction comparison table)
- [ ] **Iterative-improvement decision**: if you regenerate a working output with changes → expand as the next trial (delta) (version preservation §3 + delta/reason in `### 조건 (Conditions)`). Appearance/content changes are delta-trials; only no-change fixes (typo, path) go in a `### 시행착오 (Trial-and-Error)` table.
- [ ] `### 해석 (Interpretation)` (first line: prediction match) + `### 교훈 (Lessons)` + `### 생성 파일 (Files)`

### 5.1 Session-close Standard Procedure
Just before switching Status to `Complete`:
- [ ] **Run `elf validate` — resolve the warnings** before proceeding: once moved to Archive the log leaves the structure-check scope, so **before close is the last verification point**
- [ ] Clean the Handoff: clear the pending part (done, or carried into the registry key finding / a `12_Planning/` document) — remaining pending items trigger an `elf session close` warning
- [ ] Add a key-conclusion summary + Archive path link in `Wiki/`
- [ ] Move `S{NNN}_log.md` → `Archive/` under the same file name
- [ ] Update the status in `Session_Registry.tsv` + rewrite the key finding as the final conclusion

---

## 6. Deprecated

Deprecated = a record that was tried but is no longer a basis for current decisions. It is
not deleted; it is moved to `Deprecated/` in the same folder and stays readable. The user
decides what to deprecate; the agent proposes candidates with reasons and runs
`elf deprecate` only after an instruction or approval (usage: `elf deprecate --help`, the CLI reference).

| Unit | What moves | `Deprecated/` side | Original position |
|------|-------------|------------------|---------|
| Whole session log | The whole file | `2_Log/Deprecated/S###_log.md` — YAML header at the top (`deprecated: date` · `source` · `replaced_by` · `reason` · `status_before`), header `Status: Deprecated` | No file. Registry row: Status `Deprecated`, path column `Deprecated/S###_log.md` |
| Trial | The whole body under the heading | A block appended to `2_Log/Deprecated/S###_log.partial.md` | `## tNN:` heading + one move-marker line (number preserved) |
| Section | Including the `### section` heading | The same partial file | One move-marker line (no heading) |
| Marked block · line range | Between `<!-- deprecate:begin -->` … `<!-- deprecate:end -->` / the given lines | The same partial file | One move-marker line |
| Planning document | Whole file / marked block / line range | `1_Concept/12_Planning/Deprecated/` | Same |

- Partial file: YAML header `deprecated: partial` + `source` at the top; per block a copy of the `## tNN:` heading + the marker line `> **Deprecated**: date · ID · unit [· replaced_by: …] [· reason: …]` + the original text (between `<!-- deprecated:begin ID -->` … `<!-- deprecated:end ID -->`, relative links recomputed for the folder). ID = a per-document sequence such as `S###-D01`.
- Move-marker line: `> **Deprecated** → [Deprecated/S###_log.partial.md](Deprecated/S###_log.partial.md) (ID · unit · date [· replaced_by: …])` — `elf session close` recomputes the link for the Archive location.
- Outputs (scripts, data, figures) stay where they are. The Handoff, the registry key finding, and references in other documents are not edited by the tool; they are printed as `review:` lines for the agent to handle.
- Restore: `elf deprecate --restore <ID>` (block) or `--restore S###` (whole). List and cross-check: `elf deprecate list`.
- `elf validate` includes logs under `Deprecated/` in the numbering and registry checks. It does not run structure or link checks on them.
