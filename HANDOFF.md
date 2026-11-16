# Handoff — Phai Studio (2026-11-16)

## ▶ เริ่มตรงนี้

**สถานะ (2026-11-16): เริ่มแล้ว** เจ้านายสั่งเริ่ม ("เกินไปก็ไม่ติดอะไร ขอให้ฉันได้ phai studio")
- ขั้น 0 เสร็จ: `docs/phaiform.md`
- ขั้น 1 ใช้งานได้: `crates/phai-ui` — control ธีม CSS (`@import "phai:light"`, ไฟล์เปลี่ยนแล้วหน้าต่างเปลี่ยนทันที),
  dock/anchor, ไทย/อังกฤษ, ถ่ายภาพหน้าต่าง (`PHAI_SHOT=out.png` กับทุกโปรแกรม)
- ขั้น 2 ตัวแรกใช้งานได้:
  - `crates/phai-form` — อ่าน/ตรวจ/เขียน `.phaiform` (error บอกบรรทัด), สร้างโค้ด `<name>_ui.rs` + `<name>.rs`, `live` สร้างฟอร์มจริง
  - `apps/studio` — designer: กล่องเครื่องมือ, ลากสร้าง/ย้าย/ย่อขยาย (snap 4), ลูกศร/Shift+ลูกศร/Del, Properties,
    Undo/Redo, บันทึก+สร้างโค้ด, ดับเบิลคลิก = เพิ่ม handler ในไฟล์ผู้ใช้, F5 ดูตัวอย่าง, ธีม: สร้างจากมาตรฐาน/นำเข้า CSS
  - `apps/sample` — โปรแกรมที่ build จากโค้ดที่ generate จริง; เทสต์ตรวจว่าไฟล์ตรงกับตัวสร้างโค้ด
    (เปลี่ยนตัวสร้างโค้ดแล้ว: `cargo test -p phai-form -- --ignored regenerate`)
- ขั้น 3 ใช้งานได้: โปรเจกต์ใหม่ (โฟลเดอร์ว่าง → Cargo.toml, build.rs, ฟอร์มแรก, ธีม, ตารางไทย), Build Ctrl+B / Run F5 /
  Stop Shift+F5 ผ่าน `cargo --message-format=json`, รายการ error (ดับเบิลคลิกเปิดไฟล์), ไม่มี Rust → ชวนเปิด rustup.rs
- เทียบ Visual Studio ทำแล้ว: Project explorer (ต้นไม้ไฟล์ + ไอคอน, คลิกขวา ฟอร์มใหม่), แท็บเอกสาร (Undo แยกแท็บ, Ctrl+W),
  กล่องเครื่องมือเป็นไอคอนล้วน, ฟอร์มบน canvas มีแถบชื่อ + menu bar, แก้เมนู/toolbar เป็นข้อความ (Edit > เมนู..., คลิก menu bar)
- MIT license ใส่แล้ว (Monrak Net เหมือน PhaiDesk) — ยังไม่สร้าง repo บน GitHub: รอเจ้านายสั่ง
  ก่อนเปิดสาธารณะ: โปรเจกต์ใหม่อ้าง phai-ui ด้วย path ในเครื่อง → ต้องเปลี่ยนเป็น git/crates.io
- เจ้านายบอก "พอแค่นี้" แล้วสั่ง "ทำต่อสิ" (2026-11-16) — ทำเพิ่มแล้ว:
  เลือกหลายตัว (Ctrl+คลิก, ลากกรอบ), ตัด/คัดลอก/วาง/ทำซ้ำ (TOML ในคลิปบอร์ด), เมนู Format (ชิด/เท่ากัน/เว้นระยะ),
  แผงล่าง Errors + Output (บรรทัด cargo สด), Debug/Release, ไฟล์ล่าสุด, font จาก CSS, เมนู popup/scrollbar มืด,
  `crates/phai-design` (metrics + แม่แบบ Dialog / หน้าต่างหลัก / ฟอร์มกรอกข้อมูล; File > New from template),
  README + ภาพ, `repository = ""` ใน Cargo.toml — ใส่ URL GitHub แล้วโปรเจกต์ใหม่จะดึง phai-ui จาก git เอง
- **เหลือ**: สาย E (แปลง Monaco: ตัวแก้โค้ด, จากนั้น debugger, Git), เส้นช่วยจัดแนว, Start window
- **ก่อนแจก**: ใส่ `repository` ใน Cargo.toml ของ workspace เป็น URL ของ repo
- self-test: `--selftest out.phaiform` (ลากสร้าง/ย้าย/ย่อขยาย), `--buildtest` (Build จริง), `--tab N`, `--menus`;
  `PHAI_SHOT=out.png` ถ่ายหน้าต่างหน้าสุดของโปรแกรม (รวม dialog)
