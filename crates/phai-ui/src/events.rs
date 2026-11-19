//! A control's events beyond the one Windows sends its parent - Enter and Leave, KeyDown, KeyUp,
//! KeyPress, MouseEnter, MouseLeave, MouseDown, MouseUp, MouseMove, Click, DoubleClick, Resize -
//! as WinForms raises them. [`hook`] listens to the control (a subclass) and sends each to the
//! form as [`WM_EVENT`]: which control, which event, and its arguments, which the form's
//! generated code hands to the programmer's method. Keys a method marks handled go no further.

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Sent to the form: `wparam` the control's id and the event ([`decode`]), `lparam` its
/// arguments ([`args`]).
pub const WM_EVENT: u32 = wm::WM_APP + 0x3E0;
/// Posted to a form once it shows: WinForms' Shown.
pub const WM_SHOWN: u32 = wm::WM_APP + 0x3E1;

pub const ENTER: u16 = 1;
pub const LEAVE: u16 = 2;
pub const KEY_DOWN: u16 = 3;
pub const KEY_UP: u16 = 4;
pub const KEY_PRESS: u16 = 5;
pub const MOUSE_ENTER: u16 = 6;
pub const MOUSE_LEAVE: u16 = 7;
pub const MOUSE_DOWN: u16 = 8;
pub const MOUSE_UP: u16 = 9;
pub const MOUSE_MOVE: u16 = 10;
pub const CLICK: u16 = 11;
pub const DOUBLE_CLICK: u16 = 12;
pub const RESIZE: u16 = 13;
pub const DRAG_DROP: u16 = 14;
/// A PrintDocument's page to draw (`crate::print`).
pub const PRINT_PAGE: u16 = 15;
/// A BindingSource's current row changed (`crate::binding`).
pub const POSITION_CHANGED: u16 = 16;

/// The bit of `event` in a [`hook`]'s mask.
pub const fn bit(event: u16) -> u32 {
    1 << event
}

/// The keys of the form's own KeyDown, KeyUp and KeyPress, sent from every control when the
/// form has KeyPreview: id 0.
pub const KEYS: u32 = bit(KEY_DOWN) | bit(KEY_UP) | bit(KEY_PRESS);

/// A key pressed or let go - WinForms' KeyEventArgs.
#[derive(Debug, Default)]
pub struct KeyEventArgs {
    /// The virtual key: `0x0D` Enter, `0x41` A, `0x70` F1 - as `Keys` in WinForms.
    pub key: u16,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    handled: Cell<bool>,
}

impl KeyEventArgs {
    /// The control does not get the key - WinForms' `e.Handled = true` (and SuppressKeyPress).
    pub fn set_handled(&self) {
        self.handled.set(true);
    }
    pub fn handled(&self) -> bool {
        self.handled.get()
    }
}

/// A letter typed - WinForms' KeyPressEventArgs.
#[derive(Debug, Default)]
pub struct KeyPressEventArgs {
    pub ch: char,
    handled: Cell<bool>,
}

