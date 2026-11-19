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
| P | Padding | ฟอร์ม `padding`, flow/table `gap` | ✅ |
| P | Margin | — (ระยะห่างของ flow/table คือ `gap` ตัวเดียว) | — |
| P | AllowDrop + DragDrop (ไฟล์จาก Explorer) | `allow_drop`, `events = { drag_drop = "..." }` (`e.files`) | ✅ |
| P | RightToLeft | — (ไทย/อังกฤษไม่ต้องใช้) | — |
| M | Focus, Show, Hide, Refresh, Invalidate, BringToFront, SendToBack, PointToScreen, PointToClient, FindForm, SuspendLayout, ResumeLayout, Handle | `ui.x.focus()` ... `ui.x.hwnd()` (`phai_ui::ctl`) | ✅ |
| M | Invoke, BeginInvoke (งานจาก thread อื่น) | `ui.invoker()`, `MainWindowUi::begin_invoke(inv, \|me, ui\| ...)`, `invoke` | ✅ |
| E | Click, DoubleClick, Enter, Leave, KeyDown, KeyUp, KeyPress, MouseEnter, MouseLeave, MouseDown, MouseUp, MouseMove, Resize | `events = { key_down = "..." }` (`phai_ui::events`) | ✅ |

## Form

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text | `title` (`ui.title()`, `ui.set_title()`) | ✅ |
| P | Size, StartPosition, WindowState | `size`, `start`, `maximized` | ✅ |
| P | AcceptButton, CancelButton | `accept_button`, `cancel_button` | ✅ |
| P | FormBorderStyle | `border` | ✅ |
| P | MaximizeBox, MinimizeBox, ControlBox | `maximize_box`, `minimize_box`, `control_box` | ✅ |
| P | MinimumSize, MaximumSize | `min_size`, `max_size` | ✅ |
| P | Icon | `icon` | ✅ |
| P | TopMost, ShowInTaskbar, Opacity | `top_most`, `show_in_taskbar`, `opacity` | ✅ |
| P | BackColor, ForeColor | `back_color`, `fore_color` (ของหน้าต่างนี้, control วางบนสีนี้) | ✅ |
| P | Font | `font_size` | ✅ |
| P | KeyPreview | `key_preview` | ✅ |
| P | BackgroundImage, BackgroundImageLayout | `background_image`, `background_image_layout` (label, group, ปุ่มโปร่งใสให้เห็นรูป) | ✅ |
| P | AutoScroll, Padding, AllowDrop | `auto_scroll`, `padding`, `allow_drop` | ✅ |
| E | DragDrop | `events = { drag_drop = "..." }` | ✅ |
| M | Show, ShowDialog (คืน DialogResult), Close, Hide, Activate, CenterToScreen | `show`, `show_dialog`, `close`, `hide`, `show_window`, `activate`, `center_to_screen` | ✅ |
| E | Load, Shown, FormClosing (e.cancel()), FormClosed, Resize, Activated, Deactivate | `loaded`, `events = { shown, closing, closed, resize, activated, deactivate }` | ✅ |
| E | Click, DoubleClick, MouseDown/Up/Move, KeyDown/Up/Press | `events = { click, ... }` | ✅ |

## Button

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, TextAlign | `text`, `text_align` | ✅ |
| P | DialogResult | `dialog_result` | ✅ |
| P | Image, FlatStyle | — (หน้าตามาจาก CSS) | — |
| M | PerformClick | `perform_click()` | ✅ |
| E | Click | `on_click` | ✅ |

## Label

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, TextAlign, BorderStyle | `text`, `text_align`, `border` | ✅ |
| P | AutoSize | `auto_size` (designer เขียนความกว้างที่ได้กลับลงฟอร์ม) | ✅ |
| E | Click, DoubleClick, MouseEnter/Leave | `events = { click = "..." }` | ✅ |

## TextBox (`edit`, `textarea`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, PasswordChar, Multiline | `text`, `password`, ชนิด `textarea` | ✅ |
| P | ReadOnly, MaxLength | `read_only`, `max_length` | ✅ |
| P | PlaceholderText, TextAlign, CharacterCasing | `placeholder`, `text_align`, `char_casing` | ✅ |
| P | ScrollBars, WordWrap (textarea) | `scroll_bars`, `word_wrap` | ✅ |
| M | Clear, SelectAll, Select, SelectedText, AppendText, Copy, Cut, Paste, Undo, CanUndo, ScrollToCaret, Lines | `clear()`, `select_all()`, `select()`, `selected_text()`, `append_text()`, `copy()`, `cut()`, `paste()`, `undo()`, `can_undo()`, `scroll_to_caret()`, `lines()` | ✅ |
| E | TextChanged | `on_change` | ✅ |
| E | KeyDown, KeyPress, Enter, Leave | `events` | ✅ |

## RichTextBox (`richtext`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, ReadOnly, MaxLength, WordWrap, DetectUrls | `text`, `read_only`, `max_length`, `word_wrap`, `detect_urls` | ✅ |
| M | LoadFile, SaveFile, Find (+ ทุกตัวของ TextBox) | `load_file()`, `save_file()`, `find()` | ✅ |
| E | TextChanged | `on_change` | ✅ |

## CheckBox / RadioButton

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Text, Checked | `text`, `checked`; radio `group` | ✅ |
| P | ThreeState, CheckState, Appearance | `three_state`, `indeterminate`, `appearance`; `check_state()` | ✅ |
| E | Click | `on_click` | ✅ |
| E | CheckedChanged | `events = { checked_changed = "..." }` | ✅ |

