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
    "My first window",
    "Hello!",
    "What is your name?",
    "Open a note",
];

const ID_ASK_NAME_CLICKED: i32 = 1000;
const ID_OPEN_NOTE_CLICKED: i32 = 1001;
const ID_TOOLBAR: i32 = 1999;
const CONTROL_GREETING: i32 = 2000;
const CONTROL_ASK_NAME: i32 = 2001;
const CONTROL_OPEN_NOTE: i32 = 2002;

/// The window and every control in it, by the ids the form gives them.
pub struct MainWindowUi {
    pub hwnd: HWND,
    pub font: HFONT,
    pub greeting: phai_ui::ctl::Label,
    pub ask_name: phai_ui::ctl::Button,
    pub open_note: phai_ui::ctl::Button,
    layout: Layout,
}

/// What the window tells the program. Every method does nothing until the program says otherwise.
pub trait MainWindowEvents {
    /// The window is made and about to show.
    fn loaded(&mut self, ui: &MainWindowUi) {}
    fn ask_name_clicked(&mut self, ui: &MainWindowUi) {}
    fn open_note_clicked(&mut self, ui: &MainWindowUi) {}
    fn main_window_closing(&mut self, ui: &MainWindowUi, e: &phai_ui::events::ClosingEventArgs) {}
    fn main_window_key_down(&mut self, ui: &MainWindowUi, e: &phai_ui::events::KeyEventArgs) {}
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
        let class = win::register("Phai.MainWindow", Some(proc_::<E>), None);
        let title = wide(t("My first window"));
        let (style, ex) = phai_ui::form::frame("sizable", true, true, true, false);
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
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 420), bottom: win::px(hwnd, 220) };
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
            } else {
                win::show_top(hwnd);
            }
            // F12 in a debug build: the Live Inspector, which saves into the form.
            #[cfg(debug_assertions)]
            phai_ui::inspect::attach(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main_window.phaiform")));
            hwnd
        }
    }

    fn build(hwnd: HWND) -> MainWindowUi {
        let font = win::font(hwnd, 0, false);
        let dpi = win::dpi(hwnd);
        #[allow(unused_mut)]
        let mut layout = Layout::new((420, 220));
        let greeting = win::label(hwnd, t("Hello!"), CONTROL_GREETING, font);
        win::set_css(greeting, "", "greeting");
        layout.add(greeting, Item { bounds: Rect::new(16, 16, 388, 40), dock: Dock::None, anchor: Anchor { left: true, top: true, right: true, bottom: false } });
        let ask_name = win::button(hwnd, t("What is your name?"), CONTROL_ASK_NAME, font, false);
        win::set_css(ask_name, "", "ask_name");
        layout.add(ask_name, Item { bounds: Rect::new(16, 72, 180, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let open_note = win::button(hwnd, t("Open a note"), CONTROL_OPEN_NOTE, font, false);
        win::set_css(open_note, "", "open_note");
        layout.add(open_note, Item { bounds: Rect::new(208, 72, 180, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        phai_ui::events::hook(ask_name, hwnd, 2001, 0, true);
        phai_ui::events::hook(open_note, hwnd, 2002, 0, true);
        MainWindowUi {
            hwnd,
            font,
            greeting: phai_ui::ctl::Label(greeting),
            ask_name: phai_ui::ctl::Button(ask_name),
            open_note: phai_ui::ctl::Button(open_note),
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

    /// Shown again after [`Self::hide`].
    pub fn show_window(&self) {
        win::show(self.hwnd, true);
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
        win::set_text(self.hwnd, t("My first window"));
        win::set_text(self.greeting, t("Hello!"));
        win::set_text(self.ask_name, t("What is your name?"));
        win::set_text(self.open_note, t("Open a note"));
    }
}

extern "system" fn proc_<E: MainWindowEvents + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
                CONTROL_ASK_NAME if code == 0 => events.ask_name_clicked(ui),
                CONTROL_OPEN_NOTE if code == 0 => events.open_note_clicked(ui),
                _ => {}
            }
            LRESULT(0)
        }
        phai_ui::events::WM_EVENT => {
            let Host { ui, events, .. } = host;
            match phai_ui::events::decode(w) {
                (0, phai_ui::events::KEY_DOWN) => events.main_window_key_down(ui, unsafe { phai_ui::events::args::<phai_ui::events::KeyEventArgs>(l) }),
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            let e = phai_ui::events::ClosingEventArgs::default();
            host.events.main_window_closing(&host.ui, &e);
            if e.cancelled() {
                return LRESULT(0);
            }
            win::def(hwnd, msg, w, l)
        }
        wm::WM_KEYDOWN | wm::WM_SYSKEYDOWN => {
            let e = phai_ui::events::key_of(w);
            host.events.main_window_key_down(&host.ui, &e);
            if msg == wm::WM_SYSKEYDOWN && !e.handled() { win::def(hwnd, msg, w, l) } else { LRESULT(0) }
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
