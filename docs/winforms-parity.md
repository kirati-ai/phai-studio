# Phai Studio เทียบ WinForms: Properties · Methods · Events

รายการจากเอกสาร WinForms ของ Microsoft (System.Windows.Forms) เฉพาะที่คนใช้จริง
สถานะ: ✅ มีแล้ว · 🟡 มีบางส่วน · ⬜ ยังไม่มี · — ไม่ทำ (เหตุผลในวงเล็บ)
ชื่อในไฟล์ `.phaiform` อยู่ในวงเล็บ `code`; ตารางของ property อยู่ที่ `crates/phai-form/src/props.rs`

## ทุก control (Control)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Name | `id` | ✅ |
| P | Location, Size | `bounds` | ✅ |
| P | Anchor, Dock | `anchor`, `dock` | ✅ |
| P | Enabled, Visible | `enabled`, `visible` | ✅ |
| P | TabIndex, TabStop | `tab_index`, `tab_stop` (+ โหมด Tab Order ใน designer) | ✅ |
| P | BackColor, ForeColor, Font | `back_color`, `fore_color`, `font_size` + `bold` | ✅ |
| P | ContextMenuStrip | `context_menu` | ✅ |
| P | Cursor | `cursor` | ✅ |
| P | ToolTip (ของ ToolTip component) | `tooltip` | ✅ |
| P | ขอบ/มุมมน/สีเน้น (ไม่มีใน WinForms) | `border_color`, `border_width`, `radius`, `accent_color`, `icon_color` | ✅ |
| P | Tag | — (เก็บใน struct ของผู้ใช้แทน: Rust มีชนิดข้อมูลชัดกว่า object) | — |
| P | Padding, Margin | 🟡 `gap` ของ flow/table | 🟡 |
| P | RightToLeft | — (ไทย/อังกฤษไม่ต้องใช้) | — |
| M | Focus, Show, Hide, Refresh, BringToFront, SendToBack | ดูหัวข้อ Methods | ⬜ |
| E | Click, DoubleClick, Enter, Leave, KeyDown, KeyUp, KeyPress, MouseEnter, MouseLeave, MouseDown, MouseUp, MouseMove, Resize | `events = { key_down = "..." }` (`phai_ui::events`) | ✅ |

## Form

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text | `title` | ✅ |
| P | Size, StartPosition, WindowState | `size`, `start`, `maximized` | ✅ |
| P | AcceptButton, CancelButton | `accept_button`, `cancel_button` | ✅ |
| P | FormBorderStyle | `border` | ✅ |
| P | MaximizeBox, MinimizeBox, ControlBox | `maximize_box`, `minimize_box`, `control_box` | ✅ |
| P | MinimumSize, MaximumSize | `min_size`, `max_size` | ✅ |
| P | Icon | `icon` | ✅ |
| P | TopMost, ShowInTaskbar, Opacity | `top_most`, `show_in_taskbar`, `opacity` | ✅ |
| P | BackColor, ForeColor | `back_color`, `fore_color` (ทั้งโปรแกรม ไม่ใช่รายหน้าต่าง) | 🟡 |
| P | Font | `font_size` | ✅ |
| P | KeyPreview | `key_preview` | ✅ |
| P | BackgroundImage, AutoScroll, AllowDrop | ⬜ (ทำทีหลัง) | ⬜ |
| M | Show, ShowDialog (คืน DialogResult), Close | `show`, `show_dialog`, `close` | 🟡 |
| E | Load, Shown, FormClosing (e.cancel()), FormClosed, Resize, Activated, Deactivate | `loaded`, `events = { shown, closing, closed, resize, activated, deactivate }` | ✅ |
| E | Click, DoubleClick, MouseDown/Up/Move, KeyDown/Up/Press | `events = { click, ... }` | ✅ |

## Button

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, TextAlign | `text`, `text_align` | ✅ |
| P | DialogResult | `dialog_result` | ✅ |
| P | Image, FlatStyle | — (หน้าตามาจาก CSS) | — |
| M | PerformClick | | ⬜ |
| E | Click | `on_click` | ✅ |

## Label

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, TextAlign, BorderStyle | `text`, `text_align`, `border` | ✅ |
| P | AutoSize | ⬜ | ⬜ |
| E | Click, DoubleClick, MouseEnter/Leave | `events = { click = "..." }` | ✅ |

## TextBox (`edit`, `textarea`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, PasswordChar, Multiline | `text`, `password`, ชนิด `textarea` | ✅ |
| P | ReadOnly, MaxLength | `read_only`, `max_length` | ✅ |
| P | PlaceholderText, TextAlign, CharacterCasing | `placeholder`, `text_align`, `char_casing` | ✅ |
| P | ScrollBars, WordWrap (textarea) | ⬜ (ต้องสร้าง control ใหม่) | ⬜ |
| M | Clear, SelectAll, AppendText, Copy, Cut, Paste, Undo, ScrollToCaret | | ⬜ |
| E | TextChanged | `on_change` | ✅ |
| E | KeyDown, KeyPress, Enter, Leave | `events` | ✅ |

## RichTextBox (`richtext`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, ReadOnly, MaxLength, WordWrap, DetectUrls | `text`, `read_only`, `max_length`, `word_wrap`, `detect_urls` | ✅ |
| M | LoadFile, SaveFile, Find | | ⬜ |
| E | TextChanged | `on_change` | ✅ |

