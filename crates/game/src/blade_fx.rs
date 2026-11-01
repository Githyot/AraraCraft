//! What a melee swing looks and sounds like (HP 2026-11-01, "analise todas as espadas"): a trail
//! that follows the blade's arc for every combo swing, hit or miss, coloured by what the weapon
//! is made of (a faint glint for plain steel, gold for gold, rainbow fire for diamond, the
//! glow of a magic sword), and a cut that sounds like the weapon (a light blade hisses, a
//! greatsword or club whooms) with a hit that sounds like the material biting.
//!
//! The trail is drawn with glow motes along the arc in front of the eye, so it shows the same in
//! first and third person and needs no extra renderer. Its timing follows the swing animation
//! (the weapon's cooldown): quick for a katana, slow for a greatsword.

use glam::Vec3;

use crate::mods::combat::Weapon;
use crate::sfx_extra::Fx;
use crate::Game;

/// Chance and extra damage (times the sword's) of the swords whose trick hits the same target;
/// the balance sheet counts them (`proc_factor`).
const PROC_HIVE: (f32, f32) = (0.35, 0.35);
const PROC_UMBRAL: (f32, f32) = (0.2, 0.4);
const PROC_STAR: (f32, f32) = (0.3, 0.5);

/// How much a sword's own trick adds on average to its hits on one target (balance.rs).
pub fn proc_factor(name: &str) -> f32 {
    let p = match name {
        "hive_blade" => PROC_HIVE,
        "umbral_sword" => PROC_UMBRAL,
        "celestial_blade" => PROC_STAR,
        _ => (0.0, 0.0),
    };
    1.0 + p.0 * p.1
}

/// One swing's trail.
#[derive(Default, Clone, Copy)]
pub struct Trail {
    active: bool,
    start: f32,
    dur: f32,
    style: u8,
    reach: f32,
    tint: [f32; 3],
    /// 0 plain metal (a faint glint) .. 1 magic (a bright, wide trail).
    glow: f32,
    rainbow: bool,
    weight: f32,
    /// How far along the arc the motes have been laid (0..1).
    done: f32,
}

