BODY = {}

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
