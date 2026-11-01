//! Better Combat idea: weapons swing through an arc and hit everything inside it, and each
//! weapon cycles through a combo of different attacks (a slash, a backhand, a heavy finisher).
//! Two-handed weapons (greatswords, spears, halberds) hit harder and reach further but swing
//! slower and leave no hand free for the off-hand item.
//!
//! Only melee weapons take this path; picks, shovels and the bare hand keep the classic hit.
//! The swing still goes through grass and flowers (the caller measures the block in the way).

use glam::Vec3;
use voxel::block::ToolKind;

use crate::audio::Sfx;
use crate::Game;

/// How one attack of a combo moves and hits.
#[derive(Clone, Copy, Debug)]
struct Attack {
    /// Animation (fight_anim::style): chop, slash, backhand, thrust, uppercut, heavy.
    style: u8,
    /// Damage multiplier on the weapon's damage.
    damage: f32,
    /// Half-angle of the arc in degrees (a thrust is narrow, a slash wide).
    arc: f32,
    /// Knockback push.
    push: f32,
}

/// A weapon class: reach, swing time and its combo.
#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    reach: f32,
    /// Seconds between full-strength swings.
    pub cooldown: f32,
    /// Share of the damage dealt to the other mobs in the arc (the primary takes it all).
    sweep: f32,
    pub two_handed: bool,
    combo: [Attack; 3],
}

use crate::fight_anim::style::{BACKHAND, CHOP, HEAVY, SLASH, THRUST, UPPERCUT};

const fn at(style: u8, damage: f32, arc: f32, push: f32) -> Attack {
    Attack { style, damage, arc, push }
}

const SWORD: Weapon = Weapon { reach: 3.2, cooldown: 0.625, sweep: 0.5, two_handed: false, combo: [at(SLASH, 1.0, 50.0, 1.0), at(BACKHAND, 1.0, 50.0, 1.0), at(THRUST, 1.3, 14.0, 2.0)] };
const AXE: Weapon = Weapon { reach: 3.0, cooldown: 1.0, sweep: 0.35, two_handed: false, combo: [at(CHOP, 1.0, 28.0, 1.5), at(BACKHAND, 1.0, 28.0, 1.5), at(UPPERCUT, 1.35, 30.0, 3.0)] };
const GREATSWORD: Weapon = Weapon { reach: 3.8, cooldown: 1.2, sweep: 0.8, two_handed: true, combo: [at(SLASH, 1.0, 80.0, 2.0), at(BACKHAND, 1.0, 80.0, 2.0), at(HEAVY, 1.6, 40.0, 4.0)] };
/// A spear outreaches every other melee weapon (whips reach 5, rpg.rs).
pub const SPEAR_REACH: f32 = 5.6;
const SPEAR: Weapon = Weapon { reach: SPEAR_REACH, cooldown: 0.85, sweep: 0.8, two_handed: true, combo: [at(THRUST, 1.0, 12.0, 1.5), at(THRUST, 1.0, 12.0, 1.5), at(THRUST, 1.45, 16.0, 3.5)] };
/// The assassin's dagger (assassin.rs): short, quick, a stab, a cut and a deep stab; it barely
/// sweeps, the point is one target, ideally from behind.
pub const DAGGER_REACH: f32 = 2.6;
const DAGGER: Weapon = Weapon { reach: DAGGER_REACH, cooldown: 0.34, sweep: 0.2, two_handed: false, combo: [at(THRUST, 1.0, 16.0, 0.6), at(BACKHAND, 0.9, 38.0, 0.6), at(THRUST, 1.3, 14.0, 1.0)] };
const HALBERD: Weapon = Weapon { reach: 4.3, cooldown: 1.35, sweep: 0.7, two_handed: true, combo: [at(HEAVY, 1.0, 45.0, 2.5), at(BACKHAND, 1.1, 65.0, 2.5), at(THRUST, 1.5, 15.0, 4.5)] };

