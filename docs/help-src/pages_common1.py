# Toolbox pages: CheckBox, CheckedListBox, ComboBox, DateTimePicker, Label (see control_pages.py for the fields).
PAGES = {
    'checkbox': dict(
        title='ช่องติ๊ก', winforms='CheckBox',
        intro='''<p>ช่องติ๊กให้ผู้ใช้ตอบ <b>ใช่/ไม่ใช่</b> ทีละเรื่อง เช่น "จำรหัสผ่าน" "ส่งข่าวทางอีเมล" แต่ละช่องเป็นอิสระต่อกัน ติ๊กได้หลายช่อง
(ถ้าต้องการให้เลือกได้<b>ข้อเดียว</b>จากหลายข้อ ใช้ <a href="ctl-radio.html">ตัวเลือก (RadioButton)</a> แทน)</p>
<ul><li>ตั้ง <b>Checked</b> = True ให้ติ๊กไว้ตั้งแต่เปิดหน้าต่าง</li>
<li>ตั้ง <b>ThreeState</b> ให้มีสถานะที่สาม "ครึ่ง ๆ" (Indeterminate) เช่น ติ๊กบางข้อในกลุ่ม</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนกด (Click) - ส่วน event <b>CheckedChanged</b> เกิดทุกครั้งที่ติ๊กเปลี่ยน ทั้งจากเมาส์และจากโค้ด</li></ul>''',
        examples=[
            ('1. ติ๊กแล้วปุ่มถึงกดได้', '<p>ปุ่ม "Sign up" ปิดไว้ตอนเปิดหน้าต่าง ทุกครั้งที่กดช่อง <code>agree</code> ให้ปุ่มเปิดหรือปิดตามว่าติ๊กอยู่ไหม:</p>', ['click']),
            ('2. อ่านว่าติ๊กอะไรไว้', '<p><code>checked()</code> ได้ <code>true</code> หรือ <code>false</code>:</p>', ['read']),
        ],
        tries=['เพิ่มช่องติ๊ก "แสดงรหัสผ่าน" ที่เรียก <code>set_password</code> ของช่องข้อความรหัสผ่าน',
               'เพิ่มช่อง "เลือกทั้งหมด" ที่กดแล้วติ๊กทุกช่องด้วย <code>set_checked(true)</code>'],
    ),
    'checkedlist': dict(
        title='กล่องรายการติ๊ก', winforms='CheckedListBox',
        intro='''<p>รายการที่แต่ละบรรทัดมีช่องติ๊ก เหมาะกับให้ผู้ใช้<b>เลือกหลายอย่าง</b>จากรายการยาว ๆ เช่น หน้าท็อปปิ้งพิซซ่า วิชาที่จะลงเรียน</p>
<ul><li>ใส่รายการตอนออกแบบใน <b>Items</b> (บรรทัดละรายการ) หรือในโค้ดด้วย <code>items_add</code></li>
<li>รายการนับจาก 0: บรรทัดแรกคือ 0 บรรทัดที่สองคือ 1 …</li>
<li><code>checked_indices()</code> ได้ตำแหน่งของทุกบรรทัดที่ติ๊กไว้</li></ul>''',
        examples=[
            ('1. ใส่รายการตอนเปิดหน้าต่าง', '<p>ใส่ห้ารายการ แล้วติ๊กบรรทัดแรกไว้ให้:</p>', ['fill']),
            ('2. อ่านว่าติ๊กอะไรไว้บ้าง', '<p>ได้ตำแหน่งที่ติ๊กเป็น <code>Vec&lt;usize&gt;</code> แล้วแปลงเป็นชื่อด้วย <code>item(i)</code>:</p>', ['read']),
        ],
        tries=['เพิ่มปุ่ม "ไม่เอาเลย" ที่เอาติ๊กออกทุกบรรทัด (วนตั้งแต่ 0 ถึง <code>items_count()</code>)',
               'คิดราคา: แต่ละท็อปปิ้ง 20 บาท แสดงราคารวมในป้าย'],
    ),
    'combo': dict(
        title='รายการเลือก', winforms='ComboBox',
        intro='''<p>กล่องที่กดแล้วมีรายการเลื่อนลงมาให้<b>เลือกหนึ่งอย่าง</b> ใช้พื้นที่บนหน้าต่างน้อย เหมาะกับรายการที่ไม่ยาวเกินไป เช่น จังหวัด ขนาดเสื้อ</p>
<ul><li>ใส่รายการตอนออกแบบใน <b>Items</b> หรือในโค้ดด้วย <code>items_add</code></li>
<li><b>DropDownStyle</b>: <code>DropDown</code> พิมพ์เองได้ด้วย · <code>DropDownList</code> เลือกจากรายการเท่านั้น</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนเลือกเปลี่ยน (<code>…_changed</code>)</li>
<li><code>selected_index()</code> ได้ <code>Option&lt;usize&gt;</code> - <code>None</code> คือยังไม่ได้เลือก</li></ul>''',
        examples=[
            ('1. เลือกแล้วแสดงสิ่งที่เลือก', '<p><code>selected_item()</code> ได้ข้อความของรายการที่เลือก ถ้ายังไม่ได้เลือกเป็น <code>None</code>:</p>', ['changed']),
            ('2. เพิ่มรายการเอง', '<p>ไม่ให้เพิ่มคำว่าง และไม่ให้ซ้ำ (<code>find_string_exact</code>) เพิ่มแล้วเลือกให้เลย:</p>', ['add']),
        ],
        tries=['ตั้ง DropDownStyle = DropDownList แล้วดูว่าพิมพ์เองยังได้ไหม',
               'เพิ่มปุ่ม "ลบ" ที่ลบรายการที่เลือกอยู่ด้วย <code>items_remove_at</code>'],
    ),
    'date': dict(
        title='วันที่', winforms='DateTimePicker',
        intro='''<p>ช่องสำหรับเลือก<b>วันที่</b> กดลูกศรแล้วมีปฏิทินให้เลือก ผู้ใช้ไม่ต้องพิมพ์วันที่เอง จึงไม่มีรูปแบบผิด</p>
<ul><li><code>value()</code> ได้ <code>(ปี, เดือน, วัน)</code> เป็นตัวเลขสามตัว</li>
<li><b>Format</b> และ <b>CustomFormat</b> กำหนดว่าวันที่แสดงหน้าตาแบบไหน</li>
<li><b>MinDate</b> / <b>MaxDate</b> จำกัดช่วงวันที่ที่เลือกได้ · <b>ShowCheckBox</b> ให้ผู้ใช้เลือกว่า "ไม่ระบุวันที่" ได้</li></ul>''',
        examples=[
            ('1. ตั้งวันที่ตอนเปิดหน้าต่าง', '<p><code>set_value(ปี, เดือน, วัน)</code>:</p>', ['set']),
            ('2. อ่านวันที่ที่เลือก', '<p>ทุกครั้งที่เลือกวันใหม่ method <code>…_changed</code> ทำงาน:</p>', ['changed']),
        ],
        tries=['คำนวณอายุจากวันเกิด (ปีนี้ลบปีเกิด) แล้วแสดงในป้าย',
               'เพิ่มวันที่ตัวที่สอง "วันกลับ" แล้วเตือนถ้าวันกลับมาก่อนวันไป'],
    ),
    'label': dict(
        title='ป้าย', winforms='Label',
        intro='''<p>ป้ายคือ<b>ข้อความบนหน้าต่าง</b>ที่ผู้ใช้อ่านแต่แก้ไม่ได้ เช่น หัวข้อ คำอธิบายหน้าช่องกรอก ผลลัพธ์ที่โปรแกรมคำนวณได้</p>
<ul><li>เปลี่ยนข้อความในโค้ดด้วย <code>set_text</code> - ใช้แสดงผลได้ดี</li>
<li>ตั้งขนาดตัวอักษรและตัวหนาใน Properties (<b>Font</b>) · <b>TextAlign</b> ชิดซ้าย กลาง ขวา</li>
<li>ป้ายรับการคลิกได้ (event <b>Click</b>) แต่ถ้าอยากให้เป็นลิงก์ ใช้ <a href="ctl-link.html">ลิงก์ (LinkLabel)</a></li></ul>''',
        examples=[
            ('1. เปลี่ยนข้อความเมื่อคลิก', '<p>ในแท็บ Events ของป้าย ดับเบิลคลิก <b>Click</b> แล้วเขียน - ตั้ง Cursor = <code>hand</code> ให้รู้ว่ากดได้:</p>', ['click']),
        ],
        tries=['ให้ป้ายแสดงเวลาปัจจุบัน ใช้ <a href="ctl-timer.html">ตัวจับเวลา (Timer)</a> อัปเดตทุกวินาที',
               'ตั้ง BackColor และ BorderStyle ให้ป้ายดูเหมือนกล่องผลลัพธ์'],
    ),
}

