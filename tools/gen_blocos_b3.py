#!/usr/bin/env python3
"""Lote B3 of the block review (plan/blocos-analise.md): plants and nature blocks the game lacked.

Every plant here gets a body from crates/voxel/src/flora.rs (no crossed pictures); its picture
below is what the game paints the body's parts from, and the item icon. Pictures are made from
our own plant pictures, recoloured and reworked. Amethyst buds are 3D models
(tools/gen_block_models.py), their icons drawn from the boxes like lote B2.

Writes the pictures, the blocks into packs/vanilla/blocks.ron (`// B3 start` .. `// B3 end`)
and recipes into packs/vanilla/blocos/recipes_b3.ron. Run gen_block_models.py first.
"""

import glob
import os
import re
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))
import gen_blocos_b2 as b2  # noqa: E402  (load/save/tint/iso_icon helpers)

BLOCKS = b2.BLOCKS
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes_b3.ron")


def recolour(src, dst, colour, keep=0.3, where=None):
    img = b2.load(src)
    t = b2.tint(img, colour, keep)
    if where is not None:
        m = where(img)
        img[m] = t[m]
    else:
        img = t
    b2.save(dst, img)
    return img


def green(img):
    """Leaf and stem pixels: green leads red and blue."""
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    return (g > r * 1.05) & (g > b * 1.05) & (img[..., 3] > 0)


def not_green(img):
    return ~green(img) & (img[..., 3] > 0)


def speckle(name, colour, count, seed):
    img = b2.load(name)
    r = b2.rng(seed)
    a = np.argwhere(img[..., 3] > 0)
    for y, x in a[r.choice(len(a), min(count, len(a)), replace=False)]:
        img[max(y - 1, 0):y + 1, max(x - 1, 0):x + 1, :3] = b2.hexc(colour)
    b2.save(name, img)


def pictures():
    # Grasses and ferns: the grass picture itself (tinted by the biome like short grass).
    grass = b2.load("short_grass")
    for n in ("tall_grass", "fern", "large_fern", "bush"):
        b2.save(n, grass)
    recolour("short_grass", "firefly_bush", "#4f6a2a", 0.4)
    speckle("firefly_bush", "#f0ff70", 10, "firefly_bush")
    recolour("short_grass", "short_dry_grass", "#c8a860", 0.35)
    recolour("short_grass", "tall_dry_grass", "#b89850", 0.35)
    recolour("short_grass", "crimson_roots", "#a8243a", 0.3)
    recolour("short_grass", "warped_roots", "#1fa08c", 0.3)
    recolour("short_grass", "nether_sprouts", "#2ab89c", 0.25)
    # Flowers: our flowers with new petal colours, the leaves kept.
    recolour("allium", "lilac", "#c08ad8", 0.3, not_green)
    recolour("allium", "peony", "#f0a8c8", 0.3, not_green)
    recolour("poppy", "rose_bush", "#c8202c", 0.3, not_green)
    recolour("poppy", "wither_rose", "#2a2026", 0.2)
    recolour("pink_tulip", "cactus_flower", "#f07aa8", 0.3, not_green)
    recolour("azure_bluet", "wildflowers", "#f4d040", 0.3, not_green)
    # Nether fungi: the mushroom in crimson and in warped teal.
    recolour("red_mushroom", "crimson_fungus", "#b02232", 0.35)
    recolour("red_mushroom", "warped_fungus", "#1aa08a", 0.35)
    # Vines. (Hanging roots and the spore blossom are 3D models from the caves work, PR #122.)
    recolour("vine", "weeping_vines", "#a4202e", 0.3)
    recolour("vine", "twisting_vines", "#18a890", 0.3)
    # Coral fans, and dead corals and fans: grey and chalky.
    for c, col in (("tube", "#3050d0"), ("brain", "#d050a0"), ("bubble", "#a020b0"), ("fire", "#d02a2a"), ("horn", "#d8c030")):
        recolour(f"{c}_coral", f"{c}_coral_fan", col, 0.45)
        for src, dst in ((f"{c}_coral", f"dead_{c}_coral"), (f"{c}_coral_fan", f"dead_{c}_coral_fan")):
            recolour(src, dst, "#a49c94", 0.25)
    # Powder snow: snow, finer and a touch blue.
    import gen_blocos_b6 as b6
    b2.save("powder_snow", b6.grains("powder_snow", "#eaf2fa", 0.12))


blocks = []
shapeless = []


