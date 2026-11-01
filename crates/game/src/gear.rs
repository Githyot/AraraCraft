//! Area tools (Silent Gear idea): a hammer breaks the 3x3 of stone around the block you mine,
//! across the face you hit; an excavator does the same for dirt, sand and gravel; a sickle
//! reaps every plant in a 5x5 around. Crouching works one block at a time.

use glam::IVec3;
use voxel::block::{Shape, ToolKind};
use voxel::{BlockId, AIR};

use crate::Game;

impl Game {
    pub(crate) fn area_break(&mut self, center: IVec3, normal: IVec3, id: BlockId) {
        if self.crouch > 0.5 {
            return;
        }
        let Some(name) = self.held_stack().map(|s| self.items.get(s.item).name.clone()) else { return };
        let mut spots: Vec<IVec3> = Vec::new();
        if name.ends_with("_hammer") || name.ends_with("_excavator") {
            let want = if name.ends_with("_hammer") { ToolKind::Pickaxe } else { ToolKind::Shovel };
            let hard = self.reg.get(id).hardness;
            for a in -1..=1 {
                for b in -1..=1 {
                    if a == 0 && b == 0 {
                        continue;
                    }
                    let off = match (normal.x != 0, normal.y != 0) {
                        (true, _) => IVec3::new(0, a, b),
                        (_, true) => IVec3::new(a, 0, b),
                        _ => IVec3::new(a, b, 0),
                    };
                    let p = center + off;
                    let bid = self.world.block_or_air(p);
                    let bd = self.reg.get(bid);
                    // Only blocks this kind of tool is for, no harder than the one mined.
                    if bid != AIR && bd.tool == Some(want) && bd.hardness >= 0.0 && bd.hardness <= hard + 1.5 {
                        spots.push(p);
                    }
                }
            }
        } else if name.ends_with("_sickle") && self.reg.get(id).shape == Shape::Cross {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    for dy in -1..=1 {
                        let p = center + IVec3::new(dx, dy, dz);
                        if p != center && self.reg.get(self.world.block_or_air(p)).shape == Shape::Cross {
                            spots.push(p);
                        }
                    }
                }
            }
        }
        for p in spots {
            let bid = self.world.block_or_air(p);
            if self.held_stack().is_none() || !self.world.break_block(p) {
                continue;
            }
            if self.survival {
                self.surv_block_broken(p, bid);
            }
            let (layer, tint) = self.block_look(bid);
            self.particles.block_bits(p, layer, tint, 6, 0.8);
        }
    }
}
