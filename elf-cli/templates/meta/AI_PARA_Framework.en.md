# AI PARA Framework & Context Management

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `AI_PARA_Framework.md` (Korean). The AI agent operates from the
> Korean original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See §1.1 below.

This document defines the rule by which the folder a document (session log, planning
document) sits in states its status, and how an AI agent reads and writes documents in each
location. The rule keeps withdrawn or outdated records from being used as current fact and
gives people and the AI one shared basis for managing documents. The name comes from the
Archive concept of PARA (Projects, Areas, Resources, Archives); the rule itself is defined so
that folder names alone are enough to understand it.

## 1. Location = State

No file-name prefixes, tags, or separate ledger: the folder a document sits in states its
status. Under the folder a document belongs to (e.g., `2_Log/`, `1_Concept/12_Planning/`),
three state folders sit at the same level.

| Location | State | Content | AI reading |
|------|------|------|--------|
| Folder root | In progress | Documents being written or edited now (e.g., `2_Log/S014_log.md`) | Basis for current work |
| `Wiki/` | Key summaries | One- or two-line summaries of the confirmed conclusions, parameters, and rules left after work finishes; the Session Registry | Summary of current facts — the first place a person reads |
| `Archive/` | Closed records | Original session logs and planning documents of closed work (moved under the same file name). Still-valid records, referred to when tracing past conclusions and paths | Reading allowed |
| `Deprecated/` | Withdrawn | Documents and blocks that were tried and then withdrawn (moved by `elf deprecate`). Kept, not deleted | Reading allowed — to see what was tried and withdrawn. Not a basis for current decisions |

- No separate `1_Active` folder. Documents in progress stay at the root of their parent folder.
- State folders are flat (no subfolders). File names do not change when a file moves.
- The user decides what to deprecate. The agent proposes candidates with reasons and runs `elf deprecate` only after the user's instruction or approval. Format and procedure: `.elf/managed/LogConvention.md` §6.

### 1.1 Informative Companion

A file of the form `*.en.md` (general: `*.<lang>.md`) is an **informative translation
(read-only human companion)** of a governance document — for international users to
*read*, and **not an operative source**. The operative authority is always the
same-named `*.md` (the PROJECT_LANG original, Korean by default).

*   **AI behavior rule**: the AI takes rules, structure, and instructions **only from the `*.md` original**. It does not treat `*.en.md` as a basis (even if read, it is not a rule source). On a conflict between the original and the companion, **the `*.md` original takes precedence**.
*   **Customization method**: write project-rule changes in **`ProjectRule.md` (user-owned, operative, project language)** — do not edit the managed original or the companion (the managed file is replaced by `elf update`, and a companion edit has no effect).
*   **Effect**: fixing the operative version to a single original keeps a translation difference from changing AI behavior. Every language project runs under the same governance; international users still override in their own language via ProjectRule.

---

## 2. Moving session logs and planning documents

| When | Action | Tool |
|------|------|------|
| Session start | Create `2_Log/S###_log.md` (root = in progress) | `elf session new` |
| Session close | Header `Status: Complete`, move to `2_Log/Archive/` under the same file name, update the registry, recompute relative links inside the log | `elf session close` |
| Deprecate | Move a whole document, a trial, a section, a marked block, or a line range to `Deprecated/` (a one-line move marker stays at the original position) | `elf deprecate` |
| Restore | Return a deprecated document or block to its original position | `elf deprecate --restore` |

- Summary documents in `Wiki/` include path links to the original logs (`Archive/...`) — when past data is needed, the original is read through that link.
- Planning documents (`1_Concept/12_Planning/`) follow the same rule: root = in progress, `Wiki/` = conclusion summaries, `Archive/` = closed, `Deprecated/` = withdrawn (whole file, marked block, or line range — planning documents have no trial or section units).

---

## 3. Scripts folder management

The same rule applies to scripts inside `61_Sim/Scripts` and `63_Analysis/Scripts`.

1.  **Active Scripts**: keep the latest scripts under active development or general use at the root of the Scripts folder.
2.  **Archived Scripts**: move scripts used one-off for a specific past session under `Scripts/Archive/` (not deleted — needed to reproduce the session).
3.  **Wiki Tracking**: when moving a script to Archive, record in a `Wiki/` document — **with the path** — which session used it and for what.

## 4. Reading rules in short

| Document location | Used as a basis for current decisions | Note |
|----------|----------------------|------|
| Root, `Wiki/` | Yes | Current state, confirmed conclusions |
| `Archive/` | Yes (closed, still-valid records) | Tracing conclusions and paths, checking past parameters |
| `Deprecated/` | No | Read to learn what was tried and withdrawn. Restoring is the user's decision |

- When the user names a path — "open this file and summarize it" — the AI reads that file regardless of its location.
- To use withdrawn content again, restore it with `elf deprecate --restore` first (do not copy blocks out of the deprecated state).