- debug build ของ studio เป็น console + พิมพ์ backtrace ตอนพัง (`report_crashes`)
- `phai-studio form.phaiform --selftest out.phaiform` ลากสร้าง/ย้าย/ย่อขยายด้วยข้อความเมาส์จริงแล้วบันทึก

ตรวจด้วยภาพทุกครั้ง:
```
cargo build -p phai-gallery
target\debug\phai-gallery.exe --shot out.png [--dark] [--thai] [--tab 1] [--css apps\gallery\themes\blue.css]
```
repo ยังมีแค่ในเครื่อง (ยังไม่มี GitHub)

### คืออะไร
ตัวออกแบบหน้าต่างแบบลากวางสำหรับโปรแกรม Windows ที่เขียนด้วย Rust: วางปุ่ม/รายการ/toolbar/menu
แล้วได้โค้ด Rust แบบ Win32 แท้ (ตัวเล็ก ไม่ต้องมี .NET ไทยถูก) — สิ่งที่ Rust ยังไม่มีใครทำให้ใช้ได้จริง
(RustRover / VS Code / Zed เป็นตัวแก้โค้ด ไม่มี designer; Slint วาดเองไม่ใช่ Win32; gtk-rs เป็น GTK)

### กติกาของเจ้านาย
- ตอบภาษาไทย เรียก **"เจ้านาย"** · **ไม่ต้องถาม ทำได้ก็ทำ** · commit (+ push เมื่อมี remote) ทุกรอบที่เทสต์ผ่าน
- หน้าตา: **Windows มาตรฐาน minimal** — ฟอนต์ระบบ 9 pt, ปุ่ม/ป้ายสั้น, menu bar, toolbar ไอคอนล้วน
  (ชื่อเป็น tooltip, ไอคอนจาก Segoe Fluent Icons / MDL2), status bar, พื้นขาวแบน, สีเขียว Phai `#12855f`
  เฉพาะตอนชี้/เลือก — เจ้านายไม่ชอบตัวใหญ่และข้อความอธิบายยาว
- ห้าม reboot เครื่อง · ห้ามเขียน crypto เอง · เครื่องมืออยู่ที่ `D:\Tools` (Rust: `D:\Tools\Rust`)

### ขั้น 0 — รูปแบบไฟล์ (เสร็จ: `docs/phaiform.md`)
- `.phaiform` (แนะนำ TOML หรือ JSON อ่านง่าย diff ได้): ขนาดฟอร์ม, รายการ control (ชนิด, id, ข้อความเป็นคีย์อังกฤษ,
  ตำแหน่ง/ขนาดแบบ design pixel ที่ 96 DPI, anchor/dock), เมนู, toolbar
- โค้ดที่สร้าง: `form_name_ui.rs` (designer เขียน ห้ามแก้มือ) + `form_name.rs` (ของผู้ใช้: event handler)
  แบบเดียวกับ `Form.Designer.cs` + `Form.cs` — **ไม่อ่านโค้ดผู้ใช้กลับเข้ามา**

### ขั้น 1 — ไลบรารี `phai-ui` (ทำแล้ว ดูโค้ดใน `crates/phai-ui`)
**ก๊อปปี้** (ไม่ย้าย) จาก `D:\Project.NET\phai-desk\agent-rs\agent\src\console\`:

| ไฟล์ต้นทาง | ได้อะไร |
|---|---|
| `win.rs` | ฟอนต์ตาม DPI, register class (เทา/ขาว), child control, toolbar ไอคอน Fluent + tooltip, status bar, ปุ่มวาดเอง, custom-draw การเลือกสีเขียว, popup menu |
| `text.rs` | `t()` / `tf()` ไทย-อังกฤษ คีย์เป็นอังกฤษ + เทสต์ที่จับข้อความไม่มีคำแปล |
| `dialogs.rs` | ช่องถามข้อความ, หน้าต่างรอพร้อม Cancel |
| `hosts.rs` | ตัวอย่างหน้าต่างหลักครบ: menu, toolbar, list view ไอคอน, status bar, เปลี่ยนภาษาสด |
| `files.rs` | ตัวอย่าง list แบบรายละเอียด + toolbar + แถวช่องที่อยู่ |
| `../../build.rs` | manifest Common Controls 6 (ไม่มีแล้ว image list ไม่รองรับ alpha, ปุ่มเป็นแบบเก่า) |
| `crates/ui/src/tray.rs` | `icon_from_bgra` (ไอคอนพร้อม mask จริง) |

ตรวจโค้ดจริงแล้ว (2026-11-16) ตรงตามตาราง ตอนก๊อปปี้:
- ตัด `phaidesk_ui::app_icon` ออกจาก `register_on` (ให้ผู้ใช้ส่งไอคอนเอง) และตัดประโยคของ PhaiDesk ออกจาก `text.rs`
  (เหลือแค่กลไก `t()`/`tf()` + เทสต์)
- ไม่เอา `viewer`, `picture`, `input`, `link`, `local` — เป็นงานรีโมตของ PhaiDesk

แล้วเพิ่มสิ่งที่ console ยังไม่มี: **ระบบจัดวาง anchor/dock** (ตอนนี้แต่ละหน้าต่างคำนวณตำแหน่งเองใน `layout()`),
control มาตรฐานที่ยังขาด (checkbox, radio, combobox, tab, group box), ทุกอย่างวัดขนาดจากข้อความจริง (บทเรียนภาษาไทย)

**`Theme`** (รายละเอียดใน `PLAN.md` "ธีมและหน้าตา"): `win.rs` มีสี `rgb(...)` ตายตัว 12 ค่ากระจายอยู่
(`draw_button`, `list_custom_draw`, `toolbar_rule`, `glyph_icon` สี `#3c3c3c`) → รวมเป็น struct สีมีชื่อ
แบบ `phai-desk/UI/ModernUi.cs` แล้วทุกฟังก์ชันอ่านจากธีม; เพิ่มสว่าง/มืด และ DWM (แถบชื่อ, มุมมน, Mica) บน Windows 11
แนวหน้าตา: Fluent ของ Windows 11 แบบ WPF UI — เอาแค่ภาษาออกแบบ ไม่ก๊อปปี้โค้ด

