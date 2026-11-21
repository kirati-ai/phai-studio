//! The Table Designer, as Visual Studio's: a grid of the table's columns - name, data type, allow
//! nulls, primary key, identity, default - edited in place with a click; its indexes and foreign
//! keys below; and the script that makes the design so, written as you go. Update runs it.

use phai_data::schema::{ColDef, FkDef, IndexDef, TableDef, RULES};
use phai_data::Connection;
use phai_ui::dialogs::{self, Field};
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_NAME: i32 = 10;
const ID_COLS: i32 = 11;
const ID_IDX: i32 = 12;
const ID_FK: i32 = 13;
const ID_SCRIPT: i32 = 14;
const ID_EDITOR: i32 = 15;
const ID_ADD_COL: i32 = 20;
const ID_DEL_COL: i32 = 21;
const ID_UP: i32 = 22;
const ID_DOWN: i32 = 23;
const ID_ADD_IDX: i32 = 24;
const ID_EDIT_IDX: i32 = 25;
const ID_DEL_IDX: i32 = 26;
const ID_ADD_FK: i32 = 27;
const ID_EDIT_FK: i32 = 28;
const ID_DEL_FK: i32 = 29;
/// The database's tables at the left, as phpMyAdmin lists them: one chosen is designed.
const ID_TABLES: i32 = 30;
const ID_UPDATE: i32 = 1;
const ID_CLOSE: i32 = 2;

struct Designer {
    tables: HWND,
    /// The tables listed, after "(new table)".
    names: Vec<String>,
    conn: Connection,
    db: String,
    old: Option<TableDef>,
    def: TableDef,
    name: HWND,
    cols: HWND,
    idx: HWND,
    fk: HWND,
    script: HWND,
    labels: Vec<HWND>,
    buttons: Vec<(i32, HWND)>,
    /// The box editing a cell: it, and the row and column.
    editor: Option<(HWND, usize, usize)>,
    done: bool,
    applied: bool,
    font: Gdi::HFONT,
    mono: Gdi::HFONT,
}

fn state<'a>(h: HWND) -> Option<&'a mut Designer> {
    unsafe { win::state::<Designer>(h) }
}

fn check(b: bool) -> &'static str {
    if b { "✓" } else { "" }
}

