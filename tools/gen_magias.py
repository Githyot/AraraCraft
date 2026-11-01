#!/usr/bin/env python3
"""The mage's new spells (plan/magias-analise.md, lote 2): items, recipes and icons.

Writes packs/vanilla/magias/{items,recipes}.ron and 32x32 icons grown to 64x64 (hard pixels)
in packs/vanilla/textures/item. The spells themselves live in crates/game/src/magic.rs and
their 3D shapes in spell_shapes.rs. Our own art: Terraria gave the ideas, nothing is copied.

Usage: python3 tools/gen_magias.py [sheet.png]
"""

import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "tex64"))
from gen_vulcao import Art, lit, ramp_fill  # noqa: E402

ROOT = os.path.dirname(HERE)
PACK = os.path.join(ROOT, "packs", "vanilla")
OUT = os.path.join(PACK, "magias")
ITEM_TEX = os.path.join(PACK, "textures", "item")

ITEMS, SHAPED, SHAPELESS, ICONS, EN = [], [], [], {}, {}


def item(name, pt, en):
    ITEMS.append(f'    (name: "{name}", display_name: Some("{pt}"), max_stack: Some(1)),')
    EN[pt] = en


def shapeless(ingredients, result):
    SHAPELESS.append("        (ingredients: [" + ", ".join(f'"{i}"' for i in ingredients) + f'], result: "{result}"),')


def shaped(pattern, key, result):
    keys = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    SHAPED.append("        (pattern: [" + ", ".join(f'"{r}"' for r in pattern) + "], key: {" + keys + f'}}, result: "{result}"),')


def fill(pal, f):
    return ramp_fill(pal, lambda xs, ys: np.broadcast_to(np.asarray(f(xs, ys), float), xs.shape))


def lin(a, x0, y0, x1, y1, w, f, edge):
    dx, dy = x1 - x0, y1 - y0
    n = math.hypot(dx, dy) or 1.0
    px, py = -dy / n * w / 2, dx / n * w / 2
    a.poly([(x0 + px, y0 + py), (x1 + px, y1 + py), (x1 - px, y1 - py), (x0 - px, y0 - py)], f, edge)


GOLD = ["#5a3a08", "#9a6a14", "#d4a02a", "#f2cc52", "#fff0a0"]
WOOD = ["#2a1a0e", "#4a2e18", "#6e4626", "#946236", "#b88050"]
FIRE = ["#5a1004", "#a82a08", "#e86a14", "#ffb030", "#fff0a0"]
WATER = ["#08204a", "#123e86", "#2a6ad0", "#5aa0f4", "#c0e4ff"]
VILE = ["#1e0a2a", "#3a1450", "#5e2a78", "#8a4aa0", "#c48ad8"]
VINE = ["#0e2a10", "#1a4a1c", "#2e6e2a", "#4e9a40", "#8ad070"]
PINK = ["#4a0a3a", "#8a1a6a", "#d04aa8", "#f08ad0", "#ffd8f4"]
CYAN = ["#06343e", "#0e5a6a", "#1e9aaa", "#5ad8e0", "#d0fbff"]
CURSED = ["#0a2a04", "#1a5a08", "#3aa014", "#80e030", "#e0ffa0"]
SHADOW = ["#120620", "#26103e", "#46207a", "#7a44c0", "#c8a0ff"]
MOON = ["#1a2a4a", "#3a5a8a", "#7aa0d0", "#c0dcff", "#ffffff"]
STEEL = ["#1a1e26", "#343a46", "#5a6474", "#8e98a8", "#d6dce6"]
NIGHT = ["#0a0a1a", "#16163a", "#26265e", "#3a3a8a", "#6a6ac0"]


def book(a, cover, trim, emblem):
    """A thick spell book seen from the front-left: cover, page block, a clasp and an emblem."""
    a.poly([(5, 7), (23, 4), (26, 26), (8, 29)], fill(cover, lambda xs, ys: 0.85 - (xs - 5) / 30 - (ys - 4) / 80), cover[0])
    a.poly([(23, 4), (27, 6), (29, 27), (26, 26)], fill(["#9a8a68", "#cbbd9c", "#efe4c8"], lambda xs, ys: 0.4 + (ys % 2) * 0.4), "#4a4030")
    a.poly([(5, 7), (8, 6), (11, 28), (8, 29)], fill(trim, lambda xs, ys: 0.6), trim[0])
    emblem(a)


