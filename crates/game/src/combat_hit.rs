//! The shared melee hit (thread "Analisar e melhorar o combate", plan/combate-analise.md).
//!
//! Every player melee blow on a mob lands through `strike_mob`, whichever path found the target
//! (the classic single hit in main.rs or the weapon arcs and combos of mods/combat.rs):
//! - critical hits: a jump-fall blow with a ready weapon, or the critical chance of gear and the
//!   weapon's modifier, both only with a ready swing (`melee_crit`);
//! - the melee guard: after a blow a mob takes only what a new blow adds over it for a moment
//!   (Mob::melee_guard), so a flurry of weak clicks never beats a timed swing;
//! - knockback that doesn't stack past its strength and that bosses and brutes resist
//!   (Mob::knock, Mob::kb_resist).
//!
//! It also keeps the floating damage numbers: every frame the health each mob lost is shown
//! rising over it (any source: blows, arrows, spells, minions, poison), bigger and orange for a
//! critical hit. Option "Números de dano".

use std::collections::HashMap;

use glam::Vec3;

use crate::Game;

/// How long a number floats, in seconds.
const POP_LIFE: f32 = 0.9;
/// Numbers further than this are not drawn.
const POP_FAR: f32 = 28.0;
const MAX_POPS: usize = 48;

struct Pop {
    net_id: u32,
    pos: Vec3,
    amount: f32,
    crit: bool,
    age: f32,
    /// Sideways drift, so numbers from a flurry don't sit on each other.
    drift: f32,
}

#[derive(Default)]
pub struct CombatFx {
    /// Health of each living mob last frame, by net id.
    health: HashMap<u32, f32>,
    /// Mobs hit critically this frame (net ids), for the colour of their number.
    crits: Vec<u32>,
    pops: Vec<Pop>,
    seed: u32,
    /// Game time the shield went up (None while it is down), for the parry.
    pub(crate) raised_at: Option<f32>,
    /// Game time until which the shield can't be raised (split by an axe blow).
    pub(crate) shield_off_until: f32,
    /// Where recent hits on the player came from, and when (the red marks round the
    /// crosshair).
    hurt_from: Vec<(Vec3, f32)>,
    /// Test pictures: the clock the marks are drawn against, held still.
    mark_clock: Option<f32>,
}

/// Seconds a mark of where a hit came from stays on screen.
const HURT_MARK: f32 = 1.4;
/// Monsters whose axe blow knocks a raised shield out of use, like the vindicator's.
const SHIELD_BREAKERS: &[&str] = &["brute", "piglin_brute"];
/// Seconds an axe blow keeps the shield down.
pub(crate) const SHIELD_SPLIT: f32 = 5.0;

/// A shield raised this recently before a blow parries it.
pub(crate) const PARRY_WINDOW: f32 = 0.3;

impl CombatFx {
    /// Mark the hit landing on mob `net_id` this frame as critical.
    pub(crate) fn crit(&mut self, net_id: u32) {
        self.crits.push(net_id);
    }
}

/// The guard window after a blow: a bit under the weapon's own swing time, so a timed swing
/// always lands in full, and never longer than the classic half second.
pub(crate) fn guard_window(cooldown: f32) -> f32 {
    (cooldown * 0.8).clamp(0.15, 0.45)
}

impl Game {
    /// Is this blow critical? Falling onto the target with a ready weapon always is; otherwise a
    /// ready swing rolls the melee critical chance of gear (rpg.rs) and the weapon's modifier.
    pub(crate) fn melee_crit(&mut self, charge: f32) -> bool {
        if charge <= 0.9 {
            return false;
        }
        let falling = self.body.vel.y < -0.5 && !self.body.on_ground && !self.body.in_water;
        falling || self.surv.rng.f32() < self.held_crit() + self.t5.rpg.stats.crit[crate::rpg::MELEE]
    }

