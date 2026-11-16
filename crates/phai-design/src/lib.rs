//! **phai-design** - how the Phai family's programs look, so each one does not have to decide.
//!
//! * [`metrics`]: Windows' own spacing in design pixels - what the owner chose over large type and
//!   generous space: 9 pt system font, 11 from the edges, 7 between rows, controls 23 high.
//! * [`templates`]: the windows nearly every program has - a dialog that answers OK or Cancel, a
//!   main window with menu, toolbar and status bar, a form of labelled fields - already laid out
//!   by those metrics, anchored so they grow properly. Phai Studio offers them under
//!   File > New from template.
//! * Themes: `phai:light` and `phai:dark` in phai-ui, which a product's own sheet imports and
//!   changes only the accent of.

pub mod metrics {
    //! Design pixels at 96 DPI; phai-ui scales them to the monitor.

    /// From a window's edge to the controls inside it.
    pub const MARGIN: i32 = 11;
    /// Between rows of related controls.
    pub const GAP: i32 = 7;
    /// A text box, a drop-down, a button, a label beside them.
    pub const ROW: i32 = 23;
    /// A check box or radio on its own line.
    pub const CHECK_ROW: i32 = 20;
    /// The narrowest a push button is.
    pub const BUTTON_WIDTH: i32 = 75;
    /// Between a label and its box.
    pub const LABEL_GAP: i32 = 12;
    /// Inside a group frame, below its title.
    pub const GROUP_TOP: i32 = 22;
    /// A status bar.
    pub const STATUS: i32 = 26;
    /// A toolbar: its 28-pixel buttons and 3 above and below.
    pub const TOOLBAR: i32 = 34;

    /// The width a label column needs for `longest` average characters of the system font - a
    /// first guess for a template; Phai Studio measures real words when it lays out.
    pub fn label_width(longest: usize) -> i32 {
        (longest as i32 * 7).max(60)
    }
}

pub mod templates {
    //! Forms ready made. Each is a [`FormFile`] - the same thing a `.phaiform` holds - so a
    //! template is edited in Phai Studio like any form once it is made.

    use super::metrics::*;
    use phai_form::{Control, FormFile, Kind, Menu, MenuItem, Tool};