def staff(a, shaft, x0=5, y0=29, x1=21, y1=11, w=2.4):
    lin(a, x0, y0, x1, y1, w, fill(shaft, lambda xs, ys: 0.85 - (xs - x0) / 26), shaft[0])
    # A metal band below the head.
    lin(a, x1 - 2.2, y1 + 2.4, x1 + 0.6, y1 - 0.4, w + 1.4, fill(GOLD, lambda xs, ys: 0.7), GOLD[0])


# ---------------------------------------------------------------------------------------------
# The spells, stage by stage (balance.rs GEAR has the numbers).

# Etapa 1 (Início)
item("aqua_scepter", "Cetro d'Água", "Aqua Scepter")
shapeless(["gold_ingot", "gold_ingot", "prismarine_shard", "prismarine_shard", "water_bucket"], "aqua_scepter")


def aqua(a):
    staff(a, GOLD, 6, 29, 20, 12, 2.2)
    # A golden cup at the top with water leaping out of it in an arc.
    a.poly([(16, 11), (25, 9), (24, 13), (19, 15)], fill(GOLD, lit(20, 11, 4)), GOLD[0])
    a.poly([(18, 9), (21, 3), (26, 2), (29, 5), (27, 9), (24, 6), (22, 9)], fill(WATER, lambda xs, ys: 0.95 - (ys - 2) / 10), WATER[0])
    a.disc(28, 11, 1.4, fill(WATER, lambda xs, ys: 0.8), WATER[1])
    a.disc(29, 15, 1.0, fill(WATER, lambda xs, ys: 0.7), WATER[1])
    a.px(22, 4, "#ffffff")


ICONS["aqua_scepter"] = aqua

item("creeping_thorn", "Espinho Rasteiro", "Creeping Thorn")
shapeless(["vile_powder", "vile_powder", "vile_powder", "stick", "emerald"], "creeping_thorn")


def creeping(a):
    # A gnarled wand of dark wood with thorns growing up out of its tip, purple and green.
    lin(a, 5, 29, 18, 14, 2.6, fill(VILE, lambda xs, ys: 0.55 - (xs - 5) / 40), VILE[0])
    for (x, y, h) in ((9, 24, 3), (13, 19, 3), (16, 16, 2)):
        a.poly([(x, y), (x + 3, y - 1), (x + 2, y - h - 1)], fill(VINE, lambda xs, ys: 0.7), VINE[0])
    for (bx, h, w) in ((17, 11, 3.2), (21, 15, 3.6), (25, 9, 2.8)):
        a.poly([(bx - w / 2, 15), (bx + w / 2, 15), (bx + 0.6, 15 - h)], fill(VILE, lambda xs, ys, b=bx: np.where(xs < b, 0.85, 0.5)), VILE[0])
    a.px(22, 1, VILE[4])


ICONS["creeping_thorn"] = creeping

# Etapa 2 (Pré-hardmode)
item("flower_of_fire", "Flor de Fogo", "Flower of Fire")
shapeless(["magmarite_ingot", "magmarite_ingot", "blaze_rod", "poppy"], "flower_of_fire")


def flower(a):
    # A long green stem with leaves, and a flower of flame at its head.
    lin(a, 6, 30, 19, 13, 2.0, fill(VINE, lambda xs, ys: 0.7 - (xs - 6) / 30), VINE[0])
    a.poly([(9, 24), (5, 20), (11, 21)], fill(VINE, lambda xs, ys: 0.8), VINE[0])
    a.poly([(14, 19), (17, 23), (13, 22)], fill(VINE, lambda xs, ys: 0.55), VINE[0])
    for k in range(6):
        t = k / 6 * math.tau + 0.3
        cx, cy = 22 + math.cos(t) * 4.6, 9 + math.sin(t) * 4.6
        a.disc(cx, cy, 3.0, fill(FIRE, lit(cx - 1, cy - 1, 3)), FIRE[0])
    a.disc(22, 9, 2.6, fill(FIRE, lambda xs, ys: 0.95), FIRE[2])
    a.px(21, 8, "#ffffff")


ICONS["flower_of_fire"] = flower

# Etapa 3 (Hardmode)
item("crystal_storm", "Tempestade de Cristal", "Crystal Storm")
shapeless(["book", "gelatin_crystal", "gelatin_crystal", "soul_of_light", "soul_of_light"], "crystal_storm")


def crystal_book(a):
    def emblem(a):
        for (x, y, h, pal) in ((13, 21, 9, PINK), (17, 20, 12, CYAN), (21, 21, 8, PINK)):
            a.poly([(x - 2, y), (x, y - h), (x + 2, y), (x, y + 2)], fill(pal, lambda xs, ys, cx=x: np.where(xs < cx, 0.9, 0.55)), pal[0])
        a.px(16, 10, "#ffffff")
    book(a, ["#2a0a3a", "#4a1a6a", "#6a2a92", "#8a44b4", "#b070d8"], GOLD, emblem)


