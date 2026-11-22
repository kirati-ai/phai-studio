# Toolbox pages, Containers: GroupBox, FlowPanel, Panel, SplitContainer, TabControl, TablePanel, UserControl
# (see control_pages.py for the fields).
PAGES = {
    'group': dict(
        title='กลุ่ม', winforms='GroupBox',
        intro='''<p>กรอบที่มี<b>หัวข้อ</b> ใช้จัด control ที่เกี่ยวกันไว้ด้วยกัน ให้หน้าต่างอ่านง่าย เช่น "ที่อยู่จัดส่ง" "วิธีชำระเงิน"</p>
<ul><li>วาง control ลง<b>ข้างใน</b>กรอบในตัวออกแบบ แล้วมันจะเป็นของกรอบนั้น (ย้ายกรอบ ของข้างในย้ายตาม)</li>
<li>ใช้บ่อยกับ <a href="ctl-radio.html">ตัวเลือก</a>: คำถามละกรอบ ตั้งเลขกลุ่ม (Group) ของตัวเลือกให้ต่างกันในแต่ละกรอบ</li>
<li><b>Text</b> คือหัวข้อของกรอบ</li></ul>''',
        examples=[
            ('1. สองคำถามบนหน้าต่างเดียว', '<p>ตัวเลือกในกรอบ Delivery ใช้กลุ่ม 1 ในกรอบ Payment ใช้กลุ่ม 2 จึงเลือกได้คำถามละข้อ:</p>', ['read']),
        ],
        tries=['เพิ่มกรอบที่สาม "ขนาดกล่อง" ที่มีตัวเลือกเล็ก/กลาง/ใหญ่ (กลุ่ม 3)',
               'ปิดทั้งกรอบด้วย <code>ui.payment.set_enabled(false)</code> แล้วดูว่า control ข้างในเป็นอย่างไร'],
    ),
    'flow': dict(
        title='จัดเรียงต่อกัน', winforms='FlowLayoutPanel',
        intro='''<p>แผงที่<b>เรียง control ต่อกันเอง</b> จากซ้ายไปขวา เต็มบรรทัดแล้วขึ้นบรรทัดใหม่ เหมือนตัวหนังสือ ไม่ต้องจัดตำแหน่งทีละตัว</p>
<ul><li>วาง control ลงข้างในแผง แล้วแผงจะจัดตำแหน่งให้ตามลำดับในฟอร์ม</li>
<li><b>FlowDirection</b> เรียงไปทางไหน · <b>WrapContents</b> ขึ้นบรรทัดใหม่เมื่อเต็มไหม · <b>Padding</b> ระยะจากขอบ</li>
<li>ซ่อน control ตัวหนึ่ง ตัวอื่นจะขยับมาปิดช่องว่างเอง</li></ul>''',
        examples=[
            ('1. วางในแผงตอนออกแบบ', '<p>control สี่ตัวที่อยู่ในกรอบของ <code>tags</code> ถูกแผงจัดเรียงให้ (ตำแหน่งที่เขียนไว้ใช้แค่บอกว่าอยู่ในแผงไหน):</p>', ['form']),
            ('2. ซ่อนแล้วตัวอื่นขยับมา', '', ['hide']),
        ],
        tries=['เปลี่ยน FlowDirection เป็นแนวตั้ง แล้วดูว่า control เรียงอย่างไร',
               'ทำแผงเลือกสีจากปุ่มหลาย ๆ ปุ่ม ที่เรียงเองเมื่อขยายหน้าต่าง (ตั้ง Anchor ของแผง)'],
    ),
    'panel': dict(
        title='แผง', winforms='Panel',
        intro='''<p>พื้นที่สี่เหลี่ยมสำหรับ<b>รวม control</b> ไว้ด้วยกัน (เหมือนกลุ่มแต่ไม่มีหัวข้อ) หรือเป็น<b>ผ้าใบให้โปรแกรมวาดเอง</b>ด้วย event Paint</p>
<ul><li><b>BorderStyle</b> = fixed มีเส้นขอบ · ตั้ง BackColor ให้เห็นเป็นพื้นที่</li>
<li>event <b>Paint</b> ได้ <code>e: &amp;PaintEventArgs</code> ไว้วาด: <code>clear</code>, <code>fill_rect</code>, <code>draw_line</code>, <code>draw_text</code> - สีเขียนแบบ CSS เช่น <code>"#12855f"</code> <code>"white"</code></li>
<li>อยากให้วาดใหม่ (เช่น ค่าที่วาดเปลี่ยน) เรียก <code>invalidate()</code></li></ul>''',
        examples=[
            ('1. วาดเองใน Paint', '<p><code>e.width</code> / <code>e.height</code> คือขนาดของแผง ใช้วางของไว้ตรงกลาง:</p>', ['paint']),
            ('2. ให้วาดใหม่', '<p>เปลี่ยนค่าที่ใช้วาด แล้วสั่ง <code>invalidate()</code> - Windows จะเรียก <code>board_paint</code> อีกครั้ง:</p>', ['redraw']),
        ],
        tries=['วาดวงกลมด้วยสี่เหลี่ยมเล็ก ๆ หลายอันหรือเส้นหลายเส้น',
               'ทำกราฟแท่งง่าย ๆ จากตัวเลขห้าตัวด้วย <code>fill_rect</code>'],
    ),
    'split': dict(
        title='แบ่งสองช่อง', winforms='SplitContainer',
        intro='''<p>แบ่งพื้นที่เป็น<b>สองช่อง</b>ซ้าย-ขวา (หรือบน-ล่าง) มีแถบตรงกลางที่ผู้ใช้<b>ลากปรับขนาด</b>ได้ เหมือน Explorer ที่มีรายการโฟลเดอร์ทางซ้ายและไฟล์ทางขวา</p>
<ul><li>มักตั้ง <b>Dock</b> = fill ให้เต็มหน้าต่าง · <b>Orientation</b> แนวตั้งหรือแนวนอน · <b>SplitterDistance</b> ความกว้างของช่องแรก</li>
<li>วาง control ลงในช่องซ้ายหรือขวาในตัวออกแบบ</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนแถบถูกลาก (<code>…_changed</code>)</li></ul>''',
        examples=[
            ('1. รู้ว่าแถบอยู่ตรงไหน', '', ['moved']),
        ],
        tries=['ปุ่ม "แบ่งครึ่ง" ที่เรียก <code>set_splitter_distance</code> ให้สองช่องกว้างเท่ากัน',
               'ใส่ <a href="ctl-tree.html">TreeView</a> ทางซ้ายและ <a href="ctl-list.html">ListView</a> ทางขวา แบบ Explorer'],
    ),
    'tabs': dict(
        title='แท็บ', winforms='TabControl',
        intro='''<p><b>แท็บ</b>ให้หน้าต่างเดียวมีหลายหน้า ผู้ใช้คลิกแท็บเพื่อสลับหน้า เหมาะกับหน้าตั้งค่าที่มีหลายหมวด</p>
<ul><li>ใส่ชื่อแท็บใน <b>Tabs</b> (บรรทัดละแท็บ)</li>
<li><b>ต่างจาก WinForms:</b> ใน Phai Studio แท็บคือ<b>แถบชื่อแท็บ</b> - ทำให้สูงพอดีกับชื่อแท็บ (ราว 30 px) แล้ววาง control ของแต่ละหน้า<b>ไว้ใต้แถบ</b> และตั้ง <b>page</b> ของ control นั้น (0 = แท็บแรก, 1 = แท็บที่สอง …) - control จะขึ้นเฉพาะตอนแท็บของมันถูกเลือก</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนเปลี่ยนแท็บ (<code>…_changed</code>)</li></ul>''',
        examples=[
            ('1. แถบแท็บกับ control ของแต่ละหน้า', '<p>แถบ <code>pages</code> สูง 34 px · <code>name_label</code> กับ <code>name</code> อยู่หน้า 0 · <code>dark</code> อยู่หน้า 1 (ตำแหน่งเดียวกันได้ เพราะไม่ขึ้นพร้อมกัน):</p>', ['form']),
            ('2. รู้ว่าอยู่หน้าไหน', '', ['changed']),
            ('3. ไปหน้าอื่นด้วยโค้ด', '', ['select']),
        ],
        tries=['เพิ่มแท็บที่สาม "About" ที่มีป้ายบอกชื่อโปรแกรม (page = 2)',
               'ให้ปุ่ม "ถัดไป" พาไปแท็บถัดไปเรื่อย ๆ และกลับมาแท็บแรกเมื่อสุด'],
    ),
    'table': dict(
        title='จัดเป็นตาราง', winforms='TableLayoutPanel',
        intro='''<p>แผงที่<b>จัด control เป็นแถวและคอลัมน์</b>เท่า ๆ กันเอง เหมาะกับแบบฟอร์มที่มีป้ายทางซ้ายและช่องกรอกทางขวา ไม่ต้องจัดให้ตรงกันทีละตัว</p>
<ul><li><b>Columns</b> (cols) มีกี่คอลัมน์ - control ข้างในถูกวางทีละช่อง ซ้ายไปขวา บนลงล่าง ตามลำดับในฟอร์ม</li>
<li>ขยายแผง ทุกช่องขยายตามเท่า ๆ กัน</li></ul>''',
        examples=[
            ('1. แบบฟอร์มสองคอลัมน์', '<p>ตั้ง <code>cols = 2</code> แล้ววางป้ายกับช่องกรอกสลับกันข้างในแผง:</p>', ['form']),
            ('2. อ่านค่าจาก control ในแผง', '<p>control ในแผงก็เรียกตาม Id เหมือนเดิม:</p>', ['join']),
        ],
        tries=['เพิ่มแถว "ชื่อเล่น" (ป้าย + ช่องกรอก) แล้วดูว่าแผงจัดให้เอง',
               'ลอง cols = 3 แล้วดูว่า control เรียงใหม่อย่างไร'],
    ),
    'usercontrol': dict(
        title='User control', winforms='UserControl',
        intro='''<p>control ที่<b>เราสร้างเอง</b>จาก control อื่น ๆ แล้วใช้ซ้ำได้หลายที่ เช่น กล่องนับจำนวน กล่องที่อยู่ แถบค้นหา - ออกแบบครั้งเดียว วางกี่ที่ก็ได้</p>
<ul><li>สร้าง: <b>Project &gt; Add User Control...</b> ได้ฟอร์มที่มี <code>user_control = true</code> ออกแบบเหมือนฟอร์มทั่วไป</li>
<li>ใช้: ลาก <b>User control</b> จากกล่องเครื่องมือมาวาง แล้วตั้ง <b>Type</b> เป็นชื่อของมัน (เช่น <code>CounterBox</code>)</li>
<li>มันบอกฟอร์มว่ามีอะไรเปลี่ยนด้วย <code>ui.notify_parent()</code> - ฟอร์มได้ยินทาง <code>…_changed</code></li>
<li>ฟอร์มอ่าน control ข้างในมันด้วย <code>CounterBoxUi::with(ui.ชื่อ, |c| …)</code></li></ul>''',
        examples=[
            ('1. สร้าง user control', '<p><code>counter_box.phaiform</code> มีปุ่ม -, ป้ายตัวเลข, ปุ่ม + (สังเกต <code>user_control = true</code>) และโค้ดของมัน:</p>', ['file:counter_box.phaiform', 'counter_box.rs:uc']),
            ('2. วางสองตัวบนหน้าต่าง แล้วรวมค่า', '<p>ทั้งสองตัวใช้ <code>count_changed</code> - อ่านตัวเลขจากแต่ละตัวด้วย <code>CounterBoxUi::with</code>:</p>', ['read']),
        ],
        tries=['ไม่ให้ CounterBox ติดลบ (ปิดปุ่ม - เมื่อเป็น 0)',
               'สร้าง user control "แถบค้นหา" (ช่องพิมพ์ + ปุ่ม) แล้ววางบนสองหน้าต่าง'],
    ),
}
TH = {
    'SplitContainer.splitter_distance': 'ความกว้าง (หรือความสูง) ของช่องแรก',
    'SplitContainer.set_splitter_distance': 'ย้ายแถบ - control ในสองช่องขยับตาม',
    'TabControl.selected_index': 'แท็บที่เลือกอยู่ นับจาก 0',
    'TabControl.select_tab': 'ไปแท็บที่ <code>i</code> เหมือนคลิกแท็บนั้น',
}
