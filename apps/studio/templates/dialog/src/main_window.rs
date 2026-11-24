//! MainWindow: a small unit converter - type a length, choose two units, press Convert.
//! Phai Studio made this file once; it is yours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

/// Each unit in the drop-downs, in the same order, as how many metres one of it is.
const METRES: [f64; 8] = [
    0.001,    // Millimetres
    0.01,     // Centimetres
    1.0,      // Metres
    1000.0,   // Kilometres
    0.0254,   // Inches
    0.3048,   // Feet
    0.9144,   // Yards
    1609.344, // Miles
];

#[derive(Default)]
pub struct MainWindow {}

impl MainWindowEvents for MainWindow {
    /// The window is made: show the first answer straight away.
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.convert_clicked(ui);
    }

    /// Convert (or Enter, the form's accept button): the length in the other unit.
    fn convert_clicked(&mut self, ui: &MainWindowUi) {
        // A comma between thousands is fine: "1,250.5".
        let typed = ui.amount.text().trim().replace(',', "");
        let Ok(amount) = typed.parse::<f64>() else {
            ui.result.set_text(t("Please type a number."));
            ui.amount.focus();
            return;
        };
        let (Some(from), Some(to)) = (ui.from_unit.selected_index(), ui.to_unit.selected_index()) else {
            ui.result.set_text(t("Please choose both units."));
            return;
        };
        // To metres first, then from metres to the unit wanted. The drop-downs hold the units'
        // names in the language shown already.
        let answer = amount * METRES[from] / METRES[to];
        let line = tf("{0} {1} = {2} {3}", &[&short(amount), &ui.from_unit.item(from), &short(answer), &ui.to_unit.item(to)]);
        ui.result.set_text(&line);
    }
}

/// A number with no more than 6 digits after the point and no zeros at its end: 3.28084, 1000.
fn short(x: f64) -> String {
    let s = format!("{x:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
