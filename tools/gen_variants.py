#!/usr/bin/env python3
"""Animal coats (Java's variants): recolours each animal's own skin into its other coats and
writes packs/vanilla/fauna/variants.ron (read by MobRegistry::apply_variants).

    python3 tools/gen_variants.py            # every coat
    python3 tools/gen_variants.py --sheet out.png   # also a preview sheet (tools/mob_preview.py)

How a coat is made: the pixels of the animal's coat (its fur, wool, feathers or skin, picked by
colour, never the eyes, nose, beak, collar or hooves) are re-shaded with a new colour ramp from
their own brightness, so every hair, stripe and relief detail of the original skin stays. Patterns
(calico patches, Siamese points, a spotted or striped wolf) are masks over the skin layout: by body
part (head, legs, tail) from the model's boxes, or by a smooth noise over the skin.

Skins are 64x128 model pixels drawn 4x (256x512); the lower half holds the relief layers, which
cover the same parts as the upper half, so they get the same pattern.
"""

import colorsys
import math
import os
import re
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
PACK = os.path.join(ROOT, "packs", "vanilla")
TEX = os.path.join(PACK, "textures", "entity")
FILES = ["mobs.ron", "mods/mobs.ron", "fase6/mobs.ron"]
K = 4  # texels per model pixel

PART = re.compile(r"\(size:\s*\((\d+),\s*(\d+),\s*(\d+)\),\s*origin:\s*\((-?\d+),\s*(-?\d+),\s*(-?\d+)\),\s*uv:\s*\((\d+),\s*(\d+)\)(?:,\s*anim:\s*(\w+))?")


def model(name):
    """(skin file name, [(w, h, d, u, v, anim)]) of a mob, models.ron winning over its own file."""
    text = open(os.path.join(PACK, "fauna", "models.ron"), encoding="utf-8").read()
    m = re.search(r'name:\s*"%s",\s*skin:\s*"([a-z0-9_]+)"' % name, text)
    if m:
        i = text.index("parts:", m.end())
        j = text.index("        ],", i)
        return m.group(1), [(int(g[0]), int(g[1]), int(g[2]), int(g[6]), int(g[7]), g[8] or "") for g in PART.findall(text, i, j)]
    for f in FILES:
        text = open(os.path.join(PACK, f), encoding="utf-8").read()
        m = re.search(r'name:\s*"%s",\s*display_name' % name, text)
        if not m:
            continue
        i = text.index("parts:", m.end())
        depth, j = 0, text.index("[", i)
        k = j
        while True:
            c = text[k]
            depth += c == "["
            depth -= c == "]"
            k += 1
            if depth == 0:
                break
        sk = re.search(r'skin:\s*Some\("([a-z0-9_]+)"\)', text[m.start():k + 2000])
        return (sk.group(1) if sk else name), [(int(g[0]), int(g[1]), int(g[2]), int(g[6]), int(g[7]), g[8] or "") for g in PART.findall(text, j, k)]
    raise SystemExit(f"no mob {name}")


def roles(parts):
    """Model pixel (x, y) of the upper half -> role: 'head', 'leg', 'body' or 'extra' (tail, mane...)."""
    vol = [w * h * d for w, h, d, u, v, a in parts]
    body = max((i for i, p in enumerate(parts) if not p[5].startswith(("Head", "Leg"))), key=lambda i: vol[i], default=-1)
    out = {}
    for i, (w, h, d, u, v, a) in enumerate(parts):
        role = "head" if a.startswith("Head") else "leg" if a.startswith("Leg") else "body" if i == body else "extra"
        for y in range(v, v + d + h):
            for x in range(u, u + 2 * d + 2 * w):
                if y < v + d and not (u + d <= x < u + d + 2 * w):
                    continue
                out.setdefault((x, y), role)
    return out


def hsv(c):
    return colorsys.rgb_to_hsv(c[0] / 255, c[1] / 255, c[2] / 255)


def lum(c):
    return (0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]) / 255


def hexc(s):
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def ramp(stops, t):
    """Colour at t (0..1) along evenly spaced stops."""
    t = min(max(t, 0.0), 1.0) * (len(stops) - 1)
    i = min(int(t), len(stops) - 2)
    f = t - i
    a, b = stops[i], stops[i + 1]
    return tuple(round(a[k] + (b[k] - a[k]) * f) for k in range(3))


