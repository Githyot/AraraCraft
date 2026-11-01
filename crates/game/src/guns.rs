//! The gunner's guns as real 3D models (HP 2026-11-01, "analise todos" for the atirador class).
//!
//! Before this every gun was its flat item picture held up like a sign. Now each gun is built
//! from boxes here, our own art painted in code into a skin of 32 materials, and drawn with the
//! mob pipeline: bone 0 the frame, bone 1 what slides back when it fires (slide, hammer, bolt),
//! bone 2 what spins (a revolver's cylinder, a minigun's barrels, a grenade drum, a clockwork
//! gear) and bone 3 a shotgun's pump.
//!
//! Gun space: model pixels, the fist's grip at the origin, +Z out of the muzzle, +Y up, +X to
//! the gun's right. `frame` says where its muzzle, the left hand's grip and the ejection port
//! are, so the holds (player_model.rs), the muzzle flash, the tracers and the shell casings all
//! come from the right place.
//!
//! Families follow Terraria's guns adapted to a 3D hand: a flintlock with its pan and hammer, a
//! revolver whose cylinder turns a chamber each shot, a modern pistol whose slide kicks back,
//! long guns with stocks to the shoulder held in both hands, double and quad barrels, pump
//! shotguns, sub-machine guns, the living sharks (fin, tail, eye and teeth), miniguns with six
//! barrels that spin up, launchers, a flamethrower with its tank and pilot flame, a reed
//! blowpipe, and the magic-tech ones (star cannon, coin gun, elf gun, bubble, laser, vortex).

use glam::{Mat4, Quat, Vec3};

use crate::entity::Bones;
use crate::instruments::{IBox, Piece};

// ---- Materials: the 32 swatches of the gun skin (4 across, 8 down, 16 px each) ----
pub const BLUED: u8 = 0;
pub const STEEL: u8 = 1;
pub const GUNMETAL: u8 = 2;
pub const IRON: u8 = 3;
pub const BRASS: u8 = 4;
pub const GOLD: u8 = 5;
pub const COPPER: u8 = 6;
pub const ADAMANT: u8 = 7;
pub const COBALT: u8 = 8;
pub const MAGMA: u8 = 9;
pub const VERDITE: u8 = 10;
pub const LUMITE: u8 = 11;
pub const ONYX: u8 = 12;
pub const SHARK: u8 = 13;
pub const BELLY: u8 = 14;
pub const BONE: u8 = 15;
pub const WALNUT: u8 = 16;
pub const OAK: u8 = 17;
pub const REED: u8 = 18;
pub const LEATHER: u8 = 19;
pub const POLYMER: u8 = 20;
pub const RED_PAINT: u8 = 21;
pub const PINE: u8 = 22;
pub const CERAMIC: u8 = 23;
pub const GLASS: u8 = 24;
pub const BORE: u8 = 25;
pub const VORTEX: u8 = 26;
pub const ENAMEL: u8 = 27;
/// The bottom row's first three swatches are light themselves (entity.rs sets bit 16 on the
/// gun skin layers): a pilot flame or hot metal, cyan energy, violet energy.
pub const EMBER: u8 = 28;
pub const CYAN: u8 = 29;
pub const VIOLET: u8 = 30;
pub const BUBBLE: u8 = 31;
pub const MATERIALS: u8 = 32;

/// How the finish of each material shines (material.rs classes): one skin layer per class, so
/// metal glints and wood stays matte on the same gun.
pub fn class_of(mat: u8) -> u32 {
    use crate::material as m;
    match mat {
        BLUED | STEEL | GUNMETAL | IRON | ADAMANT | COBALT => m::IRON,
        BRASS | GOLD => m::GOLD,
        COPPER => m::COPPER,
        MAGMA | ONYX => m::NETHERITE,
        VERDITE | GLASS => m::GEM,
        LUMITE | VORTEX | ENAMEL => m::CELESTIAL,
        WALNUT | OAK | REED => m::WOOD,
        LEATHER => m::LEATHER,
        SHARK | BELLY | BONE => m::BONE,
        POLYMER | RED_PAINT | PINE | CERAMIC | BUBBLE => m::LACQUER,
        _ => m::NONE,
    }
}

/// The material classes in use, in skin-layer order.
pub const CLASSES: [u32; 11] = [
    crate::material::IRON,
    crate::material::GOLD,
    crate::material::COPPER,
    crate::material::NETHERITE,
    crate::material::GEM,
    crate::material::CELESTIAL,
    crate::material::WOOD,
    crate::material::LEATHER,
    crate::material::BONE,
    crate::material::LACQUER,
    crate::material::NONE,
];

/// Which of `CLASSES` (and so which gun skin layer) a material's faces use.
pub fn layer_of(mat: u8) -> usize {
    let c = class_of(mat);
    CLASSES.iter().position(|&k| k == c).unwrap_or(CLASSES.len() - 1)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fam {
    Flintlock,
    Revolver,
    /// A modern pistol: slide, frame, polymer grip.
    Pistol,
    /// A long revolver with a heavy top rib (the Venus Magnum).
    Magnum,
    Musket,
    Rifle,
    Sniper,
    DoubleBarrel,
    Quad,
    Pump,
    Smg,
    /// A compact machine pistol with the magazine in the grip.
    Uzi,
    Shark,
    MegaShark,
    Minigun,
    Clockwork,
    Rocket,
    GrenadeLauncher,
    Flamethrower,
    Blowpipe,
    StarCannon,
    CoinGun,
    ElfGun,
    Bubble,
    Laser,
    Vortex,
}

/// Which gunshot a gun makes (sfx_extra.rs `Fx::Gun`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shot {
    Flintlock = 0,
    Pistol = 1,
    Revolver = 2,
    Musket = 3,
    Shotgun = 4,
    Rifle = 5,
    Auto = 6,
    Minigun = 7,
    Rocket = 8,
    Grenade = 9,
    Flame = 10,
    Blowpipe = 11,
    Energy = 12,
    Bubble = 13,
    Star = 14,
    Coin = 15,
    /// An explosive bullet's little blast.
    Pop = 16,
}

pub struct Gun {
    pub item: &'static str,
    pub fam: Fam,
    /// Main metal, body (stock, frame, skin), trim, and glow.
    pub metal: u8,
    pub body: u8,
    pub trim: u8,
    pub glow: u8,
    /// How hard it kicks (0..1.5): a pistol 0.5, a shotgun 1.2.
    pub kick: f32,
}

const fn g(item: &'static str, fam: Fam, metal: u8, body: u8, trim: u8, glow: u8, kick: f32) -> Gun {
    Gun { item, fam, metal, body, trim, glow, kick }
}

/// Every gun with a model; the index is its model (and mesh) number.
pub const GUNS: &[Gun] = &[
    g("flintlock_pistol", Fam::Flintlock, IRON, WALNUT, BRASS, EMBER, 0.6),
    g("musket", Fam::Musket, BLUED, WALNUT, BRASS, EMBER, 1.0),
    g("boomstick", Fam::DoubleBarrel, BLUED, OAK, STEEL, EMBER, 1.2),
    g("rapid_gun", Fam::Smg, GUNMETAL, POLYMER, BRASS, EMBER, 0.25),
    g("blowpipe", Fam::Blowpipe, REED, REED, LEATHER, EMBER, 0.15),
    g("minishark", Fam::Shark, SHARK, BELLY, GUNMETAL, EMBER, 0.2),
    g("megashark", Fam::MegaShark, SHARK, BELLY, GUNMETAL, CYAN, 0.25),
    g("star_cannon", Fam::StarCannon, ENAMEL, GOLD, GOLD, EMBER, 0.7),
    g("flamethrower", Fam::Flamethrower, GUNMETAL, RED_PAINT, BRASS, EMBER, 0.1),
    g("sniper_rifle", Fam::Sniper, GUNMETAL, POLYMER, STEEL, CYAN, 1.3),
    g("coin_gun", Fam::CoinGun, GOLD, WALNUT, BRASS, EMBER, 0.3),
    g("elf_gun", Fam::ElfGun, PINE, RED_PAINT, GOLD, EMBER, 0.3),
    g("adamantite_rifle", Fam::Rifle, ADAMANT, POLYMER, STEEL, EMBER, 0.8),
    g("laser_rifle", Fam::Laser, CERAMIC, GUNMETAL, STEEL, VIOLET, 0.4),
    g("bubble_rifle", Fam::Bubble, COPPER, BUBBLE, BRASS, CYAN, 0.3),
    g("vortex_beater", Fam::Vortex, VORTEX, GUNMETAL, LUMITE, CYAN, 0.3),
    g("rocket_launcher", Fam::Rocket, GUNMETAL, PINE, STEEL, EMBER, 1.1),
    g("grenade_launcher", Fam::GrenadeLauncher, GUNMETAL, POLYMER, BRASS, EMBER, 0.9),
    // The guns that were missing (HP 2026-11-01): one or more for every stage.
    g("revolver", Fam::Revolver, STEEL, WALNUT, BRASS, EMBER, 0.7),
    g("handgun", Fam::Pistol, GUNMETAL, POLYMER, STEEL, EMBER, 0.5),
    g("phoenix_blaster", Fam::Pistol, MAGMA, POLYMER, GOLD, EMBER, 0.55),
    g("quad_barrel_shotgun", Fam::Quad, BLUED, WALNUT, BONE, EMBER, 1.4),
    g("shotgun", Fam::Pump, COBALT, POLYMER, STEEL, EMBER, 1.2),
    g("clockwork_rifle", Fam::Clockwork, BRASS, WALNUT, COPPER, EMBER, 0.45),
    g("uzi", Fam::Uzi, GUNMETAL, POLYMER, STEEL, EMBER, 0.25),
    g("onyx_blaster", Fam::Pump, ONYX, POLYMER, GUNMETAL, VIOLET, 1.2),
    g("tactical_shotgun", Fam::Pump, GUNMETAL, POLYMER, VERDITE, EMBER, 1.1),
    g("venus_magnum", Fam::Magnum, VERDITE, WALNUT, GOLD, EMBER, 0.9),
    g("chain_gun", Fam::Minigun, GUNMETAL, POLYMER, BRASS, EMBER, 0.2),
    g("lumite_minigun", Fam::Minigun, LUMITE, GUNMETAL, VORTEX, CYAN, 0.2),
];

