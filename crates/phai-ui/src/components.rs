//! WinForms' components - what is put on a form without being a control in it: the file, folder,
//! colour and font dialogs, the ErrorProvider's marks beside controls, and the NotifyIcon in the
//! corner of the screen. Each is made by the generated code from the form's properties and is the
//! form's field: `ui.open_file.show_dialog()`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls::Dialogs as dlg;
use windows::Win32::UI::Shell::{self as shell, DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::theme::Color;
use crate::win::{self, wide};

/// WinForms' filter, `Text files|*.txt|All files|*.*`, as Windows wants it: pairs ended by nothing.
fn filter_of(filter: &str) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for part in filter.split('|') {
        out.extend(part.encode_utf16());
        out.push(0);
    }
    if filter.split('|').count() % 2 == 1 {
        out.extend("*.*".encode_utf16());
        out.push(0);
    }
    out.push(0);
    out
}

#[allow(clippy::too_many_arguments)]
fn file_dialog(owner: HWND, save: bool, title: &str, filter: &str, dir: &str, ext: &str, name: &str, multi: bool) -> Vec<PathBuf> {
    let filter = filter_of(if filter.is_empty() { "All files|*.*" } else { filter });
    let mut file = vec![0u16; 32 * 1024];
    for (i, u) in name.encode_utf16().take(1000).enumerate() {
        file[i] = u;
    }
    let (title, dir, ext) = (wide(title), wide(dir), wide(ext.trim_start_matches('.')));
    let mut flags = if save { dlg::OFN_OVERWRITEPROMPT | dlg::OFN_PATHMUSTEXIST } else { dlg::OFN_FILEMUSTEXIST | dlg::OFN_PATHMUSTEXIST };
    flags |= dlg::OFN_EXPLORER;
    if multi {
        flags |= dlg::OFN_ALLOWMULTISELECT;
    }
    let mut ofn = dlg::OPENFILENAMEW {
        lStructSize: std::mem::size_of::<dlg::OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        lpstrTitle: if title.len() > 1 { PCWSTR(title.as_ptr()) } else { PCWSTR::null() },
        lpstrInitialDir: if dir.len() > 1 { PCWSTR(dir.as_ptr()) } else { PCWSTR::null() },
        lpstrDefExt: if ext.len() > 1 { PCWSTR(ext.as_ptr()) } else { PCWSTR::null() },
        Flags: flags,
        ..Default::default()
    };
    let ok = unsafe { if save { dlg::GetSaveFileNameW(&mut ofn) } else { dlg::GetOpenFileNameW(&mut ofn) } }.as_bool();
    if !ok {
        return Vec::new();
    }
    // Several: the folder, then each name, each ended by nothing, the last by two.
    let parts: Vec<String> = file.split(|c| *c == 0).take_while(|p| !p.is_empty()).map(String::from_utf16_lossy).collect();
    match parts.as_slice() {
        [] => Vec::new(),
        [one] => vec![PathBuf::from(one)],
        [dir, names @ ..] => names.iter().map(|n| PathBuf::from(dir).join(n)).collect(),
    }
}

/// WinForms' OpenFileDialog.
#[derive(Clone, Debug, Default)]
pub struct OpenFileDialog {
    pub owner: HWND,
    pub title: String,
    /// `Text files|*.txt|All files|*.*`.
    pub filter: String,
    pub initial_directory: String,
    pub default_ext: String,
    pub multiselect: bool,
}

impl OpenFileDialog {
    pub fn new(owner: HWND, title: &str, filter: &str, initial_directory: &str, default_ext: &str, multiselect: bool) -> OpenFileDialog {
        OpenFileDialog { owner, title: title.into(), filter: filter.into(), initial_directory: initial_directory.into(), default_ext: default_ext.into(), multiselect }
    }
    /// The file chosen, or `None` for Cancel.
    pub fn show_dialog(&self) -> Option<PathBuf> {
        file_dialog(self.owner, false, &self.title, &self.filter, &self.initial_directory, &self.default_ext, "", false).into_iter().next()
    }
    /// Every file chosen (Multiselect); none for Cancel.
    pub fn show_dialog_multi(&self) -> Vec<PathBuf> {
        file_dialog(self.owner, false, &self.title, &self.filter, &self.initial_directory, &self.default_ext, "", self.multiselect)
    }
}

