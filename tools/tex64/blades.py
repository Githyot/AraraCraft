"""A design of its own for every sword that was still the generic upscaled one (HP 2026-11-01:
"analise todas as espadas").

arms.py draws the swords of the 14 plain metals. The Terraria swords, the hardmode metals, the
mod swords and the greatswords were all one picture (a straight wide blade and a cross guard)
painted a different colour. Here each gets its own silhouette: the blade's profile (broad,
leaf, curved sabre, katana, needle, cleaver, wavy, toothed, notched, star-pointed), its guard
(wings, petals, shell, horns, tsuba, ring, shards) and pommel, and its own surface (honeycomb,
feathers, runes, hamon, veins, stars, facets, vines...). Magic swords keep a bright core so they
read as glowing.

Same frame and painting as arms.py: `u` runs from the pommel (0) to the tip (~74), `v` across.
"""

import numpy as np

from arms import (
    CORD, DARK_GRIP, LEATHER_GRIP, RED_GEM, VELVET, WOOD, Art, Frame, _dilate, _pal, blade, cracks,
    dome, grip, polyline, seg, streaks, tex_for,
)
from style import N, quantize, value_noise

ONES = np.ones((N, N))


def P(base, spread=0.7, sat=1.1, warm=0.14):
    return _pal(base, spread, sat, warm)


# Common palettes.
GOLD = P("#e0aa2a", 0.78, 1.25, 0.22)
STEEL = P("#b9bcc0", 0.78, 1.0)
DARK = P("#3a3640", 0.4, 1.0)
BONE = P("#ddd2b4", 0.55, 1.0)
GREEN_GEM = P("#2fbf5a", 0.7, 1.3)
BLUE_GEM = P("#2a6ae0", 0.7, 1.3)
PURPLE_GEM = P("#9a3ad8", 0.7, 1.3)


def metal(art, f, m, h, pal, base=0.52, spec=0.38, power=16, extra=None, kind="steel", diff=1.5):
    alb = tex_for(art, m, f, kind)
    if extra is not None:
        alb = alb + extra
    art.paint(m, h, pal, base=base, diff=diff, spec=spec, power=power, albedo=alb)


def glow(art, mask, field, pal):
    """Paint the pixels of `mask` straight from a bright palette (no shading): a glowing core."""
    m = mask & (art.img[..., 3] > 0)
    if not m.any():
        return
    col = quantize(np.clip(field, 0, 1), pal, 0.2, art.rng)
    art.img[m, :3] = col[m]


def part(art, mask, pal, r=1.5, base=0.48, spec=0.35, power=14, extra=None):
    alb = 0.0 if extra is None else extra
    art.paint(mask, dome(mask, r), pal, base=base, spec=spec, power=power, albedo=alb)


def star(f, u, v, r_out, r_in, n=5, rot=0.0):
    pts = []
    for i in range(n * 2):
        a = rot + i * np.pi / n
        r = r_out if i % 2 == 0 else r_in
        pts.append((u + np.cos(a) * r, v + np.sin(a) * r))
    return f.poly(pts)


def cross_guard(art, f, pal, u=20.6, half=10.0, w=1.6, dip=0.0, flare=0.0, base=0.44, spec=0.35):
    pts = [(u - w + dip, -half), (u - w, 0), (u - w + dip, half), (u + w + dip + flare, half + flare * 0.5),
           (u + w, 0), (u + w + dip + flare, -half - flare * 0.5)]
    g = f.poly(pts)
    part(art, g, pal, 1.4, base=base, spec=spec)
    return g


def pommel_disc(art, f, pal, u=3.8, r=3.4, boss=None):
    pm = f.disc(u, 0, r)
    h = dome(pm, 2.0)
    if boss is not None:
        h = h + f.disc(u, 0, boss) * 1.2
    art.paint(pm, h, pal, base=0.46, spec=0.4, power=14)
    return pm


def gem(art, f, u, v, r, pal):
    art.dots([f.xy(u, v)], r, pal, base=0.55)


# ---------------------------------------------------------------- pre-hardmode Terraria swords


def hive_blade(art, f):
    # Honey-amber blade with honeycomb cells, a striped grip and a pair of wing quillons.
    honey = P("#e8a020", 0.7, 1.3, 0.2)
    wax = P("#f2d27a", 0.55, 1.1)
    pm = f.disc(3.6, 0, 3.4, 3.0)
    art.paint(pm, dome(pm, 2.4), honey, base=0.5, spec=0.3)
    m = f.box(6.5, 19, -2.3, 2.3)
    stripes = (np.floor((f.U - 6.5) / 2.6) % 2) == 0
    art.paint(m, np.sqrt(np.clip(1 - (f.V / 2.3) ** 2, 0, 1)) * 2, DARK, base=0.45)
    art.paint(m & stripes, ONES, honey, base=0.55, outline=False)
    us = [21, 34, 54, 64, 72]
    mb, h, s, side = blade(f, us, [3.0, 4.6, 4.8, 3.2, 0.2], [3.0, 4.6, 4.8, 3.2, 0.2], hmax=2.4, edge=0.6)
    # Hex cells: distance to the nearest centre of a hex lattice.
    a, b = f.U / 6.4, f.V / 6.4
    q = a - b / np.sqrt(3)
    r = b * 2 / np.sqrt(3)
    cell = np.abs(((q + 0.5) % 1) - 0.5) + np.abs(((r + 0.5) % 1) - 0.5)
    walls = ((np.floor((f.U - 26) / 6.0) % 2) == 1) & (f.U > 26) & (f.U < 62)
    metal(art, f, mb, h, honey, base=0.55, spec=0.42, power=14, kind="gold")
    art.paint(mb & walls, h, P("#4a2a10", 0.4), base=0.5, diff=1.2, outline=False)
    for sgn in (-1, 1):
        wg = f.poly([(21.4, sgn * 1.4), (18.8, sgn * 5.4), (21.6, sgn * 12.4), (25.8, sgn * 10.8), (24.4, sgn * 2.0)])
        art.paint(wg, dome(wg, 1.2) + streaks(art.rng, f.V, f.U, 2.4) * 0.3, wax, base=0.62, spec=0.2)
    core = f.box(19.2, 22.8, -2.4, 2.4)
    part(art, core, DARK, 1.0)


def sky_blade(art, f):
    # A feather for a blade: barbs angled toward the tip, a white quill and wing guard.
    sky = P("#8ec8f0", 0.65, 1.0)
    white = P("#eef2f6", 0.4, 1.0)
    gem(art, f, 3.6, 0, 2.6, BLUE_GEM)
    grip(art, f, 6.4, 19, 2.1, P("#3c4a6a", 0.4), "spiral", base=0.5)
    us = [21, 30, 52, 66, 74]
    mb, h, s, side = blade(f, us, [2.0, 4.6, 5.4, 3.6, 0.2], [2.0, 4.6, 5.4, 3.6, 0.2], hmax=1.6, edge=0.0)
    barbs = np.abs(((f.U - np.abs(f.V) * 1.2) / 2.2) % 1 - 0.5) < 0.16
    quill = np.abs(f.V) < 0.7
    art.paint(mb, h, sky, base=0.6, diff=1.2, spec=0.15, albedo=barbs * -0.14 + quill * 0.25)
    for sgn in (-1, 1):
        wg = f.poly([(22.4, sgn * 1.5), (19.0, sgn * 6.0), (17.4, sgn * 11.6), (20.8, sgn * 10.0), (21.4, sgn * 12.8), (24.4, sgn * 7.0)])
        art.paint(wg, dome(wg, 1.2), white, base=0.6, spec=0.15, albedo=streaks(art.rng, f.U, f.V, 2.0) * 0.2)


