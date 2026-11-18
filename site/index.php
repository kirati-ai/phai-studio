<?php
// phai-studio.monrak.net - Phai Studio's own page, in the look of phai-erp.monrak.net.
// Thai or English (?lang=, then a cookie, then the browser's language). The installer is
// downloaded from GitHub Releases through /download/setup, which counts it; /seen counts visitors.

declare(strict_types=1);

$release = json_decode((string) @file_get_contents(__DIR__ . '/release.json'), true) ?: [
    'version' => '0.1.0', 'file' => 'PhaiStudio-0.1.0-setup.exe', 'size' => 3955989,
    'sha256' => '', 'date' => '2026-11-18',
];
$github = 'https://github.com/kirati-ai/phai-studio';
$setupUrl = $github . '/releases/download/v' . $release['version'] . '/' . $release['file'];

$storage = __DIR__ . '/storage';
$countsFile = $storage . '/counts.json';

/** The counts, changed by $change under a lock when given. */
function counts(string $file, ?callable $change = null): array
{
    $dir = dirname($file);
    if (!is_dir($dir)) {
        @mkdir($dir, 0775, true);
    }
    $h = @fopen($file, 'c+');
    if (!$h) {
        return ['downloads' => 0, 'visitors' => 0];
    }
    flock($h, $change ? LOCK_EX : LOCK_SH);
    $c = json_decode((string) stream_get_contents($h), true) ?: [];
    $c += ['downloads' => 0, 'visitors' => 0];
    if ($change) {
        $c = $change($c);
        ftruncate($h, 0);
        rewind($h);
        fwrite($h, json_encode($c));
        fflush($h);
    }
    flock($h, LOCK_UN);
    fclose($h);
    return $c;
}

// Where the site sits: / on phai-studio.monrak.net, /phai-studio/ under a local web server.
$B = rtrim(str_replace(chr(92), '/', dirname($_SERVER['SCRIPT_NAME'] ?? '/index.php')), '/') . '/';
$path = '/' . ltrim(substr(parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH) ?: '/', strlen($B)), '/');
$path = rtrim($path, '/');

if ($path === '/download/setup') {
    counts($countsFile, fn ($c) => ['downloads' => $c['downloads'] + 1] + $c);
    header('Location: ' . $setupUrl, true, 302);
    exit;
}
if ($path === '/seen') {
    if (empty($_COOKIE['seen'])) {
        counts($countsFile, fn ($c) => ['visitors' => $c['visitors'] + 1] + $c);
        setcookie('seen', '1', ['expires' => time() + 86400 * 365, 'path' => $B, 'samesite' => 'Lax', 'secure' => !empty($_SERVER['HTTPS'])]);
    }
    http_response_code(204);
    exit;
}

$lang = $_GET['lang'] ?? $_COOKIE['lang'] ?? null;
if (isset($_GET['lang']) && in_array($_GET['lang'], ['th', 'en'], true)) {
    setcookie('lang', $_GET['lang'], ['expires' => time() + 86400 * 365, 'path' => $B, 'samesite' => 'Lax']);
}
if (!in_array($lang, ['th', 'en'], true)) {
    $lang = str_starts_with(strtolower($_SERVER['HTTP_ACCEPT_LANGUAGE'] ?? 'th'), 'en') ? 'en' : 'th';
}
$th = $lang === 'th';
$c = counts($countsFile);

function e(string $s): string
{
    return htmlspecialchars($s, ENT_QUOTES, 'UTF-8');
}
function v(string $f): string
{
    global $B;
    return $B . 'assets/' . $f . '?v=' . (@filemtime(__DIR__ . '/assets/' . $f) ?: 1);
}
$mb = number_format($release['size'] / 1048576, 1);
$date = date($th ? 'd/m/Y' : 'd M Y', strtotime($release['date']));

