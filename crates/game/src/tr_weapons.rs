//! Lote 4 of the Terraria side (plan/terraria-completo.md): weapons that fly.
//! - Yoyos: hold the use button and the yoyo spins out to where you aim, biting what it
//!   touches; let go and it comes back.
//! - Boomerangs and flails: thrown out, they turn around and come back to your hand.
//! - Guns: fire musket balls (or meteor shot) from the inventory; the shotgun spreads.
//! - Throwables: shuriken and knives arc and are used up; bombs and dynamite blow up the
//!   ground, grenades only hurt.
//! - The ballista rod sets down a sentry that shoots at monsters for two minutes.
//! - The enchanted sword sends a beam with each swing; the star saber calls stars down.
//! Ore bows are plain bows that hit harder (weapons.rs asks `bow_power`).

use glam::{IVec3, Vec3};
use voxel::physics::Terrain;

use crate::audio::{Material, Sfx};
use crate::sfx_extra::Fx;
use crate::effects::{BLEEDING, CHILLED, POISON};
use crate::item_render::GpuItem;
use crate::rpg::{MELEE, MOB_BURN, RANGED, ROGUE, SUMMON};
use crate::Game;

#[derive(Clone, Copy, PartialEq)]
enum Wk {
    /// Reach in blocks.
    Yoyo(f32),
    /// Seconds out before it turns around.
    Boomerang(f32),
    Flail(f32),
    /// Pellets per shot and their spread.
    Gun(u8, f32),
    /// Used up; gravity.
    Throw(f32),
    /// Blast size; breaks blocks; fuse.
    Bomb(f32, bool, f32),
    Sentry,
    /// Swung: a beam (false) or a star from the sky (true).
    Beam(bool),
    /// Fires its ammo as a blast (tr3_tools::gun_ammo): blast size, flies straight (a rocket)
    /// or arcs and bounces (a grenade).
    Launcher(f32, bool),
}

struct WDef {
    item: &'static str,
    kind: Wk,
    dmg: f32,
    cd: f32,
    speed: f32,
    debuff: Option<(u8, f32)>,
}

const fn w(item: &'static str, kind: Wk, dmg: f32, cd: f32, speed: f32, debuff: Option<(u8, f32)>) -> WDef {
    WDef { item, kind, dmg, cd, speed, debuff }
}

const WEAPONS: &[WDef] = &[
    w("wood_yoyo", Wk::Yoyo(6.0), 2.4, 0.0, 22.0, None),
    w("jungle_yoyo", Wk::Yoyo(7.0), 3.4, 0.0, 24.0, Some((POISON, 4.0))),
    w("umbral_yoyo", Wk::Yoyo(8.0), 3.6, 0.0, 26.0, None),
    w("sanguinite_yoyo", Wk::Yoyo(8.0), 3.6, 0.0, 26.0, Some((BLEEDING, 4.0))),
    w("magma_yoyo", Wk::Yoyo(9.0), 5.4, 0.0, 28.0, Some((MOB_BURN, 4.0))),
    w("wood_boomerang", Wk::Boomerang(0.45), 3.0, 0.3, 18.0, None),
    w("enchanted_boomerang", Wk::Boomerang(0.5), 5.0, 0.3, 22.0, None),
    w("ice_boomerang", Wk::Boomerang(0.5), 5.0, 0.3, 22.0, Some((CHILLED, 3.0))),
    w("thorn_chakram", Wk::Boomerang(0.55), 7.0, 0.3, 24.0, Some((POISON, 5.0))),
    w("flame_boomerang", Wk::Boomerang(0.6), 8.0, 0.3, 26.0, Some((MOB_BURN, 4.0))),
    w("iron_flail", Wk::Flail(0.32), 6.0, 0.4, 22.0, None),
    w("umbral_flail", Wk::Flail(0.36), 9.0, 0.4, 24.0, None),
    w("sanguinite_flail", Wk::Flail(0.36), 9.0, 0.4, 24.0, Some((BLEEDING, 4.0))),
    w("bone_flail", Wk::Flail(0.36), 8.0, 0.4, 24.0, None),
    w("sun_flail", Wk::Flail(0.4), 12.0, 0.4, 26.0, Some((MOB_BURN, 5.0))),
    w("flintlock_pistol", Wk::Gun(1, 0.0), 5.0, 0.55, 70.0, None),
    w("musket", Wk::Gun(1, 0.0), 10.0, 0.9, 80.0, None),
    w("boomstick", Wk::Gun(4, 0.12), 4.0, 1.0, 60.0, None),
    w("rapid_gun", Wk::Gun(1, 0.035), 1.9, 0.11, 70.0, None),
    w("shuriken", Wk::Throw(6.0), 2.4, 0.25, 26.0, None),
    w("throwing_knife", Wk::Throw(10.0), 3.0, 0.3, 24.0, None),
    w("javelin", Wk::Throw(7.0), 4.5, 0.45, 26.0, None),
    w("bone_javelin", Wk::Throw(7.0), 5.0, 0.45, 26.0, Some((BLEEDING, 3.0))),
    w("sun_javelin", Wk::Throw(5.0), 21.0, 0.35, 32.0, Some((MOB_BURN, 5.0))),
    w("bomb", Wk::Bomb(3.0, true, 2.5), 10.0, 0.8, 12.0, None),
    w("grenade", Wk::Bomb(2.5, false, 1.6), 6.0, 0.6, 16.0, None),
    w("dynamite", Wk::Bomb(5.0, true, 4.0), 20.0, 1.0, 10.0, None),
    w("ballista_rod", Wk::Sentry, 5.0, 1.0, 40.0, None),
    w("enchanted_sword", Wk::Beam(false), 2.0, 1.0, 30.0, None),
    w("star_saber", Wk::Beam(true), 2.5, 1.2, 40.0, None),
    // Lote 7: the last bosses' and the pillars' weapons.
    w("crystal_yoyo", Wk::Yoyo(11.0), 7.0, 0.0, 30.0, None),
    w("stone_fist", Wk::Flail(0.45), 30.0, 0.4, 28.0, None),
    w("tide_boomerang", Wk::Boomerang(0.6), 30.0, 0.25, 30.0, Some((CHILLED, 4.0))),
    w("bubble_rifle", Wk::Gun(3, 0.08), 5.7, 0.35, 60.0, None),
    w("prism_saber", Wk::Beam(true), 14.0, 0.8, 44.0, None),
    w("solar_eruption", Wk::Flail(0.5), 50.0, 0.35, 32.0, Some((MOB_BURN, 6.0))),
    w("vortex_beater", Wk::Gun(1, 0.03), 4.7, 0.09, 90.0, None),
    w("nebula_blade", Wk::Beam(false), 18.0, 0.62, 40.0, None),
    w("stardust_yoyo", Wk::Yoyo(14.0), 15.5, 0.0, 34.0, None),
    // Lote 14 (second round): the late swords and more sentries.
    w("night_edge", Wk::Beam(false), 3.0, 1.0, 32.0, None),
    w("radiant_blade", Wk::Beam(false), 8.0, 0.7, 34.0, None),
    w("true_night_edge", Wk::Beam(false), 10.0, 0.95, 36.0, Some((POISON, 3.0))),
    w("true_radiant_blade", Wk::Beam(false), 10.0, 1.0, 36.0, None),
    w("terra_blade", Wk::Beam(false), 9.0, 1.2, 44.0, None),
    w("celestial_wrath", Wk::Beam(true), 16.0, 1.0, 50.0, None),
    w("apex_blade", Wk::Beam(true), 16.0, 1.3, 56.0, None),
    w("explosive_trap_rod", Wk::Sentry, 16.0, 2.0, 30.0, Some((MOB_BURN, 3.0))),
    w("lightning_aura_rod", Wk::Sentry, 4.0, 0.35, 60.0, None),
    w("flame_sentry_rod", Wk::Sentry, 7.0, 0.8, 36.0, Some((MOB_BURN, 4.0))),
    // The balance sheet (plan/balanceamento.md): a gun and knives for the stages that had none.
    w("adamantite_rifle", Wk::Gun(1, 0.0), 9.0, 0.36, 80.0, None),
    w("laser_rifle", Wk::Gun(1, 0.0), 12.0, 0.33, 90.0, None),
    w("poison_knife", Wk::Throw(10.0), 3.8, 0.3, 24.0, Some((POISON, 4.0))),
    w("magmarite_knife", Wk::Throw(10.0), 5.8, 0.3, 26.0, Some((MOB_BURN, 3.0))),
    w("mythril_javelin", Wk::Throw(7.0), 13.0, 0.45, 28.0, None),
    w("verdite_knife", Wk::Throw(10.0), 13.0, 0.3, 28.0, None),
    // Reauditoria 3, L20 (tr3_tools.rs): the guns and launchers Terraria still had, and the
    // weapons of L18's mini-bosses.
    w("blowpipe", Wk::Gun(1, 0.0), 3.4, 0.6, 40.0, None),
    w("minishark", Wk::Gun(1, 0.04), 1.3, 0.11, 70.0, None),
    w("star_cannon", Wk::Gun(1, 0.0), 6.3, 0.55, 45.0, None),
    w("megashark", Wk::Gun(1, 0.04), 2.2, 0.09, 80.0, None),
    w("flamethrower", Wk::Gun(1, 0.1), 2.6, 0.1, 22.0, Some((MOB_BURN, 4.0))),
    w("sniper_rifle", Wk::Gun(1, 0.0), 24.0, 0.75, 140.0, None),
    w("coin_gun", Wk::Gun(1, 0.03), 3.4, 0.12, 80.0, None),
    w("elf_gun", Wk::Gun(1, 0.05), 3.0, 0.09, 75.0, Some((MOB_BURN, 3.0))),
    w("rocket_launcher", Wk::Launcher(2.6, true), 22.0, 0.8, 30.0, None),
    w("grenade_launcher", Wk::Launcher(2.2, false), 18.0, 0.65, 22.0, None),
    w("paladin_hammer", Wk::Boomerang(0.6), 26.0, 0.25, 28.0, None),
    w("betsy_wrath", Wk::Beam(false), 8.0, 0.7, 34.0, Some((MOB_BURN, 5.0))),
    w("razorpine", Wk::Beam(false), 7.0, 0.6, 36.0, Some((CHILLED, 3.0))),
    w("spooky_staff", Wk::Sentry, 8.0, 0.75, 36.0, None),
    // The gunner's pass (HP 2026-11-01, guns.rs): the guns Terraria has that we lacked, one or
    // more per stage, each with its own feel in 3D (plan/atirador-analise.md).
    w("revolver", Wk::Gun(1, 0.0), 5.2, 0.62, 85.0, None),
    w("handgun", Wk::Gun(1, 0.02), 5.7, 0.33, 85.0, None),
    w("phoenix_blaster", Wk::Gun(1, 0.02), 8.0, 0.45, 90.0, Some((MOB_BURN, 3.0))),
    w("quad_barrel_shotgun", Wk::Gun(6, 0.16), 4.0, 1.0, 60.0, None),
    w("shotgun", Wk::Gun(5, 0.1), 5.8, 0.85, 70.0, None),
    w("clockwork_rifle", Wk::Gun(1, 0.015), 3.8, 0.45, 95.0, None),
    w("uzi", Wk::Gun(1, 0.04), 2.1, 0.085, 85.0, None),
    w("onyx_blaster", Wk::Gun(4, 0.09), 6.0, 0.7, 70.0, None),
    w("tactical_shotgun", Wk::Gun(6, 0.07), 5.3, 0.65, 80.0, None),
    w("venus_magnum", Wk::Gun(1, 0.0), 7.8, 0.21, 100.0, None),
    w("chain_gun", Wk::Gun(1, 0.05), 2.55, 0.07, 85.0, None),
    w("lumite_minigun", Wk::Gun(1, 0.04), 3.7, 0.07, 100.0, None),
    // The sword review (packs/vanilla/espadas/): an ice bolt with each swing, then a frostier one.
    w("ice_blade", Wk::Beam(false), 3.0, 1.0, 30.0, Some((CHILLED, 2.0))),
    w("frost_brand", Wk::Beam(false), 6.0, 0.75, 36.0, Some((CHILLED, 3.0))),
];

