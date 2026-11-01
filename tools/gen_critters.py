#!/usr/bin/env python3
"""Terraria's small critters brought into the 3D world (plan/animais-analise.md, lote 3).

Each one is thought through for 3D rather than copied: worms come up through the grass when it
rains, mice live on cave floors, water striders skate on still water, glowing snails light up
cave walls, lava flies drift over the Underworld's lava lakes. All can be caught with the bug net,
let go again, used as fishing bait, and have a rare golden twin.

Writes packs/vanilla/critters/mobs.ron and items.ron, the skins in textures/entity and the icons
in textures/item. The spawning lives in crates/game/src/tr3_traps.rs (critters()).

    python3 tools/gen_critters.py
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "tex64"))
from dim_mobs_spec import Model, look, quadruped, worm  # noqa: E402
from gen_dim_mobs import render as render_mob  # noqa: E402
from gen_vulcao import Art, lit, ramp_fill  # noqa: E402
import numpy as np  # noqa: E402

ROOT = os.path.dirname(HERE)
PACK = os.path.join(ROOT, "packs", "vanilla")
OUT = os.path.join(PACK, "critters")
ENTITY = os.path.join(PACK, "textures", "entity")
ITEM_TEX = os.path.join(PACK, "textures", "item")

GOLD = ["#5a3a08", "#9a6a14", "#d4a02a", "#f2cc52", "#fff0a0"]
CRIT = "kind: Passive, spawn: Never, attack: 0, xp: 1, coins: Some(0)"

MOBS = []   # (name, display, model, ron)
ITEMS = []  # (name, display, stack, icon fn)


def shade_fill(pal, f):
    return ramp_fill(pal, lambda xs, ys: np.broadcast_to(np.asarray(f(xs, ys), float), xs.shape))


def lin(a, x0, y0, x1, y1, w, fill, edge):
    dx, dy = x1 - x0, y1 - y0
    n = math.hypot(dx, dy) or 1.0
    nx, ny = -dy / n * w, dx / n * w
    a.poly([(x0 + nx, y0 + ny), (x1 + nx, y1 + ny), (x1 - nx, y1 - ny), (x0 - nx, y0 - ny)], fill, edge)


def add(name, display, model, ron, pal, icon, stack=16):
    model.pack()
    MOBS.append((name, display, model, ron))
    ITEMS.append((name, display, stack, lambda a, p=pal: icon(a, p)))


# ------------------------------------------------------------------ icons


def worm_icon(a, p):
    pts = [(5 + k * 2.6, 20 + 5 * math.sin(k * 0.9)) for k in range(9)]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        lin(a, x0, y0, x1, y1, 2.4, shade_fill(p, lambda xs, ys: 0.6), p[0])
    a.disc(pts[-1][0], pts[-1][1], 3.0, shade_fill(p, lit(pts[-1][0], pts[-1][1], 3)), p[0])


def bug_icon(a, p, spots=True):
    a.disc(16, 18, 9.0, shade_fill(p, lit(13, 14, 9)), p[0])
    lin(a, 16, 9, 16, 27, 0.6, shade_fill(["#101010", "#202020"], lambda xs, ys: 0.5), "#000000")
    a.disc(16, 8, 4.0, shade_fill(["#101010", "#202024", "#34343a"], lit(15, 7, 4)), "#000000")
    if spots:
        for x, y in ((11, 15), (21, 15), (12, 22), (20, 22)):
            a.disc(x, y, 1.6, shade_fill(["#101010", "#202020"], lambda xs, ys: 0.4), "#000000")


def mouse_icon(a, p):
    a.disc(14, 19, 7.5, shade_fill(p, lit(12, 16, 7)), p[0])
    a.disc(22, 15, 4.5, shade_fill(p, lit(21, 14, 4)), p[0])
    a.disc(20, 10, 2.6, shade_fill(["#c88a8a", "#e8aaaa"], lambda xs, ys: 0.6), p[0])
    lin(a, 7, 21, 2, 26, 0.8, shade_fill(["#c88a8a", "#e8aaaa"], lambda xs, ys: 0.5), "#7a4a4a")
    a.px(24, 14, "#000000")


def scorpion_icon(a, p):
    a.disc(14, 20, 6.5, shade_fill(p, lit(12, 18, 6)), p[0])
    for k in range(4):
        x = 20 + k * 1.5
        y = 18 - k * 3.2
        a.disc(x, y, 2.4 - k * 0.3, shade_fill(p, lit(x, y, 2)), p[0])
    lin(a, 25, 7, 21, 6, 1.0, shade_fill(p, lambda xs, ys: 0.2), p[0])
    for sx in (-1, 1):
        lin(a, 10, 18 + sx * 2, 4, 14 + sx * 6, 1.3, shade_fill(p, lambda xs, ys: 0.5), p[0])


def strider_icon(a, p):
    a.poly([(10, 15), (22, 15), (24, 18), (8, 18)], shade_fill(p, lambda xs, ys: 0.6), p[0])
    for x0, x1, y1 in ((12, 3, 26), (14, 7, 6), (20, 29, 26), (18, 25, 6)):
        lin(a, x0, 17, x1, y1, 0.7, shade_fill(p, lambda xs, ys: 0.4), p[0])
    a.poly([(2, 27), (30, 27), (30, 29), (2, 29)], shade_fill(["#2a5a8a", "#4a8ac0", "#8ac0e8"], lambda xs, ys: 0.6), "#1a3a5a")


def snail_icon(a, p, shell=None):
    shell = shell or p
    a.poly([(4, 24), (26, 24), (28, 27), (4, 27)], shade_fill(p, lambda xs, ys: 0.6), p[0])
    a.disc(15, 16, 8.0, shade_fill(shell, lit(13, 13, 8)), shell[0])
    lin(a, 25, 24, 28, 15, 1.2, shade_fill(p, lambda xs, ys: 0.6), p[0])


def fly_icon(a, p):
    a.poly([(10, 14), (20, 12), (24, 18), (14, 21)], shade_fill(["#2a1408", "#4a2010", "#6a3018"], lambda xs, ys: 0.6), "#0a0402")
    a.disc(11, 19, 5.0, shade_fill(p, lit(11, 18, 5)), p[0])
    a.poly([(14, 9), (22, 4), (24, 10), (17, 13)], shade_fill(["#c8a080", "#e8c8a8", "#fff0e0"], lambda xs, ys: 0.6), "#6a4a30")


def butterfly_icon(a, p):
    a.poly([(16, 15), (4, 5), (6, 15), (4, 25), (16, 18), (28, 25), (26, 15), (28, 5)], shade_fill(p, lit(10, 9, 10)), p[0])
    lin(a, 16, 9, 16, 24, 0.8, shade_fill(["#141414", "#2a2a2a"], lambda xs, ys: 0.5), "#000000")


# ------------------------------------------------------------------ the critters

# Worms: a few soft segments that crawl; pink earthworm, glowing nightcrawler, mushroom truffle worm.
for key, dn, pal, extra in (("worm", "Minhoca", ["#6a2a2a", "#a04a4a", "#c86a64", "#e8948a"], ""),
                            ("gold_worm", "Minhoca Dourada", GOLD[1:], ", lit: true"),
                            ("enchanted_nightcrawler", "Minhoca Encantada", ["#3a1a6a", "#6a3aaa", "#a070e0", "#e0c8ff"], ", lit: true"),
                            ("truffle_worm", "Minhoca Trufada", ["#1a3a8a", "#2a5ac0", "#5a9af0", "#b8e0ff"], ", lit: true, flee_range: 8.0")):
    m = worm(4, (1, 1, 2), look(key + "h", pal, eyes=((10, 10, 10), None)), look(key + "b", pal, pattern="rings"))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 2, speed: 0.5, width: 0.25, height: 0.15" + extra, pal, worm_icon, 4 if "gold" in key or "truffle" in key else 16)

# Ladybugs: a domed red shell on six short legs (luck when let go, bad luck when killed).
for key, dn, pal in (("ladybug", "Joaninha", ["#5a0a0a", "#a01a14", "#d8301e", "#f86a4a"]),
                     ("gold_ladybug", "Joaninha Dourada", GOLD[1:])):
    m = Model()
    m.box((3, 2, 4), (-1, 0, -2), look(key + "s", pal, pattern="patches"))
    m.box((2, 1, 1), (-1, 0, 2), look(key + "hd", ["#101010", "#202024", "#34343a"], eyes=((240, 240, 236), None)), "Head", (0, 1, 2))
    for k, x in enumerate((-2, 2)):
        m.box((1, 1, 3), (x, 0, -1), look(key + "l", ["#101010", "#1a1a1a"]), f"Leg({k})", (x, 1, 0))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 1, speed: 0.8, width: 0.25, height: 0.15, climbs: true" + (", lit: true" if "gold" in key else ""), pal, bug_icon, 4 if "gold" in key else 16)

# Mice: cave floors.
for key, dn, pal in (("mouse", "Rato", ["#3a3a3c", "#56565a", "#727278", "#9a9aa0"]),
                     ("gold_mouse", "Rato Dourado", GOLD[1:])):
    m = quadruped((3, 3, 5), (3, 3, 3), 1, 1, look(key + "b", pal), look(key + "h", pal, eyes=((10, 10, 10), None)), look(key + "l", ["#c88a8a", "#e8aaaa"]),
                  snout=(1, 1, 1), snout_look=look(key + "n", ["#c88a8a", "#e8aaaa"]), ears=(1, 1, 1))
    m.box((1, 1, 5), (0, 1, -7), look(key + "t", ["#b87a7a", "#d89a9a"]))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 2, speed: 3.2, width: 0.3, height: 0.25, flee_range: 5.0" + (", lit: true" if "gold" in key else ""), pal, mouse_icon, 4 if "gold" in key else 16)

# Scorpions: sand, with a raised tail.
for key, dn, pal in (("scorpion", "Escorpião", ["#5a3a10", "#8a5a1a", "#b47c2a", "#d8a450"]),
                     ("black_scorpion", "Escorpião Negro", ["#0a0a0c", "#1a1a1e", "#2c2c32", "#44444c"])):
    m = Model()
    m.box((3, 2, 4), (-1, 0, -2), look(key + "b", pal, pattern="plate"))
    m.box((2, 1, 2), (-1, 0, 2), look(key + "h", pal, eyes=((10, 10, 10), None)), "Head", (0, 1, 2))
    for k, x in enumerate((-3, 2)):
        m.box((2, 1, 2), (x, 0, 3), look(key + "c", pal), "Head", (0, 1, 2))
        m.box((1, 1, 3), (x + (1 if k == 0 else 0), 0, -1), look(key + "l", pal[:3]), f"Leg({k})", (x, 1, 0))
    m.box((1, 1, 3), (0, 1, -5), look(key + "t", pal, pattern="rings"))
    m.box((1, 3, 1), (0, 2, -5), look(key + "t2", pal, pattern="rings"))
    m.box((1, 1, 2), (0, 5, -5), look(key + "st", ["#2a0a0a", "#5a1a14"]))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 3, speed: 1.6, width: 0.3, height: 0.2, flee_range: 3.0", pal, scorpion_icon)

# Water striders: a slim body high on four long splayed legs, skating on still water.
for key, dn, pal in (("water_strider", "Alfaiate", ["#1a1410", "#2e241a", "#4a3a2a", "#6a5640"]),
                     ("gold_water_strider", "Alfaiate Dourado", GOLD[1:])):
    m = Model()
    m.box((1, 1, 5), (0, 2, -2), look(key + "b", pal))
    m.box((1, 1, 1), (0, 2, 3), look(key + "h", pal, eyes=((10, 10, 10), None)), "Head", (0, 2, 3))
    for k, (x, z) in enumerate(((-5, -3), (1, -3), (-5, 2), (1, 2))):
        m.box((5, 1, 1), (x, 1, z), look(key + "l", pal[:3]), f"Leg({k % 2})", (0, 2, z))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 2, speed: 2.2, width: 0.4, height: 0.15, swims: true, flee_range: 4.0" + (", lit: true" if "gold" in key else ""), pal, strider_icon, 4 if "gold" in key else 16)

# Snails that glow: blue in caves, molten in the Underworld.
for key, dn, body, shell in (("glowing_snail", "Caracol Brilhante", ["#1a3a6a", "#2a5aa0", "#4a8ad8", "#8ac8ff"], ["#0a2a5a", "#1a4a90", "#3a7ad0", "#a8e0ff"]),
                             ("magma_snail", "Caracol de Magma", ["#3a1a10", "#5a2a18", "#7a3a20", "#9a4a28"], ["#5a0a00", "#b02a00", "#f06010", "#ffd060"])):
    m = Model()
    m.box((3, 2, 6), (-1, 0, -3), look(key + "b", body))
    m.box((4, 4, 4), (-2, 2, -3), look(key + "s", shell, pattern="rings"))
    m.box((1, 2, 1), (0, 2, 3), look(key + "e", body), "Head", (0, 2, 3))
    add(f"critter_{key}", dn, m, f"{CRIT}, health: 2, speed: 0.4, width: 0.3, height: 0.3, climbs: true, lit: true" + ("" if "magma" in key else ""), body, lambda a, p, s=shell: snail_icon(a, p, s))

# The Underworld's own: lava flies (glowing embers that drift) and hell butterflies.
m = Model()
m.box((2, 2, 3), (-1, 0, -1), look("lfbody", ["#2a1408", "#4a2010", "#6a3018"]))
m.box((2, 2, 2), (-1, 0, -3), look("lfglow", ["#a02000", "#f06010", "#ffb040", "#fff0a0"]))
for k, x in enumerate((-3, 1)):
    m.box((2, 1, 2), (x, 2, -1), look("lfwing", ["#c8a080", "#e8c8a8", "#fff0e0"]), f"Wing({k})", (x + (2 if k == 0 else 0), 2, 0))
add("critter_lavafly", "Mosca de Lava", m, f"{CRIT}, health: 2, speed: 1.2, width: 0.25, height: 0.25, flies: true, hover: 1.5, lit: true",
    ["#a02000", "#f06010", "#ffb040", "#fff0a0"], fly_icon)
m = Model()
m.box((1, 1, 4), (0, 0, -2), look("hbb", ["#141414", "#2a2a2a", "#404040"]))
for k, x in enumerate((-5, 1)):
    m.box((5, 1, 5), (x, 1, -2), look("hbw", ["#3a0a00", "#a02a00", "#f06010", "#ffc040"], pattern="petals"), f"Wing({k})", (x + (5 if k == 0 else 0), 1, 0))
add("critter_hell_butterfly", "Borboleta Infernal", m, f"{CRIT}, health: 2, speed: 1.6, width: 0.3, height: 0.25, flies: true, hover: 1.2, lit: true",
    ["#3a0a00", "#a02a00", "#f06010", "#ffc040"], butterfly_icon)


# The nautilus (Java 26.2): a coiled striped shell with a hood and a fringe of tentacles; a
# tamed one carries the player under water and lets them breathe.
SHELL = ["#6a2a10", "#b8562a", "#e8a070", "#f6e2cc"]
nau = Model()
nau.box((8, 10, 9), (-4, 0, -6), look("nshell", SHELL, pattern="rings"))
nau.box((6, 6, 3), (-3, 2, 3), look("nhood", ["#5a2a14", "#8a4a2a", "#b06a44"], eyes=((240, 200, 60), None)), "Head", (0, 4, 3))
for k, x in enumerate((-3, -1, 1)):
    nau.box((1, 1, 4), (x, 1, 6), look("ntent", ["#c88a70", "#e0aa90", "#f0c8b0"]), f"Leg({k % 2})", (x, 1, 6))
for k, x in enumerate((-2, 0, 2)):
    nau.box((1, 1, 3), (x, 3, 6), look("ntent", ["#c88a70", "#e0aa90", "#f0c8b0"]), f"Leg({(k + 1) % 2})", (x, 3, 6))
nau.pack()
MOBS.append(("nautilus", "Náutilo", nau, 'kind: Passive, health: 15, speed: 2.4, xp: 3, coins: Some(0), width: 0.9, height: 0.8, spawn: Water, group: (1, 2), weight: 2, aquatic: true, biomes: ["Ocean", "Deep Ocean", "Warm Ocean", "Lukewarm Ocean"], drops: [(item: "nautilus_shell", chance: 0.3)]'))


# Minecraft's own small animals the bug net catches too (the item lets them go again).
def bird_icon(a, p, breast):
    a.disc(15, 18, 7.5, shade_fill(p, lit(13, 15, 7)), p[0])
    a.disc(21, 11, 4.5, shade_fill(p, lit(20, 10, 4)), p[0])
    a.disc(16, 21, 4.0, shade_fill(breast, lit(15, 20, 4)), breast[0])
    a.poly([(25, 10), (29, 12), (25, 13)], shade_fill(["#7a5a10", "#c89a20"], lambda xs, ys: 0.6), "#3a2a08")
    a.poly([(8, 16), (2, 12), (4, 20)], shade_fill(p, lambda xs, ys: 0.4), p[0])
    a.px(22, 10, "#000000")


def bunny_icon(a, p):
    a.disc(14, 20, 7.5, shade_fill(p, lit(12, 17, 7)), p[0])
    a.disc(21, 14, 4.5, shade_fill(p, lit(20, 13, 4)), p[0])
    for x in (19, 22):
        a.poly([(x, 11), (x - 1, 3), (x + 1.5, 3), (x + 2, 11)], shade_fill(p, lambda xs, ys: 0.7), p[0])
    a.disc(7, 19, 2.2, shade_fill(["#d8d8d8", "#ffffff"], lambda xs, ys: 0.6), "#8a8a8a")
    a.px(23, 13, "#000000")


def frog_icon(a, p):
    a.disc(16, 19, 8.5, shade_fill(p, lit(14, 16, 8)), p[0])
    for x in (11, 21):
        a.disc(x, 11, 3.0, shade_fill(p, lit(x, 10, 3)), p[0])
        a.px(x, 11, "#000000")
    lin(a, 8, 24, 3, 27, 1.4, shade_fill(p, lambda xs, ys: 0.4), p[0])
    lin(a, 24, 24, 29, 27, 1.4, shade_fill(p, lambda xs, ys: 0.4), p[0])


CAUGHT = [
    ("rabbit", "Coelho", ["#4a3420", "#7a5a3a", "#a07c58", "#c8a47c"], bunny_icon),
    ("frog", "Sapo", ["#2a4a14", "#3e6a1e", "#5a8a2a", "#84b04a"], frog_icon),
    ("duck", "Pato", ["#3a3020", "#6a5a3a", "#9a8a6a", "#c8b898"], lambda a, p: bird_icon(a, p, ["#5a4a2a", "#8a7a5a"])),
    ("bluejay", "Gaio-Azul", ["#1a2a6a", "#2a4aa0", "#4a7ad0", "#8ab0f0"], lambda a, p: bird_icon(a, p, ["#c8c8d0", "#f0f0f8"])),
    ("cardinal", "Cardeal", ["#5a0a0a", "#a01a14", "#d8301e", "#f86a4a"], lambda a, p: bird_icon(a, p, ["#a01a14", "#d8301e"])),
    ("finch", "Tentilhão", ["#5a4a10", "#9a8a20", "#d0c040", "#f0e070"], lambda a, p: bird_icon(a, p, ["#d0c040", "#f0e070"])),
    ("robin", "Pisco", ["#3a3430", "#5a524a", "#7a7068", "#9a9088"], lambda a, p: bird_icon(a, p, ["#b04a1a", "#e07a3a"])),
    ("sparrow", "Pardal", ["#3a2a1a", "#6a4e30", "#9a7650", "#c4a07a"], lambda a, p: bird_icon(a, p, ["#c8b8a0", "#e8dcc8"])),
    ("canary", "Canário", ["#7a6a08", "#c0a810", "#f0d830", "#fff07a"], lambda a, p: bird_icon(a, p, ["#f0d830", "#fff07a"])),
]
for key, dn, pal, icon in CAUGHT:
    ITEMS.append((f"caught_{key}", dn, 16, lambda a, p=pal, f=icon: f(a, p)))


def ron_mob(name, display, model):
    parts = []
    for p in model.parts:
        s = f"(size: {p['size']}, origin: {p['origin']}, uv: {p['uv']}"
        if p["anim"]:
            s += f", anim: {p['anim']}, pivot: {p['pivot']}"
        parts.append(s + ")")
    return parts


def main():
    os.makedirs(OUT, exist_ok=True)
    head = "// Terraria's critters in 3D (plan/animais-analise.md, lote 3). Made by tools/gen_critters.py:\n// edit it and run it again.\n"
    text = head + "(mobs: [\n"
    for name, display, model, ron in MOBS:
        text += f'    (\n        name: "{name}", display_name: "{display}", {ron},\n        parts: [\n'
        text += "".join(f"            {p},\n" for p in ron_mob(name, display, model)) + "        ],\n    ),\n"
        render_mob(name, model).save(os.path.join(ENTITY, name + ".png"))
    open(os.path.join(OUT, "mobs.ron"), "w", encoding="utf-8").write(text + "])\n")
    text = head + "(items: [\n"
    for name, display, stack, icon in ITEMS:
        text += f'    (name: "{name}", display_name: Some("{display}"), max_stack: Some({stack})),\n'
        a = Art()
        icon(a)
        a.image().save(os.path.join(ITEM_TEX, name + ".png"))
    open(os.path.join(OUT, "items.ron"), "w", encoding="utf-8").write(text + "])\n")
    print(f"wrote {len(MOBS)} critters")


if __name__ == "__main__":
    main()
