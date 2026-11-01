"""Archer item icons (plan/arcos-analise.md, HP 2026-11-01): the crossbow and every arrow.

Each arrow is the same shaft lying diagonally (fletching bottom-left, point top-right, like the
classic arrow) with a head and fletching of its own, so the kinds tell apart at a glance:
a flaming rag, an ice crystal, a golden star, a bone tip, a dripping venom head, a leaf...
Tipped arrows keep the steel head and carry their potion's colour in a glass bead behind it
and in the feathers. Used by items.py (REGISTRY) and runnable alone:

    python3 tools/tex64/archery_art.py      # writes the icons into the pack
"""

import os
import re
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from items import Sprite, dilate, disc_mask, poly_mask, stroke, wood_pal  # noqa: E402
from style import ramp  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HERE))

# Shaft from the nock (bottom-left) to where the head starts; the point is further on.
NOCK = (7.0, 57.0)
NECK = (44.5, 19.5)
DIR = np.array([1.0, -1.0]) / np.sqrt(2.0)
SIDE = np.array([1.0, 1.0]) / np.sqrt(2.0)  # across the shaft


def at(base, along, across=0.0):
    p = np.array(base) + DIR * along + SIDE * across
    return (float(p[0]), float(p[1]))


def shaft(s, pal=None, w=3.6):
    s.part(stroke([NOCK, NECK], w), pal if pal is not None else wood_pal(), base=0.5, bevel=1.5, lk=1.0, grain=0.12, shine=False, outline=1)


def fletching(s, pal, length=13.0, spread=6.5):
    # Two vanes, one each side of the shaft, swept back from the nock.
    for sgn in (-1, 1):
        pts = [at(NOCK, 2.0), at(NOCK, 2.0 + length * 0.35, sgn * spread), at(NOCK, 2.0 + length, sgn * spread * 0.55), at(NOCK, 2.0 + length, 0.0)]
        s.part(poly_mask(pts), pal, base=0.55, bevel=2.0, lk=1.2, grain=0.1, outline=1, shine=False)


def broadhead(s, pal, length=16.0, half=7.5, barb=3.0):
    tip = at(NECK, length)
    pts = [at(NECK, -barb, -half), tip, at(NECK, -barb, half), at(NECK, 1.0, 0.0)]
    s.part(poly_mask(pts), pal, base=0.6, bevel=2.5, lk=1.5)


def steel():
    return ramp("#b9c0c8", 10, 0.55, 0.08)


def base_arrow(s, feathers="#e6e1d6", head=None, shaft_pal=None):
    fletching(s, ramp(feathers, 9, 0.45, 0.1))
    shaft(s, shaft_pal)
    broadhead(s, head if head is not None else steel())


def plain():
    s = Sprite("arrow")
    base_arrow(s)
    return s.done()


def flaming():
    s = Sprite("flaming_arrow")
    base_arrow(s, feathers="#b43a20")
    # An oily rag wrapped behind the head, burning.
    s.part(stroke([at(NECK, -7.0), at(NECK, -1.0)], 6.0), ramp("#5a3a24", 8, 0.35, 0.1), base=0.45, bevel=1.5, lk=0.8, grain=0.3, outline=1, shine=False)
    for k, (a, h) in enumerate([(-7.0, 12.0), (-3.5, 16.0), (0.0, 11.0)]):
        root = at(NECK, a, -1.0)
        top = (root[0] - 2.0 + k, root[1] - h)
        s.part(stroke([root, top], 6.0, 1.5), ramp("#f08a20", 9, 0.55, 0.2), base=0.65, bevel=1.5, lk=1.4, outline=1, shine=False)
    return s.done()


def frostburn():
    s = Sprite("frostburn_arrow")
    fletching(s, ramp("#dff2ff", 9, 0.4, 0.05))
    shaft(s, ramp("#7d8fa6", 9, 0.4, 0.05))
    # A long six-sided ice crystal for a head, with a smaller one beside it.
    tip = at(NECK, 15.0)
    pts = [at(NECK, -1.0), at(NECK, 3.0, -4.0), at(NECK, 11.0, -3.0), tip, at(NECK, 11.0, 3.0), at(NECK, 3.0, 4.0)]
    s.part(poly_mask(pts), ramp("#8fdcff", 10, 0.6, 0.0), base=0.62, bevel=2.5, lk=1.8)
    small = [at(NECK, 1.0, -3.0), at(NECK, 3.0, -9.0), at(NECK, 5.0, -3.0)]
    s.part(poly_mask(small), ramp("#bfeaff", 9, 0.5, 0.0), base=0.6, bevel=1.5, lk=1.5, outline=1)
    return s.done()


