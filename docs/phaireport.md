# `.phaireport`: รายงานพิมพ์ (Report Designer)

2026-11-20 · รุ่นรูปแบบ `format = 1` · โค้ด: `crates/phai-ui/src/report.rs` (ตัววางหน้าและวาด),
`crates/phai-form/src/report.rs` (ไฟล์ + สร้างโค้ด), `apps/studio/src/repdesign.rs` (ตัวออกแบบ)

แนวเดียวกับ FastReport / DevExpress XtraReports / RDLC ของ Visual Studio แต่ฟรีและเป็น Rust ล้วน:
รายงานคือ **แถบ (band)** เรียงจากบนลงล่าง แต่ละแถบมี **รายการ (item)** วางตำแหน่งเป็นร้อยส่วนของนิ้ว
(หน่วยเดียวกับ `phai_ui::print`) แล้วตัววางหน้าเอาแถวจากตารางมาเรียงลงหน้ากระดาษ

## แถบ

| `kind` | พิมพ์เมื่อไร |
|---|---|
| `report_header` | ครั้งเดียว ต้นรายงาน (ชื่อรายงาน) - ตามด้วยหัวกระดาษของหน้าแรก |
| `page_header` | บนสุดของทุกหน้า (หัวคอลัมน์) |
| `group_header` | ก่อนแถวของแต่ละกลุ่ม (`group = 0` กลุ่มนอกสุด) · `new_page = true` ขึ้นหน้าใหม่ทุกกลุ่ม |
| `detail` | ทุกแถว |
| `group_footer` | หลังแถวของแต่ละกลุ่ม (ผลรวมของกลุ่ม) |
| `page_footer` | ล่างสุดของทุกหน้า (เลขหน้า วันที่) |
| `report_footer` | ครั้งเดียว ท้ายรายงาน (ผลรวมทั้งหมด) |

แถบที่ไม่พอในหน้าที่เหลือขึ้นหน้าใหม่ หน้าใหม่เริ่มด้วยหัวกระดาษเสมอ

## รายการ

| `kind` | คืออะไร |
|---|---|
| `label` | ข้อความ - เติม `{page}` `{pages}` `{date}` `{time}` `{row}` `{title}` และ `{ชื่อคอลัมน์}` ของแถวนั้นให้ |
| `field` | ค่าของคอลัมน์ `field` ในแถว |
| `total` | `func` = `sum` (ค่าเริ่ม) / `count` / `avg` / `min` / `max` ของคอลัมน์ `field` - นับแถวของกลุ่มเมื่ออยู่ในแถบกลุ่ม, ของหน้าเมื่ออยู่ในท้ายกระดาษ, ทั้งรายงานเมื่ออยู่ในหัว/ท้ายรายงาน |
| `line` | เส้นจากมุมซ้ายบนถึงขวาล่างของ `bounds` (สูง 0 = เส้นนอน) |
| `box` | กรอบ |
| `picture` | รูปจากไฟล์ `image` |

คุณสมบัติของรายการ: `name`, `bounds = [x, y, กว้าง, สูง]` (จากมุมของแถบ), `text`, `field`, `func`,
`format` (`#,##0.00`, `0`, `฿#,##0` - ปัดครึ่งขึ้น), `align` (`left`/`center`/`right`), `font`, `size` (พอยต์),
`bold`, `italic`, `color`, `background`, `border`, `class`, `image` - ไม่ใส่ = ใช้ค่าจาก CSS

## หน้าตา = CSS

เหมือนฟอร์ม: ชื่อชนิดของรายการเป็น selector (`label`, `field`, `total`, `line`, `box`) มี class เป็นชื่อแถบ
(`.page_header`) แถบเองคือ `band` แถวของ detail มี `.odd` / `.even` เพิ่ม ค่าเริ่มอยู่ใน `REPORT_CSS`
(หัวรายงานตัวใหญ่สีน้ำเงินเข้ม หัวคอลัมน์ตัวหนามีเส้นใต้ แถวสลับสีจาง ผลรวมตัวหนา) แต่งทับได้ใน `css`:

```toml
[report]
css = """
band.group_footer { background: #fdf6e3; }
label.report_header { color: #12855f; }
total.money { color: #b91c1c; }
"""
```

## ข้อมูล

```toml
[data]
database = "main"              # SQLite ใช้ main
table = "orders"
columns = ["region", "customer", "amount"]   # ไม่ใส่ = คอลัมน์ที่รายการใช้
filter = "amount > 0"          # WHERE
order = ["customer", "amount desc"]          # เรียงต่อจากกลุ่ม
# sql = "SELECT ..."           # หรือ SQL เอง (ต้องเรียงตามกลุ่มเอง)
connection = "SQLite · shop.db"              # สำหรับตัวออกแบบเท่านั้น: การเชื่อมต่อใน Server Explorer ที่ใช้ดูตัวอย่าง
```

โปรแกรมอ่านแถวด้วย `crate::data_sources::connection()` (ไฟล์เดียวกับ grid ที่ผูกตาราง) และ
`Connection::query_all` - ทุกแถว ไม่ตัดที่ 1000 แถวแบบ `query` · `[report] groups = ["region"]` เรียงแถวตามกลุ่มให้

