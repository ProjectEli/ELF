[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Protokół Base-Delta dla Agile R&D

Zintegrowany standard rejestrowania danych sprzętu, oprogramowania i eksperymentów (Protokół) zaprojektowany do wspierania szybkich pętli sprzężenia zwrotnego (Agile) podczas rozwoju urządzeń i faz walidacji R&D. Gwarantuje pełną możliwość śledzenia danych, minimalizując zmęczenie pracowników naukowymi z dokumentacją.

## Filozofia Podstawowa

* **Jedno Źródło Prawdy:** Projekt sprzętowy, kod analizy i dane surowe są organicznie powiązane w obrębie jednego projektu.
* **Rejestrowanie Base-Delta:** Nie każda zmienna jest rejestrowana. Definiuje się Linię Bazową, a tylko zmienione zmienne (Deltas) są rejestrowane lekko, aby zapobiec opóźnieniom badawczym.
* **Egzekwowanie Systematyczne:** Obchodzi ograniczenia długości nazw plików (limit 260 znaków w systemie Windows) i gwarantuje powtarzalność poprzez kod.
* **Zarządzanie AI:** Zapewnia ciągłość pracy agentów AI poprzez dziennik przekazania `0_Meta/AI_Sync.md` i egzekwuje ujednolicony standard rejestrowania zarówno dla ludzi, jak i AI poprzez `0_Meta/LogConvention.md`.

## Struktura Katalogów Projektu

Ten projekt traktuje hierarchię folderów jako standard komunikacji.

```text
Project_Root/
├── 0_Meta/                          # Zarządzanie projektem i zasady
│   ├── EliRule.md                   # Przewodnik struktury folderów i operacyjny
│   ├── LogConvention.md             # Reguły standardu rejestrowania
│   ├── AI_PARA_Framework.md         # Reguły zarządzania kontekstem AI i archiwizacji
│   └── AI_Sync.md                   # Dziennik przekazania agenta AI
│
├── 1_Concept/                       # Planowanie badań, literatura, pomysły
│   ├── 11_Ideas/                    # Rough sketches, propozycje hipotez
│   ├── 12_Literature/               # PDF-y artykułów, informacje bibliograficzne, formuły bazowe
│   └── 13_Planning/                 # Plany badawcze, storyboardy kompozycji figur
│       └── 2_Wiki/                  # Destylowane wnioski planistyczne i kluczowe zasady
│
├── 2_HW/                            # Projekt sprzętu
│   ├── 21_Component/                # Specyfikacje poszczególnych komponentów, projekt urządzenia jednostkowego
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Zintegrowany projekt urządzenia, obudowa, modele 3D
│   └── 23_Elec/                     # Schematy PCB, Gerber, BOM, Karty Danych
│
├── 3_Fab/                           # Produkcja i przetwarzanie
│   ├── 31_Recipes/                  # Dokumentacja warunków procesu
│   └── 32_Eval/                     # Ocena pojedynczej cechy na moduł
│
├── 4_SW/                            # Oprogramowanie i firmware
│   ├── 41_FW/                       # Firmware MCU/osadzony
│   ├── 42_DAQ/                      # Systemy akwizycji danych PC/mobilne
│   └── 43_Libs/                     # Biblioteki współdzielone do ponownego użytku
│
├── 5_Exp/                           # Eksperymenty: symulacja + empiryczne + analiza
│   ├── 51_Sim/                      # Symulacja
│   │   ├── Scripts/                 # Kod symulacji (S###_sim.m)
│   │   │   └── 9_Archive/          # Wycofane skrypty
│   │   └── Data/                    # Wyniki symulacji (Data/S###/)
│   ├── 52_Empirical/                # Dane empiryczne
│   │   ├── Raw/                     # Surowe dane sensorowe (Tylko do odczytu, wykluczone z Git)
│   │   └── Processed/               # Dane pierwotnie przetwarzane
│   ├── 53_Analysis/                 # Zintegrowana analiza
│   │   ├── Scripts/                 # Kod porównania/walidacji przetwarzania końcowego
│   │   │   └── 9_Archive/          # Wycofane skrypty
│   │   └── Logs/                    # Dzienniki sesji (S###_log.md)
│   │       ├── 2_Wiki/              # Destylowane ustalenia i rejestr sesji
│   │       └── 9_Archive/           # Ukończone dzienniki sesji
│   └── 54_Viz/                      # Wyniki wizualizacji (figury generowane automatycznie)
│
└── 6_Paper/                         # Artykuły i prezentacje
    ├── 61_Figs/                     # Figury dla artykułów
    │   ├── Raw/
    │   ├── Processed/
    │   └── Final/
    ├── 62_Drafts/                   # Rękopisy (Word, LaTeX)
    │   └── 9_Archive/                # Poprzednie wersje
    └── 63_Presentations/            # Materiały prezentacyjne (PPT, plakaty)
```

> Aby zapoznać się z szczegółowymi instrukcjami użytkowania i regułami operacyjnymi dla każdego folderu, zapoznaj się z `0_Meta/EliRule.md`.

## Specyfikacja Potoku Rejestrowania Danych

### 1. Konwencja Nazewnictwa Plików (Nazewnictwo Sesji-Próby)

* Wymienianie warunków eksperymentalnych lub informacji o zmiennych w nazwach plików jest **ściśle zakazane**.
* **Format:** `[SessionID]_[TrialID].[extension]` (np. `S001_t1.csv`, `S001_t2.bin`)

### 2. Rejestrowanie Base-Delta (Rejestrowanie Hybrydowe)

* **Dziennik Bieżący (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Plik markdown narracyjny, który rejestruje natychmiastowe cykle hipoteza-test-lekcja w tekście.
  * Napisany dla każdej próby (`t1`, `t2`...) w stylu strumienia świadomości, rejestrując tylko **zmienione zmiany (Delta)** i zaobserwowane wyniki.
  * Format i szczegółowe reguły: zapoznaj się z `0_Meta/LogConvention.md`.

### 3. Reguły Dokumentu Planowania

* Plany badawcze, kompozycje figur, strategie eksperymentalne itp. są zarządzane oddzielnie w `1_Concept/13_Planning/`.
* **Format:** `P###_title.md` (np. `P001_wavelength_optimization.md`)
* Przy odwoływaniu się do Planowania z dziennika: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Specyfikacja Analizy Przetwarzania Końcowego (Skryptowanie Trybu Komórki)

* Kod analizy musi być umieszczony w `5_Exp/53_Analysis/Scripts/` lub `5_Exp/51_Sim/Scripts/` i nie może być mieszany wewnątrz folderów danych.
* Zamiast `.mlx` używa się czystych plików `.m` w celu uniknięcia blokady producenta.
* Kod jest wykonywany sekcja po sekcji za pomocą `%%` (Tryb Komórki), a uzyskane spostrzeżenia są odzwierciedlane w dzienniku bieżącym.
* Wyniki analizy (figury, pliki mat) są zapisywane w `5_Exp/54_Viz/` lub `5_Exp/52_Empirical/Processed/S###/` w folderach na sesję.

### 5. Reguły Odsyłaczy Krzyżowych

Formaty odsyłaczy krzyżowych są ujednolicone w celu zapewnienia możliwości śledzenia między dokumentami projektu.

| Od → Do | Format |
|-----------|--------|
| Dzienniki → Planowanie | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Dzienniki → Dane Sym | `→ see 5_Exp/51_Sim/Data/S###/` |
| Dzienniki → Skrypt | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planowanie → Dzienniki | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Zarządzanie AI

Gdy agenci AI (Claude, Gemini, itp.) uczestniczą w projekcie, obowiązują następujące reguły:

1. **Pozyskiwanie Kontekstu:** Przed rozpoczęciem pracy przeczytaj `0_Meta/AI_Sync.md` w celu potwierdzenia stanu poprzedniej pracy.
2. **Zgodność ze Standardem Ujednoliconym:** Postępuj zgodnie z regułami rejestrowania w `0_Meta/LogConvention.md` w taki sam sposób, jak badacz ludzi.
3. **Rejestrowanie Przekazania:** Po ukończeniu zadania zapisz wykonane działania, utworzone/zmodyfikowane pliki i Następne Kroki w `0_Meta/AI_Sync.md`. Pisz w odwrotnej kolejności chronologicznej z najnowszym wpisem u góry.
4. **Separacja Pomysłów:** Hipotezy i pomysły generowane przez AI są przechowywane oddzielnie w `1_Concept/11_Ideas/`, a nie w dziennikach.
5. **Zarządzanie Kontekstem Oparte na PARA:** Użyj folderu `9_Archive/` i `.claudeignore` aby zapobiec zanieczyszczeniu kontekstu AI. Szczegółowe reguły znajdują się w `0_Meta/AI_PARA_Framework.md`.
6. **Reguły Komunikacji:** Zachowaj obiektywny i suchy styl pisania. Brak analogii ani metafor. Dostarczaj wnioski jasno i bezpośrednio. Bez przesady lub emocjonalnych modyfikatorów. Szczegółowe reguły znajdują się w sekcji 3 `0_Meta/EliRule.md`.
7. **Ponowne Wykorzystanie Danych:** Przy generowaniu dowolnego wykresu/grafu zapisz oryginalną Tablicę Danych jako `.mat`/`.csv`. Szczegółowe reguły znajdują się w sekcji 2.6 `0_Meta/EliRule.md`.

## Szybki Start

Aby utworzyć nowy projekt ze strukturą ELF v2, uruchom `0_Meta/ELF_generator.sh`.

```bash
cd desired_parent_directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> W systemie Windows użyj Git Bash (dołączony do [Git for Windows](https://git-scm.com/)).

Wpisz nazwę projektu, a struktura folderów 0–6, dokumenty meta i `.gitignore` zostaną utworzone automatycznie. Inicjalizacja Git jest opcjonalna i proponowana tylko wtedy, gdy Git jest dostępny.

## Licencja

Ten projekt stosuje politykę Licencji Podwójnej, ponieważ natura "kodu wykonywalnego" i "specyfikacji struktury danych (Protokół)" różni się.

* **Oprogramowanie i Skrypty:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Dotyczy:** Cały kod źródłowy (`.m`, `.py`, itd.) w folderach `4_SW/` i `5_Exp/*/Scripts/`.
  * **Warunek:** Jeśli szablony podstawowych skryptów są modyfikowane i ulepszane do redystrybucji, te modyfikacje muszą być udostępniane jako oprogramowanie otwarte. Jednak unikalne algorytmy lub surowe dane dodane przez użytkownika w obrębie projektu mogą pozostać prywatne (skomercjalizowane).

* **Protokół i Dokumentacja:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Dotyczy:** `README.md`, dokumenty `0_Meta/`, hierarchia folderów Sesji-Próby, reguły metadanych rejestrowania Base-Delta i ogólna metodologia badawcza.
  * **Warunek:** Każdy może swobodnie przyjąć i dostosować tę strukturę i metodologię rejestrowania, ale przy publikowaniu szablonów pochodnych lub powiązanych wyników badań, oryginalny autor Eli (projectschnee@gmail.com) i repozytorium źródłowe muszą być przyznane.
