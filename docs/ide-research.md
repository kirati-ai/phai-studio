# Phai Studio เทียบ Visual Studio, Android Studio, Xcode (research 2026-11-20)

เป้า: ดูว่า IDE ใหญ่สามตัวทำอะไรดี แล้ว Phai Studio ควรเพิ่มอะไร เรียงตามความสำคัญ
ทิศทางที่เจ้านายตัดสิน (2026-11-20): **มือถือไปทาง PWA อย่างเดียว** ไม่ทำแอป native iOS/Android

แหล่งข้อมูลอยู่ท้ายไฟล์ (learn.microsoft.com, developer.android.com, jetbrains.com, developer.apple.com และ release notes)

---

## 1. สรุปสั้น

- **Phai มีครบเกือบทุกอย่างที่ VS มีในงาน WinForms แล้ว** (designer, properties+events, toolbox, server explorer,
  table designer, debugger, git, test, coverage, terminal, options, publish) — ช่องว่างที่เหลือไม่ใช่ "ไม่มีฟีเจอร์"
  แต่เป็น **การหาของเจอ** และ **การไหลของงาน** ซึ่งสามตัวนั้นทำดีกว่ามาก
- สิ่งที่ทั้งสามตัวมีตรงกันและ Phai ยังไม่มี:
  1. **ช่องค้นหาทุกอย่าง** (VS Ctrl+Q, Android Studio กด Shift สองครั้ง, Xcode Open Quickly) — หาคำสั่ง ไฟล์ ค่าตั้ง ได้ในช่องเดียว
  2. **ดูตัวอย่างหลายขนาดพร้อมกัน** (Android Studio Layout Validation / multipreview, Xcode Variants) — สำคัญมากสำหรับ PWA บนมือถือ
  3. **แท็บพรีวิว** (คลิกไฟล์เดียวเปิดชั่วคราว ถ้าแก้ค่อยกลายเป็นแท็บจริง) — VS และ Xcode 26
  4. **กฎ "เกิดอะไรแล้วให้แสดงอะไร"** (Xcode Behaviors, VS "Show Error List if build has errors", AS layouts)
  5. **ข้อผิดพลาดติดอยู่กับที่มันเกิด** — ป้ายบน control ใน outline, จำนวน error ที่คลิกได้ในแถบสถานะ/แถบบน
- ที่ควรยืมเฉพาะตัว:
  - VS: **Data Sources ลากตารางลงฟอร์ม** = ได้ grid + ปุ่มเลื่อนแถว + binding ในครั้งเดียว (ฟีเจอร์ "RAD" ที่สุด),
    **หน้าแจ้ง error ของ designer** แทนจอว่าง
  - Android Studio: **ข้อมูลตัวอย่างตอนออกแบบ** (ไม่ติดไปกับโปรแกรมจริง), **Infer Constraints** (วางอิสระแล้วกดทีเดียวได้ anchor/dock)
  - Xcode: **Ctrl+ลาก control เข้าโค้ด** สร้าง event/field, **Behaviors**, **แถบเครื่องมือจัดวางใต้ canvas**

---

## 2. เทียบทีละด้าน

สถานะ Phai: ✅ มี · 🟡 มีบางส่วน · ⬜ ยังไม่มี

### 2.1 เปิดโปรแกรม / โปรเจกต์ใหม่

| สิ่งที่ IDE ใหญ่ทำ | VS | AS | Xcode | Phai |
|---|---|---|---|---|
| หน้าต่างเริ่มต้น: ล่าสุด (ค้นหา, ปักหมุด) + New / Open / Clone | ✓ | ✓ | ✓ | ✅ ค้นหา, ปักหมุด (คลิกขวา), Clone, เคล็ดลับ |
| แม่แบบเป็นการ์ดมีรูป + ตัวกรอง (ภาษา/แพลตฟอร์ม/ชนิด) | ✓ | ✓ | ✓ | ✅ การ์ดมีรูป: โปรแกรม Windows / แอปมือถือ (PWA) |
| ขั้นตอน: เลือกแบบ → ชื่อ/ที่เก็บ → ตัวเลือก (เช่น "สร้าง git repo") | ✓ | ✓ | ✓ | 🟡 |
| "ช่วยเลือก" สำหรับค่าที่ตัดสินยาก (AS: Min SDK บอก % เครื่องที่รองรับ) | | ✓ | | ⬜ |

