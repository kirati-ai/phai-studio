//! The properties pane, as Visual Studio's: an object chooser, Properties | Events, and a grid -
//! name and value - in categories that fold, or alphabetical. Each value has the editor its kind
//! wants: a box for words and numbers, a list for a choice, a click for true or false, a picker
//! for anchors, Windows' colour dialog for colours, "…" for lists and files. Below, what the chosen
//! property is for.

use phai_form::{anchor_text, parse_anchor, Anchors, Kind, DOCKS};
use phai_ui::text::t;
use phai_ui::theme::{Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::studio;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    FormName,
    FormTitle,
    FormWidth,
    FormHeight,
    FormTheme,
    FormFont,
    Id,
    Text,
    X,
    Y,
    W,
    H,
    Class,
    Event,
    Items,
    Font,
    Bold,
    BackColor,
    ForeColor,
    BorderColor,
    Radius,
    Enabled,
    Visible,
    Checked,
    Password,
    Group,
    Dock,
    Anchor,
    Min,
    Max,
    Value,
}

/// How a value is edited.
#[derive(Clone, Debug, PartialEq)]
enum Ed {
    Text,
    Number,
    Bool,
    /// One of these; `free` lets other words be typed too.
    Choice(Vec<String>, bool),
    /// "…" opens a dialog.
    Dialog,
    Anchor,
    Color,
    /// Shown, not changed here.
    ReadOnly,
}

#[derive(Clone, Debug)]
struct Prop {
    category: &'static str,
    key: Key,
    name: &'static str,
    value: String,
    ed: Ed,
    help: &'static str,
}

/// A row of the grid as shown: a category's heading, or a property.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Line {
    Heading(&'static str),
    Prop(usize),
}

pub struct Props {
    pub hwnd: HWND,
    chooser: HWND,
    modes: HWND,
    heading: HWND,
    sort_cat: HWND,
    sort_az: HWND,
    grid: HWND,
    help_name: HWND,
    help_text: HWND,
    mode: usize,
    alphabetical: bool,
    folded: Vec<&'static str>,
    props: Vec<Prop>,
    lines: Vec<Line>,
    chosen: Option<usize>,
    top: usize,
    /// The in-place editor and the property it edits.
    editor: Option<(HWND, usize)>,
    font: HFONT,
    bold: HFONT,
    filling: bool,
}

const ID_CHOOSER: i32 = 3300;
const ID_MODES: i32 = 3301;
const ID_SORT_CAT: i32 = 3302;
const ID_SORT_AZ: i32 = 3303;
const ID_EDITOR: i32 = 3304;
/// Posted to the pane to fill itself again after an edit, once the editor's message is done.
const WM_REFILL: u32 = wm::WM_APP + 0x0D01;

fn yes_no(b: bool) -> String {
    if b { "True" } else { "False" }.to_string()
}

/// Every class the project's theme sheet names - `button.primary` gives `primary`.
fn theme_classes() -> Vec<String> {
    let Some(s) = studio() else { return vec![] };
    let (Some(path), Some(theme)) = (s.doc.path.as_ref(), s.doc.form.form.theme.as_ref()) else { return vec!["primary".into(), "muted".into()] };
    let css = std::fs::read_to_string(path.with_file_name("").join(theme)).unwrap_or_default();
    let mut out: Vec<String> = vec!["primary".into(), "muted".into()];
    for word in css.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '.')) {
        if let Some(cls) = word.split('.').nth(1) {
            if !cls.is_empty() && !out.iter().any(|o| o == cls) && cls.chars().next().is_some_and(|c| c.is_alphabetic()) {
                out.push(cls.to_string());
            }
        }
    }
    out
}

