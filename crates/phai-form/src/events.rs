//! The events a form and its controls raise beyond each control's one (`on_click`,
//! `on_change`...): WinForms' Enter, Leave, KeyDown, MouseEnter, DoubleClick, FormClosing and the
//! rest, in one table - their names in the file (`events = { key_down = "name_key_down" }`),
//! what WinForms calls them, which controls have them, and what the method is given.

use crate::model::Kind;

/// What an event's method is given besides the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Args {
    None,
    Key,
    KeyPress,
    Mouse,
    Closing,
    Item,
    Files,
    Print,
    FileOk,
    Paint,
}

impl Args {
    /// The parameter after `ui`, as the generated trait writes it.
    pub fn param(self) -> Option<&'static str> {
        match self {
            Args::None => None,
            Args::Key => Some("phai_ui::events::KeyEventArgs"),
            Args::KeyPress => Some("phai_ui::events::KeyPressEventArgs"),
            Args::Mouse => Some("phai_ui::events::MouseEventArgs"),
            Args::Closing => Some("phai_ui::events::ClosingEventArgs"),
            Args::Item => Some("phai_ui::events::ItemEventArgs"),
            Args::Files => Some("phai_ui::events::FileDropEventArgs"),
            Args::Print => Some("phai_ui::print::PrintPageEventArgs"),
            Args::Paint => Some("phai_ui::paint::PaintEventArgs"),
            Args::FileOk => Some("phai_ui::events::FileOkEventArgs"),
        }
    }
}

/// Where the generated code hears it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// `phai_ui::events::hook` on the control, this code.
    Hook(u16),
    /// The control's notification in WM_COMMAND, this code.
    Command(u16),
    /// A list's WM_NOTIFY, this code.
    Notify(i32),
    /// The form's own message.
    Form,
}

#[derive(Clone, Copy, Debug)]
pub struct EventSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub args: Args,
    pub kinds: &'static [Kind],
    pub source: Source,
    pub help: &'static str,
}

const fn ev(name: &'static str, label: &'static str, args: Args, kinds: &'static [Kind], source: Source, help: &'static str) -> EventSpec {
    EventSpec { name, label, args, kinds, source, help }
}

/// Those that hear keys: the focusable, and a panel or picture the program draws itself, which
/// takes the focus when clicked once it hears a key (a game board, a remote screen).
const KEYED: &[Kind] = &[Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Combo, Kind::List, Kind::Grid, Kind::Tree, Kind::Date, Kind::Number, Kind::Slider, Kind::Tabs, Kind::ListBox, Kind::CheckedList, Kind::Link, Kind::Masked, Kind::Calendar, Kind::Panel, Kind::Picture];
const CLICKABLE: &[Kind] = &[Kind::Label, Kind::Picture, Kind::Group, Kind::Edit, Kind::Textarea, Kind::Rich, Kind::List, Kind::Grid, Kind::Tree, Kind::Progress, Kind::Statusbar, Kind::Flow, Kind::Table, Kind::Panel, Kind::ListBox, Kind::CheckedList, Kind::Calendar];

