"""Ore textures (plan/minerios-analise.md, lote 2): one drawing per material, not one blob
recoloured.

Every ore is the real host block texture (stone, deepslate, netherrack, holystone...) with the
ore's deposits drawn on top as chunky pixel art: drawn on a 32x32 grid and grown to 64x64, so
each art pixel is a crisp 2x2 square like the items. Each deposit has a dark outline and a
drop shadow, so it sits proud of the rock. Light comes from the top-left like everything else.

How each material grows decides its shape (real mineral habits, simplified):
coal in flat matte plates, iron in rounded nodules with rust, copper in branching wires with
patina, tin in dark stubby prisms, lead in galena cubes, silver in curly wires, tungsten in
long blades, gold in nuggets along a quartz vein, platinum in pale grains, redstone in shards,
lapis in patches with pyrite, diamond in faceted octahedra, emerald in hexagonal columns,
ruby in flat hexagons, sapphire in spindles, topaz in pencil prisms, amethyst in spiky
clusters; the Terraria hard-mode metals, the evil and hell ores and the mod ores each have
their own too. The terrain shader makes the metals shine and the gems sparkle (material.rs).

    python3 tools/tex64/build.py --only <names>      (any ore name below)
"""

import os

import numpy as np
from PIL import Image

from style import hexrgb, light, quantize, ramp, rng_for

M = 32  # the art grid
HERE = os.path.dirname(os.path.abspath(__file__))
PACK = os.path.join(os.path.dirname(os.path.dirname(HERE)), "packs", "vanilla", "textures", "block")

YY, XX = np.mgrid[0:M, 0:M].astype(np.float64)
# Where deposits go (art pixels), jittered per texture; spread so the block reads at a glance.
LAYOUT = [(5, 5), (20, 4), (13, 15), (26, 18), (5, 23), (18, 26), (28, 29)]


def bg(name):
    """The host block's own 64x64 texture, so an ore matches the rock around it."""
    p = os.path.join(PACK, name + ".png")
    return np.asarray(Image.open(p).convert("RGBA"), dtype=np.float64).copy()


def tor(cx, cy):
    dx = (XX - cx + M / 2) % M - M / 2
    dy = (YY - cy + M / 2) % M - M / 2
    return dx, dy


class Canvas:
    """The ore layer on the art grid: where the mineral is (m), its height for lighting (h),
    its tone on the material's ramp (t), pixels with fixed shading (flat: facets), and accent
    pixels painted in another colour (acc: index into the accent list, -1 none)."""

    def __init__(self):
        self.m = np.zeros((M, M), bool)
        self.h = np.zeros((M, M))
        self.t = np.full((M, M), 0.5)
        self.flat = np.zeros((M, M), bool)
        self.acc = np.full((M, M), -1, int)
        self.noline = np.zeros((M, M), bool)

    def put(self, mask, t, h=None, flat=False):
        self.m |= mask
        self.t = np.where(mask, t, self.t)
        if h is not None:
            self.h = np.where(mask, np.maximum(self.h, h), self.h)
        self.flat = np.where(mask, flat, self.flat)

    def ellipse(self, cx, cy, rx, ry, ang=0.0, t=0.55, dome=2.0):
        dx, dy = tor(cx, cy)
        c, s = np.cos(ang), np.sin(ang)
        u, v = (dx * c + dy * s) / rx, (-dx * s + dy * c) / ry
        d = u * u + v * v
        mask = d <= 1.0
        self.put(mask, t, np.sqrt(np.clip(1 - d, 0, 1)) * dome)
        return mask

    def poly(self, pts, t=0.55, h=None, flat=False):
        mask = poly_mask(pts)
        self.put(mask, t, h if h is not None else np.where(mask, 1.0, 0.0), flat)
        return mask

    def line(self, x0, y0, x1, y1, w=1.0, t=0.55, dome=1.0):
        d = seg_dist(x0, y0, x1, y1)
        mask = d <= w / 2 + 0.01
        self.put(mask, t, np.clip(1 - d / (w / 2 + 0.5), 0, 1) * dome)
        return mask

    def px(self, x, y, t=0.6, h=1.0, flat=False):
        mask = np.zeros((M, M), bool)
        mask[int(y) % M, int(x) % M] = True
        self.put(mask, t, h, flat)
        return mask

    def accent(self, mask, i):
        self.acc[mask & self.m] = i


