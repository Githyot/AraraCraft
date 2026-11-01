//! Bows that shoot on their own while the button is held, like Terraria's: each bow has its
//! own draw time, number of arrows per shot, spread and arrow speed, and each kind of arrow its
//! own bite (fire, frost, poison, a burst of flame, piercing stars, stars falling from above).
//! Arrows that land stay lodged where they hit, at the angle they flew: in a block until picked
//! up or gone, in a monster until it dies (`Lodged`).

use glam::{Quat, Vec3};
use voxel::mob::Mob;

/// How a bow shoots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BowStats {
    /// Seconds to draw a full shot; held down, it shoots again every this often.
    pub draw: f32,
    /// Arrows per shot (one used from the quiver), fanned out over `spread` radians.
    pub arrows: u8,
    pub spread: f32,
    /// Arrow speed, times the plain bow's.
    pub speed: f32,
    /// Its arrows come off burning.
    pub fire: bool,
    /// What it does to its arrows besides (the bows of plan/arcos-analise.md, lote B).
    pub trick: Trick,
}

/// A bow's own way with its arrows, after Terraria's bows, thought out for a 3D world.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trick {
    #[default]
    None,
    /// Plain arrows become three bees that seek monsters out.
    Bees,
    /// Plain arrows become two burning bats that seek monsters out.
    Bats,
    /// Every arrow flies as a bone shard that passes through three more monsters.
    Bone,
    /// Shoots shards of ice without using arrows: they chill and freeze what they hit.
    Ice,
    /// The arrows fall from the sky around the spot aimed at (a 3D circle, not a 2D column).
    Rain,
    /// Arrows fly straight and glance off walls three times.
    Pulse,
    /// Plain arrows become a phoenix of fire that burns through everything in its way.
    Phoenix,
    /// Each arrow splits in flight into three that seek monsters out.
    Split,
}

/// A bow's way of shooting (None: not a bow).
pub fn bow_stats(name: &str) -> Option<BowStats> {
    let s = |draw, arrows, spread, speed, fire| Some(BowStats { draw, arrows, spread, speed, fire, trick: Trick::None });
    let t = |draw, arrows, spread, speed, trick| Some(BowStats { draw, arrows, spread, speed, fire: false, trick });
    match name {
        // The early bows: a little quicker and faster with each metal.
        "bow" => s(0.8, 1, 0.0, 1.0, false),
        "copper_bow" => s(0.74, 1, 0.0, 1.05, false),
        "silver_bow" => s(0.68, 1, 0.0, 1.1, false),
        "platinum_bow" => s(0.62, 1, 0.0, 1.15, false),
        // The evil bows: quick draws.
        "umbral_bow" => s(0.5, 1, 0.0, 1.15, false),
        "sanguinite_bow" => s(0.55, 1, 0.0, 1.25, false),
        // Magmarite: every arrow catches fire.
        "magmarite_bow" => s(0.6, 1, 0.0, 1.3, true),
        // Cobalt: a repeater, very quick.
        "cobalt_bow" => s(0.36, 1, 0.0, 1.2, false),
        // Mythril: two arrows side by side.
        "mythril_bow" => s(0.45, 2, 0.06, 1.25, false),
        // Adamantite: the fastest arrows, heavy draw.
        "adamantite_bow" => s(0.5, 1, 0.0, 1.6, false),
        // Radiant: three in a fan.
        "radiant_bow" => s(0.5, 3, 0.14, 1.3, false),
        // Verdite: a shotbow, four spread wide.
        "verdite_bow" => s(0.62, 4, 0.26, 1.2, false),
        // Lumite: very quick doubles at great speed.
        "lumite_bow" => s(0.3, 2, 0.05, 1.7, false),
        // The bows with a trick of their own.
        "hive_bow" => t(0.62, 1, 0.0, 1.0, Trick::Bees),
        "batwing_bow" => t(0.6, 1, 0.0, 1.1, Trick::Bats),
        "marrow_bow" => t(0.55, 1, 0.0, 1.35, Trick::Bone),
        "frost_bow" => t(0.4, 1, 0.0, 1.2, Trick::Ice),
        "storm_bow" => t(0.55, 3, 0.0, 1.0, Trick::Rain),
        "pulse_bow" => t(0.45, 1, 0.0, 1.8, Trick::Pulse),
        "tide_bow" => t(0.75, 5, 0.38, 1.3, Trick::None),
        "phoenix_bow" => t(0.45, 1, 0.0, 1.5, Trick::Phoenix),
        "skysplit_bow" => t(0.5, 1, 0.0, 1.6, Trick::Split),
        _ => None,
    }
}

