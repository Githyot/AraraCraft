# Creature sounds: credits and licences

These are real recordings, taken from the free game VoxeLibre
(<https://github.com/VoxeLibre/VoxeLibre>, `mods/ENTITIES/mobs_mc/sounds`), which collected them
from Freesound, OpenGameArt and Luanti mods. They were converted to mono, gently low-passed
(6 kHz) and trimmed (the shrillest squeaks were left out); the game also changes their pitch at play time. None of them come from
Minecraft. Files here drop the `mobs_mc_` / `mobs_` prefix and use `_N` for VoxeLibre's `.N`.

Licences: [CC0](https://creativecommons.org/publicdomain/zero/1.0/),
[CC BY 3.0](https://creativecommons.org/licenses/by/3.0/),
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) and MIT (text below).
Every file not listed below is CC0.

## CC BY (attribution)
- `cow_hurt` (CC0, heavily modified): Bird_man, <https://freesound.org/people/Bird_man/packs/16972/>
- `sheep`: Blender Foundation (CC BY 3.0)
- `green_slime_jump`, `green_slime_land`, `green_slime_death`: Dr. Minky, <https://freesound.org/people/DrMinky/> (CC BY 3.0)
- `enderman_hurt_*`, `enderman_death`: griffinjennings, <https://freesound.org/people/griffinjennings/sounds/463972/> (CC BY 3.0, modified)
- `guardian_random_2`, `guardian_random_3`: nornalbion, <https://freesound.org/people/nornalbion/sounds/195733/> (CC BY 3.0, modified)
- `squid_hurt_*`, `squid_death_*`: qubodup, <https://opengameart.org/content/slime-monster> (CC BY 3.0, modified)
- `spider_death`, `spider_random`, `spider_attack_*`: columbia23, <https://freesound.org/people/columbia23/sounds/395395/> (CC BY 3.0)
- `skeleton_random_*`: Baŝto (remix) and kantouth, <https://opengameart.org/content/walking-skeleton> (CC BY 3.0)
- `horse_random_1`, `horse_death`: GoodListener, <https://freesound.org/people/GoodListener/sounds/322454/>, <https://freesound.org/people/GoodListener/sounds/322445/> (CC BY 3.0)
- `horse_random_2`: ERH, <https://freesound.org/people/ERH/sounds/32043/> (CC BY 3.0)
- `donkey_random_1`, `donkey_hurt`, `donkey_death`: Garuda1982, <https://freesound.org/people/Garuda1982/sounds/539505/> (CC BY 3.0)
- `ocelot_hurt`: ebcrosby, <https://freesound.org/people/ebcrosby/sounds/332979/> (CC BY 3.0)
- `magma_cube_small`, `magma_cube_big`: LukeIRL, <https://freesound.org/people/LukeIRL/sounds/176123/> (CC BY 4.0, derived)
- `bear_random_*`, `bear_growl_*`: tbsounddesigns, <https://freesound.org/people/tbsounddesigns/> (CC BY 3.0)
- `bear_attack_*`, `bear_death_*`, `bear_hurt_1`: YleArkisto, <https://freesound.org/people/YleArkisto/sounds/249441/> (CC BY 3.0, modified)

## Wagons (crates/game/src/wagons.rs)
Taken from VoxeLibre too (`mcl_sounds`, `mcl_doors`), slowed down and low-passed so they sit
deep and woody (none of them is shrill):
- `hoof_1..3` (hooves): `default_hard_footstep.1..3` by Erdie, <https://freesound.org/people/Erdie/sounds/41579/> (CC BY 3.0, modified)
- `wagon_creak_1` (the wagon's wood working): `doors_door_open` by CGEffex, modified by BlockMen (CC BY 3.0, modified)
- `wagon_creak_2`: `doors_door_close` by bennstir (CC BY 3.0, modified)
- `wagon_gravel_1..2` (wheels on gravel): `default_gravel_dig.1..2` by lolamadeus, <https://freesound.org/people/lolamadeus/sounds/179341/> (CC0)

## CC0 sources (for reference)
evsecrets (chicken), Zozzy (cow), rubberduck (endermite, zombiepig), Soundscapes55 (enderman
random), pointparkcinema (guardian random 1), TheBuilder15 (guardian death), Randomation (slime
damage), Darsycho (spider hurt), Under7dude and haratman (zombie), spookymodem and Cribbler
(skeleton), JarredGibb (donkey random 2), pauliw (vex), SelsRoyalNavy
(cat idle 2), Hybrid Dog (wolf), kessir and Alshred (rabbit), thefilmbakery (fire hurt),
DarkReaven and Herowl (witch), VoxeLibre contributors (villager, horse hurt).

## MIT
`pig`, `pig_angry`, `cat_idle_1`, `llama`: TenPlus1 (mobs_animal / mobs_monster mods).
`vl_elemental_fire_breath`, `vl_elemental_fire_died`: daufinsyd.

> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the "Software"), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions: The above copyright
> notice and this permission notice shall be included in all copies or substantial portions of
> the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
> PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
> LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
> OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
> DEALINGS IN THE SOFTWARE.

## Animal voices added 2026-11-01

Converted to mono, high-passed (60 Hz), low-passed (6 kHz, the fish flops and eating 1.8 kHz),
trimmed of silence and cut to at most 3 s (whale 6 s). Recordings that measured too bright
for HP's ears (cockatoo, eagle, small songbirds, locust, bat, hiss) were left out.

- **0 A.D.** fauna sounds, © Wildfire Games, <https://www.wildfiregames.com/>,
  [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) (these files stay under CC BY-SA 3.0):
  `cattle_*`, `pig_4`, `pig_5`, `pig_death`, `sheep_4`, `sheep_5`, `chicken_6`, `goat_*`,
  `horse_5`..`horse_8`, `horse_hurt_3`, `horse_death_2`, `horse_death_3`, `camel_1`, `camel_3`,
  `camel_death`, `elephant_*`, `lion_*`, `flamingo`.
- **animalia** by ElCeejo, <https://github.com/ElCeejo/animalia> (MIT):
  `cow_3`, `cow_death`, `cow_hurt_2`, `pig_3`, `pig_hurt_2`, `pig_death_2`, `sheep_3`, `sheep_hurt`,
  `sheep_death`, `chicken_7`, `chicken_hurt_2`, `chicken_death`, `horse_3`, `horse_4`,
  `horse_hurt_2`, `horse_death_4`, `deer_*`, `turkey*`, `frog_1`..`frog_3`, `cat_3`, `cat_purr`,
  `cat_hurt`, `cardinal_*`, `bluebird_1`.
- **mobs_animal** (TenPlus1 and contributors, <https://github.com/minenux/minetest-mod-mobs_animal>),
  sounds from freesound.org under CC0: `cow_2`, `pig_2`, `sheep_2`, `chicken_4`, `chicken_5`,
  `panda_1`, `kitten` (lolamadeus, CC0).
- **animalworld** by Liil/Wilhelmine (Skandarella), <https://github.com/Skandarella/animalworld>,
  released under MIT; the author credits the original recordings to freesound.org contributors
  (Lauramel, Gleith, Theveomammoth11, Vataa, D-Jones, Pol, Robinhood76, sesmo42, InspectorJ,
  Benboncan, Felix-blume and others listed in that repository's LICENSE) and Nagaty Studio (hyena)
  and Ian Fairley (zebra): `bear_4`, `boar_grunt`, `camel_2`, `crab`, `crocodile`, `dragonfly`,
  `elephant_4`, `fox_*`, `frog_4`, `giraffe_*`, `gnu`, `goose_*`, `grouse`, `hippo`, `hyena`,
  `ibex`, `koala_*`, `lizard_hiss`, `monkey_*`, `moose`, `muskox_*`, `otter_*`, `owl`, `parrot_*`,
  `rhino_*`, `roadrunner`, `seal_*`, `beluga_*` (Liil's own), `snake_hiss`, `snow_leopard`, `tapir`,
  `tiger`, `toucan`, `vulture`, `wasp_*`, `wolf_4`, `wolf_5`, `wolverine`, `yak`, `zebra_*`.
- **petz** (runsy and contributors, <https://github.com/KaylebJay/petz-1>):
  `duck_quack`, `lamb_bleat`, `sheep_hurt_2` by Alexander, orangefreesounds.com (CC BY 4.0);
  `wolf_howl`, `wolf_whine` (public domain); `bear_moan` by Mike Koenig, soundbible.com (CC BY 3.0).
- **VoxeLibre**: `cow_milk` (Klaraschick, CC0), `chicken_lay_egg` (PilzAdam and Wuzzy, CC0),
  `fish_flop_*` (CC0), `animal_eat` (InspectorJ, CC BY 3.0).
- **water_life** by Gundul: `whale_song` from <https://en.wikipedia.org/wiki/File:Humpbackwhale2.ogg> (CC0).
