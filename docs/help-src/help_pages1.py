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
<p>เปลี่ยนภาษาเมนูได้ที่ <b>View &gt; ไทย / English</b> และเปลี่ยนธีมสว่าง/มืดได้ที่ <b>View &gt; Light / Dark</b></p>

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
<div class="note">ทุกอย่างที่ออกแบบบนฟอร์มเก็บไว้ในไฟล์ <code>.phf</code> แล้ว Phai Studio แปลงเป็นโค้ด Rust ให้ตอน build โค้ดที่คุณเขียนเองอยู่ในไฟล์ <code>.rs</code> ของคุณ ไม่โดนเขียนทับ</div>

<h2>ออกแบบฟอร์ม</h2>
<ul>
  <li>ลากมุมของสิ่งที่เลือกเพื่อปรับขนาด ลากตรงกลางเพื่อย้าย จะมีเส้นช่วยจัดแนวให้</li>
  <li>เลือกหลายอันพร้อมกันได้ แล้วใช้แถบเครื่องมือจัดแนว (ชิดซ้าย จัดกลาง ระยะห่างเท่ากัน)</li>
  <li><b>Format</b> มี Bring to front, Send to back, Lock controls และ Tab order (ลำดับการกด Tab)</li>
  <li>กด <kbd>F4</kbd> เพื่อดูตัวอย่างหน้าต่างจริงโดยไม่ต้อง build</li>
  <li>มี Menu bar, Context menu, Timer, Tree view, Picture, ตาราง (Grid), Rich text และแผงจัดวางอัตโนมัติ (Flow, Table)</li>
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
</ul>

<h2>Build รัน และแจกจ่าย</h2>
<table>
  <tr><th>คำสั่ง</th><th>ปุ่มลัด</th><th>ทำอะไร</th></tr>
  <tr><td>Start Debugging</td><td><kbd>F5</kbd></td><td>build แล้วรันพร้อมดีบักเกอร์ (หยุดที่ breakpoint ได้)</td></tr>
  <tr><td>Start Without Debugging</td><td><kbd>Ctrl+F5</kbd></td><td>build แล้วรันเฉย ๆ</td></tr>
  <tr><td>Build</td><td><kbd>Ctrl+B</kbd></td><td>build อย่างเดียว ข้อผิดพลาดขึ้นในแท็บ Errors</td></tr>
  <tr><td>Debug / Release</td><td>เมนู Build</td><td>Release ช้ากว่าตอน build แต่โปรแกรมเร็วกว่า ใช้ตอนแจกจริง</td></tr>
  <tr><td>Publish</td><td>เมนู Build</td><td>สร้างตัวติดตั้ง <code>setup.exe</code> ของโปรแกรมคุณ พร้อมเพิ่มไฟล์อื่นเข้าไปได้</td></tr>
  <tr><td>Manage Crates</td><td>เมนู Project</td><td>ค้นหาและเพิ่มไลบรารีจาก crates.io</td></tr>
</table>
<p>ถ้า build ไม่ผ่าน ดับเบิลคลิกบรรทัดในแท็บ Errors เพื่อไปยังจุดที่ผิด หรือกด <kbd>F8</kbd> ไล่ไปทีละข้อ ไม่เข้าใจข้อความ? ดู <a href="rust-10.html">บทที่ 10</a></p>

<h2>ดีบัก: หาว่าโปรแกรมผิดตรงไหน</h2>
<ol>
  <li>คลิกที่ขอบซ้ายของบรรทัด (หรือกด <kbd>F9</kbd>) จะได้จุดแดง เรียกว่า breakpoint</li>
  <li>กด <kbd>F5</kbd> โปรแกรมจะหยุดเมื่อมาถึงบรรทัดนั้น มีลูกศรเหลืองบอกตำแหน่ง</li>
  <li>แท็บ <b>Locals</b> แสดงค่าตัวแปรทุกตัว ณ ตอนนั้น ดับเบิลคลิกเพื่อแก้ค่าได้</li>
  <li>เดินทีละบรรทัดด้วย <kbd>F10</kbd> (ข้ามฟังก์ชัน) หรือ <kbd>F11</kbd> (เข้าไปในฟังก์ชัน), <kbd>Shift+F11</kbd> ออกจากฟังก์ชัน</li>
  <li>กด <kbd>F5</kbd> อีกครั้งเพื่อไปต่อ หรือ <kbd>Shift+F5</kbd> เพื่อหยุด</li>