def poly_mask(pts):
    """Filled polygon on the torus (art pixel centres)."""
    out = np.zeros((M, M), bool)
    pts = np.asarray(pts, float)
    for ox in (-M, 0, M):
        for oy in (-M, 0, M):
            px, py = XX + 0.5 - ox, YY + 0.5 - oy
            inside = np.zeros((M, M), bool)
            n = len(pts)
            for i in range(n):
                x0, y0 = pts[i]
                x1, y1 = pts[(i + 1) % n]
                cond = (y0 > py) != (y1 > py)
                xi = x0 + (py - y0) * (x1 - x0) / ((y1 - y0) + 1e-9)
                inside ^= cond & (px < xi)
            out |= inside
    return out


def seg_dist(x0, y0, x1, y1):
    best = np.full((M, M), 1e9)
    for ox in (-M, 0, M):
        for oy in (-M, 0, M):
            px, py = XX + 0.5 - ox, YY + 0.5 - oy
            vx, vy = x1 - x0, y1 - y0
            L2 = vx * vx + vy * vy + 1e-9
            t = np.clip(((px - x0) * vx + (py - y0) * vy) / L2, 0, 1)
            best = np.minimum(best, np.hypot(px - (x0 + t * vx), py - (y0 + t * vy)))
    return best


def rot(pts, cx, cy, a):
    c, s = np.cos(a), np.sin(a)
    return [(cx + x * c - y * s, cy + x * s + y * c) for x, y in pts]


# ================================================================ habits (one deposit each)


def plates(c, r, x, y):
    """Coal: flat angular plates, a couple stacked."""
    for k in range(int(r.integers(2, 4))):
        n = int(r.integers(4, 6))
        a0 = r.uniform(0, np.pi)
        rad = r.uniform(2.4, 3.6)
        pts = [(x + k * r.uniform(-1.5, 1.5) + np.cos(a0 + i * 2 * np.pi / n) * rad * r.uniform(0.7, 1.2),
                y + k * r.uniform(-1, 1) + np.sin(a0 + i * 2 * np.pi / n) * rad * 0.7 * r.uniform(0.7, 1.2)) for i in range(n)]
        m = c.poly(pts, t=0.35 + 0.1 * k)
        # A dull bevel on the upper edge of each plate.
        top = m & ~np.roll(m, 1, 0)
        c.put(top, 0.95, flat=True)


def nodules(c, r, x, y):
    """Iron: rounded nodules with rusty skins."""
    for _ in range(int(r.integers(2, 4))):
        ox, oy = r.uniform(-2.5, 2.5, 2)
        rr = r.uniform(1.6, 2.5)
        m = c.ellipse(x + ox, y + oy, rr, rr * r.uniform(0.8, 1.1), t=0.55, dome=2.2)
        c.accent(m & (r.random((M, M)) < 0.22), 0)


def dendrite(c, r, x, y):
    """Copper: native copper branches, green patina at the tips."""
    def grow(px_, py_, a, n, depth):
        for i in range(n):
            nx_, ny_ = px_ + np.cos(a), py_ + np.sin(a)
            c.line(px_, py_, nx_, ny_, 1.4 if depth == 0 else 1.0, t=0.6, dome=1.2)
            px_, py_ = nx_, ny_
            a += r.uniform(-0.5, 0.5)
            if depth < 2 and r.random() < 0.3:
                grow(px_, py_, a + r.choice([-1, 1]) * r.uniform(0.6, 1.1), int(n * 0.6), depth + 1)
        tip = c.ellipse(px_, py_, 0.8, 0.8, t=0.6)
        c.accent(tip, 0)
    a = r.uniform(0, 2 * np.pi)
    grow(x, y, a, int(r.integers(4, 7)), 0)
    grow(x, y, a + np.pi + r.uniform(-0.4, 0.4), int(r.integers(3, 5)), 0)


