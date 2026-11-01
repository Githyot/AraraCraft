#!/usr/bin/env python3
"""The gunner's pass (HP 2026-11-01, plan/atirador-analise.md): the guns, bullets and gunner
accessories that were missing.

Writes packs/vanilla/atirador/{items,recipes}.ron, adds the English names to
packs/vanilla/lang/en.ron, draws the bullet and accessory icons (32x32 pixel art grown to 64x64,
like gen_tr3.py), and turns the gun pictures the game renders from its own 3D guns
(tools/tex64/guns3d/, `cargo test -p game write_gun_icons -- --ignored`) into item icons with
pixel.py, for the new guns and the old ones alike, so every icon matches the gun in the hand.

    cargo test -p game write_gun_icons -- --ignored && python3 tools/gen_atirador.py
"""

import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "tex64"))
from gen_vulcao import Art, lit, ramp_fill  # noqa: E402
from pixel import pixelate  # noqa: E402

ROOT = os.path.dirname(HERE)
PACK = os.path.join(ROOT, "packs", "vanilla")
OUT = os.path.join(PACK, "atirador")
ITEM_TEX = os.path.join(PACK, "textures", "item")
GUNS3D = os.path.join(HERE, "tex64", "guns3d")
EN_FILE = os.path.join(PACK, "lang", "en.ron")

ITEMS = []
SHAPED = []
SHAPELESS = []
EN = {}
ICONS = {}


def item(name, display, en, stack=64):
    s = f'    (name: "{name}", display_name: Some("{display}")'
    if stack != 64:
        s += f", max_stack: Some({stack})"
    ITEMS.append(s + "),")
    EN[display] = en


def shaped(pattern, key, result, count=1):
    keys = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    pat = ", ".join(f'"{p}"' for p in pattern)
    c = f", count: {count}" if count != 1 else ""
    SHAPED.append(f"        (pattern: [{pat}], key: {{{keys}}}, result: \"{result}\"{c}),")


def shapeless(ingredients, result, count=1):
    ing = ", ".join(f'"{i}"' for i in ingredients)
    c = f", count: {count}" if count != 1 else ""
    SHAPELESS.append(f"        (ingredients: [{ing}], result: \"{result}\"{c}),")


def fill(pal, f):
    return ramp_fill(pal, lambda xs, ys: np.broadcast_to(np.asarray(f(xs, ys), float), xs.shape))


BRASS = ["#4a3008", "#8a6014", "#c2922e", "#e8c060", "#fff0b0"]
LEATHER = ["#2a160a", "#4a2a14", "#6e4222", "#92603a", "#b88454"]
STEEL = ["#1e2228", "#3a414c", "#5e6774", "#8f99a6", "#c8d0da"]


# ---- The guns that were missing ----
GUNS = [
    ("revolver", "Revólver", "Revolver"),
    ("handgun", "Pistola", "Handgun"),
    ("phoenix_blaster", "Pistola Fênix", "Phoenix Blaster"),
    ("quad_barrel_shotgun", "Espingarda de Quatro Canos", "Quad-Barrel Shotgun"),
    ("shotgun", "Escopeta", "Shotgun"),
    ("clockwork_rifle", "Fuzil de Corda", "Clockwork Rifle"),
    ("uzi", "Submetralhadora", "Uzi"),
    ("onyx_blaster", "Explosor de Ônix", "Onyx Blaster"),
    ("tactical_shotgun", "Escopeta Tática", "Tactical Shotgun"),
    ("venus_magnum", "Magnum de Vênus", "Venus Magnum"),
    ("chain_gun", "Metralhadora Giratória", "Chain Gun"),
    ("lumite_minigun", "Minigun de Lumita", "Lumite Minigun"),
]
for n, pt, en in GUNS:
    item(n, pt, en, 1)

shaped(["SSS", " FW"], {"S": "silver_ingot", "F": "flint", "W": "oak_planks"}, "revolver")
shaped(["TTT", " BG"], {"T": "tungsten_ingot", "B": "bone", "G": "gunpowder"}, "handgun")
shapeless(["handgun", "magmarite_ingot", "magmarite_ingot", "magmarite_ingot", "magmarite_ingot", "blaze_powder"], "phoenix_blaster")
shapeless(["boomstick", "boomstick", "bone", "bone", "gunpowder", "gunpowder"], "quad_barrel_shotgun")
shaped(["CCC", "CGW"], {"C": "cobalt_ingot", "G": "gunpowder", "W": "oak_planks"}, "shotgun")
shaped(["III", "CGR"], {"I": "mythril_ingot", "C": "clock", "G": "gunpowder", "R": "redstone"}, "clockwork_rifle")
shapeless(["titanium_ingot", "titanium_ingot", "titanium_ingot", "titanium_ingot", "gunpowder", "gunpowder", "iron_ingot"], "uzi")
shapeless(["shotgun", "obsidian", "obsidian", "soul_of_night", "soul_of_night"], "onyx_blaster")
shapeless(["shotgun", "soul_of_sight", "soul_of_sight", "adamantite_ingot", "adamantite_ingot"], "tactical_shotgun")
shapeless(["verdite_ingot", "verdite_ingot", "verdite_ingot", "verdite_ingot", "verdite_ingot", "verdite_ingot", "gunpowder"], "venus_magnum")
shapeless(["minishark", "soul_of_might", "soul_of_fright", "adamantite_ingot", "adamantite_ingot", "adamantite_ingot", "gunpowder"], "chain_gun")
shapeless(["megashark", "lumite_ingot", "lumite_ingot", "lumite_ingot", "lumite_ingot", "vortex_fragment", "vortex_fragment", "vortex_fragment", "vortex_fragment"], "lumite_minigun")


