//! The properties pane: the chosen control's fields - or the form's, when nothing is chosen - as
//! rows of label and box. A change applies as it is typed; one undo step per box visited.

use phai_form::{anchor_text, parse_anchor, Anchors, Kind, DOCKS};
use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::studio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

impl Key {
    const ALL: [Key; 16] = [Key::FormName, Key::FormTitle, Key::FormWidth, Key::FormHeight, Key::FormTheme, Key::FormFont, Key::Id, Key::Text, Key::X, Key::Y, Key::W, Key::H, Key::Font, Key::Class, Key::Event, Key::Items];

    fn label(self) -> &'static str {
        match self {
            Key::FormName => t("Name"),
            Key::FormTitle => t("Title"),
            Key::FormWidth | Key::W => t("Width"),
            Key::FormHeight | Key::H => t("Height"),
            Key::FormTheme => t("Theme"),
            Key::Id => t("Id"),
            Key::Text => t("Text"),
            Key::X => "X",
            Key::Y => "Y",
            Key::Class => t("Class"),
            Key::Event => t("Event"),
            Key::Items => t("Items"),
            Key::FormFont | Key::Font => t("Font size"),
        }
    }

    fn for_form(self) -> bool {
        matches!(self, Key::FormName | Key::FormTitle | Key::FormWidth | Key::FormHeight | Key::FormTheme | Key::FormFont)
    }
}

struct Row {
    key: Key,
    label: HWND,
    edit: HWND,
}

pub struct Props {
    pub hwnd: HWND,
    heading: HWND,
    rows: Vec<Row>,
    anchor_label: HWND,
    anchors: [HWND; 4],
    dock_label: HWND,
    dock: HWND,
    /// Set while the boxes are filled from the model, so their changes are not taken as edits.
    filling: bool,
}

const ID_ROW: i32 = 3000;
const ID_ANCHOR: i32 = 3100;
const ID_DOCK: i32 = 3200;

impl Props {
    pub fn new(parent: HWND, id: i32, font: HFONT) -> Props {
        let class = win::register("PhaiStudioProps", Some(proc_), None);
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        let heading = win::label(hwnd, "", 2999, font);
        let rows = Key::ALL.iter().enumerate().map(|(i, k)| Row { key: *k, label: win::label(hwnd, k.label(), 0, font), edit: win::edit(hwnd, "", ID_ROW + i as i32, font, false) }).collect();
        let anchor_label = win::label(hwnd, t("Anchor"), 0, font);
        let names = [t("Left"), t("Top"), t("Right"), t("Bottom")];
        let anchors = std::array::from_fn(|i| win::checkbox(hwnd, names[i], ID_ANCHOR + i as i32, font, false));
        let dock_label = win::label(hwnd, t("Dock"), 0, font);
        let dock = win::combo(hwnd, ID_DOCK, font, &DOCKS);
        Props { hwnd, heading, rows, anchor_label, anchors, dock_label, dock, filling: false }
    }

    /// Fills every box from the model and shows the rows that belong to what is chosen.
    pub fn refresh(&mut self) {
        let Some(s) = studio() else { return };
        self.filling = true;
        // A code file has no properties: its name, and nothing else.
        if s.doc.code.is_some() {
            let name = s.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            win::set_text(self.heading, &name);
            for r in &self.rows {
                win::show(r.label, false);
                win::show(r.edit, false);
            }
            for h in [self.anchor_label, self.dock_label, self.dock].into_iter().chain(self.anchors) {
                win::show(h, false);
            }
            self.filling = false;
            return;
        }
        let f = &s.doc.form;
        let chosen = s.selected.and_then(|i| f.control.get(i));
        win::set_text(self.heading, &match chosen {
            Some(c) => format!("{} · {}", crate::kind_label(c.kind), c.id),
            None => format!("{} · {}", t("Form"), f.form.name),
        });
        for r in &self.rows {
            let (show, value) = match (chosen, r.key) {
                (None, k) if k.for_form() => (
                    true,
                    match k {
                        Key::FormName => f.form.name.clone(),
                        Key::FormTitle => f.form.title.clone(),
                        Key::FormWidth => f.form.size[0].to_string(),
                        Key::FormHeight => f.form.size[1].to_string(),
                        Key::FormFont => f.form.font_size.map(|n| n.to_string()).unwrap_or_default(),
                        _ => f.form.theme.clone().unwrap_or_default(),
                    },
                ),
                (Some(c), k) if !k.for_form() => {
                    let [x, y, w, h] = c.rect();
                    match k {
                        Key::Id => (true, c.id.clone()),
                        Key::Text => (c.kind.has_text() || c.kind == Kind::Statusbar, c.text.clone().unwrap_or_default()),
                        Key::X => (c.dock.is_none(), x.to_string()),
                        Key::Y => (c.dock.is_none(), y.to_string()),
                        Key::W => (c.dock.is_none() || c.dock.as_deref() == Some("left") || c.dock.as_deref() == Some("right"), w.to_string()),
                        Key::H => (c.dock.is_none() || c.dock.as_deref() == Some("top") || c.dock.as_deref() == Some("bottom"), h.to_string()),
                        Key::Class => (true, c.class.clone().unwrap_or_default()),
                        Key::Font => (c.kind.has_text() || matches!(c.kind, Kind::Combo | Kind::List | Kind::Tabs | Kind::Statusbar), c.font_size.map(|n| n.to_string()).unwrap_or_default()),
                        Key::Event => (c.kind.event().is_some(), c.handler().unwrap_or_default().to_string()),
                        _ => match c.kind {
                            Kind::Combo => (true, c.items.join(", ")),
                            Kind::Tabs => (true, c.tabs.join(", ")),
                            Kind::List => (true, c.columns.iter().map(|(n, w)| format!("{n}:{w}")).collect::<Vec<_>>().join(", ")),
                            _ => (false, String::new()),
                        },
                    }
                }
                _ => (false, String::new()),
            };
            win::show(r.label, show);
            win::show(r.edit, show);
            if show && win::text_of(r.edit) != value {
                win::set_text(r.edit, &value);
            }
        }
        let control_rows = chosen.is_some();
        win::show(self.anchor_label, control_rows);
        win::show(self.dock_label, control_rows);
        win::show(self.dock, control_rows);
        let a = chosen.map(|c| parse_anchor(c.anchor.as_deref()).unwrap_or_default()).unwrap_or_default();
        for (i, on) in [a.left, a.top, a.right, a.bottom].into_iter().enumerate() {
            win::show(self.anchors[i], control_rows);
            win::set_checked(self.anchors[i], on);
        }
        if let Some(c) = chosen {
            let d = DOCKS.iter().position(|d| Some(*d) == c.dock.as_deref()).unwrap_or(0);
            unsafe {
                wm::SendMessageW(self.dock, 0x014E /* CB_SETCURSEL */, Some(WPARAM(d)), None);
            }
            for h in self.anchors {
                win::enable(h, c.dock.is_none());
            }
        }
        // Event names are the programmer's methods; the rest are words of the form.
        self.filling = false;
        self.layout();
    }

