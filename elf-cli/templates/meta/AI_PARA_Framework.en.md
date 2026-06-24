# AI PARA Framework & Context Management

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `AI_PARA_Framework.md` (Korean). The AI agent operates from the
> Korean original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See §1.1 below.

This document defines the **AI-tailored PARA (Projects, Areas, Resources, Archives)
file-management rules** that prevent a project's large body of experiment logs and
planning documents from contaminating an AI agent's context window (hallucination), and
that maximize human-AI collaboration efficiency.

## 1. The Firewall Principle

The project's top folder contains a `.claudeignore` file.
When the AI autonomously searches or reads the file system, this file **firewalls**
specific folder names so the AI treats them as invisible and cannot perceive them.

*   **Blocked targets**: `Archive/`, `*Archive*`, and other discarded or closed past records.
*   **Effect**: ensures the AI answers based only on "the project's currently valid state," and fully prevents mistaking old failed settings (e.g., a wrong parameter map, a discarded paper direction) for current fact.

### 1.1 Informative Companion isolation

A file of the form `*.en.md` (general: `*.<lang>.md`) is an **informative translation
(read-only human companion)** of a governance document — for international users to
*read*, and **not an operative source**. The operative authority is always the
same-named `*.md` (the PROJECT_LANG original, Korean by default).

*   **AI behavior rule**: the AI takes rules, structure, and instructions **only from the `*.md` original**. It does not treat `*.en.md` as a basis (even if read, it is not a rule source). On a conflict between the original and the companion, **the `*.md` original wins**.
*   **Customization path**: write project-rule changes in **`ProjectRule.md` (user-owned, operative, project language)** — do not edit the managed original or the companion (the managed file is replaced by `elf update`, and a companion edit has no effect).
*   **Effect**: fixing the operative version to a single original keeps a translation difference from changing AI behavior. Every language project runs under the same governance; international users still override in their own language via ProjectRule.

---

## 2. Hybrid PARA Structure (Focus-and-Filter)

To achieve both "cognitive-load reduction (grouping)" for humans and "path flattening"
for the AI, the project runs on a **hybrid structure**.

### Workbench (Default Root / Active Sandbox)
*   **Purpose**: where content that is "in progress" right now, or valid this week, lives.
*   **Example files**: `Current_Analysis_Task.md`, `S014_log.md`
*   **Operating rule**: instead of making a separate `1_Active` folder, use the **top (root) of the working parent folder (e.g., `12_Planning/` or `2_Log/`)** as the workbench. Write freely here until the work is done and becomes a past record.

### `Wiki/` (Human Sanctuary)
*   **Purpose**: a place that gathers only the **"unchanging facts, conclusions, and key rules"** obtained after a workbench task finishes, summarized in one or two lines.
*   **Operating rule**: a human researcher comes here when they want to read only "the currently most important facts/rules." It also serves as a fact sheet that provides the AI a concise summary context.

### `Archive/` (The Firewall Bin)
*   **Purpose**: holds original log records that are fully complete or deprecated and no longer "the current concern," but may be referenced later.
*   **Operating rule**: when a folder's top gets cluttered, move old files **under the same name** to this folder (no prefix tag — **the folder location is the state**). The moment a file enters here, `.claudeignore` makes it disappear completely from the AI's autonomous search.

---

## 3. Scripts Folder Management (code archiving)

To keep scripts from piling up indiscriminately inside `61_Sim/Scripts` or
`63_Analysis/Scripts`, apply the same PARA logic to scripts.

1.  **Active Scripts**: keep the latest scripts under active development or general use at the root of the Scripts folder.
2.  **Archived Scripts**: move scripts used one-off for a specific past session under `Scripts/Archive/`.
3.  **Wiki Tracking (Registry)**: when moving a script to Archive, record in a `Wiki/` document — **with the path** — which session used it and for what.

## 4. How to bypass the firewall

The AI cannot rummage through `Archive` on its own, but when a human developer requests
restoring/analyzing a specific past record, it can read it via an **explicit instruction**.

### Method: force an absolute/relative path
When the user gives an exact path snippet in the prompt — "open past file A" — the AI can
read that file normally and load it into context.

*   *(example user prompt)*: "Open `2_Log/Archive/S005_log.md` and summarize the parameter trend at the time."
*   To support this, `Wiki` knowledge documents should always include **explicit file-path links** toward `Archive/...` in case past data is needed.
