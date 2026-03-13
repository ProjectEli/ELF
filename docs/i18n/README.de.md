[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Base-Delta-Protokoll für agile F&E

Ein integriertes Aufzeichnungsprotokoll für Hardware-Software-Experimentdaten, entwickelt zur Unterstützung schneller Feedbackschleifen (Agile) bei der Geräteentwicklung und F&E-Validierung. Minimiert den Dokumentationsaufwand für Forscher und gewährleistet gleichzeitig vollständige Datennachverfolgbarkeit.

## Kernphilosophie

* **Single Source of Truth:** Verbindet Hardware-Design, Analysecode und Rohdaten organisch innerhalb eines einzigen Projekts.
* **Base-Delta-Protokollierung:** Zeichnet nicht jede Variable auf. Deklariert eine Baseline und erfasst leichtgewichtig nur veränderte Variablen (Delta), um Forschungsverzögerungen zu vermeiden.
* **Systematische Durchsetzung:** Umgeht Dateinamenlängenbeschränkungen (Windows 260-Zeichen) und gewährleistet Reproduzierbarkeit durch Code.
* **KI-Governance:** Sichert die Arbeitskontinuität von KI-Agenten durch `0_Meta/AI_Sync.md`-Übergabelogs und erzwingt identische Protokollierungsstandards für Menschen und KI via `0_Meta/LogConvention.md`.

## Verzeichnisstruktur

Dieses Projekt behandelt die Ordnerhierarchie selbst als Kommunikationsprotokoll.

```text
Project_Root/
├── 0_Meta/                          # Projekt-Governance & Regeln
│   ├── EliRule.md                   # Ordnerstruktur & Betriebsanleitung
│   ├── LogConvention.md             # Protokollierungsstandard-Regeln
│   ├── AI_PARA_Framework.md         # KI-Kontextverwaltung & Archivierungsregeln
│   └── AI_Sync.md                   # KI-Agenten-Übergabelog
│
├── 1_Concept/                       # Forschungsplanung, Literatur, Ideen
│   ├── 11_Ideas/                    # Grobe Skizzen, Hypothesenvorschläge
│   ├── 12_Literature/               # Paper-PDFs, Literaturangaben, Formeln
│   └── 13_Planning/                 # Forschungsroadmap, Abbildungs-Storyboards
│       └── 2_Wiki/                  # Destillierte Planungsschlüsse & Schlüsselregeln
│
├── 2_HW/                            # Hardware-Design
│   ├── 21_Component/                # Bauteilspezifikationen, Einzelgeräte-Design
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Integriertes Geräte-Design, Gehäuse, 3D-Modelle
│   └── 23_Elec/                     # PCB-Schaltpläne, Gerber, BOM, Datenblätter
│
├── 3_Fab/                           # Fertigung & Prozesse
│   ├── 31_Recipes/                  # Prozessbedingungsdokumentation
│   └── 32_Eval/                     # Modulweise Charakterisierung
│
├── 4_SW/                            # Software & Firmware
│   ├── 41_FW/                       # MCU/Embedded-Firmware
│   ├── 42_DAQ/                      # PC/Mobile-Datenerfassungssystem
│   └── 43_Libs/                     # Wiederverwendbare gemeinsame Bibliotheken
│
├── 5_Exp/                           # Experimente: Simulation + Empirie + Analyse
│   ├── 51_Sim/                      # Simulation
│   │   ├── Scripts/                 # Simulationscode (S###_sim.m)
│   │   │   └── 9_Archive/          # Ausgemusterte Skripte
│   │   └── Data/                    # Simulationsergebnisse (Data/S###/)
│   ├── 52_Empirical/                # Empirische Daten
│   │   ├── Raw/                     # Originale Sensordaten (schreibgeschützt, Git-ausgeschlossen)
│   │   └── Processed/               # Vorverarbeitete Daten
│   ├── 53_Analysis/                 # Integrierte Analyse
│   │   ├── Scripts/                 # Vergleichs-/Validierungs-Nachverarbeitungscode
│   │   │   └── 9_Archive/          # Ausgemusterte Skripte
│   │   └── Logs/                    # Sitzungslogs (S###_log.md)
│   │       ├── 2_Wiki/              # Destillierte Erkenntnisse & Sitzungsregister
│   │       └── 9_Archive/           # Abgeschlossene Sitzungslogs
│   └── 54_Viz/                      # Visualisierungsexporte (automatisch generierte Abbildungen)
│
└── 6_Paper/                         # Publikationen & Präsentationen
    ├── 61_Figs/                     # Abbildungen für Publikationen
    │   ├── Raw/
    │   ├── Processed/
    │   └── Final/
    ├── 62_Drafts/                   # Manuskripte (Word, LaTeX)
    │   └── 9_Archive/                # Frühere Versionen
    └── 63_Presentations/            # Präsentationsmaterialien (PPT, Poster)
```

> Detaillierte Ordnernutzung und Betriebsregeln: siehe `0_Meta/EliRule.md`.

## Datenprotokollierungs-Pipeline

### 1. Dateibenennungskonvention (Sitzungs-Versuchs-Benennung)

* Das Auflisten von Versuchsbedingungen oder Variableninformationen in Dateinamen ist **streng untersagt**.
* **Format:** `[SessionID]_[TrialID].[ext]` (z.B. `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta-Protokollierung (Hybridprotokollierung)

* **Laufendes Log (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Eine narrative Markdown-Datei zur Aufzeichnung unmittelbarer Hypothesen-Test-Erkenntniszyklen.
  * Wird im Bewusstseinsstrom pro Versuch (`t1`, `t2`...) geschrieben und erfasst nur **absichtlich geänderte Variablen (Delta)** und beobachtete Ergebnisse.
  * Zum Format und zu detaillierten Regeln: siehe `0_Meta/LogConvention.md`.

### 3. Planungsdokument-Regeln

* Forschungsroadmaps, Abbildungszusammenstellungen und Experimentstrategien werden separat in `1_Concept/13_Planning/` verwaltet.
* **Format:** `P###_titel.md` (z.B. `P001_wavelength_optimization.md`)
* Querverweise aus Logs: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Nachverarbeitungsanalyse (Cell-Mode-Skripting)

* Analysecode muss in `5_Exp/53_Analysis/Scripts/` oder `5_Exp/51_Sim/Scripts/` liegen; das Mischen von Code innerhalb von Datenordnern ist untersagt.
* Verwende einfache `.m`-Dateien statt `.mlx`, um Anbieterabhängigkeit zu vermeiden.
* Verwende `%%` (Cell Mode) für abschnittsweise Ausführung; erfasse abgeleitete Erkenntnisse im laufenden Log.
* Analyseausgaben (Abbildungen, mat-Dateien) werden in `5_Exp/54_Viz/` oder `5_Exp/52_Empirical/Processed/S###/` in sitzungsbezogenen Ordnern gespeichert.

### 5. Querverweisregeln

Einheitliches Querverweisformat für Dokumentnachverfolgbarkeit innerhalb des Projekts.

| Von → Nach | Format |
|------------|--------|
| Logs → Planung | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Sim-Daten | `→ see 5_Exp/51_Sim/Data/S###/` |
| Logs → Skript | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planung → Logs | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## KI-Governance

Regeln für KI-Agenten (Claude, Gemini usw.), die am Projekt teilnehmen:

1. **Kontextbewusstsein**: `0_Meta/AI_Sync.md` vor Arbeitsbeginn lesen, um den vorherigen Arbeitszustand zu verstehen.
2. **Standardkonformität**: `0_Meta/LogConvention.md`-Protokollierungsregeln identisch wie menschliche Forscher befolgen.
3. **Übergabeaufzeichnung**: Nach Arbeitsabschluss durchgeführte Aufgaben, erstellte/geänderte Dateien und nächste Schritte in `0_Meta/AI_Sync.md` erfassen. In umgekehrter chronologischer Reihenfolge schreiben (neuestes zuerst).
4. **Ideentrennung**: KI-generierte Hypothesen/Ideen gehen nach `1_Concept/11_Ideas/`, nicht in Logs.
5. **PARA-basiertes Kontextmanagement**: `9_Archive/`-Ordner und `.claudeignore` verwenden, um KI-Kontextkontaminierung zu verhindern. Details siehe `0_Meta/AI_PARA_Framework.md`.
6. **Kommunikationsregeln**: Objektiven, sachlichen Ton wahren. Keine Analogien/Metaphern. Klare, schlussfolgerungsorientierte Kommunikation. Keine übertriebenen oder emotionalen Ausdrücke. Siehe `0_Meta/EliRule.md` Abschnitt 3.
7. **Datenwiederverwertbarkeit**: Rohdaten-Arrays immer als `.mat`/`.csv` zusammen mit jedem Plot/Diagramm exportieren. Siehe `0_Meta/EliRule.md` Abschnitt 2.6.

## Schnellstart

Um ein neues Projekt mit der ELF-v2-Struktur zu erstellen, führe `0_Meta/ELF_generator.sh` aus.

```bash
cd your_target_directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> Unter Windows verwende Git Bash (enthalten in [Git for Windows](https://git-scm.com/)).

Gib einen Projektnamen ein, und die 0~6-Ordnerhierarchie, Meta-Dokumente und `.gitignore` werden automatisch erstellt. Die Git-Initialisierung ist optional und wird nur abgefragt, wenn Git verfügbar ist.

## Lizenz

Dieses Projekt wendet eine duale Lizenzpolitik an, da 'ausführbarer Code' und 'Datenstrukturprotokoll' unterschiedlichen Zwecken dienen.

* **Software & Skripte:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Geltungsbereich:** Gesamter Quellcode (`.m`, `.py` usw.) in `4_SW/` und `5_Exp/*/Scripts/`.
  * **Bedingungen:** Geänderte Kern-Template-Skripte müssen als Open Source veröffentlicht werden. Proprietäre Algorithmen und vom Nutzer hinzugefügte Rohdaten dürfen jedoch privat bleiben (kommerzialisierbar).

* **Protokoll & Dokumentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Geltungsbereich:** `README.md`, `0_Meta/`-Dokumente, Sitzungs-Versuchs-Ordnerhierarchie, Base-Delta-Metadaten-Protokollierungsregeln und Forschungsmethodik im Allgemeinen.
  * **Bedingungen:** Jeder darf diese Struktur und Methodik frei übernehmen und modifizieren, muss jedoch den ursprünglichen Autor Eli (projectschnee@gmail.com) und dieses Repository nennen, wenn abgeleitete Templates oder verwandte Forschungsergebnisse veröffentlicht werden.
