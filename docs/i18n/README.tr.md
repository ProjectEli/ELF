[English](../../README.md) | [한국어](../../README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Base-Delta Protokolü Agile R&D için

Cihaz geliştirme ve R&D doğrulama aşamalarında hızlı geri bildirim döngülerini (Agile) desteklemek için tasarlanmış donanım-yazılım-deneysel veri entegre günlüğü standardı (Protokol). Araştırmacının belgeleme yorgunluğunu en aza indirirken eksiksiz veri izlenebilirliğini garantir.

## Temel Felsefe

* **Tek Gerçek Kaynağı (Single Source of Truth):** Donanım tasarımı, analiz kodu ve ham veriler tek bir proje içinde organik olarak bağlantılıdır.
* **Base-Delta Günlüğü:** Her değişken kaydedilmez. Bir Baseline belirlenir ve yalnızca değişen değişkenler (Deltalar) hafifçe kaydedilerek araştırma gecikmelerinin önüne geçilir.
* **Sistematik Zorunluluk:** Dosya adı uzunluğu sınırlamalarını (Windows 260 karakter sınırı) aşar ve kod aracılığıyla tekrarlanabilirliği garantir.
* **AI Yönetişimi:** `0_Meta/AI_Sync.md` devretme günlüğü aracılığıyla AI aracısının iş sürekliliğini garanti eder ve `0_Meta/LogConvention.md` ile insan ve AI tarafından birleştirilmiş günlüğe uyulmasını zorlar.

## Proje Dizin Yapısı

Bu proje, aşağıdaki klasör hiyerarşisini bir iletişim standardı olarak ele alır.

```text
Project_Root/
├── 0_Meta/                          # Proje yönetişimi & kuralları
│   ├── EliRule.md                   # Klasör yapısı ve operasyon rehberi
│   ├── LogConvention.md             # Günlüğe kaydetme standart kuralları
│   ├── AI_PARA_Framework.md         # AI bağlam yönetimi & arşivleme kuralları
│   └── AI_Sync.md                   # AI aracısı devretme günlüğü
│
├── 1_Concept/                       # Araştırma planlaması, literatür, fikirler
│   ├── 11_Ideas/                    # Kaba taslaklar, hipotez önerileri
│   ├── 12_Literature/               # Makale PDF'leri, kaynakça bilgisi, temel formüller
│   └── 13_Planning/                 # Araştırma yol haritaları, şekil oluşturma storyboard'ları
│       └── 2_Wiki/                  # Damıtılmış planlama sonuçları ve temel kurallar
│
├── 2_HW/                            # Donanım tasarımı
│   ├── 21_Component/                # Bireysel bileşen özellikleri, birim cihaz tasarımı
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Entegre cihaz tasarımı, kasa, 3B modeller
│   └── 23_Elec/                     # PCB şemaları, Gerber, BOM, Veri Sayfaları
│
├── 3_Fab/                           # Üretim ve işleme
│   ├── 31_Recipes/                  # İşlem koşulu belgelendirmesi
│   └── 32_Eval/                     # Modül başına tek karakteristik değerlendirmesi
│
├── 4_SW/                            # Yazılım & Firmware
│   ├── 41_FW/                       # MCU/gömülü firmware
│   ├── 42_DAQ/                      # PC/mobil veri toplama sistemleri
│   └── 43_Libs/                     # Tekrar kullanılabilir paylaşılan kütüphaneler
│
├── 5_Exp/                           # Deneyleme: simülasyon + ampirik + analiz
│   ├── 51_Sim/                      # Simülasyon
│   │   ├── Scripts/                 # Simülasyon kodu (S###_sim.m)
│   │   │   └── 9_Archive/          # Emekli edilmiş betikler
│   │   └── Data/                    # Simülasyon sonuçları (Data/S###/)
│   ├── 52_Empirical/                # Ampirik veriler
│   │   ├── Raw/                     # Ham sensör verisi (Salt Okunur, Git'ten Hariç)
│   │   └── Processed/               # Birincil işlenmiş veriler
│   ├── 53_Analysis/                 # Entegre analiz
│   │   ├── Scripts/                 # Karşılaştırma/doğrulama son işleme kodu
│   │   │   └── 9_Archive/          # Emekli edilmiş betikler
│   │   └── Logs/                    # Oturum günlükleri (S###_log.md)
│   │       ├── 2_Wiki/              # Damıtılmış bulgular ve oturum kaydı
│   │       └── 9_Archive/           # Tamamlanmış oturum günlükleri
│   └── 54_Viz/                      # Görselleştirme çıktıları (otomatik oluşturulan şekiller)
│
└── 6_Paper/                         # Makaleler & sunumlar
    ├── 61_Figs/                     # Makaleler için şekiller
    │   ├── Raw/
    │   ├── Processed/
    │   └── Final/
    ├── 62_Drafts/                   # Yazılı eserler (Word, LaTeX)
    │   └── 9_Archive/                # Önceki sürümler
    └── 63_Presentations/            # Sunum materyalleri (PPT, posterler)
```

> Her klasörün ayrıntılı kullanım ve operasyon kuralları için `0_Meta/EliRule.md`'ye başvurun.

## Veri Günlüğü Pipeline'ı Spesifikasyonu

### 1. Dosya Adlandırma Kuralı (Oturum-Deneme Adlandırması)

* Dosya adlarında deneysel koşulları veya değişken bilgilerini listelemek **kesinlikle yasaktır**.
* **Format:** `[OturumID]_[DenemeID].[uzantı]` (örneğin `S001_t1.csv`, `S001_t2.bin`)

### 2. Base-Delta Günlüğü (Hibrit Günlüğe Alma)

* **Çalışan Günlük (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Acil hipotez-test-ders döngülerini metinde kaydeden anlatı tarzı bir markdown dosyasıdır.
  * Deneme başına (`t1`, `t2`...) bilinç akışı tarzında yazılır, yalnızca **kasıtlı olarak değiştirilmiş değişkenleri (Delta)** ve gözlemlenen sonuçları kaydeder.
  * Format ve ayrıntılı kurallar: `0_Meta/LogConvention.md`'ye bakın.

### 3. Planning Belge Kuralları

* Araştırma yol haritaları, şekil oluşturma, deneysel stratejiler vb. `1_Concept/13_Planning/` içinde ayrı olarak yönetilir.
* **Format:** `P###_başlık.md` (örneğin `P001_wavelength_optimization.md`)
* Günlükten Planning'e referans verirken: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Son İşleme Analizi Spesifikasyonu (Hücre Modu Komut Dosyası)

* Analiz kodu `5_Exp/53_Analysis/Scripts/` veya `5_Exp/51_Sim/Scripts/` içinde yer almalı ve veri klasörleri içinde karışık kullanılamaz.
* Satıcı bağlantısından kaçınmak için `.mlx` yerine salt `.m` dosyaları kullanılır.
* Kod, `%%` (Hücre Modu) kullanılarak bölüm bölüm çalıştırılır ve türetilmiş içgörüler çalışan günlükte yansıtılır.
* Analiz çıktıları (şekiller, mat dosyaları) `5_Exp/54_Viz/` veya `5_Exp/52_Empirical/Processed/S###/` içinde oturum başına klasörler oluşturularak kaydedilir.

### 5. Çapraz Referans Kuralları

Proje belgeleri arasında izlenebilirliği sağlamak için çapraz referans formatları birleştirilir.

| From → To | Format |
|-----------|--------|
| Günlükler → Planning | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Günlükler → Sim Verisi | `→ see 5_Exp/51_Sim/Data/S###/` |
| Günlükler → Komut Dosyası | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Planning → Günlükler | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## AI Yönetişimi

AI aracıları (Claude, Gemini vb.) projeye katıldığında aşağıdaki kurallar geçerli olur:

1. **Bağlam Sağlanması:** Çalışmaya başlamadan önce önceki çalışmanın durumunu doğrulamak için `0_Meta/AI_Sync.md`'yi okuyun.
2. **Birleştirilmiş Standart Uyumu:** `0_Meta/LogConvention.md` içindeki günlüğe kaydetme kurallarını insan araştırmacı gibi izleyin.
3. **Devretme Kaydı:** Görev tamamlandığında, gerçekleştirilen eylemleri, oluşturulan/değiştirilen dosyaları ve Sonraki Adımları `0_Meta/AI_Sync.md` içinde kaydedin. En son girişin üstte olması için ters kronolojik sırada yazın.
4. **Fikir Ayrımı:** AI tarafından üretilen hipotezler ve fikirler günlüklerde değil `1_Concept/11_Ideas/` içinde ayrı olarak depolanır.
5. **PARA Tabanlı Bağlam Yönetimi:** AI bağlamı kirlenmesini önlemek için `9_Archive/` klasörü ve `.claudeignore`'u kullanın. Ayrıntılı kurallar için `0_Meta/AI_PARA_Framework.md`'ye bakın.
6. **İletişim Kuralları:** Nesnel ve kuru yazı tarzı koruyun. Benzetme ve metafor yok. Sonuç odaklı net iletişim. Abartı ve duygusal değiştiriciler yok. Ayrıntılı kurallar için `0_Meta/EliRule.md` bölüm 3'e bakın.
7. **Veri Tekrar Kullanılabilirliği:** Herhangi bir Plot/Grafik oluştururken orijinal Veri Dizisini `.mat`/`.csv` olarak birlikte kaydedin. Ayrıntılı kurallar için `0_Meta/EliRule.md` bölüm 2.6'ya bakın.

## Hızlı Başlangıç

ELF v2 yapısı ile yeni bir proje oluşturmak için `0_Meta/ELF_generator.sh` dosyasını çalıştırın.

```bash
cd istenen_parent_directory
bash /path/to/ELF/0_Meta/ELF_generator.sh
```

> Windows'ta Git Bash kullanın ([Git for Windows](https://git-scm.com/) ile birlikte gelir).

Bir proje adı girin ve 0–6 klasör yapısı, meta belgeler ve `.gitignore` otomatik olarak oluşturulacaktır. Git başlatması isteğe bağlıdır ve yalnızca Git kullanılabilir olduğunda sorulur.

## Lisans

Bu proje "çalıştırılabilir kod" ve "veri yapısı spesifikasyonu (Protokol)"nin doğası farklı olduğu için Çift Lisans politikası uygulanır.

* **Yazılım & Komut Dosyaları:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Uygulanacak:** `4_SW/` ve `5_Exp/*/Scripts/` klasörleri içindeki tüm kaynak kodu (`.m`, `.py` vb.).
  * **Koşul:** Şablon temel komut dosyaları değiştirilip iyileştirilip yeniden dağıtıldığında, bu değişiklikler açık kaynak olarak yayınlanmalıdır. Ancak, kullanıcının proje içine eklediği benzersiz algoritmalar veya ham veriler özel (ticari) kalabilir.

* **Protokol & Belgeler:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Uygulanacak:** `README.md`, `0_Meta/` belgeleri, Oturum-Deneme klasör hiyerarşisi, Base-Delta meta veri günlüğü kuralları ve genel araştırma metodolojisi.
  * **Koşul:** Herkes bu yapı ve kaydetme metodolojisini serbestçe benimseyebilir ve uyarlayabilir, ancak türetilmiş şablonlar veya ilgili araştırma çıktıları yayınlarken orijinal yazar Eli (projectschnee@gmail.com) ve kaynak depo alıntı yapılmalıdır.
