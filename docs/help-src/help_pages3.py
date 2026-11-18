BODY = {}

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
