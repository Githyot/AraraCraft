//! Fight animations: how the player's attacks move, in first and third person.
//!
//! Every attack is a short story in four keys, the way animators block a hit:
//! rest -> **wind-up** (the weapon cocks back, the body coils) -> **impact** (a fast snap through
//! the target) -> **follow-through** (the weapon carries on past it) -> back to rest. The
//! wind-up is short so the hit still feels instant, the snap into the impact eases in (it
//! accelerates), and everything after it eases out. A hit that connects freezes the swing for a
//! few hundredths of a second at the impact ("hit-stop"), so blows feel like they land.
//!
//! Each weapon class has its own moves (`Arms`): melee weapons cycle their combo
//! (mods/combat.rs picks the style of each attack), guns kick back, staffs and spell books
//! thrust a palm forward, thrown weapons and yoyos are flung overhand, whips crack.
//! The swing lasts as long as the weapon's own swing time (`swing_time`).

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Quat, Vec3};

/// The attack styles (`Animator::swing_style`).
pub mod style {
    /// From above, straight down (axes, plain swings).
    pub const CHOP: u8 = 0;
    /// Across from the right to the left.
    pub const SLASH: u8 = 1;
    /// Straight ahead, point first.
    pub const THRUST: u8 = 2;
    /// From below, up past the face.
    pub const UPPERCUT: u8 = 3;
    /// Back across from the left to the right (the second hit of a combo).
    pub const BACKHAND: u8 = 4;
    /// A big two-handed blow from over the head (finishers).
    pub const HEAVY: u8 = 5;
    /// A gun going off: the kick of the recoil.
    pub const SHOOT: u8 = 6;
    /// A spell: the hand draws in and pushes forward.
    pub const CAST: u8 = 7;
    /// Thrown overhand (boomerangs, flails, knives, bombs).
    pub const THROW: u8 = 8;
    /// A whip: high back, then a snap.
    pub const CRACK: u8 = 9;
    /// A yoyo flicked out from the wrist.
    pub const FLICK: u8 = 10;
    pub const COUNT: u8 = 11;
}

/// What kind of weapon is in the hand, for its moves.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Arms {
    #[default]
    Melee,
    Gun,
    Caster,
    Thrower,
    Yoyo,
    Whip,
}

/// The kind of moves for an item by its name.
pub fn arms_of(name: &str) -> Arms {
    if crate::tr_weapons::is_gun(name) {
        Arms::Gun
    } else if crate::rpg::is_whip(name) {
        Arms::Whip
    } else if crate::tr_weapons::is_yoyo(name) {
        Arms::Yoyo
    } else if crate::tr_weapons::is_thrown(name) {
        Arms::Thrower
    } else if crate::magic::is_caster(name) || name.ends_with("_staff") || name.ends_with("_wand") || name.ends_with("_rod") && !name.contains("fishing") {
        Arms::Caster
    } else {
        Arms::Melee
    }
}

/// The style a plain swing turns into for this kind of weapon.
pub fn plain_style(arms: Arms) -> u8 {
    match arms {
        Arms::Melee => style::CHOP,
        Arms::Gun => style::SHOOT,
        Arms::Caster => style::CAST,
        Arms::Thrower => style::THROW,
        Arms::Yoyo => style::FLICK,
        Arms::Whip => style::CRACK,
    }
}

/// When the keys fall in a swing (0..1): wind-up reached, impact, follow-through reached.
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub wind: f32,
    pub hit: f32,
    pub follow: f32,
}

pub fn timing(s: u8) -> Timing {
    let t = |wind, hit, follow| Timing { wind, hit, follow };
    match s {
        style::THRUST => t(0.22, 0.36, 0.55),
        style::HEAVY => t(0.34, 0.48, 0.66),
        style::SHOOT => t(0.08, 0.14, 0.4),
        style::CAST => t(0.2, 0.34, 0.6),
        style::THROW => t(0.28, 0.42, 0.62),
        style::CRACK => t(0.3, 0.4, 0.58),
        style::FLICK => t(0.18, 0.3, 0.55),
        _ => t(0.2, 0.36, 0.56),
    }
}

/// How long a swing of this style takes when the weapon has no swing time of its own.
pub const PLAIN_TIME: f32 = 0.28;

/// The swing time for a weapon that recovers in `cooldown` seconds: heavy weapons swing slower.
pub fn swing_time(cooldown: f32) -> f32 {
    (cooldown * 0.55).clamp(PLAIN_TIME, 0.62)
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x) * (1.0 - x)
}

fn ease_in(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x
}

