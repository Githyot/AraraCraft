"""Blocks lote B1 (plan/blocos-analise.md): vanilla blocks the game was missing.

Polished stones, chiseled and cracked variants, quartz pillar and bricks, red nether bricks,
bone, honey, target, tinted glass, deepslate ores, froglights, dead corals, stripped logs,
concrete powder, the missing terracotta colours and glazed terracotta. All drawn here from our
own textures and the shared style (style.py); nothing is copied.

`BLOCKS` maps texture names to generators (merged into blocks.REGISTRY). `write_designs()`
puts a 16x16 design of each into src16/ so build.py picks them up like any other block.
"""

import os

import numpy as np
from PIL import Image

from style import (N, XX, YY, blur, canvas, clusters, fbm, hexrgb, light, luma, normalize, paint, quantize, ramp, rgba,
                   rng_for, soften, stretched_noise, to_oklab, from_oklab, value_noise, worley)

HERE = os.path.dirname(os.path.abspath(__file__))
PACK = os.path.join(HERE, "..", "..", "packs", "vanilla", "textures", "block")
SRC16 = os.path.join(HERE, "src16", "block")

COLORS = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple",
          "blue", "brown", "green", "red", "black"]
DYE = {
    "white": "#e6e9e9", "orange": "#ea7a1c", "magenta": "#b84bb0", "light_blue": "#3caed8", "yellow": "#f6c22c",
    "lime": "#72b81c", "pink": "#ec8cac", "gray": "#40464a", "light_gray": "#8e8e86", "cyan": "#158a92",
    "purple": "#7a2aaa", "blue": "#353a9c", "brown": "#724a2a", "green": "#556e1a", "red": "#a02822",
    "black": "#16161c",
}


def tex(name):
    """One of our 64x64 block pictures as float RGBA."""
    return np.asarray(Image.open(os.path.join(PACK, name + ".png")).convert("RGBA").resize((N, N), Image.NEAREST),
                      dtype=np.float64)


def mean_rgb(img):
    a = img[..., 3] > 127
    return img[a, :3].mean(0)


def bevel(img, width=2, lift=26.0, sink=0.72):
    """A worked block's edge: light along the top and left, shadow along the bottom and right."""
    out = img.copy()
    for i in range(width):
        k = (width - i) / width
        out[i, :, :3] = np.clip(out[i, :, :3] + lift * k, 0, 255)
        out[:, i, :3] = np.clip(out[:, i, :3] + lift * k * 0.8, 0, 255)
        out[N - 1 - i, :, :3] *= sink + (1 - sink) * (1 - k)
        out[:, N - 1 - i, :3] *= sink + (1 - sink) * (1 - k) * 1.1
    return out


def cracks(img, rng, count=4, length=40, dark=0.55):
    """Hairline cracks wandering across the block."""
    out = img.copy()
    for _ in range(count):
        x, y = rng.uniform(0, N, 2)
        a = rng.uniform(0, 2 * np.pi)
        for _ in range(length):
            a += rng.normal(0, 0.45)
            x = (x + np.cos(a)) % N
            y = (y + np.sin(a)) % N
            xi, yi = int(x), int(y)
            out[yi, xi, :3] *= dark
            out[(yi + 1) % N, (xi + 1) % N, :3] = np.clip(out[(yi + 1) % N, (xi + 1) % N, :3] * 1.12, 0, 255)
    return out


# ------------------------------------------------------------------ polished stones


def polished(name, src):
    """The stone ground flat: the grain stays, softer, with a sheen and a cut edge."""
    rng = rng_for(name)
    base = tex(src)
    rgb = base[..., :3]
    soft = np.stack([blur(rgb[..., c], 1) for c in range(3)], -1)
    m = rgb.reshape(-1, 3).mean(0)
    rgb = m + (soft * 0.65 + rgb * 0.35 - m) * 0.8
    # A broad soft sheen running corner to corner, like a polished slab catching the light.
    sheen = np.exp(-((XX - YY) / 22.0) ** 2) * 14 + fbm(rng, ((32, 0.6), (16, 0.4))) * 6
    rgb = np.clip(rgb + sheen[..., None], 0, 255)
    img = rgba(rgb)
    # Each face is one cut slab: a thin groove inset from the edge.
    g = (np.isin(XX, (3,)) | np.isin(YY, (3,))) & (XX >= 3) & (YY >= 3) & (XX <= 60) & (YY <= 60)
    g |= (np.isin(XX, (60,)) | np.isin(YY, (60,))) & (XX >= 3) & (YY >= 3) & (XX <= 60) & (YY <= 60)
    img[g, :3] *= 0.82
    hi = (np.isin(XX, (4,)) | np.isin(YY, (4,))) & (XX >= 4) & (YY >= 4) & (XX <= 59) & (YY <= 59)
    img[hi, :3] = np.clip(img[hi, :3] + 14, 0, 255)
    return soften(bevel(img, 2, 18, 0.8), "built")