pub const CONTROL: &[EventSpec] = &[
    ev("click", "Click", Args::None, CLICKABLE, Source::Hook(phai_ui_codes::CLICK), "Clicked with the left button."),
    ev("double_click", "DoubleClick", Args::None, &[], Source::Hook(phai_ui_codes::DOUBLE_CLICK), "Clicked twice, quickly."),
    ev("enter", "Enter", Args::None, KEYED, Source::Hook(phai_ui_codes::ENTER), "It takes the focus: the next keys are for it."),
    ev("leave", "Leave", Args::None, KEYED, Source::Hook(phai_ui_codes::LEAVE), "The focus goes elsewhere."),
    ev("key_down", "KeyDown", Args::Key, KEYED, Source::Hook(phai_ui_codes::KEY_DOWN), "A key pressed while it has the focus - e.key, e.ctrl; e.set_handled() keeps it from the control."),
    ev("key_up", "KeyUp", Args::Key, KEYED, Source::Hook(phai_ui_codes::KEY_UP), "A key let go while it has the focus."),
    ev("key_press", "KeyPress", Args::KeyPress, KEYED, Source::Hook(phai_ui_codes::KEY_PRESS), "A letter typed - e.ch; e.set_handled() keeps it out."),
    ev("mouse_enter", "MouseEnter", Args::None, &[], Source::Hook(phai_ui_codes::MOUSE_ENTER), "The mouse comes over it."),
    ev("mouse_leave", "MouseLeave", Args::None, &[], Source::Hook(phai_ui_codes::MOUSE_LEAVE), "The mouse goes off it."),
    ev("mouse_down", "MouseDown", Args::Mouse, &[], Source::Hook(phai_ui_codes::MOUSE_DOWN), "A mouse button pressed over it - e.x, e.y, e.button."),
    ev("mouse_up", "MouseUp", Args::Mouse, &[], Source::Hook(phai_ui_codes::MOUSE_UP), "A mouse button let go over it."),
    ev("mouse_move", "MouseMove", Args::Mouse, &[], Source::Hook(phai_ui_codes::MOUSE_MOVE), "The mouse moves over it."),
    ev("mouse_wheel", "MouseWheel", Args::Mouse, &[], Source::Hook(phai_ui_codes::MOUSE_WHEEL), "The mouse wheel turned over it - e.delta, 120 a notch, more than 0 away from you."),
    ev("resize", "Resize", Args::None, &[], Source::Hook(phai_ui_codes::RESIZE), "Its size changed - the window was sized, and it is anchored or docked."),
    ev("drag_drop", "DragDrop", Args::Files, &[], Source::Hook(phai_ui_codes::DRAG_DROP), "Dropped on it: files from Explorer (e.files) or words dragged from another control (e.text, e.source)."),
    ev("checked_changed", "CheckedChanged", Args::None, &[Kind::Checkbox, Kind::Radio], Source::Command(0), "Ticked or unticked."),
    ev("text_changed", "TextChanged", Args::None, &[Kind::Combo], Source::Command(5), "Words typed into a drop-down that takes them."),
    ev("column_click", "ColumnClick", Args::Item, &[Kind::List, Kind::Grid], Source::Notify(-108), "A column's heading clicked - e.index, from 0."),
    ev("double_click", "MouseDoubleClick", Args::None, &[Kind::NotifyIcon], Source::Notify(0x0203), "The icon in the corner clicked twice - as often, to show the window again."),
    ev("balloon_click", "BalloonTipClicked", Args::None, &[Kind::NotifyIcon], Source::Notify(0x0405), "Its notice (show_balloon_tip) clicked - to open what it told of."),
    ev("balloon_closed", "BalloonTipClosed", Args::None, &[Kind::NotifyIcon], Source::Notify(0x0404), "Its notice gone unclicked: timed out, or closed with its X."),
    ev("position_changed", "PositionChanged", Args::None, &[Kind::BindingSource], Source::Hook(phai_ui_codes::POSITION_CHANGED), "Another row became the current one - moved to, added, taken out, or new rows given."),
    ev("paint", "Paint", Args::Paint, &[Kind::Panel, Kind::Picture], Source::Hook(phai_ui_codes::PAINT), "The program draws it - e.clear, e.fill_rect, e.draw_text, e.draw_pixels(&bgra, (w, h), e.fit(w, h)), in its own pixels; ui.x.invalidate() asks again."),
    ev("print_page", "PrintPage", Args::Print, &[Kind::PrintDocument], Source::Hook(phai_ui_codes::PRINT_PAGE), "A page to draw - e.draw_text, e.draw_line, e.margin_bounds, in hundredths of an inch; e.set_has_more_pages(true) when another follows."),
    ev("selection_changed", "SelectionChanged", Args::None, &[Kind::Grid], Source::Notify(-101), "Another row chosen."),
    ev("opening", "Opening", Args::None, &[Kind::ContextMenu], Source::Form, "The menu is about to open - tick, grey or rename its items here."),
    ev("file_ok", "FileOk", Args::FileOk, &[Kind::OpenFile, Kind::SaveFile], Source::Hook(phai_ui_codes::FILE_OK), "Open or Save clicked - e.file(), e.files; e.cancel() keeps the dialog open for another choice."),
];

/// A menu item's own, beyond its Click (its id's method).
pub const MENU_ITEM: &[EventSpec] = &[
    ev("checked_changed", "CheckedChanged", Args::None, &[], Source::Form, "Its tick turned over (CheckOnClick)."),
    ev("drop_down_opening", "DropDownOpening", Args::None, &[], Source::Form, "The menu within it is about to open - tick or grey its items here."),
    ev("drop_down_closed", "DropDownClosed", Args::None, &[], Source::Form, "The menu within it has closed."),
];

/// A menu on the bar's.
pub const MENU: &[EventSpec] = &[
    ev("drop_down_opening", "DropDownOpening", Args::None, &[], Source::Form, "The menu is about to open - tick or grey its items here."),
    ev("drop_down_closed", "DropDownClosed", Args::None, &[], Source::Form, "The menu has closed."),
];

/// A toolbar button's own, beyond its Click (its id's method).
pub const TOOL: &[EventSpec] = &[ev("checked_changed", "CheckedChanged", Args::None, &[], Source::Form, "It went down or came up (CheckOnClick).")];