    /// Land a player's melee blow of `dmg` on mob `i`: the guard, the hurt, a push of `push`
    /// (the plain hit pushes 6), the critical mark. Returns the damage that landed (0 when the
    /// guard took it all). `primary` is the mob aimed at (its name and health show).
    pub(crate) fn strike_mob(&mut self, i: usize, dmg: f32, push: f32, crit: bool, primary: bool) -> f32 {
        let window = guard_window(if self.survival { self.attack_cooldown() } else { 0.45 });
        let eye = self.body.eye();
        let m = &mut self.mobs.list[i];
        if !m.alive() {
            return 0.0;
        }
        let landed = m.melee_guard(dmg, window);
        if landed <= 0.0 {
            return 0.0;
        }
        m.hurt(landed, eye);
        if push > 6.0 {
            let dir = m.body.pos - eye;
            m.knock(dir, push);
        }
        let net_id = m.net_id;
        if primary {
            let def = &self.mobs.reg.defs[m.def];
            self.last_hit = Some((def.display_name.clone(), m.health.max(0.0), m.max_health, self.now + 3.0));
        }
        if crit {
            self.combat_fx.crit(net_id);
        }
        landed
    }

    /// A blow from `from` meets a shield raised less than `PARRY_WINDOW` ago: the monsters
    /// that struck it from close by are thrown back and lose their next swing; the shield is
    /// not worn. True when it was a parry.
    pub(crate) fn parry(&mut self, from: Vec3) -> bool {
        let Some(t) = self.combat_fx.raised_at else { return false };
        if self.now - t > PARRY_WINDOW {
            return false;
        }
        let me = self.body.pos;
        let mut any = false;
        for m in self.mobs.list.iter_mut() {
            if m.alive() && !m.ghost && (m.body.center() - from).length() < 2.0 && (m.body.pos - me).length() < 5.0 {
                m.knock(m.body.pos - me, 9.0);
                m.stagger(1.5);
                any = true;
            }
        }
        // A parried arrow just drops; either way the clang and sparks.
        let glow = self.glow_layer();
        let at = self.body.eye() + (from - self.body.eye()).normalize_or_zero() * 0.7 - Vec3::Y * 0.2;
        for k in 0..10 {
            let a = k as f32 * 0.628;
            self.particles.glow(at, Vec3::new(a.cos() * 2.5, 1.5 + (k % 3) as f32, a.sin() * 2.5), glow, [1.6, 1.3, 0.6], 0.05, 0.3);
        }
        self.sfx(crate::audio::Sfx::Hit(crate::audio::Material::Metal));
        self.pads.rumble(0.5, 0.8, 0.12);
        if any {
            self.toast("Aparado!", 0.8);
        }
        true
    }

    /// A blow from `from` on a raised shield, by an axe-wielding monster: the shield is
    /// knocked aside for `SHIELD_SPLIT` seconds and the blow lands. True when that happened.
    pub(crate) fn shield_split(&mut self, from: Vec3) -> bool {
        let breaker = self.mobs.list.iter().any(|m| m.alive() && (m.body.center() - from).length() < 1.0 && SHIELD_BREAKERS.contains(&self.mobs.reg.defs[m.def].name.as_str()));
        if !breaker {
            return false;
        }
        self.combat_fx.shield_off_until = self.now + SHIELD_SPLIT;
        self.combat_fx.raised_at = None;
        self.extras.blocking = false;
        self.sfx(crate::audio::Sfx::Hit(crate::audio::Material::Wood));
        self.toast("O machado tirou seu escudo do lugar!", 2.0);
        true
    }

    /// Remember where a hit on the player came from (drawn by `draw_hurt_marks`).
    pub(crate) fn note_hurt_from(&mut self, from: Vec3) {
        let now = self.now;
        self.combat_fx.hurt_from.retain(|h| now - h.1 < HURT_MARK);
        self.combat_fx.hurt_from.push((from, now));
        if self.combat_fx.hurt_from.len() > 8 {
            self.combat_fx.hurt_from.remove(0);
        }
    }

