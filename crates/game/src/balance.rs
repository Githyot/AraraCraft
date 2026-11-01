//! The balance sheet of weapons and armour (plan/balanceamento.md): every weapon is placed in a
//! stage of the progression and a class, and the tests below check that at each stage the best
//! weapon of every class, with the best armour set that class can wear by then, lands close to
//! the stage's target damage per second. Nothing here runs in the game: it reads the real
//! tables (items, tr_weapons, magic, rpg, sentries, weapons) so a change to any number there
//! shows up in the sheet.
//!
//! The model, in a few lines:
//! - damage per second on one monster, held down for a 15 s fight;
//! - a 3D factor per kind of weapon: in a world with height, homing and area spells and
//!   flying minions find their mark more often than a sword (the monster can be above you) or
//!   an arrow (it drops and has to be drawn);
//! - casters run on their mana pool and regeneration for the stage, and on mana potions at the
//!   price of mana sickness (a quarter less magic damage) after that;
//! - minions count once per slot a summoner has by then (`SLOTS`), with the whip's mark;
//! - the armour set multiplies damage, crit (a crit is half again) and melee speed.

use crate::mods::combat::weapon_of;
pub use crate::classes::{set_factor, Class, CLASSES, GEAR, SET_STAGE, STAGES};
use crate::rpg::{M, MINIONS, SETS};
use Class::*;


/// Target damage per second of each stage (about 1.5 times the stage before). The first one is
/// the iron sword's, which keeps the classic numbers (parity_test.rs).
pub const TARGET: [f32; 6] = [9.0, 13.0, 19.0, 29.0, 45.0, 70.0];

/// How close the best weapon of each class must come to the target (0.85 = 15% under).
pub const BAND: (f32, f32) = (0.85, 1.15);

/// No weapon may pass the target of its own stage by more than this.
pub const CEILING: f32 = 1.2;



/// Weapons of the tables that are tools first (they blow up the ground) and don't count.
const NOT_WEAPONS: &[&str] = &["bomb", "dynamite"];


/// The share of a stage's best defense the set each class wears for damage must keep.
pub const ARMOUR_FLOOR: f32 = 0.55;

/// Defense of the four pieces of a set and its own bonus.
pub fn set_defense(items: &voxel::item::ItemRegistry, prefix: &str) -> f32 {
    let pieces: f32 = ["helmet", "chestplate", "leggings", "boots"]
        .iter()
        .filter_map(|k| items.id(&format!("{prefix}_{k}")).and_then(|id| items.get(id).armor))
        .map(|a| a.points as f32)
        .sum();
    let bonus: f32 = SETS.iter().filter(|s| s.prefix == prefix).flat_map(|s| s.mods.iter()).map(|m| if let M::Def(d) = m { *d } else { 0.0 }).sum();
    pieces + bonus
}

/// The most defense one can wear by `stage`.
pub fn best_defense(items: &voxel::item::ItemRegistry, stage: u8) -> (f32, &'static str) {
    SET_STAGE.iter().filter(|s| s.1 <= stage).map(|s| (set_defense(items, s.0), s.0)).fold((0.0, ""), |a, b| if b.0 > a.0 { b } else { a })
}

/// Minion slots a summoner has by each stage: one, the summoning potion, the totem necklace
/// (Forest Guardian), the hive set and then the orichalcum one, and the stardust set at the end.
pub const SLOTS: [f32; 6] = [2.0, 3.0, 4.0, 4.0, 4.0, 5.0];

/// Mana a caster can spend each second in a 15 s fight without potions: the pool (mana crystals
/// and accessories) over 15 s, plus regeneration while casting.
pub const MANA_BUDGET: [f32; 6] = [2.0, 2.5, 3.8, 4.5, 5.3, 6.3];

#[cfg(test)]
/// What a caster's damage keeps under mana sickness.
const SICK: f32 = 0.75;

