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
| | ช่องเลือก object, เลือกหลายตัวแสดงเฉพาะ property ที่มีร่วมกัน (ค่าต่างกันเว้นว่าง) และพิมพ์ค่าแล้วเปลี่ยนทุกตัวในขั้นเดียว (Undo ครั้งเดียว) | ✅ |
| ★ | ดับเบิลคลิก control = ไปที่ handler | ✅ |
| ★ | เลือกหลายตัว (Ctrl+คลิก, ลากกรอบ), ตัวหลัก | ✅ |
| ★ | ลูกศรย้าย, Shift+ลูกศรย่อขยาย, Ctrl+ลูกศรกระโดดตามเส้นจัดแนว | ✅ |
| ★ | จุดจับ 8 จุด | ✅ |
| ★ | Snaplines ขอบ/ระยะขอบ | ✅ (baseline ข้อความยังไม่มี) |
| ★ | Format > Align / Make Same Size / Spacing | ✅ |
| ★ | Bring to Front / Send to Back | ✅ (ลำดับในฟอร์ม = ชั้น = ลำดับ Tab แบบ Win32) |
| ★ | Tab Order mode | ✅ |
| ★ | Lock Controls: ทั้งฟอร์ม (Format > Lock) และราย control (Properties > Locked = True - เมาส์และลูกศรไม่ย้าย/ไม่ปรับขนาด, ไม่มีผลกับโปรแกรมจริง) | ✅ |
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
| ★ | Parameter Info (Ctrl+Shift+Space, และเมื่อพิมพ์ ( หรือ ,): signature ของฟังก์ชันใต้เคอร์เซอร์ พารามิเตอร์ที่กำลังพิมพ์อยู่ใน «» พร้อมคำอธิบาย ปิดเมื่อพิมพ์ ) หรือ Esc - ทดสอบกับ rust-analyzer จริง | ✅ |
| ★ | เส้นหยักใต้ error / warning | ✅ |
| ★ | Quick Actions (Ctrl+.) | ⬜ |
| ★ | Go To Definition (F12, Ctrl+click; Edit menu), Peek Definition (Alt+F12: โค้ดตรงที่ประกาศในหน้าต่างใต้บรรทัด ไม่ต้องเปลี่ยนไฟล์, Esc ปิด) - ทดสอบกับ rust-analyzer จริง | ✅ (peek ยังแก้โค้ดในหน้าต่างไม่ได้) |
| ★ | Find All References (Shift+F12) — References tab, double-click to go | ✅ |
| ★ | Quick Find / Replace (Ctrl+F / Ctrl+H) | ✅ |
| | Find in Files (Ctrl+Shift+F, เมนู Edit): ทั้ง solution ข้าม target และโฟลเดอร์ซ่อน, ตรงตัวพิมพ์หรือไม่ก็ได้, ผลในรายการ References ดับเบิลคลิกเปิดตรงบรรทัด | ✅ |
| ★ | Rename (F2; Edit menu) — every file, open ones as one undo step | ✅ |
| | Comment toggle (Ctrl+/), ย้ายบรรทัด (Alt+↑↓ ไปพร้อมส่วนที่เลือก), ทำซ้ำบรรทัดหรือส่วนที่เลือก (Ctrl+D) - ย้อนกลับได้ทีละขั้น | ✅ |
| | Multi-caret, box selection | ⬜ |
| | พับโค้ด (outlining) | ⬜ |
| | จับคู่วงเล็บ: วงเล็บข้างเคอร์เซอร์และคู่ของมันมีกรอบสี ข้ามวงเล็บใน string, char และ comment | ✅ |
| | ซูม (Ctrl+ล้อ) | ✅ |
| | Format Document (rustfmt): Ctrl+K, Ctrl+D หรือเมนู Edit - ใช้ edition ของโปรเจกต์, ย้อนกลับได้ครั้งเดียว, เคอร์เซอร์อยู่ที่เดิม; Ctrl+K, Ctrl+C/U ใส่/เอา comment | ✅ |

