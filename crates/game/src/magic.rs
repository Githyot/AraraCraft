//! Magic (Terraria's idea, our own spells): a mana bar of stars, wands that shoot quick bolts,
//! staffs with strong slow spells and spell books. Spells are glowing bolts made of particles;
//! fire sets mobs alight, frost slows them, nature heals the caster, arcane bolts home in.
//! Mana refills on its own, faster after a moment without casting; mana crystals add stars.

use glam::Vec3;

use voxel::physics::Terrain;

use crate::audio::Sfx;
use crate::spell_shapes;
use crate::elements::El;
use crate::ui::UiBatch;
use crate::Game;

#[derive(Clone, Copy, PartialEq)]
enum Shot {
    /// One straight bolt.
    Bolt,
    /// A bolt that turns toward the nearest mob in front.
    Homing,
    /// Several bolts in a fan.
    Fan(u8),
    /// A slow ball that bursts on impact, hurting everything around.
    Burst(f32),
    /// Lightning from the sky onto the point you look at.
    Strike,
    /// A short stream of flame.
    Flame,
    /// Iron's Spells ideas (Etapa 5): jump to where you look.
    Blink(f32),
    /// Heal yourself.
    Heal,
    /// A ward around you that soaks up damage.
    Ward,
    /// Lightning that jumps from mob to mob.
    Chain(u8),
    /// A burning rock from the sky onto where you look.
    Meteor(f32),
    /// Spirits that circle you and strike nearby monsters.
    Spirits(u8),
    /// A ring of ice spikes flying out all around.
    Ring(u8),
    /// A slow dark orb that pulls monsters in.
    Vortex,
    /// A fan of homing stars.
    Stars(u8),
    /// A swarm of little homing shots flying off every way first (bees).
    Swarm(u8),
    /// Lote 2 (plan/magias-analise.md). A stream of drops falling in an arc (aqua scepter).
    Stream,
    /// Thorns bursting out of the ground one after another toward where you look, following
    /// the ground up and down (creeping thorn).
    Creep(u8),
    /// A spray of shots that slow down and bounce (crystal storm).
    Spray(u8),
    /// A slow orb that zaps the monsters near it (magnet sphere).
    Orb,
    /// A beam at once, glancing off walls this many times (shadow beam).
    Ricochet(u8),
    /// Small stars falling from the sky round where you look (lunar flare).
    Flare(u8),
    /// Held down: coloured beams that close into one stronger beam (prism).
    Prism,
}

struct SpellDef {
    item: &'static str,
    /// 0 fire, 1 frost, 2 spark, 3 arcane, 4 nature, 5 storm.
    school: u8,
    cost: f32,
    cooldown: f32,
    damage: f32,
    speed: f32,
    life: f32,
    shot: Shot,
}

const SPELLS: &[SpellDef] = &[
    SpellDef { item: "ruby_wand", school: 0, cost: 3.0, cooldown: 0.4, damage: 4.0, speed: 26.0, life: 1.2, shot: Shot::Bolt },
    SpellDef { item: "sapphire_wand", school: 1, cost: 3.0, cooldown: 0.4, damage: 4.0, speed: 26.0, life: 1.2, shot: Shot::Bolt },
    SpellDef { item: "topaz_wand", school: 2, cost: 2.0, cooldown: 0.22, damage: 2.5, speed: 40.0, life: 0.8, shot: Shot::Bolt },
    SpellDef { item: "amethyst_wand", school: 3, cost: 4.0, cooldown: 0.5, damage: 5.0, speed: 16.0, life: 2.5, shot: Shot::Homing },
    SpellDef { item: "emerald_wand", school: 4, cost: 3.0, cooldown: 0.45, damage: 4.0, speed: 24.0, life: 1.2, shot: Shot::Bolt },
    SpellDef { item: "fire_staff", school: 0, cost: 8.0, cooldown: 1.0, damage: 10.0, speed: 14.0, life: 2.5, shot: Shot::Burst(3.0) },
    SpellDef { item: "frost_staff", school: 1, cost: 8.0, cooldown: 0.9, damage: 4.0, speed: 24.0, life: 1.0, shot: Shot::Fan(5) },
    SpellDef { item: "thunder_tome", school: 5, cost: 10.0, cooldown: 1.2, damage: 12.0, speed: 0.0, life: 0.0, shot: Shot::Strike },
    SpellDef { item: "guardian_staff", school: 4, cost: 6.0, cooldown: 0.6, damage: 5.2, speed: 26.0, life: 1.2, shot: Shot::Fan(3) },
    // Not a held item: the warship's cannon (mods/aircraft.rs) fires these.
    SpellDef { item: "@warship_cannon", school: 0, cost: 0.0, cooldown: 0.0, damage: 8.0, speed: 34.0, life: 1.6, shot: Shot::Burst(2.5) },
    SpellDef { item: "flame_tome", school: 0, cost: 1.0, cooldown: 0.08, damage: 0.9, speed: 12.0, life: 0.35, shot: Shot::Flame },
    SpellDef { item: "blink_scroll", school: 3, cost: 6.0, cooldown: 1.0, damage: 0.0, speed: 0.0, life: 0.0, shot: Shot::Blink(14.0) },
    SpellDef { item: "heal_scroll", school: 4, cost: 8.0, cooldown: 2.0, damage: 6.0, speed: 0.0, life: 0.0, shot: Shot::Heal },
    SpellDef { item: "ward_tome", school: 1, cost: 10.0, cooldown: 4.0, damage: 10.0, speed: 0.0, life: 0.0, shot: Shot::Ward },
    SpellDef { item: "chain_tome", school: 2, cost: 7.0, cooldown: 0.8, damage: 7.0, speed: 0.0, life: 0.0, shot: Shot::Chain(4) },
    SpellDef { item: "meteor_staff", school: 0, cost: 14.0, cooldown: 1.4, damage: 46.0, speed: 30.0, life: 3.0, shot: Shot::Meteor(4.5) },
    SpellDef { item: "spirit_staff", school: 3, cost: 12.0, cooldown: 3.0, damage: 9.0, speed: 0.0, life: 20.0, shot: Shot::Spirits(3) },
    SpellDef { item: "ice_spike_staff", school: 1, cost: 10.0, cooldown: 1.0, damage: 22.0, speed: 22.0, life: 0.8, shot: Shot::Ring(14) },
    SpellDef { item: "void_tome", school: 3, cost: 14.0, cooldown: 2.5, damage: 3.5, speed: 6.0, life: 4.0, shot: Shot::Vortex },
    SpellDef { item: "hive_staff", school: 4, cost: 6.0, cooldown: 0.45, damage: 3.2, speed: 13.0, life: 3.0, shot: Shot::Swarm(3) },
    SpellDef { item: "water_bolt", school: 1, cost: 5.0, cooldown: 0.4, damage: 7.5, speed: 20.0, life: 3.0, shot: Shot::Bolt },
    SpellDef { item: "demon_scythe", school: 3, cost: 9.0, cooldown: 0.7, damage: 9.5, speed: 12.0, life: 2.5, shot: Shot::Fan(2) },
    SpellDef { item: "ruin_thorn", school: 3, cost: 5.0, cooldown: 0.5, damage: 4.3, speed: 22.0, life: 1.0, shot: Shot::Fan(3) },
    SpellDef { item: "blight_rod", school: 4, cost: 6.0, cooldown: 0.9, damage: 11.0, speed: 14.0, life: 2.5, shot: Shot::Burst(2.0) },
    SpellDef { item: "sun_stone_staff", school: 0, cost: 12.0, cooldown: 0.8, damage: 35.0, speed: 20.0, life: 2.0, shot: Shot::Burst(4.0) },
    SpellDef { item: "meteor_blaster", school: 0, cost: 4.0, cooldown: 0.22, damage: 3.5, speed: 36.0, life: 1.0, shot: Shot::Bolt },
    SpellDef { item: "colossus_staff", school: 1, cost: 9.0, cooldown: 0.8, damage: 7.0, speed: 26.0, life: 1.0, shot: Shot::Fan(5) },
    SpellDef { item: "star_tome", school: 5, cost: 9.0, cooldown: 0.5, damage: 9.0, speed: 22.0, life: 2.5, shot: Shot::Stars(4) },
    // The balance sheet (plan/balanceamento.md): the deep worm's and the ice golem's staffs.
    SpellDef { item: "worm_staff", school: 4, cost: 7.0, cooldown: 0.8, damage: 11.0, speed: 18.0, life: 2.0, shot: Shot::Burst(2.5) },
    SpellDef { item: "blizzard_staff", school: 1, cost: 5.0, cooldown: 0.35, damage: 9.0, speed: 24.0, life: 2.0, shot: Shot::Homing },
    // Lote 2: the new spells (tools/gen_magias.py makes their items, icons and recipes).
    SpellDef { item: "aqua_scepter", school: 1, cost: 1.4, cooldown: 0.12, damage: 1.15, speed: 17.0, life: 0.8, shot: Shot::Stream },
    SpellDef { item: "creeping_thorn", school: 4, cost: 6.0, cooldown: 0.7, damage: 4.8, speed: 0.0, life: 0.0, shot: Shot::Creep(12) },
    SpellDef { item: "flower_of_fire", school: 0, cost: 6.0, cooldown: 0.5, damage: 9.0, speed: 16.0, life: 3.0, shot: Shot::Bolt },
    SpellDef { item: "crystal_storm", school: 3, cost: 3.0, cooldown: 0.16, damage: 2.6, speed: 22.0, life: 1.6, shot: Shot::Spray(3) },
    SpellDef { item: "cursed_flames", school: 0, cost: 9.0, cooldown: 0.45, damage: 12.0, speed: 18.0, life: 2.0, shot: Shot::Bolt },
    SpellDef { item: "magnet_sphere", school: 2, cost: 12.0, cooldown: 1.4, damage: 3.4, speed: 5.0, life: 4.8, shot: Shot::Orb },
    SpellDef { item: "shadow_beam", school: 3, cost: 7.0, cooldown: 0.4, damage: 8.8, speed: 0.0, life: 0.0, shot: Shot::Ricochet(6) },
    SpellDef { item: "razorblade_typhoon", school: 1, cost: 9.0, cooldown: 0.45, damage: 7.8, speed: 13.0, life: 3.0, shot: Shot::Fan(3) },
    SpellDef { item: "lunar_flare", school: 5, cost: 11.0, cooldown: 0.35, damage: 13.0, speed: 34.0, life: 1.6, shot: Shot::Flare(3) },
    SpellDef { item: "prism", school: 3, cost: 0.8, cooldown: 0.1, damage: 2.1, speed: 0.0, life: 0.0, shot: Shot::Prism },
];

/// The element a spell carries into the world (elements.rs).
fn element(sp: &SpellDef) -> El {
    if matches!(sp.item, "water_bolt" | "aqua_scepter" | "razorblade_typhoon") {
        El::Water
    } else {
        crate::elements::of_school(sp.school)
    }
}