### 2.2 หน้าต่างหลัก

| | VS | AS | Xcode | Phai |
|---|---|---|---|---|
| ค้นหาทุกอย่าง (คำสั่ง, ไฟล์, ค่าตั้ง) ช่องเดียว | Ctrl+Q | Shift Shift / Ctrl+Shift+A | Shift+Cmd+O | ✅ Ctrl+Q ค้นคำสั่ง, หน้า Options, ไฟล์ |
| แท็บพรีวิว / ปักหมุดแท็บ | ✓ | ✓ | ✓ (26) | ✅ คลิกเดียว = แท็บพรีวิว, File > Pin Tab, Close All But Pinned |
| บันทึก/เรียก layout หน้าต่าง (Ctrl+Alt+1..0), Reset layout | ✓ | ✓ | Behaviors | ✅ View > Window Layouts (Ctrl+Alt+1..0) + Reset |
| โหมดลอย/ปักหมุด/ซ่อนอัตโนมัติ ทีละแผง | ✓ | 5 โหมด | — | ✅ (ลอยได้เฉพาะบางแผง) |
| breadcrumb เหนือ editor (โปรเจกต์ › ไฟล์ › ฟังก์ชัน) กดเลือกได้ | — | ✓ | Jump bar | ✅ breadcrumb: โฟลเดอร์ › ไฟล์ › mod/impl/fn ตาม caret (ยังกดเลือกไม่ได้) |
| แถบสถานะ: จำนวน error/warning คลิกไปรายการได้, สาขา git | ✓ | ✓ | Activity pill | ✅ จำนวน error/warning บนแถบสถานะ คลิกเปิด Error List (สาขา git ยังไม่มี) |
| Compact mode / Distraction free / เต็มจอ | ✓ | ✓ | Cmd+Return | ⬜ |

### 2.3 หน้าต่างเครื่องมือ

| | IDE ใหญ่ | Phai |
|---|---|---|
| Error List: ตัวกรองขอบเขต (ทั้งโปรเจกต์/ไฟล์ที่เปิด/ไฟล์นี้), แหล่ง (build/ขณะพิมพ์), เรียงคอลัมน์, F1 เปิดคำอธิบาย error | VS | ✅ ขอบเขต (ทั้งโซลูชัน/เอกสารนี้/ที่เปิด), คลิกหัวคอลัมน์เรียง |
| Output: เลือกช่อง (Build / Debug / Git / Tests) ไม่ปนกัน, ไปข้อความถัดไป | VS | ✅ Show output from: Build / Debug / Tests / Data |
| Output กรองแบบ query (`level:` `tag:` regex) + บันทึกตัวกรอง | AS Logcat | ⬜ |
| Terminal หลายแท็บ, profile | VS, AS | 🟡 มีแท็บเดียว |
| ประวัติ build/run ทุกครั้ง พร้อม log | Xcode Report navigator | ⬜ |
| Notifications รวมที่เดียว | VS, AS | ⬜ |

### 2.4 ตัวออกแบบฟอร์ม

| | IDE ใหญ่ | Phai |
|---|---|---|
| snapline, smart tag, Format menu, tab order, component tray, lock, document outline | VS | ✅ |
| **ดูหลายขนาดพร้อมกัน** (มือถือ/แท็บเล็ต/คอม, DPI, ธีมสว่าง/มืด, ภาษา, ขนาดตัวอักษร, ตาบอดสี) | AS, Xcode | ✅ View > Device Preview (Shift+F4): มือถือ แนวนอน แท็บเล็ต เรียงกัน |
| เลือกเครื่องที่ดู (iPhone/Pixel/ขนาดเอง) + หมุนจอ | AS, Xcode | ✅ เลือกเครื่อง + หมุนจอ |
| ข้อมูลตัวอย่างตอนออกแบบ (ไม่ติดไปกับโปรแกรมจริง) | AS `tools:` | ✅ View > Sample Data in Designer |
| วางอิสระแล้ว "Infer" เป็น anchor/dock ทีเดียว, ดึงขอบไปจับ control อื่น | AS, Xcode | ✅ Format > Infer Anchors |
| ป้ายปัญหา layout บน outline (ซ้อน/ล้นจอ) + แก้ทีเดียว | Xcode, AS | ⬜ |
| หน้าแจ้ง error ของ designer (ลิงก์ไปบรรทัด, "ข้ามแล้วเปิดต่อ") | VS | ✅ เปิดเป็นข้อความที่บรรทัดผิด + Error List |
| Ctrl+ลาก control เข้าโค้ด → สร้าง event/field | Xcode | ✅ ลากด้วยปุ่มขวาจาก control → เมนูอีเวนต์ / คัดลอก ui.id |
| Data Sources: ลากตาราง/คอลัมน์ลงฟอร์ม → grid หรือช่องกรอก + binding + ปุ่มเลื่อนแถว | VS | ✅ ลากตารางจาก Server Explorer ลงฟอร์ม |
| Properties: ค่าที่ตั้งเองเป็นตัวหนา, Reset, ค้นหา, เรียงตามหมวด (ตั้งแล้ว → layout → ทั่วไป → ทั้งหมด) | VS, AS | 🟡 |
| Toolbox: รายการโปรด, กลุ่ม UserControl ของโปรเจกต์อัตโนมัติ | VS, AS | 🟡 |
| Library แบบ popup ค้นหาได้ (Ctrl+Shift+L) แทนที่ Toolbox บนจอเล็ก | Xcode | ⬜ |
| แปลภาษาในฟอร์ม (เลือกภาษาแล้วแก้ข้อความแต่ละภาษา) | VS Localizable | 🟡 มี text.rs ไทย/อังกฤษ |

