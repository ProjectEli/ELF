[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF)：面向敏捷研發的基線-增量協議

一套針對硬體-軟體-實驗數據的集成記錄協議，旨在支持設備開發與研發驗證過程中的快速反饋循環（敏捷模式）。在確保完整數據溯源性的同時，最大程度降低研究人員的文檔記錄負擔。

## 核心理念

* **單一可信源：** 在同一項目中有機整合硬體設計、分析代碼與原始數據。
* **基線-增量記錄：** 不記錄所有變數，而是聲明一個基線（Baseline），僅輕量記錄發生變化的變數（Delta），以防止研究進度受阻。
* **系統化強制執行：** 通過代碼繞過檔案名長度限制（Windows 260字符限制）並保證可復現性。
* **AI 治理：** 通過 `0_Meta/AI_Sync.md` 交接日誌確保 AI 智能體的工作連續性，並通過 `0_Meta/LogConvention.md` 對人類與 AI 強制執行統一的記錄規範。

## 目錄結構

本項目將檔案夾層級本身視為一種通信協議。

```text
Project_Root/
├── 0_Meta/                          # 項目治理與規則
│   ├── EliRule.md                   # 檔案夾結構與操作指南
│   ├── LogConvention.md             # 記錄規範規則
│   ├── AI_PARA_Framework.md         # AI 上下文管理與歸檔規則
│   └── AI_Sync.md                   # AI 智能體交接日誌
│
├── 1_Concept/                       # 研究規劃、文獻與創意
│   ├── 11_Ideas/                    # 草圖、假設提案
│   ├── 12_Literature/               # 論文 PDF、參考文獻信息、公式
│   └── 13_Planning/                 # 研究路線圖、圖表故事板
│       └── 2_Wiki/                  # 精煉後的規劃結論與關鍵規則
│
├── 2_HW/                            # 硬體設計
│   ├── 21_Component/                # 元器件規格、單元設備設計
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # 集成設備設計、外殼、3D 模型
│   └── 23_Elec/                     # PCB 原理圖、Gerber 檔案、BOM、數據手冊
│
├── 3_Fab/                           # 製造與工藝
│   ├── 31_Recipes/                  # 工藝條件文檔
│   └── 32_Eval/                     # 各模塊特性表徵
│
├── 4_SW/                            # 軟體與固件
│   ├── 41_FW/                       # MCU/嵌入式固件
│   ├── 42_DAQ/                      # PC/移動端數據採集系統
│   └── 43_Libs/                     # 可複用共享庫
│
├── 5_Exp/                           # 實驗：仿真 + 實測 + 分析
│   ├── 51_Sim/                      # 仿真
│   │   ├── Scripts/                 # 仿真代碼（S###_sim.m）
│   │   │   └── 9_Archive/          # 已退役腳本
│   │   └── Data/                    # 仿真結果（Data/S###/）
│   ├── 52_Empirical/                # 實測數據
│   │   ├── Raw/                     # 原始傳感器數據（只讀，Git 排除）
│   │   └── Processed/               # 預處理數據
│   ├── 53_Analysis/                 # 綜合分析
│   │   ├── Scripts/                 # 比較/驗證後處理代碼
│   │   │   └── 9_Archive/          # 已退役腳本
│   │   └── Logs/                    # 會話日誌（S###_log.md）
│   │       ├── 2_Wiki/              # 精煉後的發現與會話登錄簿
│   │       └── 9_Archive/           # 已完成的會話日誌
│   └── 54_Viz/                      # 可視化導出（自動生成的圖表）
│
└── 6_Paper/                         # 論文與演示
    ├── 61_Figs/                     # 論文圖表
    │   ├── Raw/
    │   ├── Processed/
    │   └── Final/
    ├── 62_Drafts/                   # 手稿（Word、LaTeX）
    │   └── 9_Archive/                # 以前的版本
    └── 63_Presentations/            # 演示材料（PPT、海報）
```

> 有關檔案夾詳細使用方法和操作規則，請參閱 `0_Meta/EliRule.md`。

## 數據記錄流程

### 1. 檔案命名規範（會話-試驗命名）

* 在檔案名中列出實驗條件或變數資訊**嚴格禁止**。
* **格式：** `[SessionID]_[TrialID].[ext]`（例如：`S001_t1.csv`、`S001_t2.bin`）