/// The look of a weapon's trail: colour, how bright (0..1) and whether it splits into a rainbow.
pub fn blade_look(name: &str) -> ([f32; 3], f32, bool) {
    // Named swords first (their own colours), then the material at the front of the name.
    let named: Option<([f32; 3], f32)> = match name {
        "hive_blade" => Some(([1.6, 1.1, 0.3], 0.7)),
        "sky_blade" => Some(([0.8, 1.3, 1.8], 0.6)),
        "umbral_sword" => Some(([1.0, 0.5, 1.6], 0.6)),
        "sanguinite_sword" => Some(([1.6, 0.3, 0.3], 0.6)),
        "enchanted_sword" => Some(([0.6, 1.4, 2.0], 0.85)),
        "star_saber" => Some(([1.8, 1.6, 0.6], 0.85)),
        "blue_katana" => Some(([0.5, 0.9, 2.0], 0.8)),
        "night_edge" => Some(([1.2, 0.5, 2.0], 0.9)),
        "true_night_edge" => Some(([0.8, 1.8, 1.0], 1.0)),
        "radiant_blade" | "true_radiant_blade" => Some(([2.2, 1.8, 0.7], 1.0)),
        "terra_blade" => Some(([0.6, 2.2, 0.7], 1.0)),
        "horseman_blade" => Some(([2.0, 1.0, 0.3], 0.85)),
        "nebula_blade" => Some(([1.8, 0.6, 1.8], 1.0)),
        "celestial_wrath" => Some(([2.0, 0.8, 2.0], 1.0)),
        "vine_blade" => Some(([0.6, 1.4, 0.4], 0.7)),
        "celestial_blade" => Some(([1.0, 1.3, 2.0], 0.85)),
        "eclipse_scythe" => Some(([1.6, 0.6, 0.3], 0.8)),
        "cactus_sword" => Some(([0.6, 1.1, 0.5], 0.1)),
        "bone_sword" => Some(([1.0, 0.95, 0.8], 0.15)),
        "ice_blade" => Some(([0.7, 1.3, 1.9], 0.6)),
        "frost_brand" => Some(([0.6, 1.4, 2.2], 0.85)),
        "light_blade" => Some(([0.6, 1.2, 2.4], 1.0)),
        "falcon_blade" => Some(([1.3, 1.1, 0.8], 0.35)),
        _ => None,
    };
    if let Some((t, g)) = named {
        return (t, g, false);
    }
    if matches!(name, "prism_saber" | "apex_blade") {
        return ([1.6, 1.6, 1.6], 1.0, true);
    }
    let mat = name.split('_').next().unwrap_or("");
    let (t, g): ([f32; 3], f32) = match mat {
        "wooden" | "skyroot" => ([0.8, 0.65, 0.45], 0.1),
        "stone" | "holystone" => ([0.8, 0.8, 0.78], 0.1),
        "golden" | "gold" => ([1.5, 1.15, 0.45], 0.45),
        "diamond" => return ([1.2, 1.6, 1.6], 0.5, true),
        "netherite" => ([0.6, 0.45, 0.5], 0.3),
        "copper" => ([1.2, 0.65, 0.4], 0.3),
        "bronze" => ([1.1, 0.75, 0.4], 0.3),
        "silver" => ([1.2, 1.2, 1.3], 0.35),
        "tungsten" => ([0.75, 1.0, 0.8], 0.3),
        "platinum" => ([0.95, 1.1, 1.25], 0.4),
        "magmarite" => ([1.9, 0.7, 0.2], 0.75),
        "cobalt" => ([0.5, 0.8, 1.8], 0.55),
        "palladium" => ([1.8, 0.7, 0.4], 0.55),
        "mythril" => ([0.6, 1.6, 1.1], 0.6),
        "orichalcum" => ([1.8, 0.6, 1.3], 0.6),
        "adamantite" => ([1.8, 0.4, 0.4], 0.6),
        "titanium" => ([1.05, 1.1, 1.2], 0.5),
        "radiant" => ([1.9, 1.7, 0.9], 0.8),
        "verdite" => ([0.7, 1.8, 0.5], 0.8),
        "lumite" => ([0.5, 1.9, 1.8], 1.0),
        "zanite" => ([1.3, 0.7, 1.9], 0.5),
        "gravitite" => ([1.7, 0.6, 1.7], 0.6),
        "resonarium" => ([0.5, 1.7, 1.6], 0.6),
        "umbral" => ([1.0, 0.5, 1.6], 0.5),
        "sanguinite" => ([1.6, 0.3, 0.3], 0.5),
        _ => ([0.9, 0.92, 0.98], 0.25),
    };
    (t, g, false)
}

/// The sound of a weapon biting: 0 wood or stone (a dull knock), 1 metal, 2 crystal, 3 magic.
pub fn bite_of(name: &str) -> u8 {
    let (_, glow, rainbow) = blade_look(name);
    let mat = name.split('_').next().unwrap_or("");
    if matches!(mat, "wooden" | "skyroot" | "stone" | "holystone" | "cactus" | "bone") || name.ends_with("_club") {
        0
    } else if rainbow || matches!(mat, "zanite" | "resonarium") {
        2
    } else if glow >= 0.8 {
        3
    } else {
        1
    }
}

/// A point on the arc of a swing, as a direction from the eye: `p` 0..1 along it.
fn arc_dir(style: u8, p: f32, fwd: Vec3, right: Vec3, up: Vec3) -> Vec3 {
    let lerp = |a: f32, b: f32| (a + (b - a) * p).to_radians();
    let (az, el) = match style {
        // Chop: from high and a little right down through the target.
        0 => (lerp(14.0, 4.0), lerp(58.0, -38.0)),
        // Slash: right to left, falling a little.
        1 => (lerp(68.0, -68.0), lerp(12.0, -14.0)),
        // Thrust: straight ahead.
        2 => (lerp(6.0, 0.0), lerp(-4.0, 0.0)),
        // Uppercut: from low right up to high left.
        3 => (lerp(34.0, -36.0), lerp(-36.0, 52.0)),
        // Any other swing (a backhand, a heavy two-handed blow...): left to right.
        _ => (lerp(-60.0, 60.0), lerp(8.0, -16.0)),
    };
    (fwd * az.cos() * el.cos() + right * az.sin() * el.cos() + up * el.sin()).normalize_or_zero()
}

