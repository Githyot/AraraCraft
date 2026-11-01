#!/usr/bin/env python3
"""Lote B2 of the block review (plan/blocos-analise.md): the vanilla work blocks the game lacked.

Campfires, smoker, blast furnace, loom, cartography and fletching tables, stonecutter, lectern,
bell, cauldron (empty, water, lava), flower pot, ender chest, trapped chest, daylight detector,
lightning rod, scaffolding, the 16 coloured candles and the 15 coloured shulker boxes.

Writes:
- their pictures into packs/vanilla/textures/block/ (drawn here from our own textures);
- item icons of the 3D ones, drawn from their boxes in tools/gen_block_models.py;
- the blocks into packs/vanilla/blocks.ron (between `// B2 start` and `// B2 end`);
- their recipes into packs/vanilla/blocos/recipes_b2.ron.

Run tools/gen_block_models.py first (it owns the 3D models), then this.
"""

import os
import re
import sys

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))
sys.path.insert(0, os.path.join(ROOT, "tools", "tex64"))
TEX = os.path.join(ROOT, "packs", "vanilla", "textures", "block")
BLOCKS = os.path.join(ROOT, "packs", "vanilla", "blocks.ron")
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes_b2.ron")
N = 64

COLORS = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple",
          "blue", "brown", "green", "red", "black"]
DYE = {
    "white": "#e6e9e9", "orange": "#ea7a1c", "magenta": "#b84bb0", "light_blue": "#3caed8", "yellow": "#f6c22c",
    "lime": "#72b81c", "pink": "#ec8cac", "gray": "#4a5054", "light_gray": "#9a9a92", "cyan": "#158a92",
    "purple": "#7a2aaa", "blue": "#353a9c", "brown": "#724a2a", "green": "#556e1a", "red": "#a02822",
    "black": "#22222a",
}
COR = {"white": "Branca", "orange": "Laranja", "magenta": "Magenta", "light_blue": "Azul-clara", "yellow": "Amarela",
       "lime": "Verde-limão", "pink": "Rosa", "gray": "Cinza", "light_gray": "Cinza-clara", "cyan": "Ciano",
       "purple": "Roxa", "blue": "Azul", "brown": "Marrom", "green": "Verde", "red": "Vermelha", "black": "Preta"}


def load(name):
    return np.asarray(Image.open(os.path.join(TEX, name + ".png")).convert("RGBA").resize((N, N), Image.NEAREST),
                      dtype=np.float64)


def save(name, img):
    Image.fromarray(np.clip(img, 0, 255).astype(np.uint8), "RGBA").save(os.path.join(TEX, name + ".png"), optimize=True)


def hexc(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], dtype=np.float64)


def rng(name):
    import zlib
    return np.random.default_rng(zlib.crc32(name.encode()))


def tint(img, colour, keep=0.35):
    """Recolour by lightness: the picture's shading on a new colour (keeps alpha)."""
    out = img.copy()
    lum = (img[..., 0] * 0.3 + img[..., 1] * 0.59 + img[..., 2] * 0.11) / 255.0
    m = lum[img[..., 3] > 0].mean() if (img[..., 3] > 0).any() else 0.5
    c = hexc(colour)
    k = (lum / max(m, 1e-3))[..., None]
    out[..., :3] = np.clip(c * k * (1 - keep) + img[..., :3] * keep * (c / 255.0) * 1.4, 0, 255)
    return out


def rect(img, x0, y0, x1, y1, c, a=255):
    img[y0:y1, x0:x1, :3] = c
    img[y0:y1, x0:x1, 3] = a


def bevel_rect(img, x0, y0, x1, y1, base, r=None):
    """A raised panel: flat colour with a light top-left and dark bottom-right edge."""
    c = hexc(base) if isinstance(base, str) else base
    rect(img, x0, y0, x1, y1, c)
    if r is not None:
        n = (r.random((y1 - y0, x1 - x0)) - 0.5) * 18
        img[y0:y1, x0:x1, :3] = np.clip(img[y0:y1, x0:x1, :3] + n[..., None], 0, 255)
    img[y0, x0:x1, :3] = np.clip(c * 1.25, 0, 255)
    img[y0:y1, x0, :3] = np.clip(c * 1.18, 0, 255)
    img[y1 - 1, x0:x1, :3] = c * 0.6
    img[y0:y1, x1 - 1, :3] = c * 0.65