/// What a trick bow's shot becomes: an arrow of kind `arrow` (None for creative's endless
/// plain arrows) turned into `n` shots with this bite, picture (an item name; None keeps the
/// arrow's own), damage factor each and extra monsters passed through. None: unchanged.
pub struct Turned {
    pub n: u8,
    pub fx: ArrowFx,
    pub look: Option<&'static str>,
    pub dmg: f32,
    pub pierce: u8,
}

pub fn turn(trick: Trick, arrow: Option<&str>) -> Option<Turned> {
    let plain = matches!(arrow, None | Some("arrow"));
    let d = ArrowFx::default();
    Some(match trick {
        Trick::Bees if plain => Turned { n: 3, fx: ArrowFx { homing: 7.0, life: 2.5, vanish: true, orb: true, trail: Some([1.5, 1.2, 0.3]), ..d }, look: None, dmg: 0.42, pierce: 0 },
        Trick::Bats if plain => Turned { n: 2, fx: ArrowFx { homing: 5.0, life: 3.0, vanish: true, orb: true, burn: 3.0, straight: true, trail: Some([1.3, 0.35, 0.15]), ..d }, look: None, dmg: 0.6, pierce: 0 },
        Trick::Bone => Turned { n: 1, fx: ArrowFx { trail: Some([0.9, 0.85, 0.75]), ..d }, look: Some("bone_arrow"), dmg: 1.0, pierce: 3 },
        Trick::Ice => Turned { n: 1, fx: ArrowFx { freeze: 0.4, debuff: Some((crate::effects::CHILLED, 3.0)), straight: true, vanish: true, trail: Some([0.6, 1.0, 1.5]), ..d }, look: Some("frostburn_arrow"), dmg: 1.0, pierce: 0 },
        Trick::Pulse => Turned { n: 1, fx: ArrowFx { bounce: 3, straight: true, trail: Some([0.5, 1.2, 1.5]), ..d }, look: None, dmg: 1.0, pierce: 0 },
        Trick::Phoenix if plain => Turned { n: 1, fx: ArrowFx { burn: 6.0, straight: true, vanish: true, orb: true, life: 2.0, trail: Some([1.6, 0.7, 0.15]), ..d }, look: None, dmg: 1.15, pierce: 8 },
        Trick::Split => Turned { n: 1, fx: ArrowFx { split: 3, trail: Some([0.8, 0.9, 1.5]), ..d }, look: None, dmg: 1.0, pierce: 0 },
        _ => return None,
    })
}

/// How heavy a bow sounds (sfx_extra.rs): 0 wood and the early metals, 1 the middle bows,
/// 2 the heaviest.
pub fn bow_weight(name: &str) -> u8 {
    match name {
        "bow" | "copper_bow" | "silver_bow" | "platinum_bow" => 0,
        "adamantite_bow" | "radiant_bow" | "verdite_bow" | "lumite_bow" | "tide_bow" | "phoenix_bow" | "skysplit_bow" | "pulse_bow" => 2,
        _ => 1,
    }
}

/// The surface an arrow strikes, for its sound (sfx_extra.rs `ArrowHit`).
pub fn hit_surface(m: crate::audio::Material) -> u8 {
    use crate::audio::Material as M;
    match m {
        M::Wood | M::Leaves => 0,
        M::Stone | M::Ore | M::Glass | M::Gravel => 1,
        M::Metal => 4,
        _ => 2,
    }
}

/// Held weapons whose use button aims and shoots (never opens doors or chests).
pub fn shoots_when_held(name: &str) -> bool {
    crate::bowdraw::is_bow(name) || matches!(name, "crossbow" | "trident")
}

/// What an arrow does besides its damage.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArrowFx {
    /// Seconds a struck monster burns.
    pub burn: f32,
    /// A debuff (effects.rs id) and its seconds.
    pub debuff: Option<(u8, f32)>,
    /// Seconds a struck monster is frozen in place.
    pub freeze: f32,
    /// Radius of a burst of fire where it lands (monsters only, no blocks broken).
    pub burst: f32,
    /// Flies straight, without falling.
    pub straight: bool,
    /// Calls this many stars down on what it hits.
    pub stars: u8,
    /// Colour of its trail (none: no trail).
    pub trail: Option<[f32; 3]>,
    /// Gone where it lands instead of staying lodged (rockets, shards, magic bolts).
    pub vanish: bool,
    /// Seconds a struck monster glows through walls (spectral arrows).
    pub glow: f32,
    /// How fast it turns toward the nearest monster ahead (radians a second; 0: it doesn't).
    pub homing: f32,
    /// Times it glances off a wall before it stops.
    pub bounce: u8,
    /// Splits into this many seekers a moment after it is shot.
    pub split: u8,
    /// Seconds it lasts in the air (0: until it lands).
    pub life: f32,
    /// A creature of light (bee, bat, phoenix): drawn as a glow, not as an arrow.
    pub orb: bool,
}

