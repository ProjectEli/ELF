# ProjectRule: [Project Name]

Building on the global rules (`0_Meta/EliRule.md`), this document defines the context,
environment, and overrides that apply only to this project.
The AI agent reads this together with EliRule.md to understand the project context.

---

## 1. Project Overview

| Item | Content |
|------|------|
| **Research goal** | [core goal in 1-2 sentences] |
| **Research period** | YYYY-MM-DD ~ |
| **Researcher** | [name] |
| **Current stage** | [e.g., early exploration / parameter optimization / paper writing] |

---

## 2. Core Domain & Terminology

| Term / Abbreviation | Definition |
|-------------|------|
| [abbrev] | [definition] |

---

## 3. Experimental Environment (Baseline)

### Hardware
- **Measurement instrument**: [model, range/precision]
- **Sensor / DUT**: [model, spec]

### Software
- **Analysis tool**: [e.g., MATLAB R2024b + Signal Processing Toolbox]
- **Other SW**: [state versions]

### Operating conditions
- [e.g., room temp 23±2°C, sampling rate 10 kHz]

---

## 4. Data Pipeline Notes

> Record only what differs from the EliRule.md default pipeline.

- **Raw data format**: [e.g., `.csv`, 2 header rows, column order: time / ch1 / ch2]
- **Standard preprocessing**: [e.g., remove DC offset → 50 Hz notch filter]
- **Unit basis**: [e.g., voltage mV, pressure Pa, time ms]

---

## 5. Naming Overrides

> Record only what differs from the EliRule.md defaults. If none, delete this section.

- (none)

---

## 6. Folder Usage Plan

| Group | Folder | In use | Note |
|------|------|-----------|------|
| Core | `0_Meta/` | Yes | governance |
| Core | `1_Concept/` | Yes | |
| Core | `2_Log/` | Yes | session logs |
| Module | `3_HW/` | Yes | |
| Module | `4_Fab/` | No | |
| Module | `5_SW/` | Yes | |
| Module | `6_Exp/` | Yes | |
| Module | `7_Paper/` | Planned | |

---

## 7. Progress Status *(update as needed)*

- **Current active session**: S001
- **Completed milestones**: (none)
- **Next goal**: [first experiment goal]
- **Current open issues**: (none)

---

## 8. Additional AI Agent Instructions

> In addition to EliRule.md Section 3, instructions specific to this project.

- (none)
