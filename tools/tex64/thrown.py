"""A design of its own for the warrior's other weapons (HP 2026-11-01, "classe guerreiro"):
yoyos, boomerangs, flails, halberds and the valkyrie lance were one picture each, recoloured
per material. Here every one gets its own shape: a yoyo's rim, hub and spikes; a boomerang's
bend and edges; a flail's head (spiked ball, skull, sun, eye); a halberd's blade and spike.

Painted like arms.py/blades.py: lit height fields, a one-pixel outline, then pixel.py.
"""

import numpy as np

from arms import WOOD, Art, Frame, _pal, dome, grip, polyline, cracks, tex_for, _shaft
from blades import BONE, DARK, GOLD, ONES, P, STEEL, glow, metal, part, star
from style import N, XX, YY, value_noise

PX, PY = XX + 0.5, YY + 0.5
STRING = P("#e8e0cc", 0.4, 1.0)


def disc(cx, cy, r, ry=None):
    ry = r if ry is None else ry
    return ((PX - cx) / r) ** 2 + ((PY - cy) / ry) ** 2 <= 1


def rad(cx, cy):
    return np.hypot(PX - cx, PY - cy), np.arctan2(PY - cy, PX - cx)


# ---------------------------------------------------------------- yoyos

C = (36.0, 36.0)


def _string(art):
    s = polyline([(4, 4), (14, 12), (24, 22), (C[0] - 2, C[1] - 2)], 2.0)
    art.paint(s, ONES, STRING, base=0.6, outline=True)


def _yoyo(art, pal, r=17.0, rim=None, hub=None, extra=None, base=0.5, spec=0.3):
    _string(art)
    m = disc(*C, r)
    d, a = rad(*C)
    h = dome(m, r * 0.6) + (np.abs(d - r * 0.62) < 1.6) * -0.8
    alb = 0.0 if extra is None else extra
    art.paint(m, h, pal, base=base, spec=spec, power=14, albedo=alb)
    if rim is not None:
        art.paint(m & (d > r - 3.0), dome(m, 2.0), rim, base=0.5, spec=0.3, outline=False)
    if hub is not None:
        hb = disc(*C, 5.0)
        art.paint(hb, dome(hb, 3.0), hub, base=0.55, spec=0.4)
    return m, d, a


def wood_yoyo(art):
    pal = P("#a07a46", 0.5, 1.05)
    m, d, a = _yoyo(art, pal, extra=np.sin(np.hypot(PX - C[0], PY - C[1]) * 1.4) * 0.07, hub=P("#6a4a28", 0.4))


def jungle_yoyo(art):
    pal = P("#3a8a2a", 0.6, 1.2)
    d, a = rad(*C)
    for k in range(8):
        aa = k * np.pi / 4
        lf = disc(C[0] + np.cos(aa) * 16, C[1] + np.sin(aa) * 16, 4.5, 4.5)
        art.paint(lf, dome(lf, 2), P("#5ab83a", 0.6, 1.2), base=0.5, spec=0.05)
    _yoyo(art, pal, r=15.0, hub=P("#e04a6a", 0.6, 1.3), extra=(np.abs(np.sin(a * 5)) > 0.8) * 0.12)


def umbral_yoyo(art):
    pal = P("#5a3a7a", 0.6, 1.2)
    d, a = rad(*C)
    spikes = (d < 22.0 - 6.0 * np.abs(np.sin(a * 4))) & (d > 12)
    art.paint(spikes, ONES, pal, base=0.4, diff=0.0)
    _yoyo(art, pal, r=14.0, hub=DARK, base=0.52)
    glow(art, disc(*C, 2.4), ONES * 0.8, P("#c060ff", 0.4, 1.4))


def sanguinite_yoyo(art):
    pal = P("#b02838", 0.62, 1.2)
    m, d, a = _yoyo(art, pal, r=16.0, rim=P("#7a1a24", 0.5))
    eye = disc(*C, 7.0, 5.0)
    art.paint(eye, dome(eye, 2.4), P("#f0e8d8", 0.4), base=0.65)
    art.paint(disc(*C, 3.0, 4.0), ONES, P("#401018", 0.3), base=0.3, outline=False)
    teeth = m & (d > 13) & (np.abs(np.sin(a * 10)) > 0.6)
    art.img[teeth, :3] = [236, 226, 200]


