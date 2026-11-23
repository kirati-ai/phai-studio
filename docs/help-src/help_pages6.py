# The guide's pages as published (docs/help/*.html) - the source of truth: edit here, then run
# python docs/help-src/help_gen.py. Loaded last, over help_pages1-5.
BODY = {}

BODY['index.html'] = r'''
<h1>คู่มือ Phai Studio</h1>
<p class="lead">เครื่องมือฟรีสำหรับเขียนโปรแกรม Windows ด้วยภาษา Rust และบทเรียน Rust ภาษาไทยสำหรับคนที่เพิ่งเริ่ม</p>

<h2>เริ่มจากตรงไหนดี</h2>
<div class="cards">
  <a class="card" href="studio.html"><b>ใช้งาน Phai Studio</b><span>สร้างโปรเจกต์ ออกแบบหน้าต่าง เขียนโค้ด รัน และดีบัก</span></a>
  <a class="card" href="rust-01.html"><b>เรียน Rust ตั้งแต่ศูนย์</b><span>11 บท ตั้งแต่ตัวแปรไปจนถึงเธรด พร้อมแบบฝึกหัด</span></a>
  <a class="card" href="rust-10.html"><b>อ่าน error ของคอมไพเลอร์</b><span>เจอข้อความแดง ๆ แล้วไม่รู้จะทำอะไร เปิดหน้านี้</span></a>
  <a class="card" href="css.html"><b>ตกแต่งหน้าตาด้วย CSS</b><span>สี เส้นขอบ มุมมน ของทั้งโปรแกรม จากไฟล์เดียว</span></a>
  <a class="card" href="database.html"><b>ทำงานกับฐานข้อมูล</b><span>MySQL, PostgreSQL, SQLite ดูตาราง รัน SQL และสร้างโค้ดให้</span></a>
  <a class="card" href="code.html"><b>เขียนโค้ดกับ control</b><span>Properties, Methods, Events ชนิดของ control ในโค้ด และงานบน thread อื่นด้วย invoke</span></a>
  <a class="card" href="winforms.html"><b>control และความสามารถเพิ่มเติม</b><span>การพิมพ์ ผูกข้อมูล user control เมนู MDI และอื่น ๆ</span></a>
  <a class="card" href="publish.html"><b>ทำตัวติดตั้ง (Publish)</b><span>ทำ setup.exe ไฟล์เดียวให้คนอื่นติดตั้งโปรแกรมของคุณ</span></a>
  <a class="card" href="web.html"><b>เว็บและมือถือ (PWA)</b><span>โปรแกรมเดิมรันในเบราว์เซอร์ ลองบนมือถือด้วย QR code ติดตั้งเป็นแอปได้</span></a>
  <a class="card" href="report.html"><b>รายงานพิมพ์</b><span>ออกแบบรายงานด้วยแถบ กลุ่ม ผลรวม ดูตัวอย่าง พิมพ์ และบันทึก PDF</span></a>
</div>

<h2>ถ้าคุณไม่เคยเขียนโปรแกรมเลย</h2>
<p>ไม่เป็นไรครับ คู่มือนี้เขียนให้คนที่เริ่มจากศูนย์อ่านได้ แนะนำให้ทำตามลำดับนี้:</p>
<ol>
  <li>อ่าน <a href="studio.html">ใช้งาน Phai Studio</a> แล้วลองสร้างโปรเจกต์แรก ลากปุ่มลงฟอร์ม กด <kbd>F5</kbd> ให้เห็นหน้าต่างของตัวเองก่อน</li>
  <li>เรียนบทที่ 1 ถึง 3 ทำแบบฝึกหัด "ลองทำ" ทุกข้อ พิมพ์เองอย่าคัดลอก</li>
  <li>บทที่ 4 (Ownership) คือหัวใจของ Rust อ่านช้า ๆ ถ้ายังไม่เข้าใจทั้งหมดก็ไปต่อได้ แล้วค่อยย้อนกลับมา</li>
  <li>เมื่อเจอ error ให้เปิดบทที่ 10 คอมไพเลอร์ของ Rust ไม่ได้ดุ มันกำลังช่วยคุณ</li>
</ol>

<div class="note">Rust ยากกว่าภาษาอื่นบางภาษาในช่วงแรก เพราะคอมไพเลอร์เข้มงวดมาก แต่ความเข้มงวดนั้นแหละที่ทำให้โปรแกรมที่ผ่านคอมไพล์แล้ว แทบไม่พังแบบงง ๆ ตอนใช้งานจริง ช่วงแรกจะรู้สึกว่าโดนติบ่อย นั่นเป็นเรื่องปกติของทุกคน</div>

<h2>อยากอ่านเพิ่ม</h2>
<ul>
  <li><a href="https://doc.rust-lang.org/book/">The Rust Programming Language</a> หนังสือทางการของ Rust (ภาษาอังกฤษ) ละเอียดที่สุด</li>
  <li><a href="https://doc.rust-lang.org/rust-by-example/">Rust by Example</a> เรียนจากตัวอย่างสั้น ๆ</li>
  <li><a href="https://doc.rust-lang.org/std/">เอกสารไลบรารีมาตรฐาน</a> ไว้ค้นว่าแต่ละฟังก์ชันทำอะไร</li>
</ul>
<p>บทเรียนในคู่มือนี้เขียนขึ้นใหม่เป็นภาษาไทยสำหรับ Phai Studio โดยเฉพาะ ไม่ได้แปลจากหนังสือเล่มใด คู่มือนี้เปิดได้โดยไม่ต้องต่ออินเทอร์เน็ต</p>
'''

BODY['studio.html'] = r'''
<h1>ใช้งาน Phai Studio</h1>
<p class="lead">Phai Studio คือโปรแกรมสำหรับสร้างโปรแกรม Windows ด้วย Rust แบบเดียวกับ Visual Studio: ลากปุ่มลงหน้าต่าง เขียนโค้ด กดรัน และไล่หาบั๊กได้ในที่เดียว ใช้ฟรีและดูซอร์สโค้ดได้</p>

<h2>ก่อนเริ่ม: ติดตั้ง Rust</h2>
<p>Phai Studio ใช้ Rust ของเครื่องคุณในการ build ถ้ายังไม่มี ตอนกด build ครั้งแรก Phai Studio จะถามว่าจะเปิดเว็บ <a href="https://rustup.rs">rustup.rs</a> ไหม ให้ดาวน์โหลด <code>rustup-init.exe</code> แล้วติดตั้งตามค่าที่ให้มา เท่านั้นก็พอ</p>
<div class="note">rustup ต้องใช้ Microsoft C++ Build Tools ด้วย ตัวติดตั้งของ rustup จะบอกและพาไปติดตั้งให้ (ฟรี)</div>

<h2>หน้าตาของโปรแกรม</h2>
<table>
  <tr><th>ส่วน</th><th>อยู่ตรงไหน</th><th>ใช้ทำอะไร</th></tr>
  <tr><td>Toolbox</td><td>ซ้าย</td><td>ปุ่ม ช่องพิมพ์ รายการ ตาราง ฯลฯ ลากไปวางบนฟอร์ม</td></tr>
  <tr><td>ฟอร์ม / ตัวแก้โค้ด</td><td>กลาง</td><td>ออกแบบหน้าต่าง หรือเขียนโค้ด เปิดหลายไฟล์เป็นแท็บได้</td></tr>
  <tr><td>Project Explorer</td><td>ขวาบน</td><td>ไฟล์ทั้งหมดในโปรเจกต์ ดับเบิลคลิกเพื่อเปิด</td></tr>
  <tr><td>Properties</td><td>ขวาล่าง</td><td>ข้อความ ขนาด สี ชื่อ และ event ของสิ่งที่เลือกบนฟอร์ม</td></tr>
  <tr><td>แผงล่าง</td><td>ล่าง</td><td>Errors, Output, ผลค้นหา, ค่าตัวแปรตอนดีบัก, เทสต์ ฯลฯ</td></tr>
</table>
<p>เปลี่ยนภาษาเมนูได้ที่ <b>View &gt; ไทย / English</b> และเปลี่ยนธีมสว่าง/มืดได้ที่ <b>Tools &gt; Studio Colours</b> หรือ <b>Tools &gt; Options &gt; Environment &gt; General</b></p>

<h2>หน้าเริ่มต้นและโปรเจกต์ใหม่</h2>
<ul>
  <li>หน้าเริ่มต้นมีไฟล์ที่เปิดล่าสุด พิมพ์ในช่อง <b>Search recent</b> เพื่อกรอง คลิกขวาที่รายการเพื่อ <b>ปักหมุดไว้บนสุด</b> หรือ <b>เอาออกจากรายการ</b></li>
  <li><b>Clone a repository...</b> ใส่ที่อยู่ของ repository (https://... หรือ git@...) แล้วเลือกโฟลเดอร์ Phai Studio รัน <code>git clone</code> ให้แล้วเปิดโปรเจกต์ต่อเลย</li>
  <li>ด้านล่างมีเคล็ดลับวันละข้อ กด <b>เคล็ดลับถัดไป</b> เพื่อดูข้อต่อไป</li>
  <li><b>File &gt; New project...</b> ถามก่อนว่าจะทำแบบไหน: <b>โปรแกรม Windows</b> หรือ <b>แอปมือถือ (PWA)</b> ซึ่งได้ฟอร์มขนาดมือถือ และเปิด Device Preview ให้ดูบนจอมือถือทันที (ดู <a href="web.html">เว็บและมือถือ</a>)</li>
</ul>

<h2>หาของให้เจอ</h2>
<table>
  <tr><th>อยากทำ</th><th>ใช้</th></tr>
  <tr><td>หาคำสั่งในเมนู หน้าตั้งค่า หรือไฟล์ โดยไม่ต้องจำว่าอยู่ตรงไหน</td><td><kbd>Ctrl+Q</kbd> (Edit &gt; Search) พิมพ์ชื่อ เช่น <code>phone</code> หรือ <code>font</code> แล้วกด Enter</td></tr>
  <tr><td>ไปที่ไฟล์ ฟังก์ชัน หรือชนิดใดก็ได้</td><td><kbd>Ctrl+T</kbd> (Go to All)</td></tr>
  <tr><td>ดูไฟล์เร็ว ๆ โดยไม่เปิดแท็บค้างไว้</td><td>คลิกไฟล์ครั้งเดียวใน Project Explorer ได้ <b>แท็บพรีวิว</b> (ชื่อตัวเอียง) คลิกไฟล์อื่นจะแทนที่แท็บเดิม พอแก้ไฟล์หรือดับเบิลคลิก แท็บจะค้างไว้</td></tr>
  <tr><td>เก็บแท็บสำคัญไว้ซ้ายสุด</td><td><b>File &gt; Pin Tab</b> แท็บที่ปักหมุดมีรูปหมุด คลิกหมุดเพื่อเลิกปัก, <b>File &gt; Close All But Pinned</b> ปิดที่เหลือทั้งหมด</td></tr>
  <tr><td>จัดหน้าต่างแบบที่ชอบแล้วเรียกกลับมา</td><td><b>View &gt; Window Layouts &gt; Save Window Layout...</b> ตั้งชื่อ แล้วเรียกด้วย <kbd>Ctrl+Alt+1</kbd> ถึง <kbd>Ctrl+Alt+0</kbd>, <b>Reset Window Layout</b> คืนแบบเริ่มต้น</td></tr>
  <tr><td>ตั้งปุ่มลัดเอง</td><td><b>Tools &gt; Keyboard Shortcuts...</b> เลือกคำสั่ง กดคีย์ในช่อง (บอกว่าคีย์นี้ใครใช้อยู่) แล้วกด <b>Assign</b>, <b>Remove</b> คืนคีย์เดิม, <b>Reset All</b> คืนทั้งหมด</td></tr>
  <tr><td>ไปที่บรรทัด</td><td><kbd>Ctrl+G</kbd> ใส่เลขบรรทัด</td></tr>
  <tr><td>สลับไปไฟล์ที่เปิดอยู่</td><td><kbd>Ctrl+Tab</kbd> (กด Tab ซ้ำเพื่อเลื่อน ปล่อย Ctrl เพื่อเปิด) เรียงจากไฟล์ที่ดูล่าสุด</td></tr>
  <tr><td>แทนที่คำในทุกไฟล์</td><td><kbd>Ctrl+Shift+H</kbd> (Replace in Files) ถามก่อนแทนที่ ไฟล์ที่เปิดอยู่ Undo ได้</td></tr>
  <tr><td>ดูชนิดและเมธอดทั้งโปรเจกต์</td><td><kbd>Ctrl+Shift+C</kbd> (Class View) ดับเบิลคลิกเพื่อไปที่นั่น</td></tr>
  <tr><td>เทียบสองไฟล์</td><td><b>Tools &gt; Compare Files...</b> แสดงข้างกัน บรรทัดที่หายเป็นแดง เพิ่มเป็นเขียว แก้เป็นน้ำเงิน</td></tr>
  <tr><td>ใช้พื้นที่จอเต็ม</td><td><kbd>Shift+Alt+Enter</kbd> (Full Screen) กดอีกครั้งเพื่อกลับ</td></tr>
  <tr><td>หาค่าตั้ง</td><td><b>Tools &gt; Options</b> พิมพ์ในช่อง <b>Search Options</b> เหลือเฉพาะหน้าที่มีคำนั้น</td></tr>
  <tr><td>ย้ายค่าตั้งไปเครื่องอื่น</td><td><b>Tools &gt; Options &gt; Export...</b> ได้ไฟล์เดียว แล้ว <b>Import...</b> ที่อีกเครื่อง</td></tr>
</table>

<h2>โปรเจกต์แรก: โปรแกรมทักทาย</h2>
<ol>
  <li><b>File &gt; New project...</b> เลือกโฟลเดอร์ ตั้งชื่อ เช่น <code>hello</code></li>
  <li>จาก Toolbox ลาก <b>Text box</b> หนึ่งช่อง <b>Button</b> หนึ่งปุ่ม และ <b>Label</b> หนึ่งอันมาวางบนฟอร์ม</li>
  <li>ที่ Properties ตั้ง Name ของช่องพิมพ์เป็น <code>name</code>, ของ label เป็น <code>greeting</code> และของปุ่มเป็น <code>greet</code> พร้อมเปลี่ยน Text ของปุ่มเป็น <code>ทักทาย</code></li>
  <li>ดับเบิลคลิกที่ปุ่ม Phai Studio จะสร้างฟังก์ชันสำหรับตอนกดปุ่มและเปิดโค้ดให้</li>
  <li>เขียนในฟังก์ชันนั้นให้เป็นแบบนี้:
<pre><code>fn greet_clicked(&amp;mut self, ui: &amp;MainWindowUi) {
    let name = phai_ui::win::text_of(ui.name);          // อ่านข้อความในช่องพิมพ์
    phai_ui::win::set_text(ui.greeting, &amp;format!("สวัสดี {name}"));  // แสดงใน label
}</code></pre></li>
  <li>กด <kbd>F5</kbd> เพื่อ build และรัน พิมพ์ชื่อแล้วกดปุ่ม</li>
</ol>
<div class="note">ทุกอย่างที่ออกแบบบนฟอร์มเก็บไว้ในไฟล์ <code>.phf</code> (Phai Form; รุ่นก่อน 0.2.6 ชื่อ <code>.phaiform</code> ซึ่งยังเปิดได้ และ Studio ถามว่าจะเปลี่ยนชื่อให้ไหม) แล้ว Phai Studio แปลงเป็นโค้ด Rust ให้ตอน build โค้ดที่คุณเขียนเองอยู่ในไฟล์ <code>.rs</code> ของคุณ ไม่โดนเขียนทับ</div>

<h2>ออกแบบฟอร์ม</h2>
<ul>
  <li>ลากมุมของสิ่งที่เลือกเพื่อปรับขนาด ลากตรงกลางเพื่อย้าย จะมีเส้นช่วยจัดแนวให้</li>
  <li>เลือกหลายอันพร้อมกันได้ แล้วใช้แถบเครื่องมือจัดแนว (ชิดซ้าย จัดกลาง ระยะห่างเท่ากัน)</li>
  <li><b>Format</b> มี Bring to front, Send to back, Lock controls และ Tab order (ลำดับการกด Tab)</li>
  <li>กด <kbd>F4</kbd> เพื่อดูตัวอย่างหน้าต่างจริงโดยไม่ต้อง build, <kbd>Shift+F4</kbd> ดูบนจอมือถือและแท็บเล็ต (Device Preview)</li>
  <li><b>Format &gt; เดา Anchor ให้</b> วาง control ตามใจแล้วสั่งทีเดียว Phai Studio เดาว่าแต่ละตัวควรยืดหรือชิดขอบไหนเมื่อหน้าต่างเปลี่ยนขนาด (ช่องยาวยืดตามความกว้าง ปุ่มมุมขวาล่างชิดมุมขวาล่าง รายการยืดลง) กด <kbd>Ctrl+Z</kbd> ย้อนได้</li>
  <li>รายการและตารางในตัวออกแบบมีข้อมูลตัวอย่าง 3 แถว เดาจากชื่อคอลัมน์ (ชื่อ จำนวน ราคา วันที่ โทรศัพท์) ให้เห็นหน้าตาตอนมีข้อมูล ตัวโปรแกรมจริงเริ่มว่างเสมอ ปิดได้ที่ <b>View &gt; Sample Data in Designer</b></li>
  <li><b>ลากด้วยปุ่มขวา</b>จาก control แล้วปล่อย จะมีเมนูอีเวนต์ของมัน เลือกแล้ว Phai Studio เขียนฟังก์ชันลงโค้ดและเปิดให้ หรือเลือก <b>คัดลอก ui.ชื่อ</b> ไปวางในโค้ด (คลิกขวาเฉย ๆ ยังเป็นเมนูเดิม)</li>
  <li>ถ้าไฟล์ <code>.phf</code> เสีย (เช่น แก้มือแล้วพิมพ์ผิด) Phai Studio เปิดเป็นข้อความตรงบรรทัดที่ผิด พร้อมบอกใน Error List แก้แล้วบันทึก เปิดใหม่ก็ได้ตัวออกแบบกลับมา</li>
  <li>มี Menu bar, Context menu, Timer, Tree view, Picture, ตาราง (Grid), Rich text และแผงจัดวางอัตโนมัติ (Flow, Table) เมนูและแถบเครื่องมือออกแบบบนฟอร์มได้เลย ดู <a href="winforms.html">control และความสามารถเพิ่มเติม</a></li>
  <li>ในหน้า <b>Properties</b> ลากเส้นแบ่งระหว่างชื่อกับค่าเพื่อขยายช่องได้ (ดับเบิลคลิกที่เส้นเพื่อคืนค่าเดิม) ช่องสีเปิดหน้าต่างเลือกสีข้างจุดที่คลิก คลิกขวาที่ control แล้วเลือก Properties จะสลับไปแท็บ Properties ให้</li>
  <li>Height ของ Drop-down ใช้ได้แล้ว กล่องสูงตามที่ตั้ง ข้อความอยู่กลางกล่อง (ไม่ต่ำกว่าที่ตัวอักษรต้องใช้)</li>
</ul>

<h3>เพิ่มฟอร์มที่สอง</h3>
<p>โปรแกรมส่วนใหญ่มีมากกว่าหนึ่งหน้าต่าง เช่น หน้าตั้งค่า หรือหน้าต่างถามยืนยัน เลือก <b>Project &gt; Add Form...</b> ตั้งชื่อ เช่น <code>Settings</code> Phai Studio จะสร้างฟอร์มใหม่ในโฟลเดอร์ <code>src</code> พร้อมไฟล์โค้ด และบอก <code>main.rs</code> ให้รู้จักให้เอง แล้วเปิดให้ออกแบบต่อ</p>
<p>เปิดฟอร์มใหม่จากโค้ดของหน้าต่างหลัก เช่น ในเมธอดของปุ่ม ได้สองแบบ เหมือน WinForms:</p>
<pre><code>// เปิดคู่กัน หน้าต่างหลักยังใช้ได้ (แบบ Show)
crate::settings_ui::SettingsUi::show(ui.hwnd, crate::settings::Settings::default());

// เปิดแล้วรอจนปิด หน้าต่างหลักกดไม่ได้ระหว่างนั้น (แบบ ShowDialog)
crate::settings_ui::SettingsUi::show_dialog(ui.hwnd, crate::settings::Settings::default());</code></pre>
<p>ปิดฟอร์มที่สองแล้ว โปรแกรมยังทำงานต่อ โปรแกรมจะจบเมื่อปิดหน้าต่างหลักเท่านั้น</p>

<p>สี เส้นขอบ และมุมมนของทุก control มาจากไฟล์ <code>theme.css</code> อ่านวิธีแต่งได้ที่ <a href="css.html">ตกแต่งหน้าตาด้วย CSS</a></p>

<h2>เขียนโค้ด</h2>
<p>ตัวแก้โค้ดใช้ rust-analyzer ช่วย จึงทำได้เกือบทุกอย่างที่ Visual Studio ทำ:</p>
<ul>
  <li>พิมพ์แล้วมีรายการเติมคำ เส้นหยักใต้โค้ดที่ผิด เอาเมาส์ชี้เพื่อดูคำอธิบาย</li>
  <li><kbd>F12</kbd> ไปที่ที่ประกาศ, <kbd>Alt+F12</kbd> ดูแบบไม่ต้องออกจากไฟล์, <kbd>Shift+F12</kbd> หาทุกที่ที่ใช้</li>
  <li><kbd>F2</kbd> เปลี่ยนชื่อทั้งโปรเจกต์, <kbd>Ctrl+.</kbd> ให้ช่วยแก้หรือปรับโค้ด</li>
  <li>เหนือฟังก์ชันจะบอกว่าถูกใช้กี่แห่ง (CodeLens) คลิกเพื่อดูรายการ</li>
  <li>Snippet: พิมพ์ <code>fn</code>, <code>for</code>, <code>match</code>, <code>test</code> แล้วกด <kbd>Tab</kbd> ได้โค้ดทั้งก้อน</li>
  <li><kbd>Ctrl+K, Ctrl+D</kbd> จัดรูปแบบโค้ดทั้งไฟล์ด้วย rustfmt</li>
  <li>แถบบนตัวแก้โค้ด (breadcrumb) บอกว่าเคอร์เซอร์อยู่ในไฟล์ไหน และอยู่ใน <code>mod</code> / <code>impl</code> / <code>fn</code> ไหน</li>
  <li>หลายเคอร์เซอร์: <kbd>Ctrl+Alt+คลิก</kbd> เพิ่มเคอร์เซอร์, <kbd>Alt+ลาก</kbd> ได้เคอร์เซอร์ทุกบรรทัดเป็นกล่อง</li>
  <li><kbd>Shift+Alt+=</kbd> ขยายส่วนที่เลือกทีละขั้น (คำ → ในวงเล็บ → รวมวงเล็บ → ทั้งบรรทัด), <kbd>Shift+Alt+-</kbd> หดกลับ</li>
  <li><kbd>Ctrl+K, Ctrl+S</kbd> ครอบบรรทัดที่เลือกด้วย <code>if</code> / <code>for</code> / <code>while</code> / <code>loop</code> / บล็อก แล้วเลือกคำที่ต้องพิมพ์ทับให้</li>
  <li><kbd>Ctrl+Shift+V</kbd> วางของที่เคยคัดลอกก่อนหน้า (15 อย่างล่าสุด จากโปรแกรมไหนก็ได้)</li>
  <li>เมนู <b>View</b>: ตัดบรรทัดอัตโนมัติ (<kbd>Ctrl+E, Ctrl+W</kbd>), แสดงช่องว่าง, เส้นย่อหน้า, เส้นที่คอลัมน์ 100, แสดงชนิดตัวแปรท้ายบรรทัด (จาก rust-analyzer), สีวงเล็บเป็นคู่, หัวฟังก์ชันค้างด้านบนตอนเลื่อน (คลิกเพื่อไปที่หัว), แผนที่บนแถบเลื่อน (จุด error / การแก้ / breakpoint คลิกเพื่อไปตรงนั้น)</li>
  <li>แถบสีที่ขอบข้างบรรทัดบอกว่าแก้อะไรไปตั้งแต่ git commit ล่าสุด: <span style="color:#2ea043">เขียว</span> = เพิ่ม, <span style="color:#1c6fd1">น้ำเงิน</span> = แก้, <span style="color:#d13438">แดง</span> = ลบบรรทัดใต้ตรงนี้</li>
</ul>

<h2>Build รัน และแจกจ่าย</h2>
<table>
  <tr><th>คำสั่ง</th><th>ปุ่มลัด</th><th>ทำอะไร</th></tr>
  <tr><td>Start Debugging</td><td><kbd>F5</kbd></td><td>build แล้วรันพร้อมดีบักเกอร์ (หยุดที่ breakpoint ได้)</td></tr>
  <tr><td>Start Without Debugging</td><td><kbd>Ctrl+F5</kbd></td><td>build แล้วรันเฉย ๆ</td></tr>
  <tr><td>Build</td><td><kbd>Ctrl+B</kbd></td><td>build อย่างเดียว ข้อผิดพลาดขึ้นในแท็บ Errors</td></tr>
  <tr><td>Debug / Release</td><td>เมนู Build</td><td>Release ช้ากว่าตอน build แต่โปรแกรมเร็วกว่า ใช้ตอนแจกจริง</td></tr>
  <tr><td>Publish</td><td>เมนู Build</td><td>สร้างตัวติดตั้ง <code>setup.exe</code> ของโปรแกรมคุณ พร้อมเพิ่มไฟล์อื่นเข้าไปได้ - ทีละขั้นที่หน้า <a href="publish.html">ทำตัวติดตั้ง</a></td></tr>
  <tr><td>Manage Crates</td><td>เมนู Project</td><td>ค้นหาและเพิ่มไลบรารีจาก crates.io</td></tr>
</table>
<p>ถ้า build ไม่ผ่าน ดับเบิลคลิกบรรทัดในแท็บ Errors เพื่อไปยังจุดที่ผิด หรือกด <kbd>F8</kbd> ไล่ไปทีละข้อ ไม่เข้าใจข้อความ? ดู <a href="rust-10.html">บทที่ 10</a></p>
<ul>
  <li>Error List เลือกขอบเขตได้: <b>ทั้งโซลูชัน</b>, <b>เอกสารปัจจุบัน</b> หรือ <b>เอกสารที่เปิดอยู่</b> และคลิกหัวคอลัมน์เพื่อเรียง (คลิกซ้ำเรียงกลับ)</li>
  <li>มุมขวาของแถบสถานะบอกจำนวน error ⊗ และ warning ⚠ คลิกเพื่อเปิด Error List</li>
  <li>แท็บ Output มีช่อง <b>Show output from</b> เลือกดูเฉพาะของ Build, Debug, Tests หรือ Data หรือดูทั้งหมด</li>
  <li><b>Tools &gt; Options &gt; Environment &gt; Error List</b> ตั้งได้ว่าเมื่อ build เสร็จ ให้เปิด error แรกให้เลย ส่งเสียง หรือกะพริบที่แถบงานถ้า Phai Studio อยู่หลังหน้าต่างอื่น</li>
</ul>

<h2>ดีบัก: หาว่าโปรแกรมผิดตรงไหน</h2>
<ol>
  <li>คลิกที่ขอบซ้ายของบรรทัด (หรือกด <kbd>F9</kbd>) จะได้จุดแดง เรียกว่า breakpoint</li>
  <li>กด <kbd>F5</kbd> โปรแกรมจะหยุดเมื่อมาถึงบรรทัดนั้น มีลูกศรเหลืองบอกตำแหน่ง</li>
  <li>แท็บ <b>Locals</b> แสดงค่าตัวแปรทุกตัว ณ ตอนนั้น ดับเบิลคลิกเพื่อแก้ค่าได้</li>
  <li>เดินทีละบรรทัดด้วย <kbd>F10</kbd> (ข้ามฟังก์ชัน) หรือ <kbd>F11</kbd> (เข้าไปในฟังก์ชัน), <kbd>Shift+F11</kbd> ออกจากฟังก์ชัน</li>
  <li>กด <kbd>F5</kbd> อีกครั้งเพื่อไปต่อ หรือ <kbd>Shift+F5</kbd> เพื่อหยุด</li>
</ol>
<h3>ไปต่อแบบอื่น</h3>
<ul>
  <li><kbd>Ctrl+F10</kbd> <b>Run To Cursor</b>: วางเคอร์เซอร์ที่บรรทัดไหนก็ได้ แล้วรันจนถึงบรรทัดนั้น</li>
  <li><kbd>Ctrl+Shift+F10</kbd> <b>Set Next Statement</b>: ให้โปรแกรมไปต่อจากบรรทัดที่เคอร์เซอร์อยู่ (ข้ามหรือทำซ้ำบรรทัดได้ เฉพาะในฟังก์ชันเดียวกัน)</li>
  <li><b>Debug &gt; Insert Tracepoint...</b>: แทนที่จะหยุด ให้เขียนข้อความลง Output แล้วไปต่อ เช่น <code>total is {total}</code> ขอบซ้ายเป็นรูปข้าวหลามตัดสีแดง</li>
  <li>เมื่อ panic หรือ exception: กล่องแดงขึ้นใต้บรรทัดในโค้ดของคุณที่ทำให้เกิด บอกว่าเกิดอะไร และข้อความของ panic</li>
  <li>แท็บ Locals มีตัวเลือก <b>Autos</b>: แสดงเฉพาะตัวแปรที่บรรทัดนี้และบรรทัดก่อนใช้</li>
</ul>
<h3>เครื่องมือดีบักเพิ่มเติม</h3>
<table>
  <tr><th>เครื่องมือ</th><th>ปุ่มลัด</th><th>ใช้เมื่อ</th></tr>
  <tr><td>Breakpoint Condition</td><td><kbd>Alt+F9</kbd></td><td>หยุดเฉพาะเมื่อเงื่อนไขเป็นจริง เช่น <code>i == 50</code> หรือเมื่อผ่านครบกี่ครั้ง</td></tr>
  <tr><td>Watch</td><td><kbd>Shift+F9</kbd></td><td>เฝ้าดูค่านิพจน์ที่สนใจทุกครั้งที่หยุด</td></tr>
  <tr><td>Immediate Window</td><td><kbd>Ctrl+Alt+I</kbd></td><td>พิมพ์ชื่อตัวแปรหรือนิพจน์ แล้วได้ค่าทันที</td></tr>
  <tr><td>Call Stack</td><td>แท็บล่าง</td><td>มาถึงตรงนี้ได้อย่างไร ใครเรียกใคร</td></tr>
  <tr><td>Breakpoints</td><td><kbd>Ctrl+Alt+B</kbd></td><td>รวมทุก breakpoint เปิด/ปิดชั่วคราวได้</td></tr>
  <tr><td>Threads</td><td><kbd>Ctrl+Alt+H</kbd></td><td>โปรแกรมที่มีหลายเธรด ดูว่าแต่ละเธรดอยู่ตรงไหน</td></tr>
  <tr><td>New Data Breakpoint</td><td>เมนู Debug</td><td>หยุดเมื่อค่าตัวแปรเปลี่ยน (หาว่าใครแก้ค่านี้)</td></tr>
  <tr><td>Exception Settings</td><td><kbd>Ctrl+Alt+E</kbd></td><td>เลือกว่าจะหยุดเมื่อ panic หรือ exception แบบไหน</td></tr>
  <tr><td>Memory / Disassembly</td><td><kbd>Ctrl+Alt+M</kbd> / <kbd>Ctrl+Alt+D</kbd></td><td>ดูไบต์ในหน่วยความจำ และภาษาเครื่อง (สำหรับผู้ชำนาญ)</td></tr>
  <tr><td>Diagnostic Tools</td><td><kbd>Ctrl+Alt+F2</kbd></td><td>กราฟหน่วยความจำและ CPU ของโปรแกรมที่รันอยู่</td></tr>
  <tr><td>Attach to Process</td><td><kbd>Ctrl+Alt+P</kbd></td><td>ดีบักโปรแกรมที่รันอยู่แล้ว</td></tr>
  <tr><td>Modules</td><td><kbd>Ctrl+Alt+U</kbd></td><td>DLL ทุกตัวที่โปรแกรมโหลด และตำแหน่งในหน่วยความจำ</td></tr>
</table>

<h2>เทสต์</h2>
<ul>
  <li><kbd>Ctrl+R, T</kbd> รันเทสต์ของโปรเจกต์, <kbd>Ctrl+R, A</kbd> รันทั้งหมด ผลขึ้นในแท็บ Tests</li>
  <li>ข้างฟังก์ชันที่มี <code>#[test]</code> มี <span style="color:#2ea043">▶</span> สีเขียวที่ขอบซ้าย คลิกเพื่อรันเทสต์นั้นเทสต์เดียว</li>
  <li><b>Tools &gt; Test &gt; Analyze Code Coverage</b> บอกว่าเทสต์รันถึงบรรทัดไหนบ้าง แถบเขียวที่ขอบซ้ายคือรันถึง แถบแดงคือยังไม่มีเทสต์</li>
  <li>วิธีเขียนเทสต์ดูได้ใน <a href="rust-09.html">บทที่ 9</a></li>
</ul>

<h2>Git</h2>
<ul>
  <li>เมนู Git อยู่ที่ <b>Tools &gt; Git</b></li>
  <li>มุมขวาของแถบสถานะบอกสาขาปัจจุบัน คลิกเพื่อสลับสาขา สร้างสาขาใหม่ เก็บการแก้ไว้ก่อน (Stash) หรือเอาคืน (Pop Stash) ไฟล์ที่เปิดอยู่จะอ่านใหม่ให้</li>
  <li><b>Git Changes</b> (<kbd>Ctrl+Alt+G</kbd>) ดูไฟล์ที่แก้ เลือก Stage แล้ว Commit พร้อม push/pull</li>
  <li><b>View History of This File</b> ดูทุกครั้งที่ไฟล์นี้ถูกแก้ และแก้อะไรไป</li>
  <li><b>Show Line Blame</b> บอกว่าใครแก้บรรทัดที่เคอร์เซอร์อยู่ เมื่อไร</li>
  <li><b>Resolve Merge Conflicts</b> เมื่อ merge แล้วชนกัน เลือกว่าจะใช้ของฝั่งไหน</li>
</ul>

<h2>ฐานข้อมูล</h2>
<p><b>Tools &gt; Data &gt; Add Connection...</b> เลือกการ์ดของฐานข้อมูลที่จะต่อ MySQL/MariaDB, PostgreSQL หรือไฟล์ SQLite ได้ ดูตารางใน Server Explorer (<kbd>Ctrl+Alt+S</kbd>) เขียน SQL แล้วกด <kbd>Ctrl+Shift+E</kbd> เพื่อรัน แก้ข้อมูลในตารางผลลัพธ์ได้โดยตรง</p>
<p>ลากตารางจาก Server Explorer ไปวางบนฟอร์ม ได้ตารางที่ผูกกับตารางนั้นพร้อมโหลดและบันทึก (เหมือนคลิกขวา &gt; <b>Add to Form</b>)</p>
<p>อ่านละเอียดที่ <a href="database.html">ทำงานกับฐานข้อมูล</a></p>

<h2 id="ai">ผู้ช่วย AI</h2>
<p><b>View &gt; AI Assistant</b> (<kbd>Ctrl+Alt+A</kbd>) เปิดแผง <b>AI</b> ข้าง Project และ Properties คุยกับ Claude เรื่องโปรเจกต์ที่เปิดอยู่ได้เลย พิมพ์แล้วกด <b>ส่ง</b> หรือ <kbd>Ctrl+Enter</kbd></p>
<figure><img class="shot" src="img/ai-pane.png" alt="แผงผู้ช่วย AI ตอบคำถามเรื่องโปรเจกต์" width="300"><figcaption>Claude ดูรายชื่อไฟล์ของโปรเจกต์เองก่อนตอบ</figcaption></figure>
<ul>
  <li><b>ถาม</b> — ตอบและอธิบายอย่างเดียว ไม่แก้ไฟล์ใด ๆ เช่น "ปุ่ม OK ทำอะไร" "ทำไม build ไม่ผ่าน"</li>
  <li><b>ลงมือ</b> — บอกสิ่งที่ต้องการ เช่น "เพิ่มช่องกรอกอีเมลใต้ชื่อ แล้วตรวจว่ามี @" Claude อ่านไฟล์ที่ต้องใช้ แก้โค้ดและฟอร์ม (.phf) build แล้วแก้ error จนผ่าน รันเทสต์ และถ่ายภาพหน้าจอโปรแกรมเพื่อดูหน้าตาที่ทำ</li>
  <li><b>ทุกการแก้ไฟล์ขึ้นหน้าต่างเทียบ</b> ซ้ายคือไฟล์ตอนนี้ ขวาคือที่จะเป็น กด <b>รับ</b>, <b>รับทั้งหมดในแชตนี้</b> หรือ <b>ไม่รับ</b> — ไม่มีอะไรถูกเขียนลงไฟล์ถ้าไม่รับ ฟอร์มที่ Phai Studio อ่านไม่ได้จะไม่ถูกเขียนเลย</li>
  <li>เลือกโมเดลได้: Claude Opus 5.5 (ค่าเริ่มต้น เก่งที่สุด), Sonnet 5.5 (เร็วกว่า ถูกกว่า), Haiku 5.5 (เร็วและถูกที่สุด) — <b>เริ่มใหม่</b> ล้างแชต</li>
</ul>
<figure><img class="shot" src="img/ai-review.png" alt="หน้าต่างเทียบการแก้ไฟล์ของ Claude" width="560"><figcaption>Claude ขอแก้ main_window.rs: บรรทัดใหม่มีแถบเขียว</figcaption></figure>
<p><b>คีย์ API:</b> ผู้ช่วย AI ใช้ Claude ของ Anthropic ผ่านคีย์ของคุณเอง สมัครและสร้างคีย์ที่ <code>console.anthropic.com</code> แล้วกด <b>คีย์ API...</b> ในแผง คีย์เก็บใน Credential Manager ของ Windows ไม่อยู่ในไฟล์ใด ๆ ของ Phai Studio ค่าใช้ Claude ทาง Anthropic เรียกเก็บจากคุณตามที่ใช้ (Phai Studio เองยังฟรี) — ส่งข้อมูลไปเฉพาะตอนกดส่ง: สิ่งที่พิมพ์ และส่วนของโปรเจกต์ที่ Claude ขออ่าน</p>

<h2>ปุ่มลัดที่ใช้บ่อย</h2>
<table>
  <tr><th>ปุ่ม</th><th>ทำอะไร</th><th>ปุ่ม</th><th>ทำอะไร</th></tr>
  <tr><td><kbd>Ctrl+S</kbd></td><td>บันทึก</td><td><kbd>Ctrl+F</kbd> / <kbd>Ctrl+H</kbd></td><td>ค้นหา / แทนที่</td></tr>
  <tr><td><kbd>Ctrl+Shift+F</kbd></td><td>ค้นหาในทุกไฟล์</td><td><kbd>Ctrl+T</kbd></td><td>ไปที่ไฟล์ ฟังก์ชัน หรือชนิดใดก็ได้</td></tr>
  <tr><td><kbd>Ctrl+/</kbd></td><td>เปิด/ปิด comment</td><td><kbd>Ctrl+D</kbd></td><td>ทำสำเนา</td></tr>
  <tr><td><kbd>Alt+↑</kbd> <kbd>Alt+↓</kbd></td><td>ย้ายบรรทัดขึ้นลง</td><td><kbd>Ctrl+M, Ctrl+M</kbd></td><td>พับ/กางโค้ด</td></tr>
  <tr><td><kbd>Ctrl+K, Ctrl+K</kbd></td><td>คั่นหน้า (bookmark)</td><td><kbd>Ctrl+Alt+T</kbd></td><td>โครงร่างของไฟล์ (Document Outline)</td></tr>
  <tr><td><kbd>Alt+F7</kbd></td><td>ตั้งค่าโปรเจกต์</td><td><kbd>Ctrl+Shift+B</kbd></td><td>Build ทั้ง solution</td></tr>
  <tr><td><kbd>Ctrl+Q</kbd></td><td>ค้นหาคำสั่ง ค่าตั้ง ไฟล์</td><td><kbd>Shift+F4</kbd></td><td>ดูฟอร์มบนมือถือ/แท็บเล็ต</td></tr>
  <tr><td><kbd>Ctrl+Alt+1</kbd>…<kbd>0</kbd></td><td>เรียกการจัดหน้าต่างที่บันทึกไว้</td><td><kbd>Ctrl+`</kbd></td><td>Terminal</td></tr>
  <tr><td><kbd>Ctrl+Alt+A</kbd></td><td>ผู้ช่วย AI</td><td><kbd>Ctrl+Enter</kbd></td><td>ส่งข้อความถึง Claude</td></tr>
</table>
'''