## โปรเจกต์ / Build

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Solution Explorer | ✅ (Project explorer) |
| ★ | Build / Run (F5) / Stop | ✅ |
| ★ | Debug / Release | ✅ |
| ★ | Error List, ดับเบิลคลิกไปที่บรรทัด, F8 / Shift+F8 ไปยังข้อผิดพลาดถัดไป/ก่อนหน้า (เปิดไฟล์ที่บรรทัดนั้น) | ✅ |
| ★ | Output window | ✅ |
| | Rebuild / Clean (ทั้งโปรเจกต์และทั้ง solution ในเมนู Build) | ✅ |
| | Task List (View > Task List): comment ที่ขึ้นต้นด้วย TODO, FIXME, HACK, UNDONE ในไฟล์ .rs ทั้ง solution ดับเบิลคลิกเปิดตรงบรรทัด | ✅ |

## Debugger

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Breakpoints (F9, คลิกขอบซ้าย), F5 เริ่ม/ทำต่อ, F10/F11/Shift+F11, Break All, Shift+F5, หยุดเมื่อ panic, Locals (ตัวเลข bool char &str String struct), Call Stack, แถบ Debug — เขียนเองบน Windows debug API + dbghelp | ✅ |
| ★ | Watch (Shift+F9, ชื่อ / ทางอย่าง order.total / การเปรียบเทียบ), data tips (ชี้ตัวแปรตอนหยุด), จุดหยุดแบบมีเงื่อนไข (Alt+F9, จุดแดงมี +) | ✅ |
| ★ | แก้ค่าตัวแปรระหว่างดีบัก (ดับเบิลคลิกในแท็บตัวแปร: ตัวเลข bool char) | ✅ |
| | Hit count ในกล่อง Breakpoint Condition (= 3, >= 3, % 3 ทุกครั้งที่สาม; นับเฉพาะครั้งที่เงื่อนไขเป็นจริง เริ่มนับใหม่เมื่อแก้) - ทดสอบกับโปรแกรมที่วนลูปจริง ✅; แก้ค่า &str (สำเนาใหม่ในหน่วยความจำของโปรแกรม) และ String (ภายในความจุเดิม) ด้วยดับเบิลคลิกใน Locals, ฟิลด์ตัวเลขใน struct ผ่านทาง order.total ✅ | ✅ |

## หน้าต่าง

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | แผง dock แยก, เส้นแบ่งปรับขนาดทุกแผง (รวม Toolbox), ปิด/เปิดจาก View | ✅ |
| ★ | แถบ Standard: ย้อนกลับ/ไปข้างหน้า, โปรเจกต์ใหม่, ฟอร์มใหม่, เปิด, บันทึก, บันทึกทั้งหมด, Undo/Redo, กล่อง Debug/Release, ▶ ชื่อโปรเจกต์, Stop, Build | ✅ |
| ★ | แถบ Layout: จัดตามเส้นตาราง, จัดแนว 6 แบบ, ขนาดเท่ากัน/ตามเส้นตาราง, ระยะแนวนอน/ตั้ง (เท่ากัน/เพิ่ม/ลด/ชิด), กึ่งกลางฟอร์ม, หน้า/หลัง, Tab order; ปุ่มเป็นสีเทาเมื่อใช้ไม่ได้ | ✅ |
| | ▾ ท้ายแถบ: Add or Remove Buttons (ติ๊กเปิด/ปิดทีละปุ่ม จำไว้ครั้งหน้า), Reset Toolbar | ✅ |
| ★ | เมนู Build: Build/Rebuild/Clean Solution (ทั้ง workspace), Run Code Analysis (cargo clippy, Alt+F11), Build/Rebuild/Clean/Analysis ของโปรเจกต์, Debug/Release; เมนู Debug: Start (F5), Start Without Debugging (Ctrl+F5), Stop | ✅ (ยังไม่มี Batch Build) |
| | แถบเครื่องมืออยู่ในถาด: ที่จับลากย้ายตามแถว ข้ามแถว หรือขึ้นไปวางข้างแถบอื่น, » แสดงปุ่มที่ไม่พอที่ | ✅ |
| ★ | ปุ่มบนหัวแผง: ▾ Window Position (Dock / Auto Hide / Hide), 📌 Auto Hide, × ปิด | ✅ |
| ★ | ลากหัวแผงไปติดขอบซ้าย/ขวา/บน/ล่าง หรือลอย (Toolbox; กล่องสีบอกตำแหน่ง, Esc ยกเลิก, ดับเบิลคลิกหัวหน้าต่างลอยเพื่อกลับ) | รูปเพชรนำทางตอนลาก: ลูกศรกลางขอบทั้งสี่และเข็มทิศกลางจอ ชี้ตัวไหนติดขอบนั้น ตัวที่ชี้อยู่สว่างขึ้น ✅ ; แผง Project/Server Explorer/Properties ลากหัวไปไว้ซ้ายหรือขวาได้ (ไปด้วยกันทั้งชุด) | ✅ (แผงขวายังลอยหรือไปบน/ล่างไม่ได้) |
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
| | Publish / ตัวติดตั้ง: Build > Publish ถามชื่อผลิตภัณฑ์ เวอร์ชัน ผู้พัฒนา ทางลัดบนเดสก์ท็อป (จำไว้ใน phai-publish.txt) แล้วสร้างแบบ Release ได้ `publish/<ชื่อ>-<เวอร์ชัน>-setup.exe` ไฟล์เดียว กับโฟลเดอร์แบบพกพา; ตัวติดตั้ง (phai-setup, อัดด้วย deflate) ติดตั้งสำหรับผู้ใช้คนเดียวไม่ต้องขอสิทธิ์ admin ลงใน %LOCALAPPDATA%\Programs, ทางลัด Start menu/เดสก์ท็อป, อยู่ใน Settings > Apps, ถอนการติดตั้งได้หมดจด; ติดตั้งเงียบ `/S /D=โฟลเดอร์ /NOSHORTCUTS /NOREG`; ทดสอบติดตั้งและถอนจริงแล้ว | ✅ (ยังไม่มี: เซ็นชื่อโค้ด, ไฟล์อื่นนอกจาก exe, ติดตั้งให้ทุกคนในเครื่อง) |

