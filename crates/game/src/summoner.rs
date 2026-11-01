//! The summoner's own feel (thread "Analisar e melhorar a classe invocador", lote 1):
//!
//! - Whips lash for real: the rope unrolls from the hand in 3D and cracks at the tip a moment
//!   after the click. Every monster along the lash is hit (a quarter less per monster after the
//!   first); the ones at the tip take the crack (+25%). The first one hit is marked: the minions
//!   go for it with the whip's bonus, and a ring in the whip's colour turns over it so you can
//!   see where they will go.
//! - Six whips, one or two per stage: leather, thorns (poison), bone (bleeding), frost (slows
//!   and frostbites), storm (sparks jump to the monsters next to the one hit) and the star whip
//!   (stars fall on the marked monster as the minions hit it).
//! - Sneak + right click with a summon staff marks the monster in the crosshair for the
//!   minions (no bonus, half a minute); sneak + right click at nothing clears it.
//! - Minions that shoot send real shots: they fly, home in a little and stop on stone.
//! - Minion sounds: each kind its own recorded voice, pitched down (mob_sounds.rs clips).

use glam::{IVec3, Mat4, Vec3};
use voxel::physics::Terrain;

use crate::audio::Sfx;
use crate::effects::*;
use crate::rpg::SUMMON;
use crate::Game;

/// What a whip does on top of its mark.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WhipFx {
    None,
    /// Slows the monsters it hits (chill) besides the frostbite.
    Frost,
    /// Sparks jump from each monster hit to up to two monsters near it.
    Spark,
    /// Stars fall on the marked monster as the minions strike it.
    Stars,
}

pub struct WhipDef {
    pub item: &'static str,
    /// The mark's bonus: the minions do this share more to the marked monster.
    pub tag: f32,
    /// Debuff the lash leaves (0 none).
    pub inflict: u8,
    /// How far the lash reaches, in blocks.
    pub reach: f32,
    pub fx: WhipFx,
    /// Colour of its mark and sparks.
    pub tint: [f32; 3],
}

pub const WHIPS: &[WhipDef] = &[
    WhipDef { item: "leather_whip", tag: 0.10, inflict: 0, reach: 5.0, fx: WhipFx::None, tint: [1.3, 0.9, 0.5] },
    WhipDef { item: "thorn_whip", tag: 0.15, inflict: POISON, reach: 5.5, fx: WhipFx::None, tint: [0.6, 1.4, 0.4] },
    WhipDef { item: "bone_whip", tag: 0.20, inflict: BLEEDING, reach: 6.0, fx: WhipFx::None, tint: [1.4, 1.3, 1.1] },
    WhipDef { item: "frost_whip", tag: 0.22, inflict: FROSTBURN, reach: 6.5, fx: WhipFx::Frost, tint: [0.6, 1.2, 1.7] },
    WhipDef { item: "storm_whip", tag: 0.24, inflict: 0, reach: 7.0, fx: WhipFx::Spark, tint: [1.1, 1.1, 1.8] },
    WhipDef { item: "star_whip", tag: 0.26, inflict: 0, reach: 7.5, fx: WhipFx::Stars, tint: [0.5, 1.5, 1.6] },
];

pub fn whip_of(name: &str) -> Option<usize> {
    WHIPS.iter().position(|w| w.item == name)
}

/// How long a lash takes, from the click until the rope is back.
pub const LASH_TIME: f32 = 0.36;
/// When in the lash the tip cracks (and the hits land), as a share of `LASH_TIME`.
const CRACK_AT: f32 = 0.45;
/// Rope segments drawn between the handle and the tip.
pub const ROPE: usize = 12;

/// A lash in the air.
pub struct Lash {
    pub whip: usize,
    pub t: f32,
    /// Where it was aimed.
    pub dir: Vec3,
    /// Damage of the swing (taken at the click, with the weapon's readiness).
    dmg: f32,
    cracked: bool,
}

/// A shot a minion sent.
pub struct MinionShot {
    pub pos: Vec3,
    pub vel: Vec3,
    /// The monster it flies at (net id).
    pub target: u32,
    pub dmg: f32,
    pub inflict: u8,
    pub tint: [f32; 3],
    pub life: f32,
    /// Fireballs are bigger than sparks.
    pub size: f32,
    /// Minion kind (its impact sound).
    pub kind: usize,
}