// Sword families (HP 2026-11-01: every sword felt the same). Each swings at its own pace and in
// its own way, but over time deals what a plain sword of the same damage would (the combo's
// average hit over the swing time equals SWORD's), so the balance sheet does not move.
/// Rapiers and needles: quick, narrow thrusts that reach a little further.
const RAPIER: Weapon = Weapon { reach: 3.6, cooldown: 0.45, sweep: 0.2, two_handed: false, combo: [at(2, 0.72, 12.0, 0.6), at(2, 0.72, 12.0, 0.6), at(2, 0.936, 16.0, 1.4)] };
/// Katanas and sabres: fast cuts, a backhand and a rising finisher.
const SABRE: Weapon = Weapon { reach: 3.3, cooldown: 0.5, sweep: 0.45, two_handed: false, combo: [at(1, 0.8, 55.0, 0.8), at(1, 0.8, 55.0, 0.8), at(3, 1.04, 45.0, 1.6)] };
/// Cleavers, falchions and other heavy blades: slow chops that knock back hard.
const CLEAVER: Weapon = Weapon { reach: 3.2, cooldown: 0.8, sweep: 0.55, two_handed: false, combo: [at(0, 1.28, 32.0, 1.8), at(0, 1.28, 32.0, 1.8), at(3, 1.664, 36.0, 3.2)] };
/// Whips lash one line far ahead (the summoner's mark comes from the hit, rpg.rs).
const WHIP: Weapon = Weapon { reach: 5.0, cooldown: 0.625, sweep: 0.3, two_handed: false, combo: [at(1, 1.0, 16.0, 0.8), at(1, 1.0, 16.0, 0.8), at(1, 1.3, 20.0, 1.2)] };
/// The mace: slow, heavy one-handed blows at one target (the falling smash is weapons.rs).
const MACE: Weapon = Weapon { reach: 3.0, cooldown: 1.0, sweep: 0.2, two_handed: false, combo: [at(0, 1.6, 24.0, 2.5), at(0, 1.6, 24.0, 2.5), at(3, 2.08, 28.0, 4.0)] };
/// Clubs: like the mace but sweeping wider.
const CLUB: Weapon = Weapon { reach: 3.2, cooldown: 1.1, sweep: 0.5, two_handed: false, combo: [at(0, 1.76, 40.0, 3.0), at(1, 1.76, 50.0, 3.0), at(0, 2.288, 40.0, 4.5)] };
/// One-handed thrusting polearms (trident, lance).
const TRIDENT: Weapon = Weapon { reach: 4.2, cooldown: 0.75, sweep: 0.3, two_handed: false, combo: [at(2, 1.2, 14.0, 1.5), at(2, 1.2, 14.0, 1.5), at(2, 1.56, 18.0, 2.5)] };
/// Scythes: wide reaping arcs.
const SCYTHE: Weapon = Weapon { reach: 3.6, cooldown: 0.75, sweep: 0.65, two_handed: false, combo: [at(1, 1.2, 85.0, 1.2), at(1, 1.2, 85.0, 1.2), at(3, 1.56, 70.0, 2.0)] };

/// The family of a sword, from its design (tools/tex64/arms.py and blades.py draw them).
fn sword_family(name: &str) -> Weapon {
    match name {
        "silver_sword" | "prism_saber" | "mythril_sword" => RAPIER,
        "platinum_sword" | "blue_katana" | "cutlass" | "tungsten_sword" | "star_saber" | "nebula_blade" | "bronze_sword" | "light_blade" | "falcon_blade" => SABRE,
        "lead_sword" | "bone_sword" | "sanguinite_sword" | "horseman_blade" | "adamantite_sword" | "cobalt_sword" | "magmarite_sword" | "netherite_sword" => CLEAVER,
        _ => SWORD,
    }
}

impl Weapon {
    /// How fast the swing animation plays next to a plain sword's (heavy weapons swing slower).
    pub fn swing_rate(&self) -> f32 {
        (SWORD.cooldown / self.cooldown).clamp(0.6, 1.35)
    }

    /// How heavy the weapon feels, 0 (needle) .. 1 (greatsword, club): its sound and trail.
    pub fn weight(&self) -> f32 {
        ((self.cooldown - 0.45) / 0.8).clamp(0.0, 1.0)
    }

