//! Monster voices: real recordings (free licences, see assets/sounds/monsters/CREDITS.md) for
//! every hostile mob and boss, with idle calls, hurt cries, death cries and an attack sound.
//!
//! Monsters are grouped into voice families (zombies, goblins, demons, golems, bugs...) built by
//! tools/monster_sounds/build.py into monster_clips.rs. Inside a family every monster still
//! sounds like itself: it has its own pitch (bigger and heavier is deeper), and it only uses its
//! own share of the family's recordings, picked from its name, so two kinds of goblin standing
//! side by side never say the same thing. Animals and passive creatures live in mob_sounds.rs.

use crate::monster_clips::{CLIPS, FAMILIES};
use crate::sfx_extra::{Call, Fx};

/// Clip numbers from here up are monster clips (mob_sounds.rs decodes them from our table).
pub const BASE: u16 = 4096;

/// The voice family and base pitch of each monster (below 1: slower and deeper). Worm bodies,
/// tails and props stay silent: the head speaks for the whole worm.
fn family(name: &str) -> Option<(&'static str, f32)> {
    let has = |s: &str| name.contains(s);
    if has("_body") || has("_tail") {
        return None;
    }
    Some(match name {
        // Undead
        "zombie" => ("zombie", 1.0),
        "zombie_villager" => ("zombie", 1.06),
        "blood_zombie" => ("zombie", 0.88),
        "mutant_zombie" => ("zombie", 0.62),
        "nether_zombie" => ("zombie", 0.92),
        "drowned" => ("zombie", 0.8),
        "sunken" => ("zombie", 0.84),
        "undead_miner" => ("zombie", 0.95),
        "mummy" => ("mummy", 1.0),
        "ghoul" => ("mummy", 0.82),
        "shattered" => ("mummy", 0.9),
        "doctor_bones" => ("mummy", 1.04),
        "skeleton" => ("skeleton", 1.0),
        "frost_archer" => ("skeleton", 1.04),
        "bogged" => ("skeleton", 0.9),
        "ash_skeleton" => ("skeleton", 0.86),
        "wither_skeleton" => ("skeleton", 0.74),
        "mutant_skeleton" => ("skeleton", 0.66),
        "risen_skeleton" => ("skeleton", 1.08),
        "skeleton_sniper" => ("skeleton", 0.94),
        "bone_king" => ("skeleton", 0.55),
        "undead_viking" => ("armored", 0.92),
        "armored_bones" => ("armored", 0.8),
        "possessed_armor" => ("armored", 0.86),
        "paladin" => ("armored", 0.7),
        "hoplite" => ("armored", 1.0),
        "lich" => ("ghost", 0.7),
        // Spirits
        "wraith" => ("ghost", 0.9),
        "reaper" => ("ghost", 0.75),
        "dune_wraith" => ("ghost", 0.95),
        "dungeon_spirit" => ("ghost", 1.05),
        "ember_wraith" => ("ghost", 0.85),
        "soul_eater" => ("ghost", 0.8),
        "dungeon_guardian" => ("ghost", 0.5),
        "possessed_sword" => ("ghost", 1.08),
        "mourning_wood" => ("tree", 0.7),
        "everscream" => ("tree", 0.8),
        "vex" => ("vex", 1.0),
        // People who fight
        "marauder" => ("warrior", 1.0),
        "pirate_deckhand" => ("warrior", 0.96),
        "pirate_gunner" => ("warrior", 1.04),
        "pirate_captain" => ("warrior", 0.78),
        "martian_soldier" => ("warrior", 0.9),
        "martian_officer" => ("warrior", 0.82),
        "solar_warrior" => ("warrior", 0.86),
        "tim" => ("caster", 1.04),
        "brute" => ("berserker", 0.9),
        "werewolf" => ("hound", 0.62),
        "spectral_wolf" => ("hound", 0.9),
        "invoker" => ("caster", 0.95),
        "necromancer" => ("caster", 0.86),
        "pyromancer" => ("caster", 1.0),
        "cryomancer" => ("caster", 1.06),
        "archmage" => ("caster", 0.74),
        "dark_mage" => ("caster", 0.8),
        "ragged_caster" => ("caster", 0.9),
        "goblin_sorcerer" => ("shaman", 1.06),
        "goblin_summoner" => ("shaman", 0.9),
        "lunar_prophet" => ("shaman", 0.7),
        "celestial_herald" => ("caster", 0.6),
        "swamp_witch" => ("witch", 1.0),
        "nymph" => ("siren", 1.0),
        "lamia" => ("siren", 0.84),
        "medusa" => ("siren", 0.76),
        "ice_queen" => ("siren", 0.72),
        "prism_empress" => ("siren", 0.8),
        "vine_queen" => ("siren", 0.66),
        "prism_pixie" => ("siren", 1.1),
        "valkyrie" => ("valkyrie", 1.0),
        "valkyrie_queen" => ("valkyrie", 0.82),
        "clown" => ("creep", 1.0),
        "scarecrow" => ("creep", 0.82),
        "evil_elf" => ("creep", 1.08),
        "pumpkin_king" => ("creep", 0.62),
        "headless_rider" => ("horse_ghost", 0.7),
        // Goblins and small folk
        "goblin_warrior" => ("goblin", 0.92),
        "goblin_archer" => ("goblin", 1.02),
        "ether_goblin" => ("goblin", 0.84),
        "fire_imp" => ("demon", 1.1),
        "fire_minion" => ("fire", 1.05),
        // Demons and beasts of the deep
        "demon" => ("demon", 0.9),
        "voodoo_demon" => ("demon", 0.82),
        "chaos_elemental" => ("demon", 0.96),
        "shade_brute" => ("demon", 0.7),
        "blight_brute" => ("demon", 0.76),
        "nether_gauntlet" => ("demon", 0.6),
        "otherside_stalker" => ("demon", 0.66),
        "ogre" => ("brute", 0.8),
        "ice_golem" => ("brute", 0.74),
        "derpling" => ("brute", 1.1),
        "giant_tortoise" => ("brute", 0.6),
        "ravager" => ("brute", 0.66),
        "warden" => ("behemoth", 0.62),
        "creaking" => ("tree", 0.9),
        "world_breaker" => ("behemoth", 0.55),
        "magma_titan" => ("behemoth", 0.6),
        "frost_colossus" => ("behemoth", 0.7),
        "scale_idol" => ("behemoth", 0.66),
        "deep_worm" => ("behemoth", 0.8),
        "tomb_crawler" => ("behemoth", 0.9),
        "shriek_worm" => ("behemoth", 1.0),
        "lava_leviathan" => ("behemoth", 0.74),
        "rolling_boulder" => ("golem", 0.6),
        // Stone and metal
        "basalt_golem" | "basalt_golem_ball" => ("golem", 0.86),
        "granite_elemental" => ("golem", 0.95),
        "obsidilith" => ("golem", 0.6),
        "forest_guardian" => ("tree", 0.62),
        "void_guardian" => ("golem", 0.66),
        "rocky_roller" => ("golem", 1.05),
        "mimic_ruin" | "mimic_blight" | "mimic_prism" => ("golem", 0.8),
        "mimic_cube" => ("slime", 0.75),
        // Machines
        "mech_eye" => ("machine", 0.9),
        "mech_worm" => ("machine", 0.7),
        "mech_skull" => ("machine", 0.8),
        "saucer" => ("machine", 0.75),
        "martian_probe" => ("machine", 1.08),
        "santa_tank" => ("machine", 0.62),
        "snowman_gunner" => ("snowman", 0.9),
        "flying_dutchman" => ("machine", 0.55),
        "sentry" => ("machine", 1.1),
        "pillar_solar" | "pillar_vortex" | "pillar_nebula" | "pillar_stardust" => ("behemoth", 0.5),
        "star_cell" => ("slime", 0.9),
        // Dragons and wyverns
        "ender_dragon" => ("dragon", 0.6),
        "sky_wyvern" => ("dragon", 0.86),
        "ether_wyvern" => ("dragon", 0.94),
        "betsy" => ("dragon", 0.7),
        "tide_duke" => ("dragon", 0.62),
        "wither" => ("wither", 0.7),
        "bone_serpent" => ("serpent", 0.8),
        // Fliers with feathers
        "harpy" => ("raptor", 1.0),
        "soul_vulture" => ("raptor", 0.86),
        "cockatrice" => ("raptor", 0.95),
        "nightwing" => ("raptor", 0.72),
        // Bugs and crawlers
        "spider" => ("spider", 0.85),
        "cave_spider" => ("spider", 0.95),
        "wall_creeper" => ("spider", 0.75),
        "black_recluse" => ("spider", 0.66),
        "antlion" => ("insect", 0.9),
        "antlion_charger" => ("insect", 0.82),
        "hive_drone" => ("insect", 1.0),
        "hive_queen" => ("insect", 0.62),
        "vortex_hornet" => ("insect", 0.86),
        "crimson_mosquito" => ("insect", 0.8),
        "eclipse_moth" => ("insect", 0.55),
        "blight_creeper" => ("insect", 0.9),
        "blight_flyer" => ("insect", 0.85),
        "gastropod" => ("slime", 0.7),
        "silverfish" => ("mite", 0.85),
        "endermite" => ("mite", 0.9),
        "sculk_leech" => ("mite", 0.8),
        "sculk_centipede" => ("mite", 0.7),
        "leech" => ("mite", 0.75),
        "anomura_fungus" => ("crab", 0.8),
        "crab" => ("crab", 1.0),
        // Bats
        "cave_bat" => ("bat", 0.7),
        "jungle_bat" => ("bat", 0.66),
        "ice_bat" => ("bat", 0.72),
        "lava_bat" => ("bat", 0.6),
        "giant_bat" => ("bat", 0.5),
        // Slimes and jellies
        "ooze" => ("slime", 1.0),
        "sludge" => ("slime", 0.8),
        "swet" => ("slime", 1.08),
        "pinky" => ("slime", 1.1),
        "flinx" => ("hound", 1.1),
        "jelly_green" => ("slime", 1.0),
        "jelly_blue" => ("slime", 0.94),
        "jelly_red" => ("slime", 0.88),
        "jelly_ice" => ("slime", 0.92),
        "jelly_pink" => ("slime", 1.06),
        "jelly_spiked" => ("slime", 0.84),
        "jelly_lava" => ("magma", 0.9),
        "king_jelly" => ("slime", 0.55),
        "jelly_queen" => ("slime", 0.62),
        "magma_cube" => ("magma", 1.0),
        "magma_cube_small" => ("magma", 1.1),
        // Eyes and floating heads
        "demon_eye" => ("eye", 1.0),
        "servant_eye" => ("eye", 1.08),
        "great_eye" => ("eye", 0.6),
        "nebula_eye" => ("eye", 0.85),
        "meteor_head" => ("fire", 0.85),
        "blight_mind" => ("eye", 0.55),
        "ghast" => ("ghost", 0.62),
        // Plants and fungi
        "man_eater" => ("plant", 0.86),
        "snatcher" => ("plant", 0.96),
        "aechor_plant" => ("plant", 1.04),
        "fungi_bulb" => ("plant", 0.9),
        "void_blossom" => ("plant", 0.6),
        "sculk_snapper" => ("plant", 0.8),
        // Water
        "shark" => ("fish", 0.6),
        "frilled_shark" => ("fish", 0.7),
        "piranha" => ("fish", 1.0),
        "arapaima" => ("fish", 0.75),
        "angler_fish" => ("fish", 0.85),
        "cave_eel" => ("serpent", 0.95),
        "jellyfish" => ("eye", 1.04),
        "caiman" => ("serpent", 0.66),
        "komodo_dragon" => ("serpent", 0.74),
        "dropbear" => ("hound", 0.8),
        "deep_watcher" => ("guardian", 1.0),
        "elder_guardian" => ("guardian", 0.6),
        "slider" => ("golem", 0.7),
        // Fire, wind, light
        "blaze" => ("fire", 1.0),
        "ember_wisp" => ("fire", 1.1),
        "sun_spirit" => ("fire", 0.7),
        "breeze" => ("wind", 1.0),
        "zephyr" => ("wind", 1.08),
        // Nether and End
        "stalker" => ("creeper", 1.0),
        "mutant_stalker" => ("creeper", 0.7),
        "piglin" => ("piglin", 1.0),
        "piglin_brute" => ("piglin", 0.76),
        "hoglin" => ("hoglin", 1.0),
        "zoglin" => ("hoglin", 0.86),
        "shulker" => ("ender", 1.05),
        "mutant_void_walker" => ("ender", 0.7),
        _ if has("void_walker") => ("ender", 1.0),
        "mutant_snowman" => ("snowman", 0.7),
        "prism_unicorn" => ("horse_ghost", 1.0),
        _ => return None,
    })
}