def prisms(c, r, x, y, n=(2, 4), w=(1.6, 2.4), ln=(2.5, 4.0), tip=1.2):
    """Short pointed prisms (tin's cassiterite, topaz's pencils with longer ones)."""
    for _ in range(int(r.integers(*n))):
        a = r.uniform(-np.pi, np.pi)
        L = r.uniform(*ln)
        W = r.uniform(*w) / 2
        cx, cy = x + r.uniform(-2, 2), y + r.uniform(-2, 2)
        pts = rot([(-L, -W), (L - tip, -W), (L, 0), (L - tip, W), (-L, W)], cx, cy, a)
        m = poly_mask(pts)
        # Two faces: the one towards the light is bright.
        side = (XX - cx) * np.cos(a + np.pi / 2) + (YY - cy) * np.sin(a + np.pi / 2)
        lit_side = side * (np.cos(a + np.pi / 2) * -0.7 + np.sin(a + np.pi / 2) * -0.7) > 0
        c.put(m & lit_side, 0.78, 1.0, flat=True)
        c.put(m & ~lit_side, 0.42, 1.0, flat=True)


def cubes(c, r, x, y, big=(2, 4)):
    """Lead (galena) and cobalt: little cubes with a lit top and side."""
    for _ in range(int(r.integers(2, 4))):
        s = int(r.integers(*big))
        cx, cy = int(x + r.integers(-3, 3)), int(y + r.integers(-3, 3))
        face = np.zeros((M, M), bool)
        for yy in range(s):
            for xx in range(s):
                face[(cy + yy) % M, (cx + xx) % M] = True
        top = np.zeros((M, M), bool)
        for xx in range(s):
            top[(cy - 1) % M, (cx + xx + 1) % M] = True
        right = np.zeros((M, M), bool)
        for yy in range(s):
            right[(cy + yy) % M, (cx + s) % M] = True
        c.put(face, 0.55, 1.0, flat=True)
        c.put(top & ~face, 0.85, 1.0, flat=True)
        c.put(right & ~face, 0.3, 1.0, flat=True)
        c.px(cx, cy, 0.95, flat=True)


def wires(c, r, x, y):
    """Silver: curly native wires."""
    for _ in range(int(r.integers(2, 4))):
        a = r.uniform(0, 2 * np.pi)
        px_, py_ = x + r.uniform(-2, 2), y + r.uniform(-2, 2)
        turn = r.uniform(-0.6, 0.6)
        for _ in range(int(r.integers(5, 9))):
            nx_, ny_ = px_ + np.cos(a), py_ + np.sin(a)
            c.line(px_, py_, nx_, ny_, 1.1, t=0.7, dome=1.0)
            px_, py_ = nx_, ny_
            a += turn + r.uniform(-0.2, 0.2)


def blades(c, r, x, y):
    """Tungsten (wolframite): long thin tabular blades, crossed."""
    a0 = r.uniform(0, np.pi)
    for k in range(int(r.integers(2, 4))):
        a = a0 + r.uniform(-0.5, 0.5) + k * 0.25
        L = r.uniform(3.0, 4.6)
        cx, cy = x + r.uniform(-1.5, 1.5), y + r.uniform(-1.5, 1.5)
        m = c.line(cx - np.cos(a) * L, cy - np.sin(a) * L, cx + np.cos(a) * L, cy + np.sin(a) * L, 1.6, t=0.45, dome=0.5)
        edge = m & ~np.roll(np.roll(m, 1, 0), 1, 1)
        c.put(edge, 0.85, flat=True)


def quartz_vein_gold(c, r, x, y):
    """Gold: nuggets in a short milky quartz vein."""
    a = r.uniform(-0.6, 0.6) + (np.pi / 2 if r.random() < 0.3 else 0)
    L = r.uniform(3, 5)
    q = c.line(x - np.cos(a) * L, y - np.sin(a) * L, x + np.cos(a) * L, y + np.sin(a) * L, 1.6, t=0.5, dome=0.6)
    c.accent(q, 0)
    for _ in range(int(r.integers(2, 4))):
        t = r.uniform(-1, 1)
        nx_, ny_ = x + np.cos(a) * L * t, y + np.sin(a) * L * t
        m = c.ellipse(nx_ + r.uniform(-0.6, 0.6), ny_ + r.uniform(-0.6, 0.6), r.uniform(1.3, 2.0), r.uniform(1.1, 1.7), r.uniform(0, 3), t=0.55, dome=2.0)
        c.acc[m] = -1