def band(img, src, y0, y1):
    """Copy a horizontal strip of another picture (log bands on the smoker...)."""
    img[y0:y1] = src[y0:y1]


# ------------------------------------------------------------------ cube pictures


def furnaces():
    side, front, top = load("furnace_side"), load("furnace_front"), load("furnace_top")
    log, logtop = load("oak_log"), load("oak_log_top")
    # Smoker: the furnace body held by dark log bands, a chimney hood on top.
    sm_side = side.copy()
    dark = tint(log, "#4a3220", 0.2)
    band(sm_side, dark, 0, 12)
    band(sm_side, dark, 52, 64)
    sm_front = front.copy()
    band(sm_front, dark, 0, 10)
    band(sm_front, dark, 54, 64)
    sm_top = tint(logtop, "#5a4030", 0.3)
    rect(sm_top, 20, 20, 44, 44, hexc("#2a2420"))
    rect(sm_top, 24, 24, 40, 40, hexc("#141210"))
    save("smoker_side", sm_side)
    save("smoker_front", sm_front)
    save("smoker_top", sm_top)
    save("smoker_bottom", tint(load("furnace_top"), "#6a6a6a", 0.5))
    # Blast furnace: smooth stone in an iron frame, a grated mouth.
    smooth = load("smooth_stone")
    iron = hexc("#4c5058")
    bf_side = smooth.copy()
    for x0, x1 in ((0, 6), (58, 64)):
        rect(bf_side, x0, 0, x1, 64, iron)
    rect(bf_side, 0, 0, 64, 6, iron)
    rect(bf_side, 0, 58, 64, 64, iron)
    for y in range(14, 54, 10):
        rect(bf_side, 10, y, 54, y + 3, iron * 0.85)
    bf_front = bf_side.copy()
    rect(bf_front, 14, 26, 50, 54, hexc("#1a1a1e"))
    for x in range(16, 50, 6):
        rect(bf_front, x, 26, x + 2, 54, iron * 1.2)
    bf_top = smooth.copy()
    rect(bf_top, 0, 0, 64, 6, iron)
    rect(bf_top, 0, 58, 64, 64, iron)
    rect(bf_top, 0, 0, 6, 64, iron)
    rect(bf_top, 58, 0, 64, 64, iron)
    rect(bf_top, 22, 22, 42, 42, hexc("#2a2c30"))
    save("blast_furnace_side", bf_side)
    save("blast_furnace_front", bf_front)
    save("blast_furnace_top", bf_top)


