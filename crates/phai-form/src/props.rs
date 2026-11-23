//! The properties a form and its controls have beyond the few the model names itself - WinForms'
//! ReadOnly, MaxLength, AcceptButton, FormBorderStyle and the rest - in one table. The file keeps
//! them as plain `name = value` lines; the Properties pane shows a row for each that a control's
//! kind has, in its category, with its default when it is not set; the checker knows which
//! belong where; the generated code and the designer's canvas call the same `phai_ui::props`
//! function for each, so what is designed is what runs.

use crate::model::Kind;

/// What a value is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ty {
    Bool,
    /// A whole number in this range.
    Int(i32, i32),
    /// Words as they are: a file, a format.
    Text,
    /// A sentence the program shows: translated, in the Thai table.
    Words,
    /// One of these.
    Choice(&'static [&'static str]),
    /// A colour as CSS writes it.
    Color,
    /// A button of this form, by its id.
    Button,
    /// Two numbers, "w, h" - kept as `[w, h]`.
    Size,
    /// A day, `2026-11-19`.
    Date,
}

/// Where a value goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum How {
    /// `phai_ui::props::<name>(control, value)` after the control is made.
    Call,
    /// A declaration of the control's own CSS, `{}` the value.
    Css(&'static str),
    /// The generator itself does what it says (a constructor's argument, a window style).
    Code,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    /// Its name in the file and in `phai_ui::props`.
    pub name: &'static str,
    /// Its name in the Properties pane, as WinForms calls it.
    pub label: &'static str,
    pub category: &'static str,
    pub ty: Ty,
    /// What it is when not set, as the pane shows it; empty: nothing.
    pub default: &'static str,
    pub help: &'static str,
    /// The kinds that have it; empty: every control on the form (not components).
    pub kinds: &'static [Kind],
    pub how: How,
    /// The designer's canvas shows it too (a tooltip or a cursor would only get in the way).
    pub canvas: bool,
}

/// The controls `data_binding` can bind (`codegen::bound_as`).
const BINDABLE: &[Kind] = &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Label, Kind::Button, Kind::Masked, Kind::Link, Kind::Group, Kind::Checkbox, Kind::Radio, Kind::Number, Kind::Slider, Kind::Progress, Kind::Date, Kind::Calendar, Kind::Combo, Kind::ListBox];

#[allow(clippy::too_many_arguments)]
const fn spec(name: &'static str, label: &'static str, category: &'static str, ty: Ty, default: &'static str, kinds: &'static [Kind], how: How, help: &'static str) -> Spec {
    Spec { name, label, category, ty, default, help, kinds, how, canvas: true }
}

const fn runtime_only(mut s: Spec) -> Spec {
    s.canvas = false;
    s
}

const ALIGN: &[&str] = &["left", "center", "right"];
const EDIT_ON: &[&str] = &["double_click", "click", "f2"];
const FIND: &[&str] = &["ctrl_f", "always", "off"];
const FOCUSABLE: &[Kind] = &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Combo, Kind::List, Kind::Grid, Kind::Tree, Kind::Date, Kind::Number, Kind::Slider, Kind::Tabs, Kind::ListBox, Kind::CheckedList, Kind::Link, Kind::Masked, Kind::Calendar];
const TEXTS: &[Kind] = &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Number];
const CURSORS: &[&str] = &["default", "hand", "ibeam", "wait", "cross", "help", "no", "size_all", "size_we", "size_ns"];