# ------------------------------------------------------------------ carved and cracked


def chiseled(name, src, motif):
    """A block with a frame and a carved picture in the middle (lit as a height field)."""
    rng = rng_for(name)
    base = tex(src)
    m = mean_rgb(base)
    pal = ramp(m, 12, 0.42, 0.12)
    h = np.zeros((N, N))
    # Raised frame round the face.
    edge = np.minimum(np.minimum(XX, YY), np.minimum(N - 1 - XX, N - 1 - YY))
    h += np.clip(6 - edge, 0, 3) * 0.0 + (edge < 6) * 2.0
    cx = cy = 31.5
    r = np.hypot(XX - cx, YY - cy)
    if motif == "rings":
        h += ((r > 9) & (r < 13)) * 1.6 + ((r > 18) & (r < 21)) * 1.2 + (r < 4) * 1.8
    elif motif == "diamond":
        d = np.abs(XX - cx) + np.abs(YY - cy)
        h += ((d > 10) & (d < 15)) * 1.8 + (d < 5) * 2.0 + ((np.abs(XX - cx) < 1.5) | (np.abs(YY - cy) < 1.5)) * (d < 22) * 0.8
    elif motif == "flutes":
        h += ((XX % 10) > 2) * ((YY > 10) & (YY < 54)) * 1.4
    elif motif == "sun":
        ang = np.arctan2(YY - cy, XX - cx)
        rays = (np.cos(ang * 8) > 0.55) & (r > 10) & (r < 22)
        h += rays * 1.4 + (r < 8) * 2.0 + ((r > 8) & (r < 10)) * 0.4
    h = blur(h, 1, wrap=False)
    surf = fbm(rng, ((16, 0.4), (8, 0.35), (4, 0.25)))
    t = 0.48 + (surf - 0.5) * 0.25 + light(h * 1.6, wrap=False) * 1.4 + (h > 0.5) * 0.05
    img = rgba(quantize(np.clip(t, 0, 1), pal, 0.3, rng))
    return soften(bevel(img, 1, 18, 0.75), "built")


def cracked(name, src):
    rng = rng_for(name)
    return soften(cracks(tex(src), rng, 5, 48), "built")


def quartz_pillar_side():
    rng = rng_for("quartz_pillar")
    pal = ramp("#e9e3d8", 12, 0.24, 0.06)
    flute = (XX % 16)
    h = np.where(flute < 13, np.sin(flute / 13 * np.pi) * 2.2, 0)
    surf = fbm(rng, ((32, 0.5), (8, 0.3), (4, 0.2)))
    t = 0.55 + (surf - 0.5) * 0.15 + light(h * 1.5) * 1.2
    img = rgba(quantize(np.clip(t, 0, 1), pal, 0.25, rng))
    return soften(img, "built")


def quartz_pillar_top():
    rng = rng_for("quartz_pillar_top")
    pal = ramp("#e9e3d8", 12, 0.24, 0.06)
    d = np.maximum(np.abs(XX - 31.5), np.abs(YY - 31.5))
    h = (d < 28) * 1.5 + (d < 20) * 0.0 - ((d > 19) & (d < 21)) * 1.0 + (d < 8) * 1.0
    surf = fbm(rng, ((32, 0.5), (8, 0.3), (4, 0.2)))
    t = 0.55 + (surf - 0.5) * 0.15 + light(blur(h, 1, False) * 1.5, wrap=False) * 1.2
    return soften(bevel(rgba(quantize(np.clip(t, 0, 1), pal, 0.25, rng)), 1, 14, 0.8), "built")


