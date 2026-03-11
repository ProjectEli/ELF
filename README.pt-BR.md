[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Protocolo Base-Delta para P&D Ágil

Um padrão integrado de logging para hardware-software-dados experimentais (Protocolo) projetado para suportar ciclos rápidos de feedback (Ágil) durante fases de desenvolvimento de dispositivos e validação de P&D. Garante rastreabilidade completa de dados enquanto minimiza a fadiga de documentação dos pesquisadores.

## Filosofia Principal

* **Única Fonte de Verdade:** Design de hardware, código de análise e dados brutos são organicamente conectados em um único projeto.
* **Logging Base-Delta:** Nem toda variável é registrada. Uma Baseline é declarada e apenas variáveis alteradas (Deltas) são registradas levemente para prevenir atrasos na pesquisa.
* **Imposição Sistemática:** Contorna limitações de comprimento de nome de arquivo (limite de 260 caracteres do Windows) e garante reprodutibilidade através de código.
* **Governança de IA:** Garante continuidade do trabalho de agentes de IA via log de handoff `0_Meta/AI_Sync.md` e impõe um padrão de logging unificado para humanos e IA através de `0_Meta/LogConvention.md`.

## Estrutura de Diretórios do Projeto

Este projeto trata a hierarquia de pastas como um padrão de comunicação.

```text
Project_Root/
├── 0_Meta/                          # Governança do projeto e regras
│   ├── EliRule.md                   # Guia de estrutura de pasta e operações
│   ├── LogConvention.md             # Regras de padrão de logging
│   ├── AI_PARA_Framework.md         # Regras de gerenciamento de contexto de IA e arquivamento
│   └── AI_Sync.md                   # Log de handoff do agente de IA
│
├── 1_Concept/                       # Planejamento de pesquisa, literatura, ideias
│   ├── 11_Ideas/                    # Esboços brutos, propostas de hipóteses
│   ├── 12_Literature/               # PDFs de artigos, informações bibliográficas, fórmulas base
│   └── 13_Planning/                 # Roteiros de pesquisa, storyboards de composição de figuras
│
├── 2_HW/                            # Design de hardware
│   ├── 21_Component/                # Specs de componentes individuais, design de dispositivos unitários
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Design integrado de dispositivos, carcaça, modelos 3D
│   └── 23_Elec/                     # Esquemas PCB, Gerber, BOM, Datasheets
│
├── 3_Fab/                           # Fabricação e processamento
│   ├── 31_Recipes/                  # Documentação de condições de processo
│   └── 32_Eval/                     # Avaliação de características unitárias por módulo
│
├── 4_SW/                            # Software e firmware
│   ├── 41_FW/                       # Firmware MCU/embarcado
│   ├── 42_DAQ/                      # Sistemas de aquisição de dados PC/mobile
│   └── 43_Libs/                     # Bibliotecas compartilhadas reutilizáveis
│
├── 5_Exp/                           # Experimentos: simulação + empírico + análise
│   ├── 51_Sim/                      # Simulação
│   │   ├── Scripts/                 # Código de simulação (S###_sim.m)
│   │   └── Data/                    # Resultados de simulação (Data/S###/)
│   ├── 52_Empirical/                # Dados empíricos
│   │   ├── Raw/                     # Dados brutos de sensores (Somente leitura, excluído do Git)
│   │   └── Processed/               # Dados processados primários
│   ├── 53_Analysis/                 # Análise integrada
│   │   ├── Scripts/                 # Código de pós-processamento de comparação/validação
│   │   └── Logs/                    # Logs de sessão (S###_log.md)
│   └── 54_Viz/                      # Saídas de visualização (figuras auto-geradas)
│
└── 6_Paper/                         # Artigos e apresentações
    ├── 61_Figs/                     # Figuras para artigos
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manuscritos (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Materiais de apresentação (PPT, posters)
```

> Para orientações detalhadas de uso e regras operacionais de cada pasta, consulte `0_Meta/EliRule.md`.

## Especificação do Pipeline de Logging de Dados

### 1. Convenção de Nomenclatura de Arquivos (Naming Session-Trial)

* Listar condições experimentais ou informações de variáveis em nomes de arquivo é **estritamente proibido**.
* **Formato:** `[SessionID]_[TrialID].[extension]` (ex: `S001_t1.csv`, `S001_t2.bin`)

### 2. Logging Base-Delta (Logging Híbrido)

* **Log de Execução (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Um arquivo markdown narrativo que registra ciclos imediatos de hipótese-teste-lição em texto.
  * Escrito por trial (`t1`, `t2`...) em estilo fluxo de consciência, registrando apenas **variáveis intencionalmente alteradas (Delta)** e resultados observados.
  * Formato e regras detalhadas: consulte `0_Meta/LogConvention.md`.

### 3. Regras de Documentação de Planejamento

* Roteiros de pesquisa, composições de figuras, estratégias experimentais, etc. são gerenciados separadamente em `1_Concept/13_Planning/`.
* **Formato:** `P###_title.md` (ex: `P001_wavelength_optimization.md`)
* Ao referenciar Planejamento a partir de um log: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Especificação de Análise de Pós-Processamento (Cell Mode Scripting)

* Código de análise deve estar localizado em `5_Exp/53_Analysis/Scripts/` ou `5_Exp/51_Sim/Scripts/` e não deve ser misturado dentro de pastas de dados.
* Arquivos puros `.m` são usados em vez de `.mlx` para prevenir lock-in de fornecedor.
* Código é executado seção por seção usando `%%` (Cell Mode), e insights derivados são refletidos no log de execução.
* Saídas de análise (figuras, arquivos mat) são salvas em `5_Exp/54_Viz/` ou `5_Exp/52_Empirical/Processed/S###/` dentro de pastas por sessão.

### 5. Regras de Referência Cruzada

Formatos de referência cruzada são unificados para garantir rastreabilidade entre documentos do projeto.

| De → Para | Formato |
|-----------|---------|
| Logs → Planejamento | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Dados de Sim | `→ see 5_Exp/51_Sim/Data/S###/` |
| Logs → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planejamento → Logs | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Governança de IA

Quando agentes de IA (Claude, Gemini, etc.) participam do projeto, as seguintes regras se aplicam:

1. **Aquisição de Contexto:** Antes de iniciar o trabalho, leia `0_Meta/AI_Sync.md` para confirmar o estado do trabalho anterior.
2. **Conformidade com Padrão Unificado:** Siga as regras de logging em `0_Meta/LogConvention.md` da mesma forma que um pesquisador humano.
3. **Registro de Handoff:** Após a conclusão da tarefa, registre ações realizadas, arquivos criados/modificados e Próximos Passos em `0_Meta/AI_Sync.md`. Escreva em ordem cronológica reversa com a entrada mais recente no topo.
4. **Separação de Ideias:** Hipóteses e ideias geradas por IA são armazenadas separadamente em `1_Concept/11_Ideas/`, não em logs.
5. **Gerenciamento de Contexto Baseado em PARA:** Use a pasta `9_Archive/` e `.claudeignore` para prevenir contaminação de contexto de IA. Para regras detalhadas, consulte `0_Meta/AI_PARA_Framework.md`.
6. **Regras de Comunicação:** Mantenha um estilo de escrita objetivo e seco. Sem analogias ou metáforas. Entregue conclusões clara e diretamente. Sem exagero ou modificadores emocionais. Para regras detalhadas, consulte a seção 3 de `0_Meta/EliRule.md`.
7. **Reutilização de Dados:** Ao gerar qualquer Plot/Gráfico, salve o Array de Dados Original junto como `.mat`/`.csv`. Para regras detalhadas, consulte a seção 2.6 de `0_Meta/EliRule.md`.

## Início Rápido

Para criar um novo projeto com a estrutura ELF v2, execute `0_Meta/ELF_generator.bat`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.bat
```

Digite um nome de projeto e a estrutura de pastas 0–6, documentos meta, `.gitignore` e inicialização de Git serão todos completados automaticamente.

## Licença

Este projeto aplica uma política de Licença Dupla porque a natureza do "código executável" e "especificação de estrutura de dados (Protocolo)" difere.

* **Software e Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Aplica-se a:** Todo código fonte (`.m`, `.py`, etc.) dentro das pastas `4_SW/` e `5_Exp/*/Scripts/`.
  * **Condição:** Se scripts principais de template forem modificados e melhorados para redistribuição, essas modificações devem ser lançadas como open source. No entanto, algoritmos únicos ou dados brutos adicionados pelo usuário no projeto podem permanecer privados (comercializados).

* **Protocolo e Documentação:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Aplica-se a:** `README.md`, documentos `0_Meta/`, hierarquia de pastas Session-Trial, regras de logging de metadados Base-Delta, e a metodologia geral de pesquisa.
  * **Condição:** Qualquer pessoa pode adotar livremente e adaptar esta estrutura e metodologia de gravação, mas ao publicar templates derivados ou outputs de pesquisa relacionados, o autor original Eli (projectschnee@gmail.com) e o repositório de origem devem ser creditados.