def magma_yoyo(art):
    pal = P("#3c302c", 0.4, 1.0)
    m, d, a = _yoyo(art, pal, r=17.0, extra=(value_noise(art.rng, 8) - 0.5) * 0.2)
    v = cracks(art, m, art.rng, 6, 1.4)
    glow(art, v > 0, v, P("#ff8a1a", 0.5, 1.3))
    glow(art, disc(*C, 4.0), ONES * 0.9, P("#ffd040", 0.4))


def crystal_yoyo(art):
    pal = P("#c070e0", 0.7, 1.2)
    _string(art)
    d, a = rad(*C)
    pts = [(C[0] + np.cos(k * np.pi / 4 + 0.39) * 18, C[1] + np.sin(k * np.pi / 4 + 0.39) * 18) for k in range(8)]
    from arms import poly
    m = poly(pts)
    facet = np.where(np.sin(a * 4) > 0, 0.14, -0.1) + (d < 9) * 0.12
    art.paint(m, ONES, pal, base=0.55, diff=0.0, albedo=facet)
    glow(art, disc(*C, 3.6), ONES * 0.9, P("#ffe0ff", 0.3))


def stardust_yoyo(art):
    _string(art)
    from arms import poly
    pts = []
    for i in range(10):
        a = -np.pi / 2 + i * np.pi / 5
        r = 20.0 if i % 2 == 0 else 9.0
        pts.append((C[0] + np.cos(a) * r, C[1] + np.sin(a) * r))
    m = poly(pts)
    art.paint(m, dome(m, 4), P("#3a7ae0", 0.7, 1.2), base=0.55, spec=0.5)
    glow(art, disc(*C, 5.0), ONES * 0.9, P("#d8f0ff", 0.3))
    sp = m & (art.rng.random((N, N)) > 0.94)
    art.img[sp, :3] = [255, 250, 230]


# ---------------------------------------------------------------- boomerangs


def _bend(pts, w):
    return polyline(pts, w)


def wood_boomerang(art):
    pal = P("#a07a46", 0.5, 1.05)
    m = _bend([(10, 52), (24, 26), (30, 12)], 9.0) | _bend([(24, 26), (52, 34)], 9.0)
    art.paint(m, dome(m, 3.0), pal, base=0.5, albedo=tex_for(art, m, None, "wood"))


def enchanted_boomerang(art):
    pal = P("#3a6ad8", 0.7, 1.15)
    m = _bend([(8, 50), (22, 28), (26, 10)], 8.0) | _bend([(22, 28), (54, 30)], 8.0)
    art.paint(m, dome(m, 3.0), pal, base=0.52, spec=0.4)
    runes = m & (((PX + PY) % 8) < 2.6) & (np.hypot(PX - 22, PY - 28) > 6)
    glow(art, runes, ONES * 0.75, P("#9af0ff", 0.4))
    s = star(Frame(x0=22, y0=28, ang=0), 0, 0, 6.0, 2.6)
    art.paint(s, dome(s, 1.4), GOLD, base=0.6, spec=0.4)


def ice_boomerang(art):
    pal = P("#9ad8f4", 0.6, 1.1)
    from arms import poly
    m = poly([(6, 54), (14, 40), (12, 30), (20, 22), (18, 8), (30, 18), (30, 26), (44, 24), (58, 30), (40, 34), (30, 38), (20, 44)])
    facet = np.where(PX + PY < 50, 0.14, -0.08)
    art.paint(m, ONES, pal, base=0.6, diff=0.0, albedo=facet)
    frost = m & (value_noise(art.rng, 4) > 0.65)
    art.img[frost, :3] = [236, 248, 255]


def flame_boomerang(art):
    pal = P("#d8501a", 0.65, 1.3, 0.2)
    m = _bend([(10, 52), (24, 28), (30, 14)], 8.0) | _bend([(24, 28), (50, 34)], 8.0)
    art.paint(m, dome(m, 3.0), pal, base=0.5, spec=0.3)
    for (x, y, dx, dy) in [(30, 14, 2, -8), (50, 34, 8, -2), (10, 52, -6, 4)]:
        fl = _bend([(x, y), (x + dx, y + dy)], 5.0)
        art.paint(fl & ~m, ONES, P("#ffb030", 0.5, 1.3), base=0.7, diff=0.0)
    glow(art, m & (np.abs(PX - PY + 4) < 2), ONES * 0.8, P("#ffd060", 0.4))