#[cfg(test)]
/// The 3D factor of a weapon (see the module comment).
fn factor_3d(name: &str, class: Class) -> f32 {
    match class {
        Warrior if crate::tr_weapons::sustained_dps(name).is_some() => {
            if name.ends_with("_yoyo") {
                1.0
            } else {
                0.95
            }
        }
        Warrior => 0.9,
        Archer => 0.9,
        Gunner => 1.0,
        Assassin => 0.85,
        Mage => {
            if matches!(name, "amethyst_wand" | "hive_staff" | "star_tome" | "spirit_staff" | "chain_tome" | "thunder_tome" | "blizzard_staff" | "razorblade_typhoon" | "magnet_sphere" | "prism") {
                1.1
            } else if matches!(name, "fire_staff" | "meteor_staff" | "sun_stone_staff" | "blight_rod" | "void_tome" | "worm_staff" | "lunar_flare" | "shadow_beam" | "creeping_thorn") {
                1.05
            } else {
                1.0
            }
        }
        Summoner => 1.0,
    }
}

#[cfg(test)]
/// Damage per second of a weapon by itself, before the armour set and the 3D factor.
pub fn raw_dps(items: &voxel::item::ItemRegistry, name: &str, class: Class, stage: u8) -> Option<f32> {
    match class {
        Warrior => {
            let tool = items.id(name).and_then(|id| items.get(id).tool.clone());
            let swing = tool.as_ref().and_then(|t| weapon_of(name, Some(t.kind)).map(|w| w.swing_dps(t.damage))).unwrap_or(0.0)
                * crate::blade_fx::proc_factor(name);
            let flying = crate::tr_weapons::sustained_dps(name).map_or(0.0, |d| d.1);
            // A yoyo, boomerang or flail is thrown, not swung (its item has no blade).
            let swing = if tool.is_some_and(|t| t.damage > 1.0) { swing } else { 0.0 };
            Some(swing + flying).filter(|d| *d > 0.0)
        }
        Archer => crate::weapons::bow_dps(name),
        Gunner => crate::tr_weapons::sustained_dps(name).map(|d| d.1),
        Assassin => {
            // A dagger is swung like a sword (its combo); the rest are thrown.
            let tool = items.id(name).and_then(|id| items.get(id).tool.clone());
            let swing = tool.and_then(|t| weapon_of(name, Some(t.kind)).map(|w| w.swing_dps(t.damage)));
            swing.or_else(|| crate::tr_weapons::sustained_dps(name).map(|d| d.1))
        }
        Mage => {
            let (dps, mana) = crate::magic::sustained_dps(name)?;
            let full = (MANA_BUDGET[stage as usize] / mana.max(1e-3)).min(1.0);
            Some(dps * (full + (1.0 - full) * SICK))
        }
        Summoner => {
            if let Some(m) = MINIONS.iter().find(|m| m.staff == name) {
                return Some(m.damage * (1.0 + best_whip(stage)) / m.cooldown * SLOTS[stage as usize]);
            }
            if crate::rpg::whip_bonus(name).is_some() {
                // A whip by itself: its swing (its mark counts with the minions).
                let tool = items.id(name).and_then(|id| items.get(id).tool.clone())?;
                return weapon_of(name, Some(tool.kind)).map(|w| w.swing_dps(tool.damage));
            }
            crate::sentries::sentry_dps(name).or_else(|| crate::tr_weapons::sustained_dps(name).map(|d| d.1))
        }
    }
}

#[cfg(test)]
/// The best whip mark a summoner can have by `stage`.
fn best_whip(stage: u8) -> f32 {
    GEAR.iter().filter(|g| g.1 == Summoner && g.2 <= stage).filter_map(|g| crate::rpg::whip_bonus(g.0)).fold(0.0, f32::max)
}


#[cfg(test)]
/// A weapon's power at its own stage: what the sheet compares.
pub fn power(items: &voxel::item::ItemRegistry, name: &str, class: Class, stage: u8) -> Option<f32> {
    let raw = raw_dps(items, name, class, stage)?;
    Some(raw * factor_3d(name, class) * set_factor(class, stage).0)
}

