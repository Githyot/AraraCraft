#!/usr/bin/env python3
"""Town folk you can tell apart (plan/npcs-analise.md, lote N2).

Every townsperson was the player's body in different colours: same belt, same strap, same boots.
This script keeps each one's body and face (their skin design in tools/tex64/src16/entity) and
gives them what their trade wears, in volume: hats, caps, hoods and crowns, beards, aprons and
skirts, packs, belts, pauldrons, goggles, glasses, a crook. Each piece is its own box riding on
the head, body or an arm (motion `follow`), painted here at the design's size in the free
corners of the skin, then grown to 4x like every other skin.

Writes packs/vanilla/npcs/models.ron (laid over the mob lists after loading and after
fauna/models.ron, like it: only parts, layers, motion and skin change) and one skin per NPC,
packs/vanilla/textures/entity/folk_<name>.png (+ its design in src16/entity).

    python3 tools/gen_npcs.py
    python3 tools/mob_preview.py --sheet out.png npc_wizard npc_pirate ...

Pieces stay within 14 parts (slot 14 is the mouth, 15 the eyelids).
"""

import math
import os
import re
import sys
import zlib

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "tex64"))
from gen_mob_skins import face_rects, fbm, hash01, hexrgb, local_pos, mix  # noqa: E402
from mob_preview import load_mobs  # noqa: E402

MOBS = load_mobs(overlays=False)

ROOT = os.path.dirname(HERE)
PACK = os.path.join(ROOT, "packs", "vanilla")
OUT_RON = os.path.join(PACK, "npcs", "models.ron")
ENTITY = os.path.join(PACK, "textures", "entity")
SRC16 = os.path.join(HERE, "tex64", "src16", "entity")
W, H = 64, 128
MAX = 14

# The player-shaped body every townsperson has (mobs.ron), with its relief layers.
BASE = [
    ((8, 8, 8), (-4, 24, -4), (0, 0), "Head", (0, 24, 0)),
    ((8, 12, 4), (-4, 12, -2), (16, 16), None, None),
    ((4, 12, 4), (-8, 12, -2), (40, 16), "Arm(0)", (-6, 22, 0)),
    ((4, 12, 4), (4, 12, -2), (32, 48), "Arm(1)", (6, 22, 0)),
    ((4, 12, 4), (-4, 0, -2), (0, 16), "Leg(0)", (-2, 12, 0)),
    ((4, 12, 4), (0, 0, -2), (16, 48), "Leg(1)", (2, 12, 0)),
]
BASE_LAYERS = [(0, 0.5), (1, 0.3), (2, 0.3), (3, 0.3), (4, 0.3), (5, 0.3)]
HEAD, BODY, ARM_R, ARM_L, LEG_R, LEG_L = range(6)


# ---------------------------------------------------------------- colours


def pal(*hexes):
    return [hexrgb(h) for h in hexes]


def ramp(p, t):
    t = max(0.0, min(0.999, t))
    k = t * (len(p) - 1)
    i = int(k)
    return mix(p[i], p[min(i + 1, len(p) - 1)], k - i)


def dim(rgb, k):
    return tuple(max(0, min(255, int(round(c * k)))) for c in rgb)


FACE_LIGHT = {"top": 1.12, "front": 1.0, "left": 0.86, "right": 0.86, "back": 0.78, "bottom": 0.62}


class Px:
    """One face pixel of a piece."""

    def __init__(self, face, i, j, fw, fh, size, origin, seed):
        self.face, self.i, self.j, self.fw, self.fh = face, i, j, fw, fh
        self.w, self.h, self.d = size
        self.lx, self.ly, self.lz = local_pos(face, i, j, size)
        self.gx, self.gy, self.gz = origin[0] + self.lx, origin[1] + self.ly, origin[2] + self.lz
        self.seed = seed
        self.n = fbm(seed, self.gx, self.gy, self.gz, 3.0)
        self.jit = hash01(seed, int(self.gx * 8), int(self.gy * 8), int(self.gz * 8), len(face))

    def edge(self):
        """Distance in pixels to the nearest edge of this face (0 on the rim)."""
        return min(self.i, self.j, self.fw - 1 - self.i, self.fh - 1 - self.j)

    def up(self):
        return self.ly / max(1, self.h)

    def side(self):
        return self.face in ("left", "right", "front", "back")


# ---------------------------------------------------------------- materials


def cloth(p, rough=0.5, rim=0.82):
    """Woven stuff: soft noise, a darker seam at the rim, a light fold now and then."""
    def f(c):
        t = 0.55 + (c.n - 0.5) * rough + (c.jit - 0.5) * 0.12
        rgb = ramp(p, t)
        if c.edge() == 0 and c.face not in ("top", "bottom"):
            rgb = dim(rgb, rim)
        return dim(rgb, FACE_LIGHT[c.face])
    return f


def felt(p):
    return cloth(p, 0.35, 0.78)


