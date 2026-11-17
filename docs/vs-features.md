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
| ★ | Snaplines ขอบ/ระยะขอบ และเส้นฐานตัวอักษร (baseline) ให้คำของ label ตรงกับคำในกล่องที่สูงกว่า | ✅ |
| ★ | Format > Align / Make Same Size / Spacing | ✅ |
| ★ | Bring to Front / Send to Back | ✅ (ลำดับในฟอร์ม = ชั้น = ลำดับ Tab แบบ Win32) |
| ★ | Tab Order mode | ✅ |
| ★ | Lock Controls: ทั้งฟอร์ม (Format > Lock) และราย control (Properties > Locked = True - เมาส์และลูกศรไม่ย้าย/ไม่ปรับขนาด, ไม่มีผลกับโปรแกรมจริง) | ✅ |
| ★ | Dock / Anchor (ตัวแก้แบบภาพ) | ✅ |
| ★ | Control: DateTimePicker, NumericUpDown, TrackBar, ProgressBar, TextBox หลายบรรทัด | ✅ |
| | Control: Tree view (รายการเว้นหน้าบรรทัด, on_select), Picture (PNG/JPEG/BMP/GIF ผ่าน WIC), Timer (on_tick), Rich text box (RichEdit ของ Windows), Data grid (แก้ในช่องได้: ดับเบิลคลิก/F2/Enter/Esc, ดับเบิลคลิกใต้แถวสุดท้ายเพิ่มแถว, Delete ลบแถว) - ทั้งหน้าออกแบบ, โค้ดที่สร้าง และโปรแกรมที่ build | ✅ |
| ★ | Smart tags: ปุ่ม ▸ ที่มุมขวาบนของ control ที่เลือก เปิดเมนูงานที่ใช้บ่อย - แก้ข้อความ, แก้รายการ/คอลัมน์/แท็บ, เลือกรูป, เต็มพื้นที่ในตัวที่บรรจุ, เขียนเมธอด event, ล็อก | ✅ |
| ★ | Component tray: แถบใต้ฟอร์มแสดง Timer เป็นชิปพร้อมไอคอน คลิกเพื่อเลือก, Properties แสดง Interval/Enabled | ✅ |
| | Document Outline (View, Ctrl+Alt+T): ต้นไม้ของ control ในฟอร์ม ตัวที่อยู่ใน Group/Tabs อยู่ใต้มัน มีไอคอนตาม Toolbox เลือกในต้นไม้ = เลือกบนฟอร์ม และกลับกัน | ✅ |
| | TableLayout / FlowLayout: กล่อง Flow เรียง control ที่อยู่ข้างในจากซ้ายไปขวาขึ้นแถวใหม่เมื่อเต็ม, กล่อง Table วางเป็นคอลัมน์เท่า ๆ กัน (Columns ตั้งได้) - ทั้งหน้าออกแบบและโปรแกรมที่ build | ✅ |
| | ตัวแก้เมนู/toolbar | ✅ (แบบข้อความ) |
| | Localization | ✅ (ตาราง ไทย/อังกฤษ + เทสต์) |