pub const COUNT: usize = GUNS.len();

pub fn index(name: &str) -> Option<usize> {
    GUNS.iter().position(|g| g.item == name)
}

/// The gunshot of gun `i`.
pub fn shot(i: usize) -> Shot {
    let gun = &GUNS[i];
    match gun.fam {
        Fam::Flintlock => Shot::Flintlock,
        Fam::Pistol => Shot::Pistol,
        Fam::Revolver | Fam::Magnum => Shot::Revolver,
        Fam::Musket => Shot::Musket,
        Fam::DoubleBarrel | Fam::Quad | Fam::Pump => Shot::Shotgun,
        Fam::Rifle | Fam::Sniper | Fam::Clockwork => Shot::Rifle,
        Fam::Smg | Fam::Uzi | Fam::Shark => Shot::Auto,
        Fam::MegaShark | Fam::Minigun => Shot::Minigun,
        Fam::Rocket => Shot::Rocket,
        Fam::GrenadeLauncher => Shot::Grenade,
        Fam::Flamethrower => Shot::Flame,
        Fam::Blowpipe => Shot::Blowpipe,
        Fam::Laser | Fam::Vortex => Shot::Energy,
        Fam::Bubble => Shot::Bubble,
        Fam::StarCannon => Shot::Star,
        Fam::CoinGun => Shot::Coin,
        Fam::ElfGun => Shot::Auto,
    }
}

/// Held in both hands (a long gun) or one (pistols).
pub fn two_handed(i: usize) -> bool {
    !matches!(GUNS[i].fam, Fam::Flintlock | Fam::Revolver | Fam::Pistol | Fam::Magnum | Fam::Uzi)
}

/// Throws out a brass case after each shot.
pub fn ejects(i: usize) -> bool {
    matches!(GUNS[i].fam, Fam::Pistol | Fam::Rifle | Fam::Sniper | Fam::Pump | Fam::Smg | Fam::Uzi | Fam::MegaShark | Fam::Minigun | Fam::Clockwork)
}

/// Black powder: a thick cloud of smoke after each shot.
pub fn black_powder(i: usize) -> bool {
    matches!(GUNS[i].fam, Fam::Flintlock | Fam::Musket | Fam::DoubleBarrel | Fam::Quad)
}

/// A minigun winds up before it fires at full speed.
pub fn spins_up(i: usize) -> bool {
    GUNS[i].fam == Fam::Minigun
}

/// Shots per pull of the trigger (the clockwork rifle fires bursts of three).
pub fn burst(name: &str) -> u8 {
    if name == "clockwork_rifle" {
        3
    } else {
        1
    }
}

/// Where things are on a gun (gun space, model pixels).
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub muzzle: Vec3,
    /// The left hand's grip on a long gun.
    pub fore: Option<Vec3>,
    /// Where the spent cases fly out.
    pub eject: Vec3,
    /// The axis the spinning bone turns about.
    pub spin_at: Vec3,
}

/// The muzzle, foregrip and ejection port of gun `i`.
pub fn frame(i: usize) -> Frame {
    let fam = GUNS[i].fam;
    let (muzzle, fore, eject, spin_at) = match fam {
        Fam::Flintlock => (Vec3::new(0.0, 2.6, 13.4), None, Vec3::new(1.0, 2.4, 0.5), Vec3::ZERO),
        Fam::Revolver => (Vec3::new(0.0, 2.7, 12.0), None, Vec3::new(1.0, 2.0, 2.0), Vec3::new(0.0, 1.9, 0.0)),
        Fam::Magnum => (Vec3::new(0.0, 2.9, 14.2), None, Vec3::new(1.0, 2.0, 2.0), Vec3::new(0.0, 1.9, 0.0)),
        Fam::Pistol => (Vec3::new(0.0, 2.5, 9.4), None, Vec3::new(1.2, 3.0, 2.5), Vec3::ZERO),
        Fam::Uzi => (Vec3::new(0.0, 2.2, 9.6), None, Vec3::new(1.4, 2.8, 2.0), Vec3::ZERO),
        Fam::Musket => (Vec3::new(0.0, 2.4, 27.2), Some(Vec3::new(0.0, 0.4, 10.0)), Vec3::new(1.0, 2.4, 0.5), Vec3::ZERO),
        Fam::Rifle => (Vec3::new(0.0, 2.2, 21.2), Some(Vec3::new(0.0, 0.3, 10.0)), Vec3::new(1.4, 2.6, 1.5), Vec3::ZERO),
        Fam::Sniper => (Vec3::new(0.0, 2.2, 27.8), Some(Vec3::new(0.0, 0.3, 11.0)), Vec3::new(1.4, 2.6, 1.5), Vec3::ZERO),
        Fam::Clockwork => (Vec3::new(0.0, 2.2, 20.2), Some(Vec3::new(0.0, 0.3, 9.5)), Vec3::new(1.4, 2.6, 1.5), Vec3::new(1.6, 1.2, 2.0)),
        Fam::DoubleBarrel => (Vec3::new(0.0, 2.2, 19.4), Some(Vec3::new(0.0, 0.5, 7.0)), Vec3::new(1.0, 2.4, 0.5), Vec3::ZERO),
        Fam::Quad => (Vec3::new(0.0, 2.4, 13.4), Some(Vec3::new(0.0, 0.4, 6.0)), Vec3::new(1.0, 2.4, 0.5), Vec3::ZERO),
        Fam::Pump => (Vec3::new(0.0, 2.4, 20.4), Some(Vec3::new(0.0, -0.2, 9.0)), Vec3::new(1.4, 2.4, 1.5), Vec3::ZERO),
        Fam::Smg => (Vec3::new(0.0, 2.2, 14.4), Some(Vec3::new(0.0, -0.6, 7.0)), Vec3::new(1.4, 2.6, 2.0), Vec3::ZERO),
        Fam::Shark => (Vec3::new(0.0, 2.0, 18.0), Some(Vec3::new(0.0, -0.8, 8.0)), Vec3::new(1.8, 2.5, 3.0), Vec3::ZERO),
        Fam::MegaShark => (Vec3::new(0.0, 2.2, 21.5), Some(Vec3::new(0.0, -1.0, 9.0)), Vec3::new(2.4, 2.8, 3.0), Vec3::new(0.0, 2.2, 0.0)),
        Fam::Minigun => (Vec3::new(0.0, 2.2, 23.0), Some(Vec3::new(-3.4, 1.8, 4.0)), Vec3::new(3.0, 1.0, 0.0), Vec3::new(0.0, 2.2, 0.0)),
        Fam::Rocket => (Vec3::new(0.0, 3.2, 16.5), Some(Vec3::new(0.0, -0.4, 7.0)), Vec3::new(0.0, 3.2, -12.0), Vec3::ZERO),
        Fam::GrenadeLauncher => (Vec3::new(0.0, 2.6, 15.0), Some(Vec3::new(0.0, -0.4, 9.5)), Vec3::new(1.5, 2.4, 2.0), Vec3::new(0.0, 2.6, 0.0)),
        Fam::Flamethrower => (Vec3::new(0.0, 2.4, 19.5), Some(Vec3::new(0.0, -0.6, 10.0)), Vec3::new(0.0, 2.4, 0.0), Vec3::ZERO),
        Fam::Blowpipe => (Vec3::new(0.0, 1.2, 21.0), Some(Vec3::new(0.0, 1.2, 9.0)), Vec3::new(0.0, 1.2, 0.0), Vec3::ZERO),
        Fam::StarCannon => (Vec3::new(0.0, 3.0, 19.0), Some(Vec3::new(0.0, -0.4, 9.0)), Vec3::new(0.0, 3.0, 0.0), Vec3::ZERO),
        Fam::CoinGun => (Vec3::new(0.0, 2.4, 16.5), Some(Vec3::new(0.0, 0.0, 8.0)), Vec3::new(0.0, 5.0, 2.0), Vec3::new(0.0, 6.4, 2.5)),
        Fam::ElfGun => (Vec3::new(0.0, 2.3, 17.4), Some(Vec3::new(0.0, -0.4, 8.0)), Vec3::new(1.4, 2.6, 2.0), Vec3::ZERO),
        Fam::Bubble => (Vec3::new(0.0, 2.2, 18.0), Some(Vec3::new(0.0, -0.4, 9.0)), Vec3::new(0.0, 5.0, 3.0), Vec3::ZERO),
        Fam::Laser => (Vec3::new(0.0, 2.2, 21.5), Some(Vec3::new(0.0, -0.4, 10.0)), Vec3::new(0.0, 2.2, 0.0), Vec3::ZERO),
        Fam::Vortex => (Vec3::new(0.0, 2.4, 20.5), Some(Vec3::new(0.0, -0.4, 9.0)), Vec3::new(0.0, 2.4, 0.0), Vec3::new(0.0, 2.4, 12.0)),
    };
    Frame { muzzle, fore, eject, spin_at }
}

// ---- Building blocks ----

fn bx(lo: [f32; 3], hi: [f32; 3], mat: u8) -> IBox {
    IBox { lo: Vec3::from(lo), hi: Vec3::from(hi), mat }
}