/// Which two keys the swing is between at `u` and how far: 0 rest, 1 wind-up, 2 impact,
/// 3 follow-through, 4 rest again. The snap into the impact speeds up; the rest eases.
pub fn segment(s: u8, u: f32) -> (usize, usize, f32) {
    let tm = timing(s);
    if u <= 0.0 || u >= 1.0 {
        return (0, 0, 0.0);
    }
    if u < tm.wind {
        (0, 1, ease_out(u / tm.wind))
    } else if u < tm.hit {
        (1, 2, ease_in((u - tm.wind) / (tm.hit - tm.wind).max(1e-3)))
    } else if u < tm.follow {
        (2, 3, ease_out((u - tm.hit) / (tm.follow - tm.hit).max(1e-3)))
    } else {
        (3, 4, smooth((u - tm.follow) / (1.0 - tm.follow)))
    }
}

/// How much the swing is "on" (0 at rest, 1 between wind-up and follow-through), to blend
/// other poses out of the way.
pub fn weight(s: u8, u: f32) -> f32 {
    let (a, b, t) = segment(s, u);
    match (a, b) {
        (0, 0) => 0.0,
        (0, _) => t,
        (3, 4) => 1.0 - t,
        _ => 1.0,
    }
}

/// The swing has reached its impact.
pub fn past_impact(s: u8, u: f32) -> bool {
    u >= timing(s).hit
}

// ---------------------------------------------------------------------------------------------
// Third person.

/// A pose of the body for one key (model space: rotations of the right arm from hanging down;
/// negative X raises it forward, positive Y swings it across to the body's left).
#[derive(Clone, Copy)]
pub struct Key3 {
    pub arm: Quat,
    /// Elbow: negative folds the forearm up.
    pub elbow: f32,
    /// The chest turning (positive towards the left) and leaning forward.
    pub twist: f32,
    pub lean: f32,
    /// The left foot forward (positive) into the blow.
    pub step: f32,
    /// Both hands on the weapon: the left arm's pose.
    pub left: Option<(Quat, f32)>,
}

fn k3(x: f32, y: f32, z: f32, elbow: f32, twist: f32, lean: f32, step: f32) -> Key3 {
    Key3 { arm: Quat::from_rotation_y(y) * Quat::from_rotation_x(x) * Quat::from_rotation_z(z), elbow, twist, lean, step, left: None }
}

/// The wind-up, impact and follow-through of a style in third person. `pitch` is where the
/// player looks (up is negative), for the aimed styles.
pub fn keys3(s: u8, pitch: f32) -> [Key3; 3] {
    let aim = -FRAC_PI_2 - pitch.clamp(-1.2, 1.2);
    match s {
        style::SLASH => [
            k3(-1.35, -1.15, 0.2, -0.55, -0.45, 0.0, 0.1),
            k3(-1.5, 0.05, 0.1, -0.08, 0.12, 0.12, 0.35),
            k3(-1.15, 1.05, -0.1, -0.45, 0.5, 0.1, 0.3),
        ],
        style::BACKHAND => [
            k3(-1.4, 1.1, -0.25, -1.3, 0.5, 0.0, 0.05),
            k3(-1.5, 0.0, 0.05, -0.1, -0.1, 0.1, 0.3),
            k3(-1.1, -1.15, 0.25, -0.35, -0.42, 0.08, 0.25),
        ],
        style::THRUST => [
            k3(-0.85, -0.2, 0.15, -1.6, -0.4, -0.05, 0.0),
            k3(-1.62, 0.1, 0.0, 0.0, 0.25, 0.16, 0.5),
            k3(-1.55, 0.08, 0.0, -0.12, 0.2, 0.12, 0.45),
        ],
        style::UPPERCUT => [
            k3(0.45, -0.25, 0.25, -0.6, -0.3, 0.22, 0.15),
            k3(-2.0, 0.15, 0.1, -0.2, 0.12, -0.05, 0.3),
            k3(-2.75, 0.2, 0.05, -0.35, 0.18, -0.15, 0.2),
        ],
        style::HEAVY => {
            let mut k = [
                k3(-2.95, -0.1, 0.1, -1.5, -0.25, -0.18, 0.0),
                k3(-1.15, 0.05, 0.0, -0.05, 0.08, 0.3, 0.55),
                k3(-0.55, 0.1, 0.0, -0.2, 0.12, 0.38, 0.5),
            ];
            // Both hands on the hilt: the left arm follows the right, turned in to meet it.
            for key in k.iter_mut() {
                key.left = Some((Quat::from_rotation_y(-0.45) * key.arm * Quat::from_rotation_z(-0.2), key.elbow - 0.2));
            }
            k
        }
        style::SHOOT => [
            k3(aim, -0.05, 0.0, 0.0, -0.12, 0.0, 0.0),
            k3(aim - 0.4, -0.05, 0.0, -0.45, -0.12, -0.08, 0.0),
            k3(aim - 0.1, -0.05, 0.0, -0.1, -0.12, -0.02, 0.0),
        ],
        style::CAST => [
            k3(-0.75, 0.25, 0.1, -1.7, -0.25, -0.06, 0.0),
            k3(aim - 0.1, 0.05, 0.0, 0.0, 0.18, 0.12, 0.3),
            k3(aim - 0.05, 0.05, 0.0, -0.08, 0.15, 0.1, 0.28),
        ],
        style::THROW => [
            k3(-2.65, -0.55, 0.3, -1.6, -0.55, -0.12, 0.0),
            k3(aim - 0.15, 0.05, 0.0, 0.0, 0.3, 0.16, 0.45),
            k3(-0.65, 0.45, -0.1, -0.3, 0.4, 0.25, 0.4),
        ],
        style::CRACK => [
            k3(-3.0, -0.3, 0.2, -0.9, -0.3, -0.15, 0.0),
            k3(-1.5, 0.05, 0.0, -0.05, 0.15, 0.15, 0.35),
            k3(-0.85, 0.2, 0.0, -0.4, 0.2, 0.2, 0.3),
        ],
        style::FLICK => [
            k3(-1.0, 0.1, 0.1, -1.4, -0.15, 0.0, 0.0),
            k3(aim, 0.0, 0.0, 0.0, 0.1, 0.08, 0.2),
            k3(aim + 0.05, 0.0, 0.0, -0.1, 0.1, 0.06, 0.2),
        ],
        // CHOP: over the head, down through the target, on down past it.
        _ => [
            k3(-2.85, -0.25, 0.2, -1.4, -0.25, -0.12, 0.0),
            k3(-1.3, 0.05, 0.05, -0.08, 0.1, 0.2, 0.4),
            k3(-0.6, 0.15, 0.0, -0.25, 0.18, 0.26, 0.35),
        ],
    }
}

