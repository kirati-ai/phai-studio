//! TextBox (Multiline): several lines - added to from code, and counted.

use crate::textarea_demo_ui::{TextareaDemoEvents, TextareaDemoUi};

#[derive(Default)]
pub struct TextareaDemo {
    added: u32,
}

impl TextareaDemoEvents for TextareaDemo {
    fn loaded(&mut self, ui: &TextareaDemoUi) {
        ui.diary.set_text("Dear diary,\r\nToday I learned Rust.");
        self.diary_changed(ui);
    }

    // help: append
    fn add_line_clicked(&mut self, ui: &TextareaDemoUi) {
        self.added += 1;
        // A new line goes in after \r\n - the end of a line on Windows.
        ui.diary.append_text(&format!("\r\nLine {} added by the program.", self.added));
    }
    // help: end

    // help: lines
    fn diary_changed(&mut self, ui: &TextareaDemoUi) {
        let lines = ui.diary.lines();
        let words: usize = lines.iter().map(|l| l.split_whitespace().count()).sum();
        ui.count.set_text(&format!("{} lines, {words} words", lines.len()));
    }
    // help: end
}