/// The colour of a sword's beam.
fn beam_tint(item: &str) -> [f32; 3] {
    match item {
        "night_edge" => [1.2, 0.5, 2.0],
        "radiant_blade" | "true_radiant_blade" => [2.2, 1.8, 0.7],
        "true_night_edge" => [0.8, 1.8, 1.0],
        "terra_blade" => [0.6, 2.2, 0.7],
        "celestial_wrath" => [2.0, 0.8, 2.0],
        "apex_blade" => [1.2, 1.8, 2.4],
        "ice_blade" | "frost_brand" => [0.7, 1.4, 2.2],
        "star_saber" | "prism_saber" => [2.0, 1.8, 0.8],
        "betsy_wrath" => [2.4, 0.9, 0.3],
        "razorpine" => [0.5, 1.8, 0.7],
        _ => [0.7, 1.1, 2.2],
    }
}

/// How much harder an ore bow hits than the plain bow (None: not one of them).
pub fn bow_power(name: &str) -> Option<f32> {
    match name {
        "copper_bow" => Some(1.08),
        "silver_bow" => Some(1.15),
        "platinum_bow" => Some(1.22),
        "umbral_bow" | "sanguinite_bow" => Some(1.65),
        "magmarite_bow" => Some(2.44),
        "cobalt_bow" => Some(2.75),
        "mythril_bow" => Some(3.1),
        "adamantite_bow" => Some(3.57),
        "radiant_bow" => Some(4.6),
        "verdite_bow" => Some(5.29),
        "lumite_bow" => Some(7.65),
        // Bows with a trick (archery.rs).
        "hive_bow" => Some(1.6),
        "batwing_bow" => Some(2.4),
        "marrow_bow" => Some(3.0),
        "frost_bow" => Some(2.9),
        "storm_bow" => Some(3.1),
        "pulse_bow" => Some(4.8),
        "tide_bow" => Some(4.7),
        "phoenix_bow" => Some(7.4),
        "skysplit_bow" => Some(7.3),
        _ => None,
    }
}

/// Damage per second on one monster and the class it counts for, for the balance sheet
/// (balance.rs): a yoyo bites every 0.3 s, a boomerang or flail lands about one and a half
/// hits a throw, a spread gun puts three quarters of its pellets on one monster, a sentry
/// rod's turret shoots on its own, and a beam sword's beams come on top of its swings.
#[cfg(test)]
pub(crate) fn sustained_dps(name: &str) -> Option<(usize, f32)> {
    let d = WEAPONS.iter().find(|w| w.item == name)?;
    Some(match d.kind {
        Wk::Yoyo(_) => (MELEE, d.dmg / 0.3),
        Wk::Boomerang(t) | Wk::Flail(t) => (MELEE, 1.5 * d.dmg / (2.0 * t + d.cd)),
        Wk::Gun(n, spread) => (RANGED, d.dmg * n as f32 * crate::guns::burst(name) as f32 * if spread > 0.05 { 0.75 } else { 1.0 } / d.cd),
        Wk::Throw(_) | Wk::Bomb(..) => (ROGUE, d.dmg / d.cd),
        Wk::Sentry => (SUMMON, d.dmg / d.cd),
        Wk::Beam(_) => (MELEE, d.dmg / d.cd),
        Wk::Launcher(..) => (RANGED, d.dmg / d.cd),
    })
}

/// Every weapon of this file, for balance.rs to check that each one has a stage.
#[cfg(test)]
pub(crate) fn all_weapons() -> impl Iterator<Item = &'static str> {
    WEAPONS.iter().map(|w| w.item)
}

pub fn is_gun(name: &str) -> bool {
    def_of(name).is_some_and(|i| matches!(WEAPONS[i].kind, Wk::Gun(..) | Wk::Launcher(..)))
}

/// A yoyo (fight_anim.rs: flicked out from the wrist).
pub fn is_yoyo(name: &str) -> bool {
    def_of(name).is_some_and(|i| matches!(WEAPONS[i].kind, Wk::Yoyo(_)))
}

/// Thrown from the hand: boomerangs, flails, knives, bombs (fight_anim.rs: overhand throw).
pub fn is_thrown(name: &str) -> bool {
    def_of(name).is_some_and(|i| matches!(WEAPONS[i].kind, Wk::Boomerang(_) | Wk::Flail(_) | Wk::Throw(_) | Wk::Bomb(..)))
}

