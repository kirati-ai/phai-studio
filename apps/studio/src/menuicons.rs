//! A picture beside every menu command, each in a colour that says what kind of thing it does,
//! as Visual Studio's menus have them: green makes or runs, blue opens, saves and moves about,
//! red stops, deletes or is an error, amber is a folder, a mark or a warning, purple is code.

use phai_ui::icon::{self, glyph};
use phai_ui::theme::Color;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::*;

const GREEN: Color = Color::rgb(0x2E, 0x8B, 0x3A);
const BLUE: Color = Color::rgb(0x1C, 0x6F, 0xD1);
const RED: Color = Color::rgb(0xC4, 0x2B, 0x1C);
const AMBER: Color = Color::rgb(0xC2, 0x7C, 0x0E);
const PURPLE: Color = Color::rgb(0x7A, 0x3E, 0xC8);
const TEAL: Color = Color::rgb(0x00, 0x7A, 0x87);
const GREY: Color = Color::rgb(0x42, 0x42, 0x42);

/// The picture and its colour for a menu command; `None` leaves it bare (a tick shows instead).
pub fn icon_of(id: i32) -> Option<(u16, Color)> {
    // Families of commands, one picture each.
    if (ID_FORMAT..ID_FORMAT + 21).contains(&id) {
        return layout_items().into_iter().find(|(i, _, _)| *i == id).map(|(_, _, g)| (g, BLUE));
    }
    if (ID_RECENT..ID_RECENT + RECENT_MAX as i32).contains(&id) {
        return Some((glyph::DOCUMENT, GREY));
    }
    if (ID_RECENT_PROJECT..ID_RECENT_PROJECT + RECENT_MAX as i32).contains(&id) {
        return Some((glyph::FOLDER, AMBER));
    }
    if (ID_TEMPLATE..ID_TEMPLATE + 10).contains(&id) {
        return Some((glyph::FORM, BLUE));
    }
    if (ID_OTHER_WINDOW..ID_OTHER_WINDOW + 16).contains(&id) {
        return Some((0xE737, GREY));
    }
    if (ID_THEME_FILE..ID_THEME_FILE + 40).contains(&id) {
        return Some((glyph::COLOR, PURPLE));
    }
    Some(match id {
        // File
        ID_NEW_PROJECT => (glyph::NEW_FOLDER, AMBER),
        ID_NEW => (glyph::NEW, GREEN),
        ID_OPEN_PROJECT => (0xE838, AMBER), // FolderOpen
        ID_OPEN => (glyph::OPEN, BLUE),
        ID_START_WINDOW => (0xE80F, BLUE), // Home
        ID_CLOSE => (0xE711, GREY),
        ID_CLOSE_PROJECT => (0xE711, RED),
        ID_SAVE => (glyph::SAVE, BLUE),
        ID_SAVE_AS => (0xE792, BLUE), // SaveAs
        ID_SAVE_ALL => (glyph::SAVE_ALL, BLUE),
        ID_EXIT => (0xE7E8, RED), // PowerButton
        // Edit
        ID_UNDO => (glyph::UNDO, BLUE),
        ID_REDO => (glyph::REDO, BLUE),
        ID_CUT => (glyph::CUT, PURPLE),
        ID_COPY => (glyph::COPY, BLUE),
        ID_PASTE => (glyph::PASTE, AMBER),
        ID_DUPLICATE => (glyph::COPY, TEAL),
        ID_DELETE => (glyph::DELETE, RED),
        ID_SELECT_ALL => (0xE8B3, BLUE), // SelectAll
        ID_GO_DEFINITION => (glyph::FORWARD, PURPLE),
        ID_PEEK => (0xE7B3, PURPLE), // RedEye
        ID_PARAM_INFO => (0xE946, BLUE), // Info
        ID_QUICK_ACTIONS => (0xE82F, AMBER), // Lightbulb
        ID_REFERENCES => (0xE71B, PURPLE), // Link
        ID_CALLS => (glyph::LIST, PURPLE),
        ID_RENAME => (glyph::RENAME, PURPLE),
        ID_COMMENT => (0xE90A, GREEN), // Comment
        ID_FORMAT_DOC => (glyph::FONT, TEAL),
        ID_FOLD => (glyph::DROPDOWN, GREY),
        ID_FOLD_ALL => (glyph::GROUP, GREY),
        ID_FIND_IN_FILES => (glyph::SEARCH, BLUE),
        ID_GO_TO_ALL => (glyph::SEARCH, PURPLE),
        ID_SNIPPET => (glyph::CODE, PURPLE),
        ID_BM_TOGGLE => (0xE8A4, AMBER), // Bookmarks
        ID_BM_NEXT => (glyph::FORWARD, AMBER),
        ID_BM_PREV => (glyph::BACK, AMBER),
        ID_BM_CLEAR => (0xE894, RED), // Clear
        ID_MOVE_UP => (glyph::UP, BLUE),
        ID_MOVE_DOWN => (0xE74B, BLUE), // Down
        ID_NEXT_ERROR => (0xE783, RED), // Error
        ID_PREV_ERROR => (0xE783, RED),
        ID_EDIT_MENUS => (glyph::MENU_BAR, TEAL),
        ID_CUSTOM_STD => (glyph::SETTINGS, TEAL),
        ID_EDIT_TOOLBAR => (glyph::TOOLBAR, TEAL),
        // View
        ID_PREVIEW => (glyph::MONITOR, BLUE),
        ID_VIEW_TOOLBOX => (glyph::TOOLBAR, RED),
        ID_VIEW_EXPLORER => (glyph::LIBRARY, PURPLE),
        ID_VIEW_PROPS => (glyph::BUILD, GREY),
        ID_VIEW_OUTPUT => (0xE8BC, BLUE), // ShowResults
        ID_VIEW_SERVERS => (0xE968, TEAL), // Network
        ID_VIEW_OUTLINE => (glyph::LIST, PURPLE),
        ID_SPLIT => (glyph::GROUP, BLUE),
        ID_NEW_WINDOW => (0xE8A7, BLUE), // OpenInNewWindow
        ID_TASK_LIST => (0xE9D5, GREEN), // CheckList
        ID_LIGHT => (glyph::BRIGHTNESS, AMBER),
        ID_DARK => (0xE708, PURPLE), // QuietHours: a moon
        ID_THAI | ID_ENGLISH => (glyph::LANGUAGE, BLUE),
        // Format
        ID_FRONT | ID_BACK | ID_TAB_ORDER => return layout_items().into_iter().find(|(i, _, _)| *i == id).map(|(_, _, g)| (g, BLUE)),
        ID_LOCK => (0xE72E, AMBER), // Lock
        // Build
        ID_BUILD_ALL | ID_BUILD => (glyph::BUILD, GREEN),
        ID_REBUILD_ALL | ID_REBUILD => (glyph::REFRESH, GREEN),
        ID_CLEAN_ALL | ID_CLEAN => (0xE894, AMBER), // Clear
        ID_BATCH_BUILD => (glyph::LIST, GREEN),
        ID_ANALYSE_ALL | ID_ANALYSE => (glyph::PROGRESS, PURPLE),
        ID_PUBLISH => (glyph::UPLOAD, BLUE),
        ID_BUILD_WEB => (0xE774, BLUE), // Globe
        ID_RUN_WEB => (0xE774, GREEN),
        ID_RUN_PHONE => (0xE8EA, GREEN),
        ID_DEVICES => (0xE8CC, BLUE),
        ID_WEB_APP => (0xE8EA, BLUE),
        ID_DEBUG | ID_RELEASE => return None,
        // Debug
        ID_RUN => (glyph::PLAY, GREEN),
        ID_RUN_NODEBUG => (glyph::PLAY, TEAL),
        ID_BREAK_ALL => (0xE769, BLUE), // Pause
        ID_STOP => (glyph::STOP, RED),
        ID_ATTACH => (0xE71B, GREY), // Link
        ID_DETACH => (0xE711, GREY),
        ID_STEP_INTO => (0xE74B, BLUE),
        ID_STEP_OVER => (glyph::FORWARD, BLUE),
        ID_STEP_OUT => (glyph::UP, BLUE),
        ID_ADD_WATCH => (0xE7B3, BLUE), // RedEye
        ID_IMMEDIATE => (0xE756, GREY), // CommandPrompt
        ID_DIAG_TOOLS => (glyph::PROGRESS, PURPLE),
        ID_BREAKPOINTS_WIN => (0xE7C8, RED), // Record: a red dot
        ID_THREADS_WIN => (glyph::LIST, GREY),
        ID_EXCEPTIONS => (0xE7BA, AMBER), // Warning
        ID_MEMORY_WIN => (glyph::NUMBER, GREY),
        ID_DISASSEMBLY => (glyph::CODE, GREY),
        ID_DATA_BP => (0xE7C8, AMBER),
        ID_TOGGLE_BP => (0xE7C8, RED),
        ID_BP_CONDITION => (0xE7C8, PURPLE),
        ID_CLEAR_BPS => (glyph::DELETE, RED),
        // Project
        ID_ADD_FORM => (glyph::FORM, GREEN),
        ID_ADD_USER_CONTROL => (glyph::GROUP, GREEN),
        ID_IMPORT_WINFORMS => (glyph::DOWNLOAD, GREEN),
        ID_ADD_REPORT => (0xE749, GREEN), // Print
        ID_MANAGE_CRATES => (0xE7B8, BLUE), // Package
        ID_PROJECT_PROPS => (glyph::BUILD, GREY),
        // Test
        ID_RUN_TESTS | ID_RUN_ALL_TESTS => (0xE9D5, GREEN), // CheckList
        ID_COVERAGE => (glyph::PROGRESS, PURPLE),
        // Git
        ID_GIT_CHANGES => (0xE895, AMBER), // Sync
        ID_GIT_HISTORY => (0xE81C, PURPLE), // History
        ID_MERGE => (0xE8AB, RED), // Switch
        ID_GIT_BLAME => (0xE77B, GREY), // Contact
        // Data
        ID_ADD_CONNECTION => (0xE71B, TEAL),
        ID_NEW_QUERY => (glyph::DOCUMENT, TEAL),
        ID_EXECUTE => (glyph::PLAY, GREEN),
        // Theme
        ID_THEME_EDITOR => (glyph::COLOR, PURPLE),
        ID_THEME_EDIT => (0xE70F, PURPLE), // Edit
        ID_THEME_NEW => (glyph::NEW, GREEN),
        ID_THEME_IMPORT => (glyph::DOWNLOAD, BLUE),
        ID_THEME_NONE => (0xE711, GREY),
        // Tools
        ID_CREATE_GUID => (glyph::NUMBER, PURPLE),
        ID_CMD_PROMPT => (0xE756, GREY),
        ID_POWERSHELL => (0xE756, BLUE),
        ID_OPTIONS => (glyph::SETTINGS, GREY),
        ID_VIEW_TERMINAL => (0xE756, GREY), // CommandPrompt
        ID_TABLE_DESIGNER => (0xE80A, TEAL), // ViewAll: a table's grid
        // Help
        ID_HELP_STUDIO => (0xE897, BLUE), // Help
        ID_HELP_RUST => (0xE736, AMBER), // ReadingMode: a book
        ID_HELP_ERRORS => (0xE783, RED),
        ID_CHECK_UPDATES => (glyph::DOWNLOAD, GREEN),
        ID_SUPPORT => (0xEB52, RED), // HeartFill
        ID_ABOUT => (0xE946, BLUE), // Info
        _ => return None,
    })
}