def hashf(x, y, seed):
    n = (x * 374761393 + y * 668265263 + seed * 2147483647) & 0xFFFFFFFF
    n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0


def noise(x, y, cell, seed):
    """Smooth value noise in 0..1 over model pixels."""
    gx, gy = x / cell, y / cell
    x0, y0 = math.floor(gx), math.floor(gy)
    fx, fy = gx - x0, gy - y0
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a, b = hashf(x0, y0, seed), hashf(x0 + 1, y0, seed)
    c, d = hashf(x0, y0 + 1, seed), hashf(x0 + 1, y0 + 1, seed)
    return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy


class Coat:
    """One coat: `paint(px, t, x, y, role) -> rgb` for each coat pixel (t = its brightness in the
    coat, 0..1), `keep(px) -> bool` for pixels that are not coat."""

    def __init__(self, key, name, paint, biomes=(), weight=10):
        self.key, self.name, self.paint, self.biomes, self.weight = key, name, paint, list(biomes), weight


def plain(*hexes, gain=1.0, mid=0.5):
    """A solid coat along a ramp; gain < 1 flattens the original's stripes."""
    stops = [hexc(h) for h in hexes]
    return lambda c, t, x, y, r: ramp(stops, mid + (t - mid) * gain)


def patches(base, spots, cell, cut, seed, gain=1.0):
    """`base` coat with `spots` where smooth noise is above `cut` (patches, calico...)."""
    def f(c, t, x, y, r):
        n = noise(x, y % 64, cell, seed)
        for sp, ct in spots:
            if n > ct:
                return sp(c, t, x, y, r)
        return base(c, t, x, y, r)
    return f


def multi_patches(base, layers):
    """Several patch layers (noise cell, cut, seed, coat), first that hits wins."""
    def f(c, t, x, y, r):
        for cell, cut, seed, coat in layers:
            if noise(x, y % 64, cell, seed) > cut:
                return coat(c, t, x, y, r)
        return base(c, t, x, y, r)
    return f


def points(body, point, roles_of=("head", "leg", "extra")):
    """Siamese style: `point` coat on head, legs and tail, `body` elsewhere."""
    def f(c, t, x, y, r):
        return point(c, t, x, y, r) if r in roles_of else body(c, t, x, y, r)
    return f


def stripes(base, dark, every, width, seed):
    """Dark bands across the skin (striped wolves), wavy so they don't look ruled."""
    def f(c, t, x, y, r):
        yy = y % 64
        w = math.sin((x + 3 * noise(x, yy, 5, seed)) * 2 * math.pi / every)
        return dark(c, t, x, y, r) if w > 1 - width else base(c, t, x, y, r)
    return f


def hue_turn(to_h, sat=1.0, val=1.0):
    """Turn the coat's hue (keeping its shading): for parrots and axolotls."""
    def f(c, t, x, y, r):
        h, s, v = hsv(c)
        h2 = (to_h / 360.0 + (h - BASE_HUE[0])) % 1.0
        rr, gg, bb = colorsys.hsv_to_rgb(h2, min(1.0, s * sat), min(1.0, v * val))
        return (round(rr * 255), round(gg * 255), round(bb * 255))
    return f


BASE_HUE = [0.0]


# ---------------------------------------------------------------- which pixels are coat

def grey_fur(c):
    h, s, v = hsv(c)
    return s < 0.2 and v > 0.1


def not_dark(c):
    return hsv(c)[2] > 0.1


def hue_in(lo, hi, smin=0.15, vmin=0.12):
    def f(c):
        h, s, v = hsv(c)
        h *= 360
        inside = lo <= h <= hi if lo <= hi else (h >= lo or h <= hi)
        return inside and s >= smin and v >= vmin
    return f


def either(*fs):
    return lambda c: any(f(c) for f in fs)


def pale(c):
    h, s, v = hsv(c)
    return s < 0.2 and v > 0.55


# ---------------------------------------------------------------- the animals