/// The summoner's state (inside `Rpg`).
#[derive(Default)]
pub struct Summoner {
    pub lash: Option<Lash>,
    pub shots: Vec<MinionShot>,
    /// The monster the staff marked (net id) and for how long.
    pub marked: Option<(u32, f32)>,
    /// Which whip made the current mark (its colour and special).
    pub tag_whip: usize,
    mark_fx: f32,
    /// Test scenes: lashes, shots and minions hold still for the picture.
    pub frozen: bool,
}

/// The rope's points, handle first, for a lash `t` seconds old aimed along `dir` from `hand`,
/// `reach` blocks long. The rope starts curled back over the shoulder and unrolls forward, the
/// far end last, so the tip whips out after the rest of it (the crack); then it sags back.
pub fn lash_points(hand: Vec3, dir: Vec3, reach: f32, t: f32) -> [Vec3; ROPE + 1] {
    let fwd = dir.normalize_or(Vec3::NEG_Z);
    let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(fwd).normalize_or(Vec3::Y);
    let k = (t / LASH_TIME).clamp(0.0, 1.0);
    let smooth = |a: f32, b: f32, x: f32| {
        let v = ((x - a) / (b - a)).clamp(0.0, 1.0);
        v * v * (3.0 - 2.0 * v)
    };
    let back = (up * 0.75 - fwd * 0.65).normalize();
    let out_k = smooth(0.0, CRACK_AT, k);
    let sag = smooth(0.6, 1.0, k);
    let len = reach * (0.45 + 0.55 * out_k) * (1.0 - 0.45 * sag);
    let step = len / ROPE as f32;
    let mut pts = [hand; ROPE + 1];
    for i in 0..ROPE {
        let u = i as f32 / ROPE as f32;
        // Further along the rope turns over later.
        let turn = smooth(0.0, 1.0, k / CRACK_AT * 1.6 - u * 0.6);
        let mut d = (back * (1.0 - turn) + fwd * turn).normalize_or(fwd);
        // A little S across, dying out, and the sag pulling the far end down at the end.
        d += right * (u * std::f32::consts::PI).sin() * 0.25 * (1.0 - k);
        d -= up * u * u * 0.9 * sag;
        pts[i + 1] = pts[i] + d.normalize_or(fwd) * step;
    }
    pts
}

impl Game {
    /// Where the lash leaves the hand.
    fn whip_hand(&self) -> Vec3 {
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
        self.body.eye() + right * 0.32 - Vec3::Y * 0.28 + fwd * 0.3
    }

    /// A swing with a whip in hand. Returns true when the swing was taken (a lash started).
    /// With a block in the crosshair and no monster near the line, the click digs instead.
    pub(crate) fn whip_swing(&mut self, block_dist: f32) -> bool {
        let Some(w) = whip_of(&self.held_key()) else { return false };
        let eye = self.body.eye();
        let fwd = self.forward();
        let reach = WHIPS[w].reach;
        let near = self.lash_hits(eye, fwd, reach).first().is_some_and(|h| h.1 <= block_dist + 0.5);
        if !near && block_dist < reach {
            return false;
        }
        let base = if self.survival {
            let full = self.attack_charge() > 0.9;
            let d = self.surv_attack_damage();
            self.prog.was_full = full;
            d
        } else {
            4.0 + 2.0 * w as f32
        };
        self.attack_cd = if self.survival { 0.1 } else { 0.4 };
        self.break_cd = 0.3;
        self.t5.rpg.sum.lash = Some(Lash { whip: w, t: 0.0, dir: fwd, dmg: base, cracked: false });
        self.anim.start_swing();
        self.sfx(Sfx::Fx(crate::sfx_extra::Fx::Swing));
        true
    }

    /// Monsters along a lash from `eye` along `dir`, nearest first, with how far along they are.
    fn lash_hits(&self, eye: Vec3, dir: Vec3, reach: f32) -> Vec<(usize, f32)> {
        let mut hits: Vec<(usize, f32)> = Vec::new();
        for (i, m) in self.mobs.list.iter().enumerate() {
            if !m.alive() || m.ghost {
                continue;
            }
            let c = m.body.center();
            let to = c - eye;
            let s = to.dot(dir);
            if s < 0.3 || s > reach + m.body.half_width {
                continue;
            }
            // The lash fans out a little toward the tip.
            let side = (to - dir * s).length();
            if side > m.body.half_width + 0.45 + 0.35 * (s / reach) {
                continue;
            }
            if !self.clear_line(eye, c) {
                continue;
            }
            hits.push((i, s));
        }
        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
        // Town folk only when they are the first in line.
        if let Some(&(first, _)) = hits.first() {
            hits.retain(|&(i, _)| i == first || !self.mobs.reg.defs[self.mobs.list[i].def].town);
        }
        hits.truncate(6);
        hits
    }

