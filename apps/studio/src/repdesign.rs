//! The Report Designer, as FastReport's and DevExpress XtraReports': the report's bands one under
//! another on a sheet, each captioned; its items - words, fields, totals, lines, boxes, pictures -
//! placed with the mouse (dragged to move, their corner to size, onto another band to move there),
//! a band's lower edge dragged to make it taller. On the left the columns of the table it is bound
//! to: double-click one for its field in the detail band and its caption in the page header. On
//! the right the chosen thing's properties: double-click one to change it. Preview shows the
//! pages with the table's real rows; Print and PDF print them. Save writes the `.phaireport` and
//! `<name>.rs`, the code a program shows and prints it with.

use std::path::{Path, PathBuf};

use phai_form::report::{generate, BandSpec, DataSpec, ItemSpec, ReportFile};
use phai_ui::report::{self as rp, BandKind};
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_CANVAS: i32 = 10;
const ID_FIELDS: i32 = 11;
const ID_PROPS: i32 = 12;
const ID_STATUS: i32 = 13;
const ID_LABEL: i32 = 20;
const ID_FIELD: i32 = 21;
const ID_TOTAL: i32 = 22;
const ID_LINE: i32 = 23;
const ID_BOX: i32 = 24;
const ID_PICTURE: i32 = 25;
const ID_DATA: i32 = 30;
const ID_GROUPS: i32 = 31;
const ID_BANDS: i32 = 32;
const ID_PREVIEW: i32 = 40;
const ID_PRINT: i32 = 41;
const ID_PDF: i32 = 42;
const ID_SAVE: i32 = 43;
const TIMER_SHOT: usize = 0x5E7;

/// The caption strip over each band, design pixels.
const CAPTION: i32 = 20;
/// Room left of the sheet, design pixels.
const GUTTER: i32 = 24;
/// Positions snap to this many hundredths of an inch.
const SNAP: i32 = 5;

/// What is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sel {
    Report,
    Band(usize),
    Item(usize, usize),
}

#[derive(Clone, Copy, Debug)]
enum Drag {
    Move { band: usize, item: usize, from: (i32, i32), orig: [i32; 4] },
    Size { band: usize, item: usize, from: (i32, i32), orig: [i32; 4] },
    Band { band: usize, from: i32, orig: i32 },
}

struct Designer {
    path: PathBuf,
    file: ReportFile,
    undo: Vec<ReportFile>,
    redo: Vec<ReportFile>,
    /// What the file was before a drag, kept for its undo step.
    before: Option<ReportFile>,
    dirty: bool,
    sel: Sel,
    drag: Option<Drag>,
    scroll: i32,
    canvas: HWND,
    fields: HWND,
    props: HWND,
    status: HWND,
    buttons: Vec<(i32, HWND)>,
    font: Gdi::HFONT,
    /// The property each row of Properties shows.
    prop_keys: Vec<&'static str>,
    /// Opened on its own (`--report`): closing it ends the program.
    alone: bool,
}

fn state(h: HWND) -> Option<&'static mut Designer> {
    unsafe { win::state::<Designer>(h) }
}

/// The connections Server Explorer has - the open Studio's, or the remembered ones.
fn connections() -> Vec<phai_data::Connection> {
    if let Some(c) = crate::studio().and_then(|s| s.servers.as_ref()).map(|s| s.conns.clone()) {
        return c;
    }
    crate::settings_dir_pub().and_then(|d| std::fs::read_to_string(d.join("connections.txt")).ok()).map(|t| phai_data::from_lines(&t)).unwrap_or_default()
}

/// The connection a report previews with: the Server Explorer one it names; else the project's
/// own SQLite file (`src/data_sources.rs` beside the report); else the only one there is.
fn connection_for(d: &DataSpec, report: &Path) -> Option<phai_data::Connection> {
    let all = connections();
    all.iter()
        .find(|c| c.label() == d.connection)
        .cloned()
        .or_else(|| report.parent().and_then(|src| std::fs::read_to_string(src.join("data_sources.rs")).ok()).and_then(|t| sqlite_of(&t)))
        .or_else(|| if all.len() == 1 { all.first().cloned() } else { None })
}

/// The SQLite file a project's `data_sources.rs` names, as Phai Studio writes it.
fn sqlite_of(data_sources: &str) -> Option<phai_data::Connection> {
    if !data_sources.contains("Kind::Sqlite") {
        return None;
    }
    let at = data_sources.find("host: \"")? + "host: \"".len();
    let rest = &data_sources[at..];
    let end = rest.find("\".into()")?;
    let host = rest[..end].replace("\\\\", "\\").replace("\\\"", "\"");
    Some(phai_data::Connection { kind: phai_data::Kind::Sqlite, host, ..Default::default() })
}

