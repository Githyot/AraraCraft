#!/usr/bin/env python3
"""Builds assets/sounds/world/ (block, footstep, tool, swing, impact and voice recordings) and
crates/game/src/sound_bank.rs from free recordings (credits: assets/sounds/world/CREDITS.md).

Usage: build_world_sounds.py SRC_DIR
SRC_DIR holds shallow git clones of:
  oad  = github.com/0ad/0ad (binaries/data/mods/public/audio, CC BY-SA 3.0, Wildfire Games)
  ev   = github.com/asuna-mt/everness (sounds/, CC0 / CC BY 3.0 / CC BY 4.0, see its LICENSE.txt)
  mtg  = github.com/luanti-org/minetest_game (mods/default/sounds, CC BY-SA 3.0 / CC BY 3.0 / CC0)
  vl   = github.com/VoxeLibre/VoxeLibre (mods/CORE/mcl_sounds/sounds)
  kimp = github.com/Boyquotes/kenney-impact-sounds-for-godot (Kenney Impact Sounds, CC0)
  krpg = github.com/Boyquotes/kenney-rpg-audio-for-godot (Kenney RPG Audio, CC0)

Every clip is made mono 44.1 kHz, leading silence trimmed, cut to a maximum length with a
short fade, gently low-passed (nothing shrill) and peak-normalised. The game picks one at
random each time and changes its speed a little, so repeats never sound the same.
"""
import glob, os, subprocess, sys

SRC = sys.argv[1]
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets/sounds/world")
BANK = os.path.join(ROOT, "crates/game/src/sound_bank.rs")

OAD = "oad/binaries/data/mods/public/audio/"
EV = "ev/sounds/everness_"
MTG = "mtg/mods/default/sounds/default_"
VL = "vl/mods/CORE/mcl_sounds/sounds/"
KI = "kenney-impact-sounds-for-godot/addons/kenney impact sounds/"
KR = "kenney-rpg-audio-for-godot/addons/kenney rpg audio/"

