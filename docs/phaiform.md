# `.phaiform`: รูปแบบไฟล์ฟอร์ม และโค้ดที่สร้าง

ขั้น 0 ของ `PLAN.md` · 2026-11-16 · รุ่นรูปแบบ `format = 1`

## หลัก

- **TOML** อ่านง่าย diff ใน git ได้ แก้มือได้ถ้าจำเป็น
- **ข้อความเป็นคีย์อังกฤษ** ผ่าน `t()` ของ `phai-ui` คำแปลไทยอยู่ในตารางของโปรเจกต์ ไม่อยู่ในฟอร์ม
- **ตำแหน่ง/ขนาดเป็น design pixel ที่ 96 DPI** ตอนรันคูณตาม DPI ของจอ (`phai_ui::layout`)
- **หน้าตามาจาก CSS** ฟอร์มบอกแค่ `class`/`id` ของ control สีและเส้นอยู่ในไฟล์ `.css`
- **ไฟล์ที่ designer สร้างห้ามแก้มือ** โค้ดของผู้ใช้อยู่อีกไฟล์ designer ไม่อ่านโค้ดผู้ใช้กลับเข้ามา

## ไฟล์ในโปรเจกต์

```
my_app/
  Cargo.toml
  build.rs                 manifest Common Controls 6 (สร้างให้ครั้งเดียว)
  theme.css                ธีมของโปรแกรม (ไม่ใส่ = ธีมสว่างในตัว)
  src/
    main.rs                ผู้ใช้ (สร้างให้ครั้งเดียว)
    text.rs                ตารางคำแปลไทย ผู้ใช้เติมเอง + เทสต์จับคำที่ยังไม่แปล
    main_window.phaiform   designer เขียน
    main_window_ui.rs      designer เขียนทุกครั้งที่บันทึก — ห้ามแก้
    main_window.rs         ผู้ใช้: event handler (สร้างโครงให้ครั้งแรก)
```

## ตัวอย่าง

```toml
format = 1

[form]
name = "MainWindow"            # ชื่อ struct; ไฟล์เป็น main_window_ui.rs / main_window.rs
title = "Customers"            # คีย์อังกฤษ → t("Customers")
size = [480, 320]              # พื้นที่ client ตอนออกแบบ
min_size = [360, 240]
theme = "theme.css"            # ไม่ใส่ = ธีมในตัว; ตอนพัฒนา phai-ui เฝ้าไฟล์ แก้แล้วเห็นทันที
font_size = 10                 # pt ของทุก control; ไม่ใส่ = ฟอนต์ระบบ 9 pt (control ตั้ง font_size ของตัวเองทับได้)

[[menu]]
label = "&File"
items = [
  { id = "open", label = "Open...", key = "Ctrl+O" },
  { separator = true },
  { id = "exit", label = "Exit" },
]

[[toolbar]]
id = "save"
label = "Save"                 # เป็น tooltip — toolbar มีแต่ไอคอน
glyph = "SAVE"                 # ชื่อใน phai_ui::icon::glyph

[[control]]
kind = "label"
id = "name_label"
text = "Name"
bounds = [11, 11, 60, 23]      # x, y, กว้าง, สูง

[[control]]
kind = "edit"
id = "name"
bounds = [83, 11, 386, 23]
anchor = "left top right"      # ค่าเริ่มต้น "left top"

[[control]]
kind = "list"
id = "customers"
columns = [["Name", 200], ["Phone", 120]]
bounds = [11, 41, 458, 230]
anchor = "all"

[[control]]
kind = "button"
id = "ok"
text = "OK"
class = "primary"              # CSS: button.primary
bounds = [313, 286, 75, 23]
anchor = "right bottom"
on_click = "ok_clicked"        # ชื่อเมธอดในไฟล์ของผู้ใช้

[[control]]
kind = "statusbar"
id = "status"
dock = "bottom"
parts = [120]
```

## ช่องของ control

