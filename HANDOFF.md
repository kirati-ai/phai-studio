# Handoff — Phai Studio (2026-11-16)

## ▶ เริ่มตรงนี้

**สถานะ: ยังไม่เริ่มเขียนโค้ด** มีแค่แผน (`PLAN.md`) และ repo นี้มีแค่ในเครื่อง (`git init` แล้ว ยังไม่มี GitHub)
เจ้านายสั่งให้ทำ PhaiDesk ให้เสร็จก่อน — เริ่มเมื่อเจ้านายบอกให้เริ่ม

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

### ขั้น 0 — รูปแบบไฟล์ (1–2 วัน)
- `.phaiform` (แนะนำ TOML หรือ JSON อ่านง่าย diff ได้): ขนาดฟอร์ม, รายการ control (ชนิด, id, ข้อความเป็นคีย์อังกฤษ,
  ตำแหน่ง/ขนาดแบบ design pixel ที่ 96 DPI, anchor/dock), เมนู, toolbar
- โค้ดที่สร้าง: `form_name_ui.rs` (designer เขียน ห้ามแก้มือ) + `form_name.rs` (ของผู้ใช้: event handler)
  แบบเดียวกับ `Form.Designer.cs` + `Form.cs` — **ไม่อ่านโค้ดผู้ใช้กลับเข้ามา**

### ขั้น 1 — ไลบรารี `phai-ui` (~1 สัปดาห์)
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