ICONS["crystal_storm"] = crystal_book

item("cursed_flames", "Chamas Malditas", "Cursed Flames")
shapeless(["book", "soul_of_night", "soul_of_night", "vile_powder", "blaze_powder"], "cursed_flames")


def cursed_book(a):
    def emblem(a):
        # A green flame with a dark skull-like hollow in it.
        a.poly([(11, 23), (12, 15), (15, 18), (16, 10), (19, 16), (21, 13), (22, 23), (16, 26)], fill(CURSED, lambda xs, ys: 1.0 - (ys - 10) / 18), CURSED[0])
        a.disc(14.5, 20, 1.2, fill(["#0a1a04"], lambda xs, ys: 0.0), "#0a1a04")
        a.disc(18.5, 20, 1.2, fill(["#0a1a04"], lambda xs, ys: 0.0), "#0a1a04")
    book(a, ["#141a10", "#24301a", "#384a26", "#4a6232", "#6a8a46"], ["#2a2a2a", "#4a4a4a", "#7a7a7a"], emblem)


ICONS["cursed_flames"] = cursed_book

item("magnet_sphere", "Esfera Magnética", "Magnet Sphere")
shapeless(["book", "soul_of_light", "soul_of_light", "copper_ingot", "redstone"], "magnet_sphere")


def magnet_book(a):
    def emblem(a):
        a.disc(16.5, 17, 6.0, fill(CYAN, lit(15, 15, 6)), CYAN[0])
        a.disc(16.5, 17, 2.4, fill(["#ffffff", "#e0ffff"], lambda xs, ys: 0.5), CYAN[2])
        # Little zaps leaving the orb.
        for (x0, y0, x1, y1) in ((22, 12, 25, 9), (11, 22, 8, 25), (22, 22, 24, 25)):
            lin(a, x0, y0, x1, y1, 1.0, fill(["#fff6a0"], lambda xs, ys: 0.0), "#c8a020")
    book(a, ["#0a1e2a", "#143648", "#1e4e66", "#2a6a86", "#4a8aa8"], STEEL, emblem)


ICONS["magnet_sphere"] = magnet_book

item("shadow_beam", "Cajado do Raio Sombrio", "Shadow Beam Staff")
shapeless(["obsidian", "obsidian", "soul_of_night", "shadow_scale", "echo_shard"], "shadow_beam")


def shadow(a):
    staff(a, NIGHT, 5, 29, 20, 13, 2.4)
    # A dark crystal held in two claws, a beam zigzagging out of it.
    a.poly([(20, 4), (25, 8), (21, 15), (16, 11)], fill(SHADOW, lambda xs, ys: np.where(xs < 20.5, 0.9, 0.5)), SHADOW[0])
    lin(a, 16, 13, 18, 9, 1.2, fill(NIGHT, lambda xs, ys: 0.6), NIGHT[0])
    lin(a, 24, 11, 22, 15, 1.2, fill(NIGHT, lambda xs, ys: 0.6), NIGHT[0])
    for (x0, y0, x1, y1) in ((24, 5, 28, 2), (28, 2, 30, 6)):
        lin(a, x0, y0, x1, y1, 1.0, fill(SHADOW, lambda xs, ys: 0.95), SHADOW[2])
    a.px(19, 7, "#ffffff")


ICONS["shadow_beam"] = shadow

# Etapa 4 (Pós-mecânicos)
item("razorblade_typhoon", "Tufão de Lâminas", "Blade Typhoon")
shapeless(["book", "tide_scale", "tide_scale", "soul_of_might", "soul_of_sight", "soul_of_fright"], "razorblade_typhoon")


def typhoon_book(a):
    def emblem(a):
        # A spinning water blade: four curved arms round a pale hub.
        for k in range(4):
            t = k / 4 * math.tau
            pts = []
            for j in range(5):
                s = t + j * 0.35
                r = 1.5 + j * 1.3
                pts.append((16.5 + math.cos(s) * r, 17 + math.sin(s) * r))
            for j in range(4, -1, -1):
                s = t + j * 0.35 + 0.5
                r = 1.5 + j * 0.9
                pts.append((16.5 + math.cos(s) * r, 17 + math.sin(s) * r))
            a.poly(pts, fill(WATER, lambda xs, ys: 0.85), WATER[0])
        a.disc(16.5, 17, 1.8, fill(WATER, lambda xs, ys: 0.99), WATER[1])
    book(a, ["#06202a", "#0e3a4a", "#18566a", "#24748a", "#3c98ae"], ["#5a5a62", "#8a8a96", "#c4c4d0"], emblem)