TH = {
    'CheckedListBox.items_add': 'เพิ่มบรรทัดท้ายสุด',
    'CheckedListBox.items_clear': 'ลบทุกบรรทัด',
    'CheckedListBox.items_count': 'มีกี่บรรทัด',
    'CheckedListBox.item': 'ข้อความของบรรทัดที่ <code>i</code>',
    'CheckedListBox.checked': 'บรรทัดที่ <code>i</code> ติ๊กอยู่ไหม',
    'CheckedListBox.set_checked': 'ติ๊กหรือเอาติ๊กออกที่บรรทัด <code>i</code>',
    'CheckedListBox.checked_indices': 'ตำแหน่งของทุกบรรทัดที่ติ๊ก',
    'CheckedListBox.selected_index': 'บรรทัดที่เลือก (ไฮไลต์) อยู่ - <code>None</code> ถ้าไม่มี',
    'ComboBox.items_add': 'เพิ่มรายการท้ายสุด ได้ตำแหน่งของมัน',
    'ComboBox.items_insert': 'แทรกรายการที่ตำแหน่ง <code>at</code>',
    'ComboBox.items_remove_at': 'ลบรายการที่ตำแหน่ง <code>at</code>',
    'ComboBox.items_clear': 'ลบทุกรายการ',
    'ComboBox.items_count': 'มีกี่รายการ',
    'ComboBox.item': 'ข้อความของรายการที่ตำแหน่ง <code>at</code>',
    'ComboBox.items': 'ทุกรายการ เป็น <code>Vec&lt;String&gt;</code>',
    'ComboBox.selected_index': 'ตำแหน่งที่เลือก นับจาก 0 - <code>None</code> ถ้ายังไม่เลือก',
    'ComboBox.set_selected_index': 'เลือกรายการที่ตำแหน่งนี้ (<code>None</code> = ไม่เลือก)',
    'ComboBox.selected_item': 'ข้อความของรายการที่เลือก',
    'ComboBox.find_string': 'รายการแรกที่ขึ้นต้นด้วย <code>prefix</code> (ไม่สนตัวพิมพ์เล็ก/ใหญ่)',
    'ComboBox.find_string_exact': 'รายการที่ตรงกับ <code>s</code> ทุกตัว',
    'ComboBox.set_dropped_down': 'เปิดหรือปิดรายการที่เลื่อนลงมา',
    'ComboBox.begin_update': 'หยุดวาดชั่วคราวระหว่างใส่รายการจำนวนมาก แล้วเรียก <code>end_update</code>',
    'ComboBox.end_update': 'กลับมาวาดตามปกติ',
    'DateTimePicker.value': 'วันที่ที่เลือก เป็น <code>(ปี, เดือน, วัน)</code>',
    'DateTimePicker.set_value': 'ตั้งวันที่',
    'DateTimePicker.checked': 'เมื่อเปิด ShowCheckBox: ผู้ใช้ระบุวันที่ไหม (ติ๊กช่องไว้)',
}