impl KeyPressEventArgs {
    /// The letter is not typed into the control.
    pub fn set_handled(&self) {
        self.handled.set(true);
    }
    pub fn handled(&self) -> bool {
        self.handled.get()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    None,
    Left,
    Right,
    Middle,
}

/// The mouse - WinForms' MouseEventArgs. `x` and `y` from the control's top left, in pixels.
#[derive(Debug, Default)]
pub struct MouseEventArgs {
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub ctrl: bool,
    pub shift: bool,
}

/// The form about to close - WinForms' FormClosingEventArgs.
#[derive(Debug, Default)]
pub struct ClosingEventArgs {
    cancel: Cell<bool>,
}

impl ClosingEventArgs {
    /// It stays open - `e.Cancel = true`.
    pub fn cancel(&self) {
        self.cancel.set(true);
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.get()
    }
}

/// What was dropped: files from Explorer (WinForms' `DataFormats.FileDrop`), or words dragged
/// from another control of the program (`DataFormats.Text`, [`crate::drag::do_drag_drop`]) and
/// the control they came from.
#[derive(Debug, Default)]
pub struct FileDropEventArgs {
    pub files: Vec<std::path::PathBuf>,
    pub text: Option<String>,
    pub source: Option<HWND>,
}

/// The files of a `WM_DROPFILES`, the drop then let go.
pub fn files_of(w: WPARAM) -> FileDropEventArgs {
    use windows::Win32::UI::Shell::{DragFinish, DragQueryFileW, HDROP};
    let drop = HDROP(w.0 as *mut _);
    let n = unsafe { DragQueryFileW(drop, u32::MAX, None) };
    let files = (0..n)
        .map(|i| {
            let len = unsafe { DragQueryFileW(drop, i, None) } as usize;
            let mut buf = vec![0u16; len + 1];
            unsafe { DragQueryFileW(drop, i, Some(&mut buf)) };
            std::path::PathBuf::from(String::from_utf16_lossy(&buf[..len]))
        })
        .collect();
    unsafe { DragFinish(drop) };
    FileDropEventArgs { files, ..Default::default() }
}

/// An item of a list - a column clicked, a row chosen.
#[derive(Debug, Default)]
pub struct ItemEventArgs {
    pub index: usize,
}

/// The control's id and the event of a [`WM_EVENT`].
pub fn decode(w: WPARAM) -> (i32, u16) {
    ((w.0 >> 16) as i32, (w.0 & 0xFFFF) as u16)
}

/// The arguments a [`WM_EVENT`] carries.
///
/// # Safety
/// `l` must be the `lparam` of a `WM_EVENT` for an event whose arguments are a `T`, read while
/// the message is being handled.
pub unsafe fn args<'a, T>(l: LPARAM) -> &'a T {
    unsafe { &*(l.0 as *const T) }
}

/// Sends event `event` of the control with `id` to `form`, with `a`; the form's answer returned.
pub fn raise<T>(form: HWND, id: i32, event: u16, a: &T) {
    unsafe {
        wm::SendMessageW(form, WM_EVENT, Some(WPARAM(((id as usize) << 16) | event as usize)), Some(LPARAM(a as *const T as isize)));
    }
}

#[derive(Clone, Copy)]
struct Hook {
    form: isize,
    id: i32,
    mask: u32,
    /// The form's KeyPreview: keys to the form first, as id 0.
    preview: bool,
    inside: bool,
    last_down: (u32, i32, i32),
}

static HOOKS: Mutex<Option<HashMap<isize, Hook>>> = Mutex::new(None);

fn hook_of(h: HWND) -> Option<Hook> {
    HOOKS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).copied())
}

/// The form and id a drop on control `h` goes to, when it hears its DragDrop event.
pub fn drop_target(h: HWND) -> Option<(HWND, i32)> {
    hook_of(h).filter(|k| k.mask & bit(DRAG_DROP) != 0).map(|k| (HWND(k.form as *mut _), k.id))
}

fn update(h: HWND, f: impl FnOnce(&mut Hook)) {
    if let Some(k) = HOOKS.lock().unwrap_or_else(|e| e.into_inner()).as_mut().and_then(|m| m.get_mut(&(h.0 as isize))) {
        f(k);
    }
}

