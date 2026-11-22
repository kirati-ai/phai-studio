# The methods' Thai lines for the Toolbox pages: 'common.<name>' every control has, 'TextEdit.<name>'
# the text boxes', '<Type>.<name>' a type's own. A method with no line here shows its English `///`
# comment from phai-ui instead, so a new method is never missing from a page.
TH = {
    # every control (the control! macro)
    'common.hwnd': 'ตัว control เอง (HWND) ใช้กับคำสั่งของ Windows ที่ต้องการหน้าต่าง - เหมือน <code>Handle</code>',
    'common.text': 'อ่านข้อความของมัน',
    'common.set_text': 'เปลี่ยนข้อความของมัน',
    'common.enabled': 'ใช้งานได้อยู่ไหม',
    'common.set_enabled': '<code>true</code> ใช้งานได้ · <code>false</code> เป็นสีเทา กดไม่ได้',
    'common.visible': 'มองเห็นอยู่ไหม',
    'common.set_visible': 'แสดงหรือซ่อน',
    'common.show': 'แสดง',
    'common.hide': 'ซ่อน',
    'common.focus': 'ให้แป้นพิมพ์มาที่มัน',
    'common.focused': 'แป้นพิมพ์อยู่ที่มันไหม',
    'common.bring_to_front': 'ยกขึ้นไปอยู่บนสุด เหนือ control ที่ซ้อนกัน',
    'common.send_to_back': 'ส่งไปอยู่ล่างสุด',
    'common.refresh': 'วาดใหม่ทันที',
    'common.invalidate': 'ขอให้วาดใหม่ (Windows วาดให้เมื่อว่าง)',
    'common.bounds': 'ตำแหน่งและขนาดบนหน้าต่าง (RECT)',
    'common.client_size': 'ความกว้างและความสูงด้านใน (กว้าง, สูง)',
    'common.set_bounds': 'ย้ายและเปลี่ยนขนาด: x, y, กว้าง, สูง',
    'common.point_to_screen': 'แปลงจุดบน control เป็นจุดบนจอ',
    'common.point_to_client': 'แปลงจุดบนจอเป็นจุดบน control',
    'common.find_form': 'หน้าต่าง (ฟอร์ม) ที่มันอยู่',
    'common.suspend_layout': 'หยุดจัดตำแหน่งชั่วคราว ระหว่างเปลี่ยนหลายอย่างพร้อมกัน',
    'common.resume_layout': 'จัดตำแหน่งต่อ หลัง <code>suspend_layout</code>',
    'common.set_tooltip': 'ข้อความเล็ก ๆ ที่ขึ้นเมื่อชี้เมาส์ค้างไว้ - เหมือน ToolTip',
    'common.do_drag_drop': 'เริ่มลากข้อความนี้ออกไปวางที่อื่น',
    'common.set_cursor': 'รูปเมาส์เมื่อชี้มัน เช่น <code>"hand"</code>, <code>"wait"</code>, <code>"ibeam"</code>',

    # text boxes: TextEdit's, which TextBox and RichTextBox have as their own too (text_edit_methods!)
    'TextEdit.clear': 'ลบข้อความทั้งหมด',
    'TextEdit.append_text': 'ต่อข้อความท้ายสุด แล้วเลื่อนไปให้เห็น',
    'TextEdit.select_all': 'เลือกข้อความทั้งหมด',
    'TextEdit.select': 'เลือกข้อความ <code>length</code> ตัว เริ่มที่ตำแหน่ง <code>start</code>',
    'TextEdit.selection_start': 'ตำแหน่งเริ่มของส่วนที่เลือก',
    'TextEdit.selection_length': 'เลือกไว้กี่ตัว',
    'TextEdit.selected_text': 'ข้อความที่เลือกไว้',
    'TextEdit.paste_text': 'ใส่ข้อความแทนส่วนที่เลือก (หรือที่ตำแหน่งเคอร์เซอร์)',
    'TextEdit.copy': 'คัดลอกส่วนที่เลือก',
    'TextEdit.cut': 'ตัดส่วนที่เลือก',
    'TextEdit.paste': 'วางจากคลิปบอร์ด',
    'TextEdit.undo': 'ย้อนสิ่งที่พิมพ์ล่าสุด',
    'TextEdit.can_undo': 'ย้อนได้ไหม',
    'TextEdit.scroll_to_caret': 'เลื่อนให้เห็นเคอร์เซอร์',
    'TextEdit.read_only': 'อ่านอย่างเดียวอยู่ไหม',
    'TextEdit.set_read_only': '<code>true</code> อ่านได้ พิมพ์ไม่ได้',
    'TextEdit.set_max_length': 'พิมพ์ได้มากสุดกี่ตัว',
    'TextEdit.lines': 'ข้อความแยกเป็นบรรทัด ได้ <code>Vec&lt;String&gt;</code>',

    'Button.perform_click': 'กดปุ่มด้วยโค้ด เหมือนเอาเมาส์กด - method Click ของมันทำงาน',
    'TextBox.set_password': '<code>true</code> แสดงเป็นจุด (ช่องรหัสผ่าน)',
    'RichTextBox.load_file': 'อ่านไฟล์ข้อความเข้ามา (UTF-8)',
    'RichTextBox.save_file': 'บันทึกข้อความลงไฟล์ (UTF-8)',
    'RichTextBox.find': 'หาคำ <code>what</code> เริ่มจากตำแหน่ง <code>from</code> เจอแล้วเลือกให้ ได้ตำแหน่งที่เจอ',
    'CheckBox.checked': 'ติ๊กอยู่ไหม',
    'CheckBox.set_checked': 'ติ๊ก หรือเอาติ๊กออก',
    'CheckBox.check_state': 'สถานะ: <code>Unchecked</code>, <code>Checked</code> หรือ <code>Indeterminate</code> (ครึ่ง ๆ)',
    'CheckBox.set_check_state': 'ตั้งสถานะ รวมถึงแบบครึ่ง ๆ',
    'RadioButton.checked': 'ถูกเลือกอยู่ไหม',
    'RadioButton.set_checked': 'เลือกตัวนี้ - ตัวอื่นในกลุ่มเดียวกันจะหลุดเอง',
    'LinkLabel.link_text': 'ข้อความ (ไม่รวมเครื่องหมายลิงก์)',
    'LinkLabel.set_link_text': 'ข้อความใหม่ ทั้งหมดเป็นลิงก์ - หรือเฉพาะส่วนระหว่าง <code>&lt;a&gt;</code> กับ <code>&lt;/a&gt;</code>',
}

# More pages, a file a batch: pages_*.py beside this one, each with PAGES and TH.
import glob as _glob, importlib.util as _util, os as _os
for _f in sorted(_glob.glob(_os.path.join(_os.path.dirname(_os.path.abspath(__file__)), 'pages_*.py'))):
    _spec = _util.spec_from_file_location(_os.path.basename(_f)[:-3], _f)
    _m = _util.module_from_spec(_spec)
    _spec.loader.exec_module(_m)
    TH.update(getattr(_m, 'TH', {}))
