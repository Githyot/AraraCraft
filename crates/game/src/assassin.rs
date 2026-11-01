//! The assassin class (HP, 2026-11-01: "vamos começar pela classe assassino"; plan/assassino-
//! analise.md). Calamity's rogue gave the idea of a class of its own that lives on throws and a
//! stealth meter; here it is built for a 3D first-person world, where walking around a monster
//! and hiding in the dark mean something:
//!
//! - Its own damage class (`rpg::ROGUE`): daggers, throwing knives, shuriken, javelins and
//!   grenades all count as the assassin's, with a higher base critical chance.
//! - Furtividade (stealth): with an assassin set or the shadow cloak on, a meter fills while you
//!   keep still, faster crouched, in the dark or inside your own smoke, and drains when you run.
//!   As it fills, monsters notice you from closer and closer (16 blocks down to about 4), and a
//!   monster that hasn't been hurt for a while loses you. A full meter turns the next dagger
//!   stab or throw into a stealth strike: double damage, it goes through, and knives fan out.
//! - Pelas costas (backstab): a dagger in the back of a monster, or in one that hasn't noticed
//!   you, hits two and a half times as hard; thrown blades get half again. Turn of the monster's
//!   body decides "behind", so circling it in 3D is the way.
//! - Passo sombrio: right click with a dagger dashes a few blocks the way you look, and you
//!   can't be hit during it. It doesn't break stealth.
//! - The smoke bomb: a cloud where monsters see nothing and forget you, and the meter fills
//!   three times as fast.
//! - Blade oils: a flask rubbed on the blades poisons, burns, chills or envenoms with every
//!   dagger stab and every thrown blade for three minutes.

use glam::{IVec3, Vec3};
use voxel::physics::Terrain;

use crate::audio::Sfx;
use crate::effects::{BLEEDING, CHILLED, POISON, VENOM};
use crate::item_render::GpuItem;
use crate::rpg::MOB_BURN;
use crate::sfx_extra::Fx;
use crate::Game;

/// Extra share of damage of a dagger stab from behind or on an unaware monster (+150%).
pub const BACKSTAB: f32 = 1.5;
/// The same for thrown blades (+50%).
pub const THROWN_BACKSTAB: f32 = 0.5;
/// A stealth strike hits this many times as hard.
pub const STRIKE_MULT: f32 = 2.0;
/// Meter points a second while keeping still (100 fills in four seconds).
const FILL: f32 = 25.0;
/// Meter points a second lost while running.
const DRAIN: f32 = 40.0;
/// How long the dash lasts, how fast, and the wait before the next.
const DASH_TIME: f32 = 0.16;
const DASH_SPEED: f32 = 32.0;
const DASH_CD: f32 = 2.5;
/// A smoke cloud: radius and seconds.
const SMOKE_R: f32 = 4.5;
const SMOKE_SECS: f32 = 8.0;
/// Seconds an oil stays on the blades.
const OIL_SECS: f32 = 180.0;

/// Blade oils: item, the effect each hit leaves, its seconds, the name on screen and its colour.
pub const OILS: &[(&str, u8, f32, &str, [u8; 3])] = &[
    ("poison_oil", POISON, 5.0, "Óleo venenoso", [110, 200, 80]),
    ("fire_oil", MOB_BURN, 4.0, "Óleo incendiário", [240, 130, 50]),
    ("frost_oil", CHILLED, 3.0, "Óleo gélido", [140, 200, 250]),
    ("venom_oil", VENOM, 4.0, "Óleo de peçonha", [170, 90, 220]),
];

pub fn is_dagger(name: &str) -> bool {
    name.ends_with("_dagger")
}

/// What a dagger leaves in the wound by itself.
fn dagger_debuff(name: &str) -> Option<(u8, f32)> {
    match name {
        "umbral_dagger" => Some((BLEEDING, 3.0)),
        "magmarite_dagger" => Some((MOB_BURN, 3.0)),
        "verdite_dagger" => Some((POISON, 4.0)),
        _ => None,
    }
}

struct Smoke {
    at: Vec3,
    t: f32,
}

struct Bomb {
    pos: Vec3,
    vel: Vec3,
    age: f32,
}