fn hue(t: f32) -> [f32; 3] {
    let t = t.rem_euclid(1.0) * 6.0;
    let r = ((t - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let g = (2.0 - (t - 2.0).abs()).clamp(0.0, 1.0);
    let b = (2.0 - (t - 4.0).abs()).clamp(0.0, 1.0);
    [0.4 + 1.4 * r, 0.4 + 1.4 * g, 0.4 + 1.4 * b]
}

impl Game {
    /// Called by every combo swing (mods/combat.rs), hit or miss: sets how fast the arm swings
    /// and starts the trail; on a hit, the bite of the blade.
    pub(crate) fn blade_swing_fx(&mut self, w: &Weapon, style: u8, target: Option<usize>) {
        let hit = target.is_some();
        // The trail lasts about as long as the swing (fight_anim.rs times it by the cooldown).
        let rate = w.swing_rate();
        let name = self.held_key();
        let (tint, glow, rainbow) = blade_look(&name);
        if !self.settings.reduce_motion {
            self.mods.combat.trail = Trail {
                active: true,
                start: self.now,
                dur: 1.0 / (3.6 * rate) * 0.8,
                style,
                reach: w.reach(),
                tint,
                glow,
                rainbow,
                weight: w.weight(),
                done: 0.0,
            };
        }
        if hit {
            self.sfx(crate::audio::Sfx::Fx(Fx::Bite(bite_of(&name))));
        }
        if let Some(i) = target {
            self.blade_special(&name, i);
        }
    }

    /// The sword's own trick on a hit (Terraria ideas, made for a 3D world: bees and pumpkins
    /// fly to their targets, a star falls from above, spores hang in a cloud you can walk round).
    fn blade_special(&mut self, name: &str, i: usize) {
        use crate::effects::{BLEEDING, POISON};
        use crate::rpg::MOB_BURN;
        if i >= self.mobs.list.len() || !self.mobs.list[i].alive() {
            return;
        }
        let d = self.held_tool().map(|t| t.damage).unwrap_or(4.0);
        let eye = self.body.eye();
        let c = self.mobs.list[i].body.center();
        let layer = self.glow_layer();
        let roll = self.surv.rng.f32();
        match name {
            "magmarite_sword" => {
                self.add_mob_debuff(i, MOB_BURN, 4.0);
                for _ in 0..6 {
                    self.particles.spark(c, Vec3::ZERO, layer);
                }
            }
            "sanguinite_sword" => self.add_mob_debuff(i, BLEEDING, 4.0),
            "vine_blade" => {
                self.add_mob_debuff(i, POISON, 4.0);
                for k in 0..6 {
                    let a = k as f32;
                    self.particles.glow(c + Vec3::new(a.sin(), 0.2 * a - 0.5, a.cos()) * 0.5, Vec3::Y * 0.4, layer, [0.5, 1.4, 0.3], 0.05, 0.6);
                }
            }
            // Two bees leave the blade and sting.
            "hive_blade" if roll < PROC_HIVE.0 => {
                for k in 0..2 {
                    let from = eye + self.forward() * 0.6 + Vec3::new(k as f32 - 0.5, 0.2, 0.0) * 0.4;
                    for j in 0..8 {
                        let p = from + (c - from) * (j as f32 / 8.0) + Vec3::Y * ((j as f32 / 8.0) * std::f32::consts::PI).sin() * 0.5;
                        self.particles.glow(p, Vec3::ZERO, layer, [1.6, 1.15, 0.2], 0.05, 0.12 + j as f32 * 0.03);
                    }
                }
                self.mobs.list[i].hurt(d * PROC_HIVE.1, c);
            }
            // A shadow bursts out of the cut.
            "umbral_sword" if roll < PROC_UMBRAL.0 => {
                for k in 0..12 {
                    let a = k as f32 / 12.0 * std::f32::consts::TAU;
                    self.particles.glow(c, Vec3::new(a.cos(), 0.3, a.sin()) * 2.2, layer, [0.9, 0.35, 1.5], 0.07, 0.35);
                }
                self.mobs.list[i].hurt(d * PROC_UMBRAL.1, eye);
            }
            // A star drops on the target from above.
            "celestial_blade" if roll < PROC_STAR.0 => {
                for k in 0..10 {
                    let at = c + Vec3::Y * (6.0 - k as f32 * 0.6);
                    self.particles.glow(at, Vec3::new(0.0, -14.0, 0.0), layer, [1.8, 1.7, 1.0], 0.09, 0.08 + k as f32 * 0.02);
                }
                self.mobs.list[i].hurt(d * PROC_STAR.1, c + Vec3::Y * 4.0);
                self.sfx(crate::audio::Sfx::SpellHit(0));
            }
            // A light knock up: the target is lifted for a follow-up hit in the air.
            "sky_blade" => {
                let boss = self.mobs.reg.defs[self.mobs.list[i].def].boss;
                let m = &mut self.mobs.list[i];
                if m.max_health < 80.0 && m.body.on_ground && !boss {
                    m.body.vel.y = m.body.vel.y.max(5.5);
                }
                for k in 0..6 {
                    self.particles.glow(c + Vec3::new(0.0, -0.6, 0.0), Vec3::new((k as f32).sin(), 2.0, (k as f32).cos()) * 0.8, layer, [1.4, 1.6, 1.8], 0.05, 0.4);
                }
            }
            // Flaming pumpkin heads fly from the target to two more monsters nearby.
            "horseman_blade" if roll < 0.4 => self.blade_splash(i, 6.0, 2, d * 0.5, [2.0, 0.9, 0.2], None),
            // A cloud of spores hangs where the hit landed and poisons what is in it.
            "verdite_sword" if roll < 0.3 => self.blade_splash(i, 2.5, 6, 0.0, [0.6, 1.8, 0.5], Some((POISON, 3.0))),
            // A lunar flare: the hit spreads to everything close to the target.
            "lumite_sword" => self.blade_splash(i, 2.5, 4, d * 0.3, [0.5, 1.9, 1.8], None),
            _ => {}
        }
    }

    /// Hurts (and maybe afflicts) up to `n` other monsters within `r` of mob `i`, with a line of
    /// glow from the target to each.
    fn blade_splash(&mut self, i: usize, r: f32, n: usize, dmg: f32, tint: [f32; 3], debuff: Option<(u8, f32)>) {
        let c = self.mobs.list[i].body.center();
        let layer = self.glow_layer();
        let mut near: Vec<(usize, f32)> = self
            .mobs
            .list
            .iter()
            .enumerate()
            .filter(|(k, m)| (*k != i || debuff.is_some()) && m.alive() && !self.mobs.reg.defs[m.def].town && self.mobs.reg.defs[m.def].kind == voxel::mob::Kind::Hostile)
            .map(|(k, m)| (k, (m.body.center() - c).length()))
            .filter(|(_, d)| *d < r)
            .collect();
        near.sort_by(|a, b| a.1.total_cmp(&b.1));
        for &(k, _) in near.iter().take(n) {
            let to = self.mobs.list[k].body.center();
            for j in 0..6 {
                let p = c + (to - c) * (j as f32 / 6.0) + Vec3::Y * ((j as f32 / 6.0) * std::f32::consts::PI).sin() * 0.6;
                self.particles.glow(p, Vec3::ZERO, layer, tint, 0.07, 0.15 + j as f32 * 0.03);
            }
            if dmg > 0.0 {
                self.mobs.list[k].hurt(dmg, c);
            }
            if let Some((id, t)) = debuff {
                self.add_mob_debuff(k, id, t);
            }
        }
    }

    /// Lays the motes of the swing's trail along the part of the arc swept since last frame.
    pub(crate) fn blade_fx_update(&mut self) {
        // Test hook MC_TRAIL=style (0 chop, 1 slash, 2 thrust, 3 uppercut): the held weapon swings
        // that way over and over in the air, so a screenshot catches its trail.
        static TEST: std::sync::OnceLock<Option<u8>> = std::sync::OnceLock::new();
        if let Some(style) = *TEST.get_or_init(|| std::env::var("MC_TRAIL").ok().and_then(|v| v.parse().ok())) {
            if !self.mods.combat.trail.active || self.now - self.mods.combat.trail.start > 0.6 {
                if let Some(w) = self.held_weapon() {
                    self.anim.start_styled_swing(style);
                    self.blade_swing_fx(&w, style, None);
                }
            }
        }
        let mut t = self.mods.combat.trail;
        if !t.active {
            return;
        }
        let p = ((self.now - t.start) / t.dur.max(0.05)).clamp(0.0, 1.0);
        // Ease out: the blade is fastest at the start of the cut.
        let e = |x: f32| 1.0 - (1.0 - x) * (1.0 - x);
        let (p0, p1) = (e(t.done), e(p));
        let eye = self.body.eye();
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(fwd).normalize_or_zero();
        let layer = self.glow_layer();
        let thrust = t.style == 2;
        let per = if thrust { 18.0 } else { 44.0 } * (0.6 + t.glow);
        let n = ((p1 - p0) * per).ceil() as usize;
        let size = 0.011 + 0.016 * t.glow + 0.008 * t.weight;
        let life = 0.07 + 0.09 * t.glow;
        // A plain blade leaves a thin bright glint; a magic one a wider streak of its colour.
        let k = 1.25;
        for i in 0..n {
            let q = p0 + (p1 - p0) * (i as f32 + 0.5) / n as f32;
            let dir = arc_dir(t.style, q, fwd, right, up);
            let tint = if t.rainbow { hue(q * 1.6 + self.now * 0.7) } else { t.tint };
            let tint = [tint[0] * k, tint[1] * k, tint[2] * k];
            // A band across the blade: from two thirds of the reach out to the tip.
            let bands = if t.glow > 0.5 { 2 } else { 1 };
            for b in 0..bands {
                let r = if thrust { t.reach * (0.35 + 0.6 * q) } else { t.reach * (0.62 + 0.1 * b as f32) };
                let at = eye + dir * r - up * 0.18;
                let along = arc_dir(t.style, (q + 0.05).min(1.0), fwd, right, up) - dir;
                self.particles.glow(at, along * 6.0, layer, tint, size * (1.0 - 0.2 * b as f32), life);
            }
        }
        // Magma and fire swords shed sparks along the cut.
        if t.tint[0] > 1.7 && t.tint[2] < 0.4 && n > 0 {
            let dir = arc_dir(t.style, p1, fwd, right, up);
            self.particles.spark(eye + dir * t.reach * 0.6, Vec3::ZERO, layer);
        }
        t.done = p;
        t.active = p < 1.0;
        self.mods.combat.trail = t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sword_has_a_look_and_a_bite() {
        assert!(blade_look("diamond_sword").2);
        assert!(blade_look("iron_sword").1 < 0.4);
        assert!(blade_look("terra_blade").1 >= 0.8);
        assert_eq!(bite_of("wooden_sword"), 0);
        assert_eq!(bite_of("iron_sword"), 1);
        assert_eq!(bite_of("diamond_sword"), 2);
        assert_eq!(bite_of("terra_blade"), 3);
    }

    #[test]
    fn arcs_start_and_end_apart() {
        let (f, r, u) = (Vec3::NEG_Z, Vec3::X, Vec3::Y);
        for s in 0..4u8 {
            let a = arc_dir(s, 0.0, f, r, u);
            let b = arc_dir(s, 1.0, f, r, u);
            assert!(a.dot(f) > 0.0 && b.dot(f) > 0.0, "style {s} swings behind");
        }
        assert!(arc_dir(1, 0.0, f, r, u).x > 0.5 && arc_dir(1, 1.0, f, r, u).x < -0.5);
    }

    #[test]
    fn rainbow_stays_in_range() {
        for i in 0..20 {
            let c = hue(i as f32 / 20.0);
            assert!(c.iter().all(|v| (0.3..=1.9).contains(v)));
        }
    }
}
