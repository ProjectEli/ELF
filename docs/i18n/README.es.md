[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Protocolo Base-Delta para I+D Ágil

Un protocolo de registro integrado para datos de hardware, software y experimentos, diseñado para sustentar ciclos de retroalimentación rápida (Ágil) durante el desarrollo de dispositivos y la validación en I+D. Minimiza la fatiga de documentación del investigador al tiempo que garantiza la trazabilidad completa de los datos.

## Filosofía Central

* **Fuente Única de Verdad:** Conecta orgánicamente el diseño de hardware, el código de análisis y los datos brutos dentro de un único proyecto.
* **Registro Base-Delta:** No registra cada variable. Declara una Línea Base (Baseline) y registra de forma ligera únicamente las variables modificadas (Delta) para evitar retrasos en la investigación.
* **Aplicación Sistemática:** Evita las limitaciones de longitud de nombre de archivo (260 caracteres en Windows) y garantiza la reproducibilidad mediante código.
* **Gobernanza de IA:** Asegura la continuidad del trabajo de agentes de IA mediante registros de transferencia en `0_Meta/AI_Sync.md`, y aplica estándares de registro idénticos para humanos e IA a través de `0_Meta/LogConvention.md`.

## Estructura de Directorios

Este proyecto trata la jerarquía de carpetas en sí misma como un protocolo de comunicación.

```text
Project_Root/
├── 0_Meta/                          # Gobernanza del proyecto y reglas
│   ├── EliRule.md                   # Guía de estructura de carpetas y operación
│   ├── LogConvention.md             # Reglas estándar de registro
│   ├── AI_PARA_Framework.md         # Gestión de contexto de IA y reglas de archivo
│   └── AI_Sync.md                   # Registro de transferencia de agentes de IA
│
├── 1_Concept/                       # Planificación de investigación, literatura, ideas
│   ├── 11_Ideas/                    # Bocetos preliminares, propuestas de hipótesis
│   ├── 12_Literature/               # PDFs de artículos, referencias bibliográficas, fórmulas
│   └── 13_Planning/                 # Hoja de ruta de investigación, guiones gráficos de figuras
│
├── 2_HW/                            # Diseño de hardware
│   ├── 21_Component/                # Especificaciones de componentes, diseño de dispositivos unitarios
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Diseño de dispositivo integrado, carcasa, modelos 3D
│   └── 23_Elec/                     # Esquemáticos PCB, Gerber, BOM, hojas de datos
│
├── 3_Fab/                           # Fabricación y procesos
│   ├── 31_Recipes/                  # Documentación de condiciones de proceso
│   └── 32_Eval/                     # Caracterización por módulo
│
├── 4_SW/                            # Software y firmware
│   ├── 41_FW/                       # Firmware MCU/embebido
│   ├── 42_DAQ/                      # Sistema de adquisición de datos PC/móvil
│   └── 43_Libs/                     # Bibliotecas compartidas reutilizables
│
├── 5_Exp/                           # Experimentos: simulación + empírico + análisis
│   ├── 51_Sim/                      # Simulación
│   │   ├── Scripts/                 # Código de simulación (S###_sim.m)
│   │   └── Data/                    # Resultados de simulación (Data/S###/)
│   ├── 52_Empirical/                # Datos empíricos
│   │   ├── Raw/                     # Datos brutos del sensor (solo lectura, excluido de Git)
│   │   └── Processed/               # Datos preprocesados
│   ├── 53_Analysis/                 # Análisis integrado
│   │   ├── Scripts/                 # Código de postprocesamiento para comparación/validación
│   │   └── Logs/                    # Registros de sesión (S###_log.md)
│   └── 54_Viz/                      # Exportaciones de visualización (figuras generadas automáticamente)
│
└── 6_Paper/                         # Artículos y presentaciones
    ├── 61_Figs/                     # Figuras del artículo
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Manuscritos (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Materiales de presentación (PPT, pósteres)
```

> Para el uso detallado de carpetas y las reglas de operación, consulte `0_Meta/EliRule.md`.

## Pipeline de Registro de Datos

### 1. Convención de Nomenclatura de Archivos (Nomenclatura Sesión-Ensayo)

* Incluir condiciones experimentales o información de variables en los nombres de archivo está **estrictamente prohibido**.
* **Formato:** `[SessionID]_[TrialID].[ext]` (p. ej., `S001_t1.csv`, `S001_t2.bin`)

### 2. Registro Base-Delta (Registro Híbrido)

* **Registro Continuo (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Un archivo markdown narrativo para registrar ciclos inmediatos de hipótesis-prueba-conclusión.
  * Se redacta en forma de flujo de conciencia por ensayo (`t1`, `t2`...), registrando únicamente las **variables modificadas intencionalmente (Delta)** y los resultados observados.
  * Para el formato y las reglas detalladas, consulte `0_Meta/LogConvention.md`.

### 3. Reglas para Documentos de Planificación

* Las hojas de ruta de investigación, la composición de figuras y las estrategias de experimentos se gestionan por separado en `1_Concept/13_Planning/`.
* **Formato:** `P###_titulo.md` (p. ej., `P001_wavelength_optimization.md`)
* Referencia cruzada desde los registros: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Análisis de Postprocesamiento (Scripting en Modo Celda)

* El código de análisis debe residir en `5_Exp/53_Analysis/Scripts/` o `5_Exp/51_Sim/Scripts/`; está prohibido mezclar código dentro de las carpetas de datos.
* Utilice archivos `.m` simples en lugar de `.mlx` para evitar la dependencia de un proveedor específico.
* Use `%%` (Modo Celda) para la ejecución sección por sección; registre las conclusiones derivadas en el registro continuo.
* Las salidas del análisis (figuras, archivos mat) se almacenan en `5_Exp/54_Viz/` o `5_Exp/52_Empirical/Processed/S###/` en carpetas por sesión.

### 5. Reglas de Referencias Cruzadas

Formato unificado de referencias cruzadas para la trazabilidad de documentos dentro del proyecto.

| Desde → Hasta | Formato |
|---------------|---------|
| Registros → Planificación | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Registros → Datos Sim | `→ see 5_Exp/51_Sim/Data/S###/` |
| Registros → Script | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planificación → Registros | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Gobernanza de IA

Reglas para los agentes de IA (Claude, Gemini, etc.) que participan en el proyecto:

1. **Conciencia del Contexto**: Leer `0_Meta/AI_Sync.md` antes de comenzar el trabajo para comprender el estado del trabajo previo.
2. **Cumplimiento de Estándares**: Seguir las reglas de registro de `0_Meta/LogConvention.md` de forma idéntica a los investigadores humanos.
3. **Registro de Transferencia**: Al completar el trabajo, registrar las tareas realizadas, los archivos creados/modificados y los Próximos Pasos en `0_Meta/AI_Sync.md`. Escribir en orden cronológico inverso (el más reciente primero).
4. **Separación de Ideas**: Las hipótesis e ideas generadas por IA van a `1_Concept/11_Ideas/`, no en los registros.
5. **Gestión de Contexto Basada en PARA**: Usar carpetas `9_Archive/` y `.claudeignore` para prevenir la contaminación del contexto de IA. Consulte `0_Meta/AI_PARA_Framework.md` para más detalles.
6. **Reglas de Comunicación**: Mantener un tono objetivo y sobrio. Sin analogías ni metáforas. Entrega clara y centrada en conclusiones. Sin modificadores exagerados o emocionales. Consulte `0_Meta/EliRule.md`, Sección 3.
7. **Reutilización de Datos**: Siempre exportar arrays de datos brutos como `.mat`/`.csv` junto a cualquier gráfico o figura. Consulte `0_Meta/EliRule.md`, Sección 2.6.

## Inicio Rápido

Para crear un nuevo proyecto con la estructura ELF v2, ejecute `0_Meta/ELF_generator.ps1`.

```
cd your_target_directory
D:\...\ELF\0_Meta\ELF_generator.ps1
```

Introduzca un nombre de proyecto y la jerarquía de carpetas 0~6, los documentos meta, `.gitignore` y la inicialización de Git se completan automáticamente.

## Licencia

Este proyecto aplica una política de licencia dual, ya que el "código ejecutable" y el "protocolo de estructura de datos" tienen propósitos diferentes.

* **Software y Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Alcance:** Todo el código fuente (`.m`, `.py`, etc.) en `4_SW/` y `5_Exp/*/Scripts/`.
  * **Términos:** Los scripts de plantilla base modificados deben publicarse como código abierto. Sin embargo, los algoritmos propietarios y los datos brutos añadidos por el usuario pueden permanecer privados (con posibilidad de comercialización).

* **Protocolo y Documentación:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Alcance:** `README.md`, documentos de `0_Meta/`, jerarquía de carpetas Sesión-Ensayo, reglas de registro de metadatos Base-Delta y la metodología de investigación en general.
  * **Términos:** Cualquier persona puede adoptar y modificar libremente esta estructura y metodología, pero debe dar crédito al autor original Eli (projectschnee@gmail.com) y a este repositorio al publicar plantillas derivadas o resultados de investigación relacionados.