pub const CONTROL: &[Spec] = &[
    // Every control.
    spec("tab_index", "TabIndex", "Behavior", Ty::Int(0, 999), "", FOCUSABLE, How::Code, "Its place in the order Tab moves through; unset, the form's order."),
    spec("tab_stop", "TabStop", "Behavior", Ty::Bool, "true", FOCUSABLE, How::Call, "Whether Tab stops on it."),
    runtime_only(spec("tooltip", "ToolTip", "Appearance", Ty::Words, "", &[], How::Call, "Words shown when the mouse rests on it.")),
    runtime_only(spec("cursor", "Cursor", "Appearance", Ty::Choice(CURSORS), "default", &[], How::Call, "The mouse pointer over it.")),
    spec("border_width", "Border width", "Appearance", Ty::Int(0, 8), "", &[], How::Css("border-width: {}px"), "Its border's thickness, pixels - #id { border-width }."),
    spec("accent_color", "Accent colour", "Appearance", Ty::Color, "", &[Kind::Checkbox, Kind::Radio, Kind::Tabs, Kind::Progress, Kind::Slider, Kind::Button], How::Css("accent-color: {}"), "The tick, the dot, the bar or the chosen tab's line - #id { accent-color }."),
    spec("animation", "Animation", "Appearance", Ty::Text, "", &[], How::Css("animation: {}"), "Its colours move on their own: blink 1s infinite, pulse 1.5s infinite, fade-in 400ms, or a @keyframes of the theme - #id { animation }."),
    spec("font_family", "FontFamily", "Appearance", Ty::Text, "", &[], How::Css("font-family: {}"), "Its words' typeface - Consolas for a log, Segoe UI is Windows' own."),
    spec("transition", "Transition", "Appearance", Ty::Text, "", &[], How::Css("transition: {}"), "Its colours change over time when its state does: 150ms, 0.3s ease-out - #id { transition }."),
    spec("gradient", "Background gradient", "Appearance", Ty::Text, "", &[Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Statusbar], How::Css("background-image: {}"), "Colours that blend across it: linear-gradient(to bottom, #ffffff, #dde3ea) - #id { background-image }."),
    spec("shadow", "Shadow", "Appearance", Ty::Text, "", &[Kind::Button, Kind::Checkbox, Kind::Radio], How::Css("box-shadow: {}"), "A soft shadow: 0 2px 6px rgba(0, 0, 0, 0.25) - across, down, blur, colour - #id { box-shadow }."),
    spec("icon_color", "Icon colour", "Appearance", Ty::Color, "", &[Kind::Tree], How::Css("icon-color: {}"), "Its icons' colour - #id { icon-color }."),
    // Words.
    spec("text_align", "TextAlign", "Appearance", Ty::Choice(ALIGN), "center", &[Kind::Button], How::Call, "Where its words sit."),
    spec("text_align", "TextAlign", "Appearance", Ty::Choice(ALIGN), "left", &[Kind::Label, Kind::Edit, Kind::Number], How::Call, "Where its words sit."),
    spec("border", "BorderStyle", "Appearance", Ty::Choice(&["none", "fixed"]), "none", &[Kind::Label, Kind::Picture, Kind::Panel], How::Call, "A thin line round it, or none."),
    spec("dialog_result", "DialogResult", "Behavior", Ty::Choice(&["none", "ok", "cancel", "yes", "no", "retry", "abort", "ignore"]), "none", &[Kind::Button], How::Code, "Clicked, it closes the window, and show_dialog returns this."),
    spec("read_only", "ReadOnly", "Behavior", Ty::Bool, "false", &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Number, Kind::Grid], How::Call, "Its words can be read and copied, not changed."),
    spec("max_length", "MaxLength", "Behavior", Ty::Int(0, 1_000_000), "", TEXTS, How::Call, "The most letters it takes; unset, no limit but Windows' own."),
    spec("placeholder", "PlaceholderText", "Appearance", Ty::Words, "", &[Kind::Edit], How::Call, "Grey words shown while it is empty."),
    spec("char_casing", "CharacterCasing", "Behavior", Ty::Choice(&["normal", "upper", "lower"]), "normal", &[Kind::Edit, Kind::Textarea], How::Call, "What is typed in, made capitals or small letters."),
    spec("word_wrap", "WordWrap", "Behavior", Ty::Bool, "true", &[Kind::Rich], How::Call, "Long lines wrap at the edge; off, it scrolls sideways."),
    spec("detect_urls", "DetectUrls", "Behavior", Ty::Bool, "true", &[Kind::Rich], How::Call, "Web addresses show as links."),
    spec("auto_size", "AutoSize", "Layout", Ty::Bool, "false", &[Kind::Label], How::Call, "As wide as its words, whatever they are."),
    spec("three_state", "ThreeState", "Behavior", Ty::Bool, "false", &[Kind::Checkbox], How::Call, "A click goes unticked, ticked, then neither."),
    spec("indeterminate", "CheckState = Indeterminate", "Behavior", Ty::Bool, "false", &[Kind::Checkbox], How::Call, "Neither ticked nor unticked when the window opens."),
    spec("appearance", "Appearance", "Appearance", Ty::Choice(&["normal", "button"]), "normal", &[Kind::Checkbox, Kind::Radio], How::Call, "A box to tick, or a button that stays pressed while ticked."),
    spec("word_wrap", "WordWrap", "Behavior", Ty::Bool, "true", &[Kind::Textarea], How::Code, "Long lines wrap at the edge; off, it scrolls sideways."),
    spec("scroll_bars", "ScrollBars", "Appearance", Ty::Choice(&["none", "vertical", "horizontal", "both"]), "vertical", &[Kind::Textarea], How::Code, "Which scroll bars it has."),
    runtime_only(spec("allow_drop", "AllowDrop", "Behavior", Ty::Bool, "false", &[], How::Call, "Files dragged from Explorer can be dropped on it - its DragDrop event.")),
    // Numbers and ranges.
    spec("increment", "Increment", "Behavior", Ty::Int(1, 1_000_000), "1", &[Kind::Number], How::Call, "How much an arrow adds or takes away."),
    spec("hexadecimal", "Hexadecimal", "Appearance", Ty::Bool, "false", &[Kind::Number], How::Call, "Shows the number in base 16."),
    spec("thousands", "ThousandsSeparator", "Appearance", Ty::Bool, "false", &[Kind::Number], How::Call, "Commas between thousands: 1,000."),
    spec("small_change", "SmallChange", "Behavior", Ty::Int(1, 1_000_000), "1", &[Kind::Slider], How::Call, "How far an arrow key moves it."),
    spec("large_change", "LargeChange", "Behavior", Ty::Int(1, 1_000_000), "", &[Kind::Slider], How::Call, "How far Page Up and Page Down move it; unset, a tenth of the range."),
    spec("tick_frequency", "TickFrequency", "Appearance", Ty::Int(0, 1_000_000), "", &[Kind::Slider], How::Call, "A tick mark every so many; unset, none."),
    spec("orientation", "Orientation", "Layout", Ty::Choice(&["vertical", "horizontal"]), "vertical", &[Kind::Split], How::Code, "Vertical: the panes side by side, the bar standing up. Horizontal: one over the other."),
    spec("splitter_distance", "SplitterDistance", "Layout", Ty::Int(0, 10_000), "0", &[Kind::Split], How::Code, "The first pane's width (or height); 0 is half."),
    spec("orientation", "Orientation", "Appearance", Ty::Choice(&["horizontal", "vertical"]), "horizontal", &[Kind::Slider], How::Call, "Lying down, or standing up with its least at the bottom."),
    spec("step", "Step", "Behavior", Ty::Int(1, 1_000_000), "10", &[Kind::Progress], How::Call, "How much perform_step adds."),
    spec("marquee", "Style = Marquee", "Appearance", Ty::Bool, "false", &[Kind::Progress], How::Call, "A block that runs along, for work of unknown length."),
    // Lists.
    spec("editable", "DropDownStyle = DropDown", "Behavior", Ty::Bool, "false", &[Kind::Combo], How::Code, "Words can be typed in as well as chosen."),
    spec("sorted", "Sorted", "Behavior", Ty::Bool, "false", &[Kind::Combo, Kind::ListBox], How::Code, "Its items in alphabetical order."),
    spec("selection_mode", "SelectionMode", "Behavior", Ty::Choice(&["one", "multi"]), "one", &[Kind::ListBox], How::Code, "One line chosen at a time, or several with Ctrl and Shift."),
    spec("control_type", "Type", "Design", Ty::Text, "", &[Kind::UserControl], How::Code, "The user control's name - a form of this project with UserControl on: AddressBox."),
    spec("fields", "DataMember", "Data", Ty::Text, "", &[Kind::BindingSource], How::Code, "Its rows' columns, with commas: name, phone - its table's when it is bound to one."),
    spec("data_binding", "DataBindings", "Data", Ty::Text, "", BINDABLE, How::Code, "A binding source's id and a column - people.name: the control shows that column of the current row, and what is changed goes back."),
    spec("document_name", "DocumentName", "Behavior", Ty::Words, "document", &[Kind::PrintDocument], How::Code, "The name in the printer's queue and on the preview's title bar."),
    spec("landscape", "Landscape", "Behavior", Ty::Bool, "false", &[Kind::PrintDocument], How::Code, "The paper turned sideways - DefaultPageSettings.Landscape."),
    spec("title", "Title", "Appearance", Ty::Words, "", &[Kind::OpenFile, Kind::SaveFile, Kind::FolderDialog], How::Code, "The words in the dialog's title bar."),
    spec("filter", "Filter", "Behavior", Ty::Words, "", &[Kind::OpenFile, Kind::SaveFile], How::Code, "The kinds of file it shows: Text files|*.txt|All files|*.*"),
    spec("initial_directory", "InitialDirectory", "Behavior", Ty::Text, "", &[Kind::OpenFile, Kind::SaveFile, Kind::FolderDialog], How::Code, "The folder it opens in; empty: the one used last."),
    spec("default_ext", "DefaultExt", "Behavior", Ty::Text, "", &[Kind::OpenFile, Kind::SaveFile], How::Code, "Added to a name typed without one: txt."),
    spec("multiselect", "Multiselect", "Behavior", Ty::Bool, "false", &[Kind::OpenFile], How::Code, "Several files can be chosen: show_dialog_multi()."),
    spec("mask", "Mask", "Behavior", Ty::Text, "", &[Kind::Masked], How::Code, "What it takes: 0 a digit, 9 a digit or none, L a letter, A a letter or digit, & anything; the rest stays as written - 000-000-0000."),
    spec("selected_index", "SelectedIndex", "Behavior", Ty::Int(-1, 9999), "0", &[Kind::Combo], How::Call, "The item chosen when the window opens, from 0; -1 none."),
    spec("max_drop_down_items", "MaxDropDownItems", "Behavior", Ty::Int(1, 100), "8", &[Kind::Combo], How::Call, "How many items the open list shows before it scrolls."),
    spec("view", "View", "Appearance", Ty::Choice(&["details", "list", "small_icons", "large_icons"]), "details", &[Kind::List], How::Call, "Rows with columns, or the items alone."),
    spec("icons", "Icons", "Appearance", Ty::Text, "", &[Kind::List], How::Call, "The pictures its rows show (LargeImageList, SmallImageList): glyph names, each with a colour - MONITOR #12855f, MONITOR #8a9a90. A row's is chosen in code by its place: items_add_with_icon."),
    spec("column_align", "ColumnAlign", "Appearance", Ty::Text, "", &[Kind::List], How::Call, "Each column's words left, center or right, in order - left, right, left for a size on the right (ColumnHeader.TextAlign)."),
    spec("icon_spacing", "IconSpacing", "Appearance", Ty::Size, "", &[Kind::List], How::Call, "The room each item takes in the icons views, wide enough for a long name under its icon - 120, 76."),
    spec("grid_lines", "GridLines", "Appearance", Ty::Bool, "false", &[Kind::List], How::Call, "Lines between rows and columns."),
    spec("grid_lines", "GridLines", "Appearance", Ty::Bool, "true", &[Kind::Grid], How::Call, "Lines between rows and columns."),
    spec("full_row_select", "FullRowSelect", "Behavior", Ty::Bool, "true", &[Kind::List, Kind::Tree], How::Call, "A click chooses the whole row."),
    spec("multi_select", "MultiSelect", "Behavior", Ty::Bool, "true", &[Kind::List, Kind::Grid], How::Call, "Several rows can be chosen at once."),
    // A grid's own, as DevExpress's GridView has them (OptionsView, OptionsBehavior).
    spec("show_column_headers", "ShowColumnHeaders", "Appearance", Ty::Bool, "true", &[Kind::Grid], How::Call, "The row of headings over the columns."),
    spec("column_auto_width", "ColumnAutoWidth", "Layout", Ty::Bool, "false", &[Kind::Grid], How::Call, "The columns share the grid's width - no scrolling sideways."),
    spec("allow_add_rows", "AllowAddRows", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "A double-click under the last row adds one."),
    spec("allow_delete_rows", "AllowDeleteRows", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "Delete takes the chosen row out."),
    spec("editor_show_mode", "EditorShowMode", "Behavior", Ty::Choice(EDIT_ON), "double_click", &[Kind::Grid], How::Call, "What opens a cell for typing: a double-click, a click, or F2 only."),
    spec("enter_moves_next", "EnterMoveNextColumn", "Behavior", Ty::Bool, "false", &[Kind::Grid], How::Call, "Enter keeps the cell and opens the next one."),
    spec("alternate_row_color", "AlternateRowColor", "Appearance", Ty::Color, "", &[Kind::Grid], How::Call, "Every other row's background, for rows easier to follow."),
    spec("header_back_color", "HeaderBackColor", "Appearance", Ty::Color, "", &[Kind::Grid], How::Call, "The headings' background (and the footer's)."),
    spec("header_fore_color", "HeaderForeColor", "Appearance", Ty::Color, "", &[Kind::Grid], How::Call, "The headings' words (and the footer's)."),
    spec("row_height", "RowHeight", "Appearance", Ty::Int(0, 80), "0", &[Kind::Grid], How::Call, "The rows this tall, pixels; 0 - as the font makes them."),
    spec("show_footer", "ShowFooter", "Appearance", Ty::Bool, "false", &[Kind::Grid], How::Call, "A row under the rows with each column's total (a column's Summary)."),
    spec("find_panel", "FindPanel", "Behavior", Ty::Choice(FIND), "ctrl_f", &[Kind::Grid], How::Call, "The find box: Ctrl+F opens it, always there, or none."),
    spec("find_filter", "FindFilter", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "The find box leaves only the rows it finds; off - it marks them."),
    spec("allow_filter", "AllowFilter", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "A drop-down on each heading filters the rows by that column's values, as Excel's."),
    spec("master_detail", "MasterDetail", "Data", Ty::Text, "", &[Kind::Grid], How::Code, "This grid shows the rows of another grid's chosen row: orders.id = order_id - the master grid, its key column, then this grid's column that holds it (the same name when left out)."),
    spec("header_menu", "HeaderMenu", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "A right-click on a heading offers sorting, filters, grouping and exporting to Excel or PDF."),
    spec("copy_headers", "CopyColumnHeaders", "Behavior", Ty::Bool, "true", &[Kind::Grid], How::Call, "Ctrl+C copies the headings with the rows."),
    spec("allow_column_moving", "AllowColumnMoving", "Behavior", Ty::Bool, "false", &[Kind::Grid], How::Call, "Columns can be dragged by their headings into another order."),
    spec("check_boxes", "CheckBoxes", "Appearance", Ty::Bool, "false", &[Kind::List, Kind::Tree], How::Call, "A box to tick before each item."),
    spec("header", "HeaderStyle", "Appearance", Ty::Choice(&["clickable", "none"]), "clickable", &[Kind::List], How::Call, "The column headings shown, or none."),
    spec("hide_selection", "HideSelection", "Behavior", Ty::Bool, "false", &[Kind::List, Kind::Tree], How::Call, "The chosen row stops showing when it loses the focus."),
    spec("show_lines", "ShowLines", "Appearance", Ty::Bool, "false", &[Kind::Tree], How::Call, "Dotted lines join items to their parents."),
    spec("show_plus_minus", "ShowPlusMinus", "Appearance", Ty::Bool, "true", &[Kind::Tree], How::Call, "A button to open and close items that hold others."),
    spec("show_root_lines", "ShowRootLines", "Appearance", Ty::Bool, "true", &[Kind::Tree], How::Call, "The top items have their buttons too."),
    spec("label_edit", "LabelEdit", "Behavior", Ty::Bool, "false", &[Kind::Tree], How::Call, "Its items can be renamed in place: a second, slow click, or F2."),
    spec("expand_all", "ExpandAll", "Behavior", Ty::Bool, "false", &[Kind::Tree], How::Call, "Every item open when the window opens."),
    spec("selected_index", "SelectedIndex", "Behavior", Ty::Int(0, 999), "0", &[Kind::Tabs], How::Code, "The page shown when the window opens, from 0."),
    // Dates.
    spec("format", "Format", "Appearance", Ty::Choice(&["short", "long", "time", "custom"]), "short", &[Kind::Date], How::Call, "How the date shows: Windows' short or long date, the time, or CustomFormat."),
    spec("custom_format", "CustomFormat", "Appearance", Ty::Text, "", &[Kind::Date], How::Call, "With Format custom: dd MMM yyyy, HH:mm and the like."),
    spec("date", "Value", "Data", Ty::Date, "", &[Kind::Date], How::Call, "The date when the window opens, 2026-11-19; unset, today."),
    spec("show_check_box", "ShowCheckBox", "Appearance", Ty::Bool, "false", &[Kind::Date], How::Code, "A tick before the date: unticked, no date is given."),
    spec("show_up_down", "ShowUpDown", "Appearance", Ty::Bool, "false", &[Kind::Date], How::Code, "Arrows to change it, in place of the calendar."),
    spec("min_date", "MinDate", "Data", Ty::Date, "", &[Kind::Date], How::Call, "The earliest date it takes."),
    spec("max_date", "MaxDate", "Data", Ty::Date, "", &[Kind::Date], How::Call, "The latest date it takes."),
    // Pictures and panels.
    spec("size_mode", "SizeMode", "Appearance", Ty::Choice(&["zoom", "normal", "stretch", "center"]), "zoom", &[Kind::Picture], How::Call, "Zoom: as large as fits, its shape kept. Normal: its own size, top left. Stretch: filling the box. Center: its own size, in the middle."),
    spec("direction", "FlowDirection", "Layout", Ty::Choice(&["left_to_right", "top_down"]), "left_to_right", &[Kind::Flow], How::Code, "Across, then the next row - or down, then the next column."),
    spec("wrap", "WrapContents", "Layout", Ty::Bool, "true", &[Kind::Flow], How::Code, "On to the next row when one is full; off, one row."),
    spec("gap", "Padding", "Layout", Ty::Int(0, 50), "6", &[Kind::Flow, Kind::Table], How::Code, "Pixels between the controls in it, and round its edge."),
];