/// Damage per second on one monster and mana per second of a caster held down, for the
/// balance sheet (balance.rs). Fans put part of their bolts on one monster, homing stars
/// nearly all of them; the spirits stay 20 s and strike every 0.9 s. None for the spells that
/// don't hurt (blink, heal, ward) and the warship's cannon.
#[cfg(test)]
pub(crate) fn sustained_dps(item: &str) -> Option<(f32, f32)> {
    let sp = SPELLS.iter().find(|s| s.item == item && !s.item.starts_with('@'))?;
    let hits = match sp.shot {
        Shot::Blink(_) | Shot::Heal | Shot::Ward => return None,
        // The typhoon's blades home in, so nearly all of them land.
        Shot::Fan(n) if sp.item == "razorblade_typhoon" => n as f32 * 0.8,
        Shot::Fan(n) => 1.0 + (n as f32 - 1.0) * 0.35,
        // Two drops a puff, most of them on the monster.
        Shot::Stream => 1.6,
        Shot::Stars(n) => n as f32 * 0.9,
        Shot::Swarm(n) => n as f32 * 0.75,
        Shot::Spirits(n) => return Some((n as f32 * sp.damage / 0.9, sp.cost / sp.life)),
        Shot::Vortex => sp.life / VORTEX_TICK,
        // Two thorns come up under a monster as the vine passes it.
        Shot::Creep(_) => 2.0,
        Shot::Spray(n) | Shot::Flare(n) => n as f32 * 0.6,
        // Up to three orbs at once, each zapping every ORB_TICK.
        Shot::Orb => sp.life.min(sp.cooldown * MAX_ORBS as f32) / ORB_TICK,
        // It cuts through, and a glance off a wall can come back through the same monster.
        Shot::Ricochet(_) => 1.3,
        // Held down the beams close in: at full strength PRISM_FULL times the damage.
        Shot::Prism => PRISM_FULL * 0.9,
        _ => 1.0,
    };
    Some((sp.damage * hits / sp.cooldown, sp.cost / sp.cooldown))
}

/// Every caster, for balance.rs to check that each one has a stage.
#[cfg(test)]
pub(crate) fn all_casters() -> impl Iterator<Item = &'static str> {
    SPELLS.iter().map(|s| s.item).filter(|i| !i.starts_with('@'))
}

/// What is special about a caster, in a few words (its tooltip's first line).
pub fn flavor(item: &str) -> Option<&'static str> {
    Some(match item {
        "ruby_wand" => "Bola de fogo que queima",
        "sapphire_wand" => "Lasca de gelo que gela e congela água",
        "topaz_wand" => "Faísca rápida",
        "amethyst_wand" => "Cristal teleguiado",
        "emerald_wand" => "Folha que devolve um pouco de vida",
        "fire_staff" => "Grande bola de fogo que explode",
        "frost_staff" => "Leque de cinco pingentes",
        "thunder_tome" => "Raio do céu onde você mira",
        "guardian_staff" => "Três espinhos vivos",
        "flame_tome" => "Jato de chamas contínuo",
        "chain_tome" => "Relâmpago que pula de monstro em monstro",
        "meteor_staff" => "Meteoro cai do céu e explode",
        "spirit_staff" => "Espíritos giram e atacam por perto",
        "ice_spike_staff" => "Anel de pingentes em volta de você",
        "void_tome" => "Orbe que puxa os monstros",
        "hive_staff" => "Enxame de abelhas teleguiadas",
        "water_bolt" => "Orbe d'água que quica nas paredes",
        "demon_scythe" => "Foices que giram, aceleram e atravessam",
        "ruin_thorn" => "Leque de espinhos da ruína",
        "blight_rod" => "Bola de praga que explode",
        "sun_stone_staff" => "Pequeno sol que explode",
        "meteor_blaster" => "Laser rápido",
        "colossus_staff" => "Rochas de gelo em leque",
        "star_tome" => "Estrelas teleguiadas",
        "worm_staff" => "Esporo que explode",
        "blizzard_staff" => "Flocos de neve teleguiados",
        "aqua_scepter" => "Jato d'água em arco que empurra e apaga fogo",
        "creeping_thorn" => "Espinhos brotam do chão até onde você mira",
        "flower_of_fire" => "Flor de fogo que quica no chão e nas paredes",
        "crystal_storm" => "Rajada de cristais que freiam e quicam",
        "cursed_flames" => "Fogo verde que atravessa e queima por mais tempo",
        "magnet_sphere" => "Esfera lenta que solta raios nos monstros perto",
        "shadow_beam" => "Raio que ricocheteia nas paredes",
        "razorblade_typhoon" => "Lâminas d'água teleguiadas que giram e atravessam",
        "lunar_flare" => "Estrelas caem do céu em volta do alvo",
        "prism" => "Segure: feixes coloridos se juntam num raio cada vez mais forte",
        _ => return None,
    })
}

/// What a caster does, for its tooltip.
pub fn describe(item: &str) -> Option<String> {
    let sp = SPELLS.iter().find(|s| s.item == item)?;
    let what = match sp.shot {
        Shot::Blink(r) => format!("Teleporta até {r:.0} blocos"),
        Shot::Heal => format!("Cura {:.0} de vida", sp.damage),
        Shot::Ward => format!("Escudo que absorve {:.0} de dano", sp.damage),
        Shot::Spirits(n) => format!("{n} espíritos lutam ao seu lado"),
        Shot::Vortex => "Um vórtice que puxa os monstros".to_string(),
        Shot::Prism => format!("Dano: {:.0} a {:.0}", sp.damage, sp.damage * PRISM_FULL),
        _ => format!("Dano: {:.0}", sp.damage),
    };
    match flavor(item) {
        Some(f) => Some(format!("{f} · {what} · Mana: {:.0}", sp.cost)),
        None => Some(format!("{what} · Mana: {:.0}", sp.cost)),
    }
}

/// A spirit of the spirit staff: it circles the player and darts at monsters close by.
pub struct Spirit {
    angle: f32,
    life: f32,
    cd: f32,
    damage: f32,
}

/// Seconds between the void orb's bites.
const VORTEX_TICK: f32 = 0.4;

/// The void book's orb: drifts slowly, pulls monsters in and hurts the ones at its heart.
pub struct Vortex {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    tick: f32,
    damage: f32,
}

/// Over the network a bolt's kind is `SHOT_BOLT + NET_SHAPE + shape`; below it, a school.
pub(crate) const NET_SHAPE: u8 = 10;

pub(crate) fn color(school: u8) -> [f32; 3] {
    match school {
        0 => [1.6, 0.55, 0.15],
        1 => [0.5, 0.85, 1.6],
        2 => [1.6, 1.3, 0.35],
        3 => [1.1, 0.45, 1.6],
        4 => [0.45, 1.5, 0.5],
        _ => [1.2, 1.2, 1.6],
    }
}

pub struct Bolt {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    spell: usize,
    /// A monster's bolt (school, damage): it hurts the player instead of mobs.
    hostile: Option<(u8, f32)>,
    /// Seconds in flight, and a number of its own (0..1) so shots don't spin in step.
    age: f32,
    seed: f32,
    /// Monsters it already cut through (the demon scythe goes on through a few).
    pierced: Vec<usize>,
    /// Times it bounced off a wall (the water bolt).
    bounces: u8,
    /// Seconds to the magnet sphere's next zap.
    tick: f32,
}

impl Bolt {
    fn new(pos: Vec3, vel: Vec3, life: f32, spell: usize, hostile: Option<(u8, f32)>) -> Bolt {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Bolt { pos, vel, life, spell, hostile, age: 0.0, seed: (n as f32 * 0.618_034).fract(), pierced: Vec::new(), bounces: 0, tick: ORB_TICK * 0.5 }
    }

    /// Its 3D shape (spell_shapes.rs) and size.
    fn shape(&self) -> (usize, f32) {
        match self.hostile {
            Some((school, _)) => (spell_shapes::school_shape(school), 1.0),
            None => spell_shapes::shape_of(SPELLS[self.spell].item, SPELLS[self.spell].school),
        }
    }
}

/// A stroke of lightning drawn for a moment (thunder book, chain lightning): its jagged points.
pub struct Arc {
    pts: Vec<Vec3>,
    life: f32,
    /// Its shape (ARC lightning, SHADOW_BEAM, PRISM_BEAM), how thick, and a dye.
    shape: usize,
    th: f32,
    tint: Option<[f32; 3]>,
}

impl Arc {
    fn bolt(pts: Vec<Vec3>, life: f32) -> Arc {
        Arc { pts, life, shape: spell_shapes::ARC, th: 0.16, tint: None }
    }
}

/// A creeping thorn's vine: it moves along the ground and a thorn comes up at each step.
pub struct Creep {
    pos: Vec3,
    dir: Vec3,
    steps: u8,
    t: f32,
    spell: usize,
    /// Monsters it hurt and how many times (twice at most).
    hit: Vec<(usize, u8)>,
}

/// One thorn of the vine, coming up and sinking back.
pub struct Thorn {
    pos: Vec3,
    dir: Vec3,
    age: f32,
    seed: f32,
}

/// Seconds a thorn stands, and between the vine's steps.
const THORN_LIFE: f32 = 0.7;
const CREEP_STEP: f32 = 0.06;
/// Seconds between the magnet sphere's zaps, how far it reaches, and how many at once.
const ORB_TICK: f32 = 0.4;
const ORB_REACH: f32 = 7.0;
const MAX_ORBS: usize = 3;
/// The prism's beams at full strength (after PRISM_CHARGE seconds held) hit this many times
/// harder than at first.
const PRISM_FULL: f32 = 3.0;
const PRISM_CHARGE: f32 = 2.0;
const PRISM_COLORS: [[f32; 3]; 6] = [[1.6, 0.3, 0.3], [1.6, 0.8, 0.2], [1.5, 1.5, 0.3], [0.35, 1.5, 0.4], [0.35, 0.7, 1.7], [1.0, 0.4, 1.7]];

/// How many times a shape glances off walls, and how much speed it keeps.
fn bounces_of(shape: usize) -> (u8, f32) {
    match shape {
        spell_shapes::WATER_ORB => (WATER_BOUNCES, 0.9),
        spell_shapes::FIRE_FLOWER => (4, 0.8),
        spell_shapes::CRYSTAL => (2, 0.7),
        _ => (0, 0.0),
    }
}

/// How many monsters a shape cuts through before it is spent (0: it stops at the first).
fn pierce_of(shape: usize) -> usize {
    match shape {
        spell_shapes::SCYTHE | spell_shapes::TYPHOON => SCYTHE_PIERCE,
        spell_shapes::CURSED_FLAME | spell_shapes::WATER_STREAM => 2,
        _ => 0,
    }
}

/// Seconds a lightning stroke stays drawn.
const ARC_LIFE: f32 = 0.28;
/// Monsters the demon scythe cuts through before it is spent.
const SCYTHE_PIERCE: usize = 3;
/// How far a falling lunar star's blast reaches.
const FLARE_R: f32 = 1.8;
/// Times the water bolt bounces off walls.
const WATER_BOUNCES: u8 = 3;

/// The caster's mana and spells in flight.
pub struct Magic {
    pub mana: f32,
    pub max_mana: f32,
    /// Stars the star amulet adds on top (Etapa 5).
    pub extra_max: f32,
    /// Damage multiplier of the spell being cast (modifier, arcane emblem).
    pub power: f32,
    /// Seconds since the last cast (mana comes back faster after a moment).
    since_cast: f32,
    cooldown: f32,
    bolts: Vec<Bolt>,
    arcs: Vec<Arc>,
    creeps: Vec<Creep>,
    thorns: Vec<Thorn>,
    /// Seconds the prism has been held, and since its last beam.
    prism_charge: f32,
    prism_idle: f32,
}

impl Magic {
    /// Bolts in flight for the other players to see (net_fx.rs): (where, velocity, school).
    /// `mine_only` leaves the monsters' bolts out.
    /// The third value is `NET_SHAPE + shape` (spell_shapes.rs) so the others see its body.
    pub(crate) fn bolts_for_net(&self, mine_only: bool) -> Vec<(Vec3, Vec3, u8)> {
        self.bolts.iter().filter(|b| !mine_only || b.hostile.is_none()).map(|b| (b.pos, b.vel, NET_SHAPE + b.shape().0 as u8)).collect()
    }

