//! Spells with a body: every caster's projectile is its own 3D model, built from boxes here
//! (our own art, painted in code into one skin layer) and drawn glowing with the mob pipeline.
//! The ideas come from Terraria's magic weapons, thought out again for a 3D world: a projectile
//! has to read from behind and from the side, so each one has depth and moves in all three
//! axes instead of being a flat sprite.
//!
//! - Foice demoníaca: a crescent blade lying flat that spins like a saw, slow at first and then
//!   faster (it cuts through several monsters).
//! - Bola de fogo, grande bola de fogo, meteoro: a white-hot core in tumbling flame lumps with
//!   a tail that licks behind; the meteor is a cracked rock glowing from inside.
//! - Lasca de gelo, pingente, rocha de gelo, floco de neve: faceted crystals, long icicles, an
//!   ice boulder, six-armed flakes that turn as they fly.
//! - Faísca: a zigzag of lightning that flickers into a new shape every few frames.
//! - Ametista: a crystal cluster with motes circling it. Espinho da ruína: a barbed dark thorn.
//! - Folha, espinho guardião, esporo, praga: a spinning leaf, a living thorn, a spore pod with
//!   glowing spots, a bubbling ball of blight that drips.
//! - Estrela: a five-pointed star with depth. Sol: a small sun with turning rays.
//! - Abelha: a little bee that buzzes after monsters. Orbe d'água: a wobbling ball of water with
//!   bubbles (it bounces off walls). Orbe do vazio: a black heart in two turning rings.
//! - Bala de canhão, laser, labareda, espírito, and the arc that draws lightning.

use std::f32::consts::{PI, TAU};

use glam::{Mat4, Quat, Vec3};

use crate::entity::Bones;
use crate::instruments::{IBox, Piece};

pub const FIREBALL: usize = 0;
pub const ICE_SHARD: usize = 1;
pub const SPARK: usize = 2;
pub const AMETHYST: usize = 3;
pub const LEAF: usize = 4;
pub const STAR: usize = 5;
pub const BIG_FIRE: usize = 6;
pub const ICICLE: usize = 7;
pub const THORN: usize = 8;
pub const CANNONBALL: usize = 9;
pub const FLAME_PUFF: usize = 10;
pub const METEOR: usize = 11;
pub const VOID_ORB: usize = 12;
pub const BEE: usize = 13;
pub const WATER_ORB: usize = 14;
pub const SCYTHE: usize = 15;
pub const RUIN_THORN: usize = 16;
pub const BLIGHT: usize = 17;
pub const SUN: usize = 18;
pub const LASER: usize = 19;
pub const FROST_BOULDER: usize = 20;
pub const SPORE: usize = 21;
pub const SNOWFLAKE: usize = 22;
pub const SPIRIT: usize = 23;
pub const ARC: usize = 24;
// Lote 2 (plan/magias-analise.md): the new spells.
pub const FIRE_FLOWER: usize = 25;
pub const WATER_STREAM: usize = 26;
pub const GROUND_THORN: usize = 27;
pub const CRYSTAL: usize = 28;
pub const CURSED_FLAME: usize = 29;
pub const MAGNET: usize = 30;
pub const SHADOW_BEAM: usize = 31;
pub const TYPHOON: usize = 32;
pub const LUNAR: usize = 33;
pub const PRISM_BEAM: usize = 34;
pub const PRISM_GEM: usize = 35;
pub const COUNT: usize = 36;

/// The shape a caster's spell flies as, and how big (1 = as modelled).
pub fn shape_of(item: &str, school: u8) -> (usize, f32) {
    match item {
        "ruby_wand" => (FIREBALL, 1.0),
        "sapphire_wand" => (ICE_SHARD, 1.0),
        "topaz_wand" => (SPARK, 1.0),
        "amethyst_wand" => (AMETHYST, 1.0),
        "emerald_wand" => (LEAF, 1.0),
        "fire_staff" => (BIG_FIRE, 1.0),
        "frost_staff" => (ICICLE, 0.8),
        "guardian_staff" => (THORN, 1.0),
        "@warship_cannon" => (CANNONBALL, 1.2),
        "flame_tome" => (FLAME_PUFF, 1.0),
        "meteor_staff" => (METEOR, 1.6),
        "ice_spike_staff" => (ICICLE, 1.25),
        "void_tome" => (VOID_ORB, 1.0),
        "hive_staff" => (BEE, 0.9),
        "water_bolt" => (WATER_ORB, 1.0),
        "demon_scythe" => (SCYTHE, 1.35),
        "ruin_thorn" => (RUIN_THORN, 1.0),
        "blight_rod" => (BLIGHT, 1.0),
        "sun_stone_staff" => (SUN, 1.1),
        "meteor_blaster" => (LASER, 1.0),
        "colossus_staff" => (FROST_BOULDER, 0.9),
        "star_tome" => (STAR, 1.0),
        "worm_staff" => (SPORE, 1.1),
        "blizzard_staff" => (SNOWFLAKE, 1.0),
        "spirit_staff" => (SPIRIT, 1.0),
        "flower_of_fire" => (FIRE_FLOWER, 1.0),
        "aqua_scepter" => (WATER_STREAM, 0.8),
        "creeping_thorn" => (GROUND_THORN, 1.0),
        "crystal_storm" => (CRYSTAL, 0.8),
        "cursed_flames" => (CURSED_FLAME, 1.1),
        "magnet_sphere" => (MAGNET, 0.75),
        "shadow_beam" => (SHADOW_BEAM, 1.0),
        "razorblade_typhoon" => (TYPHOON, 1.1),
        "lunar_flare" => (LUNAR, 1.7),
        "prism" => (PRISM_BEAM, 1.0),
        _ => (school_shape(school), 1.0),
    }
}

/// A monster's bolt (or a caster we have no shape for): the plain shape of its school.
pub fn school_shape(school: u8) -> usize {
    match school {
        0 => FIREBALL,
        1 => ICE_SHARD,
        2 => SPARK,
        3 => AMETHYST,
        4 => LEAF,
        _ => STAR,
    }
}

/// The colour of the light a shape gives off (and of its trail).
pub fn light_color(shape: usize) -> [f32; 3] {
    match shape {
        FIREBALL | BIG_FIRE | FLAME_PUFF | METEOR => [1.0, 0.5, 0.15],
        CANNONBALL => [1.0, 0.6, 0.25],
        ICE_SHARD | ICICLE | FROST_BOULDER | SNOWFLAKE => [0.45, 0.8, 1.0],
        SPARK | ARC => [1.0, 0.95, 0.5],
        AMETHYST | SPIRIT => [0.75, 0.35, 1.0],
        VOID_ORB => [0.5, 0.15, 0.9],
        SCYTHE => [0.85, 0.2, 1.0],
        RUIN_THORN => [0.9, 0.15, 0.35],
        LEAF | THORN | SPORE => [0.4, 1.0, 0.35],
        BLIGHT => [0.6, 0.95, 0.15],
        STAR => [1.0, 0.85, 0.35],
        SUN => [1.0, 0.75, 0.25],
        LASER => [1.0, 0.25, 0.1],
        WATER_ORB => [0.25, 0.55, 1.0],
        BEE => [1.0, 0.8, 0.2],
        FIRE_FLOWER => [1.0, 0.45, 0.12],
        WATER_STREAM | TYPHOON => [0.3, 0.6, 1.0],
        GROUND_THORN => [0.65, 0.3, 0.9],
        CRYSTAL => [1.0, 0.5, 0.9],
        CURSED_FLAME => [0.45, 1.0, 0.15],
        MAGNET => [0.35, 0.95, 1.0],
        SHADOW_BEAM => [0.6, 0.25, 1.0],
        LUNAR => [0.6, 0.8, 1.0],
        _ => [1.0, 1.0, 1.0],
    }
}

/// The voice a shape is cast and lands with (sfx.rs `spell`): 0 fire, 1 ice, 2 lightning,
/// 3 arcane, 4 leaves and spores, 5 water, 6 scythe, 7 bees, 8 void, 9 stars, 10 rock.
pub fn sound_of(shape: usize) -> u8 {
    match shape {
        FIREBALL | BIG_FIRE | FLAME_PUFF | SUN | LASER | FIRE_FLOWER | CURSED_FLAME => 0,
        ICE_SHARD | ICICLE | FROST_BOULDER | SNOWFLAKE | CRYSTAL => 1,
        SPARK | ARC | MAGNET => 2,
        AMETHYST | SPIRIT | RUIN_THORN | SHADOW_BEAM | PRISM_BEAM | PRISM_GEM => 3,
        LEAF | THORN | SPORE | BLIGHT | GROUND_THORN => 4,
        WATER_ORB | WATER_STREAM | TYPHOON => 5,
        SCYTHE => 6,
        BEE => 7,
        VOID_ORB => 8,
        STAR | LUNAR => 9,
        _ => 10,
    }
}

/// How brightly a shape lights its surroundings (0..1, like a block light level).
pub fn light_level(shape: usize) -> f32 {
    match shape {
        BEE | CANNONBALL | GROUND_THORN | PRISM_BEAM | SHADOW_BEAM | WATER_STREAM => 0.0,
        BIG_FIRE | METEOR | SUN | MAGNET => 0.95,
        VOID_ORB => 0.55,
        FLAME_PUFF => 0.6,
        _ => 0.75,
    }
}

/// Does the shape shine by itself (most do; the cannonball is lit by the world).
fn glows(shape: usize) -> bool {
    shape != CANNONBALL && shape != GROUND_THORN
}