/// Listens to control `h` for the events in `mask` ([`bit`]s), sending them to `form` as the
/// control with `id`; `preview`, its keys go to the form first (KeyPreview). Hooking again
/// adds to what it listens for.
pub fn hook(h: HWND, form: HWND, id: i32, mask: u32, preview: bool) {
    {
        let mut m = HOOKS.lock().unwrap_or_else(|e| e.into_inner());
        let map = m.get_or_insert_with(HashMap::new);
        let k = map.entry(h.0 as isize).or_insert(Hook { form: form.0 as isize, id, mask: 0, preview: false, inside: false, last_down: (0, 0, 0) });
        k.mask |= mask;
        k.preview |= preview;
    }
    // A label hears the mouse only when it asks to: SS_NOTIFY.
    if crate::win::kind_of(h) == Some(crate::win::Kind::Label) && mask & (bit(CLICK) | bit(DOUBLE_CLICK) | bit(MOUSE_DOWN) | bit(MOUSE_UP) | bit(MOUSE_MOVE) | bit(MOUSE_ENTER) | bit(MOUSE_LEAVE)) != 0 {
        unsafe {
            let s = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
            wm::SetWindowLongW(h, wm::GWL_STYLE, (s | 0x0100) as i32);
        }
    }
    // Files can be dropped on a control that hears its DragDrop.
    if mask & bit(DRAG_DROP) != 0 {
        unsafe { windows::Win32::UI::Shell::DragAcceptFiles(h, true) };
    }
    unsafe {
        let _ = SetWindowSubclass(h, Some(proc_), 0x4556, 0);
    }
}

fn mods() -> (bool, bool, bool) {
    let down = |vk: u16| unsafe { km::GetKeyState(vk as i32) } < 0;
    (down(0x11), down(0x10), down(0x12))
}

fn mouse(l: LPARAM, button: MouseButton) -> MouseEventArgs {
    let (ctrl, shift, _) = mods();
    MouseEventArgs { x: (l.0 & 0xFFFF) as i16 as i32, y: ((l.0 >> 16) & 0xFFFF) as i16 as i32, button, ctrl, shift }
}

