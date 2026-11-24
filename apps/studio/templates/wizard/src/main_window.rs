//! MainWindow: a step-by-step wizard. Its pages are the pages of a tab control whose tabs are
//! hidden (`visible = false` on `pages`); Back and Next go from one to the next, and Next stays
//! greyed until the page is filled in. Finish shows what was chosen.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

/// The pages, in order - their place in the form's `tabs`.
const WELCOME: usize = 0;
const OPTIONS: usize = 1;
const DETAILS: usize = 2;
const SUMMARY: usize = 3;
const PAGES: usize = 4;

#[derive(Default)]
pub struct MainWindow {
    /// Finish was clicked: the window closes without asking.
    finished: bool,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.page_changed(ui);
    }

    /// Another page is showing (Back or Next): its heading, the step, and which buttons work.
    fn page_changed(&mut self, ui: &MainWindowUi) {
        let page = ui.pages.selected_index();
        let (heading, subheading) = match page {
            WELCOME => (t("Welcome"), t("This wizard takes you through a few short steps.")),
            OPTIONS => (t("Choose options"), t("Choose how the program will be used.")),
            DETAILS => (t("Enter your details"), t("Who the program is for.")),
            _ => (t("Ready to finish"), t("Check your answers, then click Finish.")),
        };
        ui.heading.set_text(heading);
        ui.subheading.set_text(subheading);
        ui.step.set_text(&tf("Step {0} of {1}", &[&(page + 1), &PAGES]));
        if page == SUMMARY {
            // The answers, a row each.
            ui.summary.items_clear();
            for (question, answer) in self.answers(ui) {
                ui.summary.items_add(&[question, &answer]);
            }
        }
        ui.back.set_enabled(page > WELCOME);
        ui.finish.set_enabled(page == SUMMARY);
        self.check_page(ui);
    }

    // The pages' controls: each change asks again whether Next may be clicked.
    fn usage_clicked(&mut self, ui: &MainWindowUi) {
        self.check_page(ui);
    }

    fn details_changed(&mut self, ui: &MainWindowUi) {
        self.check_page(ui);
    }

    fn back_clicked(&mut self, ui: &MainWindowUi) {
        let page = ui.pages.selected_index();
        if page > WELCOME {
            // As a click on the hidden tab: its page shows, and page_changed runs.
            ui.pages.select_tab(page - 1);
        }
    }

    fn next_clicked(&mut self, ui: &MainWindowUi) {
        let page = ui.pages.selected_index();
        if page + 1 < PAGES && self.page_filled_in(ui, page) {
            ui.pages.select_tab(page + 1);
        }
    }

    fn finish_clicked(&mut self, ui: &MainWindowUi) {
        // Here the program would do what the wizard was for; this one shows the answers.
        let lines: Vec<String> = self.answers(ui).iter().map(|(q, a)| format!("{q}: {a}")).collect();
        ui.alert(&format!("{}\r\n\r\n{}", t("All done. Your answers:"), lines.join("\r\n")));
        self.finished = true;
        ui.close();
    }

    fn cancel_clicked(&mut self, ui: &MainWindowUi) {
        ui.close();
    }

    /// Closing before Finish (Cancel, Esc or the close box) asks first.
    fn closing(&mut self, ui: &MainWindowUi, e: &ClosingEventArgs) {
        if !self.finished && !ui.confirm(t("Leave the wizard? Your answers will not be kept.")) {
            e.cancel();
        }
    }
}

impl MainWindow {
    /// Next works only when the page showing has what it needs.
    fn check_page(&self, ui: &MainWindowUi) {
        let page = ui.pages.selected_index();
        ui.next.set_enabled(page + 1 < PAGES && self.page_filled_in(ui, page));
    }

    /// Whether `page` has what it needs: a choice on Options, a name and an e-mail on Details.
    fn page_filled_in(&self, ui: &MainWindowUi, page: usize) -> bool {
        match page {
            OPTIONS => ui.just_me.checked() || ui.my_team.checked() || ui.my_school.checked(),
            DETAILS => !ui.name.text().trim().is_empty() && ui.email.text().contains('@'),
            _ => true,
        }
    }

    /// Every question with its answer.
    fn answers(&self, ui: &MainWindowUi) -> Vec<(&'static str, String)> {
        let usage = if ui.my_team.checked() {
            t("My team at work")
        } else if ui.my_school.checked() {
            t("A class at school")
        } else {
            t("Just me")
        };
        let yes_no = |on: bool| if on { t("Yes") } else { t("No") }.to_string();
        vec![
            (t("Used by"), usage.to_string()),
            (t("Shortcut on the desktop"), yes_no(ui.shortcut.checked())),
            (t("Look for updates"), yes_no(ui.updates.checked())),
            (t("Name"), ui.name.text().trim().to_string()),
            (t("E-mail"), ui.email.text().trim().to_string()),
        ]
    }
}