def unholy():
    s = Sprite("unholy_arrow")
    fletching(s, ramp("#3c2448", 9, 0.4, 0.1))
    shaft(s, ramp("#4a3a44", 9, 0.4, 0.1))
    # A jagged, hooked head of dark violet bone.
    pts = [at(NECK, -2.0, -6.5), at(NECK, 4.0, -3.0), at(NECK, 6.0, -6.0), at(NECK, 15.0), at(NECK, 6.0, 6.0), at(NECK, 4.0, 3.0), at(NECK, -2.0, 6.5), at(NECK, 1.0)]
    s.part(poly_mask(pts), ramp("#9a52d8", 10, 0.6, 0.1), base=0.55, bevel=2.5, lk=1.5)
    return s.done()


def star(cx, cy, r0, r1, n=5, rot=-np.pi / 2):
    pts = []
    for i in range(n * 2):
        r = r1 if i % 2 == 0 else r0
        a = rot + i * np.pi / n
        pts.append((cx + np.cos(a) * r, cy + np.sin(a) * r))
    return poly_mask(pts)


def jester():
    s = Sprite("jester_arrow")
    # Motley feathers: one vane red, one blue.
    for sgn, col in ((-1, "#d8304a"), (1, "#3060d8")):
        pts = [at(NOCK, 2.0), at(NOCK, 6.5, sgn * 6.5), at(NOCK, 15.0, sgn * 3.6), at(NOCK, 15.0, 0.0)]
        s.part(poly_mask(pts), ramp(col, 9, 0.45, 0.1), base=0.55, bevel=2.0, lk=1.2, outline=1, shine=False)
    shaft(s, ramp("#e8d8a8", 9, 0.4, 0.1))
    c = at(NECK, 6.0)
    s.part(star(c[0], c[1], 4.5, 10.5), ramp("#f2c53a", 10, 0.6, 0.15), base=0.65, bevel=2.5, lk=1.8)
    return s.done()


def hellfire():
    s = Sprite("hellfire_arrow")
    fletching(s, ramp("#3a1a14", 9, 0.35, 0.1))
    shaft(s, ramp("#3c2c26", 9, 0.35, 0.05))
    # A red-hot head, glowing from inside, with embers behind.
    broadhead(s, ramp("#ff5a1a", 10, 0.7, 0.25), length=14.0, half=7.0)
    for a, o in ((-4.0, -5.0), (-8.0, 4.0), (-12.0, -3.0)):
        c = at(NECK, a, o)
        s.part(disc_mask(c[0], c[1], 1.8), ramp("#ffb030", 8, 0.4, 0.2), base=0.7, bevel=1.0, lk=1.0, outline=1, shine=False)
    return s.done()


def holy():
    s = Sprite("holy_arrow")
    # White wings for fletching.
    for sgn in (-1, 1):
        pts = [at(NOCK, 1.0), at(NOCK, 4.0, sgn * 9.0), at(NOCK, 10.0, sgn * 8.0), at(NOCK, 17.0, sgn * 3.0), at(NOCK, 17.0, 0.0)]
        s.part(poly_mask(pts), ramp("#f4f4ff", 9, 0.35, 0.05), base=0.62, bevel=2.0, lk=1.2, outline=1, shine=False)
    shaft(s, ramp("#e8dcc0", 9, 0.4, 0.1))
    broadhead(s, ramp("#f2c53a", 10, 0.6, 0.15), length=12.0, half=5.5)
    c = at(NECK, 0.5)
    s.part(star(c[0], c[1], 1.8, 4.0, n=4, rot=0.0), ramp("#fff6c0", 8, 0.3, 0.05), base=0.75, bevel=1.0, lk=0.8, outline=1, shine=False)
    return s.done()