/// The body in third person at swing progress `u`, blended from `rest` (the arm and elbow the
/// rest of the animation chose this frame).
pub fn pose3(s: u8, u: f32, pitch: f32, rest_arm: Quat, rest_elbow: f32, rest_left: (Quat, f32)) -> Key3 {
    let keys = keys3(s, pitch);
    let rest = Key3 { arm: rest_arm, elbow: rest_elbow, twist: 0.0, lean: 0.0, step: 0.0, left: Some(rest_left) };
    let at = |i: usize| if i == 0 || i == 4 { rest } else { keys[i - 1] };
    let (a, b, t) = segment(s, u);
    let (ka, kb) = (at(a), at(b));
    let left = match (ka.left, kb.left) {
        (None, None) => None,
        (l0, l1) => {
            let (q0, e0) = l0.unwrap_or(rest_left);
            let (q1, e1) = l1.unwrap_or(rest_left);
            Some((q0.slerp(q1, t), e0 + (e1 - e0) * t))
        }
    };
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    Key3 {
        arm: ka.arm.slerp(kb.arm, t),
        elbow: lerp(ka.elbow, kb.elbow),
        twist: lerp(ka.twist, kb.twist),
        lean: lerp(ka.lean, kb.lean),
        step: lerp(ka.step, kb.step),
        left,
    }
}

// ---------------------------------------------------------------------------------------------
// First person.

/// A key in first person: how far the fist moves (camera space, x right, y up, z towards the
/// eye; before `HAND_REACH`) and how the item turns around it.
#[derive(Clone, Copy)]
pub struct Key1 {
    pub shift: Vec3,
    pub turn: Quat,
}

fn k1(x: f32, y: f32, z: f32, turn: Quat) -> Key1 {
    Key1 { shift: Vec3::new(x, y, z), turn }
}

fn rx(a: f32) -> Quat {
    Quat::from_rotation_x(a)
}
fn ry(a: f32) -> Quat {
    Quat::from_rotation_y(a)
}
fn rz(a: f32) -> Quat {
    Quat::from_rotation_z(a)
}

