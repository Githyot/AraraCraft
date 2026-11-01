"""Voice families for monsters (crates/game/src/monster_sounds.rs).

Each family names recordings by source and file pattern. build.py trims, low-passes (no shrill
highs), normalises and encodes them to assets/sounds/monsters/, then writes the Rust table.
Sources (all free licences, see assets/sounds/monsters/CREDITS.md):
  vl    VoxeLibre mobs (CC0 / CC BY 3.0)
  flare Flare fantasycore enemies (CC BY-SA 3.0)
  mg    MegaGlest megapack (CC BY-SA 3.0)
  oc    OpenClonk (CC BY 3.0 / CC0)
  lug   Lugaru (CC BY-SA 3.0)
  cdda  CDDA-Soundpacks CC-Sounds (CC BY-SA 4.0)
"""

ROOTS = {
    "vl": "/tmp/claude-0/dl/vl/mods/ENTITIES",
    "flare": "/tmp/claude-0/dl/flare-game/mods/fantasycore/soundfx/enemies",
    "mg": "/tmp/claude-0/dl/megaglest-data/techs/megapack",
    "oc": "/tmp/claude-0/dl/openclonk/planet/Sound.ocg/Animals.ocg",
    "lug": "/tmp/claude-0/dl/lugaru/Data/Sounds",
    "cdda": "/tmp/claude-0/dl/CDDA-Soundpacks/sound/CC-Sounds",
}

MGC = "commondata/sounds/"
MGU = "factions/*/units/"