BODY['css.html'] = r'''
<h1>ตกแต่งหน้าตาด้วย CSS</h1>
<p class="lead">สีพื้น สีตัวอักษร เส้นขอบ มุมมน และตัวหนา ของทุกปุ่มทุกช่องในโปรแกรม มาจากไฟล์ <code>theme.css</code> ไฟล์เดียว แก้ที่เดียวแล้วเปลี่ยนทั้งโปรแกรม ไม่ต้องไล่แก้ทีละปุ่ม</p>

<h2>ทำไมใช้ CSS</h2>
<p>ถ้าเคยทำเว็บมาก่อน จะคุ้นกับ CSS อยู่แล้ว Phai Studio ใช้ CSS ส่วนเล็ก ๆ ที่พอสำหรับบอกว่า control แต่ละแบบ หน้าตาเป็นอย่างไรในแต่ละสถานะ ตัว control ยังเป็นของ Windows จริงทุกตัว CSS แค่บอกสีกับเส้น</p>
<ul>
  <li><b>ทั้งโปรแกรมเป็นสไตล์เดียวกัน</b> ปุ่มทุกปุ่มหน้าตาเหมือนกัน เพราะมาจากกฎเดียวกัน</li>
  <li><b>เปลี่ยนธีมง่าย</b> อยากได้สีหลักเป็นสีน้ำเงิน แก้บรรทัดเดียว</li>
  <li><b>เห็นผลทันที</b> บันทึกไฟล์ CSS แล้วหน้าต่างที่เปิดอยู่เปลี่ยนตามเลย ทั้งในตัวออกแบบและในโปรแกรมที่กำลังรัน</li>
</ul>

<h2>ไฟล์ theme.css อยู่ตรงไหน</h2>
<p>ตอนสร้างโปรเจกต์ใหม่ Phai Studio ใส่ <code>theme.css</code> ไว้ในโฟลเดอร์โปรเจกต์ให้แล้ว เป็นธีมมาตรฐานสีสว่าง มีคำอธิบายทุกบรรทัด เปิดจากหน้าต่าง Project แล้วแก้ได้เลย ตอน build ธีมถูกรวมเข้าไปในโปรแกรม ไม่ต้องแจกไฟล์ CSS แยก</p>
<table>
  <tr><th>เมนู Theme</th><th>ทำอะไร</th></tr>
  <tr><td>New theme from the standard...</td><td>สร้าง <code>theme.css</code> จากธีมมาตรฐาน ข้างไฟล์ฟอร์ม (ฟอร์มต้องบันทึกก่อน)</td></tr>
  <tr><td>Import CSS...</td><td>เลือกไฟล์ CSS ที่มีอยู่แล้ว Phai Studio คัดลอกเข้าโปรเจกต์ให้ แล้วใช้เป็นธีม</td></tr>
  <tr><td>No theme</td><td>เลิกใช้ไฟล์ธีม กลับไปใช้ธีมในตัว</td></tr>
</table>
<div class="note">เมนู View &gt; Light / Dark เปลี่ยนสีของตัว Phai Studio เอง ไม่เกี่ยวกับธีมของโปรแกรมที่คุณสร้าง</div>

<h2>หน้าตาของกฎ</h2>
<pre><code>:root { --accent: #12855f; }

button          { background: #ffffff; border-color: #d4d4d4; border-radius: 4px; }
button:hover    { background: #e7f3ef; border-color: var(--accent); }
button.primary  { background: var(--accent); color: #ffffff; }
#save           { font-weight: 600; }</code></pre>
<p>อ่านว่า: ปุ่มทุกปุ่มพื้นขาว ขอบเทา มุมมน 4 พิกเซล พอเอาเมาส์ชี้ให้พื้นเป็นเขียวอ่อน ปุ่มที่มี class <code>primary</code> เป็นปุ่มสีเขียวตัวหนังสือขาว และปุ่มที่ id เป็น <code>save</code> ให้ตัวหนา</p>

<h2>ตัวเลือก (selector)</h2>
<table>
  <tr><th>เขียนแบบนี้</th><th>หมายถึง</th></tr>
  <tr><td><code>button</code></td><td>control ทุกตัวชนิดนั้น: <code>window label button checkbox radio edit combo list group tabs toolbar statusbar date progress slider menubar tree</code> (ช่องตัวเลขนับเป็น <code>edit</code>)</td></tr>
  <tr><td><code>*</code></td><td>ทุก control</td></tr>
  <tr><td><code>.primary</code></td><td>control ที่ใส่ class นี้ไว้ ในช่อง Class ของหน้าต่าง Properties</td></tr>
  <tr><td><code>#save</code></td><td>control ตัวเดียว ตาม Id ของมัน</td></tr>
  <tr><td><code>:hover :pressed :focus :disabled :selected :checked</code></td><td>สถานะ: เมาส์ชี้ กดค้าง กำลังพิมพ์ ใช้ไม่ได้ ถูกเลือก ถูกติ๊ก ใส่ต่อกันได้ เช่น <code>button.primary:hover</code></td></tr>
  <tr><td><code>button, label</code></td><td>หลายตัวเลือกใช้กฎเดียวกัน คั่นด้วยจุลภาค</td></tr>
</table>
<p>กฎที่เจาะจงกว่าชนะ: id ชนะ class และสถานะ ซึ่งชนะชนิดของ control ถ้าเจาะจงเท่ากัน กฎที่อยู่ล่างกว่าชนะ เหมือนในเว็บ</p>
<div class="note">ไม่มี <code>a b</code> หรือ <code>a &gt; b</code> (ตัวเลือกแบบซ้อนกัน) เพราะหน้าต่างของ Windows วาง control แบน ๆ ไม่ได้ซ้อนเป็นชั้นแบบหน้าเว็บ กฎแบบนั้นจะเดาผลยาก</div>

<h2>สิ่งที่ตั้งได้</h2>
<table>
  <tr><th>property</th><th>ค่า</th></tr>
  <tr><td><code>background</code> <code>color</code> <code>border-color</code></td><td>สี: <code>#rgb</code> <code>#rrggbb</code> <code>#rrggbbaa</code> <code>rgb(r, g, b)</code> ชื่อสีอย่าง <code>white</code> <code>black</code> <code>transparent</code> หรือ <code>var(--ชื่อ)</code></td></tr>
  <tr><td><code>border-width</code> <code>border-radius</code></td><td>พิกเซลที่ขนาดจอ 100% เช่น <code>4</code> หรือ <code>4px</code></td></tr>
  <tr><td><code>accent-color</code></td><td>สีเครื่องหมายถูก จุดของ radio และเส้นใต้แท็บที่เลือก</td></tr>
  <tr><td><code>icon-color</code></td><td>สีไอคอนในแถบเครื่องมือ</td></tr>
  <tr><td><code>font-size</code> <code>font-weight</code></td><td>ขนาดเป็น point เช่น <code>10pt</code> และความหนา <code>400</code> <code>600</code> <code>700</code> หรือ <code>bold</code></td></tr>
</table>

<h2>ตัวแปร: เปลี่ยนสีหลักทั้งโปรแกรมในบรรทัดเดียว</h2>
<p>ตั้งค่าไว้ใน <code>:root</code> แล้วเรียกใช้ด้วย <code>var(--ชื่อ)</code> ธีมมาตรฐานใช้ <code>--accent</code> เป็นสีหลัก เปลี่ยนตัวเดียว ปุ่ม แท็บ และช่องที่ถูกเลือก เปลี่ยนตามหมด</p>
<pre><code>:root { --accent: #2563eb; }   /* จากเขียวเป็นน้ำเงิน */</code></pre>

<h2>แก้นิดเดียว ไม่ต้องคัดลอกทั้งไฟล์</h2>
<p>ถ้าชอบธีมมาตรฐานอยู่แล้ว แค่อยากเปลี่ยนบางอย่าง เขียนไฟล์ของตัวเองสั้น ๆ แบบนี้ได้</p>
<pre><code>@import "phai:light";          /* หรือ "phai:dark" ธีมมืด */

:root  { --accent: #2563eb; }
button { border-radius: 10px; }</code></pre>
<p><code>@import "other.css";</code> ดึงไฟล์อื่นที่อยู่ข้างกันเข้ามาได้ด้วย</p>

<h2>แก้ทีละตัวในหน้าต่าง Properties</h2>
<p>ในหัวข้อ Appearance ของหน้าต่าง Properties มี Back colour, Fore colour, Border colour, Corner radius และ Font ตั้งให้ control ตัวเดียวได้ ค่าที่ตั้งตรงนี้ทับธีม เหมาะกับจุดพิเศษที่มีจุดเดียว ถ้าหลายตัวต้องหน้าตาเหมือนกัน ใส่ class ให้ทั้งหมด แล้วเขียนกฎใน CSS ทีเดียวดีกว่า</p>

<h2>ถ้าเขียนผิด</h2>
<p>ถ้า CSS มีจุดที่อ่านไม่ออก Phai Studio จะบอกว่าผิดบรรทัดไหน และหน้าต่างใช้ธีมเดิมไปก่อน ไม่ข้ามกฎที่ผิดไปเงียบ ๆ เพราะธีมที่ใช้ได้ครึ่งเดียว จะดูเหมือน control เสีย</p>

<h2>ลองทำ</h2>
<ol>
  <li>ลากปุ่มลงฟอร์ม ในหน้าต่าง Properties พิมพ์ <code>danger</code> ในช่อง Class</li>
  <li>เปิด <code>theme.css</code> แล้วเพิ่มกฎ <code>button.danger { background: #dc2626; color: white; }</code> บันทึกไฟล์</li>
  <li>ดูปุ่มในตัวออกแบบเปลี่ยนเป็นสีแดง แล้วลองเพิ่ม <code>button.danger:hover</code> ให้สีเข้มขึ้นตอนเอาเมาส์ชี้</li>
</ol>

<h2>ภาพเคลื่อนไหว: animation และ @keyframes</h2>
<p><code>transition</code> ทำให้สีค่อย ๆ เปลี่ยนตอนสถานะเปลี่ยน (เช่น เอาเมาส์ชี้) ส่วน <code>animation</code> ทำให้สีเคลื่อนไหวเองโดยไม่ต้องรอใคร ซ้ำไปเรื่อย ๆ หรือกี่รอบก็ได้ เหมาะกับปุ่มที่อยากให้คนเห็น หรือสถานะเตือน</p>
<pre><code>/* สำเร็จรูป: blink (ตัวหนังสือกะพริบ) pulse (พื้นเต้นเป็นสี accent) fade-in (ค่อย ๆ ปรากฏ) */
button.urgent { animation: pulse 1.5s infinite; }
#warning      { animation: blink 1s infinite; }
#panel        { animation: fade-in 400ms; }

/* ทำเอง */
@keyframes alarm {
  50% { background: #dc2626; color: #ffffff; border-color: #991b1b; }
}
#stop { animation: alarm 2s ease-in-out infinite alternate; }</code></pre>
<table>
  <tr><th>ส่วนของ animation</th><th>หมายถึง</th></tr>
  <tr><td>ชื่อ</td><td><code>blink</code> <code>pulse</code> <code>fade-in</code> หรือชื่อ <code>@keyframes</code> ที่เขียนเอง</td></tr>
  <tr><td>เวลาแรก</td><td>หนึ่งรอบนานเท่าไร เช่น <code>1s</code> <code>400ms</code> เวลาที่สองคือรอก่อนเริ่ม</td></tr>
  <tr><td>easing</td><td><code>ease</code> <code>linear</code> <code>ease-in</code> <code>ease-out</code> <code>ease-in-out</code></td></tr>
  <tr><td>จำนวนรอบ</td><td>ตัวเลข หรือ <code>infinite</code> ไม่หยุด (ไม่ใส่ = รอบเดียว)</td></tr>
  <tr><td><code>alternate</code></td><td>รอบคู่เดินย้อนกลับ ไปกลับนุ่ม ๆ</td></tr>
</table>
<p>ใน <code>@keyframes</code> เขียน <code>from</code>, <code>to</code> หรือเปอร์เซ็นต์ (<code>0%, 100% { }</code> ก็ได้) เฟรมที่ไม่ได้เขียนใช้หน้าตาปกติของ control ขยับได้เฉพาะสี (<code>background color border-color accent-color icon-color</code>) และ <code>border-width</code> เพราะ control ของ Windows หมุนหรือโปร่งใสแบบหน้าเว็บไม่ได้ <code>fade-in</code> จึงเป็นการค่อย ๆ เปลี่ยนสีจากสีพื้นหน้าต่าง</p>
<p>ไม่ต้องเขียน CSS เองก็ได้: เลือก control แล้วไปที่แท็บ <b>Style</b> 🖌 ในหน้าต่าง Properties มีช่อง <b>Animation</b> ให้เลือกแบบสำเร็จรูป (ใช้กับปุ่ม ช่องติ๊ก radio ช่องกรอก slider และ progress)</p>
'''