/// WinForms' SaveFileDialog: asks before writing over a file that is there.
#[derive(Clone, Debug, Default)]
pub struct SaveFileDialog {
    pub owner: HWND,
    pub title: String,
    pub filter: String,
    pub initial_directory: String,
    pub default_ext: String,
}

impl SaveFileDialog {
    pub fn new(owner: HWND, title: &str, filter: &str, initial_directory: &str, default_ext: &str) -> SaveFileDialog {
        SaveFileDialog { owner, title: title.into(), filter: filter.into(), initial_directory: initial_directory.into(), default_ext: default_ext.into() }
    }
    pub fn show_dialog(&self) -> Option<PathBuf> {
        self.show_dialog_named("")
    }
    /// With `name` (FileName) in its box to begin with.
    pub fn show_dialog_named(&self, name: &str) -> Option<PathBuf> {
        file_dialog(self.owner, true, &self.title, &self.filter, &self.initial_directory, &self.default_ext, name, false).into_iter().next()
    }
}

/// WinForms' FolderBrowserDialog: Windows' own folder chooser.
#[derive(Clone, Debug, Default)]
pub struct FolderBrowserDialog {
    pub owner: HWND,
    pub title: String,
    pub initial_directory: String,
}

impl FolderBrowserDialog {
    pub fn new(owner: HWND, title: &str, initial_directory: &str) -> FolderBrowserDialog {
        FolderBrowserDialog { owner, title: title.into(), initial_directory: initial_directory.into() }
    }
    pub fn show_dialog(&self) -> Option<PathBuf> {
        use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let d: shell::IFileOpenDialog = CoCreateInstance(&shell::FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
            let opts = d.GetOptions().ok()?;
            d.SetOptions(opts | shell::FOS_PICKFOLDERS | shell::FOS_FORCEFILESYSTEM).ok()?;
            if !self.title.is_empty() {
                let t = wide(&self.title);
                let _ = d.SetTitle(PCWSTR(t.as_ptr()));
            }
            if !self.initial_directory.is_empty() {
                let p = wide(&self.initial_directory);
                if let Ok(item) = shell::SHCreateItemFromParsingName::<_, _, shell::IShellItem>(PCWSTR(p.as_ptr()), None) {
                    let _ = d.SetFolder(&item);
                }
            }
            d.Show(Some(self.owner)).ok()?;
            let item = d.GetResult().ok()?;
            let name = item.GetDisplayName(shell::SIGDN_FILESYSPATH).ok()?;
            let s = name.to_string().ok();
            CoTaskMemFree(Some(name.0 as *const _));
            s.map(PathBuf::from)
        }
    }
}

/// WinForms' ColorDialog.
#[derive(Clone, Copy, Debug, Default)]
pub struct ColorDialog {
    pub owner: HWND,
}

