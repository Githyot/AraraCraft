"""The assassin's items at 64x64 (crates/game/src/assassin.rs, HP 2026-11-01).

Same hand as arms.py (one top-left light, a one-pixel outline in the part's own darkest colour,
lit height fields, material texture on top), and every dagger has a shape of its own, not one
drawing recoloured:

* iron: a straight double-edged dirk with a fuller, a small cross and a ring pommel;
* umbral: a hooked talon (a karambit) of violet-black shadow steel with a finger ring;
* magmarite: a black rock blade with a saw-toothed spine and glowing veins;
* mythril: a leaf blade of polished sea-green metal and a gem pommel;
* verdite: a thorn blade, green, with a vine wound round the grip and a drop of poison;
* lumite: a faceted crystal blade, pale cyan, bright at the core.

Also the smoke bomb (a clay ball with a fuse), the blade oils (stoppered flasks, each its own
colour and shape), and the accessories (cloak, emblem, soft soles, glove, the moonless mark).
"""

import numpy as np

import arms
from arms import (LEATHER_GRIP, DARK_GRIP, Art, Frame, _pal, blade, cracks, dome, grip, mpal, poly, polyline,
                  tex_for, PX, PY)

# Metals the sword designs don't have.
arms.MAT.setdefault("umbral", dict(base="#3b2a52", spread=0.55, sat=1.15, kind="dark", spec=0.3, power=14))
arms.MAT.setdefault("mythril", dict(base="#4fb59a", spread=0.72, sat=1.1, kind="polish", spec=0.5, power=22))
arms.MAT.setdefault("verdite", dict(base="#5aa83a", spread=0.66, sat=1.15, kind="steel", spec=0.3, power=14))
arms.MAT.setdefault("lumite", dict(base="#8fe6f0", spread=0.7, sat=1.05, kind="gem", spec=0.6, power=26))

# The dagger frame: shorter than a sword, so it starts further in to sit in the middle.
def dframe():
    return Frame(x0=11.5, y0=52.5, ang=-45.0)


def _paint_blade(art, f, m, h, mat, base=0.52, glow=None, extra=None):
    d = arms.MAT[mat]
    alb = tex_for(art, m, f, d["kind"])
    if extra is not None:
        alb = alb + extra
    art.paint(m, h, mpal(mat), base=base, diff=1.5 if d["kind"] != "gem" else 1.9, spec=d["spec"],
              power=d["power"], albedo=alb, glow=glow)


def dg_iron(art, f):
    pal = mpal("iron")
    ring = f.disc(2.6, 0, 3.0) & ~f.disc(2.6, 0, 1.4)
    art.paint(ring, dome(ring, 1.2), pal, base=0.45, spec=0.3, power=14)
    grip(art, f, 5.2, 15, 2.1, LEATHER_GRIP, "cross", base=0.45)
    m, h, s, side = blade(f, [17.5, 38, 46, 50.5], [2.9, 2.5, 1.6, 0.15], [2.9, 2.5, 1.6, 0.15], hmax=2.4)
    h = h - ((f.U > 19) & (f.U < 36) & (np.abs(f.V) < 0.7)) * 1.4
    _paint_blade(art, f, m, h, "iron")
    g = f.poly([(15.4, -7.2), (16.8, -7.8), (17.8, -6.4), (17.8, 6.4), (16.8, 7.8), (15.4, 7.2)])
    art.paint(g, dome(g, 1.4), pal, base=0.42, spec=0.3, power=14, albedo=tex_for(art, g, f, "steel"))


def dg_umbral(art, f):
    pal = mpal("umbral")
    # The finger ring at the pommel, then a short grip and a blade that hooks down like a claw.
    ring = f.disc(2.2, 0, 3.6) & ~f.disc(2.2, 0, 1.9)
    art.paint(ring, dome(ring, 1.3), pal, base=0.5, spec=0.3, power=14)
    grip(art, f, 5.5, 15.5, 2.0, DARK_GRIP, "spiral", base=0.45)
    us = np.linspace(16, 50, 40)
    k = (us - 16) / 34
    center = 9.0 * k ** 2.2
    m, h, s, side = blade(f, us, 4.2 * (1 - k ** 1.5) + 0.3, 2.4 * (1 - k ** 1.5) + 0.2, center=center, ridge=-0.6, hmax=2.6)
    _paint_blade(art, f, m, h, "umbral", base=0.5, extra=np.where(m, (PX + PY) % 7 < 1, 0) * 0.05)
    # A cold violet edge.
    edge = m & (s < -side + 0.9)
    art.tint(edge, _pal("#b89aff", 0.4, 1.1), 9)


