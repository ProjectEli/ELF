[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Italiano](README.it.md) | [Português](README.pt-BR.md) | [Русский](README.ru.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [ภาษาไทย](README.th.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Bahasa Indonesia](README.id.md)

# Eli's Lab Framework (ELF): Giao Thức Base-Delta cho Nghiên Cứu Agile

Tiêu chuẩn ghi nhật ký dữ liệu tích hợp phần cứng-phần mềm-thí nghiệm (Giao Thức) được thiết kế để hỗ trợ các vòng phản hồi nhanh chóng (Agile) trong giai đoạn phát triển thiết bị và xác thực R&D. Đảm bảo tính truy xuất đầy đủ dữ liệu đồng thời giảm thiểu mệt mỏi trong tài liệu của nhà nghiên cứu.

## Triết Lý Cơ Bản

* **Nguồn Duy Nhất Sự Thật:** Thiết kế phần cứng, mã phân tích và dữ liệu thô được kết nối hữu cơ trong một dự án duy nhất.
* **Ghi Nhật Ký Base-Delta:** Không phải mọi biến đều được ghi lại. Một Baseline được khai báo, và chỉ các biến thay đổi (Deltas) được ghi nhẹ nhàng để tránh trì hoãn nghiên cứu.
* **Thực Thi Có Hệ Thống:** Vượt qua hạn chế độ dài tên tệp (giới hạn 260 ký tự của Windows) và đảm bảo khả năng tái tạo lại thông qua mã.
* **Quản Lý AI:** Đảm bảo tính liên tục của công việc đại lý AI thông qua nhật ký bàn giao `0_Meta/AI_Sync.md`, và thực thi tiêu chuẩn ghi nhật ký thống nhất cho cả con người và AI thông qua `0_Meta/LogConvention.md`.

## Cấu Trúc Thư Mục Dự Án

Dự án này coi phân cấp thư mục như một tiêu chuẩn giao tiếp.

```text
Project_Root/
├── 0_Meta/                          # Quản lý dự án & quy tắc
│   ├── EliRule.md                   # Hướng dẫn cấu trúc thư mục và hoạt động
│   ├── LogConvention.md             # Quy tắc tiêu chuẩn ghi nhật ký
│   ├── AI_PARA_Framework.md         # Quy tắc quản lý & lưu trữ ngữ cảnh AI
│   └── AI_Sync.md                   # Nhật ký bàn giao đại lý AI
│
├── 1_Concept/                       # Kế hoạch nghiên cứu, tài liệu, ý tưởng
│   ├── 11_Ideas/                    # Phác thảo sơ khai, đề xuất giả thuyết
│   ├── 12_Literature/               # PDF bài báo, thông tin tài liệu tham khảo, công thức cơ bản
│   └── 13_Planning/                 # Lộ trình nghiên cứu, sơ đồ thành phần hình ảnh
│
├── 2_HW/                            # Thiết kế phần cứng
│   ├── 21_Component/                # Thông số kỹ thuật thành phần riêng lẻ, thiết kế thiết bị đơn vị
│   │   ├── Design/
│   │   └── Calibration/
│   ├── 22_System/                   # Thiết kế thiết bị tích hợp, vỏ, mô hình 3D
│   └── 23_Elec/                     # Sơ đồ PCB, Gerber, BOM, Bảng tính
│
├── 3_Fab/                           # Chế tạo và xử lý
│   ├── 31_Recipes/                  # Tài liệu điều kiện quy trình
│   └── 32_Eval/                     # Đánh giá đặc tính đơn lẻ theo mô-đun
│
├── 4_SW/                            # Phần mềm & firmware
│   ├── 41_FW/                       # MCU/firmware nhúng
│   ├── 42_DAQ/                      # Hệ thống thu dữ liệu PC/di động
│   └── 43_Libs/                     # Thư viện chia sẻ có thể tái sử dụng
│
├── 5_Exp/                           # Thí nghiệm: mô phỏng + thực nghiệm + phân tích
│   ├── 51_Sim/                      # Mô phỏng
│   │   ├── Scripts/                 # Mã mô phỏng (S###_sim.m)
│   │   └── Data/                    # Kết quả mô phỏng (Data/S###/)
│   ├── 52_Empirical/                # Dữ liệu thực nghiệm
│   │   ├── Raw/                     # Dữ liệu cảm biến thô (Chỉ đọc, loại trừ khỏi Git)
│   │   └── Processed/               # Dữ liệu xử lý chính
│   ├── 53_Analysis/                 # Phân tích tích hợp
│   │   ├── Scripts/                 # Mã xử lý hậu kỳ so sánh/xác thực
│   │   └── Logs/                    # Nhật ký phiên (S###_log.md)
│   └── 54_Viz/                      # Kết quả hình ảnh (tự động tạo hình)
│
└── 6_Paper/                         # Bài báo & bài thuyết trình
    ├── 61_Figs/                     # Hình cho bài báo
    │   ├── rawFig/
    │   ├── processedFig/
    │   └── finalFig/
    ├── 62_Drafts/                   # Bản thảo (Word, LaTeX)
    │   └── archive/
    └── 63_Presentations/            # Tài liệu trình bày (PPT, áp phích)
```

> Để biết chi tiết về cách sử dụng và quy tắc vận hành cho từng thư mục, hãy tham khảo `0_Meta/EliRule.md`.

## Thông Số Kỹ Thuật Đường Ống Ghi Nhật Ký Dữ Liệu

### 1. Quy Ước Đặt Tên Tệp (Đặt Tên Phiên-Lần Thử)

* Liệt kê điều kiện thí nghiệm hoặc thông tin biến trong tên tệp **hoàn toàn bị cấm**.
* **Định dạng:** `[SessionID]_[TrialID].[extension]` (ví dụ: `S001_t1.csv`, `S001_t2.bin`)

### 2. Ghi Nhật Ký Base-Delta (Ghi Nhật Ký Lai)

* **Nhật Ký Chạy (`5_Exp/53_Analysis/Logs/S###_log.md`):**
  * Một tệp markdown tường thuật ghi lại các chu kỳ giả thuyết-kiểm thử-bài học ngay lập tức dưới dạng văn bản.
  * Viết mỗi lần thử (`t1`, `t2`...) theo kiểu dòng ý thức, chỉ ghi lại các **biến thay đổi có ý định (Delta)** và kết quả quan sát.
  * Định dạng và quy tắc chi tiết: tham khảo `0_Meta/LogConvention.md`.

### 3. Quy Tắc Tài Liệu Kế Hoạch

* Lộ trình nghiên cứu, thành phần hình ảnh, chiến lược thí nghiệm, v.v. được quản lý riêng biệt trong `1_Concept/13_Planning/`.
* **Định dạng:** `P###_title.md` (ví dụ: `P001_wavelength_optimization.md`)
* Khi tham chiếu Kế hoạch từ nhật ký: `→ see 1_Concept/13_Planning/P###_xxx.md`

### 4. Thông Số Kỹ Thuật Phân Tích Xử Lý Hậu Kỳ (Lập Trình Chế Độ Ô)

* Mã phân tích phải được đặt trong `5_Exp/53_Analysis/Scripts/` hoặc `5_Exp/51_Sim/Scripts/` và không được trộn lẫn bên trong các thư mục dữ liệu.
* Sử dụng các tệp `.m` thuần túy thay vì `.mlx` để tránh khóa nhà cung cấp.
* Mã được thực thi từng phần bằng `%%` (Chế Độ Ô), và những hiểu biết suy ra được phản ánh trong nhật ký chạy.
* Kết quả phân tích (hình, tệp mat) được lưu trong `5_Exp/54_Viz/` hoặc `5_Exp/52_Empirical/Processed/S###/` trong các thư mục cho mỗi phiên.

### 5. Quy Tắc Tham Chiếu Chéo

Các định dạng tham chiếu chéo được thống nhất để đảm bảo tính truy xuất giữa các tài liệu dự án.

| Từ → Đến | Định Dạng |
|-----------|--------|
| Nhật Ký → Kế Hoạch | `→ see 1_Concept/13_Planning/P###_xxx.md` |
| Nhật Ký → Dữ Liệu Mô Phỏng | `→ see 5_Exp/51_Sim/Data/S###/` |
| Nhật Ký → Kịch Bản | `→ see 5_Exp/53_Analysis/Scripts/S###_analysis.m` |
| Kế Hoạch → Nhật Ký | `← tracked in 5_Exp/53_Analysis/Logs/S###_log.md` |

## Quản Lý AI

Khi các đại lý AI (Claude, Gemini, v.v.) tham gia vào dự án, các quy tắc sau áp dụng:

1. **Thu Thập Ngữ Cảnh:** Trước khi bắt đầu công việc, đọc `0_Meta/AI_Sync.md` để xác nhận trạng thái công việc trước đó.
2. **Tuân Thủ Tiêu Chuẩn Thống Nhất:** Tuân theo các quy tắc ghi nhật ký trong `0_Meta/LogConvention.md` theo cách giống như một nhà nghiên cứu con người.
3. **Ghi Nhận Bàn Giao:** Khi hoàn thành nhiệm vụ, ghi lại các hành động được thực hiện, tệp được tạo/sửa đổi và Các Bước Tiếp Theo trong `0_Meta/AI_Sync.md`. Viết theo thứ tự thời gian ngược với mục nhập gần đây nhất ở trên cùng.
4. **Tách Biệt Ý Tưởng:** Giả thuyết và ý tưởng được tạo bởi AI được lưu trữ riêng biệt trong `1_Concept/11_Ideas/`, không phải trong nhật ký.
5. **Quản Lý Ngữ Cảnh Dựa Trên PARA:** Sử dụng thư mục `9_Archive/` và `.claudeignore` để ngăn chặn ô nhiễm ngữ cảnh AI. Để biết quy tắc chi tiết, hãy tham khảo `0_Meta/AI_PARA_Framework.md`.
6. **Quy Tắc Giao Tiếp:** Duy trì phong cách viết khách quan và khô khan. Không có phép ẩn dụ hay ẩn dụ. Đưa ra kết luận rõ ràng và trực tiếp. Không phóng đại hoặc bổ sung cảm xúc. Để biết quy tắc chi tiết, hãy tham khảo phần 3 của `0_Meta/EliRule.md`.
7. **Khả Năng Tái Sử Dụng Dữ Liệu:** Khi tạo bất kỳ Biểu Đồ/Đồ Thị nào, hãy lưu Mảng Dữ Liệu Gốc cùng với dưới dạng `.mat`/`.csv`. Để biết quy tắc chi tiết, hãy tham khảo phần 2.6 của `0_Meta/EliRule.md`.

## Bắt Đầu Nhanh

Để tạo dự án mới với cấu trúc ELF v2, chạy `0_Meta/ELF_generator.bat`.

```
cd desired_parent_directory
D:\...\ELF\0_Meta\ELF_generator.bat
```

Nhập tên dự án và cấu trúc thư mục 0–6, tài liệu meta, `.gitignore` và khởi tạo Git sẽ được hoàn thành tự động.

## Giấy Phép

Dự án này áp dụng chính sách Giấy Phép Kép vì tính chất "mã thực thi" và "thông số kỹ thuật cấu trúc dữ liệu (Giao Thức)" khác nhau.

* **Phần Mềm & Kịch Bản:** [Mozilla Public License 2.0 (MPL 2.0)](https://www.mozilla.org/en-US/MPL/2.0/)
  * **Áp dụng cho:** Tất cả mã nguồn (`.m`, `.py`, v.v.) trong các thư mục `4_SW/` và `5_Exp/*/Scripts/`.
  * **Điều Kiện:** Nếu các kịch bản lõi mẫu được sửa đổi và cải thiện để phân phối lại, những sửa đổi đó phải được phát hành dưới dạng mã nguồn mở. Tuy nhiên, các thuật toán duy nhất hoặc dữ liệu thô được người dùng thêm vào trong dự án có thể vẫn ở tư nhân (thương mại hóa).

* **Giao Thức & Tài Liệu:** [Creative Commons Attribution 4.0 (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/)
  * **Áp dụng cho:** `README.md`, tài liệu `0_Meta/`, phân cấp thư mục Phiên-Lần Thử, quy tắc ghi nhật ký siêu dữ liệu Base-Delta và phương pháp luận nghiên cứu tổng thể.
  * **Điều Kiện:** Bất kỳ ai cũng có thể tự do áp dụng và điều chỉnh cấu trúc này và phương pháp ghi nhận, nhưng khi xuất bản các mẫu dẫn xuất hoặc đầu ra nghiên cứu liên quan, tác giả gốc Eli (projectschnee@gmail.com) và kho lưu trữ nguồn phải được ghi công.
