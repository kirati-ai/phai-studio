//! The form's toolbar designed on the form, as Visual Studio's ToolStrip designer: its buttons
//! over the form's own toolbar, an add button at the end (a click adds a button, its arrow offers
//! Button, Separator and the standard buttons); a click chooses one, a double-click opens its
//! Click method; a right-click has Properties, Set Image, Enabled, CheckOnClick, Checked, Insert,
//! Move and Delete; Left/Right go along, Ctrl+Left/Right move, Delete takes one out. Every change
//! is the form's at once, one undo step each; a click anywhere else closes it.

use phai_form::Tool;
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::studio;

/// The colour that is not there: clicks go through to the form.
const CLEAR: COLORREF = COLORREF(0x00FF00FF);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Hit {
    Tool(usize),
    /// The add button: its main part, and its arrow.
    Add,
    AddMore,
}

struct Designer {
    tools: Vec<Tool>,
    sel: Option<usize>,
    /// The strip on the screen: where the form's toolbar is (or will be).
    strip: RECT,
    cells: Vec<(RECT, Hit)>,
    busy: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Designer> {
    unsafe { win::state::<Designer>(h) }
}

static OPEN: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Opens the designer over the form's toolbar, button `chosen` chosen.
pub fn open(chosen: Option<usize>) {
    let Some(s) = studio() else { return };
    let Some(c) = s.canvas.as_ref() else { return };
    let old = HWND(OPEN.load(std::sync::atomic::Ordering::Relaxed) as *mut _);
    if !old.is_invalid() && unsafe { wm::IsWindow(Some(old)) }.as_bool() {
        unsafe {
            let _ = wm::DestroyWindow(old);
        }
    }
    let tools = s.doc.form.toolbar.clone();
    let strip = c.toolbar_screen(&s.doc.form);
    win::init();
    // Double-clicks heard: one on a button opens its Click method.
    let class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiToolDesigner", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(wm::WS_EX_LAYERED | wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, strip.left, strip.top, strip.right - strip.left, strip.bottom - strip.top, Some(s.hwnd), None, Some(win::instance()), None).unwrap_or_default();
        let _ = wm::SetLayeredWindowAttributes(h, CLEAR, 0, wm::LWA_COLORKEY);
        let sel = chosen.filter(|i| *i < tools.len());
        win::set_state(h, Box::new(Designer { tools, sel, strip, cells: vec![], busy: false, done: false }));
        OPEN.store(h.0 as isize, std::sync::atomic::Ordering::Relaxed);
        relayout(h);
        let _ = wm::ShowWindow(h, wm::SW_SHOW);
        let _ = km::SetFocus(Some(h));
        say(h);
    }
}

fn px(h: HWND, v: i32) -> i32 {
    win::px(h, v)
}