/// The cooldown between uses, for the length of the swing (fight_anim.rs).
pub fn use_time(name: &str) -> Option<f32> {
    def_of(name).map(|i| WEAPONS[i].cd)
}

pub fn is_weapon(name: &str) -> bool {
    WEAPONS.iter().any(|w| w.item == name)
}

#[derive(Clone, Copy, PartialEq)]
enum Fly {
    Yoyo,
    Boomerang,
    Flail,
    Bullet,
    Thrown,
    Bomb,
    Sentry,
    Bolt,
}

struct TrShot {
    fly: Fly,
    def: usize,
    pos: Vec3,
    vel: Vec3,
    age: f32,
    life: f32,
    dmg: f32,
    class: usize,
    pierce: u8,
    /// Mobs hit lately and seconds until each can be hit again.
    hits: Vec<(usize, f32)>,
    back: bool,
    tint: [f32; 3],
    /// Sentries: seconds to the next shot.
    shot_t: f32,
    /// Bullets: from where the shot leaves to the gun's muzzle, so the tracer starts at the
    /// muzzle and joins the line of the shot (the shot itself flies from the eye, true to the
    /// crosshair); and which `guns::AMMO` it is (+1, 0 none).
    off: Vec3,
    ammo: u8,
}

#[derive(Default)]
pub struct Weapons {
    shots: Vec<TrShot>,
    cd: f32,
    swing_cd: f32,
    was_held: bool,
    /// The held gun's kick, spin, pump and aim (guns.rs).
    pub(crate) gun: crate::guns::GunState,
    /// Where the held gun's muzzle was last drawn (world), for the flash and the tracers.
    pub(crate) muzzle: Option<Vec3>,
    /// Seconds since the last shot of a gun.
    since_shot: f32,
    /// A burst going on: shots left and seconds to the next.
    burst: (u8, f32),
    /// Counts impacts, so a fast gun's dust and sounds are thinned out.
    impacts: u32,
}

/// A slot meaning "no ammo is used up" (creative).
const FREE: usize = usize::MAX - 1;

fn def_of(name: &str) -> Option<usize> {
    WEAPONS.iter().position(|w| w.item == name)
}

/// A sword swung for its beam (terra blade, night's edge...): held by the hilt like a tool.
pub(crate) fn is_blade(name: &str) -> bool {
    def_of(name).is_some_and(|i| matches!(WEAPONS[i].kind, Wk::Beam(_)))
}

impl Game {
    /// Right click with one of these weapons: say we handle it (the firing is in the update).
    pub(crate) fn tr_weapon_use(&mut self, name: &str) -> Option<bool> {
        let i = def_of(name)?;
        if let Wk::Beam(_) = WEAPONS[i].kind {
            return None;
        }
        Some(true)
    }

    pub(crate) fn update_tr_weapons(&mut self, dt: f32, use_held: bool) {
        let wp = &mut self.t5.tr.wep;
        wp.cd -= dt;
        wp.swing_cd -= dt;
        let name = self.held_key();
        let held = def_of(&name);
        let playing = self.screen == crate::Screen::Playing;
        let pressed = use_held && playing;
        if let Some(i) = held {
            let d = &WEAPONS[i];
            match d.kind {
                Wk::Yoyo(_) => {
                    if pressed && !self.t5.tr.wep.shots.iter().any(|s| s.fly == Fly::Yoyo && !s.back) {
                        let from = self.body.eye() + self.forward() * 0.6;
                        self.add_shot(Fly::Yoyo, i, from, self.forward() * d.speed, 999.0, MELEE);
                        self.sfx(Sfx::Place(Material::Cloth));
                    }
                }
                Wk::Beam(star) => {
                    if self.mouse_held[0] && playing && self.t5.tr.wep.swing_cd <= 0.0 {
                        self.t5.tr.wep.swing_cd = d.cd;
                        self.fire_beam(i, star);
                    }
                }
                Wk::Gun(..) => {
                    let g = crate::guns::index(d.item);
                    if pressed && self.t5.tr.wep.cd <= 0.0 && self.t5.tr.wep.burst.0 == 0 {
                        // A minigun fires slower while it winds up.
                        let slow = g.filter(|&g| crate::guns::spins_up(g)).map_or(1.0, |_| crate::guns::wind_slowdown(&self.t5.tr.wep.gun));
                        self.t5.tr.wep.cd = d.cd * slow;
                        let n = crate::guns::burst(d.item);
                        self.t5.tr.wep.burst = (n - 1, 0.07);
                        self.fire_weapon(i);
                    }
                    // The rest of a burst follows on its own.
                    let b = &mut self.t5.tr.wep.burst;
                    if b.0 > 0 {
                        b.1 -= dt;
                        if b.1 <= 0.0 {
                            b.0 -= 1;
                            b.1 = 0.07;
                            self.fire_weapon(i);
                        }
                    }
                }
                _ => {
                    if pressed && self.t5.tr.wep.cd <= 0.0 {
                        // The throwing glove throws quicker (assassin.rs).
                        self.t5.tr.wep.cd = if matches!(d.kind, Wk::Throw(_) | Wk::Bomb(..)) { d.cd * self.rogue_throw_speed() } else { d.cd };
                        self.fire_weapon(i);
                    }
                }
            }
        }
        // The held gun: kick, spin, the pump, and raised to aim while shooting, a moment after,
        // or looking down a scope.
        let g = crate::guns::index(&name);
        let wp = &mut self.t5.tr.wep;
        wp.since_shot += dt;
        if g.is_none() {
            wp.burst = (0, 0.0);
            wp.muzzle = None;
        }
        wp.gun.update(g, pressed && g.is_some(), dt);
        let want = if g.is_some() && (pressed || wp.since_shot < 1.4) { 1.0 } else { 0.0 };
        let want = if self.scope_up() { 1.0 } else { want };
        let wp = &mut self.t5.tr.wep;
        let rate = if want > wp.gun.aim { 9.0 } else { 3.0 };
        wp.gun.aim += (want - wp.gun.aim).clamp(-dt * rate, dt * rate);
        // Yoyos come back when the button is let go or the yoyo is put away.
        let yoyo_held = held.is_some_and(|i| matches!(WEAPONS[i].kind, Wk::Yoyo(_)));
        if !pressed || !yoyo_held {
            for s in self.t5.tr.wep.shots.iter_mut().filter(|s| s.fly == Fly::Yoyo) {
                s.back = true;
            }
        }
        self.t5.tr.wep.was_held = pressed;
        self.update_tr_shots(dt);
    }

    fn add_shot(&mut self, fly: Fly, def: usize, pos: Vec3, vel: Vec3, life: f32, class: usize) {
        let d = &WEAPONS[def];
        let tint = match d.debuff {
            Some((MOB_BURN, _)) => [2.0, 0.9, 0.3],
            Some((CHILLED, _)) => [0.6, 1.2, 2.0],
            Some((POISON, _)) => [0.6, 1.8, 0.5],
            Some((BLEEDING, _)) => [1.8, 0.3, 0.3],
            _ => [1.6, 1.4, 1.0],
        };
        self.t5.tr.wep.shots.push(TrShot { fly, def, pos, vel, age: 0.0, life, dmg: d.dmg, class, pierce: 0, hits: Vec::new(), back: false, tint, shot_t: 0.5, off: Vec3::ZERO, ammo: 0 });
    }

    /// The first stack of `names` in the inventory: (slot, which name).
    fn find_ammo(&self, names: &[&str]) -> Option<(usize, usize)> {
        for (slot, s) in self.surv.inv.slots.iter().enumerate() {
            let Some(s) = s else { continue };
            let n = &self.items.get(s.item).name;
            if let Some(k) = names.iter().position(|a| a == n) {
                return Some((slot, k));
            }
        }
        None
    }

    /// The first bullet a gun can fire: the off hand first (so you choose by putting a stack
    /// there), then the inventory in order. (slot, which of `names`); usize::MAX is the off hand.
    fn find_gun_ammo(&self, names: &[&str]) -> Option<(usize, usize)> {
        if let Some(s) = self.surv.offhand {
            let n = &self.items.get(s.item).name;
            if let Some(k) = names.iter().position(|a| a == n) {
                return Some((usize::MAX, k));
            }
        }
        self.find_ammo(names)
    }

