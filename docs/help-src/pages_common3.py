# Toolbox pages: NotifyIcon, NumericUpDown, PictureBox, ProgressBar, RadioButton (see control_pages.py for the fields).
PAGES = {
    'notifyicon': dict(
        title='ไอคอนมุมจอ', winforms='NotifyIcon',
        intro='''<p>ไอคอนเล็ก ๆ ที่<b>มุมขวาล่างของแถบงาน</b> ข้างนาฬิกา ใช้บอกว่าโปรแกรมทำงานอยู่ แม้หน้าต่างจะปิดหรือซ่อนไว้ และส่งข้อความแจ้งเตือนแบบที่ Windows ใช้</p>
<ul><li><b>Text</b> คือข้อความที่ขึ้นเมื่อชี้เมาส์ที่ไอคอน</li>
<li><code>show_balloon_tip(หัวข้อ, ข้อความ)</code> ส่งข้อความแจ้งเตือน</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนคลิกไอคอน - มักใช้เรียกหน้าต่างกลับมา</li>
<li>ไอคอนหายไปเองเมื่อหน้าต่างปิด (โค้ดที่ Phai Studio เขียนจัดการให้)</li></ul>''',
        examples=[
            ('1. ส่งข้อความแจ้งเตือน', '', ['balloon']),
            ('2. คลิกไอคอนแล้วเรียกหน้าต่างกลับ', '', ['click']),
        ],
        tries=['ซ่อนหน้าต่าง (<code>ui.hide()</code>) เมื่อย่อหน้าต่าง แล้วให้ไอคอนเรียกกลับมา',
               'ใช้ <a href="ctl-timer.html">ตัวจับเวลา</a> แจ้งเตือน "ได้เวลาพัก" ทุก 30 นาที'],
    ),
    'number': dict(
        title='ตัวเลข', winforms='NumericUpDown',
        intro='''<p>ช่องสำหรับ<b>ตัวเลข</b>ที่มีลูกศรขึ้น/ลง ผู้ใช้พิมพ์ตัวเลขหรือกดลูกศรได้ และไม่มีทางกรอกเกินช่วงที่กำหนด</p>
<ul><li><b>Minimum</b> / <b>Maximum</b> ช่วงที่อนุญาต · <b>Value</b> ค่าตอนเปิด · <b>Increment</b> กดลูกศรหนึ่งครั้งเพิ่มเท่าไหร่</li>
<li><code>value()</code> ได้ตัวเลขเป็น <code>i32</code> ไม่ต้องแปลงข้อความเอง</li>
<li>ดับเบิลคลิกในตัวออกแบบ ได้ method ตอนค่าเปลี่ยน (<code>…_changed</code>)</li></ul>''',
        examples=[
            ('1. คำนวณจากตัวเลขที่เลือก', '<p>ทุกครั้งที่ค่าเปลี่ยน คำนวณราคาใหม่ (และเรียกครั้งแรกใน <code>loaded</code> ให้มีราคาตั้งแต่เปิดหน้าต่าง):</p>', ['changed']),
        ],
        tries=['ตั้ง Increment = 5 แล้วลองกดลูกศร',
               'เพิ่มช่องตัวเลข "ส่วนลด %" แล้วคิดราคาหลังหักส่วนลด'],
    ),
    'picture': dict(
        title='รูปภาพ', winforms='PictureBox',
        intro='''<p>แสดง<b>รูปภาพ</b>บนหน้าต่าง จากไฟล์ PNG, JPEG, BMP หรือ GIF เช่น โลโก้ รูปสินค้า รูปประจำตัว</p>
<ul><li>คลิกช่อง <b>Image</b> ใน Properties แล้วเลือกไฟล์รูป รูปขึ้นในตัวออกแบบทันที (ไฟล์ที่อยู่นอกโปรเจกต์ Phai Studio คัดลอกมาไว้ในโปรเจกต์ให้ กด <kbd>Delete</kbd> ที่ช่องนี้เพื่อเอารูปออก)</li>
<li><b>SizeMode</b>: <code>zoom</code> ใหญ่ที่สุดที่พอดีโดยไม่เบี้ยว · <code>stretch</code> ยืดเต็มกรอบ · <code>center</code> อยู่กลาง · <code>normal</code> ขนาดจริงชิดซ้ายบน</li>
<li><code>load(ไฟล์)</code> เปลี่ยนรูปในโค้ด · event <b>Paint</b> วาดเองทับรูปได้</li></ul>''',
        examples=[
            ('1. เปลี่ยนรูปด้วยโค้ด', '', ['load']),
            ('2. คลิกแล้วเปลี่ยนวิธีวางรูป', '<p>รูปรับการคลิกได้ (event Click) - ทุกครั้งที่คลิกเปลี่ยน SizeMode ไปแบบถัดไป:</p>', ['mode']),
        ],
        tries=['ทำปุ่ม "ก่อนหน้า/ถัดไป" ที่สลับรูปหลาย ๆ รูปแบบสไลด์',
               'ให้ผู้ใช้เลือกรูปเองด้วย <a href="ctl-openfiledialog.html">หน้าต่างเปิดไฟล์</a> แล้ว <code>load</code>'],
    ),
    'progress': dict(
        title='แถบความคืบหน้า', winforms='ProgressBar',
        intro='''<p>แถบที่บอกว่างาน<b>ทำไปได้แค่ไหนแล้ว</b> เช่น ดาวน์โหลด คัดลอกไฟล์ ติดตั้ง ผู้ใช้จะได้รู้ว่าโปรแกรมไม่ได้ค้าง</p>
<ul><li><b>Minimum</b> / <b>Maximum</b> (มักเป็น 0 ถึง 100) · <b>Value</b> ตอนนี้อยู่ที่เท่าไหร่ · <b>Step</b> ขยับครั้งละเท่าไหร่</li>
<li>งานที่ไม่รู้ว่านานเท่าไหร่ ตั้ง <b>Style = Marquee</b> ให้มีแถบวิ่งไปเรื่อย ๆ (หรือ <code>set_marquee(true)</code>)</li></ul>''',
        examples=[
            ('1. ขยับทีละขั้น', '<p><code>perform_step()</code> บวกเพิ่มตาม Step:</p>', ['step']),
            ('2. เริ่มใหม่', '', ['again']),
        ],
        tries=['ใช้ <a href="ctl-timer.html">ตัวจับเวลา</a> ขยับแถบเองทุก 200 มิลลิวินาทีจนเต็ม',
               'ลอง <code>set_marquee(true)</code> แล้วดูความต่าง'],
    ),
    'radio': dict(
        title='ตัวเลือก', winforms='RadioButton',
        intro='''<p>ปุ่มกลม ๆ ให้<b>เลือกได้ข้อเดียว</b>จากหลายข้อ เลือกข้อหนึ่งแล้วข้ออื่นในกลุ่มเดียวกันหลุดเอง เช่น ขนาดแก้ว เพศ วิธีจ่ายเงิน
(ถ้าเลือกได้หลายข้อ ใช้ <a href="ctl-checkbox.html">ช่องติ๊ก (CheckBox)</a>)</p>
<ul><li><b>Group</b>: ตัวเลือกที่มีเลขกลุ่มเดียวกันจะกันกันเอง - ถ้ามีสองคำถามบนหน้าต่างเดียว ให้คนละเลขกลุ่ม (หรือใส่คนละ <a href="ctl-group.html">กลุ่ม (GroupBox)</a>)</li>
<li>ตั้ง <b>Checked</b> = True ให้ข้อหนึ่งถูกเลือกไว้ก่อนเสมอ</li>
<li>หลายตัวเลือกใช้ method เดียวกันได้ - ตั้งชื่อเดียวกันใน Click ของทุกตัว</li></ul>''',
        examples=[
            ('1. รู้ว่าเลือกข้อไหน', '<p>ทั้งสามตัวเลือกใช้ method <code>size_clicked</code> ตัวเดียวกัน แล้วถามว่าตอนนี้ตัวไหน <code>checked()</code>:</p>', ['which']),
        ],
        tries=['เพิ่มตัวเลือก "ใส่ไข่มุก +10 บาท" เป็น<b>ช่องติ๊ก</b> แล้วรวมราคา',
               'เพิ่มคำถามที่สอง "ร้อน/เย็น" ในกลุ่มเลข 2'],
    ),
}