$T = $th ? [
    'title' => 'Phai Studio — เขียนโปรแกรม Windows ด้วย Rust แบบ Visual Studio ฟรี โอเพนซอร์ส พร้อมคู่มือภาษาไทย',
    'desc' => 'Phai Studio เครื่องมือเขียนโปรแกรม (IDE) ฟรีแบบโอเพนซอร์ส (MIT) สำหรับสร้างโปรแกรม Windows ด้วยภาษา Rust ลากวางออกแบบหน้าจอ เขียนโค้ด ดีบัก ทดสอบ และทำตัวติดตั้งได้ในที่เดียว หน้าจอภาษาไทย พร้อมคู่มือสอน Rust ภาษาไทยสำหรับมือใหม่',
    'nav' => ['features' => 'ความสามารถ', 'learn' => 'คู่มือภาษาไทย', 'install' => 'วิธีติดตั้ง', 'download' => 'ดาวน์โหลด'],
    'tag' => 'วิทยาทาน · โอเพนซอร์ส · ฟรี',
    'h1' => 'เขียนโปรแกรม Windows ด้วย Rust <em>ง่ายแบบ Visual Studio</em>',
    'lead' => 'ลากวางปุ่ม ช่องกรอก ตาราง ลงบนฟอร์ม ดับเบิลคลิกเขียนโค้ด กด F5 ก็ได้โปรแกรม Windows จริง พร้อมตัวแก้โค้ดที่เติมคำให้ ดีบักเกอร์ เทสต์ Git และทำตัวติดตั้งให้ด้วย หน้าจอภาษาไทย และมีคู่มือสอน Rust ภาษาไทยในตัว',
    'btnDownload' => 'ดาวน์โหลดตัวติดตั้ง', 'btnGithub' => 'โค้ดบน GitHub', 'btnGuide' => 'อ่านคู่มือภาษาไทย',
    'meta' => fn ($c) => "รุ่น {$release['version']} · {$mb} MB · Windows 10/11 (64 บิต) · <b>ดาวน์โหลดแล้ว {$c['downloads']} ครั้ง</b> · เข้าชม {$c['visitors']} คน",
    'learnH' => 'หัดเขียน Rust ด้วยภาษาไทย',
    'learnSub' => 'Rust เป็นภาษาที่เร็วและปลอดภัย แต่ตำราส่วนใหญ่เป็นภาษาอังกฤษ Phai Studio จึงมีคู่มือภาษาไทยในตัว (กด F1) เริ่มจากศูนย์ทีละบท ทุกตัวอย่างลองรันได้จริง เพื่อให้นักเรียน นักศึกษา และคนทำงานที่อยากเขียนโปรแกรมเอง เริ่มได้โดยไม่ติดเรื่องภาษา',
    'learn' => [
        ['ti-player-play', 'บทที่ 1–3', 'เริ่มต้น ตัวแปรและชนิดข้อมูล if / loop / match'],
        ['ti-key', 'บทที่ 4', 'ความเป็นเจ้าของ (ownership) และการยืม หัวใจของ Rust อธิบายด้วยภาพ'],
        ['ti-box', 'บทที่ 5–7', 'struct, enum, การจัดการข้อผิดพลาด, Vec และ HashMap'],
        ['ti-puzzle', 'บทที่ 8–9', 'trait, generic, โมดูล และการเขียนเทสต์'],
        ['ti-alert-triangle', 'บทที่ 10', 'อ่านข้อความ error ของคอมไพเลอร์ให้เป็น พร้อมวิธีแก้ที่เจอบ่อย'],
        ['ti-arrows-split', 'บทที่ 11', 'ทำงานหลายอย่างพร้อมกันด้วย thread อย่างปลอดภัย'],
    ],
    'learnBtn' => 'เปิดคู่มือ', 'learnShot' => 'โปรแกรมหน้าจอภาษาไทยที่สร้างด้วย Phai Studio',
    'featH' => 'ครบตั้งแต่ออกแบบหน้าจอ จนถึงตัวติดตั้ง',
    'featSub' => 'ทุกอย่างที่ต้องใช้ทำโปรแกรม Windows หนึ่งตัว อยู่ในโปรแกรมเดียว',
    'feat' => [
        ['ti-layout-dashboard', 'ออกแบบฟอร์ม', 'ลากวางปุ่ม ช่องกรอก ตาราง เมนู แถบเครื่องมือ จัดแนว จัดระยะ แก้คุณสมบัติในหน้าต่าง Properties ตกแต่งด้วย CSS'],
        ['ti-code', 'ตัวแก้โค้ด', 'สีตามไวยากรณ์ เติมคำอัตโนมัติ ไปยังที่ประกาศ หาที่ใช้ เปลี่ยนชื่อ จัดรูปแบบ พับโค้ด ด้วย rust-analyzer'],
        ['ti-hammer', 'Build และ Publish', 'build แบบ Debug/Release เห็น error เป็นรายการ คลิกไปที่บรรทัดได้ Publish ออกมาเป็นตัวติดตั้งไฟล์เดียว'],
        ['ti-bug', 'ดีบักเกอร์ของตัวเอง', 'breakpoint ทีละบรรทัด ดูตัวแปร call stack thread หน่วยความจำ disassembly และ data breakpoint'],
        ['ti-flask', 'เทสต์และ Code Coverage', 'รันเทสต์ทั้งหมดหรือทีละตัว เห็นผลผ่าน/ไม่ผ่าน และแถบสีบอกบรรทัดที่เทสต์ครอบคลุม'],
        ['ti-git-branch', 'Git', 'commit, push, pull, branch, ประวัติไฟล์, blame และหน้าต่างแก้ conflict'],
        ['ti-database', 'ฐานข้อมูล', 'Server Explorer ต่อ PostgreSQL, MySQL/MariaDB, SQLite ดูตาราง รันคำสั่ง SQL'],
        ['ti-package', 'จัดการ crate', 'ค้นหาไลบรารีจาก crates.io แล้วกดเพิ่มหรือลบได้ เหมือน NuGet ของ Visual Studio'],
        ['ti-language', 'ไทย / อังกฤษ', 'สลับภาษาหน้าจอได้ทันที โปรแกรมที่สร้างก็ทำสองภาษาได้ และแสดงตัวอักษรไทยถูกต้อง'],
    ],
    'shotsH' => 'หน้าจอจริง', 'shotsSub' => 'ภาพจากการใช้งานจริง หน้าตาแบบ Windows มาตรฐาน คนที่เคยใช้ Visual Studio คุ้นมือทันที',
    'shots' => [
        ['designer', 'ออกแบบฟอร์ม', 'ลากวางปุ่ม ช่องกรอก และตาราง แล้วแก้คุณสมบัติในหน้าต่าง Properties'],
        ['editor', 'ตัวแก้โค้ด', 'สีตามไวยากรณ์ จำนวนที่ใช้งาน และเติมคำอัตโนมัติด้วย rust-analyzer'],
        ['debugger', 'ดีบักทีละบรรทัด', 'หยุดที่ breakpoint และดู thread ทั้งหมดของโปรแกรม'],
        ['breakpoints', 'Breakpoints', 'breakpoint แบบมีเงื่อนไข และนับจำนวนครั้ง เปิดปิดได้จากรายการ'],
        ['memory', 'หน่วยความจำ', 'ดูค่าในหน่วยความจำแบบ hex ระหว่างหยุดโปรแกรม'],
        ['disassembly', 'Disassembly', 'ดูภาษาเครื่อง คู่กับบรรทัดในซอร์สโค้ด'],
        ['tests', 'เทสต์', 'รันเทสต์ทั้งหมดหรือทีละตัว เห็นผลผ่านและไม่ผ่าน'],
        ['coverage', 'Code Coverage', 'แถบสีบอกบรรทัดที่เทสต์ครอบคลุม และยังไม่ครอบคลุม'],
        ['program', 'โปรแกรมที่สร้าง', 'โปรแกรม Windows หน้าจอภาษาไทย ที่สร้างด้วย Phai Studio'],
    ],
    'statVisitors' => 'ผู้เข้าชมทั้งหมด', 'statDownloads' => 'ดาวน์โหลดแล้ว', 'statVersion' => 'เวอร์ชัน', 'cta' => 'ดาวน์โหลด',
    'zoom' => 'คลิกเพื่อดูภาพเต็ม', 'prev' => 'ก่อนหน้า', 'next' => 'ถัดไป', 'close' => 'ปิด',
    'hud' => ['PHAI STUDIO · RUST · WINDOWS', 'วิทยาทาน · โอเพนซอร์ส', 'คู่มือ RUST ภาษาไทย', 'ดาวน์โหลดฟรี'],
    'whyH' => 'ทำไมต้อง Phai Studio',
    'why' => [
        ['ฟรีทุกฟีเจอร์ ไม่มีรุ่นเสียเงิน', 'สัญญาอนุญาต MIT ใช้ แก้ และแจกต่อได้ รวมถึงใช้ทำโปรแกรมขาย'],
        ['โปรแกรมที่ได้เล็กและเร็ว', 'เป็นโปรแกรม Windows แท้ ไฟล์เดียว ไม่ต้องลง .NET, Java หรือ runtime อื่นที่เครื่องลูกค้า'],
        ['ภาษา Rust ปลอดภัยตั้งแต่ตอนเขียน', 'คอมไพเลอร์จับข้อผิดพลาดเรื่องหน่วยความจำและ thread ให้ก่อนโปรแกรมจะได้รัน'],
        ['หน้าตาแบบ Visual Studio', 'เมนู แถบเครื่องมือ Toolbox, Properties, Solution ที่เดิม ย้ายมาจาก C# / VB ได้ง่าย'],
        ['เหมาะกับมือใหม่', 'หน้าจอภาษาไทย คู่มือภาษาไทย มีบทสอนอ่านข้อความ error และโปรเจกต์ตัวอย่างให้ลอง'],
        ['ไม่เก็บข้อมูลผู้ใช้', 'ไม่มี telemetry ไม่ต้องสมัครสมาชิก ส่งข้อมูลออกเฉพาะเมื่อสั่งเอง เช่น ดาวน์โหลด crate หรือ push Git'],
    ],
    'insH' => 'ติดตั้งใน 3 ขั้น', 'insSub' => 'ใช้กับ Windows 10 หรือ 11 แบบ 64 บิต',
    'ins' => [
        '<b>ดาวน์โหลดตัวติดตั้ง</b> แล้วดับเบิลคลิก เลือกที่ติดตั้งและทางลัดบนเดสก์ท็อป',
        '<b>ติดตั้ง Rust</b> (ฟรี) ถ้ายังไม่มี ตอน build ครั้งแรก Phai Studio จะพาไปที่ <a href="https://rustup.rs" rel="noopener">rustup.rs</a> ให้ดาวน์โหลด <code>rustup-init.exe</code> แล้วติดตั้งตามค่าเริ่มต้น ตัวติดตั้งจะพาลง Microsoft C++ Build Tools (ฟรี) ให้ด้วย',
        '<b>เปิด Phai Studio</b> เลือก "สร้างโปรเจกต์ใหม่" แล้วกด F5 กด F1 เมื่อไรก็ได้เพื่อเปิดคู่มือ',
    ],
    'smart' => 'ถ้า Windows ขึ้นหน้าจอ "Windows protected your PC" ให้กด <b>More info</b> แล้วกด <b>Run anyway</b> เพราะตัวติดตั้งยังไม่ได้เซ็นโค้ด (กำลังขอใบรับรองฟรีจาก SignPath Foundation)',
    'uninst' => 'ถอนการติดตั้งได้ที่ Settings > Apps เหมือนโปรแกรมทั่วไป',
    'supH' => 'เรียนไปด้วยกัน', 'supSub' => 'ฟรี ไม่มีค่าสมาชิก ไม่ต้องลงทะเบียน',
    'selfH' => 'เริ่มเรียนเอง', 'selfP' => 'ดาวน์โหลด เปิดคู่มือภาษาไทย แล้วทำตามทีละบท ค่อย ๆ ไปตามจังหวะของตัวเอง',
    'proH' => 'ช่วยกันทำให้ดีขึ้น', 'proP' => 'Phai Studio เพิ่งเริ่มต้น ผู้สร้างเองก็กำลังหัดเขียน Rust ไปพร้อมกับทุกคน เจอปัญหา อยากได้อะไรเพิ่ม หรืออยากช่วย บอกกันได้เลย',
    'pro' => [
        'แจ้งปัญหาหรือขอความสามารถใหม่ที่ <a href="https://github.com/kirati-ai/phai-studio/issues" rel="noopener">GitHub Issues</a>',
        'แก้โค้ดหรือคู่มือแล้วส่ง pull request มาได้',
        'ชอบก็กดดาวบน GitHub และบอกต่อเพื่อนที่อยากหัดเขียนโปรแกรม',
        'อยากคุยเรื่องอื่น ๆ หรืออยากให้ช่วย ทักมาที่ <a href="https://www.monrak.net/">Monrak Net</a> ได้เลย',
    ],
    'contact' => 'คุยกับ Monrak Net',
    'dlH' => 'ดาวน์โหลด', 'dlName' => 'Phai Studio (ตัวติดตั้ง)',
    'dlDesc' => 'ไฟล์เดียว มีโปรแกรม ตัวถอนการติดตั้ง และคู่มือภาษาไทยในตัว',
    'dlMeta' => fn ($c) => "รุ่น {$release['version']} · วันที่ {$date} · ดาวน์โหลดแล้ว {$c['downloads']} ครั้ง",
    'dlBuilt' => 'ตัวติดตั้ง build จากซอร์สโค้ดโดย GitHub Actions ตรวจสอบได้ทุกขั้น ดูได้ที่ <a href="' . $github . '/releases" rel="noopener">หน้า Releases</a>',
    'dlSign' => 'การเซ็นโค้ด: Phai Studio ขอใช้บริการเซ็นโค้ดฟรีสำหรับโอเพนซอร์สจาก <a href="https://signpath.org" rel="noopener">SignPath Foundation</a> อยู่ ระหว่างนี้ไฟล์ยังไม่ได้เซ็น อ่าน <a href="' . $github . '/blob/main/docs/code-signing-policy.md" rel="noopener">นโยบายการเซ็นโค้ด</a>',
    'dlGit' => 'หรือดึงโค้ดล่าสุดจาก GitHub (แจก แก้ และส่ง pull request ได้):',
    'license' => 'สัญญาอนุญาต MIT — ใช้ แก้ และแจกต่อได้ฟรี รวมถึงใช้ในเชิงพาณิชย์',
    'quote' => 'เราได้เรียนรู้จากซอฟต์แวร์โอเพนซอร์ส ของผู้อื่นมามาก จึงขอมอบซอฟต์แวร์ตระกูลไผ่ เป็นวิทยาทานแก่ทุกคน ให้ใครก็ได้มาเรียนรู้ ใช้งาน และต่อยอดได้ฟรี',
    'family' => 'ตระกูลไผ่',
    'suite' => [
        ['https://phai-erp.monrak.net/', 'Phai ERP', 'ERP ภาษาไทย จัดซื้อ ขาย คลัง ผลิต บัญชี และภาษีไทย ฟรีและโอเพนซอร์ส'],
        ['https://phai-desk.monrak.net/', 'Phai Desk', 'รีโมทเข้าเครื่อง Windows ในสำนักงาน ดูสเปก โปรแกรมที่ลง และปัญหาของทุกเครื่องจากหน้าเดียว'],
        ['https://phai-suite.monrak.net/', 'Phai Suite', 'แผ่นติดตั้งเซิร์ฟเวอร์สำนักงานสำเร็จรูป ลงครั้งเดียวได้ Phai ERP และ Phai Desk พร้อมใช้'],
        ['https://desktopserver.monrak.net/', 'Monrak Desktop Server', 'Apache, PHP, MySQL บน Windows'],
        ['https://www.monrak.net/', 'Monrak Net', 'มนต์รักที่มีต่ออินเทอร์เน็ต ตั้งแต่ปี 2545 บ้านของซอฟต์แวร์ตระกูลไผ่'],
    ],
] : [
    'title' => 'Phai Studio — build Windows programs in Rust, Visual Studio style. Free and open source, with a Thai guide',
    'desc' => 'Phai Studio is a free, open-source (MIT) IDE for building Windows programs in Rust: drag-and-drop form designer, code editor, debugger, tests and installer in one place. English and Thai interface, with a Thai Rust guide for beginners.',
    'nav' => ['features' => 'Features', 'learn' => 'Thai guide', 'install' => 'Install', 'download' => 'Download'],
    'tag' => 'A gift of knowledge · Open source · Free',
    'h1' => 'Build Windows programs in Rust <em>the Visual Studio way</em>',
    'lead' => 'Drag buttons, text boxes and grids onto a form, double-click to write the code, press F5 and you have a real Windows program. A code editor that completes for you, a debugger, tests, Git and an installer maker come with it, in English or Thai, with a Thai Rust guide built in.',
    'btnDownload' => 'Download the installer', 'btnGithub' => 'Code on GitHub', 'btnGuide' => 'Read the guide (Thai)',
    'meta' => fn ($c) => "Version {$release['version']} · {$mb} MB · Windows 10/11 (64-bit) · <b>Downloaded {$c['downloads']} times</b> · {$c['visitors']} visitors",
    'learnH' => 'Learn Rust in Thai',
    'learnSub' => 'Rust is fast and safe, but most books are in English. Phai Studio carries a Thai guide (press F1) that starts from zero, one lesson at a time, with examples that really run, so students and working people who want to write their own programs can start without a language barrier.',
    'learn' => [
        ['ti-player-play', 'Lessons 1–3', 'Getting started, variables and types, if / loop / match'],
        ['ti-key', 'Lesson 4', 'Ownership and borrowing, the heart of Rust, explained with pictures'],
        ['ti-box', 'Lessons 5–7', 'struct, enum, handling errors, Vec and HashMap'],
        ['ti-puzzle', 'Lessons 8–9', 'traits, generics, modules and writing tests'],
        ['ti-alert-triangle', 'Lesson 10', 'Reading compiler errors, and the fixes for the common ones'],
        ['ti-arrows-split', 'Lesson 11', 'Doing several things at once, safely, with threads'],
    ],
    'learnBtn' => 'Open the guide', 'learnShot' => 'A program with a Thai interface, made with Phai Studio',
    'featH' => 'From the first form to the installer',
    'featSub' => 'Everything it takes to make a Windows program, in one place',
    'feat' => [
        ['ti-layout-dashboard', 'Form designer', 'Drag buttons, text boxes, grids, menus and toolbars; align and space them; set properties in the Properties window; style with CSS'],
        ['ti-code', 'Code editor', 'Syntax colours, completion, go to definition, find references, rename, formatting and folding, with rust-analyzer'],
        ['ti-hammer', 'Build and Publish', 'Debug and Release builds, errors in a list that takes you to the line, and Publish to a one-file installer'],
        ['ti-bug', 'Its own debugger', 'Breakpoints, stepping, locals, call stack, threads, memory, disassembly and data breakpoints'],
        ['ti-flask', 'Tests and code coverage', 'Run every test or one, see what passed and failed, and a coloured bar by every line the tests reach'],
        ['ti-git-branch', 'Git', 'Commit, push, pull, branches, file history, blame and a merge conflict editor'],
        ['ti-database', 'Databases', 'Server Explorer for PostgreSQL, MySQL/MariaDB and SQLite: browse tables, run SQL'],
        ['ti-package', 'Crates', 'Search crates.io and add or remove libraries with a click, like Visual Studio\'s NuGet'],
        ['ti-language', 'English / Thai', 'Switch the interface language at once; the programs you make can be bilingual too, with Thai shown right'],
    ],
    'shotsH' => 'Real screens', 'shotsSub' => 'Taken from real use: a standard Windows look that anyone who has used Visual Studio knows',
    'shots' => [
        ['designer', 'Form designer', 'Drag buttons, text boxes and grids, and set their properties in the Properties window'],
        ['editor', 'Code editor', 'Syntax colours, reference counts and completion, with rust-analyzer'],
        ['debugger', 'Step by step', 'Stopped at a breakpoint, with every thread of the program'],
        ['breakpoints', 'Breakpoints', 'Conditions and hit counts, ticked on and off from the list'],
        ['memory', 'Memory', 'The memory of the program in hex while it is paused'],
        ['disassembly', 'Disassembly', 'Machine code beside the source lines it comes from'],
        ['tests', 'Tests', 'Run every test or one, and see what passed and failed'],
        ['coverage', 'Code coverage', 'A coloured bar by every line the tests reach, and the ones they miss'],
        ['program', 'What you make', 'A Windows program with a Thai interface, made with Phai Studio'],
    ],
    'statVisitors' => 'Visitors', 'statDownloads' => 'Downloads', 'statVersion' => 'Version', 'cta' => 'Download',
    'zoom' => 'Click to see it full size', 'prev' => 'Previous', 'next' => 'Next', 'close' => 'Close',
    'hud' => ['PHAI STUDIO · RUST · WINDOWS', 'A GIFT OF KNOWLEDGE', 'LEARN RUST IN THAI', 'FREE DOWNLOAD'],
    'whyH' => 'Why Phai Studio',
    'why' => [
        ['Every feature free, no paid edition', 'MIT licence: use it, change it, pass it on, and sell the programs you make'],
        ['Small, fast programs', 'Real Windows programs in one file; no .NET, Java or other runtime on your customers\' computers'],
        ['Rust catches mistakes while you write', 'The compiler stops memory and threading mistakes before the program ever runs'],
        ['The Visual Studio layout', 'Menus, toolbars, Toolbox, Properties and Solution where you expect them; easy to come from C# or VB'],
        ['Made for beginners', 'Thai interface, a Thai guide with a lesson on reading errors, and a sample project to try'],
        ['Nothing collected about you', 'No telemetry, no sign-up; it goes online only when you ask, such as downloading crates or pushing to Git'],
    ],
    'insH' => 'Install in 3 steps', 'insSub' => 'For Windows 10 or 11, 64-bit',
    'ins' => [
        '<b>Download the installer</b> and double-click it; choose where to install and whether to add a desktop shortcut',
        '<b>Install Rust</b> (free) if you do not have it: on the first build Phai Studio takes you to <a href="https://rustup.rs" rel="noopener">rustup.rs</a>; download <code>rustup-init.exe</code> and accept its defaults. It installs the free Microsoft C++ Build Tools for you too',
        '<b>Open Phai Studio</b>, choose "Create a new project" and press F5. Press F1 at any time for the guide',
    ],
    'smart' => 'If Windows shows "Windows protected your PC", choose <b>More info</b> then <b>Run anyway</b>: the installer is not code-signed yet (a free certificate has been requested from SignPath Foundation)',
    'uninst' => 'Uninstall it from Settings > Apps like any other program',
    'supH' => 'Learning together', 'supSub' => 'Free: no subscription, no sign-up',
    'selfH' => 'Start on your own', 'selfP' => 'Download, open the guide and follow it lesson by lesson, at your own pace',
    'proH' => 'Help make it better', 'proP' => 'Phai Studio is just starting, and its maker is learning Rust alongside everyone. Found a problem, want something added, or want to help? Just say so',
    'pro' => [
        'Report problems or ask for features on <a href="https://github.com/kirati-ai/phai-studio/issues" rel="noopener">GitHub Issues</a>',
        'Fix the code or the guide and send a pull request',
        'If you like it, star it on GitHub and tell friends who want to learn to program',
        'Want to talk about anything else, or need a hand? Say hello to <a href="https://www.monrak.net/">Monrak Net</a>',
    ],
    'contact' => 'Talk to Monrak Net',
    'dlH' => 'Download', 'dlName' => 'Phai Studio (installer)',
    'dlDesc' => 'One file with the program, its uninstaller and the Thai guide',
    'dlMeta' => fn ($c) => "Version {$release['version']} · {$date} · Downloaded {$c['downloads']} times",
    'dlBuilt' => 'The installer is built from source by GitHub Actions, every step open to check, on the <a href="' . $github . '/releases" rel="noopener">Releases page</a>',
    'dlSign' => 'Code signing: Phai Studio has applied for free code signing for open source from <a href="https://signpath.org" rel="noopener">SignPath Foundation</a>; until then the files are not signed. See the <a href="' . $github . '/blob/main/docs/code-signing-policy.md" rel="noopener">code signing policy</a>',
    'dlGit' => 'Or get the latest code from GitHub (share it, change it, send pull requests):',
    'license' => 'MIT licence — free to use, change and pass on, commercial use included',
    'quote' => 'We have learned a great deal from other people\'s open-source software, so we give the Phai family of software to everyone as a gift of knowledge: anyone may learn from it, use it and build on it, free.',
    'family' => 'The Phai family',
    'suite' => [
        ['https://phai-erp.monrak.net/', 'Phai ERP', 'A Thai ERP: purchasing, sales, inventory, production, accounting and Thai tax, free and open source'],
        ['https://phai-desk.monrak.net/', 'Phai Desk', 'Remote into the office\'s Windows computers and see every machine\'s specs, programs and problems from one page'],
        ['https://phai-suite.monrak.net/', 'Phai Suite', 'A ready-made office server disc: install once and Phai ERP and Phai Desk are ready'],
        ['https://desktopserver.monrak.net/', 'Monrak Desktop Server', 'Apache, PHP and MySQL on Windows'],
        ['https://www.monrak.net/', 'Monrak Net', 'A love of the internet since 2002, and the home of the Phai family'],
    ],
];
$base = 'https://phai-studio.monrak.net/';
?><!doctype html>
<html lang="<?= $lang ?>">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title><?= e($T['title']) ?></title>
    <meta name="description" content="<?= e($T['desc']) ?>">
    <link rel="canonical" href="<?= $base ?>">
    <link rel="alternate" hreflang="th" href="<?= $base ?>?lang=th">
    <link rel="alternate" hreflang="en" href="<?= $base ?>?lang=en">
    <meta property="og:title" content="Phai Studio">
    <meta property="og:description" content="<?= e($T['desc']) ?>">
    <meta property="og:type" content="website">
    <meta property="og:image" content="<?= $base ?>assets/shots/designer.jpg">
    <link rel="icon" href="<?= v('logo.svg') ?>" type="image/svg+xml">
    <link rel="stylesheet" href="<?= v('tabler-icons/tabler-icons.min.css') ?>">
    <link rel="stylesheet" href="<?= v('site.css') ?>">