def umbral_sword(art, f):
    # Dark purple, both edges toothed like a saw, an eye set in a thorny guard.
    pal = P("#8a62b8", 0.7, 1.2)
    pm = f.poly([(0, 0), (3.6, -3.6), (6.4, -2.0), (6.4, 2.0), (3.6, 3.6)])
    part(art, pm, pal, 1.6)
    grip(art, f, 6.4, 19, 2.2, DARK_GRIP, "bands", base=0.45)
    us = np.linspace(21, 72, 90)
    base_w = np.interp(us, [21, 30, 60, 72], [3.2, 4.2, 3.8, 0.2])
    teeth = (((us - 24) % 10.0) / 10.0) * 4.2 * (us > 24) * (us < 64)
    mb, h, s, side = blade(f, us, base_w + teeth, base_w, ridge=-0.3, hmax=2.6)
    metal(art, f, mb, h, pal, base=0.52, spec=0.35, kind="dark")
    for sgn in (-1, 1):
        th = f.poly([(19.6, sgn * 2), (24.6, sgn * 9.6), (21.6, sgn * 6.6), (19.2, sgn * 11), (18.6, sgn * 3)])
        part(art, th, pal, 1.2, base=0.44)
    eye = f.disc(21, 0, 3.0, 3.6)
    art.paint(eye, dome(eye, 1.4), P("#e0e0d0", 0.4), base=0.65)
    glow(art, f.disc(21, 0, 1.8, 2.0), ONES * 0.7, P("#c060ff", 0.5, 1.4))


def sanguinite_sword(art, f):
    # A butcher's blade of crimson flesh-steel with dark veins, a jaw for a guard.
    pal = P("#b02838", 0.62, 1.2)
    rng = art.rng
    pm = f.disc(3.6, 0, 3.0)
    art.paint(pm, dome(pm, 2.4), BONE, base=0.55)
    grip(art, f, 6.0, 19, 2.3, P("#4a1a20", 0.35), "spiral", base=0.45)
    us = [21, 26, 60, 66, 70]
    mb, h, s, side = blade(f, us, [3.0, 3.4, 3.6, 3.6, 0.2], [3.6, 6.4, 6.8, 2.6, 0.2], ridge=-0.7, hmax=2.4)
    veins = cracks(art, mb, rng, 4, 1.6) > 0
    metal(art, f, mb, h, pal, base=0.5, spec=0.3, kind="dark", extra=veins * -0.25)
    jaw = f.box(17.6, 22.2, -8.4, 9.0)
    part(art, jaw, BONE, 1.2, base=0.55, spec=0.1)
    teeth = ((np.floor(f.V / 2.6) % 2) == 0) & f.box(22.2, 25.0, -8.0, 8.6)
    art.paint(teeth, ONES, BONE, base=0.75, outline=False)


def enchanted_sword(art, f):
    # Blue steel, a broad fuller lit with runes, a gold guard with a blue gem.
    pal = P("#4a7ad0", 0.72, 1.1)
    pommel_disc(art, f, GOLD, r=3.0)
    grip(art, f, 6.4, 19, 2.2, BLUE_GEM, "cross", base=0.4)
    us = [21, 60, 68, 74]
    mb, h, s, side = blade(f, us, [3.6, 3.4, 2.4, 0.2], [3.6, 3.4, 2.4, 0.2], hmax=2.6)
    fuller = (f.U > 24) & (f.U < 58) & (np.abs(f.V) < 1.5)
    metal(art, f, mb, h - fuller * 1.2, pal, base=0.52, spec=0.45, power=20)
    runes = fuller & ((np.floor((f.U - 24) / 4.0) % 2) == 0)
    glow(art, fuller & mb, ONES * 0.45, P("#5ac8ff", 0.5, 1.2))
    glow(art, runes & mb, ONES * 0.85, P("#b8f8ff", 0.4, 1.2))
    cross_guard(art, f, GOLD, half=9.4, dip=1.2, flare=1.0)
    gem(art, f, 20.8, 0, 1.6, BLUE_GEM)


def star_saber(art, f):
    # A slim gold-and-sky sabre that ends in a star; a star pommel.
    pal = P("#5a8ae8", 0.7, 1.1)
    pm = star(f, 3.6, 0, 4.2, 1.9, rot=np.pi)
    part(art, pm, GOLD, 1.4, base=0.55, spec=0.45)
    grip(art, f, 7, 19, 2.0, P("#2a2a5a", 0.4), "spiral", base=0.45)
    us = np.linspace(21, 66, 30)
    center = -((us - 21) / 45) ** 2 * 2.6
    mb, h, s, side = blade(f, us, np.interp(us, [21, 60, 66], [2.0, 1.8, 0.4]), np.interp(us, [21, 60, 66], [2.6, 2.4, 0.4]), center=center, ridge=-0.4, hmax=1.8)
    metal(art, f, mb, h, pal, base=0.55, spec=0.45, power=20)
    tip = star(f, 68.0, -2.4, 6.0, 2.4, rot=0.3)
    art.paint(tip, dome(tip, 1.6), GOLD, base=0.62, spec=0.4)
    glow(art, f.disc(68.0, -2.4, 1.6), ONES * 0.9, P("#fff2a0", 0.4))
    g = f.disc(20.6, 0, 1.6, 6.4)
    part(art, g, GOLD, 1.2)


def blue_katana(art, f):
    # A long katana with a cold blue blade, a square tsuba and a blue-black ito grip.
    pal = P("#3a7ae8", 0.72, 1.2)
    art.paint(f.box(0.6, 2.8, -2.3, 2.3), dome(f.box(0.6, 2.8, -2.3, 2.3), 1), DARK, base=0.45)
    gm = f.box(2.6, 19.2, -2.2, 2.2)
    art.paint(gm, ONES, P("#1c2a4a", 0.4), base=0.45)
    k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.0) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.0) % 1 - 0.5))
    art.paint(gm & (k < 0.18), ONES, P("#5a8ae0", 0.4), base=0.6, outline=False)
    us = np.linspace(22.6, 76, 30)
    center = -((us - 22.6) / 53.4) ** 2 * 4.6
    mb, h, s, side = blade(f, us, np.interp(us, [22.6, 70, 76], [1.4, 1.3, 0.2]), np.interp(us, [22.6, 68, 73, 76], [2.2, 2.0, 1.8, 0.2]),
                           center=center, ridge=-0.4, hmax=1.6, edge=0.8)
    hamon = (s > 0.3 + 0.4 * np.abs(np.sin(f.U * 0.8))) & (f.U < 72)
    metal(art, f, mb, h, pal, base=0.5, spec=0.5, power=22, extra=hamon * 0.2)
    ts = f.box(19.6, 22.0, -5.2, 5.2)
    part(art, ts, DARK, 1.0, base=0.45, spec=0.3)
    glow(art, mb & hamon, ONES * 0.6, P("#a8d8ff", 0.4))


def night_edge(art, f):
    # Purple-black blade with a ragged, flame-like edge and a green heart.
    pal = P("#4a2a6a", 0.6, 1.2)
    rng = art.rng
    pm = f.poly([(0.0, 0.0), (3.0, -3.8), (6.6, -2.2), (5.2, 0.0), (6.6, 2.2), (3.0, 3.8)])
    part(art, pm, pal, 1.4)
    grip(art, f, 6.6, 19, 2.2, DARK_GRIP, "cross", base=0.45)
    us = np.linspace(21, 74, 80)
    w = np.interp(us, [21, 32, 58, 68, 74], [3.0, 4.6, 4.8, 3.0, 0.2])
    rag = np.abs(np.sin(us * 0.5)) * 2.4 * (us > 30) * (us < 68)
    mb, h, s, side = blade(f, us, w + rag, w + np.abs(np.sin(us * 0.5 + 1.4)) * 2.4 * (us > 30) * (us < 68), hmax=2.6)
    metal(art, f, mb, h, pal, base=0.5, spec=0.35, kind="dark")
    core = mb & (np.abs(f.V) < 0.9) & (f.U > 26) & (f.U < 62)
    glow(art, core, 0.5 + 0.5 * np.sin(f.U * 0.4), P("#40d870", 0.5, 1.3))
    for sgn in (-1, 1):
        hn = polyline([f.xy(21.0, 0), f.xy(22.6, sgn * 5.4), f.xy(26.4, sgn * 9.0)], 2.6)
        part(art, hn, pal, 1.2, base=0.45)


# ---------------------------------------------------------------- hardmode Terraria swords