/// `amount_due` as a caption: `Amount due`.
fn caption_of(column: &str) -> String {
    let s = column.replace('_', " ");
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

/// The bands in the order the designer shows them, as XtraReports does.
fn display_order(f: &ReportFile) -> Vec<usize> {
    let rank = |b: &BandSpec| -> i32 {
        match BandKind::from_word(&b.kind).unwrap_or_default() {
            BandKind::ReportHeader => 0,
            BandKind::PageHeader => 10,
            BandKind::GroupHeader => 20 + b.group as i32,
            BandKind::Detail => 100,
            BandKind::GroupFooter => 200 - b.group as i32,
            BandKind::ReportFooter => 300,
            BandKind::PageFooter => 310,
        }
    };
    let mut order: Vec<usize> = (0..f.bands.len()).collect();
    order.sort_by_key(|i| rank(&f.bands[*i]));
    order
}

impl Designer {
    fn u2p(&self, u: i32) -> i32 {
        u * self.zoom_dpi() / 100
    }
    fn p2u(&self, p: i32) -> i32 {
        p * 100 / self.zoom_dpi().max(1)
    }

    /// Pixels to an inch on the canvas: the screen's, or fewer when the sheet would be wider than
    /// the canvas - it is made to fit.
    fn zoom_dpi(&self) -> i32 {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.canvas, &mut r);
        }
        let room = r.right - 2 * win::px(self.canvas, GUTTER);
        let dpi = win::dpi(self.canvas);
        if room <= 0 {
            return dpi;
        }
        dpi.min(room * 100 / self.page_width()).max(24)
    }

    /// The width items have, hundredths of an inch: the sheet inside its margins (A4).
    fn page_width(&self) -> i32 {
        let paper = if self.file.report.landscape { 1169 } else { 827 };
        (paper - self.file.report.margins[0] - self.file.report.margins[2]).max(100)
    }

    /// Each band shown: its index, its caption's top and its body's top and height, canvas
    /// pixels before scrolling.
    fn rows(&self) -> Vec<(usize, i32, i32, i32)> {
        let mut y = win::px(self.canvas, 12);
        let cap = win::px(self.canvas, CAPTION);
        let mut out = Vec::new();
        for i in display_order(&self.file) {
            let h = self.u2p(self.file.bands[i].height);
            out.push((i, y, y + cap, h));
            y += cap + h + 1;
        }
        out
    }

    fn left(&self) -> i32 {
        win::px(self.canvas, GUTTER)
    }

    fn total_height(&self) -> i32 {
        self.rows().last().map_or(0, |r| r.2 + r.3) + win::px(self.canvas, 40)
    }

    fn change(&mut self) {
        self.undo.push(self.file.clone());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }

    fn band_of_sel(&self) -> Option<usize> {
        match self.sel {
            Sel::Band(b) | Sel::Item(b, _) => Some(b),
            Sel::Report => None,
        }
    }

    fn band_index(&self, kind: BandKind, group: usize) -> Option<usize> {
        self.file.bands.iter().position(|b| b.kind == kind.word() && b.group == group)
    }

    /// The band of `kind`, added when the report has none.
    fn ensure_band(&mut self, kind: BandKind, group: usize, height: i32) -> usize {
        if let Some(i) = self.band_index(kind, group) {
            return i;
        }
        self.file.bands.push(BandSpec { kind: kind.word().into(), group, height, ..Default::default() });
        self.file.bands.len() - 1
    }

    fn unique_name(&self, base: &str) -> String {
        let taken = |n: &str| self.file.bands.iter().flat_map(|b| &b.items).any(|i| i.name == n);
        if !base.ends_with(|c: char| c.is_ascii_digit()) && !taken(base) && !matches!(base, "label" | "line" | "box" | "picture" | "total") {
            return base.to_string();
        }
        (1..).map(|n| format!("{base}{n}")).find(|n| !taken(n)).unwrap_or_default()
    }

    /// Where a new item goes across `band`: after the last one, or at the left when that is full.
    fn next_x(&self, band: usize, width: i32) -> i32 {
        let right = self.file.bands[band].items.iter().map(|i| i.bounds[0] + i.bounds[2]).max().unwrap_or(0);
        if right + width <= self.page_width() {
            right
        } else {
            0
        }
    }

    fn add_item(&mut self, band: usize, item: ItemSpec) {
        self.change();
        self.file.bands[band].items.push(item);
        self.sel = Sel::Item(band, self.file.bands[band].items.len() - 1);
        self.refresh();
    }

    /// A column's field in the detail band, its caption above it in the page header.
    fn add_field(&mut self, column: &str) {
        let detail = self.ensure_band(BandKind::Detail, 0, 22);
        let x = self.next_x(detail, 150);
        let header = self.ensure_band(BandKind::PageHeader, 0, 26);
        let numeric = self.is_numeric(column);
        let align = if numeric { "right" } else { "" };
        let caption = ItemSpec { kind: "label".into(), name: self.unique_name(&format!("caption_{column}")), bounds: [x, 4, 150, 18], text: caption_of(column), align: align.into(), ..Default::default() };
        self.change();
        self.file.bands[header].items.push(caption);
        let field = ItemSpec { kind: "field".into(), name: self.unique_name(column), bounds: [x, 2, 150, 18], field: column.into(), align: align.into(), ..Default::default() };
        self.file.bands[detail].items.push(field);
        self.sel = Sel::Item(detail, self.file.bands[detail].items.len() - 1);
        self.refresh();
    }

    /// Whether the table's column holds numbers, by its type.
    fn is_numeric(&self, column: &str) -> bool {
        let Some(c) = connection_for(&self.file.data, &self.path) else { return false };
        let Ok(cols) = c.columns(&self.file.data.database, &self.file.data.table) else { return false };
        cols.iter().find(|x| x.name == column).is_some_and(|x| {
            let ty = x.ty.to_ascii_lowercase();
            ["int", "dec", "num", "real", "float", "double", "money"].iter().any(|k| ty.contains(k))
        })
    }

    fn selected_item(&mut self) -> Option<&mut ItemSpec> {
        match self.sel {
            Sel::Item(b, i) => self.file.bands.get_mut(b)?.items.get_mut(i),
            _ => None,
        }
    }

    fn refresh(&mut self) {
        self.fill_props();
        self.fill_fields();
        self.update_scroll();
        self.title();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.canvas), None, false);
        }
        let problems = self.file.check();
        let words = match problems.first() {
            Some(p) => format!("⚠ {}", p.0),
            None if self.file.data.is_bound() => tf("Rows from {0} - double-click a column to put it on the report", &[&self.file.data.table]),
            None => t("Not bound to a table - Data... chooses one from Server Explorer").to_string(),
        };
        win::set_text(self.status, &words);
    }

    fn title(&self) {
        let name = self.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let star = if self.dirty { "*" } else { "" };
        let w = win::wide(&format!("{name}{star} - {}", t("Report Designer")));
        unsafe {
            let _ = wm::SetWindowTextW(self.canvas_parent(), PCWSTR(w.as_ptr()));
        }
    }

    fn canvas_parent(&self) -> HWND {
        unsafe { wm::GetParent(self.canvas) }.unwrap_or_default()
    }

    fn fill_fields(&mut self) {
        list_clear(self.fields);
        for c in &self.file.data.columns {
            win::list_add(self.fields, &[c]);
        }
    }

    fn props_of(&self) -> Vec<(&'static str, String, String)> {
        let f = &self.file;
        let yes = |b: bool| if b { "true".to_string() } else { "false".to_string() };
        match self.sel {
            Sel::Report => vec![
                ("name", t("Name").into(), f.report.name.clone()),
                ("title", t("Title").into(), f.report.title.clone()),
                ("landscape", t("Landscape").into(), yes(f.report.landscape)),
                ("margins", t("Margins").into(), f.report.margins.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ")),
                ("groups", t("Groups").into(), f.report.groups.join(", ")),
                ("css", "CSS".into(), f.report.css.replace('\n', " ")),
                ("connection", t("Connection").into(), f.data.connection.clone()),
                ("database", t("Database").into(), f.data.database.clone()),
                ("table", t("Table").into(), f.data.table.clone()),
                ("columns", t("Columns").into(), f.data.columns.join(", ")),
                ("filter", t("Filter (WHERE)").into(), f.data.filter.clone()),
                ("order", t("Sort").into(), f.data.order.join(", ")),
                ("sql", "SQL".into(), f.data.sql.replace('\n', " ")),
            ],
            Sel::Band(b) => {
                let band = &f.bands[b];
                let mut v = vec![("kind", t("Band").into(), t(BandKind::from_word(&band.kind).unwrap_or_default().caption()).to_string()), ("height", t("Height").into(), band.height.to_string())];
                if band.kind == "group_header" {
                    v.push(("group", t("Group by").into(), f.report.groups.get(band.group).cloned().unwrap_or_default()));
                    v.push(("new_page", t("New page for each group").into(), yes(band.new_page)));
                }
                v
            }
            Sel::Item(b, i) => {
                let it = &f.bands[b].items[i];
                let mut v = vec![("name", t("Name").into(), it.name.clone()), ("kind", t("Kind").into(), it.kind.clone())];
                match it.kind.as_str() {
                    "label" => v.push(("text", t("Text").into(), it.text.clone())),
                    "field" => v.push(("field", t("Column").into(), it.field.clone())),
                    "total" => {
                        v.push(("func", t("Function").into(), if it.func.is_empty() { "sum".into() } else { it.func.clone() }));
                        v.push(("field", t("Column").into(), it.field.clone()));
                    }
                    "picture" => v.push(("image", t("Picture").into(), it.image.clone())),
                    _ => {}
                }
                if matches!(it.kind.as_str(), "field" | "total") {
                    v.push(("format", t("Format").into(), it.format.clone()));
                }
                for (k, l, n) in [("x", "X", 0), ("y", "Y", 1), ("width", "Width", 2), ("height", "Height", 3)] {
                    v.push((k, t(l).into(), it.bounds[n].to_string()));
                }
                if matches!(it.kind.as_str(), "label" | "field" | "total") {
                    v.push(("align", t("Align").into(), if it.align.is_empty() { "left".into() } else { it.align.clone() }));
                    v.push(("font", t("Font").into(), it.font.clone()));
                    v.push(("size", t("Size (points)").into(), if it.size > 0 { it.size.to_string() } else { String::new() }));
                    v.push(("bold", t("Bold").into(), it.bold.map(yes).unwrap_or_default()));
                    v.push(("italic", t("Italic").into(), yes(it.italic)));
                    v.push(("background", t("Back colour").into(), it.background.clone()));
                }
                if it.kind != "picture" {
                    v.push(("color", t("Colour").into(), it.color.clone()));
                    v.push(("border", t("Border").into(), if it.border > 0 { it.border.to_string() } else { String::new() }));
                }
                v.push(("class", t("CSS class").into(), it.class.clone()));
                v
            }
        }
    }

    fn fill_props(&mut self) {
        list_clear(self.props);
        let rows = self.props_of();
        self.prop_keys = rows.iter().map(|r| r.0).collect();
        for (_, label, value) in &rows {
            win::list_add(self.props, &[label, value]);
        }
    }

    /// Asks for property `key`'s new value: a menu of its choices, or a box to type it in.
    fn edit_prop(&mut self, key: &str) {
        let owner = self.canvas_parent();
        let current = self.props_of().into_iter().find(|p| p.0 == key).map(|p| p.2).unwrap_or_default();
        let choose = |items: &[&str]| -> Option<String> {
            let menu: Vec<(u32, &str)> = items.iter().enumerate().map(|(i, s)| (i as u32 + 1, *s)).collect();
            let n = win::popup(owner, &menu);
            (n > 0).then(|| items[n as usize - 1].to_string())
        };
        let value = match key {
            "kind" | "connection" | "database" | "table" | "group" => {
                if matches!(key, "connection" | "database" | "table") {
                    self.pick_data();
                }
                return;
            }
            "groups" => {
                self.groups_menu();
                return;
            }
            "landscape" | "italic" | "new_page" => choose(&["true", "false"]),
            "bold" => choose(&["", "true", "false"]),
            "align" => choose(&["left", "center", "right"]),
            "func" => choose(&["sum", "count", "avg", "min", "max"]),
            "field" => {
                let cols: Vec<&str> = self.file.data.columns.iter().map(String::as_str).collect();
                if cols.is_empty() {
                    phai_ui::dialogs::ask(Some(owner), t("Column"), t("Column"), &current, false)
                } else {
                    choose(&cols)
                }
            }
            "format" => choose(&["", "0", "0.00", "#,##0", "#,##0.00", "฿#,##0.00"]).or(None),
            "image" => crate::file_dialog_pub(owner, false, "Pictures (*.png;*.jpg;*.bmp;*.gif)", "*.png;*.jpg;*.jpeg;*.bmp;*.gif", "").map(|p| p.to_string_lossy().to_string()),
            "css" | "sql" => phai_ui::dialogs::ask_lines(Some(owner), key, key, &if key == "css" { self.file.report.css.clone() } else { self.file.data.sql.clone() }),
            "text" => phai_ui::dialogs::ask(Some(owner), t("Text"), t("Words - {page}, {pages}, {date}, {time}, {row}, {title} or a {column} are filled in"), &current, false),
            _ => phai_ui::dialogs::ask(Some(owner), key, key, &current, false),
        };
        if let Some(v) = value {
            self.set_prop(key, v.trim());
        }
    }

    fn set_prop(&mut self, key: &str, v: &str) {
        let num = |s: &str| s.trim().parse::<i32>().ok();
        let list = |s: &str| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect::<Vec<_>>();
        self.change();
        match self.sel {
            Sel::Report => {
                let f = &mut self.file;
                match key {
                    "name" => f.report.name = crate::form_type_name(v),
                    "title" => f.report.title = v.into(),
                    "landscape" => f.report.landscape = v == "true",
                    "margins" => {
                        let m: Vec<i32> = v.split(',').filter_map(num).collect();
                        if let [a] = m[..] {
                            f.report.margins = [a; 4];
                        } else if let [a, b, c, d] = m[..] {
                            f.report.margins = [a, b, c, d];
                        }
                    }
                    "css" => f.report.css = v.into(),
                    "columns" => f.data.columns = list(v),
                    "filter" => f.data.filter = v.into(),
                    "order" => f.data.order = list(v),
                    "sql" => f.data.sql = v.into(),
                    _ => {}
                }
            }
            Sel::Band(b) => {
                let band = &mut self.file.bands[b];
                match key {
                    "height" => band.height = num(v).unwrap_or(band.height).max(0),
                    "new_page" => band.new_page = v == "true",
                    _ => {}
                }
            }
            Sel::Item(..) => {
                let Some(it) = self.selected_item() else { return };
                match key {
                    "name" => it.name = v.into(),
                    "text" => it.text = v.into(),
                    "field" => it.field = v.into(),
                    "func" => it.func = if v == "sum" { String::new() } else { v.into() },
                    "format" => it.format = v.into(),
                    "align" => it.align = if v == "left" { String::new() } else { v.into() },
                    "font" => it.font = v.into(),
                    "size" => it.size = num(v).unwrap_or(0).max(0),
                    "bold" => it.bold = match v {
                        "true" => Some(true),
                        "false" => Some(false),
                        _ => None,
                    },
                    "italic" => it.italic = v == "true",
                    "color" => it.color = v.into(),
                    "background" => it.background = v.into(),
                    "border" => it.border = num(v).unwrap_or(0).max(0),
                    "class" => it.class = v.into(),
                    "image" => it.image = v.into(),
                    "x" => it.bounds[0] = num(v).unwrap_or(it.bounds[0]),
                    "y" => it.bounds[1] = num(v).unwrap_or(it.bounds[1]),
                    "width" => it.bounds[2] = num(v).unwrap_or(it.bounds[2]).max(0),
                    "height" => it.bounds[3] = num(v).unwrap_or(it.bounds[3]).max(0),
                    _ => {}
                }
            }
        }
        self.refresh();
    }

    /// Data...: a table from Server Explorer - which connection, database and table, by menus.
    fn pick_data(&mut self) {
        let owner = self.canvas_parent();
        let conns = connections();
        if conns.is_empty() {
            win::message(Some(owner), t("Add a connection in Server Explorer first: the report reads its rows from one of its tables."), t("Report Designer"), wm::MB_ICONINFORMATION);
            return;
        }
        let pick = |items: &[String]| -> Option<usize> {
            if items.len() == 1 {
                return Some(0);
            }
            let menu: Vec<(u32, &str)> = items.iter().enumerate().map(|(i, s)| (i as u32 + 1, s.as_str())).collect();
            let n = win::popup(owner, &menu);
            (n > 0).then(|| n as usize - 1)
        };
        let labels: Vec<String> = conns.iter().map(|c| c.label()).collect();
        let Some(ci) = pick(&labels) else { return };
        let c = &conns[ci];
        let fail = |e: String| {
            win::message(Some(owner), &e, t("Report Designer"), wm::MB_ICONWARNING);
        };
        let dbs = match c.databases() {
            Ok(d) if !d.is_empty() => d,
            Ok(_) => vec!["main".into()],
            Err(e) => return fail(e),
        };
        let Some(di) = pick(&dbs) else { return };
        let tables = match c.tables(&dbs[di]) {
            Ok(t) => t.into_iter().map(|(n, _)| n).collect::<Vec<_>>(),
            Err(e) => return fail(e),
        };
        if tables.is_empty() {
            return fail(tf("{0} has no tables", &[&dbs[di]]));
        }
        let Some(ti) = pick(&tables) else { return };
        let cols = match c.columns(&dbs[di], &tables[ti]) {
            Ok(c) => c,
            Err(e) => return fail(e),
        };
        self.change();
        self.file.data = DataSpec { connection: c.label(), database: dbs[di].clone(), table: tables[ti].clone(), columns: cols.iter().map(|c| c.name.clone()).collect(), ..Default::default() };
        if self.file.report.title.is_empty() || self.file.report.title == self.file.report.name {
            self.file.report.title = caption_of(&tables[ti]);
        }
        // A detail band with nothing in it yet: every column laid across it, as a wizard does.
        let empty = self.band_index(BandKind::Detail, 0).is_none_or(|d| self.file.bands[d].items.is_empty());
        if empty {
            let names: Vec<String> = cols.iter().map(|c| c.name.clone()).collect();
            let undo_len = self.undo.len();
            let width = (self.page_width() / names.len().max(1) as i32).clamp(60, 200);
            for n in names.iter().take((self.page_width() / 60) as usize) {
                self.add_field(n);
                if let Sel::Item(b, i) = self.sel {
                    let x = self.file.bands[b].items[i].bounds[0];
                    self.file.bands[b].items[i].bounds[2] = width;
                    let header = self.band_index(BandKind::PageHeader, 0).unwrap_or(0);
                    if let Some(cap) = self.file.bands[header].items.last_mut() {
                        cap.bounds[0] = x;
                        cap.bounds[2] = width;
                    }
                }
            }
            // One undo step for the table and its fields.
            self.undo.truncate(undo_len);
        }
        self.sel = Sel::Report;
        self.refresh();
    }

    /// Groups...: a group added on a column (its header showing it), or taken away.
    fn groups_menu(&mut self) {
        let owner = self.canvas_parent();
        let cols = self.file.data.columns.clone();
        let mut items: Vec<(u32, String)> = Vec::new();
        for (i, c) in cols.iter().enumerate().filter(|(_, c)| !self.file.report.groups.contains(c)) {
            items.push((100 + i as u32, tf("Group by {0}", &[c])));
        }
        if !self.file.report.groups.is_empty() {
            items.push((0, String::new()));
        }
        for (i, g) in self.file.report.groups.iter().enumerate() {
            items.push((200 + i as u32, tf("Remove group {0}", &[g])));
        }
        if items.is_empty() {
            win::message(Some(owner), t("Choose a table first (Data...): a group breaks on one of its columns."), t("Report Designer"), wm::MB_ICONINFORMATION);
            return;
        }
        let menu: Vec<(u32, &str)> = items.iter().map(|(i, s)| (*i, s.as_str())).collect();
        let n = win::popup(owner, &menu);
        if (100..200).contains(&n) {
            let col = cols[(n - 100) as usize].clone();
            self.change();
            let level = self.file.report.groups.len();
            self.file.report.groups.push(col.clone());
            let field = ItemSpec { kind: "field".into(), name: self.unique_name(&format!("group_{col}")), bounds: [0, 3, 400, 20], field: col.clone(), ..Default::default() };
            self.file.bands.push(BandSpec { kind: "group_header".into(), group: level, height: 26, items: vec![field], ..Default::default() });
            self.file.bands.push(BandSpec { kind: "group_footer".into(), group: level, height: 26, ..Default::default() });
            self.sel = Sel::Band(self.file.bands.len() - 2);
            self.refresh();
        } else if n >= 200 {
            let level = (n - 200) as usize;
            self.change();
            self.file.report.groups.remove(level);
            self.file.bands.retain(|b| !(matches!(b.kind.as_str(), "group_header" | "group_footer") && b.group == level));
            for b in &mut self.file.bands {
                if matches!(b.kind.as_str(), "group_header" | "group_footer") && b.group > level {
                    b.group -= 1;
                }
            }
            self.sel = Sel::Report;
            self.refresh();
        }
    }

    /// Bands...: the report's and page's headers and footers, each put in or taken out.
    fn bands_menu(&mut self) {
        let owner = self.canvas_parent();
        let kinds = [BandKind::ReportHeader, BandKind::PageHeader, BandKind::PageFooter, BandKind::ReportFooter];
        let labels: Vec<String> = kinds.iter().map(|k| format!("{}{}", if self.band_index(*k, 0).is_some() { "✓ " } else { "    " }, t(k.caption()))).collect();
        let menu: Vec<(u32, &str)> = labels.iter().enumerate().map(|(i, s)| (i as u32 + 1, s.as_str())).collect();
        let n = win::popup(owner, &menu);
        if n == 0 {
            return;
        }
        let k = kinds[n as usize - 1];
        self.change();
        match self.band_index(k, 0) {
            Some(i) => {
                self.file.bands.remove(i);
            }
            None => {
                self.file.bands.push(BandSpec { kind: k.word().into(), height: 30, ..Default::default() });
            }
        }
        self.sel = Sel::Report;
        self.refresh();
    }

    /// A tool's item put in the chosen band (the detail band when none is chosen).
    fn tool(&mut self, id: i32) {
        let band = self.band_of_sel().unwrap_or_else(|| self.ensure_band(BandKind::Detail, 0, 22));
        let owner = self.canvas_parent();
        let item = match id {
            ID_LABEL => ItemSpec { kind: "label".into(), name: self.unique_name("label"), bounds: [self.next_x(band, 150), 2, 150, 18], text: t("Label").into(), ..Default::default() },
            ID_FIELD => {
                let cols: Vec<&str> = self.file.data.columns.iter().map(String::as_str).collect();
                if cols.is_empty() {
                    self.pick_data();
                    return;
                }
                let menu: Vec<(u32, &str)> = cols.iter().enumerate().map(|(i, s)| (i as u32 + 1, *s)).collect();
                let n = win::popup(owner, &menu);
                if n == 0 {
                    return;
                }
                let col = cols[n as usize - 1].to_string();
                if self.band_of_sel().is_none() {
                    self.add_field(&col);
                    return;
                }
                ItemSpec { kind: "field".into(), name: self.unique_name(&col), bounds: [self.next_x(band, 150), 2, 150, 18], field: col, ..Default::default() }
            }
            ID_TOTAL => {
                // Sums the chosen field's column, in its group's footer or the report's.
                let col = match self.sel {
                    Sel::Item(b, i) if !self.file.bands[b].items[i].field.is_empty() => self.file.bands[b].items[i].field.clone(),
                    _ => {
                        let cols: Vec<&str> = self.file.data.columns.iter().map(String::as_str).collect();
                        if cols.is_empty() {
                            self.pick_data();
                            return;
                        }
                        let menu: Vec<(u32, &str)> = cols.iter().enumerate().map(|(i, s)| (i as u32 + 1, *s)).collect();
                        let n = win::popup(owner, &menu);
                        if n == 0 {
                            return;
                        }
                        cols[n as usize - 1].to_string()
                    }
                };
                let x = match self.sel {
                    Sel::Item(b, i) => self.file.bands[b].items[i].bounds[0],
                    _ => 0,
                };
                let target = match self.band_of_sel().map(|b| self.file.bands[b].kind.clone()) {
                    Some(k) if matches!(k.as_str(), "group_footer" | "report_footer" | "page_footer" | "group_header" | "report_header") => band,
                    _ => {
                        let level = self.file.report.groups.len();
                        if level > 0 {
                            self.ensure_band(BandKind::GroupFooter, level - 1, 26)
                        } else {
                            self.ensure_band(BandKind::ReportFooter, 0, 30)
                        }
                    }
                };
                let item = ItemSpec { kind: "total".into(), name: self.unique_name(&format!("total_{col}")), bounds: [x, 3, 150, 18], field: col, format: "#,##0.00".into(), align: "right".into(), ..Default::default() };
                self.add_item(target, item);
                return;
            }
            ID_LINE => ItemSpec { kind: "line".into(), name: self.unique_name("line"), bounds: [0, 2, self.page_width(), 0], ..Default::default() },
            ID_BOX => ItemSpec { kind: "box".into(), name: self.unique_name("box"), bounds: [self.next_x(band, 200), 0, 200, 40], ..Default::default() },
            ID_PICTURE => {
                let Some(p) = crate::file_dialog_pub(owner, false, "Pictures (*.png;*.jpg;*.bmp;*.gif)", "*.png;*.jpg;*.jpeg;*.bmp;*.gif", "") else { return };
                ItemSpec { kind: "picture".into(), name: self.unique_name("picture"), bounds: [self.next_x(band, 100), 0, 100, 60], image: self.relative(&p), ..Default::default() }
            }
            _ => return,
        };
        // An item taller than its band makes the band taller.
        if self.file.bands[band].height < item.bounds[1] + item.bounds[3] {
            self.file.bands[band].height = item.bounds[1] + item.bounds[3] + 4;
        }
        self.add_item(band, item);
    }

    /// A picture's path as the project names it: from its folder when inside it.
    fn relative(&self, p: &Path) -> String {
        let project = self.path.parent().and_then(Path::parent);
        project.and_then(|d| p.strip_prefix(d).ok()).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_else(|| p.to_string_lossy().to_string())
    }

    /// The report's rows, read as the program will read them.
    fn fetch(&self) -> Result<rp::Data, String> {
        let d = &self.file.data;
        if !d.is_bound() {
            return Ok(rp::Data::default());
        }
        let c = connection_for(d, &self.path).ok_or_else(|| tf("The connection {0} is not in Server Explorer - add it there, or choose the table again with Data...", &[&d.connection]))?;
        let sql = if d.sql.is_empty() {
            let cols = if d.columns.is_empty() { self.file.used_columns() } else { d.columns.clone() };
            let cols: Vec<&str> = cols.iter().map(String::as_str).collect();
            let sort = self.file.sort();
            let sort: Vec<&str> = sort.iter().map(String::as_str).collect();
            c.select_sql(&d.database, &d.table, &cols, &d.filter, &sort)
        } else {
            d.sql.clone()
        };
        let db = (!d.database.is_empty()).then_some(d.database.as_str());
        let g = c.query_all(db, &sql)?;
        Ok(rp::Data::new(g.columns, g.rows))
    }

    fn run(&mut self, id: i32) {
        let owner = self.canvas_parent();
        let data = match id {
            ID_PREVIEW | ID_PRINT | ID_PDF => match self.fetch() {
                Ok(d) => d,
                Err(e) => {
                    win::message(Some(owner), &e, t("Report Designer"), wm::MB_ICONWARNING);
                    return;
                }
            },
            _ => return,
        };
        let r = self.file.to_report();
        let rows = data.rows.len();
        match id {
            ID_PREVIEW => {
                r.preview(owner, data);
                win::set_text(self.status, &tf("Previewed with {0} rows", &[&rows]));
            }
            ID_PRINT => {
                let n = r.print(owner, data);
                if n > 0 {
                    win::set_text(self.status, &tf("Printed {0} pages", &[&n]));
                }
            }
            _ => {
                let suggested = format!("{}.pdf", self.file.module());
                let Some(p) = crate::file_dialog_pub(owner, true, "PDF (*.pdf)", "*.pdf", &suggested) else { return };
                let n = r.export_pdf(owner, data, &p);
                if n > 0 {
                    win::set_text(self.status, &tf("Saved {0} pages to {1}", &[&n, &p.display()]));
                    crate::open_with_windows(&p.to_string_lossy());
                } else {
                    win::message(Some(owner), t("The PDF could not be made: Windows' \"Microsoft Print to PDF\" printer is needed."), t("Report Designer"), wm::MB_ICONWARNING);
                }
            }
        }
    }

    /// Writes the `.phaireport` and its code, the code declared in the project's main.rs.
    fn save(&mut self) -> bool {
        let owner = self.canvas_parent();
        if let Some(p) = self.file.check().into_iter().next() {
            win::message(Some(owner), &format!("{}\n\n{}", t("The report has a mistake, so it is not saved:"), p.0), t("Report Designer"), wm::MB_ICONWARNING);
            return false;
        }
        if let Err(e) = save_files(&self.path, &self.file, connection_for(&self.file.data, &self.path).as_ref()) {
            win::message(Some(owner), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&self.path.display()])), t("Report Designer"), wm::MB_ICONWARNING);
            return false;
        }
        self.dirty = false;
        self.title();
        win::set_text(self.status, &tf("Saved - show it from a form with {0}::preview(ui.hwnd)", &[&self.file.module()]));
        true
    }

    fn do_undo(&mut self, redo: bool) {
        let (from, to) = if redo { (&mut self.redo, &mut self.undo) } else { (&mut self.undo, &mut self.redo) };
        if let Some(f) = from.pop() {
            to.push(std::mem::replace(&mut self.file, f));
            self.sel = Sel::Report;
            self.dirty = true;
            self.refresh();
        }
    }

    fn delete(&mut self) {
        if let Sel::Item(b, i) = self.sel {
            self.change();
            self.file.bands[b].items.remove(i);
            self.sel = Sel::Band(b);
            self.refresh();
        } else if let Sel::Band(b) = self.sel {
            let k = self.file.bands[b].kind.clone();
            if matches!(k.as_str(), "report_header" | "page_header" | "page_footer" | "report_footer") {
                self.change();
                self.file.bands.remove(b);
                self.sel = Sel::Report;
                self.refresh();
            }
        }
    }

    // --- the canvas ---------------------------------------------------------------------------

    /// What is at canvas point (`x`, `y`) - scrolled: a band's lower edge, an item's corner, an
    /// item, or a band.
    fn hit(&self, x: i32, y: i32) -> Option<(Sel, u8)> {
        let y = y + self.scroll;
        let left = self.left();
        let edge = win::px(self.canvas, 4);
        for (b, _cap, top, h) in self.rows() {
            if (y - (top + h)).abs() <= edge {
                return Some((Sel::Band(b), 2));
            }
            if y >= top && y < top + h {
                let band = &self.file.bands[b];
                for (i, it) in band.items.iter().enumerate().rev() {
                    let [ix, iy, iw, ih] = it.bounds;
                    let (px, py, pw, ph) = (left + self.u2p(ix), top + self.u2p(iy), self.u2p(iw).max(4), self.u2p(ih).max(4));
                    if (x - (px + pw)).abs() <= edge && (y - (py + ph)).abs() <= edge {
                        return Some((Sel::Item(b, i), 1));
                    }
                    if x >= px - 2 && x <= px + pw + 2 && y >= py - 2 && y <= py + ph + 2 {
                        return Some((Sel::Item(b, i), 0));
                    }
                }
                return Some((Sel::Band(b), 0));
            }
        }
        None
    }

    fn band_at(&self, y: i32) -> Option<(usize, i32)> {
        let y = y + self.scroll;
        self.rows().into_iter().find(|(_, cap, top, h)| y >= *cap && y < top + h + 1).map(|(b, _, top, _)| (b, top))
    }

    fn mouse_down(&mut self, x: i32, y: i32) {
        unsafe {
            let _ = km::SetFocus(Some(self.canvas));
        }
        let Some((sel, part)) = self.hit(x, y) else {
            self.sel = Sel::Report;
            self.refresh();
            return;
        };
        self.sel = sel;
        self.before = Some(self.file.clone());
        self.drag = match (sel, part) {
            (Sel::Band(b), 2) => Some(Drag::Band { band: b, from: y, orig: self.file.bands[b].height }),
            (Sel::Item(b, i), 1) => Some(Drag::Size { band: b, item: i, from: (x, y), orig: self.file.bands[b].items[i].bounds }),
            (Sel::Item(b, i), _) => Some(Drag::Move { band: b, item: i, from: (x, y), orig: self.file.bands[b].items[i].bounds }),
            _ => None,
        };
        if self.drag.is_some() {
            unsafe {
                km::SetCapture(self.canvas);
            }
        }
        self.refresh();
    }

    fn mouse_move(&mut self, x: i32, y: i32) {
        let snap = |v: i32| (v as f32 / SNAP as f32).round() as i32 * SNAP;
        match self.drag {
            Some(Drag::Move { band, item, from, orig }) => {
                let (dx, dy) = (self.p2u(x - from.0), self.p2u(y - from.1));
                let b = &mut self.file.bands[band].items[item].bounds;
                b[0] = snap(orig[0] + dx).max(0);
                b[1] = snap(orig[1] + dy);
            }
            Some(Drag::Size { band, item, from, orig }) => {
                let (dx, dy) = (self.p2u(x - from.0), self.p2u(y - from.1));
                let b = &mut self.file.bands[band].items[item].bounds;
                b[2] = snap(orig[2] + dx).max(0);
                b[3] = snap(orig[3] + dy).max(0);
            }
            Some(Drag::Band { band, from, orig }) => {
                self.file.bands[band].height = snap(orig + self.p2u(y - from)).max(0);
                self.update_scroll();
            }
            None => {
                let cursor = match self.hit(x, y) {
                    Some((Sel::Band(_), 2)) => wm::IDC_SIZENS,
                    Some((Sel::Item(..), 1)) => wm::IDC_SIZENWSE,
                    Some((Sel::Item(..), 0)) => wm::IDC_SIZEALL,
                    _ => wm::IDC_ARROW,
                };
                unsafe {
                    if let Ok(c) = wm::LoadCursorW(None, cursor) {
                        wm::SetCursor(Some(c));
                    }
                }
                return;
            }
        }
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.canvas), None, false);
        }
    }

    fn mouse_up(&mut self, _x: i32, y: i32) {
        let Some(drag) = self.drag.take() else { return };
        unsafe {
            let _ = km::ReleaseCapture();
        }
        // Dropped on another band: it moves there, where it was let go.
        if let Drag::Move { band, item, .. } = drag {
            let it = &self.file.bands[band].items[item];
            let (top_now, _) = self.rows().into_iter().find(|r| r.0 == band).map(|r| (r.2, r.3)).unwrap_or((0, 0));
            let item_top = top_now + self.u2p(it.bounds[1]) - self.scroll;
            if let Some((to, to_top)) = self.band_at(y).filter(|(b, _)| *b != band) {
                let mut moved = self.file.bands[band].items.remove(item);
                moved.bounds[1] = self.p2u(item_top + self.scroll - to_top).max(0) / SNAP * SNAP;
                self.file.bands[to].items.push(moved);
                self.sel = Sel::Item(to, self.file.bands[to].items.len() - 1);
            } else if let Some(b) = self.file.bands.get_mut(band) {
                // Kept inside its band.
                let max_y = (b.height - b.items[item].bounds[3]).max(0);
                b.items[item].bounds[1] = b.items[item].bounds[1].clamp(0, max_y);
            }
        }
        if let Some(before) = self.before.take() {
            if before != self.file {
                self.undo.push(before);
                self.redo.clear();
                self.dirty = true;
            }
        }
        self.refresh();
    }

    fn key(&mut self, vk: u16) -> bool {
        let ctrl = unsafe { km::GetKeyState(km::VK_CONTROL.0 as i32) } < 0;
        let shift = unsafe { km::GetKeyState(km::VK_SHIFT.0 as i32) } < 0;
        match (vk, ctrl) {
            (0x2E, _) => self.delete(),
            (0x5A, true) => self.do_undo(false),
            (0x59, true) => self.do_undo(true),
            (0x53, true) => {
                self.save();
            }
            (0x1B, _) => {
                self.sel = Sel::Report;
                self.refresh();
            }
            (0x25..=0x28, _) => {
                let (dx, dy) = match vk {
                    0x25 => (-1, 0),
                    0x26 => (0, -1),
                    0x27 => (1, 0),
                    _ => (0, 1),
                };
                let step = if ctrl { SNAP * 2 } else { 1 };
                if let Sel::Item(b, i) = self.sel {
                    self.change();
                    let bounds = &mut self.file.bands[b].items[i].bounds;
                    if shift {
                        bounds[2] = (bounds[2] + dx * step).max(0);
                        bounds[3] = (bounds[3] + dy * step).max(0);
                    } else {
                        bounds[0] = (bounds[0] + dx * step).max(0);
                        bounds[1] = (bounds[1] + dy * step).max(0);
                    }
                    self.refresh();
                }
            }
            _ => return false,
        }
        true
    }

    fn update_scroll(&mut self) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.canvas, &mut r);
        }
        let total = self.total_height();
        self.scroll = self.scroll.clamp(0, (total - r.bottom).max(0));
        let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: total, nPage: r.bottom.max(0) as u32, nPos: self.scroll, ..Default::default() };
        unsafe {
            cc::SetScrollInfo(self.canvas, wm::SB_VERT, &si, true);
        }
    }

    fn paint(&self, dc: Gdi::HDC, r: &RECT) {
        let th = phai_ui::theme::current();
        let desk = th.var_color("--rule").unwrap_or(phai_ui::theme::Color::rgb(0xe5, 0xe7, 0xeb));
        let text = th.var_color("--text").unwrap_or(phai_ui::theme::Color::rgb(0x1b, 0x1b, 0x1b));
        let accent = th.var_color("--accent").unwrap_or(phai_ui::theme::Color::rgb(0x0f, 0x6c, 0xbd));
        win::fill(dc, r, desk);
        let left = self.left();
        let width = self.u2p(self.page_width());
        let dpi = self.zoom_dpi();
        let report = self.file.to_report();
        let cap_bg = phai_ui::theme::Color::rgb(0xd6, 0xdf, 0xea);
        let cap_on = phai_ui::theme::Color::rgb(0xbf, 0xd7, 0xf2);
        let caption_font = win::font(self.canvas, 0, false);
        for (b, cap, top, h) in self.rows() {
            let (cap, top) = (cap - self.scroll, top - self.scroll);
            let band = &self.file.bands[b];
            let kind = BandKind::from_word(&band.kind).unwrap_or_default();
            let chosen = self.sel == Sel::Band(b) || matches!(self.sel, Sel::Item(x, _) if x == b);
            win::fill(dc, &RECT { left, top: cap, right: left + width, bottom: top }, if chosen { cap_on } else { cap_bg });
            let mut words = t(kind.caption()).to_string();
            if matches!(kind, BandKind::GroupHeader | BandKind::GroupFooter) {
                words = format!("{words}: {}", self.file.report.groups.get(band.group).cloned().unwrap_or_default());
            }
            unsafe {
                let old = Gdi::SelectObject(dc, caption_font.into());
                let mut cr = RECT { left: left + win::px(self.canvas, 6), top: cap, right: left + width, bottom: top };
                win::draw_text(dc, self.canvas, &words, &mut cr, text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(dc, old);
            }
            win::fill(dc, &RECT { left, top, right: left + width, bottom: top + h }, phai_ui::theme::Color::rgb(0xff, 0xff, 0xff));
            // The band drawn as it prints: hundredths of an inch, from its corner.
            if let Some(rb) = report.bands.get(b) {
                let draws = report.design_band(rb, self.page_width());
                unsafe {
                    let saved = Gdi::SaveDC(dc);
                    Gdi::IntersectClipRect(dc, left, top, left + width, top + h);
                    Gdi::SetMapMode(dc, Gdi::MM_ANISOTROPIC);
                    let _ = Gdi::SetWindowExtEx(dc, 100, 100, None);
                    let _ = Gdi::SetViewportExtEx(dc, dpi, dpi, None);
                    let _ = Gdi::SetViewportOrgEx(dc, left, top, None);
                    rp::draw(dc, &draws);
                    let _ = Gdi::RestoreDC(dc, saved);
                }
            }
            // Each item's outline, faint; the chosen one's in the accent, with its size grip.
            for (i, it) in band.items.iter().enumerate() {
                let [x, y, w, ih] = it.bounds;
                let ir = RECT { left: left + self.u2p(x), top: top + self.u2p(y), right: left + self.u2p(x + w).max(self.u2p(x) + 2), bottom: top + self.u2p(y + ih).max(self.u2p(y) + 2) };
                let on = self.sel == Sel::Item(b, i);
                frame(dc, &ir, if on { accent } else { phai_ui::theme::Color::rgb(0xc4, 0xcb, 0xd4) }, !on);
                if on {
                    let g = win::px(self.canvas, 3);
                    win::fill(dc, &RECT { left: ir.right - g, top: ir.bottom - g, right: ir.right + g, bottom: ir.bottom + g }, accent);
                }
            }
            win::fill(dc, &RECT { left, top: top + h, right: left + width, bottom: top + h + 1 }, phai_ui::theme::Color::rgb(0x9c, 0xa3, 0xaf));
        }
        unsafe {
            let _ = Gdi::DeleteObject(caption_font.into());
        }
    }
}