def spectral():
    s = Sprite("spectral_arrow")
    fletching(s, ramp("#f0e090", 9, 0.4, 0.1))
    shaft(s, ramp("#d8b860", 9, 0.4, 0.15))
    # Glowstone head: a faceted gold point with lit specks.
    broadhead(s, ramp("#f8d850", 10, 0.6, 0.2), length=12.0, half=6.0)
    for a in (-5.0, -10.0, -15.0):
        c = at(NECK, a, 0.0)
        s.part(disc_mask(c[0], c[1], 1.6), ramp("#fff0a0", 8, 0.3, 0.1), base=0.75, bevel=1.0, lk=0.6, outline=1, shine=False)
    return s.done()


def bone():
    s = Sprite("bone_arrow")
    fletching(s, ramp("#9a8c78", 9, 0.4, 0.1))
    shaft(s, ramp("#e8e0cc", 9, 0.35, 0.08))
    # A splinter of bone, knobbly at the neck.
    pts = [at(NECK, -1.0, -3.5), at(NECK, 14.0), at(NECK, -1.0, 3.5)]
    s.part(poly_mask(pts), ramp("#f0ead8", 9, 0.45, 0.08), base=0.62, bevel=2.0, lk=1.3)
    c = at(NECK, -1.5)
    s.part(disc_mask(c[0], c[1], 4.0), ramp("#e0d8c0", 9, 0.4, 0.08), base=0.58, bevel=2.0, lk=1.2)
    return s.done()


def venom():
    s = Sprite("venom_arrow")
    fletching(s, ramp("#4a7a30", 9, 0.4, 0.1))
    shaft(s)
    broadhead(s, ramp("#7a4ab8", 10, 0.55, 0.1), length=12.0)
    for a, d in ((2.0, 7.0), (-2.0, 10.0)):
        p = at(NECK, a, 2.0)
        s.part(stroke([p, (p[0], p[1] + d)], 2.6, 3.6), ramp("#7ad040", 9, 0.5, 0.15), base=0.62, bevel=1.5, lk=1.4, outline=1, shine=False)
    return s.done()


def cursed():
    s = Sprite("cursed_arrow")
    fletching(s, ramp("#3a4a2a", 9, 0.35, 0.1))
    shaft(s, ramp("#4a4038", 9, 0.35, 0.05))
    broadhead(s, steel())
    for k, (a, h) in enumerate([(-5.0, 10.0), (-1.5, 13.0), (2.0, 8.0)]):
        root = at(NECK, a, -1.0)
        s.part(stroke([root, (root[0] - 1.0 + k, root[1] - h)], 4.0, 1.0), ramp("#9cf040", 9, 0.55, 0.15), base=0.6, bevel=1.5, lk=1.4, outline=1, shine=False)
    return s.done()


def ichor():
    s = Sprite("ichor_arrow")
    fletching(s, ramp("#c89a30", 9, 0.4, 0.15))
    shaft(s)
    # A golden drop for a head.
    tip = at(NECK, 15.0)
    c = at(NECK, 4.5)
    s.part(disc_mask(c[0], c[1], 6.0) | poly_mask([at(NECK, 4.5, -5.6), tip, at(NECK, 4.5, 5.6)]), ramp("#ffc020", 10, 0.6, 0.2), base=0.62, bevel=3.0, lk=1.8)
    return s.done()


def verdite():
    s = Sprite("verdite_arrow")
    fletching(s, ramp("#2a8a3a", 9, 0.45, 0.1))
    shaft(s, ramp("#6a5a34", 9, 0.4, 0.1))
    # A leaf-shaped head of living crystal, with a vein.
    tip = at(NECK, 16.0)
    pts = [at(NECK, -1.0)] + [at(NECK, t, -6.5 * np.sin(np.pi * (t + 1) / 17.0)) for t in np.linspace(0, 15, 8)] + [tip] + [at(NECK, t, 6.5 * np.sin(np.pi * (t + 1) / 17.0)) for t in np.linspace(15, 0, 8)]
    s.part(poly_mask(pts), ramp("#40d060", 10, 0.6, 0.1), base=0.58, bevel=3.0, lk=1.6)
    s.part(stroke([at(NECK, 0.0), at(NECK, 13.0)], 1.2), ramp("#a0f0a0", 6, 0.2, 0.05), base=0.7, bevel=0.5, lk=0.5, outline=0, shine=False)
    return s.done()


