[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): アジャイルR&DのためのBase-Deltaプロトコル

デバイス開発およびR&D検証における迅速なフィードバックループ（アジャイル）をサポートするために設計された、ハードウェア・ソフトウェア・実験データの統合記録プロトコル。研究者のドキュメント作成負荷を最小化しつつ、完全なデータトレーサビリティを確保します。

## コア哲学

* **単一の信頼できる情報源:** ハードウェア設計、解析コード、生データを単一プロジェクト内で有機的に接続する。
* **Base-Deltaロギング:** すべての変数を記録しない。ベースラインを宣言し、変更された変数（Delta）のみを軽量に記録することで、研究の遅延を防ぐ。
* **体系的な強制:** ファイル名の長さ制限（Windows 260文字）を回避し、コードを通じて再現性を確保する。
* **AIガバナンス:** `0_Meta/AI_Sync.md` のハンドオフログによりAIエージェントの作業継続性を確保し、`0_Meta/LogConvention.md` を通じて人間とAIの両方に同一のロギング基準を適用する。

## ディレクトリ構造

このプロジェクトは、フォルダ階層そのものをコミュニケーションプロトコルとして扱います。

```text
Project_Root/
├── 0_Meta/                          # プロジェクトガバナンス＆ルール
│   ├── EliRule.md                   # フォルダ構造＆運用ガイド
│   ├── LogConvention.md             # ロギング標準ルール
│   ├── AI_PARA_Framework.md         # AIコンテキスト管理＆アーカイブルール
│   └── AI_Sync.md                   # AIエージェント引き継ぎログ
│
├── 1_Concept/                       # 研究計画、文献、アイデア
│   ├── 11_Ideas/                    # ラフスケッチ、仮説提案
│   ├── 12_Literature/               # 論文PDF、文献情報、数式
│   └── 13_Planning/                 # 研究ロードマップ、図のストーリーボード
│
├── 2_HW/                            # ハードウェア設計
│   ├── 21_Component/                # コンポーネント仕様、単体デバイス設計
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # 統合デバイス設計、筐体、3Dモデル
│   └── 23_Elec/                     # PCB回路図、Gerber、BOM、データシート
│
├── 3_Fab/                           # 製造＆プロセス
│   ├── 31_Recipes/                  # プロセス条件ドキュメント
│   └── 32_Eval/                     # モジュール別特性評価
│
├── 4_SW/                            # ソフトウェア＆ファームウェア
│   ├── 41_FW/                       # MCU/組み込みファームウェア
│   ├── 42_DAQ/                      # PC/モバイルデータ収集システム
│   └── 43_Libs/                     # 再利用可能な共有ライブラリ
│
├── 5_Exp/                           # 実験：シミュレーション＋実験的＋解析
│   ├── 51_Sim/                      # シミュレーション
│   │   ├── Scripts/                 # シミュレーションコード (S###_sim.m)
│   │   └── Data/                    # シミュレーション結果 (Data/S###/)
│   ├── 52_Empirical/                # 実験データ
│   │   ├── Raw/                     # 元のセンサーデータ（読み取り専用、Git除外）
│   │   └── Processed/               # 前処理済みデータ
│   ├── 53_Analysis/                 # 統合解析
│   │   ├── Scripts/                 # 比較/検証後処理コード
│   │   └── Logs/                    # セッションログ (S###_log.md)
│   └── 54_Viz/                      # 可視化エクスポート（自動生成図）
│
└── 6_Paper/                         # 論文＆プレゼンテーション
    ├── 61_Figs/                     # 論文図版
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # 原稿（Word、LaTeX）
    │   └── archive/
    └── 63_Presentations/            # プレゼンテーション資料（PPT、ポスター）
```

> フォルダの詳細な使用方法および運用ルールは `0_Meta/EliRule.md` を参照してください。

## データロギングパイプライン

### 1. ファイル命名規則（セッション-トライアル命名）

* 実験条件や変数情報をファイル名に記載することは**厳禁**です。
* **フォーマット:** `[SessionID]_[TrialID].[ext]`（例: `S001_t1.csv`、`S001_t2.bin`）

### 2. Base-Deltaロギング（ハイブリッドロギング）