## ตัวอย่างเต็ม

```toml
format = 1

[report]
name = "SalesReport"
title = "ยอดขายตามภาค"
groups = ["region"]

[data]
database = "main"
table = "orders"
columns = ["region", "customer", "amount"]

[[band]]
kind = "group_header"
height = 26
[[band.item]]
kind = "label"
name = "g_region"
bounds = [0, 4, 400, 20]
text = "ภาค: {region}"

[[band]]
kind = "detail"
height = 20
[[band.item]]
kind = "field"
name = "customer"
bounds = [20, 1, 300, 18]
field = "customer"
[[band.item]]
kind = "field"
name = "amount"
bounds = [560, 1, 160, 18]
field = "amount"
format = "#,##0.00"
align = "right"

[[band]]
kind = "group_footer"
height = 26
[[band.item]]
kind = "total"
name = "region_total"
bounds = [560, 4, 160, 18]
field = "amount"
format = "#,##0.00"
align = "right"
```

## โค้ดที่สร้าง

บันทึกแล้ว Studio เขียน `src/<ชื่อ>.rs` ทั้งไฟล์ทุกครั้ง (ห้ามแก้มือ) และใส่ `mod <ชื่อ>;` ใน `main.rs`
ถ้าผูกตาราง ใส่ `src/data_sources.rs` และ phai-data ใน Cargo.toml ให้เหมือน grid ที่ผูกตาราง

```rust
sales_report::preview(ui.hwnd)?;                 // หน้าต่างดูตัวอย่าง (มีปุ่มพิมพ์)
sales_report::print(ui.hwnd)?;                   // หน้าต่าง Print ของ Windows แล้วพิมพ์
sales_report::export_pdf(ui.hwnd, Path::new("sales.pdf"))?;   // ผ่าน Microsoft Print to PDF
// หรือแถวของตัวเอง:
sales_report::report().preview(ui.hwnd, phai_ui::report::Data::new(columns, rows));
```

`phai_ui::report::Report::layout(&data, area, &now)` คืนหน้าที่วางแล้ว (`Page::texts()`) โดยไม่วาด -
ใช้เขียนเทสต์ของรายงานได้

## ตัวออกแบบ (Report Designer)

Project > Add Report... หรือดับเบิลคลิกไฟล์ `.phaireport` ในหน้าต่าง Project เปิดหน้าต่างของตัวเอง
(Studio ใช้ต่อได้) · `phai-studio --report ไฟล์.phaireport` เปิดตัวออกแบบอย่างเดียว

- แถบเรียงแบบ XtraReports มีหัวบอกชื่อแถบ ลากรายการเพื่อย้าย (ลากไปปล่อยในแถบอื่นได้) ลากมุมขวาล่างเพื่อปรับขนาด
  ลากขอบล่างของแถบเพื่อปรับความสูง ตำแหน่งเกาะทีละ 5 ร้อยส่วนนิ้ว ลูกศรขยับทีละ 1 (Shift = ขนาด, Ctrl = 10)
  Delete ลบ, Ctrl+Z / Ctrl+Y, Ctrl+S
- ซ้าย: คอลัมน์ของตาราง ดับเบิลคลิก = ฟิลด์ในแถบ detail + หัวคอลัมน์ใน page header
- ขวา: คุณสมบัติของสิ่งที่เลือก (รายงาน / แถบ / รายการ) ดับเบิลคลิกเพื่อแก้ ค่าที่มีตัวเลือกขึ้นเป็นเมนู
- ปุ่ม: Label, Field, Total (ผลรวมของคอลัมน์ที่เลือกไว้ ลงท้ายกลุ่มหรือท้ายรายงาน), Line, Box, Picture ·
  Data... (เลือกการเชื่อมต่อ ฐานข้อมูล ตาราง จาก Server Explorer - แถบ detail ว่างจะวางทุกคอลัมน์ให้) ·
  Groups... (จัดกลุ่มตามคอลัมน์ / เอากลุ่มออก) · Bands... (เปิดปิดหัว/ท้ายรายงานและหัว/ท้ายกระดาษ) ·
  Preview (แถวจริงจากฐานข้อมูล) · Print... · PDF... · Save
- ดูตัวอย่างใช้การเชื่อมต่อใน Server Explorer ที่ชื่อตรง `connection` ถ้าไม่มี ใช้ไฟล์ SQLite ที่
  `src/data_sources.rs` ของโปรเจกต์บอก

## ยังไม่มี (รอบต่อไป)

ข้อความยาวที่ขยายแถบเอง (CanGrow), หัวกลุ่มซ้ำบนหน้าใหม่, subreport / master-detail, พารามิเตอร์ตอนเรียก
(เช่นช่วงวันที่), นิพจน์คำนวณ (`qty * price`), กราฟในรายงาน, บาร์โค้ด, ส่งออก Excel, ย่อ/ขยายในตัวออกแบบ,
ตัวออกแบบเป็นแท็บเอกสารใน Studio (ตอนนี้เป็นหน้าต่างแยก), รายงานบนเว็บ (phai-web)
