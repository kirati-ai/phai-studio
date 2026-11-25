# Builds docs/help/*.html from the page bodies in help_pages*.py - one menu for every page.
# Run: python docs/help-src/help_gen.py
import importlib.util, html, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'help')

PAGES = [
    ('index.html', 'หน้าแรก', None),
    ('studio.html', 'ใช้งาน Phai Studio', 'Phai Studio'),
    ('templates.html', 'เริ่มจากแม่แบบ', 'Phai Studio'),
    ('css.html', 'ตกแต่งหน้าตาด้วย CSS', 'Phai Studio'),
    ('database.html', 'ทำงานกับฐานข้อมูล', 'Phai Studio'),
    ('code.html', 'เขียนโค้ดกับ control', 'Phai Studio'),
    ('winforms.html', 'control และความสามารถเพิ่มเติม', 'Phai Studio'),
    ('inspector.html', 'ตัวตรวจสด (F12)', 'Phai Studio'),
    ('web.html', 'เว็บและมือถือ (PWA)', 'Phai Studio'),
    ('report.html', 'รายงานพิมพ์', 'Phai Studio'),
    ('publish.html', 'ทำตัวติดตั้ง (Publish)', 'Phai Studio'),
    ('rust-01.html', '1. เริ่มต้นกับ Rust', 'Rust Language'),
    ('rust-02.html', '2. ตัวแปร ชนิดข้อมูล ฟังก์ชัน', 'Rust Language'),
    ('rust-03.html', '3. เงื่อนไขและการวนซ้ำ', 'Rust Language'),
    ('rust-04.html', '4. Ownership และการยืม', 'Rust Language'),
    ('rust-05.html', '5. struct, enum และ match', 'Rust Language'),
    ('rust-06.html', '6. จัดการข้อผิดพลาด', 'Rust Language'),
    ('rust-07.html', '7. Vec, String, HashMap', 'Rust Language'),
    ('rust-08.html', '8. trait และ generic', 'Rust Language'),
    ('rust-09.html', '9. โมดูล crate และเทสต์', 'Rust Language'),
    ('rust-10.html', '10. อ่าน error ของคอมไพเลอร์', 'Rust Language'),
    ('rust-11.html', '11. เธรด และก้าวต่อไป', 'Rust Language'),
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


# The Toolbox pages (help_controls.py), with the icons and pictures they show.
sys.path.insert(0, HERE)
import help_controls
# The Toolbox group goes right after Phai Studio's own pages, before Rust.
at = max(i for i, p in enumerate(PAGES) if p[2] == 'Phai Studio') + 1
for f, title, body, group in help_controls.pages():
    PAGES.insert(at, (f, title, group))
    at += 1
    bodies[f] = body




# Line icons for the guide's own pages (24-unit boxes, drawn in the text's colour).
def _svg(d):
    return f'<svg class="ni" viewBox="0 0 24 24" aria-hidden="true">{d}</svg>'

PAGE_ICONS = {
    'index.html': _svg('<path d="M3 11.5 12 4l9 7.5"/><path d="M5.5 9.5V20h13V9.5"/><path d="M10 20v-5h4v5"/>'),
    'studio.html': _svg('<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/><path d="M7 6.5h.01M10 6.5h.01"/>'),
    'css.html': _svg('<path d="M12 3a9 9 0 1 0 0 18c1.1 0 1.5-.9 1.2-1.7-.4-1 .2-2.3 1.5-2.3H17a4 4 0 0 0 4-4c0-5-4-10-9-10z"/><circle cx="7.5" cy="11" r="1"/><circle cx="10" cy="7" r="1"/><circle cx="14.5" cy="7" r="1"/>'),
    'database.html': _svg('<ellipse cx="12" cy="6" rx="7" ry="3"/><path d="M5 6v12c0 1.7 3.1 3 7 3s7-1.3 7-3V6"/><path d="M5 12c0 1.7 3.1 3 7 3s7-1.3 7-3"/>'),
    'code.html': _svg('<path d="M8 8l-4 4 4 4M16 8l4 4-4 4M13.5 5l-3 14"/>'),
    'winforms.html': _svg('<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><path d="M16.5 13v7M13 16.5h7"/>'),
    'inspector.html': _svg('<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>'),
    'web.html': _svg('<rect x="7" y="3" width="10" height="18" rx="2"/><path d="M11 18h2"/>'),
    'templates.html': _svg('<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><path d="M17.5 14v7M14 17.5h7"/>'),
    'report.html': _svg('<path d="M7 9V4h10v5"/><rect x="4" y="9" width="16" height="7" rx="1.5"/><path d="M7 14h10v6H7z"/>'),
    'publish.html': _svg('<path d="M12 3l8 4.5v9L12 21l-8-4.5v-9z"/><path d="M4 7.5l8 4.5 8-4.5M12 12v9"/>'),
}
CHEVRON = _svg('<path d="M9 6l6 6-6 6"/>')
CLOSE = _svg('<path d="M15 6l-6 6 6 6"/><path d="M20 6l-6 6 6 6"/>')


def _icon(f, title):
    if f in PAGE_ICONS:
        return PAGE_ICONS[f]
    if f.startswith('ctl-'):
        # The Toolbox's own icon for the tool (help_gen copies data/icons to icons/).
        return f'<img class="ti" src="icons/{f[4:-5]}.png" alt="" width="18" height="18">'
    if f.startswith('rust-'):
        return f'<span class="num">{int(f[5:7])}</span>'
    return _svg('<path d="M6 4h9l3 3v13H6z"/>')


def _link(f, title, current):
    cls = ' class="here" aria-current="page"' if f == current else ''
    shown = title
    if f.startswith('rust-'):
        shown = title.split('. ', 1)[-1]  # the number is the badge
    return f'<a href="{f}"{cls}>{_icon(f, title)}<span>{html.escape(shown)}</span></a>'


def nav(current):
    """The sidebar: the guide's groups as folding sections (Toolbox's groups folding inside it),
    a search box, and a button that puts the whole sidebar away. help.js remembers the folds."""
    out = ['<div class="nav-head"><a class="brand" href="index.html"><img src="icons/studio.png" alt="" width="26" height="26">คู่มือ Phai Studio</a>'
           f'<button class="nav-close" type="button" title="ปิดเมนู" aria-label="ปิดเมนู">{CLOSE}</button>'
           '<input class="nav-search" type="search" placeholder="ค้นหาในเมนู…" aria-label="ค้นหาในเมนู"></div>',
           '<div class="nav-body">']
    top_open = sub_open = None
    current_top = next(((g or '').partition('/')[0] for f, _, g in PAGES if f == current), '')
    for f, title, g in PAGES:
        top, _, sub = (g or '').partition('/')
        if top != top_open:
            if sub_open:
                out.append('</div></details>')
                sub_open = None
            if top_open:
                out.append('</div></details>')
            top_open = top or None
            if top:
                # Opened at first: Phai Studio's own pages; Toolbox and Rust Language folded until opened
                # (or until a page of theirs is the one shown) - as the owner wants the guide to open.
                fold = '' if top == 'Phai Studio' or top == current_top else ' data-folded="1"'
                attr = '' if fold else ' open'
                out.append(f'<details class="grp" data-key="{html.escape(top)}"{fold}{attr}><summary>{CHEVRON}<span>{html.escape(top)}</span></summary><div class="grp-body">')
        if sub != (sub_open or ''):
            if sub_open:
                out.append('</div></details>')
            sub_open = sub or None
            if sub:
                out.append(f'<details class="grp sub" data-key="{html.escape(g)}" open><summary>{CHEVRON}<span>{html.escape(sub)}</span></summary><div class="grp-body">')
        out.append(_link(f, title, current))
    if sub_open:
        out.append('</div></details>')
    if top_open:
        out.append('</div></details>')
    out.append('<p class="nav-none" hidden>ไม่พบหน้าที่ตรงกัน</p></div>')
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
<script>try{{if(JSON.parse(localStorage.getItem('phai-help-hidden'))&&innerWidth>860)document.documentElement.classList.add('nav-hidden')}}catch(e){{}}</script>
<script src="help.js" defer></script>
</head>
<body>
<div class="layout">
<nav id="nav">
{nav(f)}
</nav>
<div class="nav-shade"></div>
<main>
<button class="nav-open" type="button" title="เปิดเมนู" aria-label="เปิดเมนู"><svg class="ni" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16"/></svg></button>
{body.strip()}
{pager(i)}
</main>
</div>
</body>
</html>
'''
    open(os.path.join(OUT, f), 'w', encoding='utf-8', newline='\n').write(page)
    print('wrote', f)
