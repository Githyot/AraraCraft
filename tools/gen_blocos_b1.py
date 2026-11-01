#!/usr/bin/env python3
"""Lote B1 of the block review (plan/blocos-analise.md): writes the new blocks into
packs/vanilla/blocks.ron (between `// B1 start` and `// B1 end`) and their recipes into
packs/vanilla/blocos/recipes.ron, so it can be re-run.

Textures: tools/tex64/blocos_b1.py.
"""

import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BLOCKS = os.path.join(ROOT, "packs", "vanilla", "blocks.ron")
RECIPES = os.path.join(ROOT, "packs", "vanilla", "blocos", "recipes.ron")

COLORS = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple",
          "blue", "brown", "green", "red", "black"]
COR = {"white": "Branco", "orange": "Laranja", "magenta": "Magenta", "light_blue": "Azul-claro", "yellow": "Amarelo",
       "lime": "Verde-limão", "pink": "Rosa", "gray": "Cinza", "light_gray": "Cinza-claro", "cyan": "Ciano",
       "purple": "Roxo", "blue": "Azul", "brown": "Marrom", "green": "Verde", "red": "Vermelho", "black": "Preto"}
# Feminine forms (terracota, painel is masc. but "de vidro" adjective agrees with vidro).
CORA = dict(COR, white="Branca", yellow="Amarela", purple="Roxa", red="Vermelha", black="Preta")

blocks = []
recipes = []
smelt = []


def tex(t):
    if t is None:
        return ""
    if isinstance(t, str):
        return f', textures: (all: Some("{t}"))'
    parts = ", ".join(f'{k}: Some("{v}")' for k, v in t.items())
    return f", textures: ({parts})"


def blk(name, pt, hardness, tool="Pickaxe", tier=1, shape=None, t=None, extra=""):
    s = f'    (name: "{name}", display_name: Some("{pt}")'
    if shape:
        s += f", shape: {shape}"
    s += f", hardness: {hardness}"
    if tool:
        s += f", tool: Some({tool})"
        if tool == "Pickaxe" and tier:
            s += f", tier: {tier}"
    s += tex(t) + extra + "),"
    blocks.append(s)


def shaped(pattern, key, result, count=1):
    ks = ", ".join(f"'{k}': \"{v}\"" for k, v in key.items())
    pat = ", ".join(f'"{p}"' for p in pattern)
    recipes.append(f"        (pattern: [{pat}], key: {{{ks}}}, result: \"{result}\", count: {count}),")


def family(base, pt, hardness, t, slab=True, stairs=True, wall=True, slab_h=None, have=()):
    """Slab, stairs and wall of a stone, those it doesn't have yet. pt: the stone's name in Portuguese."""
    if slab and "slab" not in have:
        blk(f"{base}_slab".replace("bricks_slab", "brick_slab"), f"Laje de {pt}", slab_h or hardness, shape="Slab", t=t)
        shaped(["###"], {"#": src_of(base)}, f"{base}_slab".replace("bricks_slab", "brick_slab"), 6)
    if stairs and "stairs" not in have:
        blk(f"{base}_stairs".replace("bricks_stairs", "brick_stairs"), f"Escada de {pt}", hardness, shape="Stairs", t=t)
        shaped(["#  ", "## ", "###"], {"#": src_of(base)}, f"{base}_stairs".replace("bricks_stairs", "brick_stairs"), 4)
    if wall and "wall" not in have:
        blk(f"{base}_wall".replace("bricks_wall", "brick_wall"), f"Muro de {pt}", hardness, shape="Wall", t=t)
        shaped(["###", "###"], {"#": src_of(base)}, f"{base}_wall".replace("bricks_wall", "brick_wall"), 6)


SRC = {}


def src_of(base):
    return SRC.get(base, base)


# ---------------------------------------------------------------- polished stones and their shapes
for st, pt in (("granite", "Granito"), ("diorite", "Diorito"), ("andesite", "Andesito")):
    blk(f"polished_{st}", f"{pt} Polido", 1.5)
    shaped(["##", "##"], {"#": st}, f"polished_{st}", 4)
    family(st, pt, 1.5, st)
    family(f"polished_{st}", f"{pt} Polido", 1.5, f"polished_{st}", wall=False)