# family: idle, hurt, death, attack -> lists of "src:glob" (glob relative to the root, ** allowed)
FAMILIES = {
    "zombie": dict(
        idle=["vl:**/mobs_mc_zombie_growl.ogg", "cdda:**/speech_zombie_groan*.ogg"],
        hurt=["vl:**/mobs_mc_zombie_hurt.ogg", "flare:zombie_hit.ogg"],
        death=["vl:**/mobs_mc_zombie_death.ogg", "cdda:**/zombie_death_*.ogg"],
        attack=["flare:zombie_phys.ogg", "cdda:**/monster_melee_hit_*.ogg"],
    ),
    "mummy": dict(
        idle=["mg:" + MGU + "mummy/sounds/mummy_attack*.wav"],
        hurt=["flare:zombie_hit.ogg", "flare:zombie_ment.ogg"],
        death=["mg:" + MGU + "mummy/sounds/mummy_die*.wav", "flare:zombie_die.ogg"],
        attack=["flare:zombie_phys.ogg"],
    ),
    "skeleton": dict(
        idle=["vl:**/mobs_mc_skeleton_random.*.ogg", "flare:skeleton_phys.ogg"],
        hurt=["vl:**/mobs_mc_skeleton_hurt.ogg", "flare:skeleton_hit.ogg"],
        death=["vl:**/mobs_mc_skeleton_death.ogg", "flare:skeleton_die.ogg"],
        attack=["flare:skeleton_phys.ogg"],
    ),
    "armored": dict(  # bones or spirits inside armour
        idle=["mg:" + MGU + "ghost_armor/sounds/ghost_armor_attack*.wav"],
        hurt=["lug:Clank*.ogg"],
        death=["mg:" + MGU + "ghost_armor/sounds/ghost_armor_die*.wav"],
        attack=["mg:" + MGU + "ghost_armor/sounds/ghost_armor_attack[1-4].wav"],
    ),
    "goblin": dict(
        idle=["flare:goblin_ment.ogg", "flare:goblin_phys.ogg"],
        hurt=["flare:goblin_hit.ogg"],
        death=["flare:goblin_die.ogg"],
        attack=["flare:goblin_phys.ogg"],
    ),
    "warrior": dict(  # human fighters: raiders, pirates, soldiers
        idle=["mg:" + MGC + "swordman_attack*.wav"],
        hurt=["mg:" + MGC + "stickfighter_attack1[0-2].wav", "mg:" + MGC + "stickfighter_attack[3-5].wav"],
        death=["mg:" + MGC + "swordman_die*.wav", "mg:" + MGC + "guard_die*.wav"],
        attack=["mg:" + MGC + "guard_attack*.wav", "mg:" + MGC + "swordman_attack*.wav"],
    ),
    "berserker": dict(  # big brutes and northmen
        idle=["mg:" + MGU + "battleaxe/sounds/daemon_attack*.wav"],
        hurt=["mg:" + MGC + "stickfighter_attack*.wav"],
        death=["mg:" + MGU + "battleaxe/sounds/daemon_die*.wav"],
        attack=["mg:" + MGU + "battleaxe/sounds/daemon_attack*.wav"],
    ),
    "caster": dict(  # mages, necromancers, cultists
        idle=["cdda:**/male_mumbling_*.ogg", "mg:" + MGU + "battlemage/sounds/battlemage_attack[1-4].wav"],
        hurt=["mg:" + MGU + "initiate/sounds/initiate_hit*.wav", "mg:" + MGC + "summoner_hit*.wav"],
        death=["mg:" + MGU + "initiate/sounds/initiate_die*.wav", "mg:" + MGU + "summoner/sounds/summoner_die*.wav"],
        attack=["mg:" + MGU + "initiate/sounds/initiate_attack*.wav", "mg:" + MGC + "summoner_attack[1-4].wav"],
    ),
    "shaman": dict(
        idle=["mg:" + MGU + "shaman/sounds/shaman_attac*.wav", "cdda:**/male_mumbling_*.ogg"],
        hurt=["mg:" + MGU + "shaman/sounds/shaman_hit*.wav"],
        death=["mg:" + MGU + "shaman/sounds/shaman_die*.wav"],
        attack=["mg:" + MGU + "shaman/sounds/shaman_attac*.wav"],
    ),
    "witch": dict(
        idle=["vl:**/vl_witch_laugh*.ogg"],
        hurt=["cdda:**/female_anguish_heavy_breathing_*.ogg"],
        death=["vl:**/vl_witch_death.ogg"],
        attack=["vl:**/vl_witch_laugh.2.ogg"],
    ),
    "valkyrie": dict(  # winged women and warrior queens
        idle=["mg:" + MGU + "valkyrie/sounds/valkyrie_attack*.wav"],
        hurt=["mg:" + MGU + "valkyrie/sounds/valkyrie_attack[1-3].wav"],
        death=["mg:" + MGU + "valkyrie/sounds/valkyrie_die*.wav"],
        attack=["mg:" + MGU + "valkyrie/sounds/valkyrie_attack*.wav"],
    ),
    "siren": dict(  # nymphs, lamias, medusas, queens of ice and vines
        idle=["cdda:**/female_mad_whisper_*.ogg", "mg:" + MGU + "princess/sounds/princess_attack*.wav"],
        hurt=["cdda:**/female_anguish_heavy_breathing_*.ogg"],
        death=["cdda:**/female_scream_*.ogg"],
        attack=["mg:" + MGU + "princess/sounds/princess_attack*.wav"],
    ),
    "creep": dict(  # clowns, scarecrows, evil elves: unsettling people
        idle=["cdda:**/creepy_laugh.ogg", "cdda:**/old_man_hesitant_ranbling.ogg", "cdda:**/male_mumbling_*.ogg"],
        hurt=["cdda:**/male_scared_heavy_breathing_*.ogg"],
        death=["cdda:**/male_scream_*.ogg"],
        attack=["cdda:**/creepy_laugh.ogg"],
    ),
    "demon": dict(
        idle=["mg:" + MGC + "daemon_attack*.wav"],
        hurt=["mg:" + MGC + "daemon_attack[1-6].wav"],
        death=["mg:" + MGU + "axe_indian/sounds/daemon_die*.wav"],
        attack=["mg:" + MGC + "daemon_attack1[0-6].wav"],
    ),
    "brute": dict(  # ogres, minotaurs, giants: deep throats
        idle=["flare:minotaur_phys.ogg", "oc:Monster.ocg/Growl*.wav"],
        hurt=["flare:minotaur_hit.ogg"],
        death=["flare:minotaur_die.ogg", "oc:Monster.ocg/Die.wav"],
        attack=["oc:Monster.ocg/Growl*.wav", "flare:minotaur_phys.ogg"],
    ),
    "behemoth": dict(  # titans, worms and other huge things
        idle=["mg:" + MGU + "behemoth/sounds/behemoth_walk*.wav", "mg:" + MGU + "behemoth/sounds/behemoth_attack1[a-d].wav"],
        hurt=["mg:" + MGU + "behemoth/sounds/behemoth_attack1[e-h].wav"],
        death=["mg:" + MGU + "behemoth/sounds/behemoth_die*.wav"],
        attack=["mg:" + MGU + "behemoth/sounds/behemoth_attack*.wav"],
    ),
    "golem": dict(  # stone and metal bodies
        idle=["mg:" + MGC + "golem_walk*.wav"],
        hurt=["mg:" + MGC + "golem_hit*.wav", "lug:HeavyImpact.ogg"],
        death=["mg:" + MGC + "golem_die*.wav"],
        attack=["mg:" + MGC + "golem_attack*.wav"],
    ),
    "dragon": dict(
        idle=["flare:wyvern_ment.ogg", "mg:" + MGU + "evil_dragon/sounds/evil_dragon_hit*.wav"],
        hurt=["flare:wyvern_hit.ogg", "mg:" + MGC + "dragon_hit*.wav"],
        death=["flare:wyvern_die.ogg", "mg:" + MGU + "dragon/sounds/dragon_die*.wav"],
        attack=["vl:**/mobs_mc_ender_dragon_attack.ogg", "mg:" + MGU + "dragon/sounds/dragon_attack*.wav", "flare:wyvern_phys.ogg"],
    ),
    "ghost": dict(
        idle=["mg:" + MGU + "genie/sounds/genie_attack*.wav", "mg:" + MGU + "ghost_armor/sounds/ghost_armor_attack[5-9].wav"],
        hurt=["vl:**/mobs_mc_vex_hurt.ogg", "mg:" + MGU + "genie/sounds/genie_attack*.wav"],
        death=["mg:" + MGU + "genie/sounds/genie_die*.wav", "vl:**/mobs_mc_vex_death.ogg"],
        attack=["mg:" + MGU + "ghost_armor/sounds/ghost_armor_attack[5-9].wav"],
    ),
    "vex": dict(
        idle=["vl:**/mobs_mc_vex_hurt.ogg"],
        hurt=["vl:**/mobs_mc_vex_hurt.ogg"],
        death=["vl:**/mobs_mc_vex_death.ogg"],
        attack=["vl:**/mobs_mc_vex_hurt.ogg"],
    ),
    "insect": dict(  # big bugs: antlions, hornets, mosquitoes
        idle=["oc:MosquitoBuzz.ogg", "oc:Zap.ocg/Zap*.ogg", "flare:antlion_ment.ogg"],
        hurt=["flare:antlion_hit.ogg"],
        death=["flare:antlion_die.ogg", "flare:antlion_critdie.ogg"],
        attack=["flare:antlion_phys.ogg", "oc:Chippie.ocg/Bite*.wav"],
    ),
    "mite": dict(  # silverfish, endermites, little crawlers
        idle=["vl:**/mobs_mc_silverfish_idle.ogg"],
        hurt=["vl:**/mobs_mc_endermite_hurt.*.ogg"],
        death=["vl:**/mobs_mc_silverfish_death.ogg", "vl:**/mobs_mc_endermite_death.*.ogg"],
        attack=["oc:Chippie.ocg/Bite*.wav"],
    ),
    "bat": dict(
        idle=["oc:Bat.ocg/Flutter*.wav"],
        hurt=["vl:**/mobs_mc_bat_hurt.*.ogg"],
        death=["vl:**/mobs_mc_bat_death.ogg"],
        attack=["cdda:**/bite_hit_*.ogg"],
    ),
    "slime": dict(
        idle=["vl:**/green_slime_jump.ogg", "vl:**/green_slime_land.ogg"],
        hurt=["vl:**/green_slime_damage.ogg"],
        death=["vl:**/green_slime_death.ogg"],
        attack=["vl:**/green_slime_attack.ogg"],
    ),
    "magma": dict(
        idle=["vl:**/mobs_mc_magma_cube_big.ogg", "vl:**/mobs_mc_magma_cube_small.ogg"],
        hurt=["vl:**/green_slime_damage.ogg"],
        death=["vl:**/green_slime_death.ogg"],
        attack=["vl:**/mobs_mc_magma_cube_attack.ogg"],
    ),
    "eye": dict(  # flying eyes and floating heads: wet and squelchy
        idle=["oc:Puka.ocg/Gulp.wav", "vl:**/mobs_mc_squid_flop.*.ogg"],
        hurt=["vl:**/mobs_mc_squid_hurt.*.ogg", "vl:**/green_slime_damage.ogg"],
        death=["vl:**/mobs_mc_squid_death.1.ogg", "oc:Mooq.ocg/DieFat.wav"],
        attack=["oc:Fish.ocg/Munch1.ogg", "cdda:**/bite_hit_*.ogg"],
    ),
    "serpent": dict(
        idle=["oc:Puka.ocg/Hiss*.wav", "mg:" + MGU + "snake_basket/sounds/snake_attack*.wav"],
        hurt=["oc:Puka.ocg/Hurt*.wav"],
        death=["oc:Puka.ocg/Die.wav"],
        attack=["cdda:**/bite_hit_*.ogg"],
    ),
    "hound": dict(  # werewolves, spectral wolves
        idle=["lug:Growl*.ogg", "lug:Snarl*.ogg", "vl:**/mobs_mc_wolf_growl.ogg"],
        hurt=["vl:**/mobs_mc_wolf_hurt.*.ogg"],
        death=["vl:**/mobs_mc_wolf_death.ogg"],
        attack=["lug:BarkGrowl.ogg", "lug:Snarl*.ogg", "cdda:**/bite_hit_*.ogg"],
    ),
    "raptor": dict(  # harpies, vultures, cockatrices
        idle=["mg:" + MGU + "eagle_pillar/sounds/eagle_attack*.wav", "mg:" + MGU + "thunderbird/sounds/eagle_attack*.wav"],
        hurt=["mg:" + MGU + "eagle_pillar/sounds/eagle_hit*.wav"],
        death=["mg:" + MGU + "thunderbird/sounds/eagle_die*.wav"],
        attack=["mg:" + MGU + "eagle_pillar/sounds/eagle_attack*.wav"],
    ),
    "machine": dict(  # martian craft, mechanical bosses, tanks
        idle=["mg:" + MGU + "battle_machine/sounds/machine_walk*.wav", "mg:" + MGC + "catapult_walk*.wav"],
        hurt=["lug:Clank*.ogg", "lug:MetalHit.ogg"],
        death=["mg:" + MGU + "battle_machine/sounds/machine_die*.wav", "mg:" + MGU + "ornithopter/sounds/orni_die*.wav"],
        attack=["mg:" + MGU + "battle_machine/sounds/machine_attack*.wav"],
    ),
    "plant": dict(  # man-eaters, living trees, aechor plants
        idle=["oc:Mooq.ocg/Snorting*.wav", "oc:Mooq.ocg/Munch*.wav"],
        hurt=["oc:Mooq.ocg/Hurt*.wav", "lug:BushRustle.ogg"],
        death=["oc:Mooq.ocg/Die*.wav"],
        attack=["oc:Mooq.ocg/Munch*.wav", "cdda:**/bite_hit_*.ogg"],
    ),
    "tree": dict(  # huge wooden bodies
        idle=["mg:" + MGC + "tower_hit*.wav"],
        hurt=["mg:" + MGC + "catapult_hit*.wav"],
        death=["mg:" + MGC + "tower_attack*.wav"],
        attack=["mg:" + MGC + "tower_hit*.wav"],
    ),
    "fish": dict(  # predators of the water
        idle=["vl:**/mobs_mc_squid_flop.*.ogg"],
        hurt=["vl:**/mobs_mc_squid_hurt.*.ogg"],
        death=["vl:**/mobs_mc_squid_death.1.ogg"],
        attack=["oc:Fish.ocg/Munch1.ogg", "cdda:**/bite_hit_*.ogg"],
    ),
    "fire": dict(
        idle=["vl:**/vl_elemental_fire_breath.ogg", "lug:FireStart.ogg"],
        hurt=["vl:**/vl_elemental_fire_hurt.ogg"],
        death=["vl:**/vl_elemental_fire_died.ogg", "lug:FireEnd.ogg"],
        attack=["vl:**/mobs_fireball.ogg"],
    ),
    "wind": dict(  # breezes, zephyrs, air spirits
        idle=["mg:" + MGC + "air_attack*.wav", "lug:LowWhoosh.ogg", "lug:MidWhoosh.ogg"],
        hurt=["mg:" + MGC + "air_hit*.wav"],
        death=["lug:Whoosh.ogg"],
        attack=["lug:MoveWhoosh.ogg", "mg:" + MGC + "air_attack*.wav"],
    ),
    "creeper": dict(
        idle=[],
        hurt=["vl:**/mobs_mc_creeper_hurt.ogg"],
        death=["vl:**/mobs_mc_creeper_death.ogg"],
        attack=[],
    ),
    "snowman": dict(
        idle=[],
        hurt=["vl:**/mobs_mc_snowman_hurt.*.ogg"],
        death=["vl:**/mobs_mc_snowman_death.*.ogg"],
        attack=[],
    ),
    "hoglin": dict(
        idle=["oc:Mooq.ocg/Snort*.wav", "vl:**/mobs_pig.ogg"],
        hurt=["vl:**/mobs_pig_angry.ogg"],
        death=["oc:Mooq.ocg/Die[1-3].wav"],
        attack=["vl:**/mobs_pig_angry.ogg", "oc:Mooq.ocg/Snort*.wav"],
    ),
    "piglin": dict(
        idle=["vl:**/mobs_mc_zombiepig_random.1.ogg"],
        hurt=["vl:**/mobs_mc_zombiepig_hurt.*.ogg"],
        death=["vl:**/mobs_mc_zombiepig_death.*.ogg"],
        attack=["vl:**/mobs_mc_zombiepig_war_cry.1.ogg"],
    ),
    "spider": dict(
        idle=["vl:**/mobs_mc_spider_random.ogg"],
        hurt=["vl:**/mobs_mc_spider_hurt.1.ogg"],
        death=["vl:**/mobs_mc_spider_death.ogg"],
        attack=["vl:**/mobs_mc_spider_attack.*.ogg"],
    ),
    "crab": dict(  # clacking shells
        idle=["oc:Chippie.ocg/Bite*.wav"],
        hurt=["vl:**/mobs_mc_spider_hurt.1.ogg"],
        death=["vl:**/mobs_mc_spider_death.ogg"],
        attack=["oc:Chippie.ocg/Bite*.wav"],
    ),
    "guardian": dict(
        idle=["vl:**/mobs_mc_guardian_random.*.ogg"],
        hurt=["vl:**/mobs_mc_guardian_hurt.*.ogg"],
        death=["vl:**/mobs_mc_guardian_death.ogg"],
        attack=["vl:**/mobs_mc_guardian_random.2.ogg"],
    ),
    "ender": dict(
        idle=["vl:**/mobs_mc_enderman_random.1.ogg"],
        hurt=["vl:**/mobs_mc_enderman_hurt.*.ogg"],
        death=["vl:**/mobs_mc_enderman_death.ogg"],
        attack=["vl:**/mobs_mc_enderman_teleport_src.ogg"],
    ),
    "wither": dict(
        idle=["mg:" + MGU + "behemoth/sounds/behemoth_die*.wav"],
        hurt=["mg:" + MGC + "daemon_attack[7-9].wav"],
        death=["vl:**/mobs_mc_wither_spawn.ogg"],
        attack=["vl:**/mobs_mc_ender_dragon_shoot.ogg"],
    ),
    "horse_ghost": dict(
        idle=["vl:**/mobs_mc_horse_random.*.ogg"],
        hurt=["vl:**/mobs_mc_horse_hurt.ogg"],
        death=["vl:**/mobs_mc_horse_death.ogg"],
        attack=["mg:" + MGC + "horseman_attack*.wav"],
    ),
}
