#!/usr/bin/env python3
"""Lote B4 of the block review (plan/blocos-analise.md): the bamboo wood set, hanging signs and mob heads.

Pictures are our own, drawn here at 64x64 in the chunky pixel style of the pack (a 16-pixel
design scaled four times, with a little grain per pixel). Hanging signs are 3D models made in
tools/gen_block_models.py; their item icons are drawn from the boxes like lote B2.

Writes the pictures, the blocks into packs/vanilla/blocks.ron (`// B4 start` .. `// B4 end`)
and recipes into packs/vanilla/blocos/recipes_b4.ron. Run gen_block_models.py first.
"""

import glob
import os
import re
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))
import gen_blocos_b2 as b2  # noqa: E402  (load/save/hexc/rng/iso_icon helpers)

BLOCKS = b2.BLOCKS
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes_b4.ron")
N = 64


def up(design, palette, name, grain=0.025):
    """A 16x16 grid of palette keys ('.' is clear) scaled to 64x64, each big pixel with a little grain."""
    r = b2.rng(name)
    img = np.zeros((N, N, 4))
    for y, row in enumerate(design):
        for x, ch in enumerate(row):
            if ch == ".":
                continue
            c = b2.hexc(palette[ch])
            block = c[None, None, :] * (1 + r.normal(0, grain, (4, 4, 1)))
            img[y * 4:y * 4 + 4, x * 4:x * 4 + 4, :3] = block
            img[y * 4:y * 4 + 4, x * 4:x * 4 + 4, 3] = 255
    return img


def ramp(cols, t):
    """Colour from a list of hex colours at 0..1."""
    cs = [b2.hexc(c) for c in cols]
    t = min(max(t, 0.0), 0.999) * (len(cs) - 1)
    i = int(t)
    return cs[i] * (1 - (t - i)) + cs[i + 1] * (t - i)