def leather(p):
    def f(c):
        t = 0.5 + (c.n - 0.5) * 0.8 + (c.jit - 0.5) * 0.08
        rgb = ramp(p, t)
        if c.edge() == 0:
            rgb = dim(rgb, 0.7)
        # Stitching a pixel in from the rim.
        if c.edge() == 1 and (c.i + c.j) % 2 == 0 and c.side():
            rgb = mix(rgb, p[-1], 0.45)
        return dim(rgb, FACE_LIGHT[c.face])
    return f


def straw(p):
    def f(c):
        weave = ((int(c.gx) + int(c.gz) + int(c.gy)) % 2) * 0.18
        t = 0.45 + weave + (c.jit - 0.5) * 0.25
        rgb = ramp(p, t)
        if c.edge() == 0:
            rgb = dim(rgb, 0.8)
        return dim(rgb, FACE_LIGHT[c.face])
    return f


def metal(p, shine=(250, 250, 245)):
    """Metal: a bright band where the light catches, dark rims, a few glints."""
    def f(c):
        band = math.exp(-((c.up() - 0.72) ** 2) / 0.02) if c.side() else (0.7 if c.face == "top" else 0.0)
        t = 0.35 + 0.45 * band + (c.n - 0.5) * 0.15
        rgb = ramp(p, t)
        if c.edge() == 0:
            rgb = dim(rgb, 0.62)
        if c.jit > 0.965:
            rgb = mix(rgb, shine, 0.7)
        return dim(rgb, FACE_LIGHT[c.face] ** 1.3)
    return f


def gold(c):
    return metal(GOLD, (255, 250, 210))(c)


def fur(p):
    def f(c):
        t = 0.5 + (c.n - 0.5) * 1.1 + (c.jit - 0.5) * 0.4
        rgb = ramp(p, t)
        return dim(rgb, FACE_LIGHT[c.face] ** 0.7)
    return f


def leafy(p):
    def f(c):
        if c.face == "bottom":
            return dim(p[0], 0.7)
        leaf = hash01(c.seed, int(c.gx / 2), int(c.gy / 2), int(c.gz / 2))
        t = 0.3 + leaf * 0.6 + (c.jit - 0.5) * 0.15
        rgb = ramp(p, t)
        # Ragged hem: leaf tips cut out along the bottom row.
        if c.side() and c.j == c.fh - 1 and c.jit < 0.45:
            return None
        return dim(rgb, FACE_LIGHT[c.face])
    return f


def hair(p):
    def f(c):
        strand = ((int(c.gx * 1.0) * 7 + int(c.gz) * 3) % 5) / 5.0
        t = 0.35 + strand * 0.35 + (c.n - 0.5) * 0.3
        return dim(ramp(p, t), FACE_LIGHT[c.face] ** 0.8)
    return f


def solid(rgb, k=0.06):
    def f(c):
        return dim(mix(rgb, (255, 255, 255) if c.jit > 0.5 else (0, 0, 0), k * abs(c.jit - 0.5) * 2), FACE_LIGHT[c.face])
    return f


def wood(p):
    def f(c):
        grain = (math.sin(c.gy * 2.2 + c.n * 6.0) * 0.5 + 0.5) * 0.35
        return dim(ramp(p, 0.3 + grain + (c.jit - 0.5) * 0.1), FACE_LIGHT[c.face])
    return f


def glass(c):
    return dim(mix((40, 60, 80), (200, 230, 255), 0.25 + 0.6 * (c.edge() == 1 and c.i < c.fw / 2)), 1.0)


# ---- details laid over a material


def with_(base, extra):
    """`extra(c, rgb)` may change the pixel the base painted (or return None to cut it out)."""
    def f(c):
        rgb = base(c)
        if rgb is None:
            return None
        return extra(c, rgb)
    return f


def no_faces(base, *faces):
    """See-through on these faces (hoods open in front, crowns open on top)."""
    def f(c):
        return None if c.face in faces else base(c)
    return f


def band(rgb, rows):
    """A coloured band on the sides across these rows from the bottom (0 = bottom row)."""
    def f(c, cur):
        if c.side() and int(c.ly) in rows:
            return dim(rgb, FACE_LIGHT[c.face])
        return cur
    return f


def front_mark(pattern, rgb):
    """A mark on the front face: `pattern` lines of '#' over the face, centred."""
    rows = pattern.strip("\n").split("\n")

    def f(c, cur):
        if c.face != "front":
            return cur
        oy = (c.fh - len(rows)) // 2
        ox = (c.fw - len(rows[0])) // 2
        y, x = c.j - oy, c.i - ox
        if 0 <= y < len(rows) and 0 <= x < len(rows[y]) and rows[y][x] == "#":
            return dim(rgb, 1.0)
        return cur
    return f


def zigzag_top(c, cur):
    """Crown points: the side faces' upper row cut every other pixel."""
    if c.side() and c.j == 0 and (c.i % 2 == 1):
        return None
    if c.face == "top":
        return None
    if c.side() and c.j == 1 and c.i % 4 == 0:
        return (200, 30, 60)
    return cur


def studs(rgb):
    def f(c, cur):
        if c.side() and c.edge() == 1 and c.i % 3 == 1:
            return rgb
        return cur
    return f


