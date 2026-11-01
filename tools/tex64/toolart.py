"""Pickaxes, axes, shovels and hoes with a design of their own per material (HP, 2026-10-29:
"reavalie as picaretas e machados, é muito feio").

The old drawings (items.py) were one tool recoloured per material, and the shapes were weak:
the pick a thin crescent, the axe a round blob, the shovel a lollipop. Here every tool is built
like the swords in arms.py: in its own frame (`u` along the handle from the butt, `v` across it,
positive toward the bottom-right), from real parts with real cross-sections lit from the
top-left: a round handle with grips, wraps and ferrules; a socket or lashing where the head sits;
and a head whose profile belongs to the material (a knapped flint pick lashed on, a classic
iron pick, a faceted crystal pick, a netherite pick with a chisel back, a bearded viking axe, a
Greek crescent axe, a samurai-style pick wrapped in blue cord, black volcanic rock with glowing
veins...). pixel.py then turns each into 32x32 pixel art, and the game gives the picture its 3D
volume from its silhouette (item_render.rs), so clear, chunky silhouettes matter most.
"""

import numpy as np

from arms import (BLUE_CORD, CORD, DARK_GRIP, LAVA, LEATHER_GRIP, MAT, RED_GEM, WOOD, Art, Frame, _dilate, _pal, cracks,
                  dome, mpal, outline_of, patina, polyline, streaks, tex_for)
from style import N, value_noise

ASH = _pal("#6a5a4c", 0.42, 1.0)
CHAR = _pal("#2a2220", 0.3, 1.0)
STEEL = _pal("#9ea4ac", 0.6, 1.0)
DARK_WOOD = _pal("#5e4128", 0.42, 1.0)
PALE_WOOD = _pal("#b08a58", 0.45, 1.0)


# ---------------------------------------------------------------- painting helpers


def metal(art, f, m, h, mat, base=0.52, extra=None, glow=None, diff=None):
    """Paint a head part in the material: its ramp, shine and surface."""
    d = MAT[mat]
    alb = tex_for(art, m, f, d["kind"])
    if extra is not None:
        alb = alb + extra
    if d["kind"] == "lava" and glow is None:
        glow = cracks(art, m, art.rng, max(2, int(m.sum() / 120)), 0.9)
    art.paint(m, h, mpal(mat), base=base, diff=(1.9 if d["kind"] == "gem" else 1.5) if diff is None else diff,
              spec=d["spec"], power=d["power"], albedo=alb, glow=glow)


def handle(art, f, u0, u1, w=2.4, pal=None, grip=None, rings=(), cap=None, bend=0.0):
    """A round wooden handle from u0 to u1. `grip`: (u_a, u_b, pal, style) a wrapped grip near
    the butt; `rings`: (u, pal) metal ferrules; `cap`: pal of a butt cap. `bend`: a slight
    curve of the shaft (an axe's haft)."""
    pal = WOOD if pal is None else pal
    t = np.clip((f.U - u0) / max(u1 - u0, 1), 0, 1)
    c = bend * np.sin(t * np.pi)
    vv = f.V - c
    m = (f.U >= u0) & (f.U <= u1) & (np.abs(vv) <= w)
    cyl = np.sqrt(np.clip(1 - (vv / w) ** 2, 0, 1)) * w
    grain = streaks(art.rng, f.U, vv * 2.2, 1.7) * 0.12
    art.paint(m, cyl, pal, base=0.5, diff=1.3, spec=0.04, albedo=grain)
    if grip is not None:
        ua, ub, gp, style = grip
        g = (f.U >= ua) & (f.U <= ub) & (np.abs(vv) <= w + 0.5)
        k = np.abs(((f.U + vv * (0.9 if style == "spiral" else 0.0)) / 2.6) % 1 - 0.5)
        if style == "cross":
            a = np.abs(((f.U + vv * 1.1) / 3.2) % 1 - 0.5)
            b = np.abs(((f.U - vv * 1.1) / 3.2) % 1 - 0.5)
            k = np.minimum(a, b)
        hh = np.sqrt(np.clip(1 - (vv / (w + 0.5)) ** 2, 0, 1)) * (w + 0.5) + k * 1.4
        art.paint(g, hh, gp, base=0.48, diff=1.3, spec=0.05)
    for (u, rp) in rings:
        r = (np.abs(f.U - u) <= 1.1) & (np.abs(vv) <= w + 0.6)
        art.paint(r, np.sqrt(np.clip(1 - (vv / (w + 0.6)) ** 2, 0, 1)) * 2 + 0.5, rp, base=0.55, spec=0.3, power=12)
    if cap is not None:
        cp = (f.U >= u0 - 1.2) & (f.U <= u0 + 1.6) & (np.abs(vv) <= w + 0.7)
        art.paint(cp, np.sqrt(np.clip(1 - (vv / (w + 0.7)) ** 2, 0, 1)) * 2 + 0.5, cap, base=0.5, spec=0.3, power=12)
    return m


def lashing(art, f, u0, u1, w, pal=CORD):
    """Cord or thong wound around the joint of head and handle: an X over a few turns."""
    m = (f.U >= u0) & (f.U <= u1) & (np.abs(f.V) <= w)
    turns = np.abs(((f.U - f.V * 0.5) / 2.0) % 1 - 0.5) < 0.3
    x = (np.abs(f.U - (u0 + u1) / 2 - f.V * 0.9) < 0.9) | (np.abs(f.U - (u0 + u1) / 2 + f.V * 0.9) < 0.9)
    h = np.sqrt(np.clip(1 - (f.V / w) ** 2, 0, 1)) * w + turns * 0.8 + x * 1.2
    art.paint(m & (turns | x), h, pal, base=0.55, diff=1.3)


def socket(art, f, u0, u1, w, mat, pal=None):
    """The metal eye/collar the handle goes through, just below the head."""
    m = f.box(u0, u1, -w, w)
    h = np.sqrt(np.clip(1 - (f.V / w) ** 2, 0, 1)) * w * 0.8 + 0.6
    if pal is None:
        metal(art, f, m, h, mat, base=0.45)
    else:
        art.paint(m, h, pal, base=0.45, spec=0.3, power=12)


def band(f, vs, c, lo, hi):
    """A head that runs across the handle: for each station v (from `vs`), the head spans
    u from c - lo to c + hi. Returns the mask, a ridged height (two lit planes meeting along the
    middle, like a forged pick) and the distance from that middle."""
    vs = np.asarray(vs, float)
    C = np.interp(f.V, vs, c)
    L = np.interp(f.V, vs, lo)
    H = np.interp(f.V, vs, hi)
    m = (f.V >= vs[0]) & (f.V <= vs[-1]) & (f.U >= C - L) & (f.U <= C + H) & ((L + H) > 0.4)
    mid = C + (H - L) / 2
    half = np.maximum((L + H) / 2, 0.5)
    x = np.clip(np.abs(f.U - mid) / half, 0, 1)
    h = (1 - x) * np.minimum(half, 3.0) * 0.9
    return m, h, f.U - mid


def arc(vs, top, bend, power=2.0):
    """Centre line of a curved pick head: highest (`top`) over the handle, the arms dropping
    `bend` toward the hand at the ends."""
    vs = np.asarray(vs, float)
    span = np.max(np.abs(vs))
    return top - bend * (np.abs(vs) / span) ** power


def cut(art, mask, pal):
    """A see-through hole with an outline."""
    a = art.img
    inside = mask & (a[..., 3] > 0)
    a[inside, 3] = 0
    rim = _dilate(inside) & ~inside & (a[..., 3] > 0)
    a[rim, :3] = outline_of(pal)


def knapped(art, f, m, rng, n=18, lo=(40, -26), hi=(70, 26)):
    """Flake scars over a stone head: Voronoi cells, each a slightly tilted plane."""
    pts = rng.uniform(lo, hi, (n, 2))
    du = f.U[..., None] - pts[:, 0]
    dv = f.V[..., None] - pts[:, 1]
    dd = du * du + dv * dv
    cid = np.argmin(dd, -1)
    tilt = rng.uniform(-0.3, 0.3, (n, 2))
    ii, jj = np.meshgrid(np.arange(N), np.arange(N), indexing="ij")
    scar = tilt[cid, 0] * du[ii, jj, cid] + tilt[cid, 1] * dv[ii, jj, cid]
    srt = np.sort(dd, -1)
    seam = (np.sqrt(srt[..., 1]) - np.sqrt(srt[..., 0])) < 0.7
    return scar * 0.5, seam * -0.1


def facet_albedo(f, cuts=7.0):
    """Gem facets: planes split along the ridge and by cuts across, alternately lit."""
    band_ = np.floor((f.V + np.abs(f.U - 56) * 0.6) / cuts)
    return np.where(f.U > 56, 0.16, -0.12) + ((band_ % 2) - 0.5) * 0.2


# ---------------------------------------------------------------- pickaxes
# The head sits across the top of the handle (u ~ 50..62), arms reaching v = -24..24.


def pk_wooden(art, f, mat="wooden"):
    handle(art, f, 3, 54, 2.5, PALE_WOOD)
    vs = np.linspace(-19, 19, 39)
    m, h, s = band(f, vs, arc(vs, 56, 5), np.interp(vs, [-19, -15, 0, 15, 19], [2.0, 3.0, 3.8, 3.0, 2.0]),
                   np.interp(vs, [-19, -15, 0, 15, 19], [1.2, 2.4, 3.8, 2.4, 1.2]))
    hh = dome(m, 2.6)
    metal(art, f, m, hh, mat, base=0.5, extra=(value_noise(art.rng, 16) > 0.86) * -0.12)
    lashing(art, f, 50, 57, 3.6, CORD)


def pk_stone(art, f, mat="stone"):
    rng = art.rng
    handle(art, f, 3, 54, 2.5, WOOD)
    vs = np.linspace(-21, 21, 22)
    w = np.interp(vs, [-21, -14, 0, 14, 21], [0.4, 3.0, 4.6, 3.0, 0.4]) - rng.random(22) * 0.9 * (np.abs(vs) < 19)
    m, h, s = band(f, vs, arc(vs, 56, 6, 1.6), w, w * 0.9)
    scar, seam = knapped(art, f, m, rng)
    metal(art, f, m, h * 0.7 + scar, mat, base=0.5, extra=seam)
    lashing(art, f, 49, 58, 3.4, LEATHER_GRIP)


def pk_iron(art, f, mat="iron"):
    handle(art, f, 3, 54, 2.4, WOOD, grip=(4, 17, LEATHER_GRIP, "cross"))
    socket(art, f, 48.5, 53, 3.2, mat)
    vs = np.linspace(-24, 24, 49)
    w = np.interp(np.abs(vs), [0, 6, 18, 24], [4.2, 3.6, 2.0, 0.3])
    m, h, s = band(f, vs, arc(vs, 57, 9), w, w)
    metal(art, f, m, h, mat, base=0.52)


def pk_golden(art, f, mat="golden"):
    gold = mpal(mat)
    handle(art, f, 3, 54, 2.3, DARK_WOOD, rings=[(10, gold), (30, gold)], cap=gold)
    socket(art, f, 48, 53, 3.0, mat)
    vs = np.linspace(-24, 24, 49)
    # Slender arms that flare into broad leaf tips.
    w = np.interp(np.abs(vs), [0, 5, 14, 19, 22, 24], [3.6, 2.4, 1.8, 3.0, 2.4, 0.3])
    m, h, s = band(f, vs, arc(vs, 57, 10), w, w)
    metal(art, f, m, h, mat, base=0.54)
    art.dots([f.xy(57.2, 0)], 2.0, RED_GEM, base=0.5)


