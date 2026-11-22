//! The Live Inspector. Press F12 (or Ctrl+Shift+I) in a debug build of a program Phai Studio made
//! and it opens beside the window, as a browser's developer tools do:
//!
//! - **Pick** a control with the mouse, or choose it from the list. A frame shows which one.
//! - Change its **look in any state** (normal, hover, pressed...), the ImageStyler way. The control
//!   shows that state while it is being changed, and each value says where it comes from.
//! - Change its **words, Enabled, Visible** and any property `phai_ui::props` knows.
//!
//! Every change shows at once in the program, which keeps running. **Save to form** writes them
//! into the window's `.phaiform`, the HomeSite way: only that control's lines change, and the
//! rest of the file stays as it was, comments and all. The program's code is never touched.
//! Phai Studio, with the form open, loads it again.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::text::{t, tf};
use crate::theme::{self, State};
use crate::win::{self, Kind};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// The states a look can be changed for; the first is the control as it is.
pub const STATES: [&str; 7] = ["", "hover", "pressed", "focus", "disabled", "checked", "selected"];

/// The look's properties, as CSS names them.
pub const LOOKS: [&str; 10] = ["background", "background-image", "box-shadow", "color", "border-color", "border-width", "border-radius", "font-weight", "accent-color", "icon-color"];

const ID_SAVE: i32 = 1;
const ID_PICK: i32 = 10;
const ID_LIST: i32 = 11;
const ID_STATE: i32 = 12;
const ID_TEXT: i32 = 20;
const ID_ENABLED: i32 = 21;
const ID_VISIBLE: i32 = 22;
const ID_PROP_NAME: i32 = 23;
const ID_PROP_VALUE: i32 = 24;
const ID_PROP_SET: i32 = 25;
const ID_UNDO: i32 = 26;
const ID_LOOK: i32 = 100;
const EN_CHANGE: u16 = 0x0300;
const TIMER_FRAME: usize = 0x1F12;
const TIMER_SHOT: usize = 0x1F13;

// --- the form file -------------------------------------------------------------------------------

/// Where a control's look in its normal state goes in a `.phaiform`: the property, and what its
/// value is (`t` words, `n` a whole number, `b` true or false). `None`: a control of `kind` (the
/// form's name for it) has no such property, and the look is kept for a state only.
pub fn form_key(kind: &str, look: &str) -> Option<(&'static str, char)> {
    Some(match look {
        "background" => ("back_color", 't'),
        "color" => ("fore_color", 't'),
        "border-color" => ("border_color", 't'),
        "border-radius" => ("radius", 'n'),
        "font-weight" => ("bold", 'b'),
        "border-width" => ("border_width", 'n'),
        "background-image" if matches!(kind, "button" | "checkbox" | "radio" | "statusbar") => ("gradient", 't'),
        "box-shadow" if matches!(kind, "button" | "checkbox" | "radio") => ("shadow", 't'),
        "accent-color" if matches!(kind, "checkbox" | "radio" | "tabs" | "progress" | "slider" | "button") => ("accent_color", 't'),
        "icon-color" if kind == "tree" => ("icon_color", 't'),
        _ => return None,
    })
}