    /// Test pictures: no bolts in front of the camera.
    pub(crate) fn clear_bolts_for_test(&mut self) {
        self.bolts.clear();
        self.thorns.clear();
        self.creeps.clear();
    }

    pub fn new(max_mana: f32) -> Magic {
        let max_mana = max_mana.clamp(20.0, 40.0);
        Magic { mana: max_mana, max_mana, extra_max: 0.0, power: 1.0, since_cast: 10.0, cooldown: 0.0, bolts: Vec::new(), arcs: Vec::new(), creeps: Vec::new(), thorns: Vec::new(), prism_charge: 0.0, prism_idle: 9.0 }
    }
}

/// Is it a spell cast by holding the button down (test clips cast it every frame)?
pub(crate) fn is_held(item: &str) -> bool {
    SPELLS.iter().any(|s| s.item == item && matches!(s.shot, Shot::Flame | Shot::Stream | Shot::Prism))
}

/// Is this item a spell caster?
pub fn is_caster(name: &str) -> bool {
    SPELLS.iter().any(|s| s.item == name) || crate::mods::spellbooks::is_book(name)
}

impl Game {
    fn held_item_name(&self) -> Option<&str> {
        self.held_stack().map(|s| self.items.get(s.item).name.as_str())
    }

    pub(crate) fn glow_layer(&self) -> u16 {
        self.block_look(self.reg.id_or_air("white_wool")).0
    }

    /// Right click with a mana crystal: two more stars for good.
    pub(crate) fn use_mana_crystal(&mut self) -> bool {
        if self.magic.max_mana >= 40.0 {
            return false;
        }
        self.magic.max_mana = (self.magic.max_mana + 4.0).min(40.0);
        self.magic.mana = self.magic.max_mana;
        let sel = self.selected;
        self.surv.inv.take(sel, 1);
        self.sfx(Sfx::Cast(3));
        self.anim.start_swing();
        self.toast(format!("Mana máxima: {} estrelas", (self.magic.max_mana / 2.0) as i32), 3.0);
        true
    }

    /// Mana, casting while the use button is held, and the spells in flight.
    pub(crate) fn update_magic(&mut self, dt: f32, use_held: bool) {
        let m = &mut self.magic;
        m.since_cast += dt;
        m.cooldown -= dt;
        m.prism_idle += dt;
        if m.prism_idle > 0.25 {
            m.prism_charge = 0.0;
        }
        let regen = if m.since_cast > 1.2 { 4.0 } else { 0.6 };
        m.mana = (m.mana + regen * dt).min(m.max_mana + m.extra_max);
        let casting = use_held && self.screen == crate::Screen::Playing;
        if casting {
            if let Some(i) = self.held_item_name().and_then(|n| SPELLS.iter().position(|s| s.item == n)) {
                self.try_cast(i);
            }
        }
        self.update_bolts(dt);
        self.update_creeps(dt);
        // Burning mobs flicker with flames.
        if dt == 0.0 || self.surv.rng.f32() > dt * 12.0 {
            return;
        }
        let layer = self.glow_layer();
        let fire: Vec<Vec3> = self.mobs.list.iter().filter(|m| m.alive() && m.on_fire > 0.0).map(|m| m.body.center()).collect();
        for c in fire {
            self.particles.glow(c, Vec3::Y * 1.5, layer, color(0), 0.09, 0.4);
        }
    }

    /// Test pictures: cast the spell of this caster now, ready or not.
    pub(crate) fn cast_now(&mut self, item: &str) {
        if let Some(i) = SPELLS.iter().position(|s| s.item == item) {
            self.magic.cooldown = 0.0;
            self.try_cast(i);
        }
    }