def lumite():
    s = Sprite("lumite_arrow")
    fletching(s, ramp("#4ad8c8", 9, 0.5, 0.05), length=15.0)
    shaft(s, ramp("#2a5a64", 9, 0.4, 0.05))
    # A long, slim, glowing point.
    pts = [at(NECK, -3.0, -4.5), at(NECK, 18.0), at(NECK, -3.0, 4.5), at(NECK, 0.0)]
    s.part(poly_mask(pts), ramp("#60f0e0", 10, 0.6, 0.0), base=0.68, bevel=2.5, lk=1.9)
    return s.done()


def effect_colors():
    """Potion colours, read from effects.rs like gen_fase6_art.py does."""
    text = open(os.path.join(ROOT, "crates", "game", "src", "effects.rs")).read()
    consts = dict((n, int(v)) for n, v in re.findall(r"pub const ([A-Z_]+): u8 = (\d+);", text))
    colors = {}
    for name, rgb in re.findall(r"([A-Z_]+) => \(\"[^\"]+\", \[(\d+, \d+, \d+)\]", text):
        if name in consts:
            colors[consts[name]] = [int(v) for v in rgb.split(",")]
    out = {}
    for key, body in re.findall(r'p!\("([a-z_]+)", "[^"]+", \[([^\]]*)\]\)', text):
        effs = [consts[e] for e, _ in re.findall(r"\(([A-Z_]+), (\d+), [\d.]+\)", body)]
        c = np.mean([colors[e] for e in effs], axis=0) if effs else np.array([80, 110, 200])
        out[key] = c
    return out


def tipped(key, color):
    s = Sprite(f"tipped_arrow_{key}")
    c = np.clip(np.asarray(color, dtype=float), 30, 255)
    fletching(s, ramp(c, 9, 0.45, 0.05))
    shaft(s)
    broadhead(s, steel())
    # The dipped part behind the head: a coat of the potion, glossy.
    s.part(stroke([at(NECK, -8.0), at(NECK, 0.0)], 5.5), ramp(c, 10, 0.55, 0.05), base=0.6, bevel=1.5, lk=1.6, outline=1)
    return s.done()


def crossbow():
    s = Sprite("crossbow")
    wood = ramp("#7a5230", 10, 0.45, 0.12)
    # The stock, from the butt (bottom-left) to the nose (top-right), with a trigger under it.
    s.part(stroke([(8.0, 57.0), (17.0, 47.0), (46.0, 18.0)], 7.5, 5.5), wood, base=0.5, bevel=2.0, lk=1.2, grain=0.25, shine=False)
    s.part(stroke([(20.0, 46.0), (24.0, 53.0)], 3.0), ramp("#4a4a50", 8, 0.4, 0.05), base=0.5, bevel=1.0, lk=1.0, outline=1, shine=False)
    # The prod across the nose, bent forward, with iron tips.
    a, b = (26.0, 6.0), (58.0, 38.0)
    mid = (45.0, 18.0)
    bend = (49.0, 13.0)
    s.part(stroke([a, ((a[0] + bend[0]) / 2 + 1.0, (a[1] + bend[1]) / 2 - 2.0), bend, ((b[0] + bend[0]) / 2 + 2.0, (b[1] + bend[1]) / 2 - 1.0), b], 5.0, 5.0), ramp("#5a3c22", 10, 0.45, 0.12), base=0.5, bevel=2.0, lk=1.3, grain=0.2, shine=False)
    for p in (a, b):
        s.part(disc_mask(p[0], p[1], 3.0), ramp("#9aa0a8", 9, 0.5, 0.05), base=0.6, bevel=1.5, lk=1.4, outline=1)
    # The string, drawn back to the latch.
    latch = (31.0, 33.0)
    for p in (a, b):
        s.part(stroke([p, latch], 1.6), ramp("#e6e1d6", 6, 0.25, 0.05), base=0.7, bevel=0.5, lk=0.5, outline=1, shine=False)
    # Iron bands on the stock.
    for p in ((38.0, 26.0), (22.0, 42.0)):
        s.part(stroke([(p[0] - 2.5, p[1] - 2.5), (p[0] + 2.5, p[1] + 2.5)], 2.6), ramp("#8a9098", 8, 0.45, 0.05), base=0.55, bevel=1.0, lk=1.2, outline=1, shine=False)
    del mid
    return s.done()


