#!/usr/bin/env python3
"""Lote B5 of the block review (plan/blocos-analise.md): the newer copper pieces and the sulfur and cinnabar stones.

Copper: bars, chain, lantern stages, chest, torch and the golem statue, each in the four
stages of rust with waxed twins. The names follow the copper family (crates/game/src/copper.rs),
so they turn green in the open, take wax and scrape back with an axe like the other copper.
Rust is drawn as patches that spread with each stage, from our own copper pictures.

Sulfur and cinnabar: the raw stone, polished, bricks and chiseled, with slabs, stairs and walls.
Sulfur blocks are made from the sulfur the volcanoes give; cinnabar from sulfur and redstone
(a mercury ore in real life, red and heavy).

Writes pictures, blocks between `// B5 start` .. `// B5 end` in packs/vanilla/blocks.ron and
packs/vanilla/blocos/recipes_b5.ron. Run gen_block_models.py first.
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
from style import N, fbm, quantize, ramp, rgba, rng_for, soften, value_noise, normalize, light, blur, clusters  # noqa: E402

BLOCKS = b2.BLOCKS
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes_b5.ron")
RAW = {"sulfur": "sulfur_block", "cinnabar": "cinnabar"}
STAGES = ["", "exposed_", "weathered_", "oxidized_"]
STAGE_PT = ["", " Exposto", " Desgastado", " Oxidado"]
COVER = [0.0, 0.32, 0.62, 1.01]
COPPER = "#c46e48"
PATINA = "#4fa88a"


def metal(img, glow=200):
    """Copper pixels: warm and not glowing (keeps flames and glass)."""
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    lum = r * 0.3 + g * 0.59 + b * 0.11
    return (img[..., 3] > 0) & (r > b + 18) & (lum < glow)


def coppered(img, keep=0.3):
    """A grey or wooden picture remade in copper (by lightness)."""
    out = img.copy()
    t = b2.tint(img, COPPER, keep)
    a = img[..., 3] > 0
    out[a] = t[a]
    return out


def patina(img, stage, seed, glow=200):
    """Green patches over the copper, more each stage; their edges a step lighter."""
    if stage == 0:
        return img
    r = rng_for(seed)
    n = normalize(value_noise(r, 16) * 0.6 + value_noise(r, 8) * 0.3 + value_noise(r, 4) * 0.1)
    green = b2.tint(img, PATINA, 0.25)
    m = metal(img, glow)
    out = img.copy()
    inside = m & (n < COVER[stage])
    out[inside] = green[inside]
    edge = m & (n >= COVER[stage]) & (n < COVER[stage] + 0.05)
    out[edge, :3] = np.clip(out[edge, :3] * 0.8 + b2.hexc("#8fbfa0") * 0.2, 0, 255)
    return out


def staged(src_img, name):
    """Save the four stages of a copper picture: name, exposed_name, ..."""
    for k, s in enumerate(STAGES):
        b2.save(s + name, patina(src_img, k, s + name))


def copper_pictures():
    staged(coppered(b2.load("iron_bars")), "copper_bars")
    staged(coppered(b2.load("chain")), "copper_chain")
    # Lantern: our copper lantern's frame rusts; the glass and flame stay.
    lan, top = b2.load("copper_lantern"), b2.load("copper_lantern_top")
    for k, s in enumerate(STAGES[1:], 1):
        b2.save(s + "copper_lantern", patina(lan, k, s + "copper_lantern", glow=128))
        b2.save(s + "copper_lantern_top", patina(top, k, s + "copper_lantern_top", glow=128))
    # Chest: the wooden chest clad in copper plates; its latch stays dark iron.
    for part in ("top", "side", "front"):
        img = b2.load("chest_" + part)
        wood = (img[..., 0] > img[..., 2] + 20) & (img[..., 3] > 0)
        c = img.copy()
        t = b2.tint(img, COPPER, 0.3)
        c[wood] = t[wood]
        # Rivets in the corners of each plate.
        for y in (6, 57):
            for x in (6, 57):
                c[y:y + 2, x:x + 2, :3] = b2.hexc("#f0b890")
        staged(c, "copper_chest_" + part)
    # Torch: the same stick, a green copper flame.
    t = b2.load("torch")
    r, g, b = t[..., 0], t[..., 1], t[..., 2]
    flame = (t[..., 3] > 0) & (r > 150) & (g > 110)
    t2 = t.copy()
    t2[flame] = b2.tint(t, "#7af08a", 0.2)[flame]
    b2.save("copper_torch", t2)


def stone(name, base, vein):
    """A raw stone: grainy layers with brighter crystal clusters."""
    rng = rng_for(name)
    pal = ramp(base, 12, 0.36, 0.10)
    f = fbm(rng, ((32, 0.4), (16, 0.3), (8, 0.2), (4, 0.1)))
    layers = np.sin((np.arange(N)[:, None] + f * 18) / 5.5) * 0.08
    c = clusters(rng, 4)
    t = 0.5 + (f - 0.5) * 0.35 + layers + light(blur(f, 1) * 3) * 0.6
    img = rgba(quantize(np.clip(t, 0, 1), pal, 0.3, rng))
    spots = c > 0.78
    img[spots, :3] = quantize(np.clip(0.75 + (c[spots] - 0.78) * 1.5, 0, 1), ramp(vein, 6, 0.3, 0.1))
    return soften(img, "natural")


def stone_pictures():
    for n, base, vein in (("sulfur", "#c8b02a", "#f4e470"), ("cinnabar", "#9a2a26", "#e0545a")):
        b2.save(RAW[n], stone(n, base, vein))
        b2.save(f"polished_{n}", t1.polished(f"polished_{n}", RAW[n]))
        import blocks as tb
        b2.save(f"{n}_bricks", tb.stone_bricks(f"{n}_bricks", base))
        b2.save(f"chiseled_{n}", t1.chiseled(f"chiseled_{n}", RAW[n], "sun" if n == "sulfur" else "diamond"))


blocks = []
recipes = []
shapeless = []


def blk(line):
    blocks.append("    " + line + ",")


def shaped(pattern, key, result, count=1):
    keys = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    pats = ", ".join(f'"{p}"' for p in pattern)
    recipes.append(f"        (pattern: [{pats}], key: {{{keys}}}, result: \"{result}\", count: {count}),")


def data():
    pick = "tool: Some(Pickaxe)"
    for k, s in enumerate(STAGES):
        for w, wpt in (("", ""), ("waxed_", " Encerado")):
            pt = STAGE_PT[k] + wpt
            blk(f'(name: "{w}{s}copper_bars", display_name: Some("Grade de Cobre{pt}"), shape: Pane, transparency: Cutout, hardness: 5.0, '
                f'{pick}, textures: (all: Some("{s}copper_bars")))')
            blk(f'(name: "{w}{s}copper_chain", display_name: Some("Corrente de Cobre{pt}"), shape: Chain, transparency: Cutout, hardness: 5.0, '
                f'{pick}, textures: (all: Some("{s}copper_chain")))')
            if k or w:
                blk(f'(name: "{w}{s}copper_lantern", display_name: Some("Lanterna de Cobre{pt}"), shape: Lantern, transparency: Cutout, light: 15, '
                    f'hardness: 1.5, {pick}, textures: (side: Some("{s}copper_lantern"), top: Some("{s}copper_lantern_top"), '
                    f'bottom: Some("{s}copper_lantern_top")))')
            blk(f'(name: "{w}{s}copper_chest", display_name: Some("Baú de Cobre{pt}"), hardness: 3.0, {pick}, '
                f'textures: (top: Some("{s}copper_chest_top"), side: Some("{s}copper_chest_side"), front: Some("{s}copper_chest_front"), '
                f'bottom: Some("{s}copper_chest_top")))')
            blk(f'(name: "{w}{s}copper_golem_statue", display_name: Some("Estátua de Golem de Cobre{pt}"), transparency: Cutout, hardness: 3.0, '
                f'{pick}, textures: (all: Some("{s}copper_golem_statue")))')
            if w:
                for piece in ("copper_bars", "copper_chain", "copper_lantern", "copper_chest", "copper_golem_statue"):
                    shapeless.append(f'        (ingredients: ["{s}{piece}", "honeycomb"], result: "waxed_{s}{piece}"),')
    blk(f'(name: "copper_torch", display_name: Some("Tocha de Cobre"), shape: Torch, transparency: Cutout, solid: false, light: 14, hardness: 0.0)')
    shaped(["###", "###"], {"#": "copper_ingot"}, "copper_bars", 16)
    shaped(["I", "I", "I"], {"I": "copper_ingot"}, "copper_chain", 3)
    shaped(["III", "ICI", "III"], {"I": "copper_ingot", "C": "chest"}, "copper_chest")
    shaped(["I", "C", "|"], {"I": "copper_ingot", "C": "coal", "|": "stick"}, "copper_torch", 4)
    shaped(["I", "T", "I"], {"I": "copper_ingot", "T": "copper_torch"}, "copper_lantern", 2)
    shaped(["C", "B"], {"C": "carved_pumpkin", "B": "copper_block"}, "copper_golem_statue")

    for n, pt, h in (("sulfur", "Enxofre", 1.5), ("cinnabar", "Cinábrio", 1.5)):
        raw = RAW[n]  # the item "sulfur" already exists, so its stone is "sulfur_block"
        blk(f'(name: "{raw}", display_name: Some("{"Bloco de " if raw != n else ""}{pt}"), hardness: {h}, {pick}, tier: 1)')
        blk(f'(name: "polished_{n}", display_name: Some("{pt} Polido"), hardness: {h}, {pick}, tier: 1)')
        blk(f'(name: "{n}_bricks", display_name: Some("Tijolos de {pt}"), hardness: {h}, {pick}, tier: 1)')
        blk(f'(name: "chiseled_{n}", display_name: Some("{pt} Talhado"), hardness: {h}, {pick}, tier: 1)')
        for base, bpt in ((raw, pt), (f"polished_{n}", f"{pt} Polido"), (f"{n}_bricks", f"Tijolos de {pt}")):
            stem = base.replace("_bricks", "_brick").replace("_block", "")
            t = f'textures: (all: Some("{base}"))'
            blk(f'(name: "{stem}_slab", display_name: Some("Laje de {bpt}"), shape: Slab, hardness: {h}, {pick}, tier: 1, {t})')
            blk(f'(name: "{stem}_stairs", display_name: Some("Escada de {bpt}"), shape: Stairs, hardness: {h}, {pick}, tier: 1, {t})')
            blk(f'(name: "{stem}_wall", display_name: Some("Muro de {bpt}"), shape: Wall, hardness: {h}, {pick}, tier: 1, {t})')
            shaped(["###"], {"#": base}, f"{stem}_slab", 6)
            shaped(["#  ", "## ", "###"], {"#": base}, f"{stem}_stairs", 4)
            shaped(["###", "###"], {"#": base}, f"{stem}_wall", 6)
        shaped(["##", "##"], {"#": raw}, f"polished_{n}", 4)
        shaped(["##", "##"], {"#": f"polished_{n}"}, f"{n}_bricks", 4)
        shaped(["#", "#"], {"#": f"{n}_brick_slab"}, f"chiseled_{n}")
    shaped(["SS", "SS"], {"S": "sulfur"}, "sulfur_block")
    shapeless.append('        (ingredients: ["sulfur", "sulfur", "redstone", "redstone", "stone"], result: "cinnabar", count: 2),')


def main():
    import gen_block_models as gm
    copper_pictures()
    stone_pictures()
    for k, s in enumerate(STAGES):
        b2.iso_icon(gm.MODELS[s + "copper_golem_statue"], s + "copper_golem_statue")
    data()
    text = open(BLOCKS, encoding="utf-8").read()
    text = re.sub(r"\n *// B5 start.*?// B5 end\n", "\n", text, flags=re.S)
    have = set(re.findall(r'\(name: "([^"]+)"', text))
    for f in glob.glob(os.path.join(ROOT, "packs", "vanilla", "*", "blocks.ron")):
        if os.sep + "models" + os.sep not in f:
            have |= set(re.findall(r'\(name: "([^"]+)"', open(f, encoding="utf-8").read()))
    keep = [b for b in blocks if re.search(r'name: "([^"]+)"', b).group(1) not in have]
    block = "    // B5 start (tools/gen_blocos_b5.py)\n" + "\n".join(keep) + "\n    // B5 end\n"
    at = re.search(r"^\]\)", text, flags=re.M).start()
    open(BLOCKS, "w", encoding="utf-8").write(text[:at] + block + text[at:])
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B5 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b5.py:\n"
                "// edit it and run it again.\n(\n    shaped: [\n" + "\n".join(recipes) + "\n    ],\n    shapeless: [\n"
                + "\n".join(shapeless) + "\n    ],\n)\n")
    print(f"{len(keep)} blocks ({len(blocks) - len(keep)} already in the game), {len(recipes) + len(shapeless)} recipes")


if __name__ == "__main__":
    main()