def tables():
    r = rng("tables")
    planks, dark, birch = load("oak_planks"), load("dark_oak_planks"), load("birch_planks")
    # Loom: a frame with a warp of threads and a wooden spool.
    lt = planks.copy()
    for x in range(8, 56, 4):
        rect(lt, x, 6, x + 1, 58, hexc("#e8e0d0"))
    rect(lt, 4, 4, 60, 8, hexc("#6a4a28"))
    rect(lt, 4, 56, 60, 60, hexc("#6a4a28"))
    save("loom_top", lt)
    ls = planks.copy()
    rect(ls, 6, 6, 58, 58, hexc("#5e4024"))
    bevel_rect(ls, 10, 10, 54, 54, "#8c6a40", r)
    for y in range(14, 50, 5):
        rect(ls, 14, y, 50, y + 2, hexc("#c8b8a0"))
    save("loom_side", ls)
    lf = ls.copy()
    rect(lf, 18, 26, 46, 38, hexc("#d0a050"))  # the spool
    rect(lf, 16, 24, 20, 40, hexc("#7a5030"))
    rect(lf, 44, 24, 48, 40, hexc("#7a5030"))
    save("loom_front", lf)
    # Cartography table: a map pinned on dark wood, a compass rose drawn on it.
    ct = dark.copy()
    bevel_rect(ct, 6, 6, 58, 58, "#e4d6aa", r)
    for y in range(14, 54, 8):
        rect(ct, 10, y, 54, y + 1, hexc("#b8a478"))
    rect(ct, 30, 16, 34, 48, hexc("#8a3a2a"))
    rect(ct, 16, 30, 48, 34, hexc("#8a3a2a"))
    rect(ct, 26, 26, 38, 38, hexc("#3a5a8a"))
    save("cartography_table_top", ct)
    cs = dark.copy()
    rect(cs, 0, 0, 64, 8, hexc("#e4d6aa"))
    rect(cs, 6, 14, 58, 58, hexc("#2c2014"))
    save("cartography_table_side", cs)
    # Fletching table: pale birch with flint heads and feathers laid on top.
    ft = birch.copy()
    for i, (x, y) in enumerate(((12, 12), (36, 16), (16, 38), (40, 40))):
        rect(ft, x, y, x + 12, y + 3, hexc("#8a6a40"))
        rect(ft, x + 10, y - 2, x + 15, y + 5, hexc("#3c3c44"))
        rect(ft, x - 2, y - 2, x + 2, y + 5, hexc("#f0f0f0"))
    save("fletching_table_top", ft)
    fs = birch.copy()
    rect(fs, 0, 0, 64, 6, hexc("#d8c890"))
    rect(fs, 8, 14, 56, 56, hexc("#6a5030"))
    save("fletching_table_side", fs)
    ff = fs.copy()
    rect(ff, 20, 22, 44, 26, hexc("#8a6a40"))
    rect(ff, 40, 18, 46, 30, hexc("#3c3c44"))
    rect(ff, 16, 18, 22, 30, hexc("#f0f0f0"))
    save("fletching_table_front", ff)
    # Stonecutter top: smooth stone with the blade slot.
    st = load("smooth_stone")
    rect(st, 4, 28, 60, 36, hexc("#1c1c20"))
    save("stonecutter_top", st)


def chests():
    r = rng("chests")
    obs = load("obsidian")
    # Ender chest: obsidian body, a lid seam, a green eye for a latch.
    for face in ("front", "side", "top"):
        img = obs.copy()
        if face != "top":
            rect(img, 0, 20, 64, 23, hexc("#0c0a12"))
        else:
            rect(img, 0, 0, 64, 3, hexc("#0c0a12"))
            rect(img, 0, 61, 64, 64, hexc("#0c0a12"))
        if face == "front":
            bevel_rect(img, 26, 14, 38, 30, "#1f6a4e", r)
            rect(img, 29, 19, 35, 25, hexc("#68f0b0"))
            rect(img, 31, 20, 33, 24, hexc("#0a2a1e"))
        save(f"ender_chest_{face}", img)
    # Trapped chest: the chest with a red tripwire hook under its latch.
    for face in ("front", "side", "top"):
        img = load(f"chest_{face}")
        if face == "front":
            rect(img, 28, 34, 36, 40, hexc("#a01818"))
            rect(img, 30, 40, 34, 46, hexc("#6a4a28"))
        else:
            # A thin red stripe on the lid edge, so it reads apart from a plain chest.
            rect(img, 0, 22, 64, 24, hexc("#8a1c1c")) if face == "side" else None
        save(f"trapped_chest_{face}", img)


def detector():
    r = rng("daylight")
    top = np.zeros((N, N, 4))
    rect(top, 0, 0, 64, 64, hexc("#5a4630"))
    for i in range(3):
        for j in range(3):
            bevel_rect(top, 4 + i * 20, 4 + j * 20, 20 + i * 20, 20 + j * 20, "#b6d6e8", r)
    save("daylight_detector_top", top)
    side = load("oak_planks")
    rect(side, 0, 0, 64, 6, hexc("#d0d0d6"))
    save("daylight_detector_side", side)