## ComboBox (`combo`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Items, SelectedIndex | `items`, `selected_index` | ✅ |
| P | DropDownStyle (DropDown/DropDownList), Sorted, MaxDropDownItems | `editable`, `sorted`, `max_drop_down_items` | ✅ |
| M | Items.Add/Insert/RemoveAt/Clear/Count, SelectedIndex, SelectedItem, FindString, FindStringExact, DroppedDown, BeginUpdate/EndUpdate | `items_add()` ... `begin_update()` | ✅ |
| E | SelectedIndexChanged | `on_change` | ✅ |
| E | TextChanged (พิมพ์ได้) | `events = { text_changed = "..." }` | ✅ |

## ListView (`list`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Columns, View, FullRowSelect, GridLines, MultiSelect, CheckBoxes, HeaderStyle, HideSelection | `columns`, `view`, `full_row_select`, `grid_lines`, `multi_select`, `check_boxes`, `header`, `hide_selection` | ✅ |
| M | Items.Add/RemoveAt/Clear/Count, SelectedIndices, EnsureVisible, Checked, BeginUpdate/EndUpdate | `items_add()`, `cell()`, `set_cell()`, `selected_indices()`, `ensure_visible()`, `checked()` | ✅ |
| E | SelectedIndexChanged | `on_select` | ✅ |
| E | DoubleClick, ColumnClick | `events = { double_click, column_click }` | ✅ |

## DataGridView (`grid`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Columns, DataSource | `columns`, `bind` | ✅ |
| P | ReadOnly, GridLines, MultiSelect | `read_only`, `grid_lines`, `multi_select` | ✅ |
| M | Rows.Add, Rows.Clear, Rows.Count, ClearSelection, Refresh, ค่าในช่อง | `rows_add()`, `rows_clear()`, `rows_count()`, `clear_selection()`, `refresh()`, `cell()`, `set_cell()`, `fill()` | ✅ |
| E | CellValueChanged | `on_change` | ✅ |
| E | SelectionChanged, DoubleClick, ColumnClick | `events` | ✅ |

## DateTimePicker (`date`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Format, CustomFormat, MinDate, MaxDate | `date`, `format`, `custom_format`, `min_date`, `max_date` | ✅ |
| P | ShowCheckBox, ShowUpDown | `show_check_box`, `show_up_down`; `checked()` | ✅ |
| E | ValueChanged | `on_change` | ✅ |

## NumericUpDown (`number`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Minimum, Maximum | `value`, `min`, `max` | ✅ |
| P | Increment, Hexadecimal, ThousandsSeparator, ReadOnly, TextAlign | `increment`, `hexadecimal`, `thousands`, `read_only`, `text_align` | ✅ |
| P | DecimalPlaces | — (Windows' up-down นับเป็นจำนวนเต็ม) | — |
| M | UpButton, DownButton | `up_button()`, `down_button()` | ✅ |
| E | ValueChanged | `on_change` | ✅ |

## TrackBar (`slider`) / ProgressBar (`progress`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Value, Minimum, Maximum | `value`, `min`, `max` | ✅ |
| P | SmallChange, LargeChange, TickFrequency | `small_change`, `large_change`, `tick_frequency` | ✅ |
| P | Orientation | `orientation` | ✅ |
| P | Step, Style = Marquee | `step`, `marquee` | ✅ |
| M | PerformStep, Increment | `perform_step()`, `increment()` | ✅ |
| E | ValueChanged / Scroll | `on_change` | ✅ |

## PictureBox (`picture`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Image, SizeMode, BorderStyle | `image`, `size_mode`, `border` | ✅ |
| M | Load | `load()` | ✅ |
| E | Click, DoubleClick | `events` | ✅ |

## TreeView (`tree`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Nodes | `items` (เว้นหน้าบรรทัดเป็นชั้น) | ✅ |
| P | ShowLines, ShowPlusMinus, ShowRootLines, CheckBoxes, FullRowSelect, HideSelection | `show_lines`, `show_plus_minus`, `show_root_lines`, `check_boxes`, `full_row_select`, `hide_selection` | ✅ |
| P | LabelEdit | `label_edit` | ✅ |
| M | ExpandAll, CollapseAll, Nodes.Add, Nodes.Clear, SelectedNode, Expand, Collapse | `expand_all()`, `collapse_all()`, `nodes_add()`, `nodes_add_under()`, `nodes_clear()`, `selected_node()`, `selected_text()` | ✅ |
| E | AfterSelect | `on_select` | ✅ |
| E | DoubleClick | `events = { double_click = "..." }` | ✅ |

## TabControl (`tabs`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | TabPages, SelectedIndex | `tabs` + `page`, `selected_index` | ✅ |
| P | Alignment, Multiline | — (แท็บวาดเองตามธีม) | — |
| M | SelectTab, SelectedIndex | `select_tab()`, `selected_index()` | ✅ |
| E | SelectedIndexChanged | `on_change` | ✅ |

## Timer

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Interval, Enabled | `value`, `enabled` | ✅ |
| M | Start, Stop, Interval, Enabled | `start()`, `stop()`, `set_interval()`, `enabled()` | ✅ |
| E | Tick | `on_tick` | ✅ |

## ContextMenuStrip (`contextmenu`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | Items | `items` (`เมธอด \| ข้อความ`) | ✅ |
| E | ItemClicked (ต่อรายการ) | เมธอดของแต่ละรายการ | ✅ |
| E | Opening | — (เมนูสร้างตอนคลิกขวา ไม่มีขั้นก่อนเปิด) | — |

## FlowLayoutPanel (`flow`) / TableLayoutPanel (`table`)

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| P | FlowDirection, WrapContents, Padding | `direction`, `wrap`, `gap` | ✅ |
| P | ColumnCount | `cols` | ✅ |