/// A rectangle's edge, dotted when `dots`.
fn frame(dc: Gdi::HDC, r: &RECT, c: phai_ui::theme::Color, dots: bool) {
    unsafe {
        let pen = Gdi::CreatePen(if dots { Gdi::PS_DOT } else { Gdi::PS_SOLID }, 1, windows::Win32::Foundation::COLORREF(c.colorref()));
        let old = Gdi::SelectObject(dc, pen.into());
        let old_brush = Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_BRUSH));
        let _ = Gdi::Rectangle(dc, r.left, r.top, r.right, r.bottom);
        Gdi::SelectObject(dc, old_brush);
        Gdi::SelectObject(dc, old);
        let _ = Gdi::DeleteObject(pen.into());
    }
}

fn list_clear(h: HWND) {
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
    }
}

fn list_selected(h: HWND) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(h, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(cc::LVNI_SELECTED as isize))) }.0;
    (i >= 0).then_some(i as usize)
}

fn xy(l: LPARAM) -> (i32, i32) {
    ((l.0 & 0xFFFF) as i16 as i32, ((l.0 >> 16) & 0xFFFF) as i16 as i32)
}

extern "system" fn canvas_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let parent = unsafe { wm::GetParent(h) }.unwrap_or_default();
    let Some(s) = state(parent) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_ERASEBKGND => return LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let mem = Gdi::CreateCompatibleDC(Some(dc));
            let bmp = Gdi::CreateCompatibleBitmap(dc, r.right.max(1), r.bottom.max(1));
            let old = Gdi::SelectObject(mem, bmp.into());
            s.paint(mem, &r);
            let _ = Gdi::BitBlt(dc, 0, 0, r.right, r.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
            Gdi::SelectObject(mem, old);
            let _ = Gdi::DeleteObject(bmp.into());
            let _ = Gdi::DeleteDC(mem);
            let _ = Gdi::EndPaint(h, &ps);
            return LRESULT(0);
        },
        wm::WM_LBUTTONDOWN => {
            let (x, y) = xy(l);
            s.mouse_down(x, y);
            return LRESULT(0);
        }
        wm::WM_MOUSEMOVE => {
            let (x, y) = xy(l);
            s.mouse_move(x, y);
            return LRESULT(0);
        }
        wm::WM_LBUTTONUP => {
            let (x, y) = xy(l);
            s.mouse_up(x, y);
            return LRESULT(0);
        }
        wm::WM_LBUTTONDBLCLK => {
            // A label's words, a field's column: what is changed most.
            let key = match s.sel {
                Sel::Item(b, i) => match s.file.bands[b].items[i].kind.as_str() {
                    "label" => Some("text"),
                    "field" | "total" => Some("field"),
                    _ => None,
                },
                Sel::Band(_) => Some("height"),
                Sel::Report => None,
            };
            if let Some(k) = key {
                s.edit_prop(k);
            }
            return LRESULT(0);
        }
        wm::WM_KEYDOWN => {
            if s.key(w.0 as u16) {
                return LRESULT(0);
            }
        }
        // The cursor is the one the mouse's place asks for (set as it moves), not the class's.
        wm::WM_SETCURSOR if win::loword(l.0 as usize) == 1 => return LRESULT(1),
        wm::WM_GETDLGCODE => return LRESULT(0x0001 | 0x0004), // DLGC_WANTARROWS | DLGC_WANTALLKEYS
        wm::WM_SIZE => s.update_scroll(),
        wm::WM_MOUSEWHEEL => {
            let delta = ((w.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
            s.scroll -= delta * win::px(h, 40) / 120;
            s.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            return LRESULT(0);
        }
        wm::WM_VSCROLL => {
            let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
            unsafe {
                let _ = wm::GetScrollInfo(h, wm::SB_VERT, &mut si);
            }
            let line = win::px(h, 20);
            s.scroll = match wm::SCROLLBAR_COMMAND(win::loword(w.0) as i32) {
                wm::SB_LINEUP => s.scroll - line,
                wm::SB_LINEDOWN => s.scroll + line,
                wm::SB_PAGEUP => s.scroll - si.nPage as i32,
                wm::SB_PAGEDOWN => s.scroll + si.nPage as i32,
                wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
                _ => s.scroll,
            };
            s.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            return LRESULT(0);
        }
        _ => {}
    }
    win::def(h, msg, w, l)
}

