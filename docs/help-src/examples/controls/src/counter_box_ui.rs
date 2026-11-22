// Made by Phai Studio from counter_box.phaiform - do not change this file; change the form.
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
    "CounterBox",
    "-",
    "0",
    "+",
];

const ID_MINUS_CLICKED: i32 = 1000;
const ID_PLUS_CLICKED: i32 = 1001;
const ID_TOOLBAR: i32 = 1999;
const CONTROL_MINUS: i32 = 2000;
const CONTROL_VALUE: i32 = 2001;
const CONTROL_PLUS: i32 = 2002;

/// The window and every control in it, by the ids the form gives them.
pub struct CounterBoxUi {
    pub hwnd: HWND,
    pub font: HFONT,
    pub minus: phai_ui::ctl::Button,
    pub value: phai_ui::ctl::Label,
    pub plus: phai_ui::ctl::Button,
    layout: Layout,
}

/// What the window tells the program. Every method does nothing until the program says otherwise.
pub trait CounterBoxEvents {
    /// The window is made and about to show.
    fn loaded(&mut self, ui: &CounterBoxUi) {}
    fn minus_clicked(&mut self, ui: &CounterBoxUi) {}
    fn plus_clicked(&mut self, ui: &CounterBoxUi) {}
}

thread_local! {
    /// Each CounterBox on a form: its window and its controls, for the form's code ([`CounterBoxUi::with`]).
    static PLACED: std::cell::RefCell<std::collections::HashMap<isize, *const CounterBoxUi>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

struct Host<E> {
    ui: CounterBoxUi,
    events: E,
    /// The program's main window: the program ends with it.
    main: bool,
}

impl CounterBoxUi {
    /// Makes the window and runs it until it is closed.
    pub fn run<E: CounterBoxEvents + 'static>(events: E) {
        let hwnd = Self::create(events);
        win::run_loop(Some(hwnd));
    }

    /// Makes the window and shows it; the caller runs the message loop.
    pub fn create<E: CounterBoxEvents + 'static>(events: E) -> HWND {
        Self::make(None, events)
    }

    /// Opens this window inside `parent`, an MDI container (`is_mdi_container`), as WinForms'
    /// `MdiParent` and `Show()`; it closes with it.
    pub fn show_mdi_child<E: CounterBoxEvents + 'static>(parent: HWND, events: E) -> HWND {
        phai_ui::mdi::set_next_parent(parent);
        Self::make(Some(parent), events)
    }

    /// Shows this window beside `owner` (another window of the program) and returns at once,
    /// as `Show()` in WinForms. Closing it leaves the program running.
    pub fn show<E: CounterBoxEvents + 'static>(owner: HWND, events: E) -> HWND {
        Self::make(Some(owner), events)
    }

    /// Shows this window over `owner` and waits until it is closed, `owner` taking no input
    /// meanwhile - as `ShowDialog()` in WinForms. What it answers: the DialogResult of the
    /// button that closed it, or `Cancel`.
    pub fn show_dialog<E: CounterBoxEvents + 'static>(owner: HWND, events: E) -> phai_ui::form::DialogResult {
        let hwnd = Self::make(Some(owner), events);
        win::modal_loop(hwnd, owner);
        phai_ui::form::take_result(hwnd)
    }

    /// As [`Self::show_dialog`], `events` given back when the window has closed - to read what
    /// was typed in it, as a WinForms form's fields after `ShowDialog()`:
    /// `if let (DialogResult::Ok, Some(f)) = Ui::show_dialog_for(owner, f) { f.answer }`.
    pub fn show_dialog_for<E: CounterBoxEvents + 'static>(owner: HWND, events: E) -> (phai_ui::form::DialogResult, Option<E>) {
        let hwnd = Self::make(Some(owner), events);
        phai_ui::form::want_back(hwnd);
        win::modal_loop(hwnd, owner);
        (phai_ui::form::take_result(hwnd), phai_ui::form::take_kept::<E>(hwnd))
    }

    /// Puts this user control on `parent` - a form, or a panel on it - as control `id`, at its
    /// own size; the form places it. Its controls answer to `events`, its own code.
    pub fn create_in<E: CounterBoxEvents + 'static>(parent: HWND, id: i32, events: E) -> HWND {
        win::init();
        let class = win::register("Phai.CounterBox", Some(proc_::<E>), None);
        unsafe {
            let style = wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS | wm::WS_TABSTOP;
            let hwnd = wm::CreateWindowExW(wm::WS_EX_CONTROLPARENT, PCWSTR(class.as_ptr()), PCWSTR::null(), style, 0, 0, win::px(parent, 200), win::px(parent, 40), Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
            let ui = Self::build(hwnd);
            let host = Box::new(Host { ui, events, main: false });
            let at: *const CounterBoxUi = &host.ui;
            PLACED.with(|m| m.borrow_mut().insert(hwnd.0 as isize, at));
            let mut host = host;
            // Its first WM_SIZE came before it had its controls: laid out now.
            host.ui.layout.apply(hwnd);
            host.events.loaded(&host.ui);
            win::set_state(hwnd, host);
            hwnd
        }
    }

    /// The controls of the CounterBox that is window `h`, for the form it is on: `f` given them.
    /// None when `h` is not one.
    pub fn with<R>(h: impl Into<HWND>, f: impl FnOnce(&CounterBoxUi) -> R) -> Option<R> {
        let h: HWND = h.into();
        let at = PLACED.with(|m| m.borrow().get(&(h.0 as isize)).copied())?;
        // Its window's state holds it until the window is gone, and it leaves the map then.
        Some(f(unsafe { &*at }))
    }

    /// Tells the form it is on that it changed: the form's `on_change` for it runs - as a
    /// UserControl raising its own event.
    pub fn notify_parent(&self) {
        unsafe {
            let id = wm::GetDlgCtrlID(self.hwnd) as usize & 0xFFFF;
            if let Ok(parent) = wm::GetParent(self.hwnd) {
                wm::SendMessageW(parent, wm::WM_COMMAND, Some(WPARAM(id | (0x0300 << 16))), Some(LPARAM(self.hwnd.0 as isize)));
            }
        }
    }

    fn make<E: CounterBoxEvents + 'static>(owner: Option<HWND>, events: E) -> HWND {
        win::init();
        let class = win::register("Phai.CounterBox", Some(proc_::<E>), None);
        let title = wide(t("CounterBox"));
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
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 200), bottom: win::px(hwnd, 40) };
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
            phai_ui::inspect::attach(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/counter_box.phaiform")));
            hwnd
        }
    }

    fn build(hwnd: HWND) -> CounterBoxUi {
        let font = win::font(hwnd, 0, false);
        let dpi = win::dpi(hwnd);
        #[allow(unused_mut)]
        let mut layout = Layout::new((200, 40));
        let minus = win::button(hwnd, t("-"), CONTROL_MINUS, font, false);
        win::set_css(minus, "", "minus");
        layout.add(minus, Item { bounds: Rect::new(0, 4, 40, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let value = win::label(hwnd, t("0"), CONTROL_VALUE, font);
        win::set_css(value, "", "value");
        phai_ui::props::text_align(value, "center");
        layout.add(value, Item { bounds: Rect::new(48, 8, 100, 24), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let plus = win::button(hwnd, t("+"), CONTROL_PLUS, font, false);
        win::set_css(plus, "", "plus");
        layout.add(plus, Item { bounds: Rect::new(156, 4, 40, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        CounterBoxUi {
            hwnd,
            font,
            minus: phai_ui::ctl::Button(minus),
            value: phai_ui::ctl::Label(value),
            plus: phai_ui::ctl::Button(plus),
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

    /// From any thread: `f` runs on the window's thread with the program's CounterBox and the
    /// controls, soon; returns at once - WinForms' BeginInvoke. `false` when the window is gone.
    pub fn begin_invoke<E: CounterBoxEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &CounterBoxUi) + Send + 'static) -> bool {
        inv.post(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<CounterBoxUi>()) {
                f(e, ui);
            }
        }))
    }

    /// As [`Self::begin_invoke`], waiting until `f` has run - WinForms' Invoke.
    pub fn invoke<E: CounterBoxEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &CounterBoxUi) + Send + 'static) {
        inv.send(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<CounterBoxUi>()) {
                f(e, ui);
            }
        }))
    }

    /// Every word again in the language now chosen - after `phai_ui::text::set_thai`.
    pub fn relabel(&self) {
        win::set_text(self.hwnd, t("CounterBox"));
        win::set_text(self.minus, t("-"));
        win::set_text(self.value, t("0"));
        win::set_text(self.plus, t("+"));
    }
}

extern "system" fn proc_<E: CounterBoxEvents + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
                CONTROL_MINUS if code == 0 => events.minus_clicked(ui),
                CONTROL_PLUS if code == 0 => events.plus_clicked(ui),
                _ => {}
            }
            LRESULT(0)
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
            PLACED.with(|m| m.borrow_mut().remove(&(hwnd.0 as isize)));
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
