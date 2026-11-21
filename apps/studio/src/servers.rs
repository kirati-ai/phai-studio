//! Server Explorer, as Visual Studio's: Data Connections - each a MySQL server - and under each its
//! databases, their tables and views, and the columns of each. Branches are read from the server
//! when first opened. Right-click for what a node can do: add a connection, a new query, the rows
//! of a table, code for a program to read it. The connections are remembered, a password only
//! encrypted by Windows.

use std::path::PathBuf;

use phai_data::{Column, Connection};
use phai_ui::dialogs;
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::studio;

pub const ID_SERVERS_TREE: i32 = 420;

/// Icons by index: the connections' root, a server, a database, a table, a view, a column.
pub const ICONS: [u16; 6] = [0xE8F1, 0xE7F4, 0xE8B7, 0xE8A9, 0xE890, 0xE8A5];

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Root,
    Server,
    Database,
    Table { view: bool },
    Column,
    /// Stands in for children not read yet.
    Waiting,
}

#[derive(Clone, Debug)]
struct Node {
    kind: Kind,
    conn: usize,
    db: String,
    table: String,
    loaded: bool,
}

pub struct Servers {
    pub tree: HWND,
    pub conns: Vec<Connection>,
    nodes: Vec<Node>,
}

/// The Table Designer, in its own window over Studio's.
fn design(c: Connection, db: &str, def: Option<phai_data::schema::TableDef>) {
    let owner = studio().map(|s| s.hwnd).unwrap_or_default();
    crate::designer::open(owner, c, db, def);
}

fn file() -> Option<PathBuf> {
    Some(crate::settings_dir_pub()?.join("connections.txt"))
}

impl Servers {
    pub fn new(parent: HWND, font: windows::Win32::Graphics::Gdi::HFONT) -> Servers {
        let tree = win::tree(parent, ID_SERVERS_TREE, font);
        win::tree_icons(tree, &ICONS);
        let conns = file().and_then(|f| std::fs::read_to_string(f).ok()).map(|t| phai_data::from_lines(&t)).unwrap_or_default();
        let mut s = Servers { tree, conns, nodes: vec![] };
        s.fill();
        s
    }

    fn save(&self) {
        if let Some(f) = file() {
            if let Some(d) = f.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let _ = std::fs::write(f, phai_data::to_lines(&self.conns));
        }
    }

    fn add_node(&mut self, parent: isize, text: &str, icon: i32, node: Node) -> isize {
        let expandable = matches!(node.kind, Kind::Root | Kind::Server | Kind::Database | Kind::Table { .. });
        self.nodes.push(node.clone());
        let item = win::tree_add(self.tree, parent, text, icon, self.nodes.len() as isize - 1);
        if expandable && node.kind != Kind::Root {
            // A child to show the [+] until the branch is read.
            self.nodes.push(Node { kind: Kind::Waiting, ..node });
            win::tree_add(self.tree, item, "…", icon, self.nodes.len() as isize - 1);
        }
        item
    }

    /// Everything again: the root, and a node for each connection.
    pub fn fill(&mut self) {
        win::tree_clear(self.tree);
        self.nodes.clear();
        let root = self.add_node(0, t("Data Connections"), 0, Node { kind: Kind::Root, conn: 0, db: String::new(), table: String::new(), loaded: true });
        for i in 0..self.conns.len() {
            let label = self.conns[i].label();
            self.add_node(root, &label, 1, Node { kind: Kind::Server, conn: i, db: String::new(), table: String::new(), loaded: false });
        }
        win::tree_expand(self.tree, root);
    }

    fn node_of(&self, item_data: isize) -> Option<Node> {
        self.nodes.get(usize::try_from(item_data).ok()?).cloned()
    }

    fn selected(&self) -> Option<(isize, Node)> {
        let item = tree_caret(self.tree)?;
        let data = item_data(self.tree, item)?;
        Some((item, self.node_of(data)?))
    }

