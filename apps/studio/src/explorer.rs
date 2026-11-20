//! The project explorer: the project's files as a tree, on the right above the properties, as
//! Visual Studio has its Solution Explorer. Double-click a form to open it in a tab; anything
//! else opens the way Windows opens it. Right-click for a new form.

use std::path::{Path, PathBuf};

use phai_form::codegen;
use phai_form::{snake, FormFile};
use phai_ui::icon::glyph;
use phai_ui::layout::{Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::studio;

/// After a click in the tree: the file it chose, in the preview tab.
/// A table of Server Explorer being dragged towards the designer.
static DRAGGING_TABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the screen point is over the form being designed.
fn over_designer(pt: windows::Win32::Foundation::POINT) -> bool {
    let Some(c) = studio().and_then(|s| s.canvas.as_ref()) else { return false };
    let hit = unsafe { wm::WindowFromPoint(pt) };
    hit == c.hwnd || unsafe { wm::IsChild(c.hwnd, hit) }.as_bool()
}

const WM_PREVIEW_SELECTED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 0x0E20;
const ID_TREE: i32 = 400;
const CMD_NEW_FORM: u32 = 1;
const CMD_SHOW: u32 = 2;
const CMD_REFRESH: u32 = 3;
/// New form from a template: `CMD_TEMPLATE + n` in `Template::ALL` order.
const CMD_TEMPLATE: u32 = 10;

/// Icons, by index: folder, file, form, Rust source, the project, the theme, dependencies, a crate.
const ICONS: [u16; 8] = [glyph::FOLDER, glyph::DOCUMENT, glyph::FORM, glyph::CODE, glyph::LIBRARY, glyph::BRIGHTNESS, 0xE71D, 0xE7B8];
/// The explorer's bar, as Solution Explorer's: its buttons, and the search box under them.
const ID_EXPLORER_BAR: i32 = 420;
const ID_EXPLORER_SEARCH: i32 = 421;
const BAR_NEW_FORM: i32 = 422;
const BAR_REFRESH: i32 = 423;
const BAR_COLLAPSE: i32 = 424;
const BAR_FOLDER: i32 = 425;

pub struct Explorer {
    pub hwnd: HWND,
    pub tree: HWND,
    pub bar: HWND,
    pub search: HWND,
    root: Option<PathBuf>,
    /// The search box's words: only files whose names have them show.
    filter: String,
    /// The path behind each item, by the number kept with it.
    entries: Vec<PathBuf>,
}

const ID_EXPLORER_CAPTION: i32 = 410;
const ID_PROPS_CAPTION: i32 = 411;
const ID_SIDE_SPLIT: i32 = 412;
const ID_SERVERS_CAPTION: i32 = 413;
const ID_SIDE_SPLIT2: i32 = 414;
const ID_OUTLINE_CAPTION: i32 = 415;
const ID_SIDE_SPLIT3: i32 = 416;

/// The right-hand side: two docked panes, the project explorer above the properties, each with
/// its caption and close button, a splitter between them.
struct Side {
    layout: Layout,
    parts: Option<SideParts>,
    /// The explorer pane's height, design pixels.
    explorer_h: i32,
    /// Server Explorer's height when a pane lies below it.
    servers_h: i32,
    pub explorer_on: bool,
    pub props_on: bool,
    pub servers_on: bool,
    /// Document Outline: shown, and its height when a pane lies below it.
    pub outline_on: bool,
    outline_h: i32,
    /// Auto Hide, each pane on its own: Project, Properties.
    explorer_auto: bool,
    props_auto: bool,
    /// The panes as tabs, one shown at a time, their tabs along the bottom - as Visual Studio
    /// groups Solution Explorer and Properties. `active` is the pane shown (0 Project, 1 Server
    /// Explorer, 2 Document Outline, 3 Properties).
    tabbed: bool,
    active: usize,
    tabs: HWND,
    /// The pane slid out over the form from its Auto Hide tab: its windows lent to the slide host.
    sliding: Option<usize>,
}

/// The pane names on the tabs along the bottom, by pane.
fn tab_name(k: usize) -> &'static str {
    [t("Project"), t("Server Explorer"), t("Document Outline"), t("Properties")][k]
}

#[derive(Clone, Copy)]
struct SideParts {
    explorer_caption: HWND,
    explorer_bar: HWND,
    explorer_search: HWND,
    tree: HWND,
    split: HWND,
    props_caption: HWND,
    props: HWND,
    servers_caption: HWND,
    servers: HWND,
    split2: HWND,
    outline_caption: HWND,
    outline: HWND,
    split3: HWND,
}

impl Side {
    /// Whether pane `k` is wanted (View menu ticked).
    fn wanted(&self, p: &SideParts, k: usize) -> bool {
        match k {
            0 => self.explorer_on,
            1 => self.servers_on,
            2 => self.outline_on && !p.outline.is_invalid(),
            _ => self.props_on,
        }
    }

    fn auto(&self, k: usize) -> bool {
        (k == 0 && self.explorer_auto) || (k == 3 && self.props_auto)
    }

    /// The panes in the side itself: wanted, not hidden at the edge, not slid out.
    fn docked(&self) -> Vec<usize> {
        let Some(p) = self.parts else { return vec![] };
        (0..4).filter(|k| self.wanted(&p, *k) && !self.auto(*k) && self.sliding != Some(*k)).collect()
    }

    /// Whether the tabs show: tabbed, with two panes or more.
    fn tabs_on(&self) -> bool {
        self.tabbed && self.docked().len() > 1
    }