def dg_magmarite(art, f):
    rng = art.rng
    orb = f.disc(3.0, 0, 2.8)
    art.paint(orb, dome(orb, 2.4), arms.LAVA, base=0.5, spec=0.3)
    grip(art, f, 5.6, 15.5, 2.2, _pal("#2a2220", 0.3), "bands", base=0.4)
    us = np.linspace(17, 50, 34)
    w = np.interp(us, [17, 40, 50], [3.4, 2.8, 0.15])
    teeth = (np.abs(((us - 17) / 3.0) % 1 - 0.5) * 2) * 1.3 * (us < 44)
    m, h, s, side = blade(f, us, w + teeth, np.maximum(w - 0.6, 0.2), hmax=2.6, edge=0.6)
    glow = cracks(art, m, rng, 4, 0.9)
    _paint_blade(art, f, m, h, "magmarite", base=0.5, glow=glow)
    g = f.poly([(15.0, -5.2), (17.8, -4.0), (17.8, 4.0), (15.0, 5.2), (16.2, 0)])
    art.paint(g, dome(g, 1.2), mpal("magmarite"), base=0.45, glow=cracks(art, g, rng, 1, 0.8))


def dg_mythril(art, f):
    pal = mpal("mythril")
    gem = f.disc(2.8, 0, 2.4)
    art.paint(gem, dome(gem, 2.2), _pal("#2fa0e0", 0.7, 1.2), base=0.55, spec=0.6, power=12)
    grip(art, f, 5.0, 15, 2.0, _pal("#e8e2d0", 0.4, 1.0), "ito", base=0.5)
    us = [17.5, 25, 36, 46, 51]
    m, h, s, side = blade(f, us, [1.8, 3.8, 3.6, 1.6, 0.15], [1.8, 3.8, 3.6, 1.6, 0.15], hmax=2.6)
    _paint_blade(art, f, m, h, "mythril", base=0.55)
    g = f.poly([(15.6, -5.4), (17.6, -5.8), (18.0, 0), (17.6, 5.8), (15.6, 5.4), (16.6, 0)])
    art.paint(g, dome(g, 1.3), _pal("#d9b44a", 0.7, 1.2), base=0.5, spec=0.45, power=14)


def dg_verdite(art, f):
    pal = mpal("verdite")
    bud = f.disc(2.6, 0, 2.4)
    art.paint(bud, dome(bud, 2.0), _pal("#8a3a8a", 0.6, 1.2), base=0.5, spec=0.3)
    grip(art, f, 5.0, 15.5, 2.0, _pal("#4a3420", 0.4), "cross", base=0.42)
    # A vine wound round the grip.
    vine = np.zeros_like(bud)
    for t in np.linspace(5.0, 15.5, 7):
        vine |= f.disc(t, np.sin(t * 1.3) * 1.8, 0.9)
    art.paint(vine, dome(vine, 0.8), _pal("#3f8a2a", 0.5, 1.1), base=0.5)
    us = np.linspace(17, 51, 30)
    k = (us - 17) / 34
    w = 4.0 * (1 - k) ** 0.7 + 0.15
    m, h, s, side = blade(f, us, w, w * 0.9, center=-1.2 * np.sin(k * np.pi), hmax=2.6)
    _paint_blade(art, f, m, h, "verdite", base=0.52)
    # Leaf-like thorns off the back and a drop of poison at the tip.
    for u in (24.0, 31.0, 38.0):
        th = f.poly([(u, -2.4), (u + 3.4, -4.6), (u + 2.6, -1.8)])
        art.paint(th, dome(th, 0.8), pal, base=0.45)
    x, y = f.xy(51.5, 1.8)
    drop = ((PX - x) ** 2 + ((PY - y - 1.5) / 1.4) ** 2) <= 1.6 ** 2
    art.paint(drop, dome(drop, 1.4), _pal("#a6f05a", 0.6, 1.2), base=0.6, spec=0.6, power=10)
    g = f.poly([(15.4, -4.2), (17.6, -3.6), (17.6, 3.6), (15.4, 4.2)])
    art.paint(g, dome(g, 1.0), _pal("#3f8a2a", 0.5, 1.1), base=0.45)


