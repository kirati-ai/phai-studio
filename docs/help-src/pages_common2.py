# Toolbox pages: LinkLabel, ListBox, ListView, MaskedTextBox, MonthCalendar (see control_pages.py for the fields).
PAGES = {
    'link': dict(
        title='ลิงก์', winforms='LinkLabel',
        intro='''<p>ข้อความที่มี<b>ลิงก์</b>อยู่ข้างใน เหมือนลิงก์บนเว็บ คลิกแล้วโปรแกรมทำอะไรสักอย่าง เช่น เปิดเว็บไซต์ เปิดหน้าต่างช่วยเหลือ</p>
<ul><li>ใน <b>Text</b> ใส่ส่วนที่เป็นลิงก์ไว้ระหว่าง <code>&lt;a&gt;</code> กับ <code>&lt;/a&gt;</code> เช่น <code>อ่านต่อที่ &lt;a&gt;เว็บไซต์&lt;/a&gt;</code> - ไม่ใส่เลย ทั้งข้อความเป็นลิงก์</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนคลิกลิงก์ (<code>…_clicked</code>)</li></ul>''',
        examples=[
            ('1. คลิกแล้วเปิดเว็บไซต์', '<p>ให้ Windows เปิดที่อยู่เว็บด้วยเบราว์เซอร์ของเครื่อง:</p>', ['click']),
        ],
        tries=['เปลี่ยนให้ลิงก์เปิดโฟลเดอร์ Documents แทน (ส่งพาธโฟลเดอร์ให้ <code>explorer</code>)',
               'ใช้ <code>set_link_text</code> เปลี่ยนข้อความหลังคลิกเป็น "ขอบคุณที่แวะมา"'],
    ),
    'listbox': dict(
        title='กล่องรายการ', winforms='ListBox',
        intro='''<p>กล่องที่แสดง<b>รายการเป็นบรรทัด ๆ</b> ให้เห็นทั้งหมดพร้อมกัน (ต่างจาก <a href="ctl-combo.html">ComboBox</a> ที่ต้องกดก่อน) เหมาะกับรายการสิ่งที่ต้องทำ รายชื่อ</p>
<ul><li>ใส่รายการตอนออกแบบใน <b>Items</b> หรือในโค้ดด้วย <code>items_add</code></li>
<li><b>Sorted</b> เรียงตามตัวอักษรเอง · <b>SelectionMode</b> = <code>multi</code> เลือกได้หลายบรรทัดด้วย <kbd>Ctrl</kbd>/<kbd>Shift</kbd></li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนเลือกบรรทัด (<code>…_chosen</code>)</li></ul>''',
        examples=[
            ('1. รู้ว่าเลือกบรรทัดไหน', '<p><code>selected_item()</code> ได้ข้อความของบรรทัดที่เลือก:</p>', ['chosen']),
            ('2. เพิ่มบรรทัด', '<p>เอาข้อความจากช่องพิมพ์มาต่อท้าย (ไม่เอาช่องว่าง):</p>', ['add']),
            ('3. ลบบรรทัดที่เลือก', '<p><code>selected_index()</code> ได้ตำแหน่ง แล้ว <code>items_remove_at</code> ลบทิ้ง:</p>', ['remove']),
        ],
        tries=['ตั้ง Sorted = True แล้วดูว่ารายการใหม่ไปอยู่ตรงไหน',
               'ดับเบิลคลิกบรรทัด (event DoubleClick) แล้วให้ข้อความไปอยู่ในช่องพิมพ์เพื่อแก้ไข'],
    ),
    'list': dict(
        title='รายการ', winforms='ListView',
        intro='''<p>ตารางที่มี<b>หลายคอลัมน์</b> แต่ละแถวเป็นข้อมูลหนึ่งชุด เช่น รายชื่อนักเรียนกับห้องและคะแนน (ถ้าต้องการให้แก้ในช่องได้และผูกฐานข้อมูล ใช้ <a href="ctl-grid.html">ตารางข้อมูล (DataGridView)</a>)</p>
<ul><li>กำหนดคอลัมน์ตอนออกแบบใน <b>Columns</b> (บรรทัดละ <code>ชื่อ:ความกว้าง</code>) · <b>ColumnAlign</b> ชิดซ้าย/ขวาทีละคอลัมน์ เช่น ตัวเลขชิดขวา</li>
<li>แถวและคอลัมน์นับจาก 0 · <code>cell(แถว, คอลัมน์)</code> อ่านช่อง</li>
<li><b>Icons</b> รูปเล็ก ๆ หน้าแถว เช่น <code>CONTACT #12855f</code> (ชื่อรูป สี) · <b>View</b> ดูแบบตาราง รูปใหญ่ หรือรูปเล็กได้</li></ul>''',
        examples=[
            ('1. ใส่แถว', '<p>หนึ่งแถว = ข้อความของทุกคอลัมน์ เรียงตามลำดับ - <code>items_add_with_icon</code> ใส่รูปที่ 0 ของ Icons ไว้หน้าแถวด้วย:</p>', ['fill']),
            ('2. อ่านแถวที่เลือก', '<p>เลือกแถวแล้ว method <code>…_chosen</code> ทำงาน:</p>', ['chosen']),
        ],
        tries=['คลิกหัวคอลัมน์ (event ColumnClick) แล้วแสดงว่าคลิกคอลัมน์ที่เท่าไหร่ (<code>e.index</code>)',
               'ตั้ง CheckBoxes = True แล้วนับว่าติ๊กกี่แถวด้วย <code>checked(i)</code>'],
    ),
    'masked': dict(
        title='ช่องกรอกตามแบบ', winforms='MaskedTextBox',
        intro='''<p>ช่องพิมพ์ที่มี<b>แบบ (Mask)</b> บังคับว่าต้องพิมพ์อะไร ตรงไหน เช่น เบอร์โทร เลขบัตรประชาชน รหัสไปรษณีย์ ผู้ใช้พิมพ์ผิดรูปแบบไม่ได้</p>
<ul><li>ตั้ง <b>Mask</b>: <code>0</code> ตัวเลข · <code>9</code> ตัวเลขหรือเว้นว่าง · <code>L</code> ตัวอักษร · <code>A</code> ตัวอักษรหรือตัวเลข · ตัวอื่นคงอยู่ตามนั้น เช่น <code>000-000-0000</code> หรือ <code>0-0000-00000-00-0</code></li>
<li><code>text()</code> ได้ข้อความพร้อมขีด · <code>value()</code> ได้เฉพาะที่พิมพ์ · <code>mask_completed()</code> กรอกครบแล้วไหม</li></ul>''',
        examples=[
            ('1. กดบันทึกได้เมื่อกรอกครบ', '<p>ทุกครั้งที่พิมพ์ method <code>…_changed</code> ทำงาน:</p>', ['changed']),
            ('2. อ่านสิ่งที่กรอก', '', ['value']),
        ],
        tries=['เปลี่ยน Mask เป็นเลขบัตรประชาชน <code>0-0000-00000-00-0</code>',
               'เพิ่มช่องรหัสไปรษณีย์ 5 หลัก แล้วให้ปุ่มกดได้เมื่อกรอกครบทั้งสองช่อง'],
    ),
    'calendar': dict(
        title='ปฏิทินเดือน', winforms='MonthCalendar',
        intro='''<p>ปฏิทินทั้งเดือนบนหน้าต่าง ให้คลิกเลือกวัน เหมาะกับหน้าที่ต้องเห็นวันต่าง ๆ ตลอด เช่น จองห้อง ตารางนัด
(ถ้าอยากใช้พื้นที่น้อย ใช้ <a href="ctl-date.html">วันที่ (DateTimePicker)</a> ซึ่งกดแล้วค่อยมีปฏิทินขึ้นมา)</p>
<ul><li><code>selection_start()</code> ได้วันที่คลิก เป็น <code>(ปี, เดือน, วัน)</code></li>
<li><code>set_date(ปี, เดือน, วัน)</code> เลือกวันให้ด้วยโค้ด</li></ul>''',
        examples=[
            ('1. เลือกวันตอนเปิดหน้าต่าง', '', ['set']),
            ('2. อ่านวันที่คลิก', '<p>คลิกวันใหม่แล้ว method <code>…_changed</code> ทำงาน:</p>', ['changed']),
        ],
        tries=['ให้ป้ายบอกชื่อเดือนเป็นภาษาไทย (มกราคม … ธันวาคม) จากเลขเดือน',
               'นับว่าอีกกี่วันจะถึงวันที่เลือก'],
    ),
}

