//! The panel under the canvas: the error list and Cargo's output, one at a time, under two tabs -
//! Visual Studio's Error List and Output window.

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

pub struct Bottom {
    pub hwnd: HWND,
    tabs: HWND,
    pub errors: HWND,
    output: HWND,
    layout: Layout,
}

impl Bottom {
    pub fn new(parent: HWND, font: HFONT, errors_id: i32) -> Box<Bottom> {
        let class = win::register("PhaiStudioBottom", Some(proc_), None);
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default()
        };
        let tabs = controls::tabs(hwnd, ID_TABS, font, &[t("Errors"), t("Output")]);
        let errors = win::list(hwnd, errors_id, font, &[(t("Kind"), 80), (t("Message"), 560), (t("File"), 170), (t("Line"), 60)]);
        let output = win::edit_lines(hwnd, "", ID_OUTPUT, font);
        unsafe {
            wm::SendMessageW(output, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        }
        let mut layout = Layout::new((800, 140));
        layout.add(tabs, Item { bounds: Rect::new(0, 0, 0, 28), dock: Dock::Top, anchor: Anchor::default() });
        layout.add(errors, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        layout.add(output, Item { bounds: Rect::new(0, 28, 800, 112), dock: Dock::None, anchor: Anchor::ALL });
        let b = Box::new(Bottom { hwnd, tabs, errors, output, layout });
        unsafe {
            wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, &*b as *const Bottom as isize);
        }
        b.show_tab(0);
        b
    }

    /// 0 the errors, 1 the output.
    pub fn show_tab(&self, i: usize) {
        controls::tabs_select(self.tabs, i);
        win::show(self.errors, i == 0);
        win::show(self.output, i == 1);
    }

    pub fn relabel(&self) {
        controls::tabs_relabel(self.tabs, &[t("Errors"), t("Output")]);
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