CAT_COATS = [
    Coat("cat_tabby", "Malhado", plain("2a1d14", "6b4a2c", "a77d52", "d9b88c")),
    Coat("cat_black", "Preto e branco", lambda c, t, x, y, r: ramp([hexc("ece8e0"), hexc("fbf9f4")], t) if t > 0.86 else ramp([hexc("121216"), hexc("2c2b30"), hexc("46454c")], t)),
    Coat("cat_red", "Laranja", plain("6e2d0e", "b65a1f", "e08a3c", "f5c07a")),
    Coat("cat_siamese", "Siamês", points(plain("b9a58a", "e3d6bf", "f6efe2", gain=0.5), plain("2a1c15", "4a3326", "6d4c38", gain=0.6))),
    Coat("cat_british_shorthair", "Azul britânico", plain("3e4650", "66707c", "8c95a0", "b2bac4", gain=0.55)),
    Coat("cat_calico", "Tricolor", multi_patches(plain("cfc8bc", "efe9df", "fffdf8", gain=0.6), [(5, 0.66, 11, plain("6e2d0e", "c06422", "e89a4a")), (4, 0.7, 23, plain("141418", "2c2b30", "4b4a50"))])),
    Coat("cat_persian", "Persa", plain("8e6a46", "c9a67c", "e7cfa8", "f8ead2", gain=0.5)),
    Coat("cat_ragdoll", "Ragdoll", points(plain("cfc8bc", "efebe4", "fffdf9", gain=0.5), plain("4b3a30", "7a6352", "a48c78", gain=0.7))),
    Coat("cat_white", "Branco", plain("b8b4ad", "dedbd5", "f6f4ef", "ffffff", gain=0.5)),
    Coat("cat_jellie", "Jellie", patches(plain("35383d", "5d6168", "8a8f96"), [(plain("cfcac2", "efebe4", "fffdf8", gain=0.6), 0.62)], 5, 0, 31)),
    Coat("cat_all_black", "Todo preto", plain("0c0c10", "1d1d22", "33333a", gain=0.6), weight=3),
]

WOLF_COATS = [
    Coat("wolf_woods", "Da floresta", plain("2b2219", "54453a", "7d6a58", "a8957f"), ["Forest", "Flower Forest", "Birch Forest", "Dark Forest"]),
    Coat("wolf_ashen", "Cinzento", plain("4e5560", "7c8590", "a9b1ba", "d1d7dd"), ["Snowy Taiga", "Snowy Plains"]),
    Coat("wolf_black", "Negro", plain("0f0f12", "1f1f24", "36363d", "55555c"), ["Old Growth Taiga"], weight=5),
    Coat("wolf_chestnut", "Castanho", plain("3a2416", "6e4429", "a06a42", "cfa27a"), ["Old Growth Taiga", "Mata de Araucárias"], weight=5),
    Coat("wolf_rusty", "Ferrugem", plain("4a1d0c", "8e3e18", "c46a33", "e7a06b"), ["Sparse Jungle", "Jungle", "Bamboo Jungle"]),
    Coat("wolf_spotted", "Pintado", patches(plain("a46b33", "d19a5b", "efc68c"), [(plain("3a2414", "5c3b22", "7a5233"), 0.68)], 3, 0, 41), ["Savanna", "Windswept Savanna", "Cerrado"]),
    Coat("wolf_striped", "Listrado", stripes(plain("8f6d45", "c4a072", "e7cda0"), plain("2c2016", "4a3626", "634a35"), 5, 0.38, 47), ["Badlands", "Wooded Badlands", "Eroded Badlands"]),
    Coat("wolf_snowy", "Da neve", plain("a9adb3", "d6d9dd", "f2f4f6", "ffffff", gain=0.6), ["Grove", "Snowy Slopes", "Ice Spikes"]),
]

FROG_COATS = [
    Coat("frog_temperate", "Laranja (temperado)", plain("5a2a10", "9c4f1f", "cf7c39", "eab06a"), ["Swamp", "Pantanal"]),
    Coat("frog_warm", "Branco (quente)", plain("7b6f60", "b6aa98", "ded5c6", "f6f0e6"), ["Mangrove Swamp", "Jungle", "Bamboo Jungle", "Desert"]),
]