SAND_T = lambda c: {"side": f"{c}_side", "top": f"{c}_top", "bottom": f"{c}_bottom"}
family("red_sandstone", "Arenito Vermelho", 0.8, SAND_T("red_sandstone"), slab_h=2.0)
family("sandstone", "Arenito", 0.8, SAND_T("sandstone"), have=("slab", "stairs"))
blk("cut_red_sandstone", "Arenito Vermelho Lapidado", 0.8)
shaped(["##", "##"], {"#": "red_sandstone"}, "cut_red_sandstone", 4)
blk("cut_sandstone_slab", "Laje de Arenito Lapidado", 2.0, shape="Slab", t="cut_sandstone")
shaped(["###"], {"#": "cut_sandstone"}, "cut_sandstone_slab", 6)
blk("cut_red_sandstone_slab", "Laje de Arenito Vermelho Lapidado", 2.0, shape="Slab", t="cut_red_sandstone")
shaped(["###"], {"#": "cut_red_sandstone"}, "cut_red_sandstone_slab", 6)
blk("chiseled_red_sandstone", "Arenito Vermelho Talhado", 0.8)
shaped(["#", "#"], {"#": "red_sandstone_slab"}, "chiseled_red_sandstone")
family("smooth_sandstone", "Arenito Liso", 2.0, "smooth_sandstone", wall=False)
family("smooth_red_sandstone", "Arenito Vermelho Liso", 2.0, "smooth_red_sandstone", wall=False)
family("smooth_quartz", "Quartzo Liso", 2.0, "smooth_quartz", wall=False)
blk("smooth_stone_slab", "Laje de Pedra Lisa", 2.0, shape="Slab", t="smooth_stone")
shaped(["###"], {"#": "smooth_stone"}, "smooth_stone_slab", 6)
family("prismarine", "Prismarinho", 1.5, "prismarine")
family("prismarine_bricks", "Tijolos de Prismarinho", 1.5, "prismarine_bricks", wall=False)
family("dark_prismarine", "Prismarinho Escuro", 1.5, "dark_prismarine", wall=False)
family("mossy_stone_bricks", "Tijolos de Pedra Musgosos", 1.5, "mossy_stone_bricks")
family("stone_bricks", "Tijolos de Pedra", 1.5, "stone_bricks", have=("slab", "stairs"))
family("bricks", "Tijolos", 2.0, "bricks", have=("slab", "stairs"))
family("mossy_cobblestone", "Pedregulho com Musgo", 2.0, "mossy_cobblestone", have=("slab", "stairs"))
family("nether_bricks", "Tijolos do Nether", 2.0, "nether_bricks", have=("slab", "stairs"))
family("blackstone", "Pedra-negra", 1.5, "blackstone", have=("slab", "stairs"))
family("polished_blackstone", "Pedra-negra Polida", 2.0, "polished_blackstone")
family("polished_blackstone_bricks", "Tijolos de Pedra-negra Polida", 1.5, "polished_blackstone_bricks")
blk("polished_blackstone_button", "Botão de Pedra-negra Polida", 0.5, shape="Button", t="polished_blackstone",
    extra=", solid: false")
recipes.append('        (pattern: ["#"], key: {\'#\': "polished_blackstone"}, result: "polished_blackstone_button"),')
blk("polished_blackstone_pressure_plate", "Placa de Pressão de Pedra-negra Polida", 0.5, shape="Plate",
    t="polished_blackstone", extra=", solid: false")
shaped(["##"], {"#": "polished_blackstone"}, "polished_blackstone_pressure_plate")
for d, pt in (("cobbled_deepslate", "Ardósia em Pedregulho"), ("polished_deepslate", "Ardósia Polida"),
              ("deepslate_bricks", "Tijolos de Ardósia"), ("deepslate_tiles", "Ladrilhos de Ardósia")):
    family(d, pt, 3.5, d, have=("slab", "stairs"))
for d, pt in (("tuff", "Tufo"), ("polished_tuff", "Tufo Polido"), ("tuff_bricks", "Tijolos de Tufo")):
    family(d, pt, 1.5, d, have=("slab", "stairs"))
family("end_stone_bricks", "Tijolos de Pedra do End", 3.0, "end_stone_bricks", have=("slab", "stairs"))
family("mud_bricks", "Tijolos de Lama", 1.5, "mud_bricks", have=("slab", "stairs"))

blk("red_nether_bricks", "Tijolos do Nether Vermelhos", 2.0)
shaped(["NW", "WN"], {"N": "nether_brick", "W": "nether_wart"}, "red_nether_bricks")
family("red_nether_bricks", "Tijolos do Nether Vermelhos", 2.0, "red_nether_bricks")
blk("chiseled_stone_bricks", "Tijolos de Pedra Talhados", 1.5)
shaped(["#", "#"], {"#": "stone_brick_slab"}, "chiseled_stone_bricks")
blk("chiseled_nether_bricks", "Tijolos do Nether Talhados", 2.0)
shaped(["#", "#"], {"#": "nether_brick_slab"}, "chiseled_nether_bricks")
blk("cracked_nether_bricks", "Tijolos do Nether Rachados", 2.0)
smelt.append('        (input: "nether_bricks", result: "cracked_nether_bricks", xp: 0.1),')
blk("cracked_deepslate_tiles", "Ladrilhos de Ardósia Rachados", 3.5)
smelt.append('        (input: "deepslate_tiles", result: "cracked_deepslate_tiles", xp: 0.1),')
blk("chiseled_quartz_block", "Bloco de Quartzo Talhado", 0.8,
    t={"side": "chiseled_quartz_block", "top": "chiseled_quartz_block_top", "bottom": "chiseled_quartz_block_top"})
