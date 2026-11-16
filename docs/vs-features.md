# Phai Studio เทียบ Visual Studio

รายการจากเอกสารทางการของ Microsoft Learn (2026-11-16) — **★** = ที่คนทำ WinForms ใช้ทุกวัน
สถานะ: ✅ มีแล้ว · 🟡 มีบางส่วน · ⬜ ยังไม่มี

## Designer

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Toolbox ลากวาด / คลิกแล้ววาด | ✅ |
| ★ | ดับเบิลคลิกเครื่องมือ = วางบนฟอร์มเลย | ✅ |
| | Toolbox ชื่อข้างไอคอน, ค้นหา, หมวดพับได้ (Common controls / Containers / Menus and toolbars) | ✅ |
| ★ | Properties (F4), ค่าเปลี่ยนทันที | ✅ |
| ★ | Properties ⇄ Events, ดับเบิลคลิกสร้าง handler | ✅ |
| ★ | Properties แบบ VS: ตัวเลือก object (ชื่อตัวหนา + ชนิด), แถบไอคอน Categorized / Alphabetical / Properties / Events, ค่าที่ต่างจากค่าเริ่มต้นเป็นตัวหนา, Location / Size / Font พับเปิดได้ | ✅ |
| ★ | เรียงหมวด (พับได้) / ตามตัวอักษร, คำอธิบายใต้ตาราง | ✅ |
| ★ | ตัวแก้ค่าตามชนิด: รายการ, True/False, ตัวเลือกสี Windows, anchor แบบภาพ, “…” | ✅ |
| ★ | BackColor / ForeColor / Font bold / มุมมน ราย control (ต่อเป็น CSS `#id`) | ✅ |
| | ช่องเลือก object, เลือกหลายตัวแสดงค่าร่วม | 🟡 (เลือก object ได้, ค่าร่วมยังไม่มี) |
| ★ | ดับเบิลคลิก control = ไปที่ handler | ✅ |
| ★ | เลือกหลายตัว (Ctrl+คลิก, ลากกรอบ), ตัวหลัก | ✅ |
| ★ | ลูกศรย้าย, Shift+ลูกศรย่อขยาย, Ctrl+ลูกศรกระโดดตามเส้นจัดแนว | ✅ |
| ★ | จุดจับ 8 จุด | ✅ |
| ★ | Snaplines ขอบ/ระยะขอบ | ✅ (baseline ข้อความยังไม่มี) |
| ★ | Format > Align / Make Same Size / Spacing | ✅ |
| ★ | Bring to Front / Send to Back | ✅ (ลำดับในฟอร์ม = ชั้น = ลำดับ Tab แบบ Win32) |
| ★ | Tab Order mode | ✅ |
| ★ | Lock Controls | ✅ (ทั้งฟอร์ม ยังไม่ราย control) |
| ★ | Dock / Anchor (ตัวแก้แบบภาพ) | ✅ |
| ★ | Control: DateTimePicker, NumericUpDown, TrackBar, ProgressBar, TextBox หลายบรรทัด | ✅ |
| | Control: PictureBox, Timer, MenuStrip ในฟอร์ม, TreeView, RichTextBox, DataGridView | ⬜ |
| ★ | Smart tags | ⬜ |
| ★ | Component tray (ของที่มองไม่เห็น เช่น Timer) | ⬜ |
| | Document Outline | ⬜ |
| | TableLayout / FlowLayout | ⬜ |
| | ตัวแก้เมนู/toolbar | ✅ (แบบข้อความ) |
| | Localization | ✅ (ตาราง ไทย/อังกฤษ + เทสต์) |

