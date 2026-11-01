"""Arms the scaled temple (templo_escamado in packs/vanilla/tr/structures.ron): pressure plates
on its corridor floors with dart, spear and flame traps set in the wall beside them, and the
creatures that guard it. tools/gen_tr.py writes the temple; run this after it (gen_tr.py calls
it at the end). Running it twice changes nothing: an armed temple is left as it is."""
import random
import re
from pathlib import Path

RON = Path(__file__).resolve().parent.parent / "packs" / "vanilla" / "tr" / "structures.ron"
MOBS = '        mobs: [("jungle_bat", 3), ("giant_tortoise", 2), ("man_eater", 2)],\n'


def arm(text):
    i = text.index('name: "templo_escamado"')
    j = text.find("\n    ),", i)
    block = text[i:j]
    if "dart_trap" in block:
        return text
    m = re.search(r"layers: \[\n(.*?)\n        \],", block, re.S)
    lines = m.group(1).split("\n")
    layers = [[list(r) for r in re.findall(r'"([^"]*)"', ln)] for ln in lines]
    rnd = random.Random(1590)
    spots = []
    for y in range(1, len(layers) - 1):
        L, below = layers[y], layers[y - 1]
        for z in range(1, len(L) - 1):
            for x in range(1, len(L[z]) - 1):
                if L[z][x] != "." or below[z][x] != "B":
                    continue
                near = [L[z + dz][x + dx] for dz in (-1, 0, 1) for dx in (-1, 0, 1)]
                if any(c in "AKLl" for c in near):
                    continue
                # A wall beside it, with the corridor running past (air on the far side).
                for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    wx, wz = x + dx, z + dz
                    if L[wz][wx] == "B" and L[z - dx if dz == 0 else z][x - dz if dx == 0 else x] == ".":
                        spots.append((y, z, x, wz, wx))
                        break
    rnd.shuffle(spots)
    used = []
    traps = "DSF"
    for (y, z, x, wz, wx) in spots:
        if len(used) >= 14:
            break
        if any(abs(y - u[0]) < 2 and abs(z - u[1]) + abs(x - u[2]) < 6 for u in used):
            continue
        layers[y][z][x] = "P"
        layers[y][wz][wx] = traps[len(used) % 3]
        used.append((y, z, x))
    new_lines = ["            [" + ", ".join('"' + "".join(r) + '"' for r in L) + "]," for L in layers]
    block = block[: m.start(1)] + "\n".join(new_lines) + block[m.end(1):]
    block = block.replace("('L', \"ladder\")]", "('L', \"ladder\"), ('P', \"stone_pressure_plate\"), ('D', \"dart_trap\"), ('S', \"spear_trap\"), ('F', \"flame_trap\")]", 1)
    if "mobs:" not in block:
        k = block.index("        palette:")
        block = block[:k] + MOBS + block[k:]
    print(f"templo_escamado: {len(used)} traps")
    return text[:i] + block + text[j:]


def harpies(text):
    """Harpies circle the small sky islands, as in Terraria."""
    i = text.index('name: "ilha_do_ceu"')
    j = text.find("\n    ),", i)
    block = text[i:j]
    if "mobs:" in block:
        return text
    k = block.index("        palette:")
    return text[:i] + block[:k] + '        mobs: [("harpy", 2)],\n' + block[k:] + text[j:]


def all_fixes(text):
    import sky_island
    return sky_island.add(harpies(arm(text)))


if __name__ == "__main__":
    RON.write_text(all_fixes(RON.read_text()))