def candles_and_boxes():
    candle = load("candle")
    for c in COLORS:
        img = candle.copy()
        wax = img[..., 3] > 0
        # Keep the wick (dark, thin, near the top) and the flame tip; recolour the wax only.
        lum = img[..., :3].mean(-1)
        body = wax & (lum > 150)
        t = tint(img, DYE[c], 0.25)
        img[body] = t[body]
        save(f"{c}_candle", img)
    side, top = load("white_shulker_box"), load("white_shulker_box_top")
    for c in COLORS:
        if c == "white":
            continue
        save(f"{c}_shulker_box", tint(side, DYE[c], 0.3))
        save(f"{c}_shulker_box_top", tint(top, DYE[c], 0.3))


# ------------------------------------------------------------------ icons from the 3D models


def iso_icon(boxes, name):
    """An item picture of a model: its boxes seen from above and in front, each face textured
    with the part of its picture the box covers (like in the game), shaded top/left/right."""
    S = 4  # sub-samples per pixel of the model
    d = np.array([1.0, 1.0, -1.0]) / np.sqrt(3)       # toward the viewer
    rr = np.array([1.0, 0.0, 1.0]) / np.sqrt(2)       # screen right
    up = np.cross(rr, d)                               # screen up
    img = np.zeros((N, N, 4))
    depth = np.full((N, N), -1e9)
    cache = {}
    scale = 2.25
    for (x0, y0, z0, x1, y1, z1, tex, top, glow) in boxes:
        for face in ("top", "east", "north"):
            tname = top if (face == "top" and top) else tex
            if tname not in cache:
                cache[tname] = load(tname)
            t = cache[tname]
            if face == "top":
                us = np.arange(x0, x1, 1 / S) + 0.5 / S
                vs = np.arange(z0, z1, 1 / S) + 0.5 / S
                U, V = np.meshgrid(us, vs)
                P = np.stack([U, np.full_like(U, y1), V], -1)
                tu, tv, shade = U, V, 1.0
            elif face == "east":
                us = np.arange(z0, z1, 1 / S) + 0.5 / S
                vs = np.arange(y0, y1, 1 / S) + 0.5 / S
                U, V = np.meshgrid(us, vs)
                P = np.stack([np.full_like(U, x1), V, U], -1)
                tu, tv, shade = 16 - U, 16 - V, 0.62
            else:
                us = np.arange(x0, x1, 1 / S) + 0.5 / S
                vs = np.arange(y0, y1, 1 / S) + 0.5 / S
                U, V = np.meshgrid(us, vs)
                P = np.stack([U, V, np.full_like(U, z0)], -1)
                tu, tv, shade = U, 16 - V, 0.8
            if glow:
                shade = 1.0
            P = P - 8.0
            sx = (P @ rr) * scale + N / 2
            sy = -(P @ up) * scale + N / 2
            dz = P @ d
            px = (np.clip(tu, 0, 15.999) * 4).astype(int)
            py = (np.clip(tv, 0, 15.999) * 4).astype(int)
            col = t[py, px]
            ix, iy = sx.astype(int), sy.astype(int)
            ok = (ix >= 0) & (ix < N) & (iy >= 0) & (iy < N) & (col[..., 3] > 8)
            for a, b, z, c in zip(ix[ok], iy[ok], dz[ok], col[ok]):
                if z > depth[b, a]:
                    depth[b, a] = z
                    img[b, a, :3] = c[:3] * shade
                    img[b, a, 3] = 255
    # A one-pixel dark outline round the shape, like the other icons.
    a = img[..., 3] > 0
    edge = ~a & (np.roll(a, 1, 0) | np.roll(a, -1, 0) | np.roll(a, 1, 1) | np.roll(a, -1, 1))
    img[edge, :3] = 30
    img[edge, 3] = 200
    save(name, img)


# ------------------------------------------------------------------ data

blocks = []
recipes = []
shapeless = []
smelt = []


def shaped(pattern, key, result, count=1):
    ks = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    pat = ", ".join(f'"{p}"' for p in pattern)
    recipes.append(f"        (pattern: [{pat}], key: {{{ks}}}, result: \"{result}\", count: {count}),")


def tx(**k):
    return "textures: (" + ", ".join(f'{a}: Some("{b}")' for a, b in k.items()) + ")"