</head>
<body>
<div id="grid-bg" aria-hidden="true"></div>
<div id="hud" aria-hidden="true"><i></i><span id="hud-pct">0%</span><b>|</b><span id="hud-text"><?= e($T['hud'][0]) ?></span></div>

<header class="top">
    <div class="wrap top-row">
        <a class="brand" href="<?= $B ?>"><img src="<?= v('logo.svg') ?>" alt="" width="34" height="34"><span><small>MONRAK</small><b>Phai Studio</b></span></a>
        <nav class="nav">
            <a class="hide-sm" href="#screens"><?= e($T['shotsH']) ?></a>
            <a class="hide-sm" href="#learn"><?= e($T['nav']['learn']) ?></a>
            <a class="hide-sm" href="#features"><?= e($T['nav']['features']) ?></a>
            <a class="hide-sm" href="#install"><?= e($T['nav']['install']) ?></a>
            <a class="lang" href="?lang=<?= $th ? 'en' : 'th' ?>"><i class="ti ti-world"></i> <?= $th ? 'EN' : 'ไทย' ?></a>
            <a class="cta" href="#download"><?= e($T['cta']) ?> <i class="ti ti-arrow-right"></i></a>
        </nav>
    </div>
</header>

<main>
<section class="hero">
    <div class="wrap hero-row">
        <div>
            <span class="label"><i class="ti ti-bolt"></i> <?= e($T['tag']) ?></span>
            <h1><?= $T['h1'] ?></h1>
            <p class="lead"><?= e($T['lead']) ?></p>
            <div class="actions">
                <a class="btn btn-primary" href="<?= $B ?>download/setup"><i class="ti ti-download"></i> <?= e($T['btnDownload']) ?></a>
                <a class="btn btn-ghost" href="<?= $B ?>help/"><i class="ti ti-book"></i> <?= e($T['btnGuide']) ?></a>
                <a class="btn btn-ghost" href="<?= $github ?>" rel="noopener"><i class="ti ti-brand-github"></i> GitHub</a>
            </div>
            <div class="stats">
                <div><small><?= e($T['statVisitors']) ?></small><b><?= number_format($c['visitors']) ?></b></div>
                <div><small><?= e($T['statDownloads']) ?></small><b><?= number_format($c['downloads']) ?></b></div>
                <div><small><?= e($T['statVersion']) ?></small><b><?= e($release['version']) ?></b></div>
            </div>
        </div>
        <div class="hero-art" aria-hidden="true"><img src="<?= v('logo.svg') ?>" alt="" width="300" height="300"></div>
    </div>