    pub fn reach(&self) -> f32 {
        self.reach
    }

    /// Damage per second of steady swinging at one monster: the combo's average hit over the
    /// swing time (balance.rs).
    #[cfg(test)]
    pub fn swing_dps(&self, damage: f32) -> f32 {
        damage * self.combo.iter().map(|a| a.damage).sum::<f32>() / 3.0 / self.cooldown
    }
}

/// After this long without a swing the combo starts over.
const COMBO_WINDOW: f32 = 0.9;

#[derive(Default)]
pub struct Combat {
    /// Which attack of the combo comes next (0..3).
    step: u8,
    /// Game time of the last swing.
    last: f32,
    /// The off-hand item was put away for a two-handed weapon (one message per weapon).
    warned: bool,
    /// The trail of the swing in progress (blade_fx.rs).
    pub(crate) trail: crate::blade_fx::Trail,
}

/// The weapon class of an item, from its name and tool kind.
pub fn weapon_of(name: &str, kind: Option<ToolKind>) -> Option<Weapon> {
    if name.ends_with("_bow") || name == "bow" || name == "crossbow" {
        // Bows are kind: Sword in the item files; a click with one is a plain punch.
        None
    } else if crate::rpg::is_whip(name) {
        Some(WHIP)
    } else if name == "mace" {
        Some(MACE)
    } else if name.ends_with("_club") {
        Some(CLUB)
    } else if name == "trident" || name.ends_with("_lance") {
        Some(TRIDENT)
    } else if name.ends_with("_scythe") {
        Some(SCYTHE)
    } else if name.ends_with("_greatsword") {
        Some(GREATSWORD)
    } else if name.ends_with("_spear") {
        Some(SPEAR)
    } else if name.ends_with("_halberd") {
        Some(HALBERD)
    } else if name.ends_with("_dagger") {
        Some(DAGGER)
    } else {
        match kind {
            Some(ToolKind::Sword) => Some(sword_family(name)),
            Some(ToolKind::Axe) => Some(AXE),
            _ => None,
        }
    }
}

impl Game {
    pub(crate) fn held_weapon(&self) -> Option<Weapon> {
        let st = self.held_stack()?;
        let it = self.items.get(st.item);
        let mut w = weapon_of(&it.name, it.tool.as_ref().map(|t| t.kind))?;
        // Bigger weapons (gauntlets, rpg.rs `M::Size`) reach further.
        w.reach *= self.t5.rpg.stats.size.clamp(1.0, 1.6);
        Some(w)
    }

    /// The swing time of the weapon in hand, when it is a combat weapon.
    pub(crate) fn weapon_cooldown(&self) -> Option<f32> {
        self.held_weapon().map(|w| w.cooldown)
    }

