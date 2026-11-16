//! The form as it will run: a window of its own built from the model, every control alive.
//! Esc or the close box ends it. Handlers are the program's, so here nothing answers a click.

use phai_form::{live, FormFile};
use phai_ui::text::t;
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging as wm;

struct Preview {
    live: live::Live,
    font: HFONT,
}

pub fn open(owner: HWND, f: &FormFile) {
    let class = win::register("PhaiStudioPreview", Some(proc_), None);
    let title = wide(&format!("{} - {}", t(Box::leak(f.form.title.clone().into_boxed_str())), t("Preview")));
    unsafe {
        let Ok(hwnd) = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None) else { return };
        let font = win::font(hwnd, 0, false);
        let built = live::build(hwnd, f, font, 0);
        win::set_state(hwnd, Box::new(Preview { live: built, font }));
        win::theme_frame(hwnd);
        let st = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, f.form.size[0]), bottom: win::px(hwnd, f.form.size[1]) };
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(st), false, Default::default(), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        win::show_top(hwnd);
    }
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(p) = (unsafe { win::state::<Preview>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            p.live.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => {
            unsafe {
                let _ = wm::DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(p) = unsafe { win::take_state::<Preview>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(p.font.into());
                }
            }
            win::def(hwnd, msg, w, l)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}