/// Opens the designer for `existing` (or a new table in `db`) and runs it until closed; whether
/// a change was made in the database.
pub fn open(owner: HWND, conn: Connection, db: &str, existing: Option<TableDef>) -> bool {
    win::init();
    let class = win::register("PhaiTableDesigner", Some(proc_), None);
    let title = match &existing {
        Some(d) => tf("Table Designer - {0}", &[&d.name]),
        None => t("Table Designer - new table").to_string(),
    };
    let title_w = win::wide(&title);
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title_w.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_MAXIMIZEBOX | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let mono = {
            let mut lf = Gdi::LOGFONTW { lfHeight: -win::px(hwnd, 13), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
            for (i, u) in "Cascadia Mono".encode_utf16().enumerate() {
                lf.lfFaceName[i] = u;
            }
            Gdi::CreateFontIndirectW(&lf)
        };
        let name_text = existing.as_ref().map(|d| d.name.clone()).unwrap_or_else(|| "new_table".into());
        let mut labels = Vec::new();
        for l in [t("Table name"), t("Columns"), t("Indexes"), t("Foreign Keys"), t("Script")] {
            labels.push(win::label(hwnd, l, 0, font));
        }
        let name = win::edit(hwnd, &name_text, ID_NAME, font, false);
        // The database's tables, the one designed chosen.
        let tables = win::list(hwnd, ID_TABLES, font, &[(&tf("Tables of {0}", &[&db]), 190)]);
        let names: Vec<String> = conn.tables(db).map(|v| v.into_iter().filter(|(_, view)| !view).map(|(n, _)| n).collect()).unwrap_or_default();
        win::list_add(tables, &[t("(new table)")]);
        for n in &names {
            win::list_add(tables, &[n]);
        }
        let at = existing.as_ref().and_then(|d| names.iter().position(|n| *n == d.name)).map(|i| i + 1).unwrap_or(0);
        win::list_select(tables, at);
        let cols = win::list(hwnd, ID_COLS, font, &[(t("Column Name"), 170), (t("Data Type"), 150), (t("Allow Nulls"), 80), (t("Primary Key"), 85), (t("Identity"), 70), (t("Default"), 140)]);
        let idx = win::list(hwnd, ID_IDX, font, &[(t("Name"), 200), (t("Columns"), 260), (t("Unique"), 70)]);
        let fk = win::list(hwnd, ID_FK, font, &[(t("Name"), 160), (t("Columns"), 140), (t("References"), 220), (t("On Delete"), 100), (t("On Update"), 100)]);
        for l in [cols, idx, fk] {
            win::grid_lines(l);
        }
        let script = win::edit_lines(hwnd, "", ID_SCRIPT, mono);
        wm::SendMessageW(script, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        let mut buttons = Vec::new();
        for (id, label) in [
            (ID_ADD_COL, t("Add Column")),
            (ID_DEL_COL, t("Delete Column")),
            (ID_UP, t("Move Up")),
            (ID_DOWN, t("Move Down")),
            (ID_ADD_IDX, t("Add Index...")),
            (ID_EDIT_IDX, t("Edit...")),
            (ID_DEL_IDX, t("Delete")),
            (ID_ADD_FK, t("Add Key...")),
            (ID_EDIT_FK, t("Edit...")),
            (ID_DEL_FK, t("Delete")),
            (ID_UPDATE, t("Update")),
            (ID_CLOSE, t("Close")),
        ] {
            buttons.push((id, win::button(hwnd, label, id, font, id == ID_UPDATE)));
        }
        let def = existing.clone().unwrap_or_else(|| TableDef {
            name: name_text.clone(),
            columns: vec![ColDef { name: "id".into(), ty: phai_data::schema::types_for(conn.kind)[0].to_string(), nullable: false, pk: true, auto: true, ..Default::default() }],
            ..Default::default()
        });
        win::set_state(hwnd, Box::new(Designer { tables, names, conn, db: db.to_string(), old: existing, def, name, cols, idx, fk, script, labels, buttons, editor: None, done: false, applied: false, font, mono }));
        refresh(hwnd);
        let (w, h) = (win::px(hwnd, 900), win::px(hwnd, 720));
        // In the middle of the screen (its owner may be a narrow pane at the side).
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, w, h, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on_screen(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|s| !s.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let applied = state(hwnd).is_some_and(|s| s.applied);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        applied
    }
}

/// A table chosen at the left (0: a new one): its design loaded - what was changed and not yet
/// updated asked about first.
fn choose_table(h: HWND, k: usize) {
    let Some(s) = state(h) else { return };
    let now = s.old.as_ref().map(|d| d.name.clone());
    let wanted = if k == 0 { None } else { s.names.get(k - 1).cloned() };
    if now == wanted && (wanted.is_some() || s.old.is_none()) {
        return;
    }
    if !s.conn.design_script(&s.db, s.old.as_ref(), &s.def).is_empty() && s.old.is_some() {
        let ask = win::message(Some(h), t("The changes to this table are not updated yet. Leave them?"), t("Table Designer"), wm::MB_YESNO | wm::MB_ICONQUESTION);
        if ask != wm::IDYES {
            let back = now.and_then(|n| s.names.iter().position(|x| *x == n)).map(|i| i + 1).unwrap_or(0);
            win::list_select(s.tables, back);
            return;
        }
    }
    let def = match &wanted {
        Some(name) => match s.conn.table_def(&s.db, name) {
            Ok(d) => Some(d),
            Err(e) => {
                win::message(Some(h), &e, t("Table Designer"), wm::MB_ICONWARNING);
                return;
            }
        },
        None => None,
    };
    s.old = def.clone();
    s.def = def.unwrap_or_else(|| TableDef {
        name: "new_table".into(),
        columns: vec![ColDef { name: "id".into(), ty: phai_data::schema::types_for(s.conn.kind)[0].to_string(), nullable: false, pk: true, auto: true, ..Default::default() }],
        ..Default::default()
    });
    win::set_text(s.name, &s.def.name);
    let title = match &s.old {
        Some(d) => tf("Table Designer - {0}", &[&d.name]),
        None => t("Table Designer - new table").to_string(),
    };
    win::set_text(h, &title);
    refresh(h);
}

/// The lists and the script again, from the design.
fn refresh(h: HWND) {
    let Some(s) = state(h) else { return };
    let selected = |l: HWND| unsafe { wm::SendMessageW(l, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2 /* LVNI_SELECTED */))).0 };
    let (cs, is, fs) = (selected(s.cols), selected(s.idx), selected(s.fk));
    for l in [s.cols, s.idx, s.fk] {
        unsafe {
            wm::SendMessageW(l, cc::LVM_DELETEALLITEMS, None, None);
        }
    }
    for c in &s.def.columns {
        win::list_add(s.cols, &[&c.name, &c.ty, check(c.nullable), check(c.pk), check(c.auto), &c.default]);
    }
    for i in &s.def.indexes {
        win::list_add(s.idx, &[&i.name, &i.columns.join(", "), check(i.unique)]);
    }
    for f in &s.def.fkeys {
        win::list_add(s.fk, &[&f.name, &f.columns.join(", "), &format!("{} ({})", f.ref_table, f.ref_columns.join(", ")), &f.on_delete, &f.on_update]);
    }
    for (l, i) in [(s.cols, cs), (s.idx, is), (s.fk, fs)] {
        if i >= 0 {
            select_row(l, i as usize);
        }
    }
    let script = s.conn.design_script(&s.db, s.old.as_ref(), &s.def);
    let text = if script.is_empty() { format!("-- {}", t("No changes")) } else { script.iter().map(|x| format!("{x};")).collect::<Vec<_>>().join("\r\n\r\n").replace('\n', "\r\n").replace("\r\r\n", "\r\n") };
    win::set_text(s.script, &text);
}