    fn take_gun_ammo(&mut self, slot: usize) {
        if slot == FREE {
            return;
        }
        if slot == usize::MAX {
            if let Some(s) = &mut self.surv.offhand {
                s.count -= 1;
                if s.count == 0 {
                    self.surv.offhand = None;
                }
            }
        } else {
            self.surv.inv.take(slot, 1);
        }
    }

    /// The held gun's model index (guns.rs).
    pub(crate) fn held_gun(&self) -> Option<usize> {
        crate::guns::index(&self.held_key())
    }

    /// A gun going off at `muzzle` (world) aimed along `f`: its sound, the kick, a flash of
    /// light and fire at the muzzle, smoke, and a spent case thrown out to the right.
    fn gun_fire_fx(&mut self, def: usize, g: Option<usize>, muzzle: Vec3, f: Vec3) {
        self.t5.tr.wep.since_shot = 0.0;
        self.anim.start_swing();
        let Some(g) = g else {
            self.sfx(Sfx::Piston(true));
            return;
        };
        let side = self.surv.rng.f32() * 2.0 - 1.0;
        self.t5.tr.wep.gun.fire(g, side);
        self.sfx(Sfx::Fx(Fx::Gun(crate::guns::shot(g) as u8)));
        let glow = self.glow_layer();
        // The first-person gun is drawn nearer the eye (player_model.rs HAND_NEAR): its fire is
        // as much smaller.
        let k = if self.camera_mode == 0 { 0.3 } else { 1.0 };
        let carry = self.body.vel;
        let fam = crate::guns::GUNS[g].fam;
        use crate::guns::Fam;
        let tint = match fam {
            Fam::Laser => Some([1.8, 0.8, 2.4]),
            Fam::Vortex => Some([0.6, 2.2, 2.0]),
            Fam::Bubble => None,
            Fam::Flamethrower | Fam::Blowpipe => None,
            Fam::StarCannon => Some([2.2, 2.0, 1.0]),
            _ if crate::guns::GUNS[g].item == "lumite_minigun" => Some([0.8, 2.2, 2.0]),
            _ => Some([2.4, 1.6, 0.7]),
        };
        if let Some(t) = tint {
            // A short star of fire out of the muzzle, brightest at its heart.
            self.particles.glow(muzzle, carry, glow, [t[0] * 1.2, t[1] * 1.2, t[2] * 1.2], 0.11 * k, 0.05);
            for _ in 0..4 {
                let r = &mut self.surv.rng;
                let v = f * (3.0 + r.f32() * 4.0) + Vec3::new(r.f32() - 0.5, r.f32() - 0.5, r.f32() - 0.5) * 2.0;
                self.particles.glow(muzzle + f * 0.05 * k, carry + v * k, glow, t, 0.05 * k, 0.06);
            }
        }
        // Smoke: a thick white-grey cloud off black powder, a wisp off the rest.
        let stone = self.block_look(self.reg.id_or_air("stone")).0;
        let puffs = if crate::guns::black_powder(g) { 7 } else if matches!(fam, Fam::Bubble | Fam::Laser | Fam::Vortex) { 0 } else { 1 };
        for n in 0..puffs {
            let at = muzzle + f * (0.08 + n as f32 * 0.05) * k;
            self.particles.mote(at, stone, [0.72, 0.72, 0.7], (0.08 + n as f32 * 0.012) * k, -0.25, 1.4 + n as f32 * 0.15);
        }
        if fam == Fam::Bubble {
            for _ in 0..3 {
                let r = &mut self.surv.rng;
                let v = f * 2.0 + Vec3::new(r.f32() - 0.5, r.f32(), r.f32() - 0.5);
                self.particles.glow(muzzle, carry + v * k, glow, [0.7, 1.4, 1.8], 0.04 * k, 0.4);
            }
        }
        if crate::guns::ejects(g) {
            // The spent case flies out to the right and up, tumbling down on the ground.
            let fr = crate::guns::frame(g);
            let right = f.cross(Vec3::Y).normalize_or(Vec3::X);
            let at = muzzle - f * ((fr.muzzle.z - fr.eject.z) * crate::guns::PX * k) + right * 0.04 * k + Vec3::Y * 0.02 * k;
            let gold = self.block_look(self.reg.id_or_air("gold_block")).0;
            let r = &mut self.surv.rng;
            let v = carry + right * (2.2 + r.f32()) + Vec3::Y * (2.0 + r.f32() * 1.5) - f * 0.5;
            self.particles.casing(at, v, gold, [1.0, 0.82, 0.5], 0.022 * k);
        }
        let _ = def;
    }

