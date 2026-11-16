// Made by Phai Studio from main_window.phaiform - do not change this file; change the form.
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
    pub details: HWND,
    pub name_label: HWND,
    pub name: HWND,
    pub active: HWND,
    pub kind: HWND,
    pub customers: HWND,
    pub save_button: HWND,
    pub close: HWND,
    pub status: HWND,
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
}

impl MainWindowUi {
    /// Makes the window and runs it until it is closed.
    pub fn run<E: MainWindowEvents + 'static>(events: E) {
        let hwnd = Self::create(events);
        win::run_loop(Some(hwnd));
    }

    /// Makes the window and shows it; the caller runs the message loop.
    pub fn create<E: MainWindowEvents + 'static>(events: E) -> HWND {
        win::init();
        if let Ok(sheet) = theme::Theme::parse(include_str!("../theme.css")) {
            theme::set(sheet);
        }
        let class = win::register("Phai.MainWindow", Some(proc_::<E>), None);
        let title = wide(t("Customers"));
        unsafe {
            let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, None, Some(Self::menu()), Some(win::instance()), None).unwrap_or_default();
            let ui = Self::build(hwnd);
            let mut host = Box::new(Host { ui, events });
            host.events.loaded(&host.ui);
            win::set_state(hwnd, host);
            win::theme_frame(hwnd);
            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 480), bottom: win::px(hwnd, 360) };
            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), true, Default::default(), win::dpi(hwnd) as u32);
            let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
            win::show_top(hwnd);
            // A debug build follows the theme file as it is saved.
            #[cfg(debug_assertions)]
            win::watch_theme(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/theme.css")));
            hwnd
        }
    }

    fn menu() -> wm::HMENU {
        unsafe {
            let bar = wm::CreateMenu().unwrap_or_default();
            let m = wm::CreatePopupMenu().unwrap_or_default();
            let s = wide(t("New customer"));
            let _ = wm::AppendMenuW(m, wm::MF_STRING, ID_NEW_CUSTOMER as usize, PCWSTR(s.as_ptr()));
            let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);
            let s = wide(t("Exit"));
            let _ = wm::AppendMenuW(m, wm::MF_STRING, ID_EXIT as usize, PCWSTR(s.as_ptr()));
            let s = wide(t("&File"));
            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m.0 as usize, PCWSTR(s.as_ptr()));
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
        let kind = win::combo(hwnd, CONTROL_KIND, font, &[t("Shop"), t("Office")]);
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
            details,
            name_label,
            name,
            active,
            kind,
            customers,
            save_button,
            close,
            status,
            layout,
        }
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
        let info = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
        let style = unsafe { wm::GetWindowLongW(hwnd, wm::GWL_STYLE) } as u32;
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 400), bottom: win::px(hwnd, 300) };
        let _ = unsafe { windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), true, Default::default(), win::dpi(hwnd) as u32) };
        info.ptMinTrackSize.x = r.right - r.left;
        info.ptMinTrackSize.y = r.bottom - r.top;
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
            let Host { ui, events } = host;
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
            if hdr.code == cc::LVN_ITEMCHANGED {
                let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                let chosen = it.uNewState & cc::LVIS_SELECTED.0 != 0 && it.uOldState & cc::LVIS_SELECTED.0 == 0;
                let Host { ui, events } = host;
                if chosen && hdr.idFrom == CONTROL_CUSTOMERS as usize {
                    events.customer_chosen(ui);
                }
            }
            win::def(hwnd, msg, w, l)
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
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(host) = unsafe { win::take_state::<Host<E>>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(host.ui.font.into());
                }
            }
            win::def(hwnd, msg, w, l)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}
