//! A magic robe worn whole shows its school around the wearer (HP, 2026-11-01: "os mantos que
//! forem mágicos, quando estiverem completos, poderiam ter algum efeito de brilho ou de
//! partículas"): fire licks and embers for the pyromancer, falling snow for the cryomancer,
//! crackling sparks for the electromancer, a turning halo for the priest, motes pulled into the
//! dark for the voidwalker, dripping blood for the blood mage, a ring of runes for the cultist,
//! circling green glyphs for the archevoker and rising wisps for the spectral set.

use glam::Vec3;
use std::f32::consts::TAU;

use crate::Game;

/// Robes with an aura, by set prefix.
pub const ROBES: [&str; 9] = ["pyromancer", "cryomancer", "electromancer", "priest", "voidwalker", "bloodmage", "cultist", "archevoker", "spectral"];

impl Game {
    /// The worn robe's aura (`dt` seconds of it), around the player standing at `me`.
    pub(crate) fn robe_aura(&mut self, me: Vec3, glow: u16, dt: f32) {
        let Some(set) = self.t5.rpg.stats.set.map(|i| crate::rpg::SETS[i].prefix) else { return };
        if !ROBES.contains(&set) {
            return;
        }
        let t = self.now;
        let vel = self.body.vel;
        // About `rate` particles a second, whatever the frame rate.
        let n = |rate: f32, rng: &mut voxel::rng::Rng| {
            let want = rate * dt;
            want.floor() as usize + (rng.f32() < want.fract()) as usize
        };
        // A point on the robe: round the body at height `y` (feet 0, head top 1.8).
        let around = |rng: &mut voxel::rng::Rng, y0: f32, y1: f32, r: f32| {
            let a = rng.f32() * TAU;
            me + Vec3::new(a.cos() * r, y0 + rng.f32() * (y1 - y0), a.sin() * r)
        };
        let rng = &mut self.surv.rng;
        let p = &mut self.particles;
        match set {
            "pyromancer" => {
                for _ in 0..n(18.0, rng) {
                    let at = around(rng, 0.05, 0.5, 0.36);
                    p.flame(at, vel, glow, 0.09, 0.5, false);
                }
                for _ in 0..n(4.0, rng) {
                    let at = around(rng, 0.2, 1.2, 0.3);
                    p.spark(at, vel, glow);
                }
            }
            "cryomancer" => {
                for _ in 0..n(12.0, rng) {
                    let at = around(rng, 1.6, 2.1, 0.55);
                    p.mote(at, glow, [0.85, 0.95, 1.1], 0.06, 0.5, 2.2);
                }
                for _ in 0..n(6.0, rng) {
                    let at = around(rng, 0.02, 0.1, 0.45);
                    p.glow(at, vel, glow, [0.5, 0.9, 1.4], 0.09, 0.7);
                }
            }
            "electromancer" => {
                // Short zigzags that jump between two points on the body.
                for _ in 0..n(7.0, rng) {
                    let (a, b) = (around(rng, 0.6, 1.5, 0.38), around(rng, 0.6, 1.5, 0.38));
                    let mut q = a;
                    for k in 1..=6 {
                        let next = a.lerp(b, k as f32 / 6.0) + Vec3::new(rng.f32() - 0.5, rng.f32() - 0.5, rng.f32() - 0.5) * 0.12;
                        for j in 0..3 {
                            p.glow(q.lerp(next, j as f32 / 3.0), vel, glow, [1.4, 1.3, 0.6], 0.05, 0.35);
                        }
                        q = next;
                    }
                }
            }
            "priest" => {
                // A halo turning over the head and a few golden motes rising.
                for k in 0..10 {
                    let a = t * 1.2 + k as f32 / 10.0 * TAU;
                    if rng.f32() < dt * 30.0 {
                        p.glow(me + Vec3::new(a.cos() * 0.36, 2.3, a.sin() * 0.36), vel, glow, [1.5, 1.3, 0.6], 0.07, 0.25);
                    }
                }
                for _ in 0..n(3.0, rng) {
                    let at = around(rng, 0.2, 1.4, 0.35);
                    p.glow(at, vel + Vec3::Y * 0.4, glow, [1.4, 1.2, 0.6], 0.06, 1.2);
                }
            }
            "voidwalker" => {
                // Dark motes drawn in toward the body from a little way off.
                for _ in 0..n(10.0, rng) {
                    let at = around(rng, 0.1, 1.8, 1.1);
                    let target = me + Vec3::Y * (0.6 + rng.f32() * 0.8);
                    p.glow(at, (target - at) * 1.6 + vel, glow, [0.55, 0.15, 0.9], 0.08, 0.7);
                }
            }
            "bloodmage" => {
                for _ in 0..n(9.0, rng) {
                    let at = around(rng, 0.4, 1.3, 0.36);
                    p.dust(at, vel + Vec3::new(0.0, -1.2, 0.0), glow, [0.75, 0.05, 0.08], 0.06, 0.7);
                }
                for _ in 0..n(7.0, rng) {
                    let at = around(rng, 0.8, 1.6, 0.4);
                    p.glow(at, vel, glow, [1.3, 0.15, 0.2], 0.08, 0.8);
                }
            }
            "cultist" => {
                // A ring of runes slowly turning round the waist.
                for k in 0..8 {
                    let a = -t * 0.8 + k as f32 / 8.0 * TAU;
                    if rng.f32() < dt * 20.0 {
                        p.glow(me + Vec3::new(a.cos() * 0.6, 0.85 + (t * 2.0 + k as f32).sin() * 0.05, a.sin() * 0.6), vel, glow, [0.3, 1.3, 1.2], 0.08, 0.25);
                    }
                }
            }
            "archevoker" => {
                // Glyphs circling the body at three heights.
                for k in 0..3 {
                    let a = t * (1.0 + k as f32 * 0.4) + k as f32 * 2.1;
                    if rng.f32() < dt * 25.0 {
                        p.glow(me + Vec3::new(a.cos() * 0.55, 0.4 + k as f32 * 0.5, a.sin() * 0.55), vel, glow, [0.6, 1.4, 0.5], 0.09, 0.4);
                    }
                }
            }
            _ => {
                // Spectral: pale wisps rising off the robe.
                for _ in 0..n(8.0, rng) {
                    let at = around(rng, 0.0, 1.2, 0.3);
                    p.glow(at, vel + Vec3::Y * 0.7, glow, [0.7, 1.0, 1.5], 0.08, 1.1);
                }
            }
        }
    }
}

impl Game {
    /// Test hook MC_AURA=1: screenshots freeze time, so run the aura for a second and a half
    /// once (the particles moving too) before the picture is taken.
    pub(crate) fn robe_aura_warmup(&mut self) {
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if std::env::var("MC_AURA").is_err() || !self.world_open || self.now < 1.0 || DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        let (me, glow, t0) = (self.body.pos, self.glow_layer(), self.now);
        for _ in 0..90 {
            self.now += 1.0 / 60.0;
            self.robe_aura(me, glow, 1.0 / 60.0);
            self.particles.update(&self.world, 1.0 / 60.0);
        }
        self.now = t0;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_aura_robe_is_a_set() {
        for r in super::ROBES {
            assert!(crate::rpg::SETS.iter().any(|s| s.prefix == r), "{r} has no set");
        }
    }
}