def bricks_from(name, base, bh=16, bw=32):
    import blocks
    return blocks.stone_bricks(name, base)


# ------------------------------------------------------------------ bone, honey, target, glass


def bone_side():
    rng = rng_for("bone_block_side")
    pal = ramp("#e4dcc2", 12, 0.3, 0.1)
    fib = stretched_noise(rng, 2, 32) * 0.6 + stretched_noise(rng, 4, 64) * 0.4
    pores = clusters(rng, 2, 0.12)
    h = fib * 2.0 - pores * 1.2
    t = 0.55 + (fib - 0.5) * 0.35 - pores * 0.25 + light(h) * 0.9
    return soften(bevel(rgba(quantize(np.clip(t, 0, 1), pal, 0.3, rng)), 2, 14, 0.78), "natural")


def bone_top():
    rng = rng_for("bone_block_top")
    pal = ramp("#e4dcc2", 12, 0.34, 0.1)
    img = bone_side()
    r = np.hypot(XX - 31.5, YY - 31.5)
    spongy = (value_noise(rng, 2) > 0.55) & (r < 22)
    ring = (r > 22) & (r < 30)
    t = np.where(ring, 0.7 + (value_noise(rng, 4) - 0.5) * 0.2, 0.5)
    t = np.where(spongy, 0.2, t)
    t = np.where(r < 22, t + (value_noise(rng, 4) - 0.5) * 0.15, t)
    core = rgba(quantize(np.clip(t, 0, 1), pal, 0.2, rng))
    m = r < 30
    img[m] = core[m]
    return img


def honey(name, top=False):
    rng = rng_for(name)
    pal = ramp("#f0a420", 12, 0.38, 0.14, sat=1.15)
    f = fbm(rng, ((32, 0.5), (16, 0.3), (8, 0.2)))
    t = 0.5 + (f - 0.5) * 0.4
    img = rgba(quantize(np.clip(t, 0, 1), pal, 0.2, rng), 200.0)
    # Trapped bubbles with a bright speck.
    for cx, cy in rng.uniform(6, 58, (9, 2)):
        rr = rng.uniform(1.5, 3.2)
        d = np.hypot(XX - cx, YY - cy)
        img[d < rr, :3] = np.clip(img[d < rr, :3] * 1.15, 0, 255)
        img[(np.abs(XX - (cx - rr * 0.4)) < 0.8) & (np.abs(YY - (cy - rr * 0.4)) < 0.8), :3] = (255, 244, 200)
    # A thicker darker wall round the sides; the top is clear through.
    if not top:
        edge = np.minimum(np.minimum(XX, YY), np.minimum(N - 1 - XX, N - 1 - YY))
        w = edge < 3
        img[w, :3] *= 0.72
        img[w, 3] = 235
    img[0, :, :3] = np.clip(img[0, :, :3] + 40, 0, 255)
    return img


def target(name):
    rng = rng_for(name)
    straw = ramp("#d9c48a", 10, 0.3, 0.1)
    red = ramp("#c02a26", 10, 0.3, 0.1)
    fib = stretched_noise(rng, 2, 16) * 0.7 + clusters(rng, 2) * 0.3
    r = np.hypot(XX - 31.5, YY - 31.5)
    band = np.floor(r / 7.5).astype(int)
    is_red = (band % 2 == 0) & (r < 30)
    t = 0.5 + (fib - 0.5) * 0.5
    img = rgba(quantize(np.clip(t, 0, 1), straw, 0.3, rng))
    rc = rgba(quantize(np.clip(t, 0, 1), red, 0.3, rng))
    img[is_red] = rc[is_red]
    return soften(bevel(img, 1, 12, 0.8), "built")


def tinted_glass():
    rng = rng_for("tinted_glass")
    pal = ramp("#3a2e44", 10, 0.3, 0.1)
    f = fbm(rng, ((32, 0.6), (16, 0.4)))
    img = rgba(quantize(np.clip(0.45 + (f - 0.5) * 0.3, 0, 1), pal, 0.2, rng), 180.0)
    edge = np.minimum(np.minimum(XX, YY), np.minimum(N - 1 - XX, N - 1 - YY))
    img[edge < 2, :3] = hexrgb("#1c1622")
    img[edge < 2, 3] = 245
    streak = (np.abs((XX + YY) - 40) < 1.5) & (XX > 10) & (XX < 30)
    streak |= (np.abs((XX + YY) - 52) < 0.8) & (XX > 16) & (XX < 32)
    img[streak, :3] = np.clip(img[streak, :3] + 60, 0, 255)
    img[streak, 3] = 210
    return img