## ตัวแก้โค้ด

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | IntelliSense หลัง `.` `::`, Ctrl+Space | ✅ (rust-analyzer) |
| ★ | Quick Info (hover) | ✅ |
| ★ | Parameter Info (Ctrl+Shift+Space, และเมื่อพิมพ์ ( หรือ ,): signature ของฟังก์ชันใต้เคอร์เซอร์ พารามิเตอร์ที่กำลังพิมพ์อยู่ใน «» พร้อมคำอธิบาย ปิดเมื่อพิมพ์ ) หรือ Esc - ทดสอบกับ rust-analyzer จริง | ✅ |
| ★ | เส้นหยักใต้ error / warning | ✅ |
| ★ | Quick Actions (Ctrl+., เมนู Edit): การแก้ไขและปรับโครงสร้างจาก rust-analyzer เป็นเมนูใต้เคอร์เซอร์ การแก้ไขขึ้นก่อน เลือกแล้วแก้ทุกไฟล์ที่เกี่ยว - ทดสอบ Insert explicit type จริง | ✅ |
| ★ | Go To Definition (F12, Ctrl+click; Edit menu), Peek Definition (Alt+F12: โค้ดตรงที่ประกาศในหน้าต่างใต้บรรทัด ไม่ต้องเปลี่ยนไฟล์, Esc ปิด) - ทดสอบกับ rust-analyzer จริง ; หน้าต่าง peek เป็นตัวแก้โค้ดจริง แก้ได้ - ไฟล์ที่เปิดเป็นแท็บอยู่เปลี่ยนตามทันที ไฟล์ที่ไม่ได้เปิดบันทึกเมื่อปิด peek | ✅ |
| ★ | Find All References (Shift+F12) — References tab, double-click to go | ✅ |
| ★ | Quick Find / Replace (Ctrl+F / Ctrl+H) | ✅ |
| | Find in Files (Ctrl+Shift+F, เมนู Edit): ทั้ง solution ข้าม target และโฟลเดอร์ซ่อน, ตรงตัวพิมพ์หรือไม่ก็ได้, ผลในรายการ References ดับเบิลคลิกเปิดตรงบรรทัด | ✅ |
| ★ | Rename (F2; Edit menu) — every file, open ones as one undo step | ✅ |
| | Comment toggle (Ctrl+/), ย้ายบรรทัด (Alt+↑↓ ไปพร้อมส่วนที่เลือก), ทำซ้ำบรรทัดหรือส่วนที่เลือก (Ctrl+D) - ย้อนกลับได้ทีละขั้น | ✅ |
| | Multi-caret, box selection: Alt+ลาก เลือกเป็นกล่อง (เคอร์เซอร์ทุกบรรทัด), Ctrl+Alt+คลิก เพิ่มเคอร์เซอร์, Shift+Alt+. เลือกคำเดียวกันถัดไป; พิมพ์/Backspace/Delete/Enter ทำทุกเคอร์เซอร์ ย้อนกลับครั้งเดียว, Esc เหลือเคอร์เซอร์เดียว | ✅ |
| | พับโค้ด (outlining): กล่อง −/+ ในขอบซ้ายของทุกบล็อก { } ที่ยาวกว่าหนึ่งบรรทัด, บล็อกที่พับแสดง …} ต่อท้าย, Ctrl+M Ctrl+M พับบล็อกที่เคอร์เซอร์อยู่, Ctrl+M Ctrl+L พับ/ขยายทั้งหมด, ลูกศรขึ้นลงข้ามบล็อกที่พับ, ไปที่บรรทัดในบล็อกที่พับจะขยายให้เอง | ✅ |
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
| ★ | เมนู Build: Build/Rebuild/Clean Solution (ทั้ง workspace), Run Code Analysis (cargo clippy, Alt+F11), Build/Rebuild/Clean/Analysis ของโปรเจกต์, Debug/Release; เมนู Debug: Start (F5), Start Without Debugging (Ctrl+F5), Stop | Batch Build (ติ๊กโปรเจกต์ × Debug/Release แล้ว Build/Rebuild/Clean ทีละตัว) ✅ |
| | แถบเครื่องมืออยู่ในถาด: ที่จับลากย้ายตามแถว ข้ามแถว หรือขึ้นไปวางข้างแถบอื่น, » แสดงปุ่มที่ไม่พอที่ | ✅ |
| ★ | ปุ่มบนหัวแผง: ▾ Window Position (Dock / Auto Hide / Hide), 📌 Auto Hide, × ปิด | ✅ |
| ★ | ลากหัวแผงไปติดขอบซ้าย/ขวา/บน/ล่าง หรือลอย (Toolbox; กล่องสีบอกตำแหน่ง, Esc ยกเลิก, ดับเบิลคลิกหัวหน้าต่างลอยเพื่อกลับ) | รูปเพชรนำทางตอนลาก: ลูกศรกลางขอบทั้งสี่และเข็มทิศกลางจอ ชี้ตัวไหนติดขอบนั้น ตัวที่ชี้อยู่สว่างขึ้น ✅ ; แผง Project/Server Explorer/Properties ลากหัวไปไว้ซ้ายหรือขวาได้ (ไปด้วยกันทั้งชุด)  ไปซ้าย/ขวา/บน/ล่าง หรือลอยเป็นหน้าต่างของตัวเอง (ดับเบิลคลิกหัวหน้าต่างเพื่อกลับ); ตำแหน่งของแผงขวาและ Toolbox จำไว้เปิดครั้งหน้า | ✅ |
| ★ | Auto Hide: แท็บแนวตั้งที่ขอบ, ชี้หรือคลิกแล้วเลื่อนออก, ออกจากแผงแล้วเก็บเอง (แผงขวาซ่อนพร้อมกันเป็นกลุ่ม) | ✅ |
| ★ | แท็บเอกสาร, จุดบอกยังไม่บันทึก | ✅ |
| | Split editor (View > Split): ไฟล์เดียวกันสองช่องบน-ล่าง ลากเส้นแบ่งปรับขนาดได้; View > New Window: เปิดไฟล์ในหน้าต่างของตัวเอง ย้ายไปจออื่นได้ พิมพ์ที่ไหนก็เห็นทุกที่ | ✅ |

