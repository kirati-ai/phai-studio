//! PrintDocument: a page drawn in code - seen in a preview, or printed.

use crate::printdocument_demo_ui::{PrintdocumentDemoEvents, PrintdocumentDemoUi};
use phai_ui::print::PrintPageEventArgs;

#[derive(Default)]
pub struct PrintdocumentDemo;

impl PrintdocumentDemoEvents for PrintdocumentDemo {
    // help: page
    fn certificate_print_page(&mut self, _ui: &PrintdocumentDemoUi, e: &PrintPageEventArgs) {
        // Drawn inside the margins; units are hundredths of an inch.
        let m = e.margin_bounds;
        e.set_color("#12855f");
        e.set_pen_width(3);
        e.draw_rect(m);
        e.set_font("Segoe UI", 36, true, false);
        e.draw_text("Certificate", m.x + 60, m.y + 60);
        e.set_color("black");
        e.set_font("Segoe UI", 18, false, false);
        e.draw_text("for finishing the Phai Studio lessons", m.x + 60, m.y + 160);
        e.set_has_more_pages(false); // one page
    }
    // help: end

    // help: buttons
    fn preview_clicked(&mut self, ui: &PrintdocumentDemoUi) {
        ui.certificate.print_preview();
    }

    fn print_clicked(&mut self, ui: &PrintdocumentDemoUi) {
        ui.certificate.print_dialog(); // Windows' Print dialog, then the pages
    }
    // help: end
}