fn layout(h: HWND) {
    let Some(s) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let (m, bh, gap) = (win::px(h, 8), win::px(h, 28), win::px(h, 4));
    let mut x = m;
    for (id, b) in &s.buttons {
        let w = win::px(h, if matches!(*id, ID_DATA | ID_GROUPS | ID_BANDS | ID_PRINT) { 84 } else { 70 });
        if matches!(*id, ID_DATA | ID_PREVIEW) {
            x += win::px(h, 12);
        }
        win::move_to(*b, x, m, w, bh);
        x += w + gap;
    }
    let top = m + bh + m;
    let status_h = win::px(h, 24);
    let side = win::px(h, 180);
    let props_w = win::px(h, 300);
    let bottom = r.bottom - status_h;
    win::move_to(s.fields, m, top, side, bottom - top - m);
    win::move_to(s.canvas, m + side + m, top, (r.right - side - props_w - 4 * m).max(10), bottom - top - m);
    win::move_to(s.props, r.right - props_w - m, top, props_w, bottom - top - m);
    win::move_to(s.status, m, bottom, r.right - 2 * m, status_h);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            return LRESULT(0);
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            match id {
                ID_LABEL | ID_FIELD | ID_TOTAL | ID_LINE | ID_BOX | ID_PICTURE => s.tool(id),
                ID_DATA => s.pick_data(),
                ID_GROUPS => s.groups_menu(),
                ID_BANDS => s.bands_menu(),
                ID_PREVIEW | ID_PRINT | ID_PDF => s.run(id),
                ID_SAVE => {
                    s.save();
                }
                _ => {}
            }
            return LRESULT(0);
        }
        wm::WM_NOTIFY => {
            let n = unsafe { &*(l.0 as *const cc::NMHDR) };
            let code = n.code as i32;
            if code == cc::NM_DBLCLK as i32 || code == cc::NM_RETURN as i32 {
                if n.idFrom == ID_FIELDS as usize {
                    if let Some(c) = list_selected(s.fields).and_then(|i| s.file.data.columns.get(i).cloned()) {
                        s.add_field(&c);
                    }
                } else if n.idFrom == ID_PROPS as usize {
                    if let Some(k) = list_selected(s.props).and_then(|i| s.prop_keys.get(i).copied()) {
                        s.edit_prop(k);
                    }
                }
            }
            return LRESULT(0);
        }
        wm::WM_TIMER if w.0 == TIMER_SHOT => unsafe {
            let _ = wm::KillTimer(Some(h), TIMER_SHOT);
            if let Some(p) = std::env::var_os("PHAI_SHOT") {
                let _ = phai_ui::shot::save(h, Path::new(&p));
            }
            let _ = wm::DestroyWindow(h);
            return LRESULT(0);
        },
        wm::WM_CLOSE => {
            if s.dirty && std::env::var_os("PHAI_SHOT").is_none() {
                let name = s.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                match win::message(Some(h), &tf("Save changes to {0}?", &[&name]), t("Report Designer"), wm::MB_YESNOCANCEL | wm::MB_ICONQUESTION) {
                    wm::IDYES => {
                        if !s.save() {
                            return LRESULT(0);
                        }
                    }
                    wm::IDCANCEL => return LRESULT(0),
                    _ => {}
                }
            }
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
            return LRESULT(0);
        }
        wm::WM_DESTROY => {
            if s.alone {
                unsafe { wm::PostQuitMessage(0) };
            }
        }
        wm::WM_NCDESTROY => {
            if let Some(s) = unsafe { win::take_state::<Designer>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(s.font.into());
                }
                OPEN.with(|o| o.borrow_mut().retain(|(p, _)| *p != s.path));
            }
        }
        _ => {
            if let Some(r) = win::ctl_color(msg, w, l) {
                return r;
            }
        }
    }
    win::def(h, msg, w, l)
}

