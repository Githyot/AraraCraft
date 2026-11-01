#!/usr/bin/env python3
"""Lote B7 of the block review (plan/blocos-analise.md): flat block pictures get material.

The audit found block pictures with fewer than six colours in 64x64 (Terraria mosses, gem tree
leaves and logs, living wood, trap and station tops...). They read as flat stickers next to the
rest of the pack. This keeps each picture's design and colours but gives every colour area a
grain: soft clusters a shade lighter and darker (quantized, so the pixels stay crisp), and a
faint light from the top left. Pictures that already have six colours or more are left alone,
so running it again changes nothing.
"""

import os
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "tools"))
sys.path.insert(0, os.path.join(ROOT, "tools", "tex64"))
import gen_blocos_b2 as b2  # noqa: E402
from style import N, XX, YY, clusters, rng_for, value_noise, normalize  # noqa: E402

FLAT = """amethyst_gem_leaves anvil_top argon_moss_side argon_moss_top boulder_trap_top cactus_top dart_trap_top
diamond_gem_leaves dungeon_lantern_top emerald_gem_leaves extractinator_top fate_altar_top flame_trap_top gem_tree_log
gem_tree_log_top helium_moss_side helium_moss_top keg_side keg_top krypton_moss_side krypton_moss_top lava_moss_side
lava_moss_top living_leaves living_wood living_wood_top neon_moss_side neon_moss_top ruby_gem_leaves sapphire_gem_leaves
spear_trap spear_trap_top teleporter_side topaz_gem_leaves xenon_moss_side xenon_moss_top""".split()


def colours(img):
    op = img[img[..., 3] > 0][:, :3]
    return len(np.unique(op, axis=0)) if len(op) else 0


def enrich(name):
    img = b2.load(name)
    if colours(img) >= 6:
        return False
    rng = rng_for(name + "_b7")
    grain = normalize(clusters(rng, 4) * 0.7 + value_noise(rng, 8) * 0.3)
    step = np.select([grain < 0.3, grain > 0.72], [-1.0, 1.0], 0.0)
    # Light from the top left, in whole steps too, so each area keeps a few crisp shades.
    tilt = ((N - XX - YY) / N) * 0.6
    k = 1.0 + 0.075 * step + 0.04 * np.round(tilt * 2) / 2
    a = img[..., 3] > 0
    img[a, :3] = np.clip(img[a, :3] * k[a][:, None], 0, 255)
    b2.save(name, img)
    return True


def main():
    done = [n for n in FLAT if enrich(n)]
    print(f"{len(done)} pictures given grain: {' '.join(done)}")


if __name__ == "__main__":
    main()