/// The methods the programmer's file already has - for an event to point at.
fn user_methods() -> Vec<String> {
    let Some(s) = studio() else { return vec![] };
    let Some(path) = s.doc.path.as_ref() else { return vec![] };
    let user = path.with_file_name(format!("{}.rs", phai_form::snake(&s.doc.form.form.name)));
    let src = std::fs::read_to_string(user).unwrap_or_default();
    src.split("fn ").skip(1).filter_map(|r| r.split('(').next()).map(|n| n.trim().to_string()).filter(|n| !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_') && n != "loaded").collect()
}

impl Props {
    pub fn new(parent: HWND, id: i32, font: HFONT) -> Props {
        let class = win::register("PhaiStudioProps", Some(proc_), None);
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        let chooser = win::combo(hwnd, ID_CHOOSER, font, &[]);
        let modes = phai_ui::controls::tabs(hwnd, ID_MODES, font, &[t("Properties"), t("Events")]);
        let heading = win::label(hwnd, "", 0, font);
        let sort_cat = win::button(hwnd, "☰", ID_SORT_CAT, font, false);
        let sort_az = win::button(hwnd, "A-Z", ID_SORT_AZ, font, false);
        let grid_class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiPropGrid", Some(grid_proc), None);
        let grid = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(grid_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_VSCROLL, 0, 0, 10, 10, Some(hwnd), Some(HMENU(3310 as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        let bold = win::font(hwnd, 0, true);
        let help_name = win::label(hwnd, "", 0, bold);
        let help_text = win::child(hwnd, "STATIC", "", wm::WINDOW_STYLE(0x80), Default::default(), 0, font);
        win::register_kind(help_text, win::Kind::Label);
        win::set_css(help_text, "muted", "");
        Props { hwnd, chooser, modes, heading, sort_cat, sort_az, grid, help_name, help_text, mode: 0, alphabetical: false, folded: vec![], props: vec![], lines: vec![], chosen: None, top: 0, editor: None, font, bold, filling: false }
    }

    /// The properties of what is chosen, from the model.
    fn gather(&self) -> Vec<Prop> {
        let Some(s) = studio() else { return vec![] };
        let f = &s.doc.form;
        let p = |category, key, name, value: String, ed, help| Prop { category, key, name, value, ed, help };
        let sizes: Vec<String> = ["", "8", "9", "10", "11", "12", "14", "16", "18", "20", "24", "28", "36"].iter().map(|s| s.to_string()).collect();
        let bools = Ed::Bool;
        if self.mode == 1 {
            // Events: the control's, or the form's own and its menus' and toolbar's.
            return match s.selected.and_then(|i| f.control.get(i)) {
                Some(c) => match c.kind.event() {
                    Some(e) => {
                        let name = match e {
                            "on_click" => t("Click"),
                            "on_change" => t("Change"),
                            _ => t("Select"),
                        };
                        vec![p(t("Events"), Key::Event, name, c.handler().unwrap_or_default().to_string(), Ed::Choice(user_methods(), true), t("The method that runs when this happens. Double-click the control to write it."))]
                    }
                    None => vec![],
                },
                None => {
                    let mut v = vec![p(t("Events"), Key::Event, "loaded", "loaded".into(), Ed::ReadOnly, t("Runs once the window is made, before it shows."))];
                    for h in f.handlers() {
                        v.push(p(t("Events"), Key::Event, Box::leak(h.clone().into_boxed_str()), h, Ed::ReadOnly, t("A menu item, toolbar button or control runs this method.")));
                    }
                    v
                }
            };
        }
        let Some(c) = s.selected.and_then(|i| f.control.get(i)) else {
            return vec![
                p(t("Design"), Key::FormName, t("Name"), f.form.name.clone(), Ed::Text, t("The window's name in code: a struct, and its files' names.")),
                p(t("Appearance"), Key::FormTitle, t("Title"), f.form.title.clone(), Ed::Text, t("The words in the title bar - an English key, translated by the program.")),
                p(t("Appearance"), Key::FormFont, t("Font size"), f.form.font_size.map(|n| n.to_string()).unwrap_or_default(), Ed::Choice(sizes, true), t("Points for every control's words; empty is Windows' own 9.")),
                p(t("Appearance"), Key::FormTheme, t("Theme"), f.form.theme.clone().unwrap_or_default(), Ed::Dialog, t("The CSS file every control takes its colours from.")),
                p(t("Layout"), Key::FormWidth, t("Width"), f.form.size[0].to_string(), Ed::Number, t("The inside of the window, at 100% scale.")),
                p(t("Layout"), Key::FormHeight, t("Height"), f.form.size[1].to_string(), Ed::Number, t("The inside of the window, at 100% scale.")),
            ];
        };
        let [x, y, w, h] = c.rect();
        let docked = c.dock.is_some();
        let has_text = c.kind.has_text() || c.kind == Kind::Statusbar;
        let colour = |o: &Option<String>| o.clone().unwrap_or_default();
        let mut v = vec![
            p(t("Design"), Key::Id, t("Id"), c.id.clone(), Ed::Text, t("The control's name in code, and its #id in CSS.")),
            p(t("Design"), Key::Class, t("Class"), c.class.clone().unwrap_or_default(), Ed::Choice(theme_classes(), true), t("CSS classes from the theme, like primary. Several, space between.")),
        ];
        if has_text {
            v.push(p(t("Appearance"), Key::Text, t("Text"), c.text.clone().unwrap_or_default(), Ed::Text, t("The words it shows - an English key, translated by the program.")));
        }
        v.extend([
            p(t("Appearance"), Key::Font, t("Font size"), c.font_size.map(|n| n.to_string()).unwrap_or_default(), Ed::Choice(sizes, true), t("Points for this control's words, over the form's.")),
            p(t("Appearance"), Key::Bold, t("Bold"), yes_no(c.bold == Some(true)), bools.clone(), t("Its words in bold.")),
            p(t("Appearance"), Key::BackColor, t("Back colour"), colour(&c.back_color), Ed::Color, t("Its background, over the theme - written into CSS as #id { background }.")),
            p(t("Appearance"), Key::ForeColor, t("Fore colour"), colour(&c.fore_color), Ed::Color, t("Its words' colour, over the theme - #id { color }.")),
            p(t("Appearance"), Key::BorderColor, t("Border colour"), colour(&c.border_color), Ed::Color, t("Its border, over the theme - #id { border-color }.")),
            p(t("Appearance"), Key::Radius, t("Corner radius"), c.radius.map(|n| n.to_string()).unwrap_or_default(), Ed::Number, t("How round its corners are, pixels - #id { border-radius }.")),
            p(t("Behavior"), Key::Enabled, t("Enabled"), yes_no(c.enabled != Some(false)), bools.clone(), t("Whether it can be used, or shows greyed.")),
            p(t("Behavior"), Key::Visible, t("Visible"), yes_no(c.visible != Some(false)), bools.clone(), t("Whether it shows when the window opens.")),
        ]);
        if matches!(c.kind, Kind::Checkbox | Kind::Radio) {
            v.push(p(t("Behavior"), Key::Checked, t("Checked"), yes_no(c.checked == Some(true)), bools.clone(), t("Ticked when the window opens.")));
        }
        if c.kind == Kind::Radio {
            v.push(p(t("Behavior"), Key::Group, t("Group"), c.group.unwrap_or(0).to_string(), Ed::Number, t("Options with the same group number exclude each other.")));
        }
        if c.kind == Kind::Edit {
            v.push(p(t("Behavior"), Key::Password, t("Password"), yes_no(c.password == Some(true)), bools.clone(), t("Shows dots instead of what is typed.")));
        }
        if matches!(c.kind, Kind::Number | Kind::Progress | Kind::Slider) {
            v.push(p(t("Data"), Key::Min, t("Minimum"), c.min.unwrap_or(0).to_string(), Ed::Number, t("The smallest value.")));
            v.push(p(t("Data"), Key::Max, t("Maximum"), c.max.unwrap_or(100).to_string(), Ed::Number, t("The largest value.")));
            v.push(p(t("Data"), Key::Value, t("Value"), c.value.unwrap_or(0).to_string(), Ed::Number, t("The value when the window opens.")));
        }
        if matches!(c.kind, Kind::Combo | Kind::List | Kind::Tabs) {
            let items = match c.kind {
                Kind::Combo => c.items.join("\n"),
                Kind::Tabs => c.tabs.join("\n"),
                _ => c.columns.iter().map(|(n, w)| format!("{n}:{w}")).collect::<Vec<_>>().join("\n"),
            };
            v.push(p(t("Data"), Key::Items, if c.kind == Kind::List { t("Columns") } else { t("Items") }, items, Ed::Dialog, t("One a line; a list's columns as Name:width.")));
        }
        if !docked {
            v.extend([
                p(t("Layout"), Key::X, "X", x.to_string(), Ed::Number, t("From the form's left, at 100% scale.")),
                p(t("Layout"), Key::Y, "Y", y.to_string(), Ed::Number, t("From the form's top, at 100% scale.")),
            ]);
        }
        v.extend([
            p(t("Layout"), Key::W, t("Width"), w.to_string(), Ed::Number, t("At 100% scale.")),
            p(t("Layout"), Key::H, t("Height"), h.to_string(), Ed::Number, t("At 100% scale.")),
            p(t("Layout"), Key::Dock, t("Dock"), c.dock.clone().unwrap_or_else(|| "none".into()), Ed::Choice(DOCKS.iter().map(|d| d.to_string()).collect(), false), t("Fills an edge of the form, or all that is left.")),
        ]);
        if !docked {
            v.push(p(t("Layout"), Key::Anchor, t("Anchor"), c.anchor.clone().unwrap_or_else(|| "left top".into()), Ed::Anchor, t("The edges it keeps its distance to when the window grows.")));
        }
        v
    }

    /// The lines shown: headings and their properties, folded ones left out - or every property by name.
    fn arrange(&mut self) {
        self.lines.clear();
        if self.alphabetical {
            let mut idx: Vec<usize> = (0..self.props.len()).collect();
            idx.sort_by_key(|i| self.props[*i].name.to_lowercase());
            self.lines = idx.into_iter().map(Line::Prop).collect();
            return;
        }
        let mut cats: Vec<&'static str> = Vec::new();
        for p in &self.props {
            if !cats.contains(&p.category) {
                cats.push(p.category);
            }
        }
        for c in cats {
            self.lines.push(Line::Heading(c));
            if !self.folded.contains(&c) {
                for (i, p) in self.props.iter().enumerate() {
                    if p.category == c {
                        self.lines.push(Line::Prop(i));
                    }
                }
            }
        }
    }

    /// Fills everything from the model.
    pub fn refresh(&mut self) {
        let Some(s) = studio() else { return };
        self.close_editor(false);
        self.filling = true;
        let code = s.doc.code.is_some();
        for h in [self.chooser, self.modes, self.sort_cat, self.sort_az, self.grid, self.help_name, self.help_text] {
            win::show(h, !code);
        }
        if code {
            win::set_text(self.heading, &s.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            self.filling = false;
            self.layout();
            return;
        }
        let f = &s.doc.form;
        let chosen = s.selected.and_then(|i| f.control.get(i));
        win::set_text(self.heading, &match chosen {
            Some(c) => format!("{} · {}", crate::kind_label(c.kind), c.id),
            None => format!("{} · {}", t("Form"), f.form.name),
        });
        unsafe {
            wm::SendMessageW(self.chooser, 0x014B /* CB_RESETCONTENT */, None, None);
            let add = |s: String| {
                let w = win::wide(&s);
                wm::SendMessageW(self.chooser, 0x0143 /* CB_ADDSTRING */, None, Some(LPARAM(w.as_ptr() as isize)));
            };
            add(format!("{}  ({})", f.form.name, t("Form")));
            for c in &f.control {
                add(format!("{}  ({})", c.id, crate::kind_label(c.kind)));
            }
            wm::SendMessageW(self.chooser, 0x014E /* CB_SETCURSEL */, Some(WPARAM(s.selected.map(|i| i + 1).unwrap_or(0))), None);
        }
        let keep = self.chosen.and_then(|l| self.lines.get(l)).and_then(|l| if let Line::Prop(i) = l { self.props.get(*i).map(|p| p.key) } else { None });
        self.props = self.gather();
        self.arrange();
        self.chosen = keep.and_then(|k| self.lines.iter().position(|l| matches!(l, Line::Prop(i) if self.props[*i].key == k)));
        self.say_help();
        self.filling = false;
        self.layout();
        self.update_scroll();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
        }
    }

    fn say_help(&self) {
        match self.chosen.and_then(|l| self.lines.get(l)) {
            Some(Line::Prop(i)) => {
                win::set_text(self.help_name, self.props[*i].name);
                win::set_text(self.help_text, self.props[*i].help);
            }
            _ => {
                win::set_text(self.help_name, "");
                win::set_text(self.help_text, "");
            }
        }
    }

    pub fn layout(&self) {
        let h = self.hwnd;
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        let px = |v| win::px(h, v);
        let (m, row, gap) = (px(8), px(23), px(6));
        let width = r.right - r.left;
        let mut y = m;
        if unsafe { wm::IsWindowVisible(self.chooser) }.as_bool() {
            win::move_to(self.chooser, m, y, width - 2 * m, px(300));
            y += row + gap;
            win::move_to(self.modes, 0, y, width, px(28));
            y += px(28) + gap;
        }
        let bw = px(36);
        win::move_to(self.heading, m, y, width - 2 * m - 2 * bw - px(4), row);
        win::move_to(self.sort_cat, width - m - 2 * bw - px(2), y, bw, row);
        win::move_to(self.sort_az, width - m - bw, y, bw, row);
        y += row + gap;
        let help_h = px(70);
        let grid_h = (r.bottom - y - help_h - gap).max(0);
        win::move_to(self.grid, 0, y, width, grid_h);
        y += grid_h + gap;
        win::move_to(self.help_name, m, y, width - 2 * m, row);
        win::move_to(self.help_text, m, y + row, width - 2 * m, help_h - row);
    }

    pub fn set_mode(&mut self, mode: usize) {
        self.mode = mode;
        phai_ui::controls::tabs_select(self.modes, mode);
        self.refresh();
    }

    pub fn relabel(&mut self) {
        phai_ui::controls::tabs_relabel(self.modes, &[t("Properties"), t("Events")]);
        self.refresh();
    }

    fn row_h(&self) -> i32 {
        win::measure(self.grid, self.font, "Ag").1 + win::px(self.grid, 8)
    }

    fn visible_rows(&self) -> usize {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        (r.bottom / self.row_h().max(1)).max(1) as usize
    }

    fn update_scroll(&mut self) {
        let page = self.visible_rows();
        self.top = self.top.min(self.lines.len().saturating_sub(page));
        unsafe {
            let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: self.lines.len().max(1) as i32 - 1, nPage: page as u32, nPos: self.top as i32, nTrackPos: 0 };
            windows::Win32::UI::Controls::SetScrollInfo(self.grid, wm::SB_VERT, &si, true);
        }
    }

    /// The name column's width.
    fn split(&self) -> i32 {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        r.right * 45 / 100
    }

    fn line_rect(&self, line: usize) -> RECT {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        let rh = self.row_h();
        let y = (line as i32 - self.top as i32) * rh;
        RECT { left: 0, top: y, right: r.right, bottom: y + rh }
    }

    fn paint(&self) {
        let h = self.grid;
        unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let bg = win::window_background();
            win::fill(dc, &r, bg);
            let rule = phai_ui::theme::style("group", State::default()).border_or(Color::rgb(0xe5, 0xe5, 0xe5));
            let tint = phai_ui::theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xe7, 0xf3, 0xef));
            let head_bg = phai_ui::theme::style("tabs", State::default()).background_or(bg);
            let muted = phai_ui::theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
            let split = self.split();
            let pad = win::px(h, 8);
            for (k, line) in self.lines.iter().enumerate().skip(self.top).take(self.visible_rows() + 1) {
                let lr = self.line_rect(k);
                match line {
                    Line::Heading(name) => {
                        win::fill(dc, &lr, head_bg);
                        let mark = if self.folded.contains(name) { "▸ " } else { "▾ " };
                        let mut tr = RECT { left: pad / 2, ..lr };
                        let old = Gdi::SelectObject(dc, self.bold.into());
                        draw(dc, &format!("{mark}{name}"), &mut tr, win::window_text(), Gdi::DT_LEFT);
                        Gdi::SelectObject(dc, old);
                    }
                    Line::Prop(i) => {
                        let p = &self.props[*i];
                        let name_r = RECT { right: split, ..lr };
                        if self.chosen == Some(k) {
                            win::fill(dc, &name_r, tint);
                        }
                        let old = Gdi::SelectObject(dc, self.font.into());
                        let mut tr = RECT { left: pad * 2, right: split - pad / 2, ..lr };
                        draw(dc, p.name, &mut tr, win::window_text(), Gdi::DT_LEFT);
                        let mut vr = RECT { left: split + pad / 2, right: lr.right - pad / 2, ..lr };
                        match &p.ed {
                            Ed::Color => {
                                // A swatch before the colour's words.
                                if let Some(c) = phai_ui::theme::parse_color(&p.value) {
                                    let sw = RECT { left: vr.left, top: lr.top + pad / 2, right: vr.left + win::px(h, 18), bottom: lr.bottom - pad / 2 };
                                    win::round_rect(dc, sw, c, rule, win::px(h, 3));
                                }
                                vr.left += win::px(h, 24);
                                draw(dc, &p.value, &mut vr, win::window_text(), Gdi::DT_LEFT);
                            }
                            Ed::Dialog => {
                                let first = p.value.lines().next().unwrap_or_default();
                                let more = if p.value.lines().count() > 1 { "  …" } else { "" };
                                let mut words = RECT { right: vr.right - win::px(h, 26), ..vr };
                                draw(dc, &format!("{first}{more}"), &mut words, win::window_text(), Gdi::DT_LEFT);
                                let b = RECT { left: vr.right - win::px(h, 24), top: lr.top + pad / 3, right: vr.right, bottom: lr.bottom - pad / 3 };
                                win::round_rect(dc, b, bg, rule, win::px(h, 4));
                                let mut br = b;
                                draw(dc, "…", &mut br, win::window_text(), Gdi::DT_CENTER);
                            }
                            Ed::ReadOnly => draw(dc, &p.value, &mut vr, muted, Gdi::DT_LEFT),
                            _ => draw(dc, &p.value, &mut vr, win::window_text(), Gdi::DT_LEFT),
                        }
                        Gdi::SelectObject(dc, old);
                        win::fill(dc, &RECT { left: split, right: split + 1, ..lr }, rule);
                    }
                }
                win::fill(dc, &RECT { top: lr.bottom - 1, ..lr }, rule);
            }
            let _ = Gdi::EndPaint(h, &ps);
        }
    }

    fn line_at(&self, y: i32) -> Option<usize> {
        let k = self.top + (y / self.row_h().max(1)).max(0) as usize;
        (k < self.lines.len()).then_some(k)
    }

    fn click(&mut self, x: i32, y: i32, double: bool) {
        let Some(k) = self.line_at(y) else { return };
        self.close_editor(true);
        match self.lines[k] {
            Line::Heading(name) => {
                if let Some(at) = self.folded.iter().position(|f| *f == name) {
                    self.folded.remove(at);
                } else {
                    self.folded.push(name);
                }
                self.arrange();
                self.update_scroll();
            }
            Line::Prop(i) => {
                self.chosen = Some(k);
                self.say_help();
                // A click on a value edits it; on a name, a double-click does.
                if x >= self.split() || double {
                    self.edit(k, i);
                }
            }
        }
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
        }
    }

    /// Opens the editor a property's kind wants, over its value.
    fn edit(&mut self, line: usize, i: usize) {
        let p = self.props[i].clone();
        let lr = self.line_rect(line);
        let split = self.split();
        let cell = RECT { left: split + 1, ..lr };
        match p.ed {
            Ed::ReadOnly => {}
            Ed::Bool => {
                // As Visual Studio does on a double-click: the other value, at once.
                self.commit(p.key, &yes_no(p.value != "True"));
            }
            Ed::Text | Ed::Number => self.open_box(cell, i, &p.value),
            Ed::Color => self.pick_colour(i, &p.value),
            Ed::Choice(items, free) => {
                const CBS_DROPDOWN: u32 = 0x0002;
                const CBS_DROPDOWNLIST: u32 = 0x0003;
                let style = if free { CBS_DROPDOWN } else { CBS_DROPDOWNLIST };
                let refs: Vec<&str> = items.iter().map(String::as_str).collect();
                let combo = win::combo(self.grid, ID_EDITOR, self.font, &refs);
                unsafe {
                    let old = wm::GetWindowLongW(combo, wm::GWL_STYLE) as u32;
                    wm::SetWindowLongW(combo, wm::GWL_STYLE, ((old & !0x3) | style) as i32);
                }
                win::move_to(combo, cell.left, cell.top, cell.right - cell.left, win::px(self.grid, 300));
                if free {
                    win::set_text(combo, &p.value);
                }
                let at = items.iter().position(|s| *s == p.value).unwrap_or(usize::MAX);
                unsafe {
                    wm::SendMessageW(combo, 0x014E /* CB_SETCURSEL */, Some(WPARAM(at)), None);
                    let _ = km::SetFocus(Some(combo));
                    wm::SendMessageW(combo, 0x014F /* CB_SHOWDROPDOWN */, Some(WPARAM(1)), None);
                }
                self.editor = Some((combo, i));
            }
            Ed::Dialog => self.open_dialog(i),
            Ed::Anchor => self.pick_anchor(i, cell),
        }
    }

    fn open_box(&mut self, cell: RECT, i: usize, value: &str) {
        let e = win::edit(self.grid, value, ID_EDITOR, self.font, false);
        win::move_to(e, cell.left, cell.top, cell.right - cell.left, cell.bottom - cell.top);
        unsafe {
            let _ = windows::Win32::UI::Shell::SetWindowSubclass(e, Some(box_keys), 11, 0);
            let _ = km::SetFocus(Some(e));
            wm::SendMessageW(e, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
        }
        self.editor = Some((e, i));
    }

    /// Ends the in-place editor; `keep` puts what it holds into the model.
    fn close_editor(&mut self, keep: bool) {
        let Some((h, i)) = self.editor.take() else { return };
        let text = win::text_of(h);
        unsafe {
            let _ = wm::DestroyWindow(h);
        }
        if keep {
            if let Some(p) = self.props.get(i) {
                if text != p.value {
                    let key = p.key;
                    self.commit(key, &text);
                }
            }
        }
    }

    /// One change to the model, one undo step; the pane fills itself again afterwards.
    fn commit(&mut self, key: Key, value: &str) {
        let Some(s) = studio() else { return };
        s.checkpoint();
        s.apply(key, value);
        unsafe {
            let _ = wm::PostMessageW(Some(self.hwnd), WM_REFILL, WPARAM(0), LPARAM(0));
        }
    }

    fn pick_colour(&mut self, i: usize, value: &str) {
        use windows::Win32::UI::Controls::Dialogs::{ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW};
        static mut CUSTOM: [windows::Win32::Foundation::COLORREF; 16] = [windows::Win32::Foundation::COLORREF(0x00FF_FFFF); 16];
        let start = phai_ui::theme::parse_color(value).map(|c| c.colorref()).unwrap_or(0x00FF_FFFF);
        let mut cc = CHOOSECOLORW {
            lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
            hwndOwner: self.hwnd,
            rgbResult: windows::Win32::Foundation::COLORREF(start),
            lpCustColors: &raw mut CUSTOM as *mut _,
            Flags: CC_FULLOPEN | CC_RGBINIT,
            ..Default::default()
        };
        if unsafe { ChooseColorW(&mut cc) }.as_bool() {
            let v = cc.rgbResult.0;
            let hex = format!("#{:02x}{:02x}{:02x}", v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
            let key = self.props[i].key;
            self.commit(key, &hex);
        }
    }

    fn open_dialog(&mut self, i: usize) {
        let Some(s) = studio() else { return };
        let p = self.props[i].clone();
        match p.key {
            Key::FormTheme => s.command(crate::ID_THEME_IMPORT_PUB),
            Key::Items => {
                if let Some(text) = phai_ui::dialogs::ask_lines(Some(self.hwnd), p.name, t("One a line; a list's columns as Name:width."), &p.value) {
                    self.commit(p.key, &text);
                }
            }
            _ => {}
        }
    }

    fn pick_anchor(&mut self, i: usize, cell: RECT) {
        let a = parse_anchor(Some(&self.props[i].value)).unwrap_or_default();
        let mut pt = windows::Win32::Foundation::POINT { x: cell.left, y: cell.bottom };
        unsafe {
            let _ = Gdi::ClientToScreen(self.grid, &mut pt);
        }
        if let Some(chosen) = anchor_picker(self.hwnd, pt.x, pt.y, a) {
            let key = self.props[i].key;
            self.commit(key, &anchor_text(chosen).unwrap_or_else(|| "left top".into()));
        }
    }

    fn command(&mut self, id: i32, code: u16) {
        let Some(s) = studio() else { return };
        if self.filling {
            return;
        }
        match (id, code) {
            (ID_MODES, _) => {
                self.mode = code as usize;
                self.refresh();
            }
            (ID_CHOOSER, 1) => {
                let k = win::combo_selected(self.chooser);
                s.select(if k <= 0 { None } else { Some(k as usize - 1) });
            }
            (ID_SORT_CAT, 0) | (ID_SORT_AZ, 0) => {
                self.alphabetical = id == ID_SORT_AZ;
                self.arrange();
                self.chosen = None;
                self.update_scroll();
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
                }
            }
            _ => {}
        }
    }
}

