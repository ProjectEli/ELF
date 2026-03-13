[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Protokol Base-Delta untuk Agile R&D

Standar logging data terintegrasi hardware-software-eksperimental (Protokol) yang dirancang untuk mendukung loop umpan balik cepat (Agile) selama pengembangan perangkat dan fase validasi R&D. Menjamin traceability data lengkap sambil meminimalkan kelelahan dokumentasi peneliti.

## Filosofi Inti

* **Single Source of Truth:** Desain hardware, kode analisis, dan data mentah terhubung secara organik dalam satu proyek.
* **Base-Delta Logging:** Tidak setiap variabel dicatat. Baseline dideklarasikan, dan hanya variabel yang berubah (Deltas) dicatat ringan untuk mencegah penundaan penelitian.
* **Systematic Enforcement:** Melewati batasan panjang nama file (batasan 260 karakter Windows) dan menjamin reproducibility melalui kode.
* **AI Governance:** Memastikan kontinuitas pekerjaan agen AI melalui log handoff `0_Meta/AI_Sync.md`, dan menerapkan standar logging terpadu untuk manusia dan AI melalui `0_Meta/LogConvention.md`.

## Struktur Direktori Proyek

Proyek ini memperlakukan hierarki folder itu sendiri sebagai standar komunikasi.

```text
Project_Root/
├── 0_Meta/                          # Governance proyek & aturan
│   ├── EliRule.md                   # Panduan struktur folder dan operasional
│   ├── LogConvention.md             # Aturan standar logging
│   ├── AI_PARA_Framework.md         # Manajemen konteks AI & aturan pengarsipan
│   └── AI_Sync.md                   # Log handoff agen AI
│
├── 1_Concept/                       # Perencanaan penelitian, literatur, ide
│   ├── 11_Ideas/                    # Sketsa kasar, proposal hipotesis
│   ├── 12_Literature/               # PDF makalah, info bibliografi, rumus dasar
│   └── 13_Planning/                 # Roadmap penelitian, storyboard komposisi gambar
│
├── 2_HW/                            # Desain hardware
│   ├── 21_Component/                # Spesifikasi komponen individual, desain perangkat unit
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Desain perangkat terintegrasi, perumahan, model 3D
│   └── 23_Elec/                     # Sirkuit PCB, Gerber, BOM, Datasheet
│
├── 3_Fab/                           # Fabrikasi dan pemrosesan
│   ├── 31_Recipes/                  # Dokumentasi kondisi proses
│   └── 32_Eval/                     # Evaluasi karakteristik tunggal per-modul
│
├── 4_SW/                            # Perangkat lunak & firmware
│   ├── 41_FW/                       # Firmware MCU/embedded
│   ├── 42_DAQ/                      # Sistem akuisisi data PC/mobile
│   └── 43_Libs/                     # Pustaka bersama yang dapat digunakan kembali
│
├── 5_Exp/                           # Eksperimen: simulasi + empiris + analisis
│   ├── 51_Sim/                      # Simulasi
│   │   ├── Scripts/                 # Kode simulasi (S###_sim.m)
│   │   └── Data/                    # Hasil simulasi (Data/S###/)
│   ├── 52_Empirical/                # Data empiris
│   │   ├── Raw/                     # Data sensor mentah (Read-Only, dikecualikan dari Git)
│   │   └── Processed/               # Data yang sudah diproses utama
│   ├── 53_Analysis/                 # Analisis terintegrasi
│   │   ├── Scripts/                 # Kode post-processing perbandingan/validasi
│   │   └── Logs/                    # Log sesi (S###_log.md)
│   └── 54_Viz/                      # Output visualisasi (gambar yang dibuat secara otomatis)
│
└── 6_Paper/                         # Makalah & presentasi
    ├── 61_Figs/                     # Gambar untuk makalah
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Naskah (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Materi presentasi (PPT, poster)
```

> Untuk penggunaan terperinci dan aturan operasional untuk setiap folder, lihat `0_Meta/EliRule.md`.

## Spesifikasi Pipeline Logging Data

### 1. Konvensi Penamaan File (Penamaan Session-Trial)

* Menampilkan kondisi eksperimental atau informasi variabel dalam nama file **sangat dilarang**.
* **Format:** `[SessionID]_[TrialID].[extension]` (misalnya, `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Logging (Hybrid Logging)

* **Running Log (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * File markdown naratif yang mencatat siklus hipotesis-test-pelajaran langsung dalam teks.
  * Ditulis per trial (`t1`, `t2`...) dalam gaya stream-of-consciousness, mencatat hanya **variabel yang sengaja diubah (Delta)** dan hasil yang diamati.
  * Format dan aturan terperinci: lihat `0_Meta/LogConvention.md`.

### 3. Aturan Dokumen Perencanaan

* Roadmap penelitian, komposisi gambar, strategi eksperimental, dll. dikelola terpisah di `1_Concept/13_Planning/`.
* **Format:** `P###_title.md` (misalnya, `P001_wavelength_optimization.md`)
* Saat mereferensikan Planning dari log: `→ lihat 1_Concept/13_Planning/P###_xxx.md`

### 4. Spesifikasi Analisis Post-Processing (Cell Mode Scripting)

* Kode analisis harus terletak di `5_Exp/53_Analysis/Scripts/` atau `5_Exp/51_Sim/Scripts/` dan tidak boleh dicampur di dalam folder data.
* File `.m` murni digunakan alih-alih `.mlx` untuk mencegah vendor lock-in.
* Kode dieksekusi bagian demi bagian menggunakan `%%` (Cell Mode), dan insight yang diperoleh dicerminkan dalam running log.
* Output analisis (gambar, file mat) disimpan di `5_Exp/54_Viz/` atau `5_Exp/52_Empirical/Processed/S###/` dalam folder per-sesi.

### 5. Aturan Referensi Silang

Format referensi silang disatukan untuk memastikan traceability antar dokumen proyek.

| Dari → Ke | Format |
|-----------|--------|
| Logs → Planning | `→ lihat 1_Concept/13_Planning/P###_xxx.md` |
| Logs → Sim Data | `→ lihat 5_Exp/51_Sim/Data/S###/` |
| Logs → Script | `→ lihat 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planning → Logs | `← dilacak di 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI Governance

Ketika agen AI (Claude, Gemini, dll.) berpartisipasi dalam proyek, aturan berikut berlaku:

1. **Context Acquisition:** Sebelum memulai pekerjaan, baca `0_Meta/AI_Sync.md` untuk mengkonfirmasi keadaan pekerjaan sebelumnya.
2. **Unified Standard Compliance:** Ikuti aturan logging di `0_Meta/LogConvention.md` dengan cara yang sama seperti peneliti manusia.
3. **Handoff Recording:** Setelah penyelesaian tugas, catat tindakan yang dilakukan, file yang dibuat/dimodifikasi, dan Next Steps di `0_Meta/AI_Sync.md`. Tulis dalam urutan kronologi terbalik dengan entri terbaru di bagian atas.
4. **Idea Separation:** Hipotesis dan ide yang dihasilkan oleh AI disimpan terpisah di `1_Concept/11_Ideas/`, bukan dalam log.
5. **PARA-Based Context Management:** Gunakan folder `9_Archive/` dan `.claudeignore` untuk mencegah kontaminasi konteks AI. Untuk aturan terperinci, lihat `0_Meta/AI_PARA_Framework.md`.
6. **Communication Rules:** Pertahankan gaya penulisan yang objektif dan kering. Tidak ada analogi atau metafora. Berikan kesimpulan secara jelas dan langsung. Tidak ada berlebihan atau modifier emosional. Untuk aturan terperinci, lihat bagian 3 dari `0_Meta/EliRule.md`.
7. **Data Reusability:** Saat membuat Plot/Graph apa pun, simpan Data Array asli sebagai `.mat`/`.csv`. Untuk aturan terperinci, lihat bagian 2.6 dari `0_Meta/EliRule.md`.

## Quick Start

Untuk membuat proyek baru dengan struktur ELF v2, jalankan `0_Meta/ELF_generator.ps1`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.ps1
```

Masukkan nama proyek dan struktur folder 0–6, dokumen meta, `.gitignore`, dan inisialisasi Git semuanya akan diselesaikan secara otomatis.

## Lisensi

Proyek ini menerapkan kebijakan Dual License karena sifat "kode yang dapat dijalankan" dan "spesifikasi struktur data (Protokol)" berbeda.

* **Software & Scripts:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Berlaku untuk:** Semua kode sumber (`.m`, `.py`, dll.) dalam folder `4_SW/` dan `5_Exp/*/Scripts/`.
  * **Kondisi:** Jika skrip inti template dimodifikasi dan ditingkatkan untuk redistribusi, modifikasi tersebut harus dirilis sebagai open source. Namun, algoritma unik atau data mentah yang ditambahkan oleh pengguna dalam proyek dapat tetap pribadi (dikomersialisasi).

* **Protocol & Documentation:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Berlaku untuk:** `README.md`, dokumen `0_Meta/`, hierarki folder Session-Trial, aturan metadata logging Base-Delta, dan metodologi penelitian keseluruhan.
  * **Kondisi:** Siapa pun dapat dengan bebas mengadopsi dan beradaptasi dengan struktur ini dan metodologi perekaman, tetapi saat menerbitkan template turunan atau output penelitian terkait, penulis asli Eli (projectschnee@gmail.com) dan repositori sumber harus dikreditkan.
