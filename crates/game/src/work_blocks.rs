//! Work blocks of lote B2 of the block review (plan/blocos-analise.md, tools/gen_blocos_b2.py):
//! what happens when one is used. Smoker and blast furnace are furnaces that only take food or
//! only ores and metal, twice as fast; the ender chest opens the player's own store (the same
//! from every ender chest); the trapped chest is a chest; the bell rings; the cauldron holds a
//! bucket of water or lava.

use crate::audio::{Material, Sfx};
use crate::survival::{Container, Furnace, Open};
use crate::Game;
use glam::IVec3;
use voxel::item::Inventory;

/// What a furnace-like block cooks: 0 anything (furnace), 1 food only, twice as fast (smoker),
/// 2 everything but food, twice as fast (blast furnace).
pub fn furnace_kind(name: &str) -> Option<u8> {
    match name {
        "furnace" => Some(0),
        "smoker" => Some(1),
        "blast_furnace" => Some(2),
        _ => None,
    }
}

impl Game {
    /// Use of a lote B2 work block by its base name. True when it did something.
    pub(crate) fn work_block_use(&mut self, base: &str, pos: IVec3) -> bool {
        match base {
            "smoker" | "blast_furnace" => {
                self.surv.containers.entry(pos).or_insert_with(|| {
                    Container::Furnace(Furnace { inv: Inventory::new(3), burn: 0.0, burn_max: 0.0, cook: 0.0, xp: 0.0 })
                });
                self.open_container(Open::Furnace(pos));
                true
            }
            // Copper chests (every stage of rust, waxed or not) hold as much as a chest.
            b if b == "trapped_chest" || b.ends_with("copper_chest") => {
                self.surv.containers.entry(pos).or_insert_with(|| Container::Chest(Inventory::new(27)));
                self.open_container(Open::Chest(pos));
                self.sfx(Sfx::Place(Material::Wood));
                true
            }
            // One store per player and world, the same behind every ender chest.
            "ender_chest" => self.open_bank(3),
            "bell" => {
                self.sfx(Sfx::Hit(Material::Metal));
                self.sfx(Sfx::Hit(Material::Metal));
                self.anim.start_swing();
                true
            }
            "cauldron" | "water_cauldron" | "lava_cauldron" => self.use_cauldron(base, pos),
            _ => false,
        }
    }

    fn use_cauldron(&mut self, base: &str, pos: IVec3) -> bool {
        let Some(held) = self.held_stack().map(|s| self.items.get(s.item).name.clone()) else { return false };
        let (to, give, sound) = match (base, held.as_str()) {
            ("cauldron", "water_bucket") => ("water_cauldron", "bucket", Sfx::Splash(0.4)),
            ("cauldron", "lava_bucket") => ("lava_cauldron", "bucket", Sfx::Fizz),
            ("water_cauldron", "bucket") => ("cauldron", "water_bucket", Sfx::Splash(0.3)),
            ("lava_cauldron", "bucket") => ("cauldron", "lava_bucket", Sfx::Fizz),
            _ => return false,
        };
        let Some(id) = self.reg.id(to) else { return false };
        // Keep the way it faces.
        let v = self.reg.get(self.world.block_or_air(pos)).variant;
        let id = self.reg.variant(id, v);
        self.world.set_block(pos, id);
        self.sfx(sound);
        self.swap_held(give);
        self.anim.start_swing();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn furnace_kinds() {
        assert_eq!(furnace_kind("furnace"), Some(0));
        assert_eq!(furnace_kind("smoker"), Some(1));
        assert_eq!(furnace_kind("blast_furnace"), Some(2));
        assert_eq!(furnace_kind("chest"), None);
    }

    #[test]
    fn work_blocks_load() {
        let (reg, items, recipes) = crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).unwrap();
        for n in ["campfire", "smoker", "blast_furnace", "loom", "stonecutter", "lectern", "bell", "cauldron", "water_cauldron", "ender_chest", "trapped_chest", "scaffolding", "red_candle", "blue_shulker_box"] {
            assert!(reg.id(n).is_some(), "{n}");
        }
        // Models stand in the world with real volume, never as crossed pictures.
        assert_eq!(reg.get(reg.id("campfire").unwrap()).shape, voxel::block::Shape::Model);
        let cauldron = items.id("cauldron").unwrap();
        assert!(recipes.shaped.iter().any(|r| r.result.item == cauldron));
    }
}