def pk_diamond(art, f, mat="diamond"):
    handle(art, f, 3, 54, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3c4048", 0.4), "bands"), rings=[(20, STEEL), (46, STEEL)])
    socket(art, f, 47.5, 53, 3.1, "iron", STEEL)
    # Straight-cut crystal arms: piecewise flat, sharp points.
    vs = np.array([-25, -16, -6, 0, 6, 16, 25], float)
    c = np.array([47, 53, 57, 58, 57, 53, 47], float)
    lo = np.array([0.2, 2.8, 3.8, 4.4, 3.8, 2.8, 0.2])
    hi = np.array([0.2, 2.0, 3.4, 4.4, 3.4, 2.0, 0.2])
    m, h, s = band(f, vs, c, lo, hi)
    fac = np.where(s > 0, 0.18, -0.14) + ((np.floor((f.V + 25) / 8.3) % 2) - 0.5) * 0.18
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def pk_netherite(art, f, mat="netherite"):
    trim = _pal("#7a5a46", 0.5, 1.1)
    handle(art, f, 3, 54, 2.5, _pal("#3a2a2c", 0.32), grip=(4, 20, DARK_GRIP, "bands"), rings=[(24, trim), (44, trim)], cap=trim)
    socket(art, f, 47, 53, 3.5, mat, trim)
    # A long pick on one side and a broad chisel on the other, spikes on the crown.
    vs = np.linspace(-26, 17, 44)
    lo = np.interp(vs, [-26, -18, -6, 0, 8, 15, 17], [0.3, 2.2, 3.8, 4.4, 4.0, 4.4, 4.4])
    hi = np.interp(vs, [-26, -18, -6, 0, 8, 15, 17], [0.3, 1.8, 3.4, 4.4, 2.6, 2.8, 2.8])
    c = np.interp(vs, [-26, -14, 0, 17], [45, 53, 57, 53.5])
    m, h, s = band(f, vs, c, lo, hi)
    seams = (np.abs(((f.V - 0.3 * f.U) % 6.0) - 3.0) < 0.35) & (np.abs(s) < 2.0)
    for v0 in (-4, 4):
        m |= f.poly([(60, v0 - 1.8), (65.5, v0), (60, v0 + 1.8)])
    metal(art, f, m, dome(m, 2.4), mat, base=0.5, extra=seams * -0.12)


def pk_copper(art, f, mat="copper"):
    handle(art, f, 3, 54, 2.4, WOOD, rings=[(46, mpal(mat))])
    vs = np.linspace(-19, 19, 39)
    w = np.interp(np.abs(vs), [0, 12, 17, 19], [4.0, 3.0, 2.0, 0.6])
    m, h, s = band(f, vs, arc(vs, 56, 3), w, w)
    metal(art, f, m, h, mat, base=0.52)
    art.dots([f.xy(56, -2.6), f.xy(56, 2.6)], 1.1, mpal(mat))
    patina(art, m, 0.3)


def pk_tin(art, f, mat="tin"):
    handle(art, f, 3, 54, 2.4, WOOD)
    socket(art, f, 48, 53, 3.0, mat)
    vs = np.linspace(-21, 21, 43)
    w = np.interp(np.abs(vs), [0, 15, 21], [3.6, 2.6, 0.4])
    m, h, s = band(f, vs, arc(vs, 56.5, 4, 1.4), w, w)
    metal(art, f, m, h, mat, base=0.5)


def pk_bronze(art, f, mat="bronze"):
    br = mpal(mat)
    handle(art, f, 3, 54, 2.4, DARK_WOOD, grip=(4, 16, _pal("#3c2a1c", 0.35), "bands"))
    vs = np.linspace(-23, 23, 47)
    # Arms that dip, then curl up again at the points.
    c = 57 - 7 * np.sin(np.abs(vs) / 23 * np.pi * 0.62)
    w = np.interp(np.abs(vs), [0, 8, 18, 23], [3.6, 2.8, 2.0, 0.3])
    m, h, s = band(f, vs, c, w, w)
    metal(art, f, m, h, mat, base=0.52)
    boss = f.disc(56.8, 0, 3.8)
    art.paint(boss, dome(boss, 2.4) + f.disc(56.8, 0, 1.6) * 1.0, br, base=0.5, spec=0.3)
    patina(art, m | boss, 0.2, "#5a9a78")


def pk_lead(art, f, mat="lead"):
    handle(art, f, 3, 53, 2.7, _pal("#4a3a2e", 0.35), rings=[(12, mpal(mat)), (40, mpal(mat))])
    # A pick on one side, a heavy hammer face on the other.
    vs = np.linspace(-23, 0, 24)
    w = np.interp(vs, [-23, -16, -4, 0], [0.4, 2.4, 3.8, 4.6])
    m, h, s = band(f, vs, arc(vs, 56, 7), w, w)
    hm = f.box(50.5, 62, -1, 12)
    hm |= f.box(49, 63.5, 9, 13)
    dents = (value_noise(art.rng, 8) - 0.5) * 0.8
    metal(art, f, m | hm, dome(m | hm, 2.6) + dents, mat, base=0.48)


def pk_silver(art, f, mat="silver"):
    sv = mpal(mat)
    handle(art, f, 3, 54, 2.2, _pal("#2c2a30", 0.32), grip=(5, 17, sv, "spiral"), cap=sv)
    socket(art, f, 48, 53, 2.9, mat)
    vs = np.linspace(-25, 25, 51)
    w = np.interp(np.abs(vs), [0, 5, 20, 25], [3.4, 2.4, 1.4, 0.2])
    m, h, s = band(f, vs, arc(vs, 57.5, 13, 1.7), w, w)
    metal(art, f, m, h, mat, base=0.55)
    # A gothic spike on the crown.
    sp = f.poly([(59.5, -1.6), (66.5, 0), (59.5, 1.6)])
    metal(art, f, sp, dome(sp, 1.2), mat, base=0.55)


def pk_tungsten(art, f, mat="tungsten"):
    handle(art, f, 3, 54, 2.4, DARK_GRIP, grip=(4, 19, _pal("#4a4e48", 0.35), "cross"), rings=[(46, STEEL)])
    # Angular, chunky: straight arms with flat-cut ends and a ridge of plates.
    vs = np.array([-22, -20, -10, 0, 10, 20, 22], float)
    c = np.array([51, 51.6, 55.4, 57, 55.4, 51.6, 51], float)
    lo = np.array([1.2, 2.6, 3.6, 4.4, 3.6, 2.6, 1.2])
    m, h, s = band(f, vs, c, lo, lo)
    plates = (np.abs(((f.V + 30) % 5.0) - 2.5) < 0.45) & (np.abs(f.V) < 19)
    metal(art, f, m, dome(m, 2.2), mat, base=0.5, extra=plates * -0.14)


def pk_platinum(art, f, mat="platinum"):
    gold = mpal("golden")
    # Ito-wrapped handle in blue cord, a gold collar and one long, swept pick.
    hm = handle(art, f, 3, 54, 2.4, _pal("#ece6d6", 0.4))
    k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.2) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.2) % 1 - 0.5))
    cord = hm & (k < 0.2) & (f.U < 30)
    art.paint(cord, np.sqrt(np.clip(1 - (f.V / 2.4) ** 2, 0, 1)) * 2, BLUE_CORD, base=0.5, outline=False)
    socket(art, f, 47, 53, 3.0, "golden", gold)
    vs = np.linspace(-26, 9, 36)
    c = np.interp(vs, [-26, -18, -8, 0, 9], [44, 50, 55.6, 57, 56])
    lo = np.interp(vs, [-26, -18, -6, 0, 6, 9], [0.2, 1.8, 3.0, 3.8, 3.2, 2.4])
    hi = np.interp(vs, [-26, -18, -6, 0, 6, 9], [0.2, 1.2, 2.4, 3.8, 3.0, 2.4])
    m, h, s = band(f, vs, c, lo, hi)
    hamon = (s > 0.4 + 0.4 * np.sin(f.V * 1.2)) & (f.V < -4)
    metal(art, f, m, h, mat, base=0.52, extra=hamon * 0.12)


def pk_magmarite(art, f, mat="magmarite"):
    rng = art.rng
    handle(art, f, 3, 54, 2.5, CHAR, grip=(4, 18, ASH, "bands"))
    vs = np.linspace(-23, 23, 24)
    w = np.interp(np.abs(vs), [0, 10, 18, 23], [4.6, 3.6, 2.4, 0.3]) + (rng.random(24) - 0.5) * 1.2 * (np.abs(vs) < 21)
    m, h, s = band(f, vs, arc(vs, 56.5, 8), np.maximum(w, 0.3), np.maximum(w * 0.9, 0.3))
    metal(art, f, m, dome(m, 2.6), mat, base=0.48)


PICKAXES = {
    "wooden": pk_wooden, "stone": pk_stone, "iron": pk_iron, "golden": pk_golden, "gold": pk_golden,
    "diamond": pk_diamond, "netherite": pk_netherite, "copper": pk_copper, "tin": pk_tin,
    "bronze": pk_bronze, "lead": pk_lead, "silver": pk_silver, "tungsten": pk_tungsten,
    "platinum": pk_platinum, "magmarite": pk_magmarite,
}


# ---------------------------------------------------------------- axes
# The bit points to -v (up and to the left), the poll or a spike to +v.


def ax_wooden(art, f, mat="wooden"):
    handle(art, f, 3, 60, 2.5, PALE_WOOD, bend=-0.8)
    # A thick wooden wedge, rounded at the bit.
    m = f.poly([(51, 3), (60.5, 3), (61.5, -6), (63, -12), (60.5, -16.6), (55.5, -17.6), (50, -16.6), (47.5, -12),
                (49.5, -6)])
    metal(art, f, m, np.clip(f.V + 18, 0, 10) / 10 * 2.6, mat, base=0.5)
    lashing(art, f, 46, 51, 3.4, CORD)


def ax_stone(art, f, mat="stone"):
    rng = art.rng
    handle(art, f, 3, 61, 2.5, WOOD, bend=-0.8)
    pts = [(51, 3), (59, 3.2), (63, -4), (64, -12), (59.5, -18.6), (52, -18), (47.5, -12), (48.5, -4)]
    pts = [(u + rng.uniform(-0.8, 0.8), v + rng.uniform(-0.8, 0.8)) for (u, v) in pts]
    m = f.poly(pts)
    scar, seam = knapped(art, f, m, rng, 14, (44, -20), (66, 4))
    hh = np.clip(f.V + 20, 0, 10) / 10 * 2.4 + scar
    metal(art, f, m, hh, mat, base=0.5, extra=seam)
    lashing(art, f, 49, 61, 3.0, LEATHER_GRIP)


def ax_iron(art, f, mat="iron"):
    handle(art, f, 3, 62, 2.4, WOOD, grip=(4, 16, LEATHER_GRIP, "cross"), bend=-1.0)
    # A felling axe: narrow cheeks, a flared bit with a curved edge, a square poll.
    m = f.poly([(52, 5.6), (60.5, 5.6), (60.5, -3), (65, -13.5), (64.2, -18.4), (56, -16.6), (45.4, -18.4),
                (44.6, -13.5), (51.6, -3)])
    edge = np.clip(-f.V - 13, 0, None)
    metal(art, f, m, np.clip(f.V + 18.4, 0, 9) / 9 * 2.6 - edge * 0.3, mat, base=0.52)


def ax_golden(art, f, mat="golden"):
    gold = mpal(mat)
    handle(art, f, 3, 66, 2.3, DARK_WOOD, rings=[(10, gold), (50, gold)], cap=gold)
    # A labrys: two crescent bits, mirrored.
    m = np.zeros((N, N), bool)
    for sgn in (-1, 1):
        m |= f.poly([(53.6, 0), (61.4, 0), (61.4, sgn * 4), (66.5, sgn * 10), (65, sgn * 15.5), (58.5, sgn * 13),
                     (51.5, sgn * 15.5), (50, sgn * 10), (53.6, sgn * 4)])
    metal(art, f, m, np.clip(14 - np.abs(f.V), 0, 10) / 10 * 2.4, mat, base=0.54)
    art.dots([f.xy(57.5, 0)], 1.8, RED_GEM)


