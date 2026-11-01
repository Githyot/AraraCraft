//! Real recorded creature sounds (free licences, see assets/sounds/mobs/CREDITS.md).
//!
//! Each mob family has its own set of recordings for idle, hurt and death calls. The same clip
//! is played slower for big creatures and faster for small ones, with a little random spread,
//! so a herd never sounds like one sample repeated.

use crate::sfx_extra::{Call, Fx};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex, OnceLock};

const RATE: f32 = 44100.0;

/// Every recording, by name (Ogg Vorbis, mono).
static CLIPS: &[(&str, &[u8])] = &[
    ("bear_attack_1", include_bytes!("../../../assets/sounds/mobs/bear_attack_1.ogg")),
    ("bear_attack_2", include_bytes!("../../../assets/sounds/mobs/bear_attack_2.ogg")),
    ("bear_death_1", include_bytes!("../../../assets/sounds/mobs/bear_death_1.ogg")),
    ("bear_growl_1", include_bytes!("../../../assets/sounds/mobs/bear_growl_1.ogg")),
    ("bear_growl_2", include_bytes!("../../../assets/sounds/mobs/bear_growl_2.ogg")),
    ("bear_growl_3", include_bytes!("../../../assets/sounds/mobs/bear_growl_3.ogg")),
    ("bear_hurt_1", include_bytes!("../../../assets/sounds/mobs/bear_hurt_1.ogg")),
    ("bear_random_1", include_bytes!("../../../assets/sounds/mobs/bear_random_1.ogg")),
    ("bear_random_2", include_bytes!("../../../assets/sounds/mobs/bear_random_2.ogg")),
    ("bear_random_3", include_bytes!("../../../assets/sounds/mobs/bear_random_3.ogg")),
    ("cat_idle_1", include_bytes!("../../../assets/sounds/mobs/cat_idle_1.ogg")),
    ("cat_idle_2", include_bytes!("../../../assets/sounds/mobs/cat_idle_2.ogg")),
    ("chicken_buck_1", include_bytes!("../../../assets/sounds/mobs/chicken_buck_1.ogg")),
    ("chicken_buck_2", include_bytes!("../../../assets/sounds/mobs/chicken_buck_2.ogg")),
    ("chicken_buck_3", include_bytes!("../../../assets/sounds/mobs/chicken_buck_3.ogg")),
    ("chicken_hurt", include_bytes!("../../../assets/sounds/mobs/chicken_hurt.ogg")),
    ("cow", include_bytes!("../../../assets/sounds/mobs/cow.ogg")),
    ("cow_hurt", include_bytes!("../../../assets/sounds/mobs/cow_hurt.ogg")),
    ("donkey_death", include_bytes!("../../../assets/sounds/mobs/donkey_death.ogg")),
    ("donkey_hurt", include_bytes!("../../../assets/sounds/mobs/donkey_hurt.ogg")),
    ("donkey_random_1", include_bytes!("../../../assets/sounds/mobs/donkey_random_1.ogg")),
    ("donkey_random_2", include_bytes!("../../../assets/sounds/mobs/donkey_random_2.ogg")),
    ("enderman_death", include_bytes!("../../../assets/sounds/mobs/enderman_death.ogg")),
    ("enderman_hurt_1", include_bytes!("../../../assets/sounds/mobs/enderman_hurt_1.ogg")),
    ("enderman_hurt_2", include_bytes!("../../../assets/sounds/mobs/enderman_hurt_2.ogg")),
    ("enderman_hurt_3", include_bytes!("../../../assets/sounds/mobs/enderman_hurt_3.ogg")),
    ("enderman_random_1", include_bytes!("../../../assets/sounds/mobs/enderman_random_1.ogg")),
    ("endermite_death_1", include_bytes!("../../../assets/sounds/mobs/endermite_death_1.ogg")),
    ("green_slime_damage", include_bytes!("../../../assets/sounds/mobs/green_slime_damage.ogg")),
    ("green_slime_death", include_bytes!("../../../assets/sounds/mobs/green_slime_death.ogg")),
    ("green_slime_jump", include_bytes!("../../../assets/sounds/mobs/green_slime_jump.ogg")),
    ("green_slime_land", include_bytes!("../../../assets/sounds/mobs/green_slime_land.ogg")),
    ("guardian_death", include_bytes!("../../../assets/sounds/mobs/guardian_death.ogg")),
    ("guardian_random_1", include_bytes!("../../../assets/sounds/mobs/guardian_random_1.ogg")),
    ("guardian_random_2", include_bytes!("../../../assets/sounds/mobs/guardian_random_2.ogg")),
    ("guardian_random_3", include_bytes!("../../../assets/sounds/mobs/guardian_random_3.ogg")),
    ("hoof_1", include_bytes!("../../../assets/sounds/mobs/hoof_1.ogg")),
    ("hoof_2", include_bytes!("../../../assets/sounds/mobs/hoof_2.ogg")),
    ("hoof_3", include_bytes!("../../../assets/sounds/mobs/hoof_3.ogg")),
    ("horse_death", include_bytes!("../../../assets/sounds/mobs/horse_death.ogg")),
    ("horse_hurt", include_bytes!("../../../assets/sounds/mobs/horse_hurt.ogg")),
    ("horse_random_1", include_bytes!("../../../assets/sounds/mobs/horse_random_1.ogg")),
    ("horse_random_2", include_bytes!("../../../assets/sounds/mobs/horse_random_2.ogg")),
    ("llama", include_bytes!("../../../assets/sounds/mobs/llama.ogg")),
    ("magma_cube_big", include_bytes!("../../../assets/sounds/mobs/magma_cube_big.ogg")),
    ("magma_cube_small", include_bytes!("../../../assets/sounds/mobs/magma_cube_small.ogg")),
    ("ocelot_hurt", include_bytes!("../../../assets/sounds/mobs/ocelot_hurt.ogg")),
    ("pig", include_bytes!("../../../assets/sounds/mobs/pig.ogg")),
    ("pig_angry", include_bytes!("../../../assets/sounds/mobs/pig_angry.ogg")),
    ("rabbit_death_1", include_bytes!("../../../assets/sounds/mobs/rabbit_death_1.ogg")),
    ("rabbit_death_2", include_bytes!("../../../assets/sounds/mobs/rabbit_death_2.ogg")),
    ("rabbit_death_3", include_bytes!("../../../assets/sounds/mobs/rabbit_death_3.ogg")),
    ("rabbit_hurt_1", include_bytes!("../../../assets/sounds/mobs/rabbit_hurt_1.ogg")),
    ("rabbit_hurt_2", include_bytes!("../../../assets/sounds/mobs/rabbit_hurt_2.ogg")),
    ("rabbit_hurt_3", include_bytes!("../../../assets/sounds/mobs/rabbit_hurt_3.ogg")),
    ("sheep", include_bytes!("../../../assets/sounds/mobs/sheep.ogg")),
    ("skeleton_death", include_bytes!("../../../assets/sounds/mobs/skeleton_death.ogg")),
    ("skeleton_random_1", include_bytes!("../../../assets/sounds/mobs/skeleton_random_1.ogg")),
    ("skeleton_random_2", include_bytes!("../../../assets/sounds/mobs/skeleton_random_2.ogg")),
    ("spider_attack_1", include_bytes!("../../../assets/sounds/mobs/spider_attack_1.ogg")),
    ("spider_attack_2", include_bytes!("../../../assets/sounds/mobs/spider_attack_2.ogg")),
    ("spider_death", include_bytes!("../../../assets/sounds/mobs/spider_death.ogg")),
    ("spider_hurt_1", include_bytes!("../../../assets/sounds/mobs/spider_hurt_1.ogg")),
    ("spider_random", include_bytes!("../../../assets/sounds/mobs/spider_random.ogg")),
    ("squid_death_1", include_bytes!("../../../assets/sounds/mobs/squid_death_1.ogg")),
    ("squid_hurt_1", include_bytes!("../../../assets/sounds/mobs/squid_hurt_1.ogg")),
    ("squid_hurt_2", include_bytes!("../../../assets/sounds/mobs/squid_hurt_2.ogg")),
    ("vex_death", include_bytes!("../../../assets/sounds/mobs/vex_death.ogg")),
    ("vex_hurt", include_bytes!("../../../assets/sounds/mobs/vex_hurt.ogg")),
    ("villager_1", include_bytes!("../../../assets/sounds/mobs/villager_1.ogg")),
    ("villager_2", include_bytes!("../../../assets/sounds/mobs/villager_2.ogg")),
    ("villager_3", include_bytes!("../../../assets/sounds/mobs/villager_3.ogg")),
    ("villager_4", include_bytes!("../../../assets/sounds/mobs/villager_4.ogg")),
    ("villager_5", include_bytes!("../../../assets/sounds/mobs/villager_5.ogg")),
    ("villager_hurt_1", include_bytes!("../../../assets/sounds/mobs/villager_hurt_1.ogg")),
    ("villager_hurt_2", include_bytes!("../../../assets/sounds/mobs/villager_hurt_2.ogg")),
    ("villager_trade_1", include_bytes!("../../../assets/sounds/mobs/villager_trade_1.ogg")),
    ("villager_trade_2", include_bytes!("../../../assets/sounds/mobs/villager_trade_2.ogg")),
    ("villager_trade_3", include_bytes!("../../../assets/sounds/mobs/villager_trade_3.ogg")),
    ("villager_trade_4", include_bytes!("../../../assets/sounds/mobs/villager_trade_4.ogg")),
    ("vl_elemental_fire_breath", include_bytes!("../../../assets/sounds/mobs/vl_elemental_fire_breath.ogg")),
    ("vl_elemental_fire_died", include_bytes!("../../../assets/sounds/mobs/vl_elemental_fire_died.ogg")),
    ("vl_elemental_fire_hurt", include_bytes!("../../../assets/sounds/mobs/vl_elemental_fire_hurt.ogg")),
    ("vl_witch_death", include_bytes!("../../../assets/sounds/mobs/vl_witch_death.ogg")),
    ("vl_witch_laugh", include_bytes!("../../../assets/sounds/mobs/vl_witch_laugh.ogg")),
    ("vl_witch_laugh_2", include_bytes!("../../../assets/sounds/mobs/vl_witch_laugh_2.ogg")),
    ("wagon_creak_1", include_bytes!("../../../assets/sounds/mobs/wagon_creak_1.ogg")),
    ("wagon_creak_2", include_bytes!("../../../assets/sounds/mobs/wagon_creak_2.ogg")),
    ("wagon_gravel_1", include_bytes!("../../../assets/sounds/mobs/wagon_gravel_1.ogg")),
    ("wagon_gravel_2", include_bytes!("../../../assets/sounds/mobs/wagon_gravel_2.ogg")),
    ("wolf_bark_1", include_bytes!("../../../assets/sounds/mobs/wolf_bark_1.ogg")),
    ("wolf_bark_2", include_bytes!("../../../assets/sounds/mobs/wolf_bark_2.ogg")),
    ("wolf_bark_3", include_bytes!("../../../assets/sounds/mobs/wolf_bark_3.ogg")),
    ("wolf_death", include_bytes!("../../../assets/sounds/mobs/wolf_death.ogg")),
    ("wolf_growl", include_bytes!("../../../assets/sounds/mobs/wolf_growl.ogg")),
    ("wolf_hurt_1", include_bytes!("../../../assets/sounds/mobs/wolf_hurt_1.ogg")),
    ("wolf_hurt_3", include_bytes!("../../../assets/sounds/mobs/wolf_hurt_3.ogg")),
    ("zombie_death", include_bytes!("../../../assets/sounds/mobs/zombie_death.ogg")),
    ("zombie_growl", include_bytes!("../../../assets/sounds/mobs/zombie_growl.ogg")),
    ("zombie_hurt", include_bytes!("../../../assets/sounds/mobs/zombie_hurt.ogg")),
    ("zombiepig_death_1", include_bytes!("../../../assets/sounds/mobs/zombiepig_death_1.ogg")),
    ("zombiepig_death_2", include_bytes!("../../../assets/sounds/mobs/zombiepig_death_2.ogg")),
    ("zombiepig_hurt_1", include_bytes!("../../../assets/sounds/mobs/zombiepig_hurt_1.ogg")),
    ("zombiepig_hurt_2", include_bytes!("../../../assets/sounds/mobs/zombiepig_hurt_2.ogg")),
    ("zombiepig_hurt_3", include_bytes!("../../../assets/sounds/mobs/zombiepig_hurt_3.ogg")),
    ("zombiepig_random_1", include_bytes!("../../../assets/sounds/mobs/zombiepig_random_1.ogg")),
    // Animals (2026-11-01): see assets/sounds/mobs/CREDITS.md.
    ("animal_eat", include_bytes!("../../../assets/sounds/mobs/animal_eat.ogg")),
    ("bear_4", include_bytes!("../../../assets/sounds/mobs/bear_4.ogg")),
    ("bear_moan", include_bytes!("../../../assets/sounds/mobs/bear_moan.ogg")),
    ("beluga_1", include_bytes!("../../../assets/sounds/mobs/beluga_1.ogg")),
    ("beluga_2", include_bytes!("../../../assets/sounds/mobs/beluga_2.ogg")),
    ("beluga_3", include_bytes!("../../../assets/sounds/mobs/beluga_3.ogg")),
    ("bluebird_1", include_bytes!("../../../assets/sounds/mobs/bluebird_1.ogg")),
    ("boar_grunt", include_bytes!("../../../assets/sounds/mobs/boar_grunt.ogg")),
    ("camel_1", include_bytes!("../../../assets/sounds/mobs/camel_1.ogg")),
    ("camel_2", include_bytes!("../../../assets/sounds/mobs/camel_2.ogg")),
    ("camel_3", include_bytes!("../../../assets/sounds/mobs/camel_3.ogg")),
    ("camel_death", include_bytes!("../../../assets/sounds/mobs/camel_death.ogg")),
    ("cardinal_1", include_bytes!("../../../assets/sounds/mobs/cardinal_1.ogg")),
    ("cardinal_2", include_bytes!("../../../assets/sounds/mobs/cardinal_2.ogg")),
    ("cat_3", include_bytes!("../../../assets/sounds/mobs/cat_3.ogg")),
    ("cat_hurt", include_bytes!("../../../assets/sounds/mobs/cat_hurt.ogg")),
    ("cat_purr", include_bytes!("../../../assets/sounds/mobs/cat_purr.ogg")),
    ("cattle_1", include_bytes!("../../../assets/sounds/mobs/cattle_1.ogg")),
    ("cattle_2", include_bytes!("../../../assets/sounds/mobs/cattle_2.ogg")),
    ("cattle_3", include_bytes!("../../../assets/sounds/mobs/cattle_3.ogg")),
    ("cattle_4", include_bytes!("../../../assets/sounds/mobs/cattle_4.ogg")),
    ("cattle_5", include_bytes!("../../../assets/sounds/mobs/cattle_5.ogg")),
    ("cattle_6", include_bytes!("../../../assets/sounds/mobs/cattle_6.ogg")),
    ("cattle_7", include_bytes!("../../../assets/sounds/mobs/cattle_7.ogg")),
    ("chicken_4", include_bytes!("../../../assets/sounds/mobs/chicken_4.ogg")),
    ("chicken_5", include_bytes!("../../../assets/sounds/mobs/chicken_5.ogg")),
    ("chicken_6", include_bytes!("../../../assets/sounds/mobs/chicken_6.ogg")),
    ("chicken_7", include_bytes!("../../../assets/sounds/mobs/chicken_7.ogg")),
    ("chicken_death", include_bytes!("../../../assets/sounds/mobs/chicken_death.ogg")),
    ("chicken_hurt_2", include_bytes!("../../../assets/sounds/mobs/chicken_hurt_2.ogg")),
    ("chicken_lay_egg", include_bytes!("../../../assets/sounds/mobs/chicken_lay_egg.ogg")),
    ("cow_2", include_bytes!("../../../assets/sounds/mobs/cow_2.ogg")),
    ("cow_3", include_bytes!("../../../assets/sounds/mobs/cow_3.ogg")),
    ("cow_death", include_bytes!("../../../assets/sounds/mobs/cow_death.ogg")),
    ("cow_hurt_2", include_bytes!("../../../assets/sounds/mobs/cow_hurt_2.ogg")),
    ("cow_milk", include_bytes!("../../../assets/sounds/mobs/cow_milk.ogg")),
    ("crab", include_bytes!("../../../assets/sounds/mobs/crab.ogg")),
    ("crocodile", include_bytes!("../../../assets/sounds/mobs/crocodile.ogg")),
    ("deer_1", include_bytes!("../../../assets/sounds/mobs/deer_1.ogg")),
    ("deer_2", include_bytes!("../../../assets/sounds/mobs/deer_2.ogg")),
    ("deer_death", include_bytes!("../../../assets/sounds/mobs/deer_death.ogg")),
    ("deer_hurt", include_bytes!("../../../assets/sounds/mobs/deer_hurt.ogg")),
    ("dragonfly", include_bytes!("../../../assets/sounds/mobs/dragonfly.ogg")),
    ("duck_quack", include_bytes!("../../../assets/sounds/mobs/duck_quack.ogg")),
    ("elephant_1", include_bytes!("../../../assets/sounds/mobs/elephant_1.ogg")),
    ("elephant_2", include_bytes!("../../../assets/sounds/mobs/elephant_2.ogg")),
    ("elephant_3", include_bytes!("../../../assets/sounds/mobs/elephant_3.ogg")),
    ("elephant_4", include_bytes!("../../../assets/sounds/mobs/elephant_4.ogg")),
    ("elephant_death_1", include_bytes!("../../../assets/sounds/mobs/elephant_death_1.ogg")),
    ("elephant_death_2", include_bytes!("../../../assets/sounds/mobs/elephant_death_2.ogg")),
    ("elephant_hurt", include_bytes!("../../../assets/sounds/mobs/elephant_hurt.ogg")),
    ("fish_flop_1", include_bytes!("../../../assets/sounds/mobs/fish_flop_1.ogg")),
    ("fish_flop_2", include_bytes!("../../../assets/sounds/mobs/fish_flop_2.ogg")),
    ("fish_flop_3", include_bytes!("../../../assets/sounds/mobs/fish_flop_3.ogg")),
    ("flamingo", include_bytes!("../../../assets/sounds/mobs/flamingo.ogg")),
    ("fox_1", include_bytes!("../../../assets/sounds/mobs/fox_1.ogg")),
    ("fox_2", include_bytes!("../../../assets/sounds/mobs/fox_2.ogg")),
    ("fox_3", include_bytes!("../../../assets/sounds/mobs/fox_3.ogg")),
    ("frog_1", include_bytes!("../../../assets/sounds/mobs/frog_1.ogg")),
    ("frog_2", include_bytes!("../../../assets/sounds/mobs/frog_2.ogg")),
    ("frog_3", include_bytes!("../../../assets/sounds/mobs/frog_3.ogg")),
    ("frog_4", include_bytes!("../../../assets/sounds/mobs/frog_4.ogg")),
    ("giraffe_1", include_bytes!("../../../assets/sounds/mobs/giraffe_1.ogg")),
    ("giraffe_2", include_bytes!("../../../assets/sounds/mobs/giraffe_2.ogg")),
    ("gnu", include_bytes!("../../../assets/sounds/mobs/gnu.ogg")),
    ("goat_1", include_bytes!("../../../assets/sounds/mobs/goat_1.ogg")),
    ("goat_2", include_bytes!("../../../assets/sounds/mobs/goat_2.ogg")),
    ("goat_3", include_bytes!("../../../assets/sounds/mobs/goat_3.ogg")),
    ("goat_4", include_bytes!("../../../assets/sounds/mobs/goat_4.ogg")),
    ("goat_5", include_bytes!("../../../assets/sounds/mobs/goat_5.ogg")),
    ("goat_6", include_bytes!("../../../assets/sounds/mobs/goat_6.ogg")),
    ("goat_7", include_bytes!("../../../assets/sounds/mobs/goat_7.ogg")),
    ("goat_8", include_bytes!("../../../assets/sounds/mobs/goat_8.ogg")),
    ("goose_1", include_bytes!("../../../assets/sounds/mobs/goose_1.ogg")),
    ("goose_2", include_bytes!("../../../assets/sounds/mobs/goose_2.ogg")),
    ("goose_3", include_bytes!("../../../assets/sounds/mobs/goose_3.ogg")),
    ("grouse", include_bytes!("../../../assets/sounds/mobs/grouse.ogg")),
    ("hippo", include_bytes!("../../../assets/sounds/mobs/hippo.ogg")),
    ("horse_3", include_bytes!("../../../assets/sounds/mobs/horse_3.ogg")),
    ("horse_4", include_bytes!("../../../assets/sounds/mobs/horse_4.ogg")),
    ("horse_5", include_bytes!("../../../assets/sounds/mobs/horse_5.ogg")),
    ("horse_6", include_bytes!("../../../assets/sounds/mobs/horse_6.ogg")),
    ("horse_7", include_bytes!("../../../assets/sounds/mobs/horse_7.ogg")),
    ("horse_8", include_bytes!("../../../assets/sounds/mobs/horse_8.ogg")),
    ("horse_death_2", include_bytes!("../../../assets/sounds/mobs/horse_death_2.ogg")),
    ("horse_death_3", include_bytes!("../../../assets/sounds/mobs/horse_death_3.ogg")),
    ("horse_death_4", include_bytes!("../../../assets/sounds/mobs/horse_death_4.ogg")),
    ("horse_hurt_2", include_bytes!("../../../assets/sounds/mobs/horse_hurt_2.ogg")),
    ("horse_hurt_3", include_bytes!("../../../assets/sounds/mobs/horse_hurt_3.ogg")),
    ("hyena", include_bytes!("../../../assets/sounds/mobs/hyena.ogg")),
    ("ibex", include_bytes!("../../../assets/sounds/mobs/ibex.ogg")),
    ("kitten", include_bytes!("../../../assets/sounds/mobs/kitten.ogg")),
    ("koala_1", include_bytes!("../../../assets/sounds/mobs/koala_1.ogg")),
    ("koala_2", include_bytes!("../../../assets/sounds/mobs/koala_2.ogg")),
    ("lamb_bleat", include_bytes!("../../../assets/sounds/mobs/lamb_bleat.ogg")),
    ("lion_1", include_bytes!("../../../assets/sounds/mobs/lion_1.ogg")),
    ("lion_2", include_bytes!("../../../assets/sounds/mobs/lion_2.ogg")),
    ("lion_3", include_bytes!("../../../assets/sounds/mobs/lion_3.ogg")),
    ("lion_hurt", include_bytes!("../../../assets/sounds/mobs/lion_hurt.ogg")),
    ("lion_roar", include_bytes!("../../../assets/sounds/mobs/lion_roar.ogg")),
    ("lizard_hiss", include_bytes!("../../../assets/sounds/mobs/lizard_hiss.ogg")),
    ("monkey_1", include_bytes!("../../../assets/sounds/mobs/monkey_1.ogg")),
    ("monkey_2", include_bytes!("../../../assets/sounds/mobs/monkey_2.ogg")),
    ("monkey_3", include_bytes!("../../../assets/sounds/mobs/monkey_3.ogg")),
    ("moose", include_bytes!("../../../assets/sounds/mobs/moose.ogg")),
    ("muskox_1", include_bytes!("../../../assets/sounds/mobs/muskox_1.ogg")),
    ("muskox_2", include_bytes!("../../../assets/sounds/mobs/muskox_2.ogg")),
    ("otter_1", include_bytes!("../../../assets/sounds/mobs/otter_1.ogg")),
    ("otter_2", include_bytes!("../../../assets/sounds/mobs/otter_2.ogg")),
    ("otter_3", include_bytes!("../../../assets/sounds/mobs/otter_3.ogg")),
    ("owl", include_bytes!("../../../assets/sounds/mobs/owl.ogg")),
    ("panda_1", include_bytes!("../../../assets/sounds/mobs/panda_1.ogg")),
    ("parrot_1", include_bytes!("../../../assets/sounds/mobs/parrot_1.ogg")),
    ("parrot_2", include_bytes!("../../../assets/sounds/mobs/parrot_2.ogg")),
    ("parrot_3", include_bytes!("../../../assets/sounds/mobs/parrot_3.ogg")),
    ("pig_2", include_bytes!("../../../assets/sounds/mobs/pig_2.ogg")),
    ("pig_3", include_bytes!("../../../assets/sounds/mobs/pig_3.ogg")),
    ("pig_4", include_bytes!("../../../assets/sounds/mobs/pig_4.ogg")),
    ("pig_5", include_bytes!("../../../assets/sounds/mobs/pig_5.ogg")),
    ("pig_death", include_bytes!("../../../assets/sounds/mobs/pig_death.ogg")),
    ("pig_death_2", include_bytes!("../../../assets/sounds/mobs/pig_death_2.ogg")),
    ("pig_hurt_2", include_bytes!("../../../assets/sounds/mobs/pig_hurt_2.ogg")),
    ("rhino_1", include_bytes!("../../../assets/sounds/mobs/rhino_1.ogg")),
    ("rhino_2", include_bytes!("../../../assets/sounds/mobs/rhino_2.ogg")),
    ("roadrunner", include_bytes!("../../../assets/sounds/mobs/roadrunner.ogg")),
    ("seal_1", include_bytes!("../../../assets/sounds/mobs/seal_1.ogg")),
    ("seal_2", include_bytes!("../../../assets/sounds/mobs/seal_2.ogg")),
    ("seal_3", include_bytes!("../../../assets/sounds/mobs/seal_3.ogg")),
    ("sheep_2", include_bytes!("../../../assets/sounds/mobs/sheep_2.ogg")),
    ("sheep_3", include_bytes!("../../../assets/sounds/mobs/sheep_3.ogg")),
    ("sheep_4", include_bytes!("../../../assets/sounds/mobs/sheep_4.ogg")),
    ("sheep_5", include_bytes!("../../../assets/sounds/mobs/sheep_5.ogg")),
    ("sheep_death", include_bytes!("../../../assets/sounds/mobs/sheep_death.ogg")),
    ("sheep_hurt", include_bytes!("../../../assets/sounds/mobs/sheep_hurt.ogg")),
    ("sheep_hurt_2", include_bytes!("../../../assets/sounds/mobs/sheep_hurt_2.ogg")),
    ("snake_hiss", include_bytes!("../../../assets/sounds/mobs/snake_hiss.ogg")),
    ("snow_leopard", include_bytes!("../../../assets/sounds/mobs/snow_leopard.ogg")),
    ("tapir", include_bytes!("../../../assets/sounds/mobs/tapir.ogg")),
    ("tiger", include_bytes!("../../../assets/sounds/mobs/tiger.ogg")),
    ("toucan", include_bytes!("../../../assets/sounds/mobs/toucan.ogg")),
    ("turkey", include_bytes!("../../../assets/sounds/mobs/turkey.ogg")),
    ("turkey_death", include_bytes!("../../../assets/sounds/mobs/turkey_death.ogg")),
    ("turkey_hurt", include_bytes!("../../../assets/sounds/mobs/turkey_hurt.ogg")),
    ("vulture", include_bytes!("../../../assets/sounds/mobs/vulture.ogg")),
    ("wasp_1", include_bytes!("../../../assets/sounds/mobs/wasp_1.ogg")),
    ("wasp_2", include_bytes!("../../../assets/sounds/mobs/wasp_2.ogg")),
    ("wasp_3", include_bytes!("../../../assets/sounds/mobs/wasp_3.ogg")),
    ("whale_song", include_bytes!("../../../assets/sounds/mobs/whale_song.ogg")),
    ("wolf_4", include_bytes!("../../../assets/sounds/mobs/wolf_4.ogg")),
    ("wolf_5", include_bytes!("../../../assets/sounds/mobs/wolf_5.ogg")),
    ("wolf_howl", include_bytes!("../../../assets/sounds/mobs/wolf_howl.ogg")),
    ("wolf_whine", include_bytes!("../../../assets/sounds/mobs/wolf_whine.ogg")),
    ("wolverine", include_bytes!("../../../assets/sounds/mobs/wolverine.ogg")),
    ("yak", include_bytes!("../../../assets/sounds/mobs/yak.ogg")),
    ("zebra_1", include_bytes!("../../../assets/sounds/mobs/zebra_1.ogg")),
    ("zebra_2", include_bytes!("../../../assets/sounds/mobs/zebra_2.ogg")),
];