# (output name prefix, source glob(s), max seconds, low-pass Hz)
GROUPS = [
    # --- stone, bricks and ceramics ---
    ("stone_step", [EV + "stone_footstep.*.ogg", MTG + "hard_footstep.*.ogg", OAD + "actor/singlesteps/fs_rock[1-8].ogg"], 0.5, 6000),
    ("stone_hit", [EV + "stone_hit.*.ogg", MTG + "dig_cracky.*.ogg"], 0.5, 6000),
    ("stone_dug", [EV + "stone_dug.*.ogg"], 0.8, 6000),
    ("stone_place", [EV + "stone_place.*.ogg", MTG + "place_node_hard.*.ogg"], 0.5, 6000),
    ("ceramic_step", [EV + "ceramic_footstep.*.ogg"], 0.5, 6000),
    ("ceramic_hit", [EV + "ceramic_hit.*.ogg"], 0.5, 6000),
    ("ceramic_dug", [EV + "ceramic_dug.*.ogg"], 0.8, 6000),
    ("ceramic_place", [EV + "ceramic_place.*.ogg"], 0.5, 6000),
    # --- wood ---
    ("wood_step", [EV + "wood_footstep.*.ogg", MTG + "wood_footstep.*.ogg", OAD + "actor/singlesteps/fs_wood[1-4].ogg", KI + "footstep_wood_00[0-4].ogg"], 0.5, 6000),
    ("wood_hit", [EV + "wood_hit.*.ogg", MTG + "dig_choppy.*.ogg"], 0.5, 6000),
    ("wood_dug", [KI + "impact_wood_heavy_00[0-4].ogg"], 0.8, 6000),
    ("wood_place", [EV + "wood_place.*.ogg", KI + "impact_plank_medium_00[0-4].ogg"], 0.5, 6000),
    # --- soil ---
    ("grass_step", [EV + "grass_footstep.*.ogg", MTG + "grass_footstep.*.ogg", OAD + "actor/singlesteps/fs_grass[1-8].ogg", KI + "footstep_grass_00[0-4].ogg"], 0.5, 6000),
    ("grass_hit", [EV + "grass_hit.*.ogg", MTG + "dig_crumbly.ogg"], 0.5, 6000),
    ("soil_dug", [MTG + "dug_node.*.ogg"], 0.8, 6000),
    ("soil_place", [MTG + "place_node.*.ogg"], 0.5, 6000),
    ("dirt_step", [EV + "dirt_footstep.*.ogg", MTG + "dirt_footstep.*.ogg", OAD + "actor/human/movement/hstep_dirt_MN_1[1-3].ogg"], 0.5, 6000),
    ("dirt_hit", [EV + "dirt_hit.*.ogg"], 0.5, 6000),
    ("gravel_step", [EV + "gravel_footstep.*.ogg", MTG + "gravel_footstep.*.ogg", OAD + "actor/singlesteps/fs_gravel[1-8].ogg"], 0.5, 6000),
    ("gravel_hit", [EV + "gravel_hit.*.ogg", MTG + "gravel_dig.*.ogg"], 0.5, 6000),
    ("gravel_dug", [EV + "gravel_dug.*.ogg"], 0.8, 4500),
    ("gravel_place", [EV + "gravel_place.*.ogg"], 0.5, 6000),
    ("sand_step", [EV + "sand_footstep.*.ogg", MTG + "sand_footstep.*.ogg", OAD + "actor/singlesteps/fs_sand[1-8].ogg"], 0.5, 6000),
    ("sand_hit", [EV + "sand_hit.*.ogg"], 0.5, 4500),
    ("sand_dug", [EV + "sand_dug.*.ogg"], 0.8, 4500),
    ("sand_place", [EV + "sand_place.*.ogg"], 0.5, 6000),
    ("mud_step", [EV + "mud_footstep.*.ogg"], 0.6, 5000),
    # --- snow and ice ---
    ("snow_step", [EV + "snow_footstep.*.ogg", MTG + "snow_footstep.*.ogg", OAD + "actor/singlesteps/fs_snow_1[1-8].ogg", KI + "footstep_snow_00[0-4].ogg", VL + "pedology_snow_soft_footstep.*.ogg"], 0.5, 6000),
    ("snow_hit", [EV + "snow_hit.*.ogg", EV + "frosted_snow_hit.*.ogg"], 0.5, 6000),
    ("snow_place", [EV + "snow_place.*.ogg", EV + "frosted_snow_place.*.ogg"], 0.5, 6000),
    ("crust_step", [EV + "frosted_snow_footstep.*.ogg"], 0.5, 6000),
    ("ice_step", [EV + "ice_footstep.*.ogg", MTG + "ice_footstep.*.ogg"], 0.5, 6000),
    ("ice_hit", [EV + "ice_hit.*.ogg", MTG + "ice_dig.*.ogg"], 0.5, 6000),
    ("ice_dug", [MTG + "ice_dug.ogg"], 0.8, 6000),
    # --- glass and crystal ---
    ("glass_step", [EV + "glass_footstep.*.ogg", MTG + "glass_footstep.ogg", EV + "thin_glass_footstep.ogg"], 0.5, 6000),
    ("glass_hit", [KI + "impact_glass_light_00[0-4].ogg"], 0.5, 5000),
    ("glass_break", [MTG + "break_glass.*.ogg", EV + "break_thin_glass.*.ogg", EV + "glass_dug.*.ogg"], 0.9, 4500),
    ("glass_place", [EV + "glass_place.*.ogg"], 0.5, 5500),
    ("crystal_chime", [EV + "crystal_chime.*.ogg"], 1.0, 4000),
    # --- metal ---
    ("metal_step", [EV + "metal_footstep.*.ogg", MTG + "metal_footstep.*.ogg"], 0.5, 6000),
    ("metal_hit", [EV + "metal_hit.*.ogg", MTG + "dig_metal.ogg", KI + "impact_metal_light_00[0-4].ogg"], 0.5, 5500),
    ("metal_dug", [EV + "metal_dug.*.ogg", MTG + "dug_metal.*.ogg"], 0.8, 5500),
    ("metal_place", [EV + "metal_place.*.ogg", MTG + "place_node_metal.*.ogg"], 0.5, 5500),
    ("metal_heavy", [KI + "impact_metal_heavy_00[0-4].ogg"], 0.7, 5000),
    # --- soft things ---
    ("cloth_step", [KI + "footstep_carpet_00[0-4].ogg"], 0.5, 6000),
    ("cloth_rustle", [VL + "mcl_sounds_cloth.*.ogg", KR + "cloth_[1-4].ogg"], 0.6, 6000),
    ("soft_hit", [KI + "impact_soft_medium_00[0-4].ogg"], 0.5, 5000),
    ("soft_heavy", [KI + "impact_soft_heavy_00[0-4].ogg"], 0.6, 5000),
    ("leaves_step", [EV + "leaves_footstep.*.ogg"], 0.5, 4000),
    ("leaves_hit", [EV + "leaves_hit.*.ogg", OAD + "resource/foraging/leaves_0[1-8].ogg"], 0.5, 4000),
    ("leaves_dug", [EV + "leaves_dug.*.ogg"], 0.8, 4000),
    ("leaves_place", [EV + "leaves_place.*.ogg"], 0.5, 4000),
    ("plant_hit", [EV + "plant_hit.*.ogg", MTG + "dig_snappy.ogg"], 0.5, 4000),
    ("bamboo_hit", [EV + "bamboo_hit.*.ogg", EV + "bamboo_dug.1.ogg"], 0.5, 6000),
    ("bone_knock", [KI + "impact_wood_light_00[0-4].ogg"], 0.4, 5500),
    # --- tools at work ---
    ("tool_pick_stone", [OAD + "resource/mining/mine_stone_0[1-9].ogg", OAD + "resource/mining/mine_stone_1[0-2].ogg"], 0.35, 5000),
    ("tool_pick_metal", [OAD + "resource/mining/mine_metal_0[1-8].ogg"], 0.4, 4500),
    ("tool_axe", [OAD + "resource/lumbering/lumber_tree_0[1-9].ogg", OAD + "resource/lumbering/lumber_tree_1[0-2].ogg"], 0.45, 5000),
    ("tool_hoe", [OAD + "resource/farming/hoe_1[0-5].ogg"], 0.45, 5000),
    ("tool_chop", [KR + "chop.ogg"], 0.5, 5000),
    ("tool_mining", [KI + "impact_mining_00[0-4].ogg"], 0.5, 5000),
    # --- swings ---
    ("swing_sword", [OAD + "attack/weapon/sword_attack_*.ogg"], 0.6, 5000),
    ("swing_spear", [OAD + "attack/weapon/spear_attack_*.ogg"], 0.6, 5000),
    ("swing_heavy", [OAD + "attack/weapon/pike_attack_*.ogg"], 0.6, 5000),
    ("swing_light", [OAD + "attack/weapon/jav_attack_*.ogg", KR + "knife_slice*.ogg"], 0.45, 4000),
    ("swing_draw", [KR + "draw_knife_*.ogg"], 0.6, 4000),
    # --- bodies being hit ---
    ("impact_flesh", [OAD + "attack/impact/fleshimp_1[0-2].ogg", OAD + "attack/impact/fleshstab_0[1-3].ogg"], 0.4, 4500),
    ("impact_punch", [KI + "impact_punch_medium_00[0-4].ogg", KI + "impact_punch_heavy_00[0-4].ogg"], 0.4, 4500),
    ("impact_metal", [OAD + "attack/impact/shield_metal_0[1-9].ogg"], 0.5, 4500),
    ("impact_wood", [OAD + "attack/impact/shield_wood_0[1-9].ogg"], 0.5, 4500),
    ("impact_stone", [OAD + "attack/impact/sling_stone_0[1-8].ogg"], 0.4, 4500),
    ("impact_fall", [VL + "player_falling_damage.ogg"], 0.5, 4000),
    # --- people ---
    ("voice_male", [OAD + "actor/human/death/male_death_*.ogg"], 1.3, 4000),
    ("voice_female", [OAD + "actor/human/death/female_death_*.ogg"], 1.3, 4000),
]