    fn fire_weapon(&mut self, i: usize) {
        let d = &WEAPONS[i];
        let f = self.forward();
        let eye = self.body.eye();
        let out = self.t5.tr.wep.shots.iter().filter(|s| s.def == i).count();
        match d.kind {
            Wk::Boomerang(_) | Wk::Flail(_) => {
                if out > 0 {
                    return;
                }
                let fly = if matches!(d.kind, Wk::Flail(_)) { Fly::Flail } else { Fly::Boomerang };
                self.add_shot(fly, i, eye + f * 0.6, f * d.speed, 6.0, MELEE);
                self.anim.start_swing();
                self.sfx(Sfx::Place(Material::Cloth));
            }
            Wk::Gun(pellets, spread) => {
                let own = crate::tr3_tools::gun_ammo(d.item);
                let bullets = crate::guns::bullet_names();
                let names = own.map_or(&bullets[..], |a| a.0);
                // In creative every gun is loaded with plain bullets (or its own first ammo).
                let ammo = if self.survival { self.find_gun_ammo(names) } else { Some((FREE, 0)) };
                let Some((slot, kind)) = ammo else {
                    self.toast(format!("Sem munição ({})", own.map_or("balas", |a| a.1)), 1.5);
                    self.t5.tr.wep.cd = 0.6;
                    self.t5.tr.wep.burst = (0, 0.0);
                    self.sfx(Sfx::Fx(Fx::DryFire));
                    return;
                };
                let saved = self.t5.rpg.stats.ammo_save > 0.0 && self.surv.rng.f32() < self.t5.rpg.stats.ammo_save;
                if !saved {
                    self.take_gun_ammo(slot);
                }
                let bullet = if own.is_none() { crate::guns::AMMO.get(kind) } else { None };
                let g = crate::guns::index(d.item);
                let muzzle = self.t5.tr.wep.muzzle.unwrap_or(eye + f * 0.7 - Vec3::Y * 0.25);
                // Across the view, for the cone of pellets.
                let side = f.cross(Vec3::Y).normalize_or(Vec3::X);
                let up = side.cross(f);
                for _ in 0..pellets {
                    let r = &mut self.surv.rng;
                    // A round cone: a random point in a disc across the aim.
                    let (a, rr) = (r.f32() * std::f32::consts::TAU, r.f32().sqrt() * spread);
                    let dir = (f + (side * a.cos() + up * a.sin()) * rr).normalize_or_zero();
                    let flame = d.item == "flamethrower";
                    let speed = d.speed * bullet.map_or(1.0, |b| b.speed);
                    let life = if flame { 0.35 } else { 1.4 };
                    let from = eye + dir * 0.3;
                    self.add_shot(Fly::Bullet, i, from, dir * speed, life, RANGED);
                    if let Some(s) = self.t5.tr.wep.shots.last_mut() {
                        s.off = muzzle - from;
                        if let Some(b) = bullet {
                            s.dmg *= b.dmg;
                            s.pierce = b.pierce;
                            s.tint = b.tint;
                            s.ammo = kind as u8 + 1;
                        } else {
                            // The guns with their own ammo: coins hit by their worth, flames pass
                            // through, stars glow.
                            match d.item {
                                "coin_gun" => {
                                    s.dmg *= crate::tr3_tools::coin_factor(kind);
                                    s.tint = [[1.8, 1.0, 0.6], [1.6, 1.6, 1.7], [2.0, 1.7, 0.6], [1.4, 1.9, 2.0]][kind.min(3)];
                                }
                                "flamethrower" => {
                                    s.pierce = 3;
                                    s.tint = [2.4, 1.0, 0.3];
                                }
                                "star_cannon" => {
                                    s.pierce = 1;
                                    s.tint = [2.2, 2.0, 0.9];
                                }
                                _ => s.tint = [1.2, 1.4, 0.8],
                            }
                        }
                        // The energy guns' bolts are their own colour.
                        match d.item {
                            "laser_rifle" => s.tint = [1.8, 0.7, 2.4],
                            "vortex_beater" | "lumite_minigun" if bullet.is_some_and(|b| b.item == "musket_ball") => s.tint = [0.5, 2.2, 2.0],
                            "elf_gun" => s.tint = [2.2, 0.9, 0.4],
                            "bubble_rifle" => s.tint = [0.7, 1.6, 2.2],
                            _ => {}
                        }
                    }
                }
                // The onyx blaster sends a dark bolt that bursts along with the pellets.
                if d.item == "onyx_blaster" {
                    let from = eye + f * 0.3;
                    self.add_shot(Fly::Bullet, i, from, f * d.speed * 0.6, 1.2, RANGED);
                    if let Some(s) = self.t5.tr.wep.shots.last_mut() {
                        s.dmg *= 1.5;
                        s.pierce = 1;
                        s.tint = [1.4, 0.5, 2.2];
                        s.off = muzzle - from;
                        s.ammo = 1 + crate::guns::AMMO.iter().position(|a| a.trick == crate::guns::Trick::Pop).unwrap_or(0) as u8;
                    }
                }
                self.gun_fire_fx(i, g, muzzle, f);
            }
            Wk::Launcher(_, rocket) => {
                let (names, what) = crate::tr3_tools::gun_ammo(d.item).unwrap_or((&["rocket"], "foguetes"));
                let ammo = if self.survival { self.find_ammo(names) } else { Some((usize::MAX, 0)) };
                let Some((slot, _)) = ammo else {
                    self.toast(format!("Sem munição ({what})"), 1.5);
                    self.t5.tr.wep.cd = 0.6;
                    return;
                };
                if slot != usize::MAX {
                    self.surv.inv.take(slot, 1);
                }
                let up = if rocket { Vec3::ZERO } else { Vec3::Y * 2.5 };
                self.add_shot(Fly::Bomb, i, eye + f * 0.8, f * d.speed + up, if rocket { 3.0 } else { 1.4 }, RANGED);
                let muzzle = self.t5.tr.wep.muzzle.unwrap_or(eye + f * 0.7 - Vec3::Y * 0.25);
                self.gun_fire_fx(i, crate::guns::index(d.item), muzzle, f);
            }
            Wk::Throw(_) | Wk::Bomb(..) => {
                // The thrower's sets (ninja, fossil) and arrow savers keep some throws in hand.
                let st = &self.t5.rpg.stats;
                let keep = (st.throw_save + st.ammo_save).min(0.75);
                let saved = keep > 0.0 && self.surv.rng.f32() < keep;
                if self.survival && !saved {
                    let sel = self.selected;
                    self.surv.inv.take(sel, 1);
                }
                let (fly, class, life) = if let Wk::Bomb(_, _, fuse) = d.kind { (Fly::Bomb, ROGUE, fuse) } else { (Fly::Thrown, ROGUE, 4.0) };
                let up = if fly == Fly::Bomb { Vec3::Y * 3.0 } else { Vec3::Y * 0.5 };
                // A full stealth meter makes this throw a stealth strike (assassin.rs): double
                // damage, it goes through, and a knife or shuriken fans out into three.
                let strike = self.rogue_take_strike();
                let fan: &[f32] = if strike && fly == Fly::Thrown && !d.item.contains("javelin") { &[0.0, -0.14, 0.14] } else { &[0.0] };
                let side = f.cross(Vec3::Y).normalize_or_zero();
                for &a in fan {
                    let dir = (f + side * a).normalize_or_zero();
                    self.add_shot(fly, i, eye + f * 0.5, dir * d.speed + up + self.body.vel * 0.5, life, class);
                    if strike {
                        if let Some(s) = self.t5.tr.wep.shots.last_mut() {
                            s.dmg *= crate::assassin::STRIKE_MULT;
                            s.pierce = 2;
                            s.tint = [0.7, 0.5, 1.6];
                        }
                    }
                }
                self.anim.start_swing();
                self.sfx(Sfx::Fx(crate::sfx_extra::Fx::Throw(fly == Fly::Bomb)));
            }
            Wk::Sentry => {
                // They share the sentry slots with the staffs of sentries.rs: the oldest goes.
                let slots = self.sentry_slots();
                self.trim_sentries(slots - 1);
                let at = self.sentry_spot();
                self.add_shot(Fly::Sentry, i, at + Vec3::Y * 0.6, Vec3::ZERO, 120.0, SUMMON);
                self.anim.start_swing();
                self.sfx(Sfx::Place(Material::Wood));
                let label = format!("{} (2 minutos)", self.items.id(d.item).map(|id| self.items.get(id).display_name.clone()).unwrap_or_default());
                self.toast(label, 1.5);
            }
            _ => {}
        }
    }

    fn fire_beam(&mut self, i: usize, star: bool) {
        let d = &WEAPONS[i];
        let f = self.forward();
        if star {
            // A star falls from the sky onto what you look at.
            let target = self.aim_point(24.0);
            let r = &mut self.surv.rng;
            let from = target + Vec3::new(r.f32() * 8.0 - 4.0, 22.0, r.f32() * 8.0 - 4.0);
            self.add_shot(Fly::Bolt, i, from, (target - from).normalize_or_zero() * d.speed, 1.5, MELEE);
            if let Some(s) = self.t5.tr.wep.shots.last_mut() {
                s.tint = beam_tint(d.item);
                s.pierce = 2;
            }
        } else {
            self.add_shot(Fly::Bolt, i, self.body.eye() + f * 0.8, f * d.speed, 0.7, MELEE);
            if let Some(s) = self.t5.tr.wep.shots.last_mut() {
                s.tint = beam_tint(d.item);
                s.pierce = 3;
            }
        }
        self.sfx(Sfx::Cast(1));
    }

    /// Where the player is looking, up to `range` blocks (the first solid block).
    pub(crate) fn aim_point(&self, range: f32) -> Vec3 {
        let eye = self.body.eye();
        let f = self.forward();
        let mut t = 0.0;
        while t < range {
            let p = eye + f * t;
            if self.world.solid(p.floor().as_ivec3()) {
                return eye + f * (t - 0.3).max(0.0);
            }
            t += 0.25;
        }
        eye + f * range
    }

    fn tr_hit(&mut self, k: usize, dmg: f32, class: usize, from: Vec3, debuff: Option<(u8, f32)>) {
        let st = &self.t5.rpg.stats;
        let mut d = dmg * st.dmg[class];
        if self.surv.rng.f32() < st.crit[class] {
            d *= 1.5;
            self.combat_fx.crit(self.mobs.list[k].net_id);
        }
        if class == ROGUE {
            // An unaware monster or one hit from behind takes more (assassin.rs); the
            // assassin's oil and poison gland coat the blades too.
            d = self.rogue_ranged_hit(k, d, from);
        }
        let m = &mut self.mobs.list[k];
        m.hurt(d, from);
        m.aggro = true;
        let def = &self.mobs.reg.defs[m.def];
        self.last_hit = Some((def.display_name.clone(), m.health.max(0.0), m.max_health, self.now + 3.0));
        if let Some((id, secs)) = debuff {
            self.add_mob_debuff(k, id, secs);
        }
        self.sfx(Sfx::MobHit);
    }

    /// Hostile mobs touching a sphere at `at`.
    fn mobs_touching(&self, at: Vec3, r: f32) -> Vec<usize> {
        (0..self.mobs.list.len())
            .filter(|&k| {
                let m = &self.mobs.list[k];
                if !m.alive() || !self.is_hostile(k) {
                    return false;
                }
                let d = &self.mobs.reg.defs[m.def];
                let c = m.body.center();
                let dx = Vec3::new(at.x - c.x, 0.0, at.z - c.z).length();
                dx < d.width * 0.5 + r && (at.y - c.y).abs() < d.height * 0.5 + r
            })
            .collect()
    }