    /// A branch opened: read what is under it, the first time.
    pub fn expanding(&mut self, item: isize, data: isize) {
        let Some(node) = self.node_of(data) else { return };
        if node.loaded || matches!(node.kind, Kind::Root | Kind::Column | Kind::Waiting) {
            return;
        }
        if let Some(n) = self.nodes.get_mut(data as usize) {
            n.loaded = true;
        }
        delete_children(self.tree, item);
        let c = self.conns[node.conn].clone();
        let failed = |e: &str| {
            if std::env::var_os("PHAI_SERVERS_TEST").is_some() {
                println!("servers: {e}");
            }
            if let Some(s) = studio() {
                s.say_status(e);
            }
        };
        match &node.kind {
            // A connection that names its database: that one only, as Visual Studio's Server
            // Explorer shows the database chosen in Add Connection; none named, every one.
            Kind::Server if !c.database.trim().is_empty() && c.kind != phai_data::Kind::Sqlite => {
                let d = c.database.trim().to_string();
                let db_item = self.add_node(item, &d, 2, Node { kind: Kind::Database, db: d.clone(), loaded: false, ..node.clone() });
                let db_data = self.nodes.len() as isize - 2;
                // Its tables at once: what was asked for.
                self.expanding(db_item, db_data);
                win::tree_expand(self.tree, db_item);
            }
            Kind::Server => match c.databases() {
                Ok(dbs) => {
                    for d in dbs {
                        self.add_node(item, &d, 2, Node { kind: Kind::Database, db: d.clone(), loaded: false, ..node.clone() });
                    }
                }
                Err(e) => {
                    failed(&e);
                    self.unload(item, data, &node, &e);
                }
            },
            Kind::Database => match c.tables(&node.db) {
                Ok(ts) => {
                    for (name, view) in ts {
                        self.add_node(item, &name, if view { 4 } else { 3 }, Node { kind: Kind::Table { view }, table: name.clone(), loaded: false, ..node.clone() });
                    }
                }
                Err(e) => {
                    failed(&e);
                    self.unload(item, data, &node, &e);
                }
            },
            Kind::Table { .. } => match c.columns(&node.db, &node.table) {
                Ok(cols) => {
                    for col in cols {
                        let key = if col.key == "PRI" { ", PK" } else { "" };
                        let null = if col.nullable { ", null" } else { "" };
                        self.add_node(item, &format!("{} ({}{key}{null})", col.name, col.ty), 5, Node { kind: Kind::Column, loaded: true, ..node.clone() });
                    }
                }
                Err(e) => {
                    failed(&e);
                    self.unload(item, data, &node, &e);
                }
            },
            _ => {}
        }
    }

    /// A branch that could not be read: why, under it in the tree - read again when it is opened
    /// again, or by Refresh.
    fn unload(&mut self, item: isize, data: isize, node: &Node, why: &str) {
        if let Some(n) = self.nodes.get_mut(data as usize) {
            n.loaded = false;
        }
        self.nodes.push(Node { kind: Kind::Waiting, ..node.clone() });
        let first = why.lines().next().unwrap_or(why);
        win::tree_add(self.tree, item, &tf("⚠ {0} - right-click, Refresh", &[&first]), 1, self.nodes.len() as isize - 1);
    }