thread_local! {
    /// The designers open, by their file: opening one again brings it forward.
    static OPEN: std::cell::RefCell<Vec<(PathBuf, isize)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Opens the Report Designer on `path` beside `owner` (its own window; Studio stays usable).
/// The window, or none when the file cannot be read.
pub fn open(owner: Option<HWND>, path: &Path) -> Option<HWND> {
    if let Some(h) = OPEN.with(|o| o.borrow().iter().find(|(p, _)| p == path).map(|(_, h)| HWND(*h as *mut _))) {
        unsafe {
            let _ = wm::ShowWindow(h, wm::SW_RESTORE);
            let _ = wm::SetForegroundWindow(h);
        }
        return Some(h);
    }
    let file = match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|s| ReportFile::parse(&s).map_err(|e| e.to_string())) {
        Ok(f) => f,
        Err(e) => {
            win::message(owner, &format!("{}\n\n{e}", tf("{0} cannot be opened", &[&path.display()])), t("Report Designer"), wm::MB_ICONWARNING);
            return None;
        }
    };
    win::init();
    let class = win::register("PhaiReportDesigner", Some(proc_), None);
    let canvas_class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiReportCanvas", Some(canvas_proc), None);
    let title = win::wide(t("Report Designer"));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, owner, None, Some(win::instance()), None).ok()?;
        let font = win::font(hwnd, 0, false);
        let mut buttons = Vec::new();
        for (id, label) in [
            (ID_LABEL, t("Label")),
            (ID_FIELD, t("Field")),
            (ID_TOTAL, t("Total")),
            (ID_LINE, t("Line")),
            (ID_BOX, t("Box")),
            (ID_PICTURE, t("Picture")),
            (ID_DATA, t("Data...")),
            (ID_GROUPS, t("Groups...")),
            (ID_BANDS, t("Bands...")),
            (ID_PREVIEW, t("Preview")),
            (ID_PRINT, t("Print...")),
            (ID_PDF, "PDF..."),
            (ID_SAVE, t("Save")),
        ] {
            buttons.push((id, win::button(hwnd, label, id, font, id == ID_PREVIEW)));
        }
        let fields = win::list(hwnd, ID_FIELDS, font, &[(t("Columns"), 160)]);
        let props = win::list(hwnd, ID_PROPS, font, &[(t("Property"), 120), (t("Value"), 170)]);
        let status = win::label(hwnd, "", ID_STATUS, font);
        let canvas = wm::CreateWindowExW(wm::WS_EX_CLIENTEDGE, PCWSTR(canvas_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_VSCROLL | wm::WS_TABSTOP, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_CANVAS as usize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        let alone = owner.is_none();
        win::set_state(hwnd, Box::new(Designer { path: path.to_path_buf(), file, undo: Vec::new(), redo: Vec::new(), before: None, dirty: false, sel: Sel::Report, drag: None, scroll: 0, canvas, fields, props, status, buttons, font, prop_keys: Vec::new(), alone }));
        let (w, h) = (win::px(hwnd, 1240), win::px(hwnd, 800));
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, w, h, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        match owner {
            Some(o) => win::center_on(hwnd, o),
            None => win::center_on_screen(hwnd),
        }
        win::theme_frame(hwnd);
        layout(hwnd);
        if let Some(s) = state(hwnd) {
            s.refresh();
        }
        win::show_top(hwnd);
        OPEN.with(|o| o.borrow_mut().push((path.to_path_buf(), hwnd.0 as isize)));
        Some(hwnd)
    }
}