/// The wind-up, impact and follow-through of a style in first person.
pub fn keys1(s: u8) -> [Key1; 3] {
    match s {
        // The blade swings back out to the right, lies down and sweeps across the view.
        style::SLASH => [
            k1(0.2, 0.06, 0.04, ry(-0.45) * rz(-0.35)),
            k1(-0.14, 0.08, -0.14, ry(0.3) * rz(1.25)),
            k1(-0.46, 0.0, -0.05, ry(0.6) * rz(1.45)),
        ],
        // Back the other way: cocked across on the left, sweeping out to the right.
        style::BACKHAND => [
            k1(-0.36, 0.1, 0.02, rz(1.45) * rx(-0.2)),
            k1(-0.06, 0.08, -0.16, rz(0.05) * rx(-0.75)),
            k1(0.24, -0.02, -0.06, rz(-1.15) * rx(-0.45)),
        ],
        style::THRUST => [
            k1(0.06, -0.04, 0.16, rx(-0.55)),
            k1(-0.2, 0.12, -0.45, rx(-0.95) * ry(0.3)),
            k1(-0.2, 0.1, -0.38, rx(-0.92) * ry(0.28)),
        ],
        style::UPPERCUT => [
            k1(0.02, -0.3, 0.06, rx(0.75) * rz(0.15)),
            k1(-0.18, 0.15, -0.16, rx(-0.5)),
            k1(-0.2, 0.3, -0.06, rx(-0.95)),
        ],
        style::HEAVY => [
            k1(0.08, 0.26, 0.12, rx(0.7) * rz(-0.35)),
            k1(-0.16, -0.18, -0.2, rz(0.75) * rx(-0.95)),
            k1(-0.2, -0.36, -0.12, rz(0.85) * rx(-1.15)),
        ],
        style::SHOOT => [k1(0.0, 0.0, 0.0, Quat::IDENTITY), k1(0.0, 0.06, 0.14, rx(0.38)), k1(0.0, 0.02, 0.04, rx(0.1))],
        style::CAST => [k1(0.04, -0.06, 0.12, rx(0.25)), k1(-0.14, 0.12, -0.32, rx(-0.5)), k1(-0.12, 0.1, -0.26, rx(-0.45))],
        style::THROW => [k1(0.12, 0.22, 0.16, rx(0.65)), k1(-0.1, 0.05, -0.32, rx(-0.6)), k1(-0.16, -0.26, -0.1, rx(-1.0))],
        style::CRACK => [k1(0.06, 0.3, 0.12, rx(0.9)), k1(-0.1, -0.04, -0.26, rx(-0.7)), k1(-0.12, -0.16, -0.2, rx(-0.5))],
        style::FLICK => [k1(0.0, 0.0, 0.1, rx(0.3)), k1(-0.1, 0.05, -0.3, rx(-0.4)), k1(-0.1, 0.04, -0.27, rx(-0.38))],
        // CHOP: the blade cocks up and back towards the eye, then chops down and across
        // towards the crosshair, in the view's plane more than into it so it stays in sight.
        _ => [
            k1(0.06, 0.1, 0.05, rz(-0.3) * rx(0.35)),
            k1(-0.2, -0.02, -0.14, rz(1.0) * rx(-0.55)),
            k1(-0.24, -0.13, -0.1, rz(1.15) * rx(-0.72)),
        ],
    }
}

/// The fist's shift and the item's turn in first person at swing progress `u`.
pub fn pose1(s: u8, u: f32) -> Key1 {
    let keys = keys1(s);
    let rest = k1(0.0, 0.0, 0.0, Quat::IDENTITY);
    let at = |i: usize| if i == 0 || i == 4 { rest } else { keys[i - 1] };
    let (a, b, t) = segment(s, u);
    let (ka, kb) = (at(a), at(b));
    Key1 { shift: ka.shift.lerp(kb.shift, t), turn: ka.turn.slerp(kb.turn, t) }
}

// ---------------------------------------------------------------------------------------------
// The game's side.

impl crate::Game {
    /// Each frame: the kind of weapon in hand and how long its swing takes.
    pub(crate) fn update_fight_anim(&mut self) {
        let name = self.held_stack().map(|s| self.items.get(s.item).name.clone());
        let arms = name.as_deref().map_or(Arms::Melee, arms_of);
        self.anim.arms = arms;
        let cooldown = self.weapon_cooldown().or_else(|| name.as_deref().and_then(crate::tr_weapons::use_time).filter(|_| arms != Arms::Gun));
        self.anim.swing_time = cooldown.map_or(PLAIN_TIME, swing_time);
    }

