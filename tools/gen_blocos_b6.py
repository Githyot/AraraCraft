#!/usr/bin/env python3
"""Lote B6 of the block review (plan/blocos-analise.md): Terraria building blocks, made for 3D.

- Platforms in every wood (and stone and glass): thin decks you jump up through, stand on, and
  drop through by sneaking (Terrain::oneway). In 3D they make quick floors for farms, arenas
  and stairs without closing the room.
- Rope (climbable like a ladder, hangs as a thin cord), wooden spikes (they hurt like the dungeon
  spikes, which become 3D here too: no crossed pictures).
- Metal bricks (copper, tin, silver, gold, platinum, tungsten, lead, meteorite) and gemspark:
  gem bricks that shine in their gem's colour (gem_glow.rs), for lit builds.
- Marble (raw, polished, bricks, fluted pillar, with slabs, stairs and walls), desert fossil, silt
  and slush (the extractinator sifts them), snow and ice bricks, sun plate, confetti, bubble,
  rain and snow clouds.

Pictures are our own (drawn here or from our textures). Writes blocks between `// B6 start` ..
`// B6 end` in packs/vanilla/blocks.ron and packs/vanilla/blocos/recipes_b6.ron. Run
gen_block_models.py first.
"""

import glob
import os
import re
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))
sys.path.insert(0, os.path.join(ROOT, "tools", "tex64"))
import gen_blocos_b2 as b2  # noqa: E402
import blocos_b1 as t1  # noqa: E402
import blocks as tb  # noqa: E402
from style import N, XX, YY, blur, clusters, fbm, light, normalize, quantize, ramp, rgba, rng_for, soften, value_noise  # noqa: E402

BLOCKS = b2.BLOCKS
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes_b6.ron")

METALS = [("copper", "Cobre", "#c46e48", "copper_ingot"), ("tin", "Estanho", "#b8b4a0", "tin_ingot"),
          ("silver", "Prata", "#c8ccd4", "silver_ingot"), ("gold", "Ouro", "#e0b030", "gold_ingot"),
          ("platinum", "Platina", "#b8d0dc", "platinum_ingot"), ("tungsten", "Tungstênio", "#7a9a84", "tungsten_ingot"),
          ("lead", "Chumbo", "#5a6478", "lead_ingot"), ("meteorite", "Meteorito", "#8a4a5a", "meteorite_ingot")]
GEMS = [("amethyst", "Ametista", "#a060e0", "amethyst"), ("topaz", "Topázio", "#f0a020", "topaz"),
        ("sapphire", "Safira", "#3070f0", "sapphire"), ("emerald", "Esmeralda", "#30d060", "emerald"),
        ("ruby", "Rubi", "#e02838", "ruby"), ("diamond", "Diamante", "#90e8f8", "diamond")]


def metal_bricks(name, col):
    """Stone bricks faced with metal: the metal's colour, a bright top edge on each brick."""
    img = tb.stone_bricks(name, "#8a8a8a")
    img = b2.tint(img, col, 0.3)
    lum = img[..., :3].mean(-1)
    edge = (lum > np.roll(lum, 1, 0) + 25)
    img[edge, :3] = np.clip(img[edge, :3] * 1.25 + 20, 0, 255)
    return img


def gemspark(name, col):
    """Four glowing panes: bright middles fading to a darker rim, a thin frame between."""
    c = b2.hexc(col)
    u = (XX % 32) / 31.0 - 0.5
    v = (YY % 32) / 31.0 - 0.5
    d = np.maximum(np.abs(u), np.abs(v)) * 2
    k = 1.25 - 0.55 * d ** 1.6
    rng = rng_for(name)
    k = k + (clusters(rng, 4) - 0.5) * 0.08
    rgb = np.clip(c * k[..., None] + (k[..., None] - 1.0).clip(0) * 255 * 0.6, 0, 255)
    frame = ((XX % 32) < 2) | ((YY % 32) < 2)
    rgb[frame] = c * 0.45
    return rgba(np.round(rgb / 8) * 8)


def marble():
    rng = rng_for("marble")
    pal = ramp("#e6e2da", 10, 0.18, 0.04)
    f = fbm(rng, ((32, 0.5), (16, 0.3), (8, 0.2)))
    w = fbm(rng, ((32, 0.6), (16, 0.4)))
    vein = np.abs(np.sin((XX * 0.6 + YY * 0.9 + w * 70) / 9.0))
    t = 0.66 + (f - 0.5) * 0.18 - (vein < 0.16) * 0.3 - (vein < 0.07) * 0.3
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.2, rng)), "natural")


