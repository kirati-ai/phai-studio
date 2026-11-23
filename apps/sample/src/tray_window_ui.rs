// Made by Phai Studio from tray_window.phf - do not change this file; change the form.
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
    "Phai Tray",
    "Phai Tray lives in the corner of the taskbar. Right-click its icon for its menu.",
    "Hide",
    "Open Phai Tray",
    "Pretend a remote session",
    "End remote session",
    "Language",
    "English",
    "Thai",
    "Show a notice",
    "Exit",
];

const ID_HIDE_CLICKED: i32 = 1000;
const ID_OPEN_WINDOW: i32 = 1001;
const ID_PRETEND_SESSION: i32 = 1002;
const ID_END_SESSION: i32 = 1003;
const ID_ENGLISH: i32 = 1004;
const ID_THAI: i32 = 1005;
const ID_SHOW_NOTICE: i32 = 1006;
const ID_EXIT: i32 = 1007;
const ID_TOOLBAR: i32 = 1999;
const CONTROL_ABOUT: i32 = 2000;
const CONTROL_HIDE_BUTTON: i32 = 2001;
const CONTROL_TRAY_MENU: i32 = 2002;
const CONTROL_TRAY: i32 = 2003;

/// The window and every control in it, by the ids the form gives them.
pub struct TrayWindowUi {
    pub hwnd: HWND,
    pub font: HFONT,
    pub about: phai_ui::ctl::Label,
    pub hide_button: phai_ui::ctl::Button,
    pub tray_menu: phai_ui::ctl::ContextMenu,
    pub tray: phai_ui::ctl::NotifyIcon,
    layout: Layout,
}

/// What the window tells the program. Every method does nothing until the program says otherwise.
pub trait TrayWindowEvents {
    /// The window is made and about to show.
    fn loaded(&mut self, ui: &TrayWindowUi) {}
    fn hide_clicked(&mut self, ui: &TrayWindowUi) {}
    fn open_window(&mut self, ui: &TrayWindowUi) {}
    fn pretend_session(&mut self, ui: &TrayWindowUi) {}
    fn end_session(&mut self, ui: &TrayWindowUi) {}
    fn english(&mut self, ui: &TrayWindowUi) {}
    fn thai(&mut self, ui: &TrayWindowUi) {}
    fn show_notice(&mut self, ui: &TrayWindowUi) {}
    fn exit(&mut self, ui: &TrayWindowUi) {}
    fn window_closing(&mut self, ui: &TrayWindowUi, e: &phai_ui::events::ClosingEventArgs) {}
    fn tray_menu_opening(&mut self, ui: &TrayWindowUi) {}
    fn notice_clicked(&mut self, ui: &TrayWindowUi) {}
}

struct Host<E> {
    ui: TrayWindowUi,
    events: E,
    /// The program's main window: the program ends with it.
    main: bool,
}