    fn try_cast(&mut self, i: usize) {
        let sp = &SPELLS[i];
        if self.magic.cooldown > 0.0 {
            return;
        }
        if self.has_effect(crate::effects::SILENCED) {
            self.magic.cooldown = 0.5;
            self.toast("Silenciado: não dá para lançar magias", 1.0);
            return;
        }
        let (power, mana_k, speed_k) = self.caster_mods();
        let mut cost = sp.cost * mana_k * self.t5.rpg.stats.mana_cost;
        if sp.item == "meteor_blaster" && self.t5.rpg.stats.set.is_some_and(|s| crate::rpg::SETS[s].prefix == "meteor") {
            cost = 0.0;
        }
        let free = !self.survival;
        // The mana blossom drinks a potion when mana runs out (rpg.rs).
        if !free && self.magic.mana < cost && self.wears("mana_blossom") {
            self.quick_mana();
        }
        if !free && self.magic.mana < cost {
            return;
        }
        if !free {
            self.magic.mana -= cost;
        }
        self.magic.power = power;
        self.magic.cooldown = sp.cooldown / speed_k;
        let voice = spell_shapes::sound_of(spell_shapes::shape_of(sp.item, sp.school).0);
        if self.cast_extra(i) {
            self.magic.since_cast = 0.0;
            if sp.shot != Shot::Prism || self.surv.rng.f32() < 0.12 {
                self.sfx(Sfx::Spell(voice, false));
            }
            // The prism is held out steady while it burns.
            if sp.shot != Shot::Prism || self.magic.prism_charge < 0.15 {
                self.anim.start_cast();
            }
            return;
        }
        self.magic.since_cast = 0.0;
        let eye = self.body.eye();
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let from = eye + fwd * 0.6 + right * 0.25 - Vec3::Y * 0.15;
        match sp.shot {
            Shot::Strike => self.strike(i, eye, fwd),
            Shot::Fan(n) => {
                // The scythes go out wider and one after the other, so the blades don't overlap.
                let scythe = spell_shapes::shape_of(sp.item, sp.school).0 == spell_shapes::SCYTHE;
                let spread = if scythe { 0.32 } else { 0.12 };
                for k in 0..n {
                    let a = (k as f32 - (n - 1) as f32 / 2.0) * spread;
                    let dir = (fwd * a.cos() + right * a.sin()).normalize();
                    let at = if scythe { from - dir * 0.5 * k as f32 } else { from };
                    self.magic.bolts.push(Bolt::new(at, dir * sp.speed, sp.life, i, None));
                }
            }
            Shot::Swarm(n) => {
                // A handful of bees, each off in its own direction before they find a target.
                for _ in 0..n {
                    let j = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.3, self.surv.rng.f32() - 0.5) * 0.9;
                    let dir = (fwd + j).normalize_or(fwd);
                    let speed = sp.speed * (0.8 + self.surv.rng.f32() * 0.4);
                    self.magic.bolts.push(Bolt::new(from, dir * speed, sp.life, i, None));
                }
            }
            Shot::Flame => {
                let jitter = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.15;
                self.magic.bolts.push(Bolt::new(from, (fwd + jitter).normalize() * sp.speed, sp.life, i, None));
            }
            Shot::Stream => {
                // Two drops a puff, aimed a little up so the stream arcs onto where you look.
                for _ in 0..2 {
                    let jitter = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.06;
                    let speed = sp.speed * (0.92 + self.surv.rng.f32() * 0.16);
                    self.magic.bolts.push(Bolt::new(from, (fwd + Vec3::Y * 0.08 + jitter).normalize() * speed, sp.life, i, None));
                }
            }
            Shot::Spray(n) => {
                for _ in 0..n {
                    let j = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.3;
                    let speed = sp.speed * (0.8 + self.surv.rng.f32() * 0.4);
                    self.magic.bolts.push(Bolt::new(from, (fwd + j).normalize() * speed, sp.life, i, None));
                }
            }
            Shot::Orb => {
                // Three at most: a fourth takes the place of the oldest.
                let mine: Vec<usize> = self.magic.bolts.iter().enumerate().filter(|(_, b)| b.hostile.is_none() && b.spell == i).map(|(k, _)| k).collect();
                if mine.len() >= MAX_ORBS {
                    self.magic.bolts.remove(mine[0]);
                }
                // Set off a little ahead, so it does not hang in front of your eyes.
                self.magic.bolts.push(Bolt::new(from + fwd * 1.2, fwd * sp.speed, sp.life, i, None));
            }
            _ => self.magic.bolts.push(Bolt::new(from, fwd * sp.speed, sp.life, i, None)),
        }
        // The flame book roars on its own rhythm, not on every puff (the stream, the crystal
        // storm and the prism too).
        let rapid = matches!(sp.shot, Shot::Flame | Shot::Stream | Shot::Spray(_));
        if !rapid || self.surv.rng.f32() < 0.25 {
            self.sfx(Sfx::Spell(voice, false));
        }
        self.anim.start_cast();
    }

    /// A shot from a mounted gun (the warship's cannon): a burst of fire that hurts mobs.
    pub(crate) fn cannon_shot(&mut self, from: Vec3, dir: Vec3) {
        let Some(i) = SPELLS.iter().position(|s| s.item == "@warship_cannon") else { return };
        let sp = &SPELLS[i];
        self.magic.bolts.push(Bolt::new(from, dir.normalize_or_zero() * sp.speed, sp.life, i, None));
        self.sfx(Sfx::Explode);
    }

    /// A monster's bolt (bosses): flies straight and hurts the player.
    pub(crate) fn enemy_bolt(&mut self, from: Vec3, vel: Vec3, school: u8, damage: f32) {
        self.magic.bolts.push(Bolt::new(from, vel, 3.0, 0, Some((school, damage))));
    }

    /// A monster's (or a trap's) bolt that only flies for `life` seconds.
    pub(crate) fn enemy_bolt_for(&mut self, from: Vec3, vel: Vec3, school: u8, damage: f32, life: f32) {
        self.magic.bolts.push(Bolt::new(from, vel, life, 0, Some((school, damage))));
    }

    fn update_bolts(&mut self, dt: f32) {
        if self.magic.bolts.is_empty() || dt == 0.0 {
            return;
        }
        let layer = self.glow_layer();
        let mut bolts = std::mem::take(&mut self.magic.bolts);
        let mut hits: Vec<(Vec3, usize, Option<usize>)> = Vec::new();
        let mut player_hits: Vec<(Vec3, u8, f32)> = Vec::new();
        let mut bounced: Vec<(Vec3, usize)> = Vec::new();
        let mut zaps: Vec<(Vec3, usize)> = Vec::new();
        let me = self.body.pos;
        bolts.retain_mut(|b| {
            let sp = &SPELLS[b.spell];
            b.life -= dt;
            b.age += dt;
            if let Some((school, dmg)) = b.hostile {
                if b.life <= 0.0 {
                    return false;
                }
                let step = b.vel * dt;
                let n = (step.length() / 0.25).ceil().max(1.0) as usize;
                for _ in 0..n {
                    b.pos += step / n as f32;
                    let d = b.pos - me;
                    if d.x.abs() < 0.45 && d.z.abs() < 0.45 && d.y > -0.1 && d.y < 1.9 {
                        player_hits.push((b.pos, school, dmg));
                        return false;
                    }
                    if self.world.solid(b.pos.floor().as_ivec3()) {
                        return false;
                    }
                }
                return true;
            }
            if b.life <= 0.0 {
                if matches!(sp.shot, Shot::Burst(_) | Shot::Meteor(_) | Shot::Flare(_)) {
                    hits.push((b.pos, b.spell, None));
                }
                return false;
            }
            let shape = b.shape().0;
            if matches!(sp.shot, Shot::Homing | Shot::Stars(_) | Shot::Swarm(_)) || shape == spell_shapes::TYPHOON {
                if let Some(t) = self.nearest_mob(b.pos, b.vel.normalize_or_zero(), 14.0) {
                    let want = (t - b.pos).normalize_or_zero() * sp.speed;
                    b.vel = b.vel.lerp(want, (dt * 5.0).min(1.0));
                }
            }
            if let Shot::Burst(_) = sp.shot {
                b.vel.y -= 4.0 * dt;
            }
            match shape {
                // The fire flower falls and bounces; the water stream arcs down.
                spell_shapes::FIRE_FLOWER => b.vel.y -= 14.0 * dt,
                spell_shapes::WATER_STREAM => b.vel.y -= 12.0 * dt,
                // Crystals slow down in the air.
                spell_shapes::CRYSTAL => b.vel *= 1.0 - (dt * 1.4).min(0.5),
                spell_shapes::MAGNET => {
                    b.tick -= dt;
                    if b.tick <= 0.0 {
                        b.tick = ORB_TICK;
                        zaps.push((b.pos, b.spell));
                    }
                }
                _ => {}
            }
            if shape == spell_shapes::SCYTHE {
                // Like Terraria's: thrown slowly, then it speeds up as it spins.
                let v = b.vel.length();
                if v < sp.speed * 2.4 {
                    b.vel *= 1.0 + dt * 1.8;
                }
            }
            if let Shot::Meteor(_) = sp.shot {
                // Burns as it falls.
                b.vel.y -= 10.0 * dt;
            }
            let step = b.vel * dt;
            let n = (step.length() / 0.25).ceil().max(1.0) as usize;
            // Fire, frost, water and lightning stop where they meet water or lava (and act
            // on it); a bolt cast from under water swims on.
            let wets = !matches!(element(sp), El::Nature | El::Arcane);
            for _ in 0..n {
                let was_liquid = self.world.is_liquid(self.world.block_or_air(b.pos.floor().as_ivec3()));
                b.pos += step / n as f32;
                // The magnet sphere floats through monsters; its zaps do the hurting.
                let mob = if shape == spell_shapes::MAGNET { None } else { self.mob_at(b.pos) };
                if let Some(mi) = mob {
                    let pierce = pierce_of(shape);
                    if pierce > 0 {
                        // The scythe (and the typhoon's blades, the cursed flame, the stream)
                        // cuts through and goes on to the next one.
                        if !b.pierced.contains(&mi) {
                            hits.push((b.pos, b.spell, Some(mi)));
                            b.pierced.push(mi);
                            if b.pierced.len() >= pierce {
                                return false;
                            }
                        }
                        continue;
                    }
                    hits.push((b.pos, b.spell, Some(mi)));
                    return false;
                }
                let cell = b.pos.floor().as_ivec3();
                if self.world.solid(cell) {
                    let back = b.pos - step / n as f32;
                    let (most, keep) = bounces_of(shape);
                    if b.bounces < most {
                        // The water bolt, the fire flower and the crystals bounce off walls,
                        // floors and ceilings: turn back the part of the speed that went in.
                        for axis in 0..3 {
                            let mut q = back;
                            q[axis] = b.pos[axis];
                            if self.world.solid(q.floor().as_ivec3()) {
                                b.vel[axis] = -b.vel[axis] * keep;
                            }
                        }
                        // A flower bouncing on the floor always hops up again.
                        if shape == spell_shapes::FIRE_FLOWER && b.vel.y.abs() < 5.0 && b.vel.y >= 0.0 {
                            b.vel.y = 5.0;
                        }
                        b.pos = back;
                        b.bounces += 1;
                        bounced.push((back, shape));
                        break;
                    }
                    hits.push((back, b.spell, None));
                    return false;
                }
                if wets && !was_liquid && self.world.is_liquid(self.world.block_or_air(cell)) {
                    hits.push((b.pos, b.spell, None));
                    return false;
                }
            }
            true
        });
        // Trails: each shape leaves its own (embers, frost, sparks, leaves, drops...).
        for b in &bolts {
            if let Some((school, _)) = b.hostile {
                self.particles.glow(b.pos, -b.vel * 0.05, layer, color(school), 0.07, 0.25);
                continue;
            }
            self.spell_trail(b.shape().0, b.pos, b.vel, b.age, layer);
        }
        let water = self.reg.get(self.reg.id_or_air("water")).faces[2];
        for (at, shape) in bounced {
            if shape == spell_shapes::WATER_ORB {
                self.particles.splash(at, water, 0.4);
            } else {
                self.spell_impact(shape, at, 0.3, layer);
            }
            if shape != spell_shapes::CRYSTAL || self.surv.rng.f32() < 0.3 {
                self.sfx(Sfx::Spell(spell_shapes::sound_of(shape), true));
            }
        }
        self.magic.bolts = bolts;
        for (at, spell) in zaps {
            self.magnet_zap(at, spell);
        }
        for (at, spell, mob) in hits {
            self.spell_hit(at, spell, mob);
        }
        for (at, school, dmg) in player_hits {
            self.damage(dmg, Some(at));
            self.sfx(Sfx::Spell(spell_shapes::sound_of(spell_shapes::school_shape(school)), true));
        }
    }

    /// The alive mob whose body contains `p`.
    pub(crate) fn mob_at(&self, p: Vec3) -> Option<usize> {
        self.mobs.list.iter().position(|m| {
            if !m.alive() {
                return false;
            }
            let d = &self.mobs.reg.defs[m.def];
            let b = m.body.pos;
            let r = d.width * 0.5 + 0.15;
            (p.x - b.x).abs() < r && (p.z - b.z).abs() < r && p.y > b.y - 0.1 && p.y < b.y + d.height + 0.1
        })
    }

    /// Centre of the closest alive mob within `range` and roughly ahead of `dir`.
    pub(crate) fn nearest_mob(&self, from: Vec3, dir: Vec3, range: f32) -> Option<Vec3> {
        self.mobs
            .list
            .iter()
            .filter(|m| m.alive())
            .map(|m| m.body.center())
            .filter(|c| {
                let to = *c - from;
                to.length() < range && to.normalize_or_zero().dot(dir) > 0.3
            })
            .min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    pub(crate) fn hurt_mob(&mut self, i: usize, dmg: f32, from: Vec3, school: u8) {
        self.hurt_mob_el(i, dmg, from, school, crate::elements::of_school(school));
    }

    /// A spell's hit on a mob: its element sets it burning, slows, soaks or freezes it, and
    /// can make the hit land harder (elements.rs).
    pub(crate) fn hurt_mob_el(&mut self, i: usize, dmg: f32, from: Vec3, school: u8, el: El) {
        let mut dmg = dmg * self.magic.power;
        // Magic critical hits (rpg.rs).
        if self.surv.rng.f32() < self.t5.rpg.stats.crit[crate::rpg::MAGIC] {
            dmg *= 1.5;
            self.combat_fx.crit(self.mobs.list[i].net_id);
        }
        dmg *= self.element_mob(i, el, dmg);
        self.orichalcum_petals(i, dmg);
        let m = &mut self.mobs.list[i];
        m.hurt(dmg, from);
        let def = &self.mobs.reg.defs[m.def];
        self.last_hit = Some((def.display_name.clone(), m.health.max(0.0), m.max_health, self.now + 3.0));
        if school == 4 && self.survival {
            // Nature magic gives a little life back.
            self.health = (self.health + 1.0).min(self.max_health);
        }
        if self.survival && self.t5.rpg.stats.set.is_some_and(|s| crate::rpg::SETS[s].prefix == "spectral") {
            // The spectral set: a magic hit sends a wisp of life back to the caster.
            self.health = (self.health + (dmg * 0.08).min(1.5)).min(self.max_health);
            let (layer, at, me) = (self.glow_layer(), self.mobs.list[i].body.center(), self.body.pos + Vec3::Y);
            self.particles.glow(at, (me - at) * 1.5, layer, [0.6, 1.0, 1.3], 0.07, 0.6);
        }
    }

    fn spell_hit(&mut self, at: Vec3, spell: usize, mob: Option<usize>) {
        let sp = &SPELLS[spell];
        let layer = self.glow_layer();
        let eye = self.body.eye();
        let el = element(sp);
        // What it does to the blocks where it landed.
        match sp.shot {
            Shot::Burst(r) | Shot::Meteor(r) => {
                self.element_hit(at, el, r * 0.8, if matches!(sp.shot, Shot::Meteor(_)) { 3.0 } else { 2.0 });
                self.blast_world(at, r);
            }
            // Moonlight does not break blocks.
            Shot::Flare(_) => self.element_hit(at, el, 1.2, 0.6),
            Shot::Flame => self.element_hit(at, el, 0.8, 0.25),
            Shot::Fan(_) | Shot::Ring(_) => self.element_hit(at, el, 1.6, 0.8),
            _ => self.element_hit(at, el, 1.3, 1.0),
        }
        let sp = &SPELLS[spell];
        let area = match sp.shot {
            Shot::Burst(r) | Shot::Meteor(r) => Some(r),
            Shot::Flare(_) => Some(FLARE_R),
            _ => None,
        };
        if let Some(r) = area {
            let around: Vec<(usize, f32)> = self
                .mobs
                .list
                .iter()
                .enumerate()
                .filter(|(_, m)| m.alive())
                .map(|(k, m)| (k, m.body.center().distance(at)))
                .filter(|(_, d)| *d < r)
                .collect();
            for (k, d) in around {
                self.hurt_mob_el(k, sp.damage * (1.0 - d / r * 0.6), at, sp.school, el);
            }
            let shape = spell_shapes::shape_of(sp.item, sp.school).0;
            self.spell_impact(shape, at, r / 2.0, layer);
            if !matches!(sp.shot, Shot::Flare(_)) {
                self.sfx(Sfx::Explode);
            }
            self.sfx(Sfx::Spell(spell_shapes::sound_of(shape), true));
            return;
        }
        let shape = spell_shapes::shape_of(sp.item, sp.school).0;
        if let Some(k) = mob {
            self.hurt_mob_el(k, sp.damage, eye, sp.school, el);
            self.sfx(Sfx::MobHit);
            let m = &mut self.mobs.list[k];
            match shape {
                // Cursed fire clings: it burns on longer than plain fire.
                spell_shapes::CURSED_FLAME => m.on_fire = m.on_fire.max(5.0),
                // The water stream shoves monsters back.
                spell_shapes::WATER_STREAM => {
                    let push = (m.body.pos - eye).with_y(0.0).normalize_or_zero();
                    m.body.vel += push * 2.5 + Vec3::Y * 0.6;
                }
                _ => {}
            }
        }
        self.spell_impact(shape, at, 0.5, layer);
        let rapid = matches!(sp.shot, Shot::Flame | Shot::Stream | Shot::Spray(_));
        if !rapid || self.surv.rng.f32() < 0.15 {
            self.sfx(Sfx::Spell(spell_shapes::sound_of(shape), true));
        }
    }

    /// The magnet sphere's zap: a crackling arc to the nearest monster in reach.
    fn magnet_zap(&mut self, at: Vec3, spell: usize) {
        let sp = &SPELLS[spell];
        let Some((k, c)) = self
            .mobs
            .list
            .iter()
            .enumerate()
            .filter(|(_, m)| m.alive() && self.mobs.reg.defs[m.def].kind == voxel::mob::Kind::Hostile)
            .map(|(k, m)| (k, m.body.center()))
            .filter(|(_, c)| c.distance(at) < ORB_REACH)
            .min_by(|a, b| a.1.distance(at).total_cmp(&b.1.distance(at)))
        else {
            return;
        };
        let pts = self.jagged(at, c, 0.35);
        self.magic.arcs.push(Arc { pts, life: ARC_LIFE * 0.8, shape: spell_shapes::ARC, th: 0.09, tint: Some([0.5, 1.4, 1.7]) });
        self.hurt_mob_el(k, sp.damage, at, sp.school, El::Shock);
        let layer = self.glow_layer();
        for _ in 0..5 {
            let d = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.3, self.surv.rng.f32() - 0.5) * 5.0;
            self.particles.spark(c, d, layer);
        }
        if self.surv.rng.f32() < 0.5 {
            self.sfx(Sfx::Spell(2, true));
        }
    }

    /// Lightning onto the block or mob in sight (up to 32 blocks away).
    fn strike(&mut self, spell: usize, eye: Vec3, fwd: Vec3) {
        let sp = &SPELLS[spell];
        let mut at = eye + fwd * 32.0;
        let mut p = eye;
        for _ in 0..128 {
            p += fwd * 0.25;
            if self.mob_at(p).is_some() || self.world.solid(p.floor().as_ivec3()) || self.world.is_liquid(self.world.block_or_air(p.floor().as_ivec3())) {
                at = p;
                break;
            }
        }
        self.element_hit(at, El::Shock, 2.0, 2.5);
        let around: Vec<usize> = self.mobs.list.iter().enumerate().filter(|(_, m)| m.alive() && m.body.center().distance(at) < 2.5).map(|(k, _)| k).collect();
        for k in around {
            self.hurt_mob(k, sp.damage, at + Vec3::Y * 8.0, sp.school);
        }
        let layer = self.glow_layer();
        // A jagged bolt of light down from the sky, with a fork, and sparks where it lands.
        let top = at + Vec3::new(self.surv.rng.f32() - 0.5, 0.0, self.surv.rng.f32() - 0.5) * 3.0 + Vec3::Y * 18.0;
        let main = self.jagged(top, at, 1.1);
        let fork_from = main[main.len() / 3];
        let fork_to = fork_from + Vec3::new(self.surv.rng.f32() - 0.5, -0.8, self.surv.rng.f32() - 0.5) * 6.0;
        let fork = self.jagged(fork_from, fork_to, 0.7);
        for p in main.iter().step_by(2) {
            self.particles.glow(*p, Vec3::ZERO, layer, color(5), 0.1, 0.3);
        }
        self.magic.arcs.push(Arc::bolt(main, ARC_LIFE));
        self.magic.arcs.push(Arc::bolt(fork, ARC_LIFE * 0.7));
        for _ in 0..14 {
            self.particles.spark(at, Vec3::new(self.surv.rng.f32() - 0.5, 0.5, self.surv.rng.f32() - 0.5) * 6.0, layer);
        }
        self.weather.flash = 1.0;
        self.sfx(Sfx::Thunder(1.0));
    }

    /// Mana stars down the right edge of the screen while a caster is in hand or mana is low.
    pub(crate) fn draw_mana(&mut self, w: f32) {
        let holding = self.held_item_name().is_some_and(is_caster);
        let holding = holding || self.held_item_name() == Some("glyph_book");
        if !self.survival || (!holding && self.magic.mana >= self.magic.max_mana + self.magic.extra_max - 0.01) {
            return;
        }
        let s = self.ui.scale;
        let stars = ((self.magic.max_mana + self.magic.extra_max) / 2.0).ceil() as i32;
        for i in 0..stars {
            let col = i / 10;
            let row = i % 10;
            let x = w - (12.0 + col as f32 * 10.0) * s;
            let y = (6.0 + row as f32 * 9.0) * s;
            let v = self.magic.mana - i as f32 * 2.0;
            draw_star(&mut self.ui, x, y, s, v.clamp(0.0, 2.0) / 2.0);
        }
    }
}