def tide_boomerang(art):
    pal = P("#1a9a9a", 0.66, 1.2)
    d, a = rad(30, 34)
    m = (d < 24) & (d > 13) & (a < 0.6) & (a > -2.6)
    art.paint(m, dome(m, 3.0), pal, base=0.5, spec=0.45)
    crest = m & (d > 20) & (np.abs(np.sin(a * 9)) > 0.5)
    art.img[crest, :3] = [220, 250, 245]
    foam = disc(30 + np.cos(0.6) * 18, 34 + np.sin(0.6) * 18, 4.0)
    art.paint(foam, dome(foam, 2), P("#e8fff8", 0.3), base=0.7)


# ---------------------------------------------------------------- flails


def _flail_handle(art, pal=WOOD, chain=STEEL):
    f = Frame(x0=4.6, y0=59.4)
    grip(art, f, 0.5, 22, 2.4, pal, "bands", base=0.5)
    links = np.zeros((N, N), bool)
    for k in range(5):
        u = 25 + k * 3.6
        links |= f.disc(u, 0, 1.9, 1.4) & ~f.disc(u, 0, 0.9, 0.5)
    art.paint(links, ONES, chain, base=0.55, outline=True)
    return f.xy(44, 0)


def _ball(art, cx, cy, r, pal, spikes=0, spike_len=5.0, spike_pal=None, base=0.5, extra=None):
    d, a = rad(cx, cy)
    if spikes:
        sp = (d < r + spike_len * np.clip(np.cos(a * spikes) * 1.6 - 0.6, 0, 1)) & (d > r - 1)
        art.paint(sp, ONES, spike_pal if spike_pal is not None else pal, base=0.45, diff=0.0)
    m = disc(cx, cy, r)
    art.paint(m, dome(m, r * 0.8), pal, base=base, spec=0.4, power=14, albedo=0.0 if extra is None else extra)
    return m


def iron_flail(art):
    x, y = _flail_handle(art)
    _ball(art, x + 4, y - 4, 9.0, STEEL, spikes=8, spike_len=5.0)


def umbral_flail(art):
    x, y = _flail_handle(art, DARK, P("#5a3a7a", 0.6, 1.2))
    _ball(art, x + 4, y - 4, 9.0, P("#5a3a7a", 0.6, 1.2), spikes=6, spike_len=8.0, spike_pal=P("#3a2450", 0.5))
    glow(art, disc(x + 4, y - 4, 2.6), ONES * 0.8, P("#c060ff", 0.4, 1.4))


def sanguinite_flail(art):
    x, y = _flail_handle(art, P("#4a1a20", 0.4))
    m = _ball(art, x + 4, y - 4, 10.0, P("#b02838", 0.62, 1.2))
    eye = disc(x + 4, y - 4, 5.0, 4.0)
    art.paint(eye, dome(eye, 2), P("#f0e8d8", 0.4), base=0.65)
    art.paint(disc(x + 4, y - 4, 2.0, 3.0), ONES, P("#401018", 0.3), base=0.3, outline=False)


def bone_flail(art):
    x, y = _flail_handle(art, P("#5a3a24", 0.4), BONE)
    cx, cy = x + 4, y - 5
    skull = disc(cx, cy, 10.0, 9.0) | disc(cx, cy + 7, 6.0, 4.0)
    art.paint(skull, dome(skull, 4), BONE, base=0.58, spec=0.1)
    for sx in (-4, 4):
        art.paint(disc(cx + sx, cy + 1, 2.6, 3.0), ONES, P("#2a2018", 0.3), base=0.3, outline=False)
    teeth = (np.abs(PY - (cy + 8)) < 1.2) & (np.abs(PX - cx) < 4) & ((PX.astype(int) % 2) == 0)
    art.img[teeth & skull, :3] = [60, 50, 40]


def sun_flail(art):
    x, y = _flail_handle(art, P("#7a4a1a", 0.4), GOLD)
    _ball(art, x + 4, y - 4, 9.0, P("#f0a020", 0.7, 1.3, 0.2), spikes=12, spike_len=5.0, spike_pal=P("#ffd040", 0.5, 1.3))
    glow(art, disc(x + 4, y - 4, 4.0), ONES * 0.9, P("#fff0a0", 0.3))


