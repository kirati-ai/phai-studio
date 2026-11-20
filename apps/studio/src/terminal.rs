//! The Terminal tab, as Visual Studio's: PowerShell or Command Prompt in the project's folder,
//! run with its input and output joined to the tab - what it prints shows there, a line typed in
//! the box below goes to it. Its output comes from threads reading the pipes, sent to the panel
//! as `WM_TERMINAL` messages.

use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// The message a reading thread sends the panel: `LPARAM` a `Box<String>` of what was printed.
pub const WM_TERMINAL: u32 = wm::WM_APP + 0x0E10;

/// The shells, in the tab's drop-down order.
pub const SHELLS: [&str; 2] = ["PowerShell", "Command Prompt"];

pub struct Shell {
    child: Child,
    stdin: ChildStdin,
    /// PowerShell prints no prompt when it reads its commands from a pipe: the tab writes one.
    pub echoes: bool,
    /// Command Prompt reads a pipe in Windows' own code page (874 for Thai), not UTF-8.
    ansi_input: bool,
}

impl Shell {
    /// Starts shell `which` (see [`SHELLS`]) in `dir`; what it prints goes to `panel`.
    pub fn start(which: usize, dir: &Path, panel: HWND) -> std::io::Result<Shell> {
        // CREATE_NO_WINDOW: no console of its own; its words come to the tab.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut c = if which == 1 {
            let mut c = Command::new("cmd.exe");
            // UTF-8 for Thai and the rest; cmd shows its prompt and what was typed itself.
            c.args(["/K", "chcp 65001>nul"]);
            c
        } else {
            let mut c = Command::new("powershell.exe");
            c.args(["-NoLogo", "-NoProfile", "-NoExit", "-Command", "-"]);
            c
        };
        let mut child = c.current_dir(dir).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).creation_flags(CREATE_NO_WINDOW).spawn()?;
        let mut stdin = child.stdin.take().ok_or_else(|| std::io::Error::other("no input"))?;
        let target = panel.0 as isize;
        for out in [child.stdout.take().map(|o| Box::new(o) as Box<dyn Read + Send>), child.stderr.take().map(|e| Box::new(e) as Box<dyn Read + Send>)].into_iter().flatten() {
            std::thread::spawn(move || read_to(out, target));
        }
        if which == 0 {
            // Its words as UTF-8, so Thai comes through.
            let _ = writeln!(stdin, "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $OutputEncoding = [System.Text.Encoding]::UTF8");
            let _ = stdin.flush();
        }
        Ok(Shell { child, stdin, echoes: which == 0, ansi_input: which == 1 })
    }

    /// A line typed, sent to the shell.
    pub fn send(&mut self, line: &str) -> bool {
        let mut bytes = if self.ansi_input { to_ansi(line) } else { line.as_bytes().to_vec() };
        bytes.extend_from_slice(b"\r\n");
        self.stdin.write_all(&bytes).and_then(|_| self.stdin.flush()).is_ok()
    }
}

/// `text` in Windows' own code page, as Command Prompt reads its input.
fn to_ansi(text: &str) -> Vec<u8> {
    use windows::Win32::Globalization::{WideCharToMultiByte, CP_ACP};
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return Vec::new();
    }
    unsafe {
        let n = WideCharToMultiByte(CP_ACP, 0, &wide, None, None, None);
        if n <= 0 {
            return text.as_bytes().to_vec();
        }
        let mut out = vec![0u8; n as usize];
        WideCharToMultiByte(CP_ACP, 0, &wide, Some(&mut out), None, None);
        out
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Reads a pipe until it ends, sending what comes as text - a character split between two reads
/// kept for the next.
fn read_to(mut pipe: Box<dyn Read + Send>, target: isize) {
    let mut buf = [0u8; 4096];
    let mut carry: Vec<u8> = Vec::new();
    loop {
        match pipe.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                carry.extend_from_slice(&buf[..n]);
                // Up to the last whole character.
                let good = match std::str::from_utf8(&carry) {
                    Ok(_) => carry.len(),
                    Err(e) if e.error_len().is_none() => e.valid_up_to(),
                    Err(_) => carry.len(),
                };
                let text = String::from_utf8_lossy(&carry[..good]).to_string();
                carry.drain(..good);
                post(target, text);
            }
        }
    }
    post(target, String::new());
}

fn post(target: isize, text: String) {
    let boxed = Box::into_raw(Box::new(text));
    unsafe {
        if wm::PostMessageW(Some(HWND(target as *mut _)), WM_TERMINAL, WPARAM(0), LPARAM(boxed as isize)).is_err() {
            drop(Box::from_raw(boxed));
        }
    }
}

/// What the tab shows for `text` from the shell: Windows line ends, the console's colour codes
/// taken out.
pub fn shown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // ESC [ ... letter
            if chars.peek() == Some(&'[') {
                chars.next();
                for d in chars.by_ref() {
                    if d.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        match c {
            '\r' => {}
            '\n' => out.push_str("\r\n"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_codes_out_and_lines_as_windows_has_them() {
        assert_eq!(shown("\u{1b}[32mok\u{1b}[0m\nnext\r\n"), "ok\r\nnext\r\n");
        assert_eq!(shown("สวัสดี"), "สวัสดี");
    }
}