## อื่นๆ

| | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Git Changes (เมนู Git, Ctrl+Alt+G): branch, ไฟล์ที่เปลี่ยน (แก้/ใหม่/ลบ/เปลี่ยนชื่อ), ดับเบิลคลิกดู diff, ช่องข้อความ, Commit All, Pull, Push, Refresh - ทดสอบ commit จริงใน repo ทดลอง ; Stage/Unstage ทีละไฟล์ แล้ว Commit Staged เฉพาะที่ stage ไว้ (ทดสอบแล้ว) | ✅ |
| | Start window: เมื่อเปิดโดยไม่มีไฟล์ - รายการที่เปิดล่าสุด, เปิดฟอร์ม/ไฟล์โค้ด, สร้างโปรเจกต์ใหม่, ทำต่อโดยไม่เปิดโค้ด; ปิดไม่ให้แสดงได้ และเปิดอีกได้จาก View > Start Window | ✅ |
| | ธีมสว่าง/มืด, ฟอนต์ | ✅ |
| | ไฟล์ล่าสุด | ✅ |
| | Test Explorer: เมนู Test > Run Tests (Ctrl+R, T) / Run All Tests (Ctrl+R, A) รัน cargo test แล้วแท็บ Tests แสดงทุกเทสต์ ไม่ผ่านขึ้นก่อนพร้อมข้อความ ดับเบิลคลิกเปิดฟังก์ชันเทสต์ | ✅ |
| | Publish / ตัวติดตั้ง: Build > Publish ถามชื่อผลิตภัณฑ์ เวอร์ชัน ผู้พัฒนา ทางลัดบนเดสก์ท็อป (จำไว้ใน phai-publish.txt) แล้วสร้างแบบ Release ได้ `publish/<ชื่อ>-<เวอร์ชัน>-setup.exe` ไฟล์เดียว กับโฟลเดอร์แบบพกพา; ตัวติดตั้ง (phai-setup, อัดด้วย deflate) ติดตั้งสำหรับผู้ใช้คนเดียวไม่ต้องขอสิทธิ์ admin ลงใน %LOCALAPPDATA%\Programs, ทางลัด Start menu/เดสก์ท็อป, อยู่ใน Settings > Apps, ถอนการติดตั้งได้หมดจด; ติดตั้งเงียบ `/S /D=โฟลเดอร์ /NOSHORTCUTS /NOREG`; ทดสอบติดตั้งและถอนจริงแล้ว ; ใส่ไฟล์/โฟลเดอร์อื่นของโปรเจกต์ไปด้วย (รูปที่ฟอร์มใช้ใส่ให้เอง), เลือกติดตั้งให้ทุกคนในเครื่องได้ (Program Files, Start menu ของทุกคน, HKLM - Windows ขอสิทธิ์ผู้ดูแลระบบ; ส่วนนี้ยังไม่ได้ทดสอบจริงเพราะต้องให้คนกดอนุญาต) | ✅ (ไม่ทำ: เซ็นชื่อโค้ด เพราะต้องซื้อใบรับรอง) |

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

## รอบเพิ่มเติม (ของที่ VS มีแต่ไม่อยู่ในรายการแรก)

