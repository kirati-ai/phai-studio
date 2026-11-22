# Toolbox pages, the last groups: Data (BindingSource, DataGridView), Components (ErrorProvider, Timer),
# Printing (PrintDocument), Dialogs (ColorDialog, FolderBrowserDialog, FontDialog, OpenFileDialog,
# SaveFileDialog) - see control_pages.py for the fields.
PAGES = {
    'grid': dict(
        title='ตารางข้อมูล', winforms='DataGridView',
        intro='''<p>ตารางที่ผู้ใช้<b>พิมพ์แก้ในช่องได้</b> เพิ่มแถวที่ท้ายตารางได้ เรียง กรอง ค้นหา และผูกกับฐานข้อมูลได้ - control ที่เก่งที่สุดของ Phai Studio สำหรับงานข้อมูล
(ถ้าแค่แสดงรายการ ใช้ <a href="ctl-list.html">ListView</a> ก็พอ)</p>
<ul><li>กำหนดคอลัมน์ใน <b>Columns</b> และตั้งค่าแต่ละคอลัมน์ (ชนิด รูปแบบตัวเลข ผลรวม …) ด้วยปุ่ม Grid Columns ในตัวออกแบบ</li>
<li><code>rows_add</code> ใส่แถว · <code>rows()</code> อ่านทุกแถว · <code>cell</code> / <code>set_cell</code> ทีละช่อง (แถว, คอลัมน์ นับจาก 0)</li>
<li>ส่งออกได้ในคำสั่งเดียว: <code>export_xlsx</code> (Excel), <code>export_pdf</code>, <code>export_csv</code> และ <code>print_preview</code></li>
<li>ผูกกับตารางในฐานข้อมูลได้ - ดู <a href="database.html">ทำงานกับฐานข้อมูล</a></li></ul>''',
        examples=[
            ('1. ใส่แถว', '', ['fill']),
            ('2. คำนวณจากทุกแถว', '<p><code>rows()</code> ได้ <code>Vec&lt;Vec&lt;String&gt;&gt;</code> - แปลงคะแนนเป็นตัวเลขด้วย <code>parse()</code> ช่องที่พิมพ์ไม่ใช่ตัวเลขจะถูกข้ามไป:</p>', ['read']),
            ('3. เรียงด้วยโค้ด', '<p>เหมือนคลิกหัวคอลัมน์ - <code>false</code> คือมากไปน้อย:</p>', ['sort']),
        ],
        tries=['ปุ่ม "Excel" ที่เรียก <code>export_xlsx</code> แล้วเปิดไฟล์ดู',
               'เปิด AllowFilter ใน Properties แล้วลองกรองจากหัวคอลัมน์แบบ Excel'],
    ),
    'bindingsource': dict(
        title='แหล่งข้อมูลผูก', winforms='BindingSource',
        intro='''<p>ที่เก็บ<b>ข้อมูลหลายแถว</b> (เหมือนตารางเล็ก ๆ) ที่<b>ผูกกับ control</b> ได้ - control แต่ละตัวแสดงคอลัมน์หนึ่งของ "แถวปัจจุบัน" เลื่อนแถวแล้ว control เปลี่ยนตามเอง ผู้ใช้แก้ใน control แล้วข้อมูลในแถวเปลี่ยนตาม</p>
<ul><li><b>DataMember</b> (fields): ชื่อคอลัมน์ คั่นด้วยจุลภาค เช่น <code>name, phone</code></li>
<li>ผูก control: ตั้ง <b>DataBindings</b> ของ control เป็น <code>Id.คอลัมน์</code> เช่น <code>people.name</code></li>
<li><code>set_rows</code> ใส่ข้อมูล · <code>move_next</code> / <code>move_previous</code> เลื่อนแถว · <code>position()</code> อยู่แถวไหน · <code>add_new</code> / <code>remove_current</code> เพิ่ม/ลบ</li>
<li>ผูกกับตารางในฐานข้อมูลได้ - ดู <a href="database.html">ทำงานกับฐานข้อมูล</a></li></ul>''',
        examples=[
            ('1. ผูก control กับคอลัมน์', '<p>ช่อง <code>name</code> ตั้ง <code>data_binding = "people.name"</code> · <code>phone</code> ตั้ง <code>"people.phone"</code> · แหล่งข้อมูล <code>people</code> มี <code>fields = "name, phone"</code>:</p>', ['form']),
            ('2. ใส่ข้อมูล', '', ['rows']),
            ('3. เลื่อนไปมา', '<p>event <b>PositionChanged</b> เกิดทุกครั้งที่เปลี่ยนแถว - ใช้บอกว่าอยู่แถวที่เท่าไหร่:</p>', ['move']),
        ],
        tries=['ปุ่ม "เพิ่ม" ที่เรียก <code>add_new()</code> แล้วพิมพ์ชื่อใหม่ได้เลย',
               'ปุ่ม "แสดงทั้งหมด" ที่รวมทุกแถวจาก <code>rows()</code> ใส่ป้ายหรือ message_box'],
    ),
    'errorprovider': dict(
        title='ตัวบอกข้อผิดพลาด', winforms='ErrorProvider',
        intro='''<p>ใส่<b>เครื่องหมายสีแดง</b>ข้างช่องที่กรอกผิด ชี้เมาส์แล้วขึ้นเหตุผล ผู้ใช้รู้ทันทีว่าต้องแก้ช่องไหน โดยไม่ต้องมีกล่องข้อความเด้งขึ้นมา</p>
<ul><li>ตัวเดียวใช้ได้กับทุก control บนหน้าต่าง</li>
<li><code>set_error(control, "เหตุผล")</code> ใส่เครื่องหมาย · <code>set_error(control, "")</code> เอาออก · <code>clear()</code> เอาออกทั้งหมด</li></ul>''',
        examples=[
            ('1. ตรวจแล้วบอกว่าผิดตรงไหน', '', ['check']),
        ],
        tries=['ตรวจทุกครั้งที่พิมพ์ (ใน <code>…_changed</code> ของช่องอีเมล) แทนตอนกดปุ่ม',
               'เพิ่มช่องเบอร์โทรที่ต้องเป็นตัวเลข 10 ตัว แล้วใช้ตัวบอกข้อผิดพลาดตัวเดียวกัน'],
    ),
    'timer': dict(
        title='ตัวจับเวลา', winforms='Timer',
        intro='''<p>ทำงานบางอย่าง<b>ซ้ำทุก ๆ ช่วงเวลา</b> เช่น นาฬิกา นับถอยหลัง เกม ภาพเคลื่อนไหว ตรวจข้อมูลใหม่ทุกนาที - มองไม่เห็นบนหน้าต่าง</p>
<ul><li><b>Interval</b>: ห่างกันกี่มิลลิวินาที (1000 = หนึ่งวินาที) · <b>Enabled</b>: เริ่มนับตั้งแต่เปิดหน้าต่างไหม</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ที่ทำงานทุกครั้งที่ครบเวลา (<code>…_ticked</code>)</li>
<li><code>start()</code> / <code>stop()</code> เริ่มและหยุด · <code>set_interval</code> เปลี่ยนความเร็ว</li></ul>''',
        examples=[
            ('1. นับวินาที', '', ['tick']),
            ('2. เริ่มและหยุด', '', ['startstop']),
        ],
        tries=['ทำนับถอยหลัง 10 วินาที แล้วขึ้น message_box เมื่อถึง 0',
               'ทำนาฬิกาที่แสดงเวลาจริงของเครื่อง'],
    ),
    'printdocument': dict(
        title='เอกสารสำหรับพิมพ์', winforms='PrintDocument',
        intro='''<p><b>พิมพ์ออกเครื่องพิมพ์</b> โดยโปรแกรมวาดแต่ละหน้าเองในโค้ด - ข้อความ เส้น กรอบ รูป เช่น ใบเสร็จ ใบประกาศ รายงาน
(ถ้าต้องการออกแบบรายงานด้วยการลากวาง ดู <a href="report.html">รายงานพิมพ์</a>)</p>
<ul><li>event <b>PrintPage</b> ได้ <code>e: &amp;PrintPageEventArgs</code> ไว้วาดหน้ากระดาษ: <code>set_font</code>, <code>set_color</code>, <code>draw_text</code>, <code>draw_line</code>, <code>draw_rect</code>, <code>draw_image</code> - หน่วยเป็นร้อยส่วนของนิ้ว</li>
<li><code>e.margin_bounds</code> พื้นที่ภายในขอบกระดาษ · มีหลายหน้า: <code>e.set_has_more_pages(true)</code></li>
<li><code>print_preview()</code> ดูก่อนพิมพ์ · <code>print_dialog()</code> ให้เลือกเครื่องพิมพ์ · <b>Landscape</b> กระดาษแนวนอน</li></ul>''',
        examples=[
            ('1. วาดหน้ากระดาษ', '', ['page']),
            ('2. ดูก่อนพิมพ์ และพิมพ์', '', ['buttons']),
        ],
        tries=['ใส่ชื่อจากช่องข้อความลงในใบประกาศ',
               'ทำสองหน้า: ใช้ <code>e.page</code> ดูว่ากำลังวาดหน้าไหน และ <code>set_has_more_pages</code>'],
    ),
    'openfiledialog': dict(
        title='หน้าต่างเปิดไฟล์', winforms='OpenFileDialog',
        intro='''<p>หน้าต่าง<b>เลือกไฟล์ที่จะเปิด</b>ของ Windows เอง ผู้ใช้คุ้นเคย ไม่ต้องสร้างเอง</p>
<ul><li><b>Filter</b>: ชนิดไฟล์ที่แสดง เขียนเป็นคู่ <code>ชื่อ|รูปแบบ</code> คั่นด้วย <code>|</code> เช่น <code>Text files|*.txt|All files|*.*</code></li>
<li><b>Title</b> ข้อความบนแถบชื่อ · <b>InitialDirectory</b> โฟลเดอร์เริ่มต้น · <b>Multiselect</b> เลือกได้หลายไฟล์</li>
<li><code>show_dialog()</code> ได้ <code>Option&lt;PathBuf&gt;</code> - <code>None</code> คือกด Cancel</li></ul>''',
        examples=[
            ('1. เปิดไฟล์ข้อความ', '<p><code>let … else { return }</code> ออกเลยถ้ากด Cancel แล้วอ่านไฟล์ด้วย <code>std::fs::read_to_string</code>:</p>', ['open']),
        ],
        tries=['เปิด Multiselect แล้วใช้ <code>show_dialog_multi()</code> บอกว่าเลือกกี่ไฟล์',
               'เปลี่ยน Filter ให้เลือกรูปภาพ แล้วแสดงใน <a href="ctl-picture.html">PictureBox</a>'],
    ),
    'savefiledialog': dict(
        title='หน้าต่างบันทึกไฟล์', winforms='SaveFileDialog',
        intro='''<p>หน้าต่าง<b>เลือกที่และชื่อไฟล์สำหรับบันทึก</b>ของ Windows - ถ้าไฟล์มีอยู่แล้ว Windows ถามให้ว่าจะเขียนทับไหม</p>
<ul><li><b>Filter</b> แบบเดียวกับหน้าต่างเปิดไฟล์ · <b>DefaultExt</b> นามสกุลที่เติมให้ถ้าผู้ใช้ไม่ได้พิมพ์</li>
<li><code>show_dialog_named("note.txt")</code> มีชื่อไฟล์ใส่ไว้ให้ก่อน · ได้ <code>None</code> เมื่อกด Cancel</li></ul>''',
        examples=[
            ('1. บันทึกข้อความลงไฟล์', '', ['save']),
        ],
        tries=['ใช้คู่กับหน้าต่างเปิดไฟล์ ทำโปรแกรมจดบันทึกที่เปิดและบันทึกได้',
               'ตั้งชื่อไฟล์เริ่มต้นเป็นวันที่ของวันนี้'],
    ),
    'folderbrowserdialog': dict(
        title='หน้าต่างเลือกโฟลเดอร์', winforms='FolderBrowserDialog',
        intro='''<p>หน้าต่าง<b>เลือกโฟลเดอร์</b>ของ Windows เช่น เลือกที่เก็บรูป ที่สำรองข้อมูล</p>
<ul><li><code>show_dialog()</code> ได้ <code>Option&lt;PathBuf&gt;</code> ของโฟลเดอร์ที่เลือก</li>
<li><b>InitialDirectory</b> โฟลเดอร์ที่เปิดอยู่ตอนแรก</li></ul>''',
        examples=[
            ('1. เลือกโฟลเดอร์แล้วนับของข้างใน', '', ['choose']),
        ],
        tries=['แสดงชื่อไฟล์ทั้งหมดในโฟลเดอร์ลงใน <a href="ctl-listbox.html">ListBox</a>',
               'นับเฉพาะไฟล์ .png'],
    ),
    'colordialog': dict(
        title='หน้าต่างเลือกสี', winforms='ColorDialog',
        intro='''<p>หน้าต่าง<b>เลือกสี</b>ของ Windows - ได้สีเป็นรหัส <code>#rrggbb</code> ใช้กับ CSS และการวาดได้ทันที</p>
<ul><li><code>show_dialog(สีเริ่มต้น)</code> ได้ <code>Option&lt;String&gt;</code> - <code>None</code> คือกด Cancel</li></ul>''',
        examples=[
            ('1. เลือกสีแล้วระบายแผง', '<p>เก็บสีไว้ใน struct แล้วให้แผงวาดใหม่ (event Paint ของแผงระบายด้วยสีนั้น):</p>', ['pick']),
        ],
        tries=['ให้แผงวาดสองสี ครึ่งซ้ายสีที่เลือกครึ่งขวาสีเดิม',
               'จำห้าสีล่าสุดที่เลือก แล้วแสดงเป็นแถบสีเล็ก ๆ'],
    ),
    'fontdialog': dict(
        title='หน้าต่างเลือกฟอนต์', winforms='FontDialog',
        intro='''<p>หน้าต่าง<b>เลือกแบบตัวอักษร</b>ของ Windows: ชื่อฟอนต์ ขนาด ตัวหนา ตัวเอียง</p>
<ul><li><code>show_dialog()</code> ได้ <code>Option&lt;ChosenFont&gt;</code> ที่มี <code>face</code>, <code>points</code>, <code>bold</code>, <code>italic</code></li></ul>''',
        examples=[
            ('1. เลือกฟอนต์แล้วบอกว่าเลือกอะไร', '<p><code>match</code> กับสองค่าพร้อมกัน (bold, italic) เพื่อเขียนคำอธิบาย:</p>', ['pick']),
        ],
        tries=['ใช้ฟอนต์ที่เลือกในการพิมพ์ (<code>e.set_font</code> ของ <a href="ctl-printdocument.html">PrintDocument</a>)',
               'บอกด้วยว่าเป็นฟอนต์ขนาดใหญ่ (มากกว่า 14 pt) หรือไม่'],
    ),
}