/// Words as a TOML string: `"a \"b\""`.
pub fn toml_str(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || c as u32 == 0x7F => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A line's `[`/`{` minus its `]`/`}`, outside strings and comments.
fn depth_change(line: &str) -> i32 {
    let (mut d, mut quote, mut esc) = (0, None::<char>, false);
    for ch in line.chars() {
        match quote {
            Some(q) => {
                if esc {
                    esc = false;
                } else if ch == '\\' && q == '"' {
                    esc = true;
                } else if ch == q {
                    quote = None;
                }
            }
            None => match ch {
                '"' | '\'' => quote = Some(ch),
                '#' => break,
                '[' | '{' => d += 1,
                ']' | '}' => d -= 1,
                _ => {}
            },
        }
    }
    d
}

/// The file's lines with the depth each starts at (inside a value spread over lines: above 0).
fn lines_of(src: &str) -> Vec<(&str, i32)> {
    let mut depth = 0;
    src.split_inclusive('\n')
        .map(|l| {
            let at = depth;
            depth = (depth + depth_change(l)).max(0);
            (l, at)
        })
        .collect()
}

fn key_of(line: &str) -> Option<&str> {
    let (k, _) = line.split_once('=')?;
    let k = k.trim();
    (!k.is_empty() && !k.starts_with('#') && !k.starts_with('[')).then_some(k)
}

fn string_value(line: &str) -> Option<String> {
    let (_, v) = line.split_once('=')?;
    let v = v.trim();
    let v = v.split(" #").next().unwrap_or(v).trim();
    Some(v.trim_matches('"').to_string())
}

/// Every `[[control]]` of a form file: its first line, the line after its last, its id and kind.
fn blocks(lines: &[(&str, i32)]) -> Vec<(usize, usize, String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].1 == 0 && lines[i].0.trim() == "[[control]]" {
            let start = i;
            let mut end = i + 1;
            while end < lines.len() && !(lines[end].1 == 0 && lines[end].0.trim_start().starts_with('[')) {
                end += 1;
            }
            let (mut id, mut kind) = (String::new(), String::new());
            for (l, d) in &lines[start + 1..end] {
                match (d, key_of(l)) {
                    (0, Some("id")) => id = string_value(l).unwrap_or_default(),
                    (0, Some("kind")) => kind = string_value(l).unwrap_or_default(),
                    _ => {}
                }
            }
            out.push((start, end, id, kind));
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// The kind of each control of a form file, by its id.
pub fn form_kinds(src: &str) -> HashMap<String, String> {
    blocks(&lines_of(src)).into_iter().map(|(_, _, id, kind)| (id, kind)).collect()
}

/// `src`, a form file, with control `id`'s `key` set to `value` (as TOML writes it: `"words"`,
/// `12`, `true`) or taken out for `None`. Only that one line changes, or one is added at the end
/// of the control; a value spread over lines is replaced whole. An error when there is no such
/// control.
pub fn set_form_key(src: &str, id: &str, key: &str, value: Option<&str>) -> Result<String, String> {
    let lines = lines_of(src);
    let Some((start, end, _, _)) = blocks(&lines).into_iter().find(|b| b.2 == id) else {
        return Err(tf("The form has no control {0}.", &[&id]));
    };
    let nl = if src.contains("\r\n") { "\r\n" } else { "\n" };
    let found = (start + 1..end).find(|&i| lines[i].1 == 0 && key_of(lines[i].0) == Some(key));
    let mut out: Vec<String> = lines.iter().map(|(l, _)| l.to_string()).collect();
    match found {
        Some(i) => {
            let mut j = i + 1;
            while j < end && lines[j].1 > 0 {
                j += 1;
            }
            let repl = value.map(|v| format!("{key} = {v}{nl}"));
            out.splice(i..j, repl);
        }
        None => {
            let Some(v) = value else { return Ok(src.to_string()) };
            let mut at = end;
            while at > start + 1 && out[at - 1].trim().is_empty() {
                at -= 1;
            }
            if at > 0 && !out[at - 1].ends_with('\n') {
                out[at - 1].push_str(nl);
            }
            out.insert(at, format!("{key} = {v}{nl}"));
        }
    }
    Ok(out.concat())
}

/// `decls` (`background: red; color: white`) with `prop` set to `value`, or taken out for `None`.
pub fn with_decl(decls: &str, prop: &str, value: Option<&str>) -> String {
    let mut out: Vec<String> = decls.split(';').map(str::trim).filter(|d| !d.is_empty() && d.split_once(':').is_none_or(|(k, _)| k.trim() != prop)).map(str::to_string).collect();
    if let Some(v) = value {
        out.push(format!("{prop}: {v}"));
    }
    out.join("; ")
}

/// What `decls` gives `prop`, if anything.
pub fn decl_of(decls: &str, prop: &str) -> Option<String> {
    decls.split(';').filter_map(|d| d.split_once(':')).find(|(k, _)| k.trim() == prop).map(|(_, v)| v.trim().to_string())
}

/// The state rules as `#id:state { decls }` by (id, state); the rest of the sheet as it was.
fn split_rules(css: &str) -> (BTreeMap<(String, String), String>, String) {
    let (mut rules, mut rest) = (BTreeMap::new(), String::new());
    for line in css.lines() {
        let l = line.trim();
        let parsed = l.strip_prefix('#').and_then(|l| l.split_once('{')).and_then(|(sel, body)| {
            let (id, st) = sel.trim().split_once(':')?;
            let body = body.trim().strip_suffix('}')?;
            Some(((id.to_string(), st.to_string()), body.trim().to_string()))
        });
        match parsed {
            Some((k, v)) => {
                rules.insert(k, v);
            }
            None if !l.is_empty() => {
                rest.push_str(line);
                rest.push('\n');
            }
            None => {}
        }
    }
    (rules, rest)
}

fn join_rules(rules: &BTreeMap<(String, String), String>, rest: &str) -> String {
    let mut out = rest.to_string();
    for ((id, st), d) in rules {
        if !d.trim().is_empty() {
            out.push_str(&format!("#{id}:{st} {{ {} }}\n", d.trim()));
        }
    }
    out
}

// --- F12 -----------------------------------------------------------------------------------------

thread_local! {
    /// The windows F12 opens an inspector over, and the form file each is made from.
    static FORMS: RefCell<HashMap<isize, PathBuf>> = RefCell::new(HashMap::new());
    static INSPECTORS: RefCell<HashMap<isize, isize>> = RefCell::new(HashMap::new());
    static KEY_HOOK: Cell<Option<isize>> = const { Cell::new(None) };
    static MOUSE_HOOK: Cell<Option<isize>> = const { Cell::new(None) };
    /// The inspector picking with the mouse, and the control under the pointer.
    static PICKING: Cell<Option<isize>> = const { Cell::new(None) };
    static SWALLOW_UP: Cell<bool> = const { Cell::new(false) };
}

/// F12 (or Ctrl+Shift+I) in window `top` opens the Live Inspector, which saves into `form` - the
/// window's `.phaiform`. The generated code calls it in a debug build.
pub fn attach(top: HWND, form: &Path) {
    FORMS.with(|f| f.borrow_mut().insert(top.0 as isize, form.to_path_buf()));
    if KEY_HOOK.with(Cell::get).is_none() {
        let thread = unsafe { wm::GetWindowThreadProcessId(top, None) };
        if let Ok(h) = unsafe { wm::SetWindowsHookExW(wm::WH_KEYBOARD, Some(key_hook), None, thread) } {
            KEY_HOOK.with(|k| k.set(Some(h.0 as isize)));
        }
    }
    // For checking: `PHAI_INSPECT=id:state;prop=value;...` opens it at once on that control with
    // those looks set; `PHAI_INSPECT_SHOT` photographs it.
    if let Ok(spec) = std::env::var("PHAI_INSPECT") {
        let insp = open(top, form);
        let mut parts = spec.split(';');
        let (id, state) = parts.next().unwrap_or("").split_once(':').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((spec.split(';').next().unwrap_or("").to_string(), String::new()));
        if !id.is_empty() {
            test_select(insp, &id, &state);
        }
        for p in parts {
            if let Some((k, v)) = p.split_once('=') {
                test_set_look(insp, k.trim(), v.trim());
            }
        }
        if std::env::var_os("PHAI_INSPECT_SHOT").is_some() {
            unsafe {
                wm::SetTimer(Some(insp), TIMER_SHOT, 500, None);
            }
        }
    }
}

fn root(h: HWND) -> HWND {
    unsafe { wm::GetAncestor(h, wm::GA_ROOT) }
}

unsafe extern "system" fn key_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    let next = || unsafe { wm::CallNextHookEx(None, code, w, l) };
    let down = (l.0 >> 31) & 1 == 0 && (l.0 >> 30) & 1 == 0;
    if code < 0 || !down {
        return next();
    }
    let vk = w.0 as u16;
    let held = |k: km::VIRTUAL_KEY| unsafe { km::GetKeyState(k.0 as i32) } < 0;
    if vk == km::VK_ESCAPE.0 {
        if let Some(i) = PICKING.with(Cell::get) {
            stop_picking(HWND(i as *mut _));
            return LRESULT(1);
        }
    }
    let f12 = vk == km::VK_F12.0 && !held(km::VK_CONTROL) && !held(km::VK_MENU);
    let ctrl_shift_i = vk == b'I' as u16 && held(km::VK_CONTROL) && held(km::VK_SHIFT);
    if !(f12 || ctrl_shift_i) {
        return next();
    }
    let active = root(unsafe { km::GetActiveWindow() });
    // F12 in the inspector closes it.
    let insp = INSPECTORS.with(|m| m.borrow().iter().find(|(_, i)| **i == active.0 as isize).map(|(_, i)| *i));
    if let Some(i) = insp {
        unsafe {
            let _ = wm::DestroyWindow(HWND(i as *mut _));
        }
        return LRESULT(1);
    }
    let known = FORMS.with(|f| f.borrow().contains_key(&(active.0 as isize)));
    if !known {
        return next();
    }
    toggle(active);
    LRESULT(1)
}

/// Opens the inspector over `top`, or closes the one open; the inspector's window, if open.
pub fn toggle(top: HWND) -> Option<HWND> {
    if let Some(i) = INSPECTORS.with(|m| m.borrow().get(&(top.0 as isize)).copied()) {
        unsafe {
            let _ = wm::DestroyWindow(HWND(i as *mut _));
        }
        return None;
    }
    let form = FORMS.with(|f| f.borrow().get(&(top.0 as isize)).cloned())?;
    Some(open(top, &form))
}

/// The inspector open over `top`, if one is.
pub fn inspector_of(top: HWND) -> Option<HWND> {
    INSPECTORS.with(|m| m.borrow().get(&(top.0 as isize)).map(|i| HWND(*i as *mut _)))
}

// --- picking -------------------------------------------------------------------------------------

/// The smallest control with an id under screen point `pt` in window `top`.
pub fn control_at(top: HWND, pt: POINT) -> Option<HWND> {
    let (mut h, mut best) = (top, None);
    loop {
        let (mut found, mut area) = (None, i64::MAX);
        let mut c = unsafe { wm::GetWindow(h, wm::GW_CHILD) }.ok();
        while let Some(ch) = c.filter(|c| !c.is_invalid()) {
            let mut r = RECT::default();
            if unsafe { wm::IsWindowVisible(ch) }.as_bool() && unsafe { wm::GetWindowRect(ch, &mut r) }.is_ok() && pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom {
                let a = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
                if a < area {
                    area = a;
                    found = Some(ch);
                }
            }
            c = unsafe { wm::GetWindow(ch, wm::GW_HWNDNEXT) }.ok();
        }
        let Some(ch) = found else { break };
        if win::css_names(ch).is_some_and(|(_, _, id)| id.is_some()) {
            best = Some(ch);
        }
        h = ch;
    }
    best
}

fn start_picking(insp: HWND) {
    PICKING.with(|p| p.set(Some(insp.0 as isize)));
    if MOUSE_HOOK.with(Cell::get).is_none() {
        let thread = unsafe { wm::GetWindowThreadProcessId(insp, None) };
        if let Ok(h) = unsafe { wm::SetWindowsHookExW(wm::WH_MOUSE, Some(mouse_hook), None, thread) } {
            MOUSE_HOOK.with(|k| k.set(Some(h.0 as isize)));
        }
    }
    if let Some(e) = unsafe { win::state::<Inspector>(insp) } {
        win::set_text(e.status, t("Point at a control in the window and click it. Esc stops."));
        win::set_text(e.pick, t("Picking..."));
    }
}

fn stop_picking(insp: HWND) {
    PICKING.with(|p| p.set(None));
    if let Some(h) = MOUSE_HOOK.with(|k| k.take()) {
        unsafe {
            let _ = wm::UnhookWindowsHookEx(wm::HHOOK(h as *mut _));
        }
    }
    if let Some(e) = unsafe { win::state::<Inspector>(insp) } {
        win::set_text(e.pick, t("Pick"));
        win::set_text(e.status, "");
        e.frame_on(e.selected());
    }
}

unsafe extern "system" fn mouse_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    let next = || unsafe { wm::CallNextHookEx(None, code, w, l) };
    let Some(insp) = PICKING.with(Cell::get).map(|i| HWND(i as *mut _)) else { return next() };
    if code < 0 {
        return next();
    }
    let m = unsafe { &*(l.0 as *const wm::MOUSEHOOKSTRUCT) };
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return next() };
    let form = e.form;
    if root(m.hwnd) != form {
        return next();
    }
    let msg = w.0 as u32;
    match msg {
        wm::WM_MOUSEMOVE | wm::WM_NCMOUSEMOVE => {
            e.frame_on(control_at(form, m.pt));
            next()
        }
        wm::WM_LBUTTONDOWN | wm::WM_NCLBUTTONDOWN | wm::WM_LBUTTONDBLCLK | wm::WM_RBUTTONDOWN => {
            if let Some(h) = control_at(form, m.pt) {
                e.select_hwnd(h);
            }
            SWALLOW_UP.with(|s| s.set(true));
            stop_picking(insp);
            LRESULT(1)
        }
        wm::WM_LBUTTONUP | wm::WM_RBUTTONUP if SWALLOW_UP.with(|s| s.replace(false)) => LRESULT(1),
        _ => next(),
    }
}