| สำคัญ | ความสามารถ | สถานะ |
|---|---|---|
| ★ | Go to All (Ctrl+T, เมนู Edit): พิมพ์ชื่อแล้วเจอทั้งไฟล์และสัญลักษณ์ในโค้ด (ฟังก์ชัน, struct ฯลฯ จาก rust-analyzer) ขึ้น/ลง/Enter ไปที่นั่น | ✅ |
| ★ | Toolbox กลุ่ม Menus and toolbars: Menu bar (MenuStrip) และ Toolbar (ToolStrip) เปิดตัวแก้ของฟอร์ม, Context menu (ContextMenuStrip) เป็นคอมโพเนนต์ใน tray ใส่รายการ "เมธอด \| ข้อความ" แล้วเลือกให้ control ไหนก็ได้ใน Properties > Context menu - คลิกขวาในโปรแกรมแล้วขึ้นเมนู | ✅ |
| | Bookmarks: Ctrl+K Ctrl+K ใส่/เอา, Ctrl+K Ctrl+N ถัดไป, Ctrl+K Ctrl+P ก่อนหน้า, Ctrl+K Ctrl+L ล้าง - ธงในขอบซ้าย ย้ายตามบรรทัดเมื่อแก้โค้ด | ✅ |
| | ค้นหา/แทนที่: ตรงตัวพิมพ์ (Aa), ทั้งคำ (ab|), regular expression (.*) แทนที่ด้วยกลุ่ม $1 ได้ | ✅ |
| ★ | Project > Properties (Alt+F7): ชื่อแพ็กเกจ, เวอร์ชัน, Rust edition, คำอธิบาย, ผู้เขียน - เขียนลง Cargo.toml โดยไม่แตะส่วนอื่น | ✅ |
| ★ | Project > Manage Crates (แบบ NuGet Package Manager): ค้นหา crates.io (ชื่อ, เวอร์ชันล่าสุด, ยอดดาวน์โหลด, คำอธิบาย), รายการ crate ที่ใช้อยู่, Add/Remove ด้วย cargo add / cargo remove - ทดสอบค้นหา เพิ่ม และลบจริง | ✅ |
| ★ | ตัวติดตั้ง Phai Studio เอง: `dist/PhaiStudio-<เวอร์ชัน>-setup.exe` (cargo run --example make_installer) มี Phai Studio, phai-setup และไลบรารี phai-ui; โปรเจกต์ใหม่ใช้ phai-ui ที่ติดตั้งไว้ จึง build ได้บนเครื่องอื่น - ทดสอบติดตั้ง สร้างโปรเจกต์ build และถอนจริง | ✅ |
| | Code snippets: พิมพ์คำสั้น (fn, for, if, ifl, match, struct, impl, test, tests, main, pln ...) แล้วกด Tab ได้โค้ดทั้งก้อนตามระยะเยื้อง, Edit > Insert Snippet (Ctrl+K, Ctrl+X) | ✅ |
| | View Call Hierarchy (Ctrl+K, Ctrl+T): ใครเรียกฟังก์ชันที่เคอร์เซอร์อยู่ จาก rust-analyzer แสดงในรายการ References ดับเบิลคลิกไปที่นั่น (การเรียกในมาโครอย่าง println! rust-analyzer ไม่นับ) | ✅ |
| | Error List กรอง Errors (n) / Warnings (n) ที่ขวาของแท็บ ติ๊กเพื่อซ่อน/แสดง; F8 และดับเบิลคลิกไปตามรายการที่กรองแล้ว | ✅ |
| | Immediate Window (Debug, Ctrl+Alt+I): พิมพ์นิพจน์ระหว่างหยุดดีบัก (ชื่อตัวแปร, order.total, x > 5) Enter แล้วได้ค่าและชนิดใต้บรรทัด - ทดสอบกับโปรแกรมจริง | ✅ |
| | Document Outline ของไฟล์โค้ด: struct, impl, ฟังก์ชัน, ฟิลด์ ซ้อนกันตามที่เขียน (จาก rust-analyzer) อัปเดตเองหลังแก้โค้ด คลิกแล้วไปที่นั่น | ✅ |
| ★ | CodeLens: "ใช้ N แห่ง" ต่อท้ายบรรทัดของฟังก์ชัน, เมธอด, struct, enum, trait (นับจาก rust-analyzer) อัปเดตหลังแก้โค้ด คลิกแล้วได้รายการที่ใช้ | ✅ |
| ★ | Diagnostic Tools (Debug > Show Diagnostic Tools, Ctrl+Alt+F2): กราฟหน่วยความจำ (private bytes) และ CPU (% ของทุกตัวประมวลผล) ของโปรแกรมที่รัน (F5 หรือ Ctrl+F5) เก็บค่าทุกครึ่งวินาที ย้อนหลังหนึ่งนาที | ✅ |
| ★ | Attach to Process (Debug, Ctrl+Alt+P): เลือกโปรแกรมที่รันอยู่ (กรองตามชื่อ/รหัส โปรแกรมของโปรเจกต์ขึ้นก่อน) แล้วดีบักได้ทันที - breakpoint, step, locals; Stop Debugging หรือ Detach All ปล่อยให้โปรแกรมทำงานต่อ - ทดสอบแนบกับโปรแกรมที่วนอยู่จริง | ✅ |
| | หน้าต่าง Breakpoints (Debug, Ctrl+Alt+B): ทุก breakpoint ทุกไฟล์ พร้อมเงื่อนไขและ Hit Count, ติ๊กออกเพื่อปิดชั่วคราว (วงกลมกลวงในขอบซ้าย ดีบักเกอร์ไม่หยุด), Delete ลบ, ดับเบิลคลิกไปที่บรรทัด | ✅ |