def convert(src, dst, maxlen, lowpass):
    import numpy as np
    rate = 44100
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", src, "-ac", "1", "-ar", str(rate),
         "-af", f"highpass=f=45,lowpass=f={lowpass},lowpass=f={lowpass}", "-f", "f32le", "-"],
        capture_output=True, check=True).stdout
    x = np.frombuffer(raw, dtype=np.float32).astype(np.float64)
    if len(x) == 0 or np.abs(x).max() < 1e-6:
        raise SystemExit(f"silent source {src}")
    x /= np.abs(x).max()
    # Leading silence off (below -40 dB of the peak), a few ms kept before the onset.
    on = int(np.argmax(np.abs(x) > 0.01))
    x = x[max(0, on - int(0.004 * rate)):]
    # Trailing silence off too, then the length cap.
    loud = np.nonzero(np.abs(x) > 0.004)[0]
    x = x[: loud[-1] + int(0.03 * rate)]
    x = x[: int(maxlen * rate)]
    fade = min(int(0.05 * rate), len(x) // 3)
    x[-fade:] *= np.linspace(1.0, 0.0, fade)
    head = min(int(0.002 * rate), len(x) // 4)
    x[:head] *= np.linspace(0.0, 1.0, head)
    x *= 0.89 / max(np.abs(x).max(), 1e-6)
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ar", str(rate), "-ac", "1", "-i", "-",
         "-c:a", "libvorbis", "-q:a", "2", dst],
        input=x.astype(np.float32).tobytes(), check=True)


