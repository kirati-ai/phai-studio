<?php
// The Q&A board at /board: topics in groups, anyone may start one or answer, no sign-up and no
// waiting for approval. What is not fit is deleted afterwards in its back office, /board/admin,
// signed in as monrak.net's back office is (the same users, the same five tries a quarter hour).
// Kept in storage/board.sqlite. Reached only through index.php, which defines PHAI_SITE.

declare(strict_types=1);

if (!defined('PHAI_SITE')) {
    http_response_code(404);
    exit;
}

/** The groups: key, icon, name and what goes there (Thai, English); `admin`: only the back office starts topics. */
function board_groups(bool $th): array
{
    $g = [
        ['news', 'ti-speakerphone', 'ประกาศและข่าว', 'News', 'รุ่นใหม่และความสามารถใหม่ของ Phai Studio', 'New releases and what they bring', true],
        ['start', 'ti-plant', 'มือใหม่เริ่มต้น', 'Getting started', 'ติดตั้ง สร้างโปรเจกต์แรก ถามได้ทุกเรื่อง', 'Installing, the first project - ask anything', false],
        ['design', 'ti-layout', 'ออกแบบหน้าจอ', 'Designing forms', 'ฟอร์ม control การจัดวาง และธีม CSS', 'Forms, controls, layout and CSS themes', false],
        ['rust', 'ti-code', 'เขียนโค้ด Rust', 'Rust code', 'ภาษา Rust, error จากคอมไพเลอร์, event', 'The Rust language, compiler errors, events', false],
        ['data', 'ti-database', 'ฐานข้อมูลและรายงาน', 'Data and reports', 'SQLite, MySQL, PostgreSQL, ตาราง, พิมพ์และ PDF', 'SQLite, MySQL, PostgreSQL, grids, printing and PDF', false],
        ['mobile', 'ti-device-mobile', 'แอปมือถือและเว็บ', 'Phone apps and web', 'แอปมือถือ (PWA) และ phai-web', 'Phone apps (PWA) and phai-web', false],
        ['tips', 'ti-bulb', 'เคล็ดลับและบทเรียน', 'Tips and lessons', 'แบ่งปันวิธีทำและตัวอย่างโค้ด', 'Ways of doing things and code to share', false],
        ['show', 'ti-photo', 'อวดผลงาน', 'Show your program', 'โปรแกรมที่ทำด้วย Phai Studio', 'Programs made with Phai Studio', false],
        ['bugs', 'ti-bug', 'แจ้งปัญหา', 'Problems', 'เจอปัญหา บอกวิธีทำให้เกิดซ้ำ', 'Something wrong - how to make it happen', false],
        ['ideas', 'ti-sparkles', 'เสนอความสามารถ', 'Ideas', 'อยากให้ Phai Studio ทำอะไรเพิ่ม', 'What Phai Studio should do next', false],
        ['chat', 'ti-coffee', 'พูดคุยทั่วไป', 'General talk', 'เรื่องอื่น ๆ', 'Anything else', false],
    ];
    $out = [];
    foreach ($g as [$key, $icon, $nth, $nen, $ath, $aen, $admin]) {
        $out[$key] = ['key' => $key, 'icon' => $icon, 'name' => $th ? $nth : $nen, 'about' => $th ? $ath : $aen, 'admin' => $admin];
    }
    return $out;
}

/** A question a person answers at once and a robot does not read: two numbers in words, added.
 * The answer is kept in the session under the form's time (`t`), so each form has its own. */