def quiver(name, body, band, feathers, gem=None):
    s = Sprite(name)
    # Feathers of the arrows standing out of the mouth (top-right), then the tube.
    for k, off in enumerate((-6.0, 0.0, 6.0)):
        base = (40.0 + off * 0.7, 20.0 + off * 0.7)
        tip = (base[0] + 9.0, base[1] - 11.0)
        s.part(stroke([base, tip], 5.0, 3.0), ramp(feathers[k % len(feathers)], 9, 0.45, 0.1), base=0.58, bevel=1.5, lk=1.2, outline=1, shine=False)
    tube = stroke([(16.0, 52.0), (42.0, 22.0)], 17.0, 19.0)
    s.part(tube, ramp(body, 10, 0.45, 0.12), base=0.5, bevel=3.0, lk=1.3, grain=0.2)
    for p0 in ((22.0, 45.0), (36.0, 29.0)):
        s.part(stroke([(p0[0] - 7.0, p0[1] - 7.0), (p0[0] + 7.0, p0[1] + 7.0)], 3.4), ramp(band, 9, 0.5, 0.15), base=0.6, bevel=1.0, lk=1.4, outline=1)
    if gem is not None:
        s.part(disc_mask(29.0, 37.0, 3.5), ramp(gem, 9, 0.55, 0.1), base=0.68, bevel=2.0, lk=1.8, outline=1)
    return s.done()


def scope():
    s = Sprite("ranger_scope")
    brass = ramp("#c89a40", 10, 0.55, 0.15)
    s.part(stroke([(12.0, 50.0), (48.0, 16.0)], 12.0, 15.0), brass, base=0.55, bevel=3.0, lk=1.6)
    for p0 in ((20.0, 42.0), (40.0, 24.0)):
        s.part(stroke([(p0[0] - 8.0, p0[1] - 8.0), (p0[0] + 8.0, p0[1] + 8.0)], 3.0), ramp("#5a4020", 8, 0.4, 0.1), base=0.5, bevel=1.0, lk=1.0, outline=1, shine=False)
    s.part(disc_mask(49.0, 15.0, 6.5), ramp("#6ad0f0", 9, 0.6, 0.0), base=0.62, bevel=2.5, lk=2.0, outline=2)
    return s.done()


ITEMS = {
    "arrow": plain,
    "flaming_arrow": flaming,
    "frostburn_arrow": frostburn,
    "unholy_arrow": unholy,
    "jester_arrow": jester,
    "hellfire_arrow": hellfire,
    "holy_arrow": holy,
    "spectral_arrow": spectral,
    "bone_arrow": bone,
    "venom_arrow": venom,
    "cursed_arrow": cursed,
    "ichor_arrow": ichor,
    "verdite_arrow": verdite,
    "lumite_arrow": lumite,
    "crossbow": crossbow,
    "molten_quiver": lambda: quiver("molten_quiver", "#4a1c14", "#ff8a28", ["#ff9a30", "#ffd060"], gem="#ff5a10"),
    "magic_quiver": lambda: quiver("magic_quiver", "#2a2a6a", "#d8c060", ["#a0c0ff", "#f0f0ff"], gem="#c080ff"),
    "ranger_scope": scope,
}
try:
    for _k, _c in effect_colors().items():
        ITEMS[f"tipped_arrow_{_k}"] = (lambda k=_k, c=_c: tipped(k, c))
except OSError:
    pass


def main(names=None):
    from PIL import Image

    from pixel import pixelate
    from build import save

    out = os.path.join(ROOT, "packs", "vanilla", "textures", "item")
    src = os.path.join(HERE, "src16", "item")
    for name, fn in ITEMS.items():
        if names and name not in names:
            continue
        img = pixelate(fn())
        save(img, os.path.join(out, name + ".png"))
        # A small design in src16 so build.py keeps making it (through items.REGISTRY).
        dst = os.path.join(src, name + ".png")
        if not os.path.exists(dst):
            Image.fromarray(img.astype(np.uint8)).resize((16, 16), Image.NEAREST).save(dst)
    print(f"{len(ITEMS)} archer icons")


if __name__ == "__main__":
    main(set(sys.argv[1:]) or None)