def grains(c, r, x, y, n=(4, 7), size=(0.9, 1.5)):
    """Platinum, nether gold: loose rounded grains."""
    for _ in range(int(r.integers(*n))):
        rr = r.uniform(*size)
        c.ellipse(x + r.uniform(-3, 3), y + r.uniform(-3, 3), rr, rr, t=0.6, dome=1.5)


def shards(c, r, x, y, n=(2, 4), L=(2.4, 3.8)):
    """Redstone, adamantite: sharp triangular shards."""
    for _ in range(int(r.integers(*n))):
        a = r.uniform(0, 2 * np.pi)
        l = r.uniform(*L)
        cx, cy = x + r.uniform(-2, 2), y + r.uniform(-2, 2)
        pts = rot([(-l * 0.6, -l * 0.45), (l, 0), (-l * 0.6, l * 0.45)], cx, cy, a)
        m = poly_mask(pts)
        up = (YY - cy) * np.cos(a) - (XX - cx) * np.sin(a) < 0
        c.put(m & up, 0.82, 1.0, flat=True)
        c.put(m & ~up, 0.45, 1.0, flat=True)


def patches(c, r, x, y):
    """Lapis: soft blue patches with pyrite (gold) and calcite (white) specks."""
    for _ in range(int(r.integers(2, 4))):
        m = c.ellipse(x + r.uniform(-2.5, 2.5), y + r.uniform(-2.5, 2.5), r.uniform(1.4, 2.6), r.uniform(1.0, 2.0), r.uniform(0, 3), t=0.5, dome=1.2)
        rnd = r.random((M, M))
        c.accent(m & (rnd < 0.10), 0)
        c.accent(m & (rnd > 0.94), 1)


def octahedra(c, r, x, y):
    """Diamond: octahedra seen tip-on, four facets each, crisp edges."""
    for k in range(int(r.integers(1, 3))):
        s = r.uniform(3.0, 4.0) if k == 0 else r.uniform(1.8, 2.4)
        cx, cy = x + (0 if k == 0 else r.uniform(-3, 3)), y + (0 if k == 0 else r.uniform(-3, 3))
        tl = poly_mask([(cx - s, cy), (cx, cy - s), (cx, cy)])
        tr = poly_mask([(cx, cy - s), (cx + s, cy), (cx, cy)])
        bl = poly_mask([(cx - s, cy), (cx, cy), (cx, cy + s)])
        br = poly_mask([(cx, cy), (cx + s, cy), (cx, cy + s)])
        c.put(tl, 0.9, 1.0, flat=True)
        c.put(tr, 0.62, 1.0, flat=True)
        c.put(bl, 0.48, 1.0, flat=True)
        c.put(br, 0.25, 1.0, flat=True)
        c.px(cx - s * 0.45, cy - s * 0.2, 1.0, flat=True)


def columns(c, r, x, y):
    """Emerald: hexagonal columns standing up, a lit face and a shaded one."""
    for _ in range(int(r.integers(2, 4))):
        cx, cy = x + r.uniform(-3, 3), y + r.uniform(-2, 2)
        hgt = r.uniform(3, 5.5)
        w = r.uniform(1.0, 1.4)
        a = r.uniform(-0.35, 0.35)
        body = poly_mask(rot([(-w, -hgt / 2), (0, -hgt / 2 - 0.8), (w, -hgt / 2), (w, hgt / 2), (-w, hgt / 2)], cx, cy, a))
        left = body & ((XX + 0.5 - cx) * np.cos(a) + (YY + 0.5 - cy) * np.sin(a) < 0)
        c.put(body, 0.42, 1.0, flat=True)
        c.put(left, 0.72, 1.0, flat=True)
        top = body & ~np.roll(body, 1, 0)
        c.put(top, 0.95, flat=True)