shaped(["#", "#"], {"#": "quartz_slab"}, "chiseled_quartz_block")
blk("quartz_pillar", "Pilar de Quartzo", 0.8, t={"side": "quartz_pillar", "top": "quartz_pillar_top", "bottom": "quartz_pillar_top"})
shaped(["#", "#"], {"#": "quartz_block"}, "quartz_pillar", 2)
blk("quartz_bricks", "Tijolos de Quartzo", 0.8)
shaped(["##", "##"], {"#": "quartz_block"}, "quartz_bricks")

# ---------------------------------------------------------------- odd blocks
blk("bone_block", "Bloco de Osso", 2.0, t={"side": "bone_block_side", "top": "bone_block_top", "bottom": "bone_block_top"})
shaped(["###", "###", "###"], {"#": "bone_meal"}, "bone_block")
blocks.append('    (name: "honey_block", display_name: Some("Bloco de Mel"), transparency: Translucent, hardness: 0.0, '
              'textures: (side: Some("honey_block_side"), top: Some("honey_block_top"), bottom: Some("honey_block_top"))),')
shaped(["##", "##"], {"#": "honey_bottle"}, "honey_block")
blk("target", "Alvo", 0.5, tool="Hoe", t={"side": "target_side", "top": "target_top", "bottom": "target_top"})
shaped([" R ", "RHR", " R "], {"R": "redstone", "H": "hay_block"}, "target")
blocks.append('    (name: "tinted_glass", display_name: Some("Vidro Fumê"), transparency: Translucent, hardness: 0.3),')
shaped([" A ", "AGA", " A "], {"A": "amethyst", "G": "glass"}, "tinted_glass", 2)

# Deepslate ores: in the ores thread's PR (#115), not here.

# ---------------------------------------------------------------- froglights, dead corals
for f, pt in (("ochre", "Ocre"), ("verdant", "Verdejante"), ("pearlescent", "Perolada")):
    blocks.append(f'    (name: "{f}_froglight", display_name: Some("Luz-de-sapo {pt}"), hardness: 0.3, light: 15, '
                  f'textures: (side: Some("{f}_froglight_side"), top: Some("{f}_froglight_top"), bottom: Some("{f}_froglight_top"))),')
for c, pt in (("tube", "Tubo"), ("brain", "Cérebro"), ("bubble", "Bolha"), ("fire", "Fogo"), ("horn", "Chifre")):
    blk(f"dead_{c}_coral_block", f"Bloco de Coral-{pt} Morto", 1.5)

# ---------------------------------------------------------------- woods
WOODS = [("oak", "Carvalho"), ("birch", "Bétula"), ("spruce", "Pinheiro"), ("jungle", "Selva"), ("acacia", "Acácia"),
         ("dark_oak", "Carvalho Escuro"), ("cherry", "Cerejeira"), ("mangrove", "Mangue"), ("pale_oak", "Carvalho Pálido"),
         ("crimson", "Carmesim"), ("warped", "Distorcida"), ("maple", "Bordo"), ("walnut", "Nogueira"),
         ("araucaria", "Araucária"), ("buriti", "Buriti"), ("coconut", "Coqueiro"), ("ipe", "Ipê"),
         ("jabuticaba", "Jabuticabeira"), ("echo", "Eco"), ("skyroot", "Raiz-celeste")]
for w, pt in WOODS:
    nether = w in ("crimson", "warped")
    stem = "stem" if nether else "log"
    wood = "hyphae" if nether else "wood"
    tool = "Axe"
    # Bark on all six sides, like the log's side turned every way.
    blk(f"{w}_{wood}", ("Hifas" if nether else "Madeira") + f" de {pt}", 2.0, tool=tool, t=f"{w}_{stem}")
    shaped(["##", "##"], {"#": f"{w}_{stem}"}, f"{w}_{wood}", 3)
    blk(f"stripped_{w}_{stem}", ("Caule" if nether else "Tronco") + f" de {pt} Descascado", 2.0, tool=tool,
        t={"side": f"stripped_{w}_{stem}", "top": f"stripped_{w}_{stem}_top", "bottom": f"stripped_{w}_{stem}_top"})
    blk(f"stripped_{w}_{wood}", ("Hifas" if nether else "Madeira") + f" de {pt} Descascada", 2.0, tool=tool, t=f"stripped_{w}_{stem}")
    shaped(["##", "##"], {"#": f"stripped_{w}_{stem}"}, f"stripped_{w}_{wood}", 3)
    recipes.append(f'        (pattern: ["#"], key: {{\'#\': "stripped_{w}_{stem}"}}, result: "{w}_planks", count: 4),')

