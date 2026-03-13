[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Base-Delta Protocol voor Agile O&O

Een geïntegreerde hardware-software-experimentele data logging standaard (Protocol) ontworpen ter ondersteuning van snelle feedbacklusses (Agile) tijdens apparatuuratwikkeling en O&O-validatiefasen. Garandeert volledige gegevenstraceerbaarheid terwijl onderzoeksdocumentatie-inspanning wordt geminimaliseerd.

## Kernfilosofie

* **Single Source of Truth (Enkele Bron van Waarheid):** Hardware-ontwerp, analyticacode en onbewerkte gegevens zijn organisch verbonden binnen een enkel project.
* **Base-Delta Logging:** Niet elke variabele wordt opgenomen. Een Baseline wordt verklaard, en alleen gewijzigde variabelen (Deltas) worden licht vastgelegd om onderzoeksvertragingen te voorkomen.
* **Systematic Enforcement (Systematische Handhaving):** Omzeilt lengtebeperkingen voor bestandsnamen (Windows 260-tekenlimiet) en garandeert reproduceerbaarheid via code.
* **AI Governance:** Zorgt voor continuïteit van AI-agentwerk via het handoff-logboek `0_Meta/AI_Sync.md` en dwingt een uniforme logging-standaard af voor zowel mensen als AI via `0_Meta/LogConvention.md`.

## Projectmappenstructuur

Dit project behandelt de mappenstructuur zelf als een communicatiestandaard.

```text
Project_Root/
├── 0_Meta/                          # Projectbeheer & regels
│   ├── EliRule.md                   # Mappenstructuur en bedieningsgids
│   ├── LogConvention.md             # Logging-standaardregels
│   ├── AI_PARA_Framework.md         # AI-contextbeheer & archiveringsregels
│   └── AI_Sync.md                   # AI-agenthandoff-logboek
│
├── 1_Concept/                       # Onderzoeksplanning, literatuur, ideeën
│   ├── 11_Ideas/                    # Ruwe schetsen, hypothesestelling
│   ├── 12_Literature/               # PDF's van artikelen, bibliografische info, basisformules
│   └── 13_Planning/                 # Onderzoeksplattegronden, figuurcompositiestoryboards
│
├── 2_HW/                            # Hardware-ontwerp
│   ├── 21_Component/                # Individuele componentspecs, eenheid-apparaatontwerp
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Geïntegreerd apparaatontwerp, behuizing, 3D-modellen
│   └── 23_Elec/                     # PCB-schema's, Gerber, BOM, Datasheets
│
├── 3_Fab/                           # Fabricage en verwerking
│   ├── 31_Recipes/                  # Procesvoorwaardengeving
│   └── 32_Eval/                     # Per-module enkele karakteristiekevaluatie
│
├── 4_SW/                            # Software & firmware
│   ├── 41_FW/                       # MCU/embedded firmware
│   ├── 42_DAQ/                      # PC/mobiele data acquisitiesystemen
│   └── 43_Libs/                     # Herbruikbare gedeelde bibliotheken
│
├── 5_Exp/                           # Experimenten: simulatie + empirisch + analyse
│   ├── 51_Sim/                      # Simulatie
│   │   ├── Scripts/                 # Simulatiecode (S###_sim.m)
│   │   └── Data/                    # Simulatieresultaten (Data/S###/)
│   ├── 52_Empirical/                # Empirische gegevens
│   │   ├── Raw/                     # Onbewerkte sensorgegevens (Alleen-lezen, uitgesloten van Git)
│   │   └── Processed/               # Primair verwerkte gegevens
│   ├── 53_Analysis/                 # Geïntegreerde analyse
│   │   ├── Scripts/                 # Vergelijkings-/validatie naverwerking code
│   │   └── Logs/                    # Sessielogboeken (S###_log.md)
│   └── 54_Viz/                      # Visualisatie-uitvoer (automatisch gegenereerde figuren)
│
└── 6_Paper/                         # Papers & presentaties
    ├── 61_Figs/                     # Figuren voor papers
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manuscripten (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Presentatiematerialen (PPT, posters)
```

> Voor gedetailleerd gebruik en operationele regels voor elke map, raadpleeg `0_Meta/EliRule.md`.

## Specifikatie van de Data Logging Pipeline

### 1. Bestandsnaamconventie (Session-Trial Naming)

* Het vermelden van experimentele voorwaarden of variabeleinformatie in bestandsnamen is **strikt verboden**.
* **Format:** `[SessionID]_[TrialID].[extension]` (bijv. `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Logging (Hybrid Logging)

* **Running Log (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Een narratief markdown-bestand dat onmiddellijke hypothese-test-les cycli in tekst vastlegt.
  * Per trial (`t1`, `t2`...) in een stream-of-consciousness stijl geschreven, waarbij alleen de **opzettelijk gewijzigde variabelen (Delta)** en waargenomen resultaten worden vastgelegd.
  * Format en gedetailleerde regels: raadpleeg `0_Meta/LogConvention.md`.

### 3. Regels voor Planningsdocumenten

* Onderzoeksplattegronden, figuurcomposities, experimentele strategieën, enz. worden apart beheerd in `1_Concept/13_Planning/`.
* **Format:** `P###_title.md` (bijv. `P001_wavelength_optimization.md`)
* Bij verwijzing naar Planning uit een logboek: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Naverwerking Analysespecificatie (Cell Mode Scripting)

* Analysecode moet zich bevinden in `5_Exp/53_Analysis/Scripts/` of `5_Exp/51_Sim/Scripts/` en mag niet gemengd zijn in gegevensmappen.
* Pure `.m` bestanden worden gebruikt in plaats van `.mlx` om vendorbinding te voorkomen.
* Code wordt sectie voor sectie uitgevoerd met `%%` (Cell Mode), en afgeleide inzichten worden weerspiegeld in het uitvoerigheidslogboek.
* Analyse-uitvoer (figuren, mat-bestanden) worden opgeslagen in `5_Exp/54_Viz/` of `5_Exp/52_Empirical/Processed/S###/` in per-sessie-mappen.

### 5. Kruisverwijzingsregels

Kruisverwijzingsformaten zijn geünificeerd om traceerbaarheid tussen projectdocumenten te garanderen.

| Van → Naar | Format |
|-----------|--------|
| Logboeken → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logboeken → Sim Data | `→ see 5_Exp/51_Sim/Data/S###/` |
| Logboeken → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planning → Logboeken | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI Governance

Wanneer AI-agenten (Claude, Gemini, enz.) aan het project deelnemen, gelden de volgende regels:

1. **Context Acquisition (Contextverkrijging):** Lees vóór het begin van het werk `0_Meta/AI_Sync.md` om de status van vorig werk te bevestigen.
2. **Unified Standard Compliance (Uniforme Standaardnaleving):** Volg de logging-regels in `0_Meta/LogConvention.md` op dezelfde manier als een menselijk onderzoeker.
3. **Handoff Recording (Handoff Registratie):** Noteer bij taak voltooiing uitgevoerde acties, gemaakte/gewijzigde bestanden en volgende stappen in `0_Meta/AI_Sync.md`. Schrijf in omgekeerde chronologische volgorde met het meest recente item bovenaan.
4. **Idea Separation (Scheidingsideeën):** Hypothesen en ideeën gegenereerd door AI worden apart opgeslagen in `1_Concept/11_Ideas/`, niet in logboeken.
5. **PARA-Based Context Management:** Gebruik de `9_Archive/` map en `.claudeignore` om AI-contextbesmetting te voorkomen. Voor gedetailleerde regels, raadpleeg `0_Meta/AI_PARA_Framework.md`.
6. **Communication Rules (Communicatieregels):** Handhaaf een objectieve en droge schrijfstijl. Geen gelijkenissen of metaforen. Lever conclusies helder en direct. Geen overdrijving of emotionele wijzigingen. Voor gedetailleerde regels, raadpleeg sectie 3 van `0_Meta/EliRule.md`.
7. **Data Reusability (Gegevensherbru­ikbaarheid):** Bij het genereren van Plot/Graph slaat u de originele Data Array op als `.mat`/`.csv`. Voor gedetailleerde regels, raadpleeg sectie 2.6 van `0_Meta/EliRule.md`.

## Snel Starten

Om een nieuw project met de ELF v2 structuur te creëren, voer `0_Meta/ELF_generator.ps1` uit.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.ps1
```

Voer een projectnaam in en de mappenstructuur 0–6, metadocumenten, `.gitignore` en Git-initialisatie zullen allemaal automatisch worden voltooid.

## Licentie

Dit project hanteert een Dual License-beleid omdat de aard van "uitvoerbare code" en "datastructuurspecificatie (Protocol)" verschilt.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Van toepassing op:** Alle broncode (`.m`, `.py`, enz.) in de mappen `4_SW/` en `5_Exp/*/Scripts/`.
  * **Voorwaarde:** Indien kernscripts voor sjabloon worden gewijzigd en verbeterd voor herdistributie, moeten deze wijzigingen als open source worden vrijgegeven. Unieke algoritmen of door de gebruiker toegevoegde onbewerkte gegevens binnen het project kunnen echter privé blijven (gecommercialiseerd).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Van toepassing op:** `README.md`, `0_Meta/` documenten, de Session-Trial mappenstructuur, Base-Delta metagegevensvastleggingsregels en de algemene onderzoeksmethodologie.
  * **Voorwaarde:** Iedereen mag deze structuur en registratiemethodologie vrijelijk aannemen en aanpassen, maar bij het publiceren van afgeleide sjablonen of gerelateerde onderzoeksresultaten moet de originele auteur Eli (projectschnee@gmail.com) en de bronrepository worden erkend.
