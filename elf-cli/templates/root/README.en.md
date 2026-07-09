# PLACEHOLDER_PROJECT_NAME

## Project Overview
- **Research goal:** [write the goal and hypothesis here]
- **Research period:** PLACEHOLDER_DATE ~
- **Researcher:** [name]

## Hardware & Software Baseline
- **HW Version:** [state the baseline of the instrument/circuit/firmware]
- **SW Version:** [state the analysis scripts and tool environment]

## Data Pipeline Specification (Protocol)
This project follows the Agile R&D Boilerplate spec (ELF). (ELF version: see `.elf/version`.)
- **rawData:** original sensor data storage (read-only)
- **metaData:** Session-Trial based base-delta Markdown logging
- **scripts:** post-processing scripts for partial (cell-mode) execution

## Project Rules
For rules and goals specific to this project, see `0_Meta/ProjectRule.md`.
For folder structure and detailed operating rules, see `.elf/managed/EliRule.md`.
For AI-agent logging rules, see `.elf/managed/LogConvention.md`.
For AI context-management rules, see `.elf/managed/AI_PARA_Framework.md`.
For data-file (vocabulary/domain) customization, use a `0_Meta/<name>.project.md` overlay (EliRule §2.7).
For the current location and ownership of the canonical rules, see `AGENTS.md` (digest).
