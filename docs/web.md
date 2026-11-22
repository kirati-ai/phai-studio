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

**Build > Publish Web App...** สร้างแล้วส่ง `web/` ไปที่ปลายทางใน `Cargo.toml` `[package.metadata.phai-web] publish`:
โฟลเดอร์ (คัดลอก), `user@host:/path` (Windows `scp -r -o BatchMode=yes` ใช้ SSH key ไม่ถามรหัส) หรือ `github-pages`
(clone branch `gh-pages` ของ origin ลงโฟลเดอร์ชั่วคราว แทนไฟล์ด้วย `web/` + `.nojekyll` commit แล้ว push ไม่ force)

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

มีด้วย (2026-11-21): เมนูคลิกขวา (`phai_ui::menu::context_menu`, บนมือถือกดค้าง 550ms, Opening ก่อนเปิด) และ
tree (`phai_ui::tree`, `TreeView` เมธอดเดียวกับ Windows, `Node`, ลูกศรเปิดปิด คีย์บอร์ด, `change` = AfterSelect)

มีด้วย (2026-11-22): **Paint** ของ panel / picture (`phai_ui::paint::PaintEventArgs` ชุดเดียวกับ Windows: clear, fill_rect,
draw_line, draw_text, draw_text_centered, draw_pixels, draw_pixels_alpha, fit — วาดลง `<canvas>` ที่เต็ม control คมตาม
ความละเอียดจอมือถือ วาดใหม่เมื่อหน้าเปลี่ยนขนาดและเมื่อ `ui.x.invalidate()`), **MouseWheel** (`e.delta` 120 ต่อหนึ่งจังหวะ
เหมือน Windows, หน้าไม่เลื่อนตาม), `client_size()`; panel / picture ที่มี KeyDown/KeyPress รับโฟกัสได้ (`tabindex`)

**ตารางที่ผูกฐานข้อมูล (`bind`) บนเว็บ (2026-11-21):** เบราว์เซอร์ต่อฐานข้อมูลตรงไม่ได้ จึงมี **เซิร์ฟเวอร์ข้อมูล**
- สร้างสำหรับเว็บแล้ว Studio เขียนโฟลเดอร์ `data-server/` (package แยก ใช้ phai-data อย่างเดียว + `src/data_sources.rs`
  ของโปรเจกต์) ให้บริการเฉพาะตารางและคอลัมน์ที่ฟอร์มผูก ชื่อ `ฟอร์ม/grid` เช่น `main_window/customers` ไม่รับ SQL
- `POST .../phai-data/load` `{"table"}` และ `.../save` `{"table","loaded","rows":[[tag,[...]]]}` (`phai_data::server`)
- รัน: `cd data-server` แล้ว `cargo run --release` พร้อม `PHAI_DB_PASSWORD`, `PHAI_DATA_TOKEN` (รหัสเข้าใช้ ต้องมีเสมอ
  ไม่ตั้ง = สุ่มให้แล้วพิมพ์ออกมา เพราะ "เฉพาะเครื่องนี้" กันไม่ได้: เว็บอื่นที่เปิดในเบราว์เซอร์ก็ถาม 127.0.0.1 ได้ และ nginx ส่งต่อมา
  ก็มาจากเครื่องนี้หมด; ใส่รหัสผิดรอ 1 วินาทีทีละครั้ง เดารหัสเร็ว ๆ ไม่ได้), `PHAI_DATA_ADDR` (ค่าเริ่มต้น `127.0.0.1:8791`) วางหลัง HTTPS ของเว็บที่ `/phai-data/` เช่น nginx
  `location /phai-data/ { proxy_pass http://127.0.0.1:8791; }` หรือคนละที่: `[package.metadata.phai-web] data-url = "https://..."`
- หน้าเว็บ (`phai_web::data`): `reload_x` / `save_x` เหมือน Windows แต่เริ่มงานแล้วคืนทันที (save คืน `Ok(0)`) ตารางเติมเมื่อ
  คำตอบมา ผิดพลาดขึ้นข้อความ; รหัสเข้าใช้ถามครั้งแรกแล้วเก็บใน localStorage; ดับเบิลคลิกใต้แถวสุดท้ายเพิ่มแถว Ctrl+Delete ลบแถว
- Run in Browser / Run on Phone: Studio build และเปิดเซิร์ฟเวอร์ข้อมูลเอง (job object ปิดตามเมื่อปิด Studio) ส่งต่อ `/phai-data/`
  จากเซิร์ฟเวอร์หน้าเว็บ; รหัสเข้าใช้ 6 หลักสุ่มครั้งเดียวต่อการเปิด Studio (`web::session_key`): หน้าของเครื่องนี้
  (`localhost`/`127.0.0.1` และ Origin เดียวกัน) ได้รหัสจากเซิร์ฟเวอร์หน้าเว็บโดยไม่ต้องถาม; มือถือพิมพ์เอง (แสดงในแถบสถานะ)
- phai-data build บน Linux ได้แล้ว (`windows` เฉพาะ Windows; native-tls ต้องมี OpenSSL)

ยังไม่มี (Studio บอกชื่อ control ที่ไม่ขึ้นบนเว็บหลังสร้าง): split container,
user control, dialog แบบ component (Open/Save/Folder/Color/Font), ErrorProvider, NotifyIcon, การพิมพ์, BindingSource
— โค้ดที่เรียกสิ่งเหล่านี้จะ build สำหรับเว็บไม่ผ่าน

## เครื่องมือที่ต้องมี (ฟรี ติดตั้งครั้งเดียว)

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
```
