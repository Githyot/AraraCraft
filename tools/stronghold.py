"""The buried stronghold (Lote E5 of the structure review): a 61x61 maze of stone brick
rooms joined by corridors, about 35 blocks under the ground.

Rooms: the stair room with its spiral up to a small ruin on the surface, a treasure room,
a two-floor library with a balcony, shelves and a chandelier, a store room, a fountain hall
in the middle, a crossing with pillars, a prison of barred cells, and the portal room with
a silverfish spawner before the dais and its ring of twelve empty frames over lava.

Walls are stone bricks worn into mossy and cracked ones (a few infested), so it reads old.
Run through gen_structures.py, which writes it into packs/vanilla/structures.ron.
"""

W = D = 61
H = 12          # height of the rooms' part (the library is the tallest, ceiling at 11)
LAYERS = 37     # total layers, the ruin on top included
SX = SZ = 6     # corner of the stair well's 3x3 inside the stair room


def stronghold():
    halls = [[[" "] * W for _ in range(D)] for _ in range(H)]

    def wall_ch(x, y, z):
        h = (x * 73856093 ^ y * 19349663 ^ z * 83492791) & 0xFFFF
        if h % 97 == 0:
            return "I"
        if h % 6 == 0:
            return "m"
        if h % 7 == 0:
            return "k"
        return "S"

    def put_wall(x, y, z):
        if halls[y][z][x] != ".":
            halls[y][z][x] = wall_ch(x, y, z)

    def room(x0, z0, x1, z1, top):
        """Floor at y 0, air 1..top-1, ceiling at `top`; walls never fill another room's air."""
        for y in range(0, top + 1):
            for z in range(z0, z1 + 1):
                for x in range(x0, x1 + 1):
                    edge = x in (x0, x1) or z in (z0, z1)
                    if y in (0, top) or edge:
                        put_wall(x, y, z)
                    else:
                        halls[y][z][x] = "."

    def corridor(a, b, at, along_x):
        """From wall `a` to wall `b` (inclusive, so both walls open), 3 wide and 3 high."""
        for k in range(a, b + 1):
            for s in range(-2, 3):
                x, z = (k, at + s) if along_x else (at + s, k)
                for y in range(0, 5):
                    if abs(s) == 2 or y in (0, 4):
                        put_wall(x, y, z)
                    else:
                        halls[y][z][x] = "."
        # A torch on the floor by the wall every 8 blocks.
        for k in range(a + 4, b - 2, 8):
            x, z = (k, at + 1) if along_x else (at + 1, k)
            halls[1][z][x] = "t"

    def put(x, y, z, ch):
        halls[y][z][x] = ch

    # Rooms (x0, z0, x1, z1, ceiling).
    room(2, 2, 12, 12, 6)       # stair room
    room(24, 2, 34, 12, 6)      # treasure room
    room(42, 2, 58, 20, 11)     # library
    room(2, 20, 14, 32, 5)      # store room
    room(22, 22, 38, 38, 8)     # fountain hall
    room(24, 44, 36, 56, 6)     # crossing
    room(2, 40, 20, 58, 5)      # prison
    room(40, 40, 58, 58, 9)     # portal room
    corridor(12, 24, 7, True)
    corridor(34, 42, 7, True)
    corridor(12, 20, 7, False)
    corridor(14, 22, 27, True)
    corridor(12, 22, 29, False)
    corridor(20, 40, 49, False)
    corridor(38, 44, 30, False)
    corridor(20, 24, 50, True)
    corridor(36, 40, 50, True)

    # Treasure room: a chest on a pedestal between two torches.
    for z in range(6, 9):
        for x in range(28, 31):
            put(x, 1, z, "S")
    put(29, 2, 7, "c")
    put(27, 1, 7, "t")
    put(31, 1, 7, "t")
    put(29, 5, 4, "h")

    # Library: rows of shelves, a balcony round the walls with more shelves, steps up,
    # a chandelier, a chest below and one above.
    for x in range(46, 57, 3):
        for z in range(5, 14):
            if z != 9:
                for y in (1, 2, 3):
                    put(x, y, z, "B")
    for z in range(3, 20):
        for x in range(43, 58):
            if x in (43, 44, 56, 57) or z in (3, 4, 18, 19):
                put(x, 5, z, "W")
                if x in (43, 57) or z in (3, 19):
                    for y in (6, 7, 8):
                        put(x, y, z, "B")
                elif (x + z) % 2 == 0:
                    put(x, 6, z, "f")
    for k in range(5):
        for y in range(1, k + 2):
            put(50 - k, y, 17, "W")
    put(46, 5, 18, "W")
    put(46, 6, 18, ".")
    put(45, 6, 18, ".")
    for y in (8, 9, 10):
        put(50, y, 11, "n")
    put(50, 7, 11, "h")
    for (x, z) in ((49, 11), (51, 11), (50, 10), (50, 12)):
        put(x, 8, z, "f")
        put(x, 7, z, "h")
    put(45, 1, 16, "c")
    put(56, 6, 10, "c")
    put(43, 6, 10, "B")

    # Store room: chests and barrels down the west wall, crates of planks.
    for z in range(21, 32):
        put(3, 1, z, "c" if z % 4 == 1 else "R")
        if z % 3 == 0:
            put(3, 2, z, "R")
    for (x, z) in ((10, 22), (11, 22), (10, 23), (12, 30), (12, 31)):
        put(x, 1, z, "W")
    put(10, 2, 22, "W")
    put(8, 4, 26, "h")

    # Fountain hall: four pillars, a pool with a rim of slabs, lanterns.
    for (px, pz) in ((25, 25), (34, 25), (25, 34), (34, 34)):
        for y in range(1, 8):
            for (dx, dz) in ((0, 0), (1, 0), (0, 1), (1, 1)):
                put(px + dx, y, pz + dz, "S" if y not in (1, 7) else "C")
    for z in range(28, 33):
        for x in range(28, 33):
            edge = x in (28, 32) or z in (28, 32)
            put(x, 1, z, "r" if edge else "~")
            if not edge:
                put(x, 0, z, "S")
    for (x, z) in ((26, 30), (34, 30), (30, 26), (30, 34)):
        put(x, 7, z, "h")

    # Crossing: four pillars and a lantern.
    for (px, pz) in ((27, 47), (33, 47), (27, 53), (33, 53)):
        for y in range(1, 6):
            put(px, y, pz, "C" if y in (1, 5) else "S")
    put(30, 5, 50, "h")

    # Prison: barred cells on both sides of an aisle; one cell's bars are broken.
    for x in range(3, 20):
        for y in (1, 2, 3):
            put(x, y, 47, "b")
            put(x, y, 51, "b")
    for wx in (7, 11, 15):
        for z in list(range(41, 48)) + list(range(51, 58)):
            for y in range(1, 5):
                put(wx, y, z, wall_ch(wx, y, z))
    for y in (1, 2):
        put(13, y, 51, ".")
    put(13, 1, 55, "c")
    for (x, z) in ((4, 42), (9, 45), (17, 43), (5, 56), (12, 53), (18, 55), (14, 56)):
        put(x, 4, z, "w")
    for x in (5, 13):
        put(x, 4, 49, "h")

    # Portal room: a dais, the frames round the lava, the spawner on the way up, lava pools
    # behind iron bars, lanterns.
    for z in range(46, 53):
        for x in range(46, 53):
            put(x, 1, z, "C" if x in (46, 52) or z in (46, 52) else "S")
    for z in range(48, 51):
        for x in range(48, 51):
            put(x, 1, z, "L")
    for k in (48, 49, 50):
        for (x, z) in ((k, 47), (k, 51), (47, k), (51, k)):
            put(x, 2, z, "F")
    put(49, 1, 44, "Z")
    for z in range(42, 57):
        for x in range(54, 58):
            put(x, 0, z, "L")
        put(53, 1, z, "b")
    for x in range(42, 53):
        for z in range(54, 58):
            put(x, 0, z, "L")
        put(x, 1, 53, "b")
    put(53, 1, 53, "b")
    for (x, z) in ((44, 44), (54, 44), (44, 52), (49, 49)):
        put(x, 8, z, "h")
    put(42, 1, 42, "c")

    # The stair well: a spiral of steps round a pillar, from the stair room's floor up through
    # the ground to a small mossy ruin.
    ring = [(1, 0), (2, 0), (2, 1), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)]
    layers = []
    for y in range(LAYERS - 1):
        g = halls[y] if y < H else [[" "] * W for _ in range(D)]
        if y >= 6:
            for z in range(SZ - 1, SZ + 4):
                for x in range(SX - 1, SX + 4):
                    g[z][x] = wall_ch(x, y, z)
        if y >= 1:
            for z in range(SZ, SZ + 3):
                for x in range(SX, SX + 3):
                    g[z][x] = "."
            g[SZ + 1][SX + 1] = "S"
            dx, dz = ring[y % 8]
            g[SZ + dz][SX + dx] = "S"
        layers.append(g)
    top = [[" "] * W for _ in range(D)]
    for z in range(SZ - 1, SZ + 4):
        for x in range(SX - 1, SX + 4):
            on_edge = x in (SX - 1, SX + 3) or z in (SZ - 1, SZ + 3)
            if on_edge and (x + z) % 2 == 0:
                top[z][x] = "m"
    layers.append(top)
    return {
        "name": "fortaleza_subterranea",
        "biomes": ["Plains", "Forest", "Birch Forest", "Taiga", "Savanna", "Snowy Plains", "Dark Forest", "Mountains", "Desert",
                   "Meadow", "Grove", "Cherry Grove", "Snowy Taiga", "Jungle", "Badlands", "Swamp"],
        "spacing": 64,
        "chance": 0.5,
        # The ruin on top stands on the ground.
        "sink": LAYERS - 1,
        "spawner": "silverfish",
        "palette": {"S": "stone_bricks", "m": "mossy_stone_bricks", "k": "cracked_stone_bricks", "I": "infested_stone_bricks",
                    "C": "smooth_stone", "B": "bookshelf", "c": "chest", "W": "oak_planks", "R": "barrel", "f": "oak_fence",
                    "n": "chain", "h": "lantern:1", "t": "torch", "L": "lava", "F": "end_portal_frame", "Z": "spawner",
                    "b": "iron_bars", "w": "cobweb", "r": "stone_brick_slab", "~": "water"},
        "layers": [["".join(r) for r in l] for l in layers],
        # Each room's chests hold what that room was for.
        "chests": [
            ((24, 0, 2), (34, 6, 12), [("diamond", 1, 3, 0.5), ("gold_ingot", 3, 8, 0.8), ("golden_apple", 1, 1, 0.3),
                                       ("enchanted_book", 1, 1, 0.4), ("diamond_sword", 1, 1, 0.08), ("iron_ingot", 3, 8, 0.6),
                                       ("ender_pearl", 1, 2, 0.3)]),
            ((42, 0, 2), (58, 11, 20), [("enchanted_book", 1, 2, 0.85), ("book", 2, 6, 0.7), ("paper", 3, 9, 0.6),
                                        ("compass", 1, 1, 0.2), ("map", 1, 1, 0.2), ("ink_sac", 1, 4, 0.3),
                                        ("flame_tome", 1, 1, 0.1), ("mana_crystal", 1, 1, 0.25)]),
            ((2, 0, 20), (14, 5, 32), [("bread", 2, 6, 0.8), ("apple", 1, 4, 0.5), ("potato", 2, 5, 0.4), ("carrot", 2, 5, 0.4),
                                       ("cooked_beef", 1, 3, 0.3), ("coal", 3, 9, 0.6), ("torch", 4, 12, 0.5),
                                       ("iron_pickaxe", 1, 1, 0.15), ("iron_shovel", 1, 1, 0.15), ("string", 2, 5, 0.3)]),
            ((2, 0, 40), (20, 5, 58), [("bone", 2, 6, 0.7), ("rotten_flesh", 1, 4, 0.6), ("iron_ingot", 1, 3, 0.4),
                                       ("iron_sword", 1, 1, 0.15), ("iron_helmet", 1, 1, 0.1), ("arrow", 4, 10, 0.4)]),
            ((40, 0, 40), (58, 9, 58), [("ender_pearl", 1, 4, 0.8), ("ender_eye", 1, 2, 0.4), ("obsidian", 2, 6, 0.4),
                                        ("diamond", 1, 2, 0.2), ("enchanted_book", 1, 1, 0.3), ("golden_apple", 1, 1, 0.12)]),
        ],
        "loot": [("", "ender_pearl", 1, 3, 0.5), ("", "ender_eye", 1, 2, 0.25), ("", "iron_ingot", 2, 6, 0.6),
                 ("", "bread", 1, 4, 0.5), ("", "diamond", 1, 2, 0.15), ("", "gold_ingot", 1, 4, 0.35),
                 ("", "enchanted_book", 1, 1, 0.3), ("", "book", 1, 3, 0.4), ("", "paper", 2, 6, 0.4),
                 ("", "golden_apple", 1, 1, 0.08), ("", "iron_sword", 1, 1, 0.1), ("", "compass", 1, 1, 0.1),
                 ("", "flame_tome", 1, 1, 0.06), ("", "mana_crystal", 1, 1, 0.2)],
    }