### 2.5 ตัวแก้โค้ด

| | IDE ใหญ่ | Phai |
|---|---|---|
| completion, quick actions (Ctrl+.), rename, peek, references, CodeLens, bookmarks, format | ทั้งสาม | ✅ (ผ่าน rust-analyzer) |
| หลาย cursor (Ctrl+Alt+คลิก, Alt+Shift+.) | VS, Xcode | ✅ มีอยู่แล้ว (Ctrl+Alt+คลิก, Alt+ลาก) |
| แถบเปลี่ยนแปลงข้างบรรทัด (git) คลิกดู/ย้อนเฉพาะก้อน | ทั้งสาม | ✅ แถบเขียว/น้ำเงิน/แดงตาม git diff (ย้อนเฉพาะก้อนยังไม่มี) |
| ปุ่ม ▶ ข้างเทสต์ในขอบซ้าย, เป็นเขียว/แดงหลังรัน | AS, Xcode | ✅ ▶ ข้าง #[test] รันเทสต์นั้น |
| ตัวช่วยเรียนรู้บน error: hover แล้วมีปุ่มแก้ | ทั้งสาม | 🟡 |
| แผนผังย่อ (minimap), sticky scroll | Xcode, VS | ⬜ |

### 2.6 Build / Run / Debug

| | IDE ใหญ่ | Phai |
|---|---|---|
| Debug/Release, Start, Attach, breakpoint หลายแบบ, watch, memory, disassembly, exception settings, diagnostic tools | VS | ✅ |
| logpoint (พิมพ์ข้อความแล้วไปต่อ ไม่หยุด) | ทั้งสาม | 🟡 มี condition |
| Hot Reload / Apply Changes | VS, AS | ⬜ (Rust ทำยาก — มี Live Inspector แทนสำหรับหน้าตา) |
| "รันบนมือถือ" เลือกเครื่องจากแถบบน | AS, Xcode | ✅ Build > Run on Phone: QR code + รีโหลดเอง |
| ถามเมื่อ build ไม่ผ่าน "รันตัวล่าสุดที่ผ่านไหม" | VS | ⬜ |

### 2.7 ค่าตั้ง

| | IDE ใหญ่ | Phai |
|---|---|---|
| ต้นไม้หมวด + ค้นหาในค่าตั้ง (ไฮไลต์ตัวที่ตรง) | ทั้งสาม | ✅ ช่อง Search Options |
| แก้ปุ่มลัด (keymap) + preset "Visual Studio" + ค้นด้วยการกดปุ่ม | ทั้งสาม | ✅ Tools > Keyboard Shortcuts (กดเพื่อกำหนด บอกว่าใครใช้อยู่) |
| Behaviors: build ไม่ผ่าน → แสดง Error List + ไป error แรก + เสียง | Xcode | ✅ เปิด error แรก, เสียง, กะพริบแถบงาน |
| นำเข้า/ส่งออกค่าตั้ง | VS | ✅ Options > Export... / Import... |

### 2.8 ข้อมูล