def ax_diamond(art, f, mat="diamond"):
    handle(art, f, 3, 62, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3c4048", 0.4), "bands"), rings=[(22, STEEL)])
    sock = f.box(51, 61, -3.4, 4.4)
    art.paint(sock, dome(sock, 1.4), STEEL, base=0.45, spec=0.3, power=12)
    # A crystal bit: a cut shard, its facets alternately lit.
    m = f.poly([(51.5, -3.2), (61, -3.2), (66, -11), (63, -19.4), (56, -15.6), (47, -19.4), (45.6, -11)])
    fac = np.where(f.U > 56, 0.18, -0.12) + np.where(f.V < -11, 0.1, -0.06)
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def ax_netherite(art, f, mat="netherite"):
    trim = _pal("#7a5a46", 0.5, 1.1)
    handle(art, f, 3, 63, 2.5, _pal("#3a2a2c", 0.32), grip=(4, 20, DARK_GRIP, "bands"), rings=[(30, trim)], cap=trim)
    # A bearded axe: the bit hooks down toward the hand; a spike at the back.
    m = f.poly([(52.5, 2.6), (62, 2.6), (62, -4), (66, -12), (64.8, -19.6), (55, -16), (44, -18.4), (40.5, -13),
                (47, -9.6), (52, -3)])
    m |= f.poly([(54, 2), (60, 2), (57.6, 11.5)])
    seams = (np.abs(((f.U - 0.5 * f.V) % 6.5) - 3.25) < 0.35) & (f.V < -4)
    metal(art, f, m, dome(m, 2.4), mat, base=0.5, extra=seams * -0.12)
    sock = f.box(51.5, 62.5, -3.2, 3.2)
    art.paint(sock, dome(sock, 1.2), trim, base=0.45, spec=0.3)


def ax_copper(art, f, mat="copper"):
    handle(art, f, 3, 61, 2.4, WOOD, bend=-0.6)
    # An old flat axe: a trapezoid wedged through the haft.
    m = f.poly([(53, 4.6), (59, 4.6), (59.6, -4), (63.4, -15.6), (48.6, -15.6), (52.4, -4)])
    metal(art, f, m, np.clip(f.V + 16, 0, 10) / 10 * 2.4, mat, base=0.52)
    art.dots([f.xy(56, 1.6)], 1.2, mpal(mat))
    patina(art, m, 0.3)


def ax_tin(art, f, mat="tin"):
    handle(art, f, 3, 60, 2.3, WOOD)
    # A small hatchet.
    m = f.poly([(52.5, 3.8), (59.5, 3.8), (59.5, -3), (62, -11), (61, -13.6), (51.5, -13.6), (50.2, -11), (52.5, -3)])
    metal(art, f, m, np.clip(f.V + 14, 0, 9) / 9 * 2.2, mat, base=0.5)


def ax_bronze(art, f, mat="bronze"):
    br = mpal(mat)
    handle(art, f, 3, 64, 2.4, DARK_WOOD, grip=(4, 16, _pal("#3c2a1c", 0.35), "bands"))
    # A Greek crescent axe: a thin moon-shaped bit on a short neck, its horns reaching back.
    m = f.disc(56, -6, 12.6, 13.4) & ~f.disc(56, -1.5, 9.6, 11.0) & (f.V < -2)
    m |= f.box(52, 60, -6, 3)
    metal(art, f, m, np.clip(f.V + 19.4, 0, 10) / 10 * 2.4, mat, base=0.52)
    patina(art, m, 0.2, "#5a9a78")


def ax_lead(art, f, mat="lead"):
    handle(art, f, 3, 62, 2.7, _pal("#4a3a2e", 0.35), rings=[(14, mpal(mat))])
    # A square, heavy splitting head: a cleaver-like block.
    m = f.poly([(50, 5), (63, 5), (63, -16.6), (48.5, -16.6), (48.5, -3), (50, -3)])
    dents = (value_noise(art.rng, 8) - 0.5) * 0.8
    metal(art, f, m, np.clip(f.V + 17, 0, 12) / 12 * 2.4 + dents, mat, base=0.48)
    cut(art, f.disc(59.5, -12.5, 1.4), mpal(mat))


def ax_silver(art, f, mat="silver"):
    sv = mpal(mat)
    handle(art, f, 3, 64, 2.2, _pal("#2c2a30", 0.32), grip=(5, 17, sv, "spiral"), cap=sv)
    # A half-moon blade, thin and long, swept down toward the hand.
    ang = np.linspace(-np.pi * 0.5, np.pi * 0.6, 22)
    outer = [(53 + np.sin(a) * 14.5, -6 - np.cos(a) * 10.5) for a in ang]
    m = f.poly([(51, 2.4), (60.5, 2.4), (60.5, -4)] + outer[::-1] + [(40, -3)])
    m &= ~f.disc(48, -4.5, 6.5, 3.5)
    metal(art, f, m, np.clip(f.V + 17, 0, 12) / 12 * 2.0, mat, base=0.55)


def ax_tungsten(art, f, mat="tungsten"):
    handle(art, f, 3, 63, 2.4, DARK_GRIP, grip=(4, 19, _pal("#4a4e48", 0.35), "cross"), bend=-0.8)
    # A viking bearded axe: a long beard dropping along the haft.
    m = f.poly([(53, 3), (61.5, 3), (61.5, -3), (65, -10), (64.4, -19.6), (57.5, -17.6), (44, -19.6), (38.6, -16),
                (45, -12.6), (52.6, -3)])
    edge = np.clip(-f.V - 15, 0, None)
    metal(art, f, m, np.clip(f.V + 19, 0, 10) / 10 * 2.4 - edge * 0.25, mat, base=0.5)


def ax_platinum(art, f, mat="platinum"):
    gold = mpal("golden")
    hm = handle(art, f, 3, 64, 2.3, _pal("#ece6d6", 0.4))
    k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.2) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.2) % 1 - 0.5))
    art.paint(hm & (k < 0.2) & (f.U < 26), np.ones((N, N)) * 1.6, BLUE_CORD, base=0.5, outline=False)
    # A slim tomahawk: a narrow neck that flares to a curved edge, a pipe-like back.
    m = f.poly([(53.5, 2.6), (60, 2.6), (59.6, -5), (64, -14), (62.8, -18.4), (55.5, -17), (48.8, -18.4), (48.2, -14),
                (53.6, -5)])
    metal(art, f, m, np.clip(f.V + 18.4, 0, 10) / 10 * 2.2, mat, base=0.52)
    back = f.box(54, 59.5, 2, 7.5)
    art.paint(back, dome(back, 1.4), gold, base=0.5, spec=0.4)


def ax_magmarite(art, f, mat="magmarite"):
    rng = art.rng
    handle(art, f, 3, 62, 2.5, CHAR, grip=(4, 18, ASH, "bands"))
    pts = [(51.5, 4), (60.5, 4), (60.5, -4), (64.5, -11), (63, -18.6), (58.6, -15.6), (55, -19.6), (50.6, -16),
           (46, -18.6), (45, -11), (51.5, -4)]
    pts = [(u + rng.uniform(-0.7, 0.7), v + rng.uniform(-0.7, 0.7)) for (u, v) in pts]
    m = f.poly(pts)
    metal(art, f, m, dome(m, 2.6), mat, base=0.48)


AXES = {
    "wooden": ax_wooden, "stone": ax_stone, "iron": ax_iron, "golden": ax_golden, "gold": ax_golden,
    "diamond": ax_diamond, "netherite": ax_netherite, "copper": ax_copper, "tin": ax_tin,
    "bronze": ax_bronze, "lead": ax_lead, "silver": ax_silver, "tungsten": ax_tungsten,
    "platinum": ax_platinum, "magmarite": ax_magmarite,
}


# ---------------------------------------------------------------- shovels
# The blade carries on past the handle's end, along u.


def blade_along(f, us, left, right=None, hmax=2.2, dish=0.0):
    """A shovel blade: widths along u (left = -v side, right = +v side); `dish` makes the
    middle lower than the rims (a scoop)."""
    right = left if right is None else right
    us = np.asarray(us, float)
    L = np.interp(f.U, us, left)
    R = np.interp(f.U, us, right)
    m = (f.U >= us[0]) & (f.U <= us[-1]) & (f.V >= -L) & (f.V <= R) & ((L + R) > 0.4)
    half = np.maximum((L + R) / 2, 0.5)
    x = np.clip(np.abs(f.V - (R - L) / 2) / half, 0, 1)
    h = (1 - x) * hmax + x * x * dish * hmax
    return m, h


def shovel_base(art, f, pal=None, grip=None, rings=(), cap=None, u1=48, sock=None, mat=None):
    handle(art, f, 3, u1, 2.3, pal, grip=grip, rings=rings, cap=cap)
    if sock is not None:
        u0, ue = sock
        m = f.box(u0, ue, -2.9, 2.9)
        if mat is not None:
            metal(art, f, m, np.sqrt(np.clip(1 - (f.V / 2.9) ** 2, 0, 1)) * 2 + 0.4, mat, base=0.45)


def sh_wooden(art, f, mat="wooden"):
    shovel_base(art, f, PALE_WOOD, u1=50)
    m, h = blade_along(f, [46, 50, 66, 71, 73.5], [2.2, 6.0, 6.4, 5.0, 0.6], hmax=1.6)
    metal(art, f, m, dome(m, 2.4), mat, base=0.5)


def sh_stone(art, f, mat="stone"):
    rng = art.rng
    shovel_base(art, f, WOOD, u1=50)
    us = np.linspace(47, 74, 14)
    w = np.interp(us, [47, 51, 62, 74], [2.4, 6.0, 6.2, 0.4]) - rng.random(14) * 0.9 * (us < 72)
    m, h = blade_along(f, us, w, hmax=2.2)
    scar, seam = knapped(art, f, m, rng, 10, (46, -8), (76, 8))
    metal(art, f, m, h + scar, mat, base=0.5, extra=seam)
    lashing(art, f, 44, 50, 3.0, LEATHER_GRIP)


def sh_iron(art, f, mat="iron"):
    shovel_base(art, f, WOOD, grip=(4, 14, LEATHER_GRIP, "cross"), u1=48, sock=(43, 50), mat=mat)
    # A round-point spade with a tread step on its shoulders.
    us = [49, 50, 52, 64, 69, 73, 75]
    w = [2.8, 6.4, 6.6, 6.6, 5.2, 2.6, 0.4]
    m, h = blade_along(f, us, w, hmax=1.8, dish=0.6)
    ridge = (np.abs(f.V) < 0.8) & (f.U < 66)
    metal(art, f, m, h + ridge * 0.8, mat, base=0.52)


def sh_golden(art, f, mat="golden"):
    gold = mpal(mat)
    shovel_base(art, f, DARK_WOOD, rings=[(12, gold)], cap=gold, u1=48, sock=(43, 50), mat=mat)
    # A spade like the card suit: two round lobes and a point.
    us = np.linspace(49, 76, 28)
    lobe = 6.6 * np.sqrt(np.clip(1 - ((us - 57) / 8.5) ** 2, 0, 1))
    tip = np.interp(us, [57, 65, 76], [6.6, 4.2, 0.3])
    w = np.where(us < 57, np.maximum(lobe, 2.6), tip)
    m, h = blade_along(f, us, w, hmax=2.0)
    metal(art, f, m, h, mat, base=0.54)
    art.dots([f.xy(54.5, 0)], 1.4, RED_GEM)


def sh_diamond(art, f, mat="diamond"):
    shovel_base(art, f, DARK_GRIP, grip=(4, 16, _pal("#3c4048", 0.4), "bands"), rings=[(22, STEEL)], u1=48)
    sock = f.box(43, 50, -2.9, 2.9)
    art.paint(sock, dome(sock, 1.2), STEEL, base=0.45, spec=0.3)
    us = [49, 51, 63, 76]
    m, h = blade_along(f, us, [3.0, 6.6, 6.0, 0.3], hmax=2.0)
    fac = np.where(f.V < 0, 0.18, -0.12) + ((np.floor((f.U - np.abs(f.V) * 0.8) / 7) % 2) - 0.5) * 0.18
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def sh_netherite(art, f, mat="netherite"):
    trim = _pal("#7a5a46", 0.5, 1.1)
    shovel_base(art, f, _pal("#3a2a2c", 0.32), grip=(4, 18, DARK_GRIP, "bands"), rings=[(30, trim)], cap=trim, u1=48)
    sock = f.box(42, 50, -3.2, 3.2)
    art.paint(sock, dome(sock, 1.2), trim, base=0.45, spec=0.3)
    # A square digging blade with a toothed edge.
    m, h = blade_along(f, [49, 50.5, 70, 72], [3.0, 6.8, 6.8, 6.8], hmax=1.8, dish=0.5)
    teeth = (f.U > 70) & (np.abs(((f.V + 7) % 3.4) - 1.7) > 0.9)
    m &= ~teeth
    seams = (np.abs(f.V) < 0.6) & (f.U < 68)
    metal(art, f, m, h, mat, base=0.5, extra=seams * -0.12)


def sh_copper(art, f, mat="copper"):
    shovel_base(art, f, WOOD, rings=[(44, mpal(mat))], u1=48)
    # A deep, rounded scoop.
    us = np.linspace(46, 74, 20)
    w = 6.4 * np.sqrt(np.clip(1 - ((us - 62) / 12.5) ** 2, 0, 1)) + 0.3
    w = np.where(us < 52, np.maximum(w, 2.6), w)
    m, h = blade_along(f, us, w, hmax=1.6, dish=1.1)
    metal(art, f, m, h, mat, base=0.52)
    patina(art, m, 0.3)


def sh_tin(art, f, mat="tin"):
    shovel_base(art, f, WOOD, u1=48, sock=(43, 50), mat=mat)
    m, h = blade_along(f, [49, 51, 66, 73], [2.6, 5.6, 5.4, 1.0], hmax=1.6)
    metal(art, f, m, h, mat, base=0.5)


def sh_bronze(art, f, mat="bronze"):
    shovel_base(art, f, DARK_WOOD, grip=(4, 14, _pal("#3c2a1c", 0.35), "bands"), u1=48, sock=(43, 50), mat=mat)
    # A leaf-shaped blade with a midrib.
    us = np.linspace(49, 76, 20)
    w = 6.2 * np.sin(np.clip((us - 47) / 29, 0, 1) * np.pi) ** 0.8 + 0.3
    m, h = blade_along(f, us, w, hmax=2.0)
    rib = np.abs(f.V) < 0.7
    metal(art, f, m, h + rib * 0.6, mat, base=0.52)
    patina(art, m, 0.2, "#5a9a78")


def sh_lead(art, f, mat="lead"):
    shovel_base(art, f, _pal("#4a3a2e", 0.35), rings=[(12, mpal(mat))], u1=48, sock=(42, 50), mat=mat)
    m, h = blade_along(f, [49, 50.5, 72, 73], [3.2, 7.0, 7.0, 7.0], hmax=1.6, dish=0.4)
    dents = (value_noise(art.rng, 8) - 0.5) * 0.8
    metal(art, f, m, h + dents, mat, base=0.48)