BODY['database.html'] = r'''
<h1>ทำงานกับฐานข้อมูล</h1>
<p class="lead">หน้าต่าง Server Explorer ของ Phai Studio ต่อฐานข้อมูล MySQL / MariaDB, PostgreSQL และไฟล์ SQLite ได้ ดูตาราง ดูข้อมูล เขียนคำสั่ง SQL และสร้างโค้ด Rust สำหรับอ่านตารางให้ เหมือน Server Explorer ของ Visual Studio</p>

<h2>เปิด Server Explorer</h2>
<p>เมนู View &gt; Server Explorer หรือ Data &gt; Server Explorer หรือกด <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>S</kbd> หน้าต่างจะอยู่ข้างหน้าต่าง Project มีหัวข้อ Data Connections เป็นรากของทุกการเชื่อมต่อ</p>

<h2>เพิ่มการเชื่อมต่อ</h2>
<ol>
  <li>Data &gt; Add Connection... หรือคลิกขวาที่ Data Connections แล้วเลือก Add Connection...</li>
  <li>เลือกชนิด: MySQL / MariaDB, PostgreSQL หรือ SQLite file...</li>
  <li>ถ้าเป็น MySQL หรือ PostgreSQL กรอก Server (เช่น <code>localhost</code>) Port User Password และ Database (เว้นว่างได้ จะเห็นทุกฐานข้อมูลที่มีสิทธิ์)</li>
  <li>ถ้าเป็น SQLite เลือกไฟล์ <code>.db</code> ที่มีอยู่ หรือพิมพ์ชื่อไฟล์ใหม่ SQLite จะสร้างให้</li>
  <li>Phai Studio ลองเชื่อมต่อก่อนเก็บ ถ้าสำเร็จจะบอกชนิดและเวอร์ชันของฐานข้อมูลที่แถบสถานะ</li>
</ol>
<table>
  <tr><th>ฐานข้อมูล</th><th>พอร์ตปกติ</th><th>ผู้ใช้ที่ใส่ไว้ให้</th></tr>
  <tr><td>MySQL / MariaDB</td><td>3306</td><td>root</td></tr>
  <tr><td>PostgreSQL</td><td>5432</td><td>postgres</td></tr>
  <tr><td>SQLite</td><td>ไม่มี (เป็นไฟล์)</td><td>ไม่มี</td></tr>
</table>
<div class="note">ถ้าต่อเซิร์ฟเวอร์ข้ามอินเทอร์เน็ต ให้ติ๊ก <b>Encrypt the connection (TLS)</b> ข้อมูลกับรหัสผ่านจะถูกเข้ารหัสระหว่างทาง ส่วน <b>Trust the server's own certificate</b> ใช้กับเซิร์ฟเวอร์ที่ใช้ใบรับรองที่ทำเอง Phai Studio จำการเชื่อมต่อไว้ให้ รหัสผ่านถูกเก็บแบบเข้ารหัสด้วยระบบของ Windows</div>

<h2>ดูโครงสร้าง</h2>
<p>กดลูกศรหน้าการเชื่อมต่อ จะเห็นฐานข้อมูล ใต้ฐานข้อมูลคือตารางและ view ใต้ตารางคือคอลัมน์ พร้อมชนิดข้อมูล และบอกว่าเป็นคีย์หลักหรือเว้นว่างได้ไหม Phai Studio อ่านจากเซิร์ฟเวอร์ตอนเปิดกิ่งนั้นครั้งแรก ถ้ามีคนแก้ฐานข้อมูลระหว่างนั้น คลิกขวาแล้วเลือก Refresh</p>

<h2>คลิกขวาทำอะไรได้บ้าง</h2>
<table>
  <tr><th>คลิกขวาที่</th><th>คำสั่ง</th></tr>
  <tr><td>การเชื่อมต่อ</td><td>New Query, New Database..., Refresh, Close Connection, Add Connection..., Delete Connection</td></tr>
  <tr><td>ฐานข้อมูล</td><td>New Query, New Table..., Refresh</td></tr>
  <tr><td>ตาราง</td><td>Show Table Data, Design Table, Script as SQL, New Query, Copy code for a program, Add to Form, Refresh</td></tr>
  <tr><td>view</td><td>Show Table Data, New Query, Refresh</td></tr>
</table>
<ul>
  <li><b>Show Table Data</b> แสดงข้อมูลในตาราง ที่แท็บ Results ด้านล่าง แก้ค่าในตารางผลลัพธ์ได้โดยตรง</li>
  <li><b>Design Table</b> เปิดตัวออกแบบฐานข้อมูล (Database Designer) เป็นหน้าต่างของตัวเอง ด้านซ้ายเลือกฐานข้อมูลและตารางได้แบบ phpMyAdmin ด้านขวาเป็นแท็บ Columns, Indexes และ Foreign Keys แก้ชื่อ ชนิดข้อมูล เว้นว่างได้ไหม คีย์หลัก ตัวเลขรันอัตโนมัติ และค่าเริ่มต้น ได้ด้วยการคลิก ด้านล่างมีคำสั่ง SQL ที่จะใช้แก้ตาราง เขียนให้ดูไปพร้อมกัน กด Update หรือ <kbd>Ctrl</kbd>+<kbd>S</kbd> เพื่อแก้ตารางจริง</li>
  <li><b>New Table...</b> เปิดตัวออกแบบเดียวกัน สำหรับสร้างตารางใหม่ในฐานข้อมูลนั้น</li>
  <li><b>New Database...</b> (คลิกขวาที่การเชื่อมต่อ หรือปุ่มในตัวออกแบบ) สร้างฐานข้อมูลใหม่ MySQL ใช้ utf8mb4 PostgreSQL ใช้ UTF8 เก็บภาษาไทยได้ สร้างเสร็จเปิดตารางแรกให้ออกแบบต่อ ถ้าการเชื่อมต่อไม่ได้ระบุฐานข้อมูลและบนเซิร์ฟเวอร์ยังไม่มีเลย Phai Studio จะให้สร้างก่อน</li>
  <li><b>Close Connection</b> พับการเชื่อมต่อและปล่อยที่อ่านไว้ กางออกอีกครั้งเพื่อเชื่อมต่อใหม่</li>
  <li><b>Script as SQL</b> เปิดคำสั่ง <code>CREATE TABLE</code> ของตารางนี้ พร้อมตัวอย่างคำสั่งแก้ตารางที่ใส่ <code>--</code> ไว้ เอา <code>--</code> หน้าบรรทัดที่ต้องการออก แล้วกด <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> เพื่อแก้ตาราง</li>
</ul>

<h2>เขียนและรันคำสั่ง SQL</h2>
<ol>
  <li>Data &gt; New Query หรือคลิกขวาแล้ว New Query จะเปิดไฟล์ <code>Query1.sql</code> บรรทัดแรกบอกว่าต่อกับเซิร์ฟเวอร์และฐานข้อมูลไหน</li>
  <li>พิมพ์คำสั่ง เช่น <code>SELECT * FROM customers WHERE active = 1;</code></li>
  <li>กด <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> (Data &gt; Execute Query) ผลลัพธ์จะขึ้นที่แท็บ Results ถ้าเป็นคำสั่งแก้ข้อมูล จะบอกว่าเปลี่ยนไปกี่แถว</li>
</ol>
<p>ใส่หลายคำสั่งในไฟล์เดียวได้ ผลที่แสดงคือผลของคำสั่งสุดท้ายที่มีแถวข้อมูล</p>
<div class="note">คำสั่ง <code>UPDATE</code> และ <code>DELETE</code> ที่ไม่มี <code>WHERE</code> จะแก้หรือลบทุกแถวในตาราง ควรลองกับฐานข้อมูลทดลองก่อน และสำรองข้อมูลจริงไว้เสมอ</div>

<h2>ใส่ตารางลงฟอร์ม (Data binding)</h2>
<p>วิธีที่เร็วที่สุดในการทำโปรแกรมจัดการข้อมูล เหมือนลากตารางจากหน้าต่าง Data Sources ของ Visual Studio:</p>
<ol>
  <li>เปิดฟอร์มของโปรเจกต์ไว้ในตัวออกแบบ</li>
  <li>ใน Server Explorer คลิกขวาที่ตาราง แล้วเลือก <b>Add to Form</b></li>
  <li>Phai Studio วางตารางข้อมูล (Data grid) ที่ผูกกับตารางนั้นลงในฟอร์ม พร้อมปุ่ม <b>Reload</b> และ <b>Save</b> ด้านล่าง แล้วกด <kbd>Ctrl</kbd>+<kbd>S</kbd> บันทึกฟอร์ม</li>
  <li>กด <kbd>F5</kbd> โปรแกรมจะโหลดข้อมูลมาแสดงเองตอนเปิดหน้าต่าง</li>
</ol>
<p>ในโปรแกรมที่รัน:</p>
<ul>
  <li>ดับเบิลคลิกช่องเพื่อแก้ค่า กด <kbd>Enter</kbd> เพื่อเก็บ</li>
  <li>ดับเบิลคลิกใต้แถวสุดท้ายเพื่อเพิ่มแถวใหม่ ช่องที่เว้นว่างจะใช้ค่าเริ่มต้นของตาราง เช่น รหัสที่รันเอง</li>
  <li>เลือกแถวแล้วกด <kbd>Delete</kbd> เพื่อเอาแถวออก</li>
  <li>กด <b>Save</b> เพื่อบันทึกทุกอย่างที่แก้ลงฐานข้อมูลทีเดียว แล้วโหลดใหม่ให้ ถ้ายังไม่กด Save ข้อมูลในฐานข้อมูลยังไม่เปลี่ยน กด <b>Reload</b> เพื่อทิ้งที่แก้และโหลดใหม่</li>
  <li>ช่องที่เขียนว่า <code>NULL</code> คือไม่มีค่า</li>
</ul>
<p>ครั้งแรกที่ใส่ตารางลงฟอร์ม Phai Studio เตรียมโปรเจกต์ให้ด้วย:</p>
<ul>
  <li><code>src/data_sources.rs</code> บอกว่าข้อมูลอยู่ที่เซิร์ฟเวอร์ไหน (เหมือน connection string ของ WinForms) แก้ไฟล์นี้ได้ เช่น ตอนย้ายไปใช้เซิร์ฟเวอร์จริง รหัสผ่านไม่ได้เขียนไว้ในไฟล์ แต่อ่านจากตัวแปรสภาพแวดล้อม <code>PHAI_DB_PASSWORD</code></li>
  <li>เพิ่ม <code>phai-data</code> ใน <code>Cargo.toml</code> และ <code>mod data_sources;</code> ใน <code>main.rs</code></li>
  <li>เพิ่มคำแปลไทยของปุ่ม Reload และ Save ในตารางคำแปล</li>
</ul>
<p>ในโค้ดของฟอร์มสั่งเองได้ด้วย <code>ui.reload_customers()</code> และ <code>ui.save_customers()</code> (ชื่อตามตาราง) การบันทึกทำใน <b>transaction เดียว</b>: ถ้าแถวไหนบันทึกไม่ได้ (เช่น ช่องที่ห้ามว่าง) จะไม่มีอะไรถูกบันทึกเลย ฐานข้อมูลเหมือนเดิม แก้แล้วกดบันทึกใหม่ได้ (MySQL ต้องเป็นตาราง InnoDB ซึ่งเป็นค่าเริ่มต้น)</p>
<p><b>หลายคนใช้พร้อมกัน:</b> ก่อนบันทึก โปรแกรมอ่านแถวที่เราแก้หรือลบจากฐานข้อมูลอีกครั้ง ถ้ามีคนอื่นแก้ช่องเดียวกัน แก้แถวที่เราจะลบ หรือลบแถวที่เราแก้ไปก่อน จะไม่บันทึกอะไรเลยและบอกว่าแถวไหน ให้กดโหลดใหม่แล้วแก้อีกครั้ง ส่วนถ้าคนอื่นแก้ช่องอื่นของแถวเดียวกัน ของทั้งสองคนจะอยู่ครบ ไม่ทับกัน หัวคอลัมน์แสดงชื่อคอลัมน์ในฐานข้อมูลตามจริง ถ้าแก้หัวคอลัมน์ในฟอร์มเป็นคำอื่น คำนั้นจะแปลไทยได้เหมือนข้อความอื่น</p>
<div class="note">การแก้และลบแถวต้องรู้ว่าแถวไหนคือแถวไหน ตารางจึงต้องมีคีย์หลัก (primary key) ตารางที่ไม่มีคีย์หลักจะเพิ่มแถวได้อย่างเดียว ถ้าเป็น SQLite โปรแกรมจะเปิดไฟล์ฐานข้อมูลตามพาธที่เลือกตอนเพิ่มการเชื่อมต่อ ถ้าจะแจกโปรแกรม ให้แก้พาธใน <code>data_sources.rs</code> ให้ชี้ไปที่ไฟล์บนเครื่องผู้ใช้</div>

<h2>ตกแต่ง Data grid</h2>
<p>Data grid ของ Phai Studio ตั้งค่าได้ทั้งตารางและทีละคอลัมน์ ทุกอย่างที่ตั้งในตัวออกแบบ จะเหมือนกันตอนโปรแกรมรัน</p>
<p><b>ทีละคอลัมน์</b> เลือก grid แล้วกด <b>...</b> ที่ Columns ในหน้าต่าง Properties (หรือ Edit Columns... จากปุ่มลูกศรเล็กที่มุม grid) จะเปิดหน้าต่าง Grid Columns ด้านซ้ายเป็นรายการคอลัมน์ มีปุ่ม Add, Remove, Move Up, Move Down และ <b>Retrieve Fields</b> ที่ดึงฟิลด์ของตารางที่ผูกไว้มาให้ ด้านขวาเป็นค่าของคอลัมน์ที่เลือก:</p>
<table>
  <tr><th>ค่า</th><th>ใช้ทำอะไร</th></tr>
  <tr><td>Caption, Field name, Width</td><td>หัวคอลัมน์ ฟิลด์ในตารางที่ผูก และความกว้าง</td></tr>
  <tr><td>Visible</td><td>ซ่อนคอลัมน์ได้ ค่ายังอยู่ เช่น รหัสที่โปรแกรมต้องใช้</td></tr>
  <tr><td>Alignment</td><td>ชิดซ้าย กลาง หรือขวา (ตัวเลขควรชิดขวา)</td></tr>
  <tr><td>Read only</td><td>ดูได้อย่างเดียว แก้ไม่ได้</td></tr>
  <tr><td>Edited as</td><td>แก้ด้วยช่องข้อความ ตัวเลข ปฏิทิน ช่องติ๊ก หรือรายการเลือก (ใส่ตัวเลือกที่ Drop-down items คั่นด้วยจุลภาค)</td></tr>
  <tr><td>Sorted, ปุ่มเรียง, ลากความกว้าง</td><td>เรียงตอนเปิด, คลิกหัวคอลัมน์เพื่อเรียงได้ไหม, ลากขอบหัวคอลัมน์ได้ไหม</td></tr>
  <tr><td>Display format</td><td>รูปแบบที่แสดง ค่าที่เก็บไม่เปลี่ยน: <code>n2</code> 1,234.50, <code>c2</code> ฿1,234.50, <code>p0</code> 15%, <code>d</code> 21/11/2026, <code>dt</code> มีเวลาด้วย</td></tr>
  <tr><td>Footer total</td><td>ผลรวมใต้คอลัมน์: Sum, Count, Average, Min, Max</td></tr>
  <tr><td>Format Rules</td><td>ใส่สีตามค่า กฎละบรรทัด เช่น <code>&gt; 1000 back=#ffe4e4 fore=#b00020 row</code> (ใส่ <code>row</code> เพื่อใส่สีทั้งแถว) เงื่อนไข: <code>= != &gt; &gt;= &lt; &lt;= contains empty notempty</code></td></tr>
  <tr><td>Group the rows by this column</td><td>จัดกลุ่มแถวตามค่าในคอลัมน์นี้ มีหัวกลุ่ม เช่น "Status: Paid (3)" คลิกเพื่อพับ</td></tr>
</table>
<p><b>ทั้งตาราง</b> อยู่ในหน้าต่าง Properties: AlternateRowColor (สีแถวเว้นแถว), HeaderBackColor และ HeaderForeColor (สีหัวตาราง), RowHeight, ShowFooter, ShowColumnHeaders, ColumnAutoWidth, AllowAddRows, AllowDeleteRows, EditorShowMode (ดับเบิลคลิก คลิก หรือ F2), EnterMoveNextColumn, AllowColumnMoving, FindPanel, FindFilter, CopyColumnHeaders และ Border colour / Border width สำหรับเส้นขอบ</p>
<p>ในโปรแกรมที่รัน:</p>
<ul>
  <li><kbd>Ctrl</kbd>+<kbd>F</kbd> เปิดช่องค้นหาเหนือหัวตาราง พิมพ์แล้วเหลือเฉพาะแถวที่มีคำนั้น (ปิด FindFilter ถ้าอยากแค่ไฮไลต์) <kbd>Enter</kbd> ไปแถวถัดไป <kbd>Esc</kbd> ปิด แถวที่ถูกกรองออกยังอยู่ กด Save ก็บันทึกด้วย</li>
  <li>คลิกหัวคอลัมน์เพื่อเรียง คลิกอีกครั้งเพื่อกลับด้าน</li>
  <li><kbd>Ctrl</kbd>+<kbd>C</kbd> คัดลอกแถวที่เลือกพร้อมหัวคอลัมน์ วางใน Excel ได้เลย <kbd>Ctrl</kbd>+<kbd>V</kbd> วางจาก Excel ลงตาราง <kbd>Ctrl</kbd>+<kbd>A</kbd> เลือกทุกแถว</li>
  <li><b>ตัวกรองแบบ Excel</b> (AllowFilter): ชี้ที่หัวคอลัมน์จะเห็นปุ่มลูกศร กดแล้วได้รายการค่าทั้งหมดในคอลัมน์นั้นให้ติ๊ก มีช่องค้นหา ปุ่มเรียง ก-ฮ / ฮ-ก และ "ล้างตัวกรอง" คอลัมน์ที่กรองอยู่มีรูปกรวยบนหัว ผลรวมท้ายตารางคิดเฉพาะแถวที่เหลือ กรองหลายคอลัมน์พร้อมกันได้</li>
  <li><b>คลิกขวาที่หัวคอลัมน์</b> (HeaderMenu): เรียง, กรอง..., ล้างตัวกรอง, จัดกลุ่มตามคอลัมน์นี้, <b>ส่งออกเป็น Excel</b> (.xlsx ตัวเลขและวันที่ยังเป็นตัวเลขจริง คำนวณต่อได้ หัวตารางตรึงไว้และมีปุ่มกรองของ Excel), <b>ส่งออกเป็น PDF</b>, ส่งออกเป็น CSV และตัวอย่างก่อนพิมพ์</li>
</ul>
<h3>ตารางหลัก-ตารางลูก (master-detail)</h3>
<p>เช่น ตาราง <code>orders</code> (ใบสั่งซื้อ) กับ <code>lines</code> (รายการสินค้าในใบสั่งซื้อ) เลือก <code>lines</code> แล้วตั้ง <b>MasterDetail</b> ในหน้าต่าง Properties เป็น <code>orders.id = order_id</code> อ่านว่า ตารางนี้แสดงเฉพาะแถวที่คอลัมน์ <code>order_id</code> ตรงกับคอลัมน์ <code>id</code> ของแถวที่เลือกอยู่ในตาราง <code>orders</code> (ชื่อคอลัมน์ใช้ได้ทั้งหัวคอลัมน์และชื่อคอลัมน์ในฐานข้อมูล ถ้าชื่อเหมือนกันเขียนแค่ <code>orders.id</code>)</p>
<ul>
  <li>เลือกแถวอื่นในตารางหลัก ตารางลูกเปลี่ยนตามทันที ตารางลูกมีตารางลูกต่ออีกชั้นก็ได้</li>
  <li>เพิ่มแถวใหม่ในตารางลูก (ดับเบิลคลิกใต้แถวสุดท้าย) คีย์ของตารางหลักใส่ให้เอง</li>
  <li>แถวของใบอื่นไม่ได้หายไป แค่ไม่แสดง ตารางที่ผูกฐานข้อมูลกด Save ก็บันทึกครบทุกแถว</li>
</ul>
<p>ในโค้ดสั่งได้ด้วย เช่น <code>ui.customers.sort_by(1, true)</code>, <code>ui.customers.find("สมชาย")</code>, <code>ui.customers.selected_rows()</code>, <code>ui.customers.export_csv("customers.csv")</code> (เปิดใน Excel แล้วภาษาไทยไม่เพี้ยน), <code>ui.customers.export_xlsx("customers.xlsx")</code>, <code>ui.customers.export_pdf("customers.pdf", "ลูกค้า")</code>, <code>ui.customers.print_preview("ลูกค้า")</code>, <code>ui.customers.set_column_filter(2, Some(&amp;["Paid"]))</code>, <code>ui.customers.clear_filters()</code> และ <code>ui.lines.set_master(ui.orders, 0, 1)</code></p>
<div class="note">ในเว็บแอป (PWA) สีแถวเว้นแถว สีหัวตาราง การจัดวาง และคอลัมน์ที่ซ่อน แสดงเหมือนกัน ส่วนรูปแบบตัวเลข ผลรวม กฎสี การจัดกลุ่ม และช่องค้นหา มีเฉพาะโปรแกรม Windows</div>

<h2>ให้โปรแกรมของคุณอ่านฐานข้อมูล</h2>
<p>คลิกขวาที่ตาราง แล้วเลือก <b>Copy code for a program</b> Phai Studio จะคัดลอกโค้ด Rust ลงคลิปบอร์ดให้ วางในไฟล์ของคุณได้เลย ในโค้ดมี:</p>
<ul>
  <li>ฟังก์ชัน <code>connection()</code> ที่บอกว่าต่อเซิร์ฟเวอร์ไหน</li>
  <li><code>struct</code> หนึ่งแถวของตาราง ชื่อฟิลด์และชนิดตรงกับคอลัมน์ คอลัมน์ที่เว้นว่างได้จะเป็น <code>Option</code></li>
  <li>ฟังก์ชัน <code>all_ชื่อตาราง()</code> ที่อ่านทุกแถว คืนค่าเป็น <code>Vec</code> ของ struct นั้น</li>
</ul>
<p>โค้ดนี้ใช้ <code>phai-data</code> ตัวเดียวกับที่ Server Explorer ใช้ ให้เพิ่มใน <code>Cargo.toml</code> ของโปรเจกต์ตามบรรทัดแรกของโค้ด</p>
<div class="note">โค้ดที่สร้างให้ไม่ได้เขียนรหัสผ่านลงในโค้ด แต่อ่านจากตัวแปรสภาพแวดล้อม <code>PHAI_DB_PASSWORD</code> เพราะโค้ดมักถูกแชร์หรือขึ้น Git รหัสผ่านไม่ควรอยู่ในนั้น</div>

<h2>ลองทำ: สมุดรายชื่อลูกค้าด้วย SQLite</h2>
<p>SQLite ไม่ต้องติดตั้งเซิร์ฟเวอร์ ฐานข้อมูลทั้งก้อนเป็นไฟล์เดียว เหมาะกับการหัด</p>
<ol>
  <li>Data &gt; Add Connection... เลือก SQLite file... แล้วตั้งชื่อไฟล์ใหม่ <code>shop.db</code></li>
  <li>คลิกขวาที่การเชื่อมต่อ แล้ว New Query พิมพ์คำสั่งข้างล่าง แล้วกด <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd></li>
</ol>
<pre><code>CREATE TABLE customers (
    id    INTEGER PRIMARY KEY,
    name  TEXT NOT NULL,
    phone TEXT
);
INSERT INTO customers (name, phone) VALUES ('สมชาย ใจดี', '081-234-5678');
INSERT INTO customers (name, phone) VALUES ('Monrak Net', '02-111-2222');
SELECT * FROM customers;</code></pre>
<ol start="3">
  <li>ดูผลที่แท็บ Results แล้วคลิกขวาที่การเชื่อมต่อ เลือก Refresh จะเห็นตาราง <code>customers</code></li>
  <li>เปิดฟอร์มหลักของโปรเจกต์ คลิกขวาที่ตาราง <code>customers</code> เลือก Add to Form บันทึกฟอร์ม แล้วกด <kbd>F5</kbd> ลองแก้ชื่อลูกค้า เพิ่มลูกค้าใหม่ แล้วกด Save</li>
</ol>
'''

