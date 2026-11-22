//! RichTextBox: a long text - a word found in it, and it saved to a file.

use crate::richtext_demo_ui::{RichtextDemoEvents, RichtextDemoUi};

#[derive(Default)]
pub struct RichtextDemo {
    from: usize,
}

impl RichtextDemoEvents for RichtextDemo {
    fn loaded(&mut self, ui: &RichtextDemoUi) {
        ui.story.set_text("The bamboo grew by the river. Every morning the bamboo bent to the water, and every evening it stood up again.");
    }

    // help: find
    fn find_clicked(&mut self, ui: &RichtextDemoUi) {
        // find() chooses the word it finds; the next click goes on from after it.
        match ui.story.find(&ui.word.text(), self.from) {
            Some(at) => {
                self.from = at + 1;
                ui.status.set_text(&format!("Found at letter {at}"));
            }
            None => {
                self.from = 0;
                ui.status.set_text("Not found - the next Find starts from the top");
            }
        }
        ui.story.focus();
    }
    // help: end

    // help: save
    fn save_clicked(&mut self, ui: &RichtextDemoUi) {
        let path = std::env::temp_dir().join("story.txt");
        match ui.story.save_file(&path) {
            Ok(()) => ui.status.set_text(&format!("Saved to {}", path.display())),
            Err(e) => ui.status.set_text(&format!("Could not save: {e}")),
        }
    }
    // help: end

    // help: lines
    fn story_changed(&mut self, ui: &RichtextDemoUi) {
        // lines(): every text box has it - each line of what it holds.
        ui.status.set_text(&format!("{} lines", ui.story.lines().len()));
    }
    // help: end
}
