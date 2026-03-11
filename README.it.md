[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Protocollo Base-Delta per R&S Agile

Uno standard integrato di registrazione hardware-software-dati sperimentali (Protocollo) progettato per supportare cicli di feedback rapidi (Agile) durante le fasi di sviluppo dei dispositivi e validazione R&S. Garantisce la completa tracciabilità dei dati riducendo al minimo l'affaticamento documentale del ricercatore.

## Filosofia Fondamentale

* **Singola Fonte della Verità:** Hardware, codice di analisi e dati grezzi sono organicamente connessi all'interno di un singolo progetto.
* **Registrazione Base-Delta:** Non ogni variabile viene registrata. Una Linea di Base viene dichiarata, e solo le variabili modificate (Delta) vengono registrate leggeramente per prevenire ritardi nella ricerca.
* **Applicazione Sistematica:** Bypassa le limitazioni sulla lunghezza dei nomi di file (limite Windows di 260 caratteri) e garantisce la riproducibilità attraverso il codice.
* **Governance AI:** Assicura la continuità del lavoro degli agenti AI tramite il log di handoff `0_Meta/AI_Sync.md`, e applica uno standard di registrazione unificato sia per umani che per AI tramite `0_Meta/LogConvention.md`.

## Struttura della Directory del Progetto

Questo progetto tratta la gerarchia delle cartelle stessa come uno standard di comunicazione.

```text
Project_Root/
├── 0_Meta/                          # Governance e regole del progetto
│   ├── EliRule.md                   # Guida della struttura delle cartelle e operativa
│   ├── LogConvention.md             # Regole dello standard di registrazione
│   ├── AI_PARA_Framework.md         # Regole di gestione e archiviazione del contesto AI
│   └── AI_Sync.md                   # Log di handoff degli agenti AI
│
├── 1_Concept/                       # Pianificazione della ricerca, letteratura, idee
│   ├── 11_Ideas/                    # Schizzi grezzi, proposte di ipotesi
│   ├── 12_Literature/               # PDF di articoli, informazioni bibliografiche, formule base
│   └── 13_Planning/                 # Roadmap di ricerca, storyboard di composizione delle figure
│
├── 2_HW/                            # Progettazione hardware
│   ├── 21_Component/                # Specifiche dei singoli componenti, progettazione dei dispositivi unitari
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Progettazione integrata dei dispositivi, alloggiamento, modelli 3D
│   └── 23_Elec/                     # Schemi PCB, Gerber, BOM, Datasheet
│
├── 3_Fab/                           # Fabbricazione e lavorazione
│   ├── 31_Recipes/                  # Documentazione delle condizioni di processo
│   └── 32_Eval/                     # Valutazione della singola caratteristica per modulo
│
├── 4_SW/                            # Software e firmware
│   ├── 41_FW/                       # Firmware MCU/embedded
│   ├── 42_DAQ/                      # Sistemi di acquisizione dati PC/mobile
│   └── 43_Libs/                     # Librerie condivise riutilizzabili
│
├── 5_Exp/                           # Esperimenti: simulazione + empirico + analisi
│   ├── 51_Sim/                      # Simulazione
│   │   ├── Scripts/                 # Codice di simulazione (S###_sim.m)
│   │   └── Data/                    # Risultati di simulazione (Data/S###/)
│   ├── 52_Empirical/                # Dati empirici
│   │   ├── Raw/                     # Dati grezzi dei sensori (Sola Lettura, esclusi da Git)
│   │   └── Processed/               # Dati primari processati
│   ├── 53_Analysis/                 # Analisi integrata
│   │   ├── Scripts/                 # Codice post-processing di confronto/validazione
│   │   └── Logs/                    # Log di sessione (S###_log.md)
│   └── 54_Viz/                      # Output di visualizzazione (figure auto-generate)
│
└── 6_Paper/                         # Articoli e presentazioni
    ├── 61_Figs/                     # Figure per articoli
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manoscritti (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Materiali di presentazione (PPT, poster)
```

> Per informazioni dettagliate sull'uso e le regole operative di ogni cartella, consultare `0_Meta/EliRule.md`.

## Specifica della Pipeline di Registrazione Dati

### 1. Convenzione di Denominazione dei File (Denominazione Sessione-Prova)

* Elencare le condizioni sperimentali o le informazioni variabili nei nomi dei file è **rigorosamente vietato**.
* **Formato:** `[SessionID]_[TrialID].[extension]` (es. `S001_t1.csv`, `S001_t2.bin`)

### 2. Registrazione Base-Delta (Registrazione Ibrida)

* **Log in Esecuzione (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Un file markdown narrativo che registra i cicli immediati ipotesi-test-lezione in testo.
  * Scritto per prova (`t1`, `t2`...) in uno stile stream-of-consciousness, registrando solo le **variabili intenzionalmente modificate (Delta)** e i risultati osservati.
  * Formato e regole dettagliate: consultare `0_Meta/LogConvention.md`.

### 3. Regole dei Documenti di Pianificazione

* Roadmap di ricerca, composizioni di figure, strategie sperimentali, ecc. sono gestiti separatamente in `1_Concept/13_Planning/`.
* **Formato:** `P###_title.md` (es. `P001_wavelength_optimization.md`)
* Quando si fa riferimento a Pianificazione da un log: `→ consultare 1_Concept/13_Planning/P###_xxx.md`

### 4. Specifica dell'Analisi Post-Processing (Scripting Modalità Cella)

* Il codice di analisi deve trovarsi in `5_Exp/53_Analysis/Scripts/` o `5_Exp/51_Sim/Scripts/` e non deve essere mescolato all'interno delle cartelle dati.
* File `.m` puri vengono utilizzati invece di `.mlx` per prevenire il lock-in del fornitore.
* Il codice viene eseguito sezione per sezione utilizzando `%%` (Modalità Cella), e le intuizioni derivate vengono riflesse nel log in esecuzione.
* Gli output dell'analisi (figure, file mat) vengono salvati in `5_Exp/54_Viz/` o `5_Exp/52_Empirical/Processed/S###/` all'interno di cartelle per sessione.

### 5. Regole dei Riferimenti Incrociati

I formati di riferimento incrociato sono unificati per garantire la tracciabilità tra i documenti del progetto.

| Da → A | Formato |
|-----------|--------|
| Log → Pianificazione | `→ consultare 1_Concept/13_Planning/P###_xxx.md` |
| Log → Dati Sim | `→ consultare 5_Exp/51_Sim/Data/S###/` |
| Log → Script | `→ consultare 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Pianificazione → Log | `← tracciato in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Governance AI

Quando gli agenti AI (Claude, Gemini, ecc.) partecipano al progetto, si applicano le seguenti regole:

1. **Acquisizione del Contesto:** Prima di iniziare il lavoro, leggere `0_Meta/AI_Sync.md` per confermare lo stato del lavoro precedente.
2. **Conformità agli Standard Unificati:** Seguire le regole di registrazione in `0_Meta/LogConvention.md` allo stesso modo di un ricercatore umano.
3. **Registrazione dell'Handoff:** Dopo il completamento dell'attività, registrare le azioni eseguite, i file creati/modificati e i Prossimi Passi in `0_Meta/AI_Sync.md`. Scrivere in ordine cronologico inverso con la voce più recente in alto.
4. **Separazione delle Idee:** Le ipotesi e le idee generate dall'AI vengono archiviate separatamente in `1_Concept/11_Ideas/`, non nei log.
5. **Gestione del Contesto Basata su PARA:** Utilizzare la cartella `9_Archive/` e `.claudeignore` per prevenire la contaminazione del contesto AI. Per regole dettagliate, consultare `0_Meta/AI_PARA_Framework.md`.
6. **Regole di Comunicazione:** Mantenere uno stile di scrittura oggettivo e asciutto. Nessuna analogia o metafora. Fornire le conclusioni in modo chiaro e diretto. Nessuna esagerazione o modificatore emotivo. Per regole dettagliate, consultare la sezione 3 di `0_Meta/EliRule.md`.
7. **Riutilizzabilità dei Dati:** Quando si genera un qualsiasi Grafico/Immagine, salvare l'Array di Dati originale accanto come `.mat`/`.csv`. Per regole dettagliate, consultare la sezione 2.6 di `0_Meta/EliRule.md`.

## Avvio Rapido

Per creare un nuovo progetto con la struttura ELF v2, eseguire `0_Meta/ELF_generator.bat`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.bat
```

Inserire un nome di progetto e la struttura delle cartelle 0-6, i documenti meta, `.gitignore` e l'inizializzazione di Git verranno completati automaticamente.

## Licenza

Questo progetto applica una politica di Licenza Doppia perché la natura del "codice eseguibile" e della "specifica della struttura dati (Protocollo)" differisce.

* **Software e Script:** [Licenza Pubblica Mozilla 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Si applica a:** Tutto il codice sorgente (`.m`, `.py`, ecc.) all'interno delle cartelle `4_SW/` e `5_Exp/*/Scripts/`.
  * **Condizione:** Se gli script core del template vengono modificati e migliorati per la ridistribuzione, queste modifiche devono essere rilasciate come open source. Tuttavia, algoritmi unici o dati grezzi aggiunti dall'utente all'interno del progetto possono rimanere privati (commercializzati).

* **Protocollo e Documentazione:** [Creative Commons Attribuzione 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Si applica a:** `README.md`, documenti `0_Meta/`, la gerarchia delle cartelle Sessione-Prova, le regole di registrazione dei metadati Base-Delta e la metodologia di ricerca complessiva.
  * **Condizione:** Chiunque può adottare e adattare liberamente questa struttura e metodologia di registrazione, ma quando si pubblicano template derivati o output di ricerca correlati, l'autore originale Eli (projectschnee@gmail.com) e il repository di origine devono essere accreditati.
