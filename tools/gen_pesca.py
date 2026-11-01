"""Angler's fishing gear (our own art, 16x16 designs; tools/tex64/build.py makes the 64x64):
tackle box, angler hat, high test line and angler earring. Also writes their entries in
packs/vanilla/items.ron between the "pesca begin/end" marks.

    python3 tools/gen_pesca.py && python3 tools/tex64/build.py --only tackle_box,angler_hat,high_test_line,angler_earring
"""
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "packs" / "vanilla" / "textures" / "item"


def draw(rows, pal):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch in pal:
                im.putpixel((x, y), pal[ch] + (255,))
    return im


TACKLE = [
    "................",
    "................",
    ".....kkkkkk.....",
    ".....k....k.....",
    "..kkkkkkkkkkkk..",
    "..kGGGGGGGGGGk..",
    "..kGggggggggGk..",
    "..kkkkkyykkkkk..",
    "..kGGGGyyGGGGk..",
    "..kGggggggggGk..",
    "..kGggggggggGk..",
    "..kGddddddddGk..",
    "..kkkkkkkkkkkk..",
    "................",
    "................",
    "................",
]
HAT = [
    "................",
    "................",
    "................",
    ".....kkkkkk.....",
    "....kTTTTTTk....",
    "...kTttttttTk...",
    "...kTttttttTk...",
    "...krrrrrrorrk..",
    "...kTttttttTk...",
    ".kkkkkkkkkkkkkk.",
    "kBBBBBBBBBBBBBBk",
    ".kbbbbbbbbbbbbk.",
    "..kkkkkkkkkkkk..",
    "................",
    "................",
    "................",
]
LINE = [
    "................",
    "................",
    "....kkkkkkkk....",
    "....kWWWWWWk....",
    "....kkkkkkkk....",
    ".....kSsSsk.....",
    ".....ksSsSk.....",
    ".....kSsSsk.....",
    ".....ksSsSk.....",
    ".....kSsSsk.....",
    "....kkkkkkkks...",
    "....kWWWWWWk.s..",
    "....kkkkkkkk..s.",
    "...............s",
    "................",
    "................",
]
EARRING = [
    "................",
    "................",
    "......kkk.......",
    ".....kYyYk......",
    "....kYk.kYk.....",
    "....kYk.kYk.....",
    ".....kYyYk......",
    "......kYk.......",
    "......kYk.......",
    ".....kfffk......",
    "....kfFFFfk.k...",
    "...kfFeFFFfkfk..",
    "....kfFFFfk.k...",
    ".....kfffk......",
    "................",
    "................",
]

PAL = {
    "k": (34, 30, 28),
    "G": (54, 112, 70), "g": (78, 146, 92), "d": (40, 84, 52), "y": (214, 176, 70),
    "T": (150, 128, 82), "t": (186, 162, 110), "r": (126, 52, 40), "o": (230, 120, 50),
    "B": (168, 146, 98), "b": (130, 110, 70),
    "W": (120, 84, 50), "S": (214, 220, 226), "s": (168, 176, 186),
    "Y": (226, 186, 70), "f": (60, 110, 150), "F": (100, 160, 200), "e": (20, 20, 24),
}

ITEMS = [
    ("tackle_box", "Caixa de Pesca", TACKLE),
    ("angler_hat", "Chapéu de Pescador", HAT),
    ("high_test_line", "Linha de Alta Resistência", LINE),
    ("angler_earring", "Brinco do Pescador", EARRING),
]

for name, _, rows in ITEMS:
    draw(rows, PAL).save(OUT / f"{name}.png")

ron = ROOT / "packs" / "vanilla" / "items.ron"
text = ron.read_text()
block = "    // pesca begin (tools/gen_pesca.py)\n" + "".join(
    f'    (name: "{n}", display_name: Some("{d}"), texture: Some("{n}"), max_stack: Some(1)),\n' for n, d, _ in ITEMS
) + "    // pesca end\n"
if "// pesca begin" in text:
    a = text.index("    // pesca begin")
    b = text.index("// pesca end\n") + len("// pesca end\n")
    text = text[:a] + block + text[b:]
else:
    k = text.rindex("])")
    text = text[:k] + block + text[k:]
ron.write_text(text)
print("pesca: 4 items")