def sh_silver(art, f, mat="silver"):
    sv = mpal(mat)
    shovel_base(art, f, _pal("#2c2a30", 0.32), grip=(5, 17, sv, "spiral"), cap=sv, u1=50, sock=(45, 51), mat=mat)
    # A slender trowel point.
    m, h = blade_along(f, [50, 52, 60, 77], [2.4, 5.0, 5.0, 0.3], hmax=1.8)
    metal(art, f, m, h, mat, base=0.55)


def sh_tungsten(art, f, mat="tungsten"):
    shovel_base(art, f, DARK_GRIP, grip=(4, 18, _pal("#4a4e48", 0.35), "cross"), u1=48, sock=(42, 50), mat=mat)
    # A square blade with a serrated digging edge and stamped plates.
    m, h = blade_along(f, [49, 50.5, 70.5, 73], [3.0, 6.6, 6.6, 6.0], hmax=1.8)
    saw = (f.U > 70.5) & (np.abs(((f.V + 7) % 2.6) - 1.3) < 0.7)
    m &= ~saw
    plates = (np.abs(((f.U - 50) % 6.5) - 3.25) < 0.4)
    metal(art, f, m, h, mat, base=0.5, extra=plates * -0.12)


def sh_platinum(art, f, mat="platinum"):
    gold = mpal("golden")
    shovel_base(art, f, _pal("#ece6d6", 0.4), u1=48)
    k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.2) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.2) % 1 - 0.5))
    hm = f.box(3, 26, -2.3, 2.3)
    art.paint(hm & (k < 0.2), np.ones((N, N)) * 1.6, BLUE_CORD, base=0.5, outline=False)
    sock = f.box(43, 50, -2.9, 2.9)
    art.paint(sock, dome(sock, 1.2), gold, base=0.5, spec=0.4)
    m, h = blade_along(f, [49, 51, 64, 71, 75], [2.6, 6.2, 6.2, 4.2, 0.4], hmax=1.8, dish=0.5)
    ridge = (np.abs(f.V) < 0.7) & (f.U < 70)
    metal(art, f, m, h + ridge * 0.8, mat, base=0.52)


def sh_magmarite(art, f, mat="magmarite"):
    rng = art.rng
    shovel_base(art, f, CHAR, grip=(4, 18, ASH, "bands"), u1=50)
    us = np.linspace(47, 75, 14)
    w = np.interp(us, [47, 51, 64, 75], [2.6, 6.4, 6.0, 0.4]) + (rng.random(14) - 0.5) * 1.2 * (us < 73)
    m, h = blade_along(f, us, np.maximum(w, 0.4), hmax=2.2)
    metal(art, f, m, dome(m, 2.4), mat, base=0.48)


SHOVELS = {
    "wooden": sh_wooden, "stone": sh_stone, "iron": sh_iron, "golden": sh_golden, "gold": sh_golden,
    "diamond": sh_diamond, "netherite": sh_netherite, "copper": sh_copper, "tin": sh_tin,
    "bronze": sh_bronze, "lead": sh_lead, "silver": sh_silver, "tungsten": sh_tungsten,
    "platinum": sh_platinum, "magmarite": sh_magmarite,
}


# ---------------------------------------------------------------- hoes
# A neck runs from the top of the handle to -v, then the blade drops toward the hand.


def hoe_neck(art, f, mat, top=58, reach=-13, w=2.0, pal=None):
    m = f.box(top - w * 2, top, reach, 2.6)
    h = np.sqrt(np.clip(1 - ((f.U - (top - w)) / w) ** 2, 0, 1)) * w
    if pal is None:
        metal(art, f, m, h, mat, base=0.5)
    else:
        art.paint(m, h, pal, base=0.5, spec=0.3)
    return m


def blade_down(f, v0, v1, u_top, u_bot, inner=0.0):
    """A hoe blade between v0 and v1 (v0 < v1), from u_top down to u_bot; `inner` tilts the
    bottom edge."""
    m = (f.V >= v0) & (f.V <= v1) & (f.U <= u_top) & (f.U >= u_bot + (f.V - v0) * inner)
    x = np.clip((f.V - v0) / max(v1 - v0, 0.5), 0, 1)
    h = np.sin(x * np.pi) * 1.6 + (f.U - u_bot) / max(u_top - u_bot, 1) * 0.8
    return m, h


def ho_wooden(art, f, mat="wooden"):
    handle(art, f, 3, 58, 2.5, PALE_WOOD)
    m = f.box(53, 59.5, -13, 2.6) | f.box(46, 59.5, -14, -8.5)
    metal(art, f, m, dome(m, 2.2), mat, base=0.5)
    lashing(art, f, 50, 56, 3.2, CORD)


def ho_stone(art, f, mat="stone"):
    rng = art.rng
    handle(art, f, 3, 58, 2.5, WOOD)
    pts = [(59.5, 2.6), (59.5, -9), (57.5, -15.6), (44, -16.4), (45, -10.6), (52.6, -8.6), (53, 2.6)]
    pts = [(u + rng.uniform(-0.6, 0.6), v + rng.uniform(-0.6, 0.6)) for (u, v) in pts]
    m = f.poly(pts)
    scar, seam = knapped(art, f, m, rng, 10, (42, -18), (62, 4))
    metal(art, f, m, dome(m, 2.2) + scar, mat, base=0.5, extra=seam)
    lashing(art, f, 49, 55, 3.0, LEATHER_GRIP)


def ho_iron(art, f, mat="iron"):
    handle(art, f, 3, 57, 2.4, WOOD, grip=(4, 16, LEATHER_GRIP, "cross"))
    hoe_neck(art, f, mat, 59, -9)
    # A draw hoe: a wide flat blade.
    m, h = blade_down(f, -17, -8, 60, 45)
    metal(art, f, m, h, mat, base=0.52)


def ho_golden(art, f, mat="golden"):
    gold = mpal(mat)
    handle(art, f, 3, 57, 2.3, DARK_WOOD, rings=[(10, gold), (40, gold)], cap=gold)
    hoe_neck(art, f, mat, 59, -8, 1.8)
    # A heart-shaped blade.
    m = f.disc(55.4, -13.2, 4.6, 4.6) | f.disc(55.4, -6.2, 4.0, 3.6)
    m |= f.poly([(53.5, -17.6), (53.5, -3.4), (42.5, -12)])
    m &= f.V < -3
    metal(art, f, m, dome(m, 2.4), mat, base=0.54)
    art.dots([f.xy(57.8, 0)], 1.6, RED_GEM)


def ho_diamond(art, f, mat="diamond"):
    handle(art, f, 3, 57, 2.3, DARK_GRIP, grip=(4, 16, _pal("#3c4048", 0.4), "bands"), rings=[(22, STEEL)])
    sock = f.box(53, 59.5, -3, 3.2)
    art.paint(sock, dome(sock, 1.2), STEEL, base=0.45, spec=0.3)
    m = f.poly([(59.5, -3), (59.5, -12), (55, -18), (43, -16.6), (52, -11), (54, -3)])
    fac = np.where(f.U > 54, 0.16, -0.12) + np.where(f.V < -11, 0.1, -0.08)
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def ho_netherite(art, f, mat="netherite"):
    trim = _pal("#7a5a46", 0.5, 1.1)
    handle(art, f, 3, 57, 2.5, _pal("#3a2a2c", 0.32), grip=(4, 20, DARK_GRIP, "bands"), rings=[(30, trim)], cap=trim)
    hoe_neck(art, f, mat, 60, -18, 2.4)
    # A two-pronged fork (a mattock-like hoe) with a back spike.
    m = f.box(43, 60, -18.5, -15) | f.box(43, 60, -11.8, -8.5)
    m |= f.poly([(59, 2), (55, 2), (57, 8.5)])
    metal(art, f, m, dome(m, 1.6), mat, base=0.5)


def ho_copper(art, f, mat="copper"):
    handle(art, f, 3, 57, 2.4, WOOD, rings=[(52, mpal(mat))])
    hoe_neck(art, f, mat, 59, -8)
    m = f.poly([(59.5, -8), (59.5, -14), (45, -16), (45, -7)])
    metal(art, f, m, dome(m, 2.0), mat, base=0.52)
    patina(art, m, 0.3)


def ho_tin(art, f, mat="tin"):
    handle(art, f, 3, 57, 2.3, WOOD)
    hoe_neck(art, f, mat, 59, -8, 1.8)
    m, h = blade_down(f, -14, -7, 59.5, 47)
    metal(art, f, m, h, mat, base=0.5)


def ho_bronze(art, f, mat="bronze"):
    handle(art, f, 3, 57, 2.4, DARK_WOOD, grip=(4, 14, _pal("#3c2a1c", 0.35), "bands"))
    hoe_neck(art, f, mat, 59.5, -17)
    # Two tines.
    m = f.poly([(59.5, -17), (59.5, -13.4), (44.5, -15.4)]) | f.poly([(59.5, -11), (59.5, -7.4), (45.5, -9.4)])
    metal(art, f, m, dome(m, 1.4), mat, base=0.52)
    patina(art, m, 0.2, "#5a9a78")


def ho_lead(art, f, mat="lead"):
    handle(art, f, 3, 57, 2.7, _pal("#4a3a2e", 0.35), rings=[(14, mpal(mat))])
    hoe_neck(art, f, mat, 60, -8, 2.4)
    m, h = blade_down(f, -19, -7, 60.5, 46)
    dents = (value_noise(art.rng, 8) - 0.5) * 0.8
    metal(art, f, m, h + dents, mat, base=0.48)


def ho_silver(art, f, mat="silver"):
    sv = mpal(mat)
    handle(art, f, 3, 57, 2.2, _pal("#2c2a30", 0.32), grip=(5, 17, sv, "spiral"), cap=sv)
    # A swan neck curving over into a small, pointed blade.
    neck = polyline([f.xy(57, 1.6), f.xy(60.4, -3), f.xy(60.6, -9), f.xy(58, -13)], 3.0)
    metal(art, f, neck, dome(neck, 1.4), mat, base=0.55)
    m = f.poly([(59, -10.6), (59, -16.6), (44, -15)])
    metal(art, f, m, dome(m, 1.6), mat, base=0.55)


def ho_tungsten(art, f, mat="tungsten"):
    handle(art, f, 3, 57, 2.4, DARK_GRIP, grip=(4, 19, _pal("#4a4e48", 0.35), "cross"))
    hoe_neck(art, f, mat, 60, -18.5, 2.2)
    # A three-tined cultivator.
    m = np.zeros((N, N), bool)
    for v in (-18.4, -13.4, -8.4):
        m |= f.poly([(59, v - 1.6), (59, v + 1.6), (45.5, v - 0.4)])
    metal(art, f, m, dome(m, 1.4), mat, base=0.5)


def ho_platinum(art, f, mat="platinum"):
    gold = mpal("golden")
    hm = handle(art, f, 3, 57, 2.3, _pal("#ece6d6", 0.4))
    k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.2) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.2) % 1 - 0.5))
    art.paint(hm & (k < 0.2) & (f.U < 26), np.ones((N, N)) * 1.6, BLUE_CORD, base=0.5, outline=False)
    sock = f.box(52.5, 59, -3, 3)
    art.paint(sock, dome(sock, 1.2), gold, base=0.5, spec=0.4)
    # A curved blade sweeping down, and a small pick at the back.
    vs = np.linspace(-19, 6, 26)
    c = np.interp(vs, [-19, -12, -3, 6], [48, 55, 57, 55.5])
    lo = np.interp(vs, [-19, -12, -3, 3, 6], [0.4, 3.6, 2.6, 2.2, 0.4])
    hi = np.interp(vs, [-19, -12, -3, 3, 6], [0.4, 2.2, 2.6, 2.2, 0.4])
    from_band = band(f, vs, c, lo, hi)
    metal(art, f, from_band[0], from_band[1], mat, base=0.52)


def ho_magmarite(art, f, mat="magmarite"):
    rng = art.rng
    handle(art, f, 3, 57, 2.5, CHAR, grip=(4, 18, ASH, "bands"))
    pts = [(60, 2.6), (60, -9), (58, -16.6), (51, -14.6), (44, -17), (45.4, -10.4), (53, -8.4), (53.6, 2.6)]
    pts = [(u + rng.uniform(-0.6, 0.6), v + rng.uniform(-0.6, 0.6)) for (u, v) in pts]
    m = f.poly(pts)
    metal(art, f, m, dome(m, 2.4), mat, base=0.48)


HOES = {
    "wooden": ho_wooden, "stone": ho_stone, "iron": ho_iron, "golden": ho_golden, "gold": ho_golden,
    "diamond": ho_diamond, "netherite": ho_netherite, "copper": ho_copper, "tin": ho_tin,
    "bronze": ho_bronze, "lead": ho_lead, "silver": ho_silver, "tungsten": ho_tungsten,
    "platinum": ho_platinum, "magmarite": ho_magmarite,
}

KINDS = {"pickaxe": PICKAXES, "axe": AXES, "shovel": SHOVELS, "hoe": HOES}