    /// The lash in the air: the crack lands its hits.
    pub(crate) fn update_lash(&mut self, dt: f32) {
        let Some(l) = self.t5.rpg.sum.lash.as_mut() else { return };
        l.t += dt;
        let (w, dir, dmg, crack) = (l.whip, l.dir, l.dmg, !l.cracked && l.t >= LASH_TIME * CRACK_AT);
        if crack {
            l.cracked = true;
        }
        if l.t >= LASH_TIME {
            self.t5.rpg.sum.lash = None;
        }
        if !crack {
            return;
        }
        let def = &WHIPS[w];
        let eye = self.body.eye();
        let hand = self.whip_hand();
        let tip = lash_points(hand, dir, def.reach, LASH_TIME * CRACK_AT)[ROPE];
        let glow = self.glow_layer();
        for _ in 0..6 {
            let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.3, self.surv.rng.f32() - 0.5) * 2.5;
            self.particles.glow(tip, v, glow, def.tint, 0.06, 0.25);
        }
        let hits = self.lash_hits(eye, dir, def.reach);
        let tipped = hits.iter().any(|h| h.1 > def.reach * 0.7);
        self.sfx_at(Sfx::Fx(crate::sfx_extra::Fx::WhipCrack(tipped)), tip, 1.0, 24.0);
        if hits.is_empty() {
            return;
        }
        let mut sparked: Vec<usize> = Vec::new();
        for (n, &(i, s)) in hits.iter().enumerate() {
            let at_tip = s > def.reach * 0.7;
            let mut d = dmg * 0.75f32.powi(n as i32) * if at_tip { 1.25 } else { 1.0 };
            if self.survival {
                d = self.weapon_melee_bonus(i, d);
            }
            let m = &mut self.mobs.list[i];
            m.hurt(d, eye);
            m.body.knockback(m.body.pos - eye, if at_tip { 1.6 } else { 0.8 });
            let c = m.body.center();
            if n == 0 {
                let dd = &self.mobs.reg.defs[m.def];
                self.last_hit = Some((dd.display_name.clone(), m.health.max(0.0), m.max_health, self.now + 3.0));
                let id = m.net_id;
                self.t5.rpg.tagged = Some((id, 5.0, def.tag));
                self.t5.rpg.sum.tag_whip = w;
            }
            if def.inflict != 0 {
                self.add_mob_debuff(i, def.inflict, 4.0);
            }
            if def.fx == WhipFx::Frost {
                self.add_mob_debuff(i, CHILLED, 3.0);
            }
            if at_tip {
                for _ in 0..8 {
                    let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 3.0;
                    self.particles.glow(c, v, glow, def.tint, 0.07, 0.35);
                }
            }
            if def.fx == WhipFx::Spark {
                sparked.push(i);
            }
            if self.survival {
                self.rpg_after_melee(i, d);
                self.prog.stats.add("damage_dealt", (d * 10.0) as u64);
            }
        }
        // Storm whip: a spark from each monster hit to up to two others near it.
        for &i in &sparked {
            let from = self.mobs.list[i].body.center();
            let mut near: Vec<(usize, f32)> = (0..self.mobs.list.len())
                .filter(|&j| j != i && !hits.iter().any(|h| h.0 == j) && self.mobs.list[j].alive() && self.is_hostile(j))
                .map(|j| (j, self.mobs.list[j].body.center().distance(from)))
                .filter(|&(_, d)| d < 5.0)
                .collect();
            near.sort_by(|a, b| a.1.total_cmp(&b.1));
            for &(j, _) in near.iter().take(2) {
                let to = self.mobs.list[j].body.center();
                let seg = to - from;
                for s in 0..10 {
                    let jit = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.25;
                    self.particles.glow(from + seg * (s as f32 / 10.0) + jit, Vec3::ZERO, glow, def.tint, 0.06, 0.2);
                }
                self.mobs.list[j].hurt(dmg * 0.5 * self.t5.rpg.stats.dmg[SUMMON].max(0.5), from);
            }
        }
        if !sparked.is_empty() {
            let at = from_first(&hits, self);
            self.sfx_at(Sfx::Cast(2), at, 0.6, 20.0);
        }
        self.pads.rumble(0.3, 0.5, 0.08);
        if self.survival {
            self.wear_held(1);
        }
    }

    /// The whip's rope drawn as its 3D model (summons.rs).
    pub(crate) fn whip_instances(&self, cam_block: IVec3) -> Vec<(usize, crate::entity::Bones)> {
        let Some(l) = &self.t5.rpg.sum.lash else { return Vec::new() };
        let model = crate::summons::WHIP0 + l.whip;
        let Some(mesh) = self.entities.summon_mesh(model) else { return Vec::new() };
        let hand = self.whip_hand();
        let pts = lash_points(hand, l.dir, WHIPS[l.whip].reach, l.t);
        let c = cam_block.as_vec3();
        let pts: Vec<Vec3> = pts.iter().map(|p| *p - c).collect();
        let lt = self.world.light(hand.floor().as_ivec3());
        let light = [(lt >> 4) as f32 / 15.0, ((lt & 15) as f32 / 15.0).max(self.hand_light_level()), 0.0, 0.0];
        vec![(mesh, crate::summons::whip_bones(model, &pts, light))]
    }

    /// Test scenes: a lash of whip `w` caught `t` seconds in (screenshots run with no time).
    pub(crate) fn test_lash(&mut self, w: usize, t: f32) {
        self.t5.rpg.sum.lash = Some(Lash { whip: w, t, dir: self.forward(), dmg: 0.0, cracked: true });
        self.t5.rpg.sum.frozen = true;
        // Screenshots run no time: the marks and shots show what they would leave this frame.
        for _ in 0..3 {
            self.update_marks(0.0);
            self.update_minion_shots(0.0);
        }
    }

    // -----------------------------------------------------------------------------------------
    // Marks.

    /// Sneak + right click with a summon staff: the monster in the crosshair is the minions'
    /// target (or the mark is cleared).
    pub(crate) fn mark_for_minions(&mut self) -> bool {
        let eye = self.body.eye();
        let fwd = self.forward();
        let block = self.target.map(|t| t.distance).unwrap_or(f32::MAX);
        let hit = self.mobs.pick(eye, fwd, 40.0).filter(|&(k, t)| t < block && self.is_hostile(k));
        match hit {
            Some((k, _)) => {
                let id = self.mobs.list[k].net_id;
                self.t5.rpg.sum.marked = Some((id, 30.0));
                let name = self.mobs.reg.defs[self.mobs.list[k].def].display_name.clone();
                self.toast(format!("Lacaios: alvo {name}"), 1.5);
                let c = self.mobs.list[k].body.center();
                let glow = self.glow_layer();
                for n in 0..16 {
                    let a = n as f32 / 16.0 * std::f32::consts::TAU;
                    self.particles.glow(c + Vec3::new(a.cos(), 0.0, a.sin()) * 0.8, Vec3::Y * 1.5, glow, [1.6, 1.3, 0.5], 0.07, 0.5);
                }
                self.sfx(Sfx::Cast(3));
            }
            None => {
                if self.t5.rpg.sum.marked.take().is_some() {
                    self.toast("Lacaios: alvo livre", 1.2);
                }
            }
        }
        self.anim.start_swing();
        true
    }

    /// The staff's mark, if that monster is still about.
    pub(crate) fn marked_mob(&self) -> Option<usize> {
        let (id, _) = self.t5.rpg.sum.marked?;
        self.mobs.list.iter().position(|m| m.net_id == id && m.alive())
    }

    /// Marks time out; the marked monsters show a ring (the whip's colour) or a gold diamond
    /// (the staff's), so you can see where the minions will go.
    pub(crate) fn update_marks(&mut self, dt: f32) {
        if let Some(m) = self.t5.rpg.sum.marked.as_mut() {
            m.1 -= dt;
        }
        if self.t5.rpg.sum.marked.is_some_and(|m| m.1 <= 0.0) || (self.t5.rpg.sum.marked.is_some() && self.marked_mob().is_none()) {
            self.t5.rpg.sum.marked = None;
        }
        self.t5.rpg.sum.mark_fx -= dt;
        if self.t5.rpg.sum.mark_fx > 0.0 && !self.t5.rpg.sum.frozen {
            return;
        }
        self.t5.rpg.sum.mark_fx = 0.09;
        let glow = self.glow_layer();
        let spin = self.now * 3.0;
        if let Some((id, _, _)) = self.t5.rpg.tagged {
            if let Some(k) = self.mobs.list.iter().position(|m| m.net_id == id && m.alive()) {
                let b = &self.mobs.list[k].body;
                let top = b.pos + Vec3::Y * (b.height + 0.25);
                let r = b.half_width + 0.35;
                let tint = WHIPS[self.t5.rpg.sum.tag_whip.min(WHIPS.len() - 1)].tint;
                for n in 0..6 {
                    let a = spin + n as f32 / 6.0 * std::f32::consts::TAU;
                    self.particles.glow(top + Vec3::new(a.cos() * r, 0.0, a.sin() * r), Vec3::ZERO, glow, tint, 0.06, 0.12);
                }
            }
        }
        if let Some(k) = self.marked_mob() {
            let b = &self.mobs.list[k].body;
            let top = b.pos + Vec3::Y * (b.height + 0.7 + (self.now * 2.0).sin() * 0.08);
            for n in 0..4 {
                let a = spin * 0.5 + n as f32 / 4.0 * std::f32::consts::TAU;
                self.particles.glow(top + Vec3::new(a.cos() * 0.18, (n % 2) as f32 * 0.0, a.sin() * 0.18), Vec3::ZERO, glow, [1.7, 1.35, 0.45], 0.07, 0.12);
            }
            self.particles.glow(top - Vec3::Y * 0.2, Vec3::ZERO, glow, [1.7, 1.35, 0.45], 0.05, 0.12);
        }
    }

    // -----------------------------------------------------------------------------------------
    // Minion shots and voices.

    pub(crate) fn add_minion_shot(&mut self, s: MinionShot) {
        self.t5.rpg.sum.shots.push(s);
    }

    /// Shots fly toward their monster, homing in a little; stone stops them.
    pub(crate) fn update_minion_shots(&mut self, dt: f32) {
        if self.t5.rpg.sum.shots.is_empty() {
            return;
        }
        let glow = self.glow_layer();
        let mut shots = std::mem::take(&mut self.t5.rpg.sum.shots);
        shots.retain_mut(|s| {
            s.life -= dt;
            if s.life <= 0.0 {
                return false;
            }
            let target = self.mobs.list.iter().position(|m| m.net_id == s.target && m.alive());
            if let Some(k) = target {
                let c = self.mobs.list[k].body.center();
                let speed = s.vel.length().max(8.0);
                let want = (c - s.pos).normalize_or_zero() * speed;
                s.vel += (want - s.vel) * (dt * 5.0).min(1.0);
            }
            let from = s.pos;
            s.pos += s.vel * dt;
            // Glowing trail.
            self.particles.glow(s.pos, -s.vel * 0.05, glow, s.tint, s.size, 0.18);
            self.particles.glow((s.pos + from) * 0.5, Vec3::ZERO, glow, s.tint, s.size * 0.6, 0.12);
            if self.world.solid(s.pos.floor().as_ivec3()) {
                for _ in 0..5 {
                    let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 2.0;
                    self.particles.glow(from, v, glow, s.tint, s.size * 0.6, 0.3);
                }
                return false;
            }
            let Some(k) = target else { return true };
            let m = &self.mobs.list[k];
            if m.body.center().distance(s.pos) > m.body.half_width + 0.5 {
                return true;
            }
            self.mobs.list[k].hurt(s.dmg, from);
            if s.inflict != 0 {
                self.add_mob_debuff(k, s.inflict, 3.0);
            }
            let m = &self.mobs.list[k];
            let d = &self.mobs.reg.defs[m.def];
            self.last_hit = Some((d.display_name.clone(), m.health.max(0.0), m.max_health, self.now + 3.0));
            for _ in 0..8 {
                let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 3.0;
                self.particles.glow(s.pos, v, glow, s.tint, s.size * 0.7, 0.35);
            }
            self.minion_voice(s.kind, s.pos, true);
            self.star_fall(k, s.dmg);
            false
        });
        // Shots sent while these flew (none today) stay.
        shots.append(&mut self.t5.rpg.sum.shots);
        self.t5.rpg.sum.shots = shots;
    }

    /// The star whip: a minion hit on the monster it marked sometimes brings a star down on it.
    pub(crate) fn star_fall(&mut self, k: usize, hit: f32) {
        let Some((id, _, _)) = self.t5.rpg.tagged else { return };
        if WHIPS[self.t5.rpg.sum.tag_whip.min(WHIPS.len() - 1)].fx != WhipFx::Stars || self.mobs.list[k].net_id != id || self.surv.rng.f32() > 0.3 {
            return;
        }
        let c = self.mobs.list[k].body.center();
        let glow = self.glow_layer();
        for j in 0..14 {
            let at = c + Vec3::new(0.6, 6.0 - j as f32 * 0.42, 0.3);
            self.particles.glow(at, Vec3::new(-0.6, -12.0, -0.3), glow, [1.6, 1.4, 0.6], 0.09, 0.2);
        }
        for _ in 0..10 {
            let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 4.0;
            self.particles.glow(c, v, glow, [0.6, 1.5, 1.7], 0.08, 0.4);
        }
        self.mobs.list[k].hurt(hit * 0.8, c + Vec3::Y * 6.0);
    }

    /// A minion's voice when it strikes (or its shot lands): recorded creature sounds, pitched
    /// down so nothing is shrill, each kind its own.
    pub(crate) fn minion_voice(&mut self, kind: usize, at: Vec3, impact: bool) {
        use crate::summons::{BLADE, DRAGON, IMP, JELLY, WASP, WISP};
        let r = 0.9 + self.surv.rng.f32() * 0.15;
        let fx = match (crate::summons::minion_model(kind), impact) {
            (WISP, false) => crate::mob_sounds::clip("vex_hurt", 0.7 * r),
            (WISP, true) => Some(crate::sfx_extra::Fx::LavaPop),
            (WASP, _) => crate::mob_sounds::clip(if self.surv.rng.f32() < 0.5 { "spider_attack_1" } else { "spider_attack_2" }, 1.05 * r),
            (IMP, false) => crate::mob_sounds::clip("vl_elemental_fire_breath", 0.95 * r),
            (IMP, true) => Some(crate::sfx_extra::Fx::FireCrackle),
            (BLADE, _) => Some(crate::sfx_extra::Fx::Armor(true)),
            (JELLY, _) => crate::mob_sounds::clip("green_slime_jump", 0.85 * r + if kind == 5 { 0.1 } else { 0.0 }),
            (DRAGON, _) => crate::mob_sounds::clip("guardian_random_1", 0.6 * r),
            (crate::summons::SPIDER, _) => crate::mob_sounds::clip(if self.surv.rng.f32() < 0.5 { "spider_attack_1" } else { "spider_attack_2" }, 0.8 * r),
            (crate::summons::CROW, _) => Some(crate::sfx_extra::Fx::Wings),
            _ => None,
        };
        if let Some(fx) = fx {
            self.sfx_at(Sfx::Fx(fx), at, 0.55, 20.0);
        }
    }
}