def dg_lumite(art, f):
    gem = f.disc(2.8, 0, 2.6)
    art.paint(gem, dome(gem, 2.2), _pal("#2a6a78", 0.6, 1.1), base=0.5, spec=0.5)
    grip(art, f, 5.2, 15.5, 2.1, _pal("#1e2a3a", 0.35), "bands", base=0.42)
    us = [17.0, 22, 34, 45, 51]
    m, h, s, side = blade(f, us, [2.0, 3.6, 3.0, 1.4, 0.15], [2.0, 3.6, 3.0, 1.4, 0.15], hmax=3.0, edge=0.5)
    # Facets: the height steps in bands across the blade, the core glows.
    facet = np.floor((f.U - 17) / 5.0) % 2 * 0.08
    _paint_blade(art, f, m, h, "lumite", base=0.6, extra=np.where(m, facet, 0))
    core = m & (np.abs(f.V) < 0.8) & (f.U > 19) & (f.U < 46)
    art.tint(core, _pal("#e8ffff", 0.3, 1.0), 12)
    for sgn in (-1, 1):
        fl = f.poly([(15.6, 0), (18.4, sgn * 1.8), (17.0, sgn * 6.8), (15.4, sgn * 3.6)])
        art.paint(fl, dome(fl, 1.0), mpal("lumite"), base=0.55, spec=0.5, power=20)


DAGGERS = {"iron": dg_iron, "umbral": dg_umbral, "magmarite": dg_magmarite, "mythril": dg_mythril,
           "verdite": dg_verdite, "lumite": dg_lumite}


def dagger(mat):
    art = Art(f"{mat}_dagger")
    DAGGERS[mat](art, dframe())
    return art.done()


# ---------------------------------------------------------------- the smoke bomb


def smoke_bomb():
    art = Art("smoke_bomb")
    ball = ((PX - 30) ** 2 + (PY - 38) ** 2) <= 17.0 ** 2
    clay = _pal("#4a4648", 0.5, 1.0)
    alb = (arms.value_noise(art.rng, 4) - 0.5) * 0.12
    art.paint(ball, dome(ball, 9.0), clay, base=0.5, diff=1.3, albedo=alb)
    # A band of twine round it and a wax seal at the neck.
    band = ball & (np.abs(PY - 38 - (PX - 30) * 0.25) < 1.6)
    art.tint(band, _pal("#b49a6a", 0.42, 1.0), 6)
    neck = ((PX - 38) ** 2 + (PY - 22.5) ** 2) <= 4.2 ** 2
    art.paint(neck, dome(neck, 2.4), _pal("#7a1e2a", 0.45, 1.1), base=0.5, spec=0.3)
    fuse = polyline([(39, 21), (43, 15), (47, 13), (50, 9)], 1.8)
    art.paint(fuse, np.ones_like(PX) * 0.5, _pal("#c8b07a", 0.4, 1.0), base=0.5)
    spark = ((PX - 51) ** 2 + (PY - 8) ** 2) <= 2.0 ** 2
    art.paint(spark, dome(spark, 1.6), arms.LAVA, base=0.8, outline=False)
    return art.done()


# ---------------------------------------------------------------- oils


