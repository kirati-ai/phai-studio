# Toolbox pages, Menus and toolbars: ContextMenu, MenuStrip (the menu bar), StatusBar, ToolStrip (the toolbar)
# (see control_pages.py for the fields).
BAR_WHERE = ('<p>อยู่ในกล่องเครื่องมือกลุ่ม <b>เมนูและแถบเครื่องมือ</b> (Menus and toolbars) - ลากมาวางบนฟอร์มแล้วมันจะไปอยู่บนสุดของหน้าต่าง '
             'คลิกที่มันในตัวออกแบบเพื่อพิมพ์ชื่อรายการ และตั้ง Properties ของแต่ละรายการ · มันเป็นส่วนหนึ่งของฟอร์ม ไม่ใช่ control '
             'รายการแต่ละอันทำงานด้วย method ที่มีชื่อเดียวกับ <b>(Name)</b> ของมัน</p>')

ITEM_METHODS = [
    ('ui.item_checked("id")', 'รายการนี้ติ๊กอยู่ไหม (เมนู หรือปุ่มบนแถบเครื่องมือที่ id เดียวกัน)'),
    ('ui.set_item_checked("id", true)', 'ติ๊ก หรือเอาติ๊กออก'),
    ('ui.set_item_enabled("id", false)', '<code>false</code> เป็นสีเทา กดไม่ได้ · <code>true</code> กดได้'),
]