# Terraria's hardmode ores and the mods' metals and gems: their own colours on the design of the
# material whose character they share (cobalt plain steel, palladium bronze-like, mythril slender
# silver, orichalcum ornate, adamantite heavy and spiked, titanium angular...).
EXTRA = {
    "cobalt": (dict(base="#3a6ad8", spread=0.7, sat=1.15, kind="steel", spec=0.35, power=16), "iron"),
    "palladium": (dict(base="#e47a34", spread=0.66, sat=1.15, kind="copper", spec=0.32, power=14), "bronze"),
    "mythril": (dict(base="#3eb898", spread=0.68, sat=1.1, kind="polish", spec=0.45, power=20), "silver"),
    "orichalcum": (dict(base="#d65cbe", spread=0.7, sat=1.15, kind="gold", spec=0.42, power=14), "golden"),
    "adamantite": (dict(base="#d23446", spread=0.66, sat=1.2, kind="dark", spec=0.3, power=12), "netherite"),
    "titanium": (dict(base="#a2aab6", spread=0.72, sat=1.0, kind="steel", spec=0.4, power=18), "tungsten"),
    "lumite": (dict(base="#4ddcb4", spread=0.7, sat=1.15, kind="gem", spec=0.5, power=22), "diamond"),
    "radiant": (dict(base="#f2cc4a", spread=0.75, sat=1.2, warm=0.2, kind="gold", spec=0.45, power=14), "golden"),
    "umbral": (dict(base="#5c3c8c", spread=0.5, sat=1.15, kind="dark", spec=0.25, power=10), "netherite"),
    "verdite": (dict(base="#4cb84a", spread=0.62, sat=1.1, kind="steel", spec=0.28, power=14), "tungsten"),
    "sanguinite": (dict(base="#a8202c", spread=0.55, sat=1.2, kind="dark", spec=0.3, power=12), "platinum"),
    "resonarium": (dict(base="#7a5ad0", spread=0.7, sat=1.15, kind="gem", spec=0.5, power=22), "diamond"),
    "zanite": (dict(base="#8c5ad8", spread=0.7, sat=1.15, kind="gem", spec=0.5, power=22), "diamond"),
    "skyroot": (dict(base="#c4b88e", spread=0.5, sat=1.0, kind="wood", spec=0.06, power=6), "wooden"),
    "gravitite": (dict(base="#cf6ad0", spread=0.7, sat=1.15, kind="polish", spec=0.45, power=18), "silver"),
    "holystone": (dict(base="#c4c4bc", spread=0.5, sat=1.0, kind="stone", spec=0.06, power=6), "stone"),
}
for _m, (_d, _like) in EXTRA.items():
    MAT[_m] = _d
    for _k, _tbl in KINDS.items():
        _tbl[_m] = (lambda fn, m: lambda art, f: fn(art, f, m))(_tbl[_like], _m)


class ZFrame(Frame):
    """The tool frame, zoomed by `k` about u = `pu` and slid `du` back along the handle: the
    designs above are drawn at a comfortable scale, this makes them fill the icon (big heads,
    chunky handles, like the classic icons) while staying inside it."""

    def __init__(self, k=1.0, pu=30.0, du=0.0):
        super().__init__()
        self.k, self.pu, self.du = k, pu, du
        self.U = pu + (self.U - pu) / k + du
        self.V = self.V / k

    def xy(self, u, v):
        return super().xy(self.pu + (u - self.du - self.pu) * self.k, v * self.k)


ZOOM = {"pickaxe": (1.2, 30.0, 0.0), "axe": (1.14, 30.0, 5.0), "shovel": (1.15, 30.0, 2.0), "hoe": (1.2, 30.0, 2.5)}


def tool(kind, mat):
    art = Art(f"{mat}_{kind}")
    f = ZFrame(*ZOOM[kind])
    KINDS[kind][mat](art, f)
    return art.done()


# ---------------------------------------------------------------- Terraria's powered tools


def drill(mat):
    """A drill (HP 2026-11-01: the old one read as a knife): a chunky motor body, a pistol grip
    hanging off it, a steel collar and a long cone bit in the ore with bold spiral flutes, so
    the silhouette says "power tool" even at 32x32."""
    art = Art(f"{mat}_drill")
    f = Frame()
    housing = _pal("#575b63", 0.55, 1.0)
    # The pistol grip, hanging down from the body, with a trigger in front of it.
    g = f.poly([(12, 5), (22, 5), (21, 19.5), (12.5, 20.5), (10.5, 18)])
    art.paint(g, dome(g, 2.4), DARK_GRIP, base=0.45, diff=1.3)
    trig = f.poly([(23, 5), (25.5, 5), (24.2, 10)])
    metal(art, f, trig, dome(trig, 1.0), mat, base=0.5)
    # The motor body: rounded at the back, vents along the side, a panel in the ore.
    body = f.poly([(6, -6.5), (10, -10), (33, -10), (37, -7), (37, 7), (33, 8.5), (10, 8.5), (6, 5)])
    vents = (np.abs(((f.U - 9) % 4.4) - 2.2) < 0.75) & (f.U < 22) & (f.V > -7.5) & (f.V < -2.5)
    art.paint(body, dome(body, 3.4), housing, base=0.5, spec=0.3, power=12, albedo=vents * -0.22)
    panel = f.box(23, 34, -8.6, 6.6)
    metal(art, f, panel, dome(panel, 1.6), mat, base=0.56)
    # Collar and chuck.
    collar = f.box(36.5, 41.5, -7.2, 7.2)
    art.paint(collar, dome(collar, 1.6), STEEL, base=0.5, spec=0.35, power=14)
    # The bit: a cone with wide flutes, light and dark turns alternating.
    t = np.clip((f.U - 41) / 35, 0, 1)
    r = 6.2 * (1 - t) ** 0.9 + 0.4
    m = (f.U >= 41) & (f.U <= 76) & (np.abs(f.V) <= r)
    turn = ((f.U * 0.55 + f.V * 0.75) / 3.2) % 1.0
    flute = turn < 0.42
    h = np.sqrt(np.clip(1 - (f.V / np.maximum(r, 0.3)) ** 2, 0, 1)) * r
    metal(art, f, m, h - flute * 1.6, mat, base=0.54, extra=np.where(flute, -0.3, 0.12))
    return art.done()


def chainsaw(mat):
    """A chainsaw: motor body with a pull handle hanging below and a loop guard on top, and a
    long bar edged with big hooked teeth (readable at 32x32)."""
    art = Art(f"{mat}_chainsaw")
    f = Frame()
    housing = _pal("#575b63", 0.55, 1.0)
    # Rear grip below, loop guard above the body.
    g = f.poly([(8, 6), (17, 6), (16, 18.5), (8.5, 19.5), (6.5, 16)])
    art.paint(g, dome(g, 2.2), DARK_GRIP, base=0.45, diff=1.3)
    loop = polyline([f.xy(12, -8), f.xy(14, -15), f.xy(26, -15.5), f.xy(29, -9)], 2.8)
    art.paint(loop, dome(loop, 1.4), DARK_GRIP, base=0.45)
    body = f.poly([(4, -7), (8, -10), (30, -10), (33, -6), (33, 8), (8, 8.5), (4, 4)])
    vents = (np.abs(((f.U - 8) % 4.0) - 2.0) < 0.7) & (f.U < 20) & (f.V > 1.5) & (f.V < 6.5)
    art.paint(body, dome(body, 3.2), housing, base=0.5, spec=0.3, power=12, albedo=vents * -0.2)
    panel = f.box(18, 30, -8.6, -1.5)
    metal(art, f, panel, dome(panel, 1.4), mat, base=0.56)
    # The bar (steel core) and the chain in the ore, with teeth hooking out on both edges.
    w = 4.8
    ph = ((f.U - 33) % 7.0) / 7.0
    tooth = np.clip(1 - ph * 1.5, 0, 1) * 3.2
    bar = (f.U >= 32) & (f.U <= 70) & (np.abs(f.V) <= w + tooth)
    tip = f.disc(70, 0, w + 1.2)
    inner = f.box(33, 69.5, -w + 2.4, w - 2.4) | f.disc(69.5, 0, w - 2.4)
    chain = (bar | tip) & ~inner
    metal(art, f, chain, dome(chain, 1.4), mat, base=0.52, extra=np.where(np.abs(f.V) > w, 0.12, -0.08))
    core = inner
    art.paint(core, dome(core, 1.0), _pal("#4a4e56", 0.5, 1.0), base=0.5, spec=0.25)
    rivets = core & (np.abs(((f.U - 37) % 9.0) - 4.5) < 1.0) & (np.abs(f.V) < 1.0)
    art.img[rivets, :3] = STEEL[3]
    return art.done()


def hamaxe(name, mat):
    """Terraria's hamaxe in 3D terms: an axe bit on one side of the head and a square hammer on
    the other, on a long haft, so it reads as both tools at once."""
    art = Art(name)
    f = ZFrame(*ZOOM["axe"])
    lava = MAT[mat]["kind"] == "lava"
    handle(art, f, 3, 62, 2.5, CHAR if lava else DARK_GRIP, grip=(4, 18, ASH if lava else LEATHER_GRIP, "bands"), bend=-0.8)
    # The axe bit (toward the top-left): a wide flared blade with a curved edge.
    bit = f.poly([(51.5, -3), (60.5, -3), (64.5, -10), (66, -17), (63, -20.5), (56, -17.5), (48, -20.5), (45, -17),
                  (46.5, -10)])
    edge = np.clip(-f.V - 15, 0, None)
    # The hammer (toward the bottom-right): a block with a broader striking face.
    ham = f.box(50.5, 61.5, 2.5, 10.5) | f.box(48.5, 63.5, 10, 14)
    face = f.box(48.5, 63.5, 12.4, 14)
    metal(art, f, bit, np.clip(f.V + 20.5, 0, 10) / 10 * 2.6 - edge * 0.3, mat, base=0.53)
    metal(art, f, ham, dome(ham, 2.6), mat, base=0.5, extra=face * -0.18)
    sock = f.box(50, 62, -3.4, 3.4)
    art.paint(sock, dome(sock, 1.3), STEEL if not lava else _pal("#3a302c", 0.4), base=0.45, spec=0.3, power=12)
    return art.done()


def pickaxe_axe():
    """Terraria's Pickaxe Axe: a pick on one side, an axe bit on the other, in hallowed gold."""
    art = Art("pickaxe_axe")
    f = ZFrame(*ZOOM["pickaxe"])
    mat = "hallowed"
    gold = mpal("golden")
    handle(art, f, 3, 54, 2.4, _pal("#ece6d6", 0.4), rings=[(12, gold), (40, gold)], cap=gold)
    vs = np.linspace(0, 25, 26)
    w = np.interp(vs, [0, 6, 20, 25], [4.4, 3.6, 1.8, 0.3])
    m, h, s = band(f, vs, arc(vs, 57, 10), w, w)
    bit = f.poly([(52, -2), (61.5, -2), (64.5, -10), (63, -17.6), (56, -15), (49, -17.6), (47.6, -10)])
    metal(art, f, m | bit, dome(m | bit, 2.4), mat, base=0.54)
    art.dots([f.xy(57, 0)], 1.6, _pal("#4a8ae0", 0.6, 1.2))
    return art.done()


MAT["hallowed"] = dict(base="#e8d06a", spread=0.75, sat=1.15, warm=0.18, kind="gold", spec=0.45, power=16)
MAT["molten"] = MAT["magmarite"]

EXTRA_ITEMS = {}
for _m in ("cobalt", "mythril", "adamantite", "palladium", "orichalcum", "titanium"):  # all six exist as items
    EXTRA_ITEMS[f"{_m}_drill"] = lambda m=_m: drill(m)
    EXTRA_ITEMS[f"{_m}_chainsaw"] = lambda m=_m: chainsaw(m)
EXTRA_ITEMS["lumite_hamaxe"] = lambda: hamaxe("lumite_hamaxe", "lumite")
EXTRA_ITEMS["molten_hamaxe"] = lambda: hamaxe("molten_hamaxe", "molten")
EXTRA_ITEMS["pickaxe_axe"] = pickaxe_axe


# ---------------------------------------------------------------- hardmode and mod metals
# (HP 2026-11-01, tools pass lote 3) Each of these used to reuse another metal's design in its
# own colours. Now every one has its own silhouette, so a cobalt pick never reads as an iron one.

GLOW_TEAL = _pal("#7af0d0", 0.7, 1.2)


def pk_cobalt(art, f, mat="cobalt"):
    # Engineered: a straight beam with square-cut ends and a heavy square socket with rivets.
    handle(art, f, 3, 54, 2.4, DARK_GRIP, grip=(4, 18, _pal("#2a3a5c", 0.4), "bands"), rings=[(44, STEEL)])
    vs = np.array([-24, -21, -12, 0, 12, 21, 24], float)
    c = np.array([52.5, 53, 55.5, 57, 55.5, 53, 52.5])
    w = np.array([2.4, 2.8, 3.2, 3.6, 3.2, 2.8, 2.4])
    m, h, s = band(f, vs, c, w, w)
    sock = f.box(50, 62, -4.6, 4.6)
    metal(art, f, m | sock, dome(m | sock, 2.2), mat, base=0.52, extra=sock * 0.08)
    art.dots([f.xy(53, -2.6), f.xy(53, 2.6), f.xy(59, -2.6), f.xy(59, 2.6)], 0.9, STEEL)