def flask(name, liquid, shape):
    art = Art(name)
    glass = _pal("#c8dce0", 0.45, 0.9)
    if shape == "round":
        body = ((PX - 32) ** 2 + (PY - 41) ** 2) <= 15.0 ** 2
        top = 27.0
    elif shape == "square":
        body = poly([(19, 28), (45, 28), (46, 56), (18, 56)])
        top = 28.0
    elif shape == "tall":
        body = poly([(24, 22), (40, 22), (43, 56), (21, 56)])
        top = 22.0
    else:  # a teardrop
        body = (((PX - 32) / 13.0) ** 2 + ((PY - 44) / 13.0) ** 2 <= 1) | poly([(25, 36), (39, 36), (34, 22), (30, 22)])
        top = 22.0
    neck = poly([(28, top - 9), (36, top - 9), (36, top + 1), (28, top + 1)])
    art.paint(body | neck, dome(body | neck, 5.0) * 0.6, glass, base=0.62, spec=0.6, power=16)
    # The oil inside, up to two thirds, with its surface lit.
    level = top + (56 - top) * 0.3
    oil = body & (PY >= level) & arms._dilate(body) & ~(arms._dilate(~body))
    art.paint(oil, dome(oil, 4.0) * 0.6, _pal(liquid, 0.6, 1.2), base=0.5, spec=0.4, power=12, outline=False)
    shine = body & (np.abs(PX - (25 if shape != "tall" else 27)) < 1.2) & (PY > top + 3) & (PY < top + 14)
    art.tint(shine, glass, 14)
    cork = poly([(27, top - 15), (37, top - 15), (36.5, top - 8), (27.5, top - 8)])
    art.paint(cork, dome(cork, 2.0), _pal("#a87a48", 0.45, 1.05), base=0.5, albedo=(arms.value_noise(art.rng, 2) - 0.5) * 0.1)
    return art.done()


# ---------------------------------------------------------------- accessories


def shadow_cloak():
    art = Art("shadow_cloak")
    cloth = _pal("#26222e", 0.5, 1.1)
    hood = poly([(32, 6), (44, 12), (47, 24), (40, 30), (24, 30), (17, 24), (20, 12)])
    body = poly([(18, 24), (46, 24), (54, 58), (40, 54), (32, 58), (24, 54), (10, 58)])
    folds = np.sin(PX * 0.9) * 0.6
    art.paint(body, dome(body, 4.0) + folds, cloth, base=0.42, diff=1.3)
    art.paint(hood, dome(hood, 5.0), cloth, base=0.48, diff=1.3)
    face = ((PX - 32) ** 2 / 36 + (PY - 21) ** 2 / 42) <= 1
    art.tint(face & hood, cloth, 0)
    clasp = ((PX - 32) ** 2 + (PY - 31) ** 2) <= 2.6 ** 2
    art.paint(clasp, dome(clasp, 2.0), _pal("#9a8ab8", 0.6, 1.0), base=0.55, spec=0.5, power=14)
    return art.done()


def assassin_emblem():
    art = Art("assassin_emblem")
    disc = ((PX - 32) ** 2 + (PY - 32) ** 2) <= 24.0 ** 2
    inner = ((PX - 32) ** 2 + (PY - 32) ** 2) <= 18.0 ** 2
    art.paint(disc, dome(disc, 3.0), _pal("#5a4a6a", 0.6, 1.1), base=0.5, spec=0.35, power=12)
    art.paint(inner, dome(inner, 2.0) * 0.4, _pal("#2a2232", 0.45, 1.1), base=0.45, outline=False)
    for ang in (-45.0, 45.0):
        a = np.deg2rad(ang)
        d = np.array([np.cos(a), np.sin(a)])
        n = np.array([-d[1], d[0]])
        rx, ry = PX - 32, PY - 32
        U = rx * d[0] + ry * d[1]
        V = rx * n[0] + ry * n[1]
        m = (U > -6) & (U < 19) & (np.abs(V) < (19 - U) * 0.14 + 0.3)
        art.paint(m, (1 - np.abs(V) / 3.6).clip(0) * 2.5, mpal("silver"), base=0.7, spec=0.4, power=18)
        hilt = (U > -17) & (U < -6) & (np.abs(V) < 1.6)
        art.paint(hilt, np.ones_like(PX), LEATHER_GRIP, base=0.45)
    eye = ((PX - 32) ** 2 + (PY - 32) ** 2) <= 3.0 ** 2
    art.paint(eye, dome(eye, 2.0), _pal("#b03a5a", 0.6, 1.2), base=0.55, spec=0.6, power=12)
    return art.done()