PAGES = {
    'contextmenu': dict(
        title='เมนูคลิกขวา', winforms='ContextMenuStrip',
        intro='''<p>เมนูที่ขึ้นเมื่อผู้ใช้<b>คลิกขวา</b>ที่ control เช่น คัดลอก วาง ลบ - คำสั่งที่เกี่ยวกับสิ่งที่คลิกอยู่</p>
<ul><li>ใส่รายการใน <b>Items</b>: บรรทัดละรายการ แบบ <code>method | ข้อความ</code> เช่น <code>clear_clicked | Clear</code> · <code>-</code> อย่างเดียวคือเส้นคั่น</li>
<li>แล้วตั้ง <b>ContextMenu</b> ของ control ที่ต้องการให้คลิกขวาได้ เป็น Id ของเมนูนี้ - ใช้เมนูเดียวกับหลาย control ได้</li>
<li>แต่ละรายการเรียก method ตามชื่อที่เขียนไว้หน้า <code>|</code></li></ul>''',
        examples=[
            ('1. เมนูกับ control ที่ใช้มัน', '<p><code>note_menu</code> มีสามรายการและเส้นคั่น · ช่อง <code>note</code> ตั้ง <code>context_menu = "note_menu"</code>:</p>', ['form']),
            ('2. method ของแต่ละรายการ', '', ['items']),
        ],
        tries=['เพิ่มรายการ "นับตัวอักษร" ที่บอกจำนวนตัวอักษรด้วย <code>message_box</code>',
               'ใช้เมนูเดียวกันกับช่องข้อความอีกช่อง - ต้องรู้ว่าคลิกขวาที่ช่องไหน (ใช้ event Opening หรือแยกเมนู)'],
    ),
    'statusbar': dict(
        title='แถบสถานะ', winforms='StatusStrip',
        intro='''<p>แถบ<b>ด้านล่างสุดของหน้าต่าง</b> บอกสถานะของโปรแกรม เช่น พร้อม กำลังบันทึก จำนวนรายการ ตำแหน่งเคอร์เซอร์</p>
<ul><li>ตั้ง <b>Dock</b> = bottom ให้ติดล่างเสมอ</li>
<li>แบ่งเป็น<b>ช่อง</b>ด้วย <b>parts</b>: ความกว้างของช่องทางขวา เช่น <code>parts = [140]</code> = ช่องซ้ายกว้างเท่าที่เหลือ ช่องขวากว้าง 140</li>
<li><code>set_part_text(ช่อง, ข้อความ)</code> - ช่องนับจาก 0 ทางซ้าย</li></ul>''',
        examples=[
            ('1. บอกสถานะสองช่อง', '', ['parts']),
        ],
        tries=['เพิ่มช่องที่สามแสดงเวลาปัจจุบันด้วย <a href="ctl-timer.html">ตัวจับเวลา</a> (parts = [140, 80])',
               'แสดง "บันทึกแล้ว" สักครู่แล้วกลับเป็น "Ready"'],
    ),
    'menubar': dict(
        title='แถบเมนู', winforms='MenuStrip',
        where=BAR_WHERE,
        intro='''<p><b>เมนูบนสุดของหน้าต่าง</b> เช่น File, Edit, View, Help - ที่รวมคำสั่งทั้งหมดของโปรแกรม ผู้ใช้หาคำสั่งได้ที่เดียว</p>
<ul><li>คลิกที่แถบในตัวออกแบบแล้วพิมพ์ชื่อเมนูได้เลย · ใส่ <code>&amp;</code> หน้าตัวอักษรให้กด <kbd>Alt</kbd>+ตัวนั้นเปิดเมนูได้ เช่น <code>&amp;File</code></li>
<li>แต่ละรายการตั้ง <b>(Name)</b> = ชื่อ method ที่จะทำงาน · <b>ShortcutKeys</b> ปุ่มลัด เช่น <code>Ctrl+N</code>, <code>F1</code></li>
<li><b>CheckOnClick</b>: กดแล้วติ๊กสลับเอง (เช่น "ตัวใหญ่" เปิด/ปิด) · <b>Separator</b> เส้นคั่น</li></ul>''',
        examples=[
            ('1. เมนูในไฟล์ฟอร์ม', '<p>ทุกอย่างนี้สร้างได้ด้วยการพิมพ์ในตัวออกแบบ - ไฟล์จะออกมาแบบนี้:</p>', ['form']),
            ('2. method ของแต่ละรายการ', '<p>ชื่อ method = (Name) ของรายการ:</p>', ['items']),
            ('3. รายการที่ติ๊กได้', '<p>CheckOnClick สลับติ๊กให้เอง แล้วถามด้วย <code>ui.item_checked</code>:</p>', ['checked']),
        ],
        methods=ITEM_METHODS,
        props_html='''<table class="ref"><tr><th>Property ของรายการ</th><th>ทำอะไร</th></tr>
<tr><td><b>(Name)</b></td><td>ชื่อของรายการ และชื่อ method ที่ทำงานเมื่อเลือก</td></tr>
<tr><td><b>Text</b></td><td>ข้อความที่เห็น (คีย์ภาษาอังกฤษ โปรแกรมแปลให้)</td></tr>
<tr><td><b>ShortcutKeys</b></td><td>ปุ่มลัด เช่น <code>Ctrl+S</code>, <code>F5</code></td></tr>
<tr><td><b>Checked</b></td><td>ติ๊กไว้ตั้งแต่เปิดหน้าต่าง</td></tr>
<tr><td><b>CheckOnClick</b></td><td>กดแล้วติ๊กสลับเอง</td></tr>
<tr><td><b>Enabled</b></td><td>กดได้ หรือเป็นสีเทา</td></tr>
<tr><td><b>Image</b></td><td>รูปเล็ก ๆ หน้ารายการ</td></tr>
<tr><td><b>Separator</b></td><td>เป็นเส้นคั่นแทนรายการ</td></tr></table>''',
        events_html='''<table class="ref"><tr><th>Event</th><th>ทำอะไร</th></tr>
<tr><td><b>Click</b></td><td>เลือกรายการ - method ชื่อเดียวกับ (Name) ของมัน</td></tr>
<tr><td><b>CheckedChanged</b></td><td>ติ๊กของรายการเปลี่ยน (CheckOnClick)</td></tr>
<tr><td><b>DropDownOpening</b></td><td>เมนูกำลังจะเปิด - ติ๊กหรือปิดรายการให้ตรงกับสถานะตอนนี้ได้ที่นี่</td></tr>
<tr><td><b>DropDownClosed</b></td><td>เมนูปิดแล้ว</td></tr>
<tr><td><b>MenuStart</b> / <b>MenuComplete</b></td><td>ของแถบเมนูทั้งแถบ (อยู่ใน Events ของฟอร์ม): เริ่มใช้เมนู / ใช้เสร็จ</td></tr></table>''',
        tries=['เพิ่มเมนู Edit ที่มี Copy (<kbd>Ctrl</kbd>+<kbd>C</kbd>) และ Paste (<kbd>Ctrl</kbd>+<kbd>V</kbd>) ทำงานกับช่องข้อความ',
               'ปิดรายการ "New" ไว้ (<code>set_item_enabled</code>) จนกว่าจะพิมพ์อะไรสักอย่าง'],
    ),
    'toolbar': dict(
        title='แถบเครื่องมือ', winforms='ToolStrip',
        where=BAR_WHERE,
        intro='''<p><b>แถบปุ่มรูปภาพ</b>ใต้เมนู สำหรับคำสั่งที่ใช้บ่อย กดได้ในคลิกเดียว เช่น ใหม่ เปิด บันทึก</p>
<ul><li>แต่ละปุ่มตั้ง <b>(Name)</b> = ชื่อ method · <b>Text</b> = ข้อความที่ขึ้นเมื่อชี้เมาส์ · <b>Image</b> = ชื่อรูป เช่น <code>NEW</code>, <code>OPEN</code>, <code>SAVE</code>, <code>SEARCH</code>, <code>REFRESH</code></li>
<li>ถ้าปุ่มบนแถบเครื่องมือมี (Name) เดียวกับรายการในเมนู สองอย่างนี้ใช้ method เดียวกันและติ๊กพร้อมกัน</li>
<li>ปุ่มที่ "กดค้าง" ได้ ให้โปรแกรมสลับติ๊กเองด้วย <code>set_item_checked</code></li></ul>''',
        examples=[
            ('1. ปุ่มในไฟล์ฟอร์ม', '', ['form']),
            ('2. method ของแต่ละปุ่ม', '', ['items']),
            ('3. ปุ่มที่กดค้าง', '', ['checked']),
        ],
        methods=ITEM_METHODS,
        props_html='''<table class="ref"><tr><th>Property ของปุ่ม</th><th>ทำอะไร</th></tr>
<tr><td><b>(Name)</b></td><td>ชื่อของปุ่ม และชื่อ method ที่ทำงานเมื่อกด</td></tr>
<tr><td><b>Text</b></td><td>ข้อความที่ขึ้นเมื่อชี้เมาส์</td></tr>
<tr><td><b>Image</b></td><td>ชื่อรูปจากชุดไอคอนของ Windows เช่น <code>SAVE</code></td></tr>
<tr><td><b>Checked</b></td><td>กดค้างไว้ตั้งแต่เปิดหน้าต่าง</td></tr>
<tr><td><b>Enabled</b></td><td>กดได้ หรือเป็นสีเทา</td></tr>
<tr><td><b>Separator</b></td><td>เป็นเส้นคั่นแทนปุ่ม</td></tr></table>''',
        events_html='''<table class="ref"><tr><th>Event</th><th>ทำอะไร</th></tr>
<tr><td><b>Click</b></td><td>กดปุ่ม - method ชื่อเดียวกับ (Name) ของมัน</td></tr>
<tr><td><b>CheckedChanged</b></td><td>ปุ่มถูกกดค้างหรือปล่อย</td></tr>
<tr><td><b>ItemClicked</b></td><td>ของแถบทั้งแถบ (อยู่ใน Events ของฟอร์ม): ปุ่มไหนก็ตามถูกกด</td></tr></table>''',
        tries=['เพิ่มปุ่ม "เปิด" (รูป <code>OPEN</code>) ที่อ่าน note.txt กลับมา',
               'ทำเมนู File ที่มีรายการ New และ Save ชื่อเดียวกับปุ่ม แล้วดูว่าใช้ method เดียวกัน'],
    ),
}
TH = {
    'StatusBar.set_part_text': 'ข้อความของช่อง <code>part</code> (นับจาก 0 ทางซ้าย)',
    'StatusBar.set_parts': 'ความกว้างของช่องต่าง ๆ นับจากทางขวา',
}