/// The buttons' places along the strip, the add button after them.
fn relayout(h: HWND) {
    let Some(d) = state(h) else { return };
    let s = d.strip;
    let (w, sep) = (px(h, 32), px(h, 10));
    let mut x = s.left + px(h, 4);
    let mut cells = Vec::new();
    for (i, tool) in d.tools.iter().enumerate() {
        let cw = if tool.separator { sep } else { w };
        cells.push((RECT { left: x, top: s.top + px(h, 2), right: x + cw, bottom: s.bottom - px(h, 2) }, Hit::Tool(i)));
        x += cw + px(h, 1);
    }
    x += px(h, 4);
    cells.push((RECT { left: x, top: s.top + px(h, 4), right: x + px(h, 26), bottom: s.bottom - px(h, 4) }, Hit::Add));
    cells.push((RECT { left: x + px(h, 26), top: s.top + px(h, 4), right: x + px(h, 42), bottom: s.bottom - px(h, 4) }, Hit::AddMore));
    d.cells = cells;
    let right = (x + px(h, 46)).max(s.right);
    unsafe {
        let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), s.left, s.top, right - s.left, s.bottom - s.top, wm::SWP_NOACTIVATE);
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn paint(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut ps = Gdi::PAINTSTRUCT::default();
    let dc = unsafe { Gdi::BeginPaint(h, &mut ps) };
    let mut cr = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut cr);
    }
    let mem = unsafe { Gdi::CreateCompatibleDC(Some(dc)) };
    let bmp = unsafe { Gdi::CreateCompatibleBitmap(dc, cr.right, cr.bottom) };
    let old = unsafe { Gdi::SelectObject(mem, bmp.into()) };
    unsafe {
        let b = Gdi::CreateSolidBrush(CLEAR);
        Gdi::FillRect(mem, &cr, b);
        let _ = Gdi::DeleteObject(b.into());
    }
    let window = win::window_background();
    let text = win::window_text();
    let grey = Color::rgb(150, 150, 150);
    let border = theme::style("group", phai_ui::theme::State::default()).border_or(Color::rgb(212, 212, 212));
    let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
    let tint = if window.is_dark() { Color::rgb(40, 64, 56) } else { Color::rgb(231, 243, 239) };
    let local = |r: RECT| RECT { left: r.left - d.strip.left, top: r.top - d.strip.top, right: r.right - d.strip.left, bottom: r.bottom - d.strip.top };
    // The strip itself, over the form's toolbar.
    win::fill(mem, &RECT { left: 0, top: 0, right: d.strip.right - d.strip.left, bottom: cr.bottom }, window);
    win::fill(mem, &RECT { left: 0, top: cr.bottom - 1, right: d.strip.right - d.strip.left, bottom: cr.bottom }, border);
    let glyph = |g: u16, r: RECT, colour: Color| {
        let size = px(h, 16);
        if let Some(i) = phai_ui::icon::glyph_icon(g, size, colour) {
            unsafe {
                let _ = wm::DrawIconEx(mem, r.left + (r.right - r.left - size) / 2, r.top + (r.bottom - r.top - size) / 2, i, size, size, 0, None, wm::DI_NORMAL);
                let _ = wm::DestroyIcon(i);
            }
        }
    };
    for (r, hit) in &d.cells {
        let r = local(*r);
        match hit {
            Hit::Tool(i) => {
                let tool = &d.tools[*i];
                let chosen = d.sel == Some(*i);
                if chosen || (tool.toggle && tool.checked) {
                    win::fill(mem, &r, if chosen { accent } else { border });
                    win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 }, tint);
                }
                if tool.separator {
                    let x = (r.left + r.right) / 2;
                    win::fill(mem, &RECT { left: x, top: r.top + px(h, 4), right: x + 1, bottom: r.bottom - px(h, 4) }, border);
                } else {
                    let g = phai_ui::icon::glyph::by_name(&tool.glyph).unwrap_or(0xE8A5);
                    glyph(g, r, if tool.disabled { grey } else { text });
                }
            }
            Hit::Add => {
                win::fill(mem, &r, border);
                win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right, bottom: r.bottom - 1 }, window);
                glyph(0xE710, r, grey);
            }
            Hit::AddMore => {
                win::fill(mem, &r, border);
                win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 }, window);
                glyph(0xE70D, r, grey);
            }
        }
    }
    unsafe {
        let _ = Gdi::BitBlt(dc, 0, 0, cr.right, cr.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(mem, old);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(mem);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

/// The chosen button on the status bar: its id, words and picture.
fn say(h: HWND) {
    let Some(d) = state(h) else { return };
    let words = match d.sel.and_then(|i| d.tools.get(i)) {
        Some(tool) if tool.separator => t("Separator").to_string(),
        Some(tool) => tf("{0} - {1} ({2}) - double-click for its code, right-click for more", &[&tool.id, &tool.label, &tool.glyph]),
        None => t("Toolbar: + adds a button; its arrow adds a separator or the standard buttons").to_string(),
    };
    if let Some(s) = studio() {
        s.say_status(&words);
    }
}

/// The toolbar the form's now - one undo step - drawn again.
fn apply(h: HWND) {
    let Some(d) = state(h) else { return };
    let tools = d.tools.clone();
    d.busy = true;
    if let Some(s) = studio() {
        if s.doc.form.toolbar != tools {
            s.edit(|f| f.toolbar = tools);
            s.rebuild();
            if let Some(c) = s.canvas.as_ref() {
                d.strip = c.toolbar_screen(&s.doc.form);
            }
        }
    }
    unsafe {
        let _ = km::SetActiveWindow(h);
        let _ = km::SetFocus(Some(h));
    }
    d.busy = false;
    relayout(h);
    say(h);
}

fn close(h: HWND) {
    if let Some(d) = state(h) {
        if d.done {
            return;
        }
        d.done = true;
    }
    OPEN.store(0, std::sync::atomic::Ordering::Relaxed);
    unsafe {
        let _ = wm::DestroyWindow(h);
    }
}

fn taken(d: &Designer) -> Vec<String> {
    let mut out: Vec<String> = d.tools.iter().map(|t| t.id.clone()).filter(|i| !i.is_empty()).collect();
    if let Some(s) = studio() {
        out.extend(s.doc.form.control.iter().map(|c| c.id.clone()));
        out.extend(s.doc.form.menu.iter().flat_map(|m| m.all_items()).filter_map(|it| it.id.clone()));
    }
    out
}

/// A new button (or separator) after the chosen one - at the end when none is chosen.
fn add(h: HWND, separator: bool) {
    let Some(d) = state(h) else { return };
    let at = d.sel.map(|i| i + 1).unwrap_or(d.tools.len()).min(d.tools.len());
    let tool = if separator {
        Tool { separator: true, ..Default::default() }
    } else {
        let id = crate::menudesign::new_id("button", &taken(d));
        Tool { id, label: t("Button").to_string(), glyph: "BUTTON".into(), ..Default::default() }
    };
    d.tools.insert(at, tool);
    d.sel = Some(at);
    apply(h);
    if !separator {
        properties(h);
    }
}

/// The standard buttons, as Visual Studio's Insert Standard Items: New, Open, Save | Cut, Copy,
/// Paste | Undo, Redo - those not there yet.
fn standard(h: HWND) {
    let Some(d) = state(h) else { return };
    let set: [&[(&str, &str, &str)]; 3] = [&[("new_file", "New", "NEW"), ("open_file", "Open", "OPEN"), ("save_file", "Save", "SAVE")], &[("cut", "Cut", "CUT"), ("copy", "Copy", "COPY"), ("paste", "Paste", "PASTE")], &[("undo", "Undo", "UNDO"), ("redo", "Redo", "REDO")]];
    for (k, group) in set.iter().enumerate() {
        if k > 0 || !d.tools.is_empty() {
            d.tools.push(Tool { separator: true, ..Default::default() });
        }
        for (id, label, glyph) in group.iter() {
            if d.tools.iter().any(|t| t.id == *id) {
                continue;
            }
            d.tools.push(Tool { id: id.to_string(), label: label.to_string(), glyph: glyph.to_string(), ..Default::default() });
        }
    }
    apply(h);
}

fn delete(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(i) = d.sel.filter(|i| *i < d.tools.len()) else { return };
    d.tools.remove(i);
    d.sel = if d.tools.is_empty() { None } else { Some(i.min(d.tools.len() - 1)) };
    apply(h);
}

fn shift(h: HWND, right: bool) {
    let Some(d) = state(h) else { return };
    let Some(i) = d.sel else { return };
    let j = if right { i + 1 } else { i.wrapping_sub(1) };
    if j < d.tools.len() {
        d.tools.swap(i, j);
        d.sel = Some(j);
        apply(h);
    }
}

/// The chosen button's properties: id (its Click method), words (its tooltip), picture, and
/// whether it stays down, starts down or is greyed.
fn properties(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(i) = d.sel else { return };
    let Some(tool) = d.tools.get(i).cloned().filter(|t| !t.separator) else { return };
    d.busy = true;
    use phai_ui::dialogs::Field;
    let answer = phai_ui::dialogs::ask_fields(
        Some(h),
        &tf("Toolbar button {0}", &[&tool.label]),
        &[
            Field::Text(t("Id (its Click method)"), &tool.id),
            Field::Text(t("Text (its tooltip)"), &tool.label),
            Field::Text(t("Picture (a glyph's name, e.g. SAVE)"), &tool.glyph),
            Field::Check(t("Stays down when clicked (CheckOnClick)"), tool.toggle),
            Field::Check(t("Down when the window opens (Checked)"), tool.checked),
            Field::Check(t("Disabled"), tool.disabled),
        ],
    );
    let Some(d) = state(h) else { return };
    d.busy = false;
    let Some(a) = answer else { return };
    let id = a[0].trim().to_string();
    let glyph = a[2].trim().to_uppercase();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || id.starts_with(|c: char| c.is_ascii_digit()) {
        win::message(Some(h), &tf("{0}: an id is snake_case, like save_as", &[&id]), t("Toolbar"), wm::MB_ICONWARNING);
        return;
    }
    if phai_ui::icon::glyph::by_name(&glyph).is_none() {
        let names: Vec<&str> = phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| *n).collect();
        win::message(Some(h), &tf("{0} is not a glyph - one of {1}", &[&glyph, &names.join(", ")]), t("Toolbar"), wm::MB_ICONWARNING);
        return;
    }
    let x = &mut d.tools[i];
    x.id = id;
    x.label = Some(a[1].trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| x.label.clone());
    x.glyph = glyph;
    x.toggle = a[3] == "true";
    x.checked = a[4] == "true" && x.toggle;
    x.disabled = a[5] == "true";
    apply(h);
}

/// Set Image: the pictures to choose from, as a menu.
fn set_image(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(i) = d.sel.filter(|i| d.tools.get(*i).is_some_and(|t| !t.separator)) else { return };
    let names: Vec<&str> = phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| *n).filter(|n| !n.starts_with("LAY_")).collect();
    let items: Vec<(u32, &str)> = names.iter().enumerate().map(|(k, n)| (k as u32 + 1, *n)).collect();
    d.busy = true;
    let chosen = win::popup(h, &items);
    let Some(d) = state(h) else { return };
    d.busy = false;
    if chosen > 0 {
        d.tools[i].glyph = names[chosen as usize - 1].to_string();
        apply(h);
    }
}