/// A small number from the name, the same every time (FNV-1a).
fn seed(name: &str) -> u32 {
    name.bytes().fold(0x811c_9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

fn lists(fam: &str) -> Option<&'static (&'static str, &'static [u16], &'static [u16], &'static [u16], &'static [u16])> {
    FAMILIES.iter().find(|f| f.0 == fam)
}

/// This monster's own share of a family's recordings: about two thirds of them, starting at a
/// place given by its name (all of them when there are only one or two).
fn share(list: &'static [u16], s: u32) -> impl Iterator<Item = u16> + Clone {
    let n = list.len();
    let take = if n <= 2 { n } else { (n * 2).div_ceil(3) };
    let start = if n == 0 { 0 } else { (s as usize) % n };
    (0..take).map(move |k| list[(start + k) % n])
}

/// The pitch this monster speaks at: its family pitch nudged a little by its name, so two
/// monsters of one family with the same pitch on paper still sound apart.
fn rate_of(name: &str, base: f32) -> f32 {
    let s = seed(name);
    base * (0.95 + ((s >> 8) % 1000) as f32 / 1000.0 * 0.1)
}

fn choose(name: &str, list: &'static [u16], depth: f32, r1: f32, r2: f32, extra: f32, base: f32) -> Option<Fx> {
    let s = seed(name);
    let mine: Vec<u16> = share(list, s).collect();
    if mine.is_empty() {
        return None;
    }
    let clip = mine[((r1 * mine.len() as f32) as usize).min(mine.len() - 1)];
    let rate = rate_of(name, base) * extra / depth.clamp(0.6, 1.8).powf(0.3) * (0.95 + 0.1 * r2);
    // Never sped up into a squeak, never slowed into mud.
    Some(Fx::Clip(BASE + clip, rate.clamp(0.4, 1.1)))
}

/// Has this monster got voices here? (Animals fall through to mob_sounds.rs.)
pub fn has(name: &str) -> bool {
    family(name).is_some()
}

/// A recorded call for this monster, or None if it is not one of ours. `depth` is 1 for a
/// normal-sized creature and more for bigger ones; r1 and r2 are random numbers in 0..1.
pub fn pick(name: &str, call: Call, depth: f32, r1: f32, r2: f32) -> Option<Fx> {
    let (fam, base) = family(name)?;
    let f = lists(fam)?;
    let (list, extra) = match call {
        Call::Idle => (f.1, 1.0),
        Call::Hurt => (if f.2.is_empty() { f.1 } else { f.2 }, 1.04),
        Call::Death => (if f.3.is_empty() { f.2 } else { f.3 }, 0.9),
    };
    choose(name, list, depth, r1, r2, extra, base)
}

/// The sound of this monster attacking (a roar, a bite, a spell, a clank), or None.
pub fn attack(name: &str, depth: f32, r1: f32, r2: f32) -> Option<Fx> {
    let (fam, base) = family(name)?;
    let f = lists(fam)?;
    choose(name, f.4, depth, r1, r2, 1.0, base)
}

/// Did the mob with this key just strike? `age` is its `Mob::attack_age`, which drops back to
/// zero on every blow.
pub fn swung(key: u32, age: f32) -> bool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static LAST: OnceLock<Mutex<HashMap<u32, f32>>> = OnceLock::new();
    let Ok(mut last) = LAST.get_or_init(Default::default).lock() else {
        return false;
    };
    if last.len() > 2048 {
        last.clear();
    }
    matches!(last.insert(key, age), Some(prev) if age < prev && age < 0.25)
}