/// Windows' colour dialog, starting at `start` (any CSS colour); the colour chosen as `#rrggbb`.
/// It opens beside the mouse - where the click that asked for it was - kept on the screen.
pub fn choose_colour(owner: HWND, start: &str) -> Option<String> {
    static mut CUSTOM: [COLORREF; 16] = [COLORREF(0x00FF_FFFF); 16];
    unsafe extern "system" fn hook(h: HWND, msg: u32, _: WPARAM, _: LPARAM) -> usize {
        if msg == wm::WM_INITDIALOG {
            unsafe {
                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = wm::GetCursorPos(&mut pt);
                let mut r = RECT::default();
                let _ = wm::GetWindowRect(h, &mut r);
                let (w, ht) = (r.right - r.left, r.bottom - r.top);
                // The screen the mouse is on, the taskbar left out.
                let mon = Gdi::MonitorFromPoint(pt, Gdi::MONITOR_DEFAULTTONEAREST);
                let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
                let a = if Gdi::GetMonitorInfoW(mon, &mut mi).as_bool() { mi.rcWork } else { RECT { left: 0, top: 0, right: 1920, bottom: 1080 } };
                // Below and left of the mouse, as a drop-down would open; moved in when it would leave the screen.
                let mut x = pt.x - w + win::px(h, 24);
                let mut y = pt.y + win::px(h, 12);
                if y + ht > a.bottom {
                    y = pt.y - ht - win::px(h, 12);
                }
                x = x.clamp(a.left, (a.right - w).max(a.left));
                y = y.clamp(a.top, (a.bottom - ht).max(a.top));
                let _ = wm::SetWindowPos(h, None, x, y, 0, 0, wm::SWP_NOSIZE | wm::SWP_NOZORDER);
            }
        }
        0
    }
    let first = crate::theme::parse_color(start).map(|c| c.colorref()).unwrap_or(0x00FF_FFFF);
    let mut cc = dlg::CHOOSECOLORW {
        lStructSize: std::mem::size_of::<dlg::CHOOSECOLORW>() as u32,
        hwndOwner: owner,
        rgbResult: COLORREF(first),
        lpCustColors: &raw mut CUSTOM as *mut _,
        Flags: dlg::CC_FULLOPEN | dlg::CC_RGBINIT | dlg::CC_ENABLEHOOK,
        lpfnHook: Some(hook),
        ..Default::default()
    };
    if !unsafe { dlg::ChooseColorW(&mut cc) }.as_bool() {
        return None;
    }
    let v = cc.rgbResult.0;
    Some(format!("#{:02x}{:02x}{:02x}", v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF))
}

impl ColorDialog {
    pub fn new(owner: HWND) -> ColorDialog {
        ColorDialog { owner }
    }
    /// The colour chosen, as `#rrggbb`, starting at `start` (any CSS colour, or empty).
    pub fn show_dialog(&self, start: &str) -> Option<String> {
        static mut CUSTOM: [COLORREF; 16] = [COLORREF(0x00FF_FFFF); 16];
        let first = crate::theme::parse_color(start).map(|c| c.colorref()).unwrap_or(0);
        let mut cc = dlg::CHOOSECOLORW { lStructSize: std::mem::size_of::<dlg::CHOOSECOLORW>() as u32, hwndOwner: self.owner, rgbResult: COLORREF(first), lpCustColors: &raw mut CUSTOM as *mut _, Flags: dlg::CC_FULLOPEN | dlg::CC_RGBINIT, ..Default::default() };
        if !unsafe { dlg::ChooseColorW(&mut cc) }.as_bool() {
            return None;
        }
        let v = cc.rgbResult.0;
        Some(format!("#{:02x}{:02x}{:02x}", v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF))
    }
}

/// A font as the FontDialog answers it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChosenFont {
    pub face: String,
    pub points: i32,
    pub bold: bool,
    pub italic: bool,
}

/// WinForms' FontDialog.
#[derive(Clone, Copy, Debug, Default)]
pub struct FontDialog {
    pub owner: HWND,
}

impl FontDialog {
    pub fn new(owner: HWND) -> FontDialog {
        FontDialog { owner }
    }
    pub fn show_dialog(&self) -> Option<ChosenFont> {
        let mut lf = Gdi::LOGFONTW::default();
        let mut cf = dlg::CHOOSEFONTW { lStructSize: std::mem::size_of::<dlg::CHOOSEFONTW>() as u32, hwndOwner: self.owner, lpLogFont: &mut lf, Flags: dlg::CF_SCREENFONTS | dlg::CF_EFFECTS, ..Default::default() };
        if !unsafe { dlg::ChooseFontW(&mut cf) }.as_bool() {
            return None;
        }
        let n = lf.lfFaceName.iter().position(|c| *c == 0).unwrap_or(lf.lfFaceName.len());
        Some(ChosenFont { face: String::from_utf16_lossy(&lf.lfFaceName[..n]), points: cf.iPointSize / 10, bold: lf.lfWeight >= 600, italic: lf.lfItalic != 0 })
    }
}

// --- ErrorProvider -------------------------------------------------------------------------------

thread_local! {
    /// Each control with an error: its mark's window and the words.
    static MARKS: RefCell<HashMap<isize, (isize, String)>> = RefCell::new(HashMap::new());
}

/// WinForms' ErrorProvider: a red mark beside a control whose value is wrong, the reason in its
/// tooltip. The mark follows the control as the window is sized.
#[derive(Clone, Copy, Debug, Default)]
pub struct ErrorProvider {
    pub owner: HWND,
}