</section>

<section id="screens" class="section tint">
    <div class="wrap">
        <span class="label">Real screens</span>
        <h2><?= e($T['shotsH']) ?></h2>
        <p class="sub"><?= e($T['shotsSub']) ?></p>
        <div class="gallery">
            <div class="slider">
                <div class="slides">
                    <?php foreach ($T['shots'] as $i => [$f, $h, $p]): ?>
                        <a class="slide" href="<?= v("shots/$f.jpg") ?>" data-gallery data-caption="<?= e("$h — $p") ?>"><img src="<?= v("shots/$f.jpg") ?>" alt="<?= e($h) ?>" width="1432" height="969" loading="<?= $i ? 'lazy' : 'eager' ?>"></a>
                    <?php endforeach ?>
                </div>
                <button type="button" class="nav-btn sl-prev" aria-label="<?= e($T['prev']) ?>"><i class="ti ti-chevron-left"></i></button>
                <button type="button" class="nav-btn sl-next" aria-label="<?= e($T['next']) ?>"><i class="ti ti-chevron-right"></i></button>
                <span class="zoom"><i class="ti ti-photo"></i> <?= e($T['zoom']) ?></span>
            </div>
            <p class="caption" id="shot-caption"><b><?= e($T['shots'][0][1]) ?></b> — <?= e($T['shots'][0][2]) ?></p>
            <div class="thumbs">
                <?php foreach ($T['shots'] as $i => [$f, $h, $p]): ?>
                    <button type="button" class="<?= $i ? '' : 'on' ?>" data-caption-h="<?= e($h) ?>" data-caption-p="<?= e($p) ?>" aria-label="<?= e($h) ?>"><img src="<?= v("shots/$f.jpg") ?>" alt="" width="120" height="81" loading="lazy"></button>
                <?php endforeach ?>
            </div>
        </div>
    </div>