def radiant_blade(art, f):
    # The holy blade: broad white-gold, a long fuller, wings swept up for a guard, a sun pommel.
    pal = P("#f0e6b8", 0.6, 1.1, 0.2)
    sun = star(f, 3.6, 0, 4.4, 2.6, n=8)
    part(art, sun, GOLD, 1.4, base=0.55, spec=0.45)
    grip(art, f, 7.4, 19, 2.3, P("#3a5ab0", 0.4), "cross", base=0.45)
    us = [22, 28, 58, 66, 74]
    mb, h, s, side = blade(f, us, [3.6, 4.4, 4.4, 3.0, 0.2], [3.6, 4.4, 4.4, 3.0, 0.2], hmax=2.8)
    fuller = (f.U > 25) & (f.U < 60) & (np.abs(f.V) < 1.0)
    metal(art, f, mb, h - fuller * 1.4, pal, base=0.58, spec=0.5, power=20, kind="gold")
    glow(art, fuller & mb, ONES * 0.85, P("#fff8d0", 0.3))
    for sgn in (-1, 1):
        wg = f.poly([(20, sgn * 1.4), (21, sgn * 8), (26.6, sgn * 13.0), (24.2, sgn * 8.2), (28.0, sgn * 9.6), (23.6, sgn * 2.0)])
        part(art, wg, GOLD, 1.2, base=0.52, spec=0.45)
    gem(art, f, 21.6, 0, 1.6, BLUE_GEM)


def true_radiant_blade(art, f):
    radiant_blade(art, f)
    # Grown: a second, brighter edge and a halo where the blade meets the guard.
    tip = f.box(62, 75, -1.2, 1.2) & (art.img[..., 3] > 0)
    glow(art, tip, ONES * 0.9, P("#fff2b0", 0.3))
    ring = f.disc(26, 0, 4.2) & ~f.disc(26, 0, 3.0)
    art.paint(ring & ~(art.img[..., 3] > 0), ONES, P("#ffe890", 0.4), base=0.75, outline=False)


def true_night_edge(art, f):
    night_edge(art, f)
    vein = (art.img[..., 3] > 0) & (np.abs(np.abs(f.V) - 2.6 - 0.6 * np.sin(f.U * 0.7)) < 0.5) & (f.U > 28) & (f.U < 66)
    glow(art, vein, ONES * 0.8, P("#6af08a", 0.4, 1.3))


def terra_blade(art, f):
    # Long green blade with a notch near the tip, a chunky hilt with a green crystal.
    pal = P("#3aa84a", 0.7, 1.2)
    pm = f.poly([(0.4, 0), (3.6, -3.6), (6.6, -2.4), (6.6, 2.4), (3.6, 3.6)])
    part(art, pm, STEEL, 1.4)
    grip(art, f, 6.6, 19, 2.3, P("#2a3a2a", 0.4), "bands", base=0.45)
    us = np.linspace(22, 76, 80)
    lw = np.interp(us, [22, 30, 54, 58, 60, 70, 76], [3.0, 4.6, 4.6, 0.8, 4.8, 3.2, 0.2])
    rw = np.interp(us, [22, 30, 64, 70, 76], [3.0, 4.2, 4.0, 2.6, 0.2])
    mb, h, s, side = blade(f, us, lw, rw, hmax=2.6)
    metal(art, f, mb, h, pal, base=0.52, spec=0.42, power=18)
    core = mb & (np.abs(f.V + 0.4 * np.sin(f.U * 0.35)) < 1.0) & (f.U > 26)
    glow(art, core, 0.6 + 0.3 * np.sin(f.U * 0.5), P("#9cff8a", 0.4, 1.2))
    g = f.poly([(18.8, -9.6), (22.2, -7.0), (24.2, 0), (22.2, 7.0), (18.8, 9.6), (20.4, 0)])
    part(art, g, STEEL, 1.4)
    gem(art, f, 21.6, 0, 1.8, GREEN_GEM)


def horseman_blade(art, f):
    # A wide, dark orange blade curling like a harvest scythe, a pumpkin pommel and a vine guard.
    pal = P("#e8862a", 0.7, 1.3, 0.2)
    pump = f.disc(3.8, 0, 3.6, 4.0)
    ribs = np.abs(np.sin(f.V * 1.3)) * 0.8
    art.paint(pump, dome(pump, 2.6) + ribs, P("#f08a1a", 0.6, 1.3), base=0.52)
    grip(art, f, 7.2, 19, 2.2, P("#3a2a18", 0.4), "spiral", base=0.45)
    us = np.linspace(21, 72, 50)
    t = np.clip((us - 21) / 51, 0, 1)
    center = -np.sin(t * np.pi * 0.85) * 6.0
    mb, h, s, side = blade(f, us, np.interp(us, [21, 50, 72], [2.4, 3.0, 0.2]), np.interp(us, [21, 40, 60, 72], [3.6, 7.4, 6.6, 0.2]), center=center, ridge=-0.6, hmax=2.6)
    metal(art, f, mb, h, pal, base=0.55, spec=0.3, kind="dull")
    glow(art, mb & (s > side - 1.8), ONES * 0.75, P("#ffd060", 0.4))
    vine = polyline([f.xy(20, -8), f.xy(21.6, -3), f.xy(20.6, 2), f.xy(22.6, 6.6), f.xy(25.6, 8.4)], 3.0)
    part(art, vine, P("#3a7a2a", 0.5), 1.0, base=0.45, spec=0.05)


def cutlass(art, f):
    # A pirate's curved cutlass with a gold shell guard and a knuckle bow.
    pal = STEEL
    art.paint(f.disc(4.0, 0, 2.6), dome(f.disc(4.0, 0, 2.6), 2), GOLD, base=0.5, spec=0.45)
    grip(art, f, 6.4, 18.6, 2.0, P("#5a2a1a", 0.4), "spiral", base=0.45)
    us = np.linspace(21, 70, 40)
    center = -((us - 21) / 49) ** 1.6 * 6.0
    mb, h, s, side = blade(f, us, np.interp(us, [21, 62, 70], [1.6, 1.8, 0.2]), np.interp(us, [21, 50, 62, 70], [3.2, 4.6, 4.2, 0.2]), center=center, ridge=-0.7, hmax=2.2)
    metal(art, f, mb, h, pal, base=0.55, spec=0.5, power=22)
    shell = f.disc(21.2, 1.2, 3.2, 6.6) & (f.U < 24.4)
    flutes = np.abs(np.sin(np.arctan2(f.V - 1.2, f.U - 18) * 7)) * 0.7
    art.paint(shell, dome(shell, 2) + flutes, GOLD, base=0.5, spec=0.45)
    bow = polyline([f.xy(20.2, 6.0), f.xy(16, 7.6), f.xy(9, 6.8), f.xy(5.2, 2.6)], 1.6)
    part(art, bow, GOLD, 1.0, base=0.5)


def prism_saber(art, f):
    # A thin crystal blade split into rainbow bands, a silver hilt and a prism for a pommel.
    rainbow = np.array([[0.95, 0.3, 0.35], [1.0, 0.6, 0.2], [1.0, 0.92, 0.3], [0.4, 0.95, 0.45], [0.3, 0.7, 1.0], [0.6, 0.4, 1.0]]) * 255
    pm = f.poly([(0.4, 0), (3.8, -3.4), (7.0, 0), (3.8, 3.4)])
    art.paint(pm, ONES, P("#d8e8ff", 0.5), base=0.6, diff=0.0, albedo=np.where(f.V < 0, 0.15, -0.1))
    grip(art, f, 7, 19, 2.0, STEEL, "bands", base=0.45)
    us = [21, 66, 74]
    mb, h, s, side = blade(f, us, [2.2, 2.0, 0.2], [2.2, 2.0, 0.2], hmax=1.6, edge=0.0)
    art.paint(mb, h * 0.3, P("#e8f0ff", 0.4), base=0.7, diff=0.0)
    band = np.clip(((f.U - f.V * 2.0) / 3.4).astype(int) % 6, 0, 5)
    inner = mb & (np.abs(f.V) < side - 0.6)
    art.img[inner, :3] = rainbow[band[inner]]
    g = f.disc(20.6, 0, 1.4, 5.0)
    part(art, g, STEEL, 1.2, base=0.5, spec=0.5)