| | IDE ใหญ่ | Phai |
|---|---|---|
| เชื่อมต่อ, Server Explorer, ดูข้อมูลตาราง, ออกแบบตาราง, query | VS | ✅ |
| แก้ข้อมูลในช่องตาราง, live refresh, ส่งออก CSV/SQL, ประวัติ query | AS Database Inspector | 🟡 |
| ปุ่ม ▶ ข้าง SQL ในโค้ด (รัน query นั้นเลย) | AS | ⬜ |

### 2.9 Git / Test

| | IDE ใหญ่ | Phai |
|---|---|---|
| Git Changes (stage, commit, push), history, blame, merge 3 ทาง | VS | ✅ |
| Clone repository จากหน้าเริ่มต้น | ทั้งสาม | ✅ หน้าเริ่มต้น > Clone a repository |
| สาขาและจำนวนการเปลี่ยนแปลงที่แถบสถานะ | VS, AS | ⬜ |
| Test Explorer + รันเฉพาะที่ล้มเหลว + coverage | ทั้งสาม | ✅ (ยังไม่มี "รันเฉพาะที่ล้ม") |

### 2.10 ช่วยเหลือ / เรียนรู้

| | IDE ใหญ่ | Phai |
|---|---|---|
| F1 ตามบริบท (control/property/error ที่เลือก) | VS | 🟡 F1 เปิดคู่มือ |
| Tip of the day, บทเรียนในตัว (ทำตามแล้วตรวจให้) | AS | ✅ เคล็ดลับบนหน้าเริ่มต้น (บทเรียนในตัวยังไม่มี) |
| คู่มือภาษาไทย | — | ✅ (จุดแข็งของ Phai) |

---

## 3. มือถือ: ทาง PWA อย่างเดียว

เจ้านายตัดสินแล้วว่าไม่ทำ native ดังนั้นสิ่งที่ Android Studio / Xcode ทำเพื่อมือถือ ให้ทำในแบบ PWA:

| Android Studio / Xcode | ใน Phai (PWA) |
|---|---|
| เลือกเครื่อง Pixel / iPhone แล้วดูหน้าจอใน designer | **กรอบมือถือใน designer**: เลือก iPhone SE / iPhone 16 / Pixel / แท็บเล็ต, หมุนจอ, ดูหลายเครื่องพร้อมกัน |
| Layout Validation (หลายขนาดพร้อมกัน) | **ดูทุกขนาดในหน้าเดียว** มือถือ + แท็บเล็ต + คอม, สว่าง/มืด, ไทย/อังกฤษ, ตัวอักษรใหญ่ |
| ConstraintLayout / Auto Layout | **จัดวางที่ยืดตามจอ**: anchor/dock มีแล้ว — เพิ่ม "ซ้อนลงเมื่อจอแคบ" (flow แบบ responsive) และขนาดเป็น % |
| ปุ่ม Run → เลือก emulator / เครื่องจริง | **Run on Phone**: Studio เปิดเว็บเซิร์ฟเวอร์ในวง LAN แล้วโชว์ **QR code** สแกนด้วยมือถือเปิดได้ทันที แก้แล้วรีเฟรช |
| Running Devices (จอมือถือในตัว IDE) | **พรีวิวในเบราว์เซอร์แบบมือถือ** (ขนาดมือถือ + touch) ในแท็บของ Studio |
| ตั้งค่า app: ชื่อ, ไอคอน, สี, แนวจอ | **หน้า PWA ใน Project Properties**: ชื่อแอป, ไอคอน (เลือกรูป), สีธีม, แนวจอ, หน้าจอเริ่ม |
| Material / Human Interface Guidelines | **หน้าตามือถือในธีม**: ปุ่มและช่องสูงพอแตะ (44–48px), แถบล่าง (bottom tabs), รายการเลื่อนนิ้ว |
| Store | ไม่มี — อัปโหลดโฟลเดอร์ `web/` ขึ้น HTTPS (มีคำสั่ง Publish to web อยู่แล้ว) |

---

## 4. แผน เรียงตามความสำคัญ

> **ทำครบทั้ง 4 ระยะแล้ว (2026-11-20)** - รายละเอียดแต่ละข้อดูช่อง "Phai" ในตารางข้างบน
> ที่ยังต้องทดสอบด้วยมือ: สแกน QR ด้วยมือถือจริง, ลากตารางจาก Server Explorer (ต้องมีฐานข้อมูล),
> ลากด้วยปุ่มขวาเข้าโค้ด, Clone จาก GitHub จริง