GOLD = pal("#6a4a10", "#9a7418", "#c99a22", "#e8c040", "#fff0a0")
STEEL = pal("#3a3e46", "#5a606a", "#7d8590", "#a8b0ba", "#e0e6ee")
IRON = pal("#2c2e32", "#45484e", "#62666e", "#8a8f98", "#c0c4ca")
BRASS = pal("#4a3210", "#7a5520", "#a87a30", "#d0a050", "#f0d080")
COPPER = pal("#4a2414", "#7a3c20", "#a8582e", "#d07848", "#f0a070")
LEATHER = pal("#2e1a0e", "#4a2c16", "#6a4020", "#8a5630", "#a87040")
DARK_LEATHER = pal("#1a120c", "#2a1c12", "#3c2818", "#503620", "#664630")
STRAW = pal("#7a5a20", "#a07a30", "#c8a048", "#e0c068", "#f4dc90")
WHITE = pal("#a8a8a4", "#c8c8c4", "#dededa", "#ececea", "#fafaf8")
RED = pal("#4a0c0c", "#701616", "#9a2020", "#c03030", "#e05050")
NAVY = pal("#0e1428", "#18203e", "#243058", "#344474", "#4a5c90")
PURPLE = pal("#220c3a", "#361458", "#4c1e7a", "#66309c", "#8448bc")
BLACK = pal("#0c0c10", "#16161c", "#202028", "#2c2c36", "#3a3a46")
GREEN = pal("#0e2a10", "#184018", "#245a22", "#34782e", "#4c9a40")
LEAF = pal("#163a10", "#245a18", "#357a22", "#4c9a30", "#6cbc44")
KHAKI = pal("#4a4228", "#6a5e38", "#8a7c4a", "#a89a60", "#c8b880")
BLUE = pal("#0c2040", "#14305e", "#1e4480", "#2c5ca4", "#4078c8")
YELLOW = pal("#5a4808", "#8a700e", "#b89618", "#dcbc2c", "#f4dc58")
ORANGE = pal("#5a2008", "#8a340e", "#b84c18", "#dc6a28", "#f48c40")
PINK = pal("#6a2848", "#904064", "#b85c84", "#d87ca4", "#f0a0c4")
TEAL = pal("#0a3030", "#124848", "#1c6464", "#2a8282", "#40a4a4")
GREY_FUR = pal("#5a5a5c", "#7a7a7c", "#9a9a9c", "#bcbcbe", "#e0e0e2")
WHITE_FUR = pal("#a0a4a8", "#c0c4c8", "#d8dce0", "#eceff2", "#ffffff")
BROWN_FUR = pal("#2a1a10", "#3e2818", "#563a22", "#6e4c2e", "#8a6040")
MUSH_BLUE = pal("#0c1c4a", "#16307a", "#2448a8", "#3a68d0", "#64a0f4")
BEIGE = pal("#6a5a44", "#8a7858", "#a8966e", "#c8b68a", "#e4d6ae")


class Look:
    def __init__(self, paint, key):
        self.paint = paint
        self.key = key


# ---------------------------------------------------------------- one townsperson