fn draw_star(ui: &mut UiBatch, x: f32, y: f32, s: f32, fill: f32) {
    const ROWS: [&str; 7] = ["...#...", "..###..", "#######", ".#####.", "..###..", ".##.##.", "##...##"];
    for (r, row) in ROWS.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            if ch != '#' {
                continue;
            }
            // Mana drains from the top of each star down.
            let full = (7 - r) as f32 / 7.0 <= fill + 0.01;
            let col = if full { [70, 120, 255, 255] } else { [20, 24, 60, 220] };
            ui.rect(x + c as f32 * s, y + r as f32 * s, s, s, col);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Etapa 5: the spells of the Iron's Spells idea that are not plain bolts.

impl Game {
    /// Where the player looks, up to `range` blocks: the first mob or block in the way.
    pub(crate) fn look_point(&self, range: f32) -> Vec3 {
        let eye = self.body.eye();
        let fwd = self.forward();
        let mut p = eye;
        let mut t = 0.0;
        while t < range {
            p += fwd * 0.25;
            t += 0.25;
            if self.mob_at(p).is_some() || self.world.solid(p.floor().as_ivec3()) {
                return p - fwd * 0.25;
            }
        }
        p
    }

    /// Casts the spell if it is one of the new kinds; false leaves it to the bolt code.
    fn cast_extra(&mut self, i: usize) -> bool {
        let sp = &SPELLS[i];
        let eye = self.body.eye();
        let fwd = self.forward();
        let layer = self.glow_layer();
        let col = color(sp.school);
        match sp.shot {
            Shot::Blink(range) => {
                let mut to = self.look_point(range);
                // Land on your feet in open air.
                let mut k = 0;
                while k < 8 && (self.world.solid(to.floor().as_ivec3()) || self.world.solid((to + Vec3::Y).floor().as_ivec3())) {
                    to += Vec3::Y;
                    k += 1;
                }
                for _ in 0..20 {
                    let v = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 3.0;
                    self.particles.glow(self.body.pos + Vec3::Y, v, layer, col, 0.1, 0.5);
                    self.particles.glow(to + Vec3::Y, v, layer, col, 0.1, 0.5);
                }
                self.body.pos = Vec3::new(to.x, to.y.floor(), to.z);
                self.body.vel = Vec3::ZERO;
                self.fall_peak = self.body.pos.y;
            }
            Shot::Heal => {
                self.health = (self.health + sp.damage * self.magic.power).min(self.max_health);
                for _ in 0..16 {
                    let v = Vec3::new(self.surv.rng.f32() - 0.5, 1.0, self.surv.rng.f32() - 0.5) * 2.0;
                    self.particles.glow(self.body.pos + Vec3::Y * 0.5, v, layer, col, 0.09, 0.8);
                }
            }
            Shot::Ward => {
                self.t5.ward = sp.damage * self.magic.power;
                self.t5.ward_t = 12.0;
            }
            Shot::Chain(n) => {
                let mut from = eye;
                let mut dir = fwd;
                let mut hit: Vec<usize> = Vec::new();
                for _ in 0..n {
                    let Some(k) = self
                        .mobs
                        .list
                        .iter()
                        .enumerate()
                        .filter(|(k, m)| m.alive() && !hit.contains(k) && !self.mobs.reg.defs[m.def].town)
                        .map(|(k, m)| (k, m.body.center()))
                        .filter(|(_, c)| {
                            let to = *c - from;
                            to.length() < if hit.is_empty() { 16.0 } else { 7.0 } && (hit.len() > 0 || to.normalize_or_zero().dot(dir) > 0.8)
                        })
                        .min_by(|a, b| a.1.distance(from).total_cmp(&b.1.distance(from)))
                        .map(|(k, _)| k)
                    else {
                        break;
                    };
                    let to = self.mobs.list[k].body.center();
                    let seg = to - from;
                    let start = if hit.is_empty() { from + fwd * 0.6 - Vec3::Y * 0.2 } else { from };
                    let pts = self.jagged(start, to, 0.45);
                    for p in pts.iter() {
                        self.particles.glow(*p, Vec3::ZERO, layer, col, 0.05, 0.2);
                    }
                    self.magic.arcs.push(Arc::bolt(pts, ARC_LIFE));
                    self.hurt_mob(k, sp.damage, from, sp.school);
                    hit.push(k);
                    dir = seg.normalize_or_zero();
                    from = to;
                }
                if hit.is_empty() {
                    return false;
                }
                self.sfx(Sfx::Spell(2, true));
            }
            Shot::Meteor(_) => {
                let at = self.look_point(48.0);
                let flat = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero();
                let from = at - flat * 8.0 + Vec3::Y * 26.0;
                let vel = (at - from).normalize_or_zero() * sp.speed;
                self.magic.bolts.push(Bolt::new(from, vel, sp.life, i, None));
            }
            Shot::Spirits(n) => {
                self.t5.spirits.clear();
                for k in 0..n {
                    self.t5.spirits.push(Spirit { angle: k as f32 / n as f32 * std::f32::consts::TAU, life: sp.life, cd: 0.5 + k as f32 * 0.3, damage: sp.damage * self.magic.power });
                }
            }
            Shot::Ring(n) => {
                // The ice spikes freeze the water all around the caster.
                let feet = self.body.pos + Vec3::Y * 0.2;
                self.element_hit(feet, element(sp), 4.5, 2.0);
                let from = self.body.pos + Vec3::Y * 1.0;
                for k in 0..n {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    let dir = Vec3::new(a.cos(), 0.0, a.sin());
                    self.magic.bolts.push(Bolt::new(from + dir * 0.6, dir * sp.speed, sp.life, i, None));
                }
            }
            Shot::Vortex => {
                self.t5.vortexes.push(Vortex { pos: eye + fwd, vel: fwd * sp.speed, life: sp.life, tick: 0.0, damage: sp.damage * self.magic.power });
            }
            Shot::Creep(n) => {
                let dir = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or(Vec3::X);
                let start = self.body.pos + dir * 0.6 + Vec3::Y * 0.5;
                self.magic.creeps.push(Creep { pos: start, dir, steps: n, t: 0.0, spell: i, hit: Vec::new() });
            }
            Shot::Ricochet(n) => self.shadow_beam(i, n),
            Shot::Flare(n) => {
                // Small stars from high up, each onto its own spot round where you look.
                let at = self.look_point(48.0);
                let flat = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero();
                for _ in 0..n {
                    let a = self.surv.rng.f32() * std::f32::consts::TAU;
                    let r = 0.5 + self.surv.rng.f32() * 2.5;
                    let spot = at + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                    let from = spot - flat * 4.0 + Vec3::new(self.surv.rng.f32() - 0.5, 0.0, self.surv.rng.f32() - 0.5) * 3.0 + Vec3::Y * (16.0 + self.surv.rng.f32() * 6.0);
                    let vel = (spot - from).normalize_or_zero() * sp.speed * (0.85 + self.surv.rng.f32() * 0.3);
                    self.magic.bolts.push(Bolt::new(from, vel, sp.life, i, None));
                }
            }
            Shot::Prism => self.prism_beams(i),
            Shot::Stars(n) => {
                let right = fwd.cross(Vec3::Y).normalize_or_zero();
                for k in 0..n {
                    let a = (k as f32 - (n - 1) as f32 / 2.0) * 0.25;
                    let dir = (fwd * a.cos() + right * a.sin() + Vec3::Y * 0.1).normalize();
                    self.magic.bolts.push(Bolt::new(eye + fwd * 0.6, dir * sp.speed, sp.life, i, None));
                }
            }
            _ => return false,
        }
        true
    }

    /// The shadow beam: a ray of dark light at once, glancing off walls `n` times, cutting
    /// through every monster on its way.
    fn shadow_beam(&mut self, spell: usize, n: u8) {
        let sp = &SPELLS[spell];
        let eye = self.body.eye();
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let start = eye + fwd * 1.0 + right * 0.25 - Vec3::Y * 0.2;
        let mut pts = vec![start];
        let (mut p, mut dir) = (start, fwd);
        let mut hit: Vec<usize> = Vec::new();
        let mut bounces = 0;
        let mut went = 0.0;
        while went < 60.0 {
            let next = p + dir * 0.2;
            went += 0.2;
            if self.world.solid(next.floor().as_ivec3()) {
                if bounces >= n {
                    pts.push(p);
                    break;
                }
                for axis in 0..3 {
                    let mut q = p;
                    q[axis] = next[axis];
                    if self.world.solid(q.floor().as_ivec3()) {
                        dir[axis] = -dir[axis];
                    }
                }
                bounces += 1;
                pts.push(p);
                let layer = self.glow_layer();
                self.spell_impact(spell_shapes::SHADOW_BEAM, p, 0.3, layer);
                continue;
            }
            p = next;
            if let Some(k) = self.mob_at(p) {
                if !hit.contains(&k) {
                    hit.push(k);
                    self.hurt_mob_el(k, sp.damage, p - dir, sp.school, El::Arcane);
                    self.sfx(Sfx::MobHit);
                }
            }
        }
        if pts.len() < 2 || pts.last() != Some(&p) {
            pts.push(p);
        }
        let layer = self.glow_layer();
        for w in pts.windows(2) {
            let d = w[1] - w[0];
            let steps = (d.length() / 0.8) as usize;
            for k in 0..steps {
                let at = w[0] + d * (k as f32 / steps.max(1) as f32);
                self.particles.glow(at, Vec3::Y * 0.3, layer, [0.8, 0.4, 1.5], 0.05, 0.35);
            }
        }
        self.magic.arcs.push(Arc { pts, life: 0.32, shape: spell_shapes::SHADOW_BEAM, th: 0.07, tint: None });
    }

    /// One burst of the prism held down: six coloured beams from round the prism toward where
    /// you look, closing in as it is held, until they join into one white beam.
    fn prism_beams(&mut self, spell: usize) {
        let sp = &SPELLS[spell];
        let m = &mut self.magic;
        m.prism_charge += m.prism_idle.min(0.25);
        m.prism_idle = 0.0;
        let k = (m.prism_charge / PRISM_CHARGE).min(1.0);
        let (o, fwd) = self.prism_origin();
        let end = self.look_point(40.0);
        let side = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
        let up = side.cross(fwd).normalize_or(Vec3::Y);
        let spin = self.magic.prism_charge * 3.0;
        let spread = (1.0 - k) * 1.6;
        for (n, col) in PRISM_COLORS.iter().enumerate() {
            let a = spin + n as f32 * std::f32::consts::TAU / 6.0;
            let ring = side * a.cos() + up * a.sin();
            let from = o + ring * 0.12;
            let to = end + ring * spread;
            self.magic.arcs.push(Arc { pts: vec![from, to], life: 0.12, shape: spell_shapes::PRISM_BEAM, th: 0.05 + k * 0.03, tint: Some(*col) });
        }
        if k >= 1.0 {
            self.magic.arcs.push(Arc { pts: vec![o, end], life: 0.12, shape: spell_shapes::PRISM_BEAM, th: 0.2, tint: Some([1.5, 1.5, 1.6]) });
        }
        // It hurts what the beams cross: the narrower, the harder.
        let dmg = sp.damage * (1.0 + (PRISM_FULL - 1.0) * k);
        let seg = end - o;
        let len = seg.length().max(0.01);
        let reach = 0.5 + spread * 0.4;
        let struck: Vec<usize> = self
            .mobs
            .list
            .iter()
            .enumerate()
            .filter(|(_, m)| m.alive() && !self.mobs.reg.defs[m.def].town)
            .filter(|(_, m)| {
                let c = m.body.center();
                let t = ((c - o).dot(seg) / (len * len)).clamp(0.0, 1.0);
                c.distance(o + seg * t) < reach + self.mobs.reg.defs[m.def].width * 0.5
            })
            .map(|(k, _)| k)
            .collect();
        for k in struck {
            self.hurt_mob_el(k, dmg, o, sp.school, El::Arcane);
        }
        let layer = self.glow_layer();
        let col = PRISM_COLORS[(self.surv.rng.f32() * 6.0) as usize % 6];
        let d = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 3.0;
        self.particles.glow(end, d, layer, col, 0.07, 0.3);
    }

    /// Where the prism floats while it burns (in front of the hand), and the way it faces.
    fn prism_origin(&self) -> (Vec3, Vec3) {
        let eye = self.body.eye();
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        (eye + fwd * 1.3 + right * 0.3 - Vec3::Y * 0.25, fwd)
    }

    /// The creeping thorn's vines step along the ground, a thorn bursting up at each step.
    fn update_creeps(&mut self, dt: f32) {
        if dt == 0.0 {
            return;
        }
        for t in self.magic.thorns.iter_mut() {
            t.age += dt;
        }
        self.magic.thorns.retain(|t| t.age < THORN_LIFE);
        if self.magic.creeps.is_empty() {
            return;
        }
        let layer = self.glow_layer();
        let mut creeps = std::mem::take(&mut self.magic.creeps);
        for c in creeps.iter_mut() {
            c.t -= dt;
            while c.t <= 0.0 && c.steps > 0 {
                c.t += CREEP_STEP;
                c.steps -= 1;
                // Find the ground under the next step, up to two blocks up or six down.
                let next = c.pos + c.dir * 0.9;
                let (x, z) = (next.x.floor() as i32, next.z.floor() as i32);
                let y0 = next.y.floor() as i32;
                let ground = (-6..=2).rev().map(|dy| y0 + dy).find(|&y| self.world.solid(glam::IVec3::new(x, y, z)) && !self.world.solid(glam::IVec3::new(x, y + 1, z)));
                let Some(gy) = ground else {
                    c.steps = 0;
                    break;
                };
                c.pos = Vec3::new(next.x, gy as f32 + 1.0, next.z);
                let seed = self.surv.rng.f32();
                self.magic.thorns.push(Thorn { pos: c.pos, dir: c.dir, age: 0.0, seed });
                // Earth thrown up where it bursts out.
                let below = self.world.block_or_air(glam::IVec3::new(x, gy, z));
                let face = self.reg.get(below).faces[2];
                for _ in 0..4 {
                    let d = Vec3::new(self.surv.rng.f32() - 0.5, 1.2, self.surv.rng.f32() - 0.5) * 2.5;
                    self.particles.dust(c.pos, d, face, [1.0, 1.0, 1.0], 0.05, 0.6);
                }
                if self.surv.rng.f32() < 0.3 {
                    self.particles.leaf(c.pos + Vec3::Y * 0.5, layer, [0.5, 0.35, 0.6], 0.06);
                }
                // Monsters standing over it are pierced (each twice at most per vine).
                let sp = &SPELLS[c.spell];
                let near: Vec<usize> = self
                    .mobs
                    .list
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.alive() && !self.mobs.reg.defs[m.def].town)
                    .filter(|(_, m)| {
                        let d = m.body.pos - c.pos;
                        let r = self.mobs.reg.defs[m.def].width * 0.5 + 0.6;
                        d.x.abs() < r && d.z.abs() < r && d.y > -1.5 && d.y < 1.2
                    })
                    .map(|(k, _)| k)
                    .collect();
                for k in near {
                    let n = c.hit.iter().position(|h| h.0 == k);
                    if n.is_some_and(|n| c.hit[n].1 >= 2) {
                        continue;
                    }
                    match n {
                        Some(n) => c.hit[n].1 += 1,
                        None => c.hit.push((k, 1)),
                    }
                    self.hurt_mob_el(k, sp.damage, c.pos - c.dir, sp.school, El::Nature);
                    self.mobs.list[k].body.vel.y += 3.0;
                    self.sfx(Sfx::MobHit);
                }
                if self.surv.rng.f32() < 0.35 {
                    self.sfx(Sfx::Spell(4, true));
                }
            }
        }
        creeps.retain(|c| c.steps > 0);
        self.magic.creeps = creeps;
    }

    /// Spirits, vortexes and the ward, each frame.
    pub(crate) fn update_spell_extras(&mut self, dt: f32) {
        if dt == 0.0 {
            return;
        }
        self.update_arcs(dt);
        let layer = self.glow_layer();
        if self.t5.ward > 0.0 {
            self.t5.ward_t -= dt;
            if self.t5.ward_t <= 0.0 {
                self.t5.ward = 0.0;
            } else if self.surv.rng.f32() < dt * 10.0 {
                let a = self.surv.rng.f32() * std::f32::consts::TAU;
                let p = self.body.pos + Vec3::new(a.cos() * 0.7, 0.2 + self.surv.rng.f32() * 1.6, a.sin() * 0.7);
                self.particles.glow(p, Vec3::ZERO, layer, [0.6, 0.85, 1.5], 0.06, 0.4);
            }
        }
        // Spirits: circle, then dart at the closest monster in reach.
        let me = self.body.pos + Vec3::Y * 1.4;
        let mut spirits = std::mem::take(&mut self.t5.spirits);
        for s in spirits.iter_mut() {
            s.life -= dt;
            s.angle += dt * 2.2;
            s.cd -= dt;
            let at = me + Vec3::new(s.angle.cos() * 1.6, (s.angle * 2.0).sin() * 0.3, s.angle.sin() * 1.6);
            self.particles.glow(at, Vec3::ZERO, layer, color(3), 0.12, 0.08);
            if s.cd <= 0.0 {
                s.cd = 0.9;
                let target = self
                    .mobs
                    .list
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.alive() && self.mobs.reg.defs[m.def].kind == voxel::mob::Kind::Hostile)
                    .map(|(k, m)| (k, m.body.center()))
                    .filter(|(_, c)| c.distance(at) < 8.0)
                    .min_by(|a, b| a.1.distance(at).total_cmp(&b.1.distance(at)));
                if let Some((k, c)) = target {
                    let seg = c - at;
                    for j in 0..8 {
                        self.particles.glow(at + seg * (j as f32 / 8.0), Vec3::ZERO, layer, color(3), 0.06, 0.2);
                    }
                    let power = self.magic.power;
                    self.magic.power = 1.0;
                    self.hurt_mob(k, s.damage, at, 3);
                    self.magic.power = power;
                }
            }
        }
        spirits.retain(|s| s.life > 0.0);
        self.t5.spirits = spirits;
        // Vortexes.
        let mut vs = std::mem::take(&mut self.t5.vortexes);
        for v in vs.iter_mut() {
            v.life -= dt;
            v.vel *= 1.0 - (dt * 0.6).min(1.0);
            let next = v.pos + v.vel * dt;
            if !self.world.solid(next.floor().as_ivec3()) {
                v.pos = next;
            }
            // Its wind drags dropped items in too.
            self.blow_items(v.pos, 7.0, -dt * 25.0);
            for _ in 0..3 {
                let a = self.surv.rng.f32() * std::f32::consts::TAU;
                let r = 0.4 + self.surv.rng.f32() * 1.6;
                let p = v.pos + Vec3::new(a.cos() * r, (self.surv.rng.f32() - 0.5) * 0.8, a.sin() * r);
                let swirl = Vec3::new(-a.sin(), 0.0, a.cos()) * 3.0 + (v.pos - p) * 1.5;
                self.particles.glow(p, swirl, layer, [0.5, 0.2, 0.9], 0.1, 0.3);
            }
            v.tick -= dt;
            let hit = v.tick <= 0.0;
            if hit {
                v.tick = VORTEX_TICK;
            }
            for k in 0..self.mobs.list.len() {
                let m = &self.mobs.list[k];
                if !m.alive() || self.mobs.reg.defs[m.def].kind != voxel::mob::Kind::Hostile || self.mobs.reg.defs[m.def].boss {
                    continue;
                }
                let to = v.pos - m.body.center();
                let d = to.length();
                if d < 7.0 {
                    let m = &mut self.mobs.list[k];
                    m.body.vel += to.normalize_or_zero() * dt * 30.0;
                    if hit && d < 2.2 {
                        let power = self.magic.power;
                        self.magic.power = 1.0;
                        self.hurt_mob(k, v.damage, v.pos, 3);
                        self.magic.power = power;
                    }
                }
            }
        }
        vs.retain(|v| v.life > 0.0);
        self.t5.vortexes = vs;
    }
}

