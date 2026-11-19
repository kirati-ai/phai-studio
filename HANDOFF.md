# Handoff — Phai Studio (2026-11-19)

> เปิด session ใหม่แล้วบอกว่า **"อ่าน HANDOFF.md แล้วทำต่อ"** · ตอบเจ้านายเป็นภาษาไทย เรียก "เจ้านาย"
> ส่วนท้ายไฟล์ (ตั้งแต่ "ประวัติเดิม") เป็นบันทึกเก่าวันที่ 16 พ.ย. ไว้อ้างอิงเท่านั้น

## สถานะตอนนี้

- repo สาธารณะ `kirati-ai/phai-studio` (MIT) · รุ่นล่าสุดที่แจก **v0.1.2** (GitHub Releases + https://phai-studio.monrak.net)
- โค้ดบน `main` ใหม่กว่า v0.1.2 (ยังไม่ออกรุ่น): ไม่มีหน้าต่างคอนโซลตอนรัน (ผลลัพธ์ขึ้นแท็บ Output), ฟอร์มเปิดกลางจอ /
  โปรเจกต์ใหม่เต็มจอ (`start`, `maximized`), เมนู Theme แสดงไฟล์ CSS ที่ใช้, โหมดมืด (แถบเครื่องมือ + scrollbar),
  ไอคอนในเมนู Format, ปุ่มปิดหน้าต่างล่าง, หน้าต่าง Project แบบ Solution Explorer (แถบปุ่ม, ค้นหา, Dependencies,
  ไฟล์ .rs ซ้อนใต้ฟอร์ม), แท็บเอกสารมีปุ่ม ✕ + คลิกกลางปิด, ดับเบิลคลิกฟอร์ม/control ไปที่โค้ด
- **ออกรุ่นใหม่** (เมื่อเจ้านายสั่ง — main มี Properties/Events/Methods แบบ WinForms แล้ว ควรเป็น 0.2.0): แก้ `version` ใน `Cargo.toml` (workspace) → commit → `git tag vX.Y.Z` → push tag
  GitHub Actions build ตัวติดตั้งให้เอง → แล้ว `bash site/deploy.sh` ให้เว็บชี้รุ่นใหม่
- การเซ็นโค้ด: SignPath **ไม่อนุมัติ** (ยังไม่มีคนรู้จักพอ) — สมัครใหม่ได้ภายหลัง ดู `docs/signpath-application.md`
  ทางฟรีอีกทาง: Microsoft Store (บัญชีนักพัฒนาฟรี, Microsoft เซ็นให้เฉพาะแพ็กเกจ **MSIX**) — ยังไม่ได้ทำ

## ทิศทาง (เจ้านายยืนยัน 2026-11-19): "DevExpress ฟรี สำหรับ Rust"

DevExpress เก่งเพราะ: ทำ control โปรแกรมธุรกิจเรื่องเดียว 25 ปี, วาดเองทั้งหมด (skin เดียวทั้งชุด), **grid ที่เก่งที่สุด**
(จัดกลุ่ม กรองแบบ Excel ผลรวม master-detail ตัวแก้ค่าในช่อง server mode หลายล้านแถว ส่งออก Excel/PDF),
ชุดที่ทำงานด้วยกัน (grid รายงาน กราฟ pivot ribbon docking), designer/wizard ใน VS — ราคาหลักหมื่นบาทต่อคนต่อปี
Phai มีฐานเดียวกันแล้ว: **วาดเองด้วย CSS ธีม + ผูกฐานข้อมูล + ฟรี/โอเพนซอร์ส** + ผู้ใช้จริง (Phai ERP, โปรแกรมของ Monrak)
→ ให้เก่งสุดเรื่อง **grid และข้อมูล** ก่อน แล้วตามด้วย กราฟ, รายงานพิมพ์, Ribbon/docking — ทุกตัวใช้ธีม CSS และ binding เดียวกัน

## หลักของ control (ยืนยันกับเจ้านาย 2026-11-19)

- **CSS = หน้าตา** (สี ขอบ มุมมน ฟอนต์) · **`.phaiform` = โครงสร้าง ข้อมูล พฤติกรรม** (ตำแหน่ง ขนาด Anchor/Dock,
  Text, Checked, Items, Enabled, Password, events → ชื่อฟังก์ชัน) · **`.rs` ของผู้ใช้ = ทำอะไรเมื่อเกิดเหตุการณ์**
- ลำดับชั้นของหน้าตา (ชั้นหลังทับชั้นก่อน): ธีมในตัว (`phai:light` / `phai:dark`) → `theme.css` ของโปรเจกต์
  (ค่าเริ่มต้นของทุกตัว) → class (`button.primary`) → ค่าที่ตั้งรายตัวใน Properties (เก็บใน `.phaiform`, codegen เป็น `#id { }`)
- ต่างจาก C# ที่รวมทุกอย่างใน Designer.cs: Phai แยกหน้าตาไป CSS แบบเว็บ เปลี่ยนธีมทั้งโปรแกรมได้ในไฟล์เดียว

- **แท็บที่สามใน Properties: "Style" 🖌** (ต่อจาก Properties ⚙ และ Events ⚡) สำหรับ control ที่เลือก:
  ค่าจริงจากธีม + บอกว่ามาจากกฎไหน (`button.primary`) แก้ทับรายตัวได้, แยกตามสถานะ (ปกติ/ชี้/กด/โฟกัส/ปิดใช้)
  พร้อมตัวอย่างบนฟอร์ม, transition (ระยะเวลา + easing), animation สำเร็จรูป (จางเข้า กระพริบ หมุน),
  ปุ่ม "ส่งไปที่ธีม" ย้ายค่าที่แต่งเป็น class ใน theme.css — แท็บ Properties เดิม (Back colour, Font) คงไว้สำหรับคนจาก C#
- **CSS transition / animation** (phai-ui ยังไม่รองรับ): ลำดับ `transition` (สี ขอบ opacity — ไล่ค่าด้วยตัวจับเวลา
  ในตัววาดของ phai-ui) → `@keyframes` + `animation` → `transform` บางส่วน (control ของ Windows หมุน/ขยับแบบเว็บไม่ได้ทั้งหมด)
- **Theme Editor** แบบ Skin Editor ของ DevExpress: หน้าต่างเลือกสี/ขอบ/มุมมน เห็น control ทุกชนิดเปลี่ยนทันที
  ไม่ต้องเขียน CSS เอง แล้วบันทึกเป็น theme.css

## บันทึกจากการคุยกับเจ้านาย (2026-11-19) — อย่าลืม

- **คำแนะนำตัว:** "เครื่องมือ RAD สำหรับ Rust: ลากวางออกแบบหน้าจอ (visual designer) แล้วเขียนโค้ดแบบ event-driven"
  (RAD = Rapid Application Development แนว Visual Basic / Delphi / WinForms)
- **control = object** มี Properties (ลักษณะ) + Methods (สั่งให้ทำ) + Events (บอกเมื่อเกิดขึ้น); ทุกตัวมีส่วนร่วมจาก `Control`
- **ตารางอ้างอิง WinForms → Phai** (ใช้เริ่ม `docs/winforms-parity.md`):

  | Control | Properties หลัก | Methods หลัก | Events หลัก |
  |---|---|---|---|
  | Button | Text, Enabled, Visible, BackColor, Font | PerformClick, Focus | Click, Enter, Leave, KeyDown, MouseEnter |
  | Label | Text, AutoSize, TextAlign | — | Click |
  | TextBox | Text, ReadOnly, MaxLength, PasswordChar, Multiline | Clear, SelectAll, AppendText, Focus | TextChanged, KeyDown, KeyPress, Enter, Leave |
  | CheckBox / RadioButton | Checked, Text | — | CheckedChanged, Click |
  | ComboBox | Items, SelectedIndex, SelectedItem, DropDownStyle | Items.Add, Items.Clear | SelectedIndexChanged, TextChanged |
  | ListView | Columns, Items, SelectedItems, FullRowSelect | Items.Add, Items.Clear, EnsureVisible | SelectedIndexChanged, DoubleClick, ColumnClick |
  | DataGridView | Columns, Rows, DataSource, ReadOnly | Rows.Add, Refresh | CellValueChanged, SelectionChanged, CellDoubleClick |
  | DateTimePicker | Value, Format, MinDate, MaxDate | — | ValueChanged |
  | NumericUpDown | Value, Minimum, Maximum, Increment | UpButton, DownButton | ValueChanged |
  | TrackBar / ProgressBar | Value, Minimum, Maximum | PerformStep | ValueChanged / Scroll |
  | PictureBox | Image, SizeMode | Load | Click |
  | TreeView | Nodes, SelectedNode | ExpandAll, CollapseAll | AfterSelect, NodeMouseDoubleClick |
  | TabControl | TabPages, SelectedIndex | SelectTab | SelectedIndexChanged |
  | RichTextBox | Text, Rtf, SelectionFont | LoadFile, SaveFile, Find | TextChanged |
  | Timer | Interval, Enabled | Start, Stop | Tick |
  | ContextMenu | Items | Show | Opening, ItemClicked |
  | ทุกตัว (Control) | Name, Location, Size, Anchor, Dock, Enabled, Visible, TabIndex | Focus, Show, Hide, Refresh | Enter, Leave, KeyDown/Up, MouseDown/Up/Enter/Leave, Resize |

- **ไลบรารี control ของ C#** (บทเรียน): เสียเงิน — DevExpress (เด่นสุด), Telerik, Syncfusion (มี community license ฟรี),
  Infragistics, ComponentOne, Guna UI2 / Bunifu (หน้าตาโมเดิร์น); ฟรี — Krypton Toolkit, ReaLTaiizor, ObjectListView,
  ScintillaNET, ScottPlot / OxyPlot / LiveCharts (กราฟ), FastReport Open Source (รายงาน)
  สิ่งที่คนยอมจ่าย = grid เก่ง, กราฟ, รายงานพิมพ์, Ribbon/docking, ธีมสวย → Phai ให้ฟรีในตัว = จุดขาย
- **CSS** = Cascading Style Sheets (cascading = ไหลลงเป็นชั้น); แนวเดียวกับ Skin (DevExpress), Style (WPF), QSS (Qt)
  ภาพเคลื่อนไหวใน CSS เรียก `transition`, `@keyframes` + `animation`, `transform`, `opacity`, easing (เป็น property / at-rule ไม่ใช่ tag)
- ข้อมูลไลบรารี/ราคาเป็นความรู้ที่มี ไม่ได้ค้นเว็บ — ตรวจใหม่ก่อนใช้อ้างอิงสาธารณะ

## ทำเสร็จแล้ว 2026-11-19 (งานข้อ 0-3 เดิม) — checklist: `docs/winforms-parity.md`

- **0. Properties แบบ WinForms** ตารางเดียว `crates/phai-form/src/props.rs` (ชื่อ, ชนิดค่า, ค่าเริ่มต้น, control ที่มี, หมวด,
  คำอธิบาย) → หน้า Properties, ตัวตรวจไฟล์, codegen, canvas อ่านจากตารางนี้หมด; ในไฟล์เป็น `ชื่อ = ค่า` ธรรมดา
  (`Control.more` / `Form.more`, serde flatten) ไม่ใส่บรรทัดค่าเริ่มต้น; control เรียก `phai_ui::props::<ชื่อ>(h, ค่า)`
  ฟอร์ม: AcceptButton/CancelButton, FormBorderStyle, Maximize/Minimize/ControlBox, TopMost, ShowInTaskbar, Opacity, Icon,
  Min/MaxSize, Back/ForeColor (ทั้งโปรแกรม), KeyPreview — ฟังก์ชันใน `phai_ui::form`; `show_dialog` คืน `DialogResult`
  control: ReadOnly, MaxLength, PlaceholderText, TextAlign, CharacterCasing, ToolTip, Cursor, TabIndex/TabStop, DialogResult,
  View/GridLines/CheckBoxes/MultiSelect/HeaderStyle, เส้นของ tree, Format/CustomFormat/Value/Min/MaxDate, tick ของ slider,
  progress แบบ marquee, SizeMode, FlowDirection/WrapContents/Padding ... ค่าสีที่ไม่ได้ตั้งแสดงค่าจริงจากธีม (ตัวธรรมดา)
- **1. Events** ตาราง `crates/phai-form/src/events.rs`; ในไฟล์ `events = { key_down = "name_key_down" }`;
  `phai_ui::events::hook` subclass control แล้วส่ง `WM_EVENT` ไปที่ฟอร์ม; args: KeyEventArgs (`set_handled`),
  KeyPressEventArgs, MouseEventArgs, ClosingEventArgs (`cancel`), ItemEventArgs; ฟอร์ม: Shown, FormClosing, FormClosed,
  Resize, Activated, Deactivate, เมาส์, คีย์ (KeyPreview); หน้า Events จัดหมวดแบบ VS ดับเบิลคลิกชื่อ = เขียนเมธอดพร้อม args
- **2. Methods แบบ object** `phai_ui::ctl`: ชนิดต่อ control (TextBox, ComboBox, ListView, Grid, TreeView, Timer ...)
  field ใน `<Form>Ui` เป็นชนิดเหล่านี้, `ui.x.hwnd()` / `*ui.x` คือ HWND; ฟังก์ชัน phai-ui ที่โค้ดผู้ใช้เคยเรียกด้วย `ui.x`
  รับได้ทั้งสองแบบ (`impl Into<HWND>`) โค้ดเก่าไม่พัง ยกเว้นที่ส่ง `ui.x` ให้ Win32 ตรง ๆ (ใส่ `.hwnd()`)
  ฟอร์ม: `close`, `hide`, `show_window`, `activate`, `center_to_screen`, `set_title`, `invoker()`/`begin_invoke`/`invoke`
  (`phai_ui::invoke`: งานจาก thread อื่นไปทำบน thread ของหน้าต่าง พร้อม state ของโปรแกรม)
  ช่องเลือก object ใน Properties แสดง `name  phai_ui::ctl::TextBox` แบบ VS
- **3. เล็กน้อย** โหมดมืด: เส้นใต้แถบเครื่องมือ (RB_SETCOLORSCHEME) + gripper วาดใหม่ + ข้อความปุ่ม "Start" อ่านออก
  (TBCDRF_USECDCOLORS); ตรวจด้วย `--menu-test`: เมนู Format มีไอคอน, เมนู Theme แสดงไฟล์ CSS, ดับเบิลคลิกฟอร์มไปที่โค้ด;
  Store (MSIX): ติดตั้งใน `WindowsApps` หรือ setting `library_from_git = 1` → โปรเจกต์ใหม่อ้าง phai-ui/phai-data
  จาก git `tag = "v<รุ่น Studio>"` (ยังไม่ได้ทำแพ็กเกจ MSIX เอง)
- เทสต์โปรเจกต์จริง (ignored, รันด้วย `cargo test -p phai-studio add_form_tests -- --include-ignored`):
  `winforms_properties_reach_the_running_program`, `events_reach_the_programmers_methods`, `control_methods_and_invoke_work`

## งานถัดไป (เรียงตามความสำคัญ)

### 1. `docs/winforms-parity.md` ครบแล้ว (2026-11-19 รอบสอง) เหลือ 🟡 เดียว
- ทำเพิ่ม: BackColor/ForeColor รายหน้าต่าง (`win::window_background_for(h)` ดูหน้าต่างราก), Label AutoSize, CheckBox
  ThreeState/Appearance=Button, textarea WordWrap/ScrollBars (`win::textarea_with`), Date ShowCheckBox/ShowUpDown
  (`win::date_with`), TrackBar แนวตั้ง, TreeView LabelEdit, ฟอร์ม BackgroundImage/AutoScroll/Padding/AllowDrop,
  event DragDrop (`FileDropEventArgs`) — เทสต์ `creation_and_form_properties_reach_the_running_program`
- 🟡 BackgroundImage: label/checkbox วาดพื้นทึบทับรูป (ต้องให้ ctl_color คืน NULL_BRUSH เมื่อหน้าต่างมีรูป + invalidate
  พื้นใต้ label ตอนเปลี่ยนข้อความ)

### 2. แท็บ Style 🖌 และ Theme Editor — เสร็จ (2026-11-19) เหลือ animation
- ทำแล้ว: แท็บที่สามใน Properties (`apps/studio/src/props.rs` `gather_style`): เลือกสถานะ → ค่าจริงทุก property +
  ที่มา (`Theme::explain`, `win::explain_of`) → แก้ได้ (normal = ค่าใน Properties เดิม, สถานะอื่น = `Control.states`),
  canvas แสดง control ในสถานะนั้น (`win::set_preview_state`), Transition (ms + easing → prop `transition`),
  "ส่งไปที่ธีม" (`Studio::send_style_to_theme`); phai-ui: `transition` ใน CSS (`win::style_animated` ไล่สี
  background/color/border/accent/icon ด้วยตัวจับเวลา) ใช้กับปุ่ม/ช่องติ๊ก/radio, ขอบช่องกรอก, slider/progress
  ตรวจ: `--style N`, `--style-test`, เทสต์ `state_looks_and_transitions_reach_the_running_program`
- ต่อไป: animation สำเร็จรูป (`animation: blink 1s infinite`, fade-in = ไล่สีจากพื้นหลัง, pulse) — Win32 child
  window หมุน/โปร่งใสไม่ได้ (layered child ต้องมี manifest Windows 8+) จึงทำแบบไล่สี; transition ของแท็บและ list
  ยังไม่ทำ (วาดคนละทาง: controls.rs `tabs`, list custom draw)
- **Theme Editor เสร็จ (2026-11-19)** Theme > Theme Editor... (`apps/studio/src/themeedit.rs`): แท็บ "สี" (ตัวแปร
  `:root` 14 ตัว + เริ่มจากธีมสว่าง/มืด) และแท็บ "คอนโทรล" (ทุกชนิด 17 ตัว × 7 สถานะ × 8 property) ข้างตัวอย่าง control
  ทุกชนิดของจริง เปลี่ยนทันที (ฟอร์มที่ออกแบบก็เปลี่ยน) Save เขียนเฉพาะที่เปลี่ยน (`phai_ui::theme_edit`: set_var,
  set_base, set_prop, remove_prop - คอมเมนต์และลำดับเดิมอยู่ครบ) Cancel คืนธีมเดิม
  ตรวจ: `--theme-editor` + `PHAI_THEME_SHOT`, `PHAI_THEME_PAGE`, `PHAI_THEME_SET`, `PHAI_THEME_PROP`
- เจ้านายถามถึง HomeSite และ Adobe ImageStyler (2026-11-19) - ยังไม่ได้สั่งงาน; ที่เข้ากับ Phai: เงา/ไล่สี
  (`box-shadow`, `linear-gradient`) ใน CSS ของ phai-ui แบบ style ของ ImageStyler, ตัวแก้ HTML/CSS แบบ HomeSite

### 3. ทิศทาง DevExpress: grid ก่อน (จัดกลุ่ม, กรองแบบ Excel, ผลรวม, master-detail, ตัวแก้ในช่อง, ส่งออก Excel/PDF)

### 4. คู่มือไทย (`docs/help-src`) ยังไม่มีหน้าของ Properties ใหม่ / Events / `phai_ui::ctl` / invoke

### 5. dogfooding: ทำโปรแกรมจริงของ Monrak ด้วย Phai Studio (Desktop Server / Desktop Access) ขาดอะไรเพิ่มทันที

### 6. ออกรุ่น 0.2.0 เมื่อเจ้านายสั่ง (main เปลี่ยน API ของโค้ดที่สร้าง: field เป็น `phai_ui::ctl::*`)

## วิธีทำงาน (สำคัญ)
- เครื่องมือ: cargo อยู่ที่ `/d/Tools/Rust/cargo/bin`; ตรวจก่อน commit: `cargo test -q --workspace` + `cargo clippy -q --workspace --all-targets`
- เปลี่ยน codegen แล้วต้อง `cargo test -p phai-form -- --ignored regenerate` (ไฟล์ของ apps/sample)
- เทสต์ที่สร้างโปรเจกต์จริง: `cargo test -p phai-studio add_form_tests -- --include-ignored`
- ถ่ายภาพหน้าจอโดยไม่แย่งเมาส์/คีย์บอร์ด: `PHAI_SETTINGS_DIR=<tmp> PHAI_SHOT=out.png target/debug/phai-studio.exe [ไฟล์]`
- แพตช์ด้วย Python: เขียนสคริปต์เป็นไฟล์ด้วย Write tool (heredoc ใน bash ทำตัวขึ้นบรรทัดใหม่แบบ backslash-n เพี้ยน);
  ถ้าต้องใส่ backslash ในโค้ด Rust ผ่านสคริปต์ ใช้ `char::from(92)` / `char::from(10)` แทน
- เว็บ: `site/` (PHP) → `bash site/deploy.sh` (ใช้ key `~/.ssh/phaicity_vps`, aaPanel); monrak.net ต้นฉบับที่
  `D:\DesktopServer\www\monrak-site`; บอทถูกห้ามดาวน์โหลดทุกเว็บด้วย nginx `extension/<site>/no-bot-downloads.conf`
- คู่มือ: แก้ `docs/help-src/help_pages*.py` แล้ว `python docs/help-src/help_gen.py`
- กติกาเจ้านาย: commit + push ทุกรอบที่ผ่าน, ไม่ force-push, ไม่ใช้ของเสียเงิน, ไม่ใส่อีเมลเจ้านายในที่สาธารณะ,
  ห้ามรีบูต, หน้าต่างทดสอบห้ามแย่งเมาส์/คีย์บอร์ด

---

# ประวัติเดิม (2026-11-16)


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
- ตัวแก้โค้ดในตัวแล้ว: `crates/phai-editor` — piece table (เทสต์สุ่มเทียบ String), model (เคอร์เซอร์ข้ามสระไทย,
  ย่อหน้าอัตโนมัติ, undo ทีละคำ, Tab/Shift+Tab ทั้งบล็อก), สีโค้ด Rust/TOML/CSS, control Win32 (double buffer, เลขบรรทัด,
  เลือกด้วยเมาส์/คีย์บอร์ด, scroll, คลิปบอร์ด). ใน Studio: ดับเบิลคลิก .rs/.toml/.css ใน explorer = เปิดเป็นแท็บ,
  ดับเบิลคลิก error = ไปที่บรรทัด, Ctrl+S คงท้ายบรรทัด CRLF เดิม; `--codetest` พิมพ์ไทยด้วยข้อความแป้นพิมพ์แล้วบันทึก
  (เขียนตามแบบ Monaco ไม่ใช่แปลงทีละบรรทัด — แผน E แบบแปลงจริงยังทำต่อได้)
- rust-analyzer ต่อแล้ว (`phai-editor/src/lsp.rs`): เส้นหยักใต้ error (เปิด experimental diagnostics), hover, เติมโค้ด
  (Ctrl+Space, หลัง `.`/`::`); ไม่มี rust-analyzer → status บอกคำสั่ง `rustup component add rust-analyzer`
- ฟอนต์: ฟอร์ม/control มี font_size (Properties), ตัวแก้โค้ดซูม Ctrl+ล้อ / Ctrl+= - 0 จำค่าใน %APPDATA%\Phai Studio
- แผงแบบ dock: Project explorer และ Properties แยกแผง มีหัว + ✕, เส้นแบ่งลากได้, View เปิด/ปิด Toolbox/Explorer/Properties/Output
- Properties แบบ VS: dropdown เลือก object + แท็บ Properties | Events
- ทดสอบด้วยภาพโดยไม่แย่ง keyboard: `PHAI_SHOT` = แสดงหน้าต่างแบบไม่ activate + ทิ้งแป้นพิมพ์จริง
  (บทเรียน: ครั้งแรกหน้าต่างทดสอบรับตัวอักษรที่เจ้านายพิมพ์อยู่)
- **เหลือ**: ลากแผงไปติดขอบอื่น (re-dock), ค้นหา/แทนที่ในตัวแก้โค้ด, debugger, Git, เส้นช่วยจัดแนว, Start window
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
