[English](deprecate.md) | [한국어](deprecate.ko.md)

# Deprecating content

## `elf deprecate [target] [unit] [options]`, `show <ID>`, `--restore <ID>`

Deprecation removes content that is no longer valid from the working tree, so an AI agent does not read it and reason from it. The original is kept compressed in `.elf/deprecated/<ID>.md.xz` (standard xz, tracked by git), which plain reads and searches do not show; a neutral marker line stays at the original position. The user decides what to deprecate; the agent runs the command after an instruction or approval and shows a stored original only when the user asks (`AGENTS.md`). Outputs (scripts, data, figures) stay where they are.

| Target, unit | Stays in the document | Goes to the store |
|---|---|---|
| `S###` / `G###` (whole session log) | The 7-key header and the title line. `status: deprecated`, the handoff holds the reason and the replacement, `conclusion` and `user_note` are emptied | The body and the original status, handoff, conclusion, user_note |
| `P###` or a project-relative path (whole planning document) | The title line and a marker line | The whole text |
| `--trial t03` | `## t03: (폐기 ID)` (number kept) and a marker line | The trial heading and body |
| `--trial t03 --section 해석` | A marker line | The section including its heading |
| `--marked` | A marker line | The block wrapped in `<!-- deprecate:begin -->` … `<!-- deprecate:end -->` |
| `--lines A-B --expect "<text of line A>"` | A marker line | A line range (1-based). Refused if line A differs |

Marker line: `> **Deprecated**: S012-D01 · 2026-11-09 · t03 · 사유: wrong condition · 대체: S015 t02`. The ID is a per-document sequence (`S012-D01`, `P007-D02`). `--reason` and `--replaced-by` fill the last two fields.

| Command | Does |
|---|---|
| `elf deprecate` (no arguments) | List: one line per stored item (ID, date, unit, source, reason, replacement) and a total |
| `elf deprecate show <ID>` | Print the stored original (only when the user asks) |
| `elf deprecate --restore <ID>` | Put the original back in place (a whole session gets its original header values) and delete the store file. Links in the restored content that point to a missing file are printed as warnings |
| `--dry-run` | Print only, write nothing |

- Commit the document change and its store file together. `elf check logs` pairs them: a marker without a store file is an issue, a store file without a marker is a warning, an untracked store file is info.
- Refused (exit 3, nothing written): a path outside the scope (session logs and planning documents only), an already-deprecated document, a block containing a trial heading or the title, unbalanced markers, an `--expect` mismatch, a change that would introduce new trial-structure findings in a session log (for example the `가설` section alone while `관찰` remains), a project still in the earlier log layout. A missing target or ID exits 1, a usage error exits 2.
- Output: items the command does not modify (the handoff of the source log, references such as `S012 t03` in other documents, figure embeds inside the removed content) are printed as `review:` lines, to be resolved with the user.
- Before 2.28, deprecated content was moved to a `Deprecated/` folder (`*.partial.md` files) and could be purged. `elf update` converts those records into store files and marker lines once ([migration](projects.md#migrating-the-log-layout-228)); records purged before then keep a marker that ends with `복원 불가` and have no store file.

```bash
elf deprecate S012 --trial t03 --replaced-by "S015 t02" --reason "wrong measurement condition"
# → deprecated S012-D01 (t03) → .elf/deprecated/S012-D01.md.xz
# → review: handoff: 2_Log/S/S012_log.md header mentions t03
elf deprecate S014 --reason "wrong setup"          # whole session: header status deprecated
elf deprecate P007 --marked --reason "scope cut"   # marked block in a planning document
elf deprecate show S012-D01                        # only when the user asks
elf deprecate --restore S012-D01                   # restore the trial
elf deprecate                                      # list
```

---

Back to [README](../README.md)
