[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF) : Protocole Base-Delta pour R&D Agile

Un standard de journalisation intégré matériel-logiciel-données expérimentales (Protocole) conçu pour soutenir des boucles de rétroaction rapides (Agile) lors des phases de développement de dispositifs et de validation en R&D. Garantit une traçabilité complète des données tout en minimisant la fatigue documentaire des chercheurs.

## Philosophie Fondamentale

* **Source Unique de Vérité :** La conception matérielle, le code d'analyse et les données brutes sont organiquement connectés au sein d'un seul projet.
* **Journalisation Base-Delta :** Les variables ne sont pas toutes enregistrées. Une Baseline est déclarée, et seules les variables modifiées (Deltas) sont journalisées légèrement pour prévenir les retards de recherche.
* **Contrôle Systématique :** Contourne les limitations de longueur de nom de fichier (limite Windows de 260 caractères) et garantit la reproductibilité par le code.
* **Gouvernance IA :** Assure la continuité du travail des agents IA via le journal de transmission `0_Meta/AI_Sync.md`, et impose un standard de journalisation unifié pour les humains et l'IA par le biais de `0_Meta/LogConvention.md`.

## Structure du Répertoire du Projet

Ce projet traite la hiérarchie des dossiers elle-même comme un standard de communication.

```text
Project_Root/
├── 0_Meta/                          # Gouvernance du projet et règles
│   ├── EliRule.md                   # Guide de la structure des dossiers et d'exploitation
│   ├── LogConvention.md             # Règles standard de journalisation
│   ├── AI_PARA_Framework.md         # Gestion du contexte IA et règles d'archivage
│   └── AI_Sync.md                   # Journal de transmission des agents IA
│
├── 1_Concept/                       # Planification de la recherche, littérature, idées
│   ├── 11_Ideas/                    # Croquis bruts, propositions d'hypothèses
│   ├── 12_Literature/               # PDFs d'articles, info bibliographique, formules de base
│   └── 13_Planning/                 # Feuilles de route de recherche, storyboards de composition de figures
│
├── 2_HW/                            # Conception matérielle
│   ├── 21_Component/                # Spécifications de composants individuels, conception d'unité
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Conception de dispositif intégré, boîtier, modèles 3D
│   └── 23_Elec/                     # Schémas PCB, Gerber, BOM, Datasheets
│
├── 3_Fab/                           # Fabrication et traitement
│   ├── 31_Recipes/                  # Documentation des conditions de procédé
│   └── 32_Eval/                     # Évaluation caractéristique unique par module
│
├── 4_SW/                            # Logiciel et micrologiciel
│   ├── 41_FW/                       # Micrologiciel MCU/intégré
│   ├── 42_DAQ/                      # Systèmes d'acquisition de données PC/mobile
│   └── 43_Libs/                     # Bibliothèques partagées réutilisables
│
├── 5_Exp/                           # Expériences : simulation + empirique + analyse
│   ├── 51_Sim/                      # Simulation
│   │   ├── Scripts/                 # Code de simulation (S###_sim.m)
│   │   └── Data/                    # Résultats de simulation (Data/S###/)
│   ├── 52_Empirical/                # Données empiriques
│   │   ├── Raw/                     # Données brutes de capteur (Lecture seule, exclues de Git)
│   │   └── Processed/               # Données traitées primaires
│   ├── 53_Analysis/                 # Analyse intégrée
│   │   ├── Scripts/                 # Code de post-traitement comparaison/validation
│   │   └── Logs/                    # Journaux de session (S###_log.md)
│   └── 54_Viz/                      # Sorties de visualisation (figures auto-générées)
│
└── 6_Paper/                         # Articles et présentations
    ├── 61_Figs/                     # Figures pour articles
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manuscrits (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Matériaux de présentation (PPT, posters)
```

> Pour une utilisation détaillée et des règles d'exploitation de chaque dossier, consultez `0_Meta/EliRule.md`.

## Spécification du Pipeline de Journalisation des Données

### 1. Convention de Nommage de Fichiers (Nommage Session-Trial)

* Lister les conditions expérimentales ou les informations de variables dans les noms de fichiers est **strictement interdit**.
* **Format :** `[SessionID]_[TrialID].[extension]` (par ex., `S001_t1.csv`, `S001_t2.bin`)

### 2. Journalisation Base-Delta (Journalisation Hybride)

* **Journalisation Courante (`5_Exp/53_Analysis/Logs/S###_log.md`) :**
  * Un fichier markdown narratif qui enregistre les cycles immédiats d'hypothèse-test-apprentissage en texte.
  * Rédigé par trial (`t1`, `t2`...) dans un style de pensée fluide, enregistrant uniquement les **variables intentionnellement modifiées (Delta)** et les résultats observés.
  * Format et règles détaillées : consultez `0_Meta/LogConvention.md`.