def hexagons(c, r, x, y, tall=1.0):
    """Ruby (flat hexagons) and sapphire (tall spindles: tall > 1)."""
    for k in range(int(r.integers(1, 3))):
        s = r.uniform(2.3, 3.2) * (1 if k == 0 else 0.7)
        cx, cy = x + (0 if k == 0 else r.uniform(-3, 3)), y + (0 if k == 0 else r.uniform(-3, 3))
        pts = [(cx + np.cos(i * np.pi / 3) * s, cy + np.sin(i * np.pi / 3) * s * tall) for i in range(6)]
        m = poly_mask(pts)
        inner = poly_mask([(cx + (px_ - cx) * 0.5, cy + (py_ - cy) * 0.5) for px_, py_ in pts])
        upper = m & (YY + XX < cx + cy)
        c.put(m, 0.4, 1.0, flat=True)
        c.put(upper, 0.62, 1.0, flat=True)
        c.put(inner, 0.78, 1.0, flat=True)
        c.px(cx - s * 0.3, cy - s * 0.3 * tall, 1.0, flat=True)


def spikes(c, r, x, y):
    """Amethyst: a cluster of points fanning out of one base."""
    base_a = r.uniform(0, 2 * np.pi)
    for k in range(int(r.integers(3, 5))):
        a = base_a + (k - 1.5) * 0.5 + r.uniform(-0.15, 0.15)
        L = r.uniform(3.2, 5.0)
        pts = rot([(0, -1.2), (L, 0), (0, 1.2)], x, y, a)
        m = poly_mask(pts)
        half = ((YY + 0.5 - y) * np.cos(a) - (XX + 0.5 - x) * np.sin(a)) < 0
        c.put(m & half, 0.8, 1.0, flat=True)
        c.put(m & ~half, 0.45, 1.0, flat=True)


def ribbons(c, r, x, y, w=1.6, tone=0.55):
    """Mythril: smooth flowing ribbons with a bright crest."""
    a = r.uniform(0, 2 * np.pi)
    ph = r.uniform(0, 6)
    pts = []
    for i in range(10):
        t = (i - 4.5) * 0.9
        pts.append((x + np.cos(a) * t - np.sin(a) * np.sin(t * 0.9 + ph) * 1.4, y + np.sin(a) * t + np.cos(a) * np.sin(t * 0.9 + ph) * 1.4))
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        c.line(x0, y0, x1, y1, w, t=tone, dome=1.6)


def branches(c, r, x, y):
    """Orichalcum: thin pink veins with bright crystal knots where they fork."""
    for _ in range(int(r.integers(2, 4))):
        a = r.uniform(0, 2 * np.pi)
        L = r.uniform(3.5, 5.5)
        ex, ey = x + np.cos(a) * L, y + np.sin(a) * L
        c.line(x, y, ex, ey, 1.0, t=0.55)
        k = c.ellipse(ex, ey, 0.9, 0.9, t=0.8)
        c.accent(k, 0)
    c.ellipse(x, y, 1.3, 1.3, t=0.85, dome=2)


def needles(c, r, x, y):
    """Titanium (rutile): sprays of fine needles from one point."""
    a0 = r.uniform(0, 2 * np.pi)
    for k in range(int(r.integers(4, 7))):
        a = a0 + r.uniform(-0.7, 0.7)
        L = r.uniform(3.5, 6)
        c.line(x, y, x + np.cos(a) * L, y + np.sin(a) * L, 1.1, t=0.4 + 0.45 * (k % 2), dome=0.8)


def scales(c, r, x, y):
    """Palladium: overlapping lens-shaped flakes."""
    for k in range(int(r.integers(3, 5))):
        m = c.ellipse(x + r.uniform(-2.5, 2.5), y + r.uniform(-2.5, 2.5), r.uniform(1.5, 2.2), r.uniform(0.7, 1.0), r.uniform(0, np.pi), t=0.45 + 0.1 * k, dome=1.4)
        rim = m & ~np.roll(np.roll(m, 1, 0), 1, 1)
        c.put(rim, 0.9, flat=True)


def organic(c, r, x, y):
    """Verdite (chlorophyte): leafy crystals with a glowing midrib."""
    for _ in range(int(r.integers(2, 4))):
        a = r.uniform(0, 2 * np.pi)
        L = r.uniform(2.2, 3.4)
        cx, cy = x + r.uniform(-2, 2), y + r.uniform(-2, 2)
        m = c.ellipse(cx, cy, L, L * 0.45, a, t=0.45, dome=1.4)
        rib = c.line(cx - np.cos(a) * L * 0.8, cy - np.sin(a) * L * 0.8, cx + np.cos(a) * L * 0.8, cy + np.sin(a) * L * 0.8, 0.8, t=0.9)
        c.accent(rib & m, 0)