def plant(name, pt, tint=False, light=0, drops=None, wet=False):
    extra = ", tint: true" if tint else ""
    extra += ", waterlogged: true" if wet else ""
    extra += f", light: {light}" if light else ""
    extra += f", drops: Some([{drops}])" if drops is not None else ""
    blocks.append(f'    (name: "{name}", display_name: Some("{pt}"), shape: Cross, transparency: Cutout, solid: false, hardness: 0.0{extra}),')


def data():
    plant("tall_grass", "Grama Alta", tint=True, drops='(item: "wheat_seeds", chance: 0.125)')
    plant("fern", "Samambaia", tint=True, drops='(item: "wheat_seeds", chance: 0.125)')
    plant("large_fern", "Samambaia Grande", tint=True, drops='(item: "wheat_seeds", chance: 0.125)')
    plant("bush", "Arbusto", tint=True)
    plant("firefly_bush", "Arbusto de Vaga-lumes", light=2)
    plant("short_dry_grass", "Grama Seca Baixa")
    plant("tall_dry_grass", "Grama Seca Alta")
    plant("lilac", "Lilás")
    plant("peony", "Peônia")
    plant("rose_bush", "Roseira")
    plant("wither_rose", "Rosa de Wither")
    plant("cactus_flower", "Flor de Cacto")
    plant("wildflowers", "Flores Silvestres")
    plant("crimson_fungus", "Fungo Carmesim")
    plant("warped_fungus", "Fungo Distorcido")
    plant("crimson_roots", "Raízes Carmesins")
    plant("warped_roots", "Raízes Distorcidas")
    plant("nether_sprouts", "Brotos do Nether", drops="")
    plant("weeping_vines", "Cipós Chorões", drops="")
    plant("twisting_vines", "Cipós Retorcidos", drops="")
    for c, pt in (("tube", "Tubo"), ("brain", "Cérebro"), ("bubble", "Bolha"), ("fire", "Fogo"), ("horn", "Chifre")):
        plant(f"{c}_coral_fan", f"Leque de Coral de {pt}", wet=True)
        plant(f"dead_{c}_coral", f"Coral de {pt} Morto")
        plant(f"dead_{c}_coral_fan", f"Leque de Coral de {pt} Morto")
    for size, pt, light in (("small", "Pequeno", 1), ("medium", "Médio", 2), ("large", "Grande", 4)):
        blocks.append(f'    (name: "{size}_amethyst_bud", display_name: Some("Broto de Ametista {pt}"), transparency: Cutout, solid: false, '
                      f'hardness: 1.5, tool: Some(Pickaxe), light: {light}, drops: Some([]), textures: (all: Some("{size}_amethyst_bud"))),')
    blocks.append('    (name: "powder_snow", display_name: Some("Neve em Pó"), transparency: Translucent, solid: false, hardness: 0.25, '
                  'tool: Some(Shovel), drops: Some([]), textures: (all: Some("powder_snow"))),')
    for f, dye, n in (("lilac", "magenta_dye", 2), ("peony", "pink_dye", 2), ("rose_bush", "red_dye", 2), ("wither_rose", "black_dye", 1),
                      ("cactus_flower", "pink_dye", 1), ("wildflowers", "yellow_dye", 1)):
        shapeless.append(f'        (ingredients: ["{f}"], result: "{dye}", count: {n}),')


def main():
    import gen_block_models as gm
    pictures()
    for name in ("small_amethyst_bud", "medium_amethyst_bud", "large_amethyst_bud"):
        b2.iso_icon(gm.MODELS[name], name)
    data()
    text = open(BLOCKS, encoding="utf-8").read()
    text = re.sub(r"\n *// B3 start.*?// B3 end\n", "\n", text, flags=re.S)
    have = set(re.findall(r'\(name: "([^"]+)"', text))
    for f in glob.glob(os.path.join(ROOT, "packs", "vanilla", "*", "blocks.ron")):
        if os.sep + "models" + os.sep not in f:
            have |= set(re.findall(r'\(name: "([^"]+)"', open(f, encoding="utf-8").read()))
    keep = [b for b in blocks if re.search(r'name: "([^"]+)"', b).group(1) not in have]
    block = "    // B3 start (tools/gen_blocos_b3.py)\n" + "\n".join(keep) + "\n    // B3 end\n"
    at = re.search(r"^\]\)", text, flags=re.M).start()
    open(BLOCKS, "w", encoding="utf-8").write(text[:at] + block + text[at:])
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B3 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b3.py:\n"
                "// edit it and run it again.\n(\n    shapeless: [\n" + "\n".join(shapeless) + "\n    ],\n)\n")
    print(f"{len(keep)} blocks ({len(blocks) - len(keep)} already in the game), {len(shapeless)} recipes")


if __name__ == "__main__":
    main()
