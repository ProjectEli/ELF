# PLACEHOLDER_PROJECT_NAME

A question archive project created with the ELF **QA preset**. (created: PLACEHOLDER_DATE)

Q&A is captured, classified, archived and recalled as **bundles** (units of meaning). The canonical operational rules are the root `AGENTS.md` (`CLAUDE.md` is the pointer that makes Claude Code read it), this project's own rules are in `0_Meta/ProjectRule.md`, and the canonical format is `.elf/managed/templates/bundle_template.md`.

## Structure
- `<category>/` — active bundles (`YYYYMMDD<L>_<Subject>.md`), `<category>/archive/` — withdrawn or replaced bundles
- `AGENTS.md` — canonical operational rules (root, managed by ELF), `CLAUDE.md` — pointer for Claude Code (managed by ELF)
- `0_Meta/ProjectRule.md` — this project's own rules and egress relaxations (user-owned, never touched by `elf update`)
- `.elf/managed/templates/bundle_template.md` — canonical bundle format
- `.elf/` — ELF control area (version, manifest; `elf update` refreshes the rules). Do not edit directly.

## Use
1. When a question comes up, write a bundle in the matching category (copy the template, fill in the frontmatter and the 5 sections).
2. When the thinking continues, link bundles with a `thread:` slug and add each question to the ledger (append-only).
3. To withdraw a bundle, move it to `archive/`.

> Experimental preset, not stable. For a research project use `elf init <name> --preset full` (the default).
