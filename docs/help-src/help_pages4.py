BODY = {}

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
  <tr><td>การเชื่อมต่อ</td><td>New Query, Refresh, Add Connection..., Delete</td></tr>
  <tr><td>ฐานข้อมูล</td><td>New Query, New Table..., Refresh</td></tr>
  <tr><td>ตาราง</td><td>Show Table Data, Design Table, Script as SQL, New Query, Copy code for a program, Add to Form, Refresh</td></tr>
  <tr><td>view</td><td>Show Table Data, New Query, Refresh</td></tr>
</table>
<ul>
  <li><b>Show Table Data</b> แสดงข้อมูลในตาราง ที่แท็บ Results ด้านล่าง แก้ค่าในตารางผลลัพธ์ได้โดยตรง</li>
  <li><b>Design Table</b> เปิดตัวออกแบบตาราง (Table Designer) แบบ Visual Studio เป็นตารางของคอลัมน์ แก้ชื่อ ชนิดข้อมูล เว้นว่างได้ไหม คีย์หลัก ตัวเลขรันอัตโนมัติ และค่าเริ่มต้น ได้ด้วยการคลิก ด้านล่างมี index และ foreign key และมีคำสั่ง SQL ที่จะใช้แก้ตาราง เขียนให้ดูไปพร้อมกัน กด Update เพื่อแก้ตารางจริง</li>
  <li><b>New Table...</b> เปิดตัวออกแบบตารางเดียวกัน สำหรับสร้างตารางใหม่ในฐานข้อมูลนั้น</li>
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
<p>ในโค้ดของฟอร์มสั่งเองได้ด้วย <code>ui.reload_customers()</code> และ <code>ui.save_customers()</code> (ชื่อตามตาราง) หัวคอลัมน์แสดงชื่อคอลัมน์ในฐานข้อมูลตามจริง ถ้าแก้หัวคอลัมน์ในฟอร์มเป็นคำอื่น คำนั้นจะแปลไทยได้เหมือนข้อความอื่น</p>
<div class="note">การแก้และลบแถวต้องรู้ว่าแถวไหนคือแถวไหน ตารางจึงต้องมีคีย์หลัก (primary key) ตารางที่ไม่มีคีย์หลักจะเพิ่มแถวได้อย่างเดียว ถ้าเป็น SQLite โปรแกรมจะเปิดไฟล์ฐานข้อมูลตามพาธที่เลือกตอนเพิ่มการเชื่อมต่อ ถ้าจะแจกโปรแกรม ให้แก้พาธใน <code>data_sources.rs</code> ให้ชี้ไปที่ไฟล์บนเครื่องผู้ใช้</div>

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
