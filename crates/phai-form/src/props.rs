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

#[allow(clippy::too_many_arguments)]
const fn spec(name: &'static str, label: &'static str, category: &'static str, ty: Ty, default: &'static str, kinds: &'static [Kind], how: How, help: &'static str) -> Spec {
    Spec { name, label, category, ty, default, help, kinds, how, canvas: true }
}

const fn runtime_only(mut s: Spec) -> Spec {
    s.canvas = false;
    s
}

const ALIGN: &[&str] = &["left", "center", "right"];
const FOCUSABLE: &[Kind] = &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Combo, Kind::List, Kind::Grid, Kind::Tree, Kind::Date, Kind::Number, Kind::Slider, Kind::Tabs];
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
    spec("transition", "Transition", "Appearance", Ty::Text, "", &[], How::Css("transition: {}"), "Its colours change over time when its state does: 150ms, 0.3s ease-out - #id { transition }."),
    spec("gradient", "Background gradient", "Appearance", Ty::Text, "", &[Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Statusbar], How::Css("background-image: {}"), "Colours that blend across it: linear-gradient(to bottom, #ffffff, #dde3ea) - #id { background-image }."),
    spec("shadow", "Shadow", "Appearance", Ty::Text, "", &[Kind::Button, Kind::Checkbox, Kind::Radio], How::Css("box-shadow: {}"), "A soft shadow: 0 2px 6px rgba(0, 0, 0, 0.25) - across, down, blur, colour - #id { box-shadow }."),
    spec("icon_color", "Icon colour", "Appearance", Ty::Color, "", &[Kind::Tree], How::Css("icon-color: {}"), "Its icons' colour - #id { icon-color }."),
    // Words.
    spec("text_align", "TextAlign", "Appearance", Ty::Choice(ALIGN), "center", &[Kind::Button], How::Call, "Where its words sit."),
    spec("text_align", "TextAlign", "Appearance", Ty::Choice(ALIGN), "left", &[Kind::Label, Kind::Edit, Kind::Number], How::Call, "Where its words sit."),
    spec("border", "BorderStyle", "Appearance", Ty::Choice(&["none", "fixed"]), "none", &[Kind::Label, Kind::Picture], How::Call, "A thin line round it, or none."),
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
    spec("orientation", "Orientation", "Appearance", Ty::Choice(&["horizontal", "vertical"]), "horizontal", &[Kind::Slider], How::Call, "Lying down, or standing up with its least at the bottom."),
    spec("step", "Step", "Behavior", Ty::Int(1, 1_000_000), "10", &[Kind::Progress], How::Call, "How much perform_step adds."),
    spec("marquee", "Style = Marquee", "Appearance", Ty::Bool, "false", &[Kind::Progress], How::Call, "A block that runs along, for work of unknown length."),
    // Lists.
    spec("editable", "DropDownStyle = DropDown", "Behavior", Ty::Bool, "false", &[Kind::Combo], How::Code, "Words can be typed in as well as chosen."),
    spec("sorted", "Sorted", "Behavior", Ty::Bool, "false", &[Kind::Combo], How::Code, "Its items in alphabetical order."),
    spec("selected_index", "SelectedIndex", "Behavior", Ty::Int(-1, 9999), "0", &[Kind::Combo], How::Call, "The item chosen when the window opens, from 0; -1 none."),
    spec("max_drop_down_items", "MaxDropDownItems", "Behavior", Ty::Int(1, 100), "8", &[Kind::Combo], How::Call, "How many items the open list shows before it scrolls."),
    spec("view", "View", "Appearance", Ty::Choice(&["details", "list", "small_icons", "large_icons"]), "details", &[Kind::List], How::Call, "Rows with columns, or the items alone."),
    spec("grid_lines", "GridLines", "Appearance", Ty::Bool, "false", &[Kind::List], How::Call, "Lines between rows and columns."),
    spec("grid_lines", "GridLines", "Appearance", Ty::Bool, "true", &[Kind::Grid], How::Call, "Lines between rows and columns."),
    spec("full_row_select", "FullRowSelect", "Behavior", Ty::Bool, "true", &[Kind::List, Kind::Tree], How::Call, "A click chooses the whole row."),
    spec("multi_select", "MultiSelect", "Behavior", Ty::Bool, "true", &[Kind::List, Kind::Grid], How::Call, "Several rows can be chosen at once."),
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
    spec("maximize_box", "MaximizeBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's Maximize button."),
    spec("minimize_box", "MinimizeBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's Minimize button."),
    spec("control_box", "ControlBox", "Window Style", Ty::Bool, "true", &[], How::Code, "The title bar's buttons and its menu; off, none of them."),
    spec("top_most", "TopMost", "Window Style", Ty::Bool, "false", &[], How::Code, "It stays over every other window."),
    spec("show_in_taskbar", "ShowInTaskbar", "Window Style", Ty::Bool, "true", &[], How::Code, "It has a button on the taskbar."),
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
        (_, toml::Value::String(x)) => format!("{x:?}"),
        _ => text_of(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
