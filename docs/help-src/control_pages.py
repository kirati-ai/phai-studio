# What each Toolbox page says in Thai, a page a tool (help_controls.py builds them). The rest of a
# page - icon, picture, properties, events, methods - comes from Phai Studio and phai-ui.
#   title     its Thai name           winforms  what WinForms calls it
#   intro     what it is, when to use it (HTML)
#   examples  [(heading, text before the code (HTML), [names between "// help:" markers in src/<tag>_demo.rs])]
#   tries     exercises (HTML)
#   textedit  it has TextEdit's methods too
PAGES = {
    'button': dict(
        title='ปุ่ม', winforms='Button',
        intro='''<p>ปุ่มคือสิ่งที่ผู้ใช้<b>กด</b>เพื่อสั่งให้โปรแกรมทำอะไรสักอย่าง เช่น บันทึก ค้นหา ปิดหน้าต่าง
ข้อความบนปุ่มควรบอกว่ากดแล้วจะเกิดอะไร เช่น "บันทึก" ดีกว่า "ตกลง"</p>
<ul><li>ดับเบิลคลิกปุ่มในตัวออกแบบ Phai Studio จะเขียน method สำหรับตอนกดให้ (event <b>Click</b>)</li>
<li>ตั้ง <b>Class</b> = <code>primary</code> ให้ปุ่มหลักของหน้าต่างเป็นสีเด่น</li>
<li>ตั้งปุ่มเป็น <b>AcceptButton</b> ของฟอร์ม แล้วกด <kbd>Enter</kbd> ที่ไหนก็กดปุ่มนี้ (และ <b>CancelButton</b> สำหรับ <kbd>Esc</kbd>)</li>
<li>ปุ่มในกล่องถามตั้ง <b>DialogResult</b> ได้ กดแล้วหน้าต่างปิดพร้อมคำตอบนั้นเอง - ดูหน้า <a href="ctl-form.html">ฟอร์ม</a></li></ul>''',
        examples=[
            ('1. นับจำนวนครั้งที่กด (Click)', '<p>ดับเบิลคลิกปุ่ม <code>click_me</code> แล้วเขียน: บวกหนึ่ง แล้วเปลี่ยนข้อความของป้าย - ค่าที่ต้องจำไว้ระหว่างการกดแต่ละครั้งเก็บใน struct (<code>self.clicks</code>)</p>', ['click']),
            ('2. ปุ่มที่กดได้เฉพาะตอนที่มีความหมาย (Enabled)', '<p>ตอนเปิดหน้าต่างยังไม่มีอะไรให้เริ่มใหม่ จึงปิดปุ่ม <code>reset</code> ไว้ก่อน (สีเทา):</p>', ['loaded']),
            ('', '<p>กดแล้วกลับไปเริ่มต้น ปิดตัวเองอีกครั้ง และส่งแป้นพิมพ์กลับไปที่ปุ่มแรก:</p>', ['reset']),
        ],
        tries=['เพิ่มปุ่ม "-1" ที่ลดจำนวนลง แต่ไม่ให้ต่ำกว่าศูนย์',
               'ให้ปุ่ม <code>click_me</code> เปลี่ยนข้อความเป็น "พอแล้ว!" และกดไม่ได้เมื่อกดครบ 10 ครั้ง',
               'ใช้ <code>ui.click_me.set_tooltip("…")</code> ใน <code>loaded</code> ให้มีคำอธิบายเมื่อชี้เมาส์'],
    ),
}