impl TrayWindowUi {
    /// Makes the window and runs it until it is closed.
    pub fn run<E: TrayWindowEvents + 'static>(events: E) {
        let hwnd = Self::create(events);
        win::run_loop(Some(hwnd));
    }

    /// Makes the window and shows it; the caller runs the message loop.
    pub fn create<E: TrayWindowEvents + 'static>(events: E) -> HWND {
        Self::make(None, events)
    }

    /// Opens this window inside `parent`, an MDI container (`is_mdi_container`), as WinForms'
    /// `MdiParent` and `Show()`; it closes with it.
    pub fn show_mdi_child<E: TrayWindowEvents + 'static>(parent: HWND, events: E) -> HWND {
        phai_ui::mdi::set_next_parent(parent);
        Self::make(Some(parent), events)
    }

    /// Shows this window beside `owner` (another window of the program) and returns at once,
    /// as `Show()` in WinForms. Closing it leaves the program running.
    pub fn show<E: TrayWindowEvents + 'static>(owner: HWND, events: E) -> HWND {
        Self::make(Some(owner), events)
    }

    /// Shows this window over `owner` and waits until it is closed, `owner` taking no input
    /// meanwhile - as `ShowDialog()` in WinForms. What it answers: the DialogResult of the
    /// button that closed it, or `Cancel`.
    pub fn show_dialog<E: TrayWindowEvents + 'static>(owner: HWND, events: E) -> phai_ui::form::DialogResult {
        let hwnd = Self::make(Some(owner), events);
        Self::first_shown(hwnd);
        win::modal_loop(hwnd, owner);
        phai_ui::form::take_result(hwnd)
    }

    /// As [`Self::show_dialog`], `events` given back when the window has closed - to read what
    /// was typed in it, as a WinForms form's fields after `ShowDialog()`:
    /// `if let (DialogResult::Ok, Some(f)) = Ui::show_dialog_for(owner, f) { f.answer }`.
    pub fn show_dialog_for<E: TrayWindowEvents + 'static>(owner: HWND, events: E) -> (phai_ui::form::DialogResult, Option<E>) {
        let hwnd = Self::make(Some(owner), events);
        Self::first_shown(hwnd);
        phai_ui::form::want_back(hwnd);
        win::modal_loop(hwnd, owner);
        (phai_ui::form::take_result(hwnd), phai_ui::form::take_kept::<E>(hwnd))
    }

    fn make<E: TrayWindowEvents + 'static>(owner: Option<HWND>, events: E) -> HWND {
        win::init();
        if let Ok(sheet) = theme::Theme::parse(include_str!("../theme.css")) {
            theme::set(sheet);
        }
        let class = win::register("Phai.TrayWindow", Some(proc_::<E>), None);
        let title = wide(t("Phai Tray"));
        let (style, ex) = phai_ui::form::frame("fixed_single", false, true, true, false);
        let main = owner.is_none();
        let over = owner;
        unsafe {
            // Inside an MDI container when show_mdi_child asked; a window of its own otherwise.
            let mdi_frame = phai_ui::mdi::take_next();
            let hwnd = match mdi_frame {
                Some(frame) => phai_ui::mdi::create_child(frame, &class, &title, style),
                None => wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, owner, None, Some(win::instance()), None).unwrap_or_default(),
            };
            let ui = Self::build(hwnd);
            let mut host = Box::new(Host { ui, events, main });
            host.events.loaded(&host.ui);
            win::set_state(hwnd, host);
            win::theme_frame(hwnd);
            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
            let ex = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 360), bottom: win::px(hwnd, 140) };
            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(ex), win::dpi(hwnd) as u32);
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
            }
            // A debug build follows the theme file as it is saved.
            #[cfg(debug_assertions)]
            win::watch_theme(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/theme.css")));
            // F12 in a debug build: the Live Inspector, which saves into the form.
            #[cfg(debug_assertions)]
            phai_ui::inspect::attach(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/tray_window.phf")));
            hwnd
        }
    }

    fn build(hwnd: HWND) -> TrayWindowUi {
        let font = win::font(hwnd, 0, false);
        let dpi = win::dpi(hwnd);
        #[allow(unused_mut)]
        let mut layout = Layout::new((360, 140));
        let about = win::label(hwnd, t("Phai Tray lives in the corner of the taskbar. Right-click its icon for its menu."), CONTROL_ABOUT, font);
        win::set_css(about, "", "about");
        layout.add(about, Item { bounds: Rect::new(12, 12, 336, 60), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let hide_button = win::button(hwnd, t("Hide"), CONTROL_HIDE_BUTTON, font, false);
        win::set_css(hide_button, "", "hide_button");
        layout.add(hide_button, Item { bounds: Rect::new(260, 100, 88, 28), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let tray_menu = hwnd;
        let tray = phai_ui::ctl::NotifyIcon::new(hwnd, CONTROL_TRAY as u32, t("Phai Tray"), true);
        TrayWindowUi {
            hwnd,
            font,
            about: phai_ui::ctl::Label(about),
            hide_button: phai_ui::ctl::Button(hide_button),
            tray_menu: phai_ui::ctl::ContextMenu(tray_menu),
            tray,
            layout,
        }
    }

    /// Closes the window, as the close box would - its FormClosing first.
    pub fn close(&self) {
        unsafe {
            let _ = wm::PostMessageW(Some(self.hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    /// Out of sight, still running; [`Self::show_window`] brings it back.
    pub fn hide(&self) {
        win::show(self.hwnd, false);
    }

    /// Shown: the first time as the form says (maximized or not), its Shown raised; again
    /// after [`Self::hide`]. The form starts hidden (`start_hidden`).
    pub fn show_window(&self) {
        Self::first_shown(self.hwnd);
        win::show(self.hwnd, true);
    }

    /// Its first showing, as the form says, then its Shown; later ones do nothing here.
    fn first_shown(hwnd: HWND) {
        if !phai_ui::form::first_show(hwnd) {
            return;
        }
        win::show_top(hwnd);
    }

    /// Context menu item `item` (its name in the form) shown, or left out the next time its menu opens - Visible. A separator
    /// left at an end of the menu, or beside another, goes too.
    pub fn set_menu_item_visible(&self, item: &str, on: bool) {
        phai_ui::popup::set_visible(self.hwnd, item, on)
    }

    /// Whether context menu item `item` is visible: as the program set it, or as the form has it.
    pub fn menu_item_visible(&self, item: &str) -> bool {
        phai_ui::popup::visible(self.hwnd, item, !matches!(item, "end_session"))
    }

    /// Context menu item `item` (its name in the form) ticked or not - Checked.
    pub fn set_menu_item_checked(&self, item: &str, on: bool) {
        phai_ui::popup::set_checked(self.hwnd, item, on)
    }

    /// Whether context menu item `item` is checked: as the program set it, or as the form has it.
    pub fn menu_item_checked(&self, item: &str) -> bool {
        phai_ui::popup::checked(self.hwnd, item, matches!(item, "english"))
    }

    /// Context menu item `item` (its name in the form) usable or greyed - Enabled.
    pub fn set_menu_item_enabled(&self, item: &str, on: bool) {
        phai_ui::popup::set_enabled(self.hwnd, item, on)
    }

    /// Whether context menu item `item` is enabled: as the program set it, or as the form has it.
    pub fn menu_item_enabled(&self, item: &str) -> bool {
        phai_ui::popup::enabled(self.hwnd, item, !false)
    }

    /// In front, taking the keys.
    pub fn activate(&self) {
        unsafe {
            let _ = wm::SetForegroundWindow(self.hwnd);
        }
    }

    /// Moved to the middle of the screen it is on.
    pub fn center_to_screen(&self) {
        win::center_on_screen(self.hwnd);
    }

    /// The words in its title bar.
    pub fn title(&self) -> String {
        win::text_of(self.hwnd)
    }

    /// New words in its title bar.
    pub fn set_title(&self, s: &str) {
        win::set_text(self.hwnd, s);
    }

    /// A handle other threads use to run work on this window's thread - [`Self::begin_invoke`].
    pub fn invoker(&self) -> phai_ui::invoke::Invoker {
        phai_ui::invoke::Invoker::new(self.hwnd)
    }

    /// From any thread: `f` runs on the window's thread with the program's TrayWindow and the
    /// controls, soon; returns at once - WinForms' BeginInvoke. `false` when the window is gone.
    pub fn begin_invoke<E: TrayWindowEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &TrayWindowUi) + Send + 'static) -> bool {
        inv.post(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<TrayWindowUi>()) {
                f(e, ui);
            }
        }))
    }

    /// As [`Self::begin_invoke`], waiting until `f` has run - WinForms' Invoke.
    pub fn invoke<E: TrayWindowEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &TrayWindowUi) + Send + 'static) {
        inv.send(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<TrayWindowUi>()) {
                f(e, ui);
            }
        }))
    }

    /// Every word again in the language now chosen - after `phai_ui::text::set_thai`.
    pub fn relabel(&self) {
        win::set_text(self.hwnd, t("Phai Tray"));
        win::set_text(self.about, t("Phai Tray lives in the corner of the taskbar. Right-click its icon for its menu."));
        win::set_text(self.hide_button, t("Hide"));
    }
}