    /// What Phai Studio lists under File > New from template.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Template {
        Dialog,
        MainWindow,
        DataEntry,
    }

    impl Template {
        pub const ALL: [Template; 3] = [Template::Dialog, Template::MainWindow, Template::DataEntry];

        /// The English name - the key for a program's Thai table.
        pub fn name(self) -> &'static str {
            match self {
                Template::Dialog => "Dialog with OK and Cancel",
                Template::MainWindow => "Main window with menu, toolbar and status bar",
                Template::DataEntry => "Form of labelled fields",
            }
        }

        /// The form, named `name`.
        pub fn make(self, name: &str) -> FormFile {
            match self {
                Template::Dialog => dialog(name),
                Template::MainWindow => main_window(name),
                Template::DataEntry => data_entry(name, &["Name", "Phone", "E-mail", "Address"]),
            }
        }
    }

    fn button(id: &str, text: &str, bounds: [i32; 4], primary: bool, handler: &str) -> Control {
        let mut c = Control::new(Kind::Button, id, bounds);
        c.text = Some(text.into());
        c.anchor = Some("right bottom".into());
        c.on_click = Some(handler.into());
        if primary {
            c.class = Some("primary".into());
        }
        c
    }

    /// OK and Cancel at the bottom right, OK the primary one - the row every dialog ends with.
    pub fn ok_cancel(size: [i32; 2]) -> Vec<Control> {
        let y = size[1] - MARGIN - ROW;
        let cancel_x = size[0] - MARGIN - BUTTON_WIDTH;
        let ok_x = cancel_x - GAP - BUTTON_WIDTH;
        vec![button("ok", "OK", [ok_x, y, BUTTON_WIDTH, ROW], true, "ok_clicked"), button("cancel", "Cancel", [cancel_x, y, BUTTON_WIDTH, ROW], false, "cancel_clicked")]
    }

    /// A question and its answer box above OK and Cancel.
    pub fn dialog(name: &str) -> FormFile {
        let mut f = FormFile::new(name);
        f.form.title = "Question".into();
        f.form.size = [360, 140];
        f.form.min_size = Some([300, 140]);
        let mut prompt = Control::new(Kind::Label, "prompt", [MARGIN, MARGIN, 360 - 2 * MARGIN, ROW]);
        prompt.text = Some("What should it be?".into());
        prompt.anchor = Some("left top right".into());
        let mut answer = Control::new(Kind::Edit, "answer", [MARGIN, MARGIN + ROW + GAP, 360 - 2 * MARGIN, ROW]);
        answer.anchor = Some("left top right".into());
        f.control = vec![prompt, answer];
        f.control.extend(ok_cancel(f.form.size));
        f
    }

    /// A program's main window: File and Help menus, a toolbar, a list filling it, a status bar.
    pub fn main_window(name: &str) -> FormFile {
        let mut f = FormFile::new(name);
        f.form.title = "My program".into();
        f.form.size = [640, 420];
        f.form.min_size = Some([400, 260]);
        let item = |id: &str, label: &str, key: Option<&str>| MenuItem { id: Some(id.into()), label: Some(label.into()), key: key.map(str::to_string), separator: false };
        f.menu = vec![
            Menu { label: "&File".into(), items: vec![item("new_item", "New", Some("Ctrl+N")), item("open_item", "Open...", Some("Ctrl+O")), item("save_item", "Save", Some("Ctrl+S")), MenuItem { separator: true, ..Default::default() }, item("exit", "Exit", None)] },
            Menu { label: "&Help".into(), items: vec![item("about", "About", None)] },
        ];
        f.toolbar = vec![
            Tool { id: "new_item".into(), label: "New".into(), glyph: "NEW".into() },
            Tool { id: "open_item".into(), label: "Open".into(), glyph: "OPEN".into() },
            Tool { id: "save_item".into(), label: "Save".into(), glyph: "SAVE".into() },
        ];
        let top = TOOLBAR;
        let mut list = Control::new(Kind::List, "items", [MARGIN, top + MARGIN, 640 - 2 * MARGIN, 420 - top - 2 * MARGIN - STATUS]);
        list.columns = vec![("Name".into(), 240), ("Changed".into(), 140)];
        list.anchor = Some("all".into());
        list.on_select = Some("item_chosen".into());
        let mut status = Control::new(Kind::Statusbar, "status", [0, 0, 0, STATUS]);
        status.text = Some("Ready".into());
        status.parts = vec![160];
        f.control = vec![list, status];
        f
    }

    /// One row per field - label, then a box that grows with the window - in a group, with OK and
    /// Cancel below. The labels share one column wide enough for the longest.
    pub fn data_entry(name: &str, fields: &[&str]) -> FormFile {
        let mut f = FormFile::new(name);
        f.form.title = "Details".into();
        let width = 460;
        let lw = label_width(fields.iter().map(|s| s.chars().count()).max().unwrap_or(4));
        let rows = fields.len() as i32;
        let group_h = GROUP_TOP + rows * ROW + (rows - 1).max(0) * GAP + MARGIN;
        let height = MARGIN + group_h + MARGIN + ROW + MARGIN;
        f.form.size = [width, height];
        f.form.min_size = Some([360, height]);
        let mut group = Control::new(Kind::Group, "details", [MARGIN, MARGIN, width - 2 * MARGIN, group_h]);
        group.text = Some("Details".into());
        group.anchor = Some("left top right".into());
        f.control.push(group);
        let box_x = MARGIN + LABEL_GAP + lw + LABEL_GAP;
        for (i, field) in fields.iter().enumerate() {
            let y = MARGIN + GROUP_TOP + i as i32 * (ROW + GAP);
            let id: String = field.to_ascii_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
            let mut label = Control::new(Kind::Label, &format!("{id}_label"), [MARGIN + LABEL_GAP, y, lw, ROW]);
            label.text = Some((*field).into());
            let mut edit = Control::new(Kind::Edit, &id, [box_x, y, width - MARGIN - LABEL_GAP - box_x, ROW]);
            edit.anchor = Some("left top right".into());
            f.control.push(label);
            f.control.push(edit);
        }
        f.control.extend(ok_cancel(f.form.size));
        f
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn overlap(a: [i32; 4], b: [i32; 4]) -> bool {
            a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
        }

        #[test]
        fn every_template_is_a_good_form_that_fits_its_window() {
            for t in Template::ALL {
                let f = t.make("Made");
                assert_eq!(f.check(), vec![], "{t:?}");
                let again = FormFile::parse(&f.to_toml()).unwrap();
                assert_eq!(again, f, "{t:?} saves and reads back");
                for c in f.control.iter().filter(|c| c.dock.is_none()) {
                    let [x, y, w, h] = c.rect();
                    assert!(x >= MARGIN && y >= MARGIN, "{t:?} {} keeps the margin", c.id);
                    assert!(x + w <= f.form.size[0] - MARGIN + 1 && y + h <= f.form.size[1] - MARGIN + 1 || c.anchor.as_deref() == Some("all"), "{t:?} {} inside the window", c.id);
                }
            }
        }

        #[test]
        fn ok_and_cancel_sit_at_the_bottom_right_a_gap_apart() {
            let row = ok_cancel([360, 140]);
            let (ok, cancel) = (row[0].rect(), row[1].rect());
            assert_eq!(cancel[0] + cancel[2], 360 - MARGIN);
            assert_eq!(cancel[1] + cancel[3], 140 - MARGIN);
            assert_eq!(cancel[0] - (ok[0] + ok[2]), GAP);
            assert_eq!(row[0].class.as_deref(), Some("primary"));
        }

        #[test]
        fn fields_do_not_overlap_and_share_a_label_column() {
            let f = data_entry("Entry", &["Name", "Phone", "Address"]);
            let boxes: Vec<[i32; 4]> = f.control.iter().filter(|c| c.kind != Kind::Group).map(|c| c.rect()).collect();
            for (i, a) in boxes.iter().enumerate() {
                for b in &boxes[i + 1..] {
                    assert!(!overlap(*a, *b), "{a:?} and {b:?}");
                }
            }
            let edits: Vec<i32> = f.control.iter().filter(|c| c.kind == Kind::Edit).map(|c| c.rect()[0]).collect();
            assert!(edits.windows(2).all(|w| w[0] == w[1]), "boxes line up: {edits:?}");
        }
    }
}
