"""Draws phf.ico, the icon of a Phai form file (.phf) in Explorer: a page with its corner folded, in
Phai Studio's greens, holding a small window - its title bar, a box and a button - and a bamboo
leaf from Phai Studio's mark. The small sizes are drawn simpler, so they stay clear.

    python apps/studio/art/make_phf_icon.py      ->  apps/studio/phf.ico
"""
import io
import os
import struct

from PIL import Image, ImageDraw

DEEP = (11, 94, 66, 255)      # #0b5e42, the mark's ring and brackets
LEAF = (23, 138, 95, 255)     # #178a5f
PALE = (198, 236, 217, 255)   # #c6ecd9, the culm's joints
WHITE = (255, 255, 255, 255)
LINE = (160, 178, 170, 255)


def page(d, s, k):
    """The page, its folded corner, in a canvas `s` wide; `k` the line width."""
    left, top, right, bottom = s * 0.16, s * 0.06, s * 0.84, s * 0.94
    fold = s * 0.22
    outline = [(left, top), (right - fold, top), (right, top + fold), (right, bottom), (left, bottom)]
    d.polygon(outline, fill=WHITE)
    d.line(outline + [outline[0]], fill=DEEP, width=k, joint="curve")
    d.polygon([(right - fold, top), (right - fold, top + fold), (right, top + fold)], fill=PALE)
    d.line([(right - fold, top), (right - fold, top + fold), (right, top + fold)], fill=DEEP, width=k, joint="curve")
    return left, top, right, bottom


def full(s):
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    k = max(1, round(s * 0.022))
    left, top, right, bottom = page(d, s, k)
    # The window: a title bar and its body.
    wl, wt, wr, wb = left + s * 0.1, top + s * 0.34, right - s * 0.1, bottom - s * 0.1
    d.rounded_rectangle([wl, wt, wr, wb], radius=s * 0.03, fill=WHITE, outline=DEEP, width=k)
    d.rounded_rectangle([wl, wt, wr, wt + s * 0.11], radius=s * 0.03, fill=DEEP)
    d.rectangle([wl, wt + s * 0.06, wr, wt + s * 0.11], fill=DEEP)
    # A box and a button in it.
    bx = wl + s * 0.06
    d.rounded_rectangle([bx, wt + s * 0.18, wr - s * 0.06, wt + s * 0.26], radius=s * 0.012, outline=LINE, width=max(1, k // 2))
    d.rounded_rectangle([wr - s * 0.24, wb - s * 0.12, wr - s * 0.06, wb - s * 0.05], radius=s * 0.015, fill=LEAF)
    # The bamboo leaves above the window: the mark's own two (logo.svg), its 64 units scaled.
    ox, oy, u = left + s * 0.02, top + s * 0.02, s / 64 * 1.25
    at = lambda pts: [(ox + (x - 36) * u, oy + (y - 8) * u) for x, y in pts]
    d.polygon(at(curve((38, 17), [((42, 11.5), (47, 9), (53, 9.5)), ((49, 14.5), (44, 17), (38, 17))])), fill=LEAF)
    d.polygon(at(curve((38, 17), [((42.5, 16.5), (47, 17.5), (50.5, 20)), ((46, 21.5), (41.5, 20.5), (38, 17))])), fill=DEEP)
    return img


def curve(start, segments, steps=24):
    """The points along cubic Bezier segments from `start`, as an SVG path's C commands."""
    pts, p0 = [start], start
    for c1, c2, p3 in segments:
        for i in range(1, steps + 1):
            t = i / steps
            a, b, c, e = (1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t * t, t ** 3
            pts.append((a * p0[0] + b * c1[0] + c * c2[0] + e * p3[0], a * p0[1] + b * c1[1] + c * c2[1] + e * p3[1]))
        p0 = p3
    return pts


def small(s):
    """16 and 24 pixels: the page and the window's title bar and button, nothing finer."""
    big = 8 * s
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    k = round(big * 0.07)
    left, top, right, bottom = page(d, big, k)
    wl, wt, wr, wb = left + big * 0.12, top + big * 0.3, right - big * 0.12, bottom - big * 0.12
    d.rectangle([wl, wt, wr, wt + big * 0.16], fill=DEEP)
    d.rectangle([wr - big * 0.28, wb - big * 0.16, wr, wb], fill=LEAF)
    return img.resize((s, s), Image.LANCZOS)


def render(s):
    if s <= 24:
        return small(s)
    return full(4 * s).resize((s, s), Image.LANCZOS)


def write_ico(path, images):
    """An .ico of PNG images, one per size - what Windows Vista and later read."""
    blobs = []
    for im in images:
        b = io.BytesIO()
        im.save(b, "PNG")
        blobs.append(b.getvalue())
    out = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    for im, data in zip(images, blobs):
        w, h = im.size
        out += struct.pack("<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(data), offset)
        offset += len(data)
    with open(path, "wb") as f:
        f.write(out + b"".join(blobs))


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    sizes = [16, 24, 32, 48, 64, 256]
    write_ico(os.path.join(here, "..", "phf.ico"), [render(s) for s in sizes])
    sheet = Image.new("RGBA", (sum(sizes) + 10 * len(sizes), 256), (240, 240, 240, 255))
    x = 0
    for s in sizes:
        sheet.alpha_composite(render(s), (x, 256 - s))
        x += s + 10
    sheet.save(os.path.join(os.environ.get("PHF_SHEET_DIR", here), "phf-sheet.png"))
