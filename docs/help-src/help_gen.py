# Builds docs/help/*.html from the page bodies in help_pages*.py - one menu for every page.
# Run: python docs/help-src/help_gen.py
import importlib.util, html, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'help')

PAGES = [
    ('index.html', 'หน้าแรก', None),
    ('studio.html', 'ใช้งาน Phai Studio', 'Phai Studio'),
    ('rust-01.html', '1. เริ่มต้นกับ Rust', 'เรียน Rust'),
    ('rust-02.html', '2. ตัวแปร ชนิดข้อมูล ฟังก์ชัน', 'เรียน Rust'),
    ('rust-03.html', '3. เงื่อนไขและการวนซ้ำ', 'เรียน Rust'),
    ('rust-04.html', '4. Ownership และการยืม', 'เรียน Rust'),
    ('rust-05.html', '5. struct, enum และ match', 'เรียน Rust'),
    ('rust-06.html', '6. จัดการข้อผิดพลาด', 'เรียน Rust'),
    ('rust-07.html', '7. Vec, String, HashMap', 'เรียน Rust'),
    ('rust-08.html', '8. trait และ generic', 'เรียน Rust'),
    ('rust-09.html', '9. โมดูล crate และเทสต์', 'เรียน Rust'),
    ('rust-10.html', '10. อ่าน error ของคอมไพเลอร์', 'เรียน Rust'),
    ('rust-11.html', '11. เธรด และก้าวต่อไป', 'เรียน Rust'),
]

def load(name):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, name + '.py'))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m.BODY

bodies = {}
for mod in ['help_pages1', 'help_pages2', 'help_pages3']:
    if os.path.exists(os.path.join(HERE, mod + '.py')):
        bodies.update(load(mod))

def nav(current):
    out = ['<a class="brand" href="index.html">คู่มือ Phai Studio</a>']
    group = None
    for f, title, g in PAGES:
        if g != group and g:
            out.append(f'<h4>{html.escape(g)}</h4>')
            group = g
        cls = ' class="here"' if f == current else ''
        out.append(f'<a href="{f}"{cls}>{html.escape(title)}</a>')
    return '\n'.join(out)

def pager(i):
    prev = PAGES[i - 1] if i > 0 else None
    nxt = PAGES[i + 1] if i + 1 < len(PAGES) else None
    a = f'<a href="{prev[0]}">← {html.escape(prev[1])}</a>' if prev else '<span></span>'
    b = f'<a href="{nxt[0]}">{html.escape(nxt[1])} →</a>' if nxt else '<span></span>'
    return f'<div class="pager">{a}{b}</div>'

os.makedirs(OUT, exist_ok=True)
for i, (f, title, _) in enumerate(PAGES):
    body = bodies.get(f)
    if body is None:
        print('missing', f)
        continue
    page = f'''<!doctype html>
<html lang="th">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)} - คู่มือ Phai Studio</title>
<link rel="stylesheet" href="help.css">
</head>
<body>
<div class="layout">
<nav>
{nav(f)}
</nav>
<main>
{body.strip()}
{pager(i)}
</main>
</div>
</body>
</html>
'''
    open(os.path.join(OUT, f), 'w', encoding='utf-8', newline='\n').write(page)
    print('wrote', f)
