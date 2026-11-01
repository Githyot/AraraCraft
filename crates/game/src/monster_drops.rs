//! Loot for monsters whose file gave them none (plan/monstros-analise.md). The Terraria-side
//! mob files are written by generator scripts, so the drops are added here, after loading,
//! instead of in the RON files. A monster that already drops something keeps its own list.

use voxel::block::DropDef;
use voxel::mob::MobRegistry;

/// (monster, [(item, min, max, chance)]). Invasion bosses are left out: their bag comes with
/// the event's prize (tr_invasions.rs).
const EXTRA: &[(&str, &[(&str, u8, u8, f32)])] = &[
    // Harvest Moon: wood of the spooky trees, seeds, the odd medallion for the next night.
    ("scarecrow", &[("spooky_wood", 1, 3, 0.7), ("pumpkin_seeds", 1, 2, 0.4), ("pumpkin_medallion", 1, 1, 0.01)]),
    ("headless_rider", &[("spooky_wood", 2, 5, 1.0), ("leather", 1, 3, 0.6), ("pumpkin_medallion", 1, 1, 0.02)]),
    // Winter Moon
    ("evil_elf", &[("present", 1, 1, 0.25), ("elf_gun", 1, 1, 0.02)]),
    ("snowman_gunner", &[("snowball", 2, 6, 0.8), ("present", 1, 1, 0.15)]),
    // Sea raiders
    ("pirate_deckhand", &[("gold_nugget", 1, 4, 0.6), ("cutlass", 1, 1, 0.02), ("vanity_pirate_helmet", 1, 1, 0.03), ("pirate_map", 1, 1, 0.005)]),
    ("pirate_gunner", &[("gunpowder", 1, 3, 0.7), ("gold_nugget", 1, 3, 0.5), ("vanity_pirate_chestplate", 1, 1, 0.03), ("lucky_coin", 1, 1, 0.004)]),
    // Star visitors: their kit is circuitry and copper.
    ("martian_soldier", &[("redstone", 1, 3, 0.6), ("copper_ingot", 1, 2, 0.5), ("laser_rifle", 1, 1, 0.015)]),
    ("martian_officer", &[("redstone", 2, 4, 0.8), ("copper_ingot", 1, 3, 0.6), ("laser_rifle", 1, 1, 0.03)]),
    ("martian_probe", &[("redstone", 1, 2, 0.5), ("iron_nugget", 2, 5, 0.6)]),
    // Crystal Defense
    ("ether_goblin", &[("gold_nugget", 1, 3, 0.5), ("ether_crystal", 1, 1, 0.02)]),
    ("ether_wyvern", &[("leather", 1, 2, 0.6), ("phantom_membrane", 1, 1, 0.25)]),
    // Others that left nothing behind
    ("fire_minion", &[("blaze_powder", 1, 1, 0.35)]),
    ("angler_fish", &[("glow_ink_sac", 1, 1, 0.6)]),
    ("sculk_leech", &[("sculk_bone", 1, 1, 0.3)]),
    ("sculk_centipede", &[("sculk_bone", 1, 2, 0.5)]),
];

/// Gives every listed monster without loot its drops. Returns how many it changed.
pub fn apply(reg: &mut MobRegistry) -> usize {
    let mut n = 0;
    for (name, list) in EXTRA {
        let Some(id) = reg.id(name) else { continue };
        let d = &mut reg.defs[id];
        if !d.drops.is_empty() {
            continue;
        }
        d.drops = list.iter().map(|&(item, min, max, chance)| DropDef { item: item.into(), min, max, chance }).collect();
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every monster and item named here is real, and each list gives something.
    #[test]
    fn drops_are_real() {
        let mut reg = crate::mobs::registry_for_tests();
        let (_, items, _) = crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).unwrap();
        for (name, list) in EXTRA {
            assert!(reg.id(name).is_some(), "{name} não existe");
            for &(item, min, max, chance) in *list {
                assert!(items.id(item).is_some(), "{name}: item {item} não existe");
                assert!(min >= 1 && min <= max && chance > 0.0 && chance <= 1.0, "{name}: {item}");
            }
        }
        assert_eq!(apply(&mut reg), EXTRA.len(), "algum monstro da lista já tinha drops: tire-o da lista");
    }
}
