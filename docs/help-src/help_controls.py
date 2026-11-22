# The Toolbox pages: one page a tool (ctl-<kind>.html), the Form first. What Phai Studio knows -
# each tool's icon, the properties and events the Properties pane lists - comes from
# data/controls.json and data/icons/, which Phai Studio writes itself:
#     target\debug\phai-studio.exe --help-data docs/help-src/data
# The code on a page is quoted from a project that is built (examples/<page>/src, between
# "// help: <name>" and "// help: end"), so what a reader copies compiles.
import html, json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, 'data')
e = html.escape


def load_data():
    with open(os.path.join(DATA, 'controls.json'), encoding='utf-8') as f:
        return json.load(f)


def snippet(example, file, name):
    """The lines of examples/<example>/src/<file> between '// help: <name>' and '// help: end'."""
    path = os.path.join(HERE, 'examples', example, 'src', file)
    lines = open(path, encoding='utf-8').read().split('\n')
    start = next(i for i, l in enumerate(lines) if l.strip() == f'// help: {name}')
    end = next(i for i in range(start + 1, len(lines)) if lines[i].strip() == '// help: end')
    body = lines[start + 1:end]
    indent = min((len(l) - len(l.lstrip()) for l in body if l.strip()), default=0)
    return '<pre><code>' + e('\n'.join(l[indent:] for l in body)) + '</code></pre>'


def form_file(example, file):
    """A whole .phaiform of an example, as the reader would see it in Phai Studio's code view."""
    text = open(os.path.join(HERE, 'examples', example, 'src', file), encoding='utf-8').read().strip()
    return '<pre><code>' + e(text) + '</code></pre>'


def head(tag, title, winforms, data):
    tool = next(t for t in data['tools'] if t['tag'] == tag)
    return (f'<div class="tool-head"><img class="tool-icon" src="icons/{tag}.png" alt="" width="48" height="48">'
            f'<div><h1>{e(title)}</h1><p class="lead">ในกล่องเครื่องมือ: <b>{e(tool["label"])}</b> · '
            f'เทียบกับ WinForms: <code>{e(winforms)}</code></p></div></div>')


def props_table(rows):
    out = ['<table class="ref"><tr><th>Property</th><th>ค่าเริ่มต้น</th><th>ทำอะไร</th></tr>']
    group = None
    for r in rows:
        if r['category'] != group:
            group = r['category']
            out.append(f'<tr class="group"><td colspan="3">{e(group)}</td></tr>')
        name = ('&nbsp;&nbsp;&nbsp;' if r['part'] else '') + f'<b>{e(r["name"])}</b>'
        default = f'<code>{e(r["default"])}</code>' if r['default'] else '<span class="muted">-</span>'
        out.append(f'<tr><td>{name}</td><td>{default}</td><td>{e(r["help"])}</td></tr>')
    out.append('</table>')
    return '\n'.join(out)


def events_table(rows, form_snake, ui):
    out = ['<table class="ref"><tr><th>Event</th><th>ทำอะไร</th><th>method ที่เขียน</th></tr>']
    for r in rows:
        method = 'loaded' if r['method'] == 'loaded' else f'{form_snake}_{r["method"]}'
        arg = r['arg']
        sig = f'fn {method}(&amp;mut self, ui: &amp;{ui}' + (f', e: &amp;{e(arg.split("::")[-1])}' if arg else '') + ')'
        title = f' title="{e(arg)}"' if arg else ''
        out.append(f'<tr><td><b>{e(r["name"])}</b></td><td>{e(r["help"])}</td><td><code{title}>{sig}</code></td></tr>')
    out.append('</table>')
    return '\n'.join(out)


def methods_table(rows):
    out = ['<table class="ref"><tr><th>Method</th><th>ทำอะไร</th></tr>']
    for sig, what in rows:
        out.append(f'<tr><td><code>{e(sig)}</code></td><td>{what}</td></tr>')
    out.append('</table>')
    return '\n'.join(out)


