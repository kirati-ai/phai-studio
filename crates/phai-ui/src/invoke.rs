//! Work from another thread on a window's thread - WinForms' `Invoke` and `BeginInvoke`. A
//! window's controls belong to the thread that made it; a worker thread that has read a file or
//! asked a server hands what it found to the window this way, and the window's thread does the
//! changing. An [`Invoker`] is the window's handle, safe to send to any thread.
//!
//! ```ignore
//! let inv = ui.invoker();
//! std::thread::spawn(move || {
//!     let rows = read_a_big_file();
//!     inv.begin_invoke(move || ui_rows.fill(&rows));          // with what it captured
//! });
//! MainWindowUi::begin_invoke(inv, |me: &mut MainWindow, ui| { // with the program's state
//!     me.done = true;
//!     ui.status.set_part_text(0, "Done");
//! });
//! ```

use std::any::Any;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Posted or sent to a form: run the work `lparam` holds ([`run`]).
pub const WM_INVOKE: u32 = wm::WM_APP + 0x3E2;

/// Work for a window's thread, given the window's program state and its controls.
pub type Work = Box<dyn FnOnce(&mut dyn Any, &dyn Any) + Send>;

/// A window, for other threads to hand it work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invoker(pub isize);

impl Invoker {
    pub fn new(hwnd: HWND) -> Invoker {
        Invoker(hwnd.0 as isize)
    }

    pub fn hwnd(self) -> HWND {
        HWND(self.0 as *mut _)
    }

    /// Runs `f` on the window's thread, later; returns at once. `false` when the window is gone.
    pub fn begin_invoke(self, f: impl FnOnce() + Send + 'static) -> bool {
        self.post(Box::new(move |_, _| f()))
    }

    /// Runs `f` on the window's thread and waits until it has run.
    pub fn invoke(self, f: impl FnOnce() + Send + 'static) {
        self.send(Box::new(move |_, _| f()))
    }

    /// Hands `work` to the window, to run when its thread next looks; `false` when it is gone.
    pub fn post(self, work: Work) -> bool {
        let raw = Box::into_raw(Box::new(work));
        let ok = unsafe { wm::PostMessageW(Some(self.hwnd()), WM_INVOKE, WPARAM(0), LPARAM(raw as isize)) }.is_ok();
        if !ok {
            drop(unsafe { Box::from_raw(raw) });
        }
        ok
    }

    /// Hands `work` to the window and waits until it has run.
    pub fn send(self, work: Work) {
        let raw = Box::into_raw(Box::new(work));
        let done = unsafe { wm::SendMessageW(self.hwnd(), WM_INVOKE, Some(WPARAM(1)), Some(LPARAM(raw as isize))) }.0;
        if done == 0 {
            // No window to run it: it is dropped here.
            drop(unsafe { Box::from_raw(raw) });
        }
    }
}

/// Runs the work a [`WM_INVOKE`] carries, with the window's state and controls - from the
/// window's procedure. 1: it ran.
///
/// # Safety
/// `l` must be the `lparam` of a `WM_INVOKE` this module sent or posted, handled once.
pub unsafe fn run(l: LPARAM, state: &mut dyn Any, ui: &dyn Any) -> isize {
    let work = unsafe { Box::from_raw(l.0 as *mut Work) };
    (*work)(state, ui);
    1
}