    /// A melee swing with a combat weapon: hits every mob in its arc, nearer than the first
    /// solid block (`block_dist`). False when the held item is not a weapon or nothing is in
    /// reach, so the classic hit runs instead.
    pub(crate) fn combat_swing(&mut self, block_dist: f32) -> bool {
        let Some(w) = self.held_weapon() else { return false };
        let step = if self.now - self.mods.combat.last <= w.cooldown + COMBO_WINDOW { self.mods.combat.step } else { 0 };
        let a = w.combo[step as usize % 3];
        let eye = self.body.eye();
        let fwd = self.forward();
        let cos_arc = a.arc.to_radians().cos();
        // Every living mob whose body is inside the arc, nearest first.
        let mut hits: Vec<(usize, f32)> = Vec::new();
        for (i, m) in self.mobs.list.iter().enumerate() {
            if !m.alive() {
                continue;
            }
            let def = &self.mobs.reg.defs[m.def];
            let center = m.body.pos + Vec3::Y * (def.height * 0.5).min(1.2);
            let to = center - eye;
            let d = to.length();
            let r = def.width * 0.5 * def.scale;
            if d - r > w.reach || d - r > block_dist {
                continue;
            }
            // Measure the angle to the nearest point of the body, so big mobs are easy to hit.
            let along = to.dot(fwd);
            let side = (to - fwd * along).length();
            let angle_ok = along > 0.0 && (along / d.max(0.01) >= cos_arc || side <= r + 0.35);
            if angle_ok {
                hits.push((i, d));
            }
        }
        if hits.is_empty() {
            return false;
        }
        hits.sort_by(|a, b| a.1.total_cmp(&b.1));
        // Town folk are only hit when aimed at directly.
        let primary = hits[0].0;
        hits.retain(|&(i, _)| i == primary || !self.mobs.reg.defs[self.mobs.list[i].def].town);
        let charge = if self.survival { self.attack_charge() } else { 1.0 };
        let base = if self.survival { self.surv_attack_damage() } else { 4.0 + 3.0 * w.two_handed as u8 as f32 };
        let crit = self.melee_crit(charge);
        let dmg = base * a.damage * if crit { 1.5 } else { 1.0 };
        self.prog.was_full = charge > 0.9;
        // The mob aimed at gets everything the classic hit gives (enchantments, potions, the
        // mace's smash, fire, gear debuffs: combat_hit.rs); the others in the arc a share of
        // the plain blow, more with Sweeping Edge.
        let main = if self.survival { self.weapon_melee_bonus(primary, dmg) } else { dmg };
        // The assassin's backstab and stealth strike land on the one aimed at (assassin.rs).
        let main = self.rogue_melee_hit(primary, main);
        let side = if self.survival { self.plain_melee_bonus(dmg) } else { dmg };
        let sweeping = crate::enchanting::level(&self.held_stack(), crate::enchanting::SWEEPING) as f32;
        let share = w.sweep + (1.0 - w.sweep) * sweeping / (sweeping + 1.0) * 0.6;
        self.vulcao_melee_hit(primary);
        let push = 6.0 + a.push * charge.max(0.3);
        for (k, &(i, _)) in hits.iter().enumerate().take(6) {
            let landed = if k == 0 { self.strike_mob(i, main, push, crit, true) } else { self.strike_mob(i, side * share, push, false, false) };
            if k == 0 && landed > 0.0 && self.survival {
                self.after_melee(i, landed);
            }
            // The combo's finisher throws what it lands on off balance: its next blow waits.
            if step == 2 && landed > 0.0 && charge > 0.9 {
                self.mobs.list[i].stagger(0.8);
            }
        }
        let _ = eye;
        // A sweep of glints along the arc (a few for a thrust, a fan for a slash).
        let glow = self.glow_layer();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let n = if a.arc > 30.0 { 7 } else { 3 };
        for j in 0..n {
            let t = j as f32 / (n - 1).max(1) as f32 * 2.0 - 1.0;
            let ang = t * a.arc.to_radians() * 0.8;
            let dir = (fwd * ang.cos() + right * ang.sin()).normalize_or_zero();
            let p = eye + dir * (w.reach * 0.6) - Vec3::Y * 0.25;
            self.particles.glow(p, dir * 1.5, glow, [1.0, 1.0, 0.95], 0.05, 0.25);
        }
        self.attack_cd = if self.survival { 0.1 } else { 0.45 };
        self.break_cd = 0.3;
        self.mods.combat.last = self.now;
        self.mods.combat.step = (step + 1) % 3;
        self.anim.start_styled_swing(a.style);
        // The blow lands: the swing freezes for a moment at its impact (longer for crits and
        // big weapons).
        self.anim.land_hit(if crit { 0.09 } else { 0.05 } + if w.two_handed { 0.025 } else { 0.0 });
        self.blade_swing_fx(&w, a.style, Some(primary));
        self.sfx(Sfx::MobHit);
        if step == 2 {
            // The finisher lands heavier.
            self.sfx(Sfx::Hit(crate::audio::Material::Metal));
        }
        self.pads.rumble(0.3 + 0.2 * w.two_handed as u8 as f32, 0.6, 0.1);
        true
    }