### 3. Règles des Documents de Planification

* Les feuilles de route de recherche, compositions de figures, stratégies expérimentales, etc. sont gérées séparément dans `1_Concept/13_Planning/`.
* **Format :** `P###_title.md` (par ex., `P001_wavelength_optimization.md`)
* Lors du référencement de la Planification depuis un journal : `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Spécification de l'Analyse de Post-Traitement (Scripting en Mode Cellule)

* Le code d'analyse doit être situé dans `5_Exp/53_Analysis/Scripts/` ou `5_Exp/51_Sim/Scripts/` et ne doit pas être mélangé à l'intérieur des dossiers de données.
* Des fichiers `.m` purs sont utilisés à la place de `.mlx` pour prévenir le verrouillage du fournisseur.
* Le code est exécuté section par section en utilisant `%%` (Mode Cellule), et les apprentissages dérivés sont reflétés dans le journal courant.
* Les sorties d'analyse (figures, fichiers mat) sont enregistrées dans `5_Exp/54_Viz/` ou `5_Exp/52_Empirical/Processed/S###/` dans des dossiers par session.

### 5. Règles de Référencement Croisé

Les formats de référencement croisé sont unifiés pour assurer la traçabilité entre les documents du projet.

| De → À | Format |
|-----------|--------|
| Journaux → Planification | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Journaux → Données Sim | `→ see 5_Exp/51_Sim/Data/S###/` |
| Journaux → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planification → Journaux | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Gouvernance IA

Lorsque des agents IA (Claude, Gemini, etc.) participent au projet, les règles suivantes s'appliquent :

1. **Acquisition du Contexte :** Avant de commencer le travail, lisez `0_Meta/AI_Sync.md` pour confirmer l'état du travail précédent.
2. **Conformité Standard Unifiée :** Suivez les règles de journalisation dans `0_Meta/LogConvention.md` de la même manière qu'un chercheur humain.
3. **Enregistrement de la Transmission :** À la fin des tâches, enregistrez les actions effectuées, les fichiers créés/modifiés et les Prochaines Étapes dans `0_Meta/AI_Sync.md`. Écrivez dans un ordre antéchronologique avec l'entrée la plus récente en haut.
4. **Séparation des Idées :** Les hypothèses et idées générées par l'IA sont stockées séparément dans `1_Concept/11_Ideas/`, pas dans les journaux.
5. **Gestion du Contexte Basée sur PARA :** Utilisez le dossier `9_Archive/` et `.claudeignore` pour prévenir la contamination du contexte IA. Pour des règles détaillées, consultez `0_Meta/AI_PARA_Framework.md`.
6. **Règles de Communication :** Maintenez un style d'écriture objectif et sec. Pas d'analogies ou de métaphores. Livrez les conclusions clairement et directement. Pas d'exagération ou de modificateurs émotionnels. Pour des règles détaillées, consultez la section 3 de `0_Meta/EliRule.md`.
7. **Réutilisabilité des Données :** Lors de la génération de tout Graphique/Figure, enregistrez le Array de Données original à côté au format `.mat`/`.csv`. Pour des règles détaillées, consultez la section 2.6 de `0_Meta/EliRule.md`.

## Démarrage Rapide

Pour créer un nouveau projet avec la structure ELF v2, exécutez `0_Meta/ELF_generator.bat`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.bat
```

Entrez un nom de projet et la structure de dossiers 0–6, les documents meta, `.gitignore`, et l'initialisation Git seront tous complétés automatiquement.

## Licence

Ce projet applique une politique de Licence Duelle car la nature du « code exécutable » et de la « spécification de la structure de données (Protocole) » diffère.

* **Logiciel et Scripts :** [Licence Publique Mozilla 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **S'applique à :** Tout le code source (`.m`, `.py`, etc.) dans les dossiers `4_SW/` et `5_Exp/*/Scripts/`.
  * **Condition :** Si les scripts principaux du template sont modifiés et améliorés pour redistribution, ces modifications doivent être publiées en tant que code source ouvert. Cependant, les algorithmes uniques ou les données brutes ajoutées par l'utilisateur au sein du projet peuvent rester privés (commercialisables).

* **Protocole et Documentation :** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **S'applique à :** `README.md`, documents `0_Meta/`, la hiérarchie des dossiers Session-Trial, les règles de journalisation Base-Delta, et la méthodologie de recherche globale.
  * **Condition :** Quiconque peut librement adopter et adapter cette structure et cette méthodologie d'enregistrement, mais lors de la publication de templates dérivés ou de résultats de recherche connexes, l'auteur original Eli (projectschnee@gmail.com) et le dépôt source doivent être crédités.