def stone_fist(art):
    x, y = _flail_handle(art, P("#5a5a58", 0.4), P("#8a8884", 0.5))
    cx, cy = x + 3, y - 4
    pal = P("#8a8884", 0.5, 1.0)
    palm = disc(cx, cy + 2, 9.0, 8.0)
    art.paint(palm, dome(palm, 3), pal, base=0.5, albedo=tex_for(art, palm, None, "stone"))
    for k in range(4):
        fg = disc(cx - 6 + k * 4.2, cy - 6, 2.4, 3.4)
        art.paint(fg, dome(fg, 1.6), pal, base=0.55, albedo=tex_for(art, fg, None, "stone"))
    th = disc(cx - 9, cy + 3, 2.6, 4.0)
    art.paint(th, dome(th, 1.6), pal, base=0.5)


def solar_eruption(art):
    # A long flaming lash of sun-metal links ending in a blazing head.
    f = Frame(x0=4.6, y0=59.4)
    grip(art, f, 0.5, 18, 2.4, P("#7a3a10", 0.4), "bands", base=0.5)
    pts = [f.xy(18 + k * 4.2, 3.0 * np.sin(k * 0.9)) for k in range(12)]
    link = polyline(pts, 3.4)
    art.paint(link, dome(link, 1.2), P("#e86a1a", 0.6, 1.3, 0.2), base=0.55, spec=0.3)
    glow(art, polyline(pts, 1.2), ONES * 0.8, P("#ffc040", 0.4))
    hx, hy = pts[-1]
    fl = disc(hx, hy, 6.5)
    art.paint(fl, dome(fl, 3), P("#ff8a1a", 0.6, 1.3), base=0.6)
    glow(art, disc(hx, hy, 3.0), ONES * 0.95, P("#fff0b0", 0.3))


# ---------------------------------------------------------------- halberds and the lance


def _halberd(art, mat_pal, style, shaft=WOOD):
    f = Frame(x0=4.0, y0=60.0)
    _shaft(art, f, 70, pal=shaft)
    # Spike to the tip.
    from arms import blade
    global HEAD, HOOK
    if style == "stone":
        HEAD = ([(54, -1.6), (52, -9.6), (58, -11.0), (64, -8.0), (63, -1.6)])
        HOOK = ([(55, 1.6), (58, 6.0), (61, 1.6)])
    elif style == "iron":
        HEAD = ([(52, -1.6), (50, -11.6), (56, -12.4), (62, -10.0), (64, -1.6)])
        HOOK = ([(54, 1.6), (54, 7.4), (60, 3.0), (61, 1.6)])
    elif style == "gold":
        HEAD = ([(50, -1.6), (46, -9.0), (52, -13.6), (60, -13.0), (64, -8.0), (63, -1.6)])
        HOOK = ([(53, 1.6), (50, 8.4), (56, 6.0), (60, 1.6)])
    else:
        HEAD = ([(51, -1.6), (48, -12.0), (55, -10.0), (58, -14.6), (63, -9.6), (64, -1.6)])
        HOOK = ([(54, 1.6), (57, 9.0), (60, 4.0), (62, 1.6)])
    # The head is drawn big, like the old icons: grown around its middle.
    def grow(m_pts):
        return f.poly([(57 + (u - 57) * 1.5, v * 1.45) for (u, v) in m_pts])
    head, hook = grow(HEAD), grow(HOOK)
    sp, h, s, side = blade(f, [62, 72, 82], [2.6, 2.0, 0.2], [2.6, 2.0, 0.2], hmax=1.8)
    kind = {"stone": "stone", "gold": "gold", "gem": "gem"}.get(style, "steel")
    for m in (head, hook):
        if kind == "gem":
            art.paint(m, ONES, mat_pal, base=0.55, diff=0.0, albedo=np.where(f.V < -6, 0.14, -0.08))
        else:
            metal(art, f, m, dome(m, 1.6), mat_pal, base=0.5, spec=0.1 if kind == "stone" else 0.4, kind=kind)
    metal(art, f, sp, h, mat_pal, base=0.52, spec=0.4, kind="stone" if kind == "stone" else "steel")
    cap = f.box(46, 50, -3.0, 3.0)
    part(art, cap, mat_pal, 1.0)


def stone_halberd(art):
    _halberd(art, P("#85837f", 0.5, 1.0), "stone")


def iron_halberd(art):
    _halberd(art, STEEL, "iron")


def golden_halberd(art):
    _halberd(art, GOLD, "gold")