/// The recordings one creature family uses.
struct Set {
    idle: &'static [&'static str],
    hurt: &'static [&'static str],
    death: &'static [&'static str],
}

const COW: Set = Set { idle: &["cow"], hurt: &["cow_hurt"], death: &["cow_hurt"] };
const PIG: Set = Set { idle: &["pig"], hurt: &["pig_angry"], death: &["pig_angry"] };
const SHEEP: Set = Set { idle: &["sheep"], hurt: &["sheep"], death: &["sheep"] };
const CHICKEN: Set = Set {
    idle: &["chicken_buck_1", "chicken_buck_2", "chicken_buck_3"],
    hurt: &["chicken_hurt"],
    death: &["chicken_hurt"],
};
const SQUID: Set = Set { idle: &[], hurt: &["squid_hurt_1", "squid_hurt_2"], death: &["squid_death_1"] };
const GUARDIAN: Set = Set {
    idle: &["guardian_random_1", "guardian_random_2", "guardian_random_3"],
    hurt: &["guardian_random_2"],
    death: &["guardian_death"],
};
const VILLAGER: Set = Set {
    idle: &[
        "villager_1", "villager_2", "villager_3", "villager_4", "villager_5", "villager_trade_1",
        "villager_trade_2", "villager_trade_3", "villager_trade_4",
    ],
    hurt: &["villager_hurt_1", "villager_hurt_2"],
    death: &["villager_hurt_1", "villager_hurt_2"],
};
const ZOMBIE_PIG: Set = Set {
    idle: &["zombiepig_random_1"],
    hurt: &["zombiepig_hurt_1", "zombiepig_hurt_2", "zombiepig_hurt_3"],
    death: &["zombiepig_death_1", "zombiepig_death_2"],
};
const ZOMBIE: Set = Set { idle: &["zombie_growl"], hurt: &["zombie_hurt"], death: &["zombie_death"] };
const SKELETON: Set = Set {
    idle: &["skeleton_random_1", "skeleton_random_2"],
    hurt: &["skeleton_death"],
    death: &["skeleton_death"],
};
const SPIDER: Set = Set {
    idle: &["spider_random", "spider_attack_1", "spider_attack_2"],
    hurt: &["spider_hurt_1"],
    death: &["spider_death"],
};
const ENDER: Set = Set {
    idle: &["enderman_random_1"],
    hurt: &["enderman_hurt_1", "enderman_hurt_2", "enderman_hurt_3"],
    death: &["enderman_death"],
};
const LICH: Set = Set { idle: &["enderman_random_1"], hurt: &["vex_hurt"], death: &["vex_death"] };
const FIRE: Set = Set {
    idle: &["vl_elemental_fire_breath"],
    hurt: &["vl_elemental_fire_hurt"],
    death: &["vl_elemental_fire_died"],
};
const MAGMA: Set = Set { idle: &["magma_cube_big"], hurt: &["green_slime_damage"], death: &["green_slime_death"] };
const MAGMA_SMALL: Set = Set { idle: &["magma_cube_small"], hurt: &["green_slime_damage"], death: &["green_slime_death"] };
const SLIME: Set = Set {
    idle: &["green_slime_jump", "green_slime_land"],
    hurt: &["green_slime_damage"],
    death: &["green_slime_death"],
};
const MITE: Set = Set { idle: &[], hurt: &["endermite_death_1"], death: &["endermite_death_1"] };
const WITCH: Set = Set {
    idle: &["vl_witch_laugh", "vl_witch_laugh_2"],
    hurt: &["villager_hurt_1", "villager_hurt_2"],
    death: &["vl_witch_death"],
};
const HORSE: Set = Set { idle: &["horse_random_1", "horse_random_2"], hurt: &["horse_hurt"], death: &["horse_death"] };
const DONKEY: Set = Set { idle: &["donkey_random_1", "donkey_random_2"], hurt: &["donkey_hurt"], death: &["donkey_death"] };
const LLAMA: Set = Set { idle: &["llama"], hurt: &["llama"], death: &["llama"] };
const WOLF: Set = Set {
    idle: &["wolf_bark_1", "wolf_bark_2", "wolf_bark_3", "wolf_growl"],
    hurt: &["wolf_hurt_1", "wolf_hurt_3"],
    death: &["wolf_death"],
};
const CAT: Set = Set { idle: &["cat_idle_1", "cat_idle_2"], hurt: &["ocelot_hurt"], death: &["ocelot_hurt"] };
// Rabbits keep quiet until hurt (their squeaks were too shrill).
const RABBIT: Set = Set {
    idle: &[],
    hurt: &["rabbit_hurt_1", "rabbit_hurt_2", "rabbit_hurt_3"],
    death: &["rabbit_death_1", "rabbit_death_2", "rabbit_death_3"],
};
const BEAR: Set = Set {
    idle: &["bear_random_1", "bear_random_2", "bear_random_3", "bear_growl_1", "bear_growl_2", "bear_growl_3"],
    hurt: &["bear_hurt_1", "bear_attack_1", "bear_attack_2"],
    death: &["bear_death_1"],
};