class Folk:
    def __init__(self, name, skin_src=None):
        self.name = name
        self.src = skin_src or name
        self.skin = "folk_" + name
        self.pieces = []

    def add(self, size, origin, paint, on=HEAD, key=None):
        self.pieces.append(dict(size=tuple(size), origin=tuple(origin), paint=paint, on=on, key=key))
        return self

    # Shapes many folk share.
    def hat(self, brim, crown, paint, crown_paint=None, y=31, brim_h=1):
        bw, cw, ch = brim, crown[0], crown[1]
        self.add((bw, brim_h, bw), (-bw // 2, y, -bw // 2), paint)
        self.add((cw, ch, cw), (-cw // 2, y + brim_h, -cw // 2), crown_paint or paint)
        return self

    def beard(self, paint, w=6, h=5, y=None, d=1):
        # Hangs from the chin, under the mouth (the mouth must stay free to talk).
        return self.add((w, h, d), (-w // 2, 24 - h, 4), paint)

    def apron(self, paint, top=21, bottom=6, w=8):
        bottom = max(bottom, top - 15)  # the skin's free corners are 16 rows tall
        return self.add((w, top - bottom, 1), (-w // 2, bottom, 2), paint, on=BODY)

    def pack(self, paint, w=8, h=10, d=4, y=11):
        return self.add((w, h, d), (-w // 2, y, -2 - d), paint, on=BODY)

    def belt(self, paint, y=12, h=2):
        return self.add((10, h, 6), (-5, y, -3), paint, on=BODY)

    def skirt(self, paint, w=10, h=6, d=6, y=6):
        return self.add((w, h, d), (-w // 2, y, -d // 2), paint, on=BODY)

    def hair_back(self, paint, h=9):
        return self.add((8, h, 1), (-4, 32 - h, -5), paint)

    def cape(self, paint, h=15):
        h = min(h, 15)
        return self.add((8, h, 1), (-4, 24 - h, -3), paint, on=BODY)

    def glasses(self, paint=None):
        return self.add((8, 2, 1), (-4, 27, 4), paint or with_(solid((30, 26, 22)), lambda c, rgb: (150, 190, 210) if c.face == "front" and c.i in (1, 2, 5, 6) else rgb))

    def pauldrons(self, paint):
        self.add((6, 3, 6), (-9, 21, -3), paint, on=ARM_R)
        self.add((6, 3, 6), (3, 21, -3), paint, on=ARM_L)
        return self

    # ------------------------------------------------------------ output

    def base(self):
        """The NPC's own body boxes and relief layers as its mob file gives them (the UVs differ
        between files), with the humanoid animation hints."""
        m = MOBS[self.name]
        parts = m["parts"]
        assert len(parts) == 6, f"{self.name}: not a six-box body"
        out = []
        for k, (std, p) in enumerate(zip(BASE, parts)):
            size, origin, uv = tuple(p[0:3]), tuple(p[3:6]), tuple(p[6:8])
            assert size == std[0] and origin == std[1], f"{self.name}: box {k} differs"
            out.append((size, origin, uv, std[3], std[4]))
        return out, m["layers"]

    def parts(self):
        out = [dict(size=s, origin=o, uv=uv, anim=a, pivot=p) for s, o, uv, a, p in self.base()[0]]
        for pc in self.pieces:
            out.append(dict(size=pc["size"], origin=pc["origin"], uv=pc["uv"], anim=None, pivot=None))
        return out

    def pack_uv(self, used):
        order = sorted(range(len(self.pieces)), key=lambda k: -(2 * (self.pieces[k]["size"][0] + self.pieces[k]["size"][2]) * (self.pieces[k]["size"][2] + self.pieces[k]["size"][1])))
        for k in order:
            pc = self.pieces[k]
            w, h, d = pc["size"]
            pw, ph = 2 * (w + d), d + h
            spot = None
            for y in range(0, H - ph + 1):
                for x in range(0, W - pw + 1):
                    if not used[y:y + ph, x:x + pw].any():
                        spot = (x, y)
                        break
                if spot:
                    break
            if spot is None:
                raise AssertionError(f"skin full: {self.name} piece {k} {pc['size']}")
            x, y = spot
            used[y:y + ph, x:x + pw] = True
            pc["uv"] = spot

    def render(self):
        assert len(BASE) + len(self.pieces) <= MAX, f"{self.name}: too many pieces"
        src = Image.open(os.path.join(SRC16, self.src + ".png")).convert("RGBA")
        assert src.size == (W, H), self.src
        img = src.copy()
        # Pixels the body and its relief already use (and the space they own even where see-through).
        used = np.zeros((H, W), bool)
        base, layers = self.base()
        layered = {a for a, _ in layers}
        for k, (size, _, uv, _, _) in enumerate(base):
            w, h, d = size
            u, v = uv
            used[v:v + d + h, u:u + 2 * (w + d)] = True
            if k in layered:
                used[v + 64:v + 64 + d + h, u:u + 2 * (w + d)] = True
        self.pack_uv(used)
        px = img.load()
        seed = zlib.crc32(self.name.encode())
        for pc in self.pieces:
            for face, (x0, y0, fw, fh) in face_rects(pc["size"], pc["uv"]).items():
                for j in range(fh):
                    for i in range(fw):
                        c = Px(face, i, j, fw, fh, pc["size"], pc["origin"], seed)
                        rgb = pc["paint"](c)
                        px[x0 + i, y0 + j] = (0, 0, 0, 0) if rgb is None else (*rgb, 255)
        return img

    def ron(self):
        out = f'    (\n        name: "{self.name}", skin: "{self.skin}",\n        parts: [\n'
        for p in self.parts():
            s = f"            (size: {p['size']}, origin: {p['origin']}, uv: {p['uv']}"
            if p["anim"]:
                s += f", anim: {p['anim']}, pivot: {p['pivot']}"
            out += s + "),\n"
        out += "        ],\n"
        layers = self.base()[1]
        if layers:
            out += "        layers: [" + ", ".join(f"(part: {a}, inflate: {b})" for a, b in layers) + "],\n"
        follow = [(len(BASE) + k, pc["on"]) for k, pc in enumerate(self.pieces)]
        if follow:
            out += "        motion: (follow: [" + ", ".join(f"({a}, {b})" for a, b in follow) + "]),\n"
        out += "    ),\n"
        return out


def hair_of(name):
    """The NPC's own hair colours, read off the top of their head design."""
    src = Image.open(os.path.join(SRC16, name + ".png")).convert("RGBA")
    px = src.load()
    cols = []
    for y in range(0, 8):
        for x in range(8, 16):
            r, g, b, a = px[x, y]
            if a > 0:
                cols.append((r, g, b))
    cols.sort(key=lambda c: sum(c))
    if not cols:
        return BROWN_FUR
    n = len(cols)
    return [cols[int(k * (n - 1) / 4)] for k in range(5)]


# ---------------------------------------------------------------- everyone

FOLK = []


def folk(name, src=None):
    f = Folk(name, src)
    FOLK.append(f)
    return f


def build():
    red_cross = front_mark(".#.\n###\n.#.", (200, 30, 40))
    star = front_mark("#.#\n.#.\n#.#", (250, 230, 120))

    # The Guide: a traveller's satchel with its flap and buckle, and a book in its strap.
    f = folk("npc_guide")
    f.add((7, 6, 3), (-3, 10, -5), with_(leather(LEATHER), studs((200, 170, 90))), on=BODY)
    f.add((2, 8, 1), (-4, 13, -3), leather(DARK_LEATHER), on=BODY)
    f.add((4, 5, 2), (1, 11, -5), cloth(RED), on=BODY)

    # The Merchant: a big pack with a rolled blanket on top, a white beard.
    f = folk("npc_merchant")
    f.pack(with_(leather(LEATHER), band((60, 40, 20), (3, 7))), w=8, h=10, d=5, y=11)
    f.add((8, 3, 3), (-4, 21, -7), cloth(RED), on=BODY)
    f.beard(hair(WHITE_FUR), w=6, h=5)
    f.add((6, 3, 1), (-3, 26, 4), fur(WHITE_FUR))  # moustache

    # The Nurse: a starched cap with a red cross, a white apron with a pocket.
    f = folk("npc_nurse")
    f.add((8, 3, 8), (-4, 31, -4), with_(cloth(WHITE, 0.2), red_cross))
    f.add((6, 2, 1), (-3, 30, 4), with_(cloth(WHITE, 0.2), red_cross))
    f.apron(with_(cloth(WHITE, 0.25), band((200, 40, 50), (0,))), top=20, bottom=7)
    f.hair_back(hair(hair_of("npc_nurse")), h=8)

    # The Wizard: a tall pointed hat in steps, with stars, and a long white beard.
    f = folk("npc_wizard")
    f.add((12, 1, 12), (-6, 31, -6), with_(felt(PURPLE), band((230, 190, 60), (0,))))
    f.add((8, 4, 8), (-4, 32, -4), with_(felt(PURPLE), star))
    f.add((6, 4, 6), (-3, 36, -3), felt(PURPLE))
    f.add((4, 3, 4), (-2, 40, -1), felt(PURPLE))
    f.add((2, 3, 2), (-1, 43, 1), felt(PURPLE))
    f.beard(hair(WHITE_FUR), w=6, h=7)
    f.add((8, 1, 1), (-4, 26, 4), fur(WHITE_FUR))

    # The Smith: a heavy leather apron, a bandana, thick gloves.
    f = folk("npc_smith")
    f.apron(with_(leather(LEATHER), studs((150, 150, 150))), top=21, bottom=4)
    f.add((9, 2, 9), (-5, 28, -5), with_(cloth(RED), lambda c, rgb: dim(rgb, 0.8) if (int(c.gx) + int(c.gz)) % 3 == 0 else rgb))
    f.add((5, 4, 5), (-9, 11, -3), leather(DARK_LEATHER), on=ARM_R)
    f.add((5, 4, 5), (4, 11, -3), leather(DARK_LEATHER), on=ARM_L)

    # The Farmer: a wide straw hat with a ribbon, overall straps, a seed pouch.
    f = folk("npc_farmer")
    f.hat(14, (8, 3), straw(STRAW), with_(straw(STRAW), band((150, 40, 30), (0,))))
    f.add((8, 6, 1), (-4, 14, 2), with_(cloth(BLUE), lambda c, rgb: (210, 190, 90) if c.face == "front" and c.j == 1 and c.i in (1, 6) else rgb), on=BODY)
    f.add((4, 4, 2), (3, 9, -1), leather(LEATHER), on=BODY)

    # The Guard: a steel kettle helmet, pauldrons, a sword belt.
    f = folk("npc_guard")
    f.add((12, 1, 12), (-6, 29, -6), metal(STEEL))
    f.add((9, 4, 9), (-5, 30, -5), with_(metal(STEEL), lambda c, rgb: dim(rgb, 0.7) if c.side() and c.j == c.fh - 1 else rgb))
    f.pauldrons(metal(STEEL))
    f.belt(with_(leather(DARK_LEATHER), front_mark("##", (200, 170, 80))))
    f.add((1, 8, 2), (-6, 4, -1), metal(IRON), on=BODY)

    # The Goblin Trader: a stuffed pack with pans hanging off it.
    f = folk("goblin_trader")
    f.pack(leather(BROWN_FUR), w=8, h=9, d=4, y=12)
    f.add((3, 3, 1), (-6, 13, -6), metal(COPPER), on=BODY)
    f.add((8, 2, 8), (-4, 32, -4), felt(RED))

    # The Goblin Tinkerer: brass goggles up on the forehead, a tool belt, a wrench.
    f = folk("npc_tinkerer")
    f.add((9, 2, 9), (-5, 29, -5), leather(DARK_LEATHER))
    f.add((3, 3, 1), (-4, 30, 4), with_(metal(BRASS), lambda c, rgb: glass(c) if c.face == "front" and 0 < c.i < 2 and 0 < c.j < 2 else rgb))
    f.add((3, 3, 1), (1, 30, 4), with_(metal(BRASS), lambda c, rgb: glass(c) if c.face == "front" and 0 < c.i < 2 and 0 < c.j < 2 else rgb))
    f.belt(with_(leather(LEATHER), studs((180, 150, 80))))
    f.add((1, 6, 1), (5, 6, 1), metal(STEEL), on=BODY)

    # The Mechanic: a red bandana, a tool belt with a wrench, work gloves.
    f = folk("npc_mechanic")
    f.add((9, 3, 9), (-5, 29, -5), with_(cloth(RED), lambda c, rgb: (230, 230, 230) if c.jit > 0.9 else rgb))
    f.add((2, 2, 3), (-1, 30, -6), cloth(RED))
    f.belt(with_(leather(LEATHER), studs((160, 160, 170))))
    f.add((2, 6, 1), (4, 6, 2), metal(STEEL), on=BODY)
    f.hair_back(hair(hair_of("npc_mechanic")), h=7)

    # The Sorceress: a witch's hat with a wide brim, a long cape.
    f = folk("npc_sorceress")
    f.add((14, 1, 14), (-7, 31, -7), felt(PURPLE))
    f.add((8, 4, 8), (-4, 32, -4), with_(felt(PURPLE), band((220, 180, 60), (0,))))
    f.add((5, 4, 5), (-2, 36, -3), felt(PURPLE))
    f.add((3, 3, 3), (-1, 40, -3), felt(PURPLE))
    f.cape(with_(cloth(PURPLE), band((220, 180, 60), (0,))), h=17)
    f.hair_back(hair(hair_of("npc_sorceress")), h=10)

    # The Dryad: long hair, a leaf wreath and a skirt of leaves.
    f = folk("npc_dryad")
    f.add((10, 2, 10), (-5, 30, -5), no_faces(leafy(LEAF), "bottom"))
    f.hair_back(hair(hair_of("npc_dryad")), h=12)
    f.skirt(leafy(LEAF), w=10, h=6, d=6, y=6)
    f.add((3, 3, 1), (1, 32, 4), solid((240, 120, 160)))

    # The Demolitionist: a miner's helmet with a lamp, a dynamite belt, a bushy beard.
    f = folk("npc_demolitionist")
    f.add((10, 4, 10), (-5, 29, -5), with_(metal(YELLOW), lambda c, rgb: dim(rgb, 0.75) if c.side() and c.j == c.fh - 1 else rgb))
    f.add((2, 2, 1), (-1, 31, 5), lambda c: (255, 240, 170) if c.face == "front" else (90, 90, 90))
    f.add((10, 3, 6), (-5, 11, -3), with_(leather(LEATHER), lambda c, rgb: (200, 40, 40) if c.side() and c.i % 2 == 0 and c.j > 0 else rgb), on=BODY)
    f.beard(hair(hair_of("npc_demolitionist")), w=8, h=5)

    # The Arms Dealer: a field cap with a visor, an ammo belt, a holster.
    f = folk("npc_arms_dealer")
    f.add((9, 3, 9), (-5, 30, -5), cloth(GREEN))
    f.add((6, 1, 3), (-3, 30, 4), cloth(GREEN))
    f.belt(with_(leather(DARK_LEATHER), lambda c, rgb: (200, 160, 60) if c.face in ("front", "left", "right") and c.i % 2 == 0 else rgb))
    f.add((2, 5, 3), (4, 7, -2), leather(DARK_LEATHER), on=BODY)
    f.hair_back(hair(hair_of("npc_arms_dealer")), h=6)

    # The Angler: a bucket hat with a hook in it, a fishing vest, a creel on the back.
    f = folk("npc_angler")
    f.add((11, 1, 11), (-6, 30, -6), cloth(KHAKI))
    f.add((8, 3, 8), (-4, 31, -4), with_(cloth(KHAKI), lambda c, rgb: (200, 200, 210) if c.face == "front" and c.i == 6 and c.j == 1 else rgb))
    f.add((9, 8, 1), (-5, 13, 2), with_(cloth(KHAKI), lambda c, rgb: dim(rgb, 0.75) if c.face == "front" and c.j in (2, 5) and c.i not in (4,) else rgb), on=BODY)
    f.add((6, 6, 3), (-3, 12, -5), straw(STRAW), on=BODY)

    # The Stylist: a big hair bun with a comb in it, scissors on the apron.
    f = folk("npc_stylist")
    hp = hair(hair_of("npc_stylist"))
    f.add((5, 5, 5), (-3, 31, -5), hp)
    f.add((4, 1, 1), (-2, 34, 0), solid((240, 230, 210)))
    f.apron(with_(cloth(BLACK), lambda c, rgb: (210, 210, 220) if c.face == "front" and c.j in (3, 4) and c.i in (5, 6) else rgb), top=18, bottom=7)

    # The Dye Trader: a turban with a jewel and a sash.
    f = folk("npc_dye_trader")
    f.add((10, 4, 10), (-5, 29, -5), with_(cloth(ORANGE), lambda c, rgb: dim(rgb, 0.82) if (int(c.gy) + int(c.gx + c.gz)) % 3 == 0 else rgb))
    f.add((4, 3, 4), (-2, 33, -2), cloth(ORANGE))
    f.add((2, 2, 1), (-1, 31, 5), metal(pal("#3a0a4a", "#5a1878", "#8a30b0", "#b860e0", "#e0b0ff")))
    f.add((10, 2, 6), (-5, 12, -3), cloth(TEAL), on=BODY)

    # The Painter: a beret with a nub, a paint-spattered smock, a palette.
    f = folk("npc_painter")
    f.add((10, 2, 10), (-5, 31, -5), felt(RED))
    f.add((2, 1, 2), (-1, 33, -1), felt(RED))
    f.apron(with_(cloth(WHITE, 0.3), lambda c, rgb: [(200, 40, 40), (40, 90, 200), (230, 200, 40), (50, 160, 60)][int(c.jit * 40) % 4] if c.jit > 0.86 else rgb), top=21, bottom=5)
    f.add((1, 5, 4), (8, 10, -2), wood(LEATHER), on=ARM_L)

    # The Tailor: round glasses, a measuring tape round the neck, a pincushion on the wrist.
    f = folk("npc_tailor")
    f.glasses()
    f.add((9, 1, 5), (-5, 23, -3), with_(solid((230, 210, 80)), lambda c, rgb: (40, 40, 40) if c.i % 2 == 0 else rgb), on=BODY)
    f.add((2, 7, 1), (2, 16, 2), with_(solid((230, 210, 80)), lambda c, rgb: (40, 40, 40) if c.j % 2 == 0 else rgb), on=BODY)
    f.add((5, 2, 5), (-9, 13, -3), with_(cloth(RED), lambda c, rgb: (230, 230, 230) if c.jit > 0.75 else rgb), on=ARM_R)

    # The Witch Doctor: a feather headdress and a grass skirt.
    f = folk("npc_witch_doctor")
    f.add((10, 2, 10), (-5, 29, -5), with_(leather(LEATHER), lambda c, rgb: [(200, 40, 40), (240, 200, 60), (40, 140, 200)][c.i % 3] if c.side() else rgb))
    f.add((2, 6, 1), (-3, 31, -5), with_(solid((230, 60, 40)), lambda c, rgb: (250, 230, 80) if c.j < 2 else rgb))
    f.add((2, 7, 1), (1, 31, -5), with_(solid((40, 160, 90)), lambda c, rgb: (250, 230, 80) if c.j < 2 else rgb))
    f.skirt(leafy(STRAW), w=10, h=6, d=6, y=6)

    # The Zoologist: a pith helmet and binoculars on the chest.
    f = folk("npc_zoologist")
    f.hat(12, (8, 3), cloth(KHAKI, 0.3), with_(cloth(KHAKI, 0.3), band((90, 70, 40), (0,))), y=30)
    f.add((4, 2, 2), (-2, 18, 2), with_(metal(BLACK), lambda c, rgb: glass(c) if c.face == "front" and c.i in (0, 3) else rgb), on=BODY)
    f.hair_back(hair(hair_of("npc_zoologist")), h=6)

    # The Party Girl: a striped party hat and pigtails.
    f = folk("npc_party_girl")
    stripes = lambda c: dim([(240, 80, 160), (250, 220, 70), (80, 200, 230)][int(c.gy) % 3], FACE_LIGHT[c.face])
    f.add((6, 3, 6), (-3, 32, -3), stripes)
    f.add((4, 3, 4), (-2, 35, -2), stripes)
    f.add((2, 2, 2), (-1, 38, -1), lambda c: (255, 250, 200))
    hp = hair(hair_of("npc_party_girl"))
    f.add((2, 6, 2), (-6, 23, -1), hp)
    f.add((2, 6, 2), (4, 23, -1), hp)

    # The Pirate: a tricorn with a skull, an eye patch, a beard, a sash.
    f = folk("npc_pirate")
    f.add((12, 2, 10), (-6, 31, -5), with_(felt(BLACK), band((200, 170, 70), (0,))))
    f.add((8, 3, 8), (-4, 33, -4), with_(felt(BLACK), front_mark("###\n#.#\n.#.", (230, 230, 220))))
    f.add((3, 2, 1), (-3, 27, 4), solid((20, 18, 18)))
    f.add((10, 1, 1), (-5, 28, 4), solid((20, 18, 18)))
    f.beard(hair(hair_of("npc_pirate")), w=6, h=3)
    f.add((10, 2, 6), (-5, 12, -3), cloth(RED), on=BODY)

    # The Steampunker: a top hat with goggles, a brass boiler on the back.
    f = folk("npc_steampunker")
    f.hat(10, (8, 6), felt(BROWN_FUR), felt(BROWN_FUR))
    f.add((3, 3, 1), (-4, 33, 4), with_(metal(BRASS), lambda c, rgb: glass(c) if c.face == "front" and c.i == 1 and c.j == 1 else rgb))
    f.add((3, 3, 1), (1, 33, 4), with_(metal(BRASS), lambda c, rgb: glass(c) if c.face == "front" and c.i == 1 and c.j == 1 else rgb))
    f.add((6, 9, 4), (-3, 11, -6), with_(metal(COPPER), band((80, 50, 30), (2, 6))), on=BODY)
    f.add((2, 4, 2), (-1, 20, -5), metal(BRASS), on=BODY)
    f.hair_back(hair(hair_of("npc_steampunker")), h=6)

    # The Cyborg: a red visor, an antenna, a reactor pack, steel shoulders.
    f = folk("npc_cyborg")
    f.add((8, 2, 1), (-4, 27, 4), lambda c: (255, 60, 50) if c.face == "front" and c.jit > 0.25 else (120, 20, 20))
    f.add((1, 5, 1), (3, 32, 0), metal(STEEL))
    f.add((6, 9, 3), (-3, 11, -5), with_(metal(STEEL), front_mark("#", (80, 220, 255))), on=BODY)
    f.pauldrons(metal(IRON))

    # The Princess: a golden crown with points and rubies, long hair, a wide gown.
    f = folk("npc_princess")
    f.add((9, 3, 9), (-5, 31, -5), with_(gold, zigzag_top))
    f.hair_back(hair(hair_of("npc_princess")), h=13)
    f.add((12, 8, 8), (-6, 4, -4), with_(cloth(PINK, 0.3), band((240, 220, 120), (0,))), on=BODY)

    # The Peddler: a deep hood and a pack piled high.
    f = folk("npc_peddler")
    f.add((10, 6, 10), (-5, 27, -5), no_faces(cloth(PURPLE), "front", "bottom"))
    f.pack(leather(LEATHER), w=8, h=11, d=4, y=11)
    f.add((6, 3, 4), (-3, 22, -6), cloth(RED), on=BODY)

    # The Mycologist: a big glowing mushroom cap for a hat.
    f = folk("npc_mycologist")
    f.add((12, 3, 12), (-6, 32, -6), with_(cloth(MUSH_BLUE, 0.4), lambda c, rgb: (180, 230, 255) if c.jit > 0.88 and c.face != "bottom" else ((60, 90, 140) if c.face == "bottom" else rgb)))
    f.add((8, 2, 8), (-4, 35, -4), cloth(MUSH_BLUE, 0.4))
    f.add((4, 4, 2), (3, 9, -1), straw(STRAW), on=BODY)

    # The Winter Elder: a fur hat, a long white beard, a fur collar.
    f = folk("npc_winter_elder")
    f.add((10, 4, 10), (-5, 30, -5), fur(WHITE_FUR))
    f.add((7, 2, 7), (-4, 34, -4), cloth(RED))
    f.beard(hair(WHITE_FUR), w=8, h=8)
    f.add((10, 3, 6), (-5, 21, -3), fur(WHITE_FUR), on=BODY)

    # The Tax Collector: a tall top hat, a monocle, coat tails.
    f = folk("npc_tax_collector")
    f.hat(10, (8, 7), felt(BLACK), with_(felt(BLACK), band((120, 20, 30), (0,))))
    f.add((2, 2, 1), (1, 27, 4), with_(metal(GOLD), lambda c, rgb: glass(c) if c.face == "front" else rgb))
    f.add((8, 6, 1), (-4, 6, -3), felt(BLACK), on=BODY)

    # Village settlers.
    f = folk("settler_farmer")
    f.hat(13, (8, 3), straw(STRAW))
    f.add((4, 4, 2), (3, 9, -1), leather(LEATHER), on=BODY)
    f = folk("settler_smith")
    f.apron(leather(DARK_LEATHER), top=21, bottom=5)
    f.add((5, 3, 5), (-9, 11, -3), leather(DARK_LEATHER), on=ARM_R)
    f.add((5, 3, 5), (4, 11, -3), leather(DARK_LEATHER), on=ARM_L)
    f = folk("settler_librarian")
    f.glasses()
    f.add((1, 6, 5), (8, 10, -3), with_(leather(RED), lambda c, rgb: (240, 230, 200) if c.face == "left" and c.edge() > 0 else rgb), on=ARM_L)
    f.add((8, 2, 8), (-4, 32, -4), felt(NAVY))
    f = folk("settler_shepherd")
    f.add((10, 6, 10), (-5, 27, -5), no_faces(cloth(BEIGE), "front", "bottom"))
    f.add((1, 18, 1), (-8, 2, 3), wood(LEATHER), on=ARM_R)
    f.add((3, 1, 1), (-8, 20, 3), wood(LEATHER), on=ARM_R)


def main():
    build()
    os.makedirs(os.path.dirname(OUT_RON), exist_ok=True)
    import entities as ent  # noqa: F401  (tools/tex64)
    from style import to_u8
    text = ("// The town folk's own looks (tools/gen_npcs.py writes this file: edit that script).\n"
            "// Laid over the mob lists after loading, after fauna/models.ron: only parts, layers,\n"
            "// motion and skin change.\n"
            "(models: [\n")
    for f in FOLK:
        img = f.render()
        img.save(os.path.join(SRC16, f.skin + ".png"))
        a = np.asarray(img, dtype=np.float64)
        big = ent.grow(a, f.skin)
        Image.fromarray(to_u8(big), "RGBA").save(os.path.join(ENTITY, f.skin + ".png"), optimize=True)
        text += f.ron()
    text += "])\n"
    with open(OUT_RON, "w", encoding="utf-8") as fh:
        fh.write(text)
    print(f"wrote {len(FOLK)} folk")


if __name__ == "__main__":
    main()