</ol>
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
</table>

<h2>เทสต์</h2>
<ul>
  <li><kbd>Ctrl+R, T</kbd> รันเทสต์ของโปรเจกต์, <kbd>Ctrl+R, A</kbd> รันทั้งหมด ผลขึ้นในแท็บ Tests</li>
  <li><b>Test &gt; Analyze Code Coverage</b> บอกว่าเทสต์รันถึงบรรทัดไหนบ้าง แถบเขียวที่ขอบซ้ายคือรันถึง แถบแดงคือยังไม่มีเทสต์</li>
  <li>วิธีเขียนเทสต์ดูได้ใน <a href="rust-09.html">บทที่ 9</a></li>
</ul>

<h2>Git</h2>
<ul>
  <li><b>Git Changes</b> (<kbd>Ctrl+Alt+G</kbd>) ดูไฟล์ที่แก้ เลือก Stage แล้ว Commit พร้อม push/pull</li>
  <li><b>View History of This File</b> ดูทุกครั้งที่ไฟล์นี้ถูกแก้ และแก้อะไรไป</li>
  <li><b>Show Line Blame</b> บอกว่าใครแก้บรรทัดที่เคอร์เซอร์อยู่ เมื่อไร</li>
  <li><b>Resolve Merge Conflicts</b> เมื่อ merge แล้วชนกัน เลือกว่าจะใช้ของฝั่งไหน</li>
</ul>

<h2>ฐานข้อมูล</h2>
<p><b>Data &gt; Add Connection...</b> ต่อ MySQL/MariaDB, PostgreSQL หรือไฟล์ SQLite ได้ ดูตารางใน Server Explorer (<kbd>Ctrl+Alt+S</kbd>) เขียน SQL แล้วกด <kbd>Ctrl+Shift+E</kbd> เพื่อรัน แก้ข้อมูลในตารางผลลัพธ์ได้โดยตรง</p>
<p>อ่านละเอียดที่ <a href="database.html">ทำงานกับฐานข้อมูล</a></p>

<h2>ปุ่มลัดที่ใช้บ่อย</h2>
<table>
  <tr><th>ปุ่ม</th><th>ทำอะไร</th><th>ปุ่ม</th><th>ทำอะไร</th></tr>
  <tr><td><kbd>Ctrl+S</kbd></td><td>บันทึก</td><td><kbd>Ctrl+F</kbd> / <kbd>Ctrl+H</kbd></td><td>ค้นหา / แทนที่</td></tr>
  <tr><td><kbd>Ctrl+Shift+F</kbd></td><td>ค้นหาในทุกไฟล์</td><td><kbd>Ctrl+T</kbd></td><td>ไปที่ไฟล์ ฟังก์ชัน หรือชนิดใดก็ได้</td></tr>
  <tr><td><kbd>Ctrl+/</kbd></td><td>เปิด/ปิด comment</td><td><kbd>Ctrl+D</kbd></td><td>ทำสำเนา</td></tr>
  <tr><td><kbd>Alt+↑</kbd> <kbd>Alt+↓</kbd></td><td>ย้ายบรรทัดขึ้นลง</td><td><kbd>Ctrl+M, Ctrl+M</kbd></td><td>พับ/กางโค้ด</td></tr>
  <tr><td><kbd>Ctrl+K, Ctrl+K</kbd></td><td>คั่นหน้า (bookmark)</td><td><kbd>Ctrl+Alt+T</kbd></td><td>โครงร่างของไฟล์ (Document Outline)</td></tr>
  <tr><td><kbd>Alt+F7</kbd></td><td>ตั้งค่าโปรเจกต์</td><td><kbd>Ctrl+Shift+B</kbd></td><td>Build ทั้ง solution</td></tr>
</table>
'''