# ---------------------------------------------------------------- colours
NEW_TC = ["magenta", "light_blue", "lime", "pink", "gray", "cyan", "purple", "blue", "green", "black"]
for c in COLORS:
    blocks.append(f'    (name: "{c}_concrete_powder", display_name: Some("Pó de Concreto {COR[c]}"), hardness: 0.5, gravity: true, tool: Some(Shovel)),')
    recipes.append(f'        (pattern: ["DSS", "SSG", "GGG"], key: {{\'D\': "{c}_dye", \'S\': "sand", \'G\': "gravel"}}, result: "{c}_concrete_powder", count: 8),')
    if c in NEW_TC:
        blk(f"{c}_terracotta", f"Terracota {CORA[c]}", 1.25)
        shaped(["###", "#D#", "###"], {"#": "terracotta", "D": f"{c}_dye"}, f"{c}_terracotta", 8)
    blk(f"{c}_glazed_terracotta", f"Terracota Vitrificada {CORA[c]}", 1.4)
    smelt.append(f'        (input: "{c}_terracotta", result: "{c}_glazed_terracotta", xp: 0.1),')
    blocks.append(f'    (name: "{c}_stained_glass_pane", display_name: Some("Painel de Vidro {COR[c]}"), shape: Pane, '
                  f'transparency: Translucent, hardness: 0.3, drops: Some([]), textures: (all: Some("{c}_stained_glass"))),')
    shaped(["###", "###"], {"#": f"{c}_stained_glass"}, f"{c}_stained_glass_pane", 16)


def put(path, section_end_re, lines, tag):
    text = open(path, encoding="utf-8").read()
    text = re.sub(rf"\n *// {tag} start.*?// {tag} end\n", "\n", text, flags=re.S)
    block = f"    // {tag} start (tools/gen_blocos_b1.py)\n" + "\n".join(lines) + f"\n    // {tag} end\n"
    m = list(re.finditer(section_end_re, text, flags=re.M))
    assert m, (path, section_end_re)
    at = m[0].start()
    text = text[:at] + block + text[at:]
    open(path, "w", encoding="utf-8").write(text)


def main():
    os.makedirs(os.path.dirname(RECIPES), exist_ok=True)
    names = [re.search(r'name: "([^"]+)"', b).group(1) for b in blocks]
    assert len(names) == len(set(names)), "duplicate"
    have = set()
    for f in ("blocks.ron", "tr/blocks.ron", "tr3/blocks.ron", "fase6/blocks.ron", "mods/blocks.ron", "etapa5/blocks.ron",
              "deep/blocks.ron", "aguas/blocks.ron", "vulcao/blocks.ron"):
        t = open(os.path.join(ROOT, "packs", "vanilla", f), encoding="utf-8").read()
        t = re.sub(r"// B1 start.*?// B1 end", "", t, flags=re.S)
        have |= set(re.findall(r'\(name: "([^"]+)"', t))
    dup = [n for n in names if n in have]
    keep = [b for b, n in zip(blocks, names) if n not in have]
    if dup:
        print("already in the game, skipped:", " ".join(dup))
    newn = {n for n in names if n not in have}
    rec = [r for r in recipes if re.search(r'result: "([^"]+)"', r).group(1) in newn
           or 'result: "' in r and re.search(r'result: "([^"]+)"', r).group(1).endswith("_planks")]
    sm = [s for s in smelt if re.search(r'result: "([^"]+)"', s).group(1) in newn]
    # Blocks: before the closing "])" of the list.
    put(BLOCKS, r"^\]\)", keep, "B1")
    # Recipes in a file of their own: they use items of the other content files (prismarine,
    # deepslate, amethyst...), which the base recipes.ron alone can't see.
    with open(RECIPES, "w", encoding="utf-8") as f:
        f.write("// Lote B1 of the block review (plan/blocos-analise.md). Made by tools/gen_blocos_b1.py:\n"
                "// edit it and run it again.\n(\n    shaped: [\n" + "\n".join(rec) + "\n    ],\n    smelting: [\n"
                + "\n".join(sm) + "\n    ],\n)\n")
    print(f"{len(keep)} blocks, {len(rec)} recipes, {len(sm)} smelting")


if __name__ == "__main__":
    main()