// Materials: one 16 × 16 swatch each in the spell skin (4 across, 8 down).
const FIRE_CORE: u8 = 0;
const FIRE: u8 = 1;
const FIRE_DARK: u8 = 2;
const ICE: u8 = 3;
const ICE_DEEP: u8 = 4;
const SPARK_M: u8 = 5;
const AMETHYST_M: u8 = 6;
const AMETHYST_LIGHT: u8 = 7;
const LEAF_M: u8 = 8;
const LEAF_VEIN: u8 = 9;
const STAR_M: u8 = 10;
const STAR_CORE: u8 = 11;
const WOOD: u8 = 12;
const IRON_BALL: u8 = 13;
const ROCK: u8 = 14;
const VOID: u8 = 15;
const VOID_RIM: u8 = 16;
const BEE_YELLOW: u8 = 17;
const BEE_BLACK: u8 = 18;
const WING: u8 = 19;
const WATER: u8 = 20;
const FOAM: u8 = 21;
const SCYTHE_M: u8 = 22;
const SCYTHE_EDGE: u8 = 23;
const RUIN: u8 = 24;
const BLIGHT_M: u8 = 25;
const BLIGHT_DARK: u8 = 26;
const SUN_M: u8 = 27;
const LASER_M: u8 = 28;
const SPORE_M: u8 = 29;
const EYE: u8 = 30;
const ARC_M: u8 = 31;
// The second page (skin layer) of materials.
const VILE: u8 = 32;
const VILE_TIP: u8 = 33;
const CRYSTAL_PINK: u8 = 34;
const CRYSTAL_CYAN: u8 = 35;
const CURSED: u8 = 36;
const CURSED_CORE: u8 = 37;
const CURSED_DARK: u8 = 38;
const MAGNET_GLASS: u8 = 39;
const MAGNET_CORE: u8 = 40;
const SHADOW_M: u8 = 41;
const TYPHOON_EDGE: u8 = 42;
const MOON: u8 = 43;
const MOON_GLOW: u8 = 44;
const BEAM_WHITE: u8 = 45;
const PRISM_GLASS: u8 = 46;
const PETAL: u8 = 47;
const MATERIALS: u8 = 48;
/// Skin layers the materials fill (32 swatches each).
pub const PAGES: usize = 2;

/// A box by its centre and size (model pixels).
fn c(center: [f32; 3], size: [f32; 3], mat: u8) -> IBox {
    let (m, h) = (Vec3::from(center), Vec3::from(size) * 0.5);
    IBox { lo: m - h, hi: m + h, mat }
}

/// A rounded lump: three crossed boxes, a cube with its edges cut.
fn ball(at: [f32; 3], r: f32, mat: u8) -> Vec<IBox> {
    let (a, b) = (r * 2.0, r * 1.45);
    vec![c(at, [a, b, b], mat), c(at, [b, a, b], mat), c(at, [b, b, a], mat)]
}

/// A spike along +Z from z0, `w` wide at the root, tapering to a point `len` further on.
fn spike(z0: f32, w: f32, len: f32, steps: usize, root: u8, tip: u8) -> Vec<IBox> {
    (0..steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            let s = w * (1.0 - t * 0.85);
            let seg = len / steps as f32;
            c([0.0, 0.0, z0 + seg * (k as f32 + 0.5)], [s, s, seg + 0.2], if t > 0.6 { tip } else { root })
        })
        .collect()
}

/// A flat spike in the XZ plane (a leaf's blade, a star's point): `w` wide, `th` thick.
fn blade(z0: f32, w: f32, th: f32, len: f32, steps: usize, mat: u8) -> Vec<IBox> {
    (0..steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            let seg = len / steps as f32;
            c([0.0, 0.0, z0 + seg * (k as f32 + 0.5)], [w * (1.0 - t * 0.85), th, seg + 0.2], mat)
        })
        .collect()
}

/// A rounder ball for bone `part`: the lump again turned 45° about Y and about X, so its
/// outline is near round from any side.
fn round(part: u32, r: f32, mat: u8) -> Vec<Piece> {
    let q = std::f32::consts::FRAC_PI_4;
    vec![
        piece(part, ball([0.0; 3], r, mat)),
        placed(part, Mat4::from_rotation_y(q), ball([0.0; 3], r * 0.97, mat)),
        placed(part, Mat4::from_rotation_x(q), ball([0.0; 3], r * 0.97, mat)),
    ]
}

fn piece(part: u32, boxes: Vec<IBox>) -> Piece {
    Piece { part, place: Mat4::IDENTITY, boxes }
}

fn placed(part: u32, place: Mat4, boxes: Vec<IBox>) -> Piece {
    Piece { part, place, boxes }
}