/// An arrow's bite, by item name.
pub fn arrow_fx(name: &str) -> ArrowFx {
    use crate::effects::{FROSTBURN, POISON, VENOM};
    let d = ArrowFx::default();
    match name {
        "flaming_arrow" => ArrowFx { burn: 5.0, trail: Some([1.4, 0.6, 0.2]), ..d },
        "frostburn_arrow" => ArrowFx { debuff: Some((FROSTBURN, 6.0)), freeze: 1.2, trail: Some([0.5, 0.9, 1.4]), ..d },
        "hellfire_arrow" => ArrowFx { burn: 6.0, burst: 2.5, trail: Some([1.5, 0.4, 0.1]), ..d },
        "unholy_arrow" => ArrowFx { debuff: Some((VENOM, 4.0)), trail: Some([0.8, 0.3, 1.2]), ..d },
        "jester_arrow" => ArrowFx { straight: true, trail: Some([1.4, 1.2, 0.4]), ..d },
        "holy_arrow" => ArrowFx { stars: 2, trail: Some([1.3, 1.3, 1.5]), ..d },
        "bone_arrow" => ArrowFx { trail: Some([0.9, 0.85, 0.75]), ..d },
        "venom_arrow" => ArrowFx { debuff: Some((crate::effects::VENOM, 6.0)), trail: Some([0.6, 1.1, 0.3]), ..d },
        "cursed_arrow" => ArrowFx { debuff: Some((crate::effects::CURSED_FLAMES, 5.0)), trail: Some([0.6, 1.4, 0.3]), ..d },
        "ichor_arrow" => ArrowFx { debuff: Some((crate::effects::ICHOR, 8.0)), trail: Some([1.5, 1.2, 0.3]), ..d },
        // Verdite seeks monsters a little; lumite flies true through many.
        "verdite_arrow" => ArrowFx { homing: 3.0, trail: Some([0.4, 1.3, 0.4]), ..d },
        "lumite_arrow" => ArrowFx { straight: true, trail: Some([0.4, 1.4, 1.3]), ..d },
        "spectral_arrow" => ArrowFx { glow: 10.0, trail: Some([1.3, 1.2, 0.6]), ..d },
        // A firework rocket shot from a crossbow: flies straight and bursts where it lands.
        "firework_rocket" => ArrowFx { burst: 3.0, straight: true, vanish: true, trail: Some([1.4, 1.0, 0.5]), ..d },
        // Tipped arrows carry their potion (weapons.rs); poison ones also leave a green trail.
        n if n.starts_with("tipped_arrow_") && n.contains("poison") => ArrowFx { debuff: Some((POISON, 4.0)), trail: Some([0.4, 1.1, 0.3]), ..d },
        _ => d,
    }
}

/// The directions of one shot's arrows: `n` fanned out sideways over `spread` radians around
/// `forward`, tipped a little up and down by `jitter` (-1..1 per arrow) so a volley doesn't
/// fly as one flat line.
pub fn volley(forward: Vec3, n: u8, spread: f32, jitter: &[f32]) -> Vec<Vec3> {
    let n = n.max(1) as usize;
    let up = if forward.cross(Vec3::Y).length_squared() < 1e-6 { Vec3::Z } else { Vec3::Y };
    let side = forward.cross(up).normalize();
    let lift = side.cross(forward).normalize();
    (0..n)
        .map(|i| {
            let a = if n == 1 { 0.0 } else { spread * (i as f32 / (n - 1) as f32 - 0.5) };
            let j = jitter.get(i).copied().unwrap_or(0.0) * spread * 0.15;
            (Quat::from_axis_angle(lift, -a) * Quat::from_axis_angle(side, j) * forward).normalize()
        })
        .collect()
}

/// Where an arrow that flew along `dir` and reached `pos` inside (or just at) a solid block
/// rests: back along its path to where it went in, then its point a little way in.
pub fn lodge_in_block(pos: Vec3, dir: Vec3, solid: impl Fn(Vec3) -> bool) -> Vec3 {
    let d = dir.normalize_or_zero();
    let mut p = pos;
    // Forward until inside, if it stopped short of the face.
    for _ in 0..30 {
        if solid(p) {
            break;
        }
        p += d * 0.02;
    }
    // Back to the face.
    for _ in 0..60 {
        if !solid(p - d * 0.02) {
            break;
        }
        p -= d * 0.02;
    }
    p + d * 0.12
}

/// An arrow lodged in a monster: which one (net id), and where its point and its direction are
/// in the monster's own frame, so it stays put as the monster moves and turns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lodged {
    pub mob: u32,
    at: Vec3,
    dir: Vec3,
}

