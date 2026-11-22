# Builds docs/help/*.html from the page bodies in help_pages*.py - one menu for every page.
# Run: python docs/help-src/help_gen.py
import importlib.util, html, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'help')

PAGES = [
    ('index.html', 'หน้าแรก', None),
    ('studio.html', 'ใช้งาน Phai Studio', 'Phai Studio'),
    ('css.html', 'ตกแต่งหน้าตาด้วย CSS', 'Phai Studio'),
    ('database.html', 'ทำงานกับฐานข้อมูล', 'Phai Studio'),
    ('code.html', 'เขียนโค้ดกับ control', 'Phai Studio'),
    ('winforms.html', 'control และความสามารถเพิ่มเติม', 'Phai Studio'),
    ('inspector.html', 'ตัวตรวจสด (F12)', 'Phai Studio'),
    ('web.html', 'เว็บและมือถือ (PWA)', 'Phai Studio'),
    ('report.html', 'รายงานพิมพ์', 'Phai Studio'),
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
for mod in ['help_pages1', 'help_pages2', 'help_pages3', 'help_pages4', 'help_pages5', 'help_pages6']:
    if os.path.exists(os.path.join(HERE, mod + '.py')):
        bodies.update(load(mod))

# The Toolbox pages (help_controls.py), after the rest, with the icons and pictures they show.
sys.path.insert(0, HERE)
import help_controls
for f, title, body in help_controls.pages():
    PAGES.append((f, title, 'Toolbox ทีละตัว'))
    bodies[f] = body


def copy_assets():
    import shutil
    icons = os.path.join(OUT, 'icons')
    os.makedirs(icons, exist_ok=True)
    for n in os.listdir(os.path.join(HERE, 'data', 'icons')):
        shutil.copyfile(os.path.join(HERE, 'data', 'icons', n), os.path.join(icons, n))
    img = os.path.join(OUT, 'img')
    os.makedirs(img, exist_ok=True)
    shots = os.path.join(HERE, 'data', 'shots')
    from PIL import Image
    for n in os.listdir(shots):
        im = Image.open(os.path.join(shots, n)).convert('RGB')
        # PHAI_SHOT takes the window with its invisible frame, drawn black: trimmed off.
        # An edge column or row that is mostly black goes (the title bar crosses the side ones).
        w, h = im.size
        px = im.load()
        dark = lambda x, y: sum(px[x, y]) < 30
        col = lambda x: sum(dark(x, y) for y in range(h)) > h * 0.6
        row = lambda y: sum(dark(x, y) for x in range(w)) > w * 0.6
        l, r, t, b = 0, w, 0, h
        while l < r - 1 and col(l): l += 1
        while r - 1 > l and col(r - 1): r -= 1
        while t < b - 1 and row(t): t += 1
        while b - 1 > t and row(b - 1): b -= 1
        im.crop((l, t, r, b)).save(os.path.join(img, n))


copy_assets()


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