#[derive(Default)]
pub struct Rogue {
    /// The stealth meter, 0..stats.stealth_max.
    pub stealth: f32,
    /// The full meter has been announced (a breath out) since it last emptied.
    told: bool,
    /// The oil on the blades (index in OILS) and its seconds left.
    pub oil: Option<(usize, f32)>,
    dash_t: f32,
    dash_cd: f32,
    dash_dir: Vec3,
    smokes: Vec<Smoke>,
    bombs: Vec<Bomb>,
    /// Game time of the last attack (the meter waits a little after one).
    last_attack: f32,
    /// A short flash on screen when a strike or a backstab lands.
    pub flash: f32,
    /// Test scene MC_ASSASSIN=set (or 1): set up once.
    staged: bool,
}

impl Game {
    fn rogue_full(&self) -> bool {
        let max = self.t5.rpg.stats.stealth_max;
        max > 0.0 && self.rogue.stealth >= max - 0.01
    }

    /// The full armour set worn, by its prefix ("" for none).
    fn rogue_set(&self) -> &'static str {
        self.t5.rpg.stats.set.map(|k| crate::rpg::SETS[k].prefix).unwrap_or("")
    }

    /// The assassin attacks: a full meter is spent on a stealth strike (true); otherwise the
    /// attack gives most of the meter away.
    pub(crate) fn rogue_take_strike(&mut self) -> bool {
        self.rogue.last_attack = self.now;
        if self.rogue_full() {
            // The eclipse set gives half the meter back.
            self.rogue.stealth = if self.rogue_set() == "eclipse" { self.t5.rpg.stats.stealth_max * 0.5 } else { 0.0 };
            self.rogue.told = false;
            self.rogue.flash = 1.0;
            return true;
        }
        self.rogue.stealth *= 0.4;
        self.rogue.told = false;
        false
    }

    /// Whether mob `i` is hit "from behind": it hasn't noticed the player, or the player is
    /// behind its shoulders. Bosses always know where you are, so only their back counts.
    fn rogue_behind(&self, i: usize, from: Vec3) -> bool {
        let m = &self.mobs.list[i];
        let boss = self.mobs.reg.defs[m.def].boss;
        if !m.aggro && !boss {
            return true;
        }
        let facing = Vec3::new(m.yaw.sin(), 0.0, m.yaw.cos());
        let to = Vec3::new(from.x - m.body.pos.x, 0.0, from.z - m.body.pos.z).normalize_or_zero();
        facing.dot(to) < -0.35
    }

    /// The oil and a dagger's own bite, on mob `i`.
    fn rogue_coat(&mut self, i: usize, dagger: Option<&str>) {
        if let Some((k, _)) = self.rogue.oil {
            let (_, id, secs, _, _) = OILS[k];
            self.add_mob_debuff(i, id, secs);
        }
        if let Some((id, secs)) = dagger.and_then(dagger_debuff) {
            self.add_mob_debuff(i, id, secs);
        }
    }

    fn rogue_backstab_fx(&mut self, i: usize) {
        let at = self.mobs.list[i].body.center();
        let glow = self.glow_layer();
        for _ in 0..10 {
            let r = &mut self.surv.rng;
            let v = Vec3::new(r.f32() - 0.5, r.f32() * 0.6, r.f32() - 0.5) * 3.0;
            self.particles.glow(at, v, glow, [1.4, 0.15, 0.15], 0.05, 0.4);
        }
        self.sfx(Sfx::Fx(Fx::Stab));
        self.rogue.flash = self.rogue.flash.max(0.6);
    }

    /// A melee hit on mob `i` (both melee paths call this with the damage so far): daggers
    /// backstab, spend the meter on a strike and carry the oil; any weapon gets the
    /// accessories' backstab bonus.
    pub(crate) fn rogue_melee_hit(&mut self, i: usize, dmg: f32) -> f32 {
        if i >= self.mobs.list.len() {
            return dmg;
        }
        let held = self.held_key();
        let dagger = is_dagger(&held);
        let eye = self.body.eye();
        let extra = self.t5.rpg.stats.backstab;
        let mut d = dmg;
        if (dagger || extra > 0.0) && self.rogue_behind(i, eye) {
            let boss = self.mobs.reg.defs[self.mobs.list[i].def].boss;
            let bonus = (if dagger { BACKSTAB } else { 0.0 } + extra) * if boss { 0.5 } else { 1.0 };
            d *= 1.0 + bonus;
            self.rogue_backstab_fx(i);
        }
        if dagger {
            if self.rogue_take_strike() {
                d *= STRIKE_MULT;
                let at = self.mobs.list[i].body.center();
                let glow = self.glow_layer();
                for _ in 0..14 {
                    let r = &mut self.surv.rng;
                    let v = Vec3::new(r.f32() - 0.5, r.f32() - 0.3, r.f32() - 0.5) * 4.0;
                    self.particles.glow(at, v, glow, [0.6, 0.4, 1.5], 0.06, 0.5);
                }
                // The nightshade set's strikes poison.
                if self.rogue_set() == "nightshade" {
                    self.add_mob_debuff(i, POISON, 6.0);
                }
            }
            self.rogue_coat(i, Some(&held));
            self.sfx(Sfx::Fx(Fx::Stab));
        }
        d
    }

    /// A thrown blade of the assassin lands on mob `k` with `d` damage, thrown from `from`.
    pub(crate) fn rogue_ranged_hit(&mut self, k: usize, d: f32, from: Vec3) -> f32 {
        let mut d = d;
        // "Behind" is where the blade came from, so a knife thrown at a back counts.
        if self.rogue_behind(k, from) {
            let boss = self.mobs.reg.defs[self.mobs.list[k].def].boss;
            d *= 1.0 + (THROWN_BACKSTAB + self.t5.rpg.stats.backstab) * if boss { 0.5 } else { 1.0 };
            self.rogue_backstab_fx(k);
        }
        self.rogue_coat(k, None);
        let inflict = self.t5.rpg.stats.inflict.clone();
        for (id, secs) in inflict {
            self.add_mob_debuff(k, id, secs);
        }
        d
    }

    /// The throwing glove throws quicker.
    pub(crate) fn rogue_throw_speed(&self) -> f32 {
        if self.wears("throwing_glove") {
            0.8
        } else {
            1.0
        }
    }

    /// Hits that can't land: the dash slips past them.
    pub(crate) fn rogue_dodging(&self) -> bool {
        self.rogue.dash_t > 0.0
    }

    /// Right click with an item of the assassin (true when used).
    pub(crate) fn assassin_use(&mut self, name: &str) -> Option<bool> {
        if is_dagger(name) {
            return Some(self.rogue_dash());
        }
        if name == "smoke_bomb" {
            let f = self.forward();
            let from = self.body.eye() + f * 0.5;
            let vel = f * 14.0 + Vec3::Y * 2.5 + self.body.vel * 0.5;
            self.rogue.bombs.push(Bomb { pos: from, vel, age: 0.0 });
            if self.survival {
                let sel = self.selected;
                self.surv.inv.take(sel, 1);
            }
            self.anim.start_swing();
            self.sfx(Sfx::Fx(Fx::Throw(true)));
            return Some(true);
        }
        if let Some(k) = OILS.iter().position(|o| o.0 == name) {
            self.rogue.oil = Some((k, OIL_SECS));
            if self.survival {
                let sel = self.selected;
                self.surv.inv.take(sel, 1);
                // The empty flask comes back.
                if let Some(id) = self.items.id("glass_bottle") {
                    let left = self.surv.inv.add(&self.items, voxel::item::ItemStack::new(id, 1));
                    if let Some(st) = left {
                        self.surv.inv.slots[sel] = Some(st);
                    }
                }
            }
            self.sfx(Sfx::Fx(Fx::Brush));
            self.toast(format!("{} nas lâminas (3 minutos)", OILS[k].3), 2.0);
            return Some(true);
        }
        None
    }

    fn rogue_dash(&mut self) -> bool {
        if self.rogue.dash_cd > 0.0 || self.rogue.dash_t > 0.0 {
            return true;
        }
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        if flat == Vec3::ZERO {
            return true;
        }
        self.rogue.dash_dir = flat;
        self.rogue.dash_t = DASH_TIME;
        self.rogue.dash_cd = DASH_CD;
        self.sfx(Sfx::Fx(Fx::Dash));
        true
    }

    /// Each frame: the stealth meter, how well monsters see, the dash, smoke bombs and clouds.
    pub(crate) fn update_assassin(&mut self, dt: f32) {
        self.assassin_test_scene();
        let r = &mut self.rogue;
        r.flash = (r.flash - dt * 2.5).max(0.0);
        r.dash_cd = (r.dash_cd - dt).max(0.0);
        if let Some((_, t)) = r.oil.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                r.oil = None;
            }
        }
        self.update_rogue_dash(dt);
        self.update_smoke(dt);
        self.update_stealth(dt);
    }

    fn update_rogue_dash(&mut self, dt: f32) {
        if self.rogue.dash_t <= 0.0 {
            return;
        }
        self.rogue.dash_t -= dt;
        let v = self.rogue.dash_dir * DASH_SPEED;
        self.body.vel.x = v.x;
        self.body.vel.z = v.z;
        self.body.vel.y = self.body.vel.y.max(0.0);
        // A smear of shadow behind.
        let (layer, _) = self.block_look(self.reg.id_or_air("stone"));
        let at = self.body.pos + Vec3::Y * 0.9;
        for _ in 0..3 {
            let j = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.6;
            self.particles.dust(at + j, -v * 0.05, layer, [0.12, 0.11, 0.14], 0.12, 0.45);
        }
        if self.rogue.dash_t <= 0.0 {
            self.body.vel.x *= 0.25;
            self.body.vel.z *= 0.25;
        }
    }

    fn in_smoke(&self, p: Vec3) -> bool {
        self.rogue.smokes.iter().any(|s| (p - s.at).length() < SMOKE_R)
    }

    fn update_smoke(&mut self, dt: f32) {
        // Bombs in flight: they burst on the ground, on a monster, or after a second and a half.
        let mut bursts = Vec::new();
        let mut bombs = std::mem::take(&mut self.rogue.bombs);
        bombs.retain_mut(|b| {
            b.age += dt;
            b.vel.y -= 18.0 * dt;
            let next = b.pos + b.vel * dt;
            let hit = self.world.solid(next.floor().as_ivec3()) || self.mobs.list.iter().any(|m| m.alive() && (m.body.center() - next).length() < 0.8);
            if hit || b.age > 1.5 {
                bursts.push(b.pos);
                return false;
            }
            b.pos = next;
            true
        });
        self.rogue.bombs = bombs;
        for at in bursts {
            self.rogue.smokes.push(Smoke { at, t: SMOKE_SECS });
            self.sfx(Sfx::Fx(Fx::Smoke));
            let glow = self.glow_layer();
            for _ in 0..60 {
                let r = &mut self.surv.rng;
                let v = Vec3::new(r.f32() - 0.5, r.f32() * 0.5, r.f32() - 0.5) * 7.0;
                let g = 0.62 + r.f32() * 0.14;
                self.particles.glow(at, v * 0.6, glow, [g, g, g * 1.04], 0.2, 1.6);
            }
        }
        // Clouds: they swirl, thin out at the end and go (soft grey puffs).
        let glow = self.glow_layer();
        for s in self.rogue.smokes.iter_mut() {
            s.t -= dt;
        }
        self.rogue.smokes.retain(|s| s.t > 0.0);
        let clouds: Vec<(Vec3, f32)> = self.rogue.smokes.iter().map(|s| (s.at, s.t)).collect();
        for (at, t) in &clouds {
            let thick = (t / 2.0).min(1.0);
            let n = (14.0 * thick * (dt * 60.0).max(1.0)).ceil() as usize;
            for _ in 0..n {
                let r = &mut self.surv.rng;
                let a = r.f32() * std::f32::consts::TAU;
                let d = r.f32().sqrt() * SMOKE_R * 0.9;
                let p = *at + Vec3::new(a.cos() * d, r.f32() * 2.6 - 0.4, a.sin() * d);
                let v = Vec3::new(-a.sin(), 0.15, a.cos()) * 0.4;
                // Pale grey, small and many: it reads as a soft cloud, not as blocks.
                let g = 0.58 + r.f32() * 0.16;
                self.particles.glow(p, v, glow, [g, g, g * 1.04], 0.16 + r.f32() * 0.12, 1.4);
            }
        }
        self.mobs.smoke = clouds.iter().map(|(at, _)| (*at, SMOKE_R)).collect();
    }

    fn update_stealth(&mut self, dt: f32) {
        let st = &self.t5.rpg.stats;
        let max = st.stealth_max;
        let gen = st.stealth_gen;
        if max <= 0.0 || !self.survival {
            self.rogue.stealth = 0.0;
            self.rogue.told = false;
            self.mobs.sight = 1.0;
            return;
        }
        // Hurt: found out.
        if self.hurt_age < 0.2 {
            self.rogue.stealth = 0.0;
            self.rogue.told = false;
        }
        let speed = Vec3::new(self.body.vel.x, 0.0, self.body.vel.z).length();
        let crouched = self.crouch > 0.5;
        let soles = self.wears("silent_soles");
        let attacking = self.mouse_held[0] || self.now - self.rogue.last_attack < 0.6;
        let mut rate = if self.sprinting {
            -DRAIN
        } else if speed < 0.4 {
            FILL
        } else if crouched {
            FILL * if soles { 0.8 } else { 0.35 }
        } else if soles {
            FILL * 0.25
        } else {
            0.0
        };
        if rate > 0.0 {
            rate *= gen;
            if crouched {
                rate *= 1.5;
            }
            if self.rogue_dark() {
                rate *= 1.5;
            }
            if self.in_smoke(self.body.eye()) {
                rate *= 3.0;
            }
            if attacking {
                rate = 0.0;
            }
        }
        let before = self.rogue.stealth;
        self.rogue.stealth = (self.rogue.stealth + rate * dt).clamp(0.0, max);
        if self.rogue.staged {
            // The test scene keeps the meter full.
            self.rogue.stealth = max;
        }
        if self.rogue_full() && before < max - 0.01 && !self.rogue.told {
            self.rogue.told = true;
            self.sfx(Sfx::Fx(Fx::Hush));
        }
        // Seen from closer as the meter fills: 16 blocks, down to about 5 (4 crouched).
        let k = self.rogue.stealth / max;
        self.mobs.sight = (1.0 - 0.66 * k) * if crouched && k > 0.5 { 0.8 } else { 1.0 };
    }

    /// Dark around the player: little light from lamps, and night or shade from the sky.
    fn rogue_dark(&self) -> bool {
        let l = self.world.light(self.body.eye().floor().as_ivec3());
        let a = self.time_of_day / 24000.0 * std::f32::consts::TAU;
        let day = self.tr_daylight((a.sin() * 2.2 + 0.45).clamp(0.0, 1.0));
        let sky = (l >> 4) as f32 * day;
        let block = (l & 15) as f32;
        sky.max(block) <= 6.0
    }

    /// The meter under the crosshair, the oil on the blades and a dark edge when a strike is
    /// ready.
    pub(crate) fn draw_assassin_hud(&mut self, w: f32, h: f32) {
        let s = self.ui.scale;
        let max = self.t5.rpg.stats.stealth_max;
        let (cx, cy) = (w / 2.0, h / 2.0);
        if max > 0.0 && self.survival {
            let k = (self.rogue.stealth / max).clamp(0.0, 1.0);
            let full = self.rogue_full();
            let bw = 22.0 * s;
            let y = cy + 13.0 * s;
            self.ui.rect(cx - bw / 2.0, y, bw, 2.0 * s, [20, 16, 28, 170]);
            let col = if full { [196, 150, 255, 240] } else { [128, 104, 178, 220] };
            self.ui.rect(cx - bw / 2.0, y, bw * k, 2.0 * s, col);
            if full {
                // The edges of the view darken: you are in the shadows.
                let pulse = 0.75 + 0.25 * (self.now * 3.0).sin();
                for i in 0..6 {
                    let t = (6 - i) as f32 * 4.0 * s;
                    let a = (14.0 * pulse) as u8;
                    self.ui.rect(0.0, 0.0, t * 3.0, h, [10, 6, 18, a]);
                    self.ui.rect(w - t * 3.0, 0.0, t * 3.0, h, [10, 6, 18, a]);
                    self.ui.rect(0.0, 0.0, w, t, [10, 6, 18, a]);
                    self.ui.rect(0.0, h - t, w, t, [10, 6, 18, a]);
                }
            }
        }
        if let Some((k, t)) = self.rogue.oil {
            let (_, _, _, name, c) = OILS[k];
            let text = format!("{name} {}:{:02}", (t / 60.0) as u32, (t % 60.0) as u32);
            let tw = self.ui.text_width(&text, s);
            self.ui.text(cx - tw / 2.0, cy + 18.0 * s, &text, [c[0], c[1], c[2], 220]);
        }
        if self.rogue.flash > 0.0 {
            let a = (self.rogue.flash * 40.0) as u8;
            self.ui.rect(0.0, 0.0, w, h, [60, 10, 30, a]);
        }
    }

    /// Smoke bombs in flight, drawn as their item.
    pub(crate) fn assassin_instances(&self, cam_block: IVec3) -> Vec<GpuItem> {
        let mut out = Vec::new();
        if self.rogue.bombs.is_empty() {
            return out;
        }
        let Some(id) = self.items.id("smoke_bomb") else { return out };
        let voxel::item::Look::Sprite(l) = self.items.get(id).look else { return out };
        let layer = (self.item_base + l) as f32;
        let c = cam_block.as_vec3();
        for b in &self.rogue.bombs {
            let l = self.world.light(b.pos.floor().as_ivec3());
            let (sky, blk) = ((l >> 4) as f32 / 15.0, (l & 15) as f32 / 15.0);
            let p = b.pos - c;
            out.push(GpuItem { pos: [p.x, p.y, p.z, 0.45], info: [b.age * 7.0, 2.0, sky, blk], layers: [layer, layer, layer, 0.0], rot: [0.0, b.age * 9.0, 0.0, 0.0], ..bytemuck::Zeroable::zeroed() });
        }
        out
    }
}

