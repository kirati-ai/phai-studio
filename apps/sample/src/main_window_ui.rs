// Made by Phai Studio from main_window.phf - do not change this file; change the form.
// Everything here is written again on every save.

#![allow(dead_code, unused_imports, unused_variables, clippy::all)]

use phai_ui::controls;
use phai_ui::layout::{scale, unscale, Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::t;
use phai_ui::theme;
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Every English sentence this window shows - for the project's Thai table and its test.
pub const TEXTS: &[&str] = &[
    "Customers",
    "&File",
    "New customer",
    "Exit",
    "Save",
    "Customer",
    "Name",
    "Active",
    "Shop",
    "Office",
    "Phone",
    "Close",
    "Ready",
];

const ID_NEW_CUSTOMER: i32 = 1000;
const ID_EXIT: i32 = 1001;
const ID_SAVE: i32 = 1002;
const ID_NAME_CHANGED: i32 = 1003;
const ID_CUSTOMER_CHOSEN: i32 = 1004;
const ID_TOOLBAR: i32 = 1999;
const CONTROL_DETAILS: i32 = 2000;
const CONTROL_NAME_LABEL: i32 = 2001;
const CONTROL_NAME: i32 = 2002;
const CONTROL_ACTIVE: i32 = 2003;
const CONTROL_KIND: i32 = 2004;
const CONTROL_CUSTOMERS: i32 = 2005;
const CONTROL_SAVE_BUTTON: i32 = 2006;
const CONTROL_CLOSE: i32 = 2007;
const CONTROL_STATUS: i32 = 2008;

/// The window and every control in it, by the ids the form gives them.
pub struct MainWindowUi {
    pub hwnd: HWND,
    pub font: HFONT,
    pub toolbar: HWND,
    /// The GroupBox `details` on this window: `ui.details.` lists what it can do.
    pub details: phai_ui::ctl::GroupBox,
    /// The Label `name_label` on this window: `ui.name_label.` lists what it can do.
    pub name_label: phai_ui::ctl::Label,
    /// The TextBox `name` on this window: `ui.name.` lists what it can do.
    pub name: phai_ui::ctl::TextBox,
    /// The CheckBox `active` on this window: `ui.active.` lists what it can do.
    pub active: phai_ui::ctl::CheckBox,
    /// The ComboBox `kind` on this window: `ui.kind.` lists what it can do.
    pub kind: phai_ui::ctl::ComboBox,
    /// The ListView `customers` on this window: `ui.customers.` lists what it can do.
    pub customers: phai_ui::ctl::ListView,
    /// The Button `save_button` on this window: `ui.save_button.` lists what it can do.
    pub save_button: phai_ui::ctl::Button,
    /// The Button `close` on this window: `ui.close.` lists what it can do.
    pub close: phai_ui::ctl::Button,
    /// The StatusBar `status` on this window: `ui.status.` lists what it can do.
    pub status: phai_ui::ctl::StatusBar,
    layout: Layout,
}

/// What the window tells the program. Every method does nothing until the program says otherwise.
pub trait MainWindowEvents {
    /// The window is made and about to show.
    fn loaded(&mut self, ui: &MainWindowUi) {}
    fn new_customer(&mut self, ui: &MainWindowUi) {}
    fn exit(&mut self, ui: &MainWindowUi) {}
    fn save(&mut self, ui: &MainWindowUi) {}
    fn name_changed(&mut self, ui: &MainWindowUi) {}
    fn customer_chosen(&mut self, ui: &MainWindowUi) {}
}

struct Host<E> {
    ui: MainWindowUi,
    events: E,
    /// The program's main window: the program ends with it.
    main: bool,
}

impl MainWindowUi {
    /// Makes the window and runs it until it is closed.
    pub fn run<E: MainWindowEvents + 'static>(events: E) {
        let hwnd = Self::create(events);
        win::run_loop(Some(hwnd));
    }

    /// Makes the window and shows it; the caller runs the message loop.
    pub fn create<E: MainWindowEvents + 'static>(events: E) -> HWND {
        Self::make(None, events)
    }

    /// Opens this window inside `parent`, an MDI container (`is_mdi_container`), as WinForms'
    /// `MdiParent` and `Show()`; it closes with it.
    pub fn show_mdi_child<E: MainWindowEvents + 'static>(parent: HWND, events: E) -> HWND {
        phai_ui::mdi::set_next_parent(parent);
        Self::make(Some(parent), events)
    }

    /// Shows this window beside `owner` (another window of the program) and returns at once,
    /// as `Show()` in WinForms. Closing it leaves the program running.
    pub fn show<E: MainWindowEvents + 'static>(owner: HWND, events: E) -> HWND {
        Self::make(Some(owner), events)
    }

    /// Shows this window over `owner` and waits until it is closed, `owner` taking no input
    /// meanwhile - as `ShowDialog()` in WinForms. What it answers: the DialogResult of the
    /// button that closed it, or `Cancel`.
    pub fn show_dialog<E: MainWindowEvents + 'static>(owner: HWND, events: E) -> phai_ui::form::DialogResult {
        let hwnd = Self::make(Some(owner), events);
        win::modal_loop(hwnd, owner);
        phai_ui::form::take_result(hwnd)
    }

    /// As [`Self::show_dialog`], `events` given back when the window has closed - to read what
    /// was typed in it, as a WinForms form's fields after `ShowDialog()`:
    /// `if let (DialogResult::Ok, Some(f)) = Ui::show_dialog_for(owner, f) { f.answer }`.
    pub fn show_dialog_for<E: MainWindowEvents + 'static>(owner: HWND, events: E) -> (phai_ui::form::DialogResult, Option<E>) {
        let hwnd = Self::make(Some(owner), events);
        phai_ui::form::want_back(hwnd);
        win::modal_loop(hwnd, owner);
        (phai_ui::form::take_result(hwnd), phai_ui::form::take_kept::<E>(hwnd))
    }

    fn make<E: MainWindowEvents + 'static>(owner: Option<HWND>, events: E) -> HWND {
        win::init();
        if let Ok(sheet) = theme::Theme::parse(include_str!("../theme.css")) {
            theme::set(sheet);
        }
        let class = win::register("Phai.MainWindow", Some(proc_::<E>), None);
        let title = wide(t("Customers"));
        let (style, ex) = phai_ui::form::frame("sizable", true, true, true, false);
        let main = owner.is_none();
        let over = owner;
        unsafe {
            // Inside an MDI container when show_mdi_child asked; a window of its own otherwise.
            let mdi_frame = phai_ui::mdi::take_next();
            let hwnd = match mdi_frame {
                Some(frame) => phai_ui::mdi::create_child(frame, &class, &title, style),
                None => wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, owner, Some(Self::menu()), Some(win::instance()), None).unwrap_or_default(),
            };
            let ui = Self::build(hwnd);
            let mut host = Box::new(Host { ui, events, main });
            host.events.loaded(&host.ui);
            win::set_state(hwnd, host);
            win::theme_frame(hwnd);
            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
            let ex = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 480), bottom: win::px(hwnd, 360) };
            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), true, wm::WINDOW_EX_STYLE(ex), win::dpi(hwnd) as u32);
            let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
            if mdi_frame.is_some() {
                // Windows places a child form inside its container.
            } else if let Some(o) = over {
                win::center_on(hwnd, o);
            } else {
                win::center_on_screen(hwnd);
            }
            if mdi_frame.is_some() {
                win::show(hwnd, true);
            } else {
                win::show_top(hwnd);
            }
            // A debug build follows the theme file as it is saved.
            #[cfg(debug_assertions)]
            win::watch_theme(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/theme.css")));
            // F12 in a debug build: the Live Inspector, which saves into the form.
            #[cfg(debug_assertions)]
            phai_ui::inspect::attach(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main_window.phf")));
            hwnd
        }
    }

    fn menu() -> wm::HMENU {
        unsafe {
            let bar = wm::CreateMenu().unwrap_or_default();
            let m1 = wm::CreatePopupMenu().unwrap_or_default();
            let s = wide(t("New customer"));
            let _ = wm::AppendMenuW(m1, wm::MF_STRING, ID_NEW_CUSTOMER as usize, PCWSTR(s.as_ptr()));
            let _ = wm::AppendMenuW(m1, wm::MF_SEPARATOR, 0, None);
            let s = wide(t("Exit"));
            let _ = wm::AppendMenuW(m1, wm::MF_STRING, ID_EXIT as usize, PCWSTR(s.as_ptr()));
            let s = wide(t("&File"));
            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m1.0 as usize, PCWSTR(s.as_ptr()));
            bar
        }
    }

    fn toolbar_items() -> Vec<(i32, &'static str, u16)> {
        vec![
            (ID_NEW_CUSTOMER, t("New customer"), phai_ui::icon::glyph::NEW),
            (ID_SAVE, t("Save"), phai_ui::icon::glyph::SAVE),
        ]
    }

    fn build(hwnd: HWND) -> MainWindowUi {
        let font = win::font(hwnd, 0, false);
        let dpi = win::dpi(hwnd);
        #[allow(unused_mut)]
        let mut layout = Layout::new((480, 360));
        let toolbar = win::toolbar(hwnd, ID_TOOLBAR, &Self::toolbar_items());
        layout.add(toolbar, Item { bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() });
        let details = controls::group(hwnd, t("Customer"), CONTROL_DETAILS, font);
        win::set_css(details, "", "details");
        layout.add(details, Item { bounds: Rect::new(11, 45, 458, 90), dock: Dock::None, anchor: Anchor { left: true, top: true, right: true, bottom: false } });
        let name_label = win::label(hwnd, t("Name"), CONTROL_NAME_LABEL, font);
        win::set_css(name_label, "", "name_label");
        layout.add(name_label, Item { bounds: Rect::new(23, 67, 60, 23), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let name = win::edit(hwnd, "", CONTROL_NAME, font, false);
        win::set_css(name, "", "name");
        layout.add(name, Item { bounds: Rect::new(95, 67, 362, 23), dock: Dock::None, anchor: Anchor { left: true, top: true, right: true, bottom: false } });
        let active = win::checkbox(hwnd, t("Active"), CONTROL_ACTIVE, font, true);
        win::set_css(active, "", "active");
        layout.add(active, Item { bounds: Rect::new(95, 101, 110, 20), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let kind = win::combo_with(hwnd, CONTROL_KIND, font, &[t("Shop"), t("Office")], false);
        win::set_css(kind, "", "kind");
        layout.add(kind, Item { bounds: Rect::new(215, 99, 120, 23), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let customers = win::list(hwnd, CONTROL_CUSTOMERS, font, &[(t("Name"), 200), (t("Phone"), 120)]);
        win::set_css(customers, "", "customers");
        layout.add(customers, Item { bounds: Rect::new(11, 146, 458, 147), dock: Dock::None, anchor: Anchor { left: true, top: true, right: true, bottom: true } });
        let save_button = win::button(hwnd, t("Save"), CONTROL_SAVE_BUTTON, font, false);
        win::set_css(save_button, "primary", "save_button");
        layout.add(save_button, Item { bounds: Rect::new(312, 302, 75, 23), dock: Dock::None, anchor: Anchor { left: false, top: false, right: true, bottom: true } });
        let close = win::button(hwnd, t("Close"), CONTROL_CLOSE, font, false);
        win::set_css(close, "", "close");
        layout.add(close, Item { bounds: Rect::new(394, 302, 75, 23), dock: Dock::None, anchor: Anchor { left: false, top: false, right: true, bottom: true } });
        let status = controls::status_bar(hwnd, CONTROL_STATUS, font);
        win::set_css(status, "", "status");
        controls::status_parts(status, &[120]);
        controls::status_text(status, 0, t("Ready"));
        layout.add(status, Item { bounds: Rect::new(0, 0, 0, unscale(controls::status_height(status), dpi)), dock: Dock::Bottom, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        MainWindowUi {
            hwnd,
            font,
            toolbar,
            details: phai_ui::ctl::GroupBox(details),
            name_label: phai_ui::ctl::Label(name_label),
            name: phai_ui::ctl::TextBox(name),
            active: phai_ui::ctl::CheckBox(active),
            kind: phai_ui::ctl::ComboBox(kind),
            customers: phai_ui::ctl::ListView(customers),
            save_button: phai_ui::ctl::Button(save_button),
            close: phai_ui::ctl::Button(close),
            status: phai_ui::ctl::StatusBar(status),
            layout,
        }
    }

    /// Closes the window, as its close box would - its FormClosing comes first.
    ///
    /// Example: `ui.close();`
    pub fn close(&self) {
        unsafe {
            let _ = wm::PostMessageW(Some(self.hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    fn item_id(item: &str) -> i32 {
        match item { "new_customer" => ID_NEW_CUSTOMER, "exit" => ID_EXIT, "save" => ID_SAVE, _ => 0 }
    }

    /// Whether menu item (or toolbar button) `item` is ticked - its id in the form.
    pub fn item_checked(&self, item: &str) -> bool {
        phai_ui::form::item_checked(self.hwnd, Some(self.toolbar), Self::item_id(item))
    }

    /// Ticks menu item (and toolbar button) `item`, or takes the tick away - Checked.
    pub fn set_item_checked(&self, item: &str, on: bool) {
        phai_ui::form::set_item_checked(self.hwnd, Some(self.toolbar), Self::item_id(item), on)
    }

    /// Menu item (and toolbar button) `item` usable or greyed - Enabled.
    pub fn set_item_enabled(&self, item: &str, on: bool) {
        phai_ui::form::set_item_enabled(self.hwnd, Some(self.toolbar), Self::item_id(item), on)
    }

    fn toolbar_handle(&self) -> isize {
        self.toolbar.0 as isize
    }

    /// Hides the window; the program keeps running and `ui.show_window()` brings it back.
    ///
    /// Example: `ui.hide();`
    pub fn hide(&self) {
        win::show(self.hwnd, false);
    }

    /// Shows the window again after `ui.hide()`.
    ///
    /// Example: `ui.show_window();`
    pub fn show_window(&self) {
        win::show(self.hwnd, true);
    }

    /// Brings the window to the front, the keyboard on it.
    ///
    /// Example: `ui.activate();`
    pub fn activate(&self) {
        unsafe {
            let _ = wm::SetForegroundWindow(self.hwnd);
        }
    }

    /// A message box with the buttons and the icon you choose, its title `caption`; the button
    /// pressed comes back - DialogResult::Yes, No, Ok, Cancel... For a plain notice `ui.alert` is
    /// shorter, for Yes or No `ui.confirm`.
    ///
    /// Example: `if ui.message_box("Delete it?", "Files", MessageBoxButtons::YesNo, MessageBoxIcon::Question) == DialogResult::Yes { ... }`
    pub fn message_box(&self, text: &str, caption: &str, buttons: phai_ui::form::MessageBoxButtons, icon: phai_ui::form::MessageBoxIcon) -> phai_ui::form::DialogResult {
        phai_ui::form::message_box(self.hwnd, text, caption, buttons, icon)
    }

    /// Shows a message with an OK button and waits until it is pressed; the title is the
    /// window's - as JavaScript's alert.
    ///
    /// Example: `ui.alert("Saved");`
    pub fn alert(&self, text: &str) {
        phai_ui::form::message_box(self.hwnd, text, &self.title(), phai_ui::form::MessageBoxButtons::Ok, phai_ui::form::MessageBoxIcon::Information);
    }

    /// Asks a question with Yes and No buttons: `true` when Yes is pressed - as JavaScript's
    /// confirm.
    ///
    /// Example: `if ui.confirm("Delete it?") { ... }`
    pub fn confirm(&self, text: &str) -> bool {
        phai_ui::form::message_box(self.hwnd, text, &self.title(), phai_ui::form::MessageBoxButtons::YesNo, phai_ui::form::MessageBoxIcon::Question) == phai_ui::form::DialogResult::Yes
    }

    /// Asks for words in a small window with a box to type in: what was typed, or `None` when
    /// Cancel is pressed; `default` is in the box at first - as JavaScript's prompt.
    ///
    /// Example: `if let Some(name) = ui.prompt("Your name?", "") { ui.alert(&name); }`
    pub fn prompt(&self, prompt: &str, default: &str) -> Option<String> {
        phai_ui::dialogs::ask_fields(Some(self.hwnd), &self.title(), &[phai_ui::dialogs::Field::Text(prompt, default)]).and_then(|a| a.into_iter().next())
    }

    /// Lets the user choose a file to open: its path, or `None` when Cancel is pressed. The
    /// filter is name|pattern - empty for every file.
    ///
    /// Example: `if let Some(path) = ui.open_file("Text files|*.txt") { ... }`
    pub fn open_file(&self, filter: &str) -> Option<std::path::PathBuf> {
        phai_ui::components::OpenFileDialog::new(self.hwnd, "", filter, "", "", false).show_dialog()
    }

    /// Lets the user choose where to save a file: its path, or `None` when Cancel is pressed;
    /// `name` is the file name offered at first.
    ///
    /// Example: `if let Some(path) = ui.save_file("Text files|*.txt", "notes.txt") { ... }`
    pub fn save_file(&self, filter: &str, name: &str) -> Option<std::path::PathBuf> {
        phai_ui::components::SaveFileDialog::new(self.hwnd, "", filter, "", "").show_dialog_named(name)
    }

    /// Lets the user choose a folder: its path, or `None` when Cancel is pressed.
    ///
    /// Example: `if let Some(folder) = ui.choose_folder() { ... }`
    pub fn choose_folder(&self) -> Option<std::path::PathBuf> {
        phai_ui::components::FolderBrowserDialog::new(self.hwnd, "", "").show_dialog()
    }

    /// Moves the window to the middle of the screen.
    ///
    /// Example: `ui.center_to_screen();`
    pub fn center_to_screen(&self) {
        win::center_on_screen(self.hwnd);
    }

    /// The words in the window's title bar.
    ///
    /// Example: `let t = ui.title();`
    pub fn title(&self) -> String {
        win::text_of(self.hwnd)
    }

    /// Changes the words in the window's title bar.
    ///
    /// Example: `ui.set_title("Score book - 2026");`
    pub fn set_title(&self, s: &str) {
        win::set_text(self.hwnd, s);
    }

    /// A handle other threads use to run work on this window's thread - [`Self::begin_invoke`].
    pub fn invoker(&self) -> phai_ui::invoke::Invoker {
        phai_ui::invoke::Invoker::new(self.hwnd)
    }

    /// From any thread: `f` runs on the window's thread with the program's MainWindow and the
    /// controls, soon; returns at once - WinForms' BeginInvoke. `false` when the window is gone.
    pub fn begin_invoke<E: MainWindowEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &MainWindowUi) + Send + 'static) -> bool {
        inv.post(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<MainWindowUi>()) {
                f(e, ui);
            }
        }))
    }

    /// As [`Self::begin_invoke`], waiting until `f` has run - WinForms' Invoke.
    pub fn invoke<E: MainWindowEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &MainWindowUi) + Send + 'static) {
        inv.send(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<MainWindowUi>()) {
                f(e, ui);
            }
        }))
    }

    /// Every word again in the language now chosen - after `phai_ui::text::set_thai`.
    pub fn relabel(&self) {
        win::set_text(self.hwnd, t("Customers"));
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(Self::menu()));
            let _ = wm::DestroyMenu(old);
        }
        win::set_toolbar(self.toolbar, &Self::toolbar_items());
        win::set_text(self.details, t("Customer"));
        win::set_text(self.name_label, t("Name"));
        win::set_text(self.active, t("Active"));
        win::set_text(self.save_button, t("Save"));
        win::set_text(self.close, t("Close"));
        controls::status_text(self.status, 0, t("Ready"));
    }
}