// ---- Animals (2026-11-01): every animal its own voice. ----
const fn set(idle: &'static [&'static str], hurt: &'static [&'static str], death: &'static [&'static str]) -> Set {
    Set { idle, hurt, death }
}
const A_COW: Set = set(&["cow", "cow_2", "cow_3", "cattle_1", "cattle_2", "cattle_3", "cattle_4", "cattle_5", "cattle_6", "cattle_7"], &["cow_hurt", "cow_hurt_2"], &["cow_death"]);
const A_PIG: Set = set(&["pig", "pig_2", "pig_3", "pig_4", "pig_5"], &["pig_angry", "pig_hurt_2"], &["pig_death", "pig_death_2"]);
const A_BOAR: Set = set(&["boar_grunt", "pig_4", "pig_5", "pig"], &["pig_angry", "pig_hurt_2"], &["pig_death"]);
const A_SHEEP: Set = set(&["sheep", "sheep_2", "sheep_3", "sheep_4", "sheep_5", "lamb_bleat"], &["sheep_hurt", "sheep_hurt_2"], &["sheep_death"]);
const A_GOAT: Set = set(&["goat_1", "goat_2", "goat_4", "goat_5", "goat_6", "ibex"], &["goat_3", "goat_7"], &["goat_8"]);
const A_CHICKEN: Set = set(&["chicken_buck_1", "chicken_buck_2", "chicken_buck_3", "chicken_4", "chicken_5", "chicken_6", "chicken_7"], &["chicken_hurt", "chicken_hurt_2"], &["chicken_death"]);
const A_TURKEY: Set = set(&["turkey"], &["turkey_hurt"], &["turkey_death"]);
const A_HORSE: Set = set(&["horse_random_1", "horse_random_2", "horse_3", "horse_4", "horse_5", "horse_6", "horse_7", "horse_8"], &["horse_hurt", "horse_hurt_2", "horse_hurt_3"], &["horse_death", "horse_death_2", "horse_death_3", "horse_death_4"]);
const A_ZEBRA: Set = set(&["zebra_1", "zebra_2"], &["donkey_hurt"], &["donkey_death"]);
const A_DEER: Set = set(&["deer_1", "deer_2"], &["deer_hurt"], &["deer_death"]);
const A_MOOSE: Set = set(&["moose", "deer_1"], &["deer_hurt"], &["deer_death"]);
const A_CAMEL: Set = set(&["camel_1", "camel_2", "camel_3"], &["camel_1"], &["camel_death"]);
const A_ELEPHANT: Set = set(&["elephant_1", "elephant_2", "elephant_3", "elephant_4"], &["elephant_hurt"], &["elephant_death_1", "elephant_death_2"]);
const A_GIRAFFE: Set = set(&["giraffe_1", "giraffe_2"], &["giraffe_2"], &["giraffe_1"]);
const A_HIPPO: Set = set(&["hippo"], &["hippo"], &["hippo"]);
const A_RHINO: Set = set(&["rhino_1", "rhino_2"], &["rhino_2"], &["rhino_1"]);
const A_BISON: Set = set(&["muskox_1", "muskox_2", "yak", "gnu"], &["muskox_2"], &["cow_death"]);
const A_TAPIR: Set = set(&["tapir", "muskox_1"], &["muskox_2"], &["muskox_1"]);
const A_LION: Set = set(&["lion_1", "lion_2", "lion_3", "lion_roar"], &["lion_hurt"], &["lion_3"]);
const A_BIGCAT: Set = set(&["tiger", "snow_leopard", "lion_1"], &["lion_hurt"], &["lion_3"]);
const A_CAT: Set = set(&["cat_idle_1", "cat_idle_2", "cat_3", "kitten", "cat_purr"], &["cat_hurt", "ocelot_hurt"], &["ocelot_hurt"]);
const A_WOLF: Set = set(&["wolf_bark_1", "wolf_bark_2", "wolf_bark_3", "wolf_growl", "wolf_4", "wolf_5", "wolf_howl", "wolf_whine"], &["wolf_hurt_1", "wolf_hurt_3"], &["wolf_death"]);
const A_FOX: Set = set(&["fox_1", "fox_2", "fox_3"], &["fox_3"], &["fox_2"]);
const A_HYENA: Set = set(&["hyena"], &["wolf_hurt_1"], &["wolf_death"]);
const A_BEAR: Set = set(&["bear_random_1", "bear_random_2", "bear_random_3", "bear_growl_1", "bear_growl_2", "bear_growl_3", "bear_4", "bear_moan"], &["bear_hurt_1", "bear_attack_1", "bear_attack_2"], &["bear_death_1"]);
const A_PANDA: Set = set(&["panda_1", "koala_1"], &["panda_1"], &["bear_death_1"]);
const A_KOALA: Set = set(&["koala_1", "koala_2"], &["koala_1"], &["koala_2"]);
const A_APE: Set = set(&["monkey_1", "monkey_2", "monkey_3"], &["monkey_3"], &["monkey_2"]);
const A_SMALL_GROWL: Set = set(&["wolverine"], &["wolverine"], &["wolverine"]);
const A_CROC: Set = set(&["crocodile"], &["crocodile"], &["crocodile"]);
const A_SNAKE: Set = set(&["snake_hiss"], &["snake_hiss"], &["snake_hiss"]);
const A_LIZARD: Set = set(&["lizard_hiss"], &["lizard_hiss"], &["lizard_hiss"]);
const A_SEAL: Set = set(&["seal_1", "seal_2", "seal_3"], &["seal_1"], &["seal_2"]);
const A_OTTER: Set = set(&["otter_1", "otter_2", "otter_3"], &["otter_2"], &["otter_3"]);
const A_WHALE: Set = set(&["whale_song", "beluga_1"], &["beluga_2"], &["beluga_3"]);
const A_DOLPHIN: Set = set(&["beluga_1", "beluga_2", "beluga_3"], &["beluga_2"], &["beluga_3"]);
const A_FROG: Set = set(&["frog_1", "frog_2", "frog_3", "frog_4"], &["frog_2"], &["frog_3"]);
const A_DUCK: Set = set(&["duck_quack"], &["duck_quack"], &["duck_quack"]);
const A_GOOSE: Set = set(&["goose_1", "goose_2", "goose_3"], &["goose_2"], &["goose_3"]);
const A_FLAMINGO: Set = set(&["flamingo", "goose_3"], &["goose_2"], &["goose_3"]);
const A_PARROT: Set = set(&["parrot_1", "parrot_2", "parrot_3"], &["parrot_2"], &["parrot_3"]);
const A_TOUCAN: Set = set(&["toucan", "parrot_3"], &["parrot_2"], &["parrot_3"]);
const A_SONGBIRD: Set = set(&["cardinal_1", "cardinal_2", "bluebird_1"], &["cardinal_1"], &["bluebird_1"]);
const A_OWL: Set = set(&["owl"], &["owl"], &["owl"]);
const A_RAPTOR: Set = set(&["vulture"], &["vulture"], &["vulture"]);
const A_GROUND_BIRD: Set = set(&["roadrunner", "grouse"], &["grouse"], &["turkey_death"]);
const A_BUZZ: Set = set(&["wasp_1", "wasp_2", "wasp_3"], &["wasp_1"], &["wasp_3"]);
const A_WINGS: Set = set(&["dragonfly"], &["dragonfly"], &["dragonfly"]);
const A_CRAB: Set = set(&["crab"], &["crab"], &["crab"]);
const A_FISH: Set = set(&[], &["fish_flop_1", "fish_flop_2", "fish_flop_3"], &["fish_flop_2", "squid_death_1"]);
const A_SLIME: Set = set(&[], &["green_slime_damage"], &["green_slime_death"]);