const MARK: i32 = 16;

fn place_mark(control: HWND, mark: HWND) {
    let r = win::rect_in_parent(control);
    let s = win::px(control, MARK);
    let visible = unsafe { wm::IsWindowVisible(control) }.as_bool();
    unsafe {
        let _ = wm::SetWindowPos(mark, Some(wm::HWND_TOP), r.right + win::px(control, 4), r.top + (r.bottom - r.top - s) / 2, s, s, wm::SWP_NOACTIVATE | if visible { wm::SWP_SHOWWINDOW } else { wm::SWP_HIDEWINDOW });
    }
}

unsafe extern "system" fn watch_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    let r = unsafe { DefSubclassProc(h, msg, w, l) };
    if matches!(msg, wm::WM_WINDOWPOSCHANGED | wm::WM_SHOWWINDOW) {
        if let Some(m) = MARKS.with(|m| m.borrow().get(&(h.0 as isize)).map(|(m, _)| *m)) {
            place_mark(h, HWND(m as *mut _));
        }
    }
    r
}

extern "system" fn mark_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            win::paint_behind(h, dc, &r);
            let red = Color::rgb(0xc4, 0x2b, 0x1c);
            let brush = Gdi::CreateSolidBrush(COLORREF(red.colorref()));
            let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, COLORREF(red.colorref()));
            let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
            let _ = Gdi::Ellipse(dc, r.left, r.top, r.right, r.bottom);
            Gdi::SelectObject(dc, ob);
            Gdi::SelectObject(dc, op);
            let _ = Gdi::DeleteObject(brush.into());
            let _ = Gdi::DeleteObject(pen.into());
            let f = win::font(h, 8, true);
            let old = Gdi::SelectObject(dc, f.into());
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, COLORREF(0x00FF_FFFF));
            let mut bang = wide("!");
            Gdi::DrawTextW(dc, &mut bang[..1], &mut r, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteObject(f.into());
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}

impl ErrorProvider {
    pub fn new(owner: HWND) -> ErrorProvider {
        ErrorProvider { owner }
    }

    /// Marks `control` with `message` - or takes its mark away for an empty one (SetError).
    pub fn set_error(&self, control: impl Into<HWND>, message: &str) {
        let control: HWND = control.into();
        let key = control.0 as isize;
        let had = MARKS.with(|m| m.borrow().get(&key).map(|(m, _)| *m));
        if message.is_empty() {
            if let Some(m) = had {
                MARKS.with(|x| x.borrow_mut().remove(&key));
                unsafe {
                    let _ = wm::DestroyWindow(HWND(m as *mut _));
                }
            }
            return;
        }
        let mark = match had {
            Some(m) => HWND(m as *mut _),
            None => {
                let class = win::register("PhaiErrorMark", Some(mark_proc), None);
                let parent = unsafe { wm::GetParent(control) }.unwrap_or(self.owner);
                let m = unsafe { wm::CreateWindowExW(wm::WINDOW_EX_STYLE(0), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE, 0, 0, 1, 1, Some(parent), None, Some(win::instance()), None) }.unwrap_or_default();
                unsafe {
                    let _ = SetWindowSubclass(control, Some(watch_proc), 11, 0);
                }
                m
            }
        };
        MARKS.with(|x| x.borrow_mut().insert(key, (mark.0 as isize, message.to_string())));
        crate::props::tooltip(mark, message);
        place_mark(control, mark);
    }

    /// The words `control` is marked with; empty when none (GetError).
    pub fn error(&self, control: impl Into<HWND>) -> String {
        let key = control.into().0 as isize;
        MARKS.with(|m| m.borrow().get(&key).map(|(_, s)| s.clone())).unwrap_or_default()
    }

    /// Every mark of this window taken away.
    pub fn clear(&self) {
        let mine: Vec<isize> = MARKS.with(|m| m.borrow().keys().copied().filter(|k| unsafe { wm::GetAncestor(HWND(*k as *mut _), wm::GA_ROOT) } == self.owner).collect());
        for k in mine {
            self.set_error(HWND(k as *mut _), "");
        }
    }
}