</section>

<section id="learn" class="section">
    <div class="wrap">
        <span class="label">Learn Rust in Thai</span>
        <h2><?= e($T['learnH']) ?></h2>
        <p class="sub"><?= e($T['learnSub']) ?></p>
        <div class="grid g3">
            <?php foreach ($T['learn'] as [$icon, $b, $s]): ?>
                <a class="card lesson" href="<?= $B ?>help/"><div class="icon"><i class="ti <?= $icon ?>"></i></div><b><?= e($b) ?></b><p><?= e($s) ?></p></a>
            <?php endforeach ?>
        </div>
        <p class="mt"><a class="btn btn-primary" href="<?= $B ?>help/"><i class="ti ti-book"></i> <?= e($T['learnBtn']) ?></a></p>
    </div>
</section>

<section id="features" class="section tint">
    <div class="wrap">
        <span class="label">Features</span>
        <h2><?= e($T['featH']) ?></h2>
        <p class="sub"><?= e($T['featSub']) ?></p>
        <div class="grid g3">
            <?php foreach ($T['feat'] as [$icon, $h, $p]): ?>
                <article class="card"><div class="icon"><i class="ti <?= $icon ?>"></i></div><h3><?= e($h) ?></h3><p><?= e($p) ?></p></article>
            <?php endforeach ?>
        </div>
    </div>