fn right_click(h: HWND, hit: Option<Hit>) {
    const PROPS: u32 = 1;
    const IMAGE: u32 = 2;
    const ENABLED: u32 = 3;
    const TOGGLE: u32 = 4;
    const CHECKED: u32 = 5;
    const BUTTON: u32 = 6;
    const SEP: u32 = 7;
    const LEFT: u32 = 8;
    const RIGHT: u32 = 9;
    const DELETE: u32 = 10;
    const STANDARD: u32 = 11;
    const CODE: u32 = 12;
    let Some(d) = state(h) else { return };
    if let Some(Hit::Tool(i)) = hit {
        d.sel = Some(i);
        relayout(h);
        say(h);
    }
    let tool = d.sel.and_then(|i| d.tools.get(i)).cloned();
    let mut items: Vec<(u32, &str)> = Vec::new();
    let enabled_label = if tool.as_ref().is_some_and(|t| t.disabled) { t("Enable") } else { t("Disable") };
    let toggle_label = if tool.as_ref().is_some_and(|t| t.toggle) { t("Stop staying down") } else { t("Stay down when clicked") };
    let checked_label = if tool.as_ref().is_some_and(|t| t.checked) { t("Start up") } else { t("Start down") };
    match &tool {
        Some(t0) if !t0.separator => {
            items.extend([(CODE, t("View Code")), (PROPS, t("Properties...")), (IMAGE, t("Set Image...")), (0, ""), (ENABLED, enabled_label), (TOGGLE, toggle_label)]);
            if t0.toggle {
                items.push((CHECKED, checked_label));
            }
            items.push((0, ""));
        }
        _ => {}
    }
    items.extend([(BUTTON, t("Insert Button")), (SEP, t("Insert Separator")), (STANDARD, t("Insert Standard Items"))]);
    if tool.is_some() {
        items.extend([(0, ""), (LEFT, t("Move Left")), (RIGHT, t("Move Right")), (0, ""), (DELETE, t("Delete"))]);
    }
    d.busy = true;
    let chosen = win::popup(h, &items);
    if let Some(d) = state(h) {
        d.busy = false;
    }
    let toggle_field = |h: HWND, f: fn(&mut Tool)| {
        if let Some(d) = state(h) {
            if let Some(x) = d.sel.and_then(|i| d.tools.get_mut(i)) {
                f(x);
                apply(h);
            }
        }
    };
    match chosen {
        CODE => view_code(h),
        PROPS => properties(h),
        IMAGE => set_image(h),
        ENABLED => toggle_field(h, |x| x.disabled = !x.disabled),
        TOGGLE => toggle_field(h, |x| {
            x.toggle = !x.toggle;
            x.checked &= x.toggle;
        }),
        CHECKED => toggle_field(h, |x| x.checked = !x.checked),
        BUTTON => add(h, false),
        SEP => add(h, true),
        STANDARD => standard(h),
        LEFT => shift(h, false),
        RIGHT => shift(h, true),
        DELETE => delete(h),
        _ => {}
    }
}

