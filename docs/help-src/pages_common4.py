# Toolbox pages: RichTextBox, TextBox, TextBox (Multiline), TrackBar, TreeView (see control_pages.py for the fields).
PAGES = {
    'richtext': dict(
        title='กล่องข้อความหลายรูปแบบ', winforms='RichTextBox', textedit=True,
        intro='''<p>กล่องข้อความยาว ๆ หลายบรรทัด อ่านและบันทึกเป็นไฟล์ได้ในคำสั่งเดียว หาคำได้ และเห็นที่อยู่เว็บเป็นลิงก์เอง เหมาะกับโปรแกรมจดบันทึก ตัวแก้ข้อความ</p>
<ul><li><code>load_file</code> / <code>save_file</code> อ่านและบันทึกไฟล์ข้อความ (UTF-8) - ได้ <code>Result</code> กลับมา ต้องดูว่าสำเร็จไหม</li>
<li><code>find(คำ, เริ่มที่)</code> หาคำแล้วเลือกให้</li>
<li><b>WordWrap</b> ตัดบรรทัดตามความกว้าง · <b>DetectUrls</b> ทำให้ที่อยู่เว็บเป็นลิงก์ · <b>ReadOnly</b> อ่านอย่างเดียว</li></ul>''',
        examples=[
            ('1. หาคำ (กดซ้ำเพื่อหาคำถัดไป)', '<p>จำตำแหน่งที่เจอล่าสุดไว้ใน struct แล้วหาต่อจากตรงนั้น:</p>', ['find']),
            ('2. บันทึกลงไฟล์', '<p><code>save_file</code> อาจไม่สำเร็จ (เช่น ไม่มีสิทธิ์เขียน) จึงได้ <code>Result</code> - ใช้ <code>match</code> แยกสองกรณี:</p>', ['save']),
            ('3. นับบรรทัด', '<p><code>lines()</code> ได้แต่ละบรรทัดเป็น <code>Vec&lt;String&gt;</code>:</p>', ['lines']),
        ],
        tries=['เพิ่มปุ่ม "เปิด" ที่อ่าน story.txt กลับมาด้วย <code>load_file</code>',
               'ให้ใช้ <a href="ctl-savefiledialog.html">หน้าต่างบันทึกไฟล์</a> ถามชื่อไฟล์ก่อนบันทึก'],
    ),
    'edit': dict(
        title='ช่องข้อความ', winforms='TextBox', textedit=True,
        intro='''<p>ช่องให้ผู้ใช้<b>พิมพ์ข้อความหนึ่งบรรทัด</b> เช่น ชื่อ อีเมล รหัสผ่าน คำค้นหา - control ที่ใช้บ่อยที่สุดตัวหนึ่ง</p>
<ul><li><code>text()</code> อ่านสิ่งที่พิมพ์ (ได้ <code>String</code>) · <code>set_text</code> ใส่ข้อความให้</li>
<li><b>PlaceholderText</b> ข้อความจาง ๆ บอกว่าต้องพิมพ์อะไร · <b>MaxLength</b> พิมพ์ได้สูงสุดกี่ตัว</li>
<li><b>Password</b> = True แสดงเป็นจุด สำหรับรหัสผ่าน · <b>ReadOnly</b> อ่านได้แต่แก้ไม่ได้</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ที่ทำงานทุกครั้งที่ข้อความเปลี่ยน (<code>…_changed</code>)</li>
<li>หลายบรรทัด ใช้ <a href="ctl-textarea.html">ช่องข้อความหลายบรรทัด</a></li></ul>''',
        examples=[
            ('1. อ่านและตรวจสิ่งที่พิมพ์', '<p><code>trim()</code> ตัดช่องว่างหัวท้ายออก แล้วตรวจว่าว่างไหม ถ้าผิดให้บอกผู้ใช้และส่งแป้นพิมพ์กลับไปที่ช่องนั้น:</p>', ['read']),
            ('2. ทำงานทุกครั้งที่พิมพ์', '<p><code>chars().count()</code> นับตัวอักษร (ภาษาไทยก็นับถูก):</p>', ['changed']),
        ],
        tries=['ใช้ <code>set_password(false)</code> ทำช่องติ๊ก "แสดงรหัสผ่าน"',
               'ให้ปุ่ม Sign in กดได้เฉพาะเมื่อพิมพ์ชื่อแล้ว (ใน <code>name_changed</code>)'],
    ),
    'textarea': dict(
        title='ช่องข้อความหลายบรรทัด', winforms='TextBox (Multiline)', textedit=True,
        intro='''<p>ช่องพิมพ์ที่มี<b>หลายบรรทัด</b> เช่น ที่อยู่ หมายเหตุ ไดอารี่ - เป็น TextBox ที่ตั้ง Multiline ไว้ (ใน phai-ui เป็นชนิด <code>TextBox</code> เหมือนกัน)</p>
<ul><li>ขึ้นบรรทัดใหม่ในโค้ดด้วย <code>\\r\\n</code> (แบบของ Windows)</li>
<li><code>append_text</code> ต่อท้ายแล้วเลื่อนไปให้เห็น · <code>lines()</code> ได้แต่ละบรรทัด</li>
<li><b>ScrollBars</b> แถบเลื่อน · <b>WordWrap</b> ตัดบรรทัดตามความกว้าง · ตั้ง <b>Anchor</b> = all ให้ขยายตามหน้าต่าง</li></ul>''',
        examples=[
            ('1. ต่อข้อความท้ายสุด', '', ['append']),
            ('2. นับบรรทัดและคำ', '<p>ทุกครั้งที่ข้อความเปลี่ยน นับใหม่ - <code>split_whitespace()</code> แยกคำตามช่องว่าง:</p>', ['lines']),
        ],
        tries=['ปุ่ม "ลบทั้งหมด" ที่เรียก <code>clear()</code>',
               'ปุ่ม "ใส่วันที่" ที่ต่อวันที่ของวันนี้ท้ายข้อความ'],
    ),
    'slider': dict(
        title='ตัวเลื่อน', winforms='TrackBar',
        intro='''<p>แถบให้<b>ลากเลือกค่า</b>ในช่วงหนึ่ง เช่น ความดังเสียง ความสว่าง ขนาดตัวอักษร - เห็นทันทีว่าค่าอยู่ตรงไหนของช่วง</p>
<ul><li><b>Minimum</b> / <b>Maximum</b> ช่วงของค่า · <b>Value</b> ค่าตอนเปิด</li>
<li><b>TickFrequency</b> ระยะห่างของขีด · <b>SmallChange</b> / <b>LargeChange</b> ขยับเท่าไหร่เมื่อกดลูกศร / Page Up-Down</li>
<li><b>Orientation</b> แนวนอนหรือแนวตั้ง</li></ul>''',
        examples=[
            ('1. อ่านค่าระหว่างลาก', '<p>ค่าเปลี่ยนแล้ว method <code>…_changed</code> ทำงาน:</p>', ['changed']),
            ('2. ตั้งค่าด้วยโค้ด', '', ['set']),
        ],
        tries=['ให้ตัวเลื่อนเปลี่ยนความกว้างของ <a href="ctl-progress.html">แถบความคืบหน้า</a> ตามไปด้วย',
               'ใช้ตัวเลื่อนสามตัว (แดง เขียว น้ำเงิน) ผสมสีแล้วแสดงรหัสสี เช่น <code>#40a0ff</code>'],
    ),
    'tree': dict(
        title='มุมมองต้นไม้', winforms='TreeView',
        intro='''<p>รายการที่มี<b>รายการย่อยซ้อนอยู่ข้างใน</b> กดเปิด-ปิดได้ เหมือนโฟลเดอร์ใน Explorer เช่น หมวดหมู่สินค้า สารบัญหนังสือ ผังองค์กร</p>
<ul><li>ใส่ตอนออกแบบใน <b>Items</b>: บรรทัดละรายการ เว้นสองช่องหน้ารายการต่อหนึ่งชั้นที่ลึกลงไป</li>
<li>ในโค้ด: <code>nodes_add</code> ใส่รายการบนสุด ได้ <code>Node</code> กลับมา แล้ว <code>nodes_add_under(node, …)</code> ใส่ใต้มัน</li>
<li><code>selected_text()</code> ข้อความของรายการที่เลือก · <code>expand_all()</code> เปิดทุกชั้น</li></ul>''',
        examples=[
            ('1. สร้างต้นไม้ในโค้ด', '', ['build']),
            ('2. รู้ว่าเลือกอะไร', '<p>เลือกรายการแล้ว method <code>…_chosen</code> ทำงาน:</p>', ['chosen']),
        ],
        tries=['ใส่ต้นไม้เดียวกันนี้ผ่าน Items ในหน้าต่าง Properties แทนการเขียนโค้ด',
               'เพิ่มช่องพิมพ์และปุ่ม "เพิ่มใต้ที่เลือก" ด้วย <code>selected_node()</code> และ <code>nodes_add_under</code>'],
    ),
}

TH = {
    'TrackBar.value': 'ค่าตอนนี้',
    'TrackBar.set_value': 'ตั้งค่า (ตัวเลื่อนขยับไปตาม)',
    'TreeView.nodes_add': 'ใส่รายการบนสุด ได้ <code>Node</code> ไว้ใส่รายการใต้มัน',
    'TreeView.nodes_add_under': 'ใส่รายการใต้ <code>parent</code>',
    'TreeView.nodes_clear': 'ลบทุกรายการ',
    'TreeView.set_nodes': 'ใส่ทั้งต้นจากบรรทัด แบบเดียวกับ Items (สองช่องต่อหนึ่งชั้น)',
    'TreeView.expand_all': 'เปิดทุกชั้น',
    'TreeView.collapse_all': 'ปิดทุกชั้น',
    'TreeView.selected_node': 'รายการที่เลือก - <code>None</code> ถ้าไม่มี',
    'TreeView.select': 'เลือกรายการนี้',
    'TreeView.node_text': 'ข้อความของรายการ',
    'TreeView.selected_text': 'ข้อความของรายการที่เลือก',
    'TreeView.expand': 'เปิดรายการนี้',
    'TreeView.collapse': 'ปิดรายการนี้',
}
