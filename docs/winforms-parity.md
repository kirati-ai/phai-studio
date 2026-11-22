# Phai Studio เทียบ WinForms: Properties · Methods · Events

รายการจากเอกสาร WinForms ของ Microsoft (System.Windows.Forms) เฉพาะที่คนใช้จริง
สถานะ: ✅ มีแล้ว · 🟡 มีบางส่วน · ⬜ ยังไม่มี · — ไม่ทำ (เหตุผลในวงเล็บ)
ชื่อในไฟล์ `.phf` อยู่ในวงเล็บ `code`; ตารางของ property อยู่ที่ `crates/phai-form/src/props.rs`

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

# รอบสาม: ของที่ WinForms มีนอกจากรายการข้างบน (เจ้านายสั่ง 2026-11-19 "ทำ winform ให้จบ")

| | WinForms | Phai | สถานะ |
|---|---|---|---|
| C | Panel | `panel`: `back_color`, `border` (`fixed`), ใส่ control ข้างใน (designer ถือเป็น container) | ✅ |
| C | ListBox | `listbox`: `items`, `sorted`, `selection_mode` (`one`/`multi`); `items_add/insert/remove_at/clear/count`, `item`, `selected_index`, `selected_indices`, `selected_item`; `on_select` | ✅ |
| C | CheckedListBox | `checkedlist`: `items`; `checked(i)`, `set_checked`, `checked_indices`; `on_change` = ItemCheck | ✅ |
| C | LinkLabel | `link`: `text` (ทั้งข้อความ หรือส่วนใน `<a>`); `link_text`, `set_link_text`; `on_click` = LinkClicked | ✅ |
| C | MaskedTextBox | `masked`: `mask` (`0 9 # L ? A a & C`, `\` literal); `value()` (ไม่รวมตัวคั่น), `mask_completed()`; `on_change` | ✅ |
| C | MonthCalendar | `calendar`: `selection_start()`, `set_date()`; `on_change` = DateChanged; สีจาก `calendar { }` ในธีม | ✅ |
| C | SplitContainer | `split`: `orientation` (`vertical`/`horizontal`), `splitter_distance`; control ข้างในยึด anchor กับช่องของตัวเอง; ลากเส้นแบ่งได้; `splitter_distance()`, `set_splitter_distance()`; `on_change` = SplitterMoved | ✅ |
| C | NotifyIcon | `notifyicon`: `text`, `visible`, `context_menu`, `on_click`, `double_click`; `set_text`, `set_visible`, `show_balloon_tip`, `remove` (ทำเองตอนปิดหน้าต่าง) | ✅ |
| C | OpenFileDialog / SaveFileDialog / FolderBrowserDialog / ColorDialog / FontDialog (component) | `openfiledialog` (`title`, `filter`, `initial_directory`, `default_ext`, `multiselect`) `show_dialog()` / `show_dialog_multi()`; `savefiledialog` `show_dialog_named()`; `folderbrowserdialog`; `colordialog` `show_dialog("#hex")`; `fontdialog` → `ChosenFont` | ✅ |
| C | ErrorProvider | `errorprovider`: `set_error(control, "...")` (เครื่องหมายแดงข้าง control พร้อม tooltip ตามไปเมื่อ control ขยับ), `error()`, `clear()` | ✅ |
| C | ToolTip (component) | — (เป็น property `tooltip` ของทุก control แล้ว) | — |
| F | UserControl | Project > เพิ่ม User Control: ฟอร์มที่ `user_control = true` (ออกแบบเหมือนฟอร์ม มีโค้ด `.rs` ของตัวเอง); วางบนฟอร์มอื่นจากกล่องเครื่องมือ = `kind = "usercontrol"`, `control_type = "AddressBox"` — designer วาด control ข้างในให้เห็น; anchor/dock ได้; โค้ดฟอร์มเข้าถึงข้างในด้วย `AddressBoxUi::with(ui.home, \|a\| a.street.text())`; user control บอกฟอร์มด้วย `ui.notify_parent()` → `on_change` ของมันบนฟอร์ม | ✅ |
| F | MDI (IsMdiContainer, MdiParent) | ฟอร์ม `is_mdi_container = true`; `XxxUi::show_mdi_child(parent, events)`; `layout_mdi("cascade" / "tile_horizontal" / "tile_vertical" / "arrange_icons")`, `active_mdi_child()`, `mdi_children()`; Ctrl+F4, Ctrl+Tab; ฟอร์มลูกไม่มีเมนูของตัวเอง (Windows ไม่ให้) | ✅ |
| F | PrintDocument / PrintDialog / PageSetupDialog / PrintPreviewDialog | component `printdocument` (`document_name`, `landscape`); event `print_page` ได้ `e: &PrintPageEventArgs` — วาดด้วย `draw_text`, `draw_text_in` (ตัดบรรทัด, ชิดซ้าย/กลาง/ขวา), `measure_text`, `line_height`, `draw_line`, `draw_rect`, `fill_rect`, `draw_image`, `set_font`, `set_color`, `set_pen_width` หน่วยร้อยส่วนของนิ้วจากมุมกระดาษ, `e.margin_bounds`, `e.set_has_more_pages(true)`; `print()` (ไม่ถาม), `print_dialog()`, `page_setup()`, `print_preview()` (หน้าต่างดูตัวอย่าง พลิกหน้า ปุ่มพิมพ์), `print_to_file("Microsoft Print to PDF", path)`, `page_count()`, `set_margins`, `set_landscape`, `set_printer_name`, `phai_ui::print::printers()` | ✅ |
| F | BindingSource + DataBindings (ผูก property ของ control กับข้อมูล) | component `bindingsource`: `fields = "name, phone"` หรือ `bind = { table, columns, keys }` (โหลดจากฐานข้อมูลตอนเปิด, `reload_x()`, `save_x()`); control ใดก็ได้ `data_binding = "people.name"` — กล่องข้อความ→ข้อความ, checkbox/radio→ติ๊ก (`1`/`0` หรือ `true`/`false` ตามที่คอลัมน์เก็บ), number→ตัวเลข, slider/progress→ค่า, date/calendar→`YYYY-MM-DD`, combo/listbox→รายการที่เลือก; สองทาง (ค่าที่แก้กลับเข้าแถวเมื่อเลื่อน/อ่าน/บันทึก); `set_rows`, `rows`, `count`, `position`, `set_position`, `move_first/previous/next/last`, `current`, `value("col")`, `set_value`, `add_new`, `remove_current`, `end_edit`, `reset_bindings`, `bind`/`unbind` จากโค้ด; event `position_changed` | ✅ |
| F | DoDragDrop ระหว่าง control | `ui.x.do_drag_drop("ข้อความ")` จาก MouseDown (ป้ายข้อความตามเมาส์, Esc ยกเลิก); control ที่มี event `drag_drop` รับได้: `e.text`, `e.source` (ไฟล์จาก Explorer ยังเป็น `e.files`); ข้อความเท่านั้น ภายในโปรแกรมเดียวกัน | ✅ |
| F | MenuStrip / ToolStrip แบบละเอียด (ไอคอน, ปุ่มกดค้าง, รายการย่อยซ้อน) | เมนูย่อยซ้อนได้หลายชั้น, `checked`, `check_on_click`, `disabled`, ไอคอน (`GLYPH`); ปุ่ม toolbar `toggle` ติ๊กพร้อมเมนู id เดียวกัน; `ui.item_checked("x")`, `set_item_checked`, `set_item_enabled` | ✅ |

C = control/component, F = ความสามารถ