    /// The panes again: those shown take their room, the last one shown fills the rest, a
    /// splitter between each two.
    fn arrange(&mut self, side: HWND) {
        let Some(p) = self.parts else { return };
        let mut l = Layout::new((280, 600));
        let item = |h: i32, dock: Dock| Item { bounds: Rect::new(0, 0, 0, h), dock, anchor: Anchor::default() };
        // Project, Server Explorer, Document Outline, Properties - each caption, body, shown, height.
        let panes = [
            (p.explorer_caption, p.tree, self.explorer_on, self.explorer_h),
            (p.servers_caption, p.servers, self.servers_on, self.servers_h),
            (p.outline_caption, p.outline, self.outline_on && !p.outline.is_invalid(), self.outline_h),
            (p.props_caption, p.props, self.props_on, 0),
        ];
        let mut shown = self.docked();
        let tabbed = self.tabs_on();
        if tabbed {
            if !shown.contains(&self.active) {
                self.active = shown[0];
            }
            shown = vec![self.active];
        }
        for (k, (c, b, _, _)) in panes.iter().enumerate() {
            // A pane slid out is the slide host's to show.
            if self.sliding == Some(k) {
                continue;
            }
            let on = shown.contains(&k);
            win::show(*c, on);
            win::show(*b, on);
            if k == 0 {
                win::show(p.explorer_bar, on);
                win::show(p.explorer_search, on);
            }
        }
        let splits = [p.split, p.split2, p.split3];
        for s in splits {
            win::show(s, false);
        }
        win::show(self.tabs, tabbed);
        if tabbed {
            l.add(self.tabs, item(26, Dock::Bottom));
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(self.tabs), None, false);
            }
        }
        for (n, k) in shown.iter().enumerate() {
            let (c, b, _, h) = panes[*k];
            let last = n + 1 == shown.len();
            l.add(c, item(28, Dock::Top));
            if *k == 0 {
                // Project: its bar of buttons and its search box, as Solution Explorer has them.
                l.add(p.explorer_bar, item(28, Dock::Top));
                l.add(p.explorer_search, item(24, Dock::Top));
            }
            l.add(b, if last { item(0, Dock::Fill) } else { item((h - 52).max(40), Dock::Top) });
            if !last {
                // The splitter under a pane sizes that pane: the first under Project, the
                // second under Server Explorer, the third under Document Outline.
                let s = splits[(*k).min(2)];
                win::show(s, true);
                l.add(s, item(6, Dock::Top));
            }
        }
        self.layout = l;
        self.layout.apply(side);
    }
}

/// Shows or hides the explorer and properties panes; returns whether the side still has a pane
/// in it. A pane just shown is the tab on top.
pub fn show_side_panes(side: HWND, explorer: bool, props: bool) -> bool {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        if explorer && !s.explorer_on {
            s.active = 0;
        }
        if props && !s.props_on {
            s.active = 3;
        }
        s.explorer_on = explorer;
        s.props_on = props;
        s.arrange(side);
    }
    side_any(side)
}

/// Whether any pane lies in the side itself - not hidden at the edge, not slid out.
pub fn side_any(side: HWND) -> bool {
    unsafe { win::state::<Side>(side) }.is_some_and(|s| !s.docked().is_empty())
}

/// Auto Hide for Project (`explorer`) or Properties, each on its own; whether the side still has
/// a pane in it.
pub fn set_pane_auto(side: HWND, explorer: bool, on: bool) -> bool {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        if explorer {
            s.explorer_auto = on;
        } else {
            s.props_auto = on;
        }
        s.arrange(side);
    }
    side_any(side)
}

pub fn pane_auto(side: HWND, explorer: bool) -> bool {
    unsafe { win::state::<Side>(side) }.is_some_and(|s| if explorer { s.explorer_auto } else { s.props_auto })
}

/// The panes as tabs along the bottom, or one above another.
pub fn set_tabbed(side: HWND, on: bool) {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        s.tabbed = on;
        s.arrange(side);
    }
}

pub fn tabbed(side: HWND) -> bool {
    unsafe { win::state::<Side>(side) }.is_some_and(|s| s.tabbed)
}

/// The window that holds a pane slid out from its Auto Hide tab, over the form. What its pane's
/// windows say goes on to the side, as if they were still in it.
pub fn slide_host(parent: HWND) -> HWND {
    let class = win::register("PhaiStudioSlide", Some(host_proc), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
        win::set_state(h, Box::new(Host { side: HWND::default(), layout: Layout::new((280, 600)) }));
        h
    }
}

struct Host {
    side: HWND,
    layout: Layout,
}

/// Lends Project (`explorer`) or Properties to the slide host, laid out in it.
pub fn slide_out_pane(side: HWND, explorer: bool, host: HWND) {
    let Some(s) = (unsafe { win::state::<Side>(side) }) else { return };
    let Some(p) = s.parts else { return };
    s.sliding = Some(if explorer { 0 } else { 3 });
    s.arrange(side);
    let item = |h: i32, dock: Dock| Item { bounds: Rect::new(0, 0, 0, h), dock, anchor: Anchor::default() };
    let mut l = Layout::new((280, 600));
    let windows: Vec<(HWND, Item)> = if explorer {
        vec![(p.explorer_caption, item(28, Dock::Top)), (p.explorer_bar, item(28, Dock::Top)), (p.explorer_search, item(24, Dock::Top)), (p.tree, item(0, Dock::Fill))]
    } else {
        vec![(p.props_caption, item(28, Dock::Top)), (p.props, item(0, Dock::Fill))]
    };
    for (w, it) in windows {
        unsafe {
            let _ = wm::SetParent(w, Some(host));
        }
        win::show(w, true);
        l.add(w, it);
    }
    if let Some(hs) = unsafe { win::state::<Host>(host) } {
        hs.side = side;
        hs.layout = l;
        hs.layout.apply(host);
    }
}