// --- NotifyIcon ----------------------------------------------------------------------------------

/// What a NotifyIcon tells its window: `WPARAM` its id, `LPARAM`'s low word the mouse message
/// (`WM_LBUTTONUP`, `WM_LBUTTONDBLCLK`, `WM_RBUTTONUP`).
pub const WM_TRAY: u32 = wm::WM_APP + 0x3E5;

/// WinForms' NotifyIcon: the program's icon in the corner of the taskbar, its words when the mouse
/// rests on it.
#[derive(Clone, Copy, Debug, Default)]
pub struct NotifyIcon {
    pub owner: HWND,
    pub id: u32,
}

fn tray_data(owner: HWND, id: u32) -> shell::NOTIFYICONDATAW {
    shell::NOTIFYICONDATAW { cbSize: std::mem::size_of::<shell::NOTIFYICONDATAW>() as u32, hWnd: owner, uID: id, ..Default::default() }
}

fn copy_into(dst: &mut [u16], s: &str) {
    let w: Vec<u16> = s.encode_utf16().take(dst.len() - 1).collect();
    dst[..w.len()].copy_from_slice(&w);
    dst[w.len()] = 0;
}

impl NotifyIcon {
    /// Shown now when `visible`.
    pub fn new(owner: HWND, id: u32, text: &str, visible: bool) -> NotifyIcon {
        let n = NotifyIcon { owner, id };
        let mut d = tray_data(owner, id);
        d.uFlags = shell::NIF_ICON | shell::NIF_MESSAGE | shell::NIF_TIP | shell::NIF_STATE;
        d.uCallbackMessage = WM_TRAY;
        d.hIcon = crate::icon::app_icon(win::px(owner, 16)).unwrap_or_else(|| unsafe { wm::LoadIconW(None, wm::IDI_APPLICATION) }.unwrap_or_default());
        copy_into(&mut d.szTip, text);
        d.dwStateMask = shell::NIS_HIDDEN;
        d.dwState = if visible { shell::NOTIFY_ICON_STATE(0) } else { shell::NIS_HIDDEN };
        unsafe {
            let _ = shell::Shell_NotifyIconW(shell::NIM_ADD, &d);
        }
        n
    }

    /// The words shown when the mouse rests on it.
    pub fn set_text(&self, text: &str) {
        let mut d = tray_data(self.owner, self.id);
        d.uFlags = shell::NIF_TIP;
        copy_into(&mut d.szTip, text);
        unsafe {
            let _ = shell::Shell_NotifyIconW(shell::NIM_MODIFY, &d);
        }
    }

    pub fn set_visible(&self, on: bool) {
        let mut d = tray_data(self.owner, self.id);
        d.uFlags = shell::NIF_STATE;
        d.dwStateMask = shell::NIS_HIDDEN;
        d.dwState = if on { shell::NOTIFY_ICON_STATE(0) } else { shell::NIS_HIDDEN };
        unsafe {
            let _ = shell::Shell_NotifyIconW(shell::NIM_MODIFY, &d);
        }
    }

    /// A notice from it, as Windows shows them (ShowBalloonTip).
    pub fn show_balloon_tip(&self, title: &str, text: &str) {
        let mut d = tray_data(self.owner, self.id);
        d.uFlags = shell::NIF_INFO;
        copy_into(&mut d.szInfoTitle, title);
        copy_into(&mut d.szInfo, text);
        d.dwInfoFlags = shell::NIIF_INFO;
        unsafe {
            let _ = shell::Shell_NotifyIconW(shell::NIM_MODIFY, &d);
        }
    }

    /// Out of the corner for good - the generated code does it as the window closes (Dispose).
    pub fn remove(&self) {
        let d = tray_data(self.owner, self.id);
        unsafe {
            let _ = shell::Shell_NotifyIconW(shell::NIM_DELETE, &d);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_winforms_filter_becomes_windows_pairs() {
        let f = filter_of("Text files|*.txt|All files|*.*");
        assert_eq!(String::from_utf16_lossy(&f), "Text files\0*.txt\0All files\0*.*\0\0");
        assert_eq!(String::from_utf16_lossy(&filter_of("Pictures")), "Pictures\0*.*\0\0");
    }
}
