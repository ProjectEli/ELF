[English](examples.md) | [한국어](examples.ko.md)

# Starting examples

These examples show, step by step, how to start when you already have an idea or existing work. Both are the same study started at different points: A compares SNR by wavelength in simulation, B with data measured by a prototype optical sensor.

## A. You have an idea

| Step | Your input | Action | What you do |
|---|---|---|---|
| Idea | "I need to sort out my thinking on the wavelength choice for a wearable optical sensor. Current view: taking skin penetration and hemoglobin absorption together, around 810 nm looks favorable. Note the evidence and the counter-cases." | Writes `1_Concept/13_Ideas/wavelength_choice.md` (not in the log) | Judge the direction |
| Plan | "Write a concrete plan to test this as a plan document: compare SNR by wavelength in simulation first, then confirm with measurements from the prototype sensor." | Writes `1_Concept/12_Planning/P001_wavelength_optimization.md` | Approve the steps |
| Session | "Start with step 1 of P001. S001 goal: SNR by wavelength, 735/810/940 nm, Monte Carlo simulation." | Creates S001 with the goal (`Registry.md` updated); t01's background names P001 | Confirm the goal |
| t01 | "t01: 3-wavelength sweep. Baseline conditions as written in P001: 3-layer skin model, 1e8 photons." | Writes phase 1. After approval, runs in phase 2 and records the observation, interpretation, and lesson. Script `61_Sim/Scripts/S001_t01_*.m`, data `61_Sim/Data/S001/` | Review the hypothesis and prediction, then approve |

Reference papers go in `1_Concept/11_Literature/`. The rules include a source-reliability rule and a journal domain list.

## B. You have existing work

| Step | Your input | Action | What you do |
|---|---|---|---|
| Data in | (by hand) Copy the measured raw CSV into `6_Exp/62_Empirical/Raw/` | None. Originals stay read-only and out of git; recording starts at t01 | Check the originals |
| Session | "I have CSVs of 3-wavelength reflectance measured with the prototype optical sensor. S001 goal: SNR by wavelength from this data." | Creates S001 with the goal (`Registry.md` updated) | Confirm the goal |
| t01 cleaning | "t01: data overview and cleaning. Drop the saturated stretches and the motion-noise stretches." | Saves the cleaned data in `62_Empirical/Processed/S001/`. Script `63_Analysis/Scripts/S001_t01_*.m`; range and filters recorded in Conditions | Approve the cleaning criteria |
| t02 analysis | "t02: SNR by wavelength on the cleaned data, with the same definition as the simulation." | Writes phase 1. After approval, runs in phase 2 and records the figure (`64_Viz/S001/`), data (`.mat`, `.csv`), interpretation, and lesson | Review the hypothesis and prediction, then approve |

Closing a session and checking logs are covered in [Sessions and trials](sessions.md) and [Checks](check.md).
