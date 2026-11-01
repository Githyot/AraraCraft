//! Animals' and monsters' footsteps: each foot that lands on the ground makes the sound of
//! the block under it (grass, sand, stone, snow...), louder for heavy bodies, silent for
//! tiny critters, fliers and swimmers. The walk cycle (`Mob::walk`) says when a foot lands:
//! twice per cycle, so a trotting horse patters and a strolling cow plods.

use glam::Vec3;

use crate::audio::{Material, Sfx};
use crate::Game;

/// Farther than this, no footsteps.
const HEAR: f32 = 16.0;
/// At most this many footsteps a frame (a stampede stays a sound, not a roar).
const MOST: usize = 6;

/// How loud a body's step is, from its size (0 for critters too light to be heard).
pub(crate) fn step_volume(width: f32, height: f32, scale: f32) -> f32 {
    let mass = width * height * scale.max(0.3);
    if mass < 0.12 {
        0.0
    } else {
        (0.12 + mass * 0.12).min(0.6)
    }
}

/// Did a foot land between `before` and `now` on the walk cycle (half a turn per step)?
pub(crate) fn foot_landed(before: f32, now: f32) -> bool {
    let half = std::f32::consts::PI;
    now > before && (now / half).floor() != (before / half).floor()
}

impl Game {
    pub(crate) fn mob_steps(&mut self, dt: f32) {
        if self.audio.is_none() || dt <= 0.0 {
            return;
        }
        let me = self.body.pos;
        let mut steps: Vec<(Vec3, Material, f32)> = Vec::new();
        for m in &self.mobs.list {
            if steps.len() >= MOST {
                break;
            }
            if !m.alive() || m.ghost || m.ridden || !m.body.on_ground || m.body.in_water || m.walk_speed < 0.4 {
                continue;
            }
            let d = &self.mobs.reg.defs[m.def];
            if d.flies || d.aquatic || d.parts.is_empty() {
                continue;
            }
            let dist = m.body.pos.distance(me);
            if dist > HEAR {
                continue;
            }
            let vol = step_volume(d.width, d.height, d.scale) * m.size.max(0.4);
            if vol <= 0.0 {
                continue;
            }
            let before = m.walk - m.walk_speed * dt * 2.2;
            if !foot_landed(before, m.walk) {
                continue;
            }
            let below = self.world.block_or_air((m.body.pos - Vec3::Y * 0.2).floor().as_ivec3());
            if below == voxel::block::AIR {
                continue;
            }
            steps.push((m.body.pos, Material::of(&self.reg.get(below).name), vol));
        }
        for (at, mat, vol) in steps {
            self.sfx_at(Sfx::Step(mat), at, vol, HEAR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heavy_steps_are_louder_and_critters_silent() {
        assert_eq!(step_volume(0.25, 0.15, 1.0), 0.0);
        let cat = step_volume(0.6, 0.7, 0.85);
        let cow = step_volume(0.9, 1.4, 1.0);
        let elephant = step_volume(2.5, 3.0, 1.2);
        assert!(cat > 0.0 && cat < cow && cow < elephant);
        assert!(elephant <= 0.6);
    }

    #[test]
    fn a_foot_lands_twice_a_cycle() {
        // Three whole cycles, from just after one landing to just before the next.
        let n = (0..370).filter(|k| foot_landed(0.01 + *k as f32 * 0.05, 0.01 + (*k + 1) as f32 * 0.05)).count();
        assert_eq!(n, 5);
    }
}