/// The boxes of model `i`, each piece on its bone. Forward is +Z, up +Y.
pub fn model(i: usize) -> Vec<Piece> {
    match i {
        FIREBALL => {
            // 0 the core in its flame lumps (tumbles), 1 the tail.
            let mut core = Vec::new();
            for (p, s) in [([2.0, 0.4, 0.0], 2.2), ([-1.9, -0.3, 0.4], 2.0), ([0.3, 2.0, -0.2], 2.1), ([-0.2, -2.0, 0.3], 1.9), ([0.4, 0.2, 2.0], 1.8), ([-0.4, 0.0, -2.0], 2.2)] {
                core.push(c(p, [s, s, s], FIRE));
            }
            let tail = vec![
                c([0.0, 0.0, -3.4], [3.2, 3.2, 2.4], FIRE),
                c([0.3, 0.2, -5.4], [2.4, 2.4, 2.2], FIRE),
                c([-0.2, 0.4, -7.2], [1.7, 1.7, 2.0], FIRE_DARK),
                c([0.2, 0.6, -8.7], [1.0, 1.0, 1.6], FIRE_DARK),
            ];
            let mut pieces = round(0, 2.4, FIRE_CORE);
            pieces.extend([piece(0, core), piece(1, tail)]);
            pieces
        }
        ICE_SHARD => {
            // A long faceted crystal (two pyramids base to base) with two small ones on its sides.
            let mut b = spike(0.0, 3.4, 6.0, 4, ICE, ICE);
            b.extend(spike(0.0, 3.4, 4.0, 3, ICE_DEEP, ICE).into_iter().map(|x| IBox { lo: Vec3::new(x.lo.x, x.lo.y, -x.hi.z), hi: Vec3::new(x.hi.x, x.hi.y, -x.lo.z), mat: x.mat }));
            b.push(c([0.0, 0.0, 0.0], [2.2, 4.0, 2.2], ICE_DEEP));
            let side = Mat4::from_translation(Vec3::new(1.4, 0.6, -0.5)) * Mat4::from_rotation_y(0.5) * Mat4::from_rotation_x(-0.3);
            let other = Mat4::from_translation(Vec3::new(-1.3, -0.7, -1.0)) * Mat4::from_rotation_y(-0.6) * Mat4::from_rotation_x(0.4);
            vec![piece(0, b), placed(0, side, spike(0.0, 1.6, 3.4, 3, ICE_DEEP, ICE)), placed(0, other, spike(0.0, 1.4, 3.0, 3, ICE_DEEP, ICE))]
        }
        SPARK => {
            // A zigzag of light: four bent segments and a hot knot at the front. Bone 1 a smaller
            // fork that flickers on its own.
            let main = vec![
                c([0.0, 0.0, 0.0], [2.2, 2.2, 2.2], SPARK_M),
                c([0.8, 0.0, -1.8], [1.0, 1.0, 2.6], ARC_M),
                c([-0.6, 0.3, -3.6], [1.0, 1.0, 2.4], ARC_M),
                c([0.7, -0.2, -5.4], [0.9, 0.9, 2.4], ARC_M),
                c([-0.4, 0.2, -7.2], [0.8, 0.8, 2.2], ARC_M),
                c([0.0, 0.0, 1.6], [1.2, 1.2, 1.4], ARC_M),
            ];
            let fork = vec![c([1.4, 0.8, -2.4], [0.7, 0.7, 2.2], SPARK_M), c([2.2, 1.4, -4.0], [0.6, 0.6, 1.8], ARC_M)];
            vec![piece(0, main), piece(1, fork)]
        }
        AMETHYST => {
            // A cluster of purple crystals pointing every way; 1..=3 motes circling it.
            let mut pieces = vec![piece(0, spike(-0.5, 2.8, 5.5, 4, AMETHYST_M, AMETHYST_LIGHT)), piece(0, ball([0.0; 3], 1.7, AMETHYST_M))];
            let back = Mat4::from_rotation_y(PI);
            pieces.push(placed(0, back, spike(-0.5, 2.2, 3.5, 3, AMETHYST_M, AMETHYST_LIGHT)));
            for k in 0..4 {
                let a = k as f32 * TAU / 4.0 + 0.4;
                let m = Mat4::from_rotation_z(a) * Mat4::from_rotation_x(-1.1);
                pieces.push(placed(0, m, spike(0.5, 1.6, 3.2, 3, AMETHYST_M, AMETHYST_LIGHT)));
            }
            for k in 1..=3 {
                pieces.push(piece(k, vec![c([0.0; 3], [1.0, 1.0, 1.0], AMETHYST_LIGHT)]));
            }
            pieces
        }
        LEAF => {
            // A leaf with a midrib and a curl, a stem behind.
            let mut b = blade(-3.0, 4.0, 0.7, 7.5, 6, LEAF_M);
            b.extend(blade(-4.6, 2.4, 0.7, 1.8, 2, LEAF_M).into_iter().map(|x| IBox { lo: Vec3::new(x.lo.x, x.lo.y, -x.hi.z - 7.6), hi: Vec3::new(x.hi.x, x.hi.y, -x.lo.z - 7.6), mat: x.mat }));
            b.push(c([0.0, 0.35, 0.6], [0.6, 0.4, 8.0], LEAF_VEIN));
            b.push(c([0.0, 0.0, -4.6], [0.5, 0.5, 2.4], LEAF_VEIN));
            for z in [-1.0f32, 1.0, 2.8] {
                b.push(c([0.9, 0.35, z], [1.6, 0.3, 0.35], LEAF_VEIN));
                b.push(c([-0.9, 0.35, z], [1.6, 0.3, 0.35], LEAF_VEIN));
            }
            vec![piece(0, b)]
        }
        STAR => {
            // Five points in the XY plane with a domed middle, so it has depth from any side.
            let mut pieces = vec![piece(0, vec![c([0.0; 3], [3.0, 3.0, 2.4], STAR_CORE), c([0.0; 3], [2.0, 2.0, 3.6], STAR_CORE)])];
            for k in 0..5 {
                let a = PI / 2.0 + k as f32 * TAU / 5.0;
                // The point lies along +Z first, then is laid into the plane.
                let m = Mat4::from_rotation_z(a) * Mat4::from_rotation_y(PI / 2.0);
                pieces.push(placed(0, m, blade(0.8, 2.6, 1.6, 4.4, 4, STAR_M)));
            }
            pieces
        }
        BIG_FIRE => {
            // 0 a molten core, 1 a shell of flame lumps turning the other way, 2 a long tail.
            let mut core = ball([0.0; 3], 3.0, FIRE_CORE);
            core.extend(ball([0.0; 3], 2.4, FIRE_CORE));
            let mut shell = Vec::new();
            for x in [-1.0f32, 1.0] {
                for y in [-1.0f32, 1.0] {
                    for z in [-1.0f32, 1.0] {
                        let s = 2.4 + ((x + y * 2.0 + z * 3.0).abs() % 3.0) * 0.3;
                        shell.push(c([x * 2.5, y * 2.5, z * 2.5], [s, s, s], if (x + y + z) > 0.0 { FIRE } else { FIRE_DARK }));
                    }
                }
            }
            for p in [[3.6, 0.0, 0.0], [-3.6, 0.0, 0.0], [0.0, 3.6, 0.0], [0.0, -3.6, 0.0]] {
                shell.push(c(p, [1.8, 1.8, 1.8], FIRE));
            }
            let tail = vec![
                c([0.0, 0.0, -4.6], [4.6, 4.6, 3.0], FIRE),
                c([0.5, 0.3, -7.2], [3.6, 3.6, 2.8], FIRE),
                c([-0.4, 0.7, -9.6], [2.6, 2.6, 2.6], FIRE_DARK),
                c([0.3, 1.1, -11.8], [1.7, 1.7, 2.2], FIRE_DARK),
                c([0.0, 1.4, -13.6], [0.9, 0.9, 1.6], FIRE_DARK),
            ];
            vec![piece(0, core), piece(1, shell), piece(2, tail)]
        }
        ICICLE => {
            // A long icicle: a frosted base with ridges, then a clear point.
            let mut b = spike(-4.0, 3.6, 14.0, 7, ICE_DEEP, ICE);
            b.push(c([0.0, 0.0, -4.4], [4.0, 4.0, 1.0], ICE));
            for (x, y) in [(1.7, 0.0), (-1.7, 0.0), (0.0, 1.7), (0.0, -1.7)] {
                b.push(c([x, y, -1.5], [if x == 0.0 { 1.0 } else { 0.6 }, if y == 0.0 { 1.0 } else { 0.6 }, 4.0], ICE));
            }
            vec![piece(0, b)]
        }
        THORN => {
            // A living thorn: a green-wood shaft, a glowing point, two leaves near its end.
            let mut b = vec![c([0.0, 0.0, -1.0], [1.4, 1.4, 9.0], WOOD), c([0.0, 0.0, -5.8], [1.8, 1.8, 1.2], WOOD)];
            b.extend(spike(3.4, 1.8, 4.2, 3, LEAF_VEIN, LEAF_M));
            for (s, z) in [(1.0f32, -3.5), (-1.0, -2.0)] {
                let m = Mat4::from_translation(Vec3::new(0.6 * s, 0.0, z)) * Mat4::from_rotation_y(-0.9 * s + PI);
                b.extend(blade(0.0, 1.8, 0.4, 3.0, 3, LEAF_M).into_iter().map(|x| {
                    let (lo, hi) = (m.transform_point3(x.lo), m.transform_point3(x.hi));
                    IBox { lo: lo.min(hi), hi: lo.max(hi), mat: x.mat }
                }));
            }
            vec![piece(0, b)]
        }
        CANNONBALL => {
            // A dark iron ball with a band, and a burning fuse on top (bone 1, kept upright).
            let mut b = ball([0.0; 3], 2.8, IRON_BALL);
            b.push(c([0.0, 0.0, 0.0], [5.9, 1.0, 5.9], BEE_BLACK));
            vec![piece(0, b), piece(1, vec![c([0.0, 3.0, 0.0], [0.6, 1.6, 0.6], WOOD), c([0.0, 4.1, 0.0], [1.0, 1.0, 1.0], FIRE_CORE)])]
        }
        FLAME_PUFF => {
            let b = vec![
                c([0.0, 0.0, 0.0], [2.6, 2.6, 2.6], FIRE_CORE),
                c([1.4, 0.8, -0.4], [2.2, 2.0, 2.0], FIRE),
                c([-1.2, 0.4, 0.6], [2.0, 2.4, 2.0], FIRE),
                c([0.2, 1.8, 0.2], [1.6, 2.2, 1.6], FIRE),
                c([0.3, -1.1, -1.0], [1.8, 1.4, 1.8], FIRE_DARK),
                c([-0.4, 2.9, -0.3], [0.9, 1.4, 0.9], FIRE_DARK),
            ];
            vec![piece(0, b)]
        }
        METEOR => {
            // A craggy rock glowing through its cracks (0, tumbling), a flame tail (1).
            let mut rock = vec![
                c([0.0, 0.0, 0.0], [5.0, 4.4, 4.8], ROCK),
                c([0.8, 0.9, 0.4], [4.0, 4.2, 3.6], ROCK),
                c([-1.0, -0.6, 0.6], [3.6, 3.2, 4.4], ROCK),
                c([0.4, -1.2, -1.0], [3.8, 3.0, 3.4], ROCK),
                c([-0.5, 1.4, -0.8], [2.6, 2.4, 3.0], ROCK),
            ];
            rock.extend(ball([0.0; 3], 2.3, FIRE_CORE));
            let tail = vec![
                c([0.0, 0.0, -3.8], [4.4, 4.4, 2.6], FIRE),
                c([0.4, 0.4, -6.2], [3.4, 3.4, 2.6], FIRE),
                c([-0.3, 0.7, -8.4], [2.4, 2.4, 2.4], FIRE_DARK),
                c([0.2, 1.0, -10.4], [1.4, 1.4, 2.0], FIRE_DARK),
            ];
            vec![piece(0, rock), piece(1, tail)]
        }
        VOID_ORB => {
            // A black heart (0) with a violet glint, in two rings of shards (1 and 2).
            let core = vec![c([1.2, 1.4, 3.1], [1.2, 1.2, 0.6], VOID_RIM)];
            let ring = |r: f32, n: usize, s: f32| -> Vec<IBox> {
                (0..n)
                    .map(|k| {
                        let a = k as f32 / n as f32 * TAU;
                        c([a.cos() * r, 0.0, a.sin() * r], [s, s * 0.6, s], if k % 3 == 0 { AMETHYST_LIGHT } else { VOID_RIM })
                    })
                    .collect()
            };
            let mut pieces = round(0, 3.4, VOID);
            pieces.extend([piece(0, core), piece(1, ring(6.5, 12, 1.5)), piece(2, ring(8.5, 14, 1.1))]);
            pieces
        }
        BEE => {
            // 0 body (thorax, striped abdomen, head with eyes, stinger), 1/2 wings.
            let mut body = vec![
                c([0.0, 0.0, 0.0], [2.6, 2.6, 2.6], BEE_BLACK),
                c([0.0, 0.0, 2.2], [2.2, 2.2, 1.8], BEE_BLACK),
                c([0.0, -0.2, -2.2], [3.0, 3.0, 1.4], BEE_YELLOW),
                c([0.0, -0.2, -3.4], [3.0, 3.0, 1.0], BEE_BLACK),
                c([0.0, -0.3, -4.4], [2.6, 2.6, 1.0], BEE_YELLOW),
                c([0.0, -0.4, -5.3], [1.8, 1.8, 0.8], BEE_BLACK),
                c([0.0, -0.5, -6.1], [0.5, 0.5, 1.0], BEE_BLACK),
                c([0.0, 1.0, 0.0], [2.8, 0.6, 2.0], BEE_YELLOW),
            ];
            for s in [1.0f32, -1.0] {
                body.push(c([0.95 * s, 0.3, 2.6], [0.6, 1.0, 1.0], EYE));
                body.push(c([0.5 * s, 1.5, 3.0], [0.3, 1.2, 0.3], BEE_BLACK));
            }
            let wing = |s: f32| vec![c([2.0 * s, 1.5, -0.6], [3.4, 0.25, 2.2], WING), c([1.4 * s, 1.4, -2.2], [2.2, 0.25, 1.6], WING)];
            vec![piece(0, body), piece(1, wing(1.0)), piece(2, wing(-1.0))]
        }
        WATER_ORB => {
            // A ball of water with a pale highlight (0), three bubbles (1..=3).
            let mut pieces = round(0, 3.0, WATER);
            pieces.push(piece(0, vec![c([1.0, 1.6, 2.1], [1.4, 1.0, 1.0], FOAM), c([-1.4, -1.0, 2.0], [0.8, 0.8, 0.8], FOAM)]));
            for k in 1..=3 {
                pieces.push(piece(k, vec![c([0.0; 3], [0.9, 0.9, 0.9], FOAM)]));
            }
            pieces
        }
        SCYTHE => {
            // A crescent blade lying flat (XZ plane), its hub at the origin: a broad back that
            // sweeps round into a fine point, a bright cutting edge on the outside, a dark hub.
            let mut b = Vec::new();
            let n = 14;
            for k in 0..n {
                let t = k as f32 / (n - 1) as f32;
                let a = -0.5 + t * 3.6;
                let r = 6.2 + t * 0.6;
                let w = 3.4 * (1.0 - t * 0.8);
                let (x, z) = (a.cos() * r, a.sin() * r);
                // Each step a block turned roughly along the arc (boxes stay axis-aligned, so
                // the steps overlap a little to keep the curve unbroken).
                b.push(c([x, 0.0, z], [w + 0.6, 1.0, w + 0.6], SCYTHE_M));
                let (ex, ez) = (a.cos() * (r + w * 0.55), a.sin() * (r + w * 0.55));
                b.push(c([ex, 0.0, ez], [1.0, 0.6, 1.0], SCYTHE_EDGE));
            }
            // The spine from the hub to the blade's root, and the hub itself.
            for k in 1..5 {
                let r = k as f32 * 1.3;
                let a = -0.5f32;
                b.push(c([a.cos() * r, 0.0, a.sin() * r], [1.4, 1.2, 1.4], SCYTHE_M));
            }
            b.push(c([0.0, 0.0, 0.0], [2.4, 1.8, 2.4], VOID));
            b.push(c([0.0, 0.0, 0.0], [1.2, 2.4, 1.2], SCYTHE_EDGE));
            vec![piece(0, b)]
        }
        RUIN_THORN => {
            // A barbed black-red thorn with a glowing point.
            let mut b = spike(-5.0, 2.4, 12.0, 6, RUIN, SCYTHE_EDGE);
            for (z, s) in [(-3.0f32, 1.0f32), (-1.0, -1.0), (1.0, 1.0)] {
                b.push(c([1.3 * s, 0.0, z], [1.6, 0.8, 0.8], RUIN));
                b.push(c([2.0 * s, 0.0, z - 0.6], [0.6, 0.6, 1.0], RUIN));
                b.push(c([0.0, 1.3 * s, z + 1.0], [0.8, 1.6, 0.8], RUIN));
            }
            vec![piece(0, b)]
        }
        BLIGHT => {
            // A bubbling ball of blight with dark blotches (0) and drops hanging under it (1).
            let mut b = Vec::new();
            for p in [[2.4, 1.0, 0.6], [-2.0, 1.6, -0.8], [0.6, -2.2, 1.6], [-0.8, 0.4, 2.6], [1.0, 2.4, -1.4]] {
                b.push(c(p, [2.0, 2.0, 2.0], BLIGHT_M));
            }
            for p in [[2.9, 1.2, 0.8], [-1.0, 0.5, 3.1], [0.5, -2.8, 1.0]] {
                b.push(c(p, [1.0, 1.0, 1.0], BLIGHT_DARK));
            }
            let drips = vec![
                c([1.0, -3.6, 0.0], [1.0, 1.6, 1.0], BLIGHT_M),
                c([1.0, -4.8, 0.0], [0.6, 0.8, 0.6], BLIGHT_DARK),
                c([-1.4, -3.4, 0.8], [0.8, 1.2, 0.8], BLIGHT_M),
            ];
            let mut pieces = round(0, 3.2, BLIGHT_M);
            pieces.extend([piece(0, b), piece(1, drips)]);
            pieces
        }
        SUN => {
            // A small sun: a bright faceted core (0), rays all round (1), a wider halo of short
            // rays turning the other way (2).
            let mut rays = Vec::new();
            for k in 0..8 {
                let m = Mat4::from_rotation_z(k as f32 * TAU / 8.0) * Mat4::from_rotation_y(PI / 2.0);
                rays.push(placed(1, m, spike(3.2, 1.8, 4.6, 3, SUN_M, FIRE)));
            }
            for (a, b) in [(0.0f32, 0.8f32), (PI, 0.8), (0.0, -0.8), (PI, -0.8)] {
                let m = Mat4::from_rotation_y(a) * Mat4::from_rotation_x(b);
                rays.push(placed(2, m, spike(3.0, 1.6, 3.6, 3, SUN_M, FIRE)));
            }
            let mut pieces = round(0, 3.4, SUN_M);
            pieces.push(piece(0, vec![c([0.0, 0.0, 3.3], [2.4, 2.4, 0.6], STAR_CORE), c([0.0, 0.0, -3.3], [2.4, 2.4, 0.6], STAR_CORE)]));
            pieces.extend(rays);
            pieces
        }
        LASER => {
            // A beam of red light with a white-hot head.
            vec![piece(0, vec![c([0.0, 0.0, -4.0], [1.3, 1.3, 14.0], LASER_M), c([0.0, 0.0, 3.2], [1.6, 1.6, 1.6], FIRE_CORE), c([0.0, 0.0, -4.0], [0.7, 1.6, 13.0], LASER_M)])]
        }
        FROST_BOULDER => {
            // A chunk of ice rock with frost shards breaking out of it.
            let mut b = vec![
                c([0.0, 0.0, 0.0], [5.6, 5.0, 5.4], ICE_DEEP),
                c([0.8, 0.8, -0.4], [4.6, 4.6, 4.2], ICE),
                c([-1.0, -0.6, 0.8], [4.0, 3.6, 4.4], ICE_DEEP),
                c([0.2, -1.4, -1.0], [4.0, 3.0, 3.6], ICE),
            ];
            let mut pieces = Vec::new();
            for (k, (y, x)) in [(0.3f32, 0.4f32), (2.2, -0.6), (3.8, 0.9), (5.1, -0.2)].iter().enumerate() {
                let m = Mat4::from_rotation_y(*y) * Mat4::from_rotation_x(*x);
                pieces.push(placed(0, m, spike(2.0, 1.8 - k as f32 * 0.15, 3.4, 3, ICE, ICE)));
            }
            b.push(c([0.0; 3], [3.0, 6.0, 3.0], ICE));
            pieces.insert(0, piece(0, b));
            pieces
        }
        SPORE => {
            // A spore pod: an olive husk with glowing spots, two little tendrils behind.
            let mut b = vec![c([0.0, 0.0, 0.0], [4.0, 4.0, 5.6], SPORE_M), c([0.0, 0.0, 0.0], [5.0, 3.0, 4.0], SPORE_M), c([0.0, 0.0, 0.0], [3.0, 5.0, 4.0], SPORE_M)];
            for p in [[2.1, 0.6, 1.0], [-2.1, -0.4, 0.2], [0.5, 2.1, -1.2], [-0.6, -2.1, 1.4], [0.0, 0.0, 2.9], [1.4, -1.2, -1.8]] {
                b.push(c(p, [1.0, 1.0, 1.0], LEAF_M));
            }
            b.extend(spike(2.6, 1.6, 2.0, 2, SPORE_M, LEAF_M));
            for s in [1.0f32, -1.0] {
                b.push(c([0.8 * s, 0.4, -3.6], [0.5, 0.5, 2.2], LEAF_VEIN));
                b.push(c([1.3 * s, 0.9, -5.0], [0.4, 0.4, 1.4], LEAF_VEIN));
            }
            vec![piece(0, b)]
        }
        SNOWFLAKE => {
            // Six arms in the XY plane, each with two pairs of branches, a hexagonal heart.
            let mut pieces = vec![piece(0, vec![c([0.0; 3], [2.6, 2.6, 1.6], ICE), c([0.0; 3], [1.6, 1.6, 2.4], STAR_CORE)])];
            for k in 0..6 {
                let m = Mat4::from_rotation_z(k as f32 * TAU / 6.0);
                let mut arm = vec![c([0.0, 4.0, 0.0], [1.0, 6.4, 1.0], ICE), c([0.0, 7.6, 0.0], [1.4, 1.4, 1.2], STAR_CORE)];
                for (y, l) in [(3.4f32, 2.4f32), (5.6, 1.6)] {
                    for s in [1.0f32, -1.0] {
                        arm.push(c([s * l * 0.45, y + l * 0.35, 0.0], [l, 0.7, 0.8], ICE));
                    }
                }
                pieces.push(placed(0, m, arm));
            }
            pieces
        }
        SPIRIT => {
            // A violet spirit: a glowing head with dark eyes (0), a flame tail flowing back (0),
            // motes circling it (1..=3).
            let mut b = ball([0.0; 3], 2.4, AMETHYST_LIGHT);
            for s in [1.0f32, -1.0] {
                b.push(c([0.9 * s, 0.4, 2.3], [0.8, 1.2, 0.3], EYE));
            }
            b.push(c([0.0, -0.6, -3.0], [3.4, 3.0, 2.4], AMETHYST_M));
            b.push(c([0.0, -0.2, -5.0], [2.4, 2.2, 2.2], AMETHYST_M));
            b.push(c([0.0, 0.3, -6.8], [1.4, 1.4, 2.0], AMETHYST_LIGHT));
            b.push(c([0.0, 0.8, -8.2], [0.7, 0.7, 1.2], AMETHYST_LIGHT));
            let mut pieces = vec![piece(0, b)];
            for k in 1..=3 {
                pieces.push(piece(k, vec![c([0.0; 3], [0.9, 0.9, 0.9], AMETHYST_LIGHT)]));
            }
            pieces
        }
        FIRE_FLOWER => {
            // A flower of flame: six fiery petals round a white-hot heart, facing the way it
            // flies so it reads from behind (0, a pinwheel), and a few tongues trailing (1).
            let mut pieces = round(0, 2.2, FIRE_CORE);
            for k in 0..6 {
                let m = Mat4::from_rotation_z(k as f32 * TAU / 6.0 + 0.3) * Mat4::from_rotation_x(-PI / 2.0) * Mat4::from_rotation_x(0.35);
                let mut petal = blade(1.2, 3.6, 1.6, 5.0, 4, PETAL);
                petal.push(c([0.0, 0.0, 5.4], [1.2, 1.4, 1.2], FIRE));
                pieces.push(placed(0, m, petal));
            }
            pieces.push(piece(1, vec![c([1.0, 0.6, -3.0], [1.4, 1.4, 2.4], FIRE), c([-0.9, -0.4, -3.6], [1.2, 1.2, 2.6], FIRE_DARK), c([0.2, -0.2, -5.0], [0.9, 0.9, 2.0], FIRE_DARK)]));
            pieces
        }
        WATER_STREAM => {
            // A falling drop of the water stream: round at the front, drawn out behind.
            let mut pieces = round(0, 1.8, WATER);
            pieces.push(placed(0, Mat4::from_rotation_y(PI), spike(0.6, 2.8, 5.0, 3, WATER, FOAM)));
            pieces.push(piece(0, vec![c([0.6, 0.9, 1.1], [0.8, 0.6, 0.6], FOAM)]));
            pieces
        }
        GROUND_THORN => {
            // A thorn of the creeping vine bursting up out of the ground (+Y): a thick twisted
            // stem tapering to a pale point, barbs on its sides, a smaller one leaning beside it.
            let up = |x: f32, z: f32, w: f32, h: f32, steps: usize| -> Vec<IBox> {
                let mut b = Vec::new();
                for k in 0..steps {
                    let t = k as f32 / steps as f32;
                    let seg = h / steps as f32;
                    let s = w * (1.0 - t * 0.8);
                    // A slight twist: each step a little off to one side.
                    let wob = (t * 5.0).sin() * w * 0.12;
                    b.push(c([x + wob, seg * (k as f32 + 0.5), z - wob], [s, seg + 0.2, s], if t > 0.65 { VILE_TIP } else { VILE }));
                }
                b
            };
            let mut main = up(0.0, 0.0, 4.2, 22.0, 9);
            for (y, sx, sz) in [(5.0f32, 1.0f32, 0.0f32), (9.0, -0.6, 0.8), (13.0, 0.4, -1.0), (16.5, -0.8, -0.3)] {
                let w = 4.2 * (1.0 - y / 22.0 * 0.8);
                main.push(c([sx * (w * 0.5 + 0.8), y, sz * (w * 0.5 + 0.8)], [0.8 + sx.abs() * 0.8, 0.7, 0.8 + sz.abs() * 0.8], VILE_TIP));
            }
            // Leaves of the vine at its foot.
            main.push(c([2.6, 0.6, 0.6], [2.6, 0.5, 1.6], LEAF_VEIN));
            main.push(c([-2.2, 0.5, -1.2], [2.0, 0.5, 1.8], LEAF_VEIN));
            let side = Mat4::from_translation(Vec3::new(2.4, 0.0, 1.6)) * Mat4::from_rotation_z(-0.35) * Mat4::from_rotation_x(0.2);
            vec![piece(0, main), placed(0, side, up(0.0, 0.0, 2.4, 11.0, 5))]
        }
        CRYSTAL => {
            // A shard of the crystal storm: a long pink crystal with a cyan heart, two chips.
            let mut b = spike(0.0, 2.6, 5.0, 3, CRYSTAL_PINK, CRYSTAL_CYAN);
            b.extend(spike(0.0, 2.6, 3.0, 2, CRYSTAL_PINK, CRYSTAL_PINK).into_iter().map(|x| IBox { lo: Vec3::new(x.lo.x, x.lo.y, -x.hi.z), hi: Vec3::new(x.hi.x, x.hi.y, -x.lo.z), mat: x.mat }));
            b.push(c([0.0; 3], [1.6, 3.2, 1.6], CRYSTAL_CYAN));
            let chip = Mat4::from_translation(Vec3::new(1.3, 0.4, -0.4)) * Mat4::from_rotation_y(0.7);
            vec![piece(0, b), placed(0, chip, spike(0.0, 1.2, 2.2, 2, CRYSTAL_CYAN, CRYSTAL_PINK))]
        }
        CURSED_FLAME => {
            // A ball of cursed green fire (0, tumbling), a hollow face that keeps looking ahead
            // (2), and a long tail of dark green flame (1).
            let mut lumps = Vec::new();
            for (p, s) in [([2.2, 0.6, 0.0], 2.4), ([-2.1, -0.2, 0.4], 2.2), ([0.2, 2.2, -0.4], 2.4), ([-0.3, -2.2, 0.2], 2.0), ([0.6, 0.0, -2.2], 2.4), ([1.4, 1.6, 1.0], 1.6)] {
                lumps.push(c(p, [s, s, s], CURSED));
            }
            let face = vec![c([1.0, 0.5, 2.5], [1.2, 1.4, 0.6], EYE), c([-1.0, 0.5, 2.5], [1.2, 1.4, 0.6], EYE), c([0.0, -1.1, 2.5], [1.8, 0.6, 0.6], EYE)];
            let tail = vec![
                c([0.0, 0.0, -3.6], [3.4, 3.4, 2.6], CURSED),
                c([0.4, 0.5, -5.8], [2.6, 2.6, 2.4], CURSED_DARK),
                c([-0.3, 0.9, -7.8], [1.8, 1.8, 2.2], CURSED_DARK),
                c([0.2, 1.3, -9.4], [1.0, 1.0, 1.6], CURSED_DARK),
            ];
            let mut pieces = round(0, 2.6, CURSED_CORE);
            pieces.extend([piece(0, lumps), piece(1, tail), piece(2, face)]);
            pieces
        }
        MAGNET => {
            // A crackling ball (0) inside three rings of glass beads and sparks, each turning
            // its own way (1..=3).
            let ring = |r: f32, n: usize, s: f32| -> Vec<IBox> {
                (0..n)
                    .map(|k| {
                        let a = k as f32 / n as f32 * TAU;
                        c([a.cos() * r, 0.0, a.sin() * r], [s, s, s], if k % 4 == 0 { ARC_M } else { MAGNET_GLASS })
                    })
                    .collect()
            };
            let mut pieces = round(0, 2.6, MAGNET_CORE);
            // Bright sparks breaking out of the ball's skin.
            pieces.push(piece(0, vec![c([2.4, 0.8, 0.6], [0.8, 0.8, 0.8], ARC_M), c([-1.0, -2.3, 0.9], [0.8, 0.8, 0.8], ARC_M), c([0.3, 1.2, -2.5], [0.8, 0.8, 0.8], ARC_M)]));
            pieces.extend([piece(1, ring(5.0, 12, 1.2)), piece(2, ring(5.6, 14, 1.0)), piece(3, ring(4.4, 10, 1.1))]);
            pieces
        }
        TYPHOON => {
            // A spinning blade of water lying flat: four curved arms off a foam hub, a bright
            // edge on each arm's leading side.
            let mut b = vec![c([0.0; 3], [3.0, 1.6, 3.0], FOAM)];
            for arm in 0..4 {
                let a0 = arm as f32 * TAU / 4.0;
                for k in 0..7 {
                    let t = k as f32 / 6.0;
                    let a = a0 + t * 1.3;
                    let r = 1.6 + t * 5.0;
                    let w = 2.6 * (1.0 - t * 0.7);
                    b.push(c([a.cos() * r, 0.0, a.sin() * r], [w, 0.9, w], WATER));
                    let (ea, er) = (a + 0.25, r + 0.3);
                    b.push(c([ea.cos() * er, 0.0, ea.sin() * er], [0.8, 0.6, 0.8], TYPHOON_EDGE));
                }
            }
            vec![piece(0, b)]
        }
        LUNAR => {
            // A crescent of moonlight (0) facing ahead and turning, a trail of light (1).
            let mut b = Vec::new();
            let n = 11;
            for k in 0..n {
                let t = k as f32 / (n - 1) as f32;
                let a = -2.3 + t * 4.6;
                let w = 2.8 * (1.0 - (t * 2.0 - 1.0).abs() * 0.75);
                let (x, y) = (a.cos() * 4.0 - 1.0, a.sin() * 4.0);
                b.push(c([x, y, 0.0], [w, w, 2.2], MOON));
                b.push(c([x - w * 0.45, y, 0.0], [0.7, w * 0.8, 2.6], MOON_GLOW));
            }
            let tail = vec![c([1.0, 0.0, -2.6], [2.4, 2.4, 2.4], MOON_GLOW), c([1.0, 0.2, -4.8], [1.6, 1.6, 2.2], MOON_GLOW), c([1.0, 0.4, -6.6], [0.9, 0.9, 1.8], MOON)];
            vec![piece(0, b), piece(1, tail)]
        }
        SHADOW_BEAM => vec![piece(0, vec![c([0.0, 0.0, 0.5], [1.0, 1.0, 1.0], SHADOW_M)])],
        PRISM_BEAM => vec![piece(0, vec![c([0.0, 0.0, 0.5], [1.0, 1.0, 1.0], BEAM_WHITE)])],
        PRISM_GEM => {
            // A triangular prism of glass floating in front of the caster, lying along Z.
            let mut b = Vec::new();
            for k in 0..6 {
                let y = -3.0 + k as f32 * 1.2;
                let w = 6.4 * (1.0 - k as f32 / 6.0);
                b.push(c([0.0, y + 0.6, 0.0], [w, 1.3, 6.0], PRISM_GLASS));
            }
            b.push(c([0.0, -2.4, 0.0], [5.0, 0.4, 6.4], BEAM_WHITE));
            vec![piece(0, b)]
        }
        _ => {
            // ARC: one unit of a lightning stroke, along +Z from 0 to 1; the bones stretch it.
            vec![piece(0, vec![c([0.0, 0.0, 0.5], [1.0, 1.0, 1.0], ARC_M)])]
        }
    }
}