def diamond_halberd(art):
    _halberd(art, P("#4fd6cf", 0.72, 1.15), "gem", shaft=P("#3a3640", 0.4))


def valkyrie_lance(art):
    # A long cavalry lance: a gold-banded white shaft, a round vamplate guarding the hand and a
    # long cone tip.
    f = Frame(x0=4.0, y0=60.0)
    white = P("#e8e4dc", 0.5, 1.0)
    grip(art, f, 1, 16, 2.0, P("#4a3a6a", 0.4), "spiral", base=0.45)
    vp = f.disc(18.0, 0, 3.0, 8.0)
    part(art, vp, GOLD, 2.0, base=0.5, spec=0.45)
    from arms import blade
    m, h, s, side = blade(f, [20, 30, 66, 82], [4.0, 3.4, 1.8, 0.2], [4.0, 3.4, 1.8, 0.2], hmax=2.4)
    metal(art, f, m, h, white, base=0.55, spec=0.45, power=18, kind="polish")
    bands = m & ((np.abs(f.U - 34) < 1.2) | (np.abs(f.U - 50) < 1.2))
    art.img[bands, :3] = [224, 176, 50]
    for sgn in (-1, 1):
        wg = f.poly([(20, sgn * 3), (16, sgn * 10), (24, sgn * 8)])
        part(art, wg, white, 1.0, base=0.6, spec=0.2)


# ---------------------------------------------------------------- warrior accessories


def _gauntlet(art, pal, cuff, glow_pal=None):
    from arms import poly
    hand = poly([(14, 20), (40, 12), (52, 18), (54, 34), (46, 46), (22, 50), (14, 40)])
    art.paint(hand, dome(hand, 5), pal, base=0.5, spec=0.35)
    for k in range(4):
        fg = polyline([(40 + k * 1.2, 16 + k * 7.0), (54, 12 + k * 7.6)], 5.4)
        art.paint(fg, dome(fg, 2), pal, base=0.55, spec=0.35)
    c = poly([(8, 30), (16, 18), (24, 22), (22, 54), (12, 56), (6, 46)])
    art.paint(c, dome(c, 3), cuff, base=0.5, spec=0.2)
    knuck = np.zeros((N, N), bool)
    for k in range(4):
        knuck |= disc(44, 18 + k * 7.0, 2.4)
    art.paint(knuck, dome(knuck, 1.4), STEEL, base=0.6, spec=0.5)
    if glow_pal is not None:
        v = cracks(art, hand, art.rng, 5, 1.4)
        glow(art, v > 0, v, glow_pal)


def fire_gauntlet(art):
    _gauntlet(art, P("#3c302c", 0.45, 1.0), P("#8a2a10", 0.5, 1.2), P("#ff8a1a", 0.5, 1.3))


def berserker_glove(art):
    _gauntlet(art, P("#8a5a32", 0.5, 1.05), P("#5a3a24", 0.45))
    for (x, y) in [(28, 22), (30, 34), (26, 44)]:
        sp = polyline([(x, y), (x - 6, y - 4)], 3.0)
        art.paint(sp, ONES, STEEL, base=0.6, outline=True)


def yoyo_string(art):
    d, a = rad(32, 32)
    coil = (np.abs(((d + a * 1.6) % 4.0) - 2.0) < 0.9) & (d < 20) & (d > 6)
    art.paint(coil, ONES, STRING, base=0.6, outline=True)
    end = polyline([(48, 44), (58, 58)], 2.2)
    art.paint(end, ONES, STRING, base=0.6)
    art.paint(disc(58, 58, 3.0), dome(disc(58, 58, 3.0), 2), P("#c04040", 0.6, 1.2), base=0.55)


DESIGNS = {k: v for k, v in globals().items() if callable(v) and k.split("_")[-1] in ("yoyo", "boomerang", "flail", "halberd") }
DESIGNS.update({"stone_fist": stone_fist, "solar_eruption": solar_eruption, "valkyrie_lance": valkyrie_lance,
                "fire_gauntlet": fire_gauntlet, "berserker_glove": berserker_glove, "yoyo_string": yoyo_string})
DESIGNS.pop("_yoyo", None)
DESIGNS.pop("_halberd", None)


def make(name):
    art = Art(name)
    DESIGNS[name](art)
    return art.done()


ITEMS = {k: (lambda k=k: make(k)) for k in DESIGNS}