def ax_cobalt(art, f, mat="cobalt"):
    handle(art, f, 3, 62, 2.4, DARK_GRIP, grip=(4, 18, _pal("#2a3a5c", 0.4), "bands"))
    # A square-cut bit with a round lightening hole and a square poll.
    m = f.poly([(50.5, 6), (62, 6), (62, -4), (64.5, -19), (47.5, -19), (50.5, -4)])
    metal(art, f, m, dome(m, 2.6), mat, base=0.52)
    cut(art, f.disc(56.2, -12, 2.6), mpal(mat))


def pk_palladium(art, f, mat="palladium"):
    # Stepped: arms built from three blocky plates that get smaller to the points.
    handle(art, f, 3, 54, 2.5, DARK_WOOD, grip=(4, 16, LEATHER_GRIP, "bands"))
    m = np.zeros((N, N), bool)
    for (v0, v1, u0, u1) in [(-6, 6, 52, 61.5), (-14, 14, 51.5, 58.5), (-20, 20, 50.5, 55.5), (-24.5, 24.5, 49.5, 52.5)]:
        m |= f.box(u0, u1, v0, v1)
    steps = (np.abs(np.abs(f.V) - 6) < 0.5) | (np.abs(np.abs(f.V) - 14) < 0.5) | (np.abs(np.abs(f.V) - 20) < 0.5)
    metal(art, f, m, dome(m, 2.2), mat, base=0.52, extra=steps * -0.16)


def ax_palladium(art, f, mat="palladium"):
    handle(art, f, 3, 62, 2.5, DARK_WOOD, grip=(4, 16, LEATHER_GRIP, "bands"))
    # A broad half-moon bit with stepped plates at its neck and a small square poll.
    moon = f.disc(56, -13, 10.5, 9.0) & (f.V < -6)
    neck = f.box(51, 61, -7, 5.5) & ~(f.box(51, 53, 2, 5.5) | f.box(59, 61, 2, 5.5))
    m = moon | neck
    steps = (np.abs(f.V + 7) < 0.5) & neck
    metal(art, f, m, np.clip(f.V + 22, 0, 10) / 10 * 2.6, mat, base=0.52, extra=steps * -0.16)


def pk_mythril(art, f, mat="mythril"):
    # Elegant S-curves: arms that sweep down and flick up into hooks.
    sv = mpal(mat)
    handle(art, f, 3, 54, 2.2, _pal("#24302c", 0.32), grip=(5, 18, sv, "spiral"), cap=sv)
    vs = np.linspace(-26, 26, 53)
    c = 57 - 9 * np.sin(np.abs(vs) / 26 * np.pi * 0.8) + 4 * (np.abs(vs) > 21) * (np.abs(vs) - 21) / 5
    w = np.interp(np.abs(vs), [0, 6, 18, 24, 26], [4.2, 3.2, 2.6, 2.2, 0.4])
    m, h, s = band(f, vs, c, w, w)
    gem = f.disc(57, 0, 2.6)
    metal(art, f, m, h, mat, base=0.55)
    art.paint(gem, dome(gem, 2.0), GLOW_TEAL, base=0.55, spec=0.5)


def ax_mythril(art, f, mat="mythril"):
    sv = mpal(mat)
    handle(art, f, 3, 66, 2.2, _pal("#24302c", 0.32), grip=(5, 18, sv, "spiral"), cap=sv)
    # A long, narrow bardiche bit running down the haft.
    m = f.poly([(64, -3), (68, -10), (64, -15), (44, -13), (40, -10), (43, -7.5), (58, -3)])
    metal(art, f, m, np.clip(f.V + 15, 0, 6) / 6 * 2.2, mat, base=0.55)
    for u in (52.5, 58.5):
        art.dots([f.xy(u, -3)], 1.1, sv)


def pk_orichalcum(art, f, mat="orichalcum"):
    # Floral: a five-petal boss in the middle, thin arms that end in leaf-shaped points.
    handle(art, f, 3, 54, 2.3, _pal("#3a2434", 0.35), rings=[(14, mpal(mat)), (44, mpal(mat))])
    vs = np.linspace(-25, 25, 51)
    w = np.interp(np.abs(vs), [0, 8, 15, 19, 22.5, 25], [2.8, 2.2, 2.2, 4.0, 3.0, 0.4])
    m, h, s = band(f, vs, arc(vs, 56.5, 8), w, w)
    pet = np.zeros((N, N), bool)
    for a in np.linspace(0, 2 * np.pi, 6)[:-1]:
        pet |= f.disc(56.5 + 3.4 * np.cos(a), 3.4 * np.sin(a), 2.4)
    metal(art, f, m | pet, dome(m | pet, 2.0), mat, base=0.54)
    art.dots([f.xy(56.5, 0)], 1.7, mpal("golden"))


def ax_orichalcum(art, f, mat="orichalcum"):
    handle(art, f, 3, 62, 2.3, _pal("#3a2434", 0.35), rings=[(14, mpal(mat))])
    # A bit cut as two petals with a point between them.
    m = f.disc(62, -13, 6.4, 7.6) | f.disc(49, -13, 6.4, 7.6) | f.poly([(49, -19), (55.5, -25), (62, -19), (55.5, -4)])
    m |= f.box(50.5, 61, -4.5, 3.6)
    metal(art, f, m, dome(m, 2.6), mat, base=0.54)
    art.dots([f.xy(55.5, -0.5)], 1.6, mpal("golden"))


def pk_adamantite(art, f, mat="adamantite"):
    # Aggressive: one long curved fang, a short hooked back, saw-teeth along the crown.
    handle(art, f, 3, 54, 2.6, _pal("#2a1c1e", 0.3), grip=(4, 20, DARK_GRIP, "cross"), cap=mpal(mat))
    vs = np.linspace(-27, 12, 40)
    c = np.interp(vs, [-27, -18, -6, 0, 8, 12], [45, 51, 56, 57, 55, 51])
    lo = np.interp(vs, [-27, -18, -6, 0, 8, 12], [0.3, 2.2, 3.8, 4.2, 3.2, 0.4])
    m, h, s = band(f, vs, c, lo, lo * 0.9)
    teeth = (f.U > 59) & (f.U < 63.5 - 2.6 * (np.abs(((f.V + 30) % 4.4) - 2.2))) & (np.abs(f.V) < 9)
    metal(art, f, m | teeth, dome(m | teeth, 2.4), mat, base=0.5)


def ax_adamantite(art, f, mat="adamantite"):
    handle(art, f, 3, 63, 2.6, _pal("#2a1c1e", 0.3), grip=(4, 20, DARK_GRIP, "cross"))
    # A jagged bit with three teeth on the edge and a spike at the back.
    m = f.poly([(51.5, 3), (62, 3), (62, -4), (67, -11), (64, -13.5), (66.5, -17), (61, -19), (58.5, -16.5),
                (55, -20), (50, -17.5), (46, -20.5), (44, -13), (51, -4)])
    m |= f.poly([(53, 2.5), (60.5, 2.5), (56.8, 13)])
    metal(art, f, m, dome(m, 2.6), mat, base=0.5)


def pk_titanium(art, f, mat="titanium"):
    # Angular blade-arms: flat on top, wide at the middle, knife-sharp points, a cut-out eye.
    handle(art, f, 3, 54, 2.3, DARK_GRIP, grip=(4, 18, _pal("#5a606a", 0.4), "cross"), rings=[(46, STEEL)])
    vs = np.array([-26, -12, 0, 12, 26], float)
    m, h, s = band(f, vs, [55, 57.5, 58, 57.5, 55], [0.2, 2.6, 6.0, 2.6, 0.2], [0.2, 0.6, 1.6, 0.6, 0.2])
    metal(art, f, m, dome(m, 2.0), mat, base=0.54, extra=np.where(f.V < 0, 0.1, -0.08))
    cut(art, f.poly([(54.5, -1.6), (57, 0), (54.5, 1.6)]), mpal(mat))


def ax_titanium(art, f, mat="titanium"):
    handle(art, f, 3, 62, 2.3, DARK_GRIP, grip=(4, 18, _pal("#5a606a", 0.4), "cross"))
    # A trapezoid bit, its edge raked back, with a triangular cut-out.
    m = f.poly([(51, 4.5), (62, 4.5), (62, -3), (68, -20), (44, -17), (51, -3)])
    metal(art, f, m, np.clip(f.V + 20, 0, 9) / 9 * 2.4, mat, base=0.54)
    cut(art, f.poly([(53, -8), (59, -8), (56, -13)]), mpal(mat))


def pk_radiant(art, f, mat="radiant"):
    # Hallowed: arms like spread wings (feathered undersides) under a crown of short rays.
    handle(art, f, 3, 54, 2.3, _pal("#ece6d6", 0.4), rings=[(12, mpal(mat)), (44, mpal(mat))], cap=mpal(mat))
    vs = np.linspace(-25, 25, 51)
    w = np.interp(np.abs(vs), [0, 8, 18, 25], [3.4, 3.2, 2.4, 0.3])
    feather = np.where(np.abs(vs) > 4, 1.6 * (np.abs(((vs + 30) % 4.0) - 2.0) / 2.0), 0)
    m, h, s = band(f, vs, arc(vs, 57, 7), w + feather, w)
    rays = np.zeros((N, N), bool)
    for v in (-5, 0, 5):
        rays |= f.poly([(59, v - 1.3), (65.5 - abs(v) * 0.3, v), (59, v + 1.3)])
    metal(art, f, m | rays, dome(m | rays, 2.0), mat, base=0.55)
    art.dots([f.xy(57, 0)], 1.6, _pal("#4a8ae0", 0.6, 1.2))


def ax_radiant(art, f, mat="radiant"):
    handle(art, f, 3, 62, 2.3, _pal("#ece6d6", 0.4), rings=[(12, mpal(mat))], cap=mpal(mat))
    # One wing as the bit: three feathered lobes along its edge.
    m = f.poly([(51, 3.5), (61.5, 3.5), (61.5, -3), (66, -10), (62, -15), (55, -21), (48, -20), (46, -14), (51, -3)])
    for (u, v) in [(62.5, -12), (57.5, -18.5), (50.5, -18.5)]:
        m |= f.disc(u, v, 3.0)
    feathers = np.abs(((f.U * 0.7 - f.V) % 5.0) - 2.5) < 0.45
    metal(art, f, m, dome(m, 2.6), mat, base=0.55, extra=feathers * (f.V < -5) * -0.12)


def pk_verdite(art, f, mat="verdite"):
    # Living metal: leaf-shaped arms with a midrib and veins; a vine wound round the handle.
    hm = handle(art, f, 3, 54, 2.4, DARK_WOOD)
    vine = hm & (np.abs(((f.U + f.V * 1.4) / 6.0) % 1 - 0.5) < 0.16)
    art.paint(vine, np.ones((N, N)) * 2.6, _pal("#3c7a34", 0.4), base=0.55)
    vs = np.linspace(-25, 25, 51)
    w = 4.2 * np.sin(np.clip(np.abs(vs) / 25, 0, 1) * np.pi) ** 0.8 + 1.2 * (np.abs(vs) < 4)
    m, h, s = band(f, vs, arc(vs, 56.5, 7), w + 0.2, w + 0.2)
    rib = np.abs(s) < 0.6
    veins = np.abs(((f.V * np.sign(f.V + 1e-3) + s * 1.6) % 4.5) - 2.25) < 0.4
    metal(art, f, m, h, mat, base=0.52, extra=rib * 0.14 + veins * (~rib) * -0.1)


def ax_verdite(art, f, mat="verdite"):
    hm = handle(art, f, 3, 62, 2.4, DARK_WOOD)
    vine = hm & (np.abs(((f.U + f.V * 1.4) / 6.0) % 1 - 0.5) < 0.16)
    art.paint(vine, np.ones((N, N)) * 2.6, _pal("#3c7a34", 0.4), base=0.55)
    # A broad leaf as the bit, tip curling toward the hand.
    vs = np.linspace(-22, 2, 25)
    c = np.interp(vs, [-22, -12, 2], [50, 55.5, 56])
    w = 6.0 * np.sin(np.clip((vs + 22) / 24, 0, 1) * np.pi) ** 0.7 + 0.3
    m, h, s = band(f, vs, c, w, w)
    rib = np.abs(s) < 0.6
    metal(art, f, m, h, mat, base=0.52, extra=rib * 0.14)