/// A rotation turning +Z toward `d`, Y kept up.
fn face(d: Vec3) -> Quat {
    crate::summons::face(d)
}

/// Where a spell is and how it moves, for its bones.
pub struct Pose {
    /// Relative to the camera's block.
    pub pos: Vec3,
    pub vel: Vec3,
    /// Seconds in flight.
    pub age: f32,
    /// A number of its own (0..1), so two shots don't turn in step.
    pub seed: f32,
    pub scale: f32,
    /// Sky and block light where it is.
    pub light: [f32; 4],
}

fn bones_with(shape: usize, light: [f32; 4]) -> Bones {
    // Mode 4: shines by itself, brighter than a glowing summon.
    let glow = glows(shape);
    let light = if glow { [1.0, 1.0, light[2], light[3]] } else { light };
    // The bee only glows softly (mode 3), enough to follow it at night.
    let mode = if shape == BEE || shape == GROUND_THORN { 3.0 } else if glow { 4.0 } else { 0.0 };
    Bones { light, tint: [1.0, 1.0, 1.0, mode], ..Default::default() }
}

/// Bones of a flying spell of shape `i`.
pub fn bones(i: usize, p: &Pose) -> Bones {
    let mut b = bones_with(i, p.light);
    let t = p.age;
    let k = p.scale / 16.0;
    let dir = if p.vel.length_squared() > 1e-4 { p.vel } else { Vec3::Z };
    let root = Mat4::from_translation(p.pos) * Mat4::from_quat(face(dir)) * Mat4::from_scale(Vec3::splat(k));
    let at = Mat4::from_translation(p.pos) * Mat4::from_scale(Vec3::splat(k));
    let set = |b: &mut Bones, n: usize, m: Mat4| b.m[n] = m.to_cols_array_2d();
    let seed = p.seed * TAU;
    let tumble = |w: f32| Mat4::from_quat(Quat::from_axis_angle(Vec3::new(seed.cos(), 0.6, seed.sin()).normalize(), t * w + seed));
    let lick = |w: f32| Mat4::from_rotation_x((t * w + seed).sin() * 0.12) * Mat4::from_rotation_y((t * w * 1.3 + seed).cos() * 0.12) * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0 + (t * w * 2.1).sin() * 0.12));
    match i {
        FIREBALL => {
            set(&mut b, 0, root * tumble(9.0));
            set(&mut b, 1, root * lick(14.0));
        }
        ICE_SHARD | ICICLE => set(&mut b, 0, root * Mat4::from_rotation_z(t * 4.0 + seed)),
        SPARK => {
            // A new zigzag every 1/20 s.
            let f = (t * 20.0).floor();
            let r = (f * 1.7 + seed).sin() * PI;
            set(&mut b, 0, root * Mat4::from_rotation_z(r) * Mat4::from_scale(Vec3::new(1.0, 1.0, 0.85 + (f * 2.3).cos().abs() * 0.3)));
            let show = if (f as i32 + (p.seed * 7.0) as i32) % 3 == 0 { 0.0 } else { 1.0 };
            set(&mut b, 1, root * Mat4::from_rotation_z(-r * 1.3) * Mat4::from_scale(Vec3::splat(show)));
        }
        AMETHYST => {
            set(&mut b, 0, root * Mat4::from_rotation_z(t * 3.0 + seed) * Mat4::from_rotation_x((t * 2.0).sin() * 0.3));
            for n in 1..=3usize {
                let a = t * 5.0 + n as f32 * TAU / 3.0;
                set(&mut b, n, root * Mat4::from_translation(Vec3::new(a.cos() * 5.0, a.sin() * 5.0, (a * 2.0).sin() * 1.5)));
            }
        }
        LEAF => set(&mut b, 0, root * Mat4::from_rotation_z(t * 7.0 + seed) * Mat4::from_rotation_x((t * 11.0 + seed).sin() * 0.35)),
        STAR => set(&mut b, 0, root * Mat4::from_rotation_z(-t * 12.0 + seed) * Mat4::from_rotation_y((t * 3.0).sin() * 0.4)),
        BIG_FIRE => {
            set(&mut b, 0, root * tumble(5.0));
            set(&mut b, 1, root * Mat4::from_quat(Quat::from_axis_angle(Vec3::new(0.3, 1.0, -0.4).normalize(), -t * 3.5)) * Mat4::from_scale(Vec3::splat(1.0 + (t * 9.0).sin() * 0.06)));
            set(&mut b, 2, root * lick(11.0));
        }
        THORN | RUIN_THORN => set(&mut b, 0, root * Mat4::from_rotation_z(t * 8.0 + seed)),
        CANNONBALL => {
            set(&mut b, 0, root * Mat4::from_rotation_x(t * 8.0));
            set(&mut b, 1, at * Mat4::from_scale(Vec3::splat(1.0 + (t * 30.0).sin() * 0.15)));
        }
        FLAME_PUFF => {
            // It swells and turns as it burns out (a flame puff lives a third of a second).
            let grow = (0.6 + t * 4.5).min(2.2);
            set(&mut b, 0, at * Mat4::from_quat(Quat::from_axis_angle(Vec3::new(seed.sin(), 1.0, seed.cos()).normalize(), seed + t * 6.0)) * Mat4::from_scale(Vec3::splat(grow)));
        }
        METEOR => {
            set(&mut b, 0, root * tumble(4.0));
            set(&mut b, 1, root * lick(16.0) * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0 + p.vel.length() * 0.02)));
        }
        VOID_ORB => {
            let pulse = 1.0 + (t * 4.0).sin() * 0.08;
            set(&mut b, 0, at * Mat4::from_rotation_y(t * 1.2) * Mat4::from_scale(Vec3::splat(pulse)));
            set(&mut b, 1, at * Mat4::from_rotation_x(0.45) * Mat4::from_rotation_y(t * 3.0));
            set(&mut b, 2, at * Mat4::from_rotation_z(-0.6) * Mat4::from_rotation_x(0.2) * Mat4::from_rotation_y(-t * 2.2));
        }
        BEE => {
            let bob = Mat4::from_translation(Vec3::new(0.0, (t * 9.0 + seed).sin() * 0.8, 0.0)) * Mat4::from_rotation_z((t * 7.0 + seed).sin() * 0.25);
            set(&mut b, 0, root * bob);
            let flap = (t * 70.0).sin() * 0.7;
            for (n, s) in [(1usize, 1.0f32), (2, -1.0)] {
                let hinge = Vec3::new(0.4 * s, 1.4, -0.6);
                set(&mut b, n, root * bob * Mat4::from_translation(hinge) * Mat4::from_rotation_z(flap * s) * Mat4::from_translation(-hinge));
            }
        }
        WATER_ORB => {
            let w = (t * 10.0 + seed).sin() * 0.12;
            set(&mut b, 0, root * Mat4::from_scale(Vec3::new(1.0 + w, 1.0 - w, 1.0 + w * 0.5)) * Mat4::from_rotation_z(t * 2.0));
            for n in 1..=3usize {
                let a = t * 4.0 + n as f32 * 2.1 + seed;
                set(&mut b, n, root * Mat4::from_translation(Vec3::new(a.cos() * 4.2, (a * 1.7).sin() * 3.0, a.sin() * 4.2 - 1.5)));
            }
        }
        SCYTHE => {
            // A saw blade tipped up halfway toward the caster, so it shows its face from behind
            // and from the side, spinning fast.
            set(&mut b, 0, root * Mat4::from_rotation_x(-0.85) * Mat4::from_rotation_z(0.25) * Mat4::from_rotation_y(-t * 16.0 - seed));
        }
        BLIGHT => {
            let w = (t * 8.0 + seed).sin() * 0.08;
            set(&mut b, 0, root * Mat4::from_rotation_z(t * 2.5) * Mat4::from_scale(Vec3::new(1.0 + w, 1.0 - w, 1.0 + w)));
            set(&mut b, 1, at * Mat4::from_scale(Vec3::new(1.0, 1.0 + (t * 6.0).sin().abs() * 0.3, 1.0)));
        }
        SUN => {
            set(&mut b, 0, at * Mat4::from_rotation_y(t * 1.5) * Mat4::from_rotation_x(t * 0.7));
            let face_it = Mat4::from_quat(face(dir));
            set(&mut b, 1, at * face_it * Mat4::from_rotation_z(t * 2.0) * Mat4::from_scale(Vec3::splat(1.0 + (t * 7.0).sin() * 0.1)));
            set(&mut b, 2, at * face_it * Mat4::from_rotation_z(-t * 1.4) * Mat4::from_rotation_y(t * 0.9));
        }
        LASER => set(&mut b, 0, root * Mat4::from_rotation_z(t * 9.0)),
        FROST_BOULDER => set(&mut b, 0, root * tumble(6.0)),
        SPORE => set(&mut b, 0, root * Mat4::from_rotation_z(t * 3.0 + seed) * Mat4::from_scale(Vec3::splat(1.0 + (t * 12.0).sin() * 0.05))),
        SNOWFLAKE => set(&mut b, 0, root * Mat4::from_rotation_z(t * 5.0 + seed) * Mat4::from_rotation_x((t * 2.0).sin() * 0.3)),
        SPIRIT => {
            set(&mut b, 0, root * Mat4::from_translation(Vec3::new(0.0, (t * 4.0 + seed).sin() * 0.8, 0.0)) * Mat4::from_rotation_z((t * 3.0).sin() * 0.2));
            for n in 1..=3usize {
                let a = -t * 4.0 + n as f32 * TAU / 3.0;
                set(&mut b, n, root * Mat4::from_translation(Vec3::new(a.cos() * 4.0, a.sin() * 4.0, -1.0)));
            }
        }
        FIRE_FLOWER => {
            set(&mut b, 0, root * Mat4::from_rotation_z(t * 9.0 + seed) * Mat4::from_rotation_x((t * 5.0 + seed).sin() * 0.25));
            set(&mut b, 1, root * lick(15.0));
        }
        WATER_STREAM => set(&mut b, 0, root * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0 + p.vel.length() * 0.03))),
        GROUND_THORN => {
            // `age` is how far it has risen (0..1); it leans the way the vine creeps.
            let h = Vec3::new(p.vel.x, 0.0, p.vel.z).normalize_or(Vec3::X);
            let lean = Mat4::from_quat(Quat::from_axis_angle(h.cross(Vec3::NEG_Y).normalize_or(Vec3::Z), 0.3));
            let rise = t.clamp(0.0, 1.0);
            set(&mut b, 0, at * lean * Mat4::from_rotation_y(seed) * Mat4::from_scale(Vec3::new(0.6 + rise * 0.4, rise.max(0.02), 0.6 + rise * 0.4)));
        }
        CRYSTAL => set(&mut b, 0, root * tumble(11.0)),
        CURSED_FLAME => {
            set(&mut b, 0, root * tumble(8.0));
            set(&mut b, 1, root * lick(13.0));
            set(&mut b, 2, root * Mat4::from_translation(Vec3::new(0.0, (t * 6.0).sin() * 0.3, 0.0)));
        }
        MAGNET => {
            let pulse = 1.0 + (t * 14.0 + seed).sin() * 0.1;
            set(&mut b, 0, at * Mat4::from_rotation_y(t * 4.0) * Mat4::from_scale(Vec3::splat(pulse)));
            set(&mut b, 1, at * Mat4::from_rotation_x(0.5) * Mat4::from_rotation_y(t * 5.0));
            set(&mut b, 2, at * Mat4::from_rotation_z(1.1) * Mat4::from_rotation_y(-t * 4.0));
            set(&mut b, 3, at * Mat4::from_rotation_x(-1.0) * Mat4::from_rotation_z(0.6) * Mat4::from_rotation_y(t * 6.0));
        }
        TYPHOON => set(&mut b, 0, root * Mat4::from_rotation_x(-0.6) * Mat4::from_rotation_y(-t * 22.0 - seed)),
        LUNAR => {
            set(&mut b, 0, root * Mat4::from_rotation_z(t * 3.0 + seed));
            set(&mut b, 1, root * lick(10.0));
        }
        PRISM_GEM => set(&mut b, 0, at * Mat4::from_rotation_y(t * 1.6) * Mat4::from_rotation_x(0.35) * Mat4::from_rotation_z((t * 2.0).sin() * 0.15)),
        _ => set(&mut b, 0, root),
    }
    b
}