/// A round-ish tube along Z (two crossed slabs and a core), centred on (x, y), from z0 to z1.
fn tube(x: f32, y: f32, z0: f32, z1: f32, w: f32, mat: u8) -> Vec<IBox> {
    let (h, n) = (w / 2.0, w * 0.34);
    vec![bx([x - h, y - n, z0], [x + h, y + n, z1], mat), bx([x - n, y - h, z0], [x + n, y + h, z1], mat)]
}

/// The dark bore at a barrel's mouth, a hair in front of it.
fn bore(x: f32, y: f32, z: f32, w: f32) -> IBox {
    let h = w * 0.3;
    bx([x - h, y - h, z - 0.3], [x + h, y + h, z + 0.05], BORE)
}

/// A slanted pistol grip: from under the frame at the top back (`z_top`) down and back by
/// `rake` over `len`, `w` wide and `d` deep, in a few steps.
fn grip(z_top: f32, len: f32, rake: f32, w: f32, d: f32, mat: u8) -> Vec<IBox> {
    let steps = 4;
    (0..steps)
        .map(|k| {
            let t0 = k as f32 / steps as f32;
            let t1 = (k + 1) as f32 / steps as f32;
            let z = z_top - rake * (t0 + t1) * 0.5;
            bx([-w / 2.0, -len * t1, z - d], [w / 2.0, -len * t0 + 0.2, z], mat)
        })
        .collect()
}

/// A trigger and its guard under the frame, just ahead of the grip.
fn trigger(z: f32, mat: u8) -> Vec<IBox> {
    vec![
        bx([-0.3, -2.2, z + 0.4], [0.3, 0.0, z + 0.9], STEEL),
        bx([-0.35, -2.6, z - 0.2], [0.35, -2.2, z + 2.6], mat),
        bx([-0.35, -2.6, z + 2.2], [0.35, 0.0, z + 2.6], mat),
    ]
}

/// A stock behind the grip: from the wrist at z = `wrist` back to the butt at `butt`, its top
/// at `top`, dropping to `drop` at the butt, with a butt plate of `plate`.
fn stock(wrist: f32, butt: f32, top: f32, drop: f32, w: f32, mat: u8, plate: u8) -> Vec<IBox> {
    let steps = 4;
    let mut v: Vec<IBox> = (0..steps)
        .map(|k| {
            let t0 = k as f32 / steps as f32;
            let t1 = (k + 1) as f32 / steps as f32;
            let za = wrist + (butt - wrist) * t1;
            let zb = wrist + (butt - wrist) * t0;
            let tm = (t0 + t1) * 0.5;
            // Slim at the wrist, deep at the butt.
            let hi = top - 0.6 * tm;
            let lo = top - 2.6 - (drop + 1.2) * tm;
            bx([-w / 2.0, lo, za], [w / 2.0, hi, zb], mat)
        })
        .collect();
    let lo = top - 2.6 - (drop + 1.2);
    v.push(bx([-w / 2.0 - 0.1, lo - 0.1, butt - 0.7], [w / 2.0 + 0.1, top - 0.5, butt], plate));
    v
}

fn piece(part: u32, boxes: Vec<IBox>) -> Piece {
    Piece { part, place: Mat4::IDENTITY, boxes }
}