// --- the frame round the control -----------------------------------------------------------------

const KEY: u32 = 0x00FF_00FF;

fn frame_window() -> HWND {
    let class = win::register("PhaiInspectFrame", Some(frame_proc), None);
    let ex = wm::WS_EX_LAYERED | wm::WS_EX_TRANSPARENT | wm::WS_EX_TOOLWINDOW | wm::WS_EX_NOACTIVATE | wm::WS_EX_TOPMOST;
    unsafe {
        let h = wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, 0, 0, 10, 10, None, None, Some(win::instance()), None).unwrap_or_default();
        let _ = wm::SetLayeredWindowAttributes(h, COLORREF(KEY), 235, wm::LWA_COLORKEY | wm::LWA_ALPHA);
        h
    }
}

thread_local! {
    static FRAME_TAG: RefCell<String> = const { RefCell::new(String::new()) };
}

extern "system" fn frame_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == wm::WM_PAINT {
        unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            win::fill(dc, &r, theme::Color { r: 255, g: 0, b: 255, a: 255 });
            let tag = FRAME_TAG.with(|t| t.borrow().clone());
            let blue = theme::Color { r: 0, g: 120, b: 212, a: 255 };
            let th = win::px(h, 18);
            let thick = win::px(h, 2).max(2);
            let body = RECT { top: r.top + th, ..r };
            for side in [RECT { bottom: body.top + thick, ..body }, RECT { top: body.bottom - thick, ..body }, RECT { right: body.left + thick, ..body }, RECT { left: body.right - thick, ..body }] {
                win::fill(dc, &side, blue);
            }
            let f = win::font(h, 8, true);
            let old = Gdi::SelectObject(dc, f.into());
            let mut wtag = win::wide(&tag);
            let n = wtag.len().saturating_sub(1);
            let mut size = windows::Win32::Foundation::SIZE::default();
            let _ = Gdi::GetTextExtentPoint32W(dc, &wtag[..n], &mut size);
            let tr = RECT { left: r.left, top: r.top, right: (r.left + size.cx + win::px(h, 10)).min(r.right.max(r.left + size.cx + win::px(h, 10))), bottom: r.top + th };
            win::fill(dc, &tr, blue);
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, COLORREF(0x00FF_FFFF));
            let mut text_r = RECT { left: tr.left + win::px(h, 5), ..tr };
            Gdi::DrawTextW(dc, &mut wtag[..n], &mut text_r, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteObject(f.into());
            let _ = Gdi::EndPaint(h, &ps);
        }
        return LRESULT(0);
    }
    win::def(h, msg, w, l)
}