ICONS["razorblade_typhoon"] = typhoon_book

# Etapa 5 (Fim de jogo)
item("lunar_flare", "Clarão Lunar", "Lunar Flare")
shapeless(["book", "nebula_fragment", "nebula_fragment", "nebula_fragment", "lumite_ingot"], "lunar_flare")


def lunar_book(a):
    def emblem(a):
        a.disc(16.5, 17, 6.0, fill(MOON, lit(15, 15, 6)), MOON[0])
        a.disc(19.5, 15, 5.2, fill(["#141a3a"], lambda xs, ys: 0.0), "#141a3a")
        for (x, y) in ((21, 22), (23, 11), (12, 10)):
            a.px(x, y, "#ffffff")
            a.px(x + 1, y, MOON[3])
            a.px(x, y + 1, MOON[3])
    book(a, ["#0a0e26", "#141a3a", "#1e2652", "#2a346e", "#3e4a92"], ["#3a4a6a", "#7a9ac8", "#d0e4ff"], emblem)


ICONS["lunar_flare"] = lunar_book

item("prism", "Prisma", "Prism")
shapeless(["prism_heart", "nebula_fragment", "nebula_fragment", "lumite_ingot", "diamond"], "prism")


def prism(a):
    # A triangular glass prism, its faces tinted, a rainbow fanning out of its side.
    for k, col in enumerate(("#ff4a4a", "#ffa030", "#fff050", "#5ae05a", "#4aa0ff", "#a060ff")):
        lin(a, 19, 18, 31, 10 + k * 2.4, 1.4, fill([col], lambda xs, ys: 0.0), col)
    a.poly([(4, 26), (13, 4), (22, 26)], fill(["#5a6a8a", "#8aa0c4", "#c4d8f4", "#eaf4ff", "#ffffff"], lambda xs, ys: np.where(xs < 13, 0.9, 0.45) - (ys - 4) / 90), "#3a4a6a")
    a.poly([(13, 4), (22, 26), (18, 26), (13, 9)], fill(["#9a8ad8", "#c0b0ff"], lambda xs, ys: 0.4), "#3a4a6a")
    lin(a, 2, 15, 12, 17, 1.4, fill(["#ffffff"], lambda xs, ys: 0.0), "#c0c0c0")
    a.px(11, 9, "#ffffff")


ICONS["prism"] = prism


def main():
    os.makedirs(OUT, exist_ok=True)
    head = "// The mage's new spells (plan/magias-analise.md, lote 2). Made by tools/gen_magias.py:\n// edit it and run it again.\n"
    with open(os.path.join(OUT, "items.ron"), "w", encoding="utf-8") as f:
        f.write(head + "(items: [\n" + "\n".join(ITEMS) + "\n])\n")
    with open(os.path.join(OUT, "recipes.ron"), "w", encoding="utf-8") as f:
        f.write(head + "(\n    shaped: [\n" + "\n".join(SHAPED) + "\n    ],\n    shapeless: [\n" + "\n".join(SHAPELESS) + "\n    ],\n)\n")
    for n, fn in ICONS.items():
        a = Art()
        fn(a)
        a.image().save(os.path.join(ITEM_TEX, n + ".png"))
    merge_english()
    print(f"wrote {len(ITEMS)} items, {len(ICONS)} icons")
    if len(sys.argv) > 1:
        sheet(sys.argv[1])


def merge_english():
    """Add the English names to packs/vanilla/lang/en.ron between this script's markers."""
    p = os.path.join(PACK, "lang", "en.ron")
    text = open(p, encoding="utf-8").read()
    a, b = "    // gen_magias.py: begin\n", "    // gen_magias.py: end\n"
    if a in text:
        i, j = text.index(a), text.index(b) + len(b)
        text = text[:i] + text[j:]
    else:
        i = text.rindex("}")
    have = set(l.split('":', 1)[0].strip().strip('"') for l in text.splitlines() if '":' in l)
    body = a + "".join(f'    "{k}": "{v}",\n' for k, v in EN.items() if k not in have) + b
    text = text[:i] + body + text[i:]
    open(p, "w", encoding="utf-8").write(text)


def sheet(path):
    from PIL import Image
    names = list(ICONS)
    img = Image.new("RGBA", (len(names) * 68, 68), (40, 40, 44, 255))
    for k, n in enumerate(names):
        img.alpha_composite(Image.open(os.path.join(ITEM_TEX, n + ".png")).convert("RGBA"), (k * 68 + 2, 2))
    img.resize((img.width * 2, img.height * 2), Image.NEAREST).save(path)


if __name__ == "__main__":
    main()
