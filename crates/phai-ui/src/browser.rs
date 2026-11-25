//! A web page inside a window, as WinForms' WebView2 control: `kind = "browser"` in a form,
//! `url` the page it opens, `ui.<id>.navigate(...)` to open another.
//!
//! The page is Microsoft Edge WebView2. It is built only with this crate's `browser` feature,
//! which brings in Microsoft's WebView2 COM bindings (`webview2-com`); a program that does not
//! turn it on still compiles, and the control says in its place that the page cannot be shown.
//! The designer never opens a page: it shows the address in a frame ([`placeholder`]).
//!
//! WebView2 is loaded at run time, never at load time: `WebView2Loader.dll` is looked for beside
//! the program only (`LOAD_LIBRARY_SEARCH_APPLICATION_DIR`, so a copy planted elsewhere on the
//! search path is never picked up). A program whose loader or runtime is missing starts all the
//! same, and the control says why it shows nothing.
//!
//! The page starts once the window exists (a message the control posts to itself), never while
//! the form is still being built: making WebView2 pumps messages, and a form half made would get
//! them. Its data (cookies, cache) is kept under `%LOCALAPPDATA%\<program>\WebView2`, because the
//! default - beside the program - is not writable in Program Files.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::text::t;
use crate::win;

/// Posted by the control to itself once it exists: start the page.
const WM_START: u32 = wm::WM_APP + 0x0B10;

struct Browser {
    /// The page asked for: opened when WebView2 is ready, or now if it already is.
    url: String,
    /// What the control says while there is no page: why, or the address in the designer.
    note: String,
    #[cfg(feature = "browser")]
    view: Option<engine::View>,
}

/// A browser control showing `url` (empty: a blank page).
pub fn browser(parent: HWND, id: i32, url: &str) -> HWND {
    let h = make(parent, id, url, String::new());
    unsafe {
        let _ = wm::PostMessageW(Some(h), WM_START, WPARAM(0), LPARAM(0));
    }
    h
}

/// What the designer draws for a browser control: its address in a frame, no page.
pub fn placeholder(parent: HWND, id: i32, url: &str) -> HWND {
    let note = if url.trim().is_empty() { t("Web page").to_string() } else { format!("{}: {}", t("Web page"), url.trim()) };
    make(parent, id, url, note)
}

fn make(parent: HWND, id: i32, url: &str, note: String) -> HWND {
    let class = win::register("PhaiBrowser", Some(proc_), None);
    let h = unsafe {
        wm::CreateWindowExW(
            Default::default(),
            PCWSTR(class.as_ptr()),
            PCWSTR::null(),
            wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS | wm::WS_CLIPCHILDREN | wm::WS_TABSTOP,
            0,
            0,
            10,
            10,
            Some(parent),
            Some(wm::HMENU(id as isize as *mut _)),
            Some(win::instance()),
            None,
        )
        .unwrap_or_default()
    };
    win::set_state(
        h,
        Box::new(Browser {
            url: url.trim().to_string(),
            note,
            #[cfg(feature = "browser")]
            view: None,
        }),
    );
    h
}

/// Opens `url`. Before the page is ready it is remembered and opened when it is.
pub fn navigate(h: impl Into<HWND>, url: &str) {
    let h: HWND = h.into();
    let Some(b) = (unsafe { win::state::<Browser>(h) }) else { return };
    b.url = url.trim().to_string();
    #[cfg(feature = "browser")]
    if let Some(v) = &b.view {
        v.navigate(&b.url);
    }
}

/// The address shown now: the page's own once it has one, else the one asked for.
pub fn url(h: impl Into<HWND>) -> String {
    let h: HWND = h.into();
    let Some(b) = (unsafe { win::state::<Browser>(h) }) else { return String::new() };
    #[cfg(feature = "browser")]
    if let Some(s) = b.view.as_ref().and_then(engine::View::source) {
        return s;
    }
    b.url.clone()
}

/// Loads the page again.
pub fn reload(h: impl Into<HWND>) {
    #[cfg(feature = "browser")]
    if let Some(v) = unsafe { win::state::<Browser>(h.into()) }.and_then(|b| b.view.as_ref()) {
        v.reload();
    }
    #[cfg(not(feature = "browser"))]
    let _ = h.into();
}