RABBIT_COATS = [
    Coat("rabbit_white", "Branco", plain("aaa6a0", "d6d3ce", "f4f2ee", "ffffff", gain=0.6), ["Snowy Plains", "Snowy Taiga", "Ice Spikes", "Grove", "Snowy Slopes"]),
    Coat("rabbit_black", "Preto", plain("0f0f12", "24232a", "3b3a42", "57565e"), weight=6),
    Coat("rabbit_black_white", "Malhado", patches(plain("cfcac2", "ece8e2", "fdfbf7", gain=0.6), [(plain("121216", "28272d", "403f46"), 0.55)], 4, 0, 53), weight=6),
    Coat("rabbit_gold", "Dourado", plain("6e4e1a", "b08c45", "dcbf7a", "f2e2b4"), ["Desert", "Badlands", "Caatinga"]),
    Coat("rabbit_salt", "Sal e pimenta", lambda c, t, x, y, r: ramp([hexc("2d2a26"), hexc("6b655c"), hexc("a7a094"), hexc("d8d2c6")], 0.5 + (t - 0.5) * 0.8 + (hashf(x, y, 59) - 0.5) * 0.45), weight=6),
]

HORSE_COATS = [
    Coat("horse_white", "Branco", plain("a5a19a", "d3d0ca", "efede9", "ffffff", gain=0.7)),
    Coat("horse_creamy", "Creme", plain("7b5b33", "b89463", "dcc093", "f2e2c2")),
    Coat("horse_brown", "Baio", plain("2e1b0e", "5a3a20", "84593a", "a87a57")),
    Coat("horse_black", "Preto", plain("0d0c0e", "1c1b20", "302e34", "4a474e")),
    Coat("horse_gray", "Tordilho", lambda c, t, x, y, r: ramp([hexc("3b3d42"), hexc("6e7178"), hexc("a3a6ac"), hexc("d4d6d9")], t + (noise(x, y % 64, 2, 61) - 0.5) * 0.35)),
    Coat("horse_dark_brown", "Castanho escuro", plain("1b0f08", "3a2213", "583520", "784c31")),
]

LLAMA_COATS = [
    Coat("llama_white", "Branca", plain("aaa59c", "d6d2cb", "f1eee9", "ffffff", gain=0.7)),
    Coat("llama_brown", "Marrom", plain("3a2414", "6b4527", "966540", "bf8c62")),
    Coat("llama_gray", "Cinza", plain("3e3e42", "6a6a70", "96969c", "c2c2c6")),
]

PARROT_COATS = [
    Coat("parrot_blue", "Azul", hue_turn(222, 1.0, 0.95)),
    Coat("parrot_green", "Verde", hue_turn(110, 0.95, 0.9)),
    Coat("parrot_cyan", "Ciano", hue_turn(185, 0.9, 1.0)),
    Coat("parrot_gray", "Cinza", hue_turn(0, 0.08, 0.95)),
]

AXOLOTL_COATS = [
    Coat("axolotl_wild", "Selvagem", plain("2a1f16", "57422f", "7e6448", "a48a6a")),
    Coat("axolotl_gold", "Dourado", plain("7a4f0e", "c48a22", "eab84b", "f8de8c")),
    Coat("axolotl_cyan", "Ciano", plain("4a7a86", "86b6c2", "b9dfe6", "e6f7f9")),
    Coat("axolotl_blue", "Azul (raro)", plain("1c2a73", "334bb0", "5a76d8", "93aaf0"), weight=1),
]

FOX_COATS = [Coat("fox_snow", "Raposa da neve", plain("8a8a8c", "c4c4c6", "e6e6e6", "fafafa", gain=0.8), ["Snowy Plains", "Snowy Taiga", "Grove", "Snowy Slopes", "Ice Spikes"])]

MOOSHROOM_COATS = [Coat("mooshroom_brown", "Marrom", plain("3b2412", "6e4824", "9a6a3c", "c8975f"), weight=1)]

