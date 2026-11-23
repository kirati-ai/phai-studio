BODY = {}

BODY['code.html'] = r'''
<h1>เขียนโค้ดกับ control</h1>
<p class="lead">ฟอร์มที่ออกแบบได้ struct ชื่อ <code>&lt;ฟอร์ม&gt;Ui</code> ที่มี control ทุกตัวตาม Id และ trait <code>&lt;ฟอร์ม&gt;Events</code> ที่มีเมธอดสำหรับทุกเหตุการณ์ โค้ดของคุณอยู่ในไฟล์ <code>&lt;ฟอร์ม&gt;.rs</code> ไฟล์เดียว ไฟล์ <code>_ui.rs</code> สร้างใหม่ทุกครั้งที่บันทึก ห้ามแก้</p>

<h2>Properties, Methods, Events</h2>
<p>control แต่ละตัวเป็น object แบบ WinForms มีสามอย่าง</p>
<table>
  <tr><th>อย่าง</th><th>คืออะไร</th><th>ตั้งที่ไหน</th></tr>
  <tr><td>Properties</td><td>ลักษณะ: ข้อความ สี ตำแหน่ง ใช้งานได้ไหม มองเห็นไหม</td><td>หน้าต่าง Properties (⚙) เก็บในไฟล์ <code>.phf</code></td></tr>
  <tr><td>Methods</td><td>สั่งให้ทำ: <code>ui.name.set_text("...")</code>, <code>ui.ok.focus()</code></td><td>ในโค้ด</td></tr>
  <tr><td>Events</td><td>บอกเมื่อเกิดขึ้น: คลิก พิมพ์ กดปุ่ม เลือกแถว</td><td>แท็บ Events (⚡) ดับเบิลคลิกชื่อเหตุการณ์ได้เมธอดว่างพร้อมพารามิเตอร์</td></tr>
</table>

<h2>ชนิดของ control ในโค้ด</h2>
<p>field ใน <code>Ui</code> เป็นชนิดจาก <code>phai_ui::ctl</code> ตรงกับชื่อใน WinForms ช่องเลือก object บนหน้าต่าง Properties บอกชนิดไว้ เช่น <code>name  phai_ui::ctl::TextBox</code></p>
<pre><code>impl MainWindowEvents for MainWindow {
    fn save_clicked(&amp;mut self, ui: &amp;MainWindowUi) {
        if ui.name.text().trim().is_empty() {
            ui.name.focus();
            return;
        }
        ui.orders.items_add(&amp;[&amp;ui.name.text(), &amp;ui.qty.value().to_string()]);
        ui.status.set_part_text(0, &amp;tf("{0} orders", &amp;[&amp;ui.orders.items_count()]));
        ui.save.set_enabled(false);
    }
}</code></pre>
<table>
  <tr><th>ชนิด</th><th>เมธอดที่ใช้บ่อย</th></tr>
  <tr><td>ทุกตัว (Control)</td><td><code>text set_text enabled set_enabled visible set_visible focus set_tooltip hwnd</code></td></tr>
  <tr><td>TextBox</td><td><code>clear append_text select_all set_read_only</code></td></tr>
  <tr><td>CheckBox, RadioButton</td><td><code>checked set_checked</code></td></tr>
  <tr><td>ComboBox, ListBox</td><td><code>items_add items_clear selected_index set_selected_index selected_item</code></td></tr>
  <tr><td>ListView</td><td><code>items_add items_clear items_count selected_index select ensure_visible</code></td></tr>
  <tr><td>Grid</td><td><code>rows fill cell set_cell sort_by find set_column_filter export_xlsx export_pdf set_master</code></td></tr>
  <tr><td>TreeView</td><td><code>nodes_add nodes_add_under set_nodes selected_text expand_all</code></td></tr>
  <tr><td>NumericUpDown, TrackBar, ProgressBar</td><td><code>value set_value</code></td></tr>
  <tr><td>Timer</td><td><code>start stop enabled set_enabled interval set_interval</code></td></tr>
</table>
<p>ยังเรียกฟังก์ชันของ Win32 ได้ตรง ๆ ด้วย <code>ui.name.hwnd()</code></p>

<h2>เหตุการณ์ที่มีข้อมูลมาด้วย</h2>
<p>บางเหตุการณ์ส่ง args มาด้วย เหมือน <code>e</code> ใน C#</p>
<pre><code>fn name_key_down(&amp;mut self, ui: &amp;MainWindowUi, e: &amp;phai_ui::events::KeyEventArgs) {
    if e.key == 0x0D {             // Enter
        e.set_handled();           // ไม่ต้องให้ช่องกรอกจัดการต่อ
        self.save_clicked(ui);
    }
}

fn main_window_form_closing(&amp;mut self, ui: &amp;MainWindowUi, e: &amp;phai_ui::events::ClosingEventArgs) {
    if self.changed {
        ui.status.set_part_text(0, "บันทึกก่อนปิด");
        e.cancel();                // ยังไม่ปิด
    }
}</code></pre>
<p>ชนิด args: <code>KeyEventArgs</code>, <code>KeyPressEventArgs</code>, <code>MouseEventArgs</code>, <code>ClosingEventArgs</code>, <code>ItemEventArgs</code>, <code>FileDropEventArgs</code></p>

<h2>งานที่ใช้เวลานาน: thread กับ invoke</h2>
<p>อย่าทำงานนาน ๆ (อ่านไฟล์ใหญ่ ต่อเน็ต คิวรีฐานข้อมูล) ในเมธอดของเหตุการณ์ หน้าต่างจะค้าง ให้ทำบน thread อื่น แล้วส่งผลกลับมาแก้หน้าจอด้วย <code>invoke</code> เพราะ control ของ Windows แก้ได้เฉพาะจาก thread ของหน้าต่าง</p>
<pre><code>fn load_clicked(&amp;mut self, ui: &amp;MainWindowUi) {
    let invoker = ui.invoker();                    // ส่งข้าม thread ได้
    std::thread::spawn(move || {
        let rows = read_big_file();                // งานหนัก บน thread อื่น
        MainWindowUi::begin_invoke(invoker, move |me: &amp;mut MainWindow, ui: &amp;MainWindowUi| {
            ui.list.fill(&amp;rows);                   // กลับมาบน thread ของหน้าต่าง
            me.loaded = true;
        });
    });
}</code></pre>
<ul>
  <li><code>begin_invoke</code> ส่งงานไปแล้วทำต่อเลย ไม่รอ ได้ทั้งตัวโปรแกรม (<code>me</code>) และ control (<code>ui</code>)</li>
  <li><code>invoke</code> แบบเดียวกัน แต่รอจนงานบนหน้าต่างเสร็จ</li>
  <li>งานที่ไม่ต้องใช้ <code>me</code> หรือ <code>ui</code>: <code>invoker.begin_invoke(move || ...)</code></li>
  <li>หน้าต่างปิดไปแล้ว งานที่ส่งมาทีหลังจะถูกข้ามไปเอง</li>
</ul>

<h2>พิมพ์น้อยลง: <code>use phai_ui::prelude::*;</code></h2>
<p>ไฟล์โค้ดของทุกฟอร์มมีบรรทัด <code>use phai_ui::prelude::*;</code> อยู่แล้ว (Studio ใส่ให้) ของที่ใช้บ่อยจึงเรียกชื่อสั้นได้เลย ไม่ต้องพิมพ์ <code>phai_ui::form::</code> นำหน้า:</p>
<pre><code>message_box(ui.hwnd, "บันทึกแล้ว", "ร้านค้า", MessageBoxButtons::Ok, MessageBoxIcon::Information);
if message_box(ui.hwnd, "ลบไหม?", "ร้านค้า", MessageBoxButtons::YesNo, MessageBoxIcon::Question) == DialogResult::Yes { /* ลบ */ }
ui.status.set_part_text(0, &amp;tf("สวัสดี {0}", &amp;[&amp;name]));</code></pre>
<p>ที่ได้มาด้วย: <code>message_box</code>, <code>MessageBoxButtons</code>, <code>MessageBoxIcon</code>, <code>DialogResult</code>, <code>t</code>, <code>tf</code> และชนิดของ <code>e</code> ในเมธอดของ event (<code>KeyEventArgs</code>, <code>MouseEventArgs</code>, <code>PaintEventArgs</code> ...) — บนเว็บก็ใช้ได้ (กล่องข้อความเป็นของเบราว์เซอร์)</p>

<h2>ฟอร์มเองก็มีเมธอด</h2>
<p><code>ui.close()</code>, <code>ui.hide()</code>, <code>ui.show_window()</code>, <code>ui.activate()</code>, <code>ui.center_to_screen()</code>, <code>ui.set_title("...")</code> เปิดฟอร์มที่สองด้วย <code>OtherFormUi::show(ui.hwnd, OtherForm::default())</code> หรือแบบรอคำตอบ <code>show_dialog</code> ที่คืน <code>DialogResult</code></p>
'''