/// The page before this one, as the browser's Back button.
pub fn go_back(h: impl Into<HWND>) {
    #[cfg(feature = "browser")]
    if let Some(v) = unsafe { win::state::<Browser>(h.into()) }.and_then(|b| b.view.as_ref()) {
        v.go_back();
    }
    #[cfg(not(feature = "browser"))]
    let _ = h.into();
}

/// Whether a page is showing (WebView2 started).
pub fn ready(h: impl Into<HWND>) -> bool {
    #[cfg(feature = "browser")]
    {
        unsafe { win::state::<Browser>(h.into()) }.is_some_and(|b| b.view.is_some())
    }
    #[cfg(not(feature = "browser"))]
    {
        let _ = h.into();
        false
    }
}

fn start(h: HWND) {
    let Some(b) = (unsafe { win::state::<Browser>(h) }) else { return };
    #[cfg(feature = "browser")]
    {
        match engine::View::new(h, &b.url) {
            Ok(v) => {
                b.view = Some(v);
                fit(h);
            }
            Err(why) => b.note = why,
        }
    }
    #[cfg(not(feature = "browser"))]
    {
        b.note = t("This program was built without phai-ui's browser feature, so the page cannot be shown.").to_string();
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, true);
    }
}

fn fit(h: HWND) {
    #[cfg(feature = "browser")]
    if let Some(v) = unsafe { win::state::<Browser>(h) }.and_then(|b| b.view.as_ref()) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        v.set_bounds(r);
    }
    #[cfg(not(feature = "browser"))]
    let _ = h;
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_START => {
            start(h);
            LRESULT(0)
        }
        wm::WM_SIZE => {
            fit(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            win::fill(dc, &r, win::window_background_for(h));
            let note = win::state::<Browser>(h).map(|b| b.note.clone()).unwrap_or_default();
            if !note.is_empty() {
                // A frame, so the control can be seen and placed, and the note in the middle.
                let th = crate::theme::current();
                let line = th.var_color("--border").unwrap_or(crate::theme::Color::rgb(0xc8, 0xc8, 0xc8));
                win::fill(dc, &RECT { left: r.left, top: r.top, right: r.right, bottom: r.top + 1 }, line);
                win::fill(dc, &RECT { left: r.left, top: r.bottom - 1, right: r.right, bottom: r.bottom }, line);
                win::fill(dc, &RECT { left: r.left, top: r.top, right: r.left + 1, bottom: r.bottom }, line);
                win::fill(dc, &RECT { left: r.right - 1, top: r.top, right: r.right, bottom: r.bottom }, line);
                let muted = th.var_color("--muted").unwrap_or(crate::theme::Color::rgb(0x5f, 0x5f, 0x5f));
                let font = win::font(h, 9, false);
                let old = Gdi::SelectObject(dc, font.into());
                Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.r as u32 | (muted.g as u32) << 8 | (muted.b as u32) << 16));
                let mut text: Vec<u16> = note.encode_utf16().collect();
                let mut box_ = RECT { left: r.left + 12, top: r.top + 12, right: r.right - 12, bottom: r.bottom - 12 };
                let _ = Gdi::DrawTextW(dc, &mut text, &mut box_, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(dc, old);
            }
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_NCDESTROY => {
            // Dropping the state closes WebView2 (engine::View's Drop).
            drop(unsafe { win::take_state::<Browser>(h) });
            unsafe { wm::DefWindowProcW(h, msg, w, l) }
        }
        _ => unsafe { wm::DefWindowProcW(h, msg, w, l) },
    }
}