    /// The camera's up in first person: it rolls away from a hit for a moment (like the classic
    /// hurt tilt) and keels over when the player dies. Off with "reduce motion".
    pub(crate) fn view_tilt(&self, forward: Vec3) -> Vec3 {
        if self.settings.reduce_motion {
            return Vec3::Y;
        }
        let (jolt, push) = self.anim.flinch();
        // The push in the body's space: +x is the body's left. A hit from the right pushes left
        // and rolls the view that way.
        let side = if push.x.abs() > 0.2 { push.x.signum() } else { 1.0 };
        let dead = self.anim.dead_fall();
        let roll = side * 0.16 * jolt + dead * dead * 0.9;
        if roll.abs() < 1e-4 {
            return Vec3::Y;
        }
        let f = forward.normalize_or_zero();
        Quat::from_axis_angle(f, roll) * Vec3::Y
    }
}

// ---------------------------------------------------------------------------------------------
// Being hit.

/// How hard a flinch shows `age` seconds after the hit: a sharp jolt that settles with a
/// small rebound.
pub fn flinch(age: f32) -> f32 {
    const T: f32 = 0.4;
    if !(0.0..T).contains(&age) {
        return 0.0;
    }
    let x = age / T;
    // Up fast (first 12%), back down with a little overshoot.
    if x < 0.12 {
        ease_out(x / 0.12)
    } else {
        let y = (x - 0.12) / 0.88;
        (1.0 - y) * (1.0 - y) * (1.0 + 0.35 * (y * PI * 2.0).sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_swing_starts_and_ends_at_rest() {
        for s in 0..style::COUNT {
            let k = pose1(s, 0.0);
            assert!(k.shift.length() < 1e-4 && k.turn.angle_between(Quat::IDENTITY) < 1e-3, "style {s} start");
            let k = pose1(s, 0.999);
            assert!(k.shift.length() < 0.01, "style {s} end");
            assert_eq!(weight(s, 0.0), 0.0);
            assert!(weight(s, 0.999) < 0.01);
        }
    }

    #[test]
    fn keys_follow_each_other_smoothly() {
        // No jumps anywhere in a swing: finely sampled, every step is small.
        for s in 0..style::COUNT {
            let mut last = pose1(s, 0.0);
            let n = 400;
            for i in 1..=n {
                let u = i as f32 / n as f32 * 0.999;
                let k = pose1(s, u);
                let jump = (k.shift - last.shift).length();
                let turn = k.turn.angle_between(last.turn);
                assert!(jump < 0.03 && turn < 0.1, "style {s} jumps at {u}: {jump} {turn}");
                last = k;
            }
        }
    }

    #[test]
    fn the_wind_up_comes_before_the_impact() {
        for s in 0..style::COUNT {
            let t = timing(s);
            assert!(t.wind <= t.hit && t.hit < t.follow && t.follow < 1.0, "style {s}");
            // The snap into the impact is quicker than the follow-through.
            assert!(t.hit - t.wind <= t.follow - t.hit + 1e-3, "style {s}");
        }
    }

    #[test]
    fn a_chop_raises_the_arm_overhead_then_brings_it_down() {
        let up = keys3(style::CHOP, 0.0)[0].arm * Vec3::NEG_Y;
        let hit = keys3(style::CHOP, 0.0)[1].arm * Vec3::NEG_Y;
        assert!(up.y > 0.8, "arm overhead in the wind-up: {up}");
        assert!(hit.z > 0.8, "arm ahead at the impact: {hit}");
    }

    #[test]
    fn slash_and_backhand_go_opposite_ways() {
        let side = |s: u8, i: usize| (keys3(s, 0.0)[i].arm * Vec3::NEG_Y).x;
        assert!(side(style::SLASH, 0) < -0.4 && side(style::SLASH, 2) > 0.4);
        assert!(side(style::BACKHAND, 0) > 0.4 && side(style::BACKHAND, 2) < -0.4);
    }

    #[test]
    fn heavier_weapons_swing_slower() {
        assert!(swing_time(1.2) > swing_time(0.625));
        assert_eq!(swing_time(0.1), PLAIN_TIME);
    }

    #[test]
    fn a_flinch_jolts_then_settles() {
        assert_eq!(flinch(-0.1), 0.0);
        assert!(flinch(0.05) > 0.9);
        assert!(flinch(0.39) < 0.05);
        assert_eq!(flinch(1.0), 0.0);
    }

    #[test]
    fn weapons_get_their_own_moves() {
        assert_eq!(plain_style(arms_of("musket")), plain_style(Arms::Gun));
        assert_eq!(arms_of("leather_whip"), Arms::Whip);
        assert_eq!(arms_of("wood_yoyo"), Arms::Yoyo);
        assert_eq!(arms_of("diamond_sword"), Arms::Melee);
    }
}