def veins(c, r, x, y, glow_every=3):
    """Umbral, sanguinite, magmarite: a network of cracks with glowing beads in it."""
    for _ in range(int(r.integers(2, 4))):
        a = r.uniform(0, 2 * np.pi)
        px_, py_ = x, y
        for i in range(int(r.integers(4, 8))):
            nx_, ny_ = px_ + np.cos(a) * 1.3, py_ + np.sin(a) * 1.3
            m = c.line(px_, py_, nx_, ny_, 1.2, t=0.5)
            if i % glow_every == 0:
                c.accent(c.ellipse(nx_, ny_, 0.8, 0.8, t=0.8), 0)
            px_, py_ = nx_, ny_
            a += r.uniform(-0.8, 0.8)
    c.accent(c.ellipse(x, y, 1.2, 1.2, t=0.8), 0)


def pitted(c, r, x, y):
    """Meteorite: a fused metal lump with dark pits (regmaglypts)."""
    m = c.ellipse(x, y, r.uniform(3.0, 4.0), r.uniform(2.4, 3.2), r.uniform(0, 3), t=0.55, dome=2.5)
    for _ in range(int(r.integers(2, 5))):
        p = c.ellipse(x + r.uniform(-2, 2), y + r.uniform(-1.5, 1.5), 0.7, 0.7, t=0.2, dome=0.2)
        c.accent(p & m, 0)


def rings(c, r, x, y):
    """Resonarium: little rings, like a sound frozen in the stone."""
    rr = r.uniform(1.8, 2.8)
    dx, dy = tor(x, y)
    d = np.hypot(dx, dy)
    ring = (d > rr - 0.75) & (d <= rr + 0.25)
    c.put(ring, 0.6, np.ones((M, M)))
    c.accent(c.ellipse(x, y, 0.8, 0.8, t=0.85), 0)


def drops(c, r, x, y):
    """Ambrosium: glowing golden drops."""
    for _ in range(int(r.integers(2, 4))):
        cx, cy = x + r.uniform(-2.5, 2.5), y + r.uniform(-2.5, 2.5)
        c.poly([(cx, cy - 2.2), (cx + 1.3, cy + 0.4), (cx, cy + 1.6), (cx - 1.3, cy + 0.4)], t=0.6)
        c.px(cx - 0.4, cy - 0.3, 1.0, flat=True)


def rhombs(c, r, x, y):
    """Zanite: tall rhombic crystals."""
    for k in range(int(r.integers(2, 4))):
        cx, cy = x + r.uniform(-3, 3), y + r.uniform(-2, 2)
        s = r.uniform(1.2, 1.8)
        left = poly_mask([(cx - s, cy), (cx, cy - s * 1.9), (cx, cy + s * 1.9)])
        right = poly_mask([(cx, cy - s * 1.9), (cx + s, cy), (cx, cy + s * 1.9)])
        c.put(left, 0.8, 1.0, flat=True)
        c.put(right, 0.45, 1.0, flat=True)


def lifted(c, r, x, y):
    """Gravitite: lumps that seem to float, with a faint streak under each."""
    for _ in range(int(r.integers(2, 3))):
        cx, cy = x + r.uniform(-3, 3), y + r.uniform(-2, 2)
        c.ellipse(cx, cy, 1.6, 1.2, t=0.6, dome=2)
        s = c.line(cx, cy + 1.6, cx, cy + 3.2, 0.9, t=0.4)
        c.accent(s, 0)


def crusts(c, r, x, y):
    """Sulfur: crusty yellow crystal crusts."""
    for _ in range(int(r.integers(4, 7))):
        rr = r.uniform(0.7, 1.4)
        c.ellipse(x + r.uniform(-3, 3), y + r.uniform(-2, 2), rr, rr * 0.8, t=r.uniform(0.45, 0.75), dome=1)


def quartz_prisms(c, r, x, y):
    prisms(c, r, x, y, n=(2, 4), w=(1.4, 2.0), ln=(1.8, 3.0), tip=1.0)