/// The pane slid out comes home to the side.
pub fn slide_in_pane(side: HWND, host: HWND) {
    let Some(s) = (unsafe { win::state::<Side>(side) }) else { return };
    let Some(p) = s.parts else { return };
    let Some(k) = s.sliding.take() else { return };
    let windows = if k == 0 { vec![p.explorer_caption, p.explorer_bar, p.explorer_search, p.tree] } else { vec![p.props_caption, p.props] };
    for w in windows {
        win::show(w, false);
        unsafe {
            let _ = wm::SetParent(w, Some(side));
        }
    }
    if let Some(hs) = unsafe { win::state::<Host>(host) } {
        hs.layout = Layout::new((280, 600));
    }
    s.arrange(side);
}

extern "system" fn host_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(s) = unsafe { win::state::<Host>(h) } {
                s.layout.apply(h);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND | wm::WM_NOTIFY => match unsafe { win::state::<Host>(h) }.map(|s| s.side) {
            Some(side) if !side.is_invalid() => unsafe { wm::SendMessageW(side, msg, Some(w), Some(l)) },
            _ => win::def(h, msg, w, l),
        },
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Host>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The tabs along the bottom: each pane in the side by name, the one shown marked. A click shows
/// that pane.
fn side_tabs(side: HWND, font: HFONT) -> HWND {
    let class = win::register("PhaiStudioSideTabs", Some(tabs_proc), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(side), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
        win::set_state(h, Box::new(font));
        h
    }
}

/// Each tab's pane and its left and right edge.
fn tab_spans(h: HWND, s: &Side, font: HFONT) -> Vec<(usize, i32, i32)> {
    let mut x = 0;
    s.docked()
        .into_iter()
        .map(|k| {
            let (w, _) = win::measure(h, font, tab_name(k));
            let span = (k, x, x + w + win::px(h, 16));
            x = span.2;
            span
        })
        .collect()
}

extern "system" fn tabs_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use phai_ui::theme::{self, Color, State};
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi;
    let side = unsafe { wm::GetParent(h) }.unwrap_or_default();
    match msg {
        wm::WM_PAINT => unsafe {
            let font = win::state::<HFONT>(h).map(|f| *f).unwrap_or_default();
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let base = theme::style("tabs", State::default());
            win::fill(dc, &r, base.background_or(win::window_background()));
            let rule = base.border_or(Color::rgb(229, 229, 229));
            win::fill(dc, &RECT { bottom: r.top + 1, ..r }, rule);
            let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
            if let Some(s) = win::state::<Side>(side) {
                let old = Gdi::SelectObject(dc, font.into());
                Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
                for (k, a, b) in tab_spans(h, s, font) {
                    let on = k == s.active;
                    let mut tr = RECT { left: a, right: b, top: r.top, bottom: r.bottom };
                    if on {
                        // As Visual Studio marks the tab shown: the page's own colour, joined to
                        // the pane above, the accent along its edge.
                        win::fill(dc, &tr, win::window_background());
                        win::fill(dc, &RECT { top: tr.bottom - win::px(h, 2), ..tr }, accent);
                    }
                    Gdi::SetTextColor(dc, COLORREF(if on { accent } else { muted }.colorref()));
                    let mut text: Vec<u16> = tab_name(k).encode_utf16().collect();
                    Gdi::DrawTextW(dc, &mut text, &mut tr, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
                }
                Gdi::SelectObject(dc, old);
            }
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_LBUTTONDOWN => {
            let (x, _) = win::point_of(l);
            let font = unsafe { win::state::<HFONT>(h) }.map(|f| *f).unwrap_or_default();
            if let Some(s) = unsafe { win::state::<Side>(side) } {
                if let Some((k, _, _)) = tab_spans(h, s, font).into_iter().find(|(_, a, b)| x >= *a && x < *b) {
                    s.active = k;
                    s.arrange(side);
                }
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<HFONT>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// Whether the explorer and properties panes are shown.
pub fn side_panes(side: HWND) -> (bool, bool) {
    unsafe { win::state::<Side>(side) }.map(|s| (s.explorer_on, s.props_on)).unwrap_or((true, true))
}

/// The right-hand panel that holds the explorer and the properties.
pub fn side_panel(parent: HWND) -> HWND {
    let class = win::register("PhaiStudioSide", Some(side_proc), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
        win::set_state(h, Box::new(Side { layout: Layout::new((280, 600)), parts: None, explorer_h: 260, servers_h: 200, explorer_on: true, props_on: true, servers_on: false, outline_on: false, outline_h: 200, explorer_auto: false, props_auto: false, tabbed: false, active: 0, tabs: HWND::default(), sliding: None }));
        h
    }
}

/// Makes the panes of the side: the explorer's caption and tree, a splitter, the properties'
/// caption and the properties.
pub fn set_side_parts(side: HWND, explorer: &Explorer, props: HWND, servers: HWND, outline: HWND, font: HFONT) {
    let (heading, bar, search, tree) = (explorer.hwnd, explorer.bar, explorer.search, explorer.tree);
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        let split = phai_ui::controls::splitter(side, ID_SIDE_SPLIT, false);
        let split2 = phai_ui::controls::splitter(side, ID_SIDE_SPLIT2, false);
        let props_caption = phai_ui::controls::caption(side, t("Properties"), ID_PROPS_CAPTION, font);
        let servers_caption = phai_ui::controls::caption(side, t("Server Explorer"), ID_SERVERS_CAPTION, font);
        let outline_caption = phai_ui::controls::caption(side, t("Document Outline"), ID_OUTLINE_CAPTION, font);
        let split3 = phai_ui::controls::splitter(side, ID_SIDE_SPLIT3, false);
        s.tabs = side_tabs(side, font);
        s.parts = Some(SideParts { explorer_caption: heading, explorer_bar: bar, explorer_search: search, tree, split, props_caption, props, servers_caption, servers, split2, outline_caption, outline, split3 });
        s.arrange(side);
    }
}

/// Shows or hides Server Explorer; whether it shows now.
pub fn show_servers(side: HWND, on: bool) -> bool {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        if on && !s.servers_on {
            s.active = 1;
        }
        s.servers_on = on;
        s.arrange(side);
    }
    on
}

/// Shows or hides Document Outline; whether it shows now.
pub fn show_outline(side: HWND, on: bool) -> bool {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        if on && !s.outline_on {
            s.active = 2;
        }
        s.outline_on = on;
        s.arrange(side);
    }
    on
}

pub fn outline_shown(side: HWND) -> bool {
    unsafe { win::state::<Side>(side) }.is_some_and(|s| s.outline_on)
}

pub fn servers_shown(side: HWND) -> bool {
    unsafe { win::state::<Side>(side) }.is_some_and(|s| s.servers_on)
}

/// The explorer's and the properties' captions.
pub fn side_captions(side: HWND) -> Option<(HWND, HWND)> {
    unsafe { win::state::<Side>(side) }.and_then(|s| s.parts).map(|p| (p.explorer_caption, p.props_caption))
}

/// New words on the captions, after a language switch.
pub fn relabel_side(side: HWND) {
    if let Some(p) = unsafe { win::state::<Side>(side) }.and_then(|s| s.parts) {
        win::set_text(p.props_caption, t("Properties"));
        win::set_text(p.servers_caption, t("Server Explorer"));
        win::set_text(p.outline_caption, t("Document Outline"));
    }
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(s.tabs), None, false);
        }
    }
}