TH = {
    'Grid.rows': 'ทุกแถว เป็นข้อความของแต่ละช่อง',
    'Grid.rows_add': 'เพิ่มแถวท้ายตาราง',
    'Grid.rows_clear': 'ลบทุกแถว',
    'Grid.rows_count': 'มีกี่แถว',
    'Grid.fill': 'ใส่ทุกแถวทีเดียว แทนของเดิม',
    'Grid.cell': 'ข้อความในช่อง (แถว, คอลัมน์)',
    'Grid.set_cell': 'เปลี่ยนข้อความในช่อง',
    'Grid.selected_row': 'แถวที่เลือก - <code>None</code> ถ้าไม่มี',
    'Grid.clear_selection': 'ไม่เลือกแถวไหน',
    'Grid.set_read_only': '<code>true</code> แก้ในช่องไม่ได้',
    'Grid.selected_rows': 'ทุกแถวที่เลือก',
    'Grid.sort_by': 'เรียงตามคอลัมน์ <code>col</code> (<code>true</code> น้อยไปมาก) เหมือนคลิกหัวคอลัมน์',
    'Grid.column': 'ค่าตั้งของคอลัมน์ (ชนิด รูปแบบ ผลรวม …)',
    'Grid.set_column': 'เปลี่ยนค่าตั้งของคอลัมน์',
    'Grid.find': 'ค้นหาเหมือนช่องค้นหา - ข้อความว่างแสดงทุกแถวอีกครั้ง',
    'Grid.set_column_filter': 'กรองคอลัมน์ ให้เหลือเฉพาะแถวที่ค่าอยู่ใน <code>keep</code> (<code>None</code> = ไม่กรอง)',
    'Grid.column_filter': 'ค่าที่ตัวกรองของคอลัมน์เก็บไว้',
    'Grid.clear_filters': 'เอาตัวกรองออกทุกคอลัมน์',
    'Grid.set_master': 'หลัก-ย่อย: ตารางนี้แสดงเฉพาะแถวที่ตรงกับแถวที่เลือกในตาราง <code>master</code>',
    'Grid.export_xlsx': 'บันทึกแถวที่เห็นเป็นไฟล์ Excel (.xlsx)',
    'Grid.export_pdf': 'บันทึกแถวที่เห็นเป็น PDF ได้จำนวนหน้า (0 ถ้าไม่สำเร็จ)',
    'Grid.print_preview': 'ดูตารางก่อนพิมพ์',
    'Grid.copy_rows': 'แถวที่เลือกเป็นข้อความคั่นด้วยแท็บ เหมือนกด <kbd>Ctrl</kbd>+<kbd>C</kbd>',
    'Grid.export_csv': 'บันทึกแถวที่เห็นเป็น CSV ที่ Excel เปิดได้ (ภาษาไทยไม่เพี้ยน)',
    'BindingSource.bind': 'ผูก control กับคอลัมน์ (ปกติ Phai Studio ทำให้จาก DataBindings)',
    'BindingSource.unbind': 'เลิกผูก control',
    'BindingSource.columns': 'ชื่อคอลัมน์',
    'BindingSource.end_edit': 'เอาสิ่งที่แก้ใน control ใส่แถวปัจจุบันทันที',
    'BindingSource.reset_bindings': 'แสดงแถวปัจจุบันใน control อีกครั้ง',
    'BindingSource.rows': 'ทุกแถว รวมสิ่งที่แก้ใน control แล้ว',
    'BindingSource.tagged_rows': 'ทุกแถวพร้อมตำแหน่งเดิมตอนโหลด (0 = แถวที่เพิ่มใหม่) - ใช้บันทึกกลับฐานข้อมูล',
    'BindingSource.set_rows': 'ใส่ข้อมูลใหม่ทั้งหมด แถวแรกขึ้นใน control',
    'BindingSource.count': 'มีกี่แถว',
    'BindingSource.position': 'แถวปัจจุบันอยู่ที่เท่าไหร่ นับจาก 0',
    'BindingSource.set_position': 'ไปแถวที่ <code>i</code>',
    'BindingSource.move_first': 'ไปแถวแรก',
    'BindingSource.move_previous': 'ไปแถวก่อน',
    'BindingSource.move_next': 'ไปแถวถัดไป',
    'BindingSource.move_last': 'ไปแถวสุดท้าย',
    'BindingSource.current': 'แถวปัจจุบัน',
    'BindingSource.value': 'ค่าคอลัมน์ <code>column</code> ของแถวปัจจุบัน',
    'BindingSource.set_value': 'ตั้งค่าคอลัมน์ของแถวปัจจุบัน (control ที่ผูกไว้เปลี่ยนตาม)',
    'BindingSource.add_new': 'เพิ่มแถวว่างท้ายสุดแล้วไปที่แถวนั้น',
    'BindingSource.remove_current': 'ลบแถวปัจจุบัน',
    'ErrorProvider.set_error': 'ใส่เครื่องหมายข้าง control พร้อมเหตุผล - ข้อความว่างคือเอาออก',
    'ErrorProvider.error': 'เหตุผลที่ control มีอยู่ตอนนี้ (ว่างถ้าไม่มี)',
    'ErrorProvider.clear': 'เอาเครื่องหมายออกทั้งหมด',
    'Timer.start': 'เริ่มนับ',
    'Timer.stop': 'หยุด',
    'Timer.enabled': 'กำลังนับอยู่ไหม',
    'Timer.set_enabled': 'เริ่ม (<code>true</code>) หรือหยุด (<code>false</code>)',
    'Timer.interval': 'ช่วงเวลา (มิลลิวินาที)',
    'Timer.set_interval': 'เปลี่ยนช่วงเวลา ถ้ากำลังนับอยู่ก็นับต่อด้วยความเร็วใหม่',
    'PrintDocument.set_painter': 'ให้ฟังก์ชันอื่นวาดแต่ละหน้าแทน event PrintPage (รายงานใช้แบบนี้)',
    'PrintDocument.printer_name': 'เครื่องพิมพ์ที่ใช้ (ว่าง = เครื่องหลักของ Windows)',
    'PrintDocument.set_printer_name': 'เลือกเครื่องพิมพ์ตามชื่อ',
    'PrintDocument.landscape': 'กระดาษแนวนอนไหม',
    'PrintDocument.set_landscape': 'แนวนอน (<code>true</code>) หรือแนวตั้ง',
    'PrintDocument.margins': 'ขอบกระดาษ ซ้าย บน ขวา ล่าง (ร้อยส่วนของนิ้ว)',
    'PrintDocument.set_margins': 'ตั้งขอบกระดาษ',
    'PrintDocument.print': 'พิมพ์ทุกหน้าทันทีโดยไม่ถาม ได้จำนวนหน้า',
    'PrintDocument.print_to_file': 'พิมพ์ลงไฟล์ผ่านเครื่องพิมพ์ เช่น "Microsoft Print to PDF" ได้ไฟล์ PDF',
    'PrintDocument.print_dialog': 'หน้าต่างพิมพ์ของ Windows (เลือกเครื่อง จำนวนชุด) แล้วพิมพ์',
    'PrintDocument.page_setup': 'หน้าต่างตั้งค่ากระดาษ ขอบ แนว - <code>false</code> เมื่อกด Cancel',
    'PrintDocument.page_count': 'เอกสารมีกี่หน้า (วาดโดยไม่พิมพ์)',
    'PrintDocument.print_preview': 'หน้าต่างดูก่อนพิมพ์ มีปุ่มพิมพ์ในตัว',
    'OpenFileDialog.show_dialog': 'เปิดหน้าต่าง ได้ไฟล์ที่เลือก - <code>None</code> เมื่อกด Cancel',
    'OpenFileDialog.show_dialog_multi': 'เปิดหน้าต่างแบบเลือกได้หลายไฟล์ (Multiselect) ได้ทุกไฟล์ที่เลือก',
    'SaveFileDialog.show_dialog': 'เปิดหน้าต่าง ได้ไฟล์ที่จะบันทึก - <code>None</code> เมื่อกด Cancel',
    'SaveFileDialog.show_dialog_named': 'เหมือนข้างบน แต่มีชื่อไฟล์ใส่ไว้ให้ก่อน',
    'FolderBrowserDialog.show_dialog': 'เปิดหน้าต่าง ได้โฟลเดอร์ที่เลือก - <code>None</code> เมื่อกด Cancel',
    'ColorDialog.show_dialog': 'เปิดหน้าต่างโดยเริ่มที่สี <code>start</code> ได้ <code>"#rrggbb"</code> - <code>None</code> เมื่อกด Cancel',
    'FontDialog.show_dialog': 'เปิดหน้าต่าง ได้ <code>ChosenFont</code> (face, points, bold, italic) - <code>None</code> เมื่อกด Cancel',
}