fn select_row(l: HWND, i: usize) {
    let item = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(3), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(3), ..Default::default() };
    unsafe {
        wm::SendMessageW(l, 0x102B /* LVM_SETITEMSTATE */, Some(WPARAM(i)), Some(LPARAM(&item as *const _ as isize)));
    }
}

fn selected_row(l: HWND) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(l, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then_some(i as usize)
}

fn layout(h: HWND) {
    let Some(s) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap, bw) = (px(11), px(23), px(6), px(110));
    // The tables down the left; the design beside them.
    let side = px(210);
    win::move_to(s.tables, m, m, side, r.bottom - 2 * m);
    let x0 = m + side + gap * 2;
    let w = r.right - x0 - m;
    let list_w = w - bw - gap;
    let mut y = m;
    win::move_to(s.labels[0], x0, y, px(100), row);
    win::move_to(s.name, x0 + px(105), y, px(300), row);
    y += row + gap * 2;
    let rest = r.bottom - y - row - m * 2;
    let cols_h = rest * 36 / 100;
    let small_h = rest * 15 / 100;
    let script_h = rest - cols_h - small_h * 2 - (row + gap) * 4 + row;
    let button = |id: i32| s.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    // Columns, with Add / Delete / Up / Down at the right.
    win::move_to(s.labels[1], x0, y, list_w, row);
    y += row;
    win::move_to(s.cols, x0, y, list_w, cols_h);
    for (k, id) in [ID_ADD_COL, ID_DEL_COL, ID_UP, ID_DOWN].iter().enumerate() {
        win::move_to(button(*id), x0 + list_w + gap, y + k as i32 * (row + gap), bw, row);
    }
    y += cols_h + gap;
    for (label, list, ids) in [(s.labels[2], s.idx, [ID_ADD_IDX, ID_EDIT_IDX, ID_DEL_IDX]), (s.labels[3], s.fk, [ID_ADD_FK, ID_EDIT_FK, ID_DEL_FK])] {
        win::move_to(label, x0, y, list_w, row);
        y += row;
        win::move_to(list, x0, y, list_w, small_h);
        for (k, id) in ids.iter().enumerate() {
            win::move_to(button(*id), x0 + list_w + gap, y + k as i32 * (row + gap), bw, row);
        }
        y += small_h + gap;
    }
    win::move_to(s.labels[4], x0, y, w, row);
    y += row;
    let by = r.bottom - m - row;
    // The script takes what is left above the buttons.
    let _ = script_h;
    win::move_to(s.script, x0, y, w, (by - gap * 2 - y).max(px(60)));
    win::move_to(button(ID_CLOSE), r.right - m - px(75), by, px(75), row);
    win::move_to(button(ID_UPDATE), r.right - m - px(75) * 2 - px(7), by, px(75), row);
}

