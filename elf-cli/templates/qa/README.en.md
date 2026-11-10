# PLACEHOLDER_PROJECT_NAME

A question archive project created with the ELF **QA preset**. (created: PLACEHOLDER_DATE)

Q&A is captured, classified, archived and recalled as **bundles** (units of meaning). The canonical operational rules are the root `AGENTS.md`, this project's own rules are in `0_Meta/ProjectRule.md`, and the canonical format is `.elf/managed/templates/bundle_template.md`.

## Structure
- `<category>/` — bundles in progress (`YYYYMMDD<L>_<Subject>.md`), `<category>/archive/` — finished bundles and withdrawn ones (`status: deprecated`)
- `<category>/README.md` — rules of that category (when present), subfolders starting with `_` — tools or work area
- `0_Meta/templates/<category>.md` — the user template of that category (when present, user-owned)
- `AGENTS.md` — canonical operational rules (root, managed by ELF)
- `0_Meta/ProjectRule.md` — this project's own rules and egress relaxations (user-owned, never touched by `elf update`)
- `.elf/managed/templates/bundle_template.md` — canonical bundle format
- `.elf/` — ELF control area (version, manifest; `elf update` refreshes the rules). Do not edit directly.

## Use
1. When a question comes up, write a bundle in the matching category (copy the template, fill in the frontmatter and the 5 sections).
2. When the thinking continues, link bundles with a `thread:` slug and add each question to the ledger (append-only).
3. When a bundle has nothing more to add, move it to `archive/` (kept and recalled). A bundle that was wrong or is replaced gets `status: deprecated`.
4. `elf check logs` checks the frontmatter, locations, names and related-bundle paths (and fixes paths to bundles moved to or from `archive/`).

> Experimental preset, not stable. For a research project use `elf init <name> --preset full` (the default).
