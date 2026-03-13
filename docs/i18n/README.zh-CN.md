[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF)：面向敏捷研发的基线-增量协议

一套针对硬件-软件-实验数据的集成记录协议，旨在支持设备开发与研发验证过程中的快速反馈循环（敏捷模式）。在确保完整数据溯源性的同时，最大程度降低研究人员的文档记录负担。

## 核心理念

* **单一可信源：** 在同一项目中有机整合硬件设计、分析代码与原始数据。
* **基线-增量记录：** 不记录所有变量，而是声明一个基线（Baseline），仅轻量记录发生变化的变量（Delta），以防止研究进度受阻。
* **系统化强制执行：** 通过代码绕过文件名长度限制（Windows 260字符限制）并保证可复现性。
* **AI 治理：** 通过 `0_Meta/AI_Sync.md` 交接日志确保 AI 智能体的工作连续性，并通过 `0_Meta/LogConvention.md` 对人类与 AI 强制执行统一的记录规范。

## 目录结构

本项目将文件夹层级本身视为一种通信协议。

```text
Project_Root/
├── 0_Meta/                          # 项目治理与规则
│   ├── EliRule.md                   # 文件夹结构与操作指南
│   ├── LogConvention.md             # 记录规范规则
│   ├── AI_PARA_Framework.md         # AI 上下文管理与归档规则
│   └── AI_Sync.md                   # AI 智能体交接日志
│
├── 1_Concept/                       # 研究规划、文献与创意
│   ├── 11_Ideas/                    # 草图、假设提案
│   ├── 12_Literature/               # 论文 PDF、参考文献信息、公式
│   └── 13_Planning/                 # 研究路线图、图表故事板
│       └── 2_Wiki/                  # 提炼后的规划结论与关键规则
│
├── 2_HW/                            # 硬件设计
│   ├── 21_Component/                # 元器件规格、单元设备设计
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # 集成设备设计、外壳、3D 模型
│   └── 23_Elec/                     # PCB 原理图、Gerber 文件、BOM、数据手册
│
├── 3_Fab/                           # 制造与工艺
│   ├── 31_Recipes/                  # 工艺条件文档
│   └── 32_Eval/                     # 各模块特性表征
│
├── 4_SW/                            # 软件与固件
│   ├── 41_FW/                       # MCU/嵌入式固件
│   ├── 42_DAQ/                      # PC/移动端数据采集系统
│   └── 43_Libs/                     # 可复用共享库
│
├── 5_Exp/                           # 实验：仿真 + 实测 + 分析
│   ├── 51_Sim/                      # 仿真
│   │   ├── Scripts/                 # 仿真代码（S###_sim.m）
│   │   │   └── 9_Archive/          # 已退役脚本
│   │   └── Data/                    # 仿真结果（Data/S###/）
│   ├── 52_Empirical/                # 实测数据
│   │   ├── Raw/                     # 原始传感器数据（只读，Git 排除）
│   │   └── Processed/               # 预处理数据
│   ├── 53_Analysis/                 # 综合分析
│   │   ├── Scripts/                 # 比较/验证后处理代码
│   │   │   └── 9_Archive/          # 已退役脚本
│   │   └── Logs/                    # 会话日志（S###_log.md）
│   │       ├── 2_Wiki/              # 提炼后的发现与会话注册表
│   │       └── 9_Archive/           # 已完成的会话日志
│   └── 54_Viz/                      # 可视化导出（自动生成的图表）
│
└── 6_Paper/                         # 论文与演示
    ├── 61_Figs/                     # 论文图表
    │   ├── Raw/
    │   ├── Processed/
    │   └── Final/
    ├── 62_Drafts/                   # 手稿（Word、LaTeX）
    │   └── 9_Archive/                # 以前的版本
    └── 63_Presentations/            # 演示材料（PPT、海报）
```

> 有关文件夹详细使用方法和操作规则，请参阅 `0_Meta/EliRule.md`。

## 数据记录流程

### 1. 文件命名规范（会话-试验命名）

* 在文件名中列出实验条件或变量信息**严格禁止**。
* **格式：** `[SessionID]_[TrialID].[ext]`（例如：`S001_t1.csv`、`S001_t2.bin`）