def nebula_blade(art, f):
    # A curved blade of night sky: deep violet with pink clouds and pinpoint stars.
    pal = P("#4a2a8a", 0.55, 1.3)
    pm = star(f, 3.6, 0, 3.8, 1.6, n=4)
    part(art, pm, P("#e080e0", 0.5, 1.2), 1.2, base=0.6)
    grip(art, f, 6.8, 19, 2.1, DARK_GRIP, "spiral", base=0.45)
    us = np.linspace(21, 74, 40)
    center = -((us - 21) / 53) ** 2 * 3.6
    mb, h, s, side = blade(f, us, np.interp(us, [21, 64, 74], [2.4, 3.0, 0.2]), np.interp(us, [21, 50, 66, 74], [3.0, 4.4, 3.6, 0.2]), center=center, hmax=2.0)
    cloud = value_noise(art.rng, 8) * 0.6 + value_noise(art.rng, 4) * 0.4
    art.paint(mb, h, pal, base=0.45, diff=1.0, spec=0.2, albedo=(cloud - 0.5) * 0.3)
    pink = mb & (cloud > 0.62)
    glow(art, pink, cloud, P("#ff70d0", 0.5, 1.2))
    stars = mb & (art.rng.random((N, N)) > 0.95)
    art.img[stars, :3] = [255, 250, 230]
    for sgn in (-1, 1):
        fin = f.poly([(19.6, sgn * 1.6), (23.4, sgn * 9.4), (24.4, sgn * 2.0)])
        part(art, fin, P("#8a5ad8", 0.5, 1.2), 1.0, base=0.55)


def celestial_wrath(art, f):
    # Wide magenta blade whose tip is a five-pointed star, a crescent guard.
    pal = P("#c040c0", 0.65, 1.2)
    pm = f.disc(3.6, 0, 3.2)
    part(art, pm, GOLD, 2.0)
    grip(art, f, 6.6, 19, 2.2, P("#3a1a4a", 0.4), "cross", base=0.45)
    us = [21, 28, 54, 60]
    mb, h, s, side = blade(f, us, [3.0, 4.6, 4.4, 1.4], [3.0, 4.6, 4.4, 1.4], hmax=2.6)
    metal(art, f, mb, h, pal, base=0.52, spec=0.4)
    st = star(f, 64, 0, 9.0, 3.8, rot=0.0)
    art.paint(st, dome(st, 2.0), P("#ff8af0", 0.55, 1.2), base=0.58, spec=0.4)
    glow(art, f.disc(64, 0, 2.0) | (mb & (np.abs(f.V) < 0.8)), ONES * 0.85, P("#ffe0ff", 0.3))
    cr = f.disc(20.4, 0, 3.2, 9.4) & ~f.disc(17.6, 0, 3.0, 8.0)
    part(art, cr, GOLD, 1.2)


def apex_blade(art, f):
    # The last sword: a layered blade of every earlier one, coloured bands from hilt to tip, a
    # white-hot edge, and a ring guard.
    cols = ["#8a6438", "#b9bcc0", "#4fd6cf", "#9a3ad8", "#3aa84a", "#f0e6b8", "#c040c0"]
    gem(art, f, 3.6, 0, 3.0, P("#e8f0ff", 0.5))
    grip(art, f, 6.6, 19, 2.2, DARK_GRIP, "cross", base=0.45)
    us = [21, 30, 60, 68, 76]
    mb, h, s, side = blade(f, us, [3.2, 4.4, 4.2, 2.8, 0.2], [3.2, 4.4, 4.2, 2.8, 0.2], hmax=2.6)
    k = np.clip(((f.U - 21) / 55 * len(cols)).astype(int), 0, len(cols) - 1)
    for i, c in enumerate(cols):
        m = mb & (k == i)
        art.paint(m, h, P(c, 0.6, 1.2), base=0.55, spec=0.4, power=16, outline=False)
    glow(art, mb & (np.abs(f.V) < 0.5) & (f.U > 60), ONES * 0.9, P("#ffffff", 0.2))
    rg = f.disc(21.0, 0, 4.6) & ~f.disc(21.0, 0, 2.6)
    part(art, rg, P("#d8e0ff", 0.6), 1.2, base=0.55, spec=0.5)


def betsy_wrath(art, f):
    # A dragon's blade: red-orange scales, a fang edge, flame glow along the spine.
    pal = P("#c83a1a", 0.65, 1.25, 0.2)
    pm = f.poly([(0, 0), (4, -3), (7, -1.4), (7, 1.4), (4, 3)])
    part(art, pm, pal, 1.4)
    grip(art, f, 7, 19, 2.2, P("#3a1a10", 0.4), "bands", base=0.45)
    us = np.linspace(21, 72, 60)
    rw = np.interp(us, [21, 40, 60, 72], [3.0, 5.0, 4.0, 0.2])
    fangs = (((us - 26) % 6.0) / 6.0) ** 2 * 2.0 * (us > 26) * (us < 62)
    mb, h, s, side = blade(f, us, np.interp(us, [21, 64, 72], [2.4, 2.4, 0.2]), rw + fangs, ridge=-0.6, hmax=2.4)
    sc = np.abs(((f.U * 0.5 + np.abs(f.V)) % 2.2) - 1.1) < 0.3
    metal(art, f, mb, h, pal, base=0.5, spec=0.3, extra=sc * -0.12)
    glow(art, mb & (f.V < -0.8) & (f.V > -2.0), ONES * 0.8, P("#ffb030", 0.4, 1.3))
    for sgn in (-1, 1):
        hn = polyline([f.xy(21, 0), f.xy(20.2, sgn * 5), f.xy(17.2, sgn * 9.0)], 2.4)
        part(art, hn, BONE, 1.0)


def razorpine(art, f):
    # A blade of packed pine needles: dark green, branch quillons, a pine cone pommel.
    pal = P("#2a6a3a", 0.6, 1.1)
    cone = f.disc(3.8, 0, 3.6, 3.0)
    sc = np.abs(((f.U + np.abs(f.V)) % 2.0) - 1.0) < 0.3
    art.paint(cone, dome(cone, 2.4), P("#7a4a22", 0.55), base=0.5, albedo=sc * -0.2)
    grip(art, f, 7, 19, 2.1, P("#5a3a1e", 0.4), "plain", base=0.45)
    us = [21, 34, 60, 72]
    mb, h, s, side = blade(f, us, [2.6, 4.4, 3.6, 0.2], [2.6, 4.4, 3.6, 0.2], hmax=2.0, edge=0.0)
    needles = np.abs(((f.U - np.abs(f.V) * 0.9) / 1.6) % 1 - 0.5) < 0.18
    art.paint(mb, h, pal, base=0.5, diff=1.3, spec=0.1, albedo=needles * 0.16)
    snow = mb & (f.V < -side + 1.0) & (value_noise(art.rng, 4) > 0.5)
    art.img[snow, :3] = [236, 244, 250]
    for sgn in (-1, 1):
        br = polyline([f.xy(20.4, 0), f.xy(22.4, sgn * 6), f.xy(24.8, sgn * 9.6)], 2.0)
        part(art, br, P("#6a4a2a", 0.5), 1.0, base=0.45, spec=0.05)


# ---------------------------------------------------------------- hardmode metals (Etapa 5)


def cobalt_sword(art, f):
    # Deep blue, broad and squared like a cleaver-tipped short sword, a heavy bar guard.
    pal = P("#2a5ad0", 0.72, 1.2)
    pm = f.box(0.6, 6.2, -3.0, 3.0)
    part(art, pm, pal, 1.4)
    grip(art, f, 6.2, 19, 2.3, DARK_GRIP, "bands", base=0.45)
    us = [21, 58, 64, 67]
    mb, h, s, side = blade(f, us, [4.4, 4.6, 4.6, 0.6], [4.4, 4.6, 2.0, 0.6], hmax=2.6)
    bevel = (np.abs(f.V) > 2.2) * 0.1
    metal(art, f, mb, h, pal, base=0.52, spec=0.4, extra=bevel)
    cross_guard(art, f, pal, half=8.0, w=2.2)


