//! What each command needs before it can be used - a form open, a project, a code file, a
//! database connection, a Git repository, a program being debugged... A command without what it
//! needs is greyed in the menus (as each menu opens) and on the bars, so nothing is clicked that
//! can only say "not now".

use std::path::{Path, PathBuf};

use crate::*;

/// What is there now, worked out once for a menu's or a bar's worth of commands.
pub struct Now {
    /// Something open (not the empty start).
    open: bool,
    /// A form shown in the designer.
    form: bool,
    /// A code file shown in the editor.
    code: bool,
    /// A Rust code file.
    rust: bool,
    /// The open thing's own file.
    path: Option<PathBuf>,
    /// The Cargo project it is in.
    project: bool,
    /// A Git repository it is in.
    repo: bool,
    /// Controls chosen on the form.
    chosen: usize,
    /// The form has a theme (a style sheet beside it).
    theme: bool,
    /// Database connections in Server Explorer, and a query window in front.
    conns: bool,
    query: bool,
    /// A program being debugged, stopped at a place; a program running without the debugger.
    debugging: bool,
    paused: bool,
    running: bool,
    breakpoints: bool,
    undo: bool,
    redo: bool,
    back: bool,
    forward: bool,
}

/// The `.git` a file or folder is in, looked for up its folders (no `git` run for it).
fn in_repo(p: &Path) -> bool {
    p.ancestors().any(|d| d.join(".git").exists())
}

impl Now {
    pub fn of(s: &Studio) -> Now {
        let open = !s.nothing;
        let code = open && s.doc.code.is_some();
        let path = s.doc.path.clone().filter(|_| open);
        let project = path.as_deref().and_then(cargo::project_of);
        Now {
            open,
            form: open && !code,
            code,
            rust: code && path.as_ref().is_some_and(|p| p.extension().is_some_and(|e| e == "rs")),
            repo: path.as_deref().is_some_and(in_repo),
            project: project.is_some(),
            chosen: if open && !code { s.selection().len() } else { 0 },
            theme: open && !code && path.is_some() && s.doc.form.form.theme.is_some(),
            conns: s.servers.as_ref().is_some_and(|v| !v.conns.is_empty()),
            query: path.as_ref().is_some_and(|p| s.queries.keys().any(|q| phai_editor::lsp::same_path(q, p))),
            debugging: s.debug.is_some(),
            paused: s.debug.is_some() && s.paused,
            running: s.running.is_some(),
            breakpoints: s.bps.values().any(|v| !v.is_empty()),
            undo: code || !s.undo.is_empty(),
            redo: code || !s.redo.is_empty(),
            back: s.nav_at > 0,
            forward: s.nav_at + 1 < s.nav.len(),
            path,
        }
    }