# ---- Bullets: a little row of cartridges, each kind with its own tip ----
def bullets_icon(tip, glow=None, case=BRASS):
    def draw(a):
        for k, (x, y) in enumerate(((5, 19), (11, 13), (17, 7))):
            # The case, lying on the diagonal, and its rim.
            a.poly([(x, y + 6), (x + 4, y + 10), (x + 10, y + 4), (x + 6, y)], fill(case, lit(x + 4, y + 4, 5)), "#2a1a04")
            a.poly([(x - 1, y + 7), (x + 3, y + 11), (x + 4, y + 10), (x, y + 6)], fill(case, lambda xs, ys: 0.35), "#2a1a04")
            # The bullet's tip.
            a.poly([(x + 6, y), (x + 10, y + 4), (x + 13, y + 1), (x + 9, y - 3)], fill(tip, lit(x + 9, y, 3)), "#1a1010")
            if glow:
                a.px(x + 10, y - 1, glow)
    return draw


AMMO = [
    ("silver_bullet", "Bala de Prata", "Silver Bullet", ["#4a4e58", "#7a808c", "#a8b0bc", "#d4dae4", "#ffffff"], None, ["silver_ingot", "gunpowder"], 60),
    ("crystal_bullet", "Bala de Cristal", "Crystal Bullet", ["#3a1a5a", "#6a3a9a", "#9a6ad0", "#c8a0f0", "#f0e0ff"], "#ffffff", ["amethyst", "gunpowder"], 60),
    ("cursed_bullet", "Bala Amaldiçoada", "Cursed Bullet", ["#1a3a08", "#3a7010", "#62b020", "#9ae040", "#e0ff90"], "#e0ff90", ["soul_of_night", "gunpowder"], 60),
    ("ichor_bullet", "Bala de Icor", "Ichor Bullet", ["#5a3a04", "#9a6a0c", "#d8a018", "#f8d040", "#fff4a0"], None, ["soul_of_light", "gunpowder"], 60),
    ("venom_bullet", "Bala de Veneno", "Venom Bullet", ["#2a0a3a", "#541870", "#8030a8", "#b060d8", "#e0b0ff"], None, ["fermented_spider_eye", "gunpowder"], 60),
    ("high_velocity_bullet", "Bala de Alta Velocidade", "High Velocity Bullet", ["#2a2e36", "#4a515c", "#737c88", "#a4adb8", "#d8dee6"], "#ffe0a0", ["cobalt_ingot", "gunpowder"], 60),
    ("explosive_bullet", "Bala Explosiva", "Explosive Bullet", ["#5a0e04", "#9a2008", "#d84010", "#f88030", "#ffd090"], "#ffd090", ["tnt", "gunpowder"], 60),
    ("verdite_bullet", "Bala de Verdita", "Verdite Bullet", ["#0a3a14", "#146a28", "#28a040", "#60d070", "#c0ffc8"], "#c0ffc8", ["verdite_ingot", "gunpowder"], 70),
    ("golden_bullet", "Bala Dourada", "Golden Bullet", ["#5a3a08", "#9a6a14", "#d4a02a", "#f2cc52", "#fff0a0"], "#ffffff", ["gold_ingot", "gunpowder"], 40),
    ("party_bullet", "Bala de Festa", "Party Bullet", ["#5a0a3a", "#a01870", "#e040a0", "#f890c8", "#ffe0f0"], "#80e0ff", ["firework_rocket", "gunpowder"], 60),
    ("lumite_bullet", "Bala de Lumita", "Lumite Bullet", ["#0a3a3a", "#147070", "#28b0a8", "#70e8e0", "#d0fff8"], "#ffffff", ["lumite_ingot", "gunpowder"], 99),
]
for n, pt, en, tip, glow, ing, count in AMMO:
    item(n, pt, en)
    ICONS[n] = bullets_icon(tip, glow)
    shapeless(ing, n, count)