// ---------------------------------------------------------------------------------------------
// Spells with a body (spell_shapes.rs): the 3D shapes in flight, their trails and their light.

impl Game {
    /// Points of a jagged stroke from `a` to `b`, wandering up to `amp` blocks off the line.
    fn jagged(&mut self, a: Vec3, b: Vec3, amp: f32) -> Vec<Vec3> {
        let d = b - a;
        let n = ((d.length() / 0.9).ceil() as usize).clamp(2, 40);
        let side = d.cross(Vec3::Y).normalize_or(Vec3::X);
        let up = side.cross(d).normalize_or(Vec3::Y);
        (0..=n)
            .map(|k| {
                let t = k as f32 / n as f32;
                // Pinned at both ends, widest in the middle.
                let w = amp * (t * PI_F).sin();
                let (x, y) = (self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5);
                a + d * t + (side * x + up * y) * w * if k == 0 || k == n { 0.0 } else { 1.0 }
            })
            .collect()
    }

    /// The trail a shape leaves behind it.
    pub(crate) fn spell_trail(&mut self, shape: usize, pos: Vec3, vel: Vec3, age: f32, layer: u16) {
        use spell_shapes::*;
        let r = |g: &mut Game| g.surv.rng.f32();
        let back = -vel * 0.05;
        let col = light_color(shape);
        let hot = [col[0] * 1.5, col[1] * 1.5, col[2] * 1.5];
        match shape {
            FIREBALL | BIG_FIRE | METEOR | FIRE_FLOWER => {
                // Flame licks are small (a torch's are 0.015): a few overlap into a tongue.
                let size = match shape {
                    FIREBALL | FIRE_FLOWER => 0.035,
                    BIG_FIRE => 0.06,
                    _ => 0.08,
                };
                for _ in 0..if matches!(shape, FIREBALL | FIRE_FLOWER) { 2 } else { 4 } {
                    self.particles.flame(pos - vel.normalize_or_zero() * 0.2, back, layer, size, 0.35, false);
                }
                if r(self) < 0.5 {
                    self.particles.spark(pos, back * 2.0, layer);
                }
                if !matches!(shape, FIREBALL | FIRE_FLOWER) && r(self) < 0.5 {
                    // Smoke rolling off.
                    self.particles.mote(pos, layer, [0.25, 0.23, 0.22], size, -0.6, 1.2);
                }
            }
            FLAME_PUFF => {
                if r(self) < 0.5 {
                    self.particles.flame(pos, back, layer, 0.03, 0.3, false);
                }
            }
            CANNONBALL => {
                self.particles.mote(pos, layer, [0.35, 0.34, 0.33], 0.06, -0.3, 0.9);
                if r(self) < 0.6 {
                    self.particles.spark(pos + Vec3::Y * 0.25, back, layer);
                }
            }
            ICE_SHARD | ICICLE | FROST_BOULDER | SNOWFLAKE => {
                // Frost motes that drift down, and a pale streak.
                self.particles.mote(pos, layer, [0.85, 0.95, 1.0], 0.04, 0.8, 0.9);
                self.particles.glow(pos, back, layer, hot, 0.06, 0.2);
            }
            SPARK | LASER => {
                self.particles.glow(pos, back, layer, hot, 0.05, 0.12);
                if r(self) < 0.5 {
                    let j = Vec3::new(r(self) - 0.5, r(self) - 0.5, r(self) - 0.5) * 4.0;
                    self.particles.spark(pos, j, layer);
                }
            }
            SCYTHE => {
                // Sparks thrown off the blade's spinning edge, in the plane it turns in.
                let f = vel.normalize_or(Vec3::Z);
                let side = f.cross(Vec3::Y).normalize_or(Vec3::X);
                for _ in 0..2 {
                    let a = r(self) * std::f32::consts::TAU;
                    let rim = (f * a.cos() + side * a.sin()) * 0.44;
                    let tangent = (f * -a.sin() + side * a.cos()) * 3.0;
                    self.particles.glow(pos + rim, tangent + back, layer, hot, 0.05, 0.3);
                }
            }
            AMETHYST | SPIRIT | RUIN_THORN => {
                self.particles.glow(pos, back, layer, hot, 0.06, 0.3);
            }
            VOID_ORB => {}
            LEAF | THORN => {
                self.particles.glow(pos, back, layer, hot, 0.05, 0.25);
                if r(self) < 0.12 {
                    self.particles.leaf(pos, layer, [0.45, 0.8, 0.35], 0.06);
                }
            }
            SPORE | BLIGHT => {
                // Spores and drops of blight falling out of it.
                self.particles.mote(pos, layer, if shape == SPORE { [0.75, 0.8, 0.4] } else { [0.55, 0.85, 0.2] }, 0.05, 1.5, 0.9);
                self.particles.glow(pos, back, layer, hot, 0.05, 0.2);
            }
            STAR | SUN => {
                for _ in 0..2 {
                    let j = Vec3::new(r(self) - 0.5, r(self) - 0.5, r(self) - 0.5);
                    self.particles.glow(pos + j * 0.3, back + j, layer, hot, if shape == SUN { 0.1 } else { 0.07 }, 0.4);
                }
            }
            WATER_ORB => {
                self.particles.mote(pos, layer, [0.45, 0.65, 1.0], 0.045, 3.0, 0.6);
                if (age * 10.0) as i32 % 2 == 0 {
                    self.particles.glow(pos, back, layer, hot, 0.05, 0.2);
                }
            }
            WATER_STREAM => {
                if r(self) < 0.3 {
                    self.particles.mote(pos, layer, [0.5, 0.7, 1.0], 0.03, 3.0, 0.4);
                }
            }
            CRYSTAL => {
                if r(self) < 0.6 {
                    let tint = if r(self) < 0.5 { [1.5, 0.7, 1.4] } else { [0.6, 1.4, 1.6] };
                    self.particles.glow(pos, back, layer, tint, 0.035, 0.3);
                }
            }
            CURSED_FLAME => {
                // Green flames licking back and dark smoke.
                for _ in 0..2 {
                    let j = Vec3::new(r(self) - 0.5, r(self) - 0.5, r(self) - 0.5) * 0.25;
                    self.particles.glow(pos - vel.normalize_or_zero() * 0.25 + j, back + Vec3::Y * 0.6, layer, hot, 0.06, 0.35);
                }
                if r(self) < 0.4 {
                    self.particles.mote(pos, layer, [0.12, 0.2, 0.1], 0.06, -0.6, 1.0);
                }
            }
            MAGNET => {
                if r(self) < 0.35 {
                    let j = Vec3::new(r(self) - 0.5, r(self) - 0.5, r(self) - 0.5) * 6.0;
                    self.particles.spark(pos, j, layer);
                }
            }
            TYPHOON => {
                // Spray thrown off the spinning blade.
                let a = r(self) * std::f32::consts::TAU;
                let rim = Vec3::new(a.cos(), 0.0, a.sin()) * 0.45;
                self.particles.glow(pos + rim, Vec3::new(-a.sin(), 0.3, a.cos()) * 2.5 + back, layer, [0.6, 0.9, 1.6], 0.04, 0.3);
                if r(self) < 0.3 {
                    self.particles.mote(pos, layer, [0.5, 0.7, 1.0], 0.035, 2.5, 0.5);
                }
            }
            LUNAR => {
                for _ in 0..2 {
                    let j = Vec3::new(r(self) - 0.5, r(self) - 0.5, r(self) - 0.5);
                    self.particles.glow(pos + j * 0.3, back + j * 0.5, layer, hot, 0.07, 0.5);
                }
            }
            _ => {}
        }
    }