def palladium_sword(art, f):
    # Orange-red with a ring guard and a long leaf blade with two fullers.
    pal = P("#e0602a", 0.7, 1.2, 0.18)
    pommel_disc(art, f, pal, r=3.0, boss=1.2)
    grip(art, f, 6.4, 19, 2.1, P("#3a2420", 0.4), "cross", base=0.45)
    us = [22, 28, 50, 62, 72]
    mb, h, s, side = blade(f, us, [2.4, 3.0, 4.6, 3.8, 0.2], [2.4, 3.0, 4.6, 3.8, 0.2], hmax=2.6)
    full = (np.abs(np.abs(f.V) - 1.6) < 0.5) & (f.U > 26) & (f.U < 58)
    metal(art, f, mb, h - full * 1.0, pal, base=0.52, spec=0.4)
    rg = f.disc(21.0, 0, 4.4) & ~f.disc(21.0, 0, 2.4)
    part(art, rg, pal, 1.2)


def mythril_sword(art, f):
    # Pale green elven blade: slender, a long leaf tip, guard of two swept leaves.
    pal = P("#5ad0a0", 0.7, 1.05)
    gem(art, f, 3.6, 0, 2.6, GREEN_GEM)
    grip(art, f, 6.4, 19.6, 1.9, P("#d8d0b8", 0.4), "spiral", base=0.5)
    us = [21, 40, 58, 66, 76]
    mb, h, s, side = blade(f, us, [1.8, 2.2, 3.4, 3.0, 0.2], [1.8, 2.2, 3.4, 3.0, 0.2], hmax=2.0, edge=0.8)
    vein = (np.abs(f.V) < 0.5) & (f.U > 24)
    metal(art, f, mb, h, pal, base=0.56, spec=0.5, power=22, kind="polish", extra=vein * 0.2)
    for sgn in (-1, 1):
        lf = f.poly([(21.6, sgn * 1.0), (24.6, sgn * 4.8), (30.6, sgn * 6.0), (25.6, sgn * 2.0)])
        part(art, lf, pal, 1.0, base=0.55, spec=0.4)
        lf2 = f.poly([(20.2, sgn * 1.0), (17.6, sgn * 4.6), (13.6, sgn * 6.4), (17.4, sgn * 2.0)])
        part(art, lf2, pal, 1.0, base=0.5, spec=0.4)


def orichalcum_sword(art, f):
    # Rose-pink blade that widens like a petal, a flower guard with a gold heart.
    pal = P("#e060b0", 0.7, 1.15)
    pm = star(f, 3.6, 0, 3.6, 2.2, n=5)
    part(art, pm, pal, 1.4, base=0.55)
    grip(art, f, 6.8, 19, 2.0, P("#4a2440", 0.4), "spiral", base=0.45)
    us = [22, 30, 48, 62, 73]
    mb, h, s, side = blade(f, us, [2.0, 2.6, 4.6, 4.0, 0.2], [2.0, 2.6, 4.6, 4.0, 0.2], hmax=2.4)
    metal(art, f, mb, h, pal, base=0.54, spec=0.45, power=18)
    for k in range(5):
        a = k * 2 * np.pi / 5 + 0.3
        pt = f.disc(20.6 + np.cos(a) * 3.4, np.sin(a) * 3.4, 2.6)
        part(art, pt, P("#f08ad0", 0.6, 1.1), 1.2, base=0.58, spec=0.2)
    gem(art, f, 20.6, 0, 1.8, GOLD)


def adamantite_sword(art, f):
    # Blood-red, heavy and angular: a thick blade with a stepped spine and spiked guard.
    pal = P("#c02a30", 0.66, 1.2)
    pm = f.poly([(-0.4, 0), (3, -3.6), (6.6, -3.0), (6.6, 3.0), (3, 3.6)])
    part(art, pm, pal, 1.4)
    grip(art, f, 6.6, 19, 2.4, DARK_GRIP, "bands", base=0.45)
    us = [21, 30, 44, 46, 62, 70, 74]
    lw = [4.0, 4.6, 4.6, 3.6, 3.6, 2.6, 0.2]
    rw = [4.0, 5.2, 5.2, 5.2, 4.6, 2.4, 0.2]
    mb, h, s, side = blade(f, us, lw, rw, ridge=-0.4, hmax=3.0)
    facet = np.where(s < 0, 0.1, -0.08)
    metal(art, f, mb, h, pal, base=0.5, spec=0.4, extra=facet)
    for sgn in (-1, 1):
        sp = f.poly([(19.0, sgn * 2), (21.0, sgn * 12.4), (23.4, sgn * 2)])
        part(art, sp, pal, 1.0, base=0.45)
    core = f.box(18.8, 23.0, -2.6, 2.6)
    part(art, core, DARK, 1.0)


def titanium_sword(art, f):
    # Cool grey, geometric: a blade cut in long flat facets, an angular double guard.
    pal = P("#9aa4b0", 0.75, 1.0)
    pm = f.poly([(0.4, 0), (3.4, -3.0), (6.4, 0), (3.4, 3.0)])
    part(art, pm, pal, 1.0, base=0.55)
    grip(art, f, 6.4, 19, 2.1, P("#2a3040", 0.4), "cross", base=0.45)
    us = [21, 56, 66, 75]
    mb, h, s, side = blade(f, us, [3.0, 3.4, 3.0, 0.2], [3.0, 3.4, 3.0, 0.2], hmax=0.6, edge=0.0)
    facet = np.where(s < -1.2, 0.18, np.where(s < 0, 0.06, np.where(s < 1.2, -0.06, -0.16)))
    metal(art, f, mb, h, pal, base=0.55, spec=0.55, power=24, kind="polish", extra=facet)
    g = f.poly([(18.6, -8.6), (22.6, -10.6), (23.4, -2.4), (25.6, 0), (23.4, 2.4), (22.6, 10.6), (18.6, 8.6), (20.4, 0)])
    part(art, g, pal, 1.2, base=0.48, spec=0.5)


def radiant_sword(art, f):
    # The holy metal (hallowed bars): a white-gold blade with a rising sun guard.
    pal = P("#f2dc8a", 0.66, 1.15, 0.2)
    pommel_disc(art, f, GOLD, r=3.2)
    grip(art, f, 6.4, 19, 2.2, P("#f2f2f8", 0.4), "spiral", base=0.5)
    us = [22, 60, 68, 74]
    mb, h, s, side = blade(f, us, [3.0, 3.2, 2.4, 0.2], [3.0, 3.2, 2.4, 0.2], hmax=2.4)
    metal(art, f, mb, h, pal, base=0.58, spec=0.5, power=20, kind="gold")
    rays = (np.abs(np.sin(np.arctan2(f.V, f.U - 18.0) * 6)) > 0.5) & f.disc(18.0, 0, 7.0) & (f.U > 18.4)
    sun = f.disc(18.0, 0, 7.0) & (f.U > 18.4) & ~f.disc(18.0, 0, 3.0)
    art.paint(sun, dome(sun, 1.4), GOLD, base=0.5, spec=0.45, albedo=rays * 0.15)


def verdite_sword(art, f):
    # Chlorophyte: living green with leaf veins and a crown of leaves at the guard.
    pal = P("#4ac03a", 0.7, 1.2)
    rng = art.rng
    pm = f.disc(3.8, 0, 3.4, 2.8)
    part(art, pm, P("#7a5a30", 0.5), 1.6, base=0.5, spec=0.1)
    grip(art, f, 7, 19, 2.1, P("#3a5a2a", 0.4), "spiral", base=0.45)
    us = [21, 34, 58, 72]
    mb, h, s, side = blade(f, us, [2.6, 4.2, 4.2, 0.2], [2.6, 4.2, 4.2, 0.2], hmax=2.4)
    veins = ((np.abs(f.V) < 0.4) | (np.abs(((f.U - np.abs(f.V) * 1.4) % 6.0) - 3.0) < 0.35)) & (f.U > 24)
    metal(art, f, mb, h, pal, base=0.54, spec=0.35, extra=veins * 0.2, kind="dull")
    for k, sgn in enumerate((-1, 1, -1, 1)):
        o = 0 if k < 2 else 3.0
        lf = f.poly([(20.4 - o, sgn * 1.2), (21.6 - o, sgn * 6.0), (25.6 - o, sgn * 9.2), (23.6 - o, sgn * 2.4)])
        part(art, lf, P("#3a9a2a", 0.6, 1.2), 1.0, base=0.5, spec=0.1)