| ช่อง | ใช้กับ | ความหมาย |
|---|---|---|
| `kind` | ทุกตัว | `label` `edit` `textarea` `button` `checkbox` `radio` `combo` `list` `date` `number` `slider` `progress` `group` `tabs` `statusbar` `tree` `picture` `timer` `flow` `table` `richtext` `grid` `contextmenu` |
| `id` | ทุกตัว | ชื่อ field ใน struct ที่สร้าง และ `#id` ใน CSS — snake_case ห้ามซ้ำ |
| `text` | label, edit, textarea, button, checkbox, radio, group | คีย์อังกฤษ (edit = ข้อความเริ่มต้น ไม่แปล) |
| `bounds` | ทุกตัวที่ไม่ dock | `[x, y, w, h]` design px |
| `anchor` | | `left` `top` `right` `bottom` รวมกันได้, `all`, `none` |
| `dock` | | `top` `bottom` `left` `right` `fill` — ตามลำดับในไฟล์ |
| `class` | | CSS class คั่นด้วยเว้นวรรค |
| `enabled`, `visible` | | ค่าเริ่มต้น `true` |
| `checked` | checkbox, radio | |
| `group` | radio | เลขกลุ่ม radio ที่เลือกได้ทีละตัว |
| `password` | edit | |
| `items` | combo | คีย์อังกฤษ |
| `columns` | list, grid | `[หัว, กว้าง]` |
| `column_props` | grid | ค่าของแต่ละคอลัมน์ตามลำดับ (ดูหัวข้อ Data grid) |
| `tabs` + `page` | tabs / ตัวอื่น | `tabs = ["General", "Advanced"]`; control ที่มี `page = 1` ขึ้นเฉพาะแท็บที่ 2 |
| `parts` | statusbar | ความกว้างของช่องด้านขวา |
| `items` | combo, tree | รายการ; ของ tree เว้นหน้าบรรทัด 2 ช่องต่อหนึ่งชั้น เช่น `["Thailand", "  Bangkok"]` |
| `context_menu` | ทุกตัว | id ของ contextmenu ที่ขึ้นเมื่อคลิกขวา; contextmenu มี `items` เป็น `"เมธอด \| ข้อความ"` (`-` คือเส้นคั่น) |
| `cols` | table | จำนวนคอลัมน์ (ค่าเริ่ม 2); control ที่อยู่ในกรอบ flow/table ถูกจัดวางให้เอง |
| `image` | picture | ไฟล์รูป PNG/JPEG/BMP/GIF จากโฟลเดอร์โปรเจกต์ (ตอนรัน: ข้างโปรแกรม หรือที่ที่รัน) หรือพาธเต็ม |
| `value` | timer | ช่วงเวลาเป็นมิลลิวินาที (ค่าเริ่ม 1000); timer ไม่ต้องมี `bounds` - อยู่ใน component tray ใต้ฟอร์ม |
| `min` `max` `value` | number, slider, progress | ช่วงค่าและค่าเริ่มต้น (ค่าเริ่ม 0, 100, 0) |
| `font_size` | | pt ของ control นี้ ทับของฟอร์ม |
| `bold` | | ตัวหนา — `font-weight: bold` |
| `back_color` `fore_color` `border_color` | | สีแบบ CSS (`#12855f`, `rgb(…)`, `orange`) ทับธีม — `background` `color` `border-color` |
| `radius` | | มุมมน px 0–50 — `border-radius` |
| `on_click` `on_change` `on_select` `on_tick` | | ชื่อเมธอด handler (date, number, slider, textarea ใช้ `on_change`; list, tree ใช้ `on_select`; timer ใช้ `on_tick`) |

designer ตรวจทุกช่องตอนเปิดไฟล์ ช่องที่ไม่รู้จักหรือค่าที่ผิดเป็น error บอกบรรทัด ไม่ข้ามเงียบๆ แบบเดียวกับ CSS

## property แบบ WinForms (ตาราง `crates/phai-form/src/props.rs`)

property อื่นของฟอร์มและ control เขียนเป็น `ชื่อ = ค่า` ธรรมดา ไม่ใส่บรรทัดที่เป็นค่าเริ่มต้น ตารางเดียวบอก
ชื่อ, ชนิดค่า, ค่าเริ่มต้น, ชนิด control ที่มี, หมวดใน Properties และคำอธิบาย — Properties, ตัวตรวจไฟล์,
codegen และ canvas อ่านจากตารางนี้ทั้งหมด control เรียก `phai_ui::props::<ชื่อ>(control, ค่า)` หลังสร้าง