/// A click in the columns' grid: a check toggled, or a box over the cell to type in.
fn cell_click(h: HWND, row: i32, col: i32) {
    let Some(s) = state(h) else { return };
    close_editor(h, true);
    let Some(s) = state(h).filter(|_| row >= 0 && (row as usize) < s.def.columns.len()) else { return };
    let (row, col) = (row as usize, col.max(0) as usize);
    let c = &mut s.def.columns[row];
    match col {
        2 => c.nullable = !c.nullable && !c.pk,
        3 => {
            c.pk = !c.pk;
            if c.pk {
                c.nullable = false;
            }
        }
        4 => c.auto = !c.auto,
        _ => {
            open_editor(h, row, col);
            return;
        }
    }
    refresh(h);
}

fn open_editor(h: HWND, row: usize, col: usize) {
    let Some(s) = state(h) else { return };
    // The cell, in this window's pixels.
    let mut r = RECT { top: col as i32, left: if col == 0 { 2 } else { 0 }, ..Default::default() };
    unsafe {
        wm::SendMessageW(s.cols, 0x1038 /* LVM_GETSUBITEMRECT */, Some(WPARAM(row)), Some(LPARAM(&mut r as *mut _ as isize)));
        let mut pts = [POINT { x: r.left, y: r.top }, POINT { x: r.right, y: r.bottom }];
        Gdi::MapWindowPoints(Some(s.cols), Some(h), &mut pts);
        r = RECT { left: pts[0].x, top: pts[0].y, right: pts[1].x, bottom: pts[1].y };
    }
    if col == 0 {
        // The first column's rectangle is the label's; its full width is the column's.
        let w = unsafe { wm::SendMessageW(s.cols, 0x101D /* LVM_GETCOLUMNWIDTH */, Some(WPARAM(0)), None).0 } as i32;
        r.right = r.left + w - 2;
    }
    let c = &s.def.columns[row];
    let value = match col {
        0 => c.name.clone(),
        1 => c.ty.clone(),
        _ => c.default.clone(),
    };
    let e = if col == 1 {
        let types: Vec<&str> = phai_data::schema::types_for(s.conn.kind).to_vec();
        // Made able to be typed in from the start (a style changed later never gets its typing
        // box, so the column's own type - VARCHAR(64) - could not be shown, and the list's first,
        // INT, was taken). The type the column has is the one shown and chosen.
        let combo = win::combo_with(h, ID_EDITOR, s.font, &types, true);
        win::move_to(combo, r.left, r.top, r.right - r.left, win::px(h, 300));
        unsafe {
            let w = win::wide(&value);
            let at = wm::SendMessageW(combo, 0x0158 /* CB_FINDSTRINGEXACT */, Some(WPARAM(usize::MAX)), Some(LPARAM(w.as_ptr() as isize))).0;
            wm::SendMessageW(combo, 0x014E /* CB_SETCURSEL */, Some(WPARAM(at as usize)), None);
        }
        win::set_text(combo, &value);
        combo
    } else {
        let e = win::edit(h, &value, ID_EDITOR, s.font, false);
        win::move_to(e, r.left, r.top, r.right - r.left, r.bottom - r.top);
        e
    };
    unsafe {
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(e, Some(editor_keys), 21, 0);
        // A drop-down's typing box is a child of its own: it is the one that hears the keys.
        if let Ok(inner) = wm::GetWindow(e, wm::GW_CHILD) {
            let _ = windows::Win32::UI::Shell::SetWindowSubclass(inner, Some(editor_keys), 21, 0);
        }
        wm::SetWindowPos(e, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE).ok();
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(e));
        wm::SendMessageW(e, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
    }
    s.editor = Some((e, row, col));
}