    /// Where a spell lands, a burst made like what it is: flames and smoke, shattered ice,
    /// snapping sparks, a splash, leaves, a cloud of spores, a slash of light, sparkles.
    /// `r` is about how big (half a block for a bolt, more for a blast).
    pub(crate) fn spell_impact(&mut self, shape: usize, at: Vec3, r: f32, layer: u16) {
        use spell_shapes::*;
        let col = light_color(shape);
        let hot = [col[0] * 1.6, col[1] * 1.6, col[2] * 1.6];
        let n = (8.0 * r.max(0.5) * 2.0) as usize;
        let rnd = |g: &mut Game| Vec3::new(g.surv.rng.f32() - 0.5, g.surv.rng.f32() - 0.5, g.surv.rng.f32() - 0.5);
        match shape {
            FIREBALL | BIG_FIRE | FLAME_PUFF | METEOR | SUN | CANNONBALL | FIRE_FLOWER => {
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.flame(at + d * r * 0.6, d * 2.5 * r, layer, 0.05 * r.max(0.6), 0.5, false);
                    self.particles.spark(at, d * 5.0, layer);
                }
                for _ in 0..n / 2 {
                    let d = rnd(self);
                    self.particles.mote(at + d * r, layer, [0.22, 0.2, 0.2], 0.08 * r.max(0.6), -0.8, 1.6);
                }
                self.particles.glow(at, Vec3::ZERO, layer, hot, 0.3 * r.max(0.5), 0.15);
            }
            CRYSTAL => {
                // The crystal chips into pink and cyan splinters.
                for _ in 0..n {
                    let d = rnd(self);
                    let tint = if self.surv.rng.f32() < 0.5 { [1.0, 0.55, 0.9] } else { [0.55, 0.95, 1.0] };
                    self.particles.dust(at, d * 5.0 + Vec3::Y * 1.5, layer, tint, 0.04, 0.6);
                }
                self.particles.glow(at, Vec3::ZERO, layer, hot, 0.15, 0.1);
            }
            CURSED_FLAME => {
                for _ in 0..n + 4 {
                    let d = rnd(self);
                    self.particles.glow(at + d * 0.4, d * 3.0 + Vec3::Y * 1.2, layer, hot, 0.07, 0.6);
                }
                for _ in 0..n / 2 {
                    let d = rnd(self);
                    self.particles.mote(at + d * r, layer, [0.12, 0.2, 0.1], 0.08, -0.8, 1.4);
                }
            }
            LUNAR => {
                // A burst of moonlight: a ring of light along the ground and sparks rising.
                for k in 0..14 {
                    let a = k as f32 / 14.0 * std::f32::consts::TAU;
                    self.particles.glow(at + Vec3::Y * 0.2, Vec3::new(a.cos(), 0.15, a.sin()) * 5.0, layer, hot, 0.07, 0.4);
                }
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.glow(at, d * 3.0 + Vec3::Y * 2.5, layer, [1.4, 1.5, 1.7], 0.05, 0.7);
                }
                self.particles.glow(at, Vec3::ZERO, layer, hot, 0.4, 0.15);
            }
            SHADOW_BEAM => {
                for _ in 0..6 {
                    let d = rnd(self);
                    self.particles.glow(at, d * 4.0, layer, [0.8, 0.4, 1.6], 0.05, 0.3);
                }
            }
            ICE_SHARD | ICICLE | FROST_BOULDER | SNOWFLAKE => {
                // Shards of ice thrown out and falling, frost hanging in the air.
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.dust(at, d * 5.0 + Vec3::Y * 1.5, layer, [0.8, 0.95, 1.0], 0.05, 0.7);
                    self.particles.mote(at + d * r, layer, [0.85, 0.95, 1.0], 0.03, 0.6, 1.2);
                }
                self.particles.glow(at, Vec3::ZERO, layer, hot, 0.2 * r.max(0.5), 0.12);
            }
            SPARK | LASER | ARC | MAGNET => {
                for _ in 0..n + 4 {
                    let d = rnd(self);
                    self.particles.spark(at, d * 7.0, layer);
                }
                self.particles.glow(at, Vec3::ZERO, layer, hot, 0.35, 0.08);
            }
            WATER_ORB | WATER_STREAM | TYPHOON => {
                let water = self.reg.get(self.reg.id_or_air("water")).faces[2];
                self.particles.splash(at, water, if shape == WATER_STREAM { 0.2 } else { 0.5 });
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.dust(at, d * 4.0 + Vec3::Y * 2.0, layer, [0.45, 0.65, 1.0], 0.04, 0.6);
                }
            }
            LEAF | THORN => {
                for _ in 0..4 {
                    let d = rnd(self);
                    self.particles.leaf(at + d * 0.4, layer, [0.45, 0.8, 0.35], 0.07);
                }
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.glow(at, d * 3.0, layer, hot, 0.05, 0.35);
                }
            }
            SPORE | BLIGHT => {
                // A cloud that hangs in the air a moment.
                let tint = if shape == SPORE { [0.7, 0.75, 0.4] } else { [0.55, 0.85, 0.2] };
                for _ in 0..n * 2 {
                    let d = rnd(self);
                    self.particles.dust(at + d * r, d * 1.2, layer, tint, 0.07, 1.4);
                }
            }
            SCYTHE => {
                // A slash of light: sparks thrown along a ring, the way the blade turned.
                let a0 = self.surv.rng.f32() * std::f32::consts::TAU;
                for k in 0..16 {
                    let a = a0 + k as f32 * 0.25;
                    let rim = Vec3::new(a.cos(), (a * 0.5).sin() * 0.3, a.sin());
                    let tangent = Vec3::new(-a.sin(), 0.0, a.cos());
                    self.particles.glow(at + rim * 0.5, tangent * 4.0, layer, hot, 0.05, 0.3);
                }
            }
            BEE => {
                for _ in 0..4 {
                    let d = rnd(self);
                    self.particles.mote(at + d * 0.3, layer, [0.9, 0.75, 0.25], 0.03, 0.8, 0.6);
                }
            }
            STAR => {
                for _ in 0..n + 4 {
                    let d = rnd(self);
                    self.particles.glow(at, d * 5.0, layer, hot, 0.06, 0.5);
                }
            }
            _ => {
                // Arcane, spirit, ruin, void: a ring of motes that drift up as they fade.
                for _ in 0..n {
                    let d = rnd(self);
                    self.particles.glow(at, d * 3.5 + Vec3::Y * 0.8, layer, hot, 0.06, 0.45);
                }
            }
        }
    }

    /// Spell shapes to draw: our bolts, the others' bolts, spirits, void orbs, lightning.
    pub(crate) fn spell_instances(&self, cam_block: glam::IVec3) -> Vec<(usize, crate::entity::Bones)> {
        let c = cam_block.as_vec3();
        let mut out = Vec::new();
        let light = |p: Vec3| {
            let l = self.world.light(p.floor().as_ivec3());
            [(l >> 4) as f32 / 15.0, (l & 15) as f32 / 15.0, 0.0, 0.0]
        };
        let mut push = |shape: usize, scale: f32, pos: Vec3, vel: Vec3, age: f32, seed: f32| {
            if let Some(mesh) = self.entities.spell_mesh(shape) {
                let pose = spell_shapes::Pose { pos: pos - c, vel, age, seed, scale, light: light(pos) };
                out.push((mesh, spell_shapes::bones(shape, &pose)));
            }
        };
        for b in &self.magic.bolts {
            let (shape, scale) = b.shape();
            // A flame puff only lives a moment; the others' age decides how far they spun.
            push(shape, scale, b.pos, b.vel, b.age, b.seed);
        }
        for s in &self.net_shots {
            let Some(k) = s.kind.checked_sub(net::SHOT_BOLT) else { continue };
            let shape = if k >= NET_SHAPE { (k - NET_SHAPE) as usize } else { spell_shapes::school_shape(k) };
            push(shape, 1.0, s.pos, s.vel, s.age + (s.from % 7) as f32, (s.from % 13) as f32 / 13.0);
        }
        for (n, (shape, scale, pos, vel, age)) in self.spellbook_shapes().into_iter().enumerate() {
            push(shape, scale, pos, vel, age, (n as f32 * 0.618).fract());
        }
        let me = self.body.pos + Vec3::Y * 1.4;
        for (k, sp) in self.t5.spirits.iter().enumerate() {
            let at = me + Vec3::new(sp.angle.cos() * 1.6, (sp.angle * 2.0).sin() * 0.3, sp.angle.sin() * 1.6);
            let tangent = Vec3::new(-sp.angle.sin(), 0.0, sp.angle.cos());
            push(spell_shapes::SPIRIT, 1.0, at, tangent, self.now as f32 + k as f32, k as f32 / 3.0);
        }
        for (k, v) in self.t5.vortexes.iter().enumerate() {
            push(spell_shapes::VOID_ORB, 1.0, v.pos, v.vel, 4.0 - v.life, k as f32 * 0.3);
        }
        // The creeping thorn's thorns: up fast, a moment standing, then down into the ground.
        for t in &self.magic.thorns {
            let rise = (t.age / 0.1).min(1.0).min((THORN_LIFE - t.age) / 0.25);
            push(spell_shapes::GROUND_THORN, 0.8 + t.seed * 0.4, t.pos, t.dir, rise, t.seed * std::f32::consts::TAU);
        }
        // The prism floating in front of the hand while it burns.
        if self.magic.prism_idle < 0.2 {
            let (o, fwd) = self.prism_origin();
            push(spell_shapes::PRISM_GEM, 0.35, o, fwd, self.now as f32, 0.0);
        }
        for a in &self.magic.arcs {
            // Lightning thins out as it fades; beams keep their width.
            let th = if a.shape == spell_shapes::ARC { a.th * (a.life / ARC_LIFE).clamp(0.25, 1.0) } else { a.th * (a.life / 0.1).clamp(0.4, 1.0) };
            for w in a.pts.windows(2) {
                if let Some(mesh) = self.entities.spell_mesh(a.shape) {
                    out.push((mesh, spell_shapes::arc_bones(a.shape, w[0] - c, w[1] - c, th, a.tint)));
                }
            }
        }
        out
    }

    /// Lightning strokes fade.
    pub(crate) fn update_arcs(&mut self, dt: f32) {
        for a in self.magic.arcs.iter_mut() {
            a.life -= dt;
        }
        self.magic.arcs.retain(|a| a.life > 0.0);
    }

    /// Coloured lights the spells in flight give off: where, level 0..1, colour.
    pub(crate) fn spell_lights(&self) -> Vec<(Vec3, f32, [f32; 3])> {
        let mut out = Vec::new();
        for b in &self.magic.bolts {
            let shape = b.shape().0;
            let l = spell_shapes::light_level(shape);
            if l > 0.0 {
                out.push((b.pos, l, spell_shapes::light_color(shape)));
            }
        }
        for (shape, _, pos, _, _) in self.spellbook_shapes() {
            out.push((pos, spell_shapes::light_level(shape), spell_shapes::light_color(shape)));
        }
        for v in &self.t5.vortexes {
            out.push((v.pos, 0.6, spell_shapes::light_color(spell_shapes::VOID_ORB)));
        }
        for a in &self.magic.arcs {
            if let (Some(p), true) = (a.pts.last(), a.life > ARC_LIFE * 0.4 || a.shape == spell_shapes::PRISM_BEAM) {
                let col = match (a.shape, a.tint) {
                    (spell_shapes::PRISM_BEAM, _) => [1.0, 0.95, 1.0],
                    (_, Some(t)) => [t[0] / 1.7, t[1] / 1.7, t[2] / 1.7],
                    _ => spell_shapes::light_color(a.shape),
                };
                out.push((*p + Vec3::Y * 0.5, 1.0, col));
            }
        }
        if self.magic.prism_idle < 0.2 {
            out.push((self.prism_origin().0, 0.8, [0.9, 0.85, 1.0]));
        }
        out
    }

    /// Test picture (MC_SPELLS=gallery): every caster's spell hanging in the air in rows in
    /// front of the camera, each a little turned, so all the shapes are seen side by side.
    pub(crate) fn test_spell_gallery(&mut self) {
        let eye = self.body.eye();
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or(Vec3::Z);
        let side = flat.cross(Vec3::Y);
        self.magic.bolts.clear();
        let list: Vec<usize> = SPELLS
            .iter()
            .enumerate()
            .filter(|(_, s)| !matches!(s.shot, Shot::Blink(_) | Shot::Heal | Shot::Ward | Shot::Chain(_) | Shot::Strike | Shot::Spirits(_) | Shot::Vortex | Shot::Creep(_) | Shot::Ricochet(_) | Shot::Prism))
            .map(|(i, _)| i)
            .collect();
        let mut seen = Vec::new();
        let picks: Vec<usize> = list
            .into_iter()
            .filter(|&i| {
                let sh = spell_shapes::shape_of(SPELLS[i].item, SPELLS[i].school).0;
                let new = !seen.contains(&sh);
                seen.push(sh);
                new
            })
            .collect();
        let per_row = 6;
        let rows = picks.len().div_ceil(per_row);
        for (n, i) in picks.into_iter().enumerate() {
            let (row, col) = (n / per_row, n % per_row);
            let x = (col as f32 - (per_row - 1) as f32 / 2.0) * 1.15;
            let y = ((rows - 1) as f32 / 2.0 - row as f32) * 1.05;
            let pos = eye + flat * 5.0 + side * x + Vec3::Y * (y - 0.3);
            // Flying across and a little toward the camera, so the sides and the front show.
            let vel = (side * 1.0 - flat * 0.7 + Vec3::Y * 0.15).normalize() * SPELLS[i].speed.max(4.0);
            let mut b = Bolt::new(pos, vel, 99.0, i, None);
            b.age = if SPELLS[i].shot == Shot::Flame { 0.12 } else { 0.37 + n as f32 * 0.13 };
            self.magic.bolts.push(b);
        }
        // A spirit and a void orb at the ends.
        self.t5.spirits.clear();
        self.t5.vortexes.clear();
        self.t5.vortexes.push(Vortex { pos: eye + flat * 5.0 + side * 4.2 + Vec3::Y * 0.2, vel: Vec3::ZERO, life: 3.0, tick: 9.0, damage: 0.0 });
        self.t5.spirits.push(Spirit { angle: 0.0, life: 99.0, cd: 99.0, damage: 0.0 });
    }
}

const PI_F: f32 = std::f32::consts::PI;