```toml
[form]
border = "fixed_dialog"      # FormBorderStyle: sizable fixed_single fixed_dialog fixed_tool sizable_tool none
accept_button = "ok"         # Enter กดปุ่มนี้; cancel_button = Esc
maximize_box = false         # minimize_box, control_box, top_most, show_in_taskbar
opacity = 95                 # %
icon = "app.ico"             # ไฟล์ในโปรเจกต์ ฝังในโปรแกรมด้วย include_bytes!
min_size = [300, 200]        # max_size
back_color = "#fffbeb"       # fore_color — CSS window { } ของทั้งโปรแกรม

[[control]]
kind = "edit"
id = "name"
read_only = true             # max_length, placeholder, text_align, char_casing, tooltip, cursor, tab_index, tab_stop

[[control]]
kind = "button"
id = "ok"
dialog_result = "ok"         # คลิกแล้วปิดหน้าต่าง; show_dialog() คืน DialogResult::Ok
```

รายการเต็มต่อชนิด control: `docs/winforms-parity.md`

## หน้าตาตามสถานะ และ transition (แท็บ Style 🖌)

```toml
[[control]]
kind = "button"
id = "save"
transition = "200ms ease-out"                                  # สีค่อย ๆ เปลี่ยนเมื่อสถานะเปลี่ยน
animation = "pulse 1.5s infinite"                              # สีเคลื่อนไหวเอง: blink pulse fade-in หรือ @keyframes
states = { hover = "background: #0b5d43", pressed = "color: #ffcc00" }   # = CSS #save:hover { } ...
```

`animation` ของ CSS: `ชื่อ เวลา [รอก่อนเริ่ม] [easing] [จำนวนรอบ | infinite] [alternate]` — สำเร็จรูป `blink`
`pulse` `fade-in` หรือ `@keyframes ชื่อ { from { } 50% { } to { } }` ใน theme.css (ขยับได้เฉพาะสีและ `border-width`
เพราะ control ของ Windows หมุนหรือโปร่งใสไม่ได้) ใช้กับปุ่ม ช่องติ๊ก radio ช่องกรอก slider progress

เงาและไล่สี (แบบ style ของ ImageStyler) ใช้ได้ทั้งใน theme.css, แท็บ Style และ Theme Editor:

```css
button.glass { background: linear-gradient(to bottom, #34c08f, #0e5c41); box-shadow: 0 3px 6px rgba(0,0,0,.35); }
window       { background: linear-gradient(160deg, #ffffff, #e8f1ee); }   /* label, group โปร่งใสให้เห็นพื้น */
```

สถานะ: `hover` `pressed` `focus` `disabled` `checked` — ลำดับชั้น: ธีม → `theme.css` → class → ค่าของ control
(Properties) → ค่าตามสถานะของ control; codegen ส่ง `#id:state { }` ทั้งหมดให้ `phai_ui::theme::set_state_rules`
แท็บ Style แสดงค่าจริงของแต่ละ property ในสถานะที่เลือก (ตั้งเองเป็นตัวหนา) พร้อมที่มา เช่น
"มาจาก button.primary:hover ในธีม บรรทัด 40", แสดง control บนฟอร์มในสถานะนั้น, และปุ่ม "ส่งไปที่ธีม" ย้ายทุกค่าเป็น
`kind.class { }` / `kind.class:hover { }` ต่อท้าย theme.css แล้วให้ control ใช้ class นั้นแทน

## ตัวตรวจสด (Live Inspector) — F12 ในโปรแกรมที่รันอยู่

โปรแกรมที่ build แบบ debug: กด **F12** (หรือ Ctrl+Shift+I) ในหน้าต่าง → ตัวตรวจเปิดข้างหน้าต่าง (`phai_ui::inspect`)

- **ชี้เลือก** แล้วคลิก control ในหน้าต่าง (กรอบสีฟ้าบอกว่าตัวไหน, Esc เลิก) หรือเลือกจากรายการ
- **ลุคทีละสถานะ** (ปกติ hover pressed focus disabled checked): พิมพ์แล้ว control ในโปรแกรมเปลี่ยนทันที และแสดงสถานะนั้นให้ดู
  แต่ละค่าบอกที่มา (`own :hover`, `button.primary:hover · phai:light`)
- **ข้อความ, Enabled, Visible** และ property ใน `phai_ui::props::NAMES` (เช่น `tooltip`, `read_only`)
- **เก็บลงฟอร์ม** เขียนเฉพาะบรรทัดของ control นั้นใน `.phaiform` (คอมเมนต์และบรรทัดอื่นคงเดิม): ลุคสถานะปกติ → `back_color`
  `fore_color` `border_color` `radius` `bold` `border_width` `gradient` `shadow` `accent_color` `icon_color`; สถานะอื่น → `states = { }`