/// An animal's own voice (checked before the older families below). The rate makes big ones
/// deeper; nothing is played faster than 1.1, so no animal squeaks.
fn animal_set(name: &str) -> Option<(&'static Set, f32)> {
    Some(match name {
        "cow" | "mooshroom" | "flying_cow" => (&A_COW, 1.0),
        "ox" => (&A_COW, 0.8),
        "sniffer" => (&A_TAPIR, 0.75),
        "pig" | "phyg" => (&A_PIG, 1.0),
        "boar" | "crimson_boar" | "hoglin" => (&A_BOAR, 0.9),
        "tusklin" => (&A_BOAR, 0.75),
        "sheep" | "sheep_sheared" | "sheepuff" => (&A_SHEEP, 1.0),
        "goat" => (&A_GOAT, 1.0),
        "chicken" => (&A_CHICKEN, 1.0),
        "emu" | "moa" => (&A_TURKEY, 0.7),
        "cockatrice" => (&A_TURKEY, 0.85),
        "horse" | "skeleton_horse" | "mount_unicorn" => (&A_HORSE, 1.0),
        "zebra" => (&A_ZEBRA, 1.0),
        "zombie_horse" => (&A_HORSE, 0.8),
        "camel_husk" => (&A_CAMEL, 0.85),
        "happy_ghast" => (&A_WHALE, 1.0),
        "nautilus" => (&A_FISH, 0.9),
        "deer" | "mount_reindeer" => (&A_DEER, 1.0),
        "gazelle" => (&A_DEER, 1.1),
        "moose" => (&A_MOOSE, 0.85),
        "camel" => (&A_CAMEL, 1.0),
        "elephant" => (&A_ELEPHANT, 1.0),
        "giraffe" => (&A_GIRAFFE, 1.0),
        "hippo" => (&A_HIPPO, 1.0),
        "rhino" => (&A_RHINO, 1.0),
        "bison" => (&A_BISON, 0.9),
        "lion" => (&A_LION, 1.0),
        "lioness" => (&A_LION, 1.08),
        "onca_pintada" => (&A_BIGCAT, 1.0),
        "cat" | "cat_tame" | "ocelot" => (&A_CAT, 0.95),
        "wolf" | "wolf_tame" | "spectral_wolf" => (&A_WOLF, 1.0),
        "lobo_guara" => (&A_WOLF, 1.05),
        "fox" => (&A_FOX, 1.0),
        "hyena" => (&A_HYENA, 1.0),
        "bear" | "grizzly_bear" => (&A_BEAR, 0.95),
        "polar_bear" => (&A_BEAR, 0.88),
        "panda" => (&A_PANDA, 1.0),
        "kangaroo" => (&A_KOALA, 0.85),
        "gorilla" => (&A_APE, 0.62),
        "capuchin_monkey" | "mico_leao_dourado" => (&A_APE, 1.0),
        "raccoon" | "skunk" | "quati" => (&A_SMALL_GROWL, 1.1),
        "crocodile" | "alligator" => (&A_CROC, 0.9),
        "snapping_turtle" => (&A_CROC, 1.05),
        "anaconda" => (&A_SNAKE, 0.8),
        "rattlesnake" | "coral_snake" | "grass_snake" => (&A_SNAKE, 1.0),
        "lizard" | "magma_salamander" | "mount_basilisk" => (&A_LIZARD, 1.0),
        "tortoise" | "turtle" | "pond_turtle" | "sea_turtle" => (&A_LIZARD, 0.8),
        "seal" => (&A_SEAL, 1.0),
        "otter" | "platypus" => (&A_OTTER, 1.0),
        "blue_whale" => (&A_WHALE, 0.7),
        "cachalot_whale" => (&A_WHALE, 0.8),
        "orca" => (&A_DOLPHIN, 0.8),
        "dolphin" => (&A_DOLPHIN, 1.0),
        "cosmaw" | "endergrade" => (&A_WHALE, 0.9),
        "frog" | "warped_toad" => (&A_FROG, 1.0),
        "duck" => (&A_DUCK, 1.0),
        "swan" => (&A_GOOSE, 0.95),
        "penguin" => (&A_GOOSE, 0.85),
        "flamingo" => (&A_FLAMINGO, 1.0),
        "seagull" => (&A_RAPTOR, 1.05),
        "parrot" | "green_jay" => (&A_PARROT, 1.0),
        "toucan" => (&A_TOUCAN, 1.0),
        "bluejay" | "cardinal" | "finch" | "robin" | "sparrow" | "canary" | "hummingbird" => (&A_SONGBIRD, 1.0),
        "crow" | "vulture" => (&A_RAPTOR, 0.9),
        "roadrunner" => (&A_GROUND_BIRD, 1.0),
        "owl" => (&A_OWL, 1.0),
        "bee" | "tarantula_hawk" | "pet_bee" | "mount_bee" => (&A_BUZZ, 1.0),
        "fly" => (&A_BUZZ, 1.1),
        "dragonfly" => (&A_WINGS, 0.9),
        "lobster" | "crayfish" | "mantis_shrimp" => (&A_CRAB, 1.0),
        "banana_slug" | "snail" | "critter_snail" => (&A_SLIME, 1.1),
        "critter_grasshopper" => (&A_WINGS, 1.0),
        "critter_lavafly" | "critter_hell_butterfly" => (&A_WINGS, 0.9),
        "critter_worm" | "critter_gold_worm" | "critter_enchanted_nightcrawler" | "critter_truffle_worm" | "critter_glowing_snail" | "critter_magma_snail" => (&A_SLIME, 1.15),
        "critter_ladybug" | "critter_gold_ladybug" => (&A_WINGS, 1.1),
        "critter_mouse" | "critter_gold_mouse" => (&RABBIT, 1.1),
        "critter_scorpion" | "critter_black_scorpion" | "critter_water_strider" | "critter_gold_water_strider" => (&A_CRAB, 1.1),
        "strider" => (&A_TAPIR, 1.05),
        "mutant_snowman" => (&A_BEAR, 0.7),
        "aerwhale" => (&A_WHALE, 0.9),
        "sculk_snapper" => (&A_CROC, 0.85),
        "jacare" => (&A_CROC, 0.95),
        "capivara" => (&A_TAPIR, 1.1),
        "arara_azul" => (&A_PARROT, 0.95),
        "gralha_azul" => (&A_PARROT, 1.05),
        "tamandua" => (&A_SMALL_GROWL, 0.9),
        "tuiuiu" => (&A_GOOSE, 0.8),
        "tatu_bola" => (&RABBIT, 0.9),
        "ema" => (&A_TURKEY, 0.7),
        "asa_branca" => (&A_SONGBIRD, 0.85),
        "critter_squirrel" | "critter_gold_squirrel" => (&RABBIT, 0.95),
        _ if name.ends_with("fish") || matches!(name, "carp" | "bluegill" | "perch" | "salmon" | "trout" | "cod" | "bass" | "pike" | "seahorse" | "manta_ray" | "stingray" | "octopus") => (&A_FISH, 1.0),
        _ => return None,
    })
}