    /// Red marks round the crosshair pointing where recent hits came from (behind you: at the
    /// bottom), so a blow from out of sight can be answered.
    pub(crate) fn draw_hurt_marks(&mut self, w: f32, h: f32) {
        if self.combat_fx.hurt_from.is_empty() || self.screen != crate::Screen::Playing {
            return;
        }
        let s = self.ui.scale;
        let (cx, cy) = (w / 2.0, h / 2.0);
        let r = (h * 0.16).max(40.0 * s);
        let fwd = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        let me = self.body.pos;
        let now = self.combat_fx.mark_clock.unwrap_or(self.now);
        let marks: Vec<(Vec3, f32)> = self.combat_fx.hurt_from.iter().copied().filter(|h| now - h.1 < HURT_MARK).collect();
        for (from, t) in marks {
            let to = from - me;
            if Vec3::new(to.x, 0.0, to.z).length() < 0.05 {
                continue;
            }
            let ang = to.dot(right).atan2(to.dot(fwd));
            let fade = 1.0 - (now - t) / HURT_MARK;
            let a = (fade * 220.0) as u8;
            // A short arc of blocks, thickest in the middle.
            for k in -4i32..=4 {
                let b = ang + k as f32 * 0.05;
                let (x, y) = (cx + b.sin() * r, cy - b.cos() * r);
                let size = (if k.abs() <= 1 { 4.0 } else if k.abs() <= 3 { 3.0 } else { 2.0 }) * s;
                self.ui.rect(x - size / 2.0, y - size / 2.0, size, size, [220, 30, 20, a]);
            }
        }
    }

    /// Melee bonus that doesn't depend on the target (sharpness, strength and weakness): for
    /// blows on other players and on the mobs a sweep catches.
    pub(crate) fn plain_melee_bonus(&self, dmg: f32) -> f32 {
        let sharp = crate::enchanting::level(&self.held_stack(), crate::enchanting::SHARPNESS);
        let d = if sharp > 0 { dmg + 0.5 * sharp as f32 + 0.5 } else { dmg };
        self.melee_effects(d)
    }