    /// Stacks the shown rows: label on the left, box on the right - Windows' spacing.
    pub fn layout(&self) {
        let h = self.hwnd;
        let mut r = windows::Win32::Foundation::RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        let px = |v| win::px(h, v);
        let (m, row, gap, lw) = (px(11), px(23), px(7), px(64));
        let width = r.right - r.left;
        let mut y = m;
        win::move_to(self.heading, m, y, width - 2 * m, row);
        y += row + gap + px(4);
        for rw in self.rows.iter().filter(|rw| unsafe { wm::IsWindowVisible(rw.edit) }.as_bool()) {
            win::move_to(rw.label, m, y, lw, row);
            win::move_to(rw.edit, m + lw + px(6), y, width - 2 * m - lw - px(6), row);
            y += row + gap;
        }
        if unsafe { wm::IsWindowVisible(self.dock) }.as_bool() {
            win::move_to(self.dock_label, m, y, lw, row);
            win::move_to(self.dock, m + lw + px(6), y, width - 2 * m - lw - px(6), px(200));
            y += row + gap;
            win::move_to(self.anchor_label, m, y, lw, row);
            let col = (width - 2 * m - lw - px(6)) / 2;
            for (i, a) in self.anchors.iter().enumerate() {
                let (cx, cy) = (m + lw + px(6) + (i as i32 % 2) * col, y + (i as i32 / 2) * (px(20) + gap));
                win::move_to(*a, cx, cy, col, px(20));
            }
        }
    }

    /// New words after a language switch.
    pub fn relabel(&mut self) {
        for r in &self.rows {
            win::set_text(r.label, r.key.label());
        }
        win::set_text(self.anchor_label, t("Anchor"));
        win::set_text(self.dock_label, t("Dock"));
        for (h, name) in self.anchors.iter().zip([t("Left"), t("Top"), t("Right"), t("Bottom")]) {
            win::set_text(*h, name);
        }
        self.refresh();
    }

    fn command(&mut self, id: i32, code: u16) {
        let Some(s) = studio() else { return };
        if self.filling {
            return;
        }
        if (ID_ROW..ID_ROW + 100).contains(&id) {
            let Some(row) = self.rows.get((id - ID_ROW) as usize) else { return };
            match code {
                0x0100 => s.checkpoint(), // EN_SETFOCUS: one undo step per box visited
                0x0300 => {
                    // EN_CHANGE
                    let (key, text) = (row.key, win::text_of(row.edit));
                    s.apply(key, &text);
                }
                _ => {}
            }
        } else if (ID_ANCHOR..ID_ANCHOR + 4).contains(&id) && code == 0 {
            let on: Vec<bool> = self.anchors.iter().map(|h| win::is_checked(*h)).collect();
            let a = Anchors { left: on[0], top: on[1], right: on[2], bottom: on[3] };
            if let Some(i) = s.selected {
                s.edit(|f| f.control[i].anchor = anchor_text(a));
            }
        } else if id == ID_DOCK && code == 1 {
            let d = win::combo_selected(self.dock).max(0) as usize;
            if let Some(i) = s.selected {
                s.edit(|f| {
                    let c = &mut f.control[i];
                    c.dock = (d > 0).then(|| DOCKS[d].to_string());
                    if c.bounds.is_none() {
                        c.bounds = Some([0, 0, 100, 30]);
                    }
                });
                s.rebuild();
                self.refresh();
            }
        }
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
        wm::WM_SIZE => {
            if let Some(p) = studio().and_then(|s| s.props.as_ref()).filter(|p| p.hwnd == h) {
                p.layout();
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
