//! The two small windows an editor opens beside its text: a tip (a hover's words, a diagnostic's
//! message) and the completion list. Neither takes the focus - typing stays in the editor, which
//! moves the list's choice with the arrow keys.

use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::lsp::Completion;

pub struct Popup {
    pub hwnd: HWND,
    font: HFONT,
    /// A tip's lines, or nothing for a list.
    lines: Vec<String>,
    pub items: Vec<Completion>,
    pub chosen: usize,
    /// The choice is only a suggestion - the list opened by itself at a call's argument, nothing
    /// typed or moved to yet: Enter is a new line, not this choice; it is framed, not filled.
    pub soft: bool,
    row_h: i32,
    /// Its font is its own, made for it, deleted with it.
    own_font: bool,
    /// Beside a list: the chosen item's guide - what it does, how it is written - for a hand new
    /// to it, as Visual Studio's Quick Info beside IntelliSense.
    guide: Option<Box<Popup>>,
}

const MAX_ROWS: usize = 10;

fn make(owner: HWND) -> HWND {
    let class = win::register("PhaiEditorPopup", Some(proc_), None);
    unsafe {
        let top = wm::GetAncestor(owner, wm::GA_ROOT);
        wm::CreateWindowExW(wm::WS_EX_NOACTIVATE | wm::WS_EX_TOPMOST | wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, 0, 0, 10, 10, Some(top), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default()
    }
}

impl Popup {
    /// A tip with `text` below screen point `(x, y)`.
    pub fn tip(owner: HWND, font: HFONT, text: &str, x: i32, y: i32) -> Box<Popup> {
        let lines: Vec<String> = text.lines().take(14).map(|l| l.chars().take(110).collect()).collect();
        let p = Popup::new(owner, font, lines, vec![]);
        p.place(x, y);
        p
    }

    /// A tip whose bottom is at screen point `(x, y)` - Parameter Info above the line typed on,
    /// out of the way of the list that opens below it.
    pub fn tip_above(owner: HWND, font: HFONT, text: &str, x: i32, y: i32) -> Box<Popup> {
        let p = Popup::tip(owner, font, text, x, y);
        unsafe {
            let mut r = windows::Win32::Foundation::RECT::default();
            let _ = wm::GetWindowRect(p.hwnd, &mut r);
            let top = (y - (r.bottom - r.top)).max(0);
            let _ = wm::SetWindowPos(p.hwnd, Some(wm::HWND_TOPMOST), r.left, top, 0, 0, wm::SWP_NOACTIVATE | wm::SWP_NOSIZE);
        }
        p
    }

    /// A completion list below screen point `(x, y)`.
    /// In Windows' own letters (Segoe UI 9 pt), smaller than the code's, as Visual Studio's
    /// IntelliSense list; `font` is the code's, for nothing but the call.
    pub fn list(owner: HWND, font: HFONT, items: Vec<Completion>, x: i32, y: i32) -> Box<Popup> {
        let _ = font;
        let mut p = Popup::new(owner, win::font(owner, 0, false), vec![], items);
        p.own_font = true;
        p.place(x, y);
        p
    }

    fn new(owner: HWND, font: HFONT, lines: Vec<String>, items: Vec<Completion>) -> Box<Popup> {
        let hwnd = make(owner);
        let row_h = win::measure(hwnd, font, "Ag").1 + win::px(hwnd, 6);
        let b = Box::new(Popup { hwnd, font, lines, items, chosen: 0, soft: false, row_h, own_font: false, guide: None });
        unsafe {
            wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, &*b as *const Popup as isize);
        }
        b
    }

    fn rows(&self) -> Vec<String> {
        if self.items.is_empty() {
            self.lines.clone()
        } else {
            self.items.iter().take(MAX_ROWS).map(|c| c.label.clone()).collect()
        }
    }

    /// Sizes itself to its rows and shows below `(x, y)`, kept on the screen.
    pub fn place(&self, x: i32, y: i32) {
        let rows = self.rows();
        let pad = win::px(self.hwnd, 8);
        let label_w = rows.iter().map(|r| win::measure(self.hwnd, self.font, r).0).max().unwrap_or(0);
        // Room for the longest detail beside the longest name, up to a reasonable width; what is
        // longer still is cut with ... rather than drawn over the name.
        let detail_w = self.items.iter().take(MAX_ROWS).map(|c| win::measure(self.hwnd, self.font, &detail_of(c)).0).max().unwrap_or(0);
        let gap = if detail_w > 0 { win::px(self.hwnd, 24) } else { 0 };
        let w = (label_w + gap + detail_w + 2 * pad).min(win::px(self.hwnd, 760)).max(label_w + 2 * pad);
        let h = rows.len().max(1) as i32 * self.row_h + pad;
        unsafe {
            let mon = Gdi::MonitorFromPoint(windows::Win32::Foundation::POINT { x, y }, Gdi::MONITOR_DEFAULTTONEAREST);
            let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
            let _ = Gdi::GetMonitorInfoW(mon, &mut mi);
            let x = x.min(mi.rcWork.right - w).max(mi.rcWork.left);
            let y = if y + h > mi.rcWork.bottom { y - h - self.row_h } else { y };
            let _ = wm::SetWindowPos(self.hwnd, Some(wm::HWND_TOPMOST), x, y, w, h, wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
            let _ = Gdi::InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn move_choice(&mut self, by: i32) {
        if self.items.is_empty() {
            return;
        }
        let n = self.items.len().min(MAX_ROWS) as i32;
        self.chosen = ((self.chosen as i32 + by).rem_euclid(n)) as usize;
        self.soft = false;
        self.show_guide();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn choice(&self) -> Option<&Completion> {
        self.items.get(self.chosen)
    }

    /// The guide's window, when one shows - for tests.
    pub fn guide_hwnd(&self) -> Option<HWND> {
        self.guide.as_ref().map(|g| g.hwnd)
    }

    /// The chosen item's guide beside the list - at its right, or its left where the screen ends;
    /// none for an item with nothing to say.
    pub fn show_guide(&mut self) {
        self.guide = None;
        let Some(text) = self.choice().map(guide_text).filter(|t| !t.is_empty()) else { return };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(self.hwnd, &mut r);
        }
        let g = Popup::tip(self.hwnd, self.font, &text, r.right + win::px(self.hwnd, 2), r.top);
        unsafe {
            let mut gr = RECT::default();
            let _ = wm::GetWindowRect(g.hwnd, &mut gr);
            // Pushed back over the list by the screen's edge: to the list's left instead.
            if gr.left < r.right {
                let w = gr.right - gr.left;
                let _ = wm::SetWindowPos(g.hwnd, Some(wm::HWND_TOPMOST), (r.left - w - win::px(self.hwnd, 2)).max(0), r.top, 0, 0, wm::SWP_NOACTIVATE | wm::SWP_NOSIZE);
            }
        }
        self.guide = Some(g);
    }
}

/// A completion's guide as a beginner reads it: how it is written (its signature), then its
/// `///` words - Rust's marks taken out (`[`Self::hide`]` is hide), each line wrapped to read.
pub fn guide_text(c: &Completion) -> String {
    guide_in(c, phai_ui::text::is_thai())
}

/// The guide in Thai or in English. A comment may say it in both: its `ไทย:` line is Thai's, the
/// rest English's; Thai shows its own words and the same Example, English leaves Thai's out. With
/// no Thai line, Thai shows the English.
pub fn guide_in(c: &Completion, thai: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    let head = heading(c);
    if !head.is_empty() {
        out.push(head);
    }
    let doc = in_language(&clean_doc(&c.doc), thai);
    if !doc.trim().is_empty() {
        if !out.is_empty() {
            out.push(String::new());
        }
        // The comment's lines, cut where its writer's editor ended them, joined again into
        // paragraphs and wrapped to the box; an Example starts a line of its own.
        let mut para = String::new();
        let flush = |para: &mut String, out: &mut Vec<String>| {
            if !para.is_empty() {
                out.extend(wrap(para, 72));
                para.clear();
            }
        };
        for line in doc.lines() {
            if line.starts_with("    ") {
                flush(&mut para, &mut out);
                out.push(line.to_string());
            } else if line.trim().is_empty() {
                flush(&mut para, &mut out);
                if out.last().is_some_and(|l| !l.is_empty()) {
                    out.push(String::new());
                }
            } else {
                // An Example stands apart: a line of its own after a blank one.
                if line.starts_with("Example:") || line.starts_with("ตัวอย่าง:") {
                    flush(&mut para, &mut out);
                    if out.last().is_some_and(|l| !l.is_empty()) {
                        out.push(String::new());
                    }
                }
                if !para.is_empty() {
                    para.push(' ');
                }
                para.push_str(line.trim());
            }
        }
        flush(&mut para, &mut out);
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    out.join("\n")
}

/// A comment's words in one language: its `ไทย:` paragraph (to a blank line or an Example) is
/// Thai's; Thai keeps that, the Examples (as ตัวอย่าง:) and code; English keeps all but that.
fn in_language(doc: &str, thai: bool) -> String {
    let has_thai = doc.lines().any(|l| l.trim_start().starts_with("ไทย:"));
    let mut out = Vec::new();
    let mut in_thai = false;
    for line in doc.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("ไทย:") {
            in_thai = true;
            if thai {
                out.push(rest.trim().to_string());
            }
            continue;
        }
        if t.is_empty() || t.starts_with("Example:") || line.starts_with("    ") {
            in_thai = false;
        }
        if in_thai {
            if thai {
                out.push(t.to_string());
            }
            continue;
        }
        if thai && has_thai {
            // Thai's own words stand in for the English sentences; an Example and code stay.
            if let Some(rest) = t.strip_prefix("Example:") {
                out.push(format!("ตัวอย่าง:{rest}"));
            } else if line.starts_with("    ") || t.is_empty() {
                out.push(line.to_string());
            }
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

/// How it is written, as a beginner reads it: `alert(&str)`, `confirm(&str) -> bool`, `ok: Button` -
/// the name, then what it takes (the `self` it is called on left out) or what it is.
fn heading(c: &Completion) -> String {
    let d = c.detail.trim();
    if d.is_empty() {
        return String::new();
    }
    if let Some(rest) = d.strip_prefix("fn(") {
        let rest = ["&mut self, ", "&self, ", "self, ", "&mut self", "&self", "self"].iter().find_map(|s| rest.strip_prefix(s)).unwrap_or(rest);
        return format!("{}({rest}", c.label);
    }
    if d.contains(c.label.as_str()) { d.to_string() } else { format!("{}: {d}", c.label) }
}

/// A `///` comment in plain words: links `[`x`]` and `[x](..)` as their words, `Self::` and
/// backticks gone, a code block's lines indented and its fences dropped.
fn clean_doc(doc: &str) -> String {
    let mut out = Vec::new();
    let mut code = false;
    for line in doc.lines() {
        if line.trim_start().starts_with("```") {
            code = !code;
            continue;
        }
        if code {
            out.push(format!("    {line}"));
            continue;
        }
        let mut s = String::new();
        let mut rest = line;
        while let Some(i) = rest.find('[') {
            s.push_str(&rest[..i]);
            let after = &rest[i + 1..];
            match after.find(']') {
                Some(j) => {
                    s.push_str(&after[..j]);
                    rest = &after[j + 1..];
                    // `[words](link)`: the link dropped.
                    if rest.starts_with('(') {
                        rest = rest.find(')').map(|k| &rest[k + 1..]).unwrap_or("");
                    }
                }
                None => {
                    s.push('[');
                    rest = after;
                }
            }
        }
        s.push_str(rest);
        out.push(s.replace("Self::", "").replace('`', ""));
    }
    out.join("\n")
}

/// Words cut into lines of at most `width` letters, between words.
/// Thai's marks above and below a letter - they take no room of their own on a line.
fn is_mark(c: char) -> bool {
    matches!(c, '\u{0E31}' | '\u{0E34}'..='\u{0E3A}' | '\u{0E47}'..='\u{0E4E}')
}

/// How many letters wide words are, Thai's marks not counted.
fn width_of(s: &str) -> usize {
    s.chars().filter(|c| !is_mark(*c)).count()
}

/// Words cut into lines of at most `width` letters, between words; a word longer than a line (Thai
/// writes a phrase with no spaces) cut where a letter begins, never between it and its marks.
fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split(' ') {
        if !line.is_empty() && width_of(&line) + 1 + width_of(word) > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        let mut rest = word;
        while width_of(&line) + width_of(rest) > width {
            let room = width.saturating_sub(width_of(&line)).max(1);
            let mut n = 0;
            let mut cut = rest.len();
            for (i, c) in rest.char_indices() {
                if !is_mark(c) {
                    if n == room {
                        cut = i;
                        break;
                    }
                    n += 1;
                }
            }
            if cut == rest.len() {
                break;
            }
            line.push_str(&rest[..cut]);
            out.push(std::mem::take(&mut line));
            rest = &rest[cut..];
        }
        line.push_str(rest);
    }
    out.push(line);
    out
}

impl Drop for Popup {
    fn drop(&mut self) {
        // The guide first: it draws in this list's font.
        self.guide = None;
        unsafe {
            wm::SetWindowLongPtrW(self.hwnd, wm::GWLP_USERDATA, 0);
            let _ = wm::DestroyWindow(self.hwnd);
            if self.own_font {
                let _ = Gdi::DeleteObject(self.font.into());
            }
        }
    }
}

/// A completion's detail as the list shows it: up to 60 letters.
fn detail_of(c: &Completion) -> String {
    let n = c.detail.chars().count();
    if n > 60 { format!("{}...", c.detail.chars().take(57).collect::<String>()) } else { c.detail.clone() }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let p = unsafe { (wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA) as *const Popup).as_ref() };
    match (msg, p) {
        (wm::WM_MOUSEACTIVATE, _) => LRESULT(3), // MA_NOACTIVATE
        (wm::WM_ERASEBKGND, _) => LRESULT(1),
        (wm::WM_PAINT, Some(p)) => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = phai_ui::win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let bg = theme::style("edit", State::default()).background_or(win::window_background());
            win::fill(dc, &r, bg);
            let chosen_bg = theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xe7, 0xf3, 0xef));
            let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x80, 0x80, 0x80));
            let pad = win::px(h, 8);
            let old = Gdi::SelectObject(dc, p.font.into());
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            for (i, row) in p.rows().iter().enumerate() {
                let y = pad / 2 + i as i32 * p.row_h;
                let line = RECT { left: 0, top: y, right: r.right, bottom: y + p.row_h };
                if !p.items.is_empty() && i == p.chosen {
                    if p.soft {
                        let one = win::px(h, 1);
                        for edge in [RECT { bottom: line.top + one, ..line }, RECT { top: line.bottom - one, ..line }, RECT { right: line.left + one, ..line }, RECT { left: line.right - one, ..line }] {
                            win::fill(dc, &edge, muted);
                        }
                    } else {
                        win::fill(dc, &line, chosen_bg);
                    }
                }
                let mut tr = RECT { left: pad, ..line };
                let label_w = win::measure(h, p.font, row).0;
                let mut words: Vec<u16> = row.encode_utf16().collect();
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(win::window_text().colorref()));
                if !words.is_empty() {
                    phai_ui::win::draw_text_w(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                }
                // A completion's detail, muted, at the right.
                if let Some(c) = p.items.get(i) {
                    if !c.detail.is_empty() {
                        let mut d: Vec<u16> = detail_of(c).encode_utf16().collect();
                        // After the name, never over it.
                        let mut dr = RECT { left: pad + label_w + win::px(h, 24), right: r.right - pad, ..line };
                        if dr.right <= dr.left {
                            continue;
                        }
                        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                        phai_ui::win::draw_text_w(dc, &mut d, &mut dr, Gdi::DT_RIGHT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
                    }
                }
            }
            Gdi::SelectObject(dc, old);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guide_reads_as_words() {
        let c = Completion { label: "hide".into(), insert: "hide".into(), detail: "fn(&self)".into(), doc: "Out of sight, still running; [`Self::show_window`] brings it back.\n\n```\nui.hide();\n```".into() };
        assert_eq!(guide_in(&c, false), "hide()\n\nOut of sight, still running; show_window brings it back.\n\n    ui.hide();");
        // A comment's lines joined again; its Example on a line of its own.
        let alert = Completion { label: "alert".into(), detail: "fn(&self, &str)".into(), doc: "Shows a message with an OK button; the title is the\nwindow's.\nExample: `ui.alert(\"Saved\");`".into(), ..c.clone() };
        assert_eq!(guide_in(&alert, false), "alert(&str)\n\nShows a message with an OK button; the title is the window's.\n\nExample: ui.alert(\"Saved\");");
        let field = Completion { label: "ok".into(), detail: "Button".into(), doc: String::new(), ..c.clone() };
        assert_eq!(guide_in(&field, false), "ok: Button");
        let long = Completion { doc: "word ".repeat(30), ..c.clone() };
        assert!(guide_in(&long, false).lines().all(|l| l.chars().count() <= 72), "wrapped");
        let none = Completion { detail: String::new(), doc: String::new(), ..c };
        assert_eq!(guide_in(&none, false), "", "nothing to say: no guide");
        assert_eq!(clean_doc("See [the guide](https://x.y/z) and [`DateTime::format`]."), "See the guide and DateTime::format.");
    }

    #[test]
    fn a_guide_in_thai() {
        let c = Completion { label: "alert".into(), insert: "alert".into(), detail: "fn(&self, &str)".into(), doc: "Shows a message with an OK button.\n\nไทย: แสดงข้อความพร้อมปุ่ม OK\n\nExample: `ui.alert(\"Saved\");`".into() };
        assert_eq!(guide_in(&c, true), "alert(&str)\n\nแสดงข้อความพร้อมปุ่ม OK\n\nตัวอย่าง: ui.alert(\"Saved\");");
        assert_eq!(guide_in(&c, false), "alert(&str)\n\nShows a message with an OK button.\n\nExample: ui.alert(\"Saved\");", "English leaves Thai out");
        let english_only = Completion { doc: "Shows a message.\nExample: `ui.alert(\"x\");`".into(), ..c.clone() };
        assert_eq!(guide_in(&english_only, true), "alert(&str)\n\nShows a message.\n\nExample: ui.alert(\"x\");", "no Thai line: the English");
        // A long Thai phrase with no spaces cut, its marks kept with their letters.
        let lines = wrap(&"น้ำ".repeat(40), 72);
        assert!(lines.len() > 1 && lines.iter().all(|l| width_of(l) <= 72 && !l.starts_with(is_mark)), "{lines:?}");
    }
}