</section>

<section class="section">
    <div class="wrap">
        <span class="label">Why Phai Studio</span>
        <h2><?= e($T['whyH']) ?></h2>
        <div class="grid g3 mt">
            <?php foreach ($T['why'] as [$b, $s]): ?>
                <article class="card row"><div class="icon"><i class="ti ti-check"></i></div><div><h3><?= e($b) ?></h3><p><?= e($s) ?></p></div></article>
            <?php endforeach ?>
        </div>
    </div>
</section>

<section id="install" class="section tint">
    <div class="wrap">
        <span class="label">Install</span>
        <h2><?= e($T['insH']) ?></h2>
        <p class="sub"><?= e($T['insSub']) ?></p>
        <div class="grid g3">
            <?php foreach ($T['ins'] as $i => $s): ?>
                <article class="card step"><span class="num"><?= $i + 1 ?></span><p><?= $s ?></p></article>
            <?php endforeach ?>
        </div>
        <p class="note mt"><i class="ti ti-shield-exclamation"></i> <?= $T['smart'] ?></p>
        <p class="note"><i class="ti ti-trash"></i> <?= e($T['uninst']) ?></p>
    </div>
</section>

<section id="support" class="section">
    <div class="wrap">
        <span class="label">Learning together</span>
        <h2><?= e($T['supH']) ?></h2>
        <p class="sub"><?= e($T['supSub']) ?></p>
        <div class="grid g2">
            <article class="card"><div class="icon"><i class="ti ti-book"></i></div><h3><?= e($T['selfH']) ?></h3><p><?= e($T['selfP']) ?></p>
                <p class="mt"><a class="btn btn-ghost" href="<?= $B ?>download/setup"><i class="ti ti-download"></i> <?= e($T['btnDownload']) ?></a></p></article>
            <article class="card main"><div class="icon"><i class="ti ti-users"></i></div><h3><?= e($T['proH']) ?></h3><p><?= e($T['proP']) ?></p>
                <ul class="muted"><?php foreach ($T['pro'] as $s): ?><li><?= $s ?></li><?php endforeach ?></ul></article>
        </div>
        <h3 class="mt"><?= e($T['contact']) ?></h3>
        <div class="contacts">
            <a class="card contact line" href="https://line.me/ti/p/~fahroong55" target="_blank" rel="noopener"><div class="icon"><i class="ti ti-brand-line"></i></div><span><b>LINE</b><small>fahroong55</small></span></a>
            <a class="card contact fb" href="https://www.facebook.com/defahroong" target="_blank" rel="noopener"><div class="icon"><i class="ti ti-brand-facebook"></i></div><span><b>Facebook</b><small>defahroong</small></span></a>
            <a class="card contact mail" href="mailto:defahroong@hotmail.com"><div class="icon"><i class="ti ti-mail"></i></div><span><b>Email</b><small>defahroong@hotmail.com</small></span></a>
        </div>
    </div>