/// The in-place box closed; `keep` puts what it holds into the design.
fn close_editor(h: HWND, keep: bool) {
    let Some(s) = state(h) else { return };
    let Some((e, row, col)) = s.editor.take() else { return };
    let text = win::text_of(e).trim().to_string();
    unsafe {
        let _ = wm::DestroyWindow(e);
    }
    if keep {
        if let Some(c) = s.def.columns.get_mut(row) {
            match col {
                0 if !text.is_empty() => c.name = text,
                1 if !text.is_empty() => c.ty = text,
                5 => c.default = text,
                _ => {}
            }
        }
        refresh(h);
    }
}

unsafe extern "system" fn editor_keys(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    match msg {
        wm::WM_GETDLGCODE => LRESULT(0x0004),
        wm::WM_KEYDOWN if w.0 == 0x0D || w.0 == 0x1B => {
            let mut top = h;
            // Up to the designer through a drop-down.
            while let Ok(p) = unsafe { wm::GetParent(top) } {
                if state(p).is_some() {
                    close_editor(p, w.0 == 0x0D);
                    break;
                }
                top = p;
            }
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

fn names(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn edit_index(h: HWND, at: Option<usize>) {
    let Some(s) = state(h) else { return };
    let start = at.and_then(|i| s.def.indexes.get(i).cloned()).unwrap_or_else(|| {
        let first = s.def.columns.iter().find(|c| !c.pk).map(|c| c.name.clone()).unwrap_or_default();
        IndexDef { name: format!("ix_{}_{}", s.def.name.rsplit('.').next().unwrap_or(&s.def.name), first), columns: vec![first], unique: false }
    });
    let cols = start.columns.join(", ");
    let fields = [Field::Text(t("Name"), &start.name), Field::Text(t("Columns (comma between)"), &cols), Field::Check(t("Unique"), start.unique)];
    let Some(a) = dialogs::ask_fields(Some(h), t("Index"), &fields) else { return };
    let new = IndexDef { name: a[0].trim().into(), columns: names(&a[1]), unique: a[2] == "1" };
    if new.name.is_empty() || new.columns.is_empty() {
        return;
    }
    match at {
        Some(i) => s.def.indexes[i] = new,
        None => s.def.indexes.push(new),
    }
    refresh(h);
}

fn edit_fk(h: HWND, at: Option<usize>) {
    let Some(s) = state(h) else { return };
    let start = at.and_then(|i| s.def.fkeys.get(i).cloned()).unwrap_or_else(|| FkDef { name: format!("fk_{}", s.def.name.rsplit('.').next().unwrap_or(&s.def.name)), on_delete: "NO ACTION".into(), on_update: "NO ACTION".into(), ..Default::default() });
    let (cols, refs) = (start.columns.join(", "), start.ref_columns.join(", "));
    let rules = RULES.join(" / ");
    let delete_label = tf("On delete ({0})", &[&rules]);
    let update_label = tf("On update ({0})", &[&rules]);
    let fields = [
        Field::Text(t("Name"), &start.name),
        Field::Text(t("Columns (comma between)"), &cols),
        Field::Text(t("References table"), &start.ref_table),
        Field::Text(t("Its columns"), &refs),
        Field::Text(&delete_label, &start.on_delete),
        Field::Text(&update_label, &start.on_update),
    ];
    let Some(a) = dialogs::ask_fields(Some(h), t("Foreign Key"), &fields) else { return };
    let rule = |x: &str| {
        let up = x.trim().to_uppercase();
        if RULES.contains(&up.as_str()) { up } else { "NO ACTION".into() }
    };
    let new = FkDef { name: a[0].trim().into(), columns: names(&a[1]), ref_table: a[2].trim().into(), ref_columns: names(&a[3]), on_delete: rule(&a[4]), on_update: rule(&a[5]) };
    if new.name.is_empty() || new.columns.is_empty() || new.ref_table.is_empty() || new.columns.len() != new.ref_columns.len() {
        win::message(Some(h), t("A foreign key needs a name, its columns, the table it refers to and as many of that table's columns."), t("Foreign Key"), wm::MB_ICONINFORMATION);
        return;
    }
    match at {
        Some(i) => s.def.fkeys[i] = new,
        None => s.def.fkeys.push(new),
    }
    refresh(h);
}

/// Update: the script run against the database, waiting with a Cancel.
fn update(h: HWND) {
    close_editor(h, true);
    let Some(s) = state(h) else { return };
    if s.def.name.trim().is_empty() || s.def.columns.is_empty() {
        win::message(Some(h), t("A table needs a name and at least one column."), t("Table Designer"), wm::MB_ICONINFORMATION);
        return;
    }
    let script = s.conn.design_script(&s.db, s.old.as_ref(), &s.def);
    if script.is_empty() {
        s.done = true;
        return;
    }
    let (c, db) = (s.conn.clone(), s.db.clone());
    let sql = script.join(";\n") + ";";
    match dialogs::wait_for(Some(h), t("Table Designer"), t("Saving..."), move |_| c.query(Some(&db), &sql)) {
        Some(Ok(_)) => {
            s.applied = true;
            s.done = true;
        }
        Some(Err(e)) => {
            win::message(Some(h), &e, t("Table Designer"), wm::MB_ICONWARNING);
        }
        None => {}
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = POINT { x: win::px(h, 700), y: win::px(h, 560) };
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.tables && hdr.code == cc::NM_CLICK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    choose_table(h, it.iItem as usize);
                }
                return LRESULT(0);
            }
            if hdr.hwndFrom == s.cols && hdr.code == cc::NM_CLICK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                cell_click(h, it.iItem, it.iSubItem);
                return LRESULT(0);
            }
            if hdr.code == cc::NM_DBLCLK && (hdr.hwndFrom == s.idx || hdr.hwndFrom == s.fk) {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    if hdr.hwndFrom == s.idx {
                        edit_index(h, Some(it.iItem as usize));
                    } else {
                        edit_fk(h, Some(it.iItem as usize));
                    }
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            let code = win::hiword(w.0);
            match id {
                ID_NAME if code == 0x0300 /* EN_CHANGE */ => {
                    s.def.name = win::text_of(s.name).trim().to_string();
                    refresh(h);
                }
                ID_EDITOR if code == 0x0200 /* EN_KILLFOCUS */ || code == 10 /* CBN_KILLFOCUS */ => close_editor(h, true),
                ID_ADD_COL => {
                    let n = s.def.columns.len() + 1;
                    let ty = match s.conn.kind {
                        phai_data::Kind::MySql => "VARCHAR(100)",
                        phai_data::Kind::Postgres => "varchar(100)",
                        phai_data::Kind::Sqlite => "TEXT",
                    };
                    s.def.columns.push(ColDef { name: format!("column{n}"), ty: ty.into(), nullable: true, ..Default::default() });
                    let last = s.def.columns.len() - 1;
                    refresh(h);
                    select_row(s.cols, last);
                    open_editor(h, last, 0);
                }
                ID_DEL_COL => {
                    if let Some(i) = selected_row(s.cols) {
                        s.def.columns.remove(i);
                        refresh(h);
                    }
                }
                ID_UP | ID_DOWN => {
                    if let Some(i) = selected_row(s.cols) {
                        let j = if id == ID_UP { i.checked_sub(1) } else { (i + 1 < s.def.columns.len()).then_some(i + 1) };
                        if let Some(j) = j {
                            s.def.columns.swap(i, j);
                            refresh(h);
                            select_row(s.cols, j);
                        }
                    }
                }
                ID_ADD_IDX => edit_index(h, None),
                ID_EDIT_IDX => edit_index(h, selected_row(s.idx)),
                ID_DEL_IDX => {
                    if let Some(i) = selected_row(s.idx) {
                        s.def.indexes.remove(i);
                        refresh(h);
                    }
                }
                ID_ADD_FK => edit_fk(h, None),
                ID_EDIT_FK => edit_fk(h, selected_row(s.fk)),
                ID_DEL_FK => {
                    if let Some(i) = selected_row(s.fk) {
                        s.def.fkeys.remove(i);
                        refresh(h);
                    }
                }
                ID_UPDATE => update(h),
                ID_CLOSE => s.done = true,
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(s) = unsafe { win::take_state::<Designer>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(s.font.into());
                    let _ = Gdi::DeleteObject(s.mono.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
