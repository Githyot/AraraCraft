#!/usr/bin/env python3
"""The archer class's own content (plan/arcos-analise.md, HP 2026-11-01).

Writes packs/vanilla/arqueiro/items.ron and recipes.ron, and the 16x16 designs of the trick
bows (same drawing rules as tools/gen_bow_shapes.py, coloured like the ore bows); then run

    python3 tools/tex64/build.py --only <the bow names>

to grow them into the 64x64 pixel-art icons. The arrows' and crossbow's icons are drawn by
tools/tex64/archery_art.py. How each bow shoots is in crates/game/src/archery.rs.
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import gen_bow_shapes as bs  # noqa: E402
import gen_tr as t  # noqa: E402
import gen_textures as gt  # noqa: E402

ROOT = os.path.dirname(HERE)
OUT = os.path.join(ROOT, "packs", "vanilla", "arqueiro")
ITEM_DIR = os.path.join(ROOT, "packs", "vanilla", "textures", "item")

arc = lambda D: (lambda u: D * math.sin(math.pi * u))  # noqa: E731
const = lambda w: (lambda u: w)  # noqa: E731
S = {}

# hive: a honey-gold bow wrapped in comb, a bee at the grip
g = bs.new(); d = arc(4.4); bs.limb(g, d, lambda u: 1.4 + 0.8 * math.sin(math.pi * u)); bs.string(g)
for u in (0.2, 0.32, 0.68, 0.8):
    x, y = bs.pt(u, d(u) + 1.2); bs.put(g, x, y, '1'); bs.put(g, x + 1, y, '4')
x, y = bs.pt(0.5, d(0.5)); bs.put(g, x, y, '1'); bs.put(g, x - 1, y - 1, '5')
S["hive"] = bs.rows(g)

# batwing: two bat wings for limbs, scalloped on the back
g = bs.new(); d = lambda u: 5.0 * math.sin(math.pi * u) + 0.9 * abs(math.sin(5 * math.pi * u))  # noqa: E731
bs.limb(g, d, lambda u: 1.0 + 1.6 * math.sin(math.pi * u)); bs.string(g)
x, y = bs.pt(0.5, d(0.5)); bs.put(g, x, y, '5')
S["batwing"] = bs.rows(g)

# marrow: knuckled bone, knobs at the tips and joints
g = bs.new(); d = arc(4.0); bs.limb(g, d, const(1.5)); bs.string(g)
for u in (0.0, 0.25, 0.75, 1.0):
    cx, cy = bs.pt(u, d(u))
    for dx, dy in ((0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)):
        bs.put(g, cx + dx, cy + dy, '4' if (dx, dy) == (0, 0) else '3')
bs.grip(g, '1', depth=d)
S["marrow"] = bs.rows(g)

# frost: icicles hanging off the back of the limbs
g = bs.new(); d = arc(4.2); bs.limb(g, d, const(1.4)); bs.string(g)
for u in (0.18, 0.3, 0.42, 0.58, 0.7, 0.82):
    for k in range(3):
        x, y = bs.pt(u, d(u) + 1.0 + k * 0.8); bs.put(g, x, y, '5' if k == 2 else '4')
S["frost"] = bs.rows(g)

# storm: a cloud-grey bow, a lightning zigzag along the limbs
g = bs.new(); d = arc(4.6); bs.limb(g, d, const(2.0)); bs.string(g)
for u in (0.15, 0.22, 0.29, 0.71, 0.78, 0.85):
    x, y = bs.pt(u, d(u) + (0.5 if int(u * 100) % 2 else -0.3)); bs.put(g, x, y, '5')
bs.grip(g, '1', depth=d)
S["storm"] = bs.rows(g)

# pulse: a ring at the grip, thin straight limbs with glowing tips
g = bs.new(); d = lambda u: 4.4 * (1 - abs(2 * u - 1)) ** 0.85  # noqa: E731
bs.limb(g, d, const(1.2)); bs.string(g)
cx, cy = bs.pt(0.5, d(0.5) - 0.5)
for a in range(0, 360, 30):
    bs.put(g, cx + 1.6 * math.cos(math.radians(a)), cy + 1.6 * math.sin(math.radians(a)), '4')
bs.put(g, cx, cy, '5')
for u in (0.02, 0.98):
    x, y = bs.pt(u, d(u)); bs.put(g, x, y, '5')
S["pulse"] = bs.rows(g)

# tide: a wave crest curling off each limb
g = bs.new(); d = arc(4.4); bs.limb(g, d, lambda u: 1.2 + 1.0 * math.sin(math.pi * u)); bs.string(g)
for u in (0.25, 0.75):
    for k in range(5):
        a = k * 0.9
        x, y = bs.pt(u + (0.03 if u < 0.5 else -0.03) * math.cos(a), d(u) + 1.6 + 1.0 * math.sin(a))
        bs.put(g, x, y, '5' if k == 4 else '4')
S["tide"] = bs.rows(g)

# phoenix: feathered flame limbs, a fire gem
g = bs.new(); d = arc(4.0); bs.limb(g, d, const(1.3)); bs.string(g)
for u in (0.1, 0.2, 0.3, 0.7, 0.8, 0.9):
    lean = 0.05 if u < 0.5 else -0.05
    for k in range(3):
        x, y = bs.pt(u + lean * k, d(u) + 1.0 + k * 0.85); bs.put(g, x, y, '5' if k == 2 else '4')
x, y = bs.pt(0.5, d(0.5)); bs.put(g, x, y, '5')
S["phoenix"] = bs.rows(g)

# skysplit: a split limb, two prongs at each end
g = bs.new(); d1 = arc(5.4); d2 = arc(3.0)
bs.limb(g, d1, lambda u: 0.9 + 0.6 * math.sin(math.pi * u)); bs.limb(g, d2, const(0.8), shade=False); bs.string(g)
x, y = bs.pt(0.5, 4.2)
for dx, dy in ((0, 0), (1, 1), (-1, -1), (1, -1), (-1, 1)):
    bs.put(g, x + dx, y + dy, '5' if (dx, dy) == (0, 0) else '4')
S["skysplit"] = bs.rows(g)

PALS = {
    "hive": t.pal("#5a3404", "#9a6008", "#d89a18", "#f4c440", "#fff0a0"),
    "batwing": t.pal("#1c0c10", "#3c1820", "#6a2a30", "#a8423a", "#ff8a50"),
    "marrow": t.pal("#4a4434", "#8a8268", "#c8c0a4", "#e8e2cc", "#fffaf0"),
    "frost": t.pal("#1c3a5a", "#3a6a9a", "#68a4d4", "#a8d8f4", "#f0fcff"),
    "storm": t.pal("#1e2430", "#3c4658", "#68748a", "#a4aec0", "#fff27a"),
    "pulse": t.pal("#0c2a3a", "#1a5068", "#2a86a8", "#5cc8e8", "#d8f8ff"),
    "tide": t.pal("#082a40", "#104e74", "#1c7eac", "#4ab4d8", "#d4f4ff"),
    "phoenix": t.pal("#4a0c04", "#8e2008", "#d84c10", "#f89a28", "#fff0a0"),
    "skysplit": t.pal("#1a1a40", "#30307a", "#5050b8", "#8a8ae8", "#e0e0ff"),
}

# (item, display name, durability, recipe pattern key) per bow.
BOWS = [
    ("hive_bow", "Arco da Colmeia", 700, ([" WS", "H S", " WS"], {"W": "beeswax", "H": "honeycomb", "S": "string"})),
    ("batwing_bow", "Arco Asa-de-Morcego", 1000, ([" MS", "F S", " MS"], {"M": "magmarite_ingot", "F": "phoenix_feather", "S": "string"})),
    ("marrow_bow", "Arco de Tutano", 1200, ([" BS", "N S", " BS"], {"B": "bone", "N": "soul_of_night", "S": "string"})),
    ("frost_bow", "Arco Gélido", 1200, ([" FS", "C S", " FS"], {"F": "frost_shard", "C": "frost_core", "S": "string"})),
    ("storm_bow", "Arco da Tempestade", 1300, ([" MS", "F S", " MS"], {"M": "mythril_ingot", "F": "soul_of_flight", "S": "string"})),
    ("pulse_bow", "Arco Pulsante", 1800, ([" VS", "E S", " VS"], {"V": "verdite_ingot", "E": "soul_of_sight", "S": "string"})),
    ("tide_bow", "Arco da Maré", 1800, ([" TS", "P S", " TS"], {"T": "serpent_tooth", "P": "prismarine_shard", "S": "string"})),
    ("phoenix_bow", "Arco Fênix", 2400, ([" LS", "F S", " LS"], {"L": "lumite_ingot", "F": "phoenix_feather", "S": "string"})),
    ("skysplit_bow", "Arco Celeste Partido", 2400, ([" LS", "V S", " LS"], {"L": "lumite_ingot", "V": "vortex_fragment", "S": "string"})),
]

# Arrows of the archer's road (archery.rs `arrow_fx`, tr3_tools.rs `arrow_traits`).
ARROWS = [
    ("bone_arrow", "Flecha de Osso", "bone"),
    ("venom_arrow", "Flecha Venenosa", "jungle_spore"),
    ("cursed_arrow", "Flecha Amaldiçoada", "soul_of_night"),
    ("ichor_arrow", "Flecha de Icor", "sanguinite_ingot"),
    ("verdite_arrow", "Flecha de Verdita", "verdite_ingot"),
    ("lumite_arrow", "Flecha de Lumita", "lumite_ingot"),
]


# Accessories of the archer (rpg.rs ACCESSORIES): (item, name, recipe pattern, key).
ACCESSORIES = [
    ("molten_quiver", "Aljava Derretida", ["MQM", "M M", "MMM"], {"M": "magmarite_ingot", "Q": "hunter_quiver"}),
    ("magic_quiver", "Aljava Mágica", ["LQN", "S S", "SSS"], {"L": "soul_of_light", "N": "soul_of_night", "Q": "hunter_quiver", "S": "string"}),
    ("ranger_scope", "Mira do Patrulheiro", [" E ", "MSM", " M "], {"E": "soul_of_sight", "S": "spyglass", "M": "mythril_ingot"}),
]


def key_text(k):
    return "{" + ", ".join(f"'{c}': \"{v}\"" for c, v in k.items()) + "}"


def main():
    os.makedirs(OUT, exist_ok=True)
    items = ["// The archer class's own items (plan/arcos-analise.md). Made by tools/gen_archery.py:", "// edit it and run it again.", "(items: ["]
    shaped = []
    shapeless = []
    for name, disp, dur, (pat, key) in BOWS:
        items.append(f'    (name: "{name}", display_name: Some("{disp}"), max_stack: Some(1), tool: Some((kind: Sword, tier: 1, speed: 1.0, damage: 1.0, durability: {dur}))),')
        pats = "[" + ", ".join(f'"{r}"' for r in pat) + "]"
        shaped.append(f'        (pattern: {pats}, key: {key_text(key)}, result: "{name}"),')
        img = t.icon2(S[name[:-4]], PALS[name[:-4]], t.STRING)
        gt.to_image(img).save(os.path.join(ITEM_DIR, name + ".png"))
    for name, disp, pat, key in ACCESSORIES:
        items.append(f'    (name: "{name}", display_name: Some("{disp}"), max_stack: Some(1)),')
        pats = "[" + ", ".join(f'"{r}"' for r in pat) + "]"
        shaped.append(f'        (pattern: {pats}, key: {key_text(key)}, result: "{name}"),')
    for name, disp, extra in ARROWS:
        items.append(f'    (name: "{name}", display_name: Some("{disp}")),')
        shapeless.append(f'        (ingredients: ["arrow", "arrow", "arrow", "arrow", "arrow", "{extra}"], result: "{name}", count: 5),')
    items.append("])")
    with open(os.path.join(OUT, "items.ron"), "w") as f:
        f.write("\n".join(items) + "\n")
    rec = ["// Recipes of the archer class's items. Made by tools/gen_archery.py.", "(", "    shaped: ["] + shaped + ["    ],", "    shapeless: ["] + shapeless + ["    ],", ")"]
    with open(os.path.join(OUT, "recipes.ron"), "w") as f:
        f.write("\n".join(rec) + "\n")
    print(",".join(b[0] for b in BOWS))


if __name__ == "__main__":
    main()