fn draw(dc: Gdi::HDC, text: &str, r: &mut RECT, colour: Color, align: Gdi::DRAW_TEXT_FORMAT) {
    if text.is_empty() {
        return;
    }
    let mut w: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(colour.colorref()));
        Gdi::DrawTextW(dc, &mut w, r, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
    }
}

fn props_of<'a>(grid: HWND) -> Option<&'a mut Props> {
    studio().and_then(|s| s.props.as_mut()).filter(|p| p.grid == grid || p.hwnd == grid)
}

/// The value box keeps Enter (keep) and Escape (drop) for itself.
unsafe extern "system" fn box_keys(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    match msg {
        wm::WM_GETDLGCODE => LRESULT(0x0004),
        wm::WM_KEYDOWN if w.0 == 0x0D || w.0 == 0x1B => {
            let grid = unsafe { wm::GetParent(h).unwrap_or_default() };
            if let Some(p) = props_of(grid) {
                p.close_editor(w.0 == 0x0D);
            }
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

extern "system" fn grid_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(p) = props_of(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            p.paint();
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            p.update_scroll();
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            p.click(x, y, msg == wm::WM_LBUTTONDBLCLK);
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as i16 as i32;
            p.close_editor(true);
            p.top = (p.top as i64 - (delta / 40) as i64).max(0) as usize;
            p.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL => {
            let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
            unsafe {
                let _ = wm::GetScrollInfo(h, wm::SB_VERT, &mut si);
            }
            let pos = match wm::SCROLLBAR_COMMAND(win::loword(w.0) as i32) {
                wm::SB_LINEUP => si.nPos - 1,
                wm::SB_LINEDOWN => si.nPos + 1,
                wm::SB_PAGEUP => si.nPos - si.nPage as i32,
                wm::SB_PAGEDOWN => si.nPos + si.nPage as i32,
                wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
                _ => si.nPos,
            };
            p.close_editor(true);
            p.top = pos.max(0) as usize;
            p.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        // The in-place editor speaks: a choice made, a box left.
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_EDITOR => {
            match win::hiword(w.0) {
                1 /* CBN_SELCHANGE */ => {
                    // A list's choice is the value at once; a box that can be typed in waits.
                    let free = p.editor.as_ref().is_some_and(|(e, _)| unsafe { wm::GetWindowLongW(*e, wm::GWL_STYLE) } as u32 & 0x3 == 0x2);
                    if let Some((e, i)) = p.editor {
                        if !free {
                            let k = win::combo_selected(e).max(0) as usize;
                            let value = if let Ed::Choice(items, _) = &p.props[i].ed { items.get(k).cloned().unwrap_or_default() } else { String::new() };
                            let key = p.props[i].key;
                            p.editor = None;
                            unsafe {
                                let _ = wm::DestroyWindow(e);
                            }
                            p.commit(key, &value);
                        } else {
                            let k = win::combo_selected(e).max(0) as usize;
                            if let Ed::Choice(items, _) = &p.props[i].ed {
                                if let Some(v) = items.get(k) {
                                    win::set_text(e, v);
                                }
                            }
                        }
                    }
                }
                0x0200 /* EN_KILLFOCUS */ | 10 /* CBN_KILLFOCUS */ | 8 /* CBN_CLOSEUP */ if win::hiword(w.0) != 8 => p.close_editor(true),
                _ => {}
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_COMMAND => {
            if let Some(p) = studio().and_then(|s| s.props.as_mut()).filter(|p| p.hwnd == h) {
                p.command(win::loword(w.0) as i32, win::hiword(w.0));
            }
            LRESULT(0)
        }
        WM_REFILL => {
            if let Some(p) = studio().and_then(|s| s.props.as_mut()).filter(|p| p.hwnd == h) {
                p.refresh();
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            if let Some(p) = studio().and_then(|s| s.props.as_ref()).filter(|p| p.hwnd == h) {
                p.layout();
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

// --- the anchor picker: a box with an arm to each edge, as Visual Studio's ---------------------

struct Picker {
    anchors: Anchors,
    done: bool,
    keep: bool,
}

fn arm_rects(h: HWND) -> [RECT; 4] {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let (cx, cy) = (r.right / 2, r.bottom / 2);
    let (len, th, gap) = (r.right / 2 - win::px(h, 14), win::px(h, 8), win::px(h, 16));
    [
        RECT { left: cx - gap - len, top: cy - th / 2, right: cx - gap, bottom: cy + th / 2 },
        RECT { left: cx - th / 2, top: cy - gap - len / 2, right: cx + th / 2, bottom: cy - gap },
        RECT { left: cx + gap, top: cy - th / 2, right: cx + gap + len, bottom: cy + th / 2 },
        RECT { left: cx - th / 2, top: cy + gap, right: cx + th / 2, bottom: cy + gap + len / 2 },
    ]
}

extern "system" fn picker_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(pk) = (unsafe { win::state::<Picker>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            win::fill(dc, &r, win::window_background());
            let accent = phai_ui::theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let off = phai_ui::theme::style("group", State::default()).border_or(Color::rgb(0xd4, 0xd4, 0xd4));
            let (cx, cy, s) = (r.right / 2, r.bottom / 2, win::px(h, 12));
            win::round_rect(dc, RECT { left: cx - s, top: cy - s, right: cx + s, bottom: cy + s }, win::window_background(), win::window_text(), win::px(h, 3));
            let on = [pk.anchors.left, pk.anchors.top, pk.anchors.right, pk.anchors.bottom];
            for (k, a) in arm_rects(h).iter().enumerate() {
                win::fill(dc, a, if on[k] { accent } else { off });
            }
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_LBUTTONDOWN => {
            let (x, y) = win::point_of(l);
            let grow = win::px(h, 8);
            for (k, a) in arm_rects(h).iter().enumerate() {
                if x >= a.left - grow && x < a.right + grow && y >= a.top - grow && y < a.bottom + grow {
                    match k {
                        0 => pk.anchors.left = !pk.anchors.left,
                        1 => pk.anchors.top = !pk.anchors.top,
                        2 => pk.anchors.right = !pk.anchors.right,
                        _ => pk.anchors.bottom = !pk.anchors.bottom,
                    }
                    unsafe {
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x0D || w.0 == 0x1B => {
            pk.keep = w.0 == 0x0D;
            pk.done = true;
            LRESULT(0)
        }
        wm::WM_ACTIVATE if win::loword(w.0) == 0 => {
            // Clicked away: what was picked stands.
            pk.keep = true;
            pk.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// Shows the picker below `(x, y)` until it loses the focus; the anchors then, or `None` on Escape.
fn anchor_picker(owner: HWND, x: i32, y: i32, start: Anchors) -> Option<Anchors> {
    let class = win::register("PhaiAnchorPicker", Some(picker_proc), None);
    unsafe {
        let top = wm::GetAncestor(owner, wm::GA_ROOT);
        let (w, h) = (win::px(owner, 150), win::px(owner, 120));
        let p = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, x, y, w, h, Some(top), None, Some(win::instance()), None).ok()?;
        win::set_state(p, Box::new(Picker { anchors: start, done: false, keep: false }));
        let _ = wm::ShowWindow(p, wm::SW_SHOW);
        let _ = km::SetFocus(Some(p));
        let mut msg = wm::MSG::default();
        while win::state::<Picker>(p).is_some_and(|s| !s.done) && wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let out = win::state::<Picker>(p).and_then(|s| s.keep.then_some(s.anchors));
        drop(win::take_state::<Picker>(p));
        let _ = wm::DestroyWindow(p);
        out
    }
}