## CheckBox / RadioButton

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, Checked | `text`, `checked`; radio `group` | ✅ |
| P | ThreeState, Appearance | ⬜ | ⬜ |
| E | Click | `on_click` | ✅ |
| E | CheckedChanged | `events = { checked_changed = "..." }` | ✅ |

## ComboBox (`combo`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Items, SelectedIndex | `items`, `selected_index` | ✅ |
| P | DropDownStyle (DropDown/DropDownList), Sorted, MaxDropDownItems | `editable`, `sorted`, `max_drop_down_items` | ✅ |
| M | Items.Add, Items.Clear, FindString, BeginUpdate/EndUpdate | | ⬜ |
| E | SelectedIndexChanged | `on_change` | ✅ |
| E | TextChanged (พิมพ์ได้) | `events = { text_changed = "..." }` | ✅ |

## ListView (`list`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Columns, View, FullRowSelect, GridLines, MultiSelect, CheckBoxes, HeaderStyle, HideSelection | `columns`, `view`, `full_row_select`, `grid_lines`, `multi_select`, `check_boxes`, `header`, `hide_selection` | ✅ |
| M | Items.Add, Items.Clear, EnsureVisible | | ⬜ |
| E | SelectedIndexChanged | `on_select` | ✅ |
| E | DoubleClick, ColumnClick | `events = { double_click, column_click }` | ✅ |

## DataGridView (`grid`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Columns, DataSource | `columns`, `bind` | ✅ |
| P | ReadOnly, GridLines, MultiSelect | `read_only`, `grid_lines`, `multi_select` | ✅ |
| M | Rows.Add, Rows.Clear, Refresh | | ⬜ |
| E | CellValueChanged | `on_change` | ✅ |
| E | SelectionChanged, DoubleClick, ColumnClick | `events` | ✅ |

## DateTimePicker (`date`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Format, CustomFormat, MinDate, MaxDate | `date`, `format`, `custom_format`, `min_date`, `max_date` | ✅ |
| P | ShowCheckBox, ShowUpDown | ⬜ (ต้องสร้าง control ใหม่) | ⬜ |
| E | ValueChanged | `on_change` | ✅ |

## NumericUpDown (`number`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Minimum, Maximum | `value`, `min`, `max` | ✅ |
| P | Increment, Hexadecimal, ThousandsSeparator, ReadOnly, TextAlign | `increment`, `hexadecimal`, `thousands`, `read_only`, `text_align` | ✅ |
| P | DecimalPlaces | — (Windows' up-down นับเป็นจำนวนเต็ม) | — |
| M | UpButton, DownButton | | ⬜ |
| E | ValueChanged | `on_change` | ✅ |

## TrackBar (`slider`) / ProgressBar (`progress`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Minimum, Maximum | `value`, `min`, `max` | ✅ |
| P | SmallChange, LargeChange, TickFrequency | `small_change`, `large_change`, `tick_frequency` | ✅ |
| P | Orientation (แนวตั้ง) | ⬜ | ⬜ |
| P | Step, Style = Marquee | `step`, `marquee` | ✅ |
| M | PerformStep | `phai_ui::props::perform_step` | ✅ |
| E | ValueChanged / Scroll | `on_change` | ✅ |

## PictureBox (`picture`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Image, SizeMode, BorderStyle | `image`, `size_mode`, `border` | ✅ |
| M | Load | | ⬜ |
| E | Click, DoubleClick | `events` | ✅ |

## TreeView (`tree`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Nodes | `items` (เว้นหน้าบรรทัดเป็นชั้น) | ✅ |
| P | ShowLines, ShowPlusMinus, ShowRootLines, CheckBoxes, FullRowSelect, HideSelection | `show_lines`, `show_plus_minus`, `show_root_lines`, `check_boxes`, `full_row_select`, `hide_selection` | ✅ |
| P | LabelEdit | ⬜ | ⬜ |
| M | ExpandAll, CollapseAll | `expand_all` (property และ `phai_ui::props::expand_all`) | ✅ |
| E | AfterSelect | `on_select` | ✅ |
| E | DoubleClick | `events = { double_click = "..." }` | ✅ |

## TabControl (`tabs`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | TabPages, SelectedIndex | `tabs` + `page`, `selected_index` | ✅ |
| P | Alignment, Multiline | — (แท็บวาดเองตามธีม) | — |
| M | SelectTab | | ⬜ |
| E | SelectedIndexChanged | `on_change` | ✅ |

## Timer

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Interval, Enabled | `value`, `enabled` | ✅ |
| M | Start, Stop | | ⬜ |
| E | Tick | `on_tick` | ✅ |

## ContextMenuStrip (`contextmenu`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Items | `items` (`เมธอด \| ข้อความ`) | ✅ |
| E | ItemClicked (ต่อรายการ) | เมธอดของแต่ละรายการ | ✅ |
| E | Opening | | ⬜ |

## FlowLayoutPanel (`flow`) / TableLayoutPanel (`table`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | FlowDirection, WrapContents, Padding | `direction`, `wrap`, `gap` | ✅ |
| P | ColumnCount | `cols` | ✅ |