TH = {
    'NotifyIcon.set_text': 'ข้อความที่ขึ้นเมื่อชี้เมาส์ที่ไอคอน',
    'NotifyIcon.set_visible': 'แสดงหรือซ่อนไอคอน',
    'NotifyIcon.show_balloon_tip': 'ส่งข้อความแจ้งเตือนจากไอคอน: หัวข้อ, ข้อความ',
    'NotifyIcon.remove': 'เอาไอคอนออกถาวร (โค้ดที่ Phai Studio เขียนทำให้เองตอนปิดหน้าต่าง)',
    'NumericUpDown.value': 'ตัวเลขตอนนี้',
    'NumericUpDown.set_value': 'ตั้งตัวเลข (ไม่เกินช่วงที่กำหนด)',
    'NumericUpDown.up_button': 'เพิ่มขึ้นตาม Increment เหมือนกดลูกศรขึ้น',
    'NumericUpDown.down_button': 'ลดลงตาม Increment เหมือนกดลูกศรลง',
    'PictureBox.load': 'แสดงรูปจากไฟล์อื่น',
    'PictureBox.set_size_mode': 'วิธีวางรูป: <code>"zoom"</code>, <code>"stretch"</code>, <code>"center"</code>, <code>"normal"</code>',
    'ProgressBar.value': 'ค่าตอนนี้',
    'ProgressBar.set_value': 'ตั้งค่า',
    'ProgressBar.perform_step': 'เพิ่มตาม Step',
    'ProgressBar.increment': 'เพิ่ม <code>n</code>',
    'ProgressBar.set_marquee': '<code>true</code> แถบวิ่งไปเรื่อย ๆ สำหรับงานที่ไม่รู้ความยาว',
}
