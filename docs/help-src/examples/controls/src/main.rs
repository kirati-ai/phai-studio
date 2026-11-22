// Written by docs/help-src/build_examples.py from the *_demo.phaiform files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

mod button_demo;
mod button_demo_ui;
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

fn main() {
    match std::env::args().nth(1).as_deref().unwrap_or("") {
        "button" => button_demo_ui::ButtonDemoUi::run(button_demo::ButtonDemo::default()),
        "checkbox" => checkbox_demo_ui::CheckboxDemoUi::run(checkbox_demo::CheckboxDemo::default()),
        "checkedlist" => checkedlist_demo_ui::CheckedlistDemoUi::run(checkedlist_demo::CheckedlistDemo::default()),
        "combo" => combo_demo_ui::ComboDemoUi::run(combo_demo::ComboDemo::default()),
        "date" => date_demo_ui::DateDemoUi::run(date_demo::DateDemo::default()),
        "label" => label_demo_ui::LabelDemoUi::run(label_demo::LabelDemo::default()),
        other => eprintln!("no demo called {other:?}"),
    }
}