def lumite_sword(art, f):
    # Luminite: dark teal stone shot with glowing cyan, a crescent-moon blade edge and a
    # floating shard guard.
    pal = P("#1f5a5a", 0.55, 1.2)
    pm = f.disc(3.8, 0, 3.4)
    art.paint(pm, dome(pm, 2.4), pal, base=0.5, spec=0.3)
    glow(art, f.disc(3.8, 0, 1.6), ONES * 0.8, P("#7af8f0", 0.4))
    grip(art, f, 7, 19, 2.2, DARK_GRIP, "bands", base=0.45)
    us = np.linspace(21, 76, 50)
    center = np.sin(np.clip((us - 21) / 55, 0, 1) * np.pi) * -2.0
    mb, h, s, side = blade(f, us, np.interp(us, [21, 36, 64, 76], [2.8, 3.6, 4.6, 0.2]), np.interp(us, [21, 40, 66, 76], [3.0, 4.2, 3.4, 0.2]), center=center, hmax=2.6)
    metal(art, f, mb, h, pal, base=0.5, spec=0.4, kind="dark")
    veins = cracks(art, mb, art.rng, 7, 0.9)
    glow(art, veins > 0, veins, P("#5af0e0", 0.5, 1.3))
    glow(art, mb & (s > side - 1.0), ONES * 0.7, P("#9afcf4", 0.3))
    for sgn in (-1, 1):
        sh = f.poly([(19.0, sgn * 3.0), (23.6, sgn * 4.6), (21.0, sgn * 10.6), (18.0, sgn * 7.0)])
        part(art, sh, P("#3a9a9a", 0.6, 1.2), 1.0, base=0.55, spec=0.5)


def vine_blade(art, f):
    # Jungle: a green blade wound with a thorny vine, a flower at the pommel.
    pal = P("#5a8a2a", 0.66, 1.2)
    flower = star(f, 3.6, 0, 4.0, 2.2, n=6)
    part(art, flower, P("#e04a6a", 0.6, 1.3), 1.4, base=0.55, spec=0.1)
    gem(art, f, 3.6, 0, 1.4, GOLD)
    grip(art, f, 7, 19, 2.1, P("#3a2a18", 0.4), "plain", base=0.45)
    us = [21, 36, 60, 74]
    mb, h, s, side = blade(f, us, [2.8, 4.2, 4.0, 0.2], [2.8, 4.2, 4.0, 0.2], hmax=2.4)
    metal(art, f, mb, h, pal, base=0.52, spec=0.3, kind="dull")
    vine = (np.abs(f.V - 2.6 * np.sin(f.U * 0.38)) < 0.8) & (f.U > 22) & (f.U < 66)
    art.paint(vine & mb, ONES, P("#2a5a1a", 0.5), base=0.4, outline=False)
    thorns = vine & mb & (np.abs((f.U % 5.0) - 2.5) < 0.6)
    art.img[thorns, :3] = [230, 220, 160]
    g = polyline([f.xy(23.6, -8), f.xy(20.4, -4), f.xy(21.0, 0), f.xy(20.4, 4), f.xy(23.6, 8)], 2.4)
    part(art, g, P("#3a6a1a", 0.5), 1.0, base=0.45, spec=0.05)


def celestial_blade(art, f):
    # Starry silver-blue blade with a constellation inlaid, a wing guard and a moon pommel.
    pal = P("#7a9ae8", 0.7, 1.05)
    moon = f.disc(3.6, 0, 3.6) & ~f.disc(5.0, -1.2, 3.0)
    part(art, moon, P("#f0f0d0", 0.5), 1.6, base=0.6)
    grip(art, f, 7, 19, 2.0, P("#2a3060", 0.4), "spiral", base=0.45)
    us = [21, 28, 60, 68, 76]
    mb, h, s, side = blade(f, us, [2.8, 3.8, 3.8, 2.6, 0.2], [2.8, 3.8, 3.8, 2.6, 0.2], hmax=2.6)
    metal(art, f, mb, h, pal, base=0.5, spec=0.5, power=22, kind="polish")
    pts = [(30, -1.2), (38, 1.4), (45, -0.6), (52, 1.8), (60, 0.2)]
    for (a, b) in zip(pts, pts[1:]):
        line = seg(*f.xy(*a), *f.xy(*b)) < 0.45
        art.img[line & mb, :3] = [190, 210, 255]
    for p in pts:
        glow(art, f.disc(p[0], p[1], 1.1) & mb, ONES * 0.95, P("#ffffff", 0.2))
    for sgn in (-1, 1):
        wg = f.poly([(21.6, sgn * 1.4), (19.6, sgn * 7.0), (21.8, sgn * 12.6), (25.4, sgn * 7.6), (23.6, sgn * 2.0)])
        part(art, wg, P("#e8eef8", 0.4), 1.2, base=0.6, spec=0.2)


# ---------------------------------------------------------------- mod swords


def skyroot_sword(art, f):
    # Aether wood: pale blue-grey grain, a wooden leaf blade and a knot pommel.
    pal = P("#8aa4a0", 0.5, 1.0)
    pm = f.disc(3.6, 0, 3.0)
    art.paint(pm, dome(pm, 2), pal, base=0.5, albedo=tex_for(art, pm, f, "wood"))
    grip(art, f, 6.0, 19, 2.3, CORD, "bands", base=0.55)
    us = [21, 30, 54, 66, 71]
    mb, h, s, side = blade(f, us, [3.6, 4.4, 4.4, 2.6, 0.3], [3.6, 4.4, 4.4, 2.6, 0.3], hmax=1.6, edge=0.0)
    art.paint(mb, h, pal, base=0.55, diff=1.3, spec=0.05, albedo=tex_for(art, mb, f, "wood"))
    g = f.box(18.8, 22.4, -7.0, 7.0)
    art.paint(g, dome(g, 1.4), P("#6a8480", 0.45), base=0.45, albedo=streaks(art.rng, f.V, f.U, 1.6) * 0.3)


def holystone_sword(art, f):
    # White, slightly porous stone: a short broad blade, a round stone guard.
    pal = P("#d8dcd8", 0.5, 1.0)
    lump = f.poly([(0.6, -2.2), (3.0, -4.0), (6.4, -2.6), (6.8, 2.6), (3.0, 4.0), (0.6, 2.2)])
    art.paint(lump, dome(lump, 2.0), pal, base=0.5, albedo=tex_for(art, lump, f, "stone"))
    grip(art, f, 6.6, 18.6, 2.4, P("#6a8aa0", 0.4), "spiral", base=0.5)
    us = [21, 30, 56, 68]
    mb, h, s, side = blade(f, us, [4.6, 5.8, 4.8, 0.3], [4.6, 5.8, 4.8, 0.3], hmax=3.0, edge=0.0)
    pores = (value_noise(art.rng, 2) > 0.78) * -0.12
    art.paint(mb, h, pal, base=0.55, diff=1.3, spec=0.05, albedo=tex_for(art, mb, f, "stone") + pores)
    g = f.disc(20.4, 0, 2.2, 7.6)
    art.paint(g, dome(g, 1.6), pal, base=0.45, albedo=tex_for(art, g, f, "stone"))


def zanite_sword(art, f):
    # A blade of purple crystal shards, wider in three steps, a gem cluster guard.
    pal = P("#9a5ae0", 0.7, 1.2)
    gem(art, f, 3.6, 0, 2.6, PURPLE_GEM)
    grip(art, f, 6.4, 19, 2.1, P("#3a3048", 0.4), "bands", base=0.45)
    us = [21, 32, 33, 46, 47, 60, 61, 72]
    w = [2.6, 2.6, 4.0, 4.0, 4.8, 4.8, 3.4, 0.2]
    mb, h, s, side = blade(f, us, w, w, hmax=2.6, edge=0.0)
    band = np.floor((f.U - 21) / 13.0)
    facet = np.where(f.V < 0, 0.16, -0.12) + ((band % 2) - 0.5) * 0.18
    art.paint(mb, h * 0.25, pal, base=0.56, diff=1.8, spec=0.5, power=24, albedo=facet)
    for (u, v, r) in [(20.0, -5.0, 2.4), (21.4, 0.0, 2.8), (20.0, 5.0, 2.4)]:
        c = f.poly([(u - r, v), (u, v - r), (u + r, v), (u, v + r)])
        art.paint(c, ONES, P("#c08af0", 0.6, 1.2), base=0.55, diff=0.0, albedo=np.where(f.V < v, 0.14, -0.08))