</section>

<section id="download" class="section tint">
    <div class="wrap">
        <span class="label">Download</span>
        <h2><?= e($T['dlH']) ?></h2>
        <div class="card main file mt">
            <div>
                <h3><?= e($T['dlName']) ?></h3>
                <p><?= e($T['dlDesc']) ?></p>
                <details><summary><code><?= e($release['file']) ?></code></summary><?php if ($release['sha256']): ?><small>SHA-256 <code class="sha"><?= e($release['sha256']) ?></code></small><?php endif ?></details>
            </div>
            <a class="btn btn-primary" href="<?= $B ?>download/setup"><i class="ti ti-download"></i> <?= $mb ?> MB</a>
        </div>
        <p class="note mt"><?= $T['dlMeta']($c) ?></p>
        <p class="note"><i class="ti ti-brand-github"></i> <?= $T['dlBuilt'] ?></p>
        <p class="note"><i class="ti ti-certificate"></i> <?= $T['dlSign'] ?></p>
        <p class="note mt"><?= e($T['dlGit']) ?></p>
        <pre><code>git clone https://github.com/kirati-ai/phai-studio.git</code></pre>
    </div>
</section>
</main>

<footer class="foot">
    <div class="wrap">
        <span class="label">MIT License</span>
        <p class="note"><?= e($T['license']) ?></p>
        <blockquote><?= e($T['quote']) ?></blockquote>
        <h3><?= e($T['family']) ?></h3>
        <div class="grid g3 family">
            <?php foreach ($T['suite'] as [$href, $b, $s]): ?>
                <a class="card" href="<?= $href ?>"><b><?= e($b) ?></b><span><?= e($s) ?></span></a>
            <?php endforeach ?>
        </div>
        <p class="copy">© 2026 <a href="https://www.monrak.net/">Monrak Net</a> · <?= $th ? 'ทำด้วยความรักในอินเทอร์เน็ต' : 'made with a love of the internet' ?></p>
    </div>
</footer>

<div class="lightbox" id="lightbox" hidden>
    <button type="button" class="lb-close" aria-label="<?= e($T['close']) ?>"><i class="ti ti-x"></i></button>
    <button type="button" class="lb-prev" aria-label="<?= e($T['prev']) ?>"><i class="ti ti-chevron-left"></i></button>
    <button type="button" class="lb-next" aria-label="<?= e($T['next']) ?>"><i class="ti ti-chevron-right"></i></button>
    <div class="lb-stage"><img alt=""></div>
    <p class="lb-caption"></p>
</div>

<script src="<?= v('thai-wrap.js') ?>"></script>
<script>window.HUD_WORDS = <?= json_encode($T['hud'], JSON_UNESCAPED_UNICODE) ?>;</script>
<script src="<?= v('site.js') ?>"></script>
<script>setTimeout(() => navigator.sendBeacon && navigator.sendBeacon('<?= $B ?>seen'), 1500);</script>
</body>
</html>