/// The encoded bytes of monster clip `i` (counted from BASE).
pub fn bytes(i: u16) -> Option<&'static [u8]> {
    CLIPS.get(i.checked_sub(BASE)? as usize).map(|c| c.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every hostile mob and boss has a voice of its own (worm links speak through the head).
    #[test]
    fn every_monster_has_a_voice() {
        let reg = crate::mobs::registry_for_tests();
        let mut missing = Vec::new();
        for d in &reg.defs {
            let monster = d.kind == voxel::mob::Kind::Hostile || d.boss;
            if monster && !has(&d.name) && !d.name.ends_with("_body") && !d.name.ends_with("_tail") {
                missing.push(d.name.clone());
            }
        }
        assert!(missing.is_empty(), "monstros sem voz em monster_sounds.rs: {missing:?}");
    }

    /// Every family named in the table exists and has idle, hurt or death calls.
    #[test]
    fn families_exist() {
        let reg = crate::mobs::registry_for_tests();
        for d in &reg.defs {
            if let Some((fam, rate)) = family(&d.name) {
                let f = lists(fam).unwrap_or_else(|| panic!("família {fam} não existe ({})", d.name));
                assert!(!(f.1.is_empty() && f.2.is_empty() && f.3.is_empty()), "{fam} sem sons");
                assert!((0.4..=1.2).contains(&rate), "{}: tom {rate}", d.name);
            }
        }
        for f in FAMILIES {
            for &i in f.1.iter().chain(f.2).chain(f.3).chain(f.4) {
                assert!((i as usize) < CLIPS.len());
            }
        }
    }

    /// Two monsters of one family never sound the same: different pitch or different clips.
    #[test]
    fn monsters_of_a_family_differ() {
        let reg = crate::mobs::registry_for_tests();
        let named: Vec<&str> = reg.defs.iter().map(|d| d.name.as_str()).filter(|n| has(n)).collect();
        for (i, a) in named.iter().enumerate() {
            for b in &named[i + 1..] {
                let (fa, ra) = family(a).unwrap();
                let (fb, rb) = family(b).unwrap();
                if fa != fb || a.contains("void_walker") && b.contains("void_walker") {
                    continue;
                }
                let (pa, pb) = (rate_of(a, ra), rate_of(b, rb));
                let f = lists(fa).unwrap();
                let same_clips = share(f.1, seed(a)).eq(share(f.1, seed(b))) && share(f.2, seed(a)).eq(share(f.2, seed(b)));
                assert!((pa - pb).abs() > 0.004 || !same_clips, "{a} e {b} soam iguais");
            }
        }
    }

    /// The clips decode, are not silent, end without a click and none of them is shrill
    /// (effective pitch from how fast the wave changes, as in mob_sounds.rs).
    #[test]
    fn clips_are_not_shrill() {
        let mut bad = Vec::new();
        for (i, (name, _)) in CLIPS.iter().enumerate() {
            let v = crate::mob_sounds::render(BASE + i as u16, 1.0);
            assert!(v.len() > 1000, "{name} não decodifica");
            assert!(v.iter().any(|s| s.abs() > 0.2), "{name} mudo");
            let e: f32 = v.iter().map(|s| s * s).sum();
            let d: f32 = v.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
            let hz = 44100.0 / std::f32::consts::PI * ((d / e).sqrt() / 2.0).min(1.0).asin();
            if std::env::var("HZ").is_ok() {
                println!("HZ {hz:.0} {name}");
            }
            if hz >= 2500.0 {
                bad.push(format!("{name} {hz:.0} Hz"));
            }
        }
        assert!(bad.is_empty(), "agudos demais: {bad:?}");
    }
}
