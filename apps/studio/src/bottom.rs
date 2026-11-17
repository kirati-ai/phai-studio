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

/// The tabs, in order: 0 errors, 1 output, 2 references, 3 locals, 4 call stack.
fn labels() -> [&'static str; 5] {
    [t("Errors"), t("Output"), t("References"), t("Locals"), t("Call Stack")]
}

pub struct Bottom {
    pub hwnd: HWND,
    tabs: HWND,
    pub errors: HWND,
    output: HWND,
    pub refs: HWND,
    pub locals: HWND,
    pub stack: HWND,
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
        let b = Box::new(Bottom { hwnd, tabs, errors, output, refs, locals, stack, layout });
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
    }

    pub fn relabel(&self) {
        controls::tabs_relabel(self.tabs, &labels());
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
        // The error list's double-click is the main window's business.
        (wm::WM_NOTIFY, _) => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.code == cc::NM_DBLCLK {
                if let Ok(p) = unsafe { wm::GetParent(h) } {
                    return unsafe { wm::SendMessageW(p, msg, Some(w), Some(l)) };
                }
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}