# ------------------------------------------------------------------ ores, froglights, corals


def deepslate_ore(kind):
    """The vein of our stone ore set into deepslate: where the ore picture leaves grey stone."""
    ore = tex(f"{kind}_ore")
    stone = tex("deepslate")
    lab = to_oklab(ore[..., :3])
    chroma = np.hypot(lab[..., 1], lab[..., 2])
    vein = chroma > 0.045
    vein = vein | (blur(vein.astype(float), 1) > 0.55)
    out = stone.copy()
    out[vein, :3] = ore[vein, :3] * 0.94
    sh = np.roll(np.roll(vein, 1, 0), 1, 1) & ~vein
    out[sh, :3] *= 0.7
    return out


def froglight(name, base, top=False):
    rng = rng_for(name)
    pal = ramp(base, 12, 0.32, 0.12, sat=1.1, lift=0.08)
    if top:
        r = np.hypot(XX - 31.5, YY - 31.5)
        h = np.sin(r / 4.2) * 1.2 + fbm(rng, ((8, 0.6), (4, 0.4))) * 0.8
    else:
        cell = (YY // 16) * 4 + ((XX + (YY // 16 % 2) * 8) // 16)
        ids, d1, d2, _ = worley(rng, 14, 12)
        h = np.clip(d2 - d1, 0, 4) * 0.6 + fbm(rng, ((8, 0.6), (4, 0.4))) * 0.6
    t = 0.55 + (h - h.mean()) * 0.12 + light(h) * 0.9
    img = rgba(quantize(np.clip(t, 0, 1), pal, 0.25, rng))
    return soften(bevel(img, 2, 20, 0.85), "natural")


def dead_coral(src):
    img = tex(src)
    lab = to_oklab(img[..., :3])
    lab[..., 1:] *= 0.12
    lab[..., 0] = lab[..., 0] * 0.85 + 0.08
    out = img.copy()
    out[..., :3] = np.clip(from_oklab(lab), 0, 255)
    return out


# ------------------------------------------------------------------ stripped logs


def stripped_side(wood):
    import woods
    rng = rng_for(f"stripped_{wood}_log")
    face, _, light_c = woods.colours(wood)
    pal = ramp(np.asarray(face) * 0.4 + np.asarray(light_c) * 0.6, 12, 0.3, 0.1)
    # Long fibres running up the log, with a few darker streaks where the bark came off.
    fib = stretched_noise(rng, 1, 32) * 0.45 + stretched_noise(rng, 2, 64) * 0.35 + stretched_noise(rng, 4, 16) * 0.2
    streak = stretched_noise(rng, 8, 64) > 0.72
    h = fib * 2.2
    t = 0.52 + (fib - 0.5) * 0.45 - streak * 0.12 + light(h) * 0.8
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.3, rng)), "rough")


def stripped_top(wood):
    import woods
    rng = rng_for(f"stripped_{wood}_log_top")
    face, _, light_c = woods.colours(wood)
    pal = ramp(np.asarray(face) * 0.5 + np.asarray(light_c) * 0.5, 12, 0.32, 0.1)
    r = np.hypot(XX - 31.5 + (value_noise(rng, 16) - 0.5) * 4, YY - 31.5 + (value_noise(rng, 16) - 0.5) * 4)
    rings = (np.sin(r / 1.7 + value_noise(rng, 8) * 1.5) + 1) / 2
    edge = np.minimum(np.minimum(XX, YY), np.minimum(N - 1 - XX, N - 1 - YY))
    t = 0.5 + (rings - 0.5) * 0.42 + (fbm(rng, ((8, 0.5), (4, 0.5))) - 0.5) * 0.12
    t = np.where(edge < 3, 0.68 + stretched_noise(rng, 2, 16) * 0.1, t)
    t = np.where(r < 3, 0.3, t)
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.1, rng)), "rough")


# ------------------------------------------------------------------ colours


