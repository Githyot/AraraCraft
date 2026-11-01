# World sounds: credits and licences

Real recordings for blocks (break, place, mining), footsteps, tools at work, weapon swings,
blows on bodies and people's voices. Built by `tools/sounds/build_world_sounds.py` (the
script lists exactly which source files go into which group); `crates/game/src/sound_bank.rs`
embeds them. Every file was **modified**: made mono 44.1 kHz, leading and trailing silence
trimmed, cut short with a fade, low-passed (nothing shrill) and peak-normalised; the game also
changes speed, loudness and filter at play time and mixes some together. None of them come
from Minecraft or Terraria.

Licences used: [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/),
[CC BY 3.0](https://creativecommons.org/licenses/by/3.0/),
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/),
[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The files that come from a
CC BY-SA source stay under that same licence (as modified here); this folder as a whole may be
shared under CC BY-SA 3.0 with the attributions below.

## 0 A.D. (Wildfire Games): CC BY-SA 3.0

Copyright (C) Wildfire Games, <http://www.wildfiregames.com/>, from
<https://github.com/0ad/0ad> `binaries/data/mods/public/audio`, licensed
[CC BY-SA 3.0](http://creativecommons.org/licenses/by-sa/3.0/):

- footsteps `actor/singlesteps/fs_rock*`, `fs_wood*`, `fs_grass*`, `fs_gravel*`, `fs_sand*`,
  `fs_snow_*` and `actor/human/movement/hstep_dirt_MN_*` (in `stone_step`, `wood_step`,
  `grass_step`, `gravel_step`, `sand_step`, `snow_step`, `dirt_step`)
- `resource/mining/mine_stone_*`, `mine_metal_*` (`tool_pick_stone`, `tool_pick_metal`),
  `resource/lumbering/lumber_tree_*` (`tool_axe`), `resource/farming/hoe_*` (`tool_hoe`),
  `resource/foraging/leaves_*` (in `leaves_hit`)
- `attack/weapon/sword_attack_*` (`swing_sword`), `spear_attack_*` (`swing_spear`),
  `pike_attack_*` (`swing_heavy`), `jav_attack_*` (in `swing_light`)
- `attack/impact/fleshimp_*`, `fleshstab_*` (`impact_flesh`), `shield_metal_*`
  (`impact_metal`), `shield_wood_*` (`impact_wood`), `sling_stone_*` (`impact_stone`)
- `actor/human/death/male_death_*`, `female_death_*` (`voice_male_*`, `voice_female_*`)

## Everness (Luanti game mod by SaKeL), sounds by Freesound artists

From <https://github.com/asuna-mt/everness> `sounds/` (files named `everness_*` there),
licences and authors as listed in its LICENSE.txt:

**CC0-1.0, by ProdMultimediasHQI**, https://freesound.org/people/ProdMultimediasHQI/sounds/512501/
- frosted_snow_footstep.1.ogg
- frosted_snow_footstep.2.ogg
- frosted_snow_footstep.3.ogg
- frosted_snow_footstep.4.ogg
- frosted_snow_footstep.5.ogg

**CC0-1.0, by patchytherat**, https://freesound.org
- crystal_chime.1.ogg
- crystal_chime.2.ogg
- crystal_chime.3.ogg
- crystal_chime.4.ogg
- crystal_chime.5.ogg
- crystal_chime.6.ogg

**CC0-1.0, by afleetingspeck**, https://freesound.org
- bamboo_hit.1.ogg
- bamboo_hit.2.ogg
- bamboo_hit.3.ogg
- bamboo_hit.4.ogg
- bamboo_hit.5.ogg

**CC0-1.0, by se2001**, https://freesound.org
- bamboo_dug.1.ogg

**CC0-1.0, by Foleyhaven**, https://freesound.org
- mud_footstep.1.ogg
- mud_footstep.2.ogg
- mud_footstep.3.ogg
- mud_footstep.4.ogg
- mud_footstep.5.ogg

**CC0-1.0, by Yuval**, https://freesound.org
- grass_footstep.1.ogg
- grass_footstep.2.ogg
- grass_footstep.3.ogg

**CC0-1.0, by kyles**, https://freesound.org
- dirt_hit.1.ogg
- dirt_hit.2.ogg
- dirt_hit.3.ogg

**CC0-1.0, by Nox_Sound**, https://freesound.org
- grass_hit.1.ogg
- grass_hit.3.ogg
- grass_hit.2.ogg

**CC0-1.0, by Nox_Sound**, https://freesound.org/people/Nox_Sound/sounds/613849/
- snow_footstep.1.ogg
- snow_footstep.2.ogg
- snow_footstep.3.ogg
- snow_footstep.4.ogg

**CC0-1.0, by Fission9**, https://freesound.org
- dirt_footstep.1.ogg
- dirt_footstep.2.ogg
- dirt_footstep.3.ogg
- dirt_footstep.4.ogg
- wood_footstep.1.ogg
- wood_footstep.2.ogg
- wood_footstep.3.ogg
- wood_footstep.4.ogg
- wood_footstep.5.ogg
- wood_footstep.6.ogg

**CC-BY-4.0, by InspectorJ**, https://freesound.org
- ice_footstep.1.ogg
- ice_footstep.2.ogg
- ice_footstep.3.ogg
- ice_footstep.4.ogg

**CC-BY-4.0, by EminYILDIRIM**, https://freesound.org
- ice_hit.1.ogg
- ice_hit.2.ogg
- ice_hit.3.ogg
- ice_hit.4.ogg

**CC0-1.0, by Nox_Sound**, https://freesound.org
- stone_dug.1.ogg
- stone_dug.2.ogg
- stone_dug.3.ogg
- stone_footstep.1.ogg
- stone_footstep.2.ogg
- stone_footstep.3.ogg
- stone_footstep.4.ogg
- stone_footstep.5.ogg

**CC0-1.0, by lolamadeus**, https://freesound.org
- stone_hit.1.ogg
- stone_hit.2.ogg
- stone_hit.3.ogg

**CC-BY-4.0, by F.M.Audio**, https://freesound.org
- stone_place.1.ogg
- stone_place.2.ogg
- stone_place.3.ogg
- stone_place.4.ogg

**CC-BY-4.0, by elektroproleter**, https://freesound.org
- leaves_footstep.1.ogg
- leaves_footstep.2.ogg
- leaves_footstep.3.ogg
- leaves_footstep.4.ogg

**CC-BY-4.0, by Abacagi**, https://freesound.org
- leaves_hit.1.ogg
- leaves_hit.2.ogg

**CC-BY-4.0, by JanKoehl**, https://freesound.org
- leaves_dug.1.ogg
- leaves_dug.2.ogg

**CC0-1.0, by vgraham1**, https://freesound.org
- leaves_place.1.ogg
- leaves_place.2.ogg

**CC0-1.0, by worthahep88**, https://freesound.org
- plant_hit.1.ogg
- plant_hit.2.ogg
- plant_hit.3.ogg

**CC0-1.0, by GiocoSound**, https://freesound.org/people/GiocoSound/
- wood_hit.1.ogg
- wood_hit.2.ogg
- wood_hit.3.ogg
- wood_hit.4.ogg
- wood_hit.5.ogg
- wood_hit.6.ogg

**CC-BY-4.0, by RICHERlandTV**, https://freesound.org
- wood_place.1.ogg
- wood_place.2.ogg
- wood_place.3.ogg

**CC0-1.0, by pan14**, https://freesound.org
- sand_footstep.1.ogg
- sand_footstep.2.ogg
- sand_footstep.3.ogg

**CC-BY-3.0, by Coral_Island_Studios**, https://freesound.org
- sand_place.1.ogg
- sand_place.2.ogg
- sand_place.3.ogg
- sand_place.4.ogg
- sand_place.5.ogg

**CC0-1.0, by schneidi67**, https://freesound.org
- sand_dug.1.ogg
- sand_dug.2.ogg
- sand_dug.3.ogg

**CC0-1.0, by Lukeo135**, https://freesound.org
- sand_hit.1.ogg
- sand_hit.2.ogg
- sand_hit.3.ogg

**CC0-1.0, by GiocoSound**, https://freesound.org
- metal_footstep.1.ogg
- metal_footstep.2.ogg
- metal_footstep.3.ogg
- metal_footstep.4.ogg
- metal_footstep.5.ogg

**CC-BY-4.0, by DasDeer**, https://freesound.org
- metal_hit.1.ogg
- metal_hit.2.ogg
- metal_hit.3.ogg

**CC-BY-4.0, by MoveAwayPodcast**, https://freesound.org
- metal_place.1.ogg
- metal_place.2.ogg
- metal_place.3.ogg

**CC0-1.0, by craigsmith**, https://freesound.org
- metal_dug.1.ogg
- metal_dug.2.ogg
- metal_dug.3.ogg

**CC0-1.0, by courtneyeck**, https://freesound.org
- glass_footstep.1.ogg
- glass_footstep.2.ogg
- glass_footstep.3.ogg

**CC0-1.0, by Robson220pl**, https://freesound.org
- glass_place.1.ogg
- glass_place.2.ogg
- glass_place.3.ogg

**CC-BY-4.0, by elfgrunge**, https://freesound.org
- glass_dug.1.ogg
- glass_dug.2.ogg

**CC-BY-3.0, by Mito551**, https://freesound.org
- thin_glass_footstep.ogg

**CC-BY-3.0, by cmusounddesign**, http://www.freesound.org/people/cmusounddesign/sounds/71947/
- break_thin_glass.1.ogg

**CC-BY-3.0, by Tomlija**, http://www.freesound.org/people/Tomlija/sounds/97669/
- break_thin_glass.2.ogg

**CC-BY-3.0, by lsprice**, http://www.freesound.org/people/lsprice/sounds/88808/
- break_thin_glass.3.ogg

**CC-BY-3.0, by alexo400**, https://freesound.org/people/alexo400/sounds/543385/
- snow_place.1.ogg
- snow_place.2.ogg
- snow_place.3.ogg

**CC-BY-3.0, by soundscalpel.com**
- snow_hit.1.ogg
- snow_hit.2.ogg
- snow_hit.3.ogg

**CC-BY-3.0, by soundscalpel.com**
- frosted_snow_place.1.ogg
- frosted_snow_place.2.ogg
- frosted_snow_place.3.ogg

**CC-BY-3.0, by AtevonHes**, https://freesound.org/people/AtevonHes/sounds/454783/
- frosted_snow_hit.1.ogg
- frosted_snow_hit.2.ogg
- frosted_snow_hit.3.ogg
- frosted_snow_hit.4.ogg

**CC-BY-4.0, by jorickhoofd**, https://freesound.org/people/jorickhoofd/
- gravel_hit.1.ogg
- gravel_hit.2.ogg
- gravel_hit.3.ogg
- gravel_place.1.ogg
- gravel_place.2.ogg
- gravel_place.3.ogg

**CC-BY-3.0, by test_sound**, https://freesound.org/people/test_sound/
- gravel_dug.1.ogg
- gravel_dug.2.ogg
- gravel_dug.3.ogg
- gravel_dug.4.ogg

**CC0-1.0, by Ali_6868**, https://freesound.org/people/Ali_6868/
- gravel_footstep.1.ogg
- gravel_footstep.2.ogg
- gravel_footstep.3.ogg
- gravel_footstep.4.ogg
- gravel_footstep.5.ogg
- gravel_footstep.6.ogg

**CC-BY-4.0, by nuFF3**, https://freesound.org
- ceramic_footstep.1.ogg
- ceramic_footstep.2.ogg
- ceramic_footstep.3.ogg
- ceramic_footstep.4.ogg
- ceramic_footstep.5.ogg
- ceramic_footstep.6.ogg

**CC-BY-4.0, by Anikadl**, https://freesound.org
- ceramic_hit.1.ogg
- ceramic_hit.2.ogg
- ceramic_hit.3.ogg
- ceramic_hit.4.ogg
- ceramic_hit.5.ogg
- ceramic_hit.6.ogg

**CC0-1.0, by kyles**, https://freesound.org, mixed with default_place_node_hard.1.ogg by Mito551 (CC-BY-SA 3.0)
- ceramic_place.1.ogg
- ceramic_place.2.ogg
- ceramic_place.3.ogg

**CC0-1.0, by Kinoton**, https://freesound.org, mixed with default_hard_footstep.1.ogg by Erdie (CC-BY-SA 3.0)
- ceramic_dug.1.ogg
- ceramic_dug.2.ogg
- ceramic_dug.3.ogg

## Minetest Game and VoxeLibre

From <https://github.com/luanti-org/minetest_game> `mods/default/sounds` (files named
`default_*` there) and <https://github.com/VoxeLibre/VoxeLibre> `mods/CORE/mcl_sounds/sounds`.
Licences of these: CC BY-SA 3.0 (<https://creativecommons.org/licenses/by-sa/3.0/>),
CC BY 3.0, CC BY-SA 4.0 and CC0, per file:

- `break_glass.*`: cmusounddesign <http://www.freesound.org/people/cmusounddesign/sounds/71947/>,
  Tomlija <http://www.freesound.org/people/Tomlija/sounds/97669/>, lsprice
  <http://www.freesound.org/people/lsprice/sounds/88808/> (CC BY 3.0)
- `dig_crumbly`, `dug_node.*`, `grass_footstep.*`, `gravel_footstep.*`, `place_node.*`,
  `place_node_hard.*`, `wood_footstep.*`, `dirt_footstep.*`, `glass_footstep`: Mito551 (CC BY-SA 3.0)
- `dig_metal`: yadronoff <https://www.freesound.org/people/yadronoff/sounds/320397/> (CC BY 3.0)
- `dug_metal.*`: Iwan Gabovitch (qubodup) (CC0)
- `metal_footstep.*`: mypantsfelldown <https://freesound.org/people/mypantsfelldown/sounds/398937/> (CC0)
- `place_node_metal.*`: Ogrebane <http://opengameart.org/content/wood-and-metal-sound-effects-volume-2> (CC0)
- `dig_snappy`: blukotek <https://www.freesound.org/people/blukotek/sounds/251660/> (CC0)
- `dig_choppy.*`: Sheyvan <https://freesound.org/people/Sheyvan/sounds/476113/> (CC0)
- `gravel_dig.*`: lolamadeus <https://freesound.org/people/lolamadeus/sounds/179341/> (CC0)
- `dig_cracky.*`: Benboncan <https://freesound.org/people/Benboncan/sounds/71823/> (CC BY 3.0)
- `hard_footstep.*`: Erdie <https://freesound.org/people/Erdie/sounds/41579/> (CC BY 3.0)
- `sand_footstep.*`: worthahep88 <https://freesound.org/people/worthahep88/sounds/319224/> (CC0)
- `ice_dig.*`: dheming <https://freesound.org/people/dheming/sounds/268023/> (CC BY 3.0)
- `ice_footstep.*`: InspectorJ <https://freesound.org/people/InspectorJ/sounds/416967/> (CC BY 3.0)
- `ice_dug`: Angel_Perez_Grandi <https://freesound.org/people/Angel_Perez_Grandi/sounds/49190/> (CC BY 3.0)
- `snow_footstep.*`: Minetest Game contributors (CC BY-SA 3.0)
- `player_falling_damage` (our `impact_fall`): Adam_N <https://www.freesound.org/people/Adam_N/sounds/346692/> (CC0)
- `mcl_sounds_cloth.*` (in `cloth_rustle`): Alecia Shepherd, SnowSong pack
  <https://opengameart.org/content/snowsong-sound-and-music-pack> (CC BY-SA 4.0)
- `pedology_snow_soft_footstep.*` (in `snow_step`): unknown authors via VoxeLibre (CC BY-SA 3.0)

## Kenney: CC0

[Impact Sounds](https://kenney.nl/assets/impact-sounds) and
[RPG Audio](https://kenney.nl/assets/rpg-audio) by Kenney (www.kenney.nl), public domain
(CC0), taken from the Godot ports <https://github.com/Boyquotes/kenney-impact-sounds-for-godot>
and <https://github.com/Boyquotes/kenney-rpg-audio-for-godot>: footsteps on wood, grass, snow
and carpet, glass, metal, wood, plank, soft, punch and mining impacts, cloth, chop, knife
slices and draws.