    /// The `index`th connection opened and read - as one just added shows.
    pub fn open_connection(&mut self, index: usize) {
        let Some(k) = self.nodes.iter().enumerate().filter(|(_, n)| n.kind == Kind::Server).nth(index).map(|(k, _)| k) else { return };
        unsafe {
            let root = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(0 /* TVGN_ROOT */)), None).0;
            let mut item = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(root))).0;
            for _ in 0..index {
                item = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(1 /* TVGN_NEXT */)), Some(LPARAM(item))).0;
            }
            if item != 0 {
                self.expanding(item, k as isize);
                win::tree_expand(self.tree, item);
                wm::SendMessageW(self.tree, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9)), Some(LPARAM(item)));
            }
        }
    }

    /// Add Connection: which kind - MySQL, PostgreSQL, a SQLite file - and where and as whom, in
    /// one window; tested before it is kept.
    pub fn add_connection(&mut self, owner: HWND, start: Option<Connection>) -> bool {
        // One window: the kinds as cards, the chosen one's fields below.
        let Some(new) = crate::connectdlg::choose(owner, start.as_ref()) else { return false };
        let kind = new.kind;
        let probe = new.clone();
        match dialogs::wait_for(Some(owner), &tf("Add Connection - {0}", &[&kind.name()]), t("Connecting..."), move |_| probe.test()) {
            Some(Ok(version)) => {
                if let Some(s) = studio() {
                    s.say_status(&tf("Connected: {0} {1}", &[&kind.name(), &version]));
                }
                self.conns.push(new);
                self.save();
                self.fill();
                // Shown open, its databases read, as Visual Studio's Server Explorer shows one just added.
                self.open_connection(self.conns.len() - 1);
                // Data's commands can be used now.
                if let Some(st) = studio() {
                    st.say_bars();
                }
                true
            }
            Some(Err(e)) => {
                win::message(Some(owner), &crate::connectdlg::said_of(&new, &e), t("Could not connect"), wm::MB_ICONWARNING);
                false
            }
            None => false,
        }
    }

    /// Whether the node chosen is a table (not a view) - what can be dragged onto a form.
    pub fn table_chosen(&self) -> bool {
        self.selected().is_some_and(|(_, n)| matches!(n.kind, Kind::Table { view: false } | Kind::Column))
    }

    /// The table chosen onto the form being designed - Add to Form, or dragged there, as Visual
    /// Studio's Data Sources: a grid bound to it, Reload and Save.
    pub fn add_selected_to_form(&mut self, owner: HWND) {
        let Some((_, n)) = self.selected() else { return };
        if !matches!(n.kind, Kind::Table { view: false } | Kind::Column) {
            return;
        }
        let c = self.conns[n.conn].clone();
        let (d2, t2, c2) = (n.db.clone(), n.table.clone(), c.clone());
        match dialogs::wait_for(Some(owner), t("Add to Form"), t("Connecting..."), move |_| c2.columns(&d2, &t2)) {
            Some(Ok(cols)) => {
                if let Some(s) = studio() {
                    s.bind_table(&c, &n.db, &n.table, &cols);
                }
            }
            Some(Err(e)) => {
                win::message(Some(owner), &e, t("Add to Form"), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    /// The menu of the node right-clicked.
    pub fn menu(&mut self, owner: HWND) {
        const ADD: u32 = 1;
        const QUERY: u32 = 2;
        const ROWS: u32 = 3;
        const CODE: u32 = 4;
        const REFRESH: u32 = 5;
        const DELETE: u32 = 6;
        const NEW_TABLE: u32 = 7;
        const DESIGN: u32 = 8;
        const AS_SQL: u32 = 9;
        const TO_FORM: u32 = 10;
        const CLOSE: u32 = 11;
        const NEW_DB: u32 = 12;
        let sel = self.selected();
        let kind = sel.as_ref().map(|(_, n)| n.kind.clone()).unwrap_or(Kind::Root);
        let items: Vec<(u32, &str)> = match kind {
            Kind::Root | Kind::Waiting => vec![(ADD, t("Add Connection..."))],
            Kind::Server => vec![(QUERY, t("New Query")), (NEW_DB, t("New Database...")), (REFRESH, t("Refresh")), (CLOSE, t("Close Connection")), (0, ""), (ADD, t("Add Connection...")), (DELETE, t("Delete Connection"))],
            Kind::Database => vec![(QUERY, t("New Query")), (NEW_TABLE, t("New Table...")), (REFRESH, t("Refresh"))],
            Kind::Table { view: false } | Kind::Column => vec![(ROWS, t("Show Table Data")), (DESIGN, t("Design Table")), (AS_SQL, t("Script as SQL")), (QUERY, t("New Query")), (CODE, t("Copy code for a program")), (TO_FORM, t("Add to Form")), (REFRESH, t("Refresh"))],
            Kind::Table { view: true } => vec![(ROWS, t("Show Table Data")), (QUERY, t("New Query")), (REFRESH, t("Refresh"))],
        };
        match win::popup(owner, &items) {
            ADD => {
                self.add_connection(owner, None);
            }
            QUERY => {
                if let (Some((_, n)), Some(s)) = (sel, studio()) {
                    let c = &self.conns[n.conn];
                    let text = if n.table.is_empty() { String::new() } else { format!("SELECT * FROM {} LIMIT 100;\n", c.table_ref(&n.db, &n.table)) };
                    s.new_query(n.conn, &n.db, &text);
                }
            }
            NEW_TABLE | DESIGN => {
                self.table_designer(owner);
            }
            AS_SQL => {
                if let (Some((_, n)), Some(s)) = (sel, studio()) {
                    let c = self.conns[n.conn].clone();
                    let text = match c.create_sql(&n.db, &n.table) {
                        Ok(create) => {
                            let commented: String = create.lines().map(|l| format!("-- {l}\n")).collect();
                            format!("-- {}\n{commented}\n-- {}\n{}", t("The table as it is now:"), t("Take the -- off a change and Ctrl+Shift+E makes it:"), c.alter_examples(&n.db, &n.table))
                        }
                        Err(e) => format!("-- {e}\n"),
                    };
                    s.new_query(n.conn, &n.db, &text);
                }
            }
            ROWS => {
                if let (Some((_, n)), Some(s)) = (sel, studio()) {
                    s.show_table_rows(n.conn, &n.db, &n.table);
                }
            }
            TO_FORM => self.add_selected_to_form(owner),
            CODE => {
                if let Some((_, n)) = sel {
                    let c = &self.conns[n.conn];
                    let cols: Vec<Column> = c.columns(&n.db, &n.table).unwrap_or_default();
                    let mut cc = c.clone();
                    cc.database = n.db.clone();
                    let code = phai_data::code_for(&cc, Some((&n.table, &cols)));
                    crate::copy_text(owner, &code);
                    if let Some(s) = studio() {
                        s.say_status(&tf("Code for {0} copied - paste it into the program", &[&n.table]));
                    }
                }
            }
            // New Database: made, the server read again, the Database Designer on it.
            NEW_DB => {
                if let Some((_, n)) = sel {
                    if let Some(c) = self.conns.get(n.conn).cloned() {
                        self.new_database(owner, c);
                    }
                }
            }
            REFRESH => {
                if let Some((item, n)) = sel {
                    let data = item_data(self.tree, item).unwrap_or(-1);
                    if let Some(x) = self.nodes.get_mut(data as usize) {
                        x.loaded = false;
                    }
                    self.expanding(item, data);
                    win::tree_expand(self.tree, item);
                    let _ = n;
                }
            }
            // Close Connection, as Visual Studio's: folded, what was read of it let go - read again
            // when it is opened again. (Each look at the server is a connection of its own.)
            CLOSE => {
                if let Some((item, _)) = sel {
                    let data = item_data(self.tree, item).unwrap_or(-1);
                    if let Some(node) = self.node_of(data) {
                        unsafe {
                            wm::SendMessageW(self.tree, 0x1102 /* TVM_EXPAND */, Some(WPARAM(1 /* TVE_COLLAPSE */)), Some(LPARAM(item)));
                        }
                        delete_children(self.tree, item);
                        if let Some(n) = self.nodes.get_mut(data as usize) {
                            n.loaded = false;
                        }
                        self.nodes.push(Node { kind: Kind::Waiting, ..node });
                        win::tree_add(self.tree, item, "…", 1, self.nodes.len() as isize - 1);
                        if let Some(s) = studio() {
                            s.say_status(t("Connection closed - open it again to read it"));
                        }
                    }
                }
            }
            DELETE => {
                if let Some((_, n)) = sel {
                    if n.conn < self.conns.len() {
                        self.conns.remove(n.conn);
                        self.save();
                        self.fill();
                    }
                }
            }
            _ => {}
        }
    }

    /// A branch read from the database again.
    /// New Database on connection `c`: its name asked, made, the servers read again, and the
    /// Database Designer opened on it for its first table.
    pub fn new_database(&mut self, owner: HWND, c: Connection) {
        if c.kind == phai_data::Kind::Sqlite {
            win::message(Some(owner), t("A SQLite database is its file: add a connection to a new file"), t("New Database"), wm::MB_ICONINFORMATION);
            return;
        }
        let Some(name) = dialogs::ask(Some(owner), t("New Database"), t("The new database's name - letters, digits, _ and -:"), "", false) else { return };
        let name = name.trim().to_string();
        if name.is_empty() {
            return;
        }
        let (c2, n) = (c.clone(), name.clone());
        match dialogs::wait_for(Some(owner), t("New Database"), t("Saving..."), move |_| c2.create_database(&n)) {
            Some(Ok(())) => {
                self.reload_servers();
                design(c, &name, None);
            }
            Some(Err(e)) => {
                win::message(Some(owner), &e, t("New Database"), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    /// Every server opened read again - a database made.
    pub fn reload_servers(&mut self) {
        for item in self.all_items() {
            let data = item_data(self.tree, item).unwrap_or(-1);
            if self.node_of(data).is_some_and(|n| n.kind == Kind::Server && n.loaded) {
                self.reload(item);
            }
        }
    }

    /// Every item of the tree, top to bottom.
    fn all_items(&self) -> Vec<isize> {
        fn walk(tree: HWND, item: isize, out: &mut Vec<isize>) {
            let mut it = item;
            while it != 0 {
                out.push(it);
                let child = unsafe { wm::SendMessageW(tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(it))).0 };
                walk(tree, child, out);
                it = unsafe { wm::SendMessageW(tree, 0x110A, Some(WPARAM(1 /* TVGN_NEXT */)), Some(LPARAM(it))).0 };
            }
        }
        let root = unsafe { wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(0 /* TVGN_ROOT */)), None).0 };
        let mut all = Vec::new();
        walk(self.tree, root, &mut all);
        all
    }

    /// Every open database named `db` read again - a table made, changed or renamed in it.
    pub fn reload_database(&mut self, db: &str) {
        fn walk(tree: HWND, item: isize, out: &mut Vec<isize>) {
            let mut it = item;
            while it != 0 {
                out.push(it);
                let child = unsafe { wm::SendMessageW(tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(it))).0 };
                walk(tree, child, out);
                it = unsafe { wm::SendMessageW(tree, 0x110A, Some(WPARAM(1 /* TVGN_NEXT */)), Some(LPARAM(it))).0 };
            }
        }
        let root = unsafe { wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(0 /* TVGN_ROOT */)), None).0 };
        let mut all = Vec::new();
        walk(self.tree, root, &mut all);
        for item in all {
            let data = item_data(self.tree, item).unwrap_or(-1);
            if self.node_of(data).is_some_and(|n| n.kind == Kind::Database && n.db == db && n.loaded) {
                self.reload(item);
            }
        }
    }

    fn reload(&mut self, item: isize) {
        let data = item_data(self.tree, item).unwrap_or(-1);
        if let Some(x) = self.nodes.get_mut(data as usize) {
            x.loaded = false;
        }
        self.expanding(item, data);
        win::tree_expand(self.tree, item);
    }

    /// The Table Designer on what is chosen: a table's design, or a new table in the database
    /// chosen - from the right-click menu and from Tools > Table Designer. False when neither is
    /// chosen.
    pub fn table_designer(&mut self, owner: HWND) -> bool {
        // Nothing chosen (or a connection): a new table in the database the connection names -
        // its first one when it names none.
        let chosen = self.selected().filter(|(_, n)| !matches!(n.kind, Kind::Root | Kind::Waiting | Kind::Server));
        let Some((item, n)) = chosen else {
            let conn = self.selected().map(|(_, n)| n.conn).filter(|c| *c < self.conns.len()).unwrap_or(0);
            let Some(c) = self.conns.get(conn).cloned() else { return false };
            let db = if c.database.trim().is_empty() { c.databases().ok().and_then(|d| d.into_iter().next()).unwrap_or_default() } else { c.database.clone() };
            // No database on the server yet: one made first.
            if db.is_empty() && c.kind != phai_data::Kind::Sqlite {
                self.new_database(owner, c);
                return true;
            }
            design(c, &db, None);
            return true;
        };
        match n.kind {
            Kind::Database => {
                // A new table, in a tab; Update reads the database's tables again.
                let c = self.conns[n.conn].clone();
                design(c, &n.db, None);
                let _ = item;
                true
            }
            Kind::Table { view: false } | Kind::Column => {
                let c = self.conns[n.conn].clone();
                let (d2, t2, c2) = (n.db.clone(), n.table.clone(), c.clone());
                match dialogs::wait_for(Some(owner), t("Design Table"), t("Connecting..."), move |_| c2.table_def(&d2, &t2)) {
                    Some(Ok(def)) => design(c, &n.db, Some(def)),
                    Some(Err(e)) => {
                        win::message(Some(owner), &e, t("Design Table"), wm::MB_ICONWARNING);
                    }
                    None => {}
                }
                true
            }
            _ => false,
        }
    }

    /// The designer on the first connection's table - for checking by picture.
    pub fn design_first(&mut self, owner: HWND, table: &str) {
        let Some(c) = self.conns.first().cloned() else { return };
        let def = c.table_def("main", table).ok();
        let _ = owner;
        design(c, "main", def);
    }

    /// A click on the first connection's [+], as the mouse makes it (messages to the tree, the
    /// mouse itself untouched) - for checking what a person's click does.
    pub fn click_first(&self) {
        use windows::Win32::UI::Controls as cc;
        unsafe {
            let root = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(0)), None).0;
            let first = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(4)), Some(LPARAM(root))).0;
            if first == 0 {
                return;
            }
            let mut r = RECT::default();
            *(&mut r as *mut RECT as *mut isize) = first;
            wm::SendMessageW(self.tree, 0x1104 /* TVM_GETITEMRECT */, Some(WPARAM(1)), Some(LPARAM(&mut r as *mut _ as isize)));
            let y = (r.top + r.bottom) / 2;
            // The [+]: where the tree's own hit test says its button is.
            let mut x = 0;
            for cx in 0..r.left {
                let mut ht = cc::TVHITTESTINFO { pt: windows::Win32::Foundation::POINT { x: cx, y }, ..Default::default() };
                wm::SendMessageW(self.tree, 0x1111 /* TVM_HITTEST */, None, Some(LPARAM(&mut ht as *mut _ as isize)));
                if ht.flags.0 & 0x0010 /* TVHT_ONITEMBUTTON */ != 0 {
                    x = cx;
                    break;
                }
            }
            let at = LPARAM(((y as isize) << 16) | (x as isize & 0xFFFF));
            println!("servers: click at {x},{y} (text from {}), item {first:#x}", r.left);
            wm::SendMessageW(self.tree, wm::WM_LBUTTONDOWN, Some(WPARAM(1)), Some(at));
            wm::SendMessageW(self.tree, wm::WM_LBUTTONUP, Some(WPARAM(0)), Some(at));
            let _ = cc::NMHDR::default();
        }
    }

    /// The first connection opened - for checking by picture.
    pub fn expand_first(&mut self) {
        let Some(k) = self.nodes.iter().position(|n| n.kind == Kind::Server) else { return };
        unsafe {
            let root = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(0 /* TVGN_ROOT */)), None).0;
            let first = wm::SendMessageW(self.tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(root))).0;
            if first != 0 {
                self.expanding(first, k as isize);
                win::tree_expand(self.tree, first);
            }
        }
    }

    /// A double-click: a table's rows.
    pub fn activate(&mut self) {
        if let Some((_, n)) = self.selected() {
            if matches!(n.kind, Kind::Table { .. }) {
                if let Some(s) = studio() {
                    s.show_table_rows(n.conn, &n.db, &n.table);
                }
            }
        }
    }
}

fn tree_caret(tree: HWND) -> Option<isize> {
    let item = unsafe { wm::SendMessageW(tree, 0x110A /* TVM_GETNEXTITEM */, Some(WPARAM(9 /* TVGN_CARET */)), None).0 };
    (item != 0).then_some(item)
}

fn item_data(tree: HWND, item: isize) -> Option<isize> {
    use windows::Win32::UI::Controls as cc;
    let mut it = cc::TVITEMW { mask: cc::TVIF_PARAM, hItem: cc::HTREEITEM(item as _), ..Default::default() };
    let ok = unsafe { wm::SendMessageW(tree, 0x113E /* TVM_GETITEMW */, None, Some(LPARAM(&mut it as *mut _ as isize))).0 };
    (ok != 0).then_some(it.lParam.0)
}

fn delete_children(tree: HWND, item: isize) {
    unsafe {
        loop {
            let child = wm::SendMessageW(tree, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(item))).0;
            if child == 0 {
                break;
            }
            wm::SendMessageW(tree, 0x1101 /* TVM_DELETEITEM */, None, Some(LPARAM(child)));
        }
    }
}