#[cfg(feature = "browser")]
mod engine {
    use std::path::PathBuf;
    use std::sync::mpsc;

    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2Controller, ICoreWebView2Environment};
    use webview2_com::{CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler};
    use windows::core::{Interface, HRESULT, PCWSTR, PWSTR};
    use windows::Win32::Foundation::{E_POINTER, HWND, RECT};
    use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED};
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_APPLICATION_DIR};

    use crate::text::t;

    pub(super) struct View {
        controller: ICoreWebView2Controller,
        webview: ICoreWebView2,
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// `%LOCALAPPDATA%\<program>\WebView2`: the default, beside the program, is not writable in
    /// Program Files, and WebView2 then fails without a word.
    fn data_folder() -> PathBuf {
        let program = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "PhaiApp".to_string());
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join(program).join("WebView2")
    }

    type CreateWithOptions = unsafe extern "system" fn(PCWSTR, PCWSTR, *mut core::ffi::c_void, *mut core::ffi::c_void) -> HRESULT;

    fn environment() -> Result<ICoreWebView2Environment, String> {
        let missing = || t("The page cannot be shown: WebView2Loader.dll is not beside the program.").to_string();
        let name = wide("WebView2Loader.dll");
        let module = unsafe { LoadLibraryExW(PCWSTR(name.as_ptr()), None, LOAD_LIBRARY_SEARCH_APPLICATION_DIR) }.map_err(|_| missing())?;
        let proc = unsafe { GetProcAddress(module, windows::core::s!("CreateCoreWebView2EnvironmentWithOptions")) }.ok_or_else(missing)?;
        // The documented signature of the documented export.
        let create: CreateWithOptions = unsafe { std::mem::transmute(proc) };
        let folder = data_folder();
        let _ = std::fs::create_dir_all(&folder);
        let folder = wide(&folder.to_string_lossy());

        let (tx, rx) = mpsc::channel();
        CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
            Box::new(move |handler| {
                let hr = unsafe { create(PCWSTR::null(), PCWSTR(folder.as_ptr()), std::ptr::null_mut(), handler.as_raw()) };
                hr.ok().map_err(webview2_com::Error::WindowsError)
            }),
            Box::new(move |error, environment| {
                error?;
                let _ = tx.send(environment.ok_or_else(|| windows::core::Error::from(E_POINTER)));
                Ok(())
            }),
        )
        .map_err(|_| no_runtime())?;
        rx.recv().map_err(|_| no_runtime())?.map_err(|_| no_runtime())
    }

    fn no_runtime() -> String {
        t("The page cannot be shown: Microsoft Edge WebView2 Runtime is not installed on this computer.").to_string()
    }

    impl View {
        pub(super) fn new(host: HWND, url: &str) -> Result<View, String> {
            // WebView2 wants a single-threaded apartment; one the program made already is fine.
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            }
            let environment = environment()?;
            let (tx, rx) = mpsc::channel();
            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe { environment.CreateCoreWebView2Controller(host, &handler).map_err(webview2_com::Error::WindowsError) }),
                Box::new(move |error, controller| {
                    error?;
                    let _ = tx.send(controller.ok_or_else(|| windows::core::Error::from(E_POINTER)));
                    Ok(())
                }),
            )
            .map_err(|_| no_runtime())?;
            let controller = rx.recv().map_err(|_| no_runtime())?.map_err(|_| no_runtime())?;
            let webview = unsafe { controller.CoreWebView2() }.map_err(|_| no_runtime())?;
            unsafe {
                let _ = controller.SetIsVisible(true);
            }
            let view = View { controller, webview };
            view.navigate(url);
            Ok(view)
        }

        pub(super) fn navigate(&self, url: &str) {
            let target = if url.is_empty() { "about:blank" } else { url };
            let w = wide(target);
            unsafe {
                let _ = self.webview.Navigate(PCWSTR(w.as_ptr()));
            }
        }

        pub(super) fn source(&self) -> Option<String> {
            let mut raw = PWSTR::null();
            unsafe {
                self.webview.Source(&mut raw).ok()?;
                if raw.is_null() {
                    return None;
                }
                let s = raw.to_string().ok();
                CoTaskMemFree(Some(raw.0 as *const _));
                s
            }
        }

        pub(super) fn reload(&self) {
            unsafe {
                let _ = self.webview.Reload();
            }
        }

        pub(super) fn go_back(&self) {
            unsafe {
                let _ = self.webview.GoBack();
            }
        }

        pub(super) fn set_bounds(&self, r: RECT) {
            unsafe {
                let _ = self.controller.SetBounds(r);
            }
        }
    }

    impl Drop for View {
        fn drop(&mut self) {
            unsafe {
                let _ = self.controller.Close();
            }
        }
    }

}