pub fn find_in(table: &'static [EventSpec], name: &str) -> Option<&'static EventSpec> {
    table.iter().find(|e| e.name == name)
}

/// The form's own.
pub const FORM: &[EventSpec] = &[
    ev("shown", "Shown", Args::None, &[], Source::Form, "The window is on the screen for the first time."),
    ev("closing", "FormClosing", Args::Closing, &[], Source::Form, "The window is about to close - e.cancel() keeps it open."),
    ev("closed", "FormClosed", Args::None, &[], Source::Form, "The window has closed."),
    ev("resize", "Resize", Args::None, &[], Source::Form, "The window's size changed."),
    ev("activated", "Activated", Args::None, &[], Source::Form, "The window became the one in front, taking the keys."),
    ev("deactivate", "Deactivate", Args::None, &[], Source::Form, "Another window came in front."),
    ev("click", "Click", Args::None, &[], Source::Form, "The window itself clicked, not a control in it."),
    ev("double_click", "DoubleClick", Args::None, &[], Source::Form, "The window itself clicked twice."),
    ev("mouse_down", "MouseDown", Args::Mouse, &[], Source::Form, "A mouse button pressed on the window itself."),
    ev("mouse_up", "MouseUp", Args::Mouse, &[], Source::Form, "A mouse button let go on the window itself."),
    ev("mouse_move", "MouseMove", Args::Mouse, &[], Source::Form, "The mouse moves over the window itself."),
    ev("drag_drop", "DragDrop", Args::Files, &[], Source::Form, "Files dropped on the window from Explorer - e.files."),
    ev("key_down", "KeyDown", Args::Key, &[], Source::Form, "A key pressed anywhere in the window when KeyPreview is on - before the control has it; e.set_handled() keeps it from the control."),
    ev("key_up", "KeyUp", Args::Key, &[], Source::Form, "A key let go anywhere in the window when KeyPreview is on."),
    ev("key_press", "KeyPress", Args::KeyPress, &[], Source::Form, "A letter typed anywhere in the window when KeyPreview is on."),
    ev("menu_start", "MenuStart", Args::None, &[], Source::Form, "The menu bar is being used: a menu opened from it, or Alt pressed."),
    ev("menu_complete", "MenuComplete", Args::None, &[], Source::Form, "The menu bar is no longer being used."),
    ev("toolbar_item_clicked", "ItemClicked", Args::Item, &[], Source::Form, "A toolbar button clicked - e.index, its place on the toolbar from 0 (separators count)."),
];

/// The form's events that are the menu bar's, and the toolbar's - shown for those in Properties.
pub const MENU_STRIP: &[&str] = &["menu_start", "menu_complete"];
pub const TOOL_STRIP: &[&str] = &["toolbar_item_clicked"];

/// The codes of `phai_ui::events`, here so the table is a constant.
pub mod phai_ui_codes {
    pub const ENTER: u16 = phai_ui::events::ENTER;
    pub const LEAVE: u16 = phai_ui::events::LEAVE;
    pub const KEY_DOWN: u16 = phai_ui::events::KEY_DOWN;
    pub const KEY_UP: u16 = phai_ui::events::KEY_UP;
    pub const KEY_PRESS: u16 = phai_ui::events::KEY_PRESS;
    pub const MOUSE_ENTER: u16 = phai_ui::events::MOUSE_ENTER;
    pub const MOUSE_LEAVE: u16 = phai_ui::events::MOUSE_LEAVE;
    pub const MOUSE_DOWN: u16 = phai_ui::events::MOUSE_DOWN;
    pub const MOUSE_UP: u16 = phai_ui::events::MOUSE_UP;
    pub const MOUSE_MOVE: u16 = phai_ui::events::MOUSE_MOVE;
    pub const CLICK: u16 = phai_ui::events::CLICK;
    pub const DOUBLE_CLICK: u16 = phai_ui::events::DOUBLE_CLICK;
    pub const RESIZE: u16 = phai_ui::events::RESIZE;
    pub const DRAG_DROP: u16 = phai_ui::events::DRAG_DROP;
    pub const PRINT_PAGE: u16 = phai_ui::events::PRINT_PAGE;
    pub const POSITION_CHANGED: u16 = phai_ui::events::POSITION_CHANGED;
    pub const FILE_OK: u16 = phai_ui::events::FILE_OK;
    pub const PAINT: u16 = phai_ui::events::PAINT;
    pub const MOUSE_WHEEL: u16 = phai_ui::events::MOUSE_WHEEL;
}

/// Whether controls of `kind` raise `e`: an empty list is every control on the form.
pub fn applies(e: &EventSpec, kind: Kind) -> bool {
    if e.kinds.is_empty() {
        !kind.is_component()
    } else {
        e.kinds.contains(&kind)
    }
}

pub fn of_kind(kind: Kind) -> impl Iterator<Item = &'static EventSpec> {
    CONTROL.iter().filter(move |e| applies(e, kind))
}

pub fn find(kind: Kind, name: &str) -> Option<&'static EventSpec> {
    of_kind(kind).find(|e| e.name == name)
}

pub fn find_form(name: &str) -> Option<&'static EventSpec> {
    FORM.iter().find(|e| e.name == name)
}

/// A method's name for event `e` of control `id`, as Visual Studio names one: `name_key_down`.
pub fn method_name(id: &str, e: &str) -> String {
    format!("{id}_{e}")
}