### 2. 基线-增量记录（混合记录法）

* **运行日志（`5_Exp/53_Analysis/Logs/S###_log.md`）：**
  * 一个叙述性 Markdown 文件，用于记录即时的假设-测试-洞察循环。
  * 按试验（`t1`、`t2`……）以意识流方式写作，仅记录**有意改变的变量（Delta）**及观测结果。
  * 格式与详细规则请参阅 `0_Meta/LogConvention.md`。

### 3. 规划文档规则

* 研究路线图、图表构成和实验策略单独管理于 `1_Concept/13_Planning/`。
* **格式：** `P###_title.md`（例如：`P001_wavelength_optimization.md`）
* 从日志中交叉引用：`→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. 后处理分析（单元格模式脚本）

* 分析代码必须位于 `5_Exp/53_Analysis/Scripts/` 或 `5_Exp/51_Sim/Scripts/`；禁止将代码混入数据文件夹。
* 使用普通 `.m` 文件而非 `.mlx`，以避免供应商锁定。
* 使用 `%%`（单元格模式）进行分段执行；将推导出的洞察记录在运行日志中。
* 分析输出（图表、mat 文件）按会话存储于 `5_Exp/54_Viz/` 或 `5_Exp/52_Empirical/Processed/S###/` 文件夹中。

### 5. 交叉引用规则

项目内文档溯源的统一交叉引用格式。

| 来源 → 目标 | 格式 |
|-----------|--------|
| 日志 → 规划 | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| 日志 → 仿真数据 | `→ see 5_Exp/51_Sim/Data/S###/` |
| 日志 → 脚本 | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| 规划 → 日志 | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI 治理

参与项目的 AI 智能体（Claude、Gemini 等）适用以下规则：

1. **上下文感知**：开始工作前读取 `0_Meta/AI_Sync.md`，了解此前的工作状态。
2. **规范合规**：与人类研究人员一致地遵守 `0_Meta/LogConvention.md` 记录规则。
3. **交接记录**：工作完成后，在 `0_Meta/AI_Sync.md` 中记录已执行的任务、创建/修改的文件以及后续步骤。按逆时间顺序书写（最新内容在前）。
4. **创意分离**：AI 生成的假设/创意归入 `1_Concept/11_Ideas/`，不写入日志。
5. **基于 PARA 的上下文管理**：使用 `9_Archive/` 文件夹和 `.claudeignore` 防止 AI 上下文污染。详情请参阅 `0_Meta/AI_PARA_Framework.md`。
6. **沟通规则**：保持客观、简洁的语气。不使用类比或隐喻。表达清晰，以结论为导向。不使用夸张或情绪化的修饰语。详见 `0_Meta/EliRule.md` 第 3 节。
7. **数据可复用性**：任何图表/图形旁边，始终同步导出原始数据数组为 `.mat`/`.csv` 格式。详见 `0_Meta/EliRule.md` 第 2.6 节。

## 快速开始

如需以 ELF v2 结构创建新项目，请运行 `0_Meta/ELF_generator.sh`。

```bash
cd your_target_directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> 在 Windows 上，请使用 Git Bash（包含在 [Git for Windows](https://git-scm.com/) 中）。

输入项目名称后，0~6 文件夹层级、元文档和 `.gitignore` 将自动创建。Git 初始化为可选项，仅在 Git 可用时提示。

## 许可证

由于"可执行代码"与"数据结构协议"服务于不同目的，本项目采用双重许可证策略。

* **软件与脚本：** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **适用范围：** `4_SW/` 和 `5_Exp/*/Scripts/` 中的所有源代码（`.m`、`.py` 等）。
  * **条款：** 修改后的核心模板脚本须开源。但用户添加的专有算法和原始数据可保持私有（可商业化）。

* **协议与文档：** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **适用范围：** `README.md`、`0_Meta/` 文档、会话-试验文件夹层级、基线-增量元数据记录规则以及研究方法论总体内容。
  * **条款：** 任何人均可自由采用和修改本结构与方法论，但在发布衍生模板或相关研究成果时，须注明原作者 Eli（projectschnee@gmail.com）及本仓库的信息来源。