def flutes(name):
    """Fluted column side: vertical grooves, lit from the left."""
    rng = rng_for(name)
    pal = ramp("#e6e2da", 10, 0.2, 0.04)
    h = np.cos((XX % 10) / 10.0 * 2 * np.pi) * 1.0
    surf = fbm(rng, ((16, 0.5), (8, 0.5)))
    t = 0.58 + light(h * 1.5, wrap=True) * 0.9 + (surf - 0.5) * 0.12
    t[(YY < 4) | (YY > 59)] = 0.75
    t[(YY == 4) | (YY == 59)] = 0.4
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.2, rng)), "built")


def grains(name, base, spread=0.3, dark=0.0):
    rng = rng_for(name)
    pal = ramp(base, 10, spread, 0.06)
    c = clusters(rng, 2)
    f = fbm(rng, ((16, 0.5), (4, 0.3), (2, 0.2)))
    t = 0.5 + (c - 0.5) * 0.45 + (f - 0.5) * 0.2 - dark
    return soften(rgba(quantize(np.clip(t, 0, 1), pal, 0.35, rng)), "natural")


def fossil():
    """Sandstone with a buried skeleton: a curved spine and ribs, each bone with a dark rim."""
    img = b2.load("sandstone_top")
    rng = rng_for("desert_fossil")
    bone = np.zeros((N, N), bool)
    for k in range(7):
        x, y = 4 + k * 9, 28 + int(5 * np.sin(k * 0.8))
        bone[y - 4:y + 4, x - 3:x + 4] = True
        if k % 2 == 0 and k < 6:
            for s in (-1, 1):
                for j in range(4, 20):
                    yy, xx = y + s * j, x + int(j * 0.45)
                    if 0 <= yy < N and xx + 3 <= N:
                        bone[yy, xx:xx + 3] = True
    rim = (np.roll(bone, 1, 0) | np.roll(bone, -1, 0) | np.roll(bone, 1, 1) | np.roll(bone, -1, 1)) & ~bone
    img[rim, :3] *= 0.6
    lum = rng.random((N, N)) * 0.08 + 0.94
    img[bone, :3] = b2.hexc("#f2e8cc") * lum[bone][:, None]
    return img


def confetti():
    rng = rng_for("confetti_block")
    img = rgba(np.full((N, N, 3), 236.0))
    img[..., :3] -= (clusters(rng, 4)[..., None] - 0.5) * 18
    cols = ["#e83a3a", "#f0c020", "#3a7ae8", "#30c060", "#d050d0", "#f08030"]
    for _ in range(70):
        x, y = rng.integers(0, N - 2, 2)
        img[y:y + 2, x:x + 3 if rng.random() < 0.5 else x + 2, :3] = b2.hexc(cols[rng.integers(0, len(cols))])
    return img


def bubble():
    """A soap film: almost clear, with a rainbow sheen and a bright window highlight."""
    img = np.zeros((N, N, 4))
    t = (XX + YY) / 126.0
    rgb = np.stack([0.5 + 0.5 * np.sin(2 * np.pi * (t + o)) for o in (0.0, 0.33, 0.66)], -1) * 120 + 135
    img[..., :3] = rgb
    img[..., 3] = 60
    rim = (XX < 2) | (YY < 2) | (XX > 61) | (YY > 61)
    img[rim, 3] = 150
    hi = ((XX - 18) ** 2 + (YY - 16) ** 2 < 30) & ((XX - 18) ** 2 + (YY - 16) ** 2 > 12)
    img[hi, :3] = 255
    img[hi, 3] = 200
    return img


def rope():
    img = np.zeros((N, N, 4))
    for y in range(N):
        for x in range(28, 36):
            band = ((x - 28) + y) % 8 < 4
            c = b2.hexc("#b08a54") if band else b2.hexc("#8a6436")
            shade = 0.8 + 0.25 * np.sin((x - 28) / 7 * np.pi)
            img[y, x, :3] = c * shade
            img[y, x, 3] = 255
    return img


blocks = []
recipes = []
shapeless = []


def blk(line):
    blocks.append("    " + line + ",")


def shaped(pattern, key, result, count=1):
    keys = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    pats = ", ".join(f'"{p}"' for p in pattern)
    recipes.append(f"        (pattern: [{pats}], key: {{{keys}}}, result: \"{result}\", count: {count}),")


WOOD_PT = {"oak": "Carvalho", "spruce": "Pinheiro", "birch": "Bétula", "jungle": "Selva", "acacia": "Acácia", "dark_oak": "Carvalho Escuro",
           "mangrove": "Mangue", "cherry": "Cerejeira", "pale_oak": "Carvalho-pálido", "crimson": "Carmesim", "warped": "Distorcida",
           "bamboo": "Bambu"}