extern "system" fn proc_<E: MainWindowEvents + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == wm::WM_GETMINMAXINFO {
        phai_ui::form::min_max(hwnd, l, Some((400, 300)), None, true);
        return LRESULT(0);
    }
    let Some(host) = (unsafe { win::state::<Host<E>>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            host.ui.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let (id, code) = (win::loword(w.0) as i32, win::hiword(w.0));
            let Host { ui, events, .. } = host;
            match id {
                ID_NEW_CUSTOMER => events.new_customer(ui),
                ID_EXIT => events.exit(ui),
                ID_SAVE => events.save(ui),
                CONTROL_NAME if code == 0x0300 => events.name_changed(ui), // EN_CHANGE, win::VALUE_CHANGED
                CONTROL_SAVE_BUTTON if code == 0 => events.save(ui),
                CONTROL_CLOSE if code == 0 => events.exit(ui),
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            let Host { ui, events, .. } = host;
            if hdr.idFrom == CONTROL_CUSTOMERS as usize && hdr.code == cc::LVN_ITEMCHANGED {
                let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if it.uNewState & cc::LVIS_SELECTED.0 != 0 && it.uOldState & cc::LVIS_SELECTED.0 == 0 {
                    events.customer_chosen(ui);
                }
            }
            win::def(hwnd, msg, w, l)
        }
        phai_ui::invoke::WM_INVOKE => {
            // Work handed over from another thread (begin_invoke, invoke).
            LRESULT(unsafe { phai_ui::invoke::run(l, &mut host.events, &host.ui) })
        }
        0x02E0 => {
            // WM_DPICHANGED: the window, its font and its icons at the new scale.
            let r = unsafe { &*(l.0 as *const RECT) };
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
                let _ = Gdi::DeleteObject(host.ui.font.into());
            }
            host.ui.font = win::font(hwnd, 0, false);
            win::set_font_all(hwnd, host.ui.font);
            win::retheme(hwnd);
            host.ui.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            // The program ends with its main window; a window shown over another only closes.
            if host.main {
                unsafe { wm::PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(host) = unsafe { win::take_state::<Host<E>>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(host.ui.font.into());
                }
                // Given back to show_dialog_for, when it asked.
                phai_ui::form::keep(hwnd, Box::new(host.events));
            }
            win::def(hwnd, msg, w, l)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}