BODY['winforms.html'] = r'''
<h1>control และความสามารถเพิ่มเติม</h1>
<p class="lead">นอกจาก control พื้นฐาน Phai Studio มี control และความสามารถแบบ WinForms ครบชุด (ฟอร์มที่ทำใน Visual Studio นำเข้ามาได้ด้วย ดู<a href="#winforms-import">ข้างล่าง</a>) ทุกอย่างวางจากกล่องเครื่องมือ ตั้งค่าในหน้าต่าง Properties แล้วเรียกจากโค้ดผ่าน <code>ui.ชื่อ</code> เหมือน control อื่น</p>

<h2>control เพิ่มเติม</h2>
<table>
<tr><th>control</th><th>ใช้ทำอะไร</th><th>เรียกจากโค้ด</th></tr>
<tr><td>Panel</td><td>กล่องรวม control มีสีพื้นของตัวเอง หรือเส้นขอบ (<code>border = "fixed"</code>)</td><td>ลาก control อื่นใส่ข้างในได้</td></tr>
<tr><td>Split container</td><td>สองช่องกับเส้นแบ่งที่ลากได้ control ในแต่ละช่องยึด anchor กับช่องของตัวเอง</td><td><code>splitter_distance()</code>, <code>set_splitter_distance(200)</code></td></tr>
<tr><td>ListBox</td><td>รายการให้เลือกหนึ่งหรือหลายบรรทัด (<code>selection_mode = "multi"</code>)</td><td><code>items_add("...")</code>, <code>selected_index()</code>, <code>selected_indices()</code></td></tr>
<tr><td>CheckedListBox</td><td>รายการที่แต่ละบรรทัดมีช่องติ๊ก</td><td><code>checked(i)</code>, <code>set_checked(i, true)</code>, <code>checked_indices()</code></td></tr>
<tr><td>LinkLabel</td><td>ข้อความที่เป็นลิงก์ กดแล้วเรียก <code>on_click</code></td><td><code>set_link_text("...")</code></td></tr>
<tr><td>MaskedTextBox</td><td>ช่องที่รับเฉพาะรูปแบบที่กำหนด เช่น เบอร์โทร <code>mask = "000-000-0000"</code></td><td><code>value()</code> (ไม่รวมขีด), <code>mask_completed()</code></td></tr>
<tr><td>MonthCalendar</td><td>ปฏิทินให้เลือกวัน สีตามธีม</td><td><code>selection_start()</code>, <code>set_date(2026, 11, 20)</code></td></tr>
</table>
<p>ตัวอักษรใน mask: <code>0</code> ตัวเลข, <code>9</code> ตัวเลขหรือเว้นว่าง, <code>L</code> ตัวอักษร, <code>A</code> ตัวอักษรหรือตัวเลข, <code>\</code> ใส่ตัวถัดไปตามที่เขียน</p>

<h2>component (ไม่มีหน้าตาบนฟอร์ม)</h2>
<p>component วางแล้วไปอยู่ในถาดด้านล่างพื้นที่ออกแบบ เหมือน Visual Studio ถาดมีแถบลายเฉียงคั่นอยู่ด้านบน ลากแถบนี้เพื่อปรับความสูงของถาดได้ (Phai Studio จำไว้ใช้ครั้งหน้า) คลิกที่ component เพื่อเลือกแล้วดูคุณสมบัติใน Properties ดับเบิลคลิกเพื่อเขียนโค้ด event ของมัน เช่น Tick ของ Timer ในถาดยังมี <b>menuStrip</b> และ <b>toolStrip</b> ของฟอร์มด้วย คลิกแล้วจะเปิดตัวออกแบบของแถบนั้นบนฟอร์ม</p>
<table>
<tr><th>component</th><th>ใช้ทำอะไร</th></tr>
<tr><td>Open / Save file dialog</td><td>หน้าต่างเลือกไฟล์ของ Windows ตั้ง <code>title</code>, <code>filter</code> (<code>ไฟล์ข้อความ|*.txt|ทุกไฟล์|*.*</code>), <code>default_ext</code> แล้วเรียก <code>ui.open_file.show_dialog()</code> ได้ <code>Option&lt;PathBuf&gt;</code> มี event <b>FileOk</b>: ทำงานตอนกด Open หรือ Save ได้ไฟล์ที่เลือกจาก <code>e.file()</code> ถ้าไฟล์ใช้ไม่ได้ให้เรียก <code>e.cancel()</code> หน้าต่างเลือกไฟล์จะเปิดค้างไว้ให้เลือกใหม่</td></tr>
<tr><td>Folder / Color / Font dialog</td><td>เลือกโฟลเดอร์, สี (<code>show_dialog("#12855f")</code> ได้ <code>#rrggbb</code>), ฟอนต์ (ได้ชื่อ ขนาด ตัวหนา ตัวเอียง)</td></tr>
<tr><td>ErrorProvider</td><td>เครื่องหมายแดงข้าง control ที่กรอกผิด: <code>ui.errors.set_error(ui.name, "ใส่ชื่อด้วย")</code> ส่งข้อความว่างเพื่อลบ</td></tr>
<tr><td>NotifyIcon</td><td>ไอคอนมุมขวาล่างของ taskbar มี <code>on_click</code>, <code>double_click</code>, เมนูคลิกขวา (<code>context_menu</code>) และ <code>show_balloon_tip(...)</code></td></tr>
<tr><td>Timer</td><td>เรียก <code>on_tick</code> ทุกกี่มิลลิวินาทีตามที่ตั้ง</td></tr>
<tr><td>Context menu</td><td>เมนูคลิกขวาของ control มี event <b>Opening</b> ทำงานก่อนเมนูขึ้น ใช้ติ๊ก ปิดใช้ หรือเปลี่ยนชื่อรายการตามสถานการณ์</td></tr>
</table>

<h2>ออกแบบเมนูบนฟอร์ม</h2>
<p>เหมือน MenuStrip ของ Visual Studio: คลิกแถบเมนูบนฟอร์ม (หรือ <b>Menu bar</b> ใน Toolbox หรือ menuStrip ในถาด) แถบเมนูจะกลายเป็นตัวออกแบบ</p>
<ul>
  <li>ท้ายแถบมีช่อง <b>พิมพ์ที่นี่</b> พิมพ์ชื่อเมนูใหม่ แล้วกด <kbd>Enter</kbd></li>
  <li>คลิกเมนูเพื่อเปิด ใต้เมนูมี <b>พิมพ์ที่นี่</b> สำหรับเพิ่มรายการ และข้างรายการที่เลือกมี <b>พิมพ์ที่นี่</b> สำหรับทำเมนูย่อย</li>
  <li>พิมพ์ <code>-</code> จะได้เส้นคั่น ใส่ <code>&amp;</code> หน้าตัวอักษรที่จะขีดเส้นใต้ (กด Alt+ตัวนั้น) id ของรายการสร้างจากข้อความให้เอง เช่น <code>&amp;Save As...</code> ได้ <code>save_as</code></li>
  <li>คลิกรายการที่เลือกอยู่อีกครั้ง หรือกด <kbd>F2</kbd> หรือพิมพ์ทับ เพื่อแก้ข้อความ ใช้ลูกศรเลื่อนไปมา <kbd>Ctrl</kbd>+<kbd>↑</kbd><kbd>↓</kbd> ย้ายลำดับ <kbd>Delete</kbd> ลบ</li>
  <li><b>ดับเบิลคลิก</b>รายการ จะพาไปเขียนโค้ด Click ของรายการนั้น</li>
  <li>คลิกขวามี แก้ข้อความ, คุณสมบัติ, แทรกรายการ, แทรกเส้นคั่น, ย้าย, ลบ</li>
</ul>
<p>รายการที่เลือกจะขึ้นในหน้า <b>Properties</b> เหมือน control: (Name) คือ id, Text, Image (ไอคอน), ShortcutKeys (เช่น <code>Ctrl+S</code>), Checked (ติ๊กไว้), CheckOnClick (กดแล้วสลับติ๊ก), Enabled ส่วนแท็บ Events มี</p>
<table>
<tr><th>เลือก</th><th>Event</th></tr>
<tr><td>เมนูบนแถบ (File, View)</td><td>DropDownOpening (ก่อนเมนูเปิด เหมาะสำหรับติ๊กหรือปิดใช้รายการ), DropDownClosed</td></tr>
<tr><td>รายการเมนู</td><td>Click, CheckedChanged และถ้ามีเมนูย่อย DropDownOpening, DropDownClosed</td></tr>
<tr><td>menuStrip (ทั้งแถบ)</td><td>MenuStart, MenuComplete (เริ่มและเลิกใช้แถบเมนู)</td></tr>
</table>
<p>ดับเบิลคลิกชื่อ event ในแท็บ Events เพื่อเขียนโค้ด จากโค้ดสั่งได้ด้วย: <code>ui.item_checked("word_wrap")</code>, <code>ui.set_item_checked("word_wrap", true)</code>, <code>ui.set_item_enabled("zoom_max", false)</code></p>

<h2>ออกแบบแถบเครื่องมือบนฟอร์ม</h2>
<p>เหมือน ToolStrip ของ Visual Studio: คลิกแถบเครื่องมือบนฟอร์ม (หรือ <b>Toolbar</b> ใน Toolbox หรือ toolStrip ในถาด)</p>
<ul>
  <li>ปุ่ม <b>+</b> ท้ายแถบเพิ่มปุ่มต่อท้าย ลูกศรข้าง ๆ มี ปุ่ม, เส้นคั่น และ <b>แทรกปุ่มมาตรฐาน</b> (New, Open, Save | Cut, Copy, Paste | Undo, Redo)</li>
  <li>คลิกเพื่อเลือก <b>ดับเบิลคลิก</b>เพื่อเขียนโค้ด Click ของปุ่ม <kbd>←</kbd><kbd>→</kbd> เลือก <kbd>Ctrl</kbd>+<kbd>←</kbd><kbd>→</kbd> ย้าย <kbd>Delete</kbd> ลบ</li>
  <li>คลิกขวามี ดูโค้ด, คุณสมบัติ, เลือกรูป, เปิด/ปิดใช้, คลิกแล้วค้างกด, เริ่มแบบกดค้าง, แทรกปุ่มหรือเส้นคั่นหน้าปุ่มนี้, ย้าย, ลบ</li>
  <li>Properties ของปุ่ม: (Name), Text (คำแนะนำเมื่อชี้), Image, CheckOnClick, Checked, Enabled แท็บ Events มี Click, CheckedChanged และของ toolStrip ทั้งแถบมี ItemClicked (<code>e.index</code> คือลำดับปุ่ม)</li>
  <li>ปุ่มที่ CheckOnClick กับรายการเมนูที่ id เดียวกัน ติ๊กพร้อมกัน</li>
</ul>

<h2>ฟอร์มซ้อนในฟอร์ม (MDI)</h2>
<p>ตั้ง <code>IsMdiContainer</code> ของฟอร์มหลัก แล้วเปิดฟอร์มอื่นข้างในด้วย <code>EditorUi::show_mdi_child(ui.hwnd, editor::Editor::default())</code> จัดเรียงด้วย <code>ui.layout_mdi("cascade")</code> (หรือ <code>tile_horizontal</code>, <code>tile_vertical</code>) ใช้ <kbd>Ctrl</kbd>+<kbd>Tab</kbd> สลับ และ <kbd>Ctrl</kbd>+<kbd>F4</kbd> ปิดฟอร์มลูก</p>

<h2>นำฟอร์มจาก WinForms เข้ามา</h2>
<p>มีโปรแกรมเก่าที่ทำด้วย C# หรือ VB.NET ใน Visual Studio? เปิดฟอร์มของโปรเจกต์ใน Phai Studio แล้วเลือก <b>Project &gt; Import WinForms Form...</b> เลือกไฟล์ <code>Form1.Designer.cs</code> ของฟอร์มนั้น ฟอร์มจะเข้ามาเป็นฟอร์มใหม่ของโปรเจกต์ (<code>form1.phf</code> กับ <code>form1.rs</code>) หน้าตาเหมือนเดิม แล้วแก้ต่อด้วยการลากวางได้ทันที</p>
<figure><img class="shot" src="img/winforms-import.png" alt="ฟอร์มตั้งค่าจากโปรแกรม C# เดิม เปิดในตัวออกแบบของ Phai Studio" width="520"><figcaption>ฟอร์มที่ทำใน Visual Studio ที่ 200% เข้ามาในขนาดปกติ พร้อมสี ตัวอักษร และกลุ่มเดิม</figcaption></figure>
<ul>
  <li><b>สิ่งที่มาด้วย</b>: control ทุกตัวที่ Phai มี (ปุ่ม ช่องกรอก รายการ ตาราง แท็บ กลุ่ม panel split และอื่น ๆ) ตำแหน่ง ขนาด anchor dock สี ตัวอักษร ข้อความ เมนู แถบเครื่องมือ แถบสถานะ เมนูคลิกขวา timer และ dialog</li>
  <li><b>ขนาด</b>: ฟอร์มที่ออกแบบบนจอ 150% หรือ 200% ถูกย่อกลับเป็นขนาดปกติเอง (อ่านจาก <code>AutoScaleDimensions</code>)</li>
  <li><b>event</b>: <code>ok.Click += Ok_Click</code> กลายเป็น <code>on_click = "ok_click"</code> และเมธอดว่าง <code>fn ok_click</code> ใน <code>form1.rs</code> ให้เขียนด้วย Rust</li>
  <li><b>สิ่งที่ไม่มาด้วย</b>: โค้ด C# ใน <code>Form1.cs</code> (ต้องเขียนใหม่ด้วย Rust), รูปที่เก็บใน <code>.resx</code> และ control ที่เขียนขึ้นเอง (กลายเป็น panel) — Phai Studio บอกรายการทั้งหมดหลังนำเข้า</li>
  <li>ข้อความในฟอร์มยังไม่มีภาษาไทย เทสต์ของโปรเจกต์จะบอกว่าคำไหน ให้เพิ่มใน <code>src/text.rs</code></li>
</ul>

<h2>ลากข้อความจาก control หนึ่งไปวางอีกตัว</h2>
<p>ใน event MouseDown ของ control ต้นทาง เรียก <code>ui.list.do_drag_drop("ข้อความ")</code> ป้ายเล็ก ๆ จะตามเมาส์ไป control ปลายทางที่มี event DragDrop ได้ <code>e.text</code> และ <code>e.source</code> กด <kbd>Esc</kbd> เพื่อยกเลิก ไฟล์ที่ลากมาจาก Explorer เข้า <code>e.files</code></p>

<h2>การพิมพ์</h2>
<p>วาง component <b>Print document</b> ตั้ง <code>document_name</code> แล้วผูก event <code>print_page</code> ทุกขนาดเป็นร้อยส่วนของนิ้ว นับจากมุมกระดาษ:</p>
<pre><code>fn draw_page(&amp;mut self, ui: &amp;MainWindowUi, e: &amp;phai_ui::print::PrintPageEventArgs) {
    let m = e.margin_bounds;                     // พื้นที่ในขอบกระดาษ
    e.set_font("Leelawadee UI", 16, true, false);
    e.draw_text(&amp;format!("ใบเสร็จ หน้า {}", e.page), m.x, m.y);
    e.draw_line(m.x, m.y + 40, m.right(), m.y + 40);
    e.set_has_more_pages(e.page &lt; 3);           // ยังมีหน้าต่อไปไหม
}</code></pre>
<ul>
  <li><code>ui.report.print_preview()</code> หน้าต่างดูตัวอย่าง พลิกหน้าได้ มีปุ่มพิมพ์</li>
  <li><code>ui.report.print_dialog()</code> เลือกเครื่องพิมพ์แล้วพิมพ์, <code>ui.report.print()</code> พิมพ์เลยไม่ถาม, <code>ui.report.page_setup()</code> ตั้งกระดาษและขอบ</li>
  <li><code>ui.report.print_to_file("Microsoft Print to PDF", path)</code> ได้ไฟล์ PDF</li>
  <li>วาดได้: <code>draw_text</code>, <code>draw_text_in</code> (ตัดบรรทัด ชิดซ้าย กลาง ขวา), <code>draw_line</code>, <code>draw_rect</code>, <code>fill_rect</code>, <code>draw_image</code>, <code>measure_text</code>, <code>set_color</code></li>
</ul>

<h2>ผูกข้อมูลกับ control (DataBindings)</h2>
<p>วาง component <b>Binding source</b> ให้ข้อมูลมาจากตารางในฐานข้อมูล (ตั้ง <code>bind</code> เหมือนตาราง grid) หรือจากโค้ด (ตั้ง <code>fields = "name, phone"</code> แล้ว <code>ui.people.set_rows(...)</code>) จากนั้นที่ control แต่ละตัวตั้ง <code>DataBindings</code> เป็น <code>people.name</code></p>
<ul>
  <li>กล่องข้อความแสดงข้อความ, ช่องติ๊กแสดงค่าจริงเท็จ, ช่องตัวเลขแสดงตัวเลข, ช่องวันที่แสดงวันที่</li>
  <li>เลื่อนแถว: <code>move_next()</code>, <code>move_previous()</code>, <code>move_first()</code>, <code>move_last()</code> ค่าที่พิมพ์ในช่องกลับเข้าแถวเอง</li>
  <li>แถวใหม่ <code>add_new()</code>, ลบ <code>remove_current()</code>, บันทึกลงฐานข้อมูล <code>ui.save_people()</code>, โหลดใหม่ <code>ui.reload_people()</code></li>
  <li>event <code>position_changed</code> ทำงานเมื่อแถวปัจจุบันเปลี่ยน</li>
</ul>

<h2>User control: control ของเราเอง</h2>
<ol>
  <li>Project &gt; <b>เพิ่ม User Control...</b> ตั้งชื่อ เช่น <code>AddressBox</code> ได้ฟอร์มเล็ก ๆ ที่ออกแบบได้เหมือนฟอร์มทั่วไป และมีไฟล์โค้ดของตัวเอง</li>
  <li>เปิดฟอร์มอื่น เลือก <b>User control</b> จากกล่องเครื่องมือ (หมวด Containers) วางลงฟอร์ม หน้าออกแบบแสดง control ข้างในให้เห็น</li>
  <li>เข้าถึง control ข้างในจากโค้ดของฟอร์ม: <code>AddressBoxUi::with(ui.home, |a| a.street.text())</code></li>
  <li>ให้ user control บอกฟอร์มว่ามีอะไรเปลี่ยน: ในโค้ดของ AddressBox เรียก <code>ui.notify_parent()</code> ฟอร์มจะได้ <code>on_change</code> ของมัน</li>
</ol>
<div class="note">ตารางเทียบทุก property เมธอด และ event กับ WinForms อยู่ใน <code>docs/winforms-parity.md</code> ของโปรเจกต์ Phai Studio</div>
'''

BODY['inspector.html'] = r'''
<h1>ตัวตรวจสด (F12)</h1>
<p class="lead">ระหว่างรันโปรแกรมแบบ debug กด <kbd>F12</kbd> ในหน้าต่างโปรแกรม จะได้ตัวตรวจสดข้างหน้าต่าง ปรับหน้าตา control ดูผลทันที แล้วเก็บลงฟอร์มได้ เหมือน DevTools ของเบราว์เซอร์</p>

<h2>เปิดตัวตรวจ</h2>
<ol>
  <li>กด <kbd>F5</kbd> ใน Phai Studio เพื่อรันโปรแกรม</li>
  <li>ในหน้าต่างโปรแกรม กด <kbd>F12</kbd> (หรือ <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>I</kbd>) ตัวตรวจเปิดข้างหน้าต่าง</li>
  <li>กด <b>ชี้เลือก</b> แล้วคลิก control ในหน้าต่าง กรอบสีฟ้าบอกว่าเลือกตัวไหน กด <kbd>Esc</kbd> เพื่อเลิกชี้ หรือเลือกจากรายการก็ได้</li>
</ol>

<h2>ปรับอะไรได้</h2>
<ul>
  <li><b>หน้าตาทีละสถานะ</b> ปกติ, hover, pressed, focus, disabled, checked: สีพื้น สีตัวอักษร สีเส้นขอบ มุมมน ความหนาเส้น ตัวหนา gradient เงา พิมพ์แล้ว control ในโปรแกรมเปลี่ยนทันที</li>
  <li>แต่ละค่าบอกว่ามาจากไหน เช่น <code>own :hover</code> (ของ control เอง) หรือ <code>button.primary:hover · phai:light</code> (จากธีม)</li>
  <li><b>ข้อความ ใช้งานได้ มองเห็น</b> และ property อื่น เช่น tooltip, read only</li>
</ul>

<h2>เก็บลงฟอร์ม</h2>
<ul>
  <li><b>เก็บลงฟอร์ม</b> เขียนเฉพาะบรรทัดของ control นั้นในไฟล์ <code>.phf</code> คอมเมนต์และบรรทัดอื่นอยู่เหมือนเดิม</li>
  <li>สลับกลับมาที่ Phai Studio ฟอร์มที่เปิดอยู่โหลดใหม่ให้เอง และ Undo ได้</li>
  <li><b>ย้อนทั้งหมด</b> คืนทุกอย่างกลับเป็นตอนเปิดตัวตรวจ</li>
</ul>
<div class="note">ตัวตรวจมีเฉพาะโปรแกรมที่ build แบบ debug เท่านั้น โปรแกรมที่ build แบบ release เพื่อแจกจ่ายไม่มีตัวตรวจ และไม่มีโค้ดส่วนนี้ติดไปด้วย</div>
<div class="try">รันโปรเจกต์แรก กด <kbd>F12</kbd> ชี้ที่ปุ่ม OK เลือกสถานะ hover แล้วเปลี่ยนสีพื้นเป็น <code>#2563eb</code> เอาเมาส์ไปชี้ปุ่มดู แล้วกดเก็บลงฟอร์ม</div>
'''