def stalks(name, cols, node, wide=16):
    """Bamboo seen from the side: round stalks shaded across, with a node ring at a different height each."""
    r = b2.rng(name)
    img = np.zeros((N, N, 4))
    img[..., 3] = 255
    for s in range(N // wide):
        x0 = s * wide
        ny = [int(r.integers(4, 28)), 0]
        ny[1] = ny[0] + 32
        for x in range(wide):
            t = x / (wide - 1)
            shade = 0.62 + 0.55 * np.sin(np.pi * t) ** 0.7 - 0.18 * t
            base = ramp(cols, shade / 1.2)
            for y in range(N):
                c = base * (1 + r.normal(0, 0.03))
                if any(abs(y - n) <= 1 for n in ny):
                    c = b2.hexc(node) * (0.9 + 0.2 * np.sin(np.pi * t))
                elif any(y - n == 2 for n in ny):
                    c = c * 1.12
                img[y, x0 + x, :3] = c
        img[:, x0, :3] *= 0.55
    b2.save(name, img)


def bundle(name, cols, rim):
    """Bamboo block end: a bundle of cut stalks, rings round a pale hollow middle."""
    img = np.zeros((N, N, 4))
    img[..., :3] = b2.hexc(cols[0]) * 0.45
    img[..., 3] = 255
    yy, xx = np.mgrid[0:N, 0:N]
    for cx, cy, rad in ((16, 16, 14), (48, 16, 14), (16, 48, 14), (48, 48, 14), (32, 32, 7), (32, 2, 6), (2, 32, 6), (62, 32, 6), (32, 62, 6)):
        d = np.hypot(xx - cx + 0.5, yy - cy + 0.5)
        ring = d < rad
        img[ring, :3] = b2.hexc(rim)
        inner = d < rad - 3
        t = (d / max(rad - 3, 1))
        for c in range(3):
            img[..., c] = np.where(inner, ramp(cols, 0.9)[c] * (0.85 + 0.25 * (1 - t)), img[..., c])
        hollow = d < rad * 0.38
        img[hollow, :3] = b2.hexc(cols[0]) * 0.6
    b2.save(name, np.round(img / 6) * 6)


BAMBOO = ["#4f6a14", "#6f8f1c", "#93b02a", "#b8c845"]
STRIPPED = ["#9a8228", "#c4a83a", "#dcc458", "#efdc7c"]
PLANK = ["#a8902c", "#c8ac3c", "#dcc456", "#ecd874"]


def planks():
    # Planks: split bamboo laid in vertical slats, each with a node line and a dark seam.
    r = b2.rng("bamboo_planks")
    img = np.zeros((N, N, 4))
    img[..., 3] = 255
    for s in range(8):
        x0 = s * 8
        node = int(r.integers(0, 64))
        for x in range(8):
            t = x / 7
            base = ramp(PLANK, 0.25 + 0.65 * np.sin(np.pi * t))
            for y in range(N):
                c = base * (1 + r.normal(0, 0.035))
                if (y - node) % 32 in (0, 1):
                    c = ramp(PLANK, 0.05)
                img[y, x0 + x, :3] = c
        img[:, x0, :3] = b2.hexc("#6a5418")
    img[31:33, :, :3] *= 0.72
    b2.save("bamboo_planks", img)
    # Mosaic: small tiles, slats running across and down by turns, sunk joints.
    m = np.zeros((N, N, 4))
    m[..., 3] = 255
    for ty in range(4):
        for tx in range(4):
            across = (tx + ty) % 2 == 0
            for y in range(16):
                for x in range(16):
                    k = (y if across else x) % 4
                    c = ramp(PLANK, [0.35, 0.8, 0.95, 0.6][k]) * (1 + r.normal(0, 0.03))
                    if x == 0 or y == 0:
                        c = b2.hexc("#5e4a14")
                    elif x == 15 or y == 15:
                        c = ramp(PLANK, 0.15)
                    m[ty * 16 + y, tx * 16 + x, :3] = c
    b2.save("bamboo_mosaic", m)


def door_and_trapdoor():
    pal = {"f": "#7a6420", "s": "#c8ac3c", "S": "#e0c860", "d": "#9a8228", "k": "#3a3a40", "n": "#6a5418"}
    lower = [
        "ffffffffffffffff", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fnnnnnnnnnnnnnnf",
        "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf",
        "fffffffffffffkkf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fnnnnnnnnnnnnnnf",
        "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "ffffffffffffffff",
    ]
    upper = [
        "ffffffffffffffff", "f..f..f..f..f..f", "f..f..f..f..f..f", "f..f..f..f..f..f",
        "f..f..f..f..f..f", "ffffffffffffffff", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf",
        "fnnnnnnnnnnnnnnf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf", "fsSdsSdsSdsSdsSf",
        "fsSdsSdsSdsSdsSf", "fnnnnnnnnnnnnnnf", "fsSdsSdsSdsSdsSf", "ffffffffffffffff",
    ]
    trap = [
        "ffffffffffffffff", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf",
        "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "ffffffffffffffff",
        "ffffffffffffffff", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf",
        "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "fsS.sS.ff.sS.sSf", "ffffffffffffffff",
    ]
    b2.save("bamboo_door_lower", up(lower, pal, "bamboo_door_lower"))
    b2.save("bamboo_door_upper", up(upper, pal, "bamboo_door_upper"))
    b2.save("bamboo_trapdoor", up(trap, pal, "bamboo_trapdoor"))


# Heads: the whole picture is one face of the 8-pixel head box (like the wither skull).
HEADS = {
    "skeleton_skull": (
        {"b": "#d8d4c8", "B": "#c2bdb0", "w": "#ece9e0", "k": "#1e1c1c", "g": "#8e897e"},
        ["wwbbwwbbwwbbwwbb", "wbbwwbbBbwwbbwbw", "bwbbwbbwbbwbbwbb", "bbwbbBbbwbbwbBbw",
         "bwbbwbbbbwbbwbbb", "bkkkkbbbbbbkkkkb", "bkkkkbbbbbbkkkkb", "bkkkkbbwwbbkkkkb",
         "bkkkkbbwwbbkkkkb", "bbbbbbbkkbbbbbbb", "bBbbbbbkkbbbbBbb", "bbgbgbgbgbgbgbbb",
         "bbwkwkwkwkwkwbbb", "bbgbgbgbgbgbgbbb", "BbbbbbbbbbbbbbbB", "BBbbbbbbbbbbbbBB"],
        ["wbbwbbwbbwbbwbbw"] * 16,
        {"b": "#d8d4c8", "w": "#ece9e0"},
    ),
    "zombie_head": (
        {"g": "#4e8a3a", "G": "#3f7430", "h": "#2f5a24", "k": "#1a2418", "e": "#0e3a3e", "m": "#2a4a20", "l": "#5c9c46"},
        ["hhhhhhhhhhhhhhhh", "hhhhhhhhhhhhhhhh", "hhGhhhhhhhGhhhhh", "gglgggggggglgggg",
         "ggggglgggggggglg", "gGggggggggggGggg", "geekgggggggkeegg", "geekgggggggkeegg",
         "ggggggGmmGgggggg", "gggggggmmggggglg", "glggGggggggGgggg", "gggmmmmmmmmmmggg",
         "gggmkkkkkkkkmggg", "gGgggggggggggGgg", "ggglgggGgggggggg", "gggggggggglggggg"],
        None,
        {"h": "#2f5a24", "H": "#264c1e", "G": "#3f7430"},
    ),
    "creeper_head": (
        {"g": "#4ea83a", "G": "#3c8c2e", "l": "#7cc860", "w": "#b8e0a8", "k": "#101410", "K": "#203020"},
        ["glgGgwglgGglglgw", "lgGglgGglggGlglg", "gGlglggwglGglgGl", "glggGlgglglggwgg",
         "gkkkkggGglgkkkkg", "lkkkkglglgGkkkkl", "gkkkkgGlgglkkkkg", "lkkkkglgwglkkkKg",
         "gGlglgkkkkglgGlg", "glgGgkkkkkkglglw", "lgwlgkkkkkkgGgGg", "gGglgkkKkkkglglg",
         "lglgGkkgglkkgwlg", "gwgGlkkglgkKglgG", "glgglgGgwglgglgl", "lGlgwglgGglwgGlg"],
        None,
        {"g": "#4ea83a", "G": "#3c8c2e", "l": "#7cc860", "w": "#b8e0a8"},
    ),
    "piglin_head": (
        {"p": "#e0928a", "P": "#c87870", "d": "#a85a52", "k": "#201414", "w": "#f2e8d0", "n": "#f0aca2", "N": "#6a2a26", "e": "#e8d070"},
        ["pPppppPppppPpppp", "pppPpppppPpppPpp", "ppppppppppppppPp", "pPpppPppppppPppp",
         "pwkppppppppppkwp", "pwkpppPpppPppkwp", "ppppppppppppppPp", "ppPpnnnnnnnnpppp",
         "pppPnNnnnnNnPppp", "ppppnNnnnnNnpppp", "pPppnnnnnnnnpPpp", "pppppppppppppppp",
         "pewpddddddddpwep", "pewpPpppppPppwep", "pppppppPppppppPp", "PppPpppppppPpppP"],
        None,
        {"p": "#e0928a", "P": "#c87870", "d": "#a85a52"},
    ),
    "player_head": (
        {"h": "#4a2c18", "H": "#3a2210", "s": "#c89070", "S": "#b07a5c", "w": "#f0f0f0", "e": "#2a5ab0", "m": "#7a4030", "b": "#3a2210"},
        ["hhhhhhhhhhhhhhhh", "hhHhhhhhhHhhhhhh", "hhhhhhHhhhhhhHhh", "hsshhhhhhhhhhssh",
         "sssssssssssssSss", "sSssssssssssssss", "sswesssssssweSss", "sswessssssswesss",
         "sssssssSSsssssss", "ssssssSssSssssss", "sssssssssssssSss", "sssbmmmmmmmbssss",
         "ssssbbbbbbbbssss", "sSssssssssssssss", "sssssssssssSssss", "ssssssssssssssss"],
        None,
        {"h": "#4a2c18", "H": "#3a2210"},
    ),
}


def heads():
    for name, (pal, face, top, tpal) in HEADS.items():
        b2.save(name, up(face, pal, name))
        r = b2.rng(name + "_top")
        keys = list(tpal)
        grid = top or ["".join(keys[int(r.integers(0, len(keys)))] for _ in range(16)) for _ in range(16)]
        b2.save(name + "_top", up(grid, tpal, name + "_top"))


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


def data(hanging):
    axe = "tool: Some(Axe)"
    P = 'textures: (all: Some("bamboo_planks"))'
    M = 'textures: (all: Some("bamboo_mosaic"))'
    blk(f'(name: "bamboo_block", display_name: Some("Bloco de Bambu"), hardness: 2.0, {axe}, textures: (side: Some("bamboo_block"), top: Some("bamboo_block_top")))')
    blk(f'(name: "stripped_bamboo_block", display_name: Some("Bloco de Bambu Descascado"), hardness: 2.0, {axe}, '
        'textures: (side: Some("stripped_bamboo_block"), top: Some("stripped_bamboo_block_top")))')
    blk(f'(name: "bamboo_planks", display_name: Some("Tábuas de Bambu"), hardness: 2.0, {axe})')
    blk(f'(name: "bamboo_mosaic", display_name: Some("Mosaico de Bambu"), hardness: 2.0, {axe})')
    blk(f'(name: "bamboo_slab", display_name: Some("Laje de Bambu"), hardness: 2.0, shape: Slab, {axe}, {P})')
    blk(f'(name: "bamboo_stairs", display_name: Some("Escada de Bambu"), shape: Stairs, hardness: 2.0, {axe}, {P})')
    blk(f'(name: "bamboo_mosaic_slab", display_name: Some("Laje de Mosaico de Bambu"), hardness: 2.0, shape: Slab, {axe}, {M})')
    blk(f'(name: "bamboo_mosaic_stairs", display_name: Some("Escada de Mosaico de Bambu"), shape: Stairs, hardness: 2.0, {axe}, {M})')
    blk(f'(name: "bamboo_fence", display_name: Some("Cerca de Bambu"), shape: Fence, hardness: 2.0, {axe}, {P})')
    blk(f'(name: "bamboo_fence_gate", display_name: Some("Portão de Bambu"), shape: FenceGate, hardness: 2.0, {axe}, {P})')
    blk(f'(name: "bamboo_door", display_name: Some("Porta de Bambu"), shape: Door, transparency: Cutout, hardness: 3.0, {axe}, '
        'textures: (side: Some("bamboo_door_lower"), top: Some("bamboo_door_upper")))')
    blk(f'(name: "bamboo_trapdoor", display_name: Some("Alçapão de Bambu"), shape: Trapdoor, transparency: Cutout, hardness: 3.0, {axe}, '
        'textures: (all: Some("bamboo_trapdoor")))')
    blk(f'(name: "bamboo_button", display_name: Some("Botão de Bambu"), shape: Button, solid: false, hardness: 0.5, {axe}, {P})')
    blk(f'(name: "bamboo_pressure_plate", display_name: Some("Placa de Pressão de Bambu"), shape: Plate, solid: false, hardness: 0.5, {axe}, {P})')
    blk(f'(name: "bamboo_sign", display_name: Some("Placa de Bambu"), shape: Sign, solid: false, transparency: Cutout, hardness: 1.0, {axe}, {P})')
    for w in hanging:
        blk(f'(name: "{w}_hanging_sign", display_name: Some("Placa Pendurada de {WOOD_PT[w]}"), transparency: Cutout, solid: false, '
            f'hardness: 1.0, {axe}, textures: (all: Some("{w}_hanging_sign")))')
    for name, pt in (("skeleton_skull", "Crânio de Esqueleto"), ("zombie_head", "Cabeça de Zumbi"), ("creeper_head", "Cabeça de Creeper"),
                     ("piglin_head", "Cabeça de Piglin"), ("player_head", "Cabeça de Jogador")):
        blk(f'(name: "{name}", display_name: Some("{pt}"), shape: Skull, transparency: Cutout, hardness: 1.0, '
            f'textures: (side: Some("{name}"), top: Some("{name}_top"), bottom: Some("{name}_top")))')

    shaped(["###", "###", "###"], {"#": "bamboo"}, "bamboo_block")
    shapeless.append('        (ingredients: ["bamboo_block"], result: "bamboo_planks", count: 2),')
    shapeless.append('        (ingredients: ["stripped_bamboo_block"], result: "bamboo_planks", count: 2),')
    shapeless.append('        (ingredients: ["bamboo_planks"], result: "bamboo_button", count: 1),')
    shaped(["#", "#"], {"#": "bamboo_slab"}, "bamboo_mosaic")
    shaped(["###"], {"#": "bamboo_planks"}, "bamboo_slab", 6)
    shaped(["#  ", "## ", "###"], {"#": "bamboo_planks"}, "bamboo_stairs", 4)
    shaped(["###"], {"#": "bamboo_mosaic"}, "bamboo_mosaic_slab", 6)
    shaped(["#  ", "## ", "###"], {"#": "bamboo_mosaic"}, "bamboo_mosaic_stairs", 4)
    shaped(["#|#", "#|#"], {"#": "bamboo_planks", "|": "stick"}, "bamboo_fence", 3)
    shaped(["|#|", "|#|"], {"#": "bamboo_planks", "|": "stick"}, "bamboo_fence_gate")
    shaped(["##", "##", "##"], {"#": "bamboo_planks"}, "bamboo_door", 3)
    shaped(["###", "###"], {"#": "bamboo_planks"}, "bamboo_trapdoor", 2)
    shaped(["##"], {"#": "bamboo_planks"}, "bamboo_pressure_plate")
    shaped(["###", "###", " | "], {"#": "bamboo_planks", "|": "stick"}, "bamboo_sign", 3)
    for w in hanging:
        t = "stripped_bamboo_block" if w == "bamboo" else f"stripped_{w}_stem" if w in ("crimson", "warped") else f"stripped_{w}_log"
        shaped(["| |", "###", "###"], {"#": t, "|": "chain"}, f"{w}_hanging_sign", 6)


def main():
    import gen_block_models as gm
    stalks("bamboo_block", BAMBOO, "#3e5410")
    stalks("stripped_bamboo_block", STRIPPED, "#8a7020")
    bundle("bamboo_block_top", BAMBOO, "#3e5410")
    bundle("stripped_bamboo_block_top", STRIPPED, "#8a7020")
    planks()
    door_and_trapdoor()
    heads()
    for w in gm.HANGING_WOODS:
        b2.iso_icon(gm.MODELS[f"{w}_hanging_sign"], f"{w}_hanging_sign")
    data(gm.HANGING_WOODS)
    text = open(BLOCKS, encoding="utf-8").read()
    text = re.sub(r"\n *// B4 start.*?// B4 end\n", "\n", text, flags=re.S)
    have = set(re.findall(r'\(name: "([^"]+)"', text))
    for f in glob.glob(os.path.join(ROOT, "packs", "vanilla", "*", "blocks.ron")):
        if os.sep + "models" + os.sep not in f:
            have |= set(re.findall(r'\(name: "([^"]+)"', open(f, encoding="utf-8").read()))
    keep = [b for b in blocks if re.search(r'name: "([^"]+)"', b).group(1) not in have]
    block = "    // B4 start (tools/gen_blocos_b4.py)\n" + "\n".join(keep) + "\n    // B4 end\n"
    at = re.search(r"^\]\)", text, flags=re.M).start()
    open(BLOCKS, "w", encoding="utf-8").write(text[:at] + block + text[at:])
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B4 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b4.py:\n"
                "// edit it and run it again.\n(\n    shaped: [\n" + "\n".join(recipes) + "\n    ],\n    shapeless: [\n"
                + "\n".join(shapeless) + "\n    ],\n)\n")
    print(f"{len(keep)} blocks ({len(blocks) - len(keep)} already in the game), {len(recipes) + len(shapeless)} recipes")


if __name__ == "__main__":
    main()