    /// Each frame: turn the health every mob lost into a floating number.
    pub(crate) fn update_combat_fx(&mut self, dt: f32) {
        let fx = &mut self.combat_fx;
        for p in fx.pops.iter_mut() {
            p.age += dt;
        }
        fx.pops.retain(|p| p.age < POP_LIFE);
        let mut seen: HashMap<u32, f32> = HashMap::with_capacity(self.mobs.list.len());
        for m in &self.mobs.list {
            if m.ghost {
                continue;
            }
            let before = fx.health.get(&m.net_id).copied();
            let now = m.health.max(0.0);
            if m.alive() || before.is_some_and(|b| b > 0.0) {
                seen.insert(m.net_id, now);
            }
            let Some(before) = before else { continue };
            let lost = before - now;
            if lost < 0.05 {
                continue;
            }
            let crit = fx.crits.contains(&m.net_id);
            let def = &self.mobs.reg.defs[m.def];
            let top = m.body.pos + Vec3::Y * (def.height + 0.95);
            // A number still rising from the same mob a moment ago adds up (a sweep, a volley).
            if let Some(p) = fx.pops.iter_mut().rev().find(|p| p.net_id == m.net_id && p.age < 0.12) {
                p.amount += lost;
                p.crit |= crit;
                continue;
            }
            fx.seed = fx.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let drift = ((fx.seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 0.8;
            fx.pops.push(Pop { net_id: m.net_id, pos: top, amount: lost, crit, age: 0.0, drift });
        }
        fx.health = seen;
        fx.crits.clear();
        if fx.pops.len() > MAX_POPS {
            let extra = fx.pops.len() - MAX_POPS;
            fx.pops.drain(..extra);
        }
    }

    /// Draw the floating damage numbers.
    pub(crate) fn draw_damage_numbers(&mut self, w: f32, h: f32) {
        if !self.settings.damage_numbers || self.combat_fx.pops.is_empty() || self.screen != crate::Screen::Playing {
            return;
        }
        let Some((vp, cam_block)) = self.ui_view_proj else { return };
        let eye = self.body.eye();
        let s = self.ui.scale;
        for k in 0..self.combat_fx.pops.len() {
            let (pos, amount, crit, age, drift) = {
                let p = &self.combat_fx.pops[k];
                (p.pos, p.amount, p.crit, p.age, p.drift)
            };
            let t = age / POP_LIFE;
            // Up quickly, then slowing; a little sideways.
            let rise = 0.9 * (1.0 - (1.0 - t) * (1.0 - t));
            let at = pos + Vec3::new(drift * t, rise, 0.0);
            let dist = (at - eye).length();
            if dist > POP_FAR || !self.in_sight(&[at]) {
                continue;
            }
            let c = vp * (at - cam_block.as_vec3()).extend(1.0);
            if c.w < 0.1 {
                continue;
            }
            let (nx, ny) = (c.x / c.w, c.y / c.w);
            if nx.abs() > 1.05 || ny.abs() > 1.05 {
                continue;
            }
            let (x, y) = ((nx * 0.5 + 0.5) * w, (0.5 - ny * 0.5) * h);
            // Whole pixel steps so the digits stay crisp; a critical pops in bigger.
            let pop = if t < 0.12 { 1.0 + (0.12 - t) / 0.12 * 0.5 } else { 1.0 };
            let base = s * (5.0 / dist.max(1.0)).clamp(0.6, 1.2) * if crit { 1.5 } else { 1.0 } * pop;
            let sc = base.round().max(1.0);
            let label = if amount >= 10.0 || (amount - amount.round()).abs() < 0.05 { format!("{:.0}", amount.round().max(1.0)) } else { format!("{amount:.1}") };
            let label = if crit { format!("{label}!") } else { label };
            let fade = if t > 0.6 { 1.0 - (t - 0.6) / 0.4 } else { 1.0 };
            let a = (fade * 255.0).round() as u8;
            // Terraria's idea: warm orange for a hit, deeper red-orange for a critical one.
            let col = if crit { [255, 90, 30, a] } else { [255, 200, 70, a] };
            let tw = self.ui.text_width(&label, sc);
            let (tx, ty) = (x - tw / 2.0, y - 4.0 * sc);
            let shadow = [20, 10, 5, (fade * 200.0) as u8];
            self.ui.text_scaled(tx + sc, ty + sc, &label, shadow, sc);
            self.ui.text_scaled(tx, ty, &label, col, sc);
        }
    }
}

impl Game {
    /// Test hook MC_COMBAT=numbers: three monsters in front, held still, hit a few times each
    /// (some critically), so the floating numbers show at different heights.
    pub(crate) fn test_combat(&mut self, scene: &str) {
        if scene == "hurt" {
            // Marks of hits from the right, from behind and (older, faded) from the front left.
            let (f, r) = (Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos()), Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()));
            let me = self.body.pos;
            self.note_hurt_from(me - f * 2.0 - r * 0.5);
            self.note_hurt_from(me + r * 2.0);
            self.combat_fx.hurt_from.push((me + f * 2.0 - r * 2.0, self.now - 0.9));
            self.combat_fx.mark_clock = Some(self.now + 0.1);
            return;
        }
        if scene != "numbers" {
            log::warn!("unknown MC_COMBAT scene {scene}");
            return;
        }
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let side = Vec3::new(-flat.z, 0.0, flat.x);
        if let Some(id) = self.items.id("diamond_sword") {
            let sel = self.selected;
            self.surv.inv.slots[sel] = Some(voxel::item::ItemStack::new(id, 1));
        }
        let mut idx = Vec::new();
        for (k, name) in ["zombie", "skeleton", "spider"].iter().enumerate() {
            let at = self.body.pos + flat * 4.5 + side * (k as f32 - 1.0) * 1.8;
            let y = self.world.surface_y(at.x.floor() as i32, at.z.floor() as i32).map(|y| y as f32 + 1.0).unwrap_or(at.y);
            if self.mobs.spawn(name, Vec3::new(at.x, y, at.z)) {
                let i = self.mobs.list.len() - 1;
                let m = &mut self.mobs.list[i];
                m.still = true;
                m.yaw = (-flat.x).atan2(-flat.z);
                m.health = m.max_health * 3.0;
                m.max_health = m.health;
                idx.push(i);
            }
        }
        self.update_combat_fx(0.0);
        let hits: [(usize, f32, bool, f32); 6] = [(0, 7.0, false, 0.55), (0, 10.5, true, 0.05), (1, 4.5, false, 0.3), (1, 7.0, false, 0.02), (2, 13.5, true, 0.2), (2, 2.5, false, 0.6)];
        for (k, dmg, crit, age) in hits {
            let Some(&i) = idx.get(k) else { continue };
            self.mobs.list[i].health -= dmg;
            self.mobs.list[i].hurt_age = 0.0;
            if crit {
                self.combat_fx.crit(self.mobs.list[i].net_id);
            }
            self.update_combat_fx(0.0);
            if let Some(p) = self.combat_fx.pops.last_mut() {
                p.age = age;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel::mob::{Mob, MobRegistry};

    fn zombie() -> Mob {
        let reg = MobRegistry::from_ron(include_str!("../../../packs/vanilla/mobs.ron")).unwrap();
        let z = reg.id("zombie").unwrap();
        Mob::new(z, &reg.defs[z], Vec3::new(0.5, 10.0, 0.5), 1)
    }

    /// Clicking as fast as possible must not out-damage swinging when the weapon is ready
    /// (the classic charge curve, 0.2 + 0.8 k², and the guard).
    #[test]
    fn spam_clicks_never_beat_timed_swings() {
        for cooldown in [0.25_f32, 0.625, 1.0, 1.35] {
            let window = guard_window(cooldown);
            let factor = |k: f32| 0.2 + 0.8 * k.min(1.0) * k.min(1.0);
            let secs = 6.0;
            // Timed: one full blow per cooldown.
            let timed = (secs / cooldown).floor() * 10.0;
            for click in [0.1_f32, 0.15, 0.2, 0.3] {
                if click >= cooldown {
                    continue;
                }
                let mut m = zombie();
                m.health = 1e6;
                let (mut t, mut dealt) = (0.0, 0.0);
                while t < secs {
                    m.guard = (m.guard - click).max(0.0);
                    dealt += m.melee_guard(10.0 * factor(click / cooldown), window);
                    t += click;
                }
                assert!(dealt < timed, "cooldown {cooldown}, click every {click}: spam {dealt} vs timed {timed}");
            }
        }
    }

    /// A ready swing always lands in full, however quick the weapon.
    #[test]
    fn timed_swings_are_never_guarded() {
        for cooldown in [0.2_f32, 0.25, 0.5, 1.0] {
            let mut m = zombie();
            m.health = 1e6;
            let window = guard_window(cooldown);
            assert!(window < cooldown);
            for _ in 0..5 {
                assert_eq!(m.melee_guard(7.0, window), 7.0);
                m.guard = (m.guard - cooldown).max(0.0);
            }
        }
    }

    /// Knockback tops up instead of adding: ten quick blows don't fling a mob ten times as far,
    /// and bosses barely move.
    #[test]
    fn knockback_does_not_stack() {
        let mut m = zombie();
        m.health = 1e6;
        for _ in 0..10 {
            m.hurt(1.0, Vec3::new(-1.0, 10.0, 0.5));
        }
        let flat = Vec3::new(m.body.vel.x, 0.0, m.body.vel.z).length();
        assert!(flat <= 6.01, "{flat}");
        let mut boss = zombie();
        boss.kb_resist = 0.85;
        boss.hurt(1.0, Vec3::new(-1.0, 10.0, 0.5));
        assert!(boss.body.vel.x < 1.0);
    }
}