BODY['web.html'] = r'''
<h1>เว็บและมือถือ (PWA)</h1>
<p class="lead">ฟอร์มเดิมกับโค้ดเดิมของโปรแกรม รันในเบราว์เซอร์ได้ และติดตั้งลงมือถือ Android และ iPhone เป็นแอปได้ ไม่ต้องผ่าน Play Store หรือ App Store และไม่ต้องใช้ Mac</p>

<h2>PWA คืออะไร</h2>
<p>PWA (Progressive Web App) คือหน้าเว็บที่ติดตั้งลงเครื่องได้เหมือนแอป ได้ไอคอนบนหน้าจอ เปิดแล้วเต็มจอ ใช้ได้แม้ไม่มีเน็ต และเมื่ออัปโหลดเวอร์ชันใหม่ขึ้นเว็บ เครื่องผู้ใช้ได้เวอร์ชันใหม่เอง</p>

<h2>ติดตั้งเครื่องมือ (ครั้งเดียว ฟรี)</h2>
<pre><code>rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130</code></pre>

<h2>สร้างและลองในเบราว์เซอร์</h2>
<ol>
  <li>เปิดฟอร์มของโปรเจกต์ใน Phai Studio</li>
  <li>Build &gt; <b>รันในเบราว์เซอร์</b> ครั้งแรกใช้เวลาสักครู่ เสร็จแล้วเบราว์เซอร์ของเครื่องนี้เปิดโปรแกรมให้</li>
  <li>หรือ Build &gt; <b>สร้างสำหรับเว็บ (PWA)</b> ได้โฟลเดอร์ <code>web/</code> ในโปรเจกต์ เอาไปขึ้นเว็บได้เลย</li>
</ol>
<p>แถบสถานะบอกว่าสร้างไว้ที่ไหน และถ้าฟอร์มมี control ที่ไม่มีบนเว็บ จะบอกชื่อไว้ด้วย</p>

<h2>รันบนมือถือระหว่างทำ (Run on Phone)</h2>
<ol>
  <li>ให้มือถืออยู่ใน Wi-Fi เดียวกับคอมพิวเตอร์</li>
  <li>Build &gt; <b>รันบนมือถือ...</b> Phai Studio build สำหรับเว็บ แล้วขึ้นหน้าต่างมี <b>QR code</b> กับที่อยู่ เช่น <code>http://192.168.1.37:8790/</code></li>
  <li>เปิดกล้องมือถือส่องที่ QR code (หรือพิมพ์ที่อยู่ในเบราว์เซอร์มือถือ) โปรแกรมเปิดบนมือถือเลย</li>
  <li>แก้ฟอร์มหรือโค้ดแล้วสั่ง <b>รันบนมือถือ</b> อีกครั้ง หน้าบนมือถือ<b>รีโหลดเอง</b> ไม่ต้องสแกนใหม่</li>
</ol>
<div class="note">ครั้งแรก Windows Firewall อาจถามว่าให้ Phai Studio รับการเชื่อมต่อไหม ให้กดอนุญาตเฉพาะ <b>เครือข่ายส่วนตัว (Private)</b> ปุ่ม <b>เปิดที่นี่</b> เปิดที่อยู่เดียวกันบนคอมพิวเตอร์ และ <b>คัดลอกที่อยู่</b> ไว้ส่งต่อ</div>
<div class="warn">แบบนี้ใช้ลองระหว่างทำเท่านั้น เพราะเป็น <code>http://</code> มือถือจะยังไม่ให้ติดตั้งเป็นแอป ถ้าจะติดตั้ง ต้องขึ้นเว็บที่เป็น HTTPS ตามหัวข้อถัดไป</div>

<h2>ดูบนจอมือถือโดยไม่ต้องมีมือถือ (Device Preview)</h2>
<p><b>View &gt; ดูบนอุปกรณ์</b> (<kbd>Shift+F4</kbd>) แสดงฟอร์มในกรอบมือถือ มือถือจอเล็ก แท็บเล็ต หรือเดสก์ท็อป กด <b>หมุนจอ</b> เพื่อดูแนวนอน หรือติ๊ก <b>มือถือ แนวนอน และแท็บเล็ต เรียงกัน</b> เพื่อดูสามแบบพร้อมกัน แก้ฟอร์มใน designer แล้วภาพในกรอบเปลี่ยนตามเอง</p>

<h2>ฟอร์มบนจอแคบ (PhoneLayout)</h2>
<p>เมื่อจอแคบกว่าฟอร์มที่ออกแบบ (เช่น มือถือ) หน้าเว็บ<b>เรียง control ลงมาทีละตัว</b> ตามลำดับการอ่าน (บนลงล่าง ซ้ายไปขวา) กว้างเต็มจอ ช่องพิมพ์และปุ่มสูงพอให้นิ้วกด แถบเครื่องมืออยู่บนสุด แถบสถานะอยู่ล่างสุด จอกว้างพอเมื่อไร ก็กลับไปวางตามที่ออกแบบ</p>
<p>ถ้าอยากให้ย่อทั้งฟอร์มแบบเดิมแทน ตั้ง property ของฟอร์ม <b>PhoneLayout</b> (หมวด Layout) เป็น <code>zoom</code></p>

<h2>ตั้งค่าแอป (ชื่อ ไอคอน สี)</h2>
<p><b>Project &gt; เว็บแอป (PWA)...</b> ตั้งได้ว่าแอปที่ติดตั้งลงมือถือ:</p>
<table>
  <tr><th>ช่อง</th><th>ความหมาย</th><th>ถ้าเว้นว่าง</th></tr>
  <tr><td>ชื่อแอป</td><td>ชื่อเต็มของแอป</td><td>ชื่อหน้าต่างของฟอร์มหลัก</td></tr>
  <tr><td>ชื่อใต้ไอคอน</td><td>ไม่เกิน 12 ตัวอักษร (มือถือตัดที่ยาวกว่านี้)</td><td>12 ตัวแรกของชื่อแอป</td></tr>
  <tr><td>สีแถบด้านบน</td><td><code>#rrggbb</code></td><td>สีเน้น (accent) ของธีม</td></tr>
  <tr><td>สีพื้นตอนเปิดแอป</td><td><code>#rrggbb</code></td><td>สีพื้นของธีม</td></tr>
  <tr><td>แนวจอ</td><td><code>any</code> ตามเครื่อง, <code>portrait</code> แนวตั้ง, <code>landscape</code> แนวนอน</td><td><code>any</code></td></tr>
  <tr><td>ไอคอน</td><td>ไฟล์ PNG ในโปรเจกต์ สี่เหลี่ยมจัตุรัส 512×512 ขึ้นไปสวยสุด ย่อให้เป็น 192 และ 512 เอง</td><td>รูปใบไผ่สีของแอป</td></tr>
</table>
<p>ค่าทั้งหมดเก็บไว้ใน <code>Cargo.toml</code> หัวข้อ <code>[package.metadata.phai-web]</code> มีผลตอน build สำหรับเว็บครั้งถัดไป</p>

<h2>แต่ง CSS เฉพาะบนเว็บ (web.css)</h2>
<p>วางไฟล์ <code>web.css</code> ไว้ในโฟลเดอร์โปรเจกต์ (ข้าง <code>Cargo.toml</code>) แล้วเขียน CSS ได้ตรง ๆ Phai Studio ใส่ไว้ในหน้าเว็บต่อจากธีม จึงชนะค่าของธีมเสมอ เช่น</p>
<pre><code>/* ทุกปุ่มบนมือถือมุมมนกว่า */
@media (max-width: 600px) {
  .phai-button { border-radius: 12px; }
}

/* ปุ่มชื่อ ok (id ของ control) */
#ok { font-weight: 600; }

/* ตอนเรียงลงมาบนจอแคบ */
.phai-stacked { gap: 14px; }</code></pre>
<p>ชื่อ class ที่ใช้ได้: <code>.phai-button</code>, <code>.phai-edit</code>, <code>.phai-label</code>, <code>.phai-combo</code>, <code>.phai-list</code> ฯลฯ และ <code>#ชื่อcontrol</code> สำหรับตัวเดียว ส่วนสีและหน้าตาที่ใช้ทั้งบน Windows และเว็บ ให้แต่งใน <code>theme.css</code> ตาม <a href="css.html">ตกแต่งหน้าตาด้วย CSS</a></p>

<h2>ขึ้นเว็บและติดตั้งลงมือถือ</h2>
<p><b>Build &gt; Publish Web App...</b> สร้างสำหรับเว็บแล้วส่งขึ้นที่จริงให้ ใส่ปลายทางครั้งเดียว Phai Studio จำไว้ใน <code>Cargo.toml</code> (<code>[package.metadata.phai-web] publish</code>) ครั้งต่อไปกดอย่างเดียว</p>
<table>
  <tr><th>ปลายทาง</th><th>ตัวอย่าง</th><th>ทำอย่างไร</th></tr>
  <tr><td>โฟลเดอร์</td><td><code>D:\www\shop</code></td><td>คัดลอกไฟล์ไปไว้ เช่น โฟลเดอร์ของเว็บเซิร์ฟเวอร์ หรือไดรฟ์ที่แชร์</td></tr>
  <tr><td>เซิร์ฟเวอร์ (SSH)</td><td><code>me@example.com:/var/www/shop</code></td><td>ส่งด้วย <code>scp</code> ของ Windows ใช้ SSH key ที่ตั้งไว้ในเครื่อง ไม่ถามรหัสผ่าน</td></tr>
  <tr><td>GitHub Pages</td><td><code>github-pages</code></td><td>ส่งไป branch <code>gh-pages</code> ของ repository ของโปรเจกต์ (git remote origin) แล้วเปิด <code>https://ชื่อ.github.io/โปรเจกต์/</code> ให้ ครั้งแรกต้องเปิด Pages ใน Settings ของ repository บน GitHub ให้ใช้ branch <code>gh-pages</code></td></tr>
</table>
<p>หรือทำเอง:</p>
<ol>
  <li>อัปโหลดทุกไฟล์ในโฟลเดอร์ <code>web/</code> ขึ้นเว็บไซต์ที่เป็น <b>HTTPS</b> (เว็บของเราเอง, GitHub Pages หรืออื่น ๆ)</li>
  <li>Android: เปิดที่อยู่ใน Chrome แล้วกดเมนู ⋮ &gt; <b>ติดตั้งแอป</b> หรือ <b>เพิ่มลงหน้าจอหลัก</b></li>
  <li>iPhone: เปิดใน Safari กดปุ่มแชร์ &gt; <b>เพิ่มลงหน้าจอโฮม</b></li>
</ol>
<div class="note">ลองตัวอย่างได้ที่ <a href="https://phai-studio.monrak.net/demo/shop/">phai-studio.monrak.net/demo/shop</a> ภาษาเป็นไทยเองเมื่อเครื่องตั้งภาษาไทย หรือต่อท้ายที่อยู่ด้วย <code>?lang=th</code> หรือ <code>?lang=en</code></div>
<div class="warn">ต้องเป็น HTTPS เท่านั้น ถ้าเปิดจาก <code>http://</code> ที่ไม่ใช่เครื่องตัวเอง มือถือจะไม่ให้ติดตั้ง และใช้แบบไม่มีเน็ตไม่ได้</div>

<h2>โค้ดไม่ต้องแก้</h2>
<p>ไฟล์ <code>main_window.rs</code> ไฟล์เดียวใช้ได้ทั้ง Windows และเว็บ Phai Studio สร้างโค้ดหน้าต่างสำหรับเว็บไว้ใน <code>src/web/</code> และแก้ <code>Cargo.toml</code> กับ <code>main.rs</code> ให้ครั้งแรกที่สร้างสำหรับเว็บ หลังจากนั้นโปรแกรมยัง build สำหรับ Windows ได้เหมือนเดิม</p>
<ul>
  <li>ตำแหน่ง control ใช้ anchor และ dock เหมือนบน Windows จอใหญ่กว่าก็ขยายตาม จอแคบกว่าที่ออกแบบ (มือถือ) เรียงลงมาทีละตัว (ดู PhoneLayout ข้างบน)</li>
  <li>ฟอร์มที่สองที่เปิดด้วย <code>show</code> ขึ้นเป็นกล่องทับหน้า มีปุ่มปิดมุมขวาบน</li>
  <li><code>show_dialog</code> บนเว็บไม่รอให้ปิดก่อน ใช้ปุ่มในฟอร์มนั้นทำงานแทน</li>
</ul>

<h2>มีบนเว็บแล้ว / ยังไม่มี</h2>
<table>
<tr><th>มีแล้ว</th><td>เมนูบาร์ (เมนูย่อย ติ๊ก รายการสีเทา ปุ่มลัด <kbd>Ctrl</kbd>+...), toolbar (บนมือถือแสดงเป็นข้อความ), <b>เมนูคลิกขวา</b> (บนมือถือ: กดค้าง พร้อม Opening), <b>tree</b> (ลูกศรเปิดปิด ปุ่มลูกศรบนคีย์บอร์ด เหตุการณ์เมื่อเลือก และเมธอดเดียวกับ Windows), label, edit (รวมรหัสผ่าน), textarea, button, checkbox, radio, combo, listbox, checked list, list, grid, number, date, calendar, slider, progress, group, panel, tabs, link, picture, status bar, timer และ event หลักทั้งหมด (click, change, key, mouse, Shown)</td></tr>
<tr><th>ยังไม่มี</th><td>split container, user control, dialog เลือกไฟล์ สี ฟอนต์, ErrorProvider, NotifyIcon, การพิมพ์ และ binding source</td></tr>
</table>
<p>โค้ดที่เรียกสิ่งที่ยังไม่มี จะ build สำหรับเว็บไม่ผ่าน แต่ build สำหรับ Windows ได้ตามปกติ</p>

<h2>ตารางที่ผูกฐานข้อมูล บนเว็บ</h2>
<p>เบราว์เซอร์ต่อฐานข้อมูลตรง ๆ ไม่ได้ (และไม่ควร เพราะต้องใส่รหัสผ่านฐานข้อมูลไว้ในหน้าเว็บ) เมื่อฟอร์มมีตารางที่ผูกฐานข้อมูล Phai Studio จึงสร้าง <b>เซิร์ฟเวอร์ข้อมูล</b> ตัวเล็กให้ในโฟลเดอร์ <code>data-server/</code> ของโปรเจกต์ มันอ่านและบันทึกเฉพาะตารางและคอลัมน์ที่ฟอร์มผูกไว้ ไม่รับคำสั่ง SQL จากหน้าเว็บ</p>
<ul>
  <li><b>ตอนทำ:</b> Run in Browser และ Run on Phone เปิดเซิร์ฟเวอร์ข้อมูลให้เอง บนมือถือจะถาม <b>รหัสเข้าใช้ข้อมูล</b> ครั้งแรก (ดูได้ในแถบสถานะของ Phai Studio)</li>
  <li><b>ใช้จริง:</b> ที่เครื่องที่มีฐานข้อมูล เข้าโฟลเดอร์ <code>data-server</code> แล้ว <code>cargo run --release</code> ตั้ง <code>PHAI_DB_PASSWORD</code> (รหัสผ่านฐานข้อมูล), <code>PHAI_DATA_TOKEN</code> (รหัสเข้าใช้ที่แอปจะถาม ถ้าไม่ตั้ง โปรแกรมสุ่มให้แล้วแสดงออกมา - ต้องมีเสมอ เพราะเว็บอื่นที่เปิดในเบราว์เซอร์ก็เรียกเครื่องนี้ได้) และ <code>PHAI_DATA_ADDR</code> (ค่าเริ่มต้น <code>127.0.0.1:8791</code>) แล้วให้เว็บเซิร์ฟเวอร์ที่เป็น HTTPS ส่ง <code>/phai-data/</code> ต่อไปให้ เช่น nginx: <code>location /phai-data/ { proxy_pass http://127.0.0.1:8791; }</code></li>
  <li>เซิร์ฟเวอร์ข้อมูลอยู่คนละที่กับหน้าเว็บ: ใส่ <code>data-url = "https://data.example.com/phai-data/"</code> ใต้ <code>[package.metadata.phai-web]</code> ใน <code>Cargo.toml</code></li>
  <li>บนเว็บ <code>reload_ตาราง()</code> และ <code>save_ตาราง()</code> เริ่มงานแล้วกลับมาทันที ตารางเติมเมื่อข้อมูลมาถึง (มีปัญหาจะขึ้นข้อความ) ดับเบิลคลิกใต้แถวสุดท้ายเพื่อเพิ่มแถว <kbd>Ctrl</kbd>+<kbd>Delete</kbd> ลบแถวที่เลือก</li>
</ul>
'''

BODY['report.html'] = r'''
<h1>รายงานพิมพ์ (Report Designer)</h1>
<p class="lead">ออกแบบรายงานสำหรับพิมพ์ เช่น ใบสรุปยอดขาย รายชื่อลูกค้า ใบแจ้งหนี้ โดยลากวางบนหน้ากระดาษ ดึงข้อมูลจริงจากตารางในฐานข้อมูล มีหัวกลุ่ม ผลรวม เลขหน้า ดูตัวอย่างก่อนพิมพ์ และบันทึกเป็น PDF ได้ ฟรีทุกความสามารถ</p>

<h2>รายงานคือแถบเรียงกัน</h2>
<p>รายงานแบ่งเป็น <b>แถบ</b> (band) แต่ละแถบพิมพ์ในจังหวะของมัน แล้วในแถบมีข้อความ ฟิลด์ ผลรวม เส้น กรอบ หรือรูปภาพ</p>
<table>
  <tr><th>แถบ</th><th>พิมพ์เมื่อไร</th><th>มักใส่อะไร</th></tr>
  <tr><td>หัวรายงาน (Report Header)</td><td>ครั้งเดียวที่ต้นรายงาน</td><td>ชื่อรายงาน โลโก้</td></tr>
  <tr><td>หัวกระดาษ (Page Header)</td><td>บนสุดของทุกหน้า</td><td>หัวคอลัมน์</td></tr>
  <tr><td>หัวกลุ่ม (Group Header)</td><td>ก่อนแถวของแต่ละกลุ่ม</td><td>ชื่อกลุ่ม เช่น "ภาค: ภาคเหนือ"</td></tr>
  <tr><td>รายละเอียด (Detail)</td><td>ทุกแถวของตาราง</td><td>ฟิลด์ของแถว</td></tr>
  <tr><td>ท้ายกลุ่ม (Group Footer)</td><td>หลังแถวของแต่ละกลุ่ม</td><td>ผลรวมของกลุ่ม</td></tr>
  <tr><td>ท้ายรายงาน (Report Footer)</td><td>ครั้งเดียวท้ายรายงาน</td><td>ผลรวมทั้งหมด</td></tr>
  <tr><td>ท้ายกระดาษ (Page Footer)</td><td>ล่างสุดของทุกหน้า</td><td>วันที่พิมพ์ เลขหน้า</td></tr>
</table>

<h2>สร้างรายงาน</h2>
<ol>
  <li>เปิดฟอร์มของโปรเจกต์ แล้วเลือกเมนู Project &gt; เพิ่มรายงาน... ตั้งชื่อ เช่น <code>SalesReport</code></li>
  <li>หน้าต่างตัวออกแบบรายงานเปิดขึ้น กดปุ่ม <b>ข้อมูล...</b> เลือกการเชื่อมต่อ ฐานข้อมูล และตาราง จาก Server Explorer (ต้องเพิ่มการเชื่อมต่อใน <a href="database.html">Server Explorer</a> ก่อน)</li>
  <li>ถ้าแถบรายละเอียดยังว่าง ตัวออกแบบวางทุกคอลัมน์ให้ พร้อมหัวคอลัมน์ในหัวกระดาษ</li>
  <li>กด <b>Preview</b> ดูหน้ากระดาษจริงกับข้อมูลจริง แล้วกด <b>บันทึก</b> (<kbd>Ctrl</kbd>+<kbd>S</kbd>)</li>
</ol>
<p>รายงานที่มีอยู่แล้ว ดับเบิลคลิกไฟล์ <code>.phaireport</code> ในหน้าต่าง Project เพื่อเปิด</p>

<h2>หน้าต่างตัวออกแบบ</h2>
<ul>
  <li><b>ซ้าย</b> คอลัมน์ของตาราง ดับเบิลคลิกคอลัมน์ = วางฟิลด์ในแถบรายละเอียด และหัวคอลัมน์ในหัวกระดาษ</li>
  <li><b>กลาง</b> หน้ากระดาษ แต่ละแถบมีแถบสีบอกชื่อ ลากสิ่งของเพื่อย้าย (ลากไปปล่อยในแถบอื่นได้) ลากมุมขวาล่างเพื่อปรับขนาด ลากขอบล่างของแถบเพื่อปรับความสูง ฟิลด์แสดงเป็น <code>[ชื่อคอลัมน์]</code> ผลรวมแสดงเป็น <code>sum(คอลัมน์)</code></li>
  <li><b>ขวา</b> คุณสมบัติของสิ่งที่เลือก ดับเบิลคลิกแถวเพื่อแก้ ค่าที่มีตัวเลือก (จัดแนว ฟังก์ชัน คอลัมน์) ขึ้นเป็นเมนู คลิกที่ว่างเพื่อดูคุณสมบัติของทั้งรายงาน</li>
</ul>
<table>
  <tr><th>ปุ่ม</th><th>ทำอะไร</th></tr>
  <tr><td>ป้าย / ฟิลด์ / เส้น / กรอบ / รูปภาพ</td><td>เพิ่มลงแถบที่เลือกอยู่ (ไม่ได้เลือก = แถบรายละเอียด)</td></tr>
  <tr><td>ผลรวม</td><td>เลือกฟิลด์ก่อนแล้วกด ได้ผลรวมของคอลัมน์นั้นลงท้ายกลุ่ม (หรือท้ายรายงานถ้าไม่มีกลุ่ม) เปลี่ยนเป็นนับ เฉลี่ย ต่ำสุด สูงสุด ได้ที่คุณสมบัติ "ฟังก์ชัน"</td></tr>
  <tr><td>ข้อมูล...</td><td>เลือกตาราง</td></tr>
  <tr><td>กลุ่ม...</td><td>จัดกลุ่มตามคอลัมน์ ได้หัวกลุ่มและท้ายกลุ่มให้ หรือเอากลุ่มออก</td></tr>
  <tr><td>แถบ...</td><td>เปิดหรือปิดหัว/ท้ายรายงาน และหัว/ท้ายกระดาษ</td></tr>
  <tr><td>Preview / พิมพ์... / PDF...</td><td>ดูตัวอย่าง พิมพ์ หรือบันทึกเป็น PDF (ใช้เครื่องพิมพ์ Microsoft Print to PDF ที่มากับ Windows)</td></tr>
</table>
<p>แป้นพิมพ์: <kbd>Delete</kbd> ลบ, ลูกศรขยับทีละนิด (<kbd>Ctrl</kbd> = ทีละมาก, <kbd>Shift</kbd> = ปรับขนาด), <kbd>Ctrl</kbd>+<kbd>Z</kbd> เลิกทำ, <kbd>Ctrl</kbd>+<kbd>Y</kbd> ทำซ้ำ</p>

<h2>ข้อความที่เติมให้</h2>
<p>ในป้าย (label) ใส่คำในวงเล็บปีกกา แล้วตอนพิมพ์จะถูกแทนด้วยค่าจริง</p>
<table>
  <tr><th>เขียน</th><th>ได้</th></tr>
  <tr><td><code>{page}</code> / <code>{pages}</code></td><td>หน้านี้ / จำนวนหน้าทั้งหมด เช่น <code>หน้า {page} / {pages}</code></td></tr>
  <tr><td><code>{date}</code> <code>{time}</code></td><td>วันที่และเวลาที่พิมพ์</td></tr>
  <tr><td><code>{title}</code></td><td>ชื่อรายงาน</td></tr>
  <tr><td><code>{row}</code></td><td>ลำดับแถว</td></tr>
  <tr><td><code>{ชื่อคอลัมน์}</code></td><td>ค่าของคอลัมน์ในแถวนั้น เช่น <code>ภาค: {region}</code> ในหัวกลุ่ม</td></tr>
</table>
<p>ตัวเลขจัดรูปแบบได้ที่คุณสมบัติ "รูปแบบ" เช่น <code>#,##0.00</code> (มีจุลภาคหลักพัน ทศนิยมสองตำแหน่ง) หรือ <code>฿#,##0</code></p>

<h2>หน้าตาด้วย CSS</h2>
<p>เหมือนฟอร์ม หน้าตาของรายงานมาจาก CSS ค่าเริ่มต้นมีหัวรายงานตัวใหญ่ หัวคอลัมน์ตัวหนามีเส้นใต้ แถวสลับสีจาง ๆ และผลรวมตัวหนา แต่งเพิ่มได้ที่คุณสมบัติ CSS ของรายงาน:</p>
<pre><code>band.group_footer   { background: #fdf6e3; }
label.report_header { color: #12855f; }
band.detail.odd     { background: #ffffff; }   /* ไม่เอาแถวสลับสี */</code></pre>
<p>ชื่อชนิดคือ <code>label</code> <code>field</code> <code>total</code> <code>line</code> <code>box</code> และแถบคือ <code>band</code> โดยมี class เป็นชื่อแถบ (<code>.page_header</code>, <code>.detail</code> ...)</p>

<h2>เรียกรายงานจากโปรแกรม</h2>
<p>บันทึกแล้ว Phai Studio สร้างไฟล์ <code>src/sales_report.rs</code> ให้ (อย่าแก้ไฟล์นี้ มันถูกเขียนใหม่ทุกครั้งที่บันทึก) เรียกจากปุ่มบนฟอร์มได้เลย:</p>
<pre><code>fn print_click(&amp;mut self, ui: &amp;MainWindowUi) {
    if let Err(e) = crate::sales_report::preview(ui.hwnd) {
        phai_ui::win::message(Some(ui.hwnd), &amp;e, "รายงาน", Default::default());
    }
}</code></pre>
<table>
  <tr><th>คำสั่ง</th><th>ทำอะไร</th></tr>
  <tr><td><code>sales_report::preview(hwnd)</code></td><td>หน้าต่างดูตัวอย่าง มีปุ่มพิมพ์</td></tr>
  <tr><td><code>sales_report::print(hwnd)</code></td><td>หน้าต่าง Print ของ Windows แล้วพิมพ์</td></tr>
  <tr><td><code>sales_report::export_pdf(hwnd, path)</code></td><td>บันทึกเป็น PDF</td></tr>
  <tr><td><code>sales_report::report()</code></td><td>ตัวรายงาน ใส่แถวของตัวเองได้ด้วย <code>phai_ui::report::Data::new(คอลัมน์, แถว)</code></td></tr>
</table>
<div class="note">รายละเอียดรูปแบบไฟล์ <code>.phaireport</code> อยู่ใน <code>docs/phaireport.md</code> ของ repo</div>
'''