const BORDERS: &[&str] = &["sizable", "fixed_single", "fixed_dialog", "fixed_tool", "sizable_tool", "none"];

/// The form's own, all made by the generator.
pub const FORM: &[Spec] = &[
    spec("accept_button", "AcceptButton", "Misc", Ty::Button, "", &[], How::Code, "The button Enter clicks, wherever the focus is."),
    spec("cancel_button", "CancelButton", "Misc", Ty::Button, "", &[], How::Code, "The button Escape clicks."),
    spec("key_preview", "KeyPreview", "Misc", Ty::Bool, "false", &[], How::Code, "The window's KeyDown, KeyUp and KeyPress hear keys meant for its controls, first."),
    spec("border", "FormBorderStyle", "Appearance", Ty::Choice(BORDERS), "sizable", &[], How::Code, "Sizable: its edges drag. Fixed: they do not. Dialog: a dialog's frame. Tool: a small title bar. None: no frame at all."),
    spec("phone", "PhoneLayout", "Layout", Ty::Choice(&["stack", "zoom"]), "stack", &[], How::Code, "On a screen narrower than the form - a phone, on the web: stack puts its controls one under another, as wide as the screen, in the order they read, big enough for a finger; zoom shrinks the whole form to fit."),
    spec("maximize_box", "MaximizeBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's Maximize button."),
    spec("minimize_box", "MinimizeBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's Minimize button."),
    spec("control_box", "ControlBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's buttons and its menu; off, none of them."),
    spec("user_control", "UserControl", "Design", Ty::Bool, "false", &[], How::Code, "Not a window but a control other forms of the project put on themselves (kind usercontrol)."),
    spec("is_mdi_container", "IsMdiContainer", "Window Style", Ty::Bool, "false", &[], How::Code, "Other forms open inside it (show_mdi_child), cascaded or tiled."),
    spec("top_most", "TopMost", "Window Style", Ty::Bool, "false", &[], How::Code, "It stays over every other window."),
    spec("show_in_taskbar", "ShowInTaskbar", "Window Style", Ty::Bool, "true", &[], How::Code, "It has a button on the taskbar."),
    spec("start_hidden", "StartHidden", "Window Style", Ty::Bool, "false", &[], How::Code, "Made, its Loaded raised, but not shown - a program that lives in its icon in the corner (NotifyIcon); ui.show_window() brings it up."),
    spec("opacity", "Opacity", "Window Style", Ty::Int(10, 100), "100", &[], How::Code, "How solid it is, percent; less shows what is behind."),
    spec("icon", "Icon", "Window Style", Ty::Text, "", &[], How::Code, "An .ico file of the project for its title bar and taskbar button."),
    spec("min_size", "MinimumSize", "Layout", Ty::Size, "", &[], How::Code, "The smallest its inside can be dragged, at 100% scale."),
    spec("max_size", "MaximumSize", "Layout", Ty::Size, "", &[], How::Code, "The largest its inside can be dragged, at 100% scale."),
    spec("back_color", "BackColor", "Appearance", Ty::Color, "", &[], How::Code, "This window's background over the theme's; its controls stand on it."),
    spec("fore_color", "ForeColor", "Appearance", Ty::Color, "", &[], How::Code, "This window's words' colour over the theme's."),
    spec("background_image", "BackgroundImage", "Appearance", Ty::Text, "", &[], How::Code, "A picture file of the project under the window's controls."),
    spec("background_image_layout", "BackgroundImageLayout", "Appearance", Ty::Choice(&["tile", "none", "center", "stretch", "zoom"]), "tile", &[], How::Code, "Tile: repeated. None: its size, top left. Center. Stretch: filling the window. Zoom: as large as fits."),
    spec("auto_scroll", "AutoScroll", "Layout", Ty::Bool, "false", &[], How::Code, "Smaller than designed, the window shows scroll bars and its controls scroll."),
    spec("padding", "Padding", "Layout", Ty::Int(0, 100), "0", &[], How::Code, "Pixels kept clear round the window's edge for docked controls."),
    spec("allow_drop", "AllowDrop", "Behavior", Ty::Bool, "false", &[], How::Code, "Files dragged from Explorer can be dropped on the window - its DragDrop event."),
];