/// The model of gun `i`: bone 0 the frame, 1 the slide or hammer, 2 the spinning part, 3 the
/// pump. Pieces may be empty.
pub fn model(i: usize) -> Vec<Piece> {
    let gun = &GUNS[i];
    let (m, b, t, gl) = (gun.metal, gun.body, gun.trim, gun.glow);
    let fr = frame(i);
    let mut frame_ = Vec::new();
    let mut slide = Vec::new();
    let mut spin = Vec::new();
    let mut pump = Vec::new();
    match gun.fam {
        Fam::Flintlock => {
            // A walnut stock with a brass butt cap, the barrel bound to it, the lock plate on the
            // right with its hammer, flint and frizzen.
            frame_.extend(grip(0.4, 5.6, 2.6, 2.0, 2.8, b));
            frame_.push(bx([-1.1, -6.4, -4.6], [1.1, -5.4, -1.8], t));
            frame_.push(bx([-1.0, 0.0, -1.6], [1.0, 2.0, 9.0], b));
            frame_.extend(tube(0.0, 2.6, -0.8, 13.2, 1.9, m));
            frame_.extend(tube(0.0, 2.6, 12.4, 13.2, 2.4, t));
            frame_.push(bore(0.0, 2.6, 13.4, 1.9));
            frame_.push(bx([1.0, 0.4, -0.8], [1.35, 2.2, 3.2], GUNMETAL));
            frame_.push(bx([1.0, 2.2, 1.6], [1.6, 3.4, 2.4], STEEL));
            frame_.push(bx([-0.25, 3.4, 11.5], [0.25, 3.9, 12.3], t));
            frame_.extend(trigger(0.6, t));
            // The pan glows for a moment when it fires.
            frame_.push(bx([1.1, 2.0, 1.0], [1.5, 2.3, 1.6], gl));
            slide.push(bx([0.6, 2.0, -1.8], [1.4, 3.0, -0.4], STEEL));
            slide.push(bx([0.7, 3.0, -1.4], [1.3, 4.2, -0.6], STEEL));
            slide.push(bx([0.75, 3.8, -0.9], [1.25, 4.4, 0.1], GUNMETAL));
        }
        Fam::Revolver | Fam::Magnum => {
            let big = gun.fam == Fam::Magnum;
            let len = if big { 14.0 } else { 11.8 };
            frame_.extend(grip(-0.2, 5.6, 2.2, 2.0, 2.6, b));
            frame_.push(bx([-1.1, -6.0, -3.8], [1.1, -5.4, -1.0], t));
            frame_.push(bx([-0.9, 0.0, -1.6], [0.9, 3.4, 0.6], m));
            frame_.push(bx([-0.9, 0.0, 4.6], [0.9, 3.4, 5.6], m));
            frame_.push(bx([-0.8, 2.9, 0.5], [0.8, 3.5, 4.7], m));
            frame_.extend(tube(0.0, 2.7, 5.4, len, if big { 2.2 } else { 1.7 }, m));
            if big {
                // A heavy rib along the top with vents, and an underlug.
                frame_.push(bx([-0.7, 3.4, 5.4], [0.7, 4.2, len], t));
                frame_.push(bx([-0.8, 1.0, 5.4], [0.8, 2.2, len - 0.4], m));
            }
            frame_.push(bx([-0.25, 3.4, len - 1.0], [0.25, 4.3 + big as u8 as f32 * 0.6, len - 0.3], STEEL));
            frame_.push(bore(0.0, 2.7, len + 0.2, 1.7));
            frame_.extend(trigger(0.0, t));
            // The cylinder turns one chamber with each shot.
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU;
                let (x, y) = (a.cos() * 1.05, 1.9 + a.sin() * 1.05);
                spin.extend(tube(x, y, 0.8, 4.4, 1.2, GUNMETAL));
            }
            spin.extend(tube(0.0, 1.9, 0.6, 4.6, 2.8, m));
            slide.push(bx([-0.4, 3.0, -2.4], [0.4, 4.4, -1.2], STEEL));
        }
        Fam::Pistol => {
            frame_.extend(grip(0.6, 6.0, 1.8, 2.2, 3.0, b));
            frame_.push(bx([-0.95, 0.0, -1.6], [0.95, 1.4, 8.4], GUNMETAL));
            frame_.extend(trigger(0.4, b));
            frame_.push(bx([-0.9, -6.4, -2.6], [0.9, -5.8, 0.2], GUNMETAL));
            // The slide with its serrations, sights and the barrel's mouth.
            slide.push(bx([-1.1, 1.4, -2.2], [1.1, 3.5, 9.2], m));
            slide.push(bx([-0.25, 3.5, 8.2], [0.25, 4.0, 8.8], STEEL));
            slide.push(bx([-0.7, 3.5, -2.0], [0.7, 4.0, -1.4], STEEL));
            for k in 0..3 {
                let z = -1.8 + k as f32 * 0.7;
                slide.push(bx([-1.2, 1.8, z], [1.2, 3.2, z + 0.3], t));
            }
            slide.push(bore(0.0, 2.5, 9.4, 1.6));
            if gl == EMBER && m == MAGMA {
                // The phoenix blaster: glowing vents along the slide and a gold beak.
                slide.push(bx([-1.15, 2.2, 3.0], [1.15, 2.7, 6.5], EMBER));
                frame_.push(bx([-0.6, -0.6, 6.5], [0.6, 0.0, 9.0], t));
            }
        }
        Fam::Uzi => {
            frame_.push(bx([-1.2, 0.0, -2.8], [1.2, 3.4, 7.0], m));
            frame_.extend(grip(0.6, 6.8, 0.6, 2.2, 2.6, b));
            // The magazine runs out of the grip's bottom.
            frame_.push(bx([-0.8, -9.2, -1.8], [0.8, -6.6, 0.2], STEEL));
            frame_.extend(trigger(0.6, m));
            frame_.extend(tube(0.0, 2.2, 7.0, 9.4, 1.4, STEEL));
            frame_.push(bore(0.0, 2.2, 9.6, 1.4));
            frame_.push(bx([-0.6, 3.4, 5.6], [0.6, 4.0, 6.4], STEEL));
            // A folded wire stock under the receiver.
            frame_.push(bx([-1.0, -0.5, -2.6], [-0.7, 0.0, 5.0], STEEL));
            frame_.push(bx([0.7, -0.5, -2.6], [1.0, 0.0, 5.0], STEEL));
            slide.push(bx([-0.5, 3.4, -2.0], [0.5, 4.1, 0.2], t));
        }
        Fam::Musket => {
            frame_.extend(stock(-1.4, -14.0, 1.6, 2.2, 2.0, b, t));
            frame_.push(bx([-1.0, -0.4, -1.6], [1.0, 2.0, 22.0], b));
            frame_.extend(tube(0.0, 2.4, -1.4, 27.0, 1.8, m));
            for z in [7.0, 14.0, 21.0] {
                frame_.extend(tube(0.0, 1.6, z, z + 0.8, 3.0, t));
            }
            frame_.push(bore(0.0, 2.4, 27.2, 1.8));
            frame_.push(bx([-0.3, -0.2, 3.0], [0.3, 0.3, 25.0], OAK));
            frame_.push(bx([1.0, 0.6, -1.4], [1.35, 2.4, 3.0], GUNMETAL));
            frame_.push(bx([1.0, 2.2, 1.4], [1.6, 3.4, 2.2], STEEL));
            frame_.push(bx([1.1, 2.0, 0.8], [1.5, 2.3, 1.4], gl));
            frame_.push(bx([-0.25, 3.3, 25.8], [0.25, 3.8, 26.5], t));
            frame_.extend(trigger(0.0, t));
            slide.push(bx([0.6, 2.0, -2.4], [1.4, 3.0, -0.8], STEEL));
            slide.push(bx([0.7, 3.0, -2.0], [1.3, 4.3, -1.2], STEEL));
        }
        Fam::Rifle | Fam::Sniper | Fam::Clockwork => {
            let long = gun.fam == Fam::Sniper;
            let end = if long { 27.6 } else if gun.fam == Fam::Clockwork { 20.0 } else { 21.0 };
            let wood = b == WALNUT || b == OAK;
            frame_.extend(stock(-2.4, -13.0, 2.2, 1.6, 2.0, b, if wood { t } else { POLYMER }));
            frame_.push(bx([-1.2, -0.2, -2.6], [1.2, 3.2, 6.4], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            // Handguard, barrel, a muzzle brake.
            frame_.push(bx([-1.1, 0.3, 6.4], [1.1, 3.0, 14.0], if wood { b } else { m }));
            frame_.extend(tube(0.0, 2.2, 6.0, end, 1.6, STEEL));
            frame_.extend(tube(0.0, 2.2, end - 1.4, end, 2.2, t));
            frame_.push(bore(0.0, 2.2, end + 0.2, 1.6));
            // The magazine.
            frame_.push(bx([-0.8, -4.2, 2.4], [0.8, -0.2, 4.6], GUNMETAL));
            if long {
                // The scope with glass at both ends, its rings, and a folded bipod.
                frame_.extend(tube(0.0, 5.2, -3.0, 7.0, 2.6, GUNMETAL));
                frame_.extend(tube(0.0, 5.2, 5.4, 7.4, 3.2, GUNMETAL));
                frame_.push(bx([-0.9, 4.4, 7.2], [0.9, 6.0, 7.5], GLASS));
                frame_.push(bx([-0.8, 4.5, -3.3], [0.8, 5.9, -3.0], GLASS));
                frame_.push(bx([-0.6, 3.2, -1.2], [0.6, 4.2, -0.4], STEEL));
                frame_.push(bx([-0.6, 3.2, 4.0], [0.6, 4.2, 4.8], STEEL));
                frame_.push(bx([-0.9, 0.6, 13.0], [-0.5, 1.2, 20.0], STEEL));
                frame_.push(bx([0.5, 0.6, 13.0], [0.9, 1.2, 20.0], STEEL));
            } else if gun.fam == Fam::Clockwork {
                // Brass gears on the side: the big one turns as it fires.
                for k in 0..8 {
                    let a = k as f32 / 8.0 * std::f32::consts::TAU;
                    let (y, z) = (fr.spin_at.y + a.sin() * 1.5, fr.spin_at.z + a.cos() * 1.5);
                    spin.push(bx([1.2, y - 0.35, z - 0.35], [1.7, y + 0.35, z + 0.35], t));
                }
                spin.push(bx([1.2, fr.spin_at.y - 1.2, fr.spin_at.z - 1.2], [1.6, fr.spin_at.y + 1.2, fr.spin_at.z + 1.2], m));
                frame_.push(bx([1.2, 0.6, 4.4], [1.5, 2.2, 6.0], COPPER));
                frame_.push(bx([-0.5, 3.2, 0.0], [0.5, 4.0, 5.0], t));
            } else {
                // Iron sights and a rail.
                frame_.push(bx([-0.4, 3.2, -1.8], [0.4, 4.2, -1.0], STEEL));
                frame_.push(bx([-0.25, 3.0, end - 2.4], [0.25, 4.0, end - 1.8], STEEL));
            }
            slide.push(bx([1.2, 2.0, -1.6], [1.8, 2.6, 0.2], STEEL));
            slide.push(bx([1.8, 1.8, -0.8], [2.4, 2.8, -0.2], GUNMETAL));
        }
        Fam::DoubleBarrel | Fam::Quad => {
            let quad = gun.fam == Fam::Quad;
            let end = if quad { 13.2 } else { 19.2 };
            frame_.extend(stock(-1.4, if quad { -9.0 } else { -12.0 }, 1.6, 2.0, 2.2, b, t));
            frame_.push(bx([-1.2, -0.2, -1.6], [1.2, 1.6, 2.2], m));
            frame_.push(bx([-1.2, 0.0, 2.2], [1.2, 1.2, if quad { 9.0 } else { 11.0 }], b));
            let barrels: &[(f32, f32)] = if quad { &[(-0.95, 1.8), (0.95, 1.8), (-0.95, 3.6), (0.95, 3.6)] } else { &[(-0.95, 2.2), (0.95, 2.2)] };
            for &(x, y) in barrels {
                frame_.extend(tube(x, y, 1.6, end, 1.8, m));
                frame_.push(bore(x, y, end + 0.2, 1.8));
            }
            frame_.push(bx([-0.2, if quad { 4.4 } else { 3.1 }, end - 1.0], [0.2, if quad { 4.8 } else { 3.5 }, end - 0.4], t));
            frame_.extend(trigger(-0.2, STEEL));
            frame_.push(bx([-0.5, 1.6, -1.6], [0.5, 2.4, -0.4], STEEL));
            // Twin hammers.
            slide.push(bx([-1.2, 1.4, -2.6], [-0.4, 3.4, -1.6], t));
            slide.push(bx([0.4, 1.4, -2.6], [1.2, 3.4, -1.6], t));
        }
        Fam::Pump => {
            let onyx = m == ONYX;
            frame_.extend(stock(-2.4, -12.0, 2.4, 1.4, 2.0, b, POLYMER));
            frame_.push(bx([-1.2, 0.0, -2.6], [1.2, 3.4, 5.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.6, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            frame_.extend(tube(0.0, 2.4, 4.6, 20.2, 2.0, m));
            frame_.extend(tube(0.0, 0.6, 4.6, 17.0, 1.6, GUNMETAL));
            frame_.push(bore(0.0, 2.4, 20.4, 2.0));
            frame_.push(bx([-0.3, 3.4, 18.6], [0.3, 4.0, 19.4], t));
            frame_.push(bx([-0.6, 3.4, -1.4], [0.6, 3.9, 4.0], t));
            if onyx {
                // Onyx plates with violet seams that glow, and shell holders on the side.
                frame_.push(bx([-1.25, 1.2, -1.8], [1.25, 1.6, 4.6], VIOLET));
                frame_.push(bx([-1.15, 1.8, 12.0], [1.15, 3.0, 16.0], ONYX));
                frame_.push(bx([-1.16, 2.2, 13.0], [1.16, 2.6, 15.0], VIOLET));
            } else if t == VERDITE {
                frame_.push(bx([-1.25, 1.6, 0.0], [1.25, 2.2, 3.6], t));
            }
            for k in 0..4 {
                let z = -1.8 + k as f32 * 1.0;
                frame_.push(bx([-1.6, 0.8, z], [-1.2, 1.8, z + 0.6], RED_PAINT));
            }
            pump.extend(tube(0.0, 0.6, 8.0, 12.6, 2.6, b));
            for k in 0..3 {
                let z = 8.6 + k as f32 * 1.4;
                pump.push(bx([-1.4, -0.4, z], [1.4, -0.2, z + 0.6], POLYMER));
            }
            slide.push(bx([1.2, 1.6, 0.4], [1.4, 2.8, 3.0], STEEL));
        }
        Fam::Smg => {
            frame_.push(bx([-1.2, 0.0, -3.4], [1.2, 3.4, 8.0], m));
            frame_.extend(grip(-0.4, 5.2, 1.2, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            frame_.push(bx([-1.0, -2.4, 5.8], [1.0, 0.0, 8.0], b));
            frame_.push(bx([-0.8, -5.6, 2.2], [0.8, 0.0, 4.2], t));
            frame_.extend(tube(0.0, 2.2, 8.0, 14.2, 1.6, STEEL));
            frame_.extend(tube(0.0, 2.2, 8.6, 12.4, 2.4, GUNMETAL));
            frame_.push(bore(0.0, 2.2, 14.4, 1.6));
            // A skeleton stock to the shoulder.
            frame_.push(bx([-0.4, 1.6, -11.0], [0.4, 2.4, -3.4], STEEL));
            frame_.push(bx([-0.4, -2.0, -11.0], [0.4, 2.4, -10.2], STEEL));
            frame_.push(bx([-0.4, -2.0, -10.2], [0.4, -1.2, -5.0], STEEL));
            frame_.push(bx([-0.4, 3.4, 6.4], [0.4, 4.2, 7.2], STEEL));
            slide.push(bx([1.2, 2.0, -1.4], [1.9, 2.8, 0.0], STEEL));
        }
        Fam::Shark | Fam::MegaShark => {
            // A gun grown into a shark: the body tapers to the tail as the stock, a fin on top,
            // an eye on each side and the barrel in its toothy mouth.
            let mega = gun.fam == Fam::MegaShark;
            let k = if mega { 1.18 } else { 1.0 };
            let s = |v: f32| v * k;
            frame_.push(bx([s(-2.0), s(-0.8), s(-2.0)], [s(2.0), s(3.8), s(12.0)], m));
            frame_.push(bx([s(-1.8), s(-1.6), s(0.0)], [s(1.8), s(-0.8), s(11.0)], BELLY));
            frame_.push(bx([s(-1.6), s(-0.4), s(12.0)], [s(1.6), s(3.4), s(15.6)], m));
            frame_.push(bx([s(-1.2), s(-0.6), s(15.6)], [s(1.2), s(1.0), s(17.4)], BELLY));
            frame_.push(bx([s(-1.2), s(2.4), s(15.6)], [s(1.2), s(3.0), s(17.0)], m));
            // Teeth round the mouth.
            for kx in [-1.0f32, 1.0] {
                for z in [15.8f32, 16.6] {
                    frame_.push(bx([s(kx * 1.0 - 0.25), s(1.0), s(z)], [s(kx * 1.0 + 0.25), s(1.6), s(z + 0.4)], BONE));
                }
            }
            for kx in [-1.0f32, 1.0] {
                frame_.push(bx([s(kx * 1.62 - 0.1), s(2.4), s(13.0)], [s(kx * 1.62 + 0.1), s(3.0), s(13.6)], BORE));
                frame_.push(bx([s(kx * 1.64 - 0.06), s(2.7), s(13.3)], [s(kx * 1.64 + 0.06), s(3.0), s(13.6)], CERAMIC));
                // Gills.
                for z in [10.6f32, 11.2, 11.8] {
                    frame_.push(bx([s(kx * 2.02 - 0.06), s(0.2), s(z)], [s(kx * 2.02 + 0.06), s(2.6), s(z + 0.2)], BORE));
                }
                // Pectoral fins.
                frame_.push(bx([s(kx * 2.0 - if kx < 0.0 { 1.8 } else { 0.0 }), s(-0.6), s(6.0)], [s(kx * 2.0 + if kx > 0.0 { 1.8 } else { 0.0 }), s(-0.2), s(8.6)], m));
            }
            frame_.push(bx([s(-0.4), s(3.8), s(4.0)], [s(0.4), s(5.4), s(8.4)], m));
            frame_.push(bx([s(-0.35), s(5.4), s(4.2)], [s(0.35), s(6.6), s(6.4)], m));
            frame_.push(bx([s(-0.3), s(6.6), s(4.2)], [s(0.3), s(7.2), s(5.2)], m));
            // The tail: the body narrows into the stock, with its fins.
            frame_.push(bx([s(-1.4), s(0.0), s(-6.0)], [s(1.4), s(3.2), s(-2.0)], m));
            frame_.push(bx([s(-0.9), s(0.6), s(-10.0)], [s(0.9), s(2.6), s(-6.0)], m));
            frame_.push(bx([s(-0.5), s(-1.8), s(-12.0)], [s(0.5), s(1.0), s(-10.0)], m));
            frame_.push(bx([s(-0.5), s(2.2), s(-12.6)], [s(0.5), s(5.4), s(-10.0)], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, GUNMETAL));
            frame_.extend(trigger(-0.2, t));
            let end = fr.muzzle.z - 0.2;
            if mega {
                // Two barrels turning in the mouth and a drum magazine under the belly.
                for kx in [-0.6f32, 0.6] {
                    spin.extend(tube(kx, fr.spin_at.y, s(13.0), end, 1.0, STEEL));
                    spin.push(bore(kx, fr.spin_at.y, end + 0.2, 1.0));
                }
                frame_.extend(tube(0.0, -3.0, s(2.4), s(6.4), 4.0, GUNMETAL));
                frame_.push(bx([-0.6, -1.2, 9.0], [0.6, 4.6, 9.6], gl));
            } else {
                frame_.extend(tube(0.0, fr.muzzle.y, 13.0, end, 1.2, STEEL));
                frame_.push(bore(0.0, fr.muzzle.y, end + 0.2, 1.2));
            }
            slide.push(bx([s(2.0), s(2.2), s(1.0)], [s(2.6), s(3.0), s(3.4)], STEEL));
        }
        Fam::Minigun => {
            // The motor housing with a carry handle, an ammo box with its belt, the side grip,
            // and six barrels on a spinning cluster with rings.
            frame_.push(bx([-2.0, -0.6, -6.0], [2.0, 4.6, 4.0], m));
            frame_.push(bx([-1.6, -1.2, -7.4], [1.6, 4.2, -6.0], b));
            frame_.push(bx([-0.6, 4.6, -3.6], [0.6, 5.2, 1.8], b));
            frame_.push(bx([-0.6, 5.2, -4.4], [0.6, 7.0, -3.6], b));
            frame_.push(bx([-0.6, 5.2, 1.0], [0.6, 7.0, 1.8], b));
            frame_.extend(grip(-0.4, 5.0, 1.0, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            // The side handle for the left hand.
            frame_.push(bx([-3.8, 0.2, 3.4], [-2.0, 1.0, 4.6], m));
            frame_.push(bx([-3.9, -1.0, 3.6], [-3.0, 3.2, 4.4], b));
            frame_.push(bx([2.0, -3.0, -4.0], [5.0, 1.0, 1.0], t));
            frame_.push(bx([2.2, 1.0, -3.0], [3.0, 2.6, 0.0], BRASS));
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU;
                let (x, y) = (a.cos() * 1.35, fr.spin_at.y + a.sin() * 1.35);
                spin.extend(tube(x, y, 4.0, 22.8, 1.0, STEEL));
                spin.push(bore(x, y, 23.0, 1.0));
            }
            spin.extend(tube(0.0, fr.spin_at.y, 4.0, 22.0, 1.2, GUNMETAL));
            for z in [8.0f32, 15.0, 21.8] {
                spin.extend(tube(0.0, fr.spin_at.y, z, z + 0.8, 4.0, if z > 20.0 { gl } else { m }));
            }
        }
        Fam::Rocket => {
            frame_.extend(tube(0.0, 3.2, -12.0, 16.0, 4.6, b));
            frame_.extend(tube(0.0, 3.2, 14.6, 16.4, 5.4, m));
            frame_.extend(tube(0.0, 3.2, -12.4, -10.8, 5.4, m));
            frame_.push(bore(0.0, 3.2, 16.5, 4.6));
            frame_.push(bore(0.0, 3.2, -12.3, 4.6));
            for z in [-4.0f32, 6.0] {
                frame_.extend(tube(0.0, 3.2, z, z + 0.8, 5.0, m));
            }
            frame_.extend(grip(0.6, 5.0, 1.0, 2.0, 2.4, POLYMER));
            frame_.extend(trigger(0.6, m));
            frame_.push(bx([-0.9, -4.4, 6.4], [0.9, 0.8, 8.4], POLYMER));
            // The sight on the left with its glass.
            frame_.push(bx([-4.2, 3.6, 0.0], [-2.4, 5.6, 4.0], m));
            frame_.push(bx([-4.0, 3.8, 4.0], [-2.6, 5.4, 4.3], GLASS));
            frame_.push(bx([-0.6, 5.6, -2.0], [0.6, 6.4, 3.0], t));
        }
        Fam::GrenadeLauncher => {
            frame_.extend(stock(-2.4, -10.0, 2.6, 1.0, 2.0, b, POLYMER));
            frame_.push(bx([-1.4, 0.0, -2.6], [1.4, 4.0, 0.6], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            // The drum of six grenades turns a sixth each shot.
            spin.extend(tube(0.0, fr.spin_at.y, 0.6, 6.0, 6.4, m));
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU;
                let (x, y) = (a.cos() * 2.2, fr.spin_at.y + a.sin() * 2.2);
                spin.push(bx([x - 0.7, y - 0.7, 5.9], [x + 0.7, y + 0.7, 6.15], t));
            }
            frame_.extend(tube(0.0, fr.spin_at.y, 6.0, 14.8, 3.2, m));
            frame_.extend(tube(0.0, fr.spin_at.y, 13.6, 14.8, 3.8, STEEL));
            frame_.push(bore(0.0, fr.spin_at.y, 15.0, 3.0));
            frame_.push(bx([-0.9, -3.8, 8.6], [0.9, 0.8, 10.4], b));
            frame_.push(bx([-0.4, 4.0, -1.8], [0.4, 5.0, 0.2], STEEL));
        }
        Fam::Flamethrower => {
            frame_.extend(stock(-2.4, -9.0, 2.6, 0.6, 2.0, POLYMER, POLYMER));
            frame_.push(bx([-1.4, 0.0, -2.6], [1.4, 3.6, 6.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, POLYMER));
            frame_.extend(trigger(-0.2, m));
            // The fuel tank hangs under the gun with its gauge, a hose up to the nozzle.
            frame_.extend(tube(0.0, -3.4, -1.0, 9.0, 3.6, b));
            frame_.extend(tube(0.0, -3.4, -1.4, -1.0, 3.0, m));
            frame_.extend(tube(0.0, -3.4, 9.0, 9.4, 3.0, m));
            frame_.push(bx([1.5, -3.4, 3.0], [2.0, -2.2, 4.2], t));
            frame_.push(bx([1.6, -3.0, 3.2], [2.05, -2.6, 4.0], CERAMIC));
            frame_.push(bx([-0.4, -1.6, 8.6], [0.4, 0.6, 9.4], t));
            frame_.extend(tube(0.0, 2.4, 6.0, 18.6, 1.8, m));
            frame_.extend(tube(0.0, 2.4, 8.0, 15.0, 2.8, GUNMETAL));
            frame_.extend(tube(0.0, 2.4, 18.0, 19.4, 2.6, t));
            frame_.push(bore(0.0, 2.4, 19.5, 2.2));
            // The pilot flame under the nozzle, always lit.
            frame_.push(bx([-0.3, 0.6, 18.6], [0.3, 1.4, 19.6], gl));
            frame_.push(bx([-0.4, 0.2, 18.2], [0.4, 0.8, 19.0], STEEL));
            frame_.push(bx([-0.9, -2.2, 10.2], [0.9, 1.2, 12.0], POLYMER));
        }
        Fam::Blowpipe => {
            // A length of cane with its joints, a leather mouthpiece and a binding.
            frame_.extend(tube(0.0, 1.2, -6.0, 21.0, 1.6, m));
            for z in [-1.0f32, 5.0, 11.0, 17.0] {
                frame_.extend(tube(0.0, 1.2, z, z + 0.5, 1.9, OAK));
            }
            frame_.extend(tube(0.0, 1.2, -6.6, -4.4, 2.0, t));
            frame_.extend(tube(0.0, 1.2, 8.0, 10.0, 2.0, t));
            frame_.push(bore(0.0, 1.2, 21.1, 1.6));
        }
        Fam::StarCannon => {
            // A short, fat cannon in deep blue enamel with gold bands and stars inlaid.
            frame_.extend(stock(-2.0, -8.0, 3.0, 0.6, 2.4, m, t));
            frame_.push(bx([-1.6, 0.0, -2.4], [1.6, 4.2, 2.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, t));
            frame_.extend(tube(0.0, 3.0, 1.0, 17.0, 4.4, m));
            frame_.extend(tube(0.0, 3.0, 16.0, 19.0, 5.6, t));
            frame_.extend(tube(0.0, 3.0, 6.0, 7.0, 5.0, t));
            frame_.push(bore(0.0, 3.0, 19.1, 4.4));
            for &(y, z) in &[(4.6f32, 4.0f32), (2.0, 10.0), (4.2, 13.5)] {
                frame_.push(bx([-2.3, y - 0.4, z - 0.4], [2.3, y + 0.4, z + 0.4], gl));
            }
            frame_.push(bx([-0.9, -2.6, 9.0], [0.9, 0.8, 10.6], t));
        }
        Fam::CoinGun => {
            // A gilded blunderbuss: a flared mouth, a hopper of coins on top that turns.
            frame_.extend(stock(-1.6, -10.0, 1.8, 1.8, 2.0, b, t));
            frame_.push(bx([-1.2, -0.2, -1.6], [1.2, 2.2, 7.0], b));
            frame_.extend(tube(0.0, 2.4, -0.6, 14.0, 2.2, m));
            frame_.extend(tube(0.0, 2.4, 14.0, 15.4, 3.2, m));
            frame_.extend(tube(0.0, 2.4, 15.4, 16.4, 4.2, t));
            frame_.push(bore(0.0, 2.4, 16.5, 3.6));
            for z in [5.0f32, 10.0] {
                frame_.extend(tube(0.0, 2.4, z, z + 0.7, 3.0, t));
            }
            frame_.push(bx([-0.8, 3.4, 1.6], [0.8, 4.4, 3.4], t));
            frame_.extend(trigger(-0.2, t));
            for k in 0..5 {
                let a = k as f32 / 5.0 * std::f32::consts::TAU;
                let (x, z) = (a.cos() * 1.1, fr.spin_at.z + a.sin() * 1.1);
                spin.push(bx([x - 0.7, fr.spin_at.y - 0.2 + k as f32 * 0.25, z - 0.7], [x + 0.7, fr.spin_at.y + 0.1 + k as f32 * 0.25, z + 0.7], GOLD));
            }
            spin.push(bx([-1.6, 4.4, 0.8], [1.6, 5.0, 4.2], BRASS));
        }
        Fam::ElfGun => {
            // A festive repeater: pine green with red bands, a gold bell at the stock and
            // candy-striped barrel.
            frame_.extend(stock(-2.0, -10.0, 2.4, 1.4, 2.0, m, t));
            frame_.push(bx([-1.2, 0.0, -2.6], [1.2, 3.4, 7.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, t));
            frame_.extend(tube(0.0, 2.3, 7.0, 17.2, 1.8, CERAMIC));
            for k in 0..5 {
                let z = 7.6 + k as f32 * 2.0;
                frame_.extend(tube(0.0, 2.3, z, z + 0.8, 1.95, b));
            }
            frame_.push(bore(0.0, 2.3, 17.4, 1.8));
            frame_.push(bx([-1.0, -2.4, 5.0], [1.0, 0.0, 7.0], b));
            frame_.push(bx([-0.8, 3.4, 2.0], [0.8, 4.6, 3.6], t));
            slide.push(bx([1.2, 2.0, -1.4], [1.9, 2.8, 0.0], t));
        }
        Fam::Bubble => {
            // Copper pipes and a glass globe of soapy water that sloshes.
            frame_.extend(stock(-2.0, -9.0, 2.4, 1.0, 2.0, m, t));
            frame_.push(bx([-1.2, 0.0, -2.4], [1.2, 3.0, 8.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, LEATHER));
            frame_.extend(trigger(-0.2, t));
            frame_.push(bx([-2.0, 3.0, 0.4], [2.0, 6.6, 5.6], b));
            frame_.push(bx([-1.6, 6.6, 1.2], [1.6, 7.4, 4.8], b));
            frame_.push(bx([-0.6, 7.4, 2.4], [0.6, 8.0, 3.6], t));
            frame_.extend(tube(0.0, 2.2, 8.0, 17.6, 1.6, m));
            frame_.extend(tube(0.0, 2.2, 15.6, 17.8, 3.0, t));
            frame_.push(bore(0.0, 2.2, 18.0, 2.4));
            frame_.push(bx([-0.4, 3.0, 6.0], [0.4, 3.6, 12.0], t));
        }
        Fam::Laser => {
            // A sleek white rifle with violet coils along the barrel and a lens at the mouth.
            frame_.extend(stock(-2.4, -12.0, 2.6, 1.0, 2.0, m, b));
            frame_.push(bx([-1.3, 0.0, -2.6], [1.3, 3.6, 9.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, b));
            frame_.extend(tube(0.0, 2.2, 9.0, 21.2, 1.6, b));
            for k in 0..4 {
                let z = 11.0 + k as f32 * 2.4;
                frame_.extend(tube(0.0, 2.2, z, z + 1.0, 2.6, gl));
            }
            frame_.extend(tube(0.0, 2.2, 20.6, 21.4, 2.4, m));
            frame_.push(bx([-0.5, 1.7, 21.3], [0.5, 2.7, 21.6], gl));
            frame_.push(bx([-1.35, 1.2, 0.0], [1.35, 1.8, 8.0], gl));
            frame_.push(bx([-0.8, -3.2, 3.0], [0.8, 0.0, 5.0], b));
            frame_.extend(tube(0.0, 4.6, -1.0, 6.0, 1.8, b));
            frame_.push(bx([-0.5, 4.1, 6.0], [0.5, 5.1, 6.2], GLASS));
        }
        Fam::Vortex => {
            // Alien and angular, teal plates over dark metal; a ring at the front turns.
            frame_.extend(stock(-2.4, -11.0, 2.6, 1.6, 2.2, m, t));
            frame_.push(bx([-1.4, -0.4, -2.6], [1.4, 3.8, 10.0], b));
            frame_.push(bx([-1.6, 1.4, -1.0], [1.6, 3.2, 8.0], m));
            frame_.extend(grip(-0.4, 5.0, 1.4, 2.0, 2.4, b));
            frame_.extend(trigger(-0.2, m));
            frame_.extend(tube(0.0, 2.4, 10.0, 20.4, 1.8, m));
            frame_.push(bore(0.0, 2.4, 20.6, 1.8));
            frame_.push(bx([-0.4, 3.8, 0.0], [0.4, 5.2, 6.0], m));
            frame_.push(bx([-0.45, 4.2, 1.0], [0.45, 4.8, 5.0], gl));
            frame_.push(bx([-1.65, 1.8, 2.0], [1.65, 2.4, 7.0], gl));
            // Four vanes of the ring.
            for k in 0..4 {
                let a = k as f32 / 4.0 * std::f32::consts::TAU + 0.4;
                let (x, y) = (a.cos() * 2.4, fr.spin_at.y + a.sin() * 2.4);
                spin.push(bx([x - 0.7, y - 0.7, 11.2], [x + 0.7, y + 0.7, 13.2], t));
            }
            spin.extend(tube(0.0, fr.spin_at.y, 11.6, 12.8, 3.2, gl));
        }
    }
    vec![piece(0, frame_), piece(1, slide), piece(2, spin), piece(3, pump)]
}

/// Seconds-long state of the held gun, kept between frames (tr_weapons.rs updates it).
#[derive(Clone, Copy, Default)]
pub struct GunState {
    /// 0..1 the kick of the last shot, dying away.
    pub kick: f32,
    /// Turn of the spinning bone (radians).
    pub spin: f32,
    /// 0..1 a minigun's wind-up.
    pub wind: f32,
    /// 0..1 the pump racked back after a shotgun's shot.
    pub pump: f32,
    /// 0..1 the gun raised to aim (both hands, at the eye) rather than held at rest.
    pub aim: f32,
    /// Seconds the flash at the muzzle still shows.
    pub flash: f32,
    /// Which way the last kick threw the muzzle sideways (-1..1).
    pub kick_side: f32,
}

impl GunState {
    /// A shot of gun `i` going off.
    pub fn fire(&mut self, i: usize, side: f32) {
        self.kick = 1.0;
        self.flash = 0.06;
        self.kick_side = side;
        match GUNS[i].fam {
            Fam::Pump => self.pump = 1.0,
            Fam::Revolver | Fam::Magnum | Fam::GrenadeLauncher => self.spin += std::f32::consts::TAU / 6.0,
            Fam::Clockwork => self.spin += 0.9,
            Fam::CoinGun => self.spin += 1.3,
            _ => {}
        }
    }

    /// Time going by: `firing` while the trigger is held with gun `i`.
    pub fn update(&mut self, i: Option<usize>, firing: bool, dt: f32) {
        self.kick = (self.kick - dt * 7.0).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        // The pump is racked back and forth over half a second, a little after the shot.
        self.pump = (self.pump - dt * 2.2).max(0.0);
        let want = if firing { 1.0 } else { 0.0 };
        self.wind += (want - self.wind).clamp(-dt * 0.8, dt * 1.1);
        if let Some(i) = i {
            match GUNS[i].fam {
                Fam::Minigun | Fam::MegaShark => self.spin += dt * (2.0 + 40.0 * if spins_up(i) { self.wind } else { want }),
                Fam::Vortex => self.spin += dt * (1.0 + 6.0 * want),
                _ => {}
            }
        }
        self.spin %= std::f32::consts::TAU * 6.0;
    }

    /// How far back and up the gun is thrown right now, eased (0..1).
    pub fn recoil(&self, i: usize) -> f32 {
        let k = self.kick;
        // A sharp jolt, a quick settle.
        (k * k * (3.0 - 2.0 * k)) * GUNS[i].kick.min(1.5)
    }
}

/// The gun's state as drawn: test hooks for screenshots (MC_GUN_AIM, MC_GUN_KICK, both 0..1)
/// hold its aim and kick still; otherwise as it is.
pub fn posed(mut st: GunState) -> GunState {
    static HOLD: std::sync::OnceLock<(Option<f32>, Option<f32>)> = std::sync::OnceLock::new();
    let (aim, kick) = *HOLD.get_or_init(|| {
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
        (num("MC_GUN_AIM"), num("MC_GUN_KICK"))
    });
    if let Some(a) = aim {
        st.aim = a;
    }
    if let Some(k) = kick {
        st.kick = k;
    }
    st
}

/// How far back the pump sits at 0..1 through its rack (back, then forward again).
fn pump_travel(p: f32) -> f32 {
    if p <= 0.0 {
        return 0.0;
    }
    // From 1 to 0: still for a moment, back, forward.
    let t = 1.0 - p;
    if t < 0.2 {
        0.0
    } else {
        ((t - 0.2) / 0.8 * std::f32::consts::PI).sin()
    }
}

/// Seconds a minigun takes from rest to full speed of fire.
pub const WIND_UP: f32 = 0.9;

/// How much slower than its full rate a minigun fires while winding up (1 at full speed).
pub fn wind_slowdown(st: &GunState) -> f32 {
    1.0 + 2.0 * (1.0 - st.wind.clamp(0.0, 1.0))
}

/// The bones of gun `i` drawn with `place` (gun space to the camera's block, already scaled).
pub fn bones(i: usize, place: Mat4, st: &GunState, light: [f32; 4]) -> Bones {
    let fr = frame(i);
    let mut b = Bones { light, ..Default::default() };
    b.m[0] = place.to_cols_array_2d();
    // The slide or hammer snaps back with the kick.
    let back = match GUNS[i].fam {
        Fam::Pistol | Fam::Uzi => 2.6,
        Fam::Flintlock | Fam::Musket => 0.0,
        _ => 1.4,
    } * (st.kick * 1.6).min(1.0);
    // A flintlock's hammer falls forward on the pan.
    let slide = if matches!(GUNS[i].fam, Fam::Flintlock | Fam::Musket) {
        let pivot = Vec3::new(1.0, 2.4, -1.2);
        Mat4::from_translation(pivot) * Mat4::from_rotation_x(0.55 * (1.0 - st.kick.min(1.0)).powi(4) - 0.15) * Mat4::from_translation(-pivot)
    } else {
        Mat4::from_translation(Vec3::new(0.0, 0.0, -back))
    };
    b.m[1] = (place * slide).to_cols_array_2d();
    let spin = Mat4::from_translation(fr.spin_at) * Mat4::from_rotation_z(st.spin) * Mat4::from_translation(-fr.spin_at);
    // The clockwork gear and the coin hopper turn about their own axes.
    let spin = match GUNS[i].fam {
        Fam::Clockwork => Mat4::from_translation(fr.spin_at) * Mat4::from_rotation_x(st.spin) * Mat4::from_translation(-fr.spin_at),
        Fam::CoinGun => Mat4::from_translation(fr.spin_at) * Mat4::from_rotation_y(st.spin) * Mat4::from_translation(-fr.spin_at),
        _ => spin,
    };
    b.m[2] = (place * spin).to_cols_array_2d();
    b.m[3] = (place * Mat4::from_translation(Vec3::new(0.0, 0.0, -3.2 * pump_travel(st.pump)))).to_cols_array_2d();
    b
}

/// Model pixels per block of a gun: a rifle is about a block and a third long, a pistol two
/// thirds, the same sizes against the hand as the other held things.
pub const PX: f32 = 1.0 / 26.0;

/// The turn of a gun held in the right fist of a third-person arm bone: at rest (`aim` 0) it
/// points ahead out of the hanging arm, raised (1) along the arm pointing at the target.
pub fn arm_turn(aim: f32) -> Quat {
    // Arm space: the arm hangs along -Y, its front is +Z. Gun space: +Z the barrel, +Y up.
    // At rest the hanging forearm is bent up at the elbow; the gun is tipped back down so it
    // points ahead and a little low.
    let rest = Quat::from_rotation_x(0.8);
    // Raised straight out, -Y is the aim and +Z points up.
    let raised = Quat::from_mat3(&glam::Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::NEG_Y));
    rest.slerp(raised, aim.clamp(0.0, 1.0))
}

/// Gun `i` in the right fist of a third-person arm bone (`arm`, the shoulder at `pivot`):
/// its bones and its muzzle (relative to the camera's block).
pub fn third_person(i: usize, arm: [[f32; 4]; 4], pivot: Vec3, st: &GunState, light: [f32; 4]) -> (Bones, Vec3) {
    let arm = Mat4::from_cols_array_2d(&arm);
    let fist = pivot + Vec3::new(0.0, -10.5, 0.6);
    let r = st.recoil(i);
    let q = arm_turn(st.aim);
    // The kick throws the muzzle up about the grip (gun space -X) and the gun back.
    let kick = Mat4::from_translation(Vec3::new(0.0, 0.0, -1.2 * r)) * Mat4::from_rotation_x(-0.35 * r);
    let place = arm * Mat4::from_translation(fist) * Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(16.0 * PX)) * kick;
    let b = bones(i, place, st, light);
    (b, place.transform_point3(frame(i).muzzle))
}

/// The bones of gun `i` resting on its own somewhere (a weapon rack, a test scene): `place` is
/// where its grip is, facing `yaw`, in blocks relative to the camera's block.
pub fn standing(i: usize, at: Vec3, yaw: f32, light: [f32; 4]) -> Bones {
    let place = Mat4::from_translation(at) * Mat4::from_rotation_y(yaw) * Mat4::from_scale(Vec3::splat(PX));
    bones(i, place, &GunState::default(), light)
}

/// Paint the gun skin (`scale` texels per model pixel): 32 swatches of 16 × 16 px.
pub fn paint_skin(scale: u32) -> Vec<u8> {
    let (w, h) = (64 * scale, 128 * scale);
    let mut px = vec![0u8; (w * h * 4) as usize];
    let n = 16 * scale;
    for m in 0..MATERIALS as u32 {
        let (ox, oy) = ((m % 4) * n, (m / 4) * n);
        for y in 0..n {
            for x in 0..n {
                let (u, v) = (x as f32 / scale as f32, y as f32 / scale as f32);
                let c = texel(m as u8, u, v, hash(x, y, m));
                let i = (((oy + y) * w + ox + x) * 4) as usize;
                px[i] = c[0].clamp(0.0, 255.0) as u8;
                px[i + 1] = c[1].clamp(0.0, 255.0) as u8;
                px[i + 2] = c[2].clamp(0.0, 255.0) as u8;
                px[i + 3] = 255;
            }
        }
    }
    px
}

fn hash(x: u32, y: u32, s: u32) -> f32 {
    let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ s.wrapping_mul(0x51ED_270B);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^= v >> 12;
    (v & 0xffff) as f32 / 65535.0
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The main colour of a material (the icons use it too).
pub fn base(mat: u8) -> [f32; 3] {
    match mat {
        BLUED => [52.0, 60.0, 78.0],
        STEEL => [150.0, 154.0, 160.0],
        GUNMETAL => [62.0, 64.0, 70.0],
        IRON => [104.0, 104.0, 108.0],
        BRASS => [196.0, 156.0, 76.0],
        GOLD => [232.0, 186.0, 62.0],
        COPPER => [186.0, 106.0, 64.0],
        ADAMANT => [168.0, 38.0, 48.0],
        COBALT => [40.0, 72.0, 138.0],
        MAGMA => [70.0, 34.0, 30.0],
        VERDITE => [70.0, 168.0, 70.0],
        LUMITE => [64.0, 200.0, 186.0],
        ONYX => [38.0, 30.0, 46.0],
        SHARK => [104.0, 128.0, 150.0],
        BELLY => [214.0, 218.0, 214.0],
        BONE => [236.0, 230.0, 206.0],
        WALNUT => [104.0, 66.0, 40.0],
        OAK => [168.0, 124.0, 74.0],
        REED => [170.0, 168.0, 92.0],
        LEATHER => [118.0, 76.0, 46.0],
        POLYMER => [36.0, 36.0, 40.0],
        RED_PAINT => [172.0, 34.0, 30.0],
        PINE => [36.0, 104.0, 56.0],
        CERAMIC => [226.0, 226.0, 222.0],
        GLASS => [80.0, 170.0, 220.0],
        BORE => [10.0, 10.0, 12.0],
        VORTEX => [22.0, 82.0, 84.0],
        ENAMEL => [34.0, 46.0, 120.0],
        EMBER => [255.0, 176.0, 70.0],
        CYAN => [110.0, 240.0, 255.0],
        VIOLET => [200.0, 120.0, 255.0],
        _ => [120.0, 210.0, 230.0],
    }
}

/// One texel of a material: grain for wood, brushed lines for metal, mottling for skin.
fn texel(mat: u8, u: f32, v: f32, r: f32) -> [f32; 3] {
    let c = base(mat);
    let dark = |k: f32| [c[0] * k, c[1] * k, c[2] * k];
    match mat {
        WALNUT | OAK => {
            let grain = ((v * 0.9 + (u * 0.5).sin() * 1.3) * 2.2).sin() * 0.5 + 0.5;
            mix(c, dark(0.68), grain.powf(2.0) * 0.8 + r * 0.15)
        }
        REED => {
            // Long fibres and a sheen.
            let f = ((u * 3.1).sin() * 0.5 + 0.5) * 0.25;
            mix(c, dark(0.75), f + r * 0.2)
        }
        BLUED | STEEL | GUNMETAL | IRON | ADAMANT | COBALT | BRASS | GOLD | COPPER | LUMITE => {
            // Brushed along the length, with a soft highlight band.
            let brush = ((v * 7.0 + r * 2.0).sin() * 0.5 + 0.5) * 0.12;
            let band = (1.0 - ((u - 5.0) / 3.0).powi(2)).max(0.0) * 0.18;
            let k = 0.9 + band - brush + (r - 0.5) * 0.06;
            dark(k)
        }
        MAGMA => {
            // Dark rock with glowing cracks.
            let crack = ((u * 1.3 + (v * 0.7).sin() * 2.0).sin() * (v * 1.1 + (u * 0.5).cos() * 1.6).sin()).abs();
            if crack < 0.08 {
                mix([255.0, 120.0, 30.0], [255.0, 196.0, 80.0], r)
            } else {
                mix(c, dark(0.6), r * 0.6)
            }
        }
        ONYX | VORTEX | ENAMEL => {
            let streak = ((u * 0.8 + v * 1.4).sin() * 0.5 + 0.5).powf(3.0);
            mix(dark(0.85), [c[0] * 1.6, c[1] * 1.6, c[2] * 1.6], streak * 0.35 + r * 0.08)
        }
        SHARK => {
            // Mottled skin, a little lighter low down.
            let mot = ((u * 0.9).sin() * (v * 1.3).cos() * 0.5 + 0.5) * 0.18;
            mix(c, dark(0.8), mot + r * 0.12)
        }
        VERDITE | GLASS => {
            let facet = (((u * 0.7).floor() + (v * 0.7).floor()) as i32 % 2) as f32;
            mix(c, [c[0] * 1.3, c[1] * 1.3, c[2] * 1.3], facet * 0.25 + r * 0.1)
        }
        POLYMER => {
            // Stippled grip texture.
            let dot = if (u * 2.0).fract() < 0.5 && (v * 2.0).fract() < 0.5 { 1.0 } else { 0.0 };
            mix(c, [56.0, 56.0, 62.0], dot * 0.5 + r * 0.12)
        }
        LEATHER => {
            let stitch = (u - 2.0).rem_euclid(12.0) < 0.6 && (v * 1.5).fract() < 0.5;
            if stitch { [210.0, 190.0, 140.0] } else { mix(c, dark(0.75), r * 0.4) }
        }
        EMBER | CYAN | VIOLET => mix(c, [255.0, 250.0, 230.0], r * 0.35),
        BUBBLE => {
            let swirl = ((u * 0.9 + v * 0.6).sin() * 0.5 + 0.5) * 0.4;
            mix(c, [230.0, 250.0, 255.0], swirl + r * 0.1)
        }
        _ => mix(c, dark(0.88), r * 0.3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_gun_weapon_has_a_model_and_every_model_a_weapon() {
        for g in GUNS {
            assert!(crate::tr_weapons::is_gun(g.item), "{} has a model but is no gun", g.item);
        }
        for name in crate::tr_weapons::all_weapons() {
            if crate::tr_weapons::is_gun(name) {
                assert!(index(name).is_some(), "gun {name} has no 3D model");
            }
        }
    }

    #[test]
    fn models_are_sane_and_the_muzzle_is_at_the_front() {
        for i in 0..COUNT {
            let m = model(i);
            assert_eq!(m.len(), 4);
            let boxes: Vec<&IBox> = m.iter().flat_map(|p| p.boxes.iter()).collect();
            assert!(boxes.len() >= 8, "{} has only {} boxes", GUNS[i].item, boxes.len());
            let mut front = f32::MIN;
            for b in &boxes {
                assert!(b.lo.cmple(b.hi).all(), "{}: inverted box {:?}..{:?}", GUNS[i].item, b.lo, b.hi);
                assert!(b.mat < MATERIALS);
                front = front.max(b.hi.z);
            }
            let fr = frame(i);
            assert!((fr.muzzle.z - front).abs() < 1.2, "{}: muzzle {} but front {}", GUNS[i].item, fr.muzzle.z, front);
            if two_handed(i) {
                assert!(fr.fore.is_some(), "{} is long but has no foregrip", GUNS[i].item);
            }
        }
    }

    #[test]
    fn a_shot_kicks_and_settles() {
        let i = index("shotgun").unwrap();
        let mut st = GunState::default();
        st.fire(i, 0.0);
        assert!(st.recoil(i) > 1.0);
        for _ in 0..30 {
            st.update(Some(i), false, 0.02);
        }
        assert!(st.recoil(i) < 0.01 && st.pump == 0.0);
        // A minigun spins up while held and fires faster.
        let mg = index("chain_gun").unwrap();
        let mut st = GunState::default();
        let slow = wind_slowdown(&st);
        for _ in 0..60 {
            st.update(Some(mg), true, 0.02);
        }
        assert!(wind_slowdown(&st) < slow && (wind_slowdown(&st) - 1.0).abs() < 0.05);
    }

    #[test]
    fn every_material_has_a_layer() {
        for m in 0..MATERIALS {
            assert!(layer_of(m) < CLASSES.len());
        }
    }
}

// ---- Ammunition ----

/// What a bullet does besides hitting.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Trick {
    None,
    /// Shatters into three shards in the monster it hits.
    Split,
    /// Turns towards the nearest monster ahead.
    Homing,
    /// Bursts in a small blast that breaks no blocks.
    Pop,
    /// A monster it kills drops half again its coins.
    Midas,
    /// Confetti and a big shove.
    Party,
}

pub struct Ammo {
    pub item: &'static str,
    /// Times the gun's damage, times its speed, monsters it passes through.
    pub dmg: f32,
    pub speed: f32,
    pub pierce: u8,
    pub debuff: Option<(u8, f32)>,
    pub tint: [f32; 3],
    pub trick: Trick,
}

const fn ammo(item: &'static str, dmg: f32, speed: f32, pierce: u8, debuff: Option<(u8, f32)>, tint: [f32; 3], trick: Trick) -> Ammo {
    Ammo { item, dmg, speed, pierce, debuff, tint, trick }
}

use crate::effects::{CURSED_FLAMES, ICHOR, VENOM};

/// Bullets for every gun that shoots bullets (Terraria's, adapted: they scale the gun's own
/// damage instead of adding to it, so a fast gun and a slow one gain alike).
pub const AMMO: &[Ammo] = &[
    ammo("musket_ball", 1.0, 1.0, 0, None, [1.8, 1.6, 1.0], Trick::None),
    ammo("meteor_shot", 1.12, 1.0, 2, None, [2.0, 0.9, 0.3], Trick::None),
    ammo("silver_bullet", 1.1, 1.05, 0, None, [1.7, 1.8, 2.0], Trick::None),
    ammo("crystal_bullet", 1.1, 1.0, 0, None, [1.6, 0.9, 2.0], Trick::Split),
    ammo("cursed_bullet", 1.15, 1.0, 0, Some((CURSED_FLAMES, 5.0)), [0.8, 2.0, 0.4], Trick::None),
    ammo("ichor_bullet", 1.1, 1.0, 0, Some((ICHOR, 6.0)), [2.2, 1.8, 0.4], Trick::None),
    ammo("venom_bullet", 1.15, 1.0, 0, Some((VENOM, 5.0)), [1.4, 0.5, 2.0], Trick::None),
    ammo("high_velocity_bullet", 1.25, 1.6, 1, None, [2.0, 1.9, 1.4], Trick::None),
    ammo("explosive_bullet", 1.1, 1.0, 0, None, [2.2, 1.0, 0.3], Trick::Pop),
    ammo("verdite_bullet", 1.15, 1.0, 0, None, [0.6, 2.2, 0.6], Trick::Homing),
    ammo("golden_bullet", 1.1, 1.0, 0, None, [2.2, 1.8, 0.6], Trick::Midas),
    ammo("party_bullet", 1.15, 1.0, 0, None, [2.0, 0.8, 1.8], Trick::Party),
    ammo("lumite_bullet", 1.4, 1.3, 4, None, [0.6, 2.2, 2.0], Trick::None),
];

/// The names of every bullet, in `AMMO` order.
pub fn bullet_names() -> Vec<&'static str> {
    AMMO.iter().map(|a| a.item).collect()
}

#[test]
fn raised_gun_points_down_the_arm() {
    let mut st = GunState::default();
    st.aim = 1.0;
    let i = index("revolver").unwrap();
    let id = Mat4::IDENTITY.to_cols_array_2d();
    let (b, muzzle) = third_person(i, id, Vec3::ZERO, &st, [1.0; 4]);
    let grip = Mat4::from_cols_array_2d(&b.m[0]).transform_point3(Vec3::ZERO);
    eprintln!("grip {grip:?} muzzle {muzzle:?} q {:?}", arm_turn(1.0) * Vec3::Z);
    assert!(muzzle.y < grip.y - 3.0, "raised, the barrel runs down the arm (-Y)");
}