    /// A swing that hits nothing: the weapon's combo swing (so slashes and thrusts still
    /// alternate), or the plain arm swing.
    pub(crate) fn combat_air_swing(&mut self) {
        let Some(w) = self.held_weapon() else {
            self.anim.start_swing();
            return;
        };
        let step = if self.now - self.mods.combat.last <= w.cooldown + COMBO_WINDOW { self.mods.combat.step } else { 0 };
        self.mods.combat.last = self.now;
        self.mods.combat.step = (step + 1) % 3;
        let style = w.combo[step as usize % 3].style;
        self.anim.start_styled_swing(style);
        self.blade_swing_fx(&w, style, None);
    }

    /// Two-handed weapons need both hands: the off-hand item goes back into the bag.
    pub(crate) fn update_combat(&mut self) {
        let two = self.held_weapon().is_some_and(|w| w.two_handed);
        if !two {
            self.mods.combat.warned = false;
            return;
        }
        // A shield stays: it is slung on the back until the hands are free (shield.rs).
        if let Some(st) = self.surv.offhand.filter(|s| self.items.get(s.item).name != "shield") {
            let left = self.surv.inv.add(&self.items, st);
            self.surv.offhand = left;
            if !self.mods.combat.warned {
                self.mods.combat.warned = true;
                self.toast(if left.is_none() { "Arma de duas mãos: o item da outra mão foi para a mochila" } else { "Arma de duas mãos: libere a outra mão" }, 3.0);
            }
        }
    }

    /// The combo step shown on the F3 screen.
    pub(crate) fn combat_line(&self) -> Option<String> {
        let w = self.held_weapon()?;
        let step = if self.now - self.mods.combat.last <= w.cooldown + COMBO_WINDOW { self.mods.combat.step } else { 0 };
        Some(format!("Combate: golpe {} de 3{}", step + 1, if w.two_handed { " (duas mãos)" } else { "" }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_classes_from_names() {
        assert!(weapon_of("iron_greatsword", Some(ToolKind::Sword)).unwrap().two_handed);
        assert!(weapon_of("iron_spear", Some(ToolKind::Sword)).unwrap().reach > 4.0);
        // The spear reaches further than any other melee weapon.
        for w in [SWORD, AXE, GREATSWORD, HALBERD] {
            assert!(SPEAR.reach > w.reach + 1.0);
        }
        assert!(SPEAR_REACH > 5.0);
        assert!(!weapon_of("iron_sword", Some(ToolKind::Sword)).unwrap().two_handed);
        assert!(weapon_of("iron_pickaxe", Some(ToolKind::Pickaxe)).is_none());
        // Bows punch, whips reach far.
        assert!(weapon_of("copper_bow", Some(ToolKind::Sword)).is_none());
        assert!(weapon_of("leather_whip", Some(ToolKind::Sword)).unwrap().reach >= 5.0);
        // Every combo ends with a heavier blow.
        for w in [SWORD, AXE, GREATSWORD, SPEAR, HALBERD, RAPIER, SABRE, CLEAVER, WHIP, MACE, CLUB, TRIDENT, SCYTHE, DAGGER] {
            assert!(w.combo[2].damage > w.combo[0].damage);
        }
        // A dagger is the quickest and shortest blade.
        let d = weapon_of("iron_dagger", Some(ToolKind::Sword)).unwrap();
        assert!(d.cooldown < SWORD.cooldown && d.reach < SWORD.reach && !d.two_handed);
    }

    /// Sword families and the other one-handed classes change the rhythm, never the damage over
    /// time: the balance sheet (balance.rs) stays where it was.
    #[test]
    fn families_keep_the_sword_dps() {
        let base = SWORD.swing_dps(10.0);
        for w in [RAPIER, SABRE, CLEAVER, WHIP, MACE, CLUB, TRIDENT, SCYTHE] {
            let d = w.swing_dps(10.0);
            assert!((d - base).abs() < base * 0.01, "{w:?}: {d} vs {base}");
        }
        assert!(weapon_of("blue_katana", Some(ToolKind::Sword)).unwrap().cooldown < SWORD.cooldown);
        assert!(weapon_of("lead_sword", Some(ToolKind::Sword)).unwrap().cooldown > SWORD.cooldown);
    }
}