impl Game {
    /// Test scene (MC_ASSASSIN=eclipse|nightshade|shroud|ninja, with --survival): that set on,
    /// a full stealth meter, poison oil on the blades, a smoke cloud to the left and a zombie
    /// with its back to the player in front. MC_HOLD picks the item in hand.
    fn assassin_test_scene(&mut self) {
        if self.rogue.staged {
            return;
        }
        let Ok(set) = std::env::var("MC_ASSASSIN") else { return };
        self.rogue.staged = true;
        let set = if set == "1" { "eclipse".to_string() } else { set };
        for (i, k) in ["helmet", "chestplate", "leggings", "boots"].iter().enumerate() {
            self.surv.armor[i] = self.items.id(&format!("{set}_{k}")).map(|id| voxel::item::ItemStack::new(id, 1));
        }
        self.rogue.stealth = 1000.0;
        self.rogue.told = true;
        self.rogue.oil = Some((0, 154.0));
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let side = flat.cross(Vec3::Y);
        let ground = |g: &Game, p: Vec3| g.world.surface_y(p.x.floor() as i32, p.z.floor() as i32).map(|y| y as f32 + 1.0).unwrap_or(p.y);
        let z = self.body.pos + flat * 4.0 + side * 1.4;
        self.mobs.spawn("zombie", Vec3::new(z.x, ground(self, z), z.z));
        if let Some(m) = self.mobs.list.last_mut() {
            // Facing away from the player.
            m.yaw = flat.x.atan2(flat.z);
            m.head_yaw = m.yaw;
        }
        let c = self.body.pos + flat * 9.0 - side * 3.0;
        self.rogue.bombs.push(Bomb { pos: Vec3::new(c.x, ground(self, c) + 0.3, c.z), vel: Vec3::ZERO, age: 2.0 });
    }
}