    fn update_tr_shots(&mut self, dt: f32) {
        if self.t5.tr.wep.shots.is_empty() {
            return;
        }
        let mut shots = std::mem::take(&mut self.t5.tr.wep.shots);
        let hand = self.body.eye() - Vec3::Y * 0.4 + self.forward() * 0.3;
        let glow = self.glow_layer();
        let mut blasts: Vec<(Vec3, usize)> = Vec::new();
        // Bullets: where they struck blocks, small blasts, shards, and coins for golden kills.
        let mut impacts: Vec<(Vec3, IVec3, Vec3)> = Vec::new();
        let mut pops: Vec<(Vec3, f32)> = Vec::new();
        let mut shards: Vec<TrShot> = Vec::new();
        let mut coins: Vec<(Vec3, usize)> = Vec::new();
        let mut keep = Vec::with_capacity(shots.len());
        for mut s in shots.drain(..) {
            s.age += dt;
            for h in s.hits.iter_mut() {
                h.1 -= dt;
            }
            s.hits.retain(|h| h.1 > 0.0);
            let d = &WEAPONS[s.def];
            let mut alive = s.age < s.life;
            match s.fly {
                Fly::Yoyo => {
                    let Wk::Yoyo(reach) = d.kind else { continue };
                    // A longer string (rpg.rs `M::YoyoReach`).
                    let reach = reach * self.t5.rpg.stats.yoyo_reach.clamp(1.0, 2.0);
                    let target = if s.back { hand } else { self.aim_point(reach) };
                    let to = target - s.pos;
                    s.vel = to.normalize_or_zero() * d.speed.min(to.length() / dt.max(1e-3));
                    s.pos += s.vel * dt;
                    if s.back && to.length() < 1.0 {
                        alive = false;
                    }
                    // The string, a few faint dots from the hand.
                    for k in 1..6 {
                        let p = hand.lerp(s.pos, k as f32 / 6.0);
                        self.particles.glow(p, Vec3::ZERO, glow, [0.5, 0.5, 0.5], 0.02, 0.03);
                    }
                }
                Fly::Boomerang | Fly::Flail => {
                    let out = match d.kind {
                        Wk::Boomerang(t) | Wk::Flail(t) => t,
                        _ => 0.5,
                    };
                    if s.age > out {
                        s.back = true;
                    }
                    let next = s.pos + s.vel * dt;
                    if !s.back && self.world.solid(next.floor().as_ivec3()) {
                        s.back = true;
                        self.sfx(Sfx::Hit(Material::Wood));
                    }
                    if s.back {
                        let to = hand - s.pos;
                        s.vel = to.normalize_or_zero() * d.speed * 1.2;
                        if to.length() < 1.2 {
                            alive = false;
                        }
                        s.pos += s.vel * dt;
                    } else {
                        s.pos = next;
                    }
                    if s.fly == Fly::Flail {
                        for k in 1..8 {
                            let p = hand.lerp(s.pos, k as f32 / 8.0);
                            self.particles.glow(p, Vec3::ZERO, glow, [0.45, 0.45, 0.5], 0.03, 0.03);
                        }
                    }
                }
                Fly::Bullet | Fly::Bolt => {
                    let ammo = (s.ammo > 0).then(|| &crate::guns::AMMO[(s.ammo - 1) as usize]);
                    let trick = ammo.map_or(crate::guns::Trick::None, |a| a.trick);
                    let flame = d.item == "flamethrower";
                    if trick == crate::guns::Trick::Homing && s.age > 0.04 {
                        // Verdite bullets bend towards the nearest monster ahead of them.
                        let ahead = s.vel.normalize_or_zero();
                        let best = (0..self.mobs.list.len())
                            .filter(|&k| self.mobs.list[k].alive() && self.is_hostile(k) && !s.hits.iter().any(|h| h.0 == k))
                            .map(|k| (k, self.mobs.list[k].body.center() - s.pos))
                            .filter(|(_, to)| to.length() < 16.0 && to.normalize_or_zero().dot(ahead) > 0.55)
                            .min_by(|a, b| a.1.length().total_cmp(&b.1.length()));
                        if let Some((_, to)) = best {
                            let speed = s.vel.length();
                            s.vel = (ahead + to.normalize_or_zero() * dt * 9.0).normalize_or_zero() * speed;
                        }
                    }
                    let before = s.pos;
                    // Fast: move in short steps so nothing is skipped.
                    let steps = ((s.vel.length() * dt) / 0.5).ceil().max(1.0) as usize;
                    let step = s.vel * dt / steps as f32;
                    for _ in 0..steps {
                        s.pos += step;
                        let cell = s.pos.floor().as_ivec3();
                        if self.world.solid(cell) {
                            alive = false;
                            self.particles.glow(s.pos - step, -step.normalize_or_zero() * 2.0, glow, s.tint, 0.06, 0.2);
                            if s.fly == Fly::Bullet && !flame {
                                impacts.push((s.pos - step, cell, step.normalize_or_zero()));
                            }
                            if trick == crate::guns::Trick::Pop {
                                pops.push((s.pos - step, s.dmg * 0.6));
                            }
                            break;
                        }
                        if !alive {
                            break;
                        }
                        for k in self.mobs_touching(s.pos, 0.15) {
                            if s.hits.iter().any(|h| h.0 == k) {
                                continue;
                            }
                            let debuff = ammo.and_then(|a| a.debuff).or(d.debuff);
                            self.tr_hit(k, s.dmg, s.class, s.pos - step * 4.0, debuff);
                            s.hits.push((k, 10.0));
                            match trick {
                                crate::guns::Trick::Split => {
                                    // Three shards fly on out of the monster.
                                    for n in 0..3 {
                                        let a = n as f32 * 2.1 + s.age * 13.0;
                                        let side = s.vel.cross(Vec3::Y).normalize_or(Vec3::X);
                                        let v = (s.vel.normalize_or_zero() + side * a.cos() * 0.5 + Vec3::Y * a.sin() * 0.5).normalize_or_zero() * s.vel.length() * 0.6;
                                        shards.push(TrShot { fly: Fly::Bullet, def: s.def, pos: s.pos, vel: v, age: 0.0, life: 0.3, dmg: s.dmg * 0.3, class: s.class, pierce: 0, hits: s.hits.clone(), back: false, tint: [1.4, 0.8, 2.0], shot_t: 0.0, off: Vec3::ZERO, ammo: 0 });
                                    }
                                }
                                crate::guns::Trick::Pop => pops.push((s.pos, s.dmg * 0.6)),
                                crate::guns::Trick::Party => {
                                    let m = &mut self.mobs.list[k];
                                    m.body.knockback(s.vel.normalize_or_zero(), 6.0);
                                    for n in 0..10 {
                                        let c = [[2.2, 0.6, 0.6], [0.6, 2.0, 0.7], [0.6, 0.9, 2.2], [2.2, 2.0, 0.5]][n % 4];
                                        let v = Vec3::new(((n * 7) % 5) as f32 - 2.0, 2.0 + (n % 3) as f32, ((n * 3) % 5) as f32 - 2.0);
                                        self.particles.glow(s.pos, v, glow, c, 0.045, 0.7);
                                    }
                                }
                                crate::guns::Trick::Midas => {
                                    if !self.mobs.list[k].alive() {
                                        coins.push((self.mobs.list[k].body.center(), self.mobs.list[k].def));
                                    } else {
                                        // Gold dust on what it hit.
                                        self.particles.glow(s.pos, Vec3::Y, glow, [2.2, 1.8, 0.5], 0.05, 0.5);
                                    }
                                }
                                _ => {}
                            }
                            if s.pierce == 0 {
                                alive = false;
                                break;
                            }
                            s.pierce -= 1;
                        }
                        if !alive {
                            break;
                        }
                    }
                    if flame {
                        // Gel flames: they grow, lift and turn to smoke as they fly.
                        let k = (s.age / s.life).clamp(0.0, 1.0);
                        let at = s.pos + s.off * (1.0 - s.age / 0.08).max(0.0);
                        self.particles.flame(at, s.vel * 0.25, glow, 0.08 + 0.22 * k, 0.25, false);
                        if k > 0.6 && (s.age * 60.0) as u32 % 3 == 0 {
                            let stone = self.block_look(self.reg.id_or_air("stone")).0;
                            self.particles.mote(at, stone, [0.25, 0.24, 0.22], 0.12, -0.6, 0.9);
                        }
                    } else if s.fly == Fly::Bullet {
                        // A tracer: a bright head and a streak back along the way it came, from
                        // the gun's muzzle at first, joining the line of the shot.
                        let off = |age: f32| s.off * (1.0 - age / 0.06).max(0.0);
                        let head = s.pos + off(s.age);
                        let tail = before + off((s.age - dt).max(0.0));
                        let n = 5;
                        for j in 0..n {
                            let t = j as f32 / n as f32;
                            let p = head.lerp(tail, t);
                            self.particles.glow(p, Vec3::ZERO, glow, s.tint, 0.032 * (1.0 - t * 0.6), 0.035);
                        }
                        self.particles.glow(head, Vec3::ZERO, glow, s.tint, 0.05, 0.03);
                    } else {
                        self.particles.glow(s.pos, Vec3::ZERO, glow, s.tint, 0.16, 0.06);
                        self.particles.glow(s.pos - s.vel * 0.02, Vec3::ZERO, glow, s.tint, 0.11, 0.15);
                    }
                }
                Fly::Thrown => {
                    let Wk::Throw(g) = d.kind else { continue };
                    s.vel.y -= g * dt;
                    s.pos += s.vel * dt;
                    if self.world.solid(s.pos.floor().as_ivec3()) {
                        alive = false;
                    }
                    for k in self.mobs_touching(s.pos, 0.25) {
                        if s.hits.iter().any(|h| h.0 == k) {
                            continue;
                        }
                        self.tr_hit(k, s.dmg, s.class, s.pos - s.vel * 0.05, d.debuff);
                        s.hits.push((k, 10.0));
                        if s.pierce == 0 {
                            alive = false;
                            break;
                        }
                        s.pierce -= 1;
                    }
                }
                Fly::Bomb if matches!(d.kind, Wk::Launcher(_, true)) => {
                    // A rocket: straight on, a smoke trail, it bursts on whatever it meets.
                    let next = s.pos + s.vel * dt;
                    if self.world.solid(next.floor().as_ivec3()) || !self.mobs_touching(next, 0.3).is_empty() {
                        alive = false;
                    } else {
                        s.pos = next;
                    }
                    self.particles.glow(s.pos - s.vel.normalize_or_zero() * 0.4, -s.vel * 0.05, glow, [2.0, 1.0, 0.4], 0.07, 0.25);
                    self.particles.glow(s.pos - s.vel.normalize_or_zero() * 0.8, Vec3::Y * 0.4, glow, [0.5, 0.5, 0.55], 0.12, 0.8);
                    if !alive {
                        blasts.push((s.pos, s.def));
                    }
                }
                Fly::Bomb => {
                    s.vel.y -= 18.0 * dt;
                    let next = s.pos + s.vel * dt;
                    if matches!(d.kind, Wk::Launcher(..)) && !self.mobs_touching(next, 0.3).is_empty() {
                        alive = false;
                    }
                    if self.world.solid(next.floor().as_ivec3()) {
                        // Bounce a little, then settle.
                        if self.world.solid(IVec3::new(next.x.floor() as i32, s.pos.y.floor() as i32, next.z.floor() as i32)) {
                            s.vel.x *= -0.3;
                            s.vel.z *= -0.3;
                        }
                        s.vel.y *= -0.3;
                        s.vel *= 0.7;
                    } else {
                        s.pos = next;
                    }
                    if (s.age * 8.0) as i32 % 2 == 0 {
                        self.particles.glow(s.pos + Vec3::Y * 0.35, Vec3::Y * 0.5, glow, [2.0, 1.2, 0.4], 0.05, 0.2);
                    }
                    if !alive {
                        blasts.push((s.pos, s.def));
                    }
                }
                Fly::Sentry => {
                    s.shot_t -= dt;
                    if s.shot_t <= 0.0 {
                        s.shot_t = d.cd;
                        let eye = s.pos + Vec3::Y * 0.6;
                        if let Some(k) = self.nearest_hostile(eye, 22.0) {
                            let to = self.mobs.list[k].body.center() - eye;
                            let dmg = d.dmg;
                            keep.push(TrShot { fly: Fly::Bolt, def: s.def, pos: eye, vel: to.normalize_or_zero() * d.speed, age: 0.0, life: 1.0, dmg, class: SUMMON, pierce: 0, hits: Vec::new(), back: false, tint: if d.item == "lightning_aura_rod" { [0.8, 1.4, 2.4] } else { s.tint }, shot_t: 0.0, off: Vec3::ZERO, ammo: 0 });
                            self.sfx(Sfx::Place(Material::Cloth));
                        }
                    }
                }
            }
            // Melee throwables hit everything they pass, each mob twice a second at most.
            if matches!(s.fly, Fly::Yoyo | Fly::Boomerang | Fly::Flail) {
                let r = if s.fly == Fly::Flail { 0.5 } else { 0.35 };
                for k in self.mobs_touching(s.pos, r) {
                    if s.hits.iter().any(|h| h.0 == k) {
                        continue;
                    }
                    self.tr_hit(k, s.dmg, s.class, s.pos - s.vel.normalize_or_zero(), d.debuff);
                    s.hits.push((k, if s.fly == Fly::Yoyo { 0.3 } else { 0.5 }));
                    if s.fly == Fly::Flail {
                        s.back = true;
                    }
                }
                if d.item == "enchanted_boomerang" || d.debuff.is_some() {
                    self.particles.glow(s.pos, Vec3::ZERO, glow, s.tint, 0.08, 0.15);
                }
            }
            if alive {
                keep.push(s);
            }
        }
        // Whatever was added during the update (none today) goes after the ones kept.
        keep.append(&mut shots);
        keep.append(&mut shards);
        self.t5.tr.wep.shots = keep;
        for (at, def) in blasts {
            self.tr_blast(at, def);
        }
        for (at, cell, dir) in impacts {
            self.bullet_impact(at, cell, dir);
        }
        for (at, dmg) in pops {
            self.bullet_pop(at, dmg);
        }
        for (at, def) in coins {
            if self.survival {
                let v = self.mob_coins(def) / 2;
                self.drop_coins(at, v);
            }
        }
    }

