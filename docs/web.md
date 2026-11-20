# โปรแกรม Phai บนเว็บ และบนมือถือ (PWA)

ฟอร์ม `.phaiform` เดิม กับโค้ด Rust เดิมของโปรแกรม (`main_window.rs`) รันในเบราว์เซอร์ได้ ด้วยการคอมไพล์เป็น
WebAssembly แล้วติดตั้งลงมือถือ Android และ iPhone เป็นแอปได้ (PWA) ไม่ต้องผ่าน Play Store หรือ App Store
และไม่ต้องใช้ Mac

## ใช้อย่างไร

ใน Phai Studio: **Build > สร้างสำหรับเว็บ (PWA)** หรือ **Build > รันในเบราว์เซอร์** (สร้างแล้วเปิดในเบราว์เซอร์ของเครื่องนี้)

ได้โฟลเดอร์ `web/` ในโปรเจกต์:

| ไฟล์ | คืออะไร |
|---|---|
| `index.html` | หน้าเว็บ พร้อมธีม (`theme.css` ของโปรเจกต์ แปลงเป็น CSS ของเบราว์เซอร์) |
| `pkg/<ชื่อ>.js`, `pkg/<ชื่อ>_bg.wasm` | โปรแกรม (WebAssembly) |
| `manifest.webmanifest` | ชื่อ สี ไอคอน — ให้มือถือติดตั้งเป็นแอป |
| `sw.js` | service worker — เก็บไฟล์ไว้ ให้เปิดได้แม้ไม่มีเน็ต และรับเวอร์ชันใหม่เมื่อมีเน็ต |
| `icon-192.png`, `icon-512.png` | ไอคอนแอป (สี `--accent` ของธีม) |

ติดตั้งลงมือถือ: อัปโหลดโฟลเดอร์ `web/` ขึ้นเว็บที่เป็น **HTTPS** (เว็บไซต์ของเราเอง, GitHub Pages ฯลฯ — service
worker ทำงานเฉพาะบน HTTPS หรือ localhost) แล้วเปิดในมือถือ:

- Android (Chrome): เมนู ⋮ > **ติดตั้งแอป** / **เพิ่มลงหน้าจอหลัก**
- iPhone (Safari): ปุ่มแชร์ > **เพิ่มลงหน้าจอโฮม**

ภาษา: ไทยเมื่อเครื่องตั้งภาษาไทย หรือบังคับด้วย `?lang=th` / `?lang=en` ต่อท้ายที่อยู่

## ทำงานอย่างไร

- `phai-web` (crate ใหม่) มีชนิดและเมธอดชื่อเดียวกับ `phai_ui::ctl` (`TextBox::text()`, `ComboBox::selected_item()`,
  `ListView::items_add()`, `StatusBar::set_part_text()`, `Timer`...) และ `phai_ui::text::{t, tf}` เดียวกัน
- Studio เขียน `src/web/<ฟอร์ม>_ui.rs` จากฟอร์ม: `<Form>Ui` และ `<Form>Events` เหมือนเวอร์ชัน Windows ทุกเมธอด
- Cargo.toml ของโปรเจกต์: phai-ui และ windows ย้ายไปอยู่ใต้ `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
  และเพิ่ม phai-web ใต้ `[target.'cfg(target_arch = "wasm32")'.dependencies]`
- main.rs: `#[cfg(target_arch = "wasm32")] extern crate phai_web as phai_ui;` และ `mod xxx_ui;` บนเว็บใช้ไฟล์ใน `src/web/`
  — โค้ดของโปรแกรมเองไม่ต้องแก้ ไฟล์เดียวใช้ทั้งสองที่
- ตำแหน่ง control: anchor และ dock เป็น CSS (`left/right/top/bottom`) — ขยายตามจอ; จอแคบกว่าที่ออกแบบ (มือถือ) ย่อทั้งฟอร์มให้พอดีความกว้าง
- ฟอร์มที่สอง (`show`) เปิดเป็นกล่องทับหน้า มีปุ่มปิด; `show_dialog` บนเว็บไม่รอ (คืน `None` ทันที)

## มีบนเว็บ / ยังไม่มี

มี: เมนูบาร์ (เมนูย่อยซ้อน, ติ๊ก, check_on_click, รายการสีเทา, ไอคอน, ปุ่มลัด Ctrl+...), toolbar (ปุ่มกดค้างติ๊กพร้อมเมนู;
บนมือถือเป็นข้อความเพราะไม่มีฟอนต์ไอคอนของ Windows), `ui.item_checked`, `set_item_checked`, `set_item_enabled`,
label, edit (รวม password, placeholder), textarea, richtext (เป็นข้อความธรรมดา), button, checkbox, radio, combo,
listbox, checkedlist, list (ListView), grid (แก้ในช่องได้), number, date, calendar, slider, progress, group, panel,
tabs, link, picture, statusbar, timer; event `on_click` `on_change` `on_select`, Click DoubleClick Enter Leave KeyDown
KeyUp KeyPress MouseDown MouseUp MouseMove MouseEnter MouseLeave CheckedChanged TextChanged, Shown, `loaded`

ยังไม่มี (Studio บอกชื่อ control ที่ไม่ขึ้นบนเว็บหลังสร้าง): context menu, tree, split container,
user control, dialog แบบ component (Open/Save/Folder/Color/Font), ErrorProvider, NotifyIcon, การพิมพ์, BindingSource,
ตารางที่ผูกฐานข้อมูล (`bind`) — โค้ดที่เรียกสิ่งเหล่านี้จะ build สำหรับเว็บไม่ผ่าน

## เครื่องมือที่ต้องมี (ฟรี ติดตั้งครั้งเดียว)

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
```