- **ย้อนทั้งหมด** คืนลุค ข้อความ Enabled/Visible ตอนเปิดตัวตรวจ
- Phai Studio ที่เปิดฟอร์มนั้นอยู่ โหลดฟอร์มใหม่เมื่อสลับกลับมา (Undo ได้) และเขียน `_ui.rs` ใหม่ให้รอบต่อไป
- โค้ด `.rs` ของโปรแกรมไม่ถูกแตะ; build แบบ release ไม่มีตัวตรวจ (codegen ใส่ `#[cfg(debug_assertions)] phai_ui::inspect::attach(...)`)

## การพิมพ์ — component `printdocument`

```toml
[[control]]
kind = "printdocument"
id = "report"
document_name = "Price list"
events = { print_page = "draw_page" }
```

```rust
fn draw_page(&mut self, ui: &MainWindowUi, e: &phai_ui::print::PrintPageEventArgs) {
    let m = e.margin_bounds;                 // ร้อยส่วนของนิ้ว จากมุมกระดาษ
    e.set_font("Segoe UI", 16, true, false);
    e.draw_text(&format!("หน้า {}", e.page), m.x, m.y);
    e.set_has_more_pages(e.page < 3);        // มีหน้าต่อไปไหม
}
// ที่ไหนก็ได้: ui.report.print_preview();  ui.report.print_dialog();  ui.report.page_setup();
```

## ผูกข้อมูล — component `bindingsource` และ `data_binding`

```toml
[[control]]
kind = "bindingsource"
id = "people"
bind = { database = "main", table = "customers", columns = ["id", "name", "vip"], keys = ["id"] }  # หรือ fields = "name, vip"
events = { position_changed = "moved" }

[[control]]
kind = "edit"
id = "name_box"
data_binding = "people.name"
```

`ui.people.move_next()`, `ui.people.add_new()`, `ui.save_people()` — ค่าที่พิมพ์ใน control กลับเข้าแถวเอง

## Data grid — ค่าของแต่ละคอลัมน์ (`column_props`)

แบบ GridColumn ของ DevExpress — ตารางละช่องตามลำดับ `columns` ช่องที่ไม่ใส่คือค่าเริ่มต้น

```toml
[[control]]
kind = "grid"
id = "orders"
columns = [["Name", 160], ["Paid", 60], ["Amount", 120]]
alternate_row_color = "#f4f8f6"
header_back_color = "#12855f"
header_fore_color = "#ffffff"
show_footer = true

[[control.column_props]]
sort = "asc"                 # asc / desc - เรียงตอนเปิด

[[control.column_props]]
editor = "check"             # text number date check combo (+ items = [...])
align = "center"             # left center right

[[control.column_props]]
align = "right"
format = "n2"                # n0..n6, c0..c6 (฿), p0..p6 (%), d, dt - แสดงเท่านั้น ค่าเดิมไม่เปลี่ยน
summary = "sum"              # sum count avg min max - ใต้คอลัมน์
rules = ["> 500 back=#ffe4e4 fore=#b00020", "empty back=#eeeeee row"]
```

ช่องอื่น: `visible = false`, `read_only = true`, `allow_sort = false`, `allow_size = false`, `group = true`
(จัดกลุ่มแถวตามคอลัมน์นี้ ได้คอลัมน์เดียว) — ค่าของทั้ง grid อยู่ในตาราง props: `show_column_headers`,
`column_auto_width`, `allow_add_rows`, `allow_delete_rows`, `editor_show_mode`, `enter_moves_next`,
`allow_column_moving`, `alternate_row_color`, `header_back_color`, `header_fore_color`, `row_height`,
`show_footer`, `find_panel` (`ctrl_f` `always` `off`), `find_filter`, `copy_headers`, `allow_filter` (ปุ่มกรองแบบ
Excel ที่หัวคอลัมน์ ค่าเริ่มต้น true), `header_menu` (คลิกขวาหัวคอลัมน์: เรียง กรอง จัดกลุ่ม ส่งออก Excel/PDF/CSV
ตัวอย่างก่อนพิมพ์ ค่าเริ่มต้น true)