    /// A bullet striking a block: chips of it fly off, a spark, now and then a knock.
    fn bullet_impact(&mut self, at: Vec3, cell: IVec3, dir: Vec3) {
        let wp = &mut self.t5.tr.wep;
        wp.impacts = wp.impacts.wrapping_add(1);
        let n = wp.impacts;
        let id = self.world.block_or_air(cell);
        let (layer, tint) = self.block_look(id);
        if n % 2 == 0 {
            self.particles.block_bits(cell, layer, tint, 3, 0.5);
        }
        let stone = self.block_look(self.reg.id_or_air("stone")).0;
        self.particles.mote(at - dir * 0.05, stone, [0.6, 0.58, 0.55], 0.06, -0.15, 0.7);
        let glow = self.glow_layer();
        self.particles.spark(at - dir * 0.05, -dir * 2.0, glow);
        if n % 3 == 0 && (at - self.body.eye()).length() < 24.0 {
            let mat = crate::audio::Material::of(&self.reg.get(id).name);
            self.sfx(Sfx::Hit(mat));
        }
    }

    /// An explosive bullet's small blast: hurts the monsters near, breaks nothing.
    fn bullet_pop(&mut self, at: Vec3, dmg: f32) {
        let r = 1.8;
        let hit: Vec<(usize, f32)> = (0..self.mobs.list.len())
            .filter(|&k| self.mobs.list[k].alive() && self.is_hostile(k))
            .map(|k| (k, self.mobs.list[k].body.center().distance(at)))
            .filter(|&(_, dist)| dist < r)
            .collect();
        for (k, dist) in hit {
            self.tr_hit(k, dmg * (1.0 - dist / r * 0.5), RANGED, at, None);
        }
        let glow = self.glow_layer();
        for _ in 0..12 {
            let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.3, self.surv.rng.f32() - 0.5) * 6.0;
            self.particles.glow(at, v, glow, [2.2, 1.1, 0.4], 0.07, 0.3);
        }
        if (at - self.body.eye()).length() < 32.0 {
            self.sfx(Sfx::Fx(Fx::Gun(crate::guns::Shot::Pop as u8)));
        }
    }

    fn tr_blast(&mut self, at: Vec3, def: usize) {
        let d = &WEAPONS[def];
        let (power, breaks) = match d.kind {
            Wk::Bomb(power, breaks, _) => (power, breaks),
            Wk::Launcher(power, _) => (power, false),
            _ => return,
        };
        if breaks {
            self.explode(at, power);
            return;
        }
        let r = power * 1.6;
        let hit: Vec<(usize, f32)> = (0..self.mobs.list.len())
            .filter(|&k| self.mobs.list[k].alive() && self.is_hostile(k))
            .map(|k| (k, self.mobs.list[k].body.center().distance(at)))
            .filter(|&(_, dist)| dist < r)
            .collect();
        for (k, dist) in hit {
            self.tr_hit(k, d.dmg * (1.0 - dist / r * 0.6), RANGED, at, None);
        }
        let glow = self.glow_layer();
        for _ in 0..40 {
            let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.3, self.surv.rng.f32() - 0.5) * 10.0;
            self.particles.glow(at, v, glow, [2.0, 1.1, 0.4], 0.14, 0.5);
        }
        self.sfx(Sfx::Explode);
    }

    /// The flying weapons as items for the renderer.
    pub(crate) fn tr_weapon_instances(&self, cam_block: IVec3) -> Vec<GpuItem> {
        let shots = &self.t5.tr.wep.shots;
        if shots.is_empty() {
            return Vec::new();
        }
        let c = cam_block.as_vec3();
        let mut out = Vec::new();
        for s in shots {
            // Sentries have their own 3D models (tr_sentry_models).
            if matches!(s.fly, Fly::Bullet | Fly::Bolt) || (s.fly == Fly::Sentry && crate::summons::rod_model(WEAPONS[s.def].item).is_some()) {
                continue;
            }
            let Some(id) = self.items.id(WEAPONS[s.def].item) else { continue };
            let voxel::item::Look::Sprite(l) = self.items.get(id).look else { continue };
            let layer = (self.item_base + l) as f32;
            let l = self.world.light(s.pos.floor().as_ivec3());
            let (sky, blk) = ((l >> 4) as f32 / 15.0, (l & 15) as f32 / 15.0);
            let p = s.pos - c;
            let (size, yaw, pitch, roll) = match s.fly {
                Fly::Yoyo | Fly::Boomerang | Fly::Thrown => (0.5, 0.0, std::f32::consts::FRAC_PI_2, s.age * 25.0),
                Fly::Flail => (0.6, s.age * 12.0, 0.0, 0.0),
                Fly::Sentry => (1.2, 0.0, 0.0, 0.0),
                _ => (0.45, s.age * 6.0, 0.0, 0.0),
            };
            // Extruded like a held item, so a spinning boomerang has its thickness.
            out.push(GpuItem { pos: [p.x, p.y, p.z, size], info: [yaw, 2.0, sky, blk], layers: [layer, layer, layer, 0.0], rot: [pitch, roll, 0.0, 0.0], ..bytemuck::Zeroable::zeroed() });
        }
        out
    }

    /// Sentries the rods have set down.
    pub(crate) fn tr_sentry_count(&self) -> usize {
        self.t5.tr.wep.shots.iter().filter(|s| s.fly == Fly::Sentry).count()
    }

    pub(crate) fn tr_oldest_sentry_age(&self) -> Option<f32> {
        self.t5.tr.wep.shots.iter().filter(|s| s.fly == Fly::Sentry).map(|s| s.age).max_by(f32::total_cmp)
    }

    pub(crate) fn tr_remove_oldest_sentry(&mut self) {
        let shots = &mut self.t5.tr.wep.shots;
        if let Some(k) = (0..shots.len()).filter(|&k| shots[k].fly == Fly::Sentry).max_by(|&a, &b| shots[a].age.total_cmp(&shots[b].age)) {
            shots.remove(k);
        }
    }

    /// The sentries the rods set down, as 3D models (summons.rs) turned to the nearest monster.
    pub(crate) fn tr_sentry_models(&self, cam_block: IVec3) -> Vec<(usize, crate::entity::Bones)> {
        let c = cam_block.as_vec3();
        let mut out = Vec::new();
        for s in self.t5.tr.wep.shots.iter().filter(|s| s.fly == Fly::Sentry) {
            let d = &WEAPONS[s.def];
            let Some(model) = crate::summons::rod_model(d.item) else { continue };
            let Some(mesh) = self.entities.summon_mesh(model) else { continue };
            // Its foot: the shot sits a little over the ground.
            let foot = s.pos - Vec3::Y * 0.6;
            let eye = s.pos;
            let aim = self.nearest_hostile(eye, 22.0).map(|k| self.mobs.list[k].body.center() - eye).unwrap_or(Vec3::ZERO);
            let l = self.world.light((foot + Vec3::Y * 0.5).floor().as_ivec3());
            let pose = crate::summons::SentryPose {
                pos: foot - c,
                aim,
                t: self.now + foot.x * 0.37,
                fire: if aim == Vec3::ZERO { 0.0 } else { (1.0 - (d.cd - s.shot_t) / 0.6).clamp(0.0, 1.0) },
                light: [(l >> 4) as f32 / 15.0, ((l & 15) as f32 / 15.0).max(0.4), 0.0, 0.0],
            };
            out.push((mesh, crate::summons::sentry_bones(model, &pose)));
        }
        out
    }
}