extern "system" fn proc_<E: TrayWindowEvents + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
                ID_OPEN_WINDOW => events.open_window(ui),
                ID_PRETEND_SESSION => { phai_ui::popup::toggle(hwnd, "pretend_session", false); events.pretend_session(ui); }
                ID_END_SESSION => events.end_session(ui),
                ID_ENGLISH => events.english(ui),
                ID_THAI => events.thai(ui),
                ID_SHOW_NOTICE => events.show_notice(ui),
                ID_EXIT => events.exit(ui),
                CONTROL_HIDE_BUTTON if code == 0 => events.hide_clicked(ui),
                _ => {}
            }
            LRESULT(0)
        }
        phai_ui::components::WM_TRAY => {
            let Host { ui, events, .. } = host;
            let what = (l.0 & 0xFFFF) as u32;
            if w.0 == CONTROL_TRAY as usize {
                if what == wm::WM_LBUTTONDBLCLK { events.open_window(ui); }
                if what == phai_ui::components::NIN_BALLOONUSERCLICK { events.notice_clicked(ui); }
                if what == wm::WM_RBUTTONUP {
                    let mut p = windows::Win32::Foundation::POINT::default();
                    unsafe { let _ = wm::GetCursorPos(&mut p); let _ = wm::SetForegroundWindow(hwnd); }
                    let (x, y) = (p.x, p.y);
                    events.tray_menu_opening(ui);
                    phai_ui::popup::show(hwnd, x, y, &vec![phai_ui::popup::Item { name: "open_window", id: ID_OPEN_WINDOW, label: t("Open Phai Tray"), ..Default::default() }, phai_ui::popup::Item::separator(), phai_ui::popup::Item { name: "pretend_session", id: ID_PRETEND_SESSION, label: t("Pretend a remote session"), ..Default::default() }, phai_ui::popup::Item { name: "end_session", id: ID_END_SESSION, label: t("End remote session"), hidden: true, ..Default::default() }, phai_ui::popup::Item::separator(), phai_ui::popup::Item { label: t("Language"), items: vec![phai_ui::popup::Item { name: "english", id: ID_ENGLISH, label: t("English"), checked: true, ..Default::default() }, phai_ui::popup::Item { name: "thai", id: ID_THAI, label: t("Thai"), ..Default::default() }], ..Default::default() }, phai_ui::popup::Item { name: "show_notice", id: ID_SHOW_NOTICE, label: t("Show a notice"), ..Default::default() }, phai_ui::popup::Item::separator(), phai_ui::popup::Item { name: "exit", id: ID_EXIT, label: t("Exit"), ..Default::default() }]);
                }
            }
            LRESULT(0)
        }
        m if m == phai_ui::components::taskbar_created() => {
            phai_ui::components::add_all_again();
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            let e = phai_ui::events::ClosingEventArgs::default();
            host.events.window_closing(&host.ui, &e);
            if e.cancelled() {
                return LRESULT(0);
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
            host.ui.tray.remove();
            phai_ui::popup::forget(hwnd);
            phai_ui::form::forget_shown(hwnd);
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