fn from_first(hits: &[(usize, f32)], g: &Game) -> Vec3 {
    hits.first().map(|h| g.mobs.list[h.0].body.center()).unwrap_or(g.body.pos)
}

/// Bones of a whip model on the rope's points (camera-relative), handle first.
pub fn rope_place(a: Vec3, b: Vec3, k: f32) -> Mat4 {
    let d = b - a;
    let len = d.length().max(0.01);
    Mat4::from_translation(a) * Mat4::from_quat(crate::summons::face(d)) * Mat4::from_scale(Vec3::new(k, k, len / 16.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whips_grow_with_the_stages() {
        for w in WHIPS.windows(2) {
            assert!(w[1].tag >= w[0].tag && w[1].reach > w[0].reach, "{} -> {}", w[0].item, w[1].item);
        }
        assert_eq!(whip_of("star_whip"), Some(5));
    }

    #[test]
    fn the_lash_unrolls_to_its_reach() {
        let hand = Vec3::ZERO;
        let dir = Vec3::NEG_Z;
        let start = lash_points(hand, dir, 6.0, 0.0);
        let crack = lash_points(hand, dir, 6.0, LASH_TIME * CRACK_AT);
        // Curled back at first (the tip behind or above the hand), out in front at the crack.
        assert!(start[ROPE].z > -2.0, "{:?}", start[ROPE]);
        assert!(crack[ROPE].z < -4.5, "{:?}", crack[ROPE]);
        for p in crack {
            assert!(p.is_finite());
        }
    }
}