# What a form's generated code gives the program (phai-form/src/codegen.rs writes it for every
# form): `XxxUi::` makes a window, `ui.` works on the open one.
FORM_MAKE = [
    ('XxxUi::run(events)', 'สร้างหน้าต่างแล้วทำงานจนกว่าจะปิด ใช้กับหน้าต่างหลักใน <code>main()</code> เมื่อปิดแล้วโปรแกรมจบ - เหมือน <code>Application.Run(new Form1())</code>'),
    ('XxxUi::show(owner, events)', 'เปิดหน้าต่างข้าง ๆ หน้าต่าง <code>owner</code> แล้วทำงานต่อทันที ไม่รอ - เหมือน <code>Show()</code>'),
    ('XxxUi::show_dialog(owner, events)', 'เปิดหน้าต่างทับ <code>owner</code> แล้ว<b>รอ</b>จนปิด ระหว่างนั้นกด <code>owner</code> ไม่ได้ ได้คำตอบเป็น <code>DialogResult</code> ของปุ่มที่กดปิด - เหมือน <code>ShowDialog()</code>'),
    ('XxxUi::show_dialog_for(owner, events)', 'เหมือน <code>show_dialog</code> แต่คืน struct ของหน้าต่างให้ด้วย เพื่ออ่านสิ่งที่ผู้ใช้กรอก: ได้ <code>(DialogResult, Option&lt;Xxx&gt;)</code>'),
    ('XxxUi::show_mdi_child(parent, events)', 'เปิดหน้าต่างไว้ข้างในหน้าต่าง <code>parent</code> ที่ตั้ง IsMdiContainer ไว้ - เหมือน <code>MdiParent</code> + <code>Show()</code>'),
    ('XxxUi::create(events)', 'สร้างและแสดงหน้าต่าง แต่ให้โปรแกรมวนรับข้อความเอง (ใช้น้อย)'),
]
FORM_UI = [
    ('ui.close()', 'ปิดหน้าต่าง เหมือนกดปุ่ม ✕ - event FormClosing เกิดก่อน จึงยังยกเลิกได้'),
    ('ui.hide()', 'ซ่อนหน้าต่าง (ยังทำงานอยู่)'),
    ('ui.show_window()', 'แสดงหน้าต่างที่ซ่อนไว้อีกครั้ง'),
    ('ui.activate()', 'ดึงหน้าต่างขึ้นมาข้างหน้า ให้รับแป้นพิมพ์'),
    ('ui.center_to_screen()', 'ย้ายไปกลางจอ'),
    ('ui.title()', 'อ่านข้อความบนแถบชื่อ ได้ <code>String</code>'),
    ('ui.set_title("…")', 'เปลี่ยนข้อความบนแถบชื่อ'),
    ('ui.hwnd', 'ตัวหน้าต่างเอง (HWND) ใช้ส่งให้ <code>message_box</code>, <code>show</code>, <code>show_dialog</code> เป็น owner'),
    ('ui.ชื่อcontrol', 'control ทุกตัวบนฟอร์ม เรียกตาม Id ที่ตั้งไว้ เช่น <code>ui.greeting.set_text("…")</code>'),
    ('XxxUi::begin_invoke(inv, f)', 'จากเธรดอื่น: สั่งให้ <code>f</code> ทำงานบนหน้าต่างนี้ทีหลัง ไม่รอ (ได้ <code>inv</code> จาก <code>ui.invoker()</code>) - เหมือน <code>BeginInvoke</code>'),
    ('XxxUi::invoke(inv, f)', 'เหมือนข้างบน แต่รอจน <code>f</code> ทำเสร็จ - เหมือน <code>Invoke</code>'),
    ('ui.relabel()', 'เปลี่ยนข้อความทุกตัวเป็นภาษาที่เลือกใหม่ หลัง <code>phai_ui::text::set_thai(…)</code>'),
]
FORM_HELPERS = [
    ('message_box(owner, text, caption, buttons, icon)', 'กล่องข้อความ ได้ <code>DialogResult</code> ของปุ่มที่กด - เหมือน <code>MessageBox.Show</code> ปุ่ม: <code>Ok</code>, <code>OkCancel</code>, <code>YesNo</code>, <code>YesNoCancel</code> … ไอคอน: <code>Information</code>, <code>Warning</code>, <code>Error</code>, <code>Question</code>'),
    ('close_with(hwnd, DialogResult::Ok)', 'ปิดหน้าต่างพร้อมตั้งคำตอบเอง (ปุ่มที่ตั้ง DialogResult ไว้ทำให้อัตโนมัติ)'),
    ('set_full_screen(hwnd, true)', 'เต็มจอ ไม่มีกรอบและแถบเมนู - <code>false</code> กลับเหมือนเดิม'),
    ('is_full_screen(hwnd)', 'ตอนนี้เต็มจออยู่ไหม'),
    ('set_opacity(hwnd, 80)', 'ความทึบเป็นเปอร์เซ็นต์ น้อยกว่า 100 จะเห็นสิ่งที่อยู่ข้างหลัง'),
    ('set_icon(hwnd, bytes)', 'เปลี่ยนไอคอนหน้าต่างจากไฟล์ .ico'),
]


