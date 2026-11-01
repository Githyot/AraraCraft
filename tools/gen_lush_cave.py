#!/usr/bin/env python3
"""Pictures (item icons) for the lush cave plants: spore blossom, big and small dripleaf,
azalea, flowering azalea and hanging roots. Their 3D bodies are in tools/gen_block_models.py.

Drawn as 16x16 pixel maps (one letter per pixel) and blown up to 64x64, each pixel a 4x4
cell with a light top-left and darker bottom-right texel, like the rest of our art.

Usage: python3 tools/gen_lush_cave.py
"""

import os

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BLOCK_TEX = os.path.join(ROOT, "packs", "vanilla", "textures", "block")

PAL = {
    "o": "#1f3312",  # outline green
    "d": "#3b6b1f",  # dark leaf
    "g": "#5c9a2c",  # leaf
    "l": "#86c43f",  # light leaf
    "h": "#b9e46a",  # leaf highlight
    "s": "#4e7f2a",  # stem
    "p": "#c94f8c",  # pink
    "q": "#ec84b6",  # light pink
    "w": "#ffc4e0",  # pale pink
    "m": "#7a2a58",  # dark pink
    "y": "#e8d84a",  # pollen
    "b": "#5a3a22",  # root dark
    "r": "#7d5634",  # root
    "t": "#a37a4c",  # root light
    "k": "#26180e",  # root outline
}

ICONS = {
    "spore_blossom": [
        "................",
        "......oooo......",
        ".....ossllo.....",
        "....oggssggo....",
        "...mpqqyyqqpm...",
        "..mpqwqyyqwqpm..",
        ".mpqwqmyymqwqpm.",
        ".mpqqm.yy.mqqpm.",
        ".mpqm..yy..mqpm.",
        ".mpm...yy...mpm.",
        "..m....y.y...m..",
        ".......y..y.....",
        "......y....y....",
        "................",
        "................",
        "................",
    ],
    "big_dripleaf": [
        "................",
        "..oooooooooooo..",
        ".ohlllllllllllo.",
        "ohllgggggggglllo",
        "olggggdgdggggglo",
        "oggggdgggdggggdo",
        ".odddddddddddoo.",
        "..ooooossooooo..",
        "......osso......",
        "......osso......",
        ".......sso......",
        "......oss.......",
        "......osso......",
        "......osso......",
        ".....oossoo.....",
        "....odssssdo....",
    ],
    "small_dripleaf": [
        "................",
        "................",
        "................",
        "..ooo......ooo..",
        ".ollgo....oglho.",
        ".oggdo....odggo.",
        "..oods....sdoo..",
        "....os.ooo.so...",
        "....os.ollgo....",
        ".....soggdo.....",
        ".....s.oods.....",
        ".....s...s......",
        ".....os..s......",
        "......s.so......",
        ".....ossso......",
        "....odsssdo.....",
    ],
    "azalea": [
        "................",
        "....oooooooo....",
        "...ohlllglllo...",
        "..ohlggglgggdo..",
        ".ohlgggdgggggdo.",
        ".olggdgggggdgdo.",
        ".oggggggdggggdo.",
        ".odggdggggggddo.",
        "..oddddgdddddo..",
        "...ooooddooooo..",
        "......obbo......",
        "......orbo......",
        "......orbo......",
        ".....oorboo.....",
        "....okrrrrko....",
        "................",
    ],
    "flowering_azalea": [
        "................",
        "....oooooooo....",
        "...ohlqllglpo...",
        "..ohlqwglgggdo..",
        ".ohlgggdgqgggdo.",
        ".olpgdgggwqdgdo.",
        ".oqwgggggdgpgdo.",
        ".odggdgqgggddo..",
        "..oddddpwddddo..",
        "...ooooddooooo..",
        "......obbo......",
        "......orbo......",
        "......orbo......",
        ".....oorboo.....",
        "....okrrrrko....",
        "................",
    ],
    "hanging_roots": [
        "kkkkkkkkkkkkkkkk",
        ".ktrk.krtk.ktrk.",
        ".kbrk.kbrk..kr..",
        "..rk...rk...kr..",
        "..rk..krk...rk..",
        "..kr..kr....rk..",
        "...r..kr...krk..",
        "...rk..r...kr...",
        "...kr..rk..r....",
        "....r..kr..r....",
        "....r...r..k....",
        "....k...r.......",
        "........k.......",
        "................",
        "................",
        "................",
    ],
}


def hexc(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], dtype=np.float64)


def draw(rows):
    img = np.zeros((64, 64, 4), dtype=np.uint8)
    for y, row in enumerate(rows):
        assert len(row) == 16, row
        for x, ch in enumerate(row):
            if ch == ".":
                continue
            c = hexc(PAL[ch])
            for j in range(4):
                for i in range(4):
                    d = 1.0 + 0.06 * ((1.5 - i) + (1.5 - j)) / 3.0
                    img[y * 4 + j, x * 4 + i, :3] = np.clip(c * d, 0, 255)
                    img[y * 4 + j, x * 4 + i, 3] = 255
    return img


if __name__ == "__main__":
    for name, rows in ICONS.items():
        Image.fromarray(draw(rows), "RGBA").save(os.path.join(BLOCK_TEX, name + ".png"))
    print(f"{len(ICONS)} icons -> {BLOCK_TEX}")