BODY['rust-01.html'] = r'''
<h1>1. เริ่มต้นกับ Rust</h1>
<p class="lead">Rust คืออะไร ทำไมถึงน่าเรียน และโปรแกรมแรกของคุณ</p>

<h2>Rust คืออะไร</h2>
<p>Rust เป็นภาษาโปรแกรมที่ได้โปรแกรมเร็วเท่า C และ C++ แต่ปลอดภัยกว่ามาก คอมไพเลอร์ของ Rust ตรวจโค้ดอย่างละเอียดก่อนจะยอมสร้างโปรแกรมให้ จึงจับบั๊กประเภทที่ภาษาอื่นปล่อยหลุดไปถึงผู้ใช้ได้ตั้งแต่ตอนเขียน เช่น ใช้หน่วยความจำที่คืนไปแล้ว ลืมจัดการกรณีที่ไม่มีค่า หรือสองเธรดแก้ข้อมูลเดียวกันพร้อมกัน</p>
<p>Rust ใช้สร้างได้ทั้งโปรแกรมบน Windows, เว็บเซิร์ฟเวอร์, เครื่องมือบรรทัดคำสั่ง, เกม ไปจนถึงระบบปฏิบัติการ</p>

<h2>ภาษาคอมไพล์ หมายความว่าอะไร</h2>
<p>โค้ด Rust ที่คุณเขียน (ไฟล์ <code>.rs</code>) คอมพิวเตอร์ยังรันไม่ได้ทันที ต้องผ่าน<b>คอมไพเลอร์</b> (<code>rustc</code>) ก่อน มันจะแปลงโค้ดเป็นไฟล์ <code>.exe</code> ที่รันได้เลย ขั้นตอนนี้เรียกว่า <b>build</b> ถ้าโค้ดผิด คอมไพเลอร์จะไม่ยอมสร้าง <code>.exe</code> และบอกว่าผิดตรงไหน</p>

<h2>Cargo: ผู้ช่วยจัดการโปรเจกต์</h2>
<p>ใน Rust แทบไม่มีใครเรียก <code>rustc</code> เอง เราใช้ <b>Cargo</b> ซึ่งมากับ Rust อยู่แล้ว Cargo ทำหน้าที่สร้างโปรเจกต์ build รันเทสต์ และดาวน์โหลดไลบรารี (ใน Rust เรียกไลบรารีว่า <b>crate</b>) จาก <a href="https://crates.io">crates.io</a></p>
<table>
  <tr><th>คำสั่ง</th><th>ทำอะไร</th><th>ใน Phai Studio</th></tr>
  <tr><td><code>cargo new ชื่อ</code></td><td>สร้างโปรเจกต์ใหม่</td><td>File &gt; New project</td></tr>
  <tr><td><code>cargo build</code></td><td>build</td><td><kbd>Ctrl+B</kbd></td></tr>
  <tr><td><code>cargo run</code></td><td>build แล้วรัน</td><td><kbd>Ctrl+F5</kbd></td></tr>
  <tr><td><code>cargo test</code></td><td>รันเทสต์</td><td><kbd>Ctrl+R, T</kbd></td></tr>
  <tr><td><code>cargo add ชื่อ</code></td><td>เพิ่ม crate</td><td>Project &gt; Manage Crates</td></tr>
</table>

<h2>โครงสร้างโปรเจกต์</h2>
<pre><code>hello/
├── Cargo.toml      ← ชื่อโปรเจกต์ เวอร์ชัน และ crate ที่ใช้
└── src/
    └── main.rs     ← โปรแกรมเริ่มทำงานที่นี่</code></pre>

<h2>โปรแกรมแรก</h2>
<pre><code>fn main() {
    println!("สวัสดี Rust!");
}</code></pre>
<ul>
  <li><code>fn</code> คือการประกาศฟังก์ชัน <code>main</code> เป็นฟังก์ชันพิเศษ โปรแกรมเริ่มที่นี่เสมอ</li>
  <li><code>{ }</code> ครอบสิ่งที่อยู่ในฟังก์ชัน</li>
  <li><code>println!</code> พิมพ์ข้อความแล้วขึ้นบรรทัดใหม่ เครื่องหมาย <code>!</code> บอกว่ามันเป็น <b>macro</b> (ตอนนี้คิดว่าเป็นฟังก์ชันชนิดพิเศษไปก่อน)</li>
  <li>ทุกคำสั่งจบด้วย <code>;</code></li>
</ul>

<h3>ใส่ค่าลงในข้อความ</h3>
<pre><code>fn main() {
    let name = "ไผ่";
    let age = 7;
    println!("ชื่อ {name} อายุ {age} ปี");
    println!("ปีหน้าอายุ {} ปี", age + 1);
}</code></pre>
<p>เขียนชื่อตัวแปรใน <code>{ }</code> ได้เลย หรือใช้ <code>{}</code> เปล่า ๆ แล้วใส่ค่าตามหลัง</p>

<h3>Comment</h3>
<pre><code>// บรรทัดนี้คอมไพเลอร์ไม่อ่าน ใช้อธิบายโค้ด
/* หรือแบบ
   หลายบรรทัด */
/// comment สามขีด ใช้อธิบายฟังก์ชันที่อยู่ข้างล่าง แสดงตอนเอาเมาส์ชี้</code></pre>

<div class="try">สร้างโปรเจกต์ใหม่แบบ console ใน Phai Studio หรือ <code>cargo new hello</code> แก้ <code>main.rs</code> ให้พิมพ์ชื่อของคุณสามบรรทัด กด <kbd>Ctrl+F5</kbd> แล้วลองลบ <code>;</code> ออกหนึ่งตัวเพื่อดูว่าคอมไพเลอร์บอกอะไร</div>
'''

BODY['rust-02.html'] = r'''
<h1>2. ตัวแปร ชนิดข้อมูล ฟังก์ชัน</h1>
<p class="lead">พื้นฐานที่ทุกโปรแกรมต้องใช้</p>

<h2>ตัวแปรด้วย let</h2>
<pre><code>let price = 120;
println!("{price}");</code></pre>
<p>ใน Rust ตัวแปร<b>แก้ไม่ได้</b>โดยค่าเริ่มต้น ถ้าต้องการแก้ ต้องบอกด้วย <code>mut</code> (มาจาก mutable)</p>
<pre><code>let mut count = 0;
count = count + 1;   // ได้ เพราะมี mut
count += 1;          // แบบย่อ

let total = 10;
total = 20;          // error: cannot assign twice to immutable variable</code></pre>
<div class="note">ทำไมต้องแก้ไม่ได้? เพราะส่วนใหญ่ค่าไม่ได้เปลี่ยน การบังคับให้เขียน <code>mut</code> ทำให้คนอ่านโค้ดรู้ทันทีว่าตัวแปรไหนจะเปลี่ยนค่า และคอมไพเลอร์ช่วยจับได้ถ้าเผลอแก้</div>

<h3>Shadowing: ประกาศชื่อเดิมซ้ำ</h3>
<pre><code>let input = "42";
let input: i32 = input.parse().unwrap();  // ชื่อเดิม แต่เป็นตัวแปรใหม่ ชนิดใหม่
println!("{}", input + 1);  // 43</code></pre>

<h2>ชนิดข้อมูลพื้นฐาน</h2>
<table>
  <tr><th>ชนิด</th><th>เก็บอะไร</th><th>ตัวอย่าง</th></tr>
  <tr><td><code>i32</code></td><td>จำนวนเต็ม (บวกหรือลบ) ใช้บ่อยที่สุด</td><td><code>-5</code>, <code>2024</code></td></tr>
  <tr><td><code>i64</code>, <code>u8</code>, <code>u32</code>, <code>u64</code></td><td>จำนวนเต็มขนาดอื่น <code>u</code> = ไม่ติดลบ</td><td><code>255u8</code></td></tr>
  <tr><td><code>usize</code></td><td>จำนวนเต็มที่ใช้นับตำแหน่งและขนาด</td><td><code>list.len()</code></td></tr>
  <tr><td><code>f64</code></td><td>ทศนิยม</td><td><code>3.14</code></td></tr>
  <tr><td><code>bool</code></td><td>จริง/เท็จ</td><td><code>true</code>, <code>false</code></td></tr>
  <tr><td><code>char</code></td><td>ตัวอักษรหนึ่งตัว (ใส่ <code>' '</code>)</td><td><code>'ก'</code>, <code>'A'</code></td></tr>
  <tr><td><code>&amp;str</code></td><td>ข้อความที่อ่านอย่างเดียว (ใส่ <code>" "</code>)</td><td><code>"สวัสดี"</code></td></tr>
  <tr><td><code>String</code></td><td>ข้อความที่แก้ไขและเพิ่มได้</td><td><code>String::from("ไผ่")</code></td></tr>
</table>
<p>ส่วนใหญ่ไม่ต้องบอกชนิด Rust เดาให้จากค่า แต่จะบอกเองก็ได้ด้วย <code>:</code></p>
<pre><code>let a = 5;          // Rust รู้ว่าเป็น i32
let b: f64 = 5.0;   // บอกเอง
let c = 10 / 3;     // 3  (จำนวนเต็มหารกันได้จำนวนเต็ม)
let d = 10.0 / 3.0; // 3.3333333333333335</code></pre>
<div class="warn">Rust ไม่แปลงชนิดให้อัตโนมัติ <code>5 + 2.0</code> คือ error ต้องแปลงเองด้วย <code>as</code> เช่น <code>5 as f64 + 2.0</code></div>

<h3>Tuple และ array</h3>
<pre><code>let point = (10, 20);            // tuple: หลายค่า ต่างชนิดกันได้
println!("x = {}", point.0);

let days = ["จ", "อ", "พ"];        // array: ชนิดเดียวกัน จำนวนคงที่
println!("{}", days[0]);          // "จ"
println!("มี {} วัน", days.len());</code></pre>

<h2>ฟังก์ชัน</h2>
<pre><code>fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &amp;str) {
    println!("สวัสดี {name}");
}

fn main() {
    let sum = add(3, 4);
    greet("ไผ่");
    println!("{sum}");
}</code></pre>
<ul>
  <li>พารามิเตอร์ต้องบอกชนิดเสมอ</li>
  <li><code>-&gt; i32</code> บอกชนิดของค่าที่คืน ถ้าไม่คืนอะไรก็ไม่ต้องเขียน</li>
  <li><b>บรรทัดสุดท้ายที่ไม่มี <code>;</code> คือค่าที่คืน</b> (เขียน <code>return a + b;</code> ก็ได้ แต่คนเขียน Rust นิยมแบบไม่มี <code>;</code>)</li>
</ul>
<div class="warn">เผลอใส่ <code>;</code> หลัง <code>a + b</code> จะได้ error "mismatched types" เพราะบรรทัดนั้นกลายเป็นคำสั่ง ไม่คืนค่าแล้ว</div>

<h3>Block ก็มีค่า</h3>
<pre><code>let grade = {
    let score = 75;
    if score &gt;= 50 { "ผ่าน" } else { "ไม่ผ่าน" }
};</code></pre>

<div class="try">เขียนฟังก์ชัน <code>fn area(width: f64, height: f64) -&gt; f64</code> คืนพื้นที่สี่เหลี่ยม แล้วพิมพ์พื้นที่ของห้องขนาด 3.5 × 4 เมตร จากนั้นเขียน <code>fn celsius_to_f(c: f64) -&gt; f64</code> (สูตร c × 9 / 5 + 32)</div>
'''

BODY['rust-03.html'] = r'''
<h1>3. เงื่อนไขและการวนซ้ำ</h1>
<p class="lead">ให้โปรแกรมเลือกทำ และทำซ้ำ</p>

<h2>if / else</h2>
<pre><code>let temp = 36;
if temp &gt; 37 {
    println!("มีไข้");
} else if temp &lt; 35 {
    println!("ตัวเย็นเกินไป");
} else {
    println!("ปกติ");
}</code></pre>
<p>เงื่อนไขต้องเป็น <code>bool</code> เท่านั้น <code>if count { }</code> แบบภาษาอื่นใช้ไม่ได้ ต้องเขียน <code>if count != 0 { }</code></p>
<p>ตัวเปรียบเทียบ: <code>==</code> เท่ากับ, <code>!=</code> ไม่เท่า, <code>&lt;</code> <code>&gt;</code> <code>&lt;=</code> <code>&gt;=</code> และเชื่อมเงื่อนไขด้วย <code>&amp;&amp;</code> (และ), <code>||</code> (หรือ), <code>!</code> (ไม่)</p>

<h3>if ให้ค่าได้</h3>
<pre><code>let fee = if age &lt; 12 { 50 } else { 100 };</code></pre>

<h2>for: วนตามช่วงหรือตามรายการ</h2>
<pre><code>for i in 1..=5 {        // 1 ถึง 5
    println!("รอบที่ {i}");
}

for i in 0..3 {         // 0, 1, 2 (ไม่รวม 3)
    println!("{i}");
}

let fruits = ["มะม่วง", "ทุเรียน", "มังคุด"];
for f in fruits {
    println!("{f}");
}

for (i, f) in fruits.iter().enumerate() {   // ได้ลำดับด้วย
    println!("{}. {f}", i + 1);
}</code></pre>

<h2>while และ loop</h2>
<pre><code>let mut n = 10;
while n &gt; 0 {
    println!("{n}");
    n -= 1;
}

let mut tries = 0;
let result = loop {        // วนไปเรื่อย ๆ จนกว่าจะ break
    tries += 1;
    if tries == 3 {
        break tries * 10;  // ออกจาก loop พร้อมคืนค่า
    }
};
println!("{result}");      // 30</code></pre>
<p><code>break</code> ออกจากวง, <code>continue</code> ข้ามไปรอบถัดไป</p>

<h2>match: เลือกตามค่า</h2>
<pre><code>let day = 3;
let name = match day {
    1 =&gt; "จันทร์",
    2 =&gt; "อังคาร",
    3 =&gt; "พุธ",
    4 | 5 =&gt; "พฤหัสหรือศุกร์",
    6..=7 =&gt; "สุดสัปดาห์",
    _ =&gt; "ไม่มีวันนี้",
};</code></pre>
<p><code>match</code> ต้องครอบคลุม<b>ทุกค่าที่เป็นไปได้</b> <code>_</code> หมายถึง "อย่างอื่นทั้งหมด" ถ้าลืม คอมไพเลอร์จะเตือนทันที นี่คือหนึ่งในเหตุผลที่โปรแกรม Rust บั๊กน้อย และจะเห็นพลังของ <code>match</code> จริง ๆ ใน<a href="rust-05.html">บทที่ 5</a></p>

<div class="try">
<ol>
<li>พิมพ์สูตรคูณแม่ 2 ถึงแม่ 12 (ใช้ <code>for</code> ซ้อน <code>for</code>)</li>
<li>FizzBuzz: พิมพ์ 1 ถึง 30 แต่ถ้าหาร 3 ลงตัวพิมพ์ <code>Fizz</code> หาร 5 ลงตัวพิมพ์ <code>Buzz</code> หารได้ทั้งสองพิมพ์ <code>FizzBuzz</code> (เศษจากการหารคือ <code>%</code>)</li>
<li>เขียนฟังก์ชันรับคะแนน แล้วคืนเกรด A B C D F ด้วย <code>match</code> และช่วง <code>80..=100</code></li>
</ol>
</div>
'''

BODY['rust-04.html'] = r'''
<h1>4. Ownership และการยืม</h1>
<p class="lead">เรื่องที่ทำให้ Rust ต่างจากทุกภาษา อ่านช้า ๆ ไม่ต้องรีบ</p>

<h2>ปัญหาที่ Rust แก้</h2>
<p>ทุกโปรแกรมต้องขอหน่วยความจำมาใช้ และต้องคืนเมื่อใช้เสร็จ ภาษาอย่าง C ให้โปรแกรมเมอร์คืนเอง ลืมคืนก็เปลือง คืนซ้ำหรือใช้หลังคืนก็พัง ภาษาอย่าง C# หรือ Java มีตัวเก็บขยะ (garbage collector) คอยตามเก็บ แต่ต้องแลกด้วยความเร็วและการหยุดชะงักเป็นระยะ</p>
<p>Rust ใช้วิธีที่สาม: <b>กฎความเป็นเจ้าของ</b> ที่คอมไพเลอร์ตรวจตอน build จึงคืนหน่วยความจำได้ถูกจังหวะเสมอ โดยไม่ต้องมีตัวเก็บขยะ</p>

<h2>กฎสามข้อ</h2>
<ol>
  <li>ค่าแต่ละค่ามี<b>เจ้าของ</b>หนึ่งคน (คือตัวแปร)</li>
  <li>ในเวลาหนึ่งมีเจ้าของได้<b>คนเดียว</b></li>
  <li>เมื่อเจ้าของหมดอายุ (ออกจาก <code>{ }</code> ที่มันอยู่) ค่านั้นถูกคืนทันที</li>
</ol>

<h2>การย้าย (move)</h2>
<p>นึกถึงหนังสือหนึ่งเล่ม ถ้าผมให้หนังสือคุณ หนังสือก็เป็นของคุณ ผมไม่มีแล้ว</p>
<pre><code>let a = String::from("หนังสือ");
let b = a;              // ย้ายความเป็นเจ้าของจาก a ไป b
println!("{b}");        // ได้
println!("{a}");        // error: borrow of moved value: `a`</code></pre>
<p>ส่งให้ฟังก์ชันก็คือการย้ายเหมือนกัน:</p>
<pre><code>fn read(book: String) {
    println!("อ่าน {book}");
}   // book หมดอายุตรงนี้ หนังสือถูกคืน

fn main() {
    let mine = String::from("นิยาย");
    read(mine);
    // println!("{mine}");  // error: mine ถูกย้ายไปแล้ว
}</code></pre>
<div class="note">ชนิดเล็ก ๆ อย่าง <code>i32</code>, <code>f64</code>, <code>bool</code>, <code>char</code> ไม่ถูกย้าย แต่ถูก<b>คัดลอก</b> เพราะคัดลอกถูกมาก <code>let b = a;</code> แล้วใช้ <code>a</code> ต่อได้ตามปกติ</div>

<h2>การยืม (borrow) ด้วย &amp;</h2>
<p>ส่วนใหญ่เราไม่ได้อยากยกหนังสือให้ใคร แค่<b>ให้ยืมอ่าน</b> แล้วเขาก็คืน ใน Rust เขียนด้วย <code>&amp;</code></p>
<pre><code>fn read(book: &amp;String) {      // ยืม ไม่ได้เอาไป
    println!("อ่าน {book}");
}

fn main() {
    let mine = String::from("นิยาย");
    read(&amp;mine);                // ให้ยืม
    read(&amp;mine);                // ให้ยืมอีกก็ได้
    println!("ยังเป็นของฉัน: {mine}");
}</code></pre>

<h3>ยืมไปแก้ด้วย &amp;mut</h3>
<pre><code>fn add_note(book: &amp;mut String) {
    book.push_str(" (มีโน้ต)");
}

fn main() {
    let mut mine = String::from("สมุด");
    add_note(&amp;mut mine);
    println!("{mine}");   // สมุด (มีโน้ต)
}</code></pre>

<h2>กฎของการยืม</h2>
<p>ในช่วงเวลาหนึ่ง เลือกได้อย่างเดียว:</p>
<ul>
  <li>ยืมอ่าน (<code>&amp;</code>) กี่คนก็ได้ <b>หรือ</b></li>
  <li>ยืมแก้ (<code>&amp;mut</code>) ได้คนเดียว และระหว่างนั้นห้ามคนอื่นยืมเลย</li>
</ul>
<p>เหมือนเอกสารบนโต๊ะ หลายคนอ่านพร้อมกันได้ แต่ถ้ามีคนกำลังแก้ คนอื่นต้องรอ ไม่อย่างนั้นจะมีคนอ่านเจอข้อความที่แก้ไปครึ่งเดียว กฎนี้แหละที่ทำให้ Rust ป้องกันบั๊กประเภท "ข้อมูลเปลี่ยนใต้มือ" ได้ตั้งแต่ตอนคอมไพล์</p>
<pre><code>let mut list = vec![1, 2, 3];
let first = &amp;list[0];   // ยืมอ่าน
list.push(4);           // error: ยืมแก้ ขณะที่ first ยังยืมอ่านอยู่
println!("{first}");</code></pre>
<p>ทำไมถึงห้าม? เพราะ <code>push</code> อาจทำให้รายการย้ายไปอยู่ที่ใหม่ในหน่วยความจำ แล้ว <code>first</code> จะชี้ไปที่ที่ว่างเปล่า ภาษาอื่นปล่อยให้พังตอนรัน Rust ไม่ยอมให้ build</p>

<h2>เมื่อติดเรื่อง ownership ทำอย่างไร</h2>
<ul>
  <li>ฟังก์ชันแค่อ่านค่า: รับเป็น <code>&amp;</code> (สำหรับข้อความ ใช้ <code>&amp;str</code> ดีกว่า <code>&amp;String</code>)</li>
  <li>ฟังก์ชันต้องแก้ค่า: รับเป็น <code>&amp;mut</code></li>
  <li>อยากได้สำเนาแยกจริง ๆ: <code>.clone()</code> (ใช้ได้ ไม่ผิด แค่เปลืองกว่านิดหน่อย มือใหม่ใช้ไปก่อนได้)</li>
  <li>อ่านข้อความ error ให้จบ คอมไพเลอร์มักบอกวิธีแก้ไว้ท้ายข้อความ ดู<a href="rust-10.html">บทที่ 10</a></li>
</ul>

<div class="try">
<ol>
<li>เขียน <code>fn shout(text: &amp;str) -&gt; String</code> คืนข้อความเป็นตัวพิมพ์ใหญ่ (<code>text.to_uppercase()</code>) เรียกสองครั้งด้วยตัวแปรเดิม</li>
<li>เขียน <code>fn double_all(numbers: &amp;mut Vec&lt;i32&gt;)</code> ที่คูณทุกตัวด้วย 2 (ใช้ <code>for n in numbers.iter_mut() { *n *= 2; }</code>)</li>
<li>ลองเขียนโค้ดที่ผิดกฎในหน้านี้ แล้วอ่าน error ที่ได้</li>
</ol>
</div>
'''