/// `phai-studio --report file.phaireport`: the designer alone, until it closes. With `PHAI_SHOT`
/// it photographs itself and ends; `--report-preview` (with `PHAI_PREVIEW_SHOT`) shows the
/// preview first, with the rows.
pub fn run_alone(path: &Path, preview: bool) {
    let Some(h) = open(None, path) else { return };
    if preview {
        if let Some(s) = state(h) {
            s.run(ID_PREVIEW);
        }
    }
    // `PHAI_REPORT_PDF=out.pdf`: saved as a PDF too, asking nothing - checked by a test.
    if let (Some(out), Some(s)) = (std::env::var_os("PHAI_REPORT_PDF"), state(h)) {
        match s.fetch() {
            Ok(data) => println!("pdf pages: {}", s.file.to_report().export_pdf(h, data, Path::new(&out))),
            Err(e) => println!("pdf: {e}"),
        }
    }
    if std::env::var_os("PHAI_SHOT").is_some() {
        win::timer_start(h, TIMER_SHOT as i32, 500);
    }
    let mut msg = wm::MSG::default();
    unsafe {
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !wm::IsDialogMessageW(h, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
    }
}

/// Saves the report and its code: `<name>.phaireport`, `<name>.rs` beside it (written whole), and
/// in a project `mod <name>;` in main.rs; bound to a table, the project told where the data is
/// (`src/data_sources.rs`, phai-data in Cargo.toml) as a bound form's grid does it.
pub fn save_files(path: &Path, f: &ReportFile, conn: Option<&phai_data::Connection>) -> std::io::Result<()> {
    std::fs::write(path, f.to_toml())?;
    let Some(src) = path.parent() else { return Ok(()) };
    let code = src.join(format!("{}.rs", f.module()));
    if let Ok(old) = std::fs::read_to_string(&code) {
        if !old.starts_with("//! Generated by Phai Studio from") {
            return Err(std::io::Error::other(tf("{0} is a file of yours, not the report's code - give the report another name", &[&code.display()])));
        }
    }
    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    std::fs::write(&code, generate(f, &file_name))?;
    let main_rs = src.join("main.rs");
    let Ok(text) = std::fs::read_to_string(&main_rs) else { return Ok(()) };
    let mut out = declare_report(&text, &f.report.name, &f.module());
    if f.data.is_bound() {
        let sources = src.join("data_sources.rs");
        if !sources.exists() {
            if let Some(c) = conn {
                std::fs::write(&sources, crate::project::data_sources_rs(c))?;
            }
        }
        if sources.exists() {
            out = declare_mod(&out, "mod data_sources;", "");
        }
        if let Some(dir) = src.parent() {
            let cargo_toml = dir.join("Cargo.toml");
            if let Ok(t) = std::fs::read_to_string(&cargo_toml) {
                let with = crate::project::with_phai_data(&t);
                if with != t {
                    std::fs::write(&cargo_toml, with)?;
                }
            }
        }
    }
    if out != text {
        std::fs::write(&main_rs, out)?;
    }
    Ok(())
}

/// main.rs with `line` after its last `mod ...;`, `comment` above it - unchanged when it is there.
fn declare_mod(main_rs: &str, line: &str, comment: &str) -> String {
    if main_rs.lines().any(|l| l.trim() == line) {
        return main_rs.to_string();
    }
    let lines: Vec<&str> = main_rs.lines().collect();
    let at = lines.iter().rposition(|l| l.trim_start().starts_with("mod ") && l.trim_end().ends_with(';')).map_or(0, |i| i + 1);
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let mut block: Vec<String> = comment.lines().map(str::to_string).collect();
    block.push(line.to_string());
    for (k, b) in block.into_iter().enumerate() {
        out.insert(at + k, b);
    }
    out.join("\n") + "\n"
}

/// main.rs with the report's module declared, saying how to show it.
pub fn declare_report(main_rs: &str, name: &str, module: &str) -> String {
    let comment = format!("// {name}: show it with {module}::preview(ui.hwnd), print it with {module}::print(ui.hwnd).\n#[allow(dead_code)]");
    declare_mod(main_rs, &format!("mod {module};"), &comment)
}

/// Project > Add Report: a new report in the project's `src`, saved with its code and opened to
/// design. Its path.
pub fn add_report(owner: HWND, project: &Path) -> Option<PathBuf> {
    let caption = t("Add Report");
    let src = project.join("src");
    let mut n = 1;
    while src.join(format!("{}.phaireport", phai_form::snake(&format!("Report{n}")))).exists() {
        n += 1;
    }
    let default = format!("Report{n}");
    let answer = phai_ui::dialogs::ask_fields(Some(owner), caption, &[phai_ui::dialogs::Field::Text(t("Name"), &default)])?;
    let name = crate::form_type_name(&answer[0]);
    if name.is_empty() {
        return None;
    }
    let f = ReportFile::new(&name);
    let path = src.join(format!("{}.phaireport", f.module()));
    if path.exists() || src.join(format!("{}.rs", f.module())).exists() {
        win::message(Some(owner), &tf("The project already has {0}", &[&f.module()]), caption, wm::MB_ICONINFORMATION);
        return None;
    }
    if let Err(e) = save_files(&path, &f, None) {
        win::message(Some(owner), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&path.display()])), caption, wm::MB_ICONWARNING);
        return None;
    }
    open(Some(owner), &path);
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_is_declared_once_after_the_other_modules() {
        let main = "mod main_window;\nmod main_window_ui;\nmod text;\n\nfn main() {}\n";
        let out = declare_report(main, "SalesReport", "sales_report");
        assert!(out.contains("mod text;\n// SalesReport: show it with sales_report::preview(ui.hwnd)"), "{out}");
        assert!(out.contains("#[allow(dead_code)]\nmod sales_report;\n\nfn main()"), "{out}");
        assert_eq!(declare_report(&out, "SalesReport", "sales_report"), out);
        assert_eq!(caption_of("amount_due"), "Amount due");
        let c = phai_data::Connection { kind: phai_data::Kind::Sqlite, host: r"C:\My shop\data.db".into(), ..Default::default() };
        assert_eq!(sqlite_of(&crate::project::data_sources_rs(&c)), Some(c), "read back as written");
    }

    #[test]
    fn the_bands_show_in_the_designers_order() {
        let mut f = ReportFile::new("R");
        f.report.groups = vec!["a".into(), "b".into()];
        for (k, g) in [("group_footer", 0), ("group_header", 1), ("group_footer", 1), ("group_header", 0), ("report_footer", 0)] {
            f.bands.push(BandSpec { kind: k.into(), group: g, height: 10, ..Default::default() });
        }
        let kinds: Vec<String> = display_order(&f).into_iter().map(|i| format!("{}{}", f.bands[i].kind, f.bands[i].group)).collect();
        assert_eq!(kinds, ["report_header0", "page_header0", "group_header0", "group_header1", "detail0", "group_footer1", "group_footer0", "report_footer0", "page_footer0"]);
    }

    /// A report saved into a real project with a bound table builds with Cargo, and its code
    /// reads the rows and lays them out.
    #[test]
    #[ignore = "builds a project with cargo"]
    fn a_saved_report_builds_and_prints_its_rows() {
        let dir = std::env::temp_dir().join(format!("phai-report-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).expect("project");
        let db = dir.join("shop.db");
        let c = phai_data::Connection { host: db.to_string_lossy().to_string(), kind: phai_data::Kind::Sqlite, ..Default::default() };
        c.query(None, "CREATE TABLE orders (id INTEGER PRIMARY KEY, region TEXT, customer TEXT, amount REAL); INSERT INTO orders (region, customer, amount) VALUES ('North', 'Somchai', 100), ('South', 'Monrak', 250.5), ('North', 'Kirati', 50);").unwrap();
        let mut f = ReportFile::new("SalesReport");
        f.report.groups = vec!["region".into()];
        f.data = DataSpec { database: "main".into(), table: "orders".into(), columns: vec!["region".into(), "customer".into(), "amount".into()], order: vec!["customer".into()], ..Default::default() };
        let field = |name: &str, x| ItemSpec { kind: "field".into(), name: name.into(), field: name.into(), bounds: [x, 0, 200, 18], ..Default::default() };
        f.bands.iter_mut().find(|b| b.kind == "detail").unwrap().items = vec![field("customer", 0), field("amount", 300)];
        f.bands.push(BandSpec { kind: "group_header".into(), height: 24, items: vec![field("region", 0)], ..Default::default() });
        f.bands.push(BandSpec { kind: "group_footer".into(), height: 24, items: vec![ItemSpec { kind: "total".into(), name: "sum".into(), field: "amount".into(), format: "0.00".into(), bounds: [300, 0, 200, 18], ..Default::default() }], ..Default::default() });
        let path = dir.join("src").join("sales_report.phaireport");
        save_files(&path, &f, Some(&c)).unwrap();
        // The program lays the report out with its rows and says what is on the page.
        let main_rs = dir.join("src").join("main.rs");
        let text = std::fs::read_to_string(&main_rs).unwrap();
        assert!(text.contains("mod sales_report;") && text.contains("mod data_sources;"), "{text}");
        let body = "fn main() {\n    let data = sales_report::data().unwrap();\n    let pages = sales_report::report().layout(&data, phai_ui::print::Bounds { x: 0, y: 0, width: 750, height: 1000 }, &phai_ui::report::Now::default());\n    println!(\"{}\", pages[0].texts().join(\"|\"));\n}\n";
        let at = text.find("fn main()").unwrap();
        std::fs::write(&main_rs, format!("{}{}\n#[allow(dead_code)]\nfn old_main() {}", &text[..at], body, &text[at + "fn main()".len()..])).unwrap();
        let out = std::process::Command::new("cargo").args(["run", "--quiet"]).current_dir(&dir).output().unwrap();
        let printed = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{}\n{}", printed, String::from_utf8_lossy(&out.stderr));
        assert!(printed.contains("North|Kirati|50|Somchai|100|150.00|South|Monrak|250.5|250.50"), "{printed}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