### ระยะ 1 — มือถือ (PWA) ให้ใช้ได้จริง  ← ทางที่เจ้านายเลือก
1. กรอบมือถือ/แท็บเล็ตใน designer + หมุนจอ + ดูหลายขนาดพร้อมกัน
2. Run on Phone: เซิร์ฟเวอร์ในวง LAN + QR code + รีเฟรชเองเมื่อ build ใหม่
3. หน้า PWA ใน Project Properties (ชื่อ ไอคอน สี แนวจอ)
4. layout ยืดตามจอ: ซ้อนลงเมื่อจอแคบ, ขนาดเป็น %
5. ธีมมือถือ: ขนาดแตะได้, แถบล่าง, รายการเลื่อน
6. ทดสอบบน Android และ iPhone จริง

### ระยะ 2 — หาของเจอ ทำงานลื่น
1. ช่องค้นหาทุกอย่าง Ctrl+Q (คำสั่ง ไฟล์ ค่าตั้ง control)
2. แท็บพรีวิว + ปักหมุดแท็บ
3. Output แยกช่อง (Build / Run / Git / Tests) + Error List กรองขอบเขตและเรียงคอลัมน์ + จำนวน error คลิกได้ที่แถบสถานะ
4. Behaviors: เมื่อ build ไม่ผ่าน / หยุดที่ breakpoint / รันจบ → แสดงอะไร (ต่อยอดจาก Options > Error List)
5. บันทึก layout หน้าต่าง (Ctrl+Alt+1..0) + Reset layout

### ระยะ 3 — designer แบบ RAD
1. Data Sources: ลากตาราง/คอลัมน์ลงฟอร์ม เลือกได้ว่าจะได้ grid หรือช่องกรอก
2. หน้าแจ้ง error ของ designer
3. ข้อมูลตัวอย่างตอนออกแบบ
4. Infer anchors (วางอิสระแล้วกดทีเดียว)
5. Ctrl+ลาก control เข้าโค้ด

### ระยะ 4 — ขัดเกลา
- breadcrumb เหนือ editor, หลาย cursor, ปุ่ม ▶ ข้างเทสต์, แถบเปลี่ยนแปลง git ข้างบรรทัด
- ค้นหาใน Options, แก้ปุ่มลัด, นำเข้า/ส่งออกค่าตั้ง
- Clone repository + ค้นหา/ปักหมุดในหน้าเริ่มต้น + แม่แบบมีรูป
- Tip of the day / บทเรียนในตัว (ภาษาไทย)

---

## แหล่งข้อมูล

Visual Studio
- https://learn.microsoft.com/en-us/visualstudio/ide/create-new-project
- https://learn.microsoft.com/en-us/visualstudio/ide/customizing-window-layouts-in-visual-studio
- https://learn.microsoft.com/en-us/visualstudio/ide/reference/error-list-window
- https://learn.microsoft.com/en-us/visualstudio/ide/reference/output-window
- https://learn.microsoft.com/en-us/visualstudio/ide/reference/command-prompt-powershell
- https://learn.microsoft.com/en-us/visualstudio/designers/walkthrough-windows-forms-designer
- https://learn.microsoft.com/en-us/visualstudio/data-tools/add-new-data-sources
- https://learn.microsoft.com/en-us/visualstudio/debugger/hot-reload
- https://learn.microsoft.com/en-us/visualstudio/version-control/git-make-commit

Android Studio
- https://developer.android.com/studio/intro/user-interface
- https://developer.android.com/studio/intro/keyboard-shortcuts
- https://developer.android.com/studio/projects/create-project
- https://developer.android.com/studio/views/layout-editor
- https://developer.android.com/develop/ui/compose/tooling/previews
- https://developer.android.com/studio/debug/logcat
- https://developer.android.com/studio/inspect/database
- https://www.jetbrains.com/help/idea/new-ui.html

Xcode
- https://developer.apple.com/documentation/xcode-release-notes/xcode-26-release-notes.md
- https://developer.apple.com/documentation/xcode-release-notes/xcode-16-release-notes.md
- https://help.apple.com/xcode/mac/current/en.lproj/dev66189ede4.html (Behaviors)
- https://developer.apple.com/documentation/xcode/previewing-your-apps-interface-in-xcode
- https://developer.apple.com/videos/play/wwdc2023/10165