ตารางหลัก-ตารางลูก: `master_detail = "orders.id = order_id"` ที่ grid ลูก — grid หลัก `orders` คอลัมน์ `id`
(หัวคอลัมน์หรือชื่อคอลัมน์ที่ผูก) = คอลัมน์ `order_id` ของ grid นี้ (ชื่อเดียวกันเขียนแค่ `orders.id`); codegen
เรียก `phai_ui::grid::set_master(lines, orders, 0, 1)` แถวใหม่ใน grid ลูกได้คีย์ของแถวหลักเอง

## User control — control ของโปรเจกต์เอง

Project > เพิ่ม User Control สร้าง `address_box.phaiform` ที่มี `user_control = true` ออกแบบเหมือนฟอร์มทั่วไป
แล้ววางบนฟอร์มอื่นจากกล่องเครื่องมือ:

```toml
[[control]]
kind = "usercontrol"
id = "home"
control_type = "AddressBox"
bounds = [11, 40, 300, 150]
anchor = "left top right"
on_change = "address_changed"     # เมื่อ AddressBox เรียก ui.notify_parent()
```

```rust
AddressBoxUi::with(ui.home, |a| a.street.set_text("12 Sukhumvit"));
```

## โค้ดที่สร้าง

`main_window_ui.rs` — designer เขียนทั้งไฟล์ทุกครั้ง:

```rust
// สร้างโดย Phai Studio จาก main_window.phaiform — อย่าแก้ไฟล์นี้ แก้ในฟอร์มแทน
use phai_ui::{t, win, controls, layout::*};
use windows::Win32::Foundation::HWND;

/// control เป็น object แบบ WinForms (`phai_ui::ctl`): ui.name.text(), ui.customers.items_add(..)
/// handle ของ Windows: ui.name.hwnd() หรือ *ui.name
pub struct MainWindowUi {
    pub hwnd: HWND,
    pub name_label: phai_ui::ctl::Label,
    pub name: phai_ui::ctl::TextBox,
    pub customers: phai_ui::ctl::ListView,
    pub ok: phai_ui::ctl::Button,
    pub status: phai_ui::ctl::StatusBar,
    layout: Layout,
}

/// handler ทุกตัวที่ฟอร์มอ้างถึง — ค่าเริ่มต้นว่าง เพิ่ม handler ใหม่ในฟอร์มแล้วโค้ดผู้ใช้ไม่พัง
pub trait MainWindowEvents {
    fn ok_clicked(&mut self, _ui: &MainWindowUi) {}
    fn open(&mut self, _ui: &MainWindowUi) {}
    fn exit(&mut self, _ui: &MainWindowUi) {}
}

impl MainWindowUi {
    /// สร้างหน้าต่างและ control ทั้งหมด, ใส่ธีม, จัดวาง
    pub fn create<E: MainWindowEvents + 'static>(events: E) -> MainWindowUi { /* ... */ }

    /// แปลข้อความทุกตัวใหม่ — ตอนเปลี่ยนภาษา
    pub fn relabel(&self) { /* win::set_text(self.name_label, t("Name")); ... */ }
}

// window procedure: WM_COMMAND → เมธอดใน MainWindowEvents, WM_SIZE → layout,
// WM_DPICHANGED → ฟอนต์/ไอคอน/ตำแหน่งใหม่, อย่างอื่น → win::def (ธีม)
```

`main_window.rs` — ของผู้ใช้ สร้างโครงให้ครั้งแรกเท่านั้น:

```rust
use crate::main_window_ui::{MainWindowEvents, MainWindowUi};

pub struct MainWindow {
    // ข้อมูลของผู้ใช้
}

impl MainWindowEvents for MainWindow {
    fn ok_clicked(&mut self, ui: &MainWindowUi) {
        let name = ui.name.text();
        ui.customers.items_add(&[&name, ""]);
        ui.name.clear();
    }
}
```

**ดับเบิลคลิก control ใน designer**: ถ้าในไฟล์ผู้ใช้ยังไม่มี `fn ok_clicked(` ให้ต่อท้ายโครงเมธอดว่างใน
`impl MainWindowEvents` แล้วเปิดไฟล์ที่บรรทัดนั้น — ค้นแค่ข้อความ ไม่ parse โค้ดของผู้ใช้

## ยังไม่อยู่ในรุ่น 1

control ที่ผู้ใช้ทำเอง, รูปภาพ, ฟอร์มซ้อนฟอร์ม, data binding — เพิ่มเป็น `format = 2` เมื่อจำเป็น
designer อ่านรุ่นเก่าได้เสมอ