### 2. 基線-增量記錄（混合記錄法）

* **運行日誌（`5_Exp/53_Analysis/Logs/S###_log.md`）：**
  * 一個敘述性 Markdown 檔案，用於記錄即時的假設-測試-洞察循環。
  * 按試驗（`t1`、`t2`……）以意識流方式寫作，僅記錄**有意改變的變數（Delta）**及觀測結果。
  * 格式與詳細規則請參閱 `0_Meta/LogConvention.md`。

### 3. 規劃文檔規則

* 研究路線圖、圖表構成和實驗策略單獨管理於 `1_Concept/13_Planning/`。
* **格式：** `P###_title.md`（例如：`P001_wavelength_optimization.md`）
* 從日誌中交叉參考：`→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. 後處理分析（單元格模式腳本）

* 分析代碼必須位於 `5_Exp/53_Analysis/Scripts/` 或 `5_Exp/51_Sim/Scripts/`；禁止將代碼混入數據檔案夾。
* 使用普通 `.m` 檔案而非 `.mlx`，以避免供應商鎖定。
* 使用 `%%`（單元格模式）進行分段執行；將推導出的洞察記錄在運行日誌中。
* 分析輸出（圖表、mat 檔案）按會話存儲於 `5_Exp/54_Viz/` 或 `5_Exp/52_Empirical/Processed/S###/` 檔案夾中。

### 5. 交叉參考規則

項目內文檔溯源的統一交叉參考格式。

| 來源 → 目標 | 格式 |
|-----------|--------|
| 日誌 → 規劃 | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| 日誌 → 仿真數據 | `→ see 5_Exp/51_Sim/Data/S###/` |
| 日誌 → 腳本 | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| 規劃 → 日誌 | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI 治理

參與項目的 AI 智能體（Claude、Gemini 等）適用以下規則：

1. **上下文感知**：開始工作前讀取 `0_Meta/AI_Sync.md`，了解此前的工作狀態。
2. **規範合規**：與人類研究人員一致地遵守 `0_Meta/LogConvention.md` 記錄規則。
3. **交接記錄**：工作完成後，在 `0_Meta/AI_Sync.md` 中記錄已執行的任務、創建/修改的檔案以及後續步驟。按逆時間順序書寫（最新內容在前）。
4. **創意分離**：AI 生成的假設/創意歸入 `1_Concept/11_Ideas/`，不寫入日誌。
5. **基於 PARA 的上下文管理**：使用 `9_Archive/` 檔案夾和 `.claudeignore` 防止 AI 上下文污染。詳情請參閱 `0_Meta/AI_PARA_Framework.md`。
6. **溝通規則**：保持客觀、簡潔的語氣。不使用類比或隱喻。表達清晰，以結論為導向。不使用誇張或情緒化的修飾語。詳見 `0_Meta/EliRule.md` 第 3 節。
7. **數據可複用性**：任何圖表/圖形旁邊，始終同步導出原始數據數組為 `.mat`/`.csv` 格式。詳見 `0_Meta/EliRule.md` 第 2.6 節。

## 快速開始

如需以 ELF v2 結構創建新項目，請執行 `0_Meta/ELF_generator.sh`。

```bash
cd your_target_directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> 在 Windows 上，請使用 Git Bash（包含在 [Git for Windows](https://git-scm.com/) 中）。

輸入項目名稱後，0~6 檔案夾層級、元文檔和 `.gitignore` 將自動建立。Git 初始化為可選項，僅在 Git 可用時提示。

## 許可證

由於"可執行代碼"與"數據結構協議"服務於不同目的，本項目採用雙重許可證策略。

* **軟體與腳本：** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **適用範圍：** `4_SW/` 和 `5_Exp/*/Scripts/` 中的所有源代碼（`.m`、`.py` 等）。
  * **條款：** 修改後的核心模板腳本須開源。但用戶添加的專有算法和原始數據可保持私有（可商業化）。

* **協議與文檔：** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **適用範圍：** `README.md`、`0_Meta/` 文檔、會話-試驗檔案夾層級、基線-增量元數據記錄規則以及研究方法論總體內容。
  * **條款：** 任何人均可自由採用和修改本結構與方法論，但在發佈衍生模板或相關研究成果時，須註明原作者 Eli（projectschnee@gmail.com）及本倉庫的信息來源。