/// Whether controls of `kind` have `s`.
pub fn applies(s: &Spec, kind: Kind) -> bool {
    if s.kinds.is_empty() {
        !kind.is_component()
    } else {
        s.kinds.contains(&kind)
    }
}

/// The properties a control of `kind` has from the table, in the table's order.
pub fn of_kind(kind: Kind) -> impl Iterator<Item = &'static Spec> {
    CONTROL.iter().filter(move |s| applies(s, kind))
}

/// Property `name` of a control of `kind`.
pub fn find(kind: Kind, name: &str) -> Option<&'static Spec> {
    of_kind(kind).find(|s| s.name == name)
}

pub fn find_form(name: &str) -> Option<&'static Spec> {
    FORM.iter().find(|s| s.name == name)
}

/// A value as the pane shows it and as `phai_ui::props::apply` takes it: `true`, `12`, `left`,
/// `640, 480`.
pub fn text_of(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(n) => n.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(a) => a.iter().map(text_of).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

/// What is wrong with value `v` for `s`, if anything.
pub fn check(s: &Spec, v: &toml::Value) -> Option<String> {
    let bad = |what: &str| Some(format!("{} {}: {what}", s.name, text_of(v)));
    match (s.ty, v) {
        (Ty::Bool, toml::Value::Boolean(_)) => None,
        (Ty::Int(lo, hi), toml::Value::Integer(n)) => (!(lo as i64..=hi as i64).contains(n)).then(|| format!("{} {n}: {lo} to {hi}", s.name)),
        (Ty::Text | Ty::Words | Ty::Button, toml::Value::String(_)) => None,
        (Ty::Choice(c), toml::Value::String(x)) => (!c.contains(&x.as_str())).then(|| format!("{} {x:?}: {}", s.name, c.join(", "))),
        (Ty::Color, toml::Value::String(x)) => phai_ui::theme::parse_color(x).is_none().then(|| format!("{} {x:?} is not a colour - #rrggbb, rgb(...) or a CSS name", s.name)),
        (Ty::Date, toml::Value::String(x)) => parse_date(x).is_none().then(|| format!("{} {x:?}: a day as 2026-11-19", s.name)),
        (Ty::Size, toml::Value::Array(a)) if a.len() == 2 && a.iter().all(|n| n.as_integer().is_some_and(|n| n > 0)) => None,
        (Ty::Bool, _) => bad("true or false"),
        (Ty::Int(..), _) => bad("a whole number"),
        (Ty::Size, _) => bad("[width, height]"),
        _ => bad("words in quotes"),
    }
}

/// `2026-11-19` as (year, month, day).
pub fn parse_date(s: &str) -> Option<(i32, i32, i32)> {
    let mut it = s.trim().split('-').map(|p| p.parse::<i32>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    (it.next().is_none() && (1601..=9999).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)).then_some((y, m, d))
}

/// What the pane's words become in the file: `None` when they are the default or empty (the
/// line is left out), an error when they are not a value of its kind.
#[allow(clippy::result_unit_err)]
pub fn parse(s: &Spec, text: &str) -> Result<Option<toml::Value>, ()> {
    let t = text.trim();
    if t.is_empty() || (t.eq_ignore_ascii_case(s.default) && !s.default.is_empty()) {
        return Ok(None);
    }
    let v = match s.ty {
        Ty::Bool => match t.to_ascii_lowercase().as_str() {
            "true" | "yes" => toml::Value::Boolean(true),
            "false" | "no" => toml::Value::Boolean(false),
            _ => return Err(()),
        },
        Ty::Int(..) => toml::Value::Integer(t.parse::<i64>().map_err(|_| ())?),
        Ty::Size => {
            let mut n = t.split([',', ';', 'x', '×', ' ']).map(str::trim).filter(|p| !p.is_empty()).map(|p| p.parse::<i64>());
            match (n.next(), n.next(), n.next()) {
                (Some(Ok(a)), Some(Ok(b)), None) => toml::Value::Array(vec![toml::Value::Integer(a), toml::Value::Integer(b)]),
                _ => return Err(()),
            }
        }
        Ty::Words => toml::Value::String(text.to_string()),
        _ => toml::Value::String(t.to_string()),
    };
    // A value equal to the default is no value: `true` for a default of "true".
    if text_of(&v) == s.default {
        return Ok(None);
    }
    match check(s, &v) {
        None => Ok(Some(v)),
        Some(_) => Err(()),
    }
}

/// The value as Rust, for the generated code: `true`, `12`, `"left"`, `t("Name")`.
pub fn rust_literal(s: &Spec, v: &toml::Value) -> String {
    match (s.ty, v) {
        (Ty::Words, toml::Value::String(x)) => format!("t({x:?})"),
        // A size goes to phai-ui as its words, "w, h".
        (Ty::Size, toml::Value::Array(_)) => format!("{:?}", text_of(v)),
        (_, toml::Value::String(x)) => format!("{x:?}"),
        _ => text_of(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Live Inspector (in phai-ui, which cannot read this table) writes a look of the normal
    /// state as the property this table has for it - for exactly the kinds that have it - and a
    /// property with the type this table gives it.
    #[test]
    fn the_live_inspector_writes_what_this_table_has() {
        const FIELDS: [&str; 5] = ["back_color", "fore_color", "border_color", "radius", "bold"];
        // A timer or a context menu has no window to inspect.
        for kind in Kind::ALL.into_iter().filter(|k| !k.is_component()) {
            for look in phai_ui::inspect::LOOKS {
                let css = CONTROL.iter().find(|s| matches!(s.how, How::Css(d) if d.starts_with(&format!("{look}:"))));
                match (phai_ui::inspect::form_key(kind.name(), look), css) {
                    (Some((key, _)), _) if FIELDS.contains(&key) => {}
                    (Some((key, _)), Some(s)) => assert!(s.name == key && applies(s, kind), "{} {look}", kind.name()),
                    (None, Some(s)) => assert!(!applies(s, kind), "{} {look} could be saved as {}", kind.name(), s.name),
                    (got, None) => assert!(got.is_none(), "{look}"),
                }
            }
        }
        for (name, ty) in phai_ui::props::NAMES {
            let s = CONTROL.iter().find(|s| s.name == *name).unwrap_or_else(|| panic!("{name}"));
            let want = match s.ty {
                Ty::Bool => 'b',
                Ty::Int(..) => 'n',
                _ => 't',
            };
            assert_eq!(*ty, want, "{name}");
        }
    }

    #[test]
    fn values_parse_check_and_print() {
        let ro = find(Kind::Edit, "read_only").unwrap();
        assert_eq!(parse(ro, "True"), Ok(Some(toml::Value::Boolean(true))));
        assert_eq!(parse(ro, "false"), Ok(None), "the default is no line");
        assert_eq!(parse(ro, "maybe"), Err(()));
        let ml = find(Kind::Edit, "max_length").unwrap();
        assert_eq!(parse(ml, "-3"), Err(()), "out of range");
        assert_eq!(rust_literal(ml, &parse(ml, "40").unwrap().unwrap()), "40");
        let ph = find(Kind::Edit, "placeholder").unwrap();
        assert_eq!(rust_literal(ph, &toml::Value::String("Name".into())), "t(\"Name\")");
        assert!(find(Kind::Button, "read_only").is_none(), "a button has no ReadOnly");
        assert_eq!(find(Kind::Button, "text_align").unwrap().default, "center");
        assert_eq!(find(Kind::Label, "text_align").unwrap().default, "left");
        let ms = find_form("max_size").unwrap();
        assert_eq!(text_of(&parse(ms, "800 x 600").unwrap().unwrap()), "800, 600");
        assert_eq!(parse_date("2026-11-19"), Some((2026, 11, 19)));
        assert_eq!(parse_date("19/11/2026"), None);
        assert!(find(Kind::Timer, "tooltip").is_none(), "a component has none of the controls' own");
    }

    #[test]
    fn every_name_is_once_per_kind() {
        for k in Kind::ALL {
            let names: Vec<&str> = of_kind(k).map(|s| s.name).collect();
            for (i, n) in names.iter().enumerate() {
                assert!(!names[i + 1..].contains(n), "{n} twice for {k:?}");
            }
        }
    }
}