/// The class's numbers, for the analysis (plan/assassino-analise.md) and the tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oils_and_daggers() {
        assert!(is_dagger("iron_dagger") && !is_dagger("iron_sword"));
        assert_eq!(OILS.len(), 4);
        // Every oil's flask has a recipe and an item.
        let (_, items, recipes) = crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).unwrap();
        for o in OILS {
            let id = items.id(o.0).unwrap_or_else(|| panic!("no item {}", o.0));
            assert!(recipes.shapeless.iter().any(|r| r.result.item == id) || recipes.shaped.iter().any(|r| r.result.item == id), "no recipe for {}", o.0);
        }
        for n in ["iron_dagger", "umbral_dagger", "magmarite_dagger", "mythril_dagger", "verdite_dagger", "lumite_dagger", "smoke_bomb", "shadow_cloak", "assassin_emblem", "silent_soles", "throwing_glove", "moonless_mark"] {
            let id = items.id(n).unwrap_or_else(|| panic!("no item {n}"));
            assert!(recipes.shapeless.iter().any(|r| r.result.item == id) || recipes.shaped.iter().any(|r| r.result.item == id), "no recipe for {n}");
        }
        // The accessories do something.
        for n in ["shadow_cloak", "assassin_emblem", "silent_soles", "throwing_glove", "moonless_mark"] {
            assert!(crate::rpg::accessory(n).is_some(), "{n} is not an accessory");
        }
    }

    #[test]
    fn stealth_makes_monsters_see_less() {
        // A full meter: about a third of the usual sight, less crouched.
        let full = 1.0 - 0.66;
        assert!(16.0 * full < 6.0 && 16.0 * full * 0.8 < 5.0);
        // A strike from behind with a dagger is the class's big hit: 2.5 x 2 = 5 times.
        assert!((1.0 + BACKSTAB) * STRIKE_MULT >= 5.0);
        assert!(THROWN_BACKSTAB < BACKSTAB);
    }
}