def duration(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path], capture_output=True, text=True).stdout
    return float(out.strip() or 0)


def main():
    os.makedirs(OUT, exist_ok=True)
    for f in glob.glob(os.path.join(OUT, "*.ogg")):
        os.remove(f)
    names = []
    for prefix, pats, maxlen, lp in GROUPS:
        files = []
        for p in pats:
            files += sorted(glob.glob(os.path.join(SRC, p)))
        if not files:
            raise SystemExit(f"no sources for {prefix}: {pats}")
        if prefix.startswith("voice_"):
            # Voices split by length: the shortest two fifths are grunts (hurt), the rest
            # cries (death).
            tmp = []
            for i, f in enumerate(files, 1):
                path = os.path.join(OUT, f"tmp_{i}.ogg")
                convert(f, path, maxlen, lp)
                tmp.append((duration(path), path))
            tmp.sort()
            cut = max(3, len(tmp) * 2 // 5)
            for k, (_, path) in enumerate(tmp):
                kind, n = ("hurt", k + 1) if k < cut else ("death", k - cut + 1)
                name = f"{prefix}_{kind}_{n}"
                os.rename(path, os.path.join(OUT, name + ".ogg"))
                names.append(name)
            print(f"{prefix}: {cut} hurt, {len(tmp) - cut} death")
            continue
        for i, f in enumerate(files, 1):
            name = f"{prefix}_{i}"
            convert(f, os.path.join(OUT, name + ".ogg"), maxlen, lp)
            names.append(name)
        print(f"{prefix}: {len(files)}")
    with open(BANK, "w") as b:
        b.write("//! Generated by tools/sounds/build_world_sounds.py: every recording in assets/sounds/world/\n")
        b.write("//! (credits in assets/sounds/world/CREDITS.md). Do not edit by hand.\n\n")
        b.write("pub static CLIPS: &[(&str, &[u8])] = &[\n")
        for n in names:
            b.write(f'    ("{n}", include_bytes!("../../../assets/sounds/world/{n}.ogg")),\n')
        b.write("];\n")
    print(len(names), "clips")


main()