* **ランニングログ (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * 仮説-テスト-インサイトサイクルを即座に記録するための、叙述的なMarkdownファイル。
  * トライアル（`t1`、`t2`...）ごとに意識の流れで記述し、**意図的に変更した変数（Delta）**と観察結果のみを記録する。
  * フォーマットと詳細ルールは `0_Meta/LogConvention.md` を参照。

### 3. 計画ドキュメントのルール

* 研究ロードマップ、図の構成、実験戦略は `1_Concept/13_Planning/` で別途管理する。
* **フォーマット:** `P###_title.md`（例: `P001_wavelength_optimization.md`）
* ログからの相互参照: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. 後処理解析（セルモードスクリプティング）

* 解析コードは `5_Exp/53_Analysis/Scripts/` または `5_Exp/51_Sim/Scripts/` に配置すること。データフォルダ内にコードを混在させることは禁止。
* ベンダーロックインを避けるため、`.mlx` ではなく通常の `.m` ファイルを使用すること。
* セクション別実行には `%%`（セルモード）を使用し、導出されたインサイトはランニングログに記録すること。
* 解析出力（図、matファイル）は `5_Exp/54_Viz/` または `5_Exp/52_Empirical/Processed/S###/` のセッション別フォルダに保存する。

### 5. 相互参照ルール

プロジェクト内のドキュメントトレーサビリティのための統一相互参照フォーマット。

| 参照元 → 参照先 | フォーマット |
|----------------|------------|
| ログ → 計画 | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| ログ → シミュレーションデータ | `→ see 5_Exp/51_Sim/Data/S###/` |
| ログ → スクリプト | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| 計画 → ログ | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AIガバナンス

プロジェクトに参加するAIエージェント（Claude、Geminiなど）のためのルール:

1. **コンテキスト認識**: 作業開始前に `0_Meta/AI_Sync.md` を読み、前回の作業状態を把握すること。
2. **標準遵守**: 人間の研究者と同一の `0_Meta/LogConvention.md` ロギングルールに従うこと。
3. **引き継ぎ記録**: 作業完了時に、実施したタスク、作成/変更したファイル、ネクストステップを `0_Meta/AI_Sync.md` に記録すること。逆時系列順（最新が最初）で記述すること。
4. **アイデアの分離**: AIが生成した仮説/アイデアはログではなく `1_Concept/11_Ideas/` に保存すること。
5. **PARAベースのコンテキスト管理**: `9_Archive/` フォルダと `.claudeignore` を使用してAIコンテキストの汚染を防ぐこと。詳細は `0_Meta/AI_PARA_Framework.md` を参照。
6. **コミュニケーションルール**: 客観的でドライなトーンを維持すること。類推/比喩を使わない。明確で結論に焦点を当てた表現。誇張や感情的な修飾語を使わない。`0_Meta/EliRule.md` セクション3を参照。
7. **データ再利用性**: プロット/グラフとともに、常に生データ配列を `.mat`/`.csv` としてエクスポートすること。`0_Meta/EliRule.md` セクション2.6を参照。

## クイックスタート

ELF v2構造で新しいプロジェクトを作成するには、`0_Meta/ELF_generator.bat` を実行してください。

```
cd your_target_directory
D:\...\ELF\0_Meta\ELF_generator.bat
```

プロジェクト名を入力すると、0〜6のフォルダ階層、メタドキュメント、`.gitignore`、Gitの初期化が自動的に完了します。

## ライセンス

このプロジェクトは、「実行可能なコード」と「データ構造プロトコル」が異なる目的を持つため、デュアルライセンスポリシーを適用しています。

* **ソフトウェア＆スクリプト:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **適用範囲:** `4_SW/` および `5_Exp/*/Scripts/` 内のすべてのソースコード（`.m`、`.py` など）。
  * **条件:** 変更されたコアテンプレートスクリプトはオープンソース化する必要があります。ただし、ユーザーが追加した独自アルゴリズムや生データはプライベートのままにできます（商業利用可能）。

* **プロトコル＆ドキュメント:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **適用範囲:** `README.md`、`0_Meta/` ドキュメント、セッション-トライアルフォルダ階層、Base-Deltaメタデータロギングルール、および研究方法論全般。
  * **条件:** 誰でもこの構造と方法論を自由に採用・変更できますが、派生テンプレートや関連する研究成果を公開する際は、原著者Eli（projectschnee@gmail.com）とこのリポジトリのクレジット表記が必要です。

---