def concrete_powder(color):
    rng = rng_for(color + "_concrete_powder")
    c = mean_rgb(tex(color + "_concrete"))
    pal = ramp(c, 10, 0.2, 0.06, lift=0.03)
    grain = clusters(rng, 1, 0.5) * 0.5 + clusters(rng, 2, 0.4) * 0.3 + fbm(rng, ((16, 0.6), (8, 0.4))) * 0.2
    t = 0.5 + (grain - 0.5) * 0.7
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.4, rng)), "natural")


def terracotta_color(color):
    """Fired clay in a dye colour: the dye muted toward the clay's own brown."""
    rng = rng_for(color + "_terracotta")
    clay = hexrgb("#9a5f45")
    base = hexrgb(DYE[color]) * 0.5 + clay * 0.5
    if color == "black":
        base = hexrgb("#2c1d18")
    pal = ramp(base, 10, 0.18, 0.1)
    f = fbm(rng, ((32, 0.4), (16, 0.3), (8, 0.3)))
    c = clusters(rng, 2)
    t = 0.5 + (f - 0.5) * 0.3 + (c - 0.5) * 0.3
    t = t + light(f + c) * 0.8
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.35, rng)), "natural")


def glazed(color):
    """Glazed terracotta: a glossy tile with a motif of its own, turning round the centre."""
    rng = rng_for(color + "_glazed_terracotta")
    dye = hexrgb(DYE[color])
    if color == "white":
        accent = hexrgb("#4a8ac8")
    elif color in ("black", "gray"):
        accent = hexrgb("#c8a040")
    else:
        accent = np.clip(dye * 0.45 + 30, 0, 255)
    light_c = np.clip(dye * 0.55 + 255 * 0.45, 0, 255)
    # One quarter (32x32) from layered shapes, then turned four times round the centre.
    q = np.zeros((32, 32), int)
    y, x = np.mgrid[0:32, 0:32].astype(float)
    kind = int(rng.integers(0, 4))
    ang = np.arctan2(y - 31.5, x - 31.5)
    r = np.hypot(x - 31.5, y - 31.5)
    r0 = np.hypot(x, y)
    q[:] = 1
    if kind == 0:
        q[(r > 14) & (r < 20)] = 2
        q[r0 < 10] = 3
        q[(np.abs(x - y) < 2.5) & (r > 20)] = 2
    elif kind == 1:
        q[(np.abs(x - 16) + np.abs(y - 16)) < 9] = 2
        q[(np.abs(x - 16) + np.abs(y - 16)) < 4] = 3
        q[(x < 4) | (y < 4)] = 0
    elif kind == 2:
        q[(np.sin(r / 2.5) > 0.4) & (r < 26)] = 2
        q[r < 6] = 3
        q[(x + y) < 8] = 0
    else:
        q[(np.cos(ang * 6) > 0.6) & (r < 28) & (r > 8)] = 2
        q[(np.abs(x - 6) < 3) & (np.abs(y - 20) < 7)] = 3
        q[(np.abs(y - 6) < 3) & (np.abs(x - 20) < 7)] = 3
    full = np.zeros((N, N), int)
    full[:32, :32] = q
    full[:32, 32:] = np.rot90(q, 3)
    full[32:, 32:] = np.rot90(q, 2)
    full[32:, :32] = np.rot90(q, 1)
    cols = np.array([dye * 0.62, dye, light_c, accent])
    rgb = cols[full]
    # Glaze: tiny pooling darker in hollows, one broad gloss streak, a few crackle lines.
    f = fbm(rng, ((16, 0.5), (8, 0.5)))
    rgb = rgb * (0.94 + 0.1 * f)[..., None]
    gloss = np.exp(-(((XX + YY) - 44) / 6.0) ** 2) * 28
    rgb = np.clip(rgb + gloss[..., None], 0, 255)
    img = rgba(rgb)
    img = cracks(img, rng, 2, 26, 0.86)
    return bevel(img, 1, 16, 0.8)


# ------------------------------------------------------------------ registry

STONE_POLISH = {"polished_granite": "granite", "polished_diorite": "diorite", "polished_andesite": "andesite"}
WOODS = ["oak", "birch", "spruce", "jungle", "acacia", "dark_oak", "cherry", "mangrove", "pale_oak", "crimson",
         "warped", "maple", "walnut", "araucaria", "buriti", "coconut", "ipe", "jabuticaba", "echo", "skyroot"]