def topaz_prisms(c, r, x, y):
    prisms(c, r, x, y, n=(2, 3), w=(1.8, 2.4), ln=(3.0, 4.6), tip=1.6)


def tin_prisms(c, r, x, y):
    prisms(c, r, x, y, n=(3, 5), w=(2.0, 2.6), ln=(1.8, 2.8), tip=1.2)


def sapphire_spindles(c, r, x, y):
    hexagons(c, r, x, y, tall=1.7)


def lead_cubes(c, r, x, y):
    cubes(c, r, x, y, (2, 4))


def cobalt_cubes(c, r, x, y):
    cubes(c, r, x, y, (2, 3))
    for _ in range(2):
        c.accent(c.ellipse(x + r.uniform(-3, 3), y + r.uniform(-3, 3), 0.7, 0.7, t=0.6), 0)


def nether_flakes(c, r, x, y):
    grains(c, r, x, y, n=(3, 6), size=(0.5, 0.9))


def red_shards(c, r, x, y):
    shards(c, r, x, y)
    c.accent(c.ellipse(x, y, 0.7, 0.7, t=0.95), 0)


def adamant_shards(c, r, x, y):
    shards(c, r, x, y, n=(3, 5), L=(2.2, 3.8))


# ================================================================ materials

# name: (habit, mid colour, ramp spread, saturation, accents, deposits (min, max), size)
MATERIALS = {
    "coal": (plates, "#2e2c2b", 0.42, 0.6, [], (5, 7)),
    "iron": (nodules, "#c8977a", 0.42, 1.0, ["#9a4a22"], (5, 7)),
    "copper": (dendrite, "#d9763a", 0.45, 1.05, ["#4fb59a"], (4, 6)),
    "tin": (tin_prisms, "#8e98a6", 0.55, 0.9, [], (5, 7)),
    "lead": (lead_cubes, "#6a7386", 0.5, 0.9, [], (4, 6)),
    "silver": (wires, "#dfe3ea", 0.38, 0.6, [], (5, 7)),
    "tungsten": (blades, "#6f8c74", 0.5, 0.9, [], (4, 6)),
    "gold": (quartz_vein_gold, "#f2c21e", 0.42, 1.1, ["#ece8dc"], (4, 5)),
    "platinum": (grains, "#c4dbe4", 0.36, 0.8, [], (5, 7)),
    "redstone": (red_shards, "#d6141a", 0.5, 1.1, ["#ff8a6a"], (5, 7)),
    "lapis": (patches, "#2b55c6", 0.45, 1.1, ["#e8c040", "#e6ecf2"], (4, 6)),
    "diamond": (octahedra, "#5ce3da", 0.55, 1.0, [], (4, 6)),
    "emerald": (columns, "#19b44d", 0.55, 1.05, [], (3, 5)),
    "ruby": (hexagons, "#d2203f", 0.55, 1.1, [], (4, 6)),
    "sapphire": (sapphire_spindles, "#2458d2", 0.55, 1.1, [], (4, 6)),
    "topaz": (topaz_prisms, "#e6a31a", 0.5, 1.1, [], (4, 6)),
    "amethyst": (spikes, "#8c45cc", 0.55, 1.05, [], (4, 6)),
    "quartz": (quartz_prisms, "#ece4d8", 0.35, 0.6, [], (5, 7)),
    "nether_gold": (nether_flakes, "#f2c21e", 0.42, 1.1, [], (6, 7)),
    # Terraria's hard mode.
    "cobalt": (cobalt_cubes, "#2f6be0", 0.5, 1.1, ["#d06ad0"], (4, 6)),
    "palladium": (scales, "#e0623a", 0.45, 1.05, [], (4, 6)),
    "mythril": (ribbons, "#3ab0a0", 0.5, 1.0, [], (4, 5)),
    "orichalcum": (branches, "#e05ab8", 0.5, 1.05, ["#ffd0f0"], (4, 6)),
    "adamantite": (adamant_shards, "#d02838", 0.55, 1.1, [], (4, 6)),
    "titanium": (needles, "#b8bec8", 0.45, 0.6, [], (4, 6)),
    "verdite": (organic, "#3cc040", 0.5, 1.1, ["#c8ff7a"], (4, 6)),
    # Evil, hell and sky.
    "umbral": (veins, "#7a3cc0", 0.45, 1.0, ["#e0a0ff"], (4, 5)),
    "sanguinite": (veins, "#c02838", 0.45, 1.0, ["#ff9a7a"], (4, 5)),
    "magmarite": (veins, "#ff7a1a", 0.4, 1.1, ["#fff07a"], (5, 6)),
    "meteorite": (pitted, "#b07a5a", 0.45, 0.7, ["#2a2020"], (4, 5)),
    # Mods.
    "ambrosium": (drops, "#f0c83a", 0.45, 1.1, [], (4, 6)),
    "zanite": (rhombs, "#8a5ae0", 0.5, 1.05, [], (4, 6)),
    "gravitite": (lifted, "#e05ac8", 0.5, 1.05, ["#f8c8f0"], (4, 5)),
    "resonarium": (rings, "#2ac0b8", 0.45, 1.05, ["#d0fff8"], (4, 6)),
    "sulfur": (crusts, "#e8d43a", 0.45, 1.1, [], (4, 6)),
}

