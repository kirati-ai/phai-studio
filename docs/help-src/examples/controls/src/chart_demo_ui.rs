// Made by Phai Studio from chart_demo.phf - do not change this file; change the form.
// Everything here is written again on every save.

#![allow(dead_code, unused_imports, unused_variables, clippy::all)]

use phai_ui::controls;
use phai_ui::layout::{scale, unscale, Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::t;
use phai_ui::theme;
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Every English sentence this window shows - for the project's Thai table and its test.
pub const TEXTS: &[&str] = &[
    "Chart",
    "Sales by month",
    "Month, Sales, Cost",
    "Jan, 120, 80",
    "Feb, 150, 95",
    "Mar, 135, 90",
    "Apr, 180, 110",
    "May, 210, 120",
    "Jun, 195, 130",
    "Live price",
    "Time, Price",
    "Columns",
    "Lines",
    "Pie",
    "Stop",
];

const ID_COLUMN_CLICKED: i32 = 1000;
const ID_LINE_CLICKED: i32 = 1001;
const ID_PIE_CLICKED: i32 = 1002;
const ID_START_STOP_CLICKED: i32 = 1003;
const ID_CLOCK_TICKED: i32 = 1004;
const ID_TOOLBAR: i32 = 1999;
const CONTROL_SALES: i32 = 2000;
const CONTROL_PRICE: i32 = 2001;
const CONTROL_COLUMN: i32 = 2002;
const CONTROL_LINE: i32 = 2003;
const CONTROL_PIE: i32 = 2004;
const CONTROL_START_STOP: i32 = 2005;
const CONTROL_CLOCK: i32 = 2006;

/// The window and every control in it, by the ids the form gives them.
pub struct ChartDemoUi {
    pub hwnd: HWND,
    pub font: HFONT,
    /// The Chart `sales` on this window: `ui.sales.` lists what it can do.
    /// ไทย: Chart ชื่อ sales บนหน้าต่างนี้ - พิมพ์ ui.sales. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub sales: phai_ui::ctl::Chart,
    /// The Chart `price` on this window: `ui.price.` lists what it can do.
    /// ไทย: Chart ชื่อ price บนหน้าต่างนี้ - พิมพ์ ui.price. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub price: phai_ui::ctl::Chart,
    /// The Button `column` on this window: `ui.column.` lists what it can do.
    /// ไทย: Button ชื่อ column บนหน้าต่างนี้ - พิมพ์ ui.column. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub column: phai_ui::ctl::Button,
    /// The Button `line` on this window: `ui.line.` lists what it can do.
    /// ไทย: Button ชื่อ line บนหน้าต่างนี้ - พิมพ์ ui.line. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub line: phai_ui::ctl::Button,
    /// The Button `pie` on this window: `ui.pie.` lists what it can do.
    /// ไทย: Button ชื่อ pie บนหน้าต่างนี้ - พิมพ์ ui.pie. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub pie: phai_ui::ctl::Button,
    /// The Button `start_stop` on this window: `ui.start_stop.` lists what it can do.
    /// ไทย: Button ชื่อ start_stop บนหน้าต่างนี้ - พิมพ์ ui.start_stop. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub start_stop: phai_ui::ctl::Button,
    /// The Timer `clock` on this window: `ui.clock.` lists what it can do.
    /// ไทย: Timer ชื่อ clock บนหน้าต่างนี้ - พิมพ์ ui.clock. แล้วเลือกได้ว่ามันทำอะไรได้บ้าง
    pub clock: phai_ui::ctl::Timer,
    layout: Layout,
}

/// What the window tells the program. Every method does nothing until the program says otherwise.
pub trait ChartDemoEvents {
    /// The window is made and about to show.
    fn loaded(&mut self, ui: &ChartDemoUi) {}
    fn column_clicked(&mut self, ui: &ChartDemoUi) {}
    fn line_clicked(&mut self, ui: &ChartDemoUi) {}
    fn pie_clicked(&mut self, ui: &ChartDemoUi) {}
    fn start_stop_clicked(&mut self, ui: &ChartDemoUi) {}
    fn clock_ticked(&mut self, ui: &ChartDemoUi) {}
}

struct Host<E> {
    ui: ChartDemoUi,
    events: E,
    /// The program's main window: the program ends with it.
    main: bool,
}

impl ChartDemoUi {
    /// Makes the window and runs it until it is closed.
    pub fn run<E: ChartDemoEvents + 'static>(events: E) {
        let hwnd = Self::create(events);
        win::run_loop(Some(hwnd));
    }

    /// Makes the window and shows it; the caller runs the message loop.
    pub fn create<E: ChartDemoEvents + 'static>(events: E) -> HWND {
        Self::make(None, events)
    }

    /// Opens this window inside `parent`, an MDI container (`is_mdi_container`), as WinForms'
    /// `MdiParent` and `Show()`; it closes with it.
    pub fn show_mdi_child<E: ChartDemoEvents + 'static>(parent: HWND, events: E) -> HWND {
        phai_ui::mdi::set_next_parent(parent);
        Self::make(Some(parent), events)
    }

    /// Shows this window beside `owner` (another window of the program) and returns at once,
    /// as `Show()` in WinForms. Closing it leaves the program running.
    pub fn show<E: ChartDemoEvents + 'static>(owner: HWND, events: E) -> HWND {
        Self::make(Some(owner), events)
    }

    /// Shows this window over `owner` and waits until it is closed, `owner` taking no input
    /// meanwhile - as `ShowDialog()` in WinForms. What it answers: the DialogResult of the
    /// button that closed it, or `Cancel`.
    pub fn show_dialog<E: ChartDemoEvents + 'static>(owner: HWND, events: E) -> phai_ui::form::DialogResult {
        let hwnd = Self::make(Some(owner), events);
        win::modal_loop(hwnd, owner);
        phai_ui::form::take_result(hwnd)
    }

    /// As [`Self::show_dialog`], `events` given back when the window has closed - to read what
    /// was typed in it, as a WinForms form's fields after `ShowDialog()`:
    /// `if let (DialogResult::Ok, Some(f)) = Ui::show_dialog_for(owner, f) { f.answer }`.
    pub fn show_dialog_for<E: ChartDemoEvents + 'static>(owner: HWND, events: E) -> (phai_ui::form::DialogResult, Option<E>) {
        let hwnd = Self::make(Some(owner), events);
        phai_ui::form::want_back(hwnd);
        win::modal_loop(hwnd, owner);
        (phai_ui::form::take_result(hwnd), phai_ui::form::take_kept::<E>(hwnd))
    }

    fn make<E: ChartDemoEvents + 'static>(owner: Option<HWND>, events: E) -> HWND {
        win::init();
        let class = win::register("Phai.ChartDemo", Some(proc_::<E>), None);
        let title = wide(t("Chart"));
        let (style, ex) = phai_ui::form::frame("sizable", true, true, true, false);
        let main = owner.is_none();
        let over = owner;
        unsafe {
            // Inside an MDI container when show_mdi_child asked; a window of its own otherwise.
            let mdi_frame = phai_ui::mdi::take_next();
            let hwnd = match mdi_frame {
                Some(frame) => phai_ui::mdi::create_child(frame, &class, &title, style),
                None => wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, owner, None, Some(win::instance()), None).unwrap_or_default(),
            };
            let ui = Self::build(hwnd);
            let mut host = Box::new(Host { ui, events, main });
            host.events.loaded(&host.ui);
            win::set_state(hwnd, host);
            win::theme_frame(hwnd);
            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
            let ex = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
            let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 660), bottom: win::px(hwnd, 340) };
            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(ex), win::dpi(hwnd) as u32);
            let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
            if mdi_frame.is_some() {
                // Windows places a child form inside its container.
            } else if let Some(o) = over {
                win::center_on(hwnd, o);
            } else {
                win::center_on_screen(hwnd);
            }
            if mdi_frame.is_some() {
                win::show(hwnd, true);
            } else {
                win::show_top(hwnd);
            }
            // F12 in a debug build: the Live Inspector, which saves into the form.
            #[cfg(debug_assertions)]
            phai_ui::inspect::attach(hwnd, std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/chart_demo.phf")));
            hwnd
        }
    }

    fn build(hwnd: HWND) -> ChartDemoUi {
        let font = win::font(hwnd, 0, false);
        let dpi = win::dpi(hwnd);
        #[allow(unused_mut)]
        let mut layout = Layout::new((660, 340));
        let sales = { let h = phai_ui::chart::chart(hwnd, CONTROL_SALES, font); phai_ui::chart::set_title(h, t("Sales by month")); phai_ui::chart::set_data_text(h, "Month, Sales, Cost\nJan, 120, 80\nFeb, 150, 95\nMar, 135, 90\nApr, 180, 110\nMay, 210, 120\nJun, 195, 130"); h };
        win::set_css(sales, "", "sales");
        layout.add(sales, Item { bounds: Rect::new(16, 16, 306, 250), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let price = { let h = phai_ui::chart::chart(hwnd, CONTROL_PRICE, font); phai_ui::chart::set_title(h, t("Live price")); phai_ui::chart::set_data_text(h, "Time, Price"); h };
        win::set_css(price, "", "price");
        phai_ui::props::chart_type(price, "line");
        phai_ui::props::max_points(price, 30);
        layout.add(price, Item { bounds: Rect::new(338, 16, 306, 250), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let column = win::button(hwnd, t("Columns"), CONTROL_COLUMN, font, false);
        win::set_css(column, "", "column");
        layout.add(column, Item { bounds: Rect::new(16, 280, 96, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let line = win::button(hwnd, t("Lines"), CONTROL_LINE, font, false);
        win::set_css(line, "", "line");
        layout.add(line, Item { bounds: Rect::new(121, 280, 96, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let pie = win::button(hwnd, t("Pie"), CONTROL_PIE, font, false);
        win::set_css(pie, "", "pie");
        layout.add(pie, Item { bounds: Rect::new(226, 280, 96, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let start_stop = win::button(hwnd, t("Stop"), CONTROL_START_STOP, font, false);
        win::set_css(start_stop, "", "start_stop");
        layout.add(start_stop, Item { bounds: Rect::new(338, 280, 120, 30), dock: Dock::None, anchor: Anchor { left: true, top: true, right: false, bottom: false } });
        let clock = phai_ui::ctl::Timer::new(hwnd, CONTROL_CLOCK, 500, true);
        ChartDemoUi {
            hwnd,
            font,
            sales: phai_ui::ctl::Chart(sales),
            price: phai_ui::ctl::Chart(price),
            column: phai_ui::ctl::Button(column),
            line: phai_ui::ctl::Button(line),
            pie: phai_ui::ctl::Button(pie),
            start_stop: phai_ui::ctl::Button(start_stop),
            clock,
            layout,
        }
    }

    /// Closes the window, as its close box would - its FormClosing comes first.
    ///
    /// ไทย: ปิดหน้าต่าง เหมือนกดปุ่มปิด - FormClosing ทำงานก่อน
    /// Example: `ui.close();`
    pub fn close(&self) {
        unsafe {
            let _ = wm::PostMessageW(Some(self.hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    /// Hides the window; the program keeps running and `ui.show_window()` brings it back.
    ///
    /// ไทย: ซ่อนหน้าต่าง โปรแกรมยังทำงานอยู่ ui.show_window() เรียกกลับมาได้
    /// Example: `ui.hide();`
    pub fn hide(&self) {
        win::show(self.hwnd, false);
    }

    /// Shows the window again after `ui.hide()`.
    ///
    /// ไทย: แสดงหน้าต่างอีกครั้งหลัง ui.hide()
    /// Example: `ui.show_window();`
    pub fn show_window(&self) {
        win::show(self.hwnd, true);
    }

    /// Brings the window to the front, the keyboard on it.
    ///
    /// ไทย: ยกหน้าต่างขึ้นมาข้างหน้า ให้แป้นพิมพ์อยู่ที่มัน
    /// Example: `ui.activate();`
    pub fn activate(&self) {
        unsafe {
            let _ = wm::SetForegroundWindow(self.hwnd);
        }
    }

    /// A message box with the buttons and the icon you choose, its title `caption`; the button
    /// pressed comes back - DialogResult::Yes, No, Ok, Cancel... For a plain notice `ui.alert` is
    /// shorter, for Yes or No `ui.confirm`.
    ///
    /// ไทย: กล่องข้อความที่เลือกปุ่มและไอคอนเองได้ หัวเรื่องคือ caption ได้ปุ่มที่ถูกกดกลับมา (DialogResult::Yes, No, Ok, Cancel...) - แค่แจ้งเฉย ๆ ใช้ ui.alert สั้นกว่า ถามใช่/ไม่ใช่ใช้ ui.confirm
    /// Example: `if ui.message_box("Delete it?", "Files", MessageBoxButtons::YesNo, MessageBoxIcon::Question) == DialogResult::Yes { ... }`
    pub fn message_box(&self, text: &str, caption: &str, buttons: phai_ui::form::MessageBoxButtons, icon: phai_ui::form::MessageBoxIcon) -> phai_ui::form::DialogResult {
        phai_ui::form::message_box(self.hwnd, text, caption, buttons, icon)
    }

    /// Shows a message with an OK button and waits until it is pressed; the title is the
    /// window's - as JavaScript's alert.
    ///
    /// ไทย: แสดงข้อความพร้อมปุ่ม OK แล้วรอจนกด หัวเรื่องเป็นชื่อหน้าต่าง - เหมือน alert ของ JavaScript
    /// Example: `ui.alert("Saved");`
    pub fn alert(&self, text: &str) {
        phai_ui::form::message_box(self.hwnd, text, &self.title(), phai_ui::form::MessageBoxButtons::Ok, phai_ui::form::MessageBoxIcon::Information);
    }

    /// Asks a question with Yes and No buttons: `true` when Yes is pressed - as JavaScript's
    /// confirm.
    ///
    /// ไทย: ถามคำถามที่มีปุ่ม Yes กับ No ได้ true เมื่อกด Yes - เหมือน confirm ของ JavaScript
    /// Example: `if ui.confirm("Delete it?") { ... }`
    pub fn confirm(&self, text: &str) -> bool {
        phai_ui::form::message_box(self.hwnd, text, &self.title(), phai_ui::form::MessageBoxButtons::YesNo, phai_ui::form::MessageBoxIcon::Question) == phai_ui::form::DialogResult::Yes
    }

    /// Asks for words in a small window with a box to type in: what was typed, or `None` when
    /// Cancel is pressed; `default` is in the box at first - as JavaScript's prompt.
    ///
    /// ไทย: ถามให้พิมพ์ในหน้าต่างเล็ก ได้สิ่งที่พิมพ์ หรือ None เมื่อกด Cancel - default คือคำที่ใส่ไว้ให้ก่อน เหมือน prompt ของ JavaScript
    /// Example: `if let Some(name) = ui.prompt("Your name?", "") { ui.alert(&name); }`
    pub fn prompt(&self, prompt: &str, default: &str) -> Option<String> {
        phai_ui::dialogs::ask_fields(Some(self.hwnd), &self.title(), &[phai_ui::dialogs::Field::Text(prompt, default)]).and_then(|a| a.into_iter().next())
    }

    /// Lets the user choose a file to open: its path, or `None` when Cancel is pressed. The
    /// filter is name|pattern - empty for every file.
    ///
    /// ไทย: ให้ผู้ใช้เลือกไฟล์ที่จะเปิด ได้ที่อยู่ไฟล์ หรือ None เมื่อกด Cancel - filter เขียนแบบ ชื่อ|รูปแบบ ว่างไว้คือทุกไฟล์
    /// Example: `if let Some(path) = ui.open_file("Text files|*.txt") { ... }`
    pub fn open_file(&self, filter: &str) -> Option<std::path::PathBuf> {
        phai_ui::components::OpenFileDialog::new(self.hwnd, "", filter, "", "", false).show_dialog()
    }

    /// Lets the user choose where to save a file: its path, or `None` when Cancel is pressed;
    /// `name` is the file name offered at first.
    ///
    /// ไทย: ให้ผู้ใช้เลือกว่าจะบันทึกไฟล์ที่ไหน ได้ที่อยู่ไฟล์ หรือ None เมื่อกด Cancel - name คือชื่อไฟล์ที่เสนอให้ก่อน
    /// Example: `if let Some(path) = ui.save_file("Text files|*.txt", "notes.txt") { ... }`
    pub fn save_file(&self, filter: &str, name: &str) -> Option<std::path::PathBuf> {
        phai_ui::components::SaveFileDialog::new(self.hwnd, "", filter, "", "").show_dialog_named(name)
    }

    /// Lets the user choose a folder: its path, or `None` when Cancel is pressed.
    ///
    /// ไทย: ให้ผู้ใช้เลือกโฟลเดอร์ ได้ที่อยู่โฟลเดอร์ หรือ None เมื่อกด Cancel
    /// Example: `if let Some(folder) = ui.choose_folder() { ... }`
    pub fn choose_folder(&self) -> Option<std::path::PathBuf> {
        phai_ui::components::FolderBrowserDialog::new(self.hwnd, "", "").show_dialog()
    }

    /// Moves the window to the middle of the screen.
    ///
    /// ไทย: ย้ายหน้าต่างไปกลางจอ
    /// Example: `ui.center_to_screen();`
    pub fn center_to_screen(&self) {
        win::center_on_screen(self.hwnd);
    }

    /// The words in the window's title bar.
    ///
    /// ไทย: ข้อความบนแถบชื่อหน้าต่าง
    /// Example: `let t = ui.title();`
    pub fn title(&self) -> String {
        win::text_of(self.hwnd)
    }

    /// Changes the words in the window's title bar.
    ///
    /// ไทย: เปลี่ยนข้อความบนแถบชื่อหน้าต่าง
    /// Example: `ui.set_title("Score book - 2026");`
    pub fn set_title(&self, s: &str) {
        win::set_text(self.hwnd, s);
    }

    /// A handle other threads use to run work on this window's thread - [`Self::begin_invoke`].
    pub fn invoker(&self) -> phai_ui::invoke::Invoker {
        phai_ui::invoke::Invoker::new(self.hwnd)
    }

    /// From any thread: `f` runs on the window's thread with the program's ChartDemo and the
    /// controls, soon; returns at once - WinForms' BeginInvoke. `false` when the window is gone.
    pub fn begin_invoke<E: ChartDemoEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &ChartDemoUi) + Send + 'static) -> bool {
        inv.post(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<ChartDemoUi>()) {
                f(e, ui);
            }
        }))
    }

    /// As [`Self::begin_invoke`], waiting until `f` has run - WinForms' Invoke.
    pub fn invoke<E: ChartDemoEvents + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &ChartDemoUi) + Send + 'static) {
        inv.send(Box::new(move |e, ui| {
            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<ChartDemoUi>()) {
                f(e, ui);
            }
        }))
    }

    /// Every word again in the language now chosen - after `phai_ui::text::set_thai`.
    pub fn relabel(&self) {
        win::set_text(self.hwnd, t("Chart"));
        win::set_text(self.column, t("Columns"));
        win::set_text(self.line, t("Lines"));
        win::set_text(self.pie, t("Pie"));
        win::set_text(self.start_stop, t("Stop"));
    }
}

extern "system" fn proc_<E: ChartDemoEvents + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(host) = (unsafe { win::state::<Host<E>>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            host.ui.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let (id, code) = (win::loword(w.0) as i32, win::hiword(w.0));
            let Host { ui, events, .. } = host;
            match id {
                CONTROL_COLUMN if code == 0 => events.column_clicked(ui),
                CONTROL_LINE if code == 0 => events.line_clicked(ui),
                CONTROL_PIE if code == 0 => events.pie_clicked(ui),
                CONTROL_START_STOP if code == 0 => events.start_stop_clicked(ui),
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_TIMER => {
            let Host { ui, events, .. } = host;
            match w.0 as i32 {
                CONTROL_CLOCK => events.clock_ticked(ui),
                _ => {}
            }
            LRESULT(0)
        }
        phai_ui::invoke::WM_INVOKE => {
            // Work handed over from another thread (begin_invoke, invoke).
            LRESULT(unsafe { phai_ui::invoke::run(l, &mut host.events, &host.ui) })
        }
        0x02E0 => {
            // WM_DPICHANGED: the window, its font and its icons at the new scale.
            let r = unsafe { &*(l.0 as *const RECT) };
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
                let _ = Gdi::DeleteObject(host.ui.font.into());
            }
            host.ui.font = win::font(hwnd, 0, false);
            win::set_font_all(hwnd, host.ui.font);
            win::retheme(hwnd);
            host.ui.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            // The program ends with its main window; a window shown over another only closes.
            if host.main {
                unsafe { wm::PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(host) = unsafe { win::take_state::<Host<E>>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(host.ui.font.into());
                }
                // Given back to show_dialog_for, when it asked.
                phai_ui::form::keep(hwnd, Box::new(host.events));
            }
            win::def(hwnd, msg, w, l)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}