/// The chosen button's Click method, written if it is not there, opened - as Visual Studio's
/// double-click on a button.
fn view_code(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(id) = d.sel.and_then(|i| d.tools.get(i)).filter(|t| !t.separator).map(|t| t.id.clone()) else { return };
    close(h);
    if let Some(s) = studio() {
        s.open_item_handler(&id);
    }
}

fn hit(h: HWND, l: LPARAM) -> Option<Hit> {
    let d = state(h)?;
    let (x, y) = win::point_of(l);
    let p = POINT { x: x + d.strip.left, y: y + d.strip.top };
    d.cells.iter().find(|(r, _)| p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom).map(|(_, hit)| *hit)
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_MOUSEACTIVATE => LRESULT(1),
        wm::WM_LBUTTONDOWN => {
            match hit(h, l) {
                Some(Hit::Tool(i)) => {
                    d.sel = Some(i);
                    relayout(h);
                    say(h);
                }
                Some(Hit::Add) => add(h, false),
                Some(Hit::AddMore) => {
                    const BUTTON: u32 = 1;
                    const SEP: u32 = 2;
                    const STANDARD: u32 = 3;
                    d.busy = true;
                    let chosen = win::popup(h, &[(BUTTON, t("Button")), (SEP, t("Separator")), (0, ""), (STANDARD, t("Insert Standard Items"))]);
                    if let Some(d) = state(h) {
                        d.busy = false;
                    }
                    match chosen {
                        BUTTON => add(h, false),
                        SEP => add(h, true),
                        STANDARD => standard(h),
                        _ => {}
                    }
                }
                // On the strip but on no button: none chosen; off it: done.
                None => {
                    let (x, _) = win::point_of(l);
                    if x + d.strip.left < d.strip.right {
                        d.sel = None;
                        relayout(h);
                        say(h);
                    } else {
                        close(h);
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONDBLCLK => {
            if let Some(Hit::Tool(i)) = hit(h, l) {
                d.sel = Some(i);
                view_code(h);
            }
            LRESULT(0)
        }
        wm::WM_RBUTTONUP => {
            right_click(h, hit(h, l));
            LRESULT(0)
        }
        wm::WM_KEYDOWN => {
            let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
            match w.0 {
                0x1B => close(h),
                0x2E => delete(h),
                0x0D | 0x73 => properties(h),
                0x25 | 0x27 if ctrl => shift(h, w.0 == 0x27),
                0x25 | 0x27 => {
                    let n = d.tools.len();
                    if n > 0 {
                        let i = d.sel.unwrap_or(0);
                        d.sel = Some(if w.0 == 0x27 { (i + 1).min(n - 1) } else { i.saturating_sub(1) });
                        relayout(h);
                        say(h);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_ACTIVATE if win::loword(w.0) == 0 => {
            if !d.busy {
                unsafe {
                    let _ = wm::PostMessageW(Some(h), wm::WM_APP + 2, WPARAM(0), LPARAM(0));
                }
            }
            LRESULT(0)
        }
        m if m == wm::WM_APP + 2 => {
            if unsafe { km::GetActiveWindow() } != h && !d.busy {
                close(h);
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            unsafe {
                let _ = win::take_state::<Designer>(h);
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