def pictures(gm):
    for m, _, col, _ in METALS:
        b2.save(f"{m}_bricks", metal_bricks(f"{m}_bricks", col))
    for g, _, col, _ in GEMS:
        b2.save(f"{g}_gemspark", gemspark(f"{g}_gemspark", col))
    b2.save("marble", marble())
    b2.save("polished_marble", t1.polished("polished_marble", "marble"))
    b2.save("marble_bricks", tb.stone_bricks("marble_bricks", "#dcd8d0"))
    b2.save("marble_pillar", flutes("marble_pillar"))
    b2.save("marble_pillar_top", t1.chiseled("marble_pillar_top", "marble", "rings"))
    b2.save("desert_fossil", fossil())
    b2.save("silt", grains("silt", "#8a8478"))
    b2.save("slush", grains("slush", "#b8ccdc", 0.25))
    b2.save("snow_bricks", tb.stone_bricks("snow_bricks", "#e6eef6"))
    b2.save("ice_bricks", tb.stone_bricks("ice_bricks", "#9cc8ec"))
    b2.save("sun_plate", t1.chiseled("sun_plate", "gold_block", "sun"))
    b2.save("confetti_block", confetti())
    b2.save("bubble_block", bubble())
    b2.save("rain_cloud", b2.tint(b2.load("cloud_block"), "#8a92a0", 0.4))
    b2.save("snow_cloud", b2.tint(b2.load("cloud_block"), "#e0ecf8", 0.5))
    b2.save("rope", rope())
    for n in [f"{w}_platform" for w in gm.PLATFORM_WOODS] + ["stone_platform", "glass_platform", "wooden_spikes", "dungeon_spikes"]:
        b2.iso_icon(gm.MODELS[n], n)


def data(gm):
    pick, axe, shovel = "tool: Some(Pickaxe)", "tool: Some(Axe)", "tool: Some(Shovel)"
    for w in gm.PLATFORM_WOODS:
        blk(f'(name: "{w}_platform", display_name: Some("Plataforma de {WOOD_PT[w]}"), transparency: Cutout, solid: false, hardness: 1.0, '
            f'{axe}, textures: (all: Some("{w}_platform")))')
        shaped(["#|#"], {"#": f"{w}_planks", "|": "stick"}, f"{w}_platform", 6)
    blk(f'(name: "stone_platform", display_name: Some("Plataforma de Pedra"), transparency: Cutout, solid: false, hardness: 1.5, {pick}, '
        'textures: (all: Some("stone_platform")))')
    blk('(name: "glass_platform", display_name: Some("Plataforma de Vidro"), transparency: Cutout, solid: false, hardness: 0.3, '
        'textures: (all: Some("glass_platform")))')
    shaped(["#|#"], {"#": "stone_bricks", "|": "stick"}, "stone_platform", 6)
    shaped(["#|#"], {"#": "glass", "|": "stick"}, "glass_platform", 6)
    blk('(name: "rope", display_name: Some("Corda"), shape: Chain, transparency: Cutout, solid: false, hardness: 0.2)')
    shaped(["S", "S", "S"], {"S": "string"}, "rope", 6)
    blk(f'(name: "wooden_spikes", display_name: Some("Espinhos de Madeira"), transparency: Cutout, solid: false, hardness: 1.0, {axe}, '
        'textures: (all: Some("wooden_spikes")))')
    shaped(["|||", "###"], {"|": "stick", "#": "oak_planks"}, "wooden_spikes", 4)
    for m, pt, _, ingot in METALS:
        blk(f'(name: "{m}_bricks", display_name: Some("Tijolos de {pt}"), hardness: 2.0, {pick}, tier: 1)')
        shapeless.append(f'        (ingredients: ["stone_bricks", "stone_bricks", "stone_bricks", "stone_bricks", "{ingot}"], result: "{m}_bricks", count: 4),')
    for g, pt, _, gem in GEMS:
        blk(f'(name: "{g}_gemspark", display_name: Some("Bloco Cintilante de {pt}"), light: 15, hardness: 0.3)')
        shaped(["GGG", "GXG", "GGG"], {"G": "glass", "X": gem}, f"{g}_gemspark", 8)
    blk(f'(name: "marble", display_name: Some("Mármore"), hardness: 1.5, {pick}, tier: 1)')
    blk(f'(name: "polished_marble", display_name: Some("Mármore Polido"), hardness: 1.5, {pick}, tier: 1)')
    blk(f'(name: "marble_bricks", display_name: Some("Tijolos de Mármore"), hardness: 1.5, {pick}, tier: 1)')
    blk(f'(name: "marble_pillar", display_name: Some("Coluna de Mármore"), hardness: 1.5, {pick}, tier: 1, '
        'textures: (side: Some("marble_pillar"), top: Some("marble_pillar_top")))')
    shapeless.append('        (ingredients: ["calcite", "diorite"], result: "marble", count: 2),')
    shaped(["##", "##"], {"#": "marble"}, "polished_marble", 4)
    shaped(["##", "##"], {"#": "polished_marble"}, "marble_bricks", 4)
    shaped(["#", "#"], {"#": "marble"}, "marble_pillar", 2)
    for base, bpt in (("marble", "Mármore"), ("polished_marble", "Mármore Polido"), ("marble_bricks", "Tijolos de Mármore")):
        stem = base.replace("_bricks", "_brick")
        t = f'textures: (all: Some("{base}"))'
        blk(f'(name: "{stem}_slab", display_name: Some("Laje de {bpt}"), shape: Slab, hardness: 1.5, {pick}, tier: 1, {t})')
        blk(f'(name: "{stem}_stairs", display_name: Some("Escada de {bpt}"), shape: Stairs, hardness: 1.5, {pick}, tier: 1, {t})')
        blk(f'(name: "{stem}_wall", display_name: Some("Muro de {bpt}"), shape: Wall, hardness: 1.5, {pick}, tier: 1, {t})')
        shaped(["###"], {"#": base}, f"{stem}_slab", 6)
        shaped(["#  ", "## ", "###"], {"#": base}, f"{stem}_stairs", 4)
        shaped(["###", "###"], {"#": base}, f"{stem}_wall", 6)
    blk(f'(name: "desert_fossil", display_name: Some("Fóssil do Deserto"), hardness: 1.2, {pick}, tier: 1)')
    blk(f'(name: "silt", display_name: Some("Silte"), hardness: 0.5, {shovel})')
    blk(f'(name: "slush", display_name: Some("Lama de Neve"), hardness: 0.5, {shovel})')
    shapeless.append('        (ingredients: ["sandstone", "bone_meal"], result: "desert_fossil"),')
    shapeless.append('        (ingredients: ["gravel", "clay_ball"], result: "silt", count: 2),')
    shapeless.append('        (ingredients: ["gravel", "snowball"], result: "slush", count: 2),')
    blk(f'(name: "snow_bricks", display_name: Some("Tijolos de Neve"), hardness: 0.6, {shovel})')
    blk(f'(name: "ice_bricks", display_name: Some("Tijolos de Gelo"), hardness: 0.8, {pick})')
    shaped(["##", "##"], {"#": "snow"}, "snow_bricks", 4)
    shaped(["##", "##"], {"#": "ice"}, "ice_bricks", 4)
    blk(f'(name: "sun_plate", display_name: Some("Placa Solar"), hardness: 2.0, {pick}, tier: 1)')
    shaped(["GS", "SG"], {"G": "gold_ingot", "S": "stone"}, "sun_plate", 4)
    blk('(name: "confetti_block", display_name: Some("Bloco de Confete"), hardness: 0.3)')
    shapeless.append('        (ingredients: ["paper", "red_dye", "yellow_dye", "blue_dye"], result: "confetti_block", count: 8),')
    blk('(name: "bubble_block", display_name: Some("Bolha"), transparency: Translucent, hardness: 0.1)')
    shapeless.append('        (ingredients: ["glass", "slime_ball"], result: "bubble_block", count: 8),')
    blk('(name: "rain_cloud", display_name: Some("Nuvem de Chuva"), hardness: 0.3)')
    blk('(name: "snow_cloud", display_name: Some("Nuvem de Neve"), hardness: 0.3)')
    shapeless.append('        (ingredients: ["cloud_block", "water_bucket"], result: "rain_cloud"),')
    shapeless.append('        (ingredients: ["cloud_block", "snowball"], result: "snow_cloud"),')


