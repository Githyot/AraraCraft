# Monster voices: credits and licences

Real recordings for the hostile mobs and bosses (crates/game/src/monster_sounds.rs). None of them
come from Minecraft or Terraria. tools/monster_sounds/build.py took them from the free games below,
converted them to mono 22 kHz, trimmed the silence, cut them to at most 3 s, low-passed them at
5 kHz (nothing shrill) and evened their loudness; the game also changes their pitch per monster.
Each file's name starts with its source (`vl_`, `flare_`, `mg_`, `oc_`, `lug_`, `cdda_`) and keeps
the original name after it. These modified versions are shared under the same licence as their
originals.

## VoxeLibre (`vl_*`)
<https://github.com/VoxeLibre/VoxeLibre>, `mods/ENTITIES/mobs_mc/sounds`, credited in its
`LICENSE-media.md`. Files already in assets/sounds/mobs keep the credits listed there (zombie,
skeleton, spider, slime, enderman, guardian, squid, wolf, magma cube, fire elemental, witch, vex,
horse). The others used here:
- `vl_creeper_*`, `vl_bat_hurt_*`, `vl_bat_death`, `vl_endermite_*`, `vl_zombiepig_*`,
  `vl_snowman_*`, `vl_wither_spawn`, `vl_magma_cube_attack`, `vl_green_slime_attack`: CC0
  (rubberduck, kyles, bevibeldesign, Randomation and others named there)
- `vl_guardian_hurt_*`: CC BY 3.0 (see LICENSE-media.md)
- `vl_silverfish_*`: CC BY-SA 4.0 (see LICENSE-media.md)
- `vl_ender_dragon_*`: NPXcoot, CC BY-SA 4.0
- `vl_fireball`, `vl_pig`, `vl_pig_angry`: TenPlus1, mobs_monster / mobs_animal, MIT

## Flare (`flare_*`)
Flare game, fantasycore mod, `soundfx/enemies` (antlion, goblin, minotaur, skeleton, wyvern,
zombie), <https://github.com/flareteam/flare-game>. CC BY-SA 3.0, Flare Team
(<https://flarerpg.org>).

## MegaGlest (`mg_*`)
MegaGlest game data, megapack (daemon, golem, dragon, evil dragon, behemoth, ghost armour,
genie, mummy, initiate, summoner, battlemage, shaman, valkyrie, princess, swordman, guard,
stickfighter, battleaxe, eagle, thunderbird, snake basket, battle machine, catapult,
ornithopter, tower, air), <https://github.com/MegaGlest/megaglest-data>.
CC BY-SA 3.0 Unported, the MegaGlest team and Glest contributors (see their docs/AUTHORS).

## OpenClonk (`oc_*`)
<https://github.com/openclonk/openclonk>, `planet/Sound.ocg/Animals.ocg`. CC BY 3.0:
- `oc_monster_*`: Ringwaul
- `oc_bat_*`, `oc_mooq_*`: ala
- `oc_puka_*`, `oc_zap_*`, `oc_chippie_bite*`: the OpenClonk team
- `oc_fish_munch1`: Sven2, adapted from Munch1 by fresco (<https://freesound.org/people/fresco/sounds/48933/>)
- `oc_mosquitobuzz`: Zywx (<https://freesound.org/people/Zywx/sounds/188708/>), CC0

## Lugaru (`lug_*`)
Lugaru HD by Wolfire Games, <https://github.com/osslugaru/lugaru>, `Data/Sounds`.
CC BY-SA 3.0 Unported, Wolfire Games (<http://www.wolfire.com>).

## Cataclysm: DDA sound packs (`cdda_*`)
CC-Sounds by Fris0uman and contributors, <https://github.com/Fris0uman/CDDA-Soundpacks>,
`sound/CC-Sounds` (zombie groans and deaths, bites, monster hits, feral voices).
CC BY-SA 4.0; each folder's credits.md there names the original Freesound authors.