/// Bones of one stroke of a beam of shape `shape` (ARC lightning, SHADOW_BEAM, PRISM_BEAM)
/// from `a` to `b` (camera-block relative), `th` blocks thick; `tint` dyes it (prism).
pub fn arc_bones(shape: usize, a: Vec3, b: Vec3, th: f32, tint: Option<[f32; 3]>) -> Bones {
    let mut bo = bones_with(shape, [1.0, 1.0, 0.0, 0.0]);
    if let Some(t) = tint {
        // Mode 5: dyed and shining by itself (entity.wgsl).
        bo.tint = [t[0], t[1], t[2], 5.0];
    } else if shape != ARC {
        bo.tint[3] = 4.0;
    }
    let d = b - a;
    let m = Mat4::from_translation(a) * Mat4::from_quat(face(d)) * Mat4::from_scale(Vec3::new(th, th, d.length().max(0.01)));
    bo.m[0] = m.to_cols_array_2d();
    bo
}

/// The spell skin: 32 material swatches of 16 × 16 model pixels, `scale` texels each.
pub fn paint(scale: u32, page: usize) -> Vec<u8> {
    let (w, h) = (64 * scale, 128 * scale);
    let mut px = vec![0u8; (w * h * 4) as usize];
    let hash = |x: u32, y: u32, s: u32| {
        let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ s.wrapping_mul(0x1656_67B1);
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        (v & 0xffff) as f32 / 65535.0
    };
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    };
    let n = 16 * scale;
    let first = page as u32 * 32;
    for m in first..(first + 32).min(MATERIALS as u32) {
        let (ox, oy) = (((m - first) % 4) * n, ((m - first) / 4) * n);
        for y in 0..n {
            for x in 0..n {
                let (u, v) = (x as f32 / scale as f32, y as f32 / scale as f32);
                let r = hash(x, y, m);
                // One value per model pixel, for chunky blotches.
                let r1 = hash(u as u32, v as u32, m + 77);
                let wave = |a: f32, b: f32| (u * a + v * b).sin() * 0.5 + 0.5;
                let col: [f32; 3] = match m as u8 {
                    FIRE_CORE => mix([255.0, 252.0, 210.0], [255.0, 226.0, 120.0], r1 * 0.5 + r * 0.2),
                    FIRE => mix([255.0, 170.0, 40.0], [240.0, 96.0, 20.0], r1 * 0.5 + wave(0.9, 1.4) * 0.4),
                    FIRE_DARK => mix([214.0, 60.0, 18.0], [150.0, 30.0, 14.0], r1 * 0.6 + r * 0.2),
                    ICE => {
                        let streak = ((u * 0.6 - v * 0.9).sin()).abs() < 0.1;
                        if streak { [250.0, 255.0, 255.0] } else { mix([200.0, 240.0, 255.0], [150.0, 214.0, 248.0], r1 * 0.5 + r * 0.2) }
                    }
                    ICE_DEEP => mix([60.0, 140.0, 230.0], [110.0, 190.0, 250.0], wave(0.5, 0.9) * 0.5 + r1 * 0.3),
                    SPARK_M => mix([255.0, 250.0, 200.0], [255.0, 230.0, 110.0], r1 * 0.6),
                    AMETHYST_M => {
                        let facet = ((u + v) / 4.0).fract() < 0.18;
                        if facet { [220.0, 160.0, 255.0] } else { mix([150.0, 70.0, 220.0], [110.0, 40.0, 180.0], r1 * 0.6 + r * 0.2) }
                    }
                    AMETHYST_LIGHT => mix([240.0, 200.0, 255.0], [210.0, 150.0, 255.0], r1 * 0.5),
                    LEAF_M => mix([90.0, 220.0, 80.0], [50.0, 170.0, 50.0], r1 * 0.5 + wave(0.6, 0.3) * 0.3),
                    LEAF_VEIN => mix([40.0, 110.0, 34.0], [26.0, 80.0, 26.0], r * 0.5),
                    STAR_M => {
                        let glint = r > 0.93;
                        if glint { [255.0, 255.0, 240.0] } else { mix([255.0, 210.0, 70.0], [240.0, 170.0, 40.0], r1 * 0.5 + v / 16.0 * 0.3) }
                    }
                    STAR_CORE => mix([255.0, 255.0, 236.0], [255.0, 240.0, 170.0], r1 * 0.5),
                    WOOD => {
                        let grain = ((v + (u * 0.4).sin() * 0.7) * 2.3).sin() * 0.5 + 0.5;
                        mix([120.0, 140.0, 60.0], [80.0, 92.0, 40.0], grain * 0.6 + r * 0.2)
                    }
                    IRON_BALL => {
                        let shine = wave(0.3, 0.5);
                        mix([46.0, 46.0, 54.0], [96.0, 96.0, 104.0], shine * 0.5 + r * 0.2)
                    }
                    ROCK => {
                        // Dark rock with glowing veins running through it.
                        let crack = ((u * 0.9 + (v * 0.7).sin() * 2.0).sin() * (v * 0.8 + (u * 0.5).cos() * 1.6).sin()).abs() < 0.12;
                        if crack { mix([255.0, 140.0, 30.0], [255.0, 210.0, 80.0], r) } else { mix([70.0, 54.0, 50.0], [40.0, 32.0, 32.0], r1 * 0.6 + r * 0.2) }
                    }
                    VOID => {
                        let speck = r > 0.96;
                        if speck { [200.0, 150.0, 255.0] } else { mix([22.0, 8.0, 36.0], [44.0, 16.0, 70.0], r1 * 0.5 + wave(0.4, 0.7) * 0.3) }
                    }
                    VOID_RIM => mix([150.0, 60.0, 240.0], [100.0, 30.0, 200.0], r1 * 0.6),
                    BEE_YELLOW => {
                        let fuzz = r > 0.8;
                        let c = mix([246.0, 196.0, 40.0], [220.0, 160.0, 24.0], r1 * 0.6);
                        if fuzz { mix(c, [255.0, 230.0, 130.0], 0.5) } else { c }
                    }
                    BEE_BLACK => mix([26.0, 22.0, 18.0], [56.0, 48.0, 40.0], wave(0.5, 0.8) * 0.5 + r * 0.2),
                    WING => {
                        let vein = (u * 0.9).fract() < 0.1;
                        if vein { [120.0, 120.0, 130.0] } else { mix([220.0, 236.0, 246.0], [236.0, 226.0, 246.0], wave(0.4, 0.3) * 0.7) }
                    }
                    WATER => {
                        let ripple = ((u * 0.8 + (v * 0.6).sin() * 1.5).sin()).abs() < 0.12;
                        if ripple { [150.0, 210.0, 255.0] } else { mix([40.0, 110.0, 230.0], [70.0, 150.0, 250.0], wave(0.5, 0.7) * 0.5 + r1 * 0.3) }
                    }
                    FOAM => mix([235.0, 248.0, 255.0], [200.0, 230.0, 255.0], r1 * 0.5),
                    SCYTHE_M => {
                        // Dark purple metal with a lighter sheen across it.
                        let sheen = wave(0.35, 0.55);
                        mix([70.0, 24.0, 110.0], [140.0, 60.0, 200.0], sheen * 0.6 + r * 0.15)
                    }
                    SCYTHE_EDGE => mix([255.0, 150.0, 255.0], [240.0, 90.0, 255.0], r1 * 0.5),
                    RUIN => mix([60.0, 14.0, 24.0], [100.0, 24.0, 40.0], r1 * 0.5 + wave(0.8, 1.2) * 0.3),
                    BLIGHT_M => {
                        let bubble = r1 > 0.85;
                        if bubble { [210.0, 255.0, 120.0] } else { mix([140.0, 200.0, 40.0], [100.0, 160.0, 30.0], r * 0.4 + wave(0.7, 0.9) * 0.4) }
                    }
                    BLIGHT_DARK => mix([60.0, 90.0, 20.0], [40.0, 64.0, 14.0], r * 0.5),
                    SUN_M => mix([255.0, 214.0, 80.0], [255.0, 170.0, 40.0], r1 * 0.5 + wave(1.0, 0.6) * 0.3),
                    LASER_M => mix([255.0, 90.0, 50.0], [255.0, 50.0, 30.0], r1 * 0.4 + wave(0.0, 1.6) * 0.3),
                    SPORE_M => {
                        let ridge = ((v * 1.2).sin() * 0.5 + 0.5) * 0.3;
                        mix([120.0, 120.0, 60.0], [80.0, 84.0, 40.0], ridge + r1 * 0.4)
                    }
                    EYE => mix([20.0, 14.0, 30.0], [40.0, 30.0, 50.0], r),
                    VILE => {
                        // Dark purple bark with sickly green streaks along it.
                        let streak = ((u * 0.7 + (v * 0.5).sin() * 1.4).sin()).abs() < 0.1;
                        if streak { [90.0, 140.0, 60.0] } else { mix([60.0, 24.0, 80.0], [92.0, 40.0, 116.0], r1 * 0.5 + wave(0.3, 1.1) * 0.3) }
                    }
                    VILE_TIP => mix([200.0, 150.0, 230.0], [160.0, 100.0, 200.0], r1 * 0.5 + v / 16.0 * 0.3),
                    CRYSTAL_PINK => {
                        let facet = ((u - v) / 4.0).fract() < 0.2;
                        if facet { [255.0, 220.0, 250.0] } else { mix([240.0, 110.0, 200.0], [200.0, 70.0, 170.0], r1 * 0.5 + r * 0.2) }
                    }
                    CRYSTAL_CYAN => mix([170.0, 250.0, 255.0], [90.0, 210.0, 240.0], r1 * 0.5 + wave(0.6, 0.4) * 0.3),
                    CURSED => mix([140.0, 240.0, 60.0], [80.0, 190.0, 30.0], r1 * 0.5 + wave(0.9, 1.4) * 0.4),
                    CURSED_CORE => mix([240.0, 255.0, 190.0], [210.0, 255.0, 130.0], r1 * 0.5 + r * 0.2),
                    CURSED_DARK => mix([40.0, 110.0, 20.0], [20.0, 70.0, 14.0], r1 * 0.6 + r * 0.2),
                    MAGNET_GLASS => {
                        let shine = wave(0.5, 0.3) > 0.85;
                        if shine { [150.0, 240.0, 255.0] } else { mix([30.0, 150.0, 210.0], [20.0, 110.0, 180.0], r1 * 0.5) }
                    }
                    MAGNET_CORE => mix([120.0, 230.0, 255.0], [60.0, 190.0, 240.0], r1 * 0.5 + r * 0.2),
                    SHADOW_M => mix([150.0, 80.0, 255.0], [110.0, 50.0, 220.0], r1 * 0.4 + wave(0.0, 1.6) * 0.3),
                    TYPHOON_EDGE => mix([230.0, 250.0, 255.0], [190.0, 230.0, 255.0], r1 * 0.5),
                    MOON => {
                        let crater = r1 > 0.86;
                        if crater { [150.0, 175.0, 215.0] } else { mix([225.0, 236.0, 255.0], [190.0, 210.0, 245.0], r * 0.4 + wave(0.4, 0.6) * 0.3) }
                    }
                    MOON_GLOW => mix([200.0, 235.0, 255.0], [160.0, 210.0, 255.0], r1 * 0.5),
                    BEAM_WHITE => mix([255.0, 255.0, 255.0], [235.0, 235.0, 240.0], r * 0.5),
                    PRISM_GLASS => {
                        let edge = ((u + v * 0.5) / 5.0).fract() < 0.12;
                        if edge { [255.0, 255.0, 255.0] } else { mix([210.0, 220.0, 250.0], [180.0, 170.0, 240.0], r1 * 0.4 + wave(0.5, 0.8) * 0.4) }
                    }
                    PETAL => mix([255.0, 120.0, 30.0], [220.0, 60.0, 20.0], r1 * 0.4 + v / 16.0 * 0.4),
                    _ => mix([230.0, 250.0, 255.0], [170.0, 220.0, 255.0], r1 * 0.5),
                };
                let i = (((oy + y) * w + ox + x) * 4) as usize;
                px[i] = col[0].clamp(0.0, 255.0) as u8;
                px[i + 1] = col[1].clamp(0.0, 255.0) as u8;
                px[i + 2] = col[2].clamp(0.0, 255.0) as u8;
                px[i + 3] = 255;
            }
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_has_boxes_on_free_bones() {
        for i in 0..COUNT {
            let m = model(i);
            assert!(!m.is_empty(), "shape {i}");
            for p in &m {
                assert!((p.part as usize) < crate::entity::MOUTH_PART, "shape {i} part {}", p.part);
                for b in &p.boxes {
                    assert!(b.lo.cmple(b.hi).all(), "shape {i}: inverted box");
                    assert!(b.mat < MATERIALS);
                }
            }
            let p = Pose { pos: Vec3::ONE, vel: Vec3::new(3.0, -1.0, 2.0), age: 0.7, seed: 0.3, scale: 1.0, light: [1.0, 0.0, 0.0, 0.0] };
            assert!(bones(i, &p).m.iter().flatten().flatten().all(|v| v.is_finite()), "shape {i}");
        }
    }

    #[test]
    fn spells_look_different() {
        // Every caster that throws something has its own shape (a few share one on purpose:
        // the frost staff and the ice spike staff both throw icicles).
        let items = ["ruby_wand", "sapphire_wand", "topaz_wand", "amethyst_wand", "emerald_wand", "fire_staff", "guardian_staff", "flame_tome", "meteor_staff", "void_tome", "hive_staff", "water_bolt", "demon_scythe", "ruin_thorn", "blight_rod", "sun_stone_staff", "meteor_blaster", "colossus_staff", "star_tome", "worm_staff", "blizzard_staff", "flower_of_fire", "aqua_scepter", "creeping_thorn", "crystal_storm", "cursed_flames", "magnet_sphere", "shadow_beam", "razorblade_typhoon", "lunar_flare", "prism"];
        let mut seen = std::collections::HashSet::new();
        for it in items {
            assert!(seen.insert(shape_of(it, 9).0), "{it} shares a shape");
        }
    }
}