def gravitite_sword(art, f):
    # Gravitite: pink-violet metal with floating motes; the blade's pieces hover apart.
    pal = P("#c050c8", 0.7, 1.2)
    pm = f.disc(3.6, 0, 3.0)
    part(art, pm, pal, 2.0)
    grip(art, f, 6.4, 19, 2.1, DARK_GRIP, "cross", base=0.45)
    us = [21, 62, 70, 74]
    mb, h, s, side = blade(f, us, [3.2, 3.2, 2.2, 0.2], [3.2, 3.2, 2.2, 0.2], hmax=2.4)
    gaps = (np.abs((f.U % 12.0) - 10.0) < 1.4) & (f.U > 28) & (f.U < 66)
    mb = mb & ~gaps
    metal(art, f, mb, h, pal, base=0.54, spec=0.45, power=18)
    for (u, v) in [(34, 6.6), (48, -6.6), (58, 6.4)]:
        glow(art, f.disc(u, v, 1.3), ONES * 0.9, P("#ffb0ff", 0.4))
    cross_guard(art, f, pal, half=7.6, flare=1.2)


def resonarium_sword(art, f):
    # Resonarium: a teal tuning-fork blade (two prongs) humming with a bright line between.
    pal = P("#2ab8b0", 0.7, 1.2)
    pm = f.disc(3.6, 0, 3.0)
    part(art, pm, pal, 2.0)
    grip(art, f, 6.4, 19, 2.1, DARK_GRIP, "bands", base=0.45)
    us = [21, 30, 40, 72]
    mb, h, s, side = blade(f, us, [3.0, 3.6, 4.4, 0.2], [3.0, 3.6, 4.4, 0.2], hmax=2.6)
    slot = (np.abs(f.V) < 1.6) & (f.U > 38) & (f.U < 64 - np.abs(f.V))
    mb = mb & ~slot
    metal(art, f, mb, h, pal, base=0.54, spec=0.45, power=18)
    glow(art, (np.abs(f.V) < 0.4) & (f.U > 40) & (f.U < 60), ONES * 0.85, P("#b0fff8", 0.3))
    g = f.disc(20.6, 0, 1.6, 6.6)
    part(art, g, pal, 1.2)


# ---------------------------------------------------------------- greatswords (two hands)


def _greatsword(art, f, mat, pal, gpal, grip_pal, style):
    """A long two-handed blade: a long grip with a ricasso, wide shoulders, broad blade."""
    f = Frame(x0=2.6, y0=61.4)
    pm = f.disc(2.8, 0, 2.6)
    part(art, pm, gpal, 1.6)
    grip(art, f, 4.6, 19.6, 2.2, grip_pal, "cross", base=0.45)
    us = [21.6, 26, 27, 68, 80, 84]
    if style == "stone":
        w = [3.0, 3.0, 6.4, 6.4, 3.8, 0.4]
    elif style == "gold":
        w = [3.0, 3.0, 5.6, 6.6, 4.4, 0.2]
    elif style == "gem":
        w = [3.0, 3.0, 5.2, 6.8, 6.0, 0.2]
    else:
        w = [3.0, 3.0, 5.6, 5.6, 4.0, 0.2]
    mb, h, s, side = blade(f, us, w, w, hmax=3.2, edge=0.0 if style in ("stone", "gem") else 1.4)
    if style == "stone":
        art.paint(mb, h, pal, base=0.5, diff=1.4, spec=0.06, albedo=tex_for(art, mb, f, "stone"))
    elif style == "gem":
        band = np.floor((f.U - f.V * np.sign(f.V) * 0.6) / 10.0)
        facet = np.where(f.V < 0, 0.2, -0.16) + ((band % 2) - 0.5) * 0.22
        art.paint(mb, h * 0.2, pal, base=0.55, diff=1.9, spec=0.5, power=24, albedo=facet)
    else:
        fuller = (f.U > 28) & (f.U < 66) & (np.abs(f.V) < 1.4)
        metal(art, f, mb, h - fuller * 1.6, pal, base=0.52, spec=0.45 if style == "gold" else 0.38, kind="gold" if style == "gold" else "steel")
    # Parrying lugs on the ricasso and a long straight guard.
    for sgn in (-1, 1):
        lug = f.poly([(26.0, sgn * 3.0), (27.6, sgn * 5.0), (28.4, sgn * 3.0)])
        part(art, lug, gpal, 0.8)
    g = f.poly([(19.4, -13.0), (22.6, -13.6), (22.4, 13.6), (19.4, 13.0)])
    part(art, g, gpal, 1.4, base=0.45)
    if style == "gold":
        gem(art, f, 21.0, 0, 1.8, RED_GEM)


def stone_greatsword(art, f):
    _greatsword(art, f, "stone", P("#85837f", 0.5, 1.0), P("#6a6866", 0.5), LEATHER_GRIP, "stone")


def iron_greatsword(art, f):
    _greatsword(art, f, "iron", STEEL, STEEL, LEATHER_GRIP, "steel")


def golden_greatsword(art, f):
    _greatsword(art, f, "golden", GOLD, GOLD, VELVET, "gold")


def diamond_greatsword(art, f):
    _greatsword(art, f, "diamond", P("#4fd6cf", 0.72, 1.15), P("#c8ccd4", 0.6), DARK_GRIP, "gem")


# ---------------------------------------------------------------- swords added in the review


def cactus_sword(art, f):
    # A cactus stem for a blade: ribbed green with tufts of spines, a sandstone guard.
    pal = P("#4a9a3a", 0.6, 1.1)
    sand = P("#d8c088", 0.5, 1.0)
    pm = f.disc(3.6, 0, 3.0)
    art.paint(pm, dome(pm, 2.0), sand, base=0.5, albedo=tex_for(art, pm, f, "stone"))
    grip(art, f, 6.0, 19, 2.3, P("#7a5a30", 0.4), "bands", base=0.5)
    us = [21, 28, 62, 68]
    mb, h, s, side = blade(f, us, [3.6, 4.4, 4.2, 0.6], [3.6, 4.4, 4.2, 0.6], hmax=2.0, edge=0.0)
    ribs = np.abs(np.sin(f.V * 1.3)) * 0.8
    art.paint(mb, h + ribs, pal, base=0.5, diff=1.3, spec=0.05)
    spines = mb & (np.abs(np.abs(f.V) - (side - 0.8)) < 0.9) & (np.abs((f.U % 6.0) - 3.0) < 0.9)
    art.img[spines, :3] = [240, 232, 190]
    g = f.box(18.8, 22.4, -6.6, 6.6)
    art.paint(g, dome(g, 1.4), sand, base=0.45, albedo=tex_for(art, g, f, "stone"))


def bone_sword(art, f):
    # A long bone sharpened to an edge, knuckle joints for pommel and guard.
    pm = f.disc(2.6, -1.6, 2.2) | f.disc(2.6, 1.6, 2.2)
    art.paint(pm, dome(pm, 2.0), BONE, base=0.55)
    grip(art, f, 4.6, 19, 2.1, P("#5a3a24", 0.4), "cross", base=0.45)
    us = [21, 26, 50, 62, 70]
    mb, h, s, side = blade(f, us, [2.4, 3.6, 3.0, 3.8, 0.4], [2.4, 3.6, 3.0, 3.8, 0.4], ridge=-0.4, hmax=2.2, edge=0.6)
    pores = (value_noise(art.rng, 4) - 0.5) * 0.12
    art.paint(mb, h, BONE, base=0.55, diff=1.3, spec=0.08, albedo=pores)
    g = f.disc(21.2, -3.6, 2.6) | f.disc(21.2, 3.6, 2.6)
    art.paint(g, dome(g, 2.0), BONE, base=0.5)