// --- the inspector -------------------------------------------------------------------------------

struct Row {
    label: HWND,
    edit: HWND,
    origin: HWND,
}

struct Inspector {
    hwnd: HWND,
    form: HWND,
    path: PathBuf,
    font: HFONT,
    bold: HFONT,
    pick: HWND,
    list: HWND,
    info: HWND,
    state: HWND,
    rows: Vec<Row>,
    text: HWND,
    enabled: HWND,
    visible: HWND,
    prop_name: HWND,
    prop_value: HWND,
    status: HWND,
    frame: HWND,
    /// The form's controls with an id, in its order.
    targets: Vec<(String, HWND)>,
    kinds: HashMap<String, String>,
    sel: Option<usize>,
    /// Each control's own look now, and as it was when the inspector opened.
    own: HashMap<String, String>,
    own_at_open: HashMap<String, String>,
    /// The state rules now and as they were, and the rest of the sheet they were in.
    rules: BTreeMap<(String, String), String>,
    rules_at_open: BTreeMap<(String, String), String>,
    rules_rest: String,
    /// Words, Enabled and Visible changed: (now, as it was).
    texts: BTreeMap<String, (String, String)>,
    flags: BTreeMap<(String, &'static str), (bool, bool)>,
    props: BTreeMap<(String, String), String>,
    filling: bool,
}

fn hex(c: theme::Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

fn state_of(name: &str) -> State {
    State { hover: name == "hover", pressed: name == "pressed", focus: name == "focus", disabled: name == "disabled", checked: name == "checked", selected: name == "selected" }
}

impl Inspector {
    fn selected(&self) -> Option<HWND> {
        self.sel.and_then(|i| self.targets.get(i)).map(|(_, h)| *h)
    }

    fn state_name(&self) -> &'static str {
        STATES[(win::combo_selected(self.state).max(0) as usize).min(STATES.len() - 1)]
    }

    fn frame_on(&self, h: Option<HWND>) {
        let Some(h) = h else {
            win::show(self.frame, false);
            return;
        };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(h, &mut r);
        }
        let (id, kind) = win::css_names(h).map(|(k, _, id)| (id.unwrap_or_default(), k.css())).unwrap_or_default();
        let kind = self.kinds.get(&id).map(String::as_str).unwrap_or(kind);
        let dpi = win::dpi(h);
        let (w, ht) = (crate::layout::unscale(r.right - r.left, dpi), crate::layout::unscale(r.bottom - r.top, dpi));
        FRAME_TAG.with(|t| *t.borrow_mut() = format!("#{id}  {kind}  {w}×{ht}"));
        let th = win::px(h, 18);
        let width = (r.right - r.left).max(win::px(h, 160));
        unsafe {
            let _ = wm::SetWindowPos(self.frame, Some(wm::HWND_TOPMOST), r.left, r.top - th, width, r.bottom - r.top + th, wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
            let _ = Gdi::InvalidateRect(Some(self.frame), None, true);
        }
    }

    fn select_hwnd(&mut self, h: HWND) {
        if let Some(i) = self.targets.iter().position(|(_, x)| *x == h) {
            self.select(i);
        }
    }

    fn select(&mut self, i: usize) {
        if let Some(old) = self.selected() {
            win::set_preview_state(old, None);
        }
        self.sel = Some(i);
        unsafe {
            wm::SendMessageW(self.list, 0x014E /* CB_SETCURSEL */, Some(WPARAM(i)), None);
        }
        self.preview();
        self.fill();
        self.frame_on(self.selected());
    }

    /// The control shows the state its look is being changed for.
    fn preview(&self) {
        if let Some(h) = self.selected() {
            let st = self.state_name();
            win::set_preview_state(h, (!st.is_empty()).then(|| state_of(st)));
        }
    }

    fn id(&self) -> String {
        self.sel.and_then(|i| self.targets.get(i)).map(|(id, _)| id.clone()).unwrap_or_default()
    }

    fn kind(&self) -> String {
        let id = self.id();
        self.kinds.get(&id).cloned().or_else(|| self.selected().and_then(win::kind_of).map(|k| k.css().to_string())).unwrap_or_default()
    }

    /// Every box from the control as it is now - the one being typed in left alone.
    fn fill(&mut self) {
        let Some(h) = self.selected() else { return };
        self.filling = true;
        let focus = unsafe { km::GetFocus() };
        let (kind, classes, id) = win::css_names(h).unwrap_or((Kind::Label, vec![], None));
        let id = id.unwrap_or_default();
        let form_kind = self.kind();
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(h, &mut r);
        }
        let dpi = win::dpi(h);
        let classes = classes.iter().map(|c| format!(".{c}")).collect::<Vec<_>>().join(" ");
        win::set_text(self.info, &format!("#{id}   {form_kind}   {classes}   {}×{}", crate::layout::unscale(r.right - r.left, dpi), crate::layout::unscale(r.bottom - r.top, dpi)));
        let st = self.state_name();
        let state = state_of(st);
        // What it shows in that state, without the preview the inspector gives it.
        win::set_preview_state(h, None);
        let look = win::style_of(h, kind, state);
        let why = win::explain_of(h, kind, state);
        self.preview();
        for (i, p) in LOOKS.iter().enumerate() {
            let row = &self.rows[i];
            let v = match *p {
                "background" => look.background.map(hex),
                "color" => look.color.map(hex),
                "border-color" => look.border_color.map(hex),
                "accent-color" => look.accent_color.map(hex),
                "icon-color" => look.icon_color.map(hex),
                "border-width" => look.border_width.map(|n| n.to_string()),
                "border-radius" => look.border_radius.map(|n| n.to_string()),
                "background-image" => look.gradient.as_ref().map(|g| g.css()),
                "box-shadow" => look.shadow.flatten().map(|s| s.css()),
                _ => look.font_weight.map(|n| if n >= 600 { "bold".to_string() } else { "normal".to_string() }),
            }
            .unwrap_or_default();
            if row.edit != focus {
                win::set_text(row.edit, &v);
            }
            let from = why.iter().find(|o| o.property == *p).map(|o| {
                if o.source == "own" {
                    match o.selector.split_once(':') {
                        Some((_, st)) => format!("{} :{st}", t("own")),
                        None => t("own").to_string(),
                    }
                } else {
                    let file = Path::new(&o.source).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| "theme".into());
                    format!("{}  ·  {}", o.selector, if o.source.is_empty() { "theme".into() } else { file })
                }
            });
            let own = from.as_deref().is_some_and(|f| f.starts_with(t("own")));
            win::set_font(row.label, if own { self.bold } else { self.font });
            win::set_text(row.label, p);
            win::set_text(row.origin, &from.unwrap_or_default());
            // A look its kind keeps for a state only, as the form has no such property.
            let can = !st.is_empty() || form_key(&form_kind, p).is_some();
            win::enable(row.edit, can);
        }
        let has_words = !matches!(kind, Kind::Edit | Kind::Combo | Kind::List | Kind::Tree | Kind::Progress | Kind::Slider | Kind::Date);
        if self.text != focus {
            win::set_text(self.text, &if has_words { win::text_of(h) } else { String::new() });
        }
        win::enable(self.text, has_words);
        win::set_checked(self.enabled, unsafe { km::IsWindowEnabled(h) }.as_bool());
        win::set_checked(self.visible, unsafe { wm::IsWindowVisible(h) }.as_bool());
        self.show_changes();
        self.filling = false;
    }

    fn changes(&self) -> usize {
        let looks: usize = self.own.iter().filter(|(id, v)| self.own_at_open.get(*id) != Some(*v)).count();
        let rules = self.rules.iter().filter(|(k, v)| self.rules_at_open.get(*k) != Some(*v)).count() + self.rules_at_open.keys().filter(|k| !self.rules.contains_key(*k)).count();
        looks + rules + self.texts.len() + self.flags.len() + self.props.len()
    }

    fn show_changes(&self) {
        let n = self.changes();
        if PICKING.with(Cell::get).is_none() {
            win::set_text(self.status, &if n == 0 { t("Changes show in the program at once. Save writes them into the form.").to_string() } else { tf("{0} changed, not saved yet", &[&n.to_string()]) });
        }
    }

    /// The look `prop` of the chosen control, in the chosen state, set to `value` as typed - shown
    /// at once. Nothing when it is no value of that property.
    fn set_look(&mut self, prop: &str, value: &str) -> bool {
        let Some(h) = self.selected() else { return false };
        let id = self.id();
        let v = value.trim();
        let v = if matches!(prop, "border-width" | "border-radius") && v.parse::<i32>().is_ok() { format!("{v}px") } else { v.to_string() };
        if !v.is_empty() && theme::declarations_style(&format!("{prop}: {v}")).is_err() {
            return false;
        }
        let value = (!v.is_empty()).then_some(v.as_str());
        let st = self.state_name();
        if st.is_empty() {
            let now = with_decl(self.own.get(&id).map(String::as_str).unwrap_or(""), prop, value);
            if win::set_own_style(h, &now).is_err() {
                return false;
            }
            self.own.insert(id, now);
        } else {
            let key = (id, st.to_string());
            let now = with_decl(self.rules.get(&key).map(String::as_str).unwrap_or(""), prop, value);
            if now.is_empty() {
                self.rules.remove(&key);
            } else {
                self.rules.insert(key, now);
            }
            if theme::set_state_rules(&join_rules(&self.rules, &self.rules_rest)).is_err() {
                return false;
            }
            win::retheme(self.form);
        }
        unsafe {
            let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_FRAME | Gdi::RDW_ERASE);
        }
        self.fill();
        true
    }

    fn set_words(&mut self, s: &str) {
        let Some(h) = self.selected() else { return };
        let id = self.id();
        let was = self.texts.get(&id).map(|(_, w)| w.clone()).unwrap_or_else(|| win::text_of(h));
        win::set_text(h, s);
        if s == was {
            self.texts.remove(&id);
        } else {
            self.texts.insert(id, (s.to_string(), was));
        }
        self.show_changes();
    }

    fn set_flag(&mut self, which: &'static str, on: bool) {
        let Some(h) = self.selected() else { return };
        let id = self.id();
        let was = self.flags.get(&(id.clone(), which)).map(|(_, w)| *w).unwrap_or(if which == "enabled" { unsafe { km::IsWindowEnabled(h) }.as_bool() } else { unsafe { wm::IsWindowVisible(h) }.as_bool() });
        if which == "enabled" {
            win::enable(h, on);
        } else {
            win::show(h, on);
        }
        if on == was {
            self.flags.remove(&(id, which));
        } else {
            self.flags.insert((id, which), (on, was));
        }
        self.show_changes();
    }

    fn set_prop(&mut self, name: &str, value: &str) -> bool {
        let Some(h) = self.selected() else { return false };
        let name = name.trim();
        let Some((_, ty)) = crate::props::NAMES.iter().find(|(n, _)| *n == name) else {
            win::set_text(self.status, &tf("{0} is not a property the inspector knows.", &[&name]));
            return false;
        };
        let v = value.trim();
        let ok = match ty {
            'b' => matches!(v, "true" | "false"),
            'n' => v.parse::<i32>().is_ok(),
            _ => true,
        };
        if !ok {
            win::set_text(self.status, &format!("{name}: {}", if *ty == 'b' { "true / false" } else { t("a whole number") }));
            return false;
        }
        crate::props::apply(h, name, v);
        self.props.insert((self.id(), name.to_string()), v.to_string());
        self.show_changes();
        true
    }

    /// Everything back as it was when the inspector opened (the properties stay until the
    /// program starts again; they are not saved).
    fn undo(&mut self) {
        for (id, h) in &self.targets {
            if let Some(was) = self.own_at_open.get(id) {
                if self.own.get(id) != Some(was) {
                    let _ = win::set_own_style(*h, was);
                }
            }
            if let Some((_, was)) = self.texts.get(id) {
                win::set_text(*h, was);
            }
            for which in ["enabled", "visible"] {
                if let Some((_, was)) = self.flags.get(&(id.clone(), which)) {
                    if which == "enabled" {
                        win::enable(*h, *was);
                    } else {
                        win::show(*h, *was);
                    }
                }
            }
        }
        self.own = self.own_at_open.clone();
        self.rules = self.rules_at_open.clone();
        let _ = theme::set_state_rules(&join_rules(&self.rules, &self.rules_rest));
        self.texts.clear();
        self.flags.clear();
        self.props.clear();
        win::retheme(self.form);
        self.fill();
    }

    /// The changes written into the form file; how many lines changed.
    fn save(&mut self) -> Result<usize, String> {
        let mut src = std::fs::read_to_string(&self.path).map_err(|e| format!("{}: {e}", self.path.display()))?;
        let before = src.clone();
        let mut n = 0;
        let mut set = |src: &mut String, id: &str, key: &str, v: Option<String>| -> Result<(), String> {
            let now = set_form_key(src, id, key, v.as_deref())?;
            if now != *src {
                n += 1;
                *src = now;
            }
            Ok(())
        };
        for (id, decls) in &self.own {
            let was = self.own_at_open.get(id).map(String::as_str).unwrap_or("");
            let kind = self.kinds.get(id).cloned().unwrap_or_default();
            for p in LOOKS {
                let (a, b) = (decl_of(was, p), decl_of(decls, p));
                if a == b {
                    continue;
                }
                let Some((key, ty)) = form_key(&kind, p) else { continue };
                let v = b.map(|v| match ty {
                    'n' => v.trim_end_matches("px").trim().to_string(),
                    'b' => (v == "bold" || v.parse::<i32>().is_ok_and(|w| w >= 600)).to_string(),
                    _ => toml_str(&v),
                });
                let v = v.filter(|v| v != "false");
                set(&mut src, id, key, v)?;
            }
        }
        let ids: std::collections::BTreeSet<&String> = self.rules.keys().chain(self.rules_at_open.keys()).map(|(id, _)| id).collect();
        for id in ids {
            let now: Vec<(&String, &String)> = self.rules.iter().filter(|((i, _), _)| i == id).map(|((_, s), d)| (s, d)).collect();
            let was: Vec<(&String, &String)> = self.rules_at_open.iter().filter(|((i, _), _)| i == id).map(|((_, s), d)| (s, d)).collect();
            if now == was {
                continue;
            }
            let v = (!now.is_empty()).then(|| format!("{{ {} }}", now.iter().map(|(s, d)| format!("{s} = {}", toml_str(d))).collect::<Vec<_>>().join(", ")));
            set(&mut src, id, "states", v)?;
        }
        for (id, (now, _)) in &self.texts {
            set(&mut src, id, "text", Some(toml_str(now)))?;
        }
        for ((id, which), (now, _)) in &self.flags {
            set(&mut src, id, which, (!*now).then(|| "false".to_string()))?;
        }
        for ((id, name), v) in &self.props {
            let ty = crate::props::NAMES.iter().find(|(n, _)| n == name).map(|(_, t)| *t).unwrap_or('t');
            set(&mut src, id, name, Some(if ty == 't' { toml_str(v) } else { v.clone() }))?;
        }
        if src != before {
            std::fs::write(&self.path, &src).map_err(|e| format!("{}: {e}", self.path.display()))?;
        }
        self.own_at_open = self.own.clone();
        self.rules_at_open = self.rules.clone();
        self.texts.clear();
        self.flags.clear();
        self.props.clear();
        self.kinds = form_kinds(&src);
        Ok(n)
    }

    fn command(&mut self, id: i32, code: u16) {
        if self.filling {
            return;
        }
        match (id, code) {
            (ID_PICK, 0) => {
                if PICKING.with(Cell::get).is_some() {
                    stop_picking(self.hwnd);
                } else {
                    start_picking(self.hwnd);
                }
            }
            (ID_LIST, 1) => {
                let i = win::combo_selected(self.list);
                if i >= 0 {
                    self.select(i as usize);
                }
            }
            (ID_STATE, 1) => {
                self.preview();
                self.fill();
            }
            (i, EN_CHANGE) if (ID_LOOK..ID_LOOK + LOOKS.len() as i32).contains(&i) => {
                let k = (i - ID_LOOK) as usize;
                let text = win::text_of(self.rows[k].edit);
                self.filling = true;
                let shown = self.set_look(LOOKS[k], &text);
                self.filling = false;
                if !shown && !text.trim().is_empty() {
                    win::set_text(self.status, &tf("{0}: not a value yet", &[&LOOKS[k]]));
                }
            }
            (ID_TEXT, EN_CHANGE) => {
                let s = win::text_of(self.text);
                self.set_words(&s);
            }
            (ID_ENABLED, 0) => {
                let on = win::is_checked(self.enabled);
                self.set_flag("enabled", on);
            }
            (ID_VISIBLE, 0) => {
                let on = win::is_checked(self.visible);
                self.set_flag("visible", on);
            }
            (ID_PROP_SET, 0) => {
                let (n, v) = (win::text_of(self.prop_name), win::text_of(self.prop_value));
                self.set_prop(&n, &v);
            }
            (ID_UNDO, 0) => self.undo(),
            (ID_SAVE, 0) => {
                let file = self.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                match self.save() {
                    Ok(0) => win::set_text(self.status, &tf("Nothing to save in {0}.", &[&file])),
                    Ok(n) => win::set_text(self.status, &tf("Saved {0} lines into {1}.", &[&n.to_string(), &file])),
                    Err(e) => win::set_text(self.status, &e),
                }
                self.fill();
            }
            _ => {}
        }
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(e) = (unsafe { win::state::<Inspector>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_COMMAND => {
            e.command(win::loword(w.0) as i32, win::hiword(w.0));
            LRESULT(0)
        }
        #[cfg(feature = "shot")]
        wm::WM_TIMER if w.0 == TIMER_SHOT => {
            unsafe {
                let _ = wm::KillTimer(Some(h), TIMER_SHOT);
            }
            for (var, win) in [("PHAI_INSPECT_SHOT", h), ("PHAI_INSPECT_FORM_SHOT", e.form)] {
                if let Ok(p) = std::env::var(var) {
                    if let Err(e) = crate::shot::save(win, Path::new(&p)) {
                        eprintln!("{e}");
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_FRAME => {
            // The frame follows the control, and shows only while the program is in front.
            if PICKING.with(Cell::get).is_none() {
                let front = root(unsafe { wm::GetForegroundWindow() });
                if front == e.form || front == h {
                    e.frame_on(e.selected());
                } else {
                    win::show(e.frame, false);
                }
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            if PICKING.with(Cell::get) == Some(h.0 as isize) {
                stop_picking(h);
            }
            if let Some(c) = e.selected() {
                win::set_preview_state(c, None);
            }
            unsafe {
                let _ = wm::KillTimer(Some(h), TIMER_FRAME);
                let _ = wm::DestroyWindow(e.frame);
            }
            INSPECTORS.with(|m| m.borrow_mut().remove(&(e.form.0 as isize)));
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(e) = unsafe { win::take_state::<Inspector>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(e.font.into());
                    let _ = Gdi::DeleteObject(e.bold.into());
                }
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The controls of `top` with an id, in the order Windows has them.
fn targets_of(top: HWND) -> Vec<(String, HWND)> {
    unsafe extern "system" fn each(h: HWND, l: LPARAM) -> windows::core::BOOL {
        let out = unsafe { &mut *(l.0 as *mut Vec<(String, HWND)>) };
        if let Some((_, _, Some(id))) = win::css_names(h) {
            if !out.iter().any(|(i, _)| *i == id) {
                out.push((id, h));
            }
        }
        true.into()
    }
    let mut out: Vec<(String, HWND)> = Vec::new();
    unsafe {
        let _ = wm::EnumChildWindows(Some(top), Some(each), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// Opens the inspector over window `top`, saving into the form file `form`; its window.
pub fn open(top: HWND, form: &Path) -> HWND {
    if let Some(i) = inspector_of(top) {
        return i;
    }
    let class = win::register("PhaiInspector", Some(proc_), None);
    let name = form.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let title = win::wide(&tf("Live Inspector - {0}", &[&name]));
    let (style, ex) = crate::form::frame("fixed_tool", false, false, true, false);
    let hwnd = unsafe { wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(top), None, Some(win::instance()), None).unwrap_or_default() };
    let font = win::font(hwnd, 0, false);
    let bold = win::font(hwnd, 0, true);
    let px = |v| win::px(hwnd, v);
    let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
    let targets = targets_of(top);
    let pick = win::button(hwnd, t("Pick"), ID_PICK, font, true);
    at(pick, 11, 11, 90, 28);
    let names: Vec<String> = targets.iter().map(|(id, h)| format!("#{id}   {}", win::kind_of(*h).map(|k| k.css()).unwrap_or(""))).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let list = win::combo(hwnd, ID_LIST, font, &refs);
    crate::props::max_drop_down_items(list, 20);
    at(list, 111, 13, 378, 23);
    let info = win::label(hwnd, "", 0, bold);
    at(info, 11, 48, 478, 23);
    let state_label = win::label(hwnd, t("State"), 0, font);
    at(state_label, 11, 80, 110, 23);
    let st_names: Vec<&str> = STATES.iter().map(|s| if s.is_empty() { t("normal") } else { s }).collect();
    let state = win::combo(hwnd, ID_STATE, font, &st_names);
    at(state, 131, 78, 150, 23);
    let mut rows = Vec::new();
    for (i, _) in LOOKS.iter().enumerate() {
        let y = 112 + i as i32 * 30;
        let label = win::label(hwnd, "", 0, font);
        at(label, 11, y + 2, 118, 23);
        let edit = win::edit(hwnd, "", ID_LOOK + i as i32, font, false);
        at(edit, 131, y, 170, 25);
        let origin = win::label(hwnd, "", 0, font);
        win::set_css(origin, "muted", "");
        at(origin, 309, y + 2, 180, 23);
        rows.push(Row { label, edit, origin });
    }
    let y = 112 + LOOKS.len() as i32 * 30 + 8;
    at(win::label(hwnd, t("Text"), 0, font), 11, y + 2, 118, 23);
    let text = win::edit(hwnd, "", ID_TEXT, font, false);
    at(text, 131, y, 358, 25);
    let enabled = win::checkbox(hwnd, t("Enabled"), ID_ENABLED, font, true);
    at(enabled, 131, y + 34, 120, 24);
    let visible = win::checkbox(hwnd, t("Visible"), ID_VISIBLE, font, true);
    at(visible, 261, y + 34, 120, 24);
    at(win::label(hwnd, t("Property"), 0, font), 11, y + 70, 118, 23);
    let pnames: Vec<&str> = crate::props::NAMES.iter().map(|(n, _)| *n).collect();
    let prop_name = win::combo_with(hwnd, ID_PROP_NAME, font, &pnames, true);
    win::set_text(prop_name, "");
    crate::props::max_drop_down_items(prop_name, 16);
    at(prop_name, 131, y + 68, 170, 23);
    let prop_value = win::edit(hwnd, "", ID_PROP_VALUE, font, false);
    at(prop_value, 309, y + 68, 110, 25);
    at(win::button(hwnd, t("Set"), ID_PROP_SET, font, false), 425, y + 67, 64, 27);
    let status = win::label(hwnd, "", 0, font);
    win::set_css(status, "muted", "");
    at(status, 11, y + 106, 478, 40);
    let undo = win::button(hwnd, t("Undo all"), ID_UNDO, font, false);
    at(undo, 274, y + 152, 105, 28);
    at(win::button(hwnd, t("Save to form"), ID_SAVE, font, true), 389, y + 152, 100, 28);
    let height = y + 192;
    let kinds = std::fs::read_to_string(form).map(|s| form_kinds(&s)).unwrap_or_default();
    let (rules, rules_rest) = split_rules(&theme::state_rules_css());
    let own: HashMap<String, String> = targets.iter().map(|(id, h)| (id.clone(), win::own_style_text(*h))).collect();
    let frame = frame_window();
    let mut e = Box::new(Inspector {
        hwnd,
        form: top,
        path: form.to_path_buf(),
        font,
        bold,
        pick,
        list,
        info,
        state,
        rows,
        text,
        enabled,
        visible,
        prop_name,
        prop_value,
        status,
        frame,
        targets,
        kinds,
        sel: None,
        own_at_open: own.clone(),
        own,
        rules_at_open: rules.clone(),
        rules,
        rules_rest,
        texts: BTreeMap::new(),
        flags: BTreeMap::new(),
        props: BTreeMap::new(),
        filling: false,
    });
    // It starts on the control that has the keys, or the first.
    let focus = unsafe { km::GetFocus() };
    let first = e.targets.iter().position(|(_, h)| *h == focus || unsafe { wm::IsChild(*h, focus) }.as_bool()).unwrap_or(0);
    if !e.targets.is_empty() {
        e.sel = Some(first);
        unsafe {
            wm::SendMessageW(list, 0x014E, Some(WPARAM(first)), None);
        }
        e.fill();
    }
    win::set_state(hwnd, e);
    INSPECTORS.with(|m| m.borrow_mut().insert(top.0 as isize, hwnd.0 as isize));
    win::theme_frame(hwnd);
    unsafe {
        let s = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let mut r = RECT { left: 0, top: 0, right: px(500), bottom: px(height) };
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(s), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let (w, ht) = (r.right - r.left, r.bottom - r.top);
        // Beside the window: on its right if the screen has room, else on its left.
        let mut f = RECT::default();
        let _ = wm::GetWindowRect(top, &mut f);
        let screen = wm::GetSystemMetrics(wm::SM_CXVIRTUALSCREEN) + wm::GetSystemMetrics(wm::SM_XVIRTUALSCREEN);
        let x = if f.right + w <= screen { f.right } else { (f.left - w).max(0) };
        let _ = wm::SetWindowPos(hwnd, None, x, f.top, w, ht, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        wm::SetTimer(Some(hwnd), TIMER_FRAME, 250, None);
    }
    // Beside the program, which keeps the keys: F12 there closes it again.
    unsafe {
        let _ = wm::ShowWindow(hwnd, wm::SW_SHOWNOACTIVATE);
    }
    if let Some(e) = unsafe { win::state::<Inspector>(hwnd) } {
        e.frame_on(e.selected());
    }
    hwnd
}

// --- for checking --------------------------------------------------------------------------------

/// The open inspector `insp` on control `id` and `state` (`""` normal).
pub fn test_select(insp: HWND, id: &str, state: &str) -> bool {
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return false };
    let Some(i) = e.targets.iter().position(|(x, _)| x == id) else { return false };
    unsafe {
        wm::SendMessageW(e.state, 0x014E, Some(WPARAM(STATES.iter().position(|s| *s == state).unwrap_or(0))), None);
    }
    e.select(i);
    true
}

/// The look `prop` set to `value` on the control and state chosen, as typed in its box.
pub fn test_set_look(insp: HWND, prop: &str, value: &str) -> bool {
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return false };
    e.set_look(prop, value)
}

/// The chosen control's words, Enabled (`enabled`), Visible (`visible`) or a property.
pub fn test_set(insp: HWND, what: &str, value: &str) -> bool {
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return false };
    match what {
        "text" => e.set_words(value),
        "enabled" | "visible" => e.set_flag(if what == "enabled" { "enabled" } else { "visible" }, value == "true"),
        _ => return e.set_prop(what, value),
    }
    true
}

/// Save to form; how many lines changed.
pub fn test_save(insp: HWND) -> Result<usize, String> {
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return Err("no inspector".into()) };
    e.save()
}

/// Where the inspector says the chosen control's `prop` comes from, as it shows it.
pub fn test_origin(insp: HWND, prop: &str) -> String {
    let Some(e) = (unsafe { win::state::<Inspector>(insp) }) else { return String::new() };
    LOOKS.iter().position(|p| *p == prop).map(|i| win::text_of(e.rows[i].origin)).unwrap_or_default()
}

/// The control under screen point (`x`, `y`) of `top` as picking finds it - its id.
pub fn test_pick(insp: HWND, x: i32, y: i32) -> Option<String> {
    let e = unsafe { win::state::<Inspector>(insp) }?;
    let h = control_at(e.form, POINT { x, y })?;
    e.select_hwnd(h);
    Some(e.id())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORM: &str = "# a comment\nformat = 1\n\n[form]\nname = \"F\"\n\n[[control]]\nkind = \"button\"\nid = \"go\"\ntext = \"Go\"  # the words\nitems = [\n  \"a\",\n  \"b\",\n]\nbounds = [1, 2, 3, 4]\n\n[[control]]\nkind = \"label\"\nid = \"note\"\n";

    #[test]
    fn a_key_changes_on_its_own_line() {
        let s = set_form_key(FORM, "go", "text", Some("\"Start\"")).unwrap();
        assert!(s.contains("text = \"Start\"\n") && s.contains("# a comment") && !s.contains("the words"));
        assert_eq!(s.lines().count(), FORM.lines().count());
    }

    #[test]
    fn a_new_key_goes_at_the_end_of_its_control() {
        let s = set_form_key(FORM, "go", "back_color", Some("\"#ff0000\"")).unwrap();
        assert!(s.contains("bounds = [1, 2, 3, 4]\nback_color = \"#ff0000\"\n\n[[control]]"), "{s}");
        let s = set_form_key(FORM, "note", "bold", Some("true")).unwrap();
        assert!(s.ends_with("id = \"note\"\nbold = true\n"), "{s}");
    }

    #[test]
    fn a_value_over_lines_goes_whole_and_others_stay() {
        let s = set_form_key(FORM, "go", "items", Some("[\"x\"]")).unwrap();
        assert!(s.contains("items = [\"x\"]\nbounds"), "{s}");
        let s = set_form_key(FORM, "go", "items", None).unwrap();
        assert!(!s.contains("items") && !s.contains("\"a\","), "{s}");
        // A line inside the array that starts with [ is not a table.
        let f = FORM.replace("  \"a\",", "  [\"a\", 1],");
        assert_eq!(form_kinds(&f).get("go").map(String::as_str), Some("button"));
        assert!(set_form_key(FORM, "nobody", "text", Some("\"x\"")).is_err());
    }

    #[test]
    fn crlf_files_stay_crlf() {
        let f = FORM.replace('\n', "\r\n");
        let s = set_form_key(&f, "note", "text", Some("\"Hi\"")).unwrap();
        assert!(s.contains("id = \"note\"\r\ntext = \"Hi\"\r\n") && !s.replace("\r\n", "").contains('\n'), "{s:?}");
    }

    #[test]
    fn words_are_toml_strings() {
        assert_eq!(toml_str("a \"b\" \\ c\nd"), "\"a \\\"b\\\" \\\\ c\\nd\"");
        assert_eq!(toml_str("ไทย"), "\"ไทย\"");
    }

    #[test]
    fn state_rules_split_and_join() {
        let (r, rest) = split_rules("#go:hover { background: red }\n#go:pressed { color: blue }\nbutton { color: red }\n");
        assert_eq!(r.get(&("go".into(), "hover".into())).map(String::as_str), Some("background: red"));
        assert_eq!(rest, "button { color: red }\n");
        assert!(join_rules(&r, &rest).contains("#go:pressed { color: blue }"));
    }

    #[test]
    fn looks_go_to_their_properties() {
        assert_eq!(form_key("label", "background"), Some(("back_color", 't')));
        assert_eq!(form_key("label", "box-shadow"), None);
        assert_eq!(form_key("button", "box-shadow"), Some(("shadow", 't')));
        assert_eq!(with_decl("background: red; color: blue", "color", Some("white")), "background: red; color: white");
        assert_eq!(decl_of("background: red; color: blue", "color").as_deref(), Some("blue"));
    }
}