## ข้อมูล (Server Explorer)

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Server Explorer: Data Connections → เซิร์ฟเวอร์ MySQL/MariaDB → ฐานข้อมูล → ตาราง/วิว → คอลัมน์ (อ่านเมื่อเปิดกิ่ง) | ✅ |
| ★ | Add Connection: เซิร์ฟเวอร์ พอร์ต ผู้ใช้ รหัสผ่าน ฐานข้อมูล, TLS, เชื่อใบรับรองของเซิร์ฟเวอร์เอง; ทดสอบก่อนเก็บ; รหัสผ่านเข้ารหัสด้วย DPAPI ของ Windows | ✅ |
| ★ | ตัวคุยโปรโตคอล MySQL ของ Phai เอง: mysql_native_password, caching_sha2_password, sha256_password (RSA ไม่มี TLS), TLS ผ่าน SChannel | ✅ |
| ★ | Show Table Data (1000 แถวแรก), New Query (ไฟล์ .sql) + Ctrl+Shift+E รัน, ผลในแท็บผลลัพธ์คิวรี | ✅ |
| | คัดลอกโค้ดสำหรับโปรแกรม: struct ของแถว + ฟังก์ชันอ่านทั้งตาราง (รหัสผ่านจากตัวแปรแวดล้อม) | ✅ |
| ★ | แก้ข้อมูลในตาราง: ดับเบิลคลิกช่องเพื่อแก้, คลิกขวา เพิ่มแถว / ลบแถว (ระบุแถวด้วย primary key) | ✅ |
| ★ | Prepared statements (COM_STMT_PREPARE/EXECUTE, ค่าแยกจาก SQL กัน SQL injection) | ✅ |
| ★ | PostgreSQL (โปรโตคอลเขียนเอง: TLS, SCRAM-SHA-256, MD5, $1 parameters) และ SQLite (ไฟล์ ผ่าน rusqlite) ใน Server Explorer — SQL Server ไม่ทำ: ต้องซื้อซอฟต์แวร์ | ✅ |
| ★ | ตารางใหม่ / ออกแบบตาราง: เปิดคิวรีพร้อม CREATE TABLE ตามภาษาของฐานข้อมูลนั้น และตัวอย่าง ALTER TABLE ให้แก้แล้วรัน | ✅ |
| ★ | Table Designer แบบ VS: ตารางคอลัมน์ (ชื่อ ชนิด ว่างได้ คีย์หลัก นับอัตโนมัติ ค่าเริ่มต้น) คลิกแก้ในช่อง, ดัชนี, คีย์นอก (ON DELETE/UPDATE), สคริปต์ที่จะรันแสดงสด, อัปเดต — MySQL/PostgreSQL ใช้ ALTER TABLE, SQLite สร้างตารางใหม่โดยเก็บแถวไว้ | ✅ |

