//! Server Explorer, as Visual Studio's: Data Connections - each a MySQL server - and under each its
//! databases, their tables and views, and the columns of each. Branches are read from the server
//! when first opened. Right-click for what a node can do: add a connection, a new query, the rows
//! of a table, code for a program to read it. The connections are remembered, a password only
//! encrypted by Windows.

use std::path::PathBuf;

use phai_data::{Column, Connection};
use phai_ui::dialogs::{self, Field};
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
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
        let failed = |e: String| {
            if let Some(s) = studio() {
                s.say_status(&e);
            }
        };
        match &node.kind {
            Kind::Server => match c.databases() {
                Ok(dbs) => {
                    for d in dbs {
                        self.add_node(item, &d, 2, Node { kind: Kind::Database, db: d.clone(), loaded: false, ..node.clone() });
                    }
                }
                Err(e) => {
                    failed(e);
                    self.unload(item, data, &node);
                }
            },
            Kind::Database => match c.tables(&node.db) {
                Ok(ts) => {
                    for (name, view) in ts {
                        self.add_node(item, &name, if view { 4 } else { 3 }, Node { kind: Kind::Table { view }, table: name.clone(), loaded: false, ..node.clone() });
                    }
                }
                Err(e) => {
                    failed(e);
                    self.unload(item, data, &node);
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
                    failed(e);
                    self.unload(item, data, &node);
                }
            },
            _ => {}
        }
    }

    /// A branch that could not be read: closed again, to be read another time.
    fn unload(&mut self, item: isize, data: isize, node: &Node) {
        if let Some(n) = self.nodes.get_mut(data as usize) {
            n.loaded = false;
        }
        self.nodes.push(Node { kind: Kind::Waiting, ..node.clone() });
        win::tree_add(self.tree, item, "…", 1, self.nodes.len() as isize - 1);
        unsafe {
            // Closed: [+] again, to try another time.
            wm::PostMessageW(Some(self.tree), 0x1102 /* TVM_EXPAND */, WPARAM(1 /* TVE_COLLAPSE */), LPARAM(item)).ok();
        }
    }

    /// Add Connection: first which kind - MySQL, PostgreSQL, a SQLite file - then where and as
    /// whom; tested before it is kept.
    pub fn add_connection(&mut self, owner: HWND, start: Option<Connection>) -> bool {
        let kind = match &start {
            Some(c) => c.kind,
            None => match win::popup(owner, &[(1, "MySQL / MariaDB"), (2, "PostgreSQL"), (3, t("SQLite file..."))]) {
                1 => phai_data::Kind::MySql,
                2 => phai_data::Kind::Postgres,
                3 => phai_data::Kind::Sqlite,
                _ => return false,
            },
        };
        let new = if kind == phai_data::Kind::Sqlite {
            // A file, as one is opened - or a new name, which SQLite makes.
            let Some(path) = crate::file_dialog_pub(owner, true, "SQLite (*.db; *.sqlite; *.sqlite3)", "*.db;*.sqlite;*.sqlite3", "data.db") else { return false };
            Connection { host: path.to_string_lossy().to_string(), kind, ..Default::default() }
        } else {
            let c = start.unwrap_or(Connection { host: "localhost".into(), port: kind.default_port(), user: if kind == phai_data::Kind::Postgres { "postgres".into() } else { "root".into() }, kind, ..Default::default() });
            let port = c.port.to_string();
            let fields = [
                Field::Text(t("Server"), &c.host),
                Field::Text(t("Port"), &port),
                Field::Text(t("User"), &c.user),
                Field::Password(t("Password"), &c.password),
                Field::Text(t("Database"), &c.database),
                Field::Check(t("Encrypt the connection (TLS)"), c.ssl),
                Field::Check(t("Trust the server's own certificate"), c.trust_cert),
            ];
            let Some(a) = dialogs::ask_fields(Some(owner), &tf("Add Connection - {0}", &[&kind.name()]), &fields) else { return false };
            Connection { host: a[0].trim().into(), port: a[1].trim().parse().unwrap_or(kind.default_port()), user: a[2].trim().into(), password: a[3].clone(), database: a[4].trim().into(), ssl: a[5] == "1", trust_cert: a[6] == "1", kind }
        };
        let probe = new.clone();
        match dialogs::wait_for(Some(owner), &tf("Add Connection - {0}", &[&kind.name()]), t("Connecting..."), move |_| probe.test()) {
            Some(Ok(version)) => {
                if let Some(s) = studio() {
                    s.say_status(&tf("Connected: {0} {1}", &[&kind.name(), &version]));
                }
                self.conns.push(new);
                self.save();
                self.fill();
                true
            }
            Some(Err(e)) => {
                win::message(Some(owner), &e, t("Could not connect"), wm::MB_ICONWARNING);
                false
            }
            None => false,
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
        let sel = self.selected();
        let kind = sel.as_ref().map(|(_, n)| n.kind.clone()).unwrap_or(Kind::Root);
        let items: Vec<(u32, &str)> = match kind {
            Kind::Root | Kind::Waiting => vec![(ADD, t("Add Connection..."))],
            Kind::Server => vec![(QUERY, t("New Query")), (REFRESH, t("Refresh")), (0, ""), (ADD, t("Add Connection...")), (DELETE, t("Delete"))],
            Kind::Database => vec![(QUERY, t("New Query")), (NEW_TABLE, t("New Table...")), (REFRESH, t("Refresh"))],
            Kind::Table { view: false } | Kind::Column => vec![(ROWS, t("Show Table Data")), (DESIGN, t("Design Table")), (QUERY, t("New Query")), (CODE, t("Copy code for a program")), (REFRESH, t("Refresh"))],
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
            NEW_TABLE => {
                if let (Some((_, n)), Some(s)) = (sel, studio()) {
                    let c = &self.conns[n.conn];
                    let text = format!("-- {}\n{}", t("Change the columns, then Ctrl+Shift+E makes the table; Refresh shows it."), c.new_table_sql());
                    s.new_query(n.conn, &n.db, &text);
                }
            }
            DESIGN => {
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