/// Every command in the menu bar's menus, and in the menus within them, given its picture; the
/// bar's own words stay bare, as Visual Studio's do.
pub fn apply(bar: HMENU) {
    unsafe {
        for i in 0..wm::GetMenuItemCount(Some(bar)).max(0) {
            let sub = wm::GetSubMenu(bar, i);
            if !sub.is_invalid() {
                apply_in(sub);
            }
        }
    }
}

/// The pictures in one menu; returns the first one given, for the item that opens this menu.
fn apply_in(menu: HMENU) -> Option<(u16, Color)> {
    let mut first = None;
    unsafe {
        for i in 0..wm::GetMenuItemCount(Some(menu)).max(0) {
            let sub = wm::GetSubMenu(menu, i);
            let found = if !sub.is_invalid() {
                // A menu within: the picture of its first command (New, Open, Align...).
                let pic = apply_in(sub);
                if let Some(bmp) = pic.and_then(|(g, c)| icon::menu_bitmap_in(g, c)) {
                    let info = wm::MENUITEMINFOW { cbSize: std::mem::size_of::<wm::MENUITEMINFOW>() as u32, fMask: wm::MIIM_BITMAP, hbmpItem: bmp, ..Default::default() };
                    let _ = wm::SetMenuItemInfoW(menu, i as u32, true, &info);
                }
                pic
            } else {
                let id = wm::GetMenuItemID(menu, i);
                let pic = if id == u32::MAX || id == 0 { None } else { icon_of(id as i32) };
                if let Some((g, c)) = pic {
                    icon::set_menu_glyph_in(menu, id, g, c);
                }
                pic
            };
            if first.is_none() {
                first = found;
            }
        }
    }
    first
}

