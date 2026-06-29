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

All work and planning follows the PARA (Projects, Areas, Resources, Archives) scheme and
the `.claudeignore` rules. For the detailed principles, see `0_Meta/AI_PARA_Framework.md`.

| Item | Rule |
|------|------|
| **In progress (Active Sandbox)** | When starting a new session (S{NNN}), create and write `S{NNN}_log.md` at the **top (root) of `2_Log/`**. |
| **Conclusion summary (Wiki)** | When a session completes, summarize the key lessons/results in one or two lines in a `Wiki/` knowledge document. Always include a path link to the original archived log. |
| **Archive** | When a session ends, move the original log file to **`Archive/` under the same file name** (no prefix tag; folder location is the state). |
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
- Hypothesis hit: {hit / miss / partial}
- {interpretation of the observed facts}

### 교훈 (Lessons)
- {key insight, future caution}

### 생성 파일 (Files)

| Type | File |
|------|------|
| {type} | `path` |
```

End of session body, at session close:

```markdown
---

## 다음 세션 후보 (Next-Session Hypothesis)

### 가설 후보
- {1-3 follow-up hypotheses, nominal form}

### 예상 후보
- {1-3 predicted results for the above hypotheses}
```

> Note: the section headers above (`### 목표 (Goal)`, etc.) are the literal operative
> format defined by the Korean original; they are kept verbatim here.

### Rules
- **Writing language**: write logs in the language set by `PROJECT_LANG` in `0_Meta/EliRule.md`, but for **token economy** use nominal endings ('-음/함/임') and word-centered bullet points. English may be added for technical terms.
- **Status**: `★ 활성 (active)` (currently working), `In Progress` (intermediate stage), `Complete` (done, before archiving).
- **Handoff**: three parts separated by semicolons (`;`) — `current state; unfinished work; reference files`. Update continuously during the session so the next session can pick up context by reading just this field with a `Read(offset=0, limit=9)` pattern. Initial value `-`.
- **Header line breaks**: the trailing `\` on each blockquote (`>`) header line is a CommonMark **hard break** — it preserves line separation in strict renderers. **Do not delete.** The last line (Handoff) has no `\`. Keep the header at 6 lines → the `limit=9` quick-read stays valid.
- **Trial numbers**: `t01`, `t02`, … in order. No duplicates.
- **Base-delta trial expansion**: when regenerating/improving a working output by changing it (parameters, structure, content, style), expand each attempt as an **independent trial (`t{NN}`)**. Each trial takes the previous one as base, recording the **delta (what changed) + reason (problem with the previous attempt)** in `### 조건 (Conditions)`. **Version preservation**: do not overwrite outputs/scripts; preserve versions (losing intermediates = not reproducible). **Block survivorship bias**: no "quietly fix and report only the final" (AI included) — record each version as a trial in that turn (no after-the-fact recall). Only pre-working debugging (a failed output → success) stays in the same trial's `### 시행착오 (Trial-and-Error)` table.
- **Hypothesis/Prediction sections (Phase 1)**: write **before** running the trial. Nominal-form list. If hypotheses exceed 5 items or a reasoning chain exceeds 5 steps, escape to `1_Concept/12_Planning` and leave a one-line cross-ref stub. No paragraph prose. Quantify where possible.
- **Observation section (Phase 2)**: write **after** running the trial. A comparison table against hypothesis/prediction is recommended (2 columns: `예상 (Prediction)` vs `관찰 (Observation)`).
- **Interpretation one-line rule**: the first line states one of `Hypothesis hit: hit / miss / partial`. Interpretation from the second line.
- **Next-Session Hypothesis**: just before closing the session (Status → Complete), write at the end of the body. Carry over into the next session's t01 `### 가설 (Hypothesis)` / `### 예상 (Prediction)` to avoid breaking the hypothesis chain across sessions.
- **Background section (optional, conditional)**: separate `### 배경 (Background)` above `### 목표` **only when context exceeds the goal** (prior-session synthesis, multiple inputs). Ordinary trials let the goal absorb it — no standing heading, no empty background. Keep it light (1-2 lines).
- **Initiator**: name the initiator of the work/pivot in the background (or goal) in one word (`user-raised` / `AI-originated`) — preserves the provenance of the thinking flow. The verbatim original query is preserved by the session JSONL, so it is not duplicated in the log (SSOT = absorbed into the trial body).
- **Rejected-alternative placement (by timing)**: path rejected at entry → `### 배경` (one line, no Phase-1 after-the-fact editing) / scope exclusion → `### 조건` / result-based rejection → `### 해석`·`### 교훈`. Do not dump all into the background (timing inversion / post-hoc rationalization).
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

Write each trial (t{NN}) in two phases. **A stop point between Phase 1 and Phase 2 is required** — it prevents ad-hoc execution and forces hypothesis-observation cycle closure.

**Phase 1 — before execution (pre-execution, stop point)**
- [ ] `## t{NN}: [task title]` header + (optional) `### 배경 (Background)` (when context exceeds the goal: intent + initiator) + `### 목표 (Goal)` + `### 조건 (Conditions)` (fix parameters/constraints)
- [ ] `### 가설 (Hypothesis)` (3-5 suspected intentions/mechanisms/effects, nominal form)
- [ ] `### 예상 (Prediction)` (1-3 concrete result predictions, quantified where possible)
- [ ] **Stop** — after confirming hypothesis/prediction, enter Phase 2 (an auto-mode AI also honors this stop point)

**Phase 2 — after execution (post-execution)**
- [ ] Run the trial
- [ ] `### 관찰 (Observation)` (facts + hypothesis/prediction comparison table)
- [ ] **Iterative-improvement decision**: if you regenerate a working output with changes → expand as the next trial (delta) (version preservation §3 + delta/reason in `### 조건 (Conditions)`). Appearance/content changes are delta-trials; only no-change fixes (typo, path) go in a `### 시행착오 (Trial-and-Error)` table.
- [ ] `### 해석 (Interpretation)` (first line: hypothesis hit) + `### 교훈 (Lessons)` + `### 생성 파일 (Files)`

### 5.1 Session-close Standard Procedure
Just before switching Status to `Complete`:
- [ ] Write `## 다음 세션 후보 (Next-Session Hypothesis)` at the end of the body (1-3 hypothesis candidates + 1-3 prediction candidates, nominal form)
- [ ] Add a key-conclusion summary + Archive path link in `Wiki/`
- [ ] Move `S{NNN}_log.md` → `Archive/` under the same file name
- [ ] Update the status in `Session_Registry.tsv`