/// Each townsperson's voice: how fast their recordings play (below 1 deeper). Old, big and
/// gruff folk speak low; nobody goes above 1.08, so no voice turns shrill.
fn folk_voice(name: &str) -> f32 {
    match name {
        "npc_cyborg" => 0.72,
        "npc_winter_elder" | "npc_old_man" => 0.76,
        "npc_pirate" | "npc_smith" | "settler_smith" => 0.8,
        "npc_demolitionist" | "npc_tavernkeep" | "npc_tax_collector" => 0.83,
        "npc_guard" | "npc_wizard" | "npc_merchant" | "npc_witch_doctor" => 0.86,
        "npc_mycologist" | "settler_librarian" | "npc_peddler" | "npc_tailor" => 0.9,
        "npc_guide" | "npc_farmer" | "settler_farmer" | "npc_golfer" | "npc_painter" => 0.95,
        "npc_angler" | "settler_shepherd" | "npc_tinkerer" => 1.0,
        "npc_arms_dealer" | "npc_steampunker" | "npc_mechanic" | "npc_sorceress" | "npc_dye_trader" => 1.02,
        "npc_nurse" | "npc_dryad" | "npc_zoologist" | "npc_stylist" => 1.05,
        "npc_party_girl" | "npc_princess" => 1.08,
        _ => 1.0,
    }
}