impl Lodged {
    /// Lodge an arrow whose point is at `tip`, flying along `dir`, in `m`.
    pub fn new(m: &Mob, tip: Vec3, dir: Vec3) -> Lodged {
        Lodged::at(m.net_id, m.body.pos, m.yaw, tip, dir)
    }

    /// The same for a monster `mob` standing at `pos` facing `yaw`.
    pub fn at(mob: u32, pos: Vec3, yaw: f32, tip: Vec3, dir: Vec3) -> Lodged {
        let back = Quat::from_rotation_y(-yaw);
        Lodged { mob, at: back * (tip - pos), dir: back * dir.normalize_or_zero() }
    }

    /// Its point and direction now, the monster being at `pos` facing `yaw`.
    pub fn place(&self, pos: Vec3, yaw: f32) -> (Vec3, Vec3) {
        let turn = Quat::from_rotation_y(yaw);
        (pos + turn * self.at, turn * self.dir)
    }
}

/// The point where a ray from `from` along `dir` first meets the box `lo..hi`, pushed `depth`
/// further in (None when it misses).
pub fn into_box(from: Vec3, dir: Vec3, lo: Vec3, hi: Vec3, depth: f32) -> Option<Vec3> {
    let d = dir.normalize_or_zero();
    let inv = Vec3::ONE / d;
    let (t1, t2) = ((lo - from) * inv, (hi - from) * inv);
    let near = t1.min(t2).max_element().max(0.0);
    let far = t1.max(t2).min_element();
    (near <= far).then(|| from + d * (near + depth).min(far.max(near)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bow_shoots_its_own_way() {
        let names = ["bow", "copper_bow", "silver_bow", "platinum_bow", "umbral_bow", "sanguinite_bow", "magmarite_bow", "cobalt_bow", "mythril_bow", "adamantite_bow", "radiant_bow", "verdite_bow", "lumite_bow", "hive_bow", "batwing_bow", "marrow_bow", "frost_bow", "storm_bow", "pulse_bow", "tide_bow", "phoenix_bow", "skysplit_bow"];
        let all: Vec<BowStats> = names.iter().map(|n| bow_stats(n).unwrap_or_else(|| panic!("{n}"))).collect();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
        // Every bow added by tr_weapons.rs has stats.
        for n in names {
            assert!(crate::bowdraw::is_bow(n));
        }
    }

    #[test]
    fn a_volley_fans_out_around_the_aim() {
        let f = Vec3::new(0.0, 0.2, -1.0).normalize();
        let v = volley(f, 3, 0.2, &[0.0, 0.0, 0.0]);
        assert_eq!(v.len(), 3);
        assert!(v[1].distance(f) < 1e-4);
        assert!((v[0].angle_between(f) - 0.1).abs() < 1e-3 && (v[2].angle_between(f) - 0.1).abs() < 1e-3);
        assert!(v[0].x * v[2].x < 0.0);
    }

    #[test]
    fn arrows_rest_with_their_point_in_the_block() {
        // A wall at x >= 3; the arrow is caught a step inside it, flying down and along +x.
        let solid = |p: Vec3| p.x >= 3.0;
        let dir = Vec3::new(1.0, -0.5, 0.0);
        let tip = lodge_in_block(Vec3::new(3.3, 1.0, 0.0), dir, solid);
        assert!(tip.x > 3.0 && tip.x < 3.15, "{tip}");
        // Stopped just short of it, it is still pushed in.
        let tip = lodge_in_block(Vec3::new(2.9, 1.0, 0.0), dir, solid);
        assert!(tip.x > 3.0 && tip.x < 3.15, "{tip}");
    }

    #[test]
    fn a_lodged_arrow_turns_with_its_monster() {
        let l = Lodged::at(7, Vec3::new(5.0, 0.0, 0.0), 0.0, Vec3::new(5.0, 1.0, 0.3), Vec3::NEG_Z);
        let (tip, dir) = l.place(Vec3::new(5.0, 2.0, 0.0), std::f32::consts::FRAC_PI_2);
        assert!(tip.distance(Vec3::new(5.3, 3.0, 0.0)) < 1e-4, "{tip}");
        assert!(dir.distance(Vec3::NEG_X) < 1e-4, "{dir}");
    }

    #[test]
    fn into_box_finds_the_face() {
        let p = into_box(Vec3::new(-2.0, 0.5, 0.5), Vec3::X, Vec3::ZERO, Vec3::ONE, 0.1).unwrap();
        assert!(p.distance(Vec3::new(0.1, 0.5, 0.5)) < 1e-4);
        assert!(into_box(Vec3::new(-2.0, 3.0, 0.5), Vec3::X, Vec3::ZERO, Vec3::ONE, 0.1).is_none());
    }
}