# Texture name: (material, host block texture).
ORES = {}
for _m in ["coal", "iron", "copper", "tin", "lead", "silver", "tungsten", "gold", "platinum", "redstone", "lapis", "diamond", "emerald",
           "ruby", "sapphire", "topaz", "amethyst", "cobalt", "palladium", "mythril", "orichalcum", "adamantite", "titanium"]:
    ORES[f"{_m}_ore"] = (_m, "stone")
    ORES[f"deepslate_{_m}_ore"] = (_m, "deepslate")
ORES.update({
    "verdite_ore": ("verdite", "stone"),
    "nether_gold_ore": ("nether_gold", "netherrack"),
    "nether_quartz_ore": ("quartz", "netherrack"),
    "umbral_ore": ("umbral", "ruin_stone"),
    "sanguinite_ore": ("sanguinite", "blight_stone"),
    "magmarite_ore": ("magmarite", "ash_block"),
    "meteorite_ore": ("meteorite", "blackstone"),
    "ambrosium_ore": ("ambrosium", "holystone"),
    "zanite_ore": ("zanite", "holystone"),
    "gravitite_ore": ("gravitite", "holystone"),
    "resonarium_ore": ("resonarium", "gloomslate"),
    "sulfur_ore": ("sulfur", "blackstone"),
})


def up2(a):
    return np.repeat(np.repeat(a, 2, 0), 2, 1)


def make(name):
    mat, host = ORES[name]
    habit, mid, spread, sat, accents, (lo, hi) = MATERIALS[mat]
    r = rng_for("ore2:" + name)
    base = bg(host)
    c = Canvas()
    pts = list(LAYOUT)
    r.shuffle(pts)
    for x, y in pts[: int(r.integers(lo, hi + 1))]:
        habit(c, r, x + r.uniform(-2, 2), y + r.uniform(-2, 2))
    pal = ramp(mid, 7, spread, 0.12, sat=sat)
    t = np.where(c.flat, c.t, c.t + light(c.h * 1.4) * 0.9)
    col = quantize(np.clip(t, 0, 1), pal)
    for i, a in enumerate(accents):
        ap = ramp(a, 5, 0.3, 0.1)
        acol = quantize(np.clip(0.55 + light(c.h * 1.4) * 0.8, 0, 1), ap)
        col = np.where((c.acc == i)[..., None], acol, col)
    m = c.m
    # Outline and drop shadow on the rock (art pixels), then grow everything 2x.
    near = np.zeros((M, M), bool)
    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        near |= np.roll(np.roll(m, dy, 0), dx, 1)
    outline = near & ~m
    shadow = np.roll(np.roll(m, 1, 0), 1, 1) & ~m & ~outline
    img = base.copy()
    o64, s64, m64 = up2(outline), up2(shadow), up2(m)
    img[o64, :3] *= 0.5
    img[s64, :3] *= 0.72
    col64 = up2(col)
    img[m64, :3] = col64[m64]
    img[..., 3] = 255
    return img


def registry():
    return {n: (lambda n=n: make(n)) for n in ORES}