/// Which recordings a mob uses and how fast it plays them (below 1: deeper). None: it stays
/// quiet rather than borrow a sound that doesn't fit it (insects, fish, birds, machines).
fn set_of(name: &str) -> Option<(&'static Set, f32)> {
    if let Some(a) = animal_set(name) {
        return Some(a);
    }
    let has = |s: &str| name.contains(s);
    Some(match name {
        "cow" | "flying_cow" => (&COW, 1.0),
        "mooshroom" => (&COW, 0.95),
        "ravager" => (&COW, 0.55),
        "zoglin" => (&PIG, 0.75),
        "skeleton_horse" => (&HORSE, 0.85),
        "mule" => (&DONKEY, 0.9),
        "cave_spider" => (&SPIDER, 0.95),
        "silverfish" => (&MITE, 0.85),
        "glow_squid" | "tropical_fish" | "tadpole" => (&SQUID, 1.0),
        "elder_guardian" => (&GUARDIAN, 0.55),
        "bogged" => (&SKELETON, 0.9),
        "creaking" => (&GUARDIAN, 0.6),
        "armadillo" | "bat" => (&RABBIT, 0.8),
        "allay" | "vex" | "snow_golem" => (&VILLAGER, 0.95),
        "moose" | "bison" => (&COW, 0.8),
        "ox" => (&COW, 0.72),
        "hippo" => (&COW, 0.8),
        "elephant" => (&COW, 0.7),
        "rhino" => (&COW, 0.85),
        "camel" => (&COW, 0.9),
        "sniffer" => (&COW, 0.75),
        "pig" | "boar" | "phyg" => (&PIG, 1.0),
        "tusklin" => (&PIG, 0.9),
        "crimson_boar" | "hoglin" => (&PIG, 0.8),
        "sheep" | "sheepuff" => (&SHEEP, 1.0),
        "goat" => (&SHEEP, 0.9),
        "chicken" => (&CHICKEN, 1.0),
        "duck" => (&CHICKEN, 0.85),
        "penguin" => (&CHICKEN, 0.75),
        "emu" | "cockatrice" => (&CHICKEN, 0.75),
        "moa" => (&CHICKEN, 0.65),
        "squid" | "turtle" | "tortoise" | "snapping_turtle" => (&SQUID, 1.0),
        "giant_squid" => (&SQUID, 0.7),
        "dolphin" => (&SQUID, 1.1),
        "axolotl" => (&SQUID, 1.25),
        "deep_watcher" | "sentry" | "mech_eye" => (&GUARDIAN, 0.8),
        "sentinel" => (&GUARDIAN, 0.75),
        "forest_guardian" | "void_guardian" => (&GUARDIAN, 0.7),
        "obsidilith" => (&GUARDIAN, 0.65),
        "warden" => (&GUARDIAN, 0.6),
        "glare" => (&GUARDIAN, 0.9),
        "player" => (&VILLAGER, 0.9),
        "valkyrie" => (&VILLAGER, 1.05),
        "valkyrie_queen" => (&VILLAGER, 0.95),
        "brute" | "marauder" | "invoker" | "necromancer" => (&VILLAGER, 0.8),
        "archmage" | "pyromancer" | "cryomancer" => (&VILLAGER, 0.85),
        _ if has("npc_") || has("settler_") => (&VILLAGER, folk_voice(name)),
        _ if has("goblin_") => (&VILLAGER, 0.85),
        "piglin" | "nether_zombie" => (&ZOMBIE_PIG, 1.0),
        "piglin_brute" => (&ZOMBIE_PIG, 0.75),
        "mutant_zombie" => (&ZOMBIE, 0.75),
        _ if has("zombie") => (&ZOMBIE, 1.0),
        "mummy" | "shattered" => (&ZOMBIE, 0.85),
        "sunken" => (&ZOMBIE, 0.9),
        "wither_skeleton" => (&SKELETON, 0.8),
        "mutant_skeleton" => (&SKELETON, 0.75),
        "bone_king" => (&SKELETON, 0.7),
        _ if has("skeleton") => (&SKELETON, 1.0),
        "frost_archer" => (&SKELETON, 1.0),
        "spider" => (&SPIDER, 0.85),
        "tarantula_hawk" => (&SPIDER, 0.95),
        "stalker" => (&SPIDER, 0.75),
        "otherside_stalker" => (&SPIDER, 0.7),
        "mutant_stalker" => (&SPIDER, 0.6),
        "mutant_void_walker" => (&ENDER, 0.75),
        _ if has("void_walker") => (&ENDER, 1.0),
        "ember_wraith" => (&ENDER, 0.9),
        "void_blossom" | "shulker" => (&ENDER, 1.0),
        "ghast" => (&ENDER, 0.7),
        "wither" => (&ENDER, 0.6),
        "ender_dragon" => (&ENDER, 0.5),
        "lich" => (&LICH, 0.9),
        "blaze" => (&FIRE, 1.0),
        "fire_imp" => (&FIRE, 1.1),
        "fire_minion" => (&FIRE, 1.2),
        "magma_cube" => (&MAGMA, 1.0),
        "magma_cube_small" => (&MAGMA_SMALL, 1.0),
        "lava_leviathan" => (&MAGMA, 0.6),
        "ooze" => (&SLIME, 1.0),
        "sludge" | "mimic_cube" => (&SLIME, 0.8),
        "swet" => (&SLIME, 1.2),
        "endermite" | "sculk_leech" => (&MITE, 0.9),
        "cockroach" => (&MITE, 1.0),
        "swamp_witch" => (&WITCH, 1.0),
        "horse" => (&HORSE, 1.0),
        "deer" => (&HORSE, 1.15),
        "gazelle" => (&HORSE, 1.2),
        "zebra" | "donkey" => (&DONKEY, 1.0),
        "llama" => (&LLAMA, 1.0),
        "wolf" | "wolf_tame" => (&WOLF, 1.0),
        "spectral_wolf" => (&WOLF, 0.9),
        "fox" => (&WOLF, 1.2),
        "lobo_guara" => (&WOLF, 1.1),
        "hyena" => (&WOLF, 1.05),
        "cat" | "cat_tame" => (&CAT, 0.9),
        "rabbit" => (&RABBIT, 0.85),
        "aerbunny" | "squirrel" | "quati" | "skunk" => (&RABBIT, 0.9),
        "hedgehog" => (&RABBIT, 0.95),
        "jerboa" => (&RABBIT, 1.0),
        "raccoon" => (&RABBIT, 0.85),
        "bear" => (&BEAR, 1.0),
        "polar_bear" => (&BEAR, 0.9),
        "grizzly_bear" => (&BEAR, 0.85),
        "dropbear" | "panda" => (&BEAR, 1.1),
        "gorilla" => (&BEAR, 0.9),
        "lion" => (&BEAR, 0.85),
        "lioness" | "onca_pintada" => (&BEAR, 0.9),
        _ => return None,
    })
}

