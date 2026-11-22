// Written by docs/help-src/build_examples.py from the *_demo.phaiform files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

mod button_demo;
mod button_demo_ui;
mod calendar_demo;
mod calendar_demo_ui;
mod checkbox_demo;
mod checkbox_demo_ui;
mod checkedlist_demo;
mod checkedlist_demo_ui;
mod combo_demo;
mod combo_demo_ui;
mod date_demo;
mod date_demo_ui;
mod label_demo;
mod label_demo_ui;
mod link_demo;
mod link_demo_ui;
mod list_demo;
mod list_demo_ui;
mod listbox_demo;
mod listbox_demo_ui;
mod masked_demo;
mod masked_demo_ui;
mod notifyicon_demo;
mod notifyicon_demo_ui;
mod number_demo;
mod number_demo_ui;
mod picture_demo;
mod picture_demo_ui;
mod progress_demo;
mod progress_demo_ui;
mod radio_demo;
mod radio_demo_ui;

fn main() {
    match std::env::args().nth(1).as_deref().unwrap_or("") {
        "button" => button_demo_ui::ButtonDemoUi::run(button_demo::ButtonDemo::default()),
        "calendar" => calendar_demo_ui::CalendarDemoUi::run(calendar_demo::CalendarDemo::default()),
        "checkbox" => checkbox_demo_ui::CheckboxDemoUi::run(checkbox_demo::CheckboxDemo::default()),
        "checkedlist" => checkedlist_demo_ui::CheckedlistDemoUi::run(checkedlist_demo::CheckedlistDemo::default()),
        "combo" => combo_demo_ui::ComboDemoUi::run(combo_demo::ComboDemo::default()),
        "date" => date_demo_ui::DateDemoUi::run(date_demo::DateDemo::default()),
        "label" => label_demo_ui::LabelDemoUi::run(label_demo::LabelDemo::default()),
        "link" => link_demo_ui::LinkDemoUi::run(link_demo::LinkDemo::default()),
        "list" => list_demo_ui::ListDemoUi::run(list_demo::ListDemo::default()),
        "listbox" => listbox_demo_ui::ListboxDemoUi::run(listbox_demo::ListboxDemo::default()),
        "masked" => masked_demo_ui::MaskedDemoUi::run(masked_demo::MaskedDemo::default()),
        "notifyicon" => notifyicon_demo_ui::NotifyiconDemoUi::run(notifyicon_demo::NotifyiconDemo::default()),
        "number" => number_demo_ui::NumberDemoUi::run(number_demo::NumberDemo::default()),
        "picture" => picture_demo_ui::PictureDemoUi::run(picture_demo::PictureDemo::default()),
        "progress" => progress_demo_ui::ProgressDemoUi::run(progress_demo::ProgressDemo::default()),
        "radio" => radio_demo_ui::RadioDemoUi::run(radio_demo::RadioDemo::default()),
        other => eprintln!("no demo called {other:?}"),
    }
}