def form_page(data):
    f = data['form']
    ui = 'MainWindowUi'
    p = []
    p.append(head('form', 'ฟอร์ม (Form)', 'Form', data))
    p.append('''<p>ฟอร์มคือ<b>หน้าต่าง</b>หนึ่งบาน โปรแกรมหนึ่งมีได้หลายฟอร์ม เช่น หน้าต่างหลัก กล่องถามชื่อ หน้าต่างตั้งค่า
เราวาง control (ปุ่ม ช่องพิมพ์ ตาราง …) ลงบนฟอร์มด้วยการลากจากกล่องเครื่องมือ แล้วเขียนโค้ดว่าเมื่อเกิดอะไรขึ้นให้ทำอะไร</p>
<figure><img class="shot" src="img/form-running.png" alt="หน้าต่างของตัวอย่างในหน้านี้ตอนรัน" width="420"><figcaption>ตัวอย่างในหน้านี้ตอนรัน</figcaption></figure>
<div class="note">กด <kbd>F1</kbd> ในตัวออกแบบตอนที่ไม่ได้เลือก control ตัวไหน จะเปิดหน้านี้ - เลือก control แล้วกด <kbd>F1</kbd> จะเปิดหน้าของ control นั้น</div>''')

    p.append('''<h2>ฟอร์มหนึ่งมี 3 ไฟล์</h2>
<table class="ref"><tr><th>ไฟล์</th><th>คืออะไร</th><th>แก้เองไหม</th></tr>
<tr><td><code>main_window.phaiform</code></td><td>หน้าตาของฟอร์ม: ขนาด ชื่อ control ทุกตัวและตำแหน่ง - ตัวออกแบบแก้ไฟล์นี้ให้เวลาลากวาง</td><td>ลากวางในตัวออกแบบ (หรือแก้มือก็ได้)</td></tr>
<tr><td><code>main_window_ui.rs</code></td><td>โค้ดที่ Phai Studio <b>เขียนให้</b>ทุกครั้งที่บันทึก: สร้างหน้าต่าง สร้าง control ต่อ event</td><td><b>ห้ามแก้</b> - บันทึกครั้งหน้าจะถูกเขียนทับ</td></tr>
<tr><td><code>main_window.rs</code></td><td><b>โค้ดของเรา</b>: ฟอร์มจำอะไรไว้ และเมื่อเกิด event แต่ละอย่างให้ทำอะไร</td><td>ใช่ - ไฟล์ของเรา</td></tr>
</table>
<p>เพิ่มฟอร์มใหม่ในโปรเจกต์: เมนู <b>Project &gt; Add Form...</b> แล้วตั้งชื่อ (หรือ <b>File &gt; New &gt; Form</b> <kbd>Ctrl</kbd>+<kbd>N</kbd>) - Phai Studio สร้างให้ครบทั้ง 3 ไฟล์</p>''')

    p.append('<h2>โครงของโค้ด</h2>')
    p.append('<p>โค้ดของฟอร์มคือ <b>struct</b> หนึ่งตัว (ฟอร์มจำอะไรไว้) กับ <code>impl MainWindowEvents</code> (เกิดอะไรขึ้นแล้วทำอะไร) และโปรแกรมเริ่มที่ <code>main()</code> ซึ่งเปิดหน้าต่างหลัก:</p>')
    p.append(snippet('form', 'main.rs', 'main'))
    p.append(snippet('form', 'main_window.rs', 'struct'))
    p.append('<p>method ทุกตัวใน <code>impl MainWindowEvents for MainWindow</code> ได้ <code>&amp;mut self</code> (ข้อมูลของฟอร์ม แก้ได้) กับ <code>ui</code> (ตัวหน้าต่างและ control ทุกตัว) - event ไหนไม่เขียน ก็ไม่ทำอะไร</p>')

    p.append('<h2>ตัวอย่าง</h2>')
    p.append('<h3>1. ทำอะไรตอนหน้าต่างเปิด (Load)</h3>')
    p.append('<p>method <code>loaded</code> ทำงานครั้งเดียวก่อนหน้าต่างแสดง - ที่ที่ดีที่สุดสำหรับใส่ค่าเริ่มต้น ดับเบิลคลิกที่พื้นฟอร์มในตัวออกแบบตอนที่โค้ดยังไม่มี method เลย Phai Studio จะเขียน <code>loaded</code> ให้ (ถ้ามีแล้วจะพาไปที่โค้ด):</p>')
    p.append(snippet('form', 'main_window.rs', 'loaded'))

    p.append('<h3>2. เปิดหน้าต่างอีกบาน (Show)</h3>')
    p.append('<p>สร้างฟอร์ม <code>NoteForm</code> ไว้ก่อน แล้วในปุ่มของหน้าต่างหลัก เรียก <code>NoteFormUi::show</code> - หน้าต่างใหม่เปิดข้าง ๆ และหน้าต่างหลักใช้ต่อได้ทันที:</p>')
    p.append(snippet('form', 'main_window.rs', 'show'))

    p.append('<h3>3. เปิดกล่องถาม แล้วอ่านคำตอบ (ShowDialog)</h3>')
    p.append('<p>ฟอร์ม <code>NameForm</code> มีช่องพิมพ์ <code>answer</code> ปุ่ม <code>ok</code> (DialogResult = Ok) และ <code>cancel</code> (DialogResult = Cancel) ฟอร์มตั้ง AcceptButton = ok และ CancelButton = cancel ไว้ จึงกด <kbd>Enter</kbd> / <kbd>Esc</kbd> ได้ด้วย:</p>')
    p.append(form_file('form', 'name_form.phaiform'))
    p.append('<p>ตอนหน้าต่างปิด (event FormClosed) เก็บสิ่งที่พิมพ์ไว้ใน struct:</p>')
    p.append(snippet('form', 'name_form.rs', 'nameform'))
    p.append('<p>หน้าต่างหลักเปิดมันด้วย <code>show_dialog_for</code> ซึ่ง<b>รอ</b>จนปิด แล้วคืนทั้งปุ่มที่กดและ struct ของ NameForm:</p>')
    p.append(snippet('form', 'main_window.rs', 'dialog'))

    p.append('<h3>4. ถามก่อนปิด (FormClosing)</h3>')
    p.append('<p>ใน Properties เลือกแท็บ Events ของฟอร์ม แล้วดับเบิลคลิก <b>FormClosing</b> - ได้ <code>e</code> มาด้วย เรียก <code>e.cancel()</code> แล้วหน้าต่างจะไม่ปิด:</p>')
    p.append(snippet('form', 'main_window.rs', 'closing'))

    p.append('<h3>5. ปุ่มลัดบนแป้นพิมพ์ (KeyDown + KeyPreview)</h3>')
    p.append('<p>ตั้ง <b>KeyPreview</b> = True ให้ฟอร์มได้ยินแป้นก่อน control ข้างใน แล้วเขียน KeyDown - ตัวอย่างนี้กด <kbd>F11</kbd> เพื่อเต็มจอ:</p>')
    p.append(snippet('form', 'main_window.rs', 'keys'))
    p.append('<p class="muted">รหัสแป้น: <code>0x0D</code> Enter · <code>0x1B</code> Esc · <code>0x70</code>–<code>0x7B</code> F1–F12 · <code>0x41</code>–<code>0x5A</code> A–Z · ดู <code>e.ctrl</code>, <code>e.shift</code>, <code>e.alt</code> ว่ากดค้างไว้ไหม</p>')

    p.append('<h2>Method: สร้างและเปิดหน้าต่าง</h2>')
    p.append('<p><code>Xxx</code> คือชื่อฟอร์ม เช่น <code>MainWindowUi::run</code>, <code>NameFormUi::show_dialog</code> · <code>events</code> คือ struct ของฟอร์มนั้น เช่น <code>NameForm::default()</code> · <code>owner</code> คือหน้าต่างที่เปิดมัน (<code>ui.hwnd</code>)</p>')
    p.append(methods_table(FORM_MAKE))
    p.append('<h2>Method: ใช้กับหน้าต่างที่เปิดอยู่</h2>')
    p.append(methods_table(FORM_UI))
    p.append('<h2>เครื่องมือเพิ่มเติมใน <code>phai_ui::form</code></h2>')
    p.append(methods_table(FORM_HELPERS))

    p.append('<h2>Properties</h2>')
    p.append('<p>ตั้งได้ในหน้าต่าง Properties ตอนไม่ได้เลือก control ตัวไหน (คลิกพื้นฟอร์ม) - ตารางนี้สร้างจาก Phai Studio เอง จึงตรงกับที่เห็นในโปรแกรมเสมอ</p>')
    p.append(props_table(f['properties']))

    p.append('<h2>Events</h2>')
    p.append(f'<p>ดับเบิลคลิกชื่อ event ในแท็บ Events ของ Properties แล้ว Phai Studio จะเขียน method ให้ (ชื่อ = ชื่อฟอร์ม + ชื่อ event เช่น <code>main_window_closing</code>) - ชี้เมาส์ที่ชนิดของ <code>e</code> เพื่อดูชื่อเต็ม</p>')
    p.append(events_table(f['events'], 'main_window', ui))

    p.append('''<h2>ลองทำเอง</h2>
<div class="try"><ol>
<li>เปลี่ยน <code>loaded</code> ให้แถบชื่อแสดงชื่อของเราเอง</li>
<li>เพิ่มปุ่ม "ออก" ที่เรียก <code>ui.close()</code> - แล้วดูว่ากล่องถามก่อนปิดยังขึ้นไหม</li>
<li>ใน NameForm ถ้าไม่ได้พิมพ์อะไรเลยแล้วกด OK ให้ขึ้น <code>message_box</code> เตือน (ใช้ event FormClosing ของ NameForm)</li>
<li>ตั้ง Opacity ของ NoteForm เป็น 85 แล้วดูว่าเห็นอะไรอยู่ข้างหลัง</li>
</ol></div>
<p class="muted">โค้ดทั้งหมดในหน้านี้อยู่ในโปรเจกต์ <code>docs/help-src/examples/form</code> ของ Phai Studio - เปิดด้วย Phai Studio แล้วกด <kbd>F5</kbd> ได้เลย</p>''')
    return '\n'.join(p)


def pages():
    """(file, title, body) of every Toolbox page there is so far."""
    data = load_data()
    return [('ctl-form.html', 'ฟอร์ม (Form)', form_page(data))]