COW_COATS = [
    Coat("cow_warm", "De clima quente", plain("4c1e0c", "8c3d1a", "b8642e", "d99055"), ["Savanna", "Desert", "Badlands", "Jungle", "Windswept Savanna", "Cerrado", "Caatinga", "Sparse Jungle", "Bamboo Jungle", "Mangrove Swamp"]),
    Coat("cow_cold", "De clima frio", plain("140e0a", "2c2018", "4a382a", "6b5442"), ["Snowy Plains", "Snowy Taiga", "Taiga", "Old Growth Taiga", "Grove", "Snowy Slopes", "Ice Spikes", "Windswept Hills", "Mountains"]),
]
PIG_COATS = [
    Coat("pig_warm", "De clima quente", patches(plain("6e3c22", "a8673d", "cf946a", "e8bc98"), [(plain("2a1810", "45291b", "5f3a27"), 0.66)], 4, 0, 71), COW_COATS[0].biomes),
    Coat("pig_cold", "De clima frio", plain("3a302a", "6b5a4c", "9a8670", "c4b29a"), COW_COATS[1].biomes),
]
CHICKEN_COATS = [
    Coat("chicken_warm", "De clima quente", plain("5a2a0e", "9a4e1e", "c87a3a", "e8ab6a"), COW_COATS[0].biomes),
    Coat("chicken_cold", "De clima frio", lambda c, t, x, y, r: ramp([hexc("1e1e22"), hexc("44444a"), hexc("7a7a80"), hexc("b0b0b4")], t + (hashf(x // 2, y // 2, 73) - 0.5) * 0.4), COW_COATS[1].biomes),
]

# Wool: Java's sheep colours (natural ones spawn: white 81.8 %, black, grey and light grey 5 %,
# brown 3 %, pink 0.16 %; the others only by dye).
DYES = [
    ("white", "Branca", "f9fffe", 8180), ("orange", "Laranja", "f9801d", 0), ("magenta", "Magenta", "c74ebd", 0),
    ("light_blue", "Azul-clara", "3ab3da", 0), ("yellow", "Amarela", "fed83d", 0), ("lime", "Verde-limão", "80c71f", 0),
    ("pink", "Rosa", "f38baa", 16), ("gray", "Cinza", "474f52", 500), ("light_gray", "Cinza-clara", "9d9d97", 500),
    ("cyan", "Ciano", "169c9c", 0), ("purple", "Roxa", "8932b8", 0), ("blue", "Azul", "3c44aa", 0),
    ("brown", "Marrom", "835432", 300), ("green", "Verde", "5e7c16", 0), ("red", "Vermelha", "b02e26", 0),
    ("black", "Preta", "1d1d21", 500),
]


def wool(hexs):
    c = hexc(hexs)
    dark = tuple(round(v * 0.52) for v in c)
    light = tuple(min(255, round(v * 1.08 + 18)) for v in c)
    return plain("%02x%02x%02x" % dark, hexs, "%02x%02x%02x" % light)


SHEEP_COATS = [Coat(f"sheep_{k}", n, wool(h), weight=w) for k, n, h, w in DYES[1:]]

# (mob, coat of its plain skin: (name, biomes, weight), which pixels are coat, other coats)
ANIMALS = [
    ("cat", ("Cinza malhado", [], 10), grey_fur, CAT_COATS),
    ("cat_tame", ("Cinza malhado", [], 10), grey_fur, CAT_COATS),
    ("wolf", ("Pálido", ["Taiga"], 10), grey_fur, WOLF_COATS),
    ("wolf_tame", ("Pálido", ["Taiga"], 10), grey_fur, WOLF_COATS),
    ("frog", ("Verde (frio)", ["Snowy Plains", "Snowy Taiga", "Taiga", "Grove"], 10), hue_in(50, 175), FROG_COATS),
    ("rabbit", ("Marrom", [], 10), lambda c: hsv(c)[1] < 0.45 and hsv(c)[2] > 0.1, RABBIT_COATS),
    ("horse", ("Alazão", [], 10), hue_in(0, 50, 0.1, 0.05), HORSE_COATS),
    ("llama", ("Creme", [], 10), not_dark, LLAMA_COATS),
    ("parrot", ("Vermelha", [], 10), lambda c: hsv(c)[1] > 0.25, PARROT_COATS),
    ("axolotl", ("Rosa (lucy)", [], 10), not_dark, AXOLOTL_COATS),
    ("fox", ("Vermelha", [], 10), hue_in(8, 45, 0.35), FOX_COATS),
    ("mooshroom", ("Vermelha", [], 30), hue_in(340, 15, 0.35), MOOSHROOM_COATS),
    ("cow", ("De clima temperado", [], 10), hue_in(10, 45, 0.18), COW_COATS),
    ("pig", ("De clima temperado", [], 10), lambda c: hsv(c)[2] > 0.25 and (hsv(c)[0] * 360 < 25 or hsv(c)[0] * 360 > 330), PIG_COATS),
    ("chicken", ("De clima temperado", [], 10), lambda c: hsv(c)[1] < 0.2 and hsv(c)[2] > 0.3, CHICKEN_COATS),
    ("sheep", ("Branca", [], 8180), lambda c: hsv(c)[1] < 0.16 and hsv(c)[2] > 0.42, SHEEP_COATS),
]

# Parts that are never coat whatever their colour (collars).
KEEP_PARTS = {"cat_tame": [5], "wolf_tame": [6]}


def recolour(src, mask_fn, coat, rmap, keep_rects):
    w, h = src.size
    px = src.load()
    cells = []
    for y in range(h):
        for x in range(w):
            c = px[x, y]
            if c[3] < 8:
                continue
            mx, my = x // K, y // K
            if any(u <= mx < u2 and v <= my % 64 < v2 for u, v, u2, v2 in keep_rects):
                continue
            if mask_fn(c):
                cells.append((x, y, lum(c)))
    if not cells:
        raise SystemExit("no coat pixels")
    ls = sorted(l for _, _, l in cells)
    lo, hi = ls[len(ls) // 20], ls[len(ls) * 19 // 20]
    span = max(hi - lo, 1e-3)
    out = src.copy()
    op = out.load()
    for x, y, l in cells:
        mx, my = x // K, y // K
        role = rmap.get((mx, my % 64), "body")
        c = px[x, y]
        rgb = coat.paint(c, (l - lo) / span, mx, my, role)
        op[x, y] = (max(0, min(255, rgb[0])), max(0, min(255, rgb[1])), max(0, min(255, rgb[2])), c[3])
    return out


def ron_str(s):
    return '"' + s.replace('"', '\\"') + '"'


# Skins of their own for animals built on another's model: (model mob, mask, coat -> TEX/<key>.png).
OWN_SKINS = [
    ("horse", hue_in(0, 50, 0.1, 0.05), Coat("zombie_horse", "Cavalo-Zumbi", patches(plain("23301c", "3e5230", "5e7448", "84966a", gain=0.8), [(plain("2a1c18", "4a2e28", "6e403a"), 0.7)], 3, 0, 97))),
    ("camel", lambda c: hsv(c)[1] > 0.12 and hsv(c)[2] > 0.15, Coat("fauna_camel_husk", "Camelo Ressecado", patches(plain("4a4034", "736552", "9a8a70", "beb092", gain=0.8), [(plain("2c241c", "3e342a", "54483a"), 0.66)], 4, 0, 101))),
    ("ghast", lambda c: hsv(c)[2] > 0.35, Coat("happy_ghast", "Ghast Feliz", plain("a8b4c4", "ccd6e2", "e6ecf4", "fbfcfe", gain=0.8))),
    ("cat", grey_fur, Coat("fauna_ocelot", "Jaguatirica", multi_patches(plain("8a5a1c", "c99536", "e8bf62", "f7dd98", gain=0.7), [(2, 0.72, 83, plain("1c140c", "2e2214", "45331f")), (3, 0.7, 89, plain("5a3a16", "7e5424", "9c6c34"))]))),
]


def happy_face(img):
    """The happy ghast's own face on the front of its 16-pixel cube (u 16, v 16): soft white,
    two round dark eyes with a glint, rosy cheeks and a small smile."""
    px = img.load()
    x0, y0 = 16 * K, 16 * K
    for y in range(16 * K):
        for x in range(16 * K):
            n = noise(x / K, y / K, 3, 7)
            v = int(232 + 18 * n)
            px[x0 + x, y0 + y] = (v - 6, v - 2, v, 255)

    def fill(mx, my, w, h, rgb):
        for y in range(my * K, (my + h) * K):
            for x in range(mx * K, (mx + w) * K):
                px[x0 + x, y0 + y] = rgb + (255,)
    for ex in (3, 10):
        fill(ex, 5, 3, 4, (34, 30, 40))
        fill(ex, 5, 1, 1, (250, 250, 255))
    fill(1, 10, 2, 1, (238, 170, 178))
    fill(13, 10, 2, 1, (238, 170, 178))
    fill(6, 11, 4, 1, (60, 44, 52))
    fill(5, 10, 1, 1, (60, 44, 52))
    fill(10, 10, 1, 1, (60, 44, 52))
    return img


def main():
    entries = []
    made = []
    for mob, mask_fn, coat in OWN_SKINS:
        skin, parts = model(mob)
        src = Image.open(os.path.join(TEX, skin + ".png")).convert("RGBA")
        if src.size != (64 * K, 128 * K):
            src = src.resize((64 * K, 128 * K), Image.NEAREST)
        hs = [hsv(c)[0] for c in src.getdata() if c[3] > 8 and mask_fn(c)]
        BASE_HUE[0] = sorted(hs)[len(hs) // 2] if hs else 0.0
        keep_rects = []
        for i in KEEP_PARTS.get(mob, []):
            w, h, d, u, v, a = parts[i]
            keep_rects.append((u, v, u + 2 * d + 2 * w, v + d + h))
        img = recolour(src, mask_fn, coat, roles(parts), keep_rects)
        if coat.key == "happy_ghast":
            img = happy_face(img)
        img.save(os.path.join(TEX, coat.key + ".png"))
        made.append(coat.key)
    for mob, base, mask_fn, coats in ANIMALS:
        skin, parts = model(mob)
        rmap = roles(parts)
        keep_rects = []
        for i in KEEP_PARTS.get(mob, []):
            w, h, d, u, v, a = parts[i]
            keep_rects.append((u, v, u + 2 * d + 2 * w, v + d + h))
        src = Image.open(os.path.join(TEX, skin + ".png")).convert("RGBA")
        if src.size != (64 * K, 128 * K):
            src = src.resize((64 * K, 128 * K), Image.NEAREST)
        # Coat hue of the plain skin (for hue turns).
        hs = [hsv(c)[0] for c in src.getdata() if c[3] > 8 and mask_fn(c)]
        BASE_HUE[0] = sorted(hs)[len(hs) // 2] if hs else 0.0
        lst = [f"(name: {ron_str(base[0])}, biomes: [{', '.join(ron_str(b) for b in base[1])}], weight: {base[2]})"]
        for coat in coats:
            key = coat.key if not mob.endswith("_tame") else coat.key.replace("cat_", "cat_tame_").replace("wolf_", "wolf_tame_")
            img = recolour(src, mask_fn, coat, rmap, keep_rects)
            img.save(os.path.join(TEX, f"var_{key}.png"))
            made.append(key)
            lst.append(f"(name: {ron_str(coat.name)}, skin: \"var_{key}\", biomes: [{', '.join(ron_str(b) for b in coat.biomes)}], weight: {coat.weight})")
        entries.append(f"    (mob: \"{mob}\", list: [\n        " + ",\n        ".join(lst) + ",\n    ]),")
    head = "// Animal coats, written by tools/gen_variants.py: entry k is coat k, entry 0 the plain skin.\n"
    text = head + "(variants: [\n" + "\n".join(entries) + "\n])\n"
    open(os.path.join(PACK, "fauna", "variants.ron"), "w", encoding="utf-8").write(text)
    print(f"{len(made)} coats for {len(ANIMALS)} animals")
    if "--sheet" in sys.argv:
        sheet(sys.argv[sys.argv.index("--sheet") + 1])


def sheet(path):
    """A 3/4 view of every coat, through tools/mob_preview.py."""
    sys.path.insert(0, HERE)
    import mob_preview as mp
    mobs = mp.load_mobs()
    text = open(os.path.join(PACK, "fauna", "variants.ron"), encoding="utf-8").read()
    tiles = []
    for mob, base, _, coats in ANIMALS:
        if mob.endswith("_tame"):
            continue
        m = mobs[mob]
        skins = [m.get("skin", mob)] + re.findall(r'skin: "(var_[a-z0-9_]+)"', text.split(f'(mob: "{mob}"', 1)[1].split("]),", 1)[0])
        for s in skins:
            im = Image.open(os.path.join(TEX, s + ".png")).convert("RGBA").resize((64, 128), Image.LANCZOS)
            tiles.append((s, mp.view(m, im) if hasattr(mp, "view") else None))
    print(len(tiles), "tiles")


if __name__ == "__main__":
    main()
