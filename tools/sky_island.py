"""The big sky island (ilha_do_ceu_grande) for packs/vanilla/tr/structures.ron: it is made of the
Terraria side's blocks (cloud, sky brick), so it lives in that file. tools/temple_traps.py adds
it (and gen_tr.py runs that)."""
import math
import random


def grid(w, d, ch=" "):
    return [[ch] * w for _ in range(d)]


def rows(g):
    return ["".join(r) for r in g]


def big_sky_island(seed):
    """A big floating island (Terraria's sky islands, made for 3D): a jagged stone underside
    hanging like a root, a grass top with a little lake that spills off one edge, a sky brick
    house with the chest of wonders, a tree, and cloud tufts round the rim. Harpies circle it."""
    rnd = random.Random(seed)
    W = D = 41
    top = 16  # grass layer
    H = top + 9
    c = W // 2
    layers = [grid(W, D, ".") for _ in range(H)]
    rim = {}
    for z in range(D):
        for x in range(W):
            a = math.atan2(z - c, x - c)
            rim[(x, z)] = 17.5 + 2.2 * math.sin(a * 3 + seed) + 1.3 * math.sin(a * 7 + seed * 2)
    for y in range(top + 1):
        t = y / top
        for z in range(D):
            for x in range(W):
                d = math.hypot(x - c, z - c)
                r = rim[(x, z)] * (0.15 + 0.85 * t ** 0.55) + rnd.uniform(-0.6, 0.6)
                if d <= r:
                    layers[y][z][x] = "G" if y == top else ("D" if y >= top - 2 else "S")
    # Cloud tufts round the rim, a layer above and below the grass.
    for z in range(D):
        for x in range(W):
            d = math.hypot(x - c, z - c)
            if rim[(x, z)] - 0.5 < d < rim[(x, z)] + 1.6 and rnd.random() < 0.55:
                layers[top - 1 if rnd.random() < 0.5 else top][z][x] = "c"
    # The lake: an ellipse on the east side, two deep, with a sand shore.
    lx, lz = c + 9, c - 3
    for z in range(D):
        for x in range(W):
            e = ((x - lx) / 5.5) ** 2 + ((z - lz) / 4.0) ** 2
            if layers[top][z][x] != "G":
                continue
            if e < 1.0:
                layers[top][z][x] = "w"
                layers[top - 1][z][x] = "w" if e < 0.55 else "a"
                layers[top - 2][z][x] = "a" if layers[top - 2][z][x] != "." else "."
            elif e < 1.5:
                layers[top][z][x] = "a"
    # The house: 9 by 7 of sky brick, door to the south, glass windows, a chest and torches.
    hx, hz = c - 10, c - 4
    for y in range(top + 1, top + 6):
        for z in range(hz, hz + 7):
            for x in range(hx, hx + 9):
                edge = x in (hx, hx + 8) or z in (hz, hz + 6)
                if y == top + 5:
                    layers[y][z][x] = "B"
                elif edge:
                    layers[y][z][x] = "g" if y == top + 2 and (x - hx) % 4 == 2 and z in (hz, hz + 6) else "B"
                else:
                    layers[y][z][x] = "."
        for z in range(hz, hz + 7):
            layers[top][z][hx:hx + 9] = ["B"] * 9
    layers[top + 1][hz + 6][hx + 4] = "."
    layers[top + 2][hz + 6][hx + 4] = "."
    layers[top + 1][hz + 1][hx + 4] = "K"
    layers[top + 3][hz + 1][hx + 1] = "t"
    layers[top + 3][hz + 1][hx + 7] = "t"
    # A tree west of the lake.
    tx, tz = c + 2, c + 8
    for y in range(top + 1, top + 6):
        layers[y][tz][tx] = "o"
    for y in range(top + 4, top + 8):
        for z in range(tz - 2, tz + 3):
            for x in range(tx - 2, tx + 3):
                if layers[y][z][x] == "." and abs(x - tx) + abs(z - tz) + max(0, y - top - 6) * 2 <= 3:
                    layers[y][z][x] = "e"
    # Flowers on the grass.
    for _ in range(14):
        x, z = rnd.randrange(W), rnd.randrange(D)
        if layers[top][z][x] == "G" and layers[top + 1][z][x] == ".":
            layers[top + 1][z][x] = "f"
    return {
        "name": "ilha_do_ceu_grande",
        "biomes": ["Plains", "Forest", "Birch Forest", "Taiga", "Savanna", "Meadow", "Dark Forest", "Jungle", "Desert", "Snowy Plains", "Cherry Grove"],
        "spacing": 64,
        "chance": 0.6,
        "place": "Sky(165)",
        "mobs": [("harpy", 3)],
        "palette": {"S": "stone", "D": "dirt", "G": "grass_block", "c": "cloud_block", "B": "sky_brick", "g": "glass", "K": "chest",
                    "t": "torch", "f": "dandelion", "w": "water", "a": "sand", "o": "oak_log", "e": "oak_leaves"},
        "layers": [rows(l) for l in layers],
        "loot": [("", "lucky_horseshoe", 1, 1, 0.4), ("", "shiny_balloon", 1, 1, 0.35), ("", "cloud_bottle", 1, 1, 0.4),
                 ("", "sky_blade", 1, 1, 0.35), ("", "fallen_star", 3, 8, 0.8), ("", "gold_ingot", 3, 8, 0.6)],
    }



def ron_entry(s):
    out = ["    (", f'        name: "{s["name"]}",', "        biomes: [" + ", ".join(f'"{b}"' for b in s["biomes"]) + "],",
           f'        spacing: {s["spacing"]},', f'        chance: {s["chance"]},', f'        place: {s["place"]},',
           "        mobs: [" + ", ".join(f'("{m}", {n})' for m, n in s["mobs"]) + "],",
           "        palette: [" + ", ".join(f"('{k}', \"{v}\")" for k, v in s["palette"].items()) + "],", "        layers: ["]
    for layer in s["layers"]:
        out.append("            [" + ", ".join(f'"{r}"' for r in layer) + "],")
    out.append("        ],")
    out.append("        loot: [")
    for _, item, lo, hi, ch in s["loot"]:
        out.append(f'            (item: "{item}", min: {lo}, max: {hi}, chance: {ch}),')
    out.append("        ],")
    out.append("    ),")
    return "\n".join(out)


def add(text):
    """The island appended to a structures file's list (replacing an older copy)."""
    entry = ron_entry(big_sky_island(5))
    i = text.find('    (\n        name: "ilha_do_ceu_grande"')
    if i >= 0:
        j = text.index("\n    ),", i) + len("\n    ),")
        return text[:i] + entry + text[j:]
    k = text.rstrip().rindex("])")
    head = text[:k].rstrip()
    return head + "\n" + entry + "\n])\n"