### บทเรียนจากการทำ `phai-ui` / designer
- `DrawTextW` กับข้อความว่าง = pointer ลอย → access violation (โปรแกรมหายไปเงียบๆ exit 127 ใน bash) — `win::draw_text` กันแล้ว
- control บน canvas ตอบ `WM_NCHITTEST` = HTTRANSPARENT (`ghost`) เมาส์ทั้งหมดจึงไปที่ surface ซึ่งหา control จากสี่เหลี่ยม
- designer แสดงข้อความในฟอร์มตามที่เขียน (`live::set_translate(false)`) — ตารางไทยของ designer จะแปลผิด
- โปรแกรม GUI ที่เปิดผ่าน `cmd /c` ไม่รอ — ใช้ bash ตรงๆ หรือ `timeout`
- control ลูกที่สร้างทีหลังอยู่**ล่าง** z-order — group ต้องถูกส่งไปล่างสุดตอนจัดวาง (`Layout::apply` ทำให้แล้ว)
- ธีม Explorer ของ list วาดแถวที่เลือกเอง สี `list:selected` จะไม่ขึ้น → ไม่ใส่ธีม Explorer ให้ตัว list
- ระยะห่างตามมาตรฐาน Windows: ขอบ 11, ระหว่างแถว 7, control สูง 23, ปุ่มกว้างอย่างน้อย 75 (design px)
  ความกว้างที่มีข้อความวัดทั้งอังกฤษและไทย — เจ้านายไม่ชอบตัวใหญ่ ระยะห่าง และข้อความอธิบาย

### บทเรียนจาก PhaiDesk ที่ต้องพกมา
- ถ่ายภาพหน้าต่างจริงเพื่อตรวจทุกครั้ง (`PrintWindow`) — บั๊กหน้าตาไม่เห็นจากโค้ด
- จอเจ้านาย 2880×1800 สเกล ~200% → ทดสอบ DPI เสมอ (`SetProcessDpiAwarenessContext` PerMonitorV2)
- Enter ในหน้าต่างที่ใช้ `IsDialogMessage` กลายเป็น IDOK ทุก control → ต้องดูว่าโฟกัสอยู่ที่ไหน
- windows-rs 0.62: ค่าคงที่บางตัวไม่มีใน feature ที่เปิด เขียนเป็นตัวเลขพร้อมคอมเมนต์ชื่อ

### ขั้นต่อไปหลังจากนี้
ขั้น 2 designer (2–3 สัปดาห์), ขั้น 3 build/run (~1 สัปดาห์) — รายละเอียดใน `PLAN.md`

### ตัวแก้โค้ด: ไม่ใช้ VS Code
เจ้านายเลือก (2026-11-16) **แปลง Monaco เป็น Rust** เป็นตัวแก้โค้ดของเราเอง (`phai-editor`) — สายแยก E0–E7
ใน `PLAN.md` หัวใจ: ชั้น `common/` แปลงทีละไฟล์**พร้อมเทสต์ของ Monaco**, ชั้น `browser/` เขียนใหม่บน DirectWrite
ต้นฉบับอยู่ใน `microsoft/vscode` `src/vs/editor/` (ไม่ใช่ repo monaco-editor) — MIT เก็บ LICENSE ไว้

### รอเจ้านาย
ฟรีหรือขาย · สร้าง repo บน GitHub · วันเริ่ม