def pk_lumite(art, f, mat="lumite"):
    # Celestial: the head floats in pieces over a glowing core, nothing touching the haft.
    handle(art, f, 3, 50, 2.3, _pal("#1c2430", 0.3), grip=(4, 18, _pal("#2c3a4a", 0.4), "bands"), cap=GLOW_TEAL)
    core = f.disc(56, 0, 3.0)
    art.paint(core, dome(core, 2.4), GLOW_TEAL, base=0.62, spec=0.5, glow=core * 0.8)
    m = np.zeros((N, N), bool)
    for sgn in (-1, 1):
        vs = np.linspace(4.5, 26, 22) * sgn
        vv = np.sort(vs)
        w = np.interp(np.abs(vv), [4.5, 10, 20, 26], [3.4, 3.0, 1.8, 0.3])
        mm, _, _ = band(f, vv, arc(vv, 57, 9), w, w)
        m |= mm
    crown = f.poly([(60, -2.5), (64, 0), (60, 2.5)])
    metal(art, f, m | crown, dome(m | crown, 2.0), mat, base=0.56)


def ax_lumite(art, f, mat="lumite"):
    handle(art, f, 3, 60, 2.3, _pal("#1c2430", 0.3), grip=(4, 18, _pal("#2c3a4a", 0.4), "bands"), cap=GLOW_TEAL)
    core = f.disc(57, -1, 2.6)
    art.paint(core, dome(core, 2.0), GLOW_TEAL, base=0.62, spec=0.5, glow=core * 0.8)
    # A crescent bit hovering off the haft, and a floating shard behind.
    outer = f.disc(56, -13, 9.5, 9.0)
    inner = f.disc(52, -9, 8.0, 7.0)
    m = outer & ~inner & (f.V < -4)
    m |= f.poly([(54, 4), (59, 4), (56.5, 10)])
    metal(art, f, m, dome(m, 2.2), mat, base=0.56)


def pk_umbral(art, f, mat="umbral"):
    # Nightmare: bony spines along the arms, an eye in the middle.
    handle(art, f, 3, 54, 2.4, _pal("#2a2030", 0.3), grip=(4, 18, _pal("#4a3a5a", 0.35), "spiral"))
    vs = np.linspace(-25, 25, 51)
    w = np.interp(np.abs(vs), [0, 10, 20, 25], [4.0, 3.0, 2.0, 0.3])
    m, h, s = band(f, vs, arc(vs, 56, 10, 1.5), w, w * 0.7)
    spines = np.zeros((N, N), bool)
    for v in (-16, -9, 9, 16):
        u = 56 - 10 * (abs(v) / 25) ** 1.5
        spines |= f.poly([(u + 2, v - 1.4), (u + 6.5, v - np.sign(v) * 1.5), (u + 2, v + 1.4)])
    metal(art, f, m | spines, dome(m | spines, 2.2), mat, base=0.5)
    eye = f.disc(56, 0, 2.2, 1.6)
    art.paint(eye, dome(eye, 1.4), RED_GEM, base=0.6, glow=eye * 0.5)


def pk_sanguinite(art, f, mat="sanguinite"):
    # Flesh and bone: a fat, veined pick with a row of teeth along its underside.
    handle(art, f, 3, 54, 2.5, _pal("#3a2020", 0.3), grip=(4, 18, _pal("#6a2a2a", 0.35), "bands"))
    vs = np.linspace(-24, 24, 49)
    w = np.interp(np.abs(vs), [0, 10, 18, 24], [4.6, 4.0, 2.6, 0.3])
    m, h, s = band(f, vs, arc(vs, 57, 8), w, w)
    teeth = (s < -w.mean() + 0.5) & (np.abs(((f.V + 30) % 3.6) - 1.8) < 0.8) & (np.abs(f.V) > 4) & (np.abs(f.V) < 19)
    veins = cracks(art, m, art.rng, 4, 0.6)
    metal(art, f, m, dome(m, 2.6), mat, base=0.5, extra=veins * -0.25)
    art.img[teeth & m, :3] = _pal("#e8dcc0", 0.5)[4]


# Mods (Aether-like and Resonarium): their own looks too.

def pk_skyroot(art, f, mat="skyroot"):
    # A twisted root grown into a pick: knotted, uneven arms, no metal at all.
    rng = art.rng
    handle(art, f, 3, 55, 2.6, PALE_WOOD, bend=0.8)
    vs = np.linspace(-21, 21, 22)
    w = np.interp(np.abs(vs), [0, 10, 21], [4.2, 3.0, 0.6]) + (rng.random(22) - 0.5) * 1.4
    m, h, s = band(f, vs, arc(vs, 56, 6) + np.sin(vs * 0.6) * 1.2, np.maximum(w, 0.3), np.maximum(w * 0.8, 0.3))
    metal(art, f, m, dome(m, 2.6), mat, base=0.52)
    art.dots([f.xy(56, -9), f.xy(55, 10)], 1.2, PALE_WOOD)


def ax_skyroot(art, f, mat="skyroot"):
    handle(art, f, 3, 62, 2.6, PALE_WOOD, bend=0.8)
    m = f.poly([(51, 4), (62, 4), (63, -6), (61, -16), (54, -18.5), (48, -15), (49, -5)])
    metal(art, f, m, dome(m, 2.8), mat, base=0.52, extra=streaks(art.rng, f.V, f.U, 1.6) * 0.1)
    lashing(art, f, 48, 56, 3.2, CORD)


def sh_skyroot(art, f, mat="skyroot"):
    shovel_base(art, f, PALE_WOOD, u1=50)
    m, h = blade_along(f, [46, 50, 60, 68, 72], [2.4, 5.0, 6.0, 4.2, 0.8], hmax=1.6)
    metal(art, f, m, dome(m, 2.4), mat, base=0.5, extra=streaks(art.rng, f.V, f.U, 1.6) * 0.1)
    lashing(art, f, 44, 50, 3.0, CORD)


def pk_holystone(art, f, mat="holystone"):
    # Tumbled river stone: a smooth, round-ended head lashed on with pale cord.
    handle(art, f, 3, 54, 2.5, PALE_WOOD)
    vs = np.linspace(-20, 20, 41)
    w = np.interp(np.abs(vs), [0, 14, 18, 20], [4.6, 3.8, 3.0, 0.8])
    m, h, s = band(f, vs, arc(vs, 56, 4), w, w)
    m |= f.disc(*(lambda: (56 - 4 * 0.98, -19.2))(), 2.8) | f.disc(56 - 4 * 0.98, 19.2, 2.8)
    metal(art, f, m, dome(m, 3.0), mat, base=0.55)
    lashing(art, f, 49, 57, 3.6, _pal("#d8d0b8", 0.5))


def ax_holystone(art, f, mat="holystone"):
    handle(art, f, 3, 62, 2.5, PALE_WOOD)
    m = f.disc(55.5, -10, 8.0, 9.0) | f.box(50, 61, -4, 3)
    metal(art, f, m, dome(m, 3.0), mat, base=0.55)
    lashing(art, f, 49, 60, 3.4, _pal("#d8d0b8", 0.5))


def sh_holystone(art, f, mat="holystone"):
    shovel_base(art, f, PALE_WOOD, u1=50)
    us = np.linspace(47, 74, 28)
    w = 6.2 * np.sqrt(np.clip(1 - ((us - 61) / 13.5) ** 2, 0, 1)) + 0.4
    m, h = blade_along(f, us, w, hmax=2.4)
    metal(art, f, m, dome(m, 3.0), mat, base=0.55)
    lashing(art, f, 44, 50, 3.0, _pal("#d8d0b8", 0.5))