def data():
    # Model blocks keep their own picture as the item icon (textures: all).
    def model_block(name, pt, hardness, tool, extra=""):
        t = f", tool: Some({tool})" if tool else ""
        blocks.append(f'    (name: "{name}", display_name: Some("{pt}"), transparency: Cutout, hardness: {hardness}{t}{extra}, {tx(all=name)}),')

    model_block("campfire", "Fogueira", 2.0, "Axe", ', light: 15, drops: Some([(item: "charcoal", min: 2, max: 2)])')
    model_block("soul_campfire", "Fogueira das Almas", 2.0, "Axe", ', light: 10, drops: Some([(item: "soul_soil")])')
    shaped([" S ", "SCS", "LLL"], {"S": "stick", "C": "charcoal", "L": "#logs"}, "campfire")
    shaped([" S ", "SCS", "LLL"], {"S": "stick", "C": "soul_soil", "L": "#logs"}, "soul_campfire")
    blocks.append(f'    (name: "smoker", display_name: Some("Defumador"), hardness: 3.5, tool: Some(Pickaxe), tier: 1, {tx(top="smoker_top", side="smoker_side", front="smoker_front", bottom="smoker_bottom")}),')
    shaped([" L ", "LFL", " L "], {"L": "#logs", "F": "furnace"}, "smoker")
    blocks.append(f'    (name: "blast_furnace", display_name: Some("Alto-forno"), hardness: 3.5, tool: Some(Pickaxe), tier: 1, {tx(top="blast_furnace_top", side="blast_furnace_side", front="blast_furnace_front", bottom="blast_furnace_top")}),')
    shaped(["III", "IFI", "SSS"], {"I": "iron_ingot", "F": "furnace", "S": "smooth_stone"}, "blast_furnace")
    blocks.append(f'    (name: "loom", display_name: Some("Tear"), hardness: 2.5, tool: Some(Axe), {tx(top="loom_top", side="loom_side", front="loom_front", bottom="oak_planks")}),')
    shaped(["SS", "PP"], {"S": "string", "P": "#planks"}, "loom")
    blocks.append(f'    (name: "cartography_table", display_name: Some("Mesa de Cartografia"), hardness: 2.5, tool: Some(Axe), {tx(top="cartography_table_top", side="cartography_table_side", bottom="dark_oak_planks")}),')
    shaped(["pp", "PP", "PP"], {"p": "paper", "P": "#planks"}, "cartography_table")
    blocks.append(f'    (name: "fletching_table", display_name: Some("Mesa de Arqueiro"), hardness: 2.5, tool: Some(Axe), {tx(top="fletching_table_top", side="fletching_table_side", front="fletching_table_front", bottom="birch_planks")}),')
    shaped(["FF", "PP", "PP"], {"F": "flint", "P": "#planks"}, "fletching_table")
    model_block("stonecutter", "Cortador de Pedra", 3.5, "Pickaxe")
    shaped([" I ", "SSS"], {"I": "iron_ingot", "S": "stone"}, "stonecutter")
    model_block("lectern", "Atril", 2.5, "Axe")
    shaped(["SSS", " B ", " S "], {"S": "#wooden_slabs", "B": "bookshelf"}, "lectern")
    model_block("bell", "Sino", 5.0, "Pickaxe")
    model_block("cauldron", "Caldeirão", 2.0, "Pickaxe")
    model_block("water_cauldron", "Caldeirão com Água", 2.0, "Pickaxe", ', in_inventory: false, drops: Some([(item: "cauldron")])')
    model_block("lava_cauldron", "Caldeirão com Lava", 2.0, "Pickaxe", ', in_inventory: false, light: 15, drops: Some([(item: "cauldron")])')
    shaped(["I I", "I I", "III"], {"I": "iron_ingot"}, "cauldron")
    model_block("flower_pot", "Vaso de Flor", 0.0, None)
    shaped(["B B", " B "], {"B": "brick"}, "flower_pot")
    blocks.append(f'    (name: "ender_chest", display_name: Some("Baú do Ender"), hardness: 22.5, tool: Some(Pickaxe), tier: 1, light: 7, drops: Some([(item: "obsidian", min: 8, max: 8)]), {tx(top="ender_chest_top", side="ender_chest_side", front="ender_chest_front", bottom="ender_chest_top")}),')
    shaped(["OOO", "OEO", "OOO"], {"O": "obsidian", "E": "ender_eye"}, "ender_chest")
    blocks.append(f'    (name: "trapped_chest", display_name: Some("Baú com Armadilha"), hardness: 2.5, tool: Some(Axe), {tx(top="trapped_chest_top", side="trapped_chest_side", front="trapped_chest_front", bottom="trapped_chest_top")}),')
    # No tripwire hook in the game yet: the hook's own parts (string and a stick) stand in for it.
    shapeless.append('        (ingredients: ["chest", "string", "stick"], result: "trapped_chest"),')
    model_block("daylight_detector", "Detector de Luz do Dia", 0.2, "Axe")
    shaped(["GGG", "QQQ", "SSS"], {"G": "glass", "Q": "quartz", "S": "#wooden_slabs"}, "daylight_detector")
    model_block("lightning_rod", "Para-raios", 3.0, "Pickaxe")
    shaped(["C", "C", "C"], {"C": "copper_ingot"}, "lightning_rod")
    model_block("scaffolding", "Andaime", 0.0, None)
    shaped(["BSB", "B B", "B B"], {"B": "bamboo", "S": "string"}, "scaffolding", 6)
    for c in COLORS:
        blocks.append(f'    (name: "{c}_candle", display_name: Some("Vela {COR[c]}"), shape: Torch, transparency: Cutout, solid: false, light: 6, hardness: 0.1, {tx(all=c + "_candle")}),')
        shapeless.append(f'        (ingredients: ["candle", "{c}_dye"], result: "{c}_candle"),')
        if c != "white":
            blocks.append(f'    (name: "{c}_shulker_box", display_name: Some("Caixa de Shulker {COR[c]}"), hardness: 2.0, tool: Some(Pickaxe), {tx(top=c + "_shulker_box_top", side=c + "_shulker_box", bottom=c + "_shulker_box_top")}),')
            shapeless.append(f'        (ingredients: ["shulker_box", "{c}_dye"], result: "{c}_shulker_box"),')