impl Game {
    /// Test hooks for the held gun, for screenshots: MC_GUN_AIM=0..1 raises it, MC_GUN_FIRE=n
    /// fires n shots and lets them fly a moment (tracers, flash, smoke, cases), MC_GUN_KICK=0..1
    /// freezes it that far into its kick.
    pub(crate) fn test_gun(&mut self) {
        let Some(g) = self.held_gun() else { return };
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
        self.anim.hand_height = 1.0;
        if let Some(a) = num("MC_GUN_AIM") {
            self.t5.tr.wep.gun.aim = a;
        }
        if let Some(n) = num("MC_GUN_FIRE") {
            let eye = self.body.eye();
            let block = eye.floor().as_ivec3();
            if self.camera_mode == 0 {
                let st = crate::guns::posed(self.t5.tr.wep.gun);
                let hold = self.anim.first_person_gun(g, &st, eye - block.as_vec3(), self.yaw, self.pitch);
                self.t5.tr.wep.muzzle = Some(hold.muzzle + block.as_vec3());
            }
            let Some(i) = def_of(crate::guns::GUNS[g].item) else { return };
            for _ in 0..n as usize {
                self.fire_weapon(i);
                self.update_tr_shots(0.02);
                self.particles.update(&self.world, 0.02);
            }
            // The shots move on a little after the last one, so their tracers show.
            self.update_tr_shots(0.012);
        }
    }

    /// Test scene (MC_TR=weapons): a yoyo out, a boomerang and a flail in the air, bullets,
    /// a bomb about to go off and a sentry, with a few monsters to aim at.
    pub(crate) fn test_tr_weapons(&mut self) {
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let side = flat.cross(Vec3::Y);
        let eye = self.body.eye();
        let ground = |g: &Game, p: Vec3| g.world.surface_y(p.x.floor() as i32, p.z.floor() as i32).map(|y| y as f32 + 1.0).unwrap_or(p.y);
        for (k, n) in ["zombie", "skeleton", "zombie"].iter().enumerate() {
            let p = self.body.pos + flat * 11.0 + side * (k as f32 - 1.0) * 3.0;
            self.mobs.spawn(n, Vec3::new(p.x, ground(self, p), p.z));
        }
        let shots: [(&str, Fly, Vec3, Vec3, f32); 6] = [
            ("magma_yoyo", Fly::Yoyo, eye + flat * 4.0 - side * 1.5, Vec3::ZERO, 999.0),
            ("enchanted_boomerang", Fly::Boomerang, eye + flat * 6.0 + side * 2.0 + Vec3::Y * 0.5, flat * 20.0, 99.0),
            ("sun_flail", Fly::Flail, eye + flat * 3.5 + side * 1.2 - Vec3::Y * 0.3, flat * 20.0, 99.0),
            ("bomb", Fly::Bomb, eye + flat * 5.0 - Vec3::Y * 1.2 - side * 3.0, Vec3::ZERO, 99.0),
            ("ballista_rod", Fly::Sentry, eye + flat * 4.0 + side * 3.5 - Vec3::Y * 1.0, Vec3::ZERO, 99.0),
            ("flintlock_pistol", Fly::Bullet, eye + flat * 2.0, flat * 0.01, 99.0),
        ];
        for (item, fly, pos, vel, life) in shots {
            let Some(i) = def_of(item) else { continue };
            self.add_shot(fly, i, pos, vel, life, MELEE);
        }
        if let Some(id) = self.items.id("magma_yoyo") {
            self.surv.inv.slots[self.selected] = Some(voxel::item::ItemStack::new(id, 1));
        }
    }
}