unsafe extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    let Some(k) = hook_of(h) else { return unsafe { DefSubclassProc(h, msg, w, l) } };
    let form = HWND(k.form as *mut _);
    let wants = |e: u16| k.mask & bit(e) != 0;
    let send = |e: u16, a: &dyn std::any::Any| {
        // The form's own first, when it previews keys; then the control's.
        if let Some(a) = a.downcast_ref::<KeyEventArgs>() {
            if k.preview && matches!(e, KEY_DOWN | KEY_UP) {
                raise(form, 0, e, a);
            }
            if wants(e) && !a.handled() {
                raise(form, k.id, e, a);
            }
        } else if let Some(a) = a.downcast_ref::<KeyPressEventArgs>() {
            if k.preview {
                raise(form, 0, e, a);
            }
            if wants(e) && !a.handled() {
                raise(form, k.id, e, a);
            }
        } else if let Some(a) = a.downcast_ref::<MouseEventArgs>() {
            if wants(e) {
                raise(form, k.id, e, a);
            }
        } else if let Some(a) = a.downcast_ref::<FileDropEventArgs>() {
            if wants(e) {
                raise(form, k.id, e, a);
            }
        } else if wants(e) {
            raise(form, k.id, e, &());
        }
    };
    match msg {
        wm::WM_SETFOCUS => send(ENTER, &()),
        wm::WM_KILLFOCUS => send(LEAVE, &()),
        wm::WM_KEYDOWN | wm::WM_SYSKEYDOWN | wm::WM_KEYUP | wm::WM_SYSKEYUP => {
            let (ctrl, shift, alt) = mods();
            let a = KeyEventArgs { key: w.0 as u16, ctrl, shift, alt, ..Default::default() };
            send(if matches!(msg, wm::WM_KEYDOWN | wm::WM_SYSKEYDOWN) { KEY_DOWN } else { KEY_UP }, &a);
            if a.handled() {
                return LRESULT(0);
            }
        }
        wm::WM_CHAR => {
            let a = KeyPressEventArgs { ch: char::from_u32(w.0 as u32).unwrap_or('\0'), ..Default::default() };
            send(KEY_PRESS, &a);
            if a.handled() {
                return LRESULT(0);
            }
        }
        wm::WM_MOUSEMOVE => {
            if !k.inside {
                update(h, |k| k.inside = true);
                let mut t = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                let _ = unsafe { km::TrackMouseEvent(&mut t) };
                send(MOUSE_ENTER, &());
            }
            send(MOUSE_MOVE, &mouse(l, MouseButton::None));
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            update(h, |k| k.inside = false);
            send(MOUSE_LEAVE, &());
        }
        wm::WM_LBUTTONDOWN | wm::WM_RBUTTONDOWN | wm::WM_MBUTTONDOWN => {
            let button = match msg {
                wm::WM_LBUTTONDOWN => MouseButton::Left,
                wm::WM_RBUTTONDOWN => MouseButton::Right,
                _ => MouseButton::Middle,
            };
            send(MOUSE_DOWN, &mouse(l, button));
            if msg == wm::WM_LBUTTONDOWN {
                // A class without double-clicks of its own: two downs close in time and place are one.
                let now = unsafe { wm::GetMessageTime() } as u32;
                let (x, y) = ((l.0 & 0xFFFF) as i16 as i32, ((l.0 >> 16) & 0xFFFF) as i16 as i32);
                let (t, lx, ly) = k.last_down;
                let near = (x - lx).abs() <= unsafe { wm::GetSystemMetrics(wm::SM_CXDOUBLECLK) } && (y - ly).abs() <= unsafe { wm::GetSystemMetrics(wm::SM_CYDOUBLECLK) };
                if t != 0 && now.wrapping_sub(t) <= unsafe { km::GetDoubleClickTime() } && near {
                    update(h, |k| k.last_down = (0, 0, 0));
                    send(DOUBLE_CLICK, &());
                } else {
                    update(h, |k| k.last_down = (now, x, y));
                }
            }
        }
        wm::WM_LBUTTONDBLCLK => {
            update(h, |k| k.last_down = (0, 0, 0));
            send(MOUSE_DOWN, &mouse(l, MouseButton::Left));
            send(DOUBLE_CLICK, &());
        }
        wm::WM_LBUTTONUP | wm::WM_RBUTTONUP | wm::WM_MBUTTONUP => {
            let button = match msg {
                wm::WM_LBUTTONUP => MouseButton::Left,
                wm::WM_RBUTTONUP => MouseButton::Right,
                _ => MouseButton::Middle,
            };
            send(MOUSE_UP, &mouse(l, button));
            if msg == wm::WM_LBUTTONUP {
                let mut r = RECT::default();
                let _ = unsafe { wm::GetClientRect(h, &mut r) };
                let p = POINT { x: (l.0 & 0xFFFF) as i16 as i32, y: ((l.0 >> 16) & 0xFFFF) as i16 as i32 };
                if p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom {
                    send(CLICK, &());
                }
            }
        }
        wm::WM_SIZE => send(RESIZE, &()),
        0x0233 /* WM_DROPFILES */ if wants(DRAG_DROP) => {
            send(DRAG_DROP, &files_of(w));
            return LRESULT(0);
        }
        wm::WM_NCDESTROY => {
            if let Some(m) = HOOKS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                m.remove(&(h.0 as isize));
            }
        }
        _ => {}
    }
    unsafe { DefSubclassProc(h, msg, w, l) }
}

/// The mouse arguments of a form's own mouse message.
pub fn mouse_of(msg: u32, l: LPARAM) -> MouseEventArgs {
    let button = match msg {
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONUP | wm::WM_LBUTTONDBLCLK => MouseButton::Left,
        wm::WM_RBUTTONDOWN | wm::WM_RBUTTONUP => MouseButton::Right,
        wm::WM_MBUTTONDOWN | wm::WM_MBUTTONUP => MouseButton::Middle,
        _ => MouseButton::None,
    };
    mouse(l, button)
}

/// The key arguments of a form's own key message.
pub fn key_of(w: WPARAM) -> KeyEventArgs {
    let (ctrl, shift, alt) = mods();
    KeyEventArgs { key: w.0 as u16, ctrl, shift, alt, ..Default::default() }
}

pub fn key_press_of(w: WPARAM) -> KeyPressEventArgs {
    KeyPressEventArgs { ch: char::from_u32(w.0 as u32).unwrap_or('\0'), ..Default::default() }
}