fn index_of(name: &str) -> Option<u16> {
    CLIPS.iter().position(|(n, _)| *n == name).map(|i| i as u16)
}

/// A recording that is not a creature's call (hooves, a wagon's wood creaking), played at
/// `rate` (below 1: slower and deeper).
pub fn clip(name: &str, rate: f32) -> Option<Fx> {
    Some(Fx::Clip(index_of(name)?, rate.clamp(0.4, 1.1)))
}

/// A recorded call for this mob, or None if it has none. `depth` is 1 for a normal-sized
/// creature and more for bigger ones; r1 and r2 are random numbers in 0..1.
pub fn pick(name: &str, call: Call, depth: f32, r1: f32, r2: f32) -> Option<Fx> {
    if crate::monster_sounds::has(name) {
        return crate::monster_sounds::pick(name, call, depth, r1, r2);
    }
    let (set, rate) = set_of(name)?;
    let (list, extra) = match call {
        Call::Idle => (set.idle, 1.0),
        Call::Hurt => (if set.hurt.is_empty() { set.idle } else { set.hurt }, 1.04),
        Call::Death => (if set.death.is_empty() { set.hurt } else { set.death }, 0.9),
    };
    if list.is_empty() {
        return None;
    }
    let clip = list[((r1 * list.len() as f32) as usize).min(list.len() - 1)];
    let rate = rate * extra / depth.clamp(0.6, 1.8).powf(0.3) * (0.94 + 0.12 * r2);
    // Small creatures may sound smaller, but never sped up into a squeak.
    Some(Fx::Clip(index_of(clip)?, rate.clamp(0.4, 1.1)))
}

/// An animal munching (fed by hand, grazing), a little deeper for big ones.
pub fn eat(depth: f32, r: f32) -> Option<Fx> {
    let rate = 1.0 / depth.clamp(0.6, 1.8).powf(0.3) * (0.92 + 0.12 * r);
    Some(Fx::Clip(index_of("animal_eat")?, rate.clamp(0.4, 1.1)))
}

/// A cat purring (petted).
pub fn purr() -> Option<Fx> {
    Some(Fx::Clip(index_of("cat_purr")?, 1.0))
}

/// A hen laying an egg.
pub fn lay_egg() -> Option<Fx> {
    Some(Fx::Clip(index_of("chicken_lay_egg")?, 1.0))
}

/// Milking a cow.
pub fn milk() -> Option<Fx> {
    Some(Fx::Clip(index_of("cow_milk")?, 1.0))
}

type Decoded = Arc<(Vec<f32>, u32)>;

/// A clip decoded to mono (cached: each one is decoded once).
fn decoded(i: u16) -> Option<Decoded> {
    static CACHE: OnceLock<Mutex<HashMap<u16, Decoded>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(d) = cache.lock().ok()?.get(&i) {
        return Some(d.clone());
    }
    use rodio::Source;
    let bytes: &'static [u8] = if i >= crate::monster_sounds::BASE { crate::monster_sounds::bytes(i)? } else { CLIPS.get(i as usize)?.1 };
    let src = rodio::Decoder::new(Cursor::new(bytes)).ok()?;
    let (ch, sr) = (src.channels().max(1) as usize, src.sample_rate());
    let all: Vec<f32> = src.convert_samples::<f32>().collect();
    let mono = all.chunks(ch).map(|c| c.iter().sum::<f32>() / ch as f32).collect();
    let d = Arc::new((mono, sr));
    cache.lock().ok()?.insert(i, d.clone());
    Some(d)
}