BODY['rust-05.html'] = r'''
<h1>5. struct, enum และ match</h1>
<p class="lead">สร้างชนิดข้อมูลของตัวเอง</p>

<h2>struct: รวมข้อมูลหลายอย่างเป็นก้อนเดียว</h2>
<pre><code>struct Product {
    name: String,
    price: f64,
    stock: u32,
}

fn main() {
    let mut p = Product {
        name: String::from("กาแฟ"),
        price: 45.0,
        stock: 10,
    };
    p.stock -= 1;
    println!("{} เหลือ {} ชิ้น", p.name, p.stock);
}</code></pre>

<h2>impl: ใส่ความสามารถให้ struct</h2>
<pre><code>impl Product {
    // ฟังก์ชันสร้าง (เรียกว่า associated function) ไม่มี self
    fn new(name: &amp;str, price: f64) -&gt; Product {
        Product { name: name.to_string(), price, stock: 0 }
    }

    // method: มี &amp;self = ยืมตัวมันเองมาอ่าน
    fn total_value(&amp;self) -&gt; f64 {
        self.price * self.stock as f64
    }

    // &amp;mut self = ยืมตัวมันเองมาแก้
    fn restock(&amp;mut self, amount: u32) {
        self.stock += amount;
    }
}

fn main() {
    let mut tea = Product::new("ชาเย็น", 40.0);
    tea.restock(5);
    println!("มูลค่า {}", tea.total_value());   // 200
}</code></pre>

<h3>พิมพ์ struct ออกมาดู</h3>
<pre><code>#[derive(Debug)]
struct Point { x: i32, y: i32 }

let p = Point { x: 1, y: 2 };
println!("{:?}", p);    // Point { x: 1, y: 2 }
println!("{:#?}", p);   // แบบหลายบรรทัด อ่านง่าย</code></pre>

<h2>enum: ค่าที่เป็นได้หนึ่งในหลายแบบ</h2>
<pre><code>enum Payment {
    Cash,
    Card(String),               // เก็บเลขบัตรไว้ด้วย
    Transfer { bank: String, reference: u64 },
}</code></pre>
<p>enum ของ Rust ไม่ใช่แค่รายชื่อ แต่ละแบบเก็บข้อมูลของตัวเองได้ ใช้คู่กับ <code>match</code>:</p>
<pre><code>fn describe(p: &amp;Payment) -&gt; String {
    match p {
        Payment::Cash =&gt; "เงินสด".to_string(),
        Payment::Card(number) =&gt; format!("บัตร {number}"),
        Payment::Transfer { bank, reference } =&gt; format!("โอนผ่าน {bank} อ้างอิง {reference}"),
    }
}</code></pre>
<p>ถ้าวันหนึ่งเพิ่ม <code>Payment::QrCode</code> คอมไพเลอร์จะชี้ทุก <code>match</code> ที่ยังไม่ได้จัดการแบบใหม่ ไม่มีทางลืม</p>

<h2>Option: ค่าที่อาจไม่มี</h2>
<p>Rust ไม่มี <code>null</code> ค่าที่อาจไม่มีใช้ <code>Option</code> ซึ่งเป็น enum ธรรมดา:</p>
<pre><code>enum Option&lt;T&gt; {   // มีอยู่แล้วในภาษา ไม่ต้องเขียนเอง
    Some(T),        // มีค่า
    None,           // ไม่มี
}</code></pre>
<pre><code>fn find_price(name: &amp;str) -&gt; Option&lt;f64&gt; {
    match name {
        "กาแฟ" =&gt; Some(45.0),
        "ชา" =&gt; Some(40.0),
        _ =&gt; None,
    }
}

fn main() {
    match find_price("โกโก้") {
        Some(p) =&gt; println!("ราคา {p}"),
        None =&gt; println!("ไม่มีขาย"),
    }

    // แบบสั้น เมื่อสนใจแค่กรณีที่มีค่า
    if let Some(p) = find_price("ชา") {
        println!("ชา {p} บาท");
    }

    // ใช้ค่าสำรองเมื่อไม่มี
    let price = find_price("น้ำ").unwrap_or(0.0);
}</code></pre>
<div class="note">เพราะ <code>Option</code> บังคับให้จัดการกรณี <code>None</code> ก่อนใช้ค่า บั๊กแบบ "NullReferenceException" ที่พบบ่อยในภาษาอื่นจึงแทบไม่เกิดใน Rust</div>

<div class="try">
<ol>
<li>สร้าง <code>struct Student { name: String, scores: Vec&lt;u32&gt; }</code> พร้อม method <code>average(&amp;self) -&gt; f64</code></li>
<li>สร้าง <code>enum Shape { Circle(f64), Rect(f64, f64) }</code> และฟังก์ชัน <code>area</code> ที่ใช้ <code>match</code></li>
<li>เขียนฟังก์ชันหาตัวเลขที่มากที่สุดในรายการ คืน <code>Option&lt;i32&gt;</code> (รายการว่างคืน <code>None</code>)</li>
</ol>
</div>
'''

BODY['rust-06.html'] = r'''
<h1>6. จัดการข้อผิดพลาด</h1>
<p class="lead">panic, Result และเครื่องหมาย ?</p>

<h2>สองแบบของข้อผิดพลาด</h2>
<ul>
  <li><b>ข้อผิดพลาดที่คาดได้</b> เช่น ไฟล์ไม่มี ผู้ใช้พิมพ์ตัวเลขผิด เน็ตหลุด ควรจัดการและบอกผู้ใช้ ใช้ <code>Result</code></li>
  <li><b>บั๊ก</b> สิ่งที่ไม่ควรเกิดเลยถ้าโค้ดถูก เช่น อ่านตำแหน่งเกินขนาดรายการ Rust จะ <code>panic</code> คือหยุดโปรแกรมพร้อมบอกว่าเกิดที่ไหน</li>
</ul>

<h2>panic</h2>
<pre><code>let list = vec![1, 2, 3];
let x = list[10];   // panic: index out of bounds: the len is 3 but the index is 10
panic!("ไม่ควรมาถึงตรงนี้");  // สั่ง panic เอง</code></pre>
<p>ใน Phai Studio ถ้ารันด้วย <kbd>F5</kbd> ดีบักเกอร์จะหยุดตรงจุดที่ panic ให้ดูค่าตัวแปรและ Call Stack ได้ทันที</p>

<h2>Result: สำเร็จ หรือ ผิดพลาด</h2>
<pre><code>enum Result&lt;T, E&gt; {   // มีอยู่แล้วในภาษา
    Ok(T),             // สำเร็จ ได้ค่า T
    Err(E),            // ผิดพลาด ได้รายละเอียด E
}</code></pre>
<pre><code>fn main() {
    let input = "12a";
    match input.parse::&lt;i32&gt;() {
        Ok(n) =&gt; println!("ได้ตัวเลข {n}"),
        Err(e) =&gt; println!("ไม่ใช่ตัวเลข: {e}"),
    }
}</code></pre>

<h3>เขียนฟังก์ชันที่คืน Result เอง</h3>
<pre><code>fn withdraw(balance: f64, amount: f64) -&gt; Result&lt;f64, String&gt; {
    if amount &lt;= 0.0 {
        return Err("จำนวนเงินต้องมากกว่าศูนย์".to_string());
    }
    if amount &gt; balance {
        return Err(format!("เงินไม่พอ มี {balance} แต่จะถอน {amount}"));
    }
    Ok(balance - amount)
}</code></pre>

<h2>เครื่องหมาย ? : ส่งต่อข้อผิดพลาด</h2>
<p>เมื่อฟังก์ชันของเราเองก็คืน <code>Result</code> ใส่ <code>?</code> หลังสิ่งที่อาจผิดพลาด ถ้าผิด มันจะคืน <code>Err</code> ออกจากฟังก์ชันทันที ถ้าสำเร็จ ได้ค่าข้างในมาใช้ต่อ</p>
<pre><code>use std::fs;

fn read_total(path: &amp;str) -&gt; Result&lt;i32, Box&lt;dyn std::error::Error&gt;&gt; {
    let text = fs::read_to_string(path)?;     // ไฟล์ไม่มี? คืน Err เลย
    let mut total = 0;
    for line in text.lines() {
        total += line.trim().parse::&lt;i32&gt;()?;  // ไม่ใช่ตัวเลข? คืน Err เลย
    }
    Ok(total)
}

fn main() {
    match read_total("numbers.txt") {
        Ok(t) =&gt; println!("รวม {t}"),
        Err(e) =&gt; println!("อ่านไม่ได้: {e}"),
    }
}</code></pre>
<div class="note"><code>Box&lt;dyn std::error::Error&gt;</code> แปลว่า "ข้อผิดพลาดชนิดไหนก็ได้" สะดวกเวลาฟังก์ชันอาจผิดได้หลายแบบ</div>

<h2>unwrap และ expect</h2>
<pre><code>let n: i32 = "42".parse().unwrap();                 // ผิดเมื่อไรก็ panic
let n: i32 = "42".parse().expect("ต้องเป็นตัวเลข");  // panic พร้อมข้อความที่เราเขียน</code></pre>
<p>ใช้ได้ตอนทดลองหรือเมื่อมั่นใจว่าไม่มีทางผิด แต่ในโปรแกรมจริงที่รับข้อมูลจากผู้ใช้ ควรใช้ <code>match</code>, <code>if let</code> หรือ <code>?</code> แทน</p>

<h2>วิธีที่ใช้บ่อย</h2>
<table>
  <tr><th>เขียน</th><th>ได้อะไร</th></tr>
  <tr><td><code>x.unwrap_or(0)</code></td><td>ค่าข้างใน หรือ 0 ถ้าผิด/ไม่มี</td></tr>
  <tr><td><code>x.unwrap_or_default()</code></td><td>ค่าข้างใน หรือค่าว่างของชนิดนั้น</td></tr>
  <tr><td><code>x.is_ok()</code>, <code>x.is_err()</code></td><td>สำเร็จไหม</td></tr>
  <tr><td><code>x.ok()</code></td><td>แปลง <code>Result</code> เป็น <code>Option</code></td></tr>
  <tr><td><code>x.map_err(|e| e.to_string())</code></td><td>เปลี่ยนชนิดของข้อผิดพลาด</td></tr>
</table>

<div class="try">
<ol>
<li>ในโปรแกรมฟอร์มของคุณ อ่านตัวเลขจากช่องพิมพ์ด้วย <code>parse</code> ถ้าผิดให้แสดงข้อความเตือนใน label แทนการ panic</li>
<li>เขียน <code>fn divide(a: f64, b: f64) -&gt; Result&lt;f64, String&gt;</code> ที่คืน Err เมื่อ b เป็นศูนย์</li>
</ol>
</div>
'''

BODY['rust-07.html'] = r'''
<h1>7. Vec, String, HashMap</h1>
<p class="lead">เก็บข้อมูลหลายชิ้น และเครื่องมือจัดการรายการ</p>

<h2>Vec: รายการที่ยืดหดได้</h2>
<pre><code>let mut scores: Vec&lt;i32&gt; = Vec::new();
scores.push(80);
scores.push(65);

let names = vec!["ก้อง", "นิด", "ต้น"];   // สร้างพร้อมค่า

println!("มี {} คน", names.len());
println!("คนแรก {}", names[0]);         // เกินขนาด = panic
if let Some(n) = names.get(5) {        // ปลอดภัย ได้ Option
    println!("{n}");
}

for s in &amp;scores {                     // วนแบบยืม รายการยังอยู่
    println!("{s}");
}
scores.sort();
scores.retain(|s| *s &gt;= 70);           // เก็บเฉพาะที่ผ่านเงื่อนไข
let last = scores.pop();               // เอาตัวท้ายออก ได้ Option</code></pre>

<h2>String กับ &amp;str</h2>
<table>
  <tr><th></th><th><code>String</code></th><th><code>&amp;str</code></th></tr>
  <tr><td>คืออะไร</td><td>ข้อความที่เป็นเจ้าของเอง แก้ไขได้</td><td>มุมมองไปยังข้อความที่มีอยู่แล้ว อ่านอย่างเดียว</td></tr>
  <tr><td>ได้มาจาก</td><td><code>String::from("..")</code>, <code>"..".to_string()</code>, <code>format!(..)</code></td><td><code>"ข้อความในเครื่องหมายคำพูด"</code>, <code>&amp;my_string</code></td></tr>
  <tr><td>ใช้เป็นพารามิเตอร์</td><td>เมื่อฟังก์ชันต้องเก็บข้อความไว้</td><td>เมื่อแค่อ่าน (ส่วนใหญ่)</td></tr>
</table>
<pre><code>let mut s = String::from("สวัสดี");
s.push_str(" ครับ");
s.push('!');
let full = format!("{s} วันนี้ {}", 18);

println!("{}", s.contains("ครับ"));     // true
println!("{}", s.replace("ครับ", "ค่ะ"));
for word in "ไผ่ สตูดิโอ ฟรี".split(' ') {
    println!("{word}");
}
let trimmed = "  มีช่องว่าง  ".trim();</code></pre>
<div class="warn">ข้อความใน Rust เก็บแบบ UTF-8 อักษรไทยหนึ่งตัวใช้ 3 ไบต์ <code>s.len()</code> จึงนับ<b>ไบต์</b> ไม่ใช่ตัวอักษร นับตัวอักษรใช้ <code>s.chars().count()</code> และอย่าตัดข้อความด้วย <code>&amp;s[0..2]</code> กับภาษาไทย เพราะอาจตัดกลางตัวอักษรแล้ว panic ใช้ <code>s.chars().take(2).collect::&lt;String&gt;()</code> แทน</div>

<h2>HashMap: เก็บคู่ คีย์ → ค่า</h2>
<pre><code>use std::collections::HashMap;

let mut stock: HashMap&lt;String, u32&gt; = HashMap::new();
stock.insert("กาแฟ".to_string(), 10);
stock.insert("ชา".to_string(), 4);

if let Some(n) = stock.get("กาแฟ") {
    println!("กาแฟเหลือ {n}");
}

// นับคำ: ถ้ายังไม่มีให้เริ่มที่ 0 แล้วบวก 1
let mut counts = HashMap::new();
for word in "ชา กาแฟ ชา ชา".split(' ') {
    *counts.entry(word).or_insert(0) += 1;
}

for (name, n) in &amp;stock {
    println!("{name}: {n}");
}</code></pre>

<h2>Iterator และ closure</h2>
<p>closure คือฟังก์ชันสั้น ๆ ไม่มีชื่อ เขียนแบบ <code>|x| x * 2</code> ใช้คู่กับ iterator เพื่อแปลงและกรองรายการได้สั้นและอ่านง่าย</p>
<pre><code>let prices = vec![120, 45, 300, 80];

let total: i32 = prices.iter().sum();
let expensive: Vec&lt;i32&gt; = prices.iter().filter(|p| **p &gt; 100).cloned().collect();
let with_vat: Vec&lt;f64&gt; = prices.iter().map(|p| *p as f64 * 1.07).collect();
let max = prices.iter().max();                    // Option
let any_free = prices.iter().any(|p| *p == 0);    // false
let position = prices.iter().position(|p| *p == 300);  // Some(2)</code></pre>
<div class="note">ถ้ายังไม่ถนัด iterator เขียนด้วย <code>for</code> ธรรมดาก็ได้ผลเหมือนกัน ค่อย ๆ เปลี่ยนเมื่อคุ้นแล้ว</div>

<div class="try">
<ol>
<li>รับข้อความยาวหนึ่งย่อหน้า นับว่าแต่ละคำปรากฏกี่ครั้งด้วย <code>HashMap</code> แล้วพิมพ์คำที่ใช้บ่อยที่สุด</li>
<li>จากรายการคะแนน หาค่าเฉลี่ย คะแนนสูงสุด และจำนวนคนที่ได้มากกว่า 50 ด้วย iterator</li>
</ol>
</div>
'''

BODY['rust-08.html'] = r'''
<h1>8. trait และ generic</h1>
<p class="lead">เขียนโค้ดครั้งเดียว ใช้ได้กับหลายชนิด</p>

<h2>trait: ความสามารถที่หลายชนิดมีร่วมกัน</h2>
<p>trait คล้าย interface ในภาษาอื่น บอกว่า "ชนิดใดที่มี trait นี้ ต้องทำสิ่งเหล่านี้ได้"</p>
<pre><code>trait Describe {
    fn describe(&amp;self) -&gt; String;

    // method ที่มีค่าเริ่มต้น ชนิดไหนไม่เขียนเองก็ได้อันนี้
    fn shout(&amp;self) -&gt; String {
        self.describe().to_uppercase()
    }
}

struct Cat { name: String }
struct Car { brand: String, year: u32 }

impl Describe for Cat {
    fn describe(&amp;self) -&gt; String {
        format!("แมวชื่อ {}", self.name)
    }
}

impl Describe for Car {
    fn describe(&amp;self) -&gt; String {
        format!("รถ {} ปี {}", self.brand, self.year)
    }
}</code></pre>

<h3>รับอะไรก็ได้ที่มี trait</h3>
<pre><code>fn show(item: &amp;impl Describe) {
    println!("{}", item.describe());
}

fn main() {
    show(&amp;Cat { name: "ส้ม".into() });
    show(&amp;Car { brand: "ไผ่มอเตอร์".into(), year: 2024 });

    // รายการที่ปนหลายชนิด: ใช้ Box&lt;dyn Trait&gt;
    let things: Vec&lt;Box&lt;dyn Describe&gt;&gt; = vec![
        Box::new(Cat { name: "ดำ".into() }),
        Box::new(Car { brand: "ทดลอง".into(), year: 2020 }),
    ];
    for t in &amp;things {
        println!("{}", t.describe());
    }
}</code></pre>
<div class="note">ใน Phai Studio ฟอร์มที่คุณออกแบบจะได้ trait ชื่อประมาณ <code>MainWindowEvents</code> แล้วโค้ดของคุณ <code>impl</code> trait นั้น ฟังก์ชันอย่าง <code>greet_clicked</code> คือ method ของ trait นั่นเอง</div>

<h2>trait ที่ Rust มีให้และใช้บ่อย</h2>
<table>
  <tr><th>trait</th><th>ทำให้ทำอะไรได้</th><th>ได้มาด้วย</th></tr>
  <tr><td><code>Debug</code></td><td>พิมพ์ด้วย <code>{:?}</code></td><td><code>#[derive(Debug)]</code></td></tr>
  <tr><td><code>Clone</code></td><td><code>.clone()</code> สำเนา</td><td><code>#[derive(Clone)]</code></td></tr>
  <tr><td><code>PartialEq</code></td><td>เทียบด้วย <code>==</code></td><td><code>#[derive(PartialEq)]</code></td></tr>
  <tr><td><code>Default</code></td><td><code>Default::default()</code> ค่าเริ่มต้น</td><td><code>#[derive(Default)]</code></td></tr>
  <tr><td><code>Display</code></td><td>พิมพ์ด้วย <code>{}</code></td><td>เขียน <code>impl</code> เอง</td></tr>
</table>
<pre><code>use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Money(f64);

impl fmt::Display for Money {
    fn fmt(&amp;self, f: &amp;mut fmt::Formatter) -&gt; fmt::Result {
        write!(f, "{:.2} บาท", self.0)
    }
}
// println!("{}", Money(45.5));  →  45.50 บาท</code></pre>

<h2>generic: ชนิดเป็นตัวแปร</h2>
<pre><code>// T คือชนิดอะไรก็ได้ ที่เทียบมากน้อยได้ (PartialOrd) และคัดลอกได้ (Copy)
fn largest&lt;T: PartialOrd + Copy&gt;(list: &amp;[T]) -&gt; Option&lt;T&gt; {
    let mut best = *list.first()?;
    for &amp;x in list {
        if x &gt; best {
            best = x;
        }
    }
    Some(best)
}

fn main() {
    println!("{:?}", largest(&amp;[3, 9, 2]));        // Some(9)
    println!("{:?}", largest(&amp;[1.5, 0.2]));       // Some(1.5)
    println!("{:?}", largest(&amp;['ก', 'ฮ', 'ม']));  // Some('ฮ')
}</code></pre>
<p><code>Vec&lt;T&gt;</code>, <code>Option&lt;T&gt;</code>, <code>Result&lt;T, E&gt;</code> ที่ใช้มาตลอดก็คือ generic</p>

<h2>Lifetime (รู้จักไว้ก่อน)</h2>
<p>บางครั้งจะเห็น <code>&amp;'a str</code> เครื่องหมาย <code>'a</code> คือ <b>lifetime</b> บอกคอมไพเลอร์ว่าค่าที่ยืมมาอยู่ได้นานแค่ไหน ส่วนใหญ่ Rust เดาให้เอง เมื่อจำเป็นต้องเขียน คอมไพเลอร์จะบอกพร้อมตัวอย่าง มือใหม่ยังไม่ต้องกังวลเรื่องนี้</p>
<pre><code>// คืนข้อความที่ยาวกว่า: ผลลัพธ์อยู่ได้นานเท่ากับ a และ b
fn longer&lt;'a&gt;(a: &amp;'a str, b: &amp;'a str) -&gt; &amp;'a str {
    if a.chars().count() &gt;= b.chars().count() { a } else { b }
}</code></pre>

<div class="try">สร้าง trait <code>Area</code> ที่มี <code>fn area(&amp;self) -&gt; f64</code> ให้ <code>Circle</code> และ <code>Square</code> ใส่ทั้งสองลงใน <code>Vec&lt;Box&lt;dyn Area&gt;&gt;</code> แล้วหาพื้นที่รวม</div>
'''

BODY['rust-09.html'] = r'''
<h1>9. โมดูล crate และเทสต์</h1>
<p class="lead">จัดโค้ดให้เป็นระเบียบ ใช้ไลบรารีของคนอื่น และพิสูจน์ว่าโค้ดถูก</p>

<h2>แยกโค้ดเป็นหลายไฟล์ด้วย mod</h2>
<pre><code>src/
├── main.rs
├── shop.rs        ← โมดูล shop
└── report.rs      ← โมดูล report</code></pre>
<pre><code>// main.rs
mod shop;           // บอกว่ามีไฟล์ shop.rs
mod report;

use shop::Product;  // ดึงชื่อมาใช้สั้น ๆ

fn main() {
    let p = Product::new("กาแฟ", 45.0);
    report::print(&amp;p);
}</code></pre>
<pre><code>// shop.rs
pub struct Product {       // pub = ไฟล์อื่นมองเห็น
    pub name: String,
    pub price: f64,
}

impl Product {
    pub fn new(name: &amp;str, price: f64) -&gt; Product {
        Product { name: name.into(), price }
    }
}</code></pre>
<pre><code>// report.rs
use crate::shop::Product;  // crate:: = เริ่มจากรากของโปรเจกต์

pub fn print(p: &amp;Product) {
    println!("{} ราคา {}", p.name, p.price);
}</code></pre>
<p>ทุกอย่างเป็น <b>ส่วนตัว</b>โดยค่าเริ่มต้น ต้องใส่ <code>pub</code> ถึงจะใช้จากไฟล์อื่นได้</p>

<h2>ใช้ crate จาก crates.io</h2>
<p>ใน Phai Studio เปิด <b>Project &gt; Manage Crates</b> ค้นหา แล้วกด Add (เท่ากับ <code>cargo add ชื่อ</code>) Cargo จะเพิ่มบรรทัดใน <code>Cargo.toml</code>:</p>
<pre><code>[dependencies]
rand = "0.8"
serde = { version = "1", features = ["derive"] }</code></pre>
<pre><code>use rand::Rng;

fn main() {
    let n = rand::thread_rng().gen_range(1..=6);
    println!("ทอยได้ {n}");
}</code></pre>
<div class="note">ก่อนใช้ crate ไหน ดูที่ crates.io ว่ามีคนดาวน์โหลดเยอะไหม อัปเดตล่าสุดเมื่อไร และอ่านตัวอย่างในหน้าเอกสาร (docs.rs)</div>

<h2>เขียนเทสต์</h2>
<p>เทสต์คือโค้ดที่ตรวจว่าโค้ดของเราทำงานถูก เขียนไว้ในไฟล์เดียวกันได้เลย:</p>
<pre><code>pub fn vat(price: f64) -&gt; f64 {
    price * 0.07
}

pub fn grade(score: u32) -&gt; char {
    match score {
        80..=100 =&gt; 'A',
        70..=79 =&gt; 'B',
        60..=69 =&gt; 'C',
        50..=59 =&gt; 'D',
        _ =&gt; 'F',
    }
}

#[cfg(test)]            // คอมไพล์เฉพาะตอนรันเทสต์
mod tests {
    use super::*;       // ใช้ทุกอย่างจากไฟล์ข้างบน

    #[test]
    fn vat_is_seven_percent() {
        assert_eq!(vat(100.0), 7.0);
    }

    #[test]
    fn grades_follow_the_ranges() {
        assert_eq!(grade(85), 'A');
        assert_eq!(grade(50), 'D');
        assert_eq!(grade(10), 'F');
    }

    #[test]
    #[should_panic]
    fn reading_past_the_end_panics() {
        let v: Vec&lt;i32&gt; = vec![];
        let _ = v[0];
    }
}</code></pre>
<table>
  <tr><th>macro</th><th>ตรวจอะไร</th></tr>
  <tr><td><code>assert!(x)</code></td><td><code>x</code> เป็นจริง</td></tr>
  <tr><td><code>assert_eq!(a, b)</code></td><td><code>a == b</code> ถ้าไม่เท่าจะแสดงทั้งสองค่า</td></tr>
  <tr><td><code>assert_ne!(a, b)</code></td><td><code>a != b</code></td></tr>
</table>
<p>ใน Phai Studio กด <kbd>Ctrl+R, T</kbd> ผลแต่ละเทสต์ขึ้นในแท็บ Tests ดับเบิลคลิกเทสต์ที่ไม่ผ่านเพื่อไปที่โค้ด และใช้ <b>Test &gt; Analyze Code Coverage</b> ดูว่าบรรทัดไหนยังไม่มีเทสต์ไปถึง (แถบแดงที่ขอบซ้าย)</p>

<div class="try">เพิ่มเทสต์ให้ฟังก์ชันที่คุณเขียนในบทก่อน ๆ อย่างน้อยฟังก์ชันละ 2 เทสต์ ลองทำให้เทสต์หนึ่งไม่ผ่านโดยตั้งใจ แล้วดูว่า Phai Studio แสดงผลอย่างไร</div>
'''