#[cfg(test)]
/// The strongest weapon of a class one can have by `stage` and its power then.
/// A whip's own swing and a sentry don't count: they help the minions.
pub fn best(items: &voxel::item::ItemRegistry, class: Class, stage: u8) -> Option<(&'static str, f32)> {
    let helper = |n: &str| class == Summoner && !MINIONS.iter().any(|m| m.staff == n);
    GEAR.iter()
        .filter(|g| g.1 == class && g.2 <= stage && !helper(g.0))
        .filter_map(|g| power(items, g.0, class, stage).map(|p| (g.0, p)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> voxel::item::ItemRegistry {
        crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).unwrap().1
    }

    /// Prints the whole sheet (cargo test -p game balance -- --nocapture).
    #[test]
    fn print_sheet() {
        let items = items();
        for (s, name) in STAGES.iter().enumerate() {
            let (top, top_set) = best_defense(&items, s as u8);
            println!("== {name} (meta {:.0}, defesa máxima {top:.0} {top_set})", TARGET[s]);
            for c in CLASSES {
                let (set_k, set) = set_factor(c, s as u8);
                let b = best(&items, c, s as u8);
                println!("  {:<13} {:>6.1} {:<22} conjunto {set} x{set_k:.2} defesa {:.0}", c.name(), b.map_or(0.0, |b| b.1), b.map_or("-", |b| b.0), set_defense(&items, set));
            }
            for g in GEAR.iter().filter(|g| g.2 == s as u8) {
                let raw = raw_dps(&items, g.0, g.1, g.2).unwrap_or(-1.0);
                let p = power(&items, g.0, g.1, g.2).unwrap_or(-1.0);
                println!("    {:<22} {:<12} cru {:>6.1}  efetivo {:>6.1}  ({:>3.0}%)", g.0, g.1.name(), raw, p, 100.0 * p / TARGET[s]);
            }
        }
    }

    #[test]
    fn every_weapon_has_a_stage_and_numbers() {
        let items = items();
        for g in GEAR {
            assert!(items.id(g.0).is_some(), "no item {}", g.0);
            assert!(raw_dps(&items, g.0, g.1, g.2).is_some_and(|d| d > 0.0), "{} has no damage", g.0);
            assert_eq!(GEAR.iter().filter(|h| h.0 == g.0).count(), 1, "{} twice", g.0);
        }
        let listed = |n: &str| GEAR.iter().any(|g| g.0 == n) || NOT_WEAPONS.contains(&n);
        for n in crate::tr_weapons::all_weapons().chain(crate::magic::all_casters()) {
            if crate::magic::sustained_dps(n).is_none() && crate::tr_weapons::sustained_dps(n).is_none() {
                continue;
            }
            assert!(listed(n), "{n} is not in the balance sheet (balance.rs GEAR)");
        }
        for m in MINIONS {
            assert!(listed(m.staff), "{} is not in the balance sheet", m.staff);
        }
        for s in crate::sentries::SENTRIES {
            assert!(listed(s.staff), "{} is not in the balance sheet", s.staff);
        }
        // Every blade of the item files, except the modular tools (they copy the plain ones).
        for it in &items.items {
            let blade = it.tool.as_ref().is_some_and(|t| t.kind == voxel::block::ToolKind::Sword && t.damage > 1.0);
            if blade && !it.name.starts_with("gear_") && crate::rpg::whip_bonus(&it.name).is_none() {
                assert!(listed(&it.name), "{} is not in the balance sheet", it.name);
            }
        }
        for (p, _) in SET_STAGE {
            assert!(SETS.iter().any(|s| s.prefix == *p), "no set {p}");
        }
        for s in SETS {
            assert!(SET_STAGE.iter().any(|x| x.0 == s.prefix), "set {} has no stage", s.prefix);
        }
    }

    /// The point of the sheet: at every stage every class has something close to the target.
    #[test]
    fn classes_are_close_at_every_stage() {
        let items = items();
        let mut bad = Vec::new();
        for s in 0..STAGES.len() {
            for c in CLASSES {
                let Some((name, p)) = best(&items, c, s as u8) else {
                    bad.push(format!("{}: {} sem arma", STAGES[s], c.name()));
                    continue;
                };
                let r = p / TARGET[s];
                if r < BAND.0 || r > BAND.1 {
                    bad.push(format!("{}: {} {name} {p:.1} ({:.0}% da meta {:.0})", STAGES[s], c.name(), 100.0 * r, TARGET[s]));
                }
            }
        }
        assert!(bad.is_empty(), "fora da faixa:\n{}", bad.join("\n"));
    }

    /// No single weapon runs away from its own stage.
    #[test]
    fn no_weapon_beats_its_stage() {
        let items = items();
        let mut bad = Vec::new();
        for g in GEAR {
            let p = power(&items, g.0, g.1, g.2).unwrap();
            if p > TARGET[g.2 as usize] * CEILING {
                bad.push(format!("{} ({}, {}): {p:.1} > {:.1}", g.0, g.1.name(), STAGES[g.2 as usize], TARGET[g.2 as usize] * CEILING));
            }
        }
        assert!(bad.is_empty(), "fortes demais:\n{}", bad.join("\n"));
    }

    /// Every class's damage set still protects: no class has to go about in paper.
    #[test]
    fn armour_keeps_up() {
        let items = items();
        let mut last = 0.0;
        for s in 0..STAGES.len() {
            let (top, _) = best_defense(&items, s as u8);
            assert!(top >= last, "a defesa cai em {}", STAGES[s]);
            last = top;
            for c in CLASSES {
                let (_, set) = set_factor(c, s as u8);
                if set.is_empty() {
                    continue;
                }
                let d = set_defense(&items, set);
                assert!(d >= ARMOUR_FLOOR * top, "{} em {}: conjunto {set} tem {d} de {top}", c.name(), STAGES[s]);
            }
        }
    }

    /// The best weapon of every class keeps getting better from stage to stage.
    #[test]
    fn every_class_grows() {
        let items = items();
        for c in CLASSES {
            let mut last = 0.0;
            for s in 0..STAGES.len() {
                let p = best(&items, c, s as u8).map_or(0.0, |b| b.1);
                assert!(p > last, "{} não cresce em {}", c.name(), STAGES[s]);
                last = p;
            }
        }
    }

    /// Every armour (HP 2026-11-01): has its own set bonus and a stage; at its stage it keeps up
    /// with the best one there, so none is skipped, and each stage's best is a real but modest
    /// step over the stage before, so nothing makes the stage before pointless.
    #[test]
    fn every_armour_has_a_bonus_and_keeps_up_at_its_stage() {
        let items = items();
        for (name, _) in crate::entity::ARMOR_MATERIALS.iter().filter(|(n, _)| !n.starts_with("dye_")) {
            if set_defense(&items, name) == 0.0 {
                continue; // outfits worn for looks
            }
            assert!(SETS.iter().any(|s| s.prefix == *name), "{name} has no set bonus");
            assert!(SET_STAGE.iter().any(|s| s.0 == *name), "{name} has no stage");
        }
        let best = |stage: u8| SET_STAGE.iter().filter(|s| s.1 == stage).map(|s| set_defense(&items, s.0)).fold(0.0, f32::max);
        for (name, stage) in SET_STAGE {
            let (d, top) = (set_defense(&items, name), best(*stage));
            assert!(d >= ARMOUR_FLOOR * top, "{name}: {d} of defense, the best of its stage has {top}");
        }
        for s in 1..STAGES.len() as u8 {
            let k = best(s) / best(s - 1);
            assert!((1.05..=1.35).contains(&k), "{}: the best defense grows x{k:.2} over the stage before", STAGES[s as usize]);
        }
    }
}