def base_tags():
    """The tag lists of the base recipes (#logs, #planks...): each recipe file reads its own."""
    text = open(os.path.join(ROOT, "packs", "vanilla", "recipes.ron"), encoding="utf-8").read()
    m = re.search(r"^    tags: \{.*?^    \},\n", text, flags=re.S | re.M)
    return m.group(0) if m else ""


def put_blocks(lines):
    text = open(BLOCKS, encoding="utf-8").read()
    text = re.sub(r"\n *// B2 start.*?// B2 end\n", "\n", text, flags=re.S)
    have = set(re.findall(r'\(name: "([^"]+)"', text))
    keep = [b for b in lines if re.search(r'name: "([^"]+)"', b).group(1) not in have]
    skipped = len(lines) - len(keep)
    block = "    // B2 start (tools/gen_blocos_b2.py)\n" + "\n".join(keep) + "\n    // B2 end\n"
    at = re.search(r"^\]\)", text, flags=re.M).start()
    open(BLOCKS, "w", encoding="utf-8").write(text[:at] + block + text[at:])
    return len(keep), skipped


def main():
    import gen_block_models as gm
    furnaces()
    tables()
    chests()
    detector()
    candles_and_boxes()
    for name in ("campfire", "soul_campfire", "stonecutter", "lectern", "bell", "cauldron", "water_cauldron", "lava_cauldron",
                 "flower_pot", "daylight_detector", "lightning_rod", "scaffolding"):
        iso_icon(gm.MODELS[name], name)
    data()
    n, skipped = put_blocks(blocks)
    os.makedirs(os.path.dirname(RECIPES), exist_ok=True)
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B2 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b2.py:\n"
                "// edit it and run it again.\n(\n" + base_tags() + "    shaped: [\n" + "\n".join(recipes) + "\n    ],\n    shapeless: [\n"
                + "\n".join(shapeless) + "\n    ],\n)\n")
    print(f"{n} blocks ({skipped} already in the game), {len(recipes) + len(shapeless)} recipes")


if __name__ == "__main__":
    main()