def ice_blade(art, f):
    # Clear blue ice: a faceted blade frosted at the edges, an icicle guard.
    pal = P("#8ad0f0", 0.6, 1.1)
    gem(art, f, 3.6, 0, 2.6, P("#bff0ff", 0.5))
    grip(art, f, 6.4, 19, 2.1, P("#2a4a6a", 0.4), "spiral", base=0.45)
    us = [21, 32, 58, 72]
    mb, h, s, side = blade(f, us, [2.8, 4.0, 4.0, 0.2], [2.8, 4.0, 4.0, 0.2], hmax=2.6, edge=0.0)
    facet = np.where(f.V < 0, 0.16, -0.1) + ((np.floor((f.U - f.V * 0.8) / 8.0) % 2) - 0.5) * 0.16
    art.paint(mb, h * 0.3, pal, base=0.58, diff=1.8, spec=0.45, power=22, albedo=facet)
    frost = mb & (np.abs(f.V) > side - 1.2)
    art.img[frost, :3] = [236, 248, 255]
    for sgn in (-1, 1):
        ic = f.poly([(19.6, sgn * 1.6), (22.4, sgn * 1.6), (19.0, sgn * 11.4)])
        art.paint(ic, ONES, P("#c8f0ff", 0.5), base=0.6, diff=0.0, albedo=np.where(f.U < 20.6, 0.12, -0.06))


def light_blade(art, f):
    # A blade of pure light from a metal emitter: a white core with a blue glow round it.
    hilt = P("#8a8e98", 0.7, 1.0)
    art.paint(f.box(1.0, 5.0, -2.4, 2.4), dome(f.box(1.0, 5.0, -2.4, 2.4), 1.2), DARK, base=0.45)
    m = f.box(5.0, 20.4, -2.1, 2.1)
    art.paint(m, np.sqrt(np.clip(1 - (f.V / 2.1) ** 2, 0, 1)) * 2, hilt, base=0.5, spec=0.5, power=20)
    ribs = m & ((np.floor(f.U / 2.0) % 2) == 0) & (f.U > 8) & (f.U < 15)
    art.img[ribs, :3] = [40, 42, 50]
    art.dots([f.xy(17.0, -1.0)], 1.0, RED_GEM)
    em = f.box(19.6, 22.6, -2.8, 2.8)
    part(art, em, hilt, 1.0, base=0.5, spec=0.5)
    us = [22.6, 70, 74]
    mb, h, s, side = blade(f, us, [2.4, 2.4, 0.6], [2.4, 2.4, 0.6], hmax=1.0, edge=0.0)
    art.paint(mb, ONES, P("#4a8aff", 0.5, 1.3), base=0.6, diff=0.0)
    glow(art, mb & (np.abs(f.V) < 1.0), ONES * 0.95, P("#f0f8ff", 0.2))


def falcon_blade(art, f):
    # A light sabre with a feathered guard and a falcon-head pommel.
    pal = P("#c8c4b8", 0.72, 1.0)
    head = f.poly([(0.0, -1.2), (3.0, -3.2), (6.6, -2.4), (6.6, 2.6), (3.0, 3.0), (1.6, 0.6), (-0.6, 1.4)])
    part(art, head, P("#8a5a2a", 0.55), 1.4, base=0.5, spec=0.1)
    art.dots([f.xy(3.8, -1.0)], 0.9, GOLD)
    grip(art, f, 6.6, 19, 2.0, P("#4a3020", 0.4), "spiral", base=0.45)
    us = np.linspace(21, 72, 30)
    center = -((us - 21) / 51) ** 2 * 3.2
    mb, h, s, side = blade(f, us, np.interp(us, [21, 64, 72], [1.8, 1.8, 0.2]), np.interp(us, [21, 54, 66, 72], [2.6, 3.4, 2.8, 0.2]), center=center, ridge=-0.5, hmax=1.8)
    metal(art, f, mb, h, pal, base=0.56, spec=0.5, power=22, kind="polish")
    for sgn in (-1, 1):
        for k in range(3):
            fe = f.poly([(20.6 - k * 1.6, sgn * 1.6), (19.6 - k * 2.2, sgn * (7.4 + k * 1.2)), (21.8 - k * 1.6, sgn * (6.0 + k)), (22.0 - k * 1.2, sgn * 1.6)])
            art.paint(fe, dome(fe, 1.0), P("#9a6a3a" if k % 2 == 0 else "#e8dcc8", 0.5), base=0.55, spec=0.05)


def frost_brand(art, f):
    # Hardmode ice: a long blue-white blade with a jagged frozen edge and a snowflake guard.
    pal = P("#5ab0e8", 0.7, 1.2)
    flake = star(f, 3.6, 0, 4.0, 1.4, n=6)
    part(art, flake, P("#e8f8ff", 0.4), 1.0, base=0.62)
    grip(art, f, 7.2, 19, 2.2, P("#1c3a5a", 0.4), "cross", base=0.45)
    us = np.linspace(21, 75, 70)
    lw = np.interp(us, [21, 30, 62, 75], [3.0, 4.2, 4.0, 0.2])
    jag = (((us - 30) % 7.0) / 7.0) * 2.6 * (us > 30) * (us < 64)
    mb, h, s, side = blade(f, us, lw, lw + jag, ridge=-0.3, hmax=2.6)
    metal(art, f, mb, h, pal, base=0.54, spec=0.5, power=22, kind="polish")
    glow(art, mb & (np.abs(f.V + 0.6) < 0.8) & (f.U > 26), ONES * 0.8, P("#d8f4ff", 0.3))
    frost = mb & (s > side - 1.4) & (f.U > 28)
    art.img[frost, :3] = [228, 244, 255]
    for k in range(6):
        a = k * np.pi / 3
        arm = polyline([f.xy(21.0, 0), f.xy(21.0 + np.cos(a) * 6.2, np.sin(a) * 6.2)], 1.6) & ~(f.U > 24)
        part(art, arm, P("#c8ecff", 0.5), 0.8, base=0.6, spec=0.3)


# ---------------------------------------------------------------- registry


DESIGNS = {
    "hive_blade": hive_blade, "sky_blade": sky_blade, "umbral_sword": umbral_sword,
    "sanguinite_sword": sanguinite_sword, "enchanted_sword": enchanted_sword, "star_saber": star_saber,
    "blue_katana": blue_katana, "night_edge": night_edge, "radiant_blade": radiant_blade,
    "true_radiant_blade": true_radiant_blade, "true_night_edge": true_night_edge, "terra_blade": terra_blade,
    "horseman_blade": horseman_blade, "cutlass": cutlass, "prism_saber": prism_saber,
    "nebula_blade": nebula_blade, "celestial_wrath": celestial_wrath, "apex_blade": apex_blade,
    "cobalt_sword": cobalt_sword, "palladium_sword": palladium_sword, "mythril_sword": mythril_sword,
    "orichalcum_sword": orichalcum_sword, "adamantite_sword": adamantite_sword, "titanium_sword": titanium_sword,
    "radiant_sword": radiant_sword, "verdite_sword": verdite_sword, "lumite_sword": lumite_sword,
    "vine_blade": vine_blade, "celestial_blade": celestial_blade,
    "skyroot_sword": skyroot_sword, "holystone_sword": holystone_sword, "zanite_sword": zanite_sword,
    "gravitite_sword": gravitite_sword, "resonarium_sword": resonarium_sword,
    "stone_greatsword": stone_greatsword, "iron_greatsword": iron_greatsword,
    "golden_greatsword": golden_greatsword, "diamond_greatsword": diamond_greatsword,
    "cactus_sword": cactus_sword, "bone_sword": bone_sword, "ice_blade": ice_blade,
    "light_blade": light_blade, "falcon_blade": falcon_blade, "frost_brand": frost_brand,
}


def make(name):
    art = Art(name)
    DESIGNS[name](art, Frame())
    return art.done()


ITEMS = {k: (lambda k=k: make(k)) for k in DESIGNS}
