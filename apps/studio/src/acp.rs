//! A client of the Agent Client Protocol (agentclientprotocol.com), as Zed and JetBrains speak it
//! to GitHub Copilot's agent (`copilot-language-server --acp`): JSON-RPC, one message a line,
//! over the agent's stdin and stdout. A thread reads what it says and posts each message to the
//! AI Assistant's pane as [`WM_ACP`]; the pane's thread does everything else.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;

use serde_json::{json, Value};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Posted for each message from the agent; `LPARAM` is a `Box<serde_json::Value>` to take back.
/// `{"phaiGone": true}` when it has ended.
pub const WM_ACP: u32 = wm::WM_APP + 0x0E41;

pub struct Acp {
    child: Child,
    stdin: Mutex<ChildStdin>,
    next: i64,
}

impl Acp {
    /// Runs the agent; what it says goes to `window`.
    pub fn start(mut cmd: Command, window: HWND) -> Option<Acp> {
        let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let target = window.0 as isize;
        std::thread::spawn(move || {
            let post = |v: Value| {
                let b = Box::into_raw(Box::new(v));
                unsafe {
                    if wm::PostMessageW(Some(HWND(target as *mut _)), WM_ACP, WPARAM(0), LPARAM(b as isize)).is_err() {
                        drop(Box::from_raw(b));
                        return false;
                    }
                }
                true
            };
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Ok(v) = serde_json::from_str::<Value>(line.trim()) else { continue };
                if std::env::var_os("PHAI_ACP_LOG").is_some() {
                    eprintln!("acp: {}", line.chars().take(400).collect::<String>());
                }
                if !post(v) {
                    return;
                }
            }
            post(json!({ "phaiGone": true }));
        });
        Some(Acp { child, stdin: Mutex::new(stdin), next: 1 })
    }

    fn send(&self, v: &Value) {
        let mut w = self.stdin.lock().unwrap_or_else(|e| e.into_inner());
        let _ = writeln!(w, "{v}");
        let _ = w.flush();
    }

    /// Sends a request; its id, which the answer carries.
    pub fn request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next;
        self.next += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        id
    }

    pub fn notify(&self, method: &str, params: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// Answers a request the agent made of us.
    pub fn answer(&self, id: &Value, result: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    pub fn refuse(&self, id: &Value, code: i64, message: &str) {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }));
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for Acp {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// The answer to a permission request choosing the option of `kind` (`allow_once`,
/// `reject_once`...), or the first that allows / rejects when the agent names others.
pub fn choose(options: &Value, allow: bool) -> Value {
    let list = options.as_array().cloned().unwrap_or_default();
    let want = if allow { "allow_once" } else { "reject_once" };
    let pick = list
        .iter()
        .find(|o| o["kind"] == want)
        .or_else(|| list.iter().find(|o| o["kind"].as_str().is_some_and(|k| k.starts_with(if allow { "allow" } else { "reject" }))))
        .and_then(|o| o["optionId"].as_str());
    match pick {
        Some(id) => json!({ "outcome": { "outcome": "selected", "optionId": id } }),
        None => json!({ "outcome": { "outcome": "cancelled" } }),
    }
}

/// `old` with a unified diff (`@@ -a,b +c,d @@` hunks) made - what the agent's edit would make
/// of the file - or `None` when the diff does not fit it.
pub fn apply_unified(old: &str, diff: &str) -> Option<String> {
    let lines: Vec<&str> = old.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut at = 0usize;
    let mut seen_hunk = false;
    let mut d = diff.lines().peekable();
    while let Some(l) = d.next() {
        let Some(rest) = l.strip_prefix("@@ -") else { continue };
        seen_hunk = true;
        let old_side = rest.split(' ').next()?;
        let mut nums = old_side.split(',');
        let from: usize = nums.next()?.parse().ok()?;
        let count: usize = nums.next().map(|n| n.parse().unwrap_or(1)).unwrap_or(1);
        // No lines of the old file in it: what it adds goes after line `from` (0, a new file).
        let start = if count == 0 { from } else { from.saturating_sub(1) };
        if start < at || start > lines.len() {
            return None;
        }
        out.extend(lines[at..start].iter().map(|s| s.to_string()));
        at = start;
        while let Some(h) = d.peek() {
            if h.starts_with("@@") || h.starts_with("diff --git") {
                break;
            }
            let h = d.next()?;
            if let Some(added) = h.strip_prefix('+') {
                out.push(added.to_string());
            } else if let Some(gone) = h.strip_prefix('-') {
                if lines.get(at).map(|s| s.trim_end_matches('\r')) != Some(gone.trim_end_matches('\r')) {
                    return None;
                }
                at += 1;
            } else if h.starts_with('\\') {
                // "\ No newline at end of file"
            } else {
                let kept = h.strip_prefix(' ').unwrap_or(h);
                if at >= lines.len() && kept.is_empty() {
                    continue;
                }
                if lines.get(at).map(|s| s.trim_end_matches('\r')) != Some(kept.trim_end_matches('\r')) {
                    return None;
                }
                out.push(lines[at].to_string());
                at += 1;
            }
        }
    }
    if !seen_hunk {
        return None;
    }
    out.extend(lines[at.min(lines.len())..].iter().map(|s| s.to_string()));
    Some(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_agent_s_diff_is_made_on_the_file() {
        let old = "fn main() {\n    println!(\"hi\");\n}\n";
        // As Copilot's agent sends it with a permission request.
        let diff = "\ndiff --git a/D:/x/main.rs b/D:/x/main.rs\nindex 0000000..0000000 100644\n--- a/D:/x/main.rs\n+++ b/D:/x/main.rs\n@@ -1,4 +1,4 @@\n fn main() {\n-    println!(\"hi\");\n+    println!(\"hello world\");\n }\n \n";
        assert_eq!(apply_unified(old, diff).unwrap(), "fn main() {\n    println!(\"hello world\");\n}\n");
        // Two hunks, lines added.
        let old = "a\nb\nc\nd\ne\nf\ng\n";
        let diff = "@@ -1,2 +1,3 @@\n a\n+a2\n b\n@@ -6,2 +7,2 @@\n f\n-g\n+G\n";
        assert_eq!(apply_unified(old, diff).unwrap(), "a\na2\nb\nc\nd\ne\nf\nG\n");
        // A new file.
        assert_eq!(apply_unified("", "@@ -0,0 +1,2 @@\n+one\n+two\n").unwrap(), "one\ntwo\n");
        // Not this file's: refused.
        assert!(apply_unified(old, "@@ -1,1 +1,1 @@\n-x\n+y\n").is_none());
        assert!(apply_unified(old, "no hunks").is_none());
        // Lines put in after line 3, none taken out.
        assert_eq!(apply_unified("1
2
3
4", "@@ -3,0 +4,1 @@
+3.5
").unwrap(), "1
2
3
3.5
4");
    }

    #[test]
    fn permissions_are_answered_by_kind() {
        let options = json!([{"optionId": "allow_once", "kind": "allow_once"}, {"optionId": "allow_always", "kind": "allow_always"}, {"optionId": "reject_once", "kind": "reject_once"}]);
        assert_eq!(choose(&options, true)["outcome"]["optionId"], "allow_once");
        assert_eq!(choose(&options, false)["outcome"]["optionId"], "reject_once");
        let odd = json!([{"optionId": "yes", "kind": "allow_always"}]);
        assert_eq!(choose(&odd, true)["outcome"]["optionId"], "yes");
        assert_eq!(choose(&odd, false)["outcome"]["outcome"], "cancelled");
    }
}
