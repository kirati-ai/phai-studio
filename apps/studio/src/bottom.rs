//! The panel under the canvas: the error list, Cargo's output and the last Find All References,
//! one at a time, under tabs - Visual Studio's Error List, Output and Find Symbol Results.

use phai_ui::controls;
use phai_ui::layout::{Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

const ID_TABS: i32 = 500;
const ID_OUTPUT: i32 = 501;
pub const ID_REFS: i32 = 502;
pub const ID_LOCALS: i32 = 503;
pub const ID_STACK: i32 = 504;
pub const ID_RESULTS: i32 = 505;
pub const ID_WATCH: i32 = 506;
pub const ID_TESTS: i32 = 507;
/// The Error List's filters: errors shown, warnings shown.
pub const ID_SHOW_ERRORS: i32 = 520;
pub const ID_SHOW_WARNINGS: i32 = 521;
pub const ID_IMMEDIATE: i32 = 522;
pub const ID_IMMEDIATE_INPUT: i32 = 523;
pub const ID_DIAGNOSTICS: i32 = 524;
pub const ID_BREAKPOINTS: i32 = 525;
pub const ID_THREADS: i32 = 526;

/// The tabs, in order: 0 errors, 1 output, 2 references, 3 locals, 4 call stack, 5 query
/// results, 6 watch, 7 tests, 8 immediate, 9 diagnostic tools, 10 breakpoints, 11 threads.
fn labels() -> [&'static str; 12] {
    [t("Errors"), t("Output"), t("References"), t("Locals"), t("Call Stack"), t("Results"), t("Watch"), t("Tests"), t("Immediate"), t("Diagnostic Tools"), t("Breakpoints"), t("Threads")]
}

pub struct Bottom {
    pub hwnd: HWND,
    tabs: HWND,
    pub errors: HWND,
    output: HWND,
    pub refs: HWND,
    pub locals: HWND,
    pub stack: HWND,
    pub results: HWND,
    pub watch: HWND,
    pub tests: HWND,
    pub show_errors: HWND,
    pub show_warnings: HWND,
    /// The Immediate window: what was asked and answered, and the box to ask in.
    pub immediate: HWND,
    pub immediate_input: HWND,
    pub diagnostics: HWND,
    pub breakpoints: HWND,
    pub threads: HWND,
    layout: Layout,
}

impl Bottom {
    pub fn new(parent: HWND, font: HFONT, errors_id: i32) -> Box<Bottom> {
        let class = win::register("PhaiStudioBottom", Some(proc_), None);
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default()
        };
        let tabs = controls::tabs(hwnd, ID_TABS, font, &labels());
        let errors = win::list(hwnd, errors_id, font, &[(t("Kind"), 80), (t("Message"), 560), (t("File"), 170), (t("Line"), 60)]);
        let output = win::edit_lines(hwnd, "", ID_OUTPUT, font);
        let refs = win::list(hwnd, ID_REFS, font, &[(t("File"), 170), (t("Line"), 60), (t("Code"), 640)]);
        let locals = win::list(hwnd, ID_LOCALS, font, &[(t("Name"), 160), (t("Value"), 380), (t("Type"), 220)]);
        let stack = win::list(hwnd, ID_STACK, font, &[(t("Function"), 420), (t("File"), 200), (t("Line"), 60)]);
        let results = win::list(hwnd, ID_RESULTS, font, &[("", 100)]);
        let watch = win::list(hwnd, ID_WATCH, font, &[(t("Name"), 220), (t("Value"), 380), (t("Type"), 200)]);
        let tests = win::list(hwnd, ID_TESTS, font, &[(t("Test"), 380), (t("Outcome"), 110), (t("Message"), 480)]);
        unsafe {
            wm::SendMessageW(output, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        }
        let mut layout = Layout::new((800, 140));
        layout.add(tabs, Item { bounds: Rect::new(0, 0, 0, 28), dock: Dock::Top, anchor: Anchor::default() });
        layout.add(errors, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        layout.add(output, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(refs, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(locals, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(stack, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(results, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(watch, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(tests, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        // Errors (n) and Warnings (n), at the right of the tabs, as Visual Studio's Error List has them.
        let show_errors = win::checkbox(hwnd, t("Errors"), ID_SHOW_ERRORS, font, true);
        let show_warnings = win::checkbox(hwnd, t("Warnings"), ID_SHOW_WARNINGS, font, true);
        let right = Anchor { left: false, top: true, right: true, bottom: false };
        layout.add(show_errors, Item { bounds: Rect::new(560, 3, 110, 22), dock: Dock::None, anchor: right });
        layout.add(show_warnings, Item { bounds: Rect::new(676, 3, 120, 22), dock: Dock::None, anchor: right });
        unsafe {
            for c in [show_errors, show_warnings] {
                let _ = wm::SetWindowPos(c, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
            }
        }
        let immediate = win::edit_lines(hwnd, "", ID_IMMEDIATE, font);
        let immediate_input = win::edit(hwnd, "", ID_IMMEDIATE_INPUT, font, false);
        unsafe {
            wm::SendMessageW(immediate, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
            let _ = windows::Win32::UI::Shell::SetWindowSubclass(immediate_input, Some(input_proc), 31, 0);
        }
        layout.add(immediate, Item { bounds: Rect::new(0, 28, 800, 84), dock: Dock::None, anchor: Anchor::ALL });
        layout.add(immediate_input, Item { bounds: Rect::new(0, 114, 800, 24), dock: Dock::None, anchor: Anchor { left: true, top: false, right: true, bottom: true } });
        let diagnostics = crate::diag::view(hwnd, ID_DIAGNOSTICS);
        layout.add(diagnostics, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        let breakpoints = win::list(hwnd, ID_BREAKPOINTS, font, &[(t("Name"), 260), (t("Condition"), 220), (t("Hit Count"), 160), (t("File"), 380)]);
        unsafe {
            use windows::Win32::UI::Controls as cc;
            // A tick on each: whether it stops the program.
            wm::SendMessageW(breakpoints, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0x4)), Some(LPARAM(0x4 /* LVS_EX_CHECKBOXES */)));
        }
        layout.add(breakpoints, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        let threads = win::list(hwnd, ID_THREADS, font, &[("", 28), (t("ID"), 80), (t("Name"), 200), (t("Location"), 520)]);
        layout.add(threads, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        let b = Box::new(Bottom { hwnd, tabs, errors, output, refs, locals, stack, results, watch, tests, show_errors, show_warnings, immediate, immediate_input, diagnostics, breakpoints, threads, layout });
        unsafe {
            wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, &*b as *const Bottom as isize);
        }
        b.show_tab(0);
        b
    }

    /// 0 the errors, 1 the output, 2 the references, 3 the locals, 4 the call stack.
    pub fn show_tab(&self, i: usize) {
        controls::tabs_select(self.tabs, i);
        win::show(self.errors, i == 0);
        win::show(self.output, i == 1);
        win::show(self.refs, i == 2);
        win::show(self.locals, i == 3);
        win::show(self.stack, i == 4);
        win::show(self.results, i == 5);
        win::show(self.watch, i == 6);
        win::show(self.tests, i == 7);
        win::show(self.show_errors, i == 0);
        win::show(self.show_warnings, i == 0);
        win::show(self.immediate, i == 8);
        win::show(self.immediate_input, i == 8);
        win::show(self.diagnostics, i == 9);
        win::show(self.breakpoints, i == 10);
        win::show(self.threads, i == 11);
    }

    pub fn relabel(&self) {
        controls::tabs_relabel(self.tabs, &labels());
    }

    /// One more line at the end of the Immediate window.
    pub fn immediate_line(&self, line: &str) {
        let w = win::wide(&format!("{line}\r\n"));
        unsafe {
            let n = wm::GetWindowTextLengthW(self.immediate) as usize;
            wm::SendMessageW(self.immediate, 0x00B1 /* EM_SETSEL */, Some(WPARAM(n)), Some(LPARAM(n as isize)));
            wm::SendMessageW(self.immediate, 0x00C2 /* EM_REPLACESEL */, Some(WPARAM(0)), Some(LPARAM(w.as_ptr() as isize)));
        }
    }

    pub fn clear_output(&self) {
        win::set_text(self.output, "");
    }

    /// One more line at the end of the output, scrolled into view.
    pub fn add_line(&self, line: &str) {
        let w = win::wide(&format!("{line}\r\n"));
        unsafe {
            let n = wm::GetWindowTextLengthW(self.output) as usize;
            wm::SendMessageW(self.output, 0x00B1 /* EM_SETSEL */, Some(WPARAM(n)), Some(LPARAM(n as isize)));
            wm::SendMessageW(self.output, 0x00C2 /* EM_REPLACESEL */, Some(WPARAM(0)), Some(LPARAM(w.as_ptr() as isize)));
        }
    }
}

/// The Immediate window's box: Enter asks the debugger what was typed.
unsafe extern "system" fn input_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4 /* DLGC_WANTALLKEYS */),
        wm::WM_KEYDOWN if w.0 == 0x0D => {
            let text = win::text_of(h);
            win::set_text(h, "");
            if let Some(s) = crate::studio() {
                s.immediate(text.trim());
            }
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D => LRESULT(0),
        _ => unsafe { windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l) },
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let b = unsafe { (wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA) as *const Bottom).as_ref() };
    match (msg, b) {
        (wm::WM_SIZE, Some(b)) => {
            b.layout.apply(h);
            LRESULT(0)
        }
        (wm::WM_COMMAND, Some(b)) if win::loword(w.0) as i32 == ID_TABS => {
            b.show_tab(win::hiword(w.0) as usize);
            LRESULT(0)
        }
        // The filters are the main window's to apply.
        (wm::WM_COMMAND, _) if matches!(win::loword(w.0) as i32, ID_SHOW_ERRORS | ID_SHOW_WARNINGS) => {
            if let Some(s) = crate::studio() {
                s.fill_errors();
            }
            LRESULT(0)
        }
        // The error list's double-click is the main window's business.
        (wm::WM_NOTIFY, _) => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            // The Breakpoints window's ticks and its Delete key are the studio's to act on too.
            let from_breakpoints = unsafe { wm::GetDlgCtrlID(hdr.hwndFrom) } == ID_BREAKPOINTS && (hdr.code == cc::LVN_ITEMCHANGED || hdr.code == cc::LVN_KEYDOWN);
            if hdr.code == cc::NM_DBLCLK || from_breakpoints {
                if let Ok(p) = unsafe { wm::GetParent(h) } {
                    return unsafe { wm::SendMessageW(p, msg, Some(w), Some(l)) };
                }
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}