function board_question(bool $th, int $t): string
{
    $words = $th ? ['ศูนย์', 'หนึ่ง', 'สอง', 'สาม', 'สี่', 'ห้า', 'หก', 'เจ็ด', 'แปด', 'เก้า'] : ['zero', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine'];
    $a = random_int(1, 9);
    $b = random_int(1, 9);
    $_SESSION['cap'] ??= [];
    $_SESSION['cap'][$t] = $a + $b;
    // Only the last few forms' answers kept.
    if (count($_SESSION['cap']) > 20) {
        $_SESSION['cap'] = array_slice($_SESSION['cap'], -20, null, true);
    }
    return $th ? "{$words[$a]} บวก {$words[$b]} เท่ากับเท่าไร?" : "What is {$words[$a]} plus {$words[$b]}?";
}

function board_db(string $storage): PDO
{
    if (!is_dir($storage)) {
        @mkdir($storage, 0775, true);
    }
    $db = new PDO('sqlite:' . $storage . '/board.sqlite', null, null, [PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION, PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC]);
    $db->exec('PRAGMA journal_mode = WAL');
    $db->exec('PRAGMA foreign_keys = ON');
    $db->exec('CREATE TABLE IF NOT EXISTS topics (id INTEGER PRIMARY KEY, grp TEXT NOT NULL, title TEXT NOT NULL, body TEXT NOT NULL, name TEXT NOT NULL, ip TEXT NOT NULL, at INTEGER NOT NULL, last_at INTEGER NOT NULL, replies INTEGER NOT NULL DEFAULT 0, views INTEGER NOT NULL DEFAULT 0)');
    $db->exec('CREATE INDEX IF NOT EXISTS topics_grp ON topics (grp, last_at)');
    $db->exec('CREATE TABLE IF NOT EXISTS replies (id INTEGER PRIMARY KEY, topic INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE, body TEXT NOT NULL, name TEXT NOT NULL, ip TEXT NOT NULL, at INTEGER NOT NULL)');
    $db->exec('CREATE INDEX IF NOT EXISTS replies_topic ON replies (topic, id)');
    $db->exec('CREATE TABLE IF NOT EXISTS posts (ip TEXT NOT NULL, at INTEGER NOT NULL)');
    return $db;
}

/** A post's words for the page: plain text, its lines kept, links made links, ``` blocks as code. */
function board_text(string $s): string
{
    $parts = explode('```', $s);
    $html = '';
    foreach ($parts as $i => $p) {
        if ($i % 2 === 1) {
            $html .= '<pre class="code"><code>' . htmlspecialchars(trim($p, "\r\n"), ENT_QUOTES, 'UTF-8') . '</code></pre>';
            continue;
        }
        $t = htmlspecialchars($p, ENT_QUOTES, 'UTF-8');
        $t = preg_replace('~https?://[^\s<>"\']+~u', '<a href="$0" rel="nofollow ugc noopener" target="_blank">$0</a>', $t);
        $html .= nl2br(trim($t, "\r\n") === '' ? '' : $t);
    }
    return $html;
}

function board_ago(int $at, bool $th): string
{
    $d = time() - $at;
    if ($d < 60) return $th ? 'เมื่อสักครู่' : 'just now';
    if ($d < 3600) return $th ? intdiv($d, 60) . ' นาทีที่แล้ว' : intdiv($d, 60) . ' min ago';
    if ($d < 86400) return $th ? intdiv($d, 3600) . ' ชั่วโมงที่แล้ว' : intdiv($d, 3600) . ' h ago';
    if ($d < 86400 * 30) return $th ? intdiv($d, 86400) . ' วันที่แล้ว' : intdiv($d, 86400) . ' days ago';
    return date($th ? 'd/m/Y' : 'd M Y', $at);
}

/** monrak.net's back office sign-in: its Site class, on this server or beside this site locally. */
function board_monrak(): ?string
{
    foreach ([getenv('MONRAK_SITE') ?: '', '/www/wwwroot/monrak.net', __DIR__ . '/../monrak-site', 'D:/DesktopServer/www/monrak-site'] as $d) {
        if ($d !== '' && is_file($d . '/src/Site.php')) {
            return $d . '/src/Site.php';
        }
    }
    return null;
}

/** The board: everything under /board. Returns when the path is not the board's. */
function board_page(string $path, string $B, bool $th, string $storage): void
{
    if ($path !== '/board' && !str_starts_with($path, '/board/')) {
        return;
    }
    if (session_status() !== PHP_SESSION_ACTIVE) {
        session_name('phaiboard');
        session_set_cookie_params(['path' => $B, 'httponly' => true, 'samesite' => 'Lax', 'secure' => !empty($_SERVER['HTTPS'])]);
        session_start();
    }
    $_SESSION['csrf'] ??= bin2hex(random_bytes(16));
    $csrf = $_SESSION['csrf'];
    $e = fn (string $s): string => htmlspecialchars($s, ENT_QUOTES, 'UTF-8');
    $groups = board_groups($th);
    try {
        $db = board_db($storage);
    } catch (Throwable) {
        http_response_code(503);
        echo $th ? 'กระดานถามตอบยังใช้ไม่ได้ในตอนนี้ ลองใหม่ภายหลัง' : 'The Q&A board is not available right now - try again later.';
        exit;
    }
    $ip = substr((string) ($_SERVER['REMOTE_ADDR'] ?? ''), 0, 45);
    $admin = $_SESSION['board_admin'] ?? null;
    $go = function (string $to): never {
        header('Location: ' . $to, true, 303);
        exit;
    };
    $L = $th ? [
        'title' => 'ถามตอบ Phai Studio', 'lead' => 'ถาม ตอบ และแบ่งปันความรู้เรื่อง Phai Studio และ Rust ใครก็เขียนได้ ไม่ต้องสมัคร',
        'topics' => 'กระทู้', 'replies' => 'ตอบ', 'views' => 'อ่าน', 'latest' => 'กระทู้ล่าสุด', 'new' => 'ตั้งกระทู้ใหม่', 'none' => 'ยังไม่มีกระทู้ในหมวดนี้ เริ่มเป็นคนแรกได้เลย',
        'name' => 'ชื่อ (ไม่ใส่ก็ได้)', 'subject' => 'หัวข้อ', 'body' => 'รายละเอียด', 'reply' => 'ตอบกระทู้', 'send' => 'ส่ง', 'post' => 'ตั้งกระทู้', 'group' => 'หมวด',
        'anon' => 'ไม่ระบุชื่อ', 'code' => 'ใส่โค้ดระหว่าง ``` สองชุด จะขึ้นเป็นกรอบโค้ด', 'back' => 'กลับไปหน้ารวม', 'home' => 'หน้าหลัก', 'all' => 'ทุกหมวด',
        'tooMany' => 'ส่งบ่อยเกินไป รอสักครู่แล้วลองใหม่', 'tooFast' => 'ส่งเร็วเกินไป ลองอีกครั้ง', 'empty' => 'ใส่หัวข้อและรายละเอียดก่อนส่ง', 'links' => 'ใส่ลิงก์ได้ไม่เกิน 3 ลิงก์', 'stale' => 'หน้านี้เปิดไว้นานเกินไป ลองส่งอีกครั้ง',
        'notFound' => 'ไม่พบกระทู้นี้ อาจถูกลบไปแล้ว', 'admin' => 'หลังบ้าน', 'login' => 'เข้าสู่ระบบ', 'user' => 'ผู้ใช้', 'pass' => 'รหัสผ่าน', 'logout' => 'ออกจากระบบ', 'delete' => 'ลบ', 'sure' => 'ลบจริงไหม?',
        'wrong' => 'ผู้ใช้หรือรหัสผ่านไม่ถูก', 'locked' => 'ลองผิดหลายครั้ง รอ 15 นาที', 'noAdmin' => 'ยังเข้าหลังบ้านไม่ได้: ไม่พบระบบผู้ใช้ของ monrak.net', 'recentReplies' => 'คำตอบล่าสุด', 'onlyAdmin' => 'หมวดนี้ตั้งกระทู้ได้เฉพาะผู้ดูแล',
        'page' => 'หน้า', 'by' => 'โดย', 'rules' => 'เขียนอย่างสุภาพ ข้อความที่ไม่เหมาะสมจะถูกลบ',
        'cap' => 'คำถามกันสแปม', 'capHint' => 'ตอบเป็นตัวเลข', 'capWrong' => 'คำตอบคำถามกันสแปมไม่ถูก ลองข้อใหม่',
    ] : [
        'title' => 'Phai Studio Q&A', 'lead' => 'Ask, answer and share what you know about Phai Studio and Rust - anyone may write, no sign-up.',
        'topics' => 'topics', 'replies' => 'replies', 'views' => 'views', 'latest' => 'Latest topics', 'new' => 'New topic', 'none' => 'No topic here yet - start the first.',
        'name' => 'Name (optional)', 'subject' => 'Subject', 'body' => 'Details', 'reply' => 'Reply', 'send' => 'Send', 'post' => 'Post topic', 'group' => 'Group',
        'anon' => 'Anonymous', 'code' => 'Put code between two ``` lines to show it as code', 'back' => 'All groups', 'home' => 'Home', 'all' => 'All groups',
        'tooMany' => 'Too many posts - wait a little and try again', 'tooFast' => 'Sent too quickly - try again', 'empty' => 'Give a subject and some details first', 'links' => 'At most 3 links in a post', 'stale' => 'This page was open too long - send it again',
        'notFound' => 'No such topic - it may have been deleted', 'admin' => 'Back office', 'login' => 'Sign in', 'user' => 'User', 'pass' => 'Password', 'logout' => 'Sign out', 'delete' => 'Delete', 'sure' => 'Delete it?',
        'wrong' => 'Wrong user or password', 'locked' => 'Too many tries - wait 15 minutes', 'noAdmin' => 'The back office cannot open: monrak.net\'s users were not found', 'recentReplies' => 'Latest replies', 'onlyAdmin' => 'Only the back office starts topics here',
        'page' => 'Page', 'by' => 'by', 'rules' => 'Be kind - what is not fit will be deleted',
        'cap' => 'Anti-spam question', 'capHint' => 'answer with a number', 'capWrong' => 'The anti-spam answer was wrong - try the new one',
    ];
    $sub = substr($path, strlen('/board'));
    $error = '';
    $method = $_SERVER['REQUEST_METHOD'] ?? 'GET';
    $posted = $method === 'POST' && hash_equals($csrf, (string) ($_POST['csrf'] ?? ''));

    // A post's checks: a person (the hidden field empty, a few seconds on the page, not a robot),
    // not too often from one address, its words within bounds.
    $check = function (string $body, string $title = 'x') use ($db, $ip, $L, $admin): string {
        if (($_POST['website'] ?? '') !== '' || is_bot()) {
            return $L['tooFast'];
        }
        $shown = (int) ($_POST['t'] ?? 0);
        $at = intdiv($shown, 100);
        if ($shown <= 0 || time() - $at < 3) {
            return $L['tooFast'];
        }
        if (time() - $at > 86400) {
            return $L['stale'];
        }
        if (trim($body) === '' || trim($title) === '') {
            return $L['empty'];
        }
        if (preg_match_all('~https?://~i', $body) > 3) {
            return $L['links'];
        }
        if (!$admin) {
            $want = $_SESSION['cap'][$shown] ?? null;
            $got = trim(strtr((string) ($_POST['cap'] ?? ''), ['๐' => '0', '๑' => '1', '๒' => '2', '๓' => '3', '๔' => '4', '๕' => '5', '๖' => '6', '๗' => '7', '๘' => '8', '๙' => '9']));
            unset($_SESSION['cap'][$shown]);
            if ($want === null || $got !== (string) $want) {
                return $L['capWrong'];
            }
        }
        if (!$admin) {
            $q = $db->prepare('SELECT COUNT(*) FROM posts WHERE ip = ? AND at > ?');
            $q->execute([$ip, time() - 600]);
            if ((int) $q->fetchColumn() >= 5) {
                return $L['tooMany'];
            }
        }
        return '';
    };
    $name = fn (): string => mb_substr(trim((string) ($_POST['name'] ?? '')), 0, 40) ?: $L['anon'];
    $remember = function (string $n) use ($B, $L): void {
        if ($n !== $L['anon']) {
            setcookie('board_name', $n, ['expires' => time() + 86400 * 365, 'path' => $B, 'samesite' => 'Lax']);
        }
    };

    // --- back office -------------------------------------------------------------------------
    if ($sub === '/admin' || str_starts_with($sub, '/admin/')) {
        if ($posted && ($_POST['do'] ?? '') === 'login') {
            $site = board_monrak();
            if (!$site) {
                $error = $L['noAdmin'];
            } else {
                require_once $site;
                $r = \Site::login(trim((string) ($_POST['user'] ?? '')), (string) ($_POST['pass'] ?? ''));
                if ($r && empty($r['locked'])) {
                    session_regenerate_id(true);
                    $_SESSION['board_admin'] = $r['username'];
                    $go($B . 'board/admin');
                }
                $error = $r ? $L['locked'] : $L['wrong'];
            }
        } elseif ($posted && $admin && ($_POST['do'] ?? '') === 'logout') {
            unset($_SESSION['board_admin']);
            $go($B . 'board');
        } elseif ($posted && $admin && ($_POST['do'] ?? '') === 'delete') {
            $id = (int) ($_POST['id'] ?? 0);
            if (($_POST['kind'] ?? '') === 'topic') {
                $db->prepare('DELETE FROM topics WHERE id = ?')->execute([$id]);
            } else {
                $t = $db->prepare('SELECT topic FROM replies WHERE id = ?');
                $t->execute([$id]);
                $topic = (int) $t->fetchColumn();
                $db->prepare('DELETE FROM replies WHERE id = ?')->execute([$id]);
                $db->prepare('UPDATE topics SET replies = (SELECT COUNT(*) FROM replies WHERE topic = ?) WHERE id = ?')->execute([$topic, $topic]);
            }
            $go((string) ($_POST['back'] ?? ($B . 'board/admin')));
        }
    }

    // --- a new topic, a reply -------------------------------------------------------------------
    if ($posted && $sub === '/new') {
        $grp = (string) ($_POST['grp'] ?? '');
        $title = mb_substr(trim((string) ($_POST['title'] ?? '')), 0, 120);
        $body = mb_substr(trim((string) ($_POST['body'] ?? '')), 0, 8000);
        $error = isset($groups[$grp]) ? $check($body, $title) : $L['empty'];
        if (!$error && $groups[$grp]['admin'] && !$admin) {
            $error = $L['onlyAdmin'];
        }
        if (!$error) {
            $n = $admin && trim((string) ($_POST['name'] ?? '')) === '' ? 'Phai Studio' : $name();
            $now = time();
            $db->prepare('INSERT INTO topics (grp, title, body, name, ip, at, last_at) VALUES (?, ?, ?, ?, ?, ?, ?)')->execute([$grp, $title, $body, $n, $ip, $now, $now]);
            $id = (int) $db->lastInsertId();
            $db->prepare('INSERT INTO posts (ip, at) VALUES (?, ?)')->execute([$ip, $now]);
            $remember($n);
            $go($B . 'board/t/' . $id);
        }
        $sub = '/c/' . $grp;
    }
    if ($posted && preg_match('~^/t/(\d+)/reply$~', $sub, $m)) {
        $id = (int) $m[1];
        $body = mb_substr(trim((string) ($_POST['body'] ?? '')), 0, 8000);
        $error = $check($body);
        $exists = $db->prepare('SELECT COUNT(*) FROM topics WHERE id = ?');
        $exists->execute([$id]);
        if (!$error && (int) $exists->fetchColumn() === 1) {
            $n = $name();
            $now = time();
            $db->prepare('INSERT INTO replies (topic, body, name, ip, at) VALUES (?, ?, ?, ?, ?)')->execute([$id, $body, $n, $ip, $now]);
            $rid = (int) $db->lastInsertId();
            $db->prepare('UPDATE topics SET replies = replies + 1, last_at = ? WHERE id = ?')->execute([$now, $id]);
            $db->prepare('INSERT INTO posts (ip, at) VALUES (?, ?)')->execute([$ip, $now]);
            $db->prepare('DELETE FROM posts WHERE at < ?')->execute([$now - 86400]);
            $remember($n);
            $go($B . 'board/t/' . $id . '#r' . $rid);
        }
        $sub = '/t/' . $id;
    }

    // --- the pages ------------------------------------------------------------------------------
    $title = $L['title'];
    $body = '';
    $savedName = (string) ($_COOKIE['board_name'] ?? '');
    $form = function (string $action, bool $topic, string $grp = '') use ($e, $L, $csrf, $groups, $savedName, $admin, $th): string {
        // The form's time: when it was shown, and the key its anti-spam answer is kept under.
        $now = time() * 100 + random_int(0, 99);
        $h = '<form class="board-form" method="post" action="' . $e($action) . '">';
        $h .= '<input type="hidden" name="csrf" value="' . $e($csrf) . '"><input type="hidden" name="t" value="' . $now . '">';
        $h .= '<label class="hp" aria-hidden="true">Website<input name="website" tabindex="-1" autocomplete="off"></label>';
        if ($topic) {
            $h .= '<label>' . $e($L['group']) . '<select name="grp">';
            foreach ($groups as $g) {
                if ($g['admin'] && !$admin) {
                    continue;
                }
                $h .= '<option value="' . $e($g['key']) . '"' . ($g['key'] === $grp ? ' selected' : '') . '>' . $e($g['name']) . '</option>';
            }
            $h .= '</select></label>';
            $h .= '<label>' . $e($L['subject']) . '<input name="title" maxlength="120" required></label>';
        }
        $h .= '<label>' . $e($L['body']) . '<textarea name="body" rows="' . ($topic ? 8 : 5) . '" maxlength="8000" required></textarea><small>' . $e($L['code']) . '</small></label>';
        $h .= '<div class="row"><label>' . $e($L['name']) . '<input name="name" maxlength="40" value="' . $e($savedName) . '" autocomplete="nickname"></label>';
        if (!$admin) {
            $h .= '<label class="cap">' . $e($L['cap']) . ': ' . $e(board_question($th, $now)) . '<input name="cap" inputmode="numeric" maxlength="3" required autocomplete="off" placeholder="' . $e($L['capHint']) . '"></label>';
        }
        $h .= '<button class="btn btn-primary" type="submit"><i class="ti ti-send"></i> ' . $e($topic ? $L['post'] : $L['send']) . '</button></div>';
        $h .= '<p class="rules">' . $e($L['rules']) . '</p></form>';
        return $h;
    };
    $del = function (string $kind, int $id, string $back) use ($e, $L, $csrf, $B): string {
        return '<form class="del" method="post" action="' . $e($B . 'board/admin/delete') . '" onsubmit="return confirm(\'' . $e($L['sure']) . '\')"><input type="hidden" name="csrf" value="' . $e($csrf) . '"><input type="hidden" name="do" value="delete"><input type="hidden" name="kind" value="' . $kind . '"><input type="hidden" name="id" value="' . $id . '"><input type="hidden" name="back" value="' . $e($back) . '"><button type="submit" class="btn-del"><i class="ti ti-trash"></i> ' . $e($L['delete']) . '</button></form>';
    };
    $row = function (array $t) use ($e, $L, $B, $th, $groups): string {
        $g = $groups[$t['grp']] ?? null;
        return '<a class="topic" href="' . $e($B . 'board/t/' . $t['id']) . '"><span class="t-title">' . $e($t['title']) . '</span><span class="t-meta">' . ($g ? '<span class="t-grp"><i class="ti ' . $e($g['icon']) . '"></i> ' . $e($g['name']) . '</span> · ' : '') . $e($L['by']) . ' ' . $e($t['name']) . ' · ' . $e(board_ago((int) $t['last_at'], $th)) . '</span><span class="t-count">' . (int) $t['replies'] . ' <small>' . $e($L['replies']) . '</small></span></a>';
    };

    if ($sub === '/admin' || str_starts_with($sub, '/admin/')) {
        $title = $L['admin'] . ' - ' . $L['title'];
        if (!$admin) {
            $body .= '<h1>' . $e($L['admin']) . '</h1>' . ($error ? '<p class="err">' . $e($error) . '</p>' : '');
            $body .= '<form class="board-form narrow" method="post" action="' . $e($B . 'board/admin') . '"><input type="hidden" name="csrf" value="' . $e($csrf) . '"><input type="hidden" name="do" value="login">';
            $body .= '<label>' . $e($L['user']) . '<input name="user" autocomplete="username" required></label><label>' . $e($L['pass']) . '<input name="pass" type="password" autocomplete="current-password" required></label>';
            $body .= '<button class="btn btn-primary" type="submit">' . $e($L['login']) . '</button></form>';
        } else {
            $body .= '<div class="board-head"><h1>' . $e($L['admin']) . '</h1><form method="post" action="' . $e($B . 'board/admin') . '"><input type="hidden" name="csrf" value="' . $e($csrf) . '"><input type="hidden" name="do" value="logout"><button class="btn btn-ghost" type="submit">' . $e($admin) . ' · ' . $e($L['logout']) . '</button></form></div>';
            $body .= '<h2>' . $e($L['latest']) . '</h2><div class="admin-list">';
            foreach ($db->query('SELECT * FROM topics ORDER BY at DESC LIMIT 100') as $t) {
                $body .= '<div class="admin-item"><div><a href="' . $e($B . 'board/t/' . $t['id']) . '"><b>' . $e($t['title']) . '</b></a><div class="t-meta">' . $e($t['name']) . ' · ' . $e($t['ip']) . ' · ' . $e(board_ago((int) $t['at'], $th)) . '</div><div class="excerpt">' . $e(mb_substr($t['body'], 0, 200)) . '</div></div>' . $del('topic', (int) $t['id'], $B . 'board/admin') . '</div>';
            }
            $body .= '</div><h2>' . $e($L['recentReplies']) . '</h2><div class="admin-list">';
            foreach ($db->query('SELECT r.*, t.title FROM replies r JOIN topics t ON t.id = r.topic ORDER BY r.at DESC LIMIT 100') as $r) {
                $body .= '<div class="admin-item"><div><a href="' . $e($B . 'board/t/' . $r['topic'] . '#r' . $r['id']) . '">' . $e($r['title']) . '</a><div class="t-meta">' . $e($r['name']) . ' · ' . $e($r['ip']) . ' · ' . $e(board_ago((int) $r['at'], $th)) . '</div><div class="excerpt">' . $e(mb_substr($r['body'], 0, 200)) . '</div></div>' . $del('reply', (int) $r['id'], $B . 'board/admin') . '</div>';
            }
            $body .= '</div>';
        }
    } elseif (preg_match('~^/t/(\d+)$~', $sub, $m)) {
        $id = (int) $m[1];
        $q = $db->prepare('SELECT * FROM topics WHERE id = ?');
        $q->execute([$id]);
        $t = $q->fetch();
        if (!$t) {
            http_response_code(404);
            $body .= '<p class="err">' . $e($L['notFound']) . '</p><p><a href="' . $e($B . 'board') . '">' . $e($L['back']) . '</a></p>';
        } else {
            if (!is_bot()) {
                $db->prepare('UPDATE topics SET views = views + 1 WHERE id = ?')->execute([$id]);
            }
            $g = $groups[$t['grp']] ?? ['name' => $t['grp'], 'icon' => 'ti-folder', 'key' => $t['grp']];
            $title = $t['title'] . ' - ' . $L['title'];
            $body .= '<nav class="crumbs"><a href="' . $e($B . 'board') . '">' . $e($L['title']) . '</a> › <a href="' . $e($B . 'board/c/' . $g['key']) . '"><i class="ti ' . $e($g['icon']) . '"></i> ' . $e($g['name']) . '</a></nav>';
            $body .= '<article class="post first"><h1>' . $e($t['title']) . '</h1><div class="t-meta">' . $e($t['name']) . ' · ' . $e(board_ago((int) $t['at'], $th)) . ' · ' . ((int) $t['views'] + 1) . ' ' . $e($L['views']) . '</div><div class="post-body">' . board_text($t['body']) . '</div>' . ($admin ? $del('topic', $id, $B . 'board/c/' . $g['key']) : '') . '</article>';
            $r = $db->prepare('SELECT * FROM replies WHERE topic = ? ORDER BY id');
            $r->execute([$id]);
            $n = 0;
            foreach ($r as $reply) {
                $n++;
                $body .= '<article class="post" id="r' . (int) $reply['id'] . '"><div class="t-meta"><b>#' . $n . '</b> ' . $e($reply['name']) . ' · ' . $e(board_ago((int) $reply['at'], $th)) . '</div><div class="post-body">' . board_text($reply['body']) . '</div>' . ($admin ? $del('reply', (int) $reply['id'], $B . 'board/t/' . $id) : '') . '</article>';
            }
            $body .= '<h2 id="reply">' . $e($L['reply']) . '</h2>' . ($error ? '<p class="err">' . $e($error) . '</p>' : '') . $form($B . 'board/t/' . $id . '/reply', false);
        }
    } elseif (preg_match('~^/c/([a-z]+)$~', $sub, $m) && isset($groups[$m[1]])) {
        $g = $groups[$m[1]];
        $page = max(1, (int) ($_GET['p'] ?? 1));
        $title = $g['name'] . ' - ' . $L['title'];
        $body .= '<nav class="crumbs"><a href="' . $e($B . 'board') . '">' . $e($L['title']) . '</a> › ' . $e($g['name']) . '</nav>';
        $body .= '<div class="board-head"><h1><i class="ti ' . $e($g['icon']) . '"></i> ' . $e($g['name']) . '</h1>' . ((!$g['admin'] || $admin) ? '<a class="btn btn-primary" href="#new"><i class="ti ti-plus"></i> ' . $e($L['new']) . '</a>' : '') . '</div><p class="sub">' . $e($g['about']) . '</p>';
        $q = $db->prepare('SELECT * FROM topics WHERE grp = ? ORDER BY last_at DESC LIMIT 31 OFFSET ?');
        $q->execute([$g['key'], ($page - 1) * 30]);
        $list = $q->fetchAll();
        $body .= '<div class="topics">' . ($list ? implode('', array_map($row, array_slice($list, 0, 30))) : '<p class="none">' . $e($L['none']) . '</p>') . '</div>';
        if ($page > 1 || count($list) > 30) {
            $body .= '<nav class="pages">' . ($page > 1 ? '<a href="?p=' . ($page - 1) . '">‹</a>' : '') . '<span>' . $e($L['page']) . ' ' . $page . '</span>' . (count($list) > 30 ? '<a href="?p=' . ($page + 1) . '">›</a>' : '') . '</nav>';
        }
        if (!$g['admin'] || $admin) {
            $body .= '<h2 id="new">' . $e($L['new']) . '</h2>' . ($error ? '<p class="err">' . $e($error) . '</p>' : '') . $form($B . 'board/new', true, $g['key']);
        }
    } else {
        $counts = [];
        foreach ($db->query('SELECT grp, COUNT(*) n, MAX(last_at) last FROM topics GROUP BY grp') as $c) {
            $counts[$c['grp']] = $c;
        }
        $body .= '<div class="board-head"><div><h1>' . $e($L['title']) . '</h1><p class="sub">' . $e($L['lead']) . '</p></div><a class="btn btn-primary" href="' . $e($B . 'board/c/start#new') . '"><i class="ti ti-plus"></i> ' . $e($L['new']) . '</a></div>';
        $body .= '<div class="groups">';
        foreach ($groups as $g) {
            $c = $counts[$g['key']] ?? null;
            $body .= '<a class="group" href="' . $e($B . 'board/c/' . $g['key']) . '"><i class="ti ' . $e($g['icon']) . '"></i><span><b>' . $e($g['name']) . '</b><small>' . $e($g['about']) . '</small></span><span class="g-count">' . (int) ($c['n'] ?? 0) . ' <small>' . $e($L['topics']) . '</small></span></a>';
        }
        $body .= '</div><h2>' . $e($L['latest']) . '</h2>';
        $latest = $db->query('SELECT * FROM topics ORDER BY last_at DESC LIMIT 15')->fetchAll();
        $body .= '<div class="topics">' . ($latest ? implode('', array_map($row, $latest)) : '<p class="none">' . $e($L['none']) . '</p>') . '</div>';
    }

    header('Content-Type: text/html; charset=utf-8');
    header('X-Frame-Options: DENY');
    $other = $th ? 'en' : 'th';
    ?><!doctype html>
<html lang="<?= $th ? 'th' : 'en' ?>">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title><?= $e($title) ?></title>
    <meta name="robots" content="<?= str_starts_with($sub, '/admin') ? 'noindex, nofollow' : 'index, follow' ?>">
    <link rel="icon" href="<?= v('logo.svg') ?>" type="image/svg+xml">
    <link rel="stylesheet" href="<?= v('tabler-icons/tabler-icons.min.css') ?>">
    <link rel="stylesheet" href="<?= v('site.css') ?>">
</head>
<body class="board-page">
<header class="top">
    <div class="wrap top-row">
        <a class="brand" href="<?= $B ?>"><img src="<?= v('logo.svg') ?>" alt="" width="34" height="34"><span><small>MONRAK</small><b>Phai Studio</b></span></a>
        <nav class="nav">
            <a href="<?= $B ?>"><?= $e($L['home']) ?></a>
            <a href="<?= $B ?>board"><?= $e($L['title']) ?></a>
            <a href="<?= $B ?>help/" target="_blank" rel="noopener"><?= $th ? 'คู่มือ' : 'Guide' ?></a>
            <a class="lang" href="?lang=<?= $other ?>"><i class="ti ti-world"></i> <?= $th ? 'EN' : 'ไทย' ?></a>
        </nav>
    </div>
</header>
<main class="wrap board">
<?= $body ?>
</main>
<footer class="board-foot wrap"><a href="<?= $B ?>board/admin"><?= $e($L['admin']) ?></a></footer>
</body>
</html>
<?php
    exit;
}