def pk_zanite(art, f, mat="zanite"):
    # A cluster of raw crystals set in a steel cap: points of different lengths.
    handle(art, f, 3, 54, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3a2c4a", 0.4), "bands"))
    sock = f.box(49, 56, -5, 5)
    art.paint(sock, dome(sock, 1.4), STEEL, base=0.45, spec=0.3)
    m = np.zeros((N, N), bool)
    for (u, v, L, r) in [(55, -4, 21, 3.6), (55, 4, 21, 3.6), (56, -2, 10, 2.6), (56, 2, 10, 2.6), (55.5, 0, 7, 3.0)]:
        sgn = np.sign(v) if v else 0
        tip = (u + 1.5, v + sgn * L) if v else (u + L, 0)
        m |= f.poly([(u - r * (1 if v else 0), v), tip, (u + r, v)]) if v else f.poly([(u, -r), tip, (u, r)])
    fac = ((np.floor((f.U * 0.8 + f.V) / 3.0) % 2) - 0.5) * 0.24
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def ax_zanite(art, f, mat="zanite"):
    handle(art, f, 3, 62, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3a2c4a", 0.4), "bands"))
    sock = f.box(50, 61, -4, 4)
    art.paint(sock, dome(sock, 1.4), STEEL, base=0.45, spec=0.3)
    m = f.poly([(50, -4), (61, -4), (65, -14), (58, -21), (55.5, -15), (52, -20.5), (46, -13)])
    fac = ((np.floor((f.U * 0.8 + f.V) / 3.2) % 2) - 0.5) * 0.24
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def sh_zanite(art, f, mat="zanite"):
    shovel_base(art, f, DARK_GRIP, grip=(4, 16, _pal("#3a2c4a", 0.4), "bands"), u1=48)
    sock = f.box(43, 50, -2.9, 2.9)
    art.paint(sock, dome(sock, 1.2), STEEL, base=0.45, spec=0.3)
    # A hexagonal crystal plate.
    m = f.poly([(49, -3), (52, -6.4), (68, -6.4), (75, 0), (68, 6.4), (52, 6.4), (49, 3)])
    fac = ((np.floor((f.U - np.abs(f.V) * 1.2) / 4.0) % 2) - 0.5) * 0.24
    metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)


def pk_gravitite(art, f, mat="gravitite"):
    # Anti-gravity: the head hovers above the haft inside a ring, a gap of air between them.
    handle(art, f, 3, 49, 2.2, _pal("#2a2234", 0.3), grip=(5, 18, mpal(mat), "spiral"))
    ring = f.disc(53, 0, 4.4) & ~f.disc(53, 0, 2.6)
    art.paint(ring, dome(ring, 1.2), STEEL, base=0.5, spec=0.4)
    vs = np.linspace(-24, 24, 49)
    w = np.interp(np.abs(vs), [0, 6, 20, 24], [3.0, 2.6, 1.6, 0.3])
    m, h, s = band(f, vs, arc(vs, 60, 9, 1.6), w, w)
    metal(art, f, m, h, mat, base=0.55)


def ax_gravitite(art, f, mat="gravitite"):
    handle(art, f, 3, 57, 2.2, _pal("#2a2234", 0.3), grip=(5, 18, mpal(mat), "spiral"))
    ring = f.disc(55, 0, 4.0) & ~f.disc(55, 0, 2.4)
    art.paint(ring, dome(ring, 1.2), STEEL, base=0.5, spec=0.4)
    m = f.poly([(57, -4), (68, -7), (70.5, -21), (61, -18), (50, -22.5), (46.5, -12)])
    metal(art, f, m, dome(m, 2.2), mat, base=0.55)


def sh_gravitite(art, f, mat="gravitite"):
    shovel_base(art, f, _pal("#2a2234", 0.3), grip=(5, 16, mpal(mat), "spiral"), u1=46)
    ring = f.disc(48, 0, 3.8) & ~f.disc(48, 0, 2.2)
    art.paint(ring, dome(ring, 1.2), STEEL, base=0.5, spec=0.4)
    m, h = blade_along(f, [53, 55, 66, 72, 75], [3.0, 6.0, 6.0, 3.4, 0.4], hmax=1.8, dish=0.5)
    metal(art, f, m, h, mat, base=0.55)


def pk_resonarium(art, f, mat="resonarium"):
    # A tuning fork: two parallel prongs on each side, ringing.
    handle(art, f, 3, 54, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3a2c4a", 0.4), "cross"))
    m = np.zeros((N, N), bool)
    for c in (54, 59.5):
        vs = np.linspace(-23, 23, 47)
        w = np.interp(np.abs(vs), [0, 20, 23], [1.6, 1.4, 0.3])
        mm, _, _ = band(f, vs, arc(vs, c, 6), w, w)
        m |= mm
    m |= f.box(51, 61, -3.4, 3.4)
    metal(art, f, m, dome(m, 1.8), mat, base=0.55)


def ax_resonarium(art, f, mat="resonarium"):
    handle(art, f, 3, 62, 2.3, DARK_GRIP, grip=(4, 18, _pal("#3a2c4a", 0.4), "cross"))
    m = f.poly([(50.5, 4), (61.5, 4), (61.5, -3), (65, -18), (46, -18), (50.5, -3)])
    slots = (np.abs(((f.U - 47) % 4.6) - 2.3) < 0.8) & (f.V < -6) & (f.V > -16)
    metal(art, f, m & ~slots, dome(m & ~slots, 2.0), mat, base=0.55)


def sh_resonarium(art, f, mat="resonarium"):
    shovel_base(art, f, DARK_GRIP, grip=(4, 16, _pal("#3a2c4a", 0.4), "cross"), u1=48, sock=(43, 50), mat=mat)
    m, h = blade_along(f, [49, 51, 70, 75], [3.0, 6.4, 6.4, 0.4], hmax=1.8)
    slots = (np.abs(f.V) < 0.8) & (f.U > 54) & (f.U < 68)
    metal(art, f, m & ~slots, h, mat, base=0.55)


for _m in ("cobalt", "palladium", "mythril", "orichalcum", "adamantite", "titanium", "radiant", "verdite", "lumite",
           "skyroot", "holystone", "zanite", "gravitite", "resonarium"):
    PICKAXES[_m] = globals()[f"pk_{_m}"]
    AXES[_m] = globals()[f"ax_{_m}"]
for _m in ("umbral", "sanguinite"):
    PICKAXES[_m] = globals()[f"pk_{_m}"]
for _m in ("skyroot", "holystone", "zanite", "gravitite", "resonarium"):
    SHOVELS[_m] = globals()[f"sh_{_m}"]


# ---------------------------------------------------------------- 3x3 hammers, excavators, sickles
# One look per metal, sharing the family traits of that metal's pickaxe: iron plain forged with
# a leather grip, diamond cut in facets on a steel core, tungsten angular with plate seams,
# platinum with the blue-corded handle and gold fittings.

def _family_handle(art, f, mat, u1):
    if mat == "diamond":
        return handle(art, f, 3, u1, 2.5, DARK_GRIP, grip=(4, 18, _pal("#3c4048", 0.4), "bands"), rings=[(22, STEEL)])
    if mat == "tungsten":
        return handle(art, f, 3, u1, 2.6, DARK_GRIP, grip=(4, 20, _pal("#4a4e48", 0.35), "cross"), rings=[(u1 - 8, STEEL)])
    if mat == "platinum":
        hm = handle(art, f, 3, u1, 2.5, _pal("#ece6d6", 0.4), cap=mpal("golden"))
        k = np.minimum(np.abs(((f.U + f.V * 1.1) / 4.2) % 1 - 0.5), np.abs(((f.U - f.V * 1.1) / 4.2) % 1 - 0.5))
        art.paint(hm & (k < 0.2) & (f.U < 30), np.ones((N, N)) * 1.8, BLUE_CORD, base=0.5, outline=False)
        return hm
    return handle(art, f, 3, u1, 2.6, WOOD, grip=(4, 17, LEATHER_GRIP, "cross"))


def _family_metal(art, f, m, mat):
    if mat == "diamond":
        fac = ((np.floor((f.U * 0.7 + f.V) / 4.0) % 2) - 0.5) * 0.26 + np.where(f.V < 0, 0.1, -0.08)
        metal(art, f, m, np.ones((N, N)), mat, base=0.55, extra=fac, diff=0.0)
    elif mat == "tungsten":
        plates = (np.abs(((f.V + 30) % 5.0) - 2.5) < 0.45) | (np.abs(((f.U + 30) % 6.0) - 3.0) < 0.4)
        metal(art, f, m, dome(m, 2.4), mat, base=0.5, extra=plates * -0.14)
    else:
        metal(art, f, m, dome(m, 2.8), mat, base=0.52)


def big_hammer(mat):
    """A two-handed sledge: a long haft and a big block of a head (it breaks 3x3)."""
    art = Art(f"{mat}_hammer")
    f = ZFrame(1.1, 30.0, 4.0)
    _family_handle(art, f, mat, 54)
    if mat == "diamond":
        head = f.poly([(51, -15), (53.5, -18), (60.5, -18), (63, -15), (63, 15), (60.5, 18), (53.5, 18), (51, 15)])
    elif mat == "tungsten":
        head = f.box(50.5, 63.5, -18, 18) & ~(f.box(50.5, 53, -18, -14) | f.box(61, 63.5, 14, 18))
    elif mat == "platinum":
        # A mallet with a round drum and gold bands.
        head = f.disc(57, 0, 6.5, 18)
    else:
        head = f.box(51, 63, -16, 16) | f.box(49.5, 64.5, -18, -14) | f.box(49.5, 64.5, 14, 18)
    _family_metal(art, f, head, mat)
    if mat == "platinum":
        for v in (-8, 8):
            b = f.box(49, 65, v - 1.2, v + 1.2) & head
            art.paint(b, dome(b, 1.0) + 1.4, mpal("golden"), base=0.55, spec=0.4)
    return art.done()


def excavator(mat):
    """A broad, square-mouthed digging scoop on a long haft (it digs 3x3)."""
    art = Art(f"{mat}_excavator")
    f = ZFrame(1.1, 30.0, 6.0)
    _family_handle(art, f, mat, 48)
    sock = f.box(44, 51, -3.2, 3.2)
    art.paint(sock, dome(sock, 1.3), STEEL, base=0.45, spec=0.3)
    us = [50, 52, 70, 74.5]
    w = {"diamond": [4, 10, 10, 6], "tungsten": [5, 10.5, 10.5, 10.5], "platinum": [3.5, 9.5, 9.5, 2.0]}.get(mat, [4.5, 10, 10, 8.5])
    m, h = blade_along(f, us, w, hmax=1.8, dish=0.8)
    if mat == "iron":
        # Teeth on the mouth.
        m &= ~((f.U > 71.5) & (np.abs(((f.V + 30) % 4.0) - 2.0) < 0.8))
    _family_metal(art, f, m, mat)
    return art.done()


def sickle(mat):
    """A reaping sickle: a short handle and a long crescent blade (it cuts 3x3 of crops)."""
    art = Art(f"{mat}_sickle")
    f = ZFrame(1.15, 30.0, -2.0)
    _family_handle(art, f, mat, 40)
    # A crescent swept as a row of discs: from the ferrule up the line of the handle, over the
    # top and round toward the hand, thinning to a point.
    r = {"platinum": 13.5, "tungsten": 11.5}.get(mat, 12.5)
    thick = {"tungsten": 4.6, "diamond": 4.2}.get(mat, 3.8)
    cu, cv = 41 + r * 0.55, -r * 0.85
    m = f.box(38, 43, -3, 3)
    for t in np.linspace(0, 1, 60):
        th = np.deg2rad(200 - t * 250)
        u = cu + r * np.cos(th) * -1
        v = cv + r * np.sin(th) * -1
        m |= f.disc(u, v, max(thick * (1 - t) ** 0.6, 0.5))
    _family_metal(art, f, m, mat)
    return art.done()


for _m in ("iron", "diamond", "platinum", "tungsten"):
    EXTRA_ITEMS[f"{_m}_hammer"] = lambda m=_m: big_hammer(m)
    EXTRA_ITEMS[f"{_m}_excavator"] = lambda m=_m: excavator(m)
    EXTRA_ITEMS[f"{_m}_sickle"] = lambda m=_m: sickle(m)


# ---------------------------------------------------------------- lote 4: the missing Terraria tools

MAT["meteorite"] = dict(base="#8a4a36", spread=0.55, sat=1.15, kind="dark", spec=0.25, power=10)
MAT["spectral"] = dict(base="#8cc0f4", spread=0.75, sat=1.05, kind="polish", spec=0.5, power=22)
MAT["sporite"] = dict(base="#4a78dc", spread=0.7, sat=1.15, kind="steel", spec=0.4, power=16)
MAT["solar"] = dict(base="#f28a2c", spread=0.75, sat=1.2, warm=0.2, kind="gold", spec=0.45, power=14)
MAT["vortex"] = dict(base="#3cc8a0", spread=0.7, sat=1.15, kind="polish", spec=0.45, power=20)
MAT["nebula"] = dict(base="#c25ae0", spread=0.7, sat=1.15, kind="gem", spec=0.5, power=22)
MAT["stardust"] = dict(base="#80bcf2", spread=0.75, sat=1.05, kind="polish", spec=0.5, power=22)


def spectral_pickaxe():
    """Spectral: a pale pick whose arms trail off into wisps, with a soul-blue core."""
    art = Art("spectral_pickaxe")
    f = ZFrame(*ZOOM["pickaxe"])
    handle(art, f, 3, 54, 2.3, _pal("#2c3448", 0.35), grip=(4, 18, _pal("#4a5a7a", 0.4), "spiral"))
    vs = np.linspace(-26, 26, 53)
    w = np.interp(np.abs(vs), [0, 8, 16, 22, 26], [3.8, 3.0, 2.0, 1.2, 0.3])
    c = arc(vs, 57, 8) + np.sin(np.abs(vs) * 0.7) * (np.abs(vs) > 12) * 1.4
    m, h, s = band(f, vs, c, w, w)
    metal(art, f, m, h, "spectral", base=0.56)
    core = f.disc(57, 0, 2.4)
    art.paint(core, dome(core, 1.6), GLOW_TEAL, base=0.62, glow=core * 0.7)
    return art.done()


def digging_claw():
    """Sporite digging claw: a gauntlet with three long curved claws, worn over the fist."""
    art = Art("sporite_digging_claw")
    f = Frame()
    glove = f.poly([(12, -9), (34, -10), (40, -6), (40, 7), (34, 10), (12, 9), (8, 0)])
    art.paint(glove, dome(glove, 3.0), _pal("#3a3e4a", 0.45), base=0.5, spec=0.25)
    cuff = f.box(8, 15, -9.5, 9.5)
    metal(art, f, cuff, dome(cuff, 1.6), "sporite", base=0.5)
    m = np.zeros((N, N), bool)
    for v in (-6, 0, 6):
        pts = [f.xy(38, v), f.xy(52, v - 1.0), f.xy(62, v - 3.5), f.xy(70, v - 7.5)]
        m |= polyline(pts[:2], 4.0) | polyline(pts[1:3], 3.0) | polyline(pts[2:], 1.8)
    metal(art, f, m, dome(m, 1.6), "sporite", base=0.55)
    art.dots([f.xy(26, 0)], 2.0, _pal("#7ab0ff", 0.6, 1.2))
    return art.done()


EXTRA_ITEMS["meteor_hamaxe"] = lambda: hamaxe("meteor_hamaxe", "meteorite")
EXTRA_ITEMS["verdite_drill"] = lambda: drill("verdite")
EXTRA_ITEMS["verdite_chainsaw"] = lambda: chainsaw("verdite")
EXTRA_ITEMS["spectral_pickaxe"] = spectral_pickaxe
EXTRA_ITEMS["spectral_hamaxe"] = lambda: hamaxe("spectral_hamaxe", "spectral")
EXTRA_ITEMS["sporite_digging_claw"] = digging_claw
for _m in ("solar", "vortex", "nebula", "stardust"):
    EXTRA_ITEMS[f"{_m}_drill"] = lambda m=_m: drill(m)
    EXTRA_ITEMS[f"{_m}_hamaxe"] = lambda m=_m: hamaxe(f"{m}_hamaxe", m)


# ---------------------------------------------------------------- tools built from parts (gear_*)
# The parts mod's 108 pickaxes, axes and shovels: head material, rod and binding each show, so
# a flint pick on a bone rod with leather looks different from a copper one on a blaze rod.

GEAR_HEAD = {"flint": "stone", "copper": "copper", "iron": "iron", "gold": "golden", "diamond": "diamond"}
BONE = _pal("#e4dcc4", 0.5, 1.0)
BLAZE = _pal("#f0a02a", 0.6, 1.2)


def gear_tool(kind, head, rod, bind):
    global handle
    art = Art(f"gear_{kind}_{head}_{rod}_{bind}")
    f = ZFrame(*ZOOM[kind])
    plain = handle

    def rod_handle(art_, f_, u0, u1, w=2.4, pal=None, grip=None, rings=(), cap=None, bend=0.0):
        pal = {"stick": WOOD, "bone": BONE, "blaze_rod": BLAZE}[rod]
        m = plain(art_, f_, u0, u1, w * (1.12 if rod == "bone" else 1.0), pal, bend=bend)
        if rod == "bone":
            knob = f_.disc(u0 + 1.5, 0, w + 1.4) | f_.disc(u0 + 1.5, 0, w + 1.4)
            art_.paint(knob, dome(knob, 1.6), BONE, base=0.55)
        elif rod == "blaze_rod":
            art_.paint(m, np.zeros((N, N)) + 1.0, BLAZE, base=0.6, glow=m * 0.35, outline=False)
        return m

    handle = rod_handle
    try:
        mat = GEAR_HEAD[head]
        KINDS[kind][mat](art, f, mat)
    finally:
        handle = plain
    u0, u1 = {"pickaxe": (47, 53), "axe": (47, 54), "shovel": (42, 48)}[kind]
    lashing(art, f, u0, u1, 3.4, CORD if bind == "string" else LEATHER_GRIP)
    return art.done()


for _k in ("pickaxe", "axe", "shovel"):
    for _h in GEAR_HEAD:
        for _r in ("stick", "bone", "blaze_rod"):
            for _b in ("string", "leather"):
                EXTRA_ITEMS[f"gear_{_k}_{_h}_{_r}_{_b}"] = (lambda k=_k, h=_h, r=_r, b=_b: gear_tool(k, h, r, b))