TH = {
    'ListBox.items_add': 'เพิ่มบรรทัดท้ายสุด (หรือในที่ของมันเมื่อ Sorted) ได้ตำแหน่งที่ไปอยู่',
    'ListBox.items_insert': 'แทรกบรรทัดที่ตำแหน่ง <code>at</code>',
    'ListBox.items_remove_at': 'ลบบรรทัดที่ <code>i</code>',
    'ListBox.items_clear': 'ลบทุกบรรทัด',
    'ListBox.items_count': 'มีกี่บรรทัด',
    'ListBox.item': 'ข้อความของบรรทัดที่ <code>i</code>',
    'ListBox.selected_index': 'บรรทัดที่เลือก - <code>None</code> ถ้าไม่มี',
    'ListBox.set_selected_index': 'เลือกบรรทัดนี้ (<code>None</code> = ไม่เลือก)',
    'ListBox.selected_indices': 'ทุกบรรทัดที่เลือก (เมื่อ SelectionMode = multi)',
    'ListBox.selected_item': 'ข้อความของบรรทัดที่เลือก',
    'ListView.items_add': 'เพิ่มแถว: ข้อความของแต่ละคอลัมน์ ได้ตำแหน่งของแถว',
    'ListView.items_add_with_icon': 'เพิ่มแถวพร้อมรูปที่ <code>icon</code> ของ Icons',
    'ListView.set_item_icon': 'เปลี่ยนรูปของแถว <code>at</code>',
    'ListView.set_icons': 'ตั้งรูปทั้งชุดแบบเดียวกับ property Icons เช่น <code>"FOLDER #d9a400, DOCUMENT"</code>',
    'ListView.items_remove_at': 'ลบแถว <code>at</code>',
    'ListView.items_clear': 'ลบทุกแถว',
    'ListView.items_count': 'มีกี่แถว',
    'ListView.cell': 'ข้อความในช่อง (แถว, คอลัมน์)',
    'ListView.set_cell': 'เปลี่ยนข้อความในช่อง (แถว, คอลัมน์)',
    'ListView.selected_index': 'แถวที่เลือก (แถวแรก ถ้าเลือกหลายแถว)',
    'ListView.selected_indices': 'ทุกแถวที่เลือก',
    'ListView.select': 'เลือกแถว <code>at</code> แถวเดียว และเลื่อนไปให้เห็น',
    'ListView.ensure_visible': 'เลื่อนให้เห็นแถว <code>at</code>',
    'ListView.checked': 'แถว <code>at</code> ติ๊กอยู่ไหม (เมื่อ CheckBoxes = True)',
    'ListView.set_checked': 'ติ๊กหรือเอาติ๊กออกที่แถว <code>at</code>',
    'ListView.begin_update': 'หยุดวาดชั่วคราวระหว่างใส่แถวจำนวนมาก แล้วเรียก <code>end_update</code>',
    'ListView.end_update': 'กลับมาวาดตามปกติ',
    'MaskedTextBox.value': 'สิ่งที่พิมพ์ ไม่รวมตัวคั่นของแบบและช่องว่าง',
    'MaskedTextBox.mask_completed': 'กรอกครบทุกช่องที่ต้องกรอกแล้วไหม',
    'MaskedTextBox.clear': 'ลบสิ่งที่พิมพ์',
    'MonthCalendar.selection_start': 'วันที่เลือก เป็น <code>(ปี, เดือน, วัน)</code>',
    'MonthCalendar.set_date': 'เลือกวันที่ด้วยโค้ด',
}