/// Plays clip `i` at `rate` (1 = as recorded) as mono samples at 44.1 kHz.
pub fn render(i: u16, rate: f32) -> Vec<f32> {
    let Some(d) = decoded(i) else {
        return Vec::new();
    };
    let (src, sr) = (&d.0, d.1 as f32);
    if src.len() < 2 {
        return Vec::new();
    }
    let step = sr / RATE * rate.clamp(0.25, 4.0);
    let n = ((src.len() - 1) as f32 / step) as usize;
    let mut out: Vec<f32> = (0..n)
        .map(|k| {
            let p = k as f32 * step;
            let j = p as usize;
            let f = p - j as f32;
            src[j] * (1.0 - f) + src[(j + 1).min(src.len() - 1)] * f
        })
        .collect();
    // Two gentle low-pass stages (about 3 kHz): keeps the body of the voice and takes off the
    // hiss and squeak, so nothing sounds shrill.
    let k = 1.0 - (-std::f32::consts::TAU * 3000.0 / RATE).exp();
    let (mut a, mut b) = (0.0f32, 0.0f32);
    for s in out.iter_mut() {
        a += (*s - a) * k;
        b += (a - b) * k;
        *s = b;
    }
    // Short fades so no clip starts or ends on a click; peak at a steady level.
    let fade = (RATE * 0.006) as usize;
    let len = out.len();
    for k in 0..fade.min(len / 2) {
        let g = k as f32 / fade as f32;
        out[k] *= g;
        out[len - 1 - k] *= g;
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 1e-4 {
        let g = 0.8 / peak;
        out.iter_mut().for_each(|s| *s *= g);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETS: &[&Set] = &[
        &COW, &PIG, &SHEEP, &CHICKEN, &SQUID, &GUARDIAN, &VILLAGER, &ZOMBIE_PIG, &ZOMBIE,
        &SKELETON, &SPIDER, &ENDER, &LICH, &FIRE, &MAGMA, &MAGMA_SMALL, &SLIME, &MITE, &WITCH,
        &HORSE, &DONKEY, &LLAMA, &WOLF, &CAT, &RABBIT, &BEAR,
    ];

    #[test]
    fn every_set_names_real_clips() {
        for s in SETS {
            for n in s.idle.iter().chain(s.hurt).chain(s.death) {
                assert!(index_of(n).is_some(), "missing clip {n}");
            }
        }
    }

    #[test]
    fn every_clip_decodes_and_sounds() {
        let mut bad = Vec::new();
        for (i, (name, _)) in CLIPS.iter().enumerate() {
            let v = render(i as u16, 1.0);
            if v.len() <= 2000 {
                bad.push(format!("{name} too short: {}", v.len()));
                continue;
            }
            if !v.iter().any(|s| s.abs() > 0.5) {
                bad.push(format!("{name} is silent"));
            }
            if v.last().unwrap().abs() >= 0.01 {
                bad.push(format!("{name} ends on a click"));
            }
            // Effective pitch from how fast the wave changes: low and full, never shrill.
            let e: f32 = v.iter().map(|s| s * s).sum();
            let d: f32 = v.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
            let hz = RATE / std::f32::consts::PI * ((d / e).sqrt() / 2.0).min(1.0).asin();
            if hz >= 2500.0 {
                bad.push(format!("{name} too bright: {hz:.0} Hz"));
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
    }

    #[test]
    fn big_mobs_sound_deeper_and_quiet_ones_stay_quiet() {
        let Some(Fx::Clip(_, cow)) = pick("cow", Call::Idle, 1.0, 0.0, 0.5) else { panic!() };
        let Some(Fx::Clip(_, elephant)) = pick("elephant", Call::Idle, 1.6, 0.0, 0.5) else { panic!() };
        assert!(elephant < cow);
        assert!(pick("butterfly", Call::Idle, 1.0, 0.0, 0.0).is_none());
        assert!(pick("squid", Call::Idle, 1.0, 0.0, 0.0).is_none());
        assert!(pick("squid", Call::Hurt, 1.0, 0.0, 0.0).is_some());
        // Monsters speak through monster_sounds.rs.
        assert!(pick("nightwing", Call::Hurt, 1.0, 0.0, 0.0).is_some());
    }
}

#[cfg(test)]
mod export {
    use super::*;

    /// Writes a few calls of each mob family as WAV files into $MC_SOUNDS_OUT (listening test).
    #[test]
    #[ignore]
    fn export_mob_sounds() {
        let Ok(dir) = std::env::var("MC_SOUNDS_OUT") else { return };
        let mobs = [
            ("vaca", "cow", 1.0), ("elefante", "elephant", 1.6), ("porco", "pig", 0.8), ("ovelha", "sheep", 1.0),
            ("galinha", "chicken", 0.7), ("cavalo", "horse", 1.2), ("burro", "donkey", 1.1), ("lhama", "llama", 1.3),
            ("lobo", "wolf", 0.8), ("raposa", "fox", 0.7), ("gato", "cat", 0.6), ("coelho", "rabbit", 0.6),
            ("urso", "bear", 1.2), ("leao", "lion", 1.1), ("zumbi", "zombie", 1.4), ("esqueleto", "skeleton", 1.4),
            ("aranha", "spider", 0.8), ("aldeao", "npc_guide", 1.4), ("piglin", "piglin", 1.4),
            ("void_walker", "void_walker", 2.0), ("blaze", "blaze", 1.3), ("slime", "ooze", 1.0),
            ("magma", "magma_cube", 1.0), ("bruxa", "swamp_witch", 1.4), ("guardiao", "forest_guardian", 2.2),
            ("lula", "squid", 1.0), ("endermite", "endermite", 0.5),
        ];
        for (pt, name, height) in mobs {
            let depth = (height / 1.4f32).clamp(0.5, 2.6).sqrt();
            let mut all = Vec::new();
            for (k, call) in [Call::Idle, Call::Idle, Call::Hurt, Call::Death].into_iter().enumerate() {
                if let Some(Fx::Clip(i, r)) = pick(name, call, depth, k as f32 * 0.37 % 1.0, 0.5) {
                    all.extend(render(i, r).iter().map(|s| s * 0.7));
                    all.extend(std::iter::repeat_n(0.0, 14000));
                }
            }
            let zc = all.windows(2).filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0)).count();
            let loud = all.iter().filter(|s| s.abs() > 0.02).count().max(1);
            println!("{pt}: {:.0} Hz", zc as f32 / (loud as f32 / RATE) / 2.0);
            let data: Vec<u8> = all.iter().flat_map(|s| ((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()).collect();
            let mut b = Vec::new();
            b.extend_from_slice(b"RIFF");
            b.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
            b.extend_from_slice(b"WAVEfmt ");
            for v in [16u32, 1 | (1 << 16), 44100, 88200, 2 | (16 << 16)] {
                b.extend_from_slice(&v.to_le_bytes());
            }
            b.extend_from_slice(b"data");
            b.extend_from_slice(&(data.len() as u32).to_le_bytes());
            b.extend_from_slice(&data);
            std::fs::write(format!("{dir}/{pt}.wav"), b).unwrap();
        }
    }
}

/// Has this mob got recorded calls of its own (the monster audit)?
#[cfg(test)]
pub fn has_voice(name: &str) -> bool {
    set_of(name).is_some()
}

#[cfg(test)]
mod animal_voices {
    /// Creatures that are silent in nature (butterflies, fireflies, caterpillars) or aren't
    /// creatures at all (carts, kites, boats).
    fn quiet(n: &str) -> bool {
        const QUIET: &[&str] = &["butterfly", "firefly", "caterpillar", "kite_", "wagon", "boat", "minecart", "harness", "windmill", "armor_stand", "player", "pet_", "mount_", "ether_crystal", "rolling_boulder", "critter_firefly", "critter_butterfly", "critter_gold_butterfly", "jelly", "lost_girl", "tortured_soul", "biplane", "airship", "gyrodyne", "quadrocopter", "warship"];
        QUIET.iter().any(|q| n.contains(q))
    }

    #[test]
    fn every_animal_has_a_voice() {
        let reg = crate::mobs::registry_for_tests();
        let mute: Vec<&str> = reg.defs.iter().filter(|d| d.kind != voxel::mob::Kind::Hostile && !d.town && !quiet(&d.name)).filter(|d| super::set_of(&d.name).is_none()).map(|d| d.name.as_str()).collect();
        assert!(mute.is_empty(), "sem voz: {mute:?}");
    }

    #[test]
    fn every_clip_named_in_a_set_exists() {
        for name in ["cow", "pig", "sheep", "goat", "lion", "elephant", "frog", "duck", "parrot", "seal", "blue_whale", "bee", "lobster", "carp", "wolf", "cat"] {
            let (set, _) = super::set_of(name).unwrap_or_else(|| panic!("{name} has no voice"));
            for c in set.idle.iter().chain(set.hurt).chain(set.death) {
                assert!(super::index_of(c).is_some(), "{name}: {c}");
            }
        }
    }
}