extern "system" fn side_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(s) = unsafe { win::state::<Side>(h) } {
                s.layout.apply(h);
            }
            LRESULT(0)
        }
        // A pane's caption buttons - ▾, the pin, × - are the studio's to answer.
        wm::WM_COMMAND if matches!(win::loword(w.0) as i32, BAR_NEW_FORM | BAR_REFRESH | BAR_COLLAPSE | BAR_FOLDER) => {
            if let Some(e) = studio().and_then(|s| s.explorer.as_mut()) {
                e.bar_command(win::loword(w.0) as i32);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_EXPLORER_SEARCH && win::hiword(w.0) == 0x0300 /* EN_CHANGE */ => {
            if let Some(e) = studio().and_then(|s| s.explorer.as_mut()) {
                e.filter = win::text_of(e.search).trim().to_lowercase();
                let root = e.root.clone();
                e.fill(root);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_OUTLINE_CAPTION => {
            if win::hiword(w.0) == phai_ui::controls::CAPTION_CLOSE {
                show_outline(h, false);
                if let Some(st) = studio() {
                    st.rebuild_menu_pub();
                }
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_SERVERS_CAPTION => {
            // Server Explorer: × hides it; it docks and hides with nothing else.
            if win::hiword(w.0) == phai_ui::controls::CAPTION_CLOSE {
                show_servers(h, false);
                if let Some(st) = studio() {
                    st.rebuild_menu_pub();
                }
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if matches!(win::loword(w.0) as i32, ID_EXPLORER_CAPTION | ID_PROPS_CAPTION) => {
            let pane = if win::loword(w.0) as i32 == ID_EXPLORER_CAPTION { crate::Pane::Explorer } else { crate::Pane::Properties };
            if let Some(st) = studio() {
                st.pane_button(pane, win::hiword(w.0), HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        phai_ui::controls::WM_SPLIT => {
            if let Some(s) = unsafe { win::state::<Side>(h) } {
                let d = phai_ui::layout::unscale(l.0 as i32, win::dpi(h));
                if w.0 as i32 == ID_SIDE_SPLIT3 {
                    s.outline_h = (s.outline_h + d).clamp(60, 900);
                } else if w.0 as i32 == ID_SIDE_SPLIT2 {
                    s.servers_h = (s.servers_h + d).clamp(60, 900);
                } else {
                    s.explorer_h = (s.explorer_h + d).clamp(60, 900);
                }
                s.arrange(h);
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE if DRAGGING_TABLE.load(std::sync::atomic::Ordering::Relaxed) => {
            let mut pt = windows::Win32::Foundation::POINT::default();
            unsafe {
                let _ = wm::GetCursorPos(&mut pt);
                let cur = if over_designer(pt) { wm::IDC_HAND } else { wm::IDC_NO };
                if let Ok(c) = wm::LoadCursorW(None, cur) {
                    wm::SetCursor(Some(c));
                }
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP if DRAGGING_TABLE.load(std::sync::atomic::Ordering::Relaxed) => {
            DRAGGING_TABLE.store(false, std::sync::atomic::Ordering::Relaxed);
            let mut pt = windows::Win32::Foundation::POINT::default();
            unsafe {
                let _ = wm::GetCursorPos(&mut pt);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
            }
            if over_designer(pt) {
                let owner = studio().map(|s| s.hwnd).unwrap_or(h);
                if let Some(sv) = studio().and_then(|s| s.servers.as_mut()) {
                    sv.add_selected_to_form(owner);
                }
            }
            LRESULT(0)
        }
        wm::WM_CAPTURECHANGED => {
            DRAGGING_TABLE.store(false, std::sync::atomic::Ordering::Relaxed);
            win::def(h, msg, w, l)
        }
        WM_PREVIEW_SELECTED => {
            if let Some(e) = studio().and_then(|s| s.explorer.as_mut()) {
                e.preview_selected();
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.idFrom == crate::outline::ID_OUTLINE_TREE as usize {
                const TVN_SELCHANGEDW: u32 = (-451i32) as u32;
                if hdr.code == cc::NM_CUSTOMDRAW {
                    return win::tree_custom_draw(l);
                }
                if hdr.code == TVN_SELCHANGEDW {
                    if let Some(st) = studio() {
                        st.outline_picked();
                    }
                }
                return LRESULT(0);
            }
            if hdr.idFrom == crate::servers::ID_SERVERS_TREE as usize {
                const TVN_ITEMEXPANDINGW: u32 = (-454i32) as u32;
                const TVN_ITEMEXPANDEDW: u32 = (-455i32) as u32;
                const TVN_BEGINDRAGW: u32 = (-456i32) as u32;
                if let Some(sv) = studio().and_then(|s| s.servers.as_mut()) {
                    match hdr.code {
                        cc::NM_DBLCLK => {
                            sv.activate();
                            return LRESULT(1);
                        }
                        cc::NM_RCLICK => {
                            // The node under the pointer is the one the menu is for.
                            let mut ht = cc::TVHITTESTINFO::default();
                            unsafe {
                                let _ = wm::GetCursorPos(&mut ht.pt);
                                let _ = windows::Win32::Graphics::Gdi::ScreenToClient(hdr.hwndFrom, &mut ht.pt);
                                let hit = wm::SendMessageW(hdr.hwndFrom, 0x1111 /* TVM_HITTEST */, None, Some(LPARAM(&mut ht as *mut _ as isize))).0;
                                if hit != 0 {
                                    wm::SendMessageW(hdr.hwndFrom, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9)), Some(LPARAM(hit)));
                                }
                            }
                            sv.menu(h);
                            return LRESULT(1);
                        }
                        // A table dragged: onto the designer it goes as a bound grid (Data Sources).
                        TVN_BEGINDRAGW => {
                            let nm = unsafe { &*(l.0 as *const cc::NMTREEVIEWW) };
                            unsafe {
                                wm::SendMessageW(hdr.hwndFrom, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9)), Some(LPARAM(nm.itemNew.hItem.0)));
                            }
                            if sv.table_chosen() {
                                DRAGGING_TABLE.store(true, std::sync::atomic::Ordering::Relaxed);
                                unsafe {
                                    windows::Win32::UI::Input::KeyboardAndMouse::SetCapture(h);
                                }
                            }
                            return LRESULT(0);
                        }
                        // Opening - or opened, by whatever opened it: read the first time.
                        TVN_ITEMEXPANDINGW | TVN_ITEMEXPANDEDW => {
                            let nm = unsafe { &*(l.0 as *const cc::NMTREEVIEWW) };
                            if std::env::var_os("PHAI_SERVERS_TEST").is_some() {
                                println!("servers: notify {} action {} item {:#x} data {}", hdr.code as i32, nm.action.0, nm.itemNew.hItem.0, nm.itemNew.lParam.0);
                            }
                            if nm.action.0 & 2 != 0 {
                                sv.expanding(nm.itemNew.hItem.0, nm.itemNew.lParam.0);
                            }
                            return LRESULT(0);
                        }
                        _ => {}
                    }
                }
            }
            if hdr.idFrom == ID_TREE as usize {
                if let Some(e) = studio().and_then(|s| s.explorer.as_mut()) {
                    match hdr.code {
                        cc::NM_DBLCLK => {
                            e.open_selected();
                            return LRESULT(1);
                        }
                        cc::NM_CLICK => {
                            // One click: the file in the preview tab - once the click has chosen it.
                            unsafe {
                                let _ = wm::PostMessageW(Some(h), WM_PREVIEW_SELECTED, WPARAM(0), LPARAM(0));
                            }
                        }
                        cc::NM_RCLICK => {
                            e.menu(h);
                            return LRESULT(1);
                        }
                        _ => {}
                    }
                }
            }
            win::def(h, msg, w, l)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Side>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

impl Explorer {
    pub fn new(side: HWND, font: HFONT) -> Explorer {
        let hwnd = phai_ui::controls::caption(side, t("Project"), ID_EXPLORER_CAPTION, font);
        let bar = win::toolbar(side, ID_EXPLORER_BAR, &bar_items());
        let search = win::edit(side, "", ID_EXPLORER_SEARCH, font, false);
        let tree = win::tree(side, ID_TREE, font);
        win::tree_icons(tree, &ICONS);
        let e = Explorer { hwnd, tree, bar, search, root: None, filter: String::new(), entries: vec![] };
        e.cue();
        e
    }

    pub fn relabel(&mut self) {
        win::set_toolbar(self.bar, &bar_items());
        self.cue();
        let root = self.root.clone();
        self.fill(root);
    }

    fn cue(&self) {
        let cue = win::wide(t("Search Project"));
        unsafe {
            wm::SendMessageW(self.search, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
        }
    }

    /// A button of the bar: a new form, the tree read again, every folder closed, the folder opened.
    fn bar_command(&mut self, id: i32) {
        match id {
            BAR_NEW_FORM => {
                if let (Some(root), Some(s)) = (self.root.clone(), studio()) {
                    s.new_form(&root, None);
                }
            }
            BAR_REFRESH => {
                let root = self.root.clone();
                self.fill(root);
            }
            BAR_COLLAPSE => {
                // Everything closed but the project itself, as Solution Explorer's Collapse All.
                let root = self.root.clone();
                self.fill(root);
                collapse_below_top(self.tree);
            }
            BAR_FOLDER => {
                if let Some(p) = self.selected_path().or_else(|| self.root.clone()) {
                    let dir = if p.is_dir() { p } else { p.parent().map(Path::to_path_buf).unwrap_or(p) };
                    crate::open_with_windows(&dir.to_string_lossy());
                }
            }
            _ => {}
        }
    }

    /// Shows the project that holds `form` - or the form's folder, or nothing.
    /// The file open in the editor chosen in the tree - Visual Studio's Track Active Item.
    pub fn reveal(&mut self, path: &Path) {
        let Some(i) = self.entries.iter().position(|e| phai_editor::lsp::same_path(e, path)) else { return };
        if win::tree_selected(self.tree) != Some(i as isize) {
            win::tree_select_data(self.tree, i as isize);
        }
    }

    pub fn show_project_of(&mut self, form: Option<&Path>) {
        let root = form.and_then(|f| crate::cargo::project_of(f).or_else(|| f.parent().map(Path::to_path_buf)));
        if root != self.root || self.entries.is_empty() {
            self.fill(root);
        }
    }

    fn fill(&mut self, root: Option<PathBuf>) {
        win::tree_clear(self.tree);
        self.entries.clear();
        self.root = root.clone();
        let Some(root) = root else {
            win::set_text(self.hwnd, t("Project"));
            win::tree_add(self.tree, 0, t("No project"), 1, -1);
            return;
        };
        let name = root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        win::set_text(self.hwnd, &format!("{} · {name}", t("Project")));
        self.entries.push(root.clone());
        let top = win::tree_add(self.tree, 0, &name, 4, 0);
        // Dependencies: the crates Cargo.toml names, as Solution Explorer lists a project's packages.
        let cargo = root.join("Cargo.toml");
        let deps = std::fs::read_to_string(&cargo).map(|t| dependencies_of(&t)).unwrap_or_default();
        if !deps.is_empty() && self.filter.is_empty() {
            self.entries.push(cargo.clone());
            let node = win::tree_add(self.tree, top, t("Dependencies"), 6, self.entries.len() as isize - 1);
            for d in deps {
                self.entries.push(cargo.clone());
                win::tree_add(self.tree, node, &d, 7, self.entries.len() as isize - 1);
            }
        }
        self.add_folder(top, &root, 0);
        win::tree_expand(self.tree, top);
    }

    fn add_folder(&mut self, parent: isize, dir: &Path, depth: usize) {
        if depth > 6 {
            return;
        }
        let Ok(read) = std::fs::read_dir(dir) else { return };
        let mut items: Vec<(bool, String, PathBuf)> = read
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                // What the build makes and what git keeps are not the project's own files.
                if name.starts_with('.') || name == "target" || name == "Cargo.lock" {
                    return None;
                }
                Some((e.path().is_dir(), name, e.path()))
            })
            .collect();
        // Folders first, then files, each by name.
        items.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.to_lowercase().cmp(&b.1.to_lowercase())));
        // A form's two Rust files go under it, as a WinForms form's code and designer files do.
        let forms: Vec<String> = items.iter().filter_map(|(_, n, _)| n.strip_suffix(".phaiform").map(str::to_string)).collect();
        // A report's generated code goes under the report.
        let reports: Vec<String> = items.iter().filter_map(|(_, n, _)| n.strip_suffix(".phaireport").map(str::to_string)).collect();
        let under_form = |n: &str| forms.iter().any(|f| n == format!("{f}.rs") || n == format!("{f}_ui.rs")) || reports.iter().any(|r| n == format!("{r}.rs"));
        let filter = self.filter.clone();
        let matches = |n: &str| filter.is_empty() || n.to_lowercase().contains(&filter);
        for (is_dir, name, path) in items {
            if !is_dir && under_form(&name) && filter.is_empty() {
                continue;
            }
            if !is_dir && !matches(&name) {
                continue;
            }
            if is_dir && !filter.is_empty() && !has_match(&path, &filter, depth + 1) {
                continue;
            }
            let icon = if is_dir {
                0
            } else if name.ends_with(".phaiform") || name.ends_with(".phaireport") {
                2
            } else if name.ends_with(".rs") {
                3
            } else if name.ends_with(".css") {
                5
            } else {
                1
            };
            self.entries.push(path.clone());
            let item = win::tree_add(self.tree, parent, &name, icon, self.entries.len() as isize - 1);
            if is_dir {
                self.add_folder(item, &path, depth + 1);
                if name == "src" || !filter.is_empty() {
                    win::tree_expand(self.tree, item);
                }
            } else if let Some(stem) = name.strip_suffix(".phaireport").filter(|_| filter.is_empty()) {
                let p = dir.join(format!("{stem}.rs"));
                if p.is_file() {
                    self.entries.push(p);
                    win::tree_add(self.tree, item, &format!("{stem}.rs"), 3, self.entries.len() as isize - 1);
                }
            } else if let Some(stem) = name.strip_suffix(".phaiform").filter(|_| filter.is_empty()) {
                for child in [format!("{stem}.rs"), format!("{stem}_ui.rs")] {
                    let p = dir.join(&child);
                    if p.is_file() {
                        self.entries.push(p);
                        win::tree_add(self.tree, item, &child, 3, self.entries.len() as isize - 1);
                    }
                }
            }
        }
    }

    fn selected_path(&self) -> Option<PathBuf> {
        let i = win::tree_selected(self.tree)?;
        self.entries.get(usize::try_from(i).ok()?).cloned()
    }

    /// The chosen file in the preview tab, when it is code.
    fn preview_selected(&mut self) {
        let Some(path) = self.selected_path() else { return };
        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if path.is_file() && ["rs", "toml", "css", "md", "txt", "json"].contains(&ext.as_str()) {
            if let Some(s) = studio() {
                s.open_preview(&path);
            }
        }
    }

    fn open_selected(&mut self) {
        let Some(path) = self.selected_path() else { return };
        if path.is_dir() {
            return;
        }
        let Some(s) = studio() else { return };
        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if ext == "phaiform" {
            s.open_path(&path);
        } else if ext == "phaireport" {
            crate::repdesign::open(Some(s.hwnd), &path);
        } else if ["rs", "toml", "css", "md", "txt", "json", "gitignore"].contains(&ext.as_str()) || path.file_name().is_some_and(|n| n == ".gitignore") {
            s.open_code(&path);
        } else {
            crate::open_with_windows(&path.to_string_lossy());
        }
    }

    fn menu(&mut self, owner: HWND) {
        let mut items: Vec<(u32, &str)> = vec![(CMD_NEW_FORM, t("New form..."))];
        for (k, tpl) in phai_design::templates::Template::ALL.iter().enumerate() {
            items.push((CMD_TEMPLATE + k as u32, t(tpl.name())));
        }
        items.extend([(0, ""), (CMD_SHOW, t("Show in Explorer")), (CMD_REFRESH, t("Refresh"))]);
        let chosen = win::popup(owner, &items);
        match chosen {
            CMD_NEW_FORM | CMD_TEMPLATE..=19 => {
                let tpl = chosen.checked_sub(CMD_TEMPLATE).and_then(|k| phai_design::templates::Template::ALL.get(k as usize).copied());
                if let Some(root) = self.root.clone() {
                    if let Some(s) = studio() {
                        s.new_form(&root, tpl);
                    }
                }
            }
            CMD_SHOW => {
                if let Some(p) = self.selected_path().or_else(|| self.root.clone()) {
                    let dir = if p.is_dir() { p } else { p.parent().map(Path::to_path_buf).unwrap_or(p) };
                    crate::open_with_windows(&dir.to_string_lossy());
                }
            }
            CMD_REFRESH => {
                let root = self.root.clone();
                self.fill(root);
            }
            _ => {}
        }
    }
}

/// `MainWindow`-style names only: they become a struct and two file names.
fn bar_items() -> Vec<(i32, &'static str, u16)> {
    vec![(BAR_NEW_FORM, t("New form..."), glyph::NEW), (BAR_REFRESH, t("Refresh"), glyph::REFRESH), (BAR_COLLAPSE, t("Collapse All"), 0xF165), (BAR_FOLDER, t("Show in Explorer"), glyph::FOLDER)]
}

/// The crates a Cargo.toml's `[dependencies]` names.
pub fn dependencies_of(cargo_toml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in cargo_toml.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l == "[dependencies]";
            continue;
        }
        if inside {
            if let Some((name, _)) = l.split_once('=') {
                let name = name.trim();
                if !name.is_empty() && !name.starts_with('#') {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

/// Whether a folder holds, at some depth, a file whose name has these words.
fn has_match(dir: &Path, words: &str, depth: usize) -> bool {
    if depth > 6 {
        return false;
    }
    let Ok(read) = std::fs::read_dir(dir) else { return false };
    read.flatten().any(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" {
            return false;
        }
        if e.path().is_dir() { has_match(&e.path(), words, depth + 1) } else { name.to_lowercase().contains(words) }
    })
}

/// Closes every item under the tree's top one.
fn collapse_below_top(tree: HWND) {
    unsafe {
        let top = wm::SendMessageW(tree, 0x110A /* TVM_GETNEXTITEM */, Some(WPARAM(0 /* TVGN_ROOT */)), None).0;
        let mut child = wm::SendMessageW(tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(top))).0;
        while child != 0 {
            wm::SendMessageW(tree, 0x1102 /* TVM_EXPAND */, Some(WPARAM(1 /* TVE_COLLAPSE */)), Some(LPARAM(child)));
            child = wm::SendMessageW(tree, 0x110A, Some(WPARAM(1 /* TVGN_NEXT */)), Some(LPARAM(child))).0;
        }
    }
}

pub fn is_form_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// The lines that make a new form's two files part of the program: `mod x; mod x_ui;` after the
/// last `mod` line of `main.rs` (or at its top). `None` when they are there already.
pub fn add_mods(main_rs: &str, file: &str) -> Option<String> {
    let lines = [format!("mod {file};"), format!("mod {file}_ui;")];
    if lines.iter().all(|l| main_rs.lines().any(|x| x.trim() == l)) {
        return None;
    }
    let mut out: Vec<String> = main_rs.lines().map(str::to_string).collect();
    let at = out.iter().rposition(|l| l.trim_start().starts_with("mod ")).map(|i| i + 1).unwrap_or(0);
    for (k, l) in lines.iter().enumerate() {
        if !out.iter().any(|x| x.trim() == l) {
            out.insert(at + k, l.clone());
        }
    }
    Some(out.join("\n") + "\n")
}

impl crate::Studio {
    /// A form's name made a Rust type's: said on the status bar.
    fn say_renamed(&mut self, typed: &str, name: &str) {
        self.say_status(&tf("{0} is named {1} - a Rust type's name begins with a capital", &[&typed, &name]));
    }

    /// Right-click, New form: asks a name, writes the form and its code into `src`, makes them part
    /// of the program, and opens the form in a tab.
    pub fn new_form(&mut self, root: &Path, template: Option<phai_design::templates::Template>) {
        let Some(name) = phai_ui::dialogs::ask(Some(self.hwnd), t("New form"), t("Name of the new form, like CustomerWindow"), "", false) else { return };
        // As Project > Add Form takes it: frmAboutUs is FrmAboutUs, about us AboutUs - a Rust
        // type's name; only a name with no letters at all is refused.
        let typed = name.trim().to_string();
        let name = crate::form_type_name_pub(&typed);
        if !is_form_name(&name) {
            win::message(Some(self.hwnd), &tf("{0} is not a form name - PascalCase letters and digits, like CustomerWindow", &[&typed]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        if name != typed {
            self.say_renamed(&typed, &name);
        }
        let src = root.join("src");
        let file = snake(&name);
        let form_path = src.join(format!("{file}.phaiform"));
        if form_path.exists() {
            win::message(Some(self.hwnd), &tf("{0} already exists", &[&form_path.display()]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let mut f = template.map(|t| t.make(&name)).unwrap_or_else(|| FormFile::new(&name));
        if root.join("theme.css").exists() {
            f.form.theme = Some("../theme.css".into());
        }
        let opts = crate::theme_options(&form_path, f.form.theme.as_deref());
        let wrote = std::fs::create_dir_all(&src)
            .and_then(|_| std::fs::write(&form_path, f.to_toml()))
            .and_then(|_| std::fs::write(src.join(format!("{file}_ui.rs")), codegen::generate(&f, &opts)))
            .and_then(|_| std::fs::write(src.join(format!("{file}.rs")), codegen::user_stub(&f)));
        if let Err(e) = wrote {
            win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be made", &[&form_path.display()])), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let main_rs = src.join("main.rs");
        if let Ok(text) = std::fs::read_to_string(&main_rs) {
            if let Some(more) = add_mods(&text, &file) {
                let _ = std::fs::write(&main_rs, more);
            }
        }
        self.open_in_tab(f, Some(form_path));
        if let Some(e) = self.explorer.as_mut() {
            e.fill(Some(root.to_path_buf()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_forms_files_join_the_program_once() {
        let main = "//! x\n\nmod main_window;\nmod main_window_ui;\nmod text;\n\nfn main() {}\n";
        let more = add_mods(main, "customer_window").unwrap();
        assert!(more.contains("mod text;\nmod customer_window;\nmod customer_window_ui;\n\nfn main"), "{more}");
        assert!(add_mods(&more, "customer_window").is_none());
        assert_eq!(add_mods("fn main() {}\n", "a").unwrap(), "mod a;\nmod a_ui;\nfn main() {}\n");
    }

    #[test]
    fn dependencies_are_read_from_cargo_toml() {
        let toml = "[package]\nname = \"a\"\n\n[dependencies]\nphai-ui = { path = \"x\" }\nwindows = \"0.62\"\n\n[profile.release]\nlto = true\n";
        assert_eq!(dependencies_of(toml), ["phai-ui", "windows"]);
    }

    #[test]
    fn form_names() {
        assert!(is_form_name("CustomerWindow"));
        assert!(!is_form_name("customer"));
        assert!(!is_form_name("Customer Window"));
        assert!(!is_form_name(""));
    }

    /// Writes every template as a form into the project in `PHAI_NEW_DIR`, as New form does, for
    /// building it by hand: `cargo test -p phai-studio -- --ignored templates_into_project`.
    #[test]
    #[ignore]
    fn templates_into_project() {
        use phai_design::templates::Template;
        let root = std::path::PathBuf::from(std::env::var("PHAI_NEW_DIR").expect("PHAI_NEW_DIR"));
        let src = root.join("src");
        for (tpl, name) in [(Template::Dialog, "AskDialog"), (Template::MainWindow, "BigWindow"), (Template::DataEntry, "DetailsWindow")] {
            let mut f = tpl.make(name);
            f.form.theme = Some("../theme.css".into());
            let file = snake(name);
            let form_path = src.join(format!("{file}.phaiform"));
            let opts = crate::theme_options(&form_path, f.form.theme.as_deref());
            std::fs::write(&form_path, f.to_toml()).unwrap();
            std::fs::write(src.join(format!("{file}_ui.rs")), codegen::generate(&f, &opts)).unwrap();
            std::fs::write(src.join(format!("{file}.rs")), codegen::user_stub(&f)).unwrap();
            let main = src.join("main.rs");
            if let Some(more) = add_mods(&std::fs::read_to_string(&main).unwrap(), &file) {
                std::fs::write(&main, more).unwrap();
            }
        }
    }
}