#[cfg(test)]
mod tests {
    /// Every menu picture on one sheet, 40 to a row, at the size a 150 % screen draws them -
    /// to look over by eye: PHAI_ICON_SHEET names the PNG.
    #[test]
    #[ignore]
    fn every_menu_picture_on_one_sheet() {
        let out = std::env::var("PHAI_ICON_SHEET").expect("PHAI_ICON_SHEET");
        let icons: Vec<(u16, phai_ui::theme::Color)> = (0..2000).filter_map(super::icon_of).collect();
        let (cell, across) = (28usize, 20usize);
        let down = icons.len().div_ceil(across);
        let (w, h) = (cell * across, cell * down);
        let mut rgba = vec![255u8; w * h * 4];
        for (k, (g, c)) in icons.iter().enumerate() {
            let px = phai_ui::icon::glyph_pixels(*g, 24, *c).expect("drawn");
            let (ox, oy) = ((k % across) * cell + 2, (k / across) * cell + 2);
            for y in 0..24 {
                for x in 0..24 {
                    let p = &px[(y * 24 + x) * 4..][..4];
                    let a = p[3] as u32;
                    let o = ((oy + y) * w + ox + x) * 4;
                    // BGRA over white.
                    for (i, ch) in [2usize, 1, 0].iter().enumerate() {
                        rgba[o + i] = ((p[*ch] as u32 * a + 255 * (255 - a)) / 255) as u8;
                    }
                }
            }
        }
        let file = std::fs::File::create(out).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&rgba).unwrap();
        println!("{} pictures", icons.len());
    }
}