# ---- Gunner accessories ----
def sniper_scope(a):
    a.poly([(3, 13), (27, 13), (27, 19), (3, 19)], fill(STEEL, lambda xs, ys: 0.9 - (ys - 13) / 8), "#0a0c10")
    a.poly([(2, 12), (6, 12), (6, 20), (2, 20)], fill(STEEL, lambda xs, ys: 0.55), "#0a0c10")
    a.disc(26.5, 16, 5.0, fill(STEEL, lit(25.5, 15, 5)), "#0a0c10")
    a.disc(26.5, 16, 3.2, fill(["#0a2a4a", "#1a5a8a", "#3a90c8", "#8ad0ff", "#ffffff"], lit(25.5, 15, 3)), "#0a0c10")
    a.px(26, 16, "#ff4030")
    a.px(27, 16, "#ff4030")
    a.poly([(10, 19), (13, 19), (13, 23), (10, 23)], fill(STEEL, lambda xs, ys: 0.4), "#0a0c10")
    a.poly([(17, 19), (20, 19), (20, 23), (17, 23)], fill(STEEL, lambda xs, ys: 0.4), "#0a0c10")
    a.poly([(12, 10), (16, 10), (16, 13), (12, 13)], fill(BRASS, lambda xs, ys: 0.7), "#2a1a04")


def bandolier(a):
    # A leather belt across the slot, with cartridges in its loops.
    a.poly([(2, 23), (8, 29), (30, 7), (24, 1)], fill(LEATHER, lambda xs, ys: 0.75 - (xs + ys - 30) / 40), "#1a0c04")
    for k in range(5):
        x, y = 7 + k * 4.2, 21 - k * 4.2
        a.poly([(x - 2, y - 5), (x + 1, y - 2), (x + 3.5, y - 4.5), (x + 0.5, y - 7.5)], fill(BRASS, lit(x, y - 5, 3)), "#2a1a04")
        a.px(int(x), int(y - 7), "#c8d0da")
    a.poly([(13, 13), (19, 13), (19, 19), (13, 19)], fill(STEEL, lit(15, 15, 3)), "#0a0c10")
    a.poly([(15, 15), (17, 15), (17, 17), (15, 17)], fill(LEATHER, lambda xs, ys: 0.4), "#1a0c04")


item("sniper_scope", "Mira de Atirador de Elite", "Sniper Scope", 1)
ICONS["sniper_scope"] = sniper_scope
shapeless(["rifle_scope", "marksman_lens", "soul_of_sight", "soul_of_sight"], "sniper_scope")
item("bandolier", "Bandoleira", "Bandolier", 1)
ICONS["bandolier"] = bandolier
shaped(["L B", " L ", "B L"], {"L": "leather", "B": "musket_ball"}, "bandolier")


def write():
    os.makedirs(OUT, exist_ok=True)
    head = "// Generated by tools/gen_atirador.py (the gunner's pass, plan/atirador-analise.md).\n"
    with open(os.path.join(OUT, "items.ron"), "w") as f:
        f.write(head + "[\n" + "\n".join(ITEMS) + "\n]\n")
    with open(os.path.join(OUT, "recipes.ron"), "w") as f:
        f.write(head + "(\n    shaped: [\n" + "\n".join(SHAPED) + "\n    ],\n    shapeless: [\n" + "\n".join(SHAPELESS) + "\n    ],\n)\n")
    # Icons: drawn ones, then the guns from their 3D pictures.
    for n, fn in ICONS.items():
        a = Art()
        fn(a)
        a.image().save(os.path.join(ITEM_TEX, n + ".png"))
    if os.path.isdir(GUNS3D):
        for fname in sorted(os.listdir(GUNS3D)):
            if fname.endswith(".png"):
                img = np.asarray(Image.open(os.path.join(GUNS3D, fname)).convert("RGBA"), dtype=np.float64)
                out = pixelate(img)
                Image.fromarray(np.clip(out, 0, 255).astype(np.uint8), "RGBA").save(os.path.join(ITEM_TEX, fname))
    # English names, between our markers in lang/en.ron.
    text = open(EN_FILE, encoding="utf-8").read()
    start, end = "    // gen_atirador.py: start\n", "    // gen_atirador.py: end\n"
    if start in text:
        a = text.index(start)
        b = text.index(end) + len(end)
        text = text[:a] + text[b:]
    block = start + "".join(f'    "{k}": "{v}",\n' for k, v in EN.items()) + end
    cut = text.rstrip().rindex("}")
    text = text[:cut] + block + text[cut:]
    open(EN_FILE, "w", encoding="utf-8").write(text)


if __name__ == "__main__":
    write()
    print(f"{len(ITEMS)} items, {len(SHAPED) + len(SHAPELESS)} recipes, {len(ICONS)} drawn icons")