    /// Whether command `id` can be used now. What is not named here always can.
    pub fn enabled(&self, id: i32) -> bool {
        let n = self.chosen;
        // The Format menu's arranging: two chosen or more, or one where the change works alone
        // (centre in the form, to the grid).
        if (ID_FORMAT..ID_FORMAT + FORMATS.len() as i32).contains(&id) {
            let op = FORMATS[(id - ID_FORMAT) as usize];
            return self.form && (n >= 2 || n == 1 && op.works_alone());
        }
        if (ID_THEME_FILE..ID_THEME_FILE + 40).contains(&id) {
            return self.form && self.path.is_some();
        }
        match id {
            // File
            ID_CLOSE | ID_CLOSE_PROJECT | ID_SAVE | ID_SAVE_AS | ID_SAVE_ALL => self.open,
            // Edit
            ID_UNDO => self.undo,
            ID_REDO => self.redo,
            ID_CUT | ID_COPY | ID_DELETE => self.code || n >= 1,
            ID_DUPLICATE => n >= 1,
            ID_PASTE | ID_SELECT_ALL => self.open,
            ID_GO_DEFINITION | ID_PEEK | ID_PARAM_INFO | ID_QUICK_ACTIONS | ID_REFERENCES | ID_CALLS | ID_RENAME => self.rust && self.project,
            ID_COMMENT | ID_FORMAT_DOC | ID_FOLD | ID_FOLD_ALL | ID_SNIPPET | ID_BM_TOGGLE | ID_BM_NEXT | ID_BM_PREV | ID_BM_CLEAR | ID_MOVE_UP | ID_MOVE_DOWN => self.code,
            ID_FIND_IN_FILES | ID_GO_TO_ALL => self.project,
            ID_NAV_BACK => self.back,
            ID_NAV_FORWARD => self.forward,
            // View
            ID_PREVIEW => self.form,
            ID_SPLIT | ID_NEW_WINDOW => self.code,
            // Format
            ID_FRONT | ID_BACK => self.form && n >= 1,
            ID_LOCK | ID_TAB_ORDER => self.form,
            // Build
            ID_BUILD | ID_REBUILD | ID_CLEAN | ID_ANALYSE | ID_BUILD_ALL | ID_REBUILD_ALL | ID_CLEAN_ALL | ID_ANALYSE_ALL | ID_BATCH_BUILD | ID_PUBLISH | ID_BUILD_WEB | ID_RUN_WEB => self.project,
            // Debug
            ID_RUN => self.paused || (self.project && !self.debugging),
            ID_RUN_NODEBUG => self.project && !self.debugging,
            ID_BREAK_ALL => self.debugging && !self.paused,
            ID_STOP => self.debugging || self.running,
            ID_ATTACH => !self.debugging,
            ID_DETACH | ID_ADD_WATCH => self.debugging,
            ID_STEP_INTO | ID_STEP_OVER | ID_STEP_OUT | ID_THREADS_WIN | ID_MEMORY_WIN | ID_DISASSEMBLY | ID_DATA_BP => self.paused,
            ID_TOGGLE_BP | ID_BP_CONDITION => self.rust,
            ID_CLEAR_BPS => self.breakpoints,
            // Project
            ID_ADD_FORM | ID_ADD_USER_CONTROL | ID_ADD_REPORT | ID_MANAGE_CRATES | ID_PROJECT_PROPS => self.project,
            // Tools > Data: a connection first; a query to run.
            ID_NEW_QUERY | ID_TABLE_DESIGNER => self.conns,
            ID_EXECUTE => self.query,
            // Tools > Test
            ID_RUN_TESTS | ID_RUN_ALL_TESTS | ID_COVERAGE => self.project,
            // Tools > Git: a repository.
            ID_GIT_CHANGES | ID_MERGE => self.repo,
            ID_GIT_HISTORY | ID_GIT_BLAME => self.repo && self.path.is_some(),
            // Tools > Theme: a saved form - its theme files are beside it.
            ID_THEME_NEW | ID_THEME_IMPORT | ID_THEME_NONE => self.form && self.path.is_some(),
            ID_THEME_EDIT | ID_THEME_EDITOR => self.theme,
            _ => true,
        }
    }
}

/// Every command in `menu` greyed or not as things are now; a menu within greyed when nothing
/// in it can be used. Whether anything in `menu` can be used.
pub fn grey_menu(menu: windows::Win32::UI::WindowsAndMessaging::HMENU, now: &Now) -> bool {
    use windows::Win32::UI::WindowsAndMessaging as wm;
    let mut any = false;
    unsafe {
        for i in 0..wm::GetMenuItemCount(Some(menu)).max(0) {
            let sub = wm::GetSubMenu(menu, i);
            let on = if !sub.is_invalid() {
                grey_menu(sub, now)
            } else {
                let id = wm::GetMenuItemID(menu, i);
                if id == u32::MAX || id == 0 {
                    continue;
                }
                now.enabled(id as i32)
            };
            let _ = wm::EnableMenuItem(menu, i as u32, wm::MF_BYPOSITION | if on { wm::MF_ENABLED } else { wm::MF_GRAYED });
            any |= on;
        }
    }
    any
}