## ตัวแก้โค้ด

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | IntelliSense หลัง `.` `::`, Ctrl+Space | ✅ (rust-analyzer) |
| ★ | Quick Info (hover) | ✅ |
| ★ | Parameter Info (Ctrl+Shift+Space) | ⬜ |
| ★ | เส้นหยักใต้ error / warning | ✅ |
| ★ | Quick Actions (Ctrl+.) | ⬜ |
| ★ | Go To Definition (F12, Ctrl+click; Edit menu) — Peek not yet | 🟡 |
| ★ | Find All References (Shift+F12) — References tab, double-click to go | ✅ |
| ★ | Quick Find / Replace (Ctrl+F / Ctrl+H) | ✅ |
| | Find in Files (Ctrl+Shift+F) | ⬜ |
| ★ | Rename (F2; Edit menu) — every file, open ones as one undo step | ✅ |
| | Comment toggle (Ctrl+/) ✅; ย้ายบรรทัด (Alt+↑↓), ทำซ้ำบรรทัด ⬜ | 🟡 |
| | Multi-caret, box selection | ⬜ |
| | พับโค้ด (outlining) | ⬜ |
| | จับคู่วงเล็บ | ⬜ |
| | ซูม (Ctrl+ล้อ) | ✅ |
| | Format Document (rustfmt) | ⬜ |

## โปรเจกต์ / Build

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Solution Explorer | ✅ (Project explorer) |
| ★ | Build / Run (F5) / Stop | ✅ |
| ★ | Debug / Release | ✅ |
| ★ | Error List, ดับเบิลคลิกไปที่บรรทัด | ✅ (F8 ถัดไปยังไม่มี) |
| ★ | Output window | ✅ |
| | Rebuild / Clean | ⬜ |
| | Task List (TODO) | ⬜ |

## Debugger

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Breakpoints, F10/F11/Shift+F11, Locals, Watch, Call Stack, data tips | ⬜ (ต้องใช้ debug adapter เช่น CodeLLDB/cppvsdbg) |

## หน้าต่าง

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | แผง dock แยก, เส้นแบ่งปรับขนาดทุกแผง (รวม Toolbox), ปิด/เปิดจาก View | ✅ |
| ★ | แถบ Standard: ย้อนกลับ/ไปข้างหน้า, โปรเจกต์ใหม่, ฟอร์มใหม่, เปิด, บันทึก, บันทึกทั้งหมด, Undo/Redo, กล่อง Debug/Release, ▶ ชื่อโปรเจกต์, Stop, Build | ✅ |
| ★ | แถบ Layout: จัดตามเส้นตาราง, จัดแนว 6 แบบ, ขนาดเท่ากัน/ตามเส้นตาราง, ระยะแนวนอน/ตั้ง (เท่ากัน/เพิ่ม/ลด/ชิด), กึ่งกลางฟอร์ม, หน้า/หลัง, Tab order; ปุ่มเป็นสีเทาเมื่อใช้ไม่ได้ | ✅ |
| | ลาก toolbar ย้ายตำแหน่ง, ▾ เพิ่ม/ลบปุ่ม | ⬜ |
| ★ | ปุ่มบนหัวแผง: ▾ Window Position (Dock / Auto Hide / Hide), 📌 Auto Hide, × ปิด | ✅ |
| ★ | ลากหัวแผงไปติดขอบซ้าย/ขวา/บน/ล่าง หรือลอย (Toolbox; กล่องสีบอกตำแหน่ง, Esc ยกเลิก, ดับเบิลคลิกหัวหน้าต่างลอยเพื่อกลับ) | 🟡 (แผงขวายังลากไม่ได้, ยังไม่มีรูปเพชรนำทาง) |
| ★ | Auto Hide: แท็บแนวตั้งที่ขอบ, ชี้หรือคลิกแล้วเลื่อนออก, ออกจากแผงแล้วเก็บเอง (แผงขวาซ่อนพร้อมกันเป็นกลุ่ม) | ✅ |
| ★ | แท็บเอกสาร, จุดบอกยังไม่บันทึก | ✅ |
| | Split editor, หลายจอ | ⬜ |

## อื่นๆ

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Git Changes (stage, commit, push) | ⬜ |
| | Start window | ⬜ |
| | ธีมสว่าง/มืด, ฟอนต์ | ✅ |
| | ไฟล์ล่าสุด | ✅ |
| | Test Explorer (`cargo test`) | ⬜ |
| | Publish / ตัวติดตั้ง | ⬜ |