def silent_soles():
    art = Art("silent_soles")
    leather = _pal("#5e4a3a", 0.5, 1.05)
    wool = _pal("#d8d2c4", 0.35, 1.0)
    for dx, dy in ((-9, -4), (8, 6)):
        shoe = poly([(18 + dx, 22 + dy), (30 + dx, 22 + dy), (32 + dx, 36 + dy), (46 + dx, 40 + dy), (47 + dx, 47 + dy),
                     (17 + dx, 47 + dy)])
        art.paint(shoe, dome(shoe, 4.0), leather, base=0.48, albedo=(arms.value_noise(art.rng, 2) - 0.5) * 0.1)
        cuff = poly([(17 + dx, 19 + dy), (31 + dx, 19 + dy), (31 + dx, 25 + dy), (17 + dx, 25 + dy)])
        art.paint(cuff, dome(cuff, 2.0) + (arms.value_noise(art.rng, 2) - 0.5) * 1.5, wool, base=0.55)
        sole = poly([(17 + dx, 45 + dy), (47 + dx, 45 + dy), (47 + dx, 49 + dy), (17 + dx, 49 + dy)])
        art.paint(sole, np.ones_like(PX), wool, base=0.4)
    return art.done()


def throwing_glove():
    art = Art("throwing_glove")
    leather = _pal("#6a4428", 0.5, 1.05)
    palm = poly([(20, 30), (42, 30), (44, 54), (18, 54)])
    fingers = np.zeros_like(palm)
    for i, x in enumerate((21.5, 27.5, 33.5, 39.5)):
        top = 10 + abs(i - 1.5) * 3
        fingers |= poly([(x - 2.6, 32), (x + 2.6, 32), (x + 2.4, top), (x - 2.4, top)])
    thumb = poly([(20, 44), (13, 34), (10, 28), (14, 26), (22, 36)])
    art.paint(fingers | palm | thumb, dome(fingers | palm | thumb, 3.0), leather, base=0.48,
              albedo=(arms.value_noise(art.rng, 2) - 0.5) * 0.1)
    strap = poly([(18, 46), (44, 46), (44, 51), (18, 51)])
    art.paint(strap, dome(strap, 1.5), _pal("#2a2228", 0.4), base=0.45)
    # Three little knives in the strap.
    for x in (24, 31, 38):
        k = poly([(x - 1.2, 47), (x + 1.2, 47), (x, 38)])
        art.paint(k, dome(k, 1.0), mpal("iron"), base=0.6, spec=0.4, power=18)
    return art.done()


def moonless_mark():
    art = Art("moonless_mark")
    cord = polyline([(14, 6), (32, 18), (50, 6)], 1.6)
    art.paint(cord, np.ones_like(PX) * 0.5, _pal("#3a3040", 0.4), base=0.5)
    moon = (((PX - 32) ** 2 + (PY - 38) ** 2) <= 18 ** 2) & ~(((PX - 39) ** 2 + (PY - 33) ** 2) <= 14 ** 2)
    art.paint(moon, dome(moon, 3.0), _pal("#2e2638", 0.55, 1.1), base=0.5, spec=0.45, power=16)
    rim = moon & ~arms._dilate(~moon | ~arms._dilate(moon))
    eye = ((PX - 24) ** 2 / 9 + (PY - 41) ** 2 / 4) <= 1
    art.paint(eye, dome(eye, 1.4), _pal("#a46aff", 0.6, 1.25), base=0.7, spec=0.6, power=10)
    return art.done()


ITEMS = {f"{m}_dagger": (lambda m=m: dagger(m)) for m in DAGGERS}
ITEMS["smoke_bomb"] = smoke_bomb
ITEMS["poison_oil"] = lambda: flask("poison_oil", "#6ac040", "round")
ITEMS["fire_oil"] = lambda: flask("fire_oil", "#e86a1a", "square")
ITEMS["frost_oil"] = lambda: flask("frost_oil", "#7ac8f0", "tall")
ITEMS["venom_oil"] = lambda: flask("venom_oil", "#8a3ac8", "drop")
ITEMS["shadow_cloak"] = shadow_cloak
ITEMS["assassin_emblem"] = assassin_emblem
ITEMS["silent_soles"] = silent_soles
ITEMS["throwing_glove"] = throwing_glove
ITEMS["moonless_mark"] = moonless_mark