NEW_TERRACOTTA = ["magenta", "light_blue", "lime", "pink", "gray", "cyan", "purple", "blue", "green", "black"]


def stem_of(w):
    return "stem" if w in ("crimson", "warped") else "log"


BLOCKS = {}
for _n, _s in STONE_POLISH.items():
    BLOCKS[_n] = lambda n=_n, s=_s: polished(n, s)
BLOCKS.update({
    "chiseled_stone_bricks": lambda: chiseled("chiseled_stone_bricks", "stone_bricks", "rings"),
    "chiseled_nether_bricks": lambda: chiseled("chiseled_nether_bricks", "nether_bricks", "diamond"),
    "chiseled_quartz_block": lambda: chiseled("chiseled_quartz_block", "quartz_block", "flutes"),
    "chiseled_quartz_block_top": lambda: chiseled("chiseled_quartz_block_top", "quartz_block", "rings"),
    "chiseled_red_sandstone": lambda: chiseled("chiseled_red_sandstone", "red_sandstone_side", "sun"),
    "cracked_nether_bricks": lambda: cracked("cracked_nether_bricks", "nether_bricks"),
    "cracked_deepslate_tiles": lambda: cracked("cracked_deepslate_tiles", "deepslate_tiles"),
    "cut_red_sandstone": lambda: polished("cut_red_sandstone", "red_sandstone_side"),
    "red_nether_bricks": lambda: bricks_from("red_nether_bricks", "#5c1414"),
    "quartz_bricks": lambda: bricks_from("quartz_bricks", "#e6dfd2"),
    "quartz_pillar": quartz_pillar_side,
    "quartz_pillar_top": quartz_pillar_top,
    "bone_block_side": bone_side,
    "bone_block_top": bone_top,
    "honey_block_side": lambda: honey("honey_block_side"),
    "honey_block_top": lambda: honey("honey_block_top", True),
    "target_side": lambda: target("target_side"),
    "target_top": lambda: target("target_top"),
    "tinted_glass": tinted_glass,
    "ochre_froglight_side": lambda: froglight("ochre_froglight_side", "#f2d47a"),
    "ochre_froglight_top": lambda: froglight("ochre_froglight_top", "#f2d47a", True),
    "verdant_froglight_side": lambda: froglight("verdant_froglight_side", "#cfeab0"),
    "verdant_froglight_top": lambda: froglight("verdant_froglight_top", "#cfeab0", True),
    "pearlescent_froglight_side": lambda: froglight("pearlescent_froglight_side", "#eed2e6"),
    "pearlescent_froglight_top": lambda: froglight("pearlescent_froglight_top", "#eed2e6", True),
})
for _c in ("tube", "brain", "bubble", "fire", "horn"):
    BLOCKS[f"dead_{_c}_coral_block"] = lambda c=_c: dead_coral(f"{c}_coral_block")
for _w in WOODS:
    BLOCKS[f"stripped_{_w}_{stem_of(_w)}"] = lambda w=_w: stripped_side(w)
    BLOCKS[f"stripped_{_w}_{stem_of(_w)}_top"] = lambda w=_w: stripped_top(w)
for _c in COLORS:
    BLOCKS[f"{_c}_concrete_powder"] = lambda c=_c: concrete_powder(c)
    BLOCKS[f"{_c}_glazed_terracotta"] = lambda c=_c: glazed(c)
for _c in NEW_TERRACOTTA:
    BLOCKS[f"{_c}_terracotta"] = lambda c=_c: terracotta_color(c)


def write_designs():
    """Write each texture at 64x64 into the pack and a 16x16 design into src16/ (build.py's job list)."""
    from style import to_u8
    os.makedirs(SRC16, exist_ok=True)
    # Order matters a little: some pictures start from others made here.
    for name, fn in BLOCKS.items():
        img = Image.fromarray(to_u8(fn()), "RGBA")
        img.save(os.path.join(PACK, name + ".png"), optimize=True)
        img.resize((16, 16), Image.BOX).save(os.path.join(SRC16, name + ".png"), optimize=True)
    print(f"{len(BLOCKS)} B1 textures written")


if __name__ == "__main__":
    import sys
    sys.path.insert(0, HERE)
    write_designs()