BODY['rust-10.html'] = r'''
<h1>10. อ่าน error ของคอมไพเลอร์</h1>
<p class="lead">คอมไพเลอร์ของ Rust ไม่ได้ดุ มันเป็นครูที่ละเอียดที่สุดที่คุณจะได้เจอ</p>

<h2>อ่าน error อย่างไร</h2>
<pre><code>error[E0382]: borrow of moved value: `name`
 --&gt; src/main.rs:4:20
  |
2 |     let name = String::from("ไผ่");
  |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
3 |     let other = name;
  |                 ---- value moved here
4 |     println!("{}", name);
  |                    ^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let other = name.clone();
  |                     ++++++++</code></pre>
<ol>
  <li><b>บรรทัดแรก</b> คือปัญหาโดยรวม และรหัส <code>E0382</code></li>
  <li><b><code>--&gt;</code></b> บอกไฟล์ บรรทัด และคอลัมน์ ใน Phai Studio ดับเบิลคลิกในแท็บ Errors เพื่อไปที่นั่น</li>
  <li><b>เส้นใต้</b> ชี้แต่ละจุดที่เกี่ยวข้อง พร้อมคำอธิบายสั้น ๆ อ่านตามลำดับเลขบรรทัด จะเห็นเรื่องราวว่าเกิดอะไรขึ้น</li>
  <li><b><code>help:</code></b> คือวิธีแก้ที่คอมไพเลอร์แนะนำ บ่อยครั้งแก้ตามนี้ได้เลย</li>
</ol>
<div class="note">ใน Phai Studio กด <kbd>Ctrl+.</kbd> ที่เส้นหยัก มักมีปุ่มแก้ให้อัตโนมัติ และถ้าอยากรู้ละเอียด เปิดหน้า <code>https://doc.rust-lang.org/error_codes/E0382.html</code> (เปลี่ยนเลขตามรหัส)</div>

<h2>error ที่มือใหม่เจอบ่อย</h2>
<table>
  <tr><th>ข้อความ (ส่วนสำคัญ)</th><th>แปลว่า</th><th>แก้อย่างไร</th></tr>
  <tr><td><code>cannot assign twice to immutable variable</code></td><td>แก้ค่าตัวแปรที่ไม่ได้ประกาศ <code>mut</code></td><td><code>let mut x</code></td></tr>
  <tr><td><code>mismatched types: expected i32, found f64</code></td><td>ชนิดไม่ตรงกับที่ต้องการ</td><td>แปลงด้วย <code>as</code> หรือ <code>.parse()</code> หรือแก้ชนิดที่ประกาศไว้</td></tr>
  <tr><td><code>mismatched types: expected i32, found ()</code></td><td>ฟังก์ชันควรคืนค่า แต่บรรทัดสุดท้ายมี <code>;</code></td><td>ลบ <code>;</code> ท้ายบรรทัดที่คืนค่า</td></tr>
  <tr><td><code>borrow of moved value</code></td><td>ใช้ตัวแปรหลังจากยกให้คนอื่นไปแล้ว</td><td>ส่งเป็น <code>&amp;x</code> แทน หรือ <code>.clone()</code></td></tr>
  <tr><td><code>cannot borrow `x` as mutable because it is also borrowed as immutable</code></td><td>แก้ค่าขณะที่ยังมีคนยืมอ่านอยู่</td><td>ใช้ค่าที่ยืมให้เสร็จก่อน แล้วค่อยแก้ หรือสลับลำดับ</td></tr>
  <tr><td><code>cannot borrow `x` as mutable, as it is not declared as mutable</code></td><td>ยืมแก้ตัวแปรที่ไม่มี <code>mut</code></td><td><code>let mut x</code></td></tr>
  <tr><td><code>cannot find value `x` in this scope</code></td><td>ไม่มีตัวแปรชื่อนี้ตรงนี้ (สะกดผิด หรือประกาศไว้ใน <code>{ }</code> อื่น)</td><td>ตรวจชื่อ และตำแหน่งที่ประกาศ</td></tr>
  <tr><td><code>cannot find type `HashMap` in this scope</code></td><td>ยังไม่ได้ <code>use</code></td><td><code>use std::collections::HashMap;</code> (หรือกด <kbd>Ctrl+.</kbd>)</td></tr>
  <tr><td><code>non-exhaustive patterns</code></td><td><code>match</code> ไม่ครบทุกกรณี</td><td>เพิ่มกรณีที่ขาด หรือ <code>_ =&gt;</code></td></tr>
  <tr><td><code>the `?` operator can only be used in a function that returns `Result`</code></td><td>ใช้ <code>?</code> ในฟังก์ชันที่ไม่ได้คืน <code>Result</code></td><td>เปลี่ยนชนิดที่คืน หรือใช้ <code>match</code></td></tr>
  <tr><td><code>`Foo` doesn't implement `std::fmt::Display`</code></td><td>พิมพ์ด้วย <code>{}</code> ไม่ได้</td><td>ใช้ <code>{:?}</code> กับ <code>#[derive(Debug)]</code> หรือเขียน <code>impl Display</code></td></tr>
  <tr><td><code>missing lifetime specifier</code></td><td>คืนค่าที่ยืมมา แต่คอมไพเลอร์ไม่รู้ว่ายืมจากอะไร</td><td>คืนเป็น <code>String</code> แทน <code>&amp;str</code> (ง่ายที่สุด) หรือใส่ lifetime ตาม help</td></tr>
</table>

<h2>warning ไม่ใช่ error</h2>
<p>warning (สีเหลือง) ไม่ทำให้ build ล้มเหลว แต่ควรอ่าน เช่น <code>unused variable</code> (ประกาศแล้วไม่ใช้ ถ้าตั้งใจให้ขึ้นต้นชื่อด้วย <code>_</code>) หรือ <code>unused import</code> และ <b>Build &gt; Run Code Analysis</b> (Clippy) จะแนะนำวิธีเขียนที่ดีขึ้นเพิ่มเติม</p>

<h2>เมื่อโปรแกรม panic ตอนรัน</h2>
<pre><code>thread 'main' panicked at src/main.rs:12:18:
index out of bounds: the len is 3 but the index is 5</code></pre>
<p>บอกไฟล์และบรรทัดชัดเจน รันด้วย <kbd>F5</kbd> แทน <kbd>Ctrl+F5</kbd> ดีบักเกอร์ของ Phai Studio จะหยุดตรงจุดที่ panic ให้ดูค่าตัวแปร (Locals) และเส้นทางที่มาถึง (Call Stack) ได้ทันที</p>

<h2>ทัศนคติที่ช่วยได้</h2>
<ul>
  <li>แก้ทีละ error จาก<b>ข้อแรก</b> error ข้อหลัง ๆ มักเป็นผลพวงจากข้อแรก</li>
  <li>error เยอะไม่ได้แปลว่าโค้ดแย่ บางทีลืม <code>}</code> ตัวเดียวก็ขึ้นเป็นสิบ</li>
  <li>ทุกครั้งที่คอมไพเลอร์ติ คือบั๊กหนึ่งตัวที่ไม่หลุดไปถึงผู้ใช้</li>
</ul>
'''

BODY['rust-11.html'] = r'''
<h1>11. เธรด และก้าวต่อไป</h1>
<p class="lead">ทำหลายอย่างพร้อมกันอย่างปลอดภัย แล้วจะเรียนอะไรต่อ</p>

<h2>เธรด: ทำงานพร้อมกัน</h2>
<pre><code>use std::thread;
use std::time::Duration;

fn main() {
    let worker = thread::spawn(|| {
        for i in 1..=3 {
            println!("งานเบื้องหลัง {i}");
            thread::sleep(Duration::from_millis(100));
        }
        42   // ค่าที่เธรดคืน
    });

    println!("main ทำอย่างอื่นไปพลาง ๆ");
    let answer = worker.join().unwrap();   // รอให้เธรดเสร็จ
    println!("ได้ {answer}");
}</code></pre>
<div class="note">ในโปรแกรมฟอร์ม งานที่ใช้เวลานาน (ดาวน์โหลด คำนวณหนัก อ่านไฟล์ใหญ่) ควรทำในเธรดแยก ไม่อย่างนั้นหน้าต่างจะค้างจนกว่างานจะเสร็จ</div>

<h2>ส่งข้อมูลระหว่างเธรดด้วย channel</h2>
<pre><code>use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    for id in 1..=3 {
        let tx = tx.clone();
        thread::spawn(move || {          // move = ย้ายตัวแปรที่ใช้เข้าไปในเธรด
            tx.send(format!("งาน {id} เสร็จ")).unwrap();
        });
    }
    drop(tx);                            // ปิดฝั่งส่งของ main
    for msg in rx {                      // รับจนกว่าทุกฝั่งส่งจะปิด
        println!("{msg}");
    }
}</code></pre>

<h2>ใช้ข้อมูลร่วมกันด้วย Arc และ Mutex</h2>
<pre><code>use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));   // Arc = แชร์ได้หลายเธรด, Mutex = แก้ได้ทีละคน
    let mut handles = vec![];
    for _ in 0..10 {
        let c = Arc::clone(&amp;counter);
        handles.push(thread::spawn(move || {
            *c.lock().unwrap() += 1;
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("รวม {}", *counter.lock().unwrap());   // 10 เสมอ
}</code></pre>
<p>ถ้าลืมใส่ <code>Mutex</code> แล้วพยายามแก้ค่าจากหลายเธรด Rust จะไม่ยอม build นี่คือสิ่งที่ภาษาอื่นทำไม่ได้: บั๊กแบบ data race ถูกจับตั้งแต่ตอนคอมไพล์ ใน Phai Studio หน้าต่าง Threads (<kbd>Ctrl+Alt+H</kbd>) ช่วยดูว่าแต่ละเธรดกำลังทำอะไรตอนดีบัก</p>

<h2>ก้าวต่อไป</h2>
<h3>ฝึกด้วยโปรเจกต์จริง</h3>
<ul>
  <li><b>โปรแกรมจดรายรับรายจ่าย</b> ฟอร์มกรอกข้อมูล + ตาราง (Grid) + บันทึกลงไฟล์หรือ SQLite</li>
  <li><b>โปรแกรมจับเวลา</b> ใช้ Timer บนฟอร์ม</li>
  <li><b>ตัวเปลี่ยนชื่อไฟล์ทีละมาก</b> อ่านโฟลเดอร์ด้วย <code>std::fs::read_dir</code></li>
  <li><b>เกมทายตัวเลข</b> ใช้ crate <code>rand</code></li>
</ul>
<h3>หัวข้อที่ควรเรียนต่อ</h3>
<ul>
  <li><code>serde</code> และ <code>serde_json</code> อ่านเขียน JSON</li>
  <li>smart pointer: <code>Box</code>, <code>Rc</code>, <code>RefCell</code></li>
  <li>iterator ขั้นสูง และการเขียน iterator เอง</li>
  <li><code>async</code> / <code>await</code> สำหรับงานเครือข่าย</li>
</ul>
<h3>แหล่งเรียนรู้</h3>
<ul>
  <li><a href="https://doc.rust-lang.org/book/">The Rust Programming Language</a> หนังสือทางการ อ่านต่อจากคู่มือนี้ได้เลย</li>
  <li><a href="https://doc.rust-lang.org/rust-by-example/">Rust by Example</a></li>
  <li><a href="https://github.com/rust-lang/rustlings">Rustlings</a> แบบฝึกหัดเล็ก ๆ ที่ให้แก้โค้ดให้คอมไพล์ผ่าน</li>
  <li><a href="https://doc.rust-lang.org/std/">เอกสารไลบรารีมาตรฐาน</a></li>
</ul>
<p>ขอให้สนุกกับการเขียน Rust ครับ</p>
'''

BODY['publish.html'] = r'''
<h1>ทำตัวติดตั้ง (Publish)</h1>
<p class="lead">เขียนโปรแกรมเสร็จแล้วอยากให้เพื่อน ครู หรือลูกค้าใช้ ใช้คำสั่ง <b>Publish</b> ได้เลย Phai Studio จะ build โปรแกรมแบบ Release แล้วทำ <code>setup.exe</code> ไฟล์เดียวให้ คนที่ได้ไปแค่ดับเบิลคลิกก็ติดตั้งได้ มีทางลัดใน Start menu และถอนการติดตั้งได้จาก Settings เหมือนโปรแกรมทั่วไป</p>

<h2>ก่อนเริ่ม</h2>
<ul>
  <li>โปรแกรมต้อง build ผ่านก่อน ลองกด <kbd>F5</kbd> ให้รันได้ไม่มี error</li>
  <li>อยากให้โปรแกรมมีไอคอนของตัวเอง: เมนู <b>Project &gt; Properties</b> ช่อง "ไอคอนของไฟล์โปรแกรม" ใส่ไฟล์ <code>.ico</code> ที่อยู่ในโปรเจกต์ ไอคอนนี้จะขึ้นที่ไฟล์โปรแกรม ทางลัด และตัวติดตั้งด้วย</li>
</ul>

<h2>ขั้นตอน</h2>
<ol>
  <li>เปิดฟอร์มของโปรเจกต์ แล้วเลือกเมนู <b>Build &gt; Publish <i>ชื่อโปรเจกต์</i>...</b> (หน้าจอภาษาไทย: <b>สร้างโปรแกรม &gt; เผยแพร่...</b>)</li>
  <li>กรอกหน้าต่างที่ขึ้นมา (ดูตารางข้างล่าง) แล้วกด <b>OK</b></li>
  <li>Phai Studio build แบบ Release ซึ่งช้ากว่าปกติ ครั้งแรกอาจใช้เวลาหลายนาที ดูความคืบหน้าได้ที่แท็บ Output</li>
  <li>เสร็จแล้วแท็บ Output บอกชื่อไฟล์และขนาด และเปิดโฟลเดอร์ <code>publish</code> ของโปรเจกต์ให้</li>
</ol>
<figure><img class="shot" src="img/publish-dialog.png" alt="หน้าต่าง Publish" width="640"><figcaption>หน้าต่าง Publish ของโปรเจกต์ตัวอย่าง score_book</figcaption></figure>
<table>
  <tr><th>ช่อง</th><th>ใส่อะไร</th></tr>
  <tr><td>Product name</td><td>ชื่อโปรแกรมที่ผู้ใช้เห็น ใช้เป็นชื่อใน Start menu ใน Settings &gt; Apps ชื่อโฟลเดอร์ที่ติดตั้ง และชื่อไฟล์ตัวติดตั้ง</td></tr>
  <tr><td>Version</td><td>รุ่นของโปรแกรม เริ่มจาก <code>version</code> ใน <code>Cargo.toml</code> แจกใหม่ทุกครั้งควรเพิ่มเลข เช่น 0.1.0 → 0.1.1 ให้รู้ว่าเป็นรุ่นไหน</td></tr>
  <tr><td>Publisher</td><td>ชื่อผู้ทำ เช่น ชื่อตัวเอง โรงเรียน หรือบริษัท ขึ้นในหน้าติดตั้งและใน Settings &gt; Apps</td></tr>
  <tr><td>Offer a desktop shortcut</td><td>ติ๊ก = หน้าติดตั้งเสนอทางลัดบนเดสก์ท็อปไว้ให้ (ผู้ใช้เอาออกได้ตอนติดตั้ง)</td></tr>
  <tr><td>Also include</td><td>ไฟล์หรือโฟลเดอร์ในโปรเจกต์ที่ต้องไปกับโปรแกรม คั่นด้วยจุลภาค เช่น <code>pictures, data/config.txt</code> ค่าเริ่มต้นคือรูปที่ฟอร์มใช้</td></tr>
  <tr><td>Install for everyone</td><td>ไม่ติ๊ก (ค่าเริ่มต้น) = ติดตั้งให้ผู้ใช้คนนั้นคนเดียว ไม่ต้องใช้สิทธิ์ผู้ดูแลเครื่อง ติ๊ก = ติดตั้งลง Program Files ให้ทุกคนในเครื่อง ตอนติดตั้งจะขอสิทธิ์ผู้ดูแล</td></tr>
  <tr><td>Files it opens on a double-click</td><td>ไฟล์ที่โปรแกรมเปิดได้ เขียนนามสกุลตามด้วยชื่อที่ให้ Explorer แสดง คั่นด้วยจุลภาค เช่น <code>.score Score sheet, .exam</code> (ไม่ใส่ชื่อ = "ไฟล์ <i>ชื่อโปรแกรม</i>") เว้นว่างได้ ดูหัวข้อ <a href="#filetypes">ไฟล์ของโปรแกรมเอง</a></td></tr>
</table>
<p>คำตอบทั้งหมดถูกจำไว้ในไฟล์ <code>phai-publish.txt</code> ข้างโปรเจกต์ ครั้งหน้าเปิด Publish แล้วกด OK ได้เลย</p>

<h2>ได้อะไรมา</h2>
<p>ในโฟลเดอร์ <code>publish</code> ของโปรเจกต์มีสองอย่าง:</p>
<table>
  <tr><th>ไฟล์</th><th>ใช้อย่างไร</th></tr>
  <tr><td><code>Score-Book-0.1.0-setup.exe</code></td><td>ตัวติดตั้งไฟล์เดียว ส่งให้คนอื่นทางไลน์ อีเมล แฟลชไดรฟ์ หรือวางบนเว็บ</td></tr>
  <tr><td><code>Score-Book-0.1.0-portable\</code></td><td>โฟลเดอร์โปรแกรมพร้อมไฟล์ที่ใส่เพิ่ม คัดลอกไปที่ไหนก็เปิดได้เลยโดยไม่ต้องติดตั้ง</td></tr>
</table>

<h2>ตัวติดตั้งทำอะไรบนเครื่องผู้ใช้</h2>
<figure><img class="shot" src="img/publish-setup.png" alt="หน้าติดตั้ง Score Book" width="520"><figcaption>หน้าแรกของตัวติดตั้งที่ Publish ทำให้</figcaption></figure>
<ul>
  <li>ถามที่ติดตั้ง ค่าเริ่มต้นคือ <code>%LOCALAPPDATA%\Programs\ชื่อโปรแกรม</code> (ถ้าเลือก Install for everyone จะเป็น <code>Program Files</code>)</li>
  <li>ทำทางลัดใน Start menu และบนเดสก์ท็อป (ผู้ใช้ติ๊กออกได้)</li>
  <li>ลงชื่อโปรแกรมใน <b>Settings &gt; Apps</b> พร้อมตัวถอนการติดตั้ง (<code>uninstall.exe</code> ในโฟลเดอร์ที่ติดตั้ง) ถอนแล้วไฟล์ ทางลัด และรายชื่อใน Apps หายไปหมด</li>
</ul>
<p>ติดตั้งแบบไม่ถามอะไรเลย (เช่น ลงให้หลายเครื่องด้วยสคริปต์) ใช้สวิตช์เหล่านี้ได้:</p>
<table>
  <tr><th>สวิตช์</th><th>ทำอะไร</th></tr>
  <tr><td><code>/S</code></td><td>ติดตั้งเงียบ ๆ ไม่ขึ้นหน้าต่าง</td></tr>
  <tr><td><code>/D=โฟลเดอร์</code></td><td>ติดตั้งลงโฟลเดอร์นี้</td></tr>
  <tr><td><code>/NOSHORTCUTS</code></td><td>ไม่ทำทางลัด</td></tr>
</table>
<pre><code>Score-Book-0.1.0-setup.exe /S /D=D:\Apps\Score Book</code></pre>

<h2 id="filetypes">ไฟล์ของโปรแกรมเอง (ดับเบิลคลิกแล้วเปิด)</h2>
<p>โปรแกรมที่บันทึกงานเป็นไฟล์ของตัวเอง เช่น ใบคะแนน <code>.score</code> ใส่นามสกุลในช่อง <b>Files it opens on a double-click</b> แล้วตัวติดตั้งจะทำให้:</p>
<ul>
  <li>ไฟล์นามสกุลนั้นมีไอคอนของตัวเองใน Explorer: ถ้าในโปรเจกต์มีไฟล์ <code>score.ico</code> (ชื่อเดียวกับนามสกุล) จะใช้รูปนั้นและใส่ไปกับโปรแกรมให้เอง ถ้าไม่มีจะใช้ไอคอนของโปรแกรม</li>
  <li>ดับเบิลคลิกไฟล์แล้วเปิดโปรแกรม โดยส่งที่อยู่ของไฟล์มาให้</li>
  <li>ถ้าเครื่องนั้นมีโปรแกรมอื่นเปิดนามสกุลนี้อยู่แล้ว ตัวติดตั้งจะไม่แย่ง แต่ใส่โปรแกรมเราไว้ในเมนู <b>Open with</b> ให้</li>
  <li>ถอนการติดตั้งแล้ว ทุกอย่างที่เพิ่มไว้หายไปด้วย ของโปรแกรมอื่นไม่ถูกแตะ</li>
</ul>
<p>นามสกุลที่ Windows ใช้เอง (<code>.exe</code> <code>.dll</code> <code>.bat</code> <code>.lnk</code> ...) ใส่ไม่ได้ Phai Studio จะบอกและให้แก้ใหม่</p>
<p>โปรแกรมรู้ว่าถูกเปิดด้วยไฟล์อะไรจาก <code>std::env::args()</code> ตัวที่สอง เช่น อ่านไฟล์ตอนฟอร์มขึ้น (event <b>Shown</b>):</p>
<pre><code>use phai_ui::form::{message_box, MessageBoxButtons, MessageBoxIcon};

fn main_window_shown(&amp;mut self, ui: &amp;MainWindowUi) {
    // ดับเบิลคลิก ใบคะแนน.score → args = ["score_book.exe", "D:\\งาน\\ใบคะแนน.score"]
    if let Some(path) = std::env::args().nth(1) {
        match std::fs::read_to_string(&amp;path) {
            Ok(text) =&gt; ui.notes.set_text(&amp;text),
            Err(e) =&gt; {
                message_box(ui.hwnd, &amp;format!("เปิด {path} ไม่ได้: {e}"), "Score Book", MessageBoxButtons::Ok, MessageBoxIcon::Error);
            }
        }
    }
}</code></pre>

<h2>ก่อนแจกจริง ควรรู้</h2>
<ul>
  <li><b>หน้าจอเตือนของ Windows:</b> ตัวติดตั้งยังไม่ได้เซ็นโค้ด (code signing) ครั้งแรกที่เปิด Windows อาจขึ้น "Windows protected your PC" บอกผู้ใช้ให้กด <b>More info</b> แล้ว <b>Run anyway</b></li>
  <li><b>Visual C++ runtime:</b> โปรแกรมที่ build ด้วย Rust บน Windows ใช้ไฟล์ <code>VCRUNTIME140.dll</code> ของ Microsoft ซึ่งเครื่องส่วนใหญ่มีอยู่แล้ว ถ้าเครื่องไหนเปิดโปรแกรมแล้วขึ้นว่าหา <code>VCRUNTIME140.dll</code> ไม่เจอ ให้ติดตั้ง <b>Microsoft Visual C++ Redistributable (x64)</b> จากเว็บของ Microsoft (ฟรี) ก่อน</li>
  <li><b>ลองบนเครื่องอื่นก่อน:</b> ติดตั้งบนเครื่องที่ไม่เคยลง Phai Studio หรือ Rust เพื่อดูว่าไม่ขาดไฟล์อะไร โดยเฉพาะไฟล์ที่ต้องใส่ใน Also include</li>
</ul>
<div class="note">อยากให้โปรแกรมเปิดในเบราว์เซอร์หรือบนมือถือแทน ดู <a href="web.html">เว็บและมือถือ (PWA)</a> ซึ่งมีคำสั่ง <b>Publish Web App</b> แยกต่างหาก</div>
'''