def main():
    import gen_block_models as gm
    pictures(gm)
    data(gm)
    text = open(BLOCKS, encoding="utf-8").read()
    text = re.sub(r"\n *// B6 start.*?// B6 end\n", "\n", text, flags=re.S)
    have = set(re.findall(r'\(name: "([^"]+)"', text))
    for f in glob.glob(os.path.join(ROOT, "packs", "vanilla", "*", "blocks.ron")):
        if os.sep + "models" + os.sep not in f:
            have |= set(re.findall(r'\(name: "([^"]+)"', open(f, encoding="utf-8").read()))
    keep = [b for b in blocks if re.search(r'name: "([^"]+)"', b).group(1) not in have]
    skipped = [re.search(r'name: "([^"]+)"', b).group(1) for b in blocks if b not in keep]
    block = "    // B6 start (tools/gen_blocos_b6.py)\n" + "\n".join(keep) + "\n    // B6 end\n"
    at = re.search(r"^\]\)", text, flags=re.M).start()
    open(BLOCKS, "w", encoding="utf-8").write(text[:at] + block + text[at:])
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B6 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b6.py:\n"
                "// edit it and run it again.\n(\n    shaped: [\n" + "\n".join(recipes) + "\n    ],\n    shapeless: [\n"
                + "\n".join(shapeless) + "\n    ],\n)\n")
    print(f"{len(keep)} blocks (already in the game: {skipped}), {len(recipes) + len(shapeless)} recipes")


if __name__ == "__main__":
    main()
