//! The bard (bardo): a support class built on the musical instruments of instruments.rs.
//!
//! - **Inspiração** is the bard's own resource (a column of notes at the right of the screen).
//!   It comes back slowly by itself and faster with every sonic attack that lands.
//! - **Left click** with an instrument: a sonic attack shaped by its family. Strings throw a
//!   cone of notes, winds a long piercing line, percussion a shock ring around the bard and
//!   keys a note that flies to the nearest monster in sight. Walls stop the sound.
//! - **Right click** plays the picked melody (mods/melodies.rs). When there is enough
//!   inspiration it is also a **song**: every 1.5 s while it plays, everyone friendly within the
//!   song's sphere (the bard, town folk, pets) gets the instrument's song buff for a while, and
//!   the monsters inside get its hex (slowed, scared, weakened, confused into fighting each
//!   other, exposed, cut). A ring of light on the ground shows how far the song reaches.
//! - Songs are effects of effects.rs, so they show with the other buffs. One song of each kind:
//!   the stronger one wins, the same strength keeps the longer time. Different songs stack, up
//!   to `MAX_SONGS` at once (the oldest gives way).
//! - Bard gear (accessories in `ACCESSORIES`, more in later batches) raises inspiration, sonic
//!   damage, song reach, length and level. Town musicians playing near the player give a soft
//!   song too.
//!
//! Damage is balanced in the test at the bottom against balance.rs's stage targets: a bard
//! alone does a bit less than the other classes (75-100%) and makes up for it with the songs.

use glam::Vec3;
use voxel::physics::Terrain;

use crate::audio::Sfx;
use crate::effects::{BLEEDING, BROKEN_ARMOR, CONFUSED, SLOWNESS, WEAKNESS};
use crate::rpg::{AccDef, Stats, M, MAGIC, MELEE, RANGED, SUMMON};
use crate::Game;

// ---------------------------------------------------------------------------------------------
// Songs (player buffs). Ids far above effects.rs's own so other additions don't collide.

/// Hino do Valor: more damage, every class.
pub const SONG_VALOR: u8 = 110;
/// Passo Ligeiro: faster on foot, a little higher jumps.
pub const SONG_SWIFT: u8 = 111;
/// Cadência: faster swings and faster sonic attacks.
pub const SONG_TEMPO: u8 = 112;
/// Muralha de Bronze: defense and a share of damage taken away.
pub const SONG_WARD: u8 = 113;
/// Canção do Vigor: life comes back.
pub const SONG_VIGOR: u8 = 114;
/// Clareza: critical hits and mana.
pub const SONG_FOCUS: u8 = 115;
/// Fôlego Circular: breath under water lasts longer, swimming is faster.
pub const SONG_BREATH: u8 = 116;
/// Brado de Guerra: critical hits and no knockback.
pub const SONG_RALLY: u8 = 117;
pub const FIRST_SONG: u8 = SONG_VALOR;
pub const LAST_SONG: u8 = SONG_RALLY;

/// Different songs on at once.
pub const MAX_SONGS: usize = 3;
/// Seconds a song buff lasts after its last pulse.
pub const SONG_SECS: f32 = 20.0;
/// Seconds between pulses while a song plays.
pub const PULSE: f32 = 1.5;

pub fn is_song(id: u8) -> bool {
    (FIRST_SONG..=LAST_SONG).contains(&id)
}

/// Name and colour of a song (effects.rs `info`).
pub fn song_info(id: u8) -> (&'static str, [u8; 3], bool) {
    match id {
        SONG_VALOR => ("Hino do Valor", [236, 176, 64], false),
        SONG_SWIFT => ("Passo Ligeiro", [120, 214, 196], false),
        SONG_TEMPO => ("Cadência", [214, 120, 72], false),
        SONG_WARD => ("Muralha de Bronze", [176, 132, 82], false),
        SONG_VIGOR => ("Canção do Vigor", [222, 96, 128], false),
        SONG_FOCUS => ("Clareza", [132, 150, 236], false),
        SONG_BREATH => ("Fôlego Circular", [88, 160, 208], false),
        SONG_RALLY => ("Brado de Guerra", [200, 70, 60], false),
        _ => ("?", [200, 200, 200], false),
    }
}

/// What a song does, per level (1 = level I), in one line (tooltips and the guide).
pub fn song_desc(id: u8) -> &'static str {
    match id {
        SONG_VALOR => "+6% de dano por nível",
        SONG_SWIFT => "+8% de velocidade e pulo um pouco mais alto por nível",
        SONG_TEMPO => "+8% de velocidade de golpe e de ataque sonoro por nível",
        SONG_WARD => "+3 de defesa e 2% menos dano por nível",
        SONG_VIGOR => "+0,5 de vida por segundo por nível",
        SONG_FOCUS => "+4% de crítico e +1 de mana por segundo por nível",
        SONG_BREATH => "Fôlego 50% maior e nado mais rápido por nível",
        SONG_RALLY => "+5% de crítico por nível e não é empurrado",
        _ => "",
    }
}

/// A song's stats on the player (called from rpg.rs `compute_stats`).
fn apply_song(s: &mut Stats, id: u8, lvl: f32) {
    match id {
        SONG_VALOR => s.dmg.iter_mut().for_each(|d| *d += 0.06 * lvl),
        SONG_SWIFT => {
            s.move_speed += 0.08 * lvl;
            s.jump += 0.05 * lvl;
        }
        SONG_TEMPO => s.melee_speed += 0.08 * lvl,
        SONG_WARD => {
            s.defense += 3.0 * lvl;
            s.endurance += 0.02 * lvl;
        }
        SONG_VIGOR => s.regen += 0.5 * lvl,
        SONG_FOCUS => {
            s.crit.iter_mut().for_each(|c| *c += 0.04 * lvl);
            s.mana_regen += 1.0 * lvl;
        }
        SONG_BREATH => {
            s.breath += 0.5 * lvl;
            s.swim += 0.15 * lvl;
        }
        SONG_RALLY => {
            s.crit.iter_mut().for_each(|c| *c += 0.05 * lvl);
            s.no_knockback = true;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Hexes (what a song does to the monsters inside it).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hex {
    /// Slowed (frost's chill).
    Slow,
    /// Runs away from the bard. Not bosses.
    Fear,
    /// Hits 25% softer.
    Weak,
    /// Turns on the nearest other monster. Not bosses.
    Confuse,
    /// Takes 15% more damage (broken armour).
    Expose,
    /// Bleeds a little.
    Bleed,
}

impl Hex {
    pub fn name(self) -> &'static str {
        match self {
            Hex::Slow => "lentidão",
            Hex::Fear => "medo",
            Hex::Weak => "fraqueza",
            Hex::Confuse => "confusão (brigam entre si)",
            Hex::Expose => "armadura quebrada (+15% de dano)",
            Hex::Bleed => "sangramento",
        }
    }
}

/// How much softer a monster hits with its debuffs (mobs.rs multiplies its hits by this).
pub fn hit_scale(debuffs: &[(u8, f32)]) -> f32 {
    if debuffs.iter().any(|d| d.0 == WEAKNESS) {
        0.75
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------------------------
// Instruments.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Family {
    Strings,
    Winds,
    Percussion,
    Keys,
}

impl Family {
    pub fn name(self) -> &'static str {
        match self {
            Family::Strings => "Cordas",
            Family::Winds => "Sopro",
            Family::Percussion => "Percussão",
            Family::Keys => "Teclas",
        }
    }

    /// The colour of its notes.
    pub fn tint(self) -> [f32; 3] {
        match self {
            Family::Strings => [1.0, 0.82, 0.4],
            Family::Winds => [0.55, 1.0, 0.85],
            Family::Percussion => [1.0, 0.55, 0.35],
            Family::Keys => [0.7, 0.75, 1.0],
        }
    }
}

/// The shape of a sonic attack, in 3D.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Wave {
    /// Everything in front within `range` blocks and `half` radians of the aim.
    Cone { range: f32, half: f32 },
    /// Everything along the aim within `range` and `width` blocks of the line (pierces).
    Line { range: f32, width: f32 },
    /// Everything within `radius` of the bard, all around.
    Ring { radius: f32 },
    /// The `count` nearest monsters in sight within `range` of the aim (homing notes).
    Seek { range: f32, count: usize },
}

pub struct Inst {
    pub name: &'static str,
    pub family: Family,
    /// Balance stage (balance.rs STAGES).
    #[allow(dead_code)]
    pub stage: u8,
    /// Sonic attack damage per target hit, and seconds between attacks.
    pub damage: f32,
    pub cd: f32,
    pub wave: Wave,
    /// Song buff and its level (0 = I).
    pub song: u8,
    pub level: u8,
    /// What the song does to monsters, and for how long after each pulse.
    pub hex: Hex,
    pub hex_secs: f32,
    /// Inspiration a song costs.
    pub cost: f32,
    /// Blocks the song reaches.
    pub radius: f32,
}

/// Every instrument, in the order of `mods::melodies::INSTRUMENTS` (so `instruments::index`
/// is also an index here).
pub const INSTRUMENTS: &[Inst] = &[
    Inst {
        name: "lute",
        family: Family::Strings,
        stage: 0,
        damage: 5.2,
        cd: 0.75,
        wave: Wave::Cone {
            range: 7.0,
            half: 0.6,
        },
        song: SONG_VALOR,
        level: 0,
        hex: Hex::Weak,
        hex_secs: 3.0,
        cost: 4.0,
        radius: 9.0,
    },
    Inst {
        name: "flute",
        family: Family::Winds,
        stage: 0,
        damage: 5.6,
        cd: 0.7,
        wave: Wave::Line {
            range: 16.0,
            width: 0.6,
        },
        song: SONG_SWIFT,
        level: 0,
        hex: Hex::Slow,
        hex_secs: 3.0,
        cost: 3.0,
        radius: 10.0,
    },
    Inst {
        name: "cello",
        family: Family::Strings,
        stage: 1,
        damage: 9.5,
        cd: 0.85,
        wave: Wave::Cone {
            range: 6.0,
            half: 0.5,
        },
        song: SONG_WARD,
        level: 0,
        hex: Hex::Weak,
        hex_secs: 3.5,
        cost: 5.0,
        radius: 9.0,
    },
    Inst {
        name: "didgeridoo",
        family: Family::Winds,
        stage: 0,
        damage: 6.6,
        cd: 0.8,
        wave: Wave::Ring { radius: 4.0 },
        song: SONG_BREATH,
        level: 0,
        hex: Hex::Fear,
        hex_secs: 2.5,
        cost: 4.0,
        radius: 8.0,
    },
    Inst {
        name: "bagpipe",
        family: Family::Winds,
        stage: 1,
        damage: 8.0,
        cd: 0.75,
        wave: Wave::Line {
            range: 14.0,
            width: 0.8,
        },
        song: SONG_RALLY,
        level: 0,
        hex: Hex::Fear,
        hex_secs: 3.0,
        cost: 6.0,
        radius: 12.0,
    },
    Inst {
        name: "piano",
        family: Family::Keys,
        stage: 1,
        damage: 8.5,
        cd: 0.8,
        wave: Wave::Seek { range: 18.0, count: 1 },
        song: SONG_FOCUS,
        level: 0,
        hex: Hex::Confuse,
        hex_secs: 3.0,
        cost: 5.0,
        radius: 9.0,
    },
    Inst {
        name: "tiny_drum",
        family: Family::Percussion,
        stage: 0,
        damage: 5.0,
        cd: 0.6,
        wave: Wave::Ring { radius: 3.5 },
        song: SONG_TEMPO,
        level: 0,
        hex: Hex::Slow,
        hex_secs: 2.5,
        cost: 3.0,
        radius: 8.0,
    },
    Inst {
        name: "handpan",
        family: Family::Percussion,
        stage: 1,
        damage: 8.0,
        cd: 0.7,
        wave: Wave::Ring { radius: 4.5 },
        song: SONG_VIGOR,
        level: 0,
        hex: Hex::Expose,
        hex_secs: 3.0,
        cost: 5.0,
        radius: 10.0,
    },
    Inst {
        name: "golden_harp",
        family: Family::Strings,
        stage: 2,
        damage: 11.6,
        cd: 0.7,
        wave: Wave::Cone { range: 8.0, half: 0.6 },
        song: SONG_VIGOR,
        level: 1,
        hex: Hex::Expose,
        hex_secs: 3.0,
        cost: 5.0,
        radius: 10.0,
    },
    Inst {
        name: "war_horn",
        family: Family::Winds,
        stage: 2,
        damage: 12.8,
        cd: 0.8,
        wave: Wave::Line { range: 20.0, width: 0.9 },
        song: SONG_RALLY,
        level: 1,
        hex: Hex::Fear,
        hex_secs: 3.5,
        cost: 6.0,
        radius: 14.0,
    },
    Inst {
        name: "crystal_violin",
        family: Family::Strings,
        stage: 3,
        damage: 13.6,
        cd: 0.5,
        wave: Wave::Line { range: 18.0, width: 0.5 },
        song: SONG_SWIFT,
        level: 1,
        hex: Hex::Confuse,
        hex_secs: 3.0,
        cost: 5.0,
        radius: 11.0,
    },
    Inst {
        name: "war_drum",
        family: Family::Percussion,
        stage: 3,
        damage: 19.3,
        cd: 0.75,
        wave: Wave::Ring { radius: 5.5 },
        song: SONG_VALOR,
        level: 1,
        hex: Hex::Weak,
        hex_secs: 3.5,
        cost: 6.0,
        radius: 10.0,
    },
    Inst {
        name: "pipe_organ",
        family: Family::Keys,
        stage: 4,
        damage: 24.9,
        cd: 0.7,
        wave: Wave::Seek { range: 22.0, count: 3 },
        song: SONG_FOCUS,
        level: 2,
        hex: Hex::Slow,
        hex_secs: 3.5,
        cost: 7.0,
        radius: 12.0,
    },
    Inst {
        name: "star_lyre",
        family: Family::Strings,
        stage: 5,
        damage: 34.0,
        cd: 0.55,
        wave: Wave::Cone { range: 10.0, half: 0.7 },
        song: SONG_WARD,
        level: 2,
        hex: Hex::Bleed,
        hex_secs: 4.0,
        cost: 8.0,
        radius: 14.0,
    },
];

pub fn inst(name: &str) -> Option<&'static Inst> {
    INSTRUMENTS.iter().find(|i| i.name == name)
}

/// Tooltip lines for an instrument.
pub fn describe(name: &str) -> Option<Vec<String>> {
    if let Some(set) = set_of(name) {
        return Some(vec![format!("Roupa de bardo ({})", set.2.split(':').next().unwrap_or(""))]);
    }
    let i = inst(name)?;
    let (song, ..) = song_info(i.song);
    let wave = match i.wave {
        Wave::Cone { range, .. } => format!("leque de notas até {range:.0} blocos"),
        Wave::Line { range, .. } => format!("linha que atravessa até {range:.0} blocos"),
        Wave::Ring { radius } => format!("onda em volta de {radius:.1} blocos"),
        Wave::Seek { range, count } => format!("{count} nota(s) que perseguem até {range:.0} blocos"),
    };
    Some(vec![
        format!("Bardo · {}", i.family.name()),
        format!("Clique: {:.1} de dano sonoro, {wave}", i.damage),
        format!(
            "Botão direito: canção \"{song}\" ({:.0} de inspiração)",
            i.cost
        ),
        format!("  {} em {:.0} blocos", song_desc(i.song), i.radius),
        format!("  Inimigos: {}", i.hex.name()),
    ])
}

// ---------------------------------------------------------------------------------------------
// Gear.

/// What a piece of bard gear adds.
#[derive(Clone, Copy, Debug)]
pub enum B {
    /// Max inspiration.
    Insp(f32),
    /// Inspiration per second.
    InspRegen(f32),
    /// Sonic damage, as a fraction.
    Dmg(f32),
    /// Sonic attack speed, as a fraction.
    Speed(f32),
    /// Song reach, as a fraction.
    Reach(f32),
    /// Song buff length, as a fraction.
    Length(f32),
    /// Song levels.
    Level(u8),
    /// Cheaper songs, as a fraction.
    Cost(f32),
    /// More different songs on at once.
    Songs(u8),
}

/// The bard's clothes, one per stage (armor_design.rs draws them): all four pieces worn give
/// the set's bonus. (prefix, stage, what it says, bard stats).
pub const SETS: &[(&str, u8, &str, &[B])] = &[
    ("troubadour", 0, "Trovador: +3 de inspiração e +6% de dano sonoro", &[B::Insp(3.0), B::Dmg(0.06)]),
    ("minstrel", 1, "Menestrel: +4 de inspiração, canções 25% mais longas, +10% de dano sonoro", &[B::Insp(4.0), B::Length(0.25), B::Dmg(0.1)]),
    ("maestro", 3, "Maestro: canções um nível acima, +6 de inspiração, +15% de dano sonoro", &[B::Level(1), B::Insp(6.0), B::InspRegen(0.3), B::Dmg(0.15)]),
    ("virtuoso", 5, "Virtuoso: canções um nível acima e 30% mais longe, uma canção a mais ao mesmo tempo, +25% de dano sonoro", &[B::Level(1), B::Reach(0.3), B::Insp(8.0), B::Songs(1), B::Dmg(0.25)]),
];

/// The bard set a piece belongs to.
pub fn set_of(item: &str) -> Option<&'static (&'static str, u8, &'static str, &'static [B])> {
    SETS.iter().find(|s| item.strip_prefix(s.0).is_some_and(|r| r.starts_with('_')))
}

/// Bard accessories: (item, what it says, general stats for rpg.rs, bard stats).
pub const GEAR: &[(&str, &str, &[B])] = &[
    (
        "metronome",
        "Ataques sonoros 12% mais rápidos",
        &[B::Speed(0.12)],
    ),
    (
        "bard_songbook",
        "Canções duram 40% mais e +2 de inspiração",
        &[B::Length(0.4), B::Insp(2.0)],
    ),
    (
        "tuning_fork",
        "+10% de dano sonoro e inspiração volta mais rápido",
        &[B::Dmg(0.1), B::InspRegen(0.3)],
    ),
    (
        "echo_shell",
        "Canções alcançam 40% mais longe",
        &[B::Reach(0.4)],
    ),
    (
        "conductor_baton",
        "Canções um nível acima e 20% mais baratas",
        &[B::Level(1), B::Cost(0.2)],
    ),
];

/// The same accessories for rpg.rs (so they go in the accessory slots).
pub const ACCESSORIES: &[AccDef] = &[
    AccDef {
        name: "metronome",
        desc: "Bardo: ataques sonoros 12% mais rápidos",
        mods: &[],
    },
    AccDef {
        name: "bard_songbook",
        desc: "Bardo: canções duram 40% mais e +2 de inspiração",
        mods: &[],
    },
    AccDef {
        name: "tuning_fork",
        desc: "Bardo: +10% de dano sonoro e inspiração mais rápida",
        mods: &[],
    },
    AccDef {
        name: "echo_shell",
        desc: "Bardo: canções alcançam 40% mais longe",
        mods: &[M::Breath(0.5)],
    },
    AccDef {
        name: "conductor_baton",
        desc: "Bardo: canções um nível acima e 20% mais baratas",
        mods: &[],
    },
];

/// The bard's numbers this frame, from gear.
#[derive(Clone, Debug)]
pub struct BardStats {
    pub max_insp: f32,
    pub insp_regen: f32,
    pub dmg: f32,
    pub crit: f32,
    pub speed: f32,
    pub reach: f32,
    pub length: f32,
    pub level: u8,
    pub cost: f32,
    /// Different songs on at once.
    pub songs: usize,
}

impl Default for BardStats {
    fn default() -> BardStats {
        BardStats {
            max_insp: 10.0,
            insp_regen: 0.25,
            dmg: 1.0,
            crit: 0.04,
            speed: 1.0,
            reach: 1.0,
            length: 1.0,
            level: 0,
            cost: 1.0,
            songs: MAX_SONGS,
        }
    }
}

impl BardStats {
    fn apply(&mut self, b: &B) {
        match *b {
            B::Insp(v) => self.max_insp += v,
            B::InspRegen(v) => self.insp_regen += v,
            B::Dmg(v) => self.dmg += v,
            B::Speed(v) => self.speed += v,
            B::Reach(v) => self.reach += v,
            B::Length(v) => self.length += v,
            B::Level(v) => self.level += v,
            B::Cost(v) => self.cost -= v,
            B::Songs(v) => self.songs += v as usize,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// State.

/// A song being played: which instrument, how long it keeps pulsing.
pub struct Run {
    pub inst: usize,
    pub left: f32,
    pub pulse: f32,
}

/// The town Bard's shop: the starting instruments, a songbook and a metronome, and the
/// Troubadour's clothes (tr3.rs lists it with the other shops).
pub const SHOP: &[(&str, u8, u32)] = &[
    ("lute", 1, crate::money::s(40)),
    ("flute", 1, crate::money::s(40)),
    ("tiny_drum", 1, crate::money::s(40)),
    ("cello", 1, crate::money::s(80)),
    ("handpan", 1, crate::money::s(80)),
    ("bard_songbook", 1, crate::money::s(60)),
    ("metronome", 1, crate::money::s(60)),
    ("troubadour_helmet", 1, crate::money::s(30)),
    ("troubadour_chestplate", 1, crate::money::s(50)),
    ("troubadour_leggings", 1, crate::money::s(40)),
    ("troubadour_boots", 1, crate::money::s(30)),
];

/// What the town Bard plays when you talk to him, in turn (song, his line).
const NPC_SONGS: &[(u8, &str)] = &[
    (SONG_VALOR, "Um hino para dar coragem! Seus golpes vão doer mais."),
    (SONG_SWIFT, "Uma dança ligeira, para pés leves."),
    (SONG_WARD, "Uma balada antiga das muralhas: proteção para a estrada."),
    (SONG_VIGOR, "Uma canção de cura, para as feridas fecharem."),
    (SONG_FOCUS, "Uma melodia calma, para a mente clara."),
];

#[derive(Default)]
pub struct Bard {
    pub insp: f32,
    pub stats: BardStats,
    pub song: Option<Run>,
    /// Seconds until the next sonic attack.
    pub cd: f32,
    /// Seconds since the last sonic attack (for the hands).
    pub since: f32,
    /// A town musician's soft song: seconds until the next pulse.
    npc_t: f32,
    started: bool,
    /// The town Bard's song: seconds until he plays again, and which song is next.
    talk_cd: f32,
    talk_k: u8,
    /// Seconds until other players' songs pulse again (multiplayer).
    remote_t: f32,
}

/// Monster `k` is in sight from `eye` (no solid block on the way).
fn clear(g: &Game, eye: Vec3, to: Vec3) -> bool {
    let d = to - eye;
    let n = (d.length() / 0.5) as usize;
    (1..n).all(|s| {
        !g.world
            .solid((eye + d * (s as f32 / n as f32)).floor().as_ivec3())
    })
}

impl Game {
    fn held_inst(&self) -> Option<usize> {
        let st = self.held_stack()?;
        crate::instruments::index(&self.items.get(st.item).name)
    }

    /// Bard gear and song stats, every frame.
    fn bard_gear(&self) -> BardStats {
        let mut b = BardStats::default();
        for st in self.t5.acc[..self.acc_slots()].iter().flatten() {
            let name = &self.items.get(st.item).name;
            if let Some((_, _, mods)) = GEAR.iter().find(|g| g.0 == name) {
                for m in mods.iter() {
                    b.apply(m);
                }
            }
        }
        // The bard's clothes: all four pieces of one set.
        let worn: Vec<&str> = self.surv.armor.iter().flatten().map(|a| self.items.get(a.item).name.as_str()).collect();
        if worn.len() == 4 {
            if let Some(set) = set_of(worn[0]).filter(|set| worn.iter().all(|w| set_of(w).is_some_and(|o| o.0 == set.0))) {
                for m in set.3.iter() {
                    b.apply(m);
                }
            }
        }
        let s = &self.t5.rpg.stats;
        // Bonuses for every class count for the bard too (the smallest class bonus is the
        // shared part), and the song of tempo speeds sonic attacks.
        let all_dmg = [MELEE, RANGED, MAGIC, SUMMON]
            .iter()
            .map(|&c| s.dmg[c])
            .fold(f32::MAX, f32::min);
        let all_crit = [MELEE, RANGED, MAGIC, SUMMON]
            .iter()
            .map(|&c| s.crit[c])
            .fold(f32::MAX, f32::min);
        b.dmg *= all_dmg.max(0.1);
        b.crit += all_crit.max(0.0);
        b.speed += 0.08 * self.effect_level(SONG_TEMPO) as f32;
        b.cost = b.cost.max(0.4);
        b
    }

    /// Songs' stats on the player (rpg.rs `compute_stats`).
    pub(crate) fn bard_song_stats(&self, s: &mut Stats) {
        for e in self.prog.effects.iter().filter(|e| is_song(e.id)) {
            apply_song(s, e.id, e.amp as f32 + 1.0);
        }
    }

    /// Give the player a song (keeps at most `MAX_SONGS` different ones).
    fn give_song(&mut self, id: u8, amp: u8, secs: f32) {
        let mut on: Vec<(u8, f32)> = self
            .prog
            .effects
            .iter()
            .filter(|e| is_song(e.id) && e.id != id)
            .map(|e| (e.id, e.time))
            .collect();
        let max = self.bard.stats.songs.max(1);
        if on.len() >= max {
            on.sort_by(|a, b| a.1.total_cmp(&b.1));
            let drop: Vec<u8> = on[..on.len() + 1 - max].iter().map(|o| o.0).collect();
            self.prog.effects.retain(|e| !drop.contains(&e.id));
        }
        self.add_effect(id, amp, secs, false);
    }

    pub(crate) fn update_bard(&mut self, dt: f32) {
        self.bard.stats = self.bard_gear();
        if !self.bard.started {
            self.bard.started = true;
            self.bard.insp = self.bard.stats.max_insp;
        }
        if dt <= 0.0 {
            return;
        }
        self.bard.talk_cd -= dt;
        let b = &mut self.bard;
        b.cd -= dt;
        b.since += dt;
        b.insp = (b.insp + b.stats.insp_regen * dt).min(b.stats.max_insp);
        // The song keeps pulsing while it plays and the instrument stays in hand.
        let held = self.held_inst();
        if let Some(run) = self.bard.song.as_mut() {
            run.left -= dt;
            run.pulse -= dt;
            let (inst, pulse_now) = (run.inst, run.pulse <= 0.0);
            if pulse_now {
                run.pulse += PULSE;
            }
            if run.left <= 0.0 || held != Some(inst) || self.melody_cd <= 0.0 {
                self.bard.song = None;
            } else if pulse_now {
                self.song_pulse(inst, self.body.pos, 1.0, self.bard.stats.level, true, true);
            }
        }
        self.npc_song(dt);
        self.remote_songs(dt);
    }

    /// Multiplayer: another player playing a song (their state's SONG flag) gives it to us
    /// when we are inside its circle, and shows the ring of light. Whoever runs the mobs
    /// (the host) also casts the song's hex on the monsters around them, so it works the same
    /// whoever plays. The level is the instrument's own (their gear stays on their machine).
    fn remote_songs(&mut self, dt: f32) {
        self.bard.remote_t -= dt;
        if self.bard.remote_t > 0.0 {
            return;
        }
        self.bard.remote_t = PULSE;
        let dim = self.dim.as_u8();
        let playing: Vec<(usize, Vec3)> = self
            .net
            .remotes
            .values()
            .filter(|r| r.state.has(net::flags::SONG) && !r.state.has(net::flags::DEAD) && r.state.dim == dim)
            .filter_map(|r| INSTRUMENTS.iter().position(|i| i.name == r.state.held).map(|k| (k, Vec3::from(r.state.pos))))
            .collect();
        let hex = !self.net.is_guest();
        for (k, at) in playing {
            if self.body.pos.distance(at) <= INSTRUMENTS[k].radius * self.bard.stats.reach {
                log::debug!("bard: song of another player ({}) reaches us", INSTRUMENTS[k].name);
            }
            self.song_pulse(k, at, 1.0, 0, true, hex);
        }
    }

    /// Talking to the town Bard: he plays you a song (level I, three minutes) and, once you
    /// hold an instrument, gives a tip on playing; then the shop opens. Again a minute later.
    pub(crate) fn bard_npc_talk(&mut self, dn: &str) {
        if self.bard.talk_cd > 0.0 {
            let tip = if self.held_inst().is_some() {
                "Clique para atacar com o som; botão direito toca a canção e gasta inspiração."
            } else {
                "Toda vila fica mais alegre com música. Leve um instrumento!"
            };
            self.toast(format!("{dn}: {tip}"), 4.0);
            return;
        }
        let (id, line) = NPC_SONGS[self.bard.talk_k as usize % NPC_SONGS.len()];
        self.bard.talk_k = self.bard.talk_k.wrapping_add(1);
        self.bard.talk_cd = 60.0;
        self.give_song(id, 0, 180.0);
        self.sfx(crate::audio::Sfx::Tune(0, self.bard.talk_k as u16 % 4));
        self.toast(format!("{dn}: {line} ({})", song_info(id).0), 5.0);
    }

    /// A town musician near the player plays a soft song: level I, half as long.
    fn npc_song(&mut self, dt: f32) {
        self.bard.npc_t -= dt;
        if self.bard.npc_t > 0.0 {
            return;
        }
        self.bard.npc_t = 3.0;
        let Some(n) = self.musicians.now.as_ref() else {
            return;
        };
        let (id, inst) = (n.mob, n.inst);
        let Some(m) = self.mobs.list.iter().find(|m| m.net_id == id) else {
            return;
        };
        let at = m.body.pos;
        if at.distance(self.body.pos) < INSTRUMENTS[inst].radius {
            let i = &INSTRUMENTS[inst];
            self.give_song(i.song, 0, SONG_SECS * 0.5);
        }
    }

    /// One pulse of a song from `at`: buffs friends, hexes monsters, a ring on the ground.
    fn song_pulse(&mut self, inst: usize, at: Vec3, power: f32, extra: u8, ring: bool, hex: bool) {
        let i = &INSTRUMENTS[inst];
        let st = self.bard.stats.clone();
        let r = i.radius * st.reach;
        let amp = i.level + extra;
        if self.body.pos.distance(at) <= r {
            self.give_song(i.song, amp, SONG_SECS * st.length * power);
        }
        let glow = self.glow_layer();
        let tint = i.family.tint();
        let mut hexed = Vec::new();
        let mut friends = Vec::new();
        for k in 0..self.mobs.list.len() {
            let m = &self.mobs.list[k];
            if !m.alive() || m.body.center().distance(at) > r {
                continue;
            }
            let def = &self.mobs.reg.defs[m.def];
            let pet = def.name.ends_with("_tame") || m.saddled;
            if def.town || pet {
                friends.push(k);
            } else if hex && self.is_hostile(k) {
                hexed.push((k, def.boss));
            }
        }
        // Friends: the song of vigour heals them; every song shows a note over them.
        for k in friends {
            let m = &mut self.mobs.list[k];
            if i.song == SONG_VIGOR {
                m.health = (m.health + 0.75 * (amp as f32 + 1.0) * PULSE).min(m.max_health);
            }
            let c = m.body.center() + Vec3::Y * 0.9;
            self.particles.glow(c, Vec3::Y * 0.6, glow, tint, 0.07, 1.0);
        }
        for (k, boss) in hexed {
            self.hex_mob(k, i.hex, i.hex_secs + PULSE, boss, at);
        }
        if ring {
            let n = (r * 3.0) as usize;
            for s in 0..n {
                let a = s as f32 / n as f32 * std::f32::consts::TAU;
                let p = at + Vec3::new(a.cos() * r, 0.15, a.sin() * r);
                self.particles
                    .glow(p, Vec3::Y * 0.25, glow, tint, 0.06, 0.9);
            }
            for s in 0..6 {
                let a = s as f32 * 1.05 + self.now as f32;
                self.particles.glow(
                    at + Vec3::Y * 1.4,
                    Vec3::new(a.cos() * 0.5, 0.9, a.sin() * 0.5),
                    glow,
                    tint,
                    0.08,
                    1.4,
                );
            }
        }
    }

    /// Put a hex on monster `k`. Bosses shrug off fear and confusion and the rest lasts half.
    fn hex_mob(&mut self, k: usize, hex: Hex, secs: f32, boss: bool, from: Vec3) {
        let secs = if boss { secs * 0.5 } else { secs };
        match hex {
            Hex::Slow => self.add_mob_debuff(k, SLOWNESS, secs),
            Hex::Weak => self.add_mob_debuff(k, WEAKNESS, secs),
            Hex::Expose => self.add_mob_debuff(k, BROKEN_ARMOR, secs),
            Hex::Bleed => self.add_mob_debuff(k, BLEEDING, secs),
            Hex::Fear if !boss => self.mobs.list[k].frighten(from, secs),
            Hex::Confuse if !boss => {
                let me = self.mobs.list[k].body.pos;
                let foe = (0..self.mobs.list.len())
                    .filter(|&j| j != k && self.mobs.list[j].alive() && self.is_hostile(j))
                    .map(|j| (j, self.mobs.list[j].body.pos.distance(me)))
                    .filter(|&(_, d)| d < 12.0)
                    .min_by(|a, b| a.1.total_cmp(&b.1));
                match foe {
                    Some((j, _)) => {
                        let id = self.mobs.list[j].net_id;
                        let m = &mut self.mobs.list[k];
                        m.foe = id;
                        m.grudge = m.grudge.max(secs);
                        self.add_mob_debuff(k, CONFUSED, secs);
                    }
                    // Nobody to fight: it just wanders off, dazed.
                    None => self.mobs.list[k].frighten(from, secs * 0.6),
                }
            }
            _ => self.add_mob_debuff(k, SLOWNESS, secs * 0.5),
        }
    }

    /// Right click started melody `secs` long on instrument `inst` (mods/melodies.rs): a song,
    /// if there's inspiration for it.
    pub(crate) fn bard_song_start(&mut self, inst: usize, secs: f32) {
        let Some(i) = INSTRUMENTS.get(inst) else {
            return;
        };
        if !self.survival {
            self.bard.song = Some(Run {
                inst,
                left: secs,
                pulse: 0.0,
            });
            return;
        }
        let cost = i.cost * self.bard.stats.cost;
        if self.bard.insp + 1e-3 < cost {
            self.toast(
                format!(
                    "Pouca inspiração para a canção ({:.0}/{:.0}): só música",
                    self.bard.insp, cost
                ),
                2.5,
            );
            return;
        }
        self.bard.insp -= cost;
        self.bard.song = Some(Run {
            inst,
            left: secs,
            pulse: 0.0,
        });
        let (name, ..) = song_info(i.song);
        self.toast(format!("Canção: {name}"), 2.0);
    }

    /// Left click with an instrument: a sonic attack. True when an instrument is in hand (the
    /// click is used up even while the attack recharges, so it doesn't dig).
    pub(crate) fn bard_attack(&mut self) -> bool {
        let Some(inst) = self.held_inst() else {
            return false;
        };
        self.mining = None;
        if self.bard.cd > 0.0 {
            return true;
        }
        let i = &INSTRUMENTS[inst];
        let st = self.bard.stats.clone();
        self.bard.cd = i.cd / st.speed.max(0.3);
        self.bard.since = 0.0;
        let eye = self.body.eye();
        let fwd = self.forward();
        let glow = self.glow_layer();
        let tint = i.family.tint();
        let alive: Vec<usize> = (0..self.mobs.list.len())
            .filter(|&k| {
                self.mobs.list[k].alive() && !self.mobs.list[k].ghost && self.is_hostile(k)
            })
            .collect();
        let mut hits: Vec<usize> = Vec::new();
        match i.wave {
            Wave::Cone { range, half } => {
                for &k in &alive {
                    let c = self.mobs.list[k].body.center();
                    let d = c - eye;
                    if d.length() < range
                        && d.normalize_or_zero().dot(fwd) > half.cos()
                        && clear(self, eye, c)
                    {
                        hits.push(k);
                    }
                }
                for s in 0..14 {
                    let a = s as f32 * 2.4;
                    let side = fwd.cross(Vec3::Y).normalize_or_zero();
                    let up = side.cross(fwd);
                    let spread = (s as f32 / 14.0) * half;
                    let dir =
                        (fwd + (side * a.cos() + up * a.sin()) * spread.tan()).normalize_or_zero();
                    self.particles
                        .glow(eye + fwd * 0.6, dir * range * 1.6, glow, tint, 0.07, 0.6);
                }
            }
            Wave::Line { range, width } => {
                for &k in &alive {
                    let c = self.mobs.list[k].body.center();
                    let t = (c - eye).dot(fwd);
                    if t > 0.0
                        && t < range
                        && (eye + fwd * t).distance(c) < width + 0.5
                        && clear(self, eye, c)
                    {
                        hits.push(k);
                    }
                }
                for s in 0..12 {
                    let p = eye + fwd * (0.8 + s as f32 * 0.25);
                    self.particles
                        .glow(p, fwd * range * 1.4, glow, tint, 0.06, 0.65);
                }
            }
            Wave::Ring { radius } => {
                let at = self.body.pos + Vec3::Y * 0.9;
                for &k in &alive {
                    let c = self.mobs.list[k].body.center();
                    if c.distance(at) < radius && clear(self, at, c) {
                        hits.push(k);
                    }
                }
                for s in 0..24 {
                    let a = s as f32 / 24.0 * std::f32::consts::TAU;
                    self.particles.glow(
                        at - Vec3::Y * 0.6,
                        Vec3::new(a.cos(), 0.05, a.sin()) * radius * 2.2,
                        glow,
                        tint,
                        0.08,
                        0.45,
                    );
                }
            }
            Wave::Seek { range, count } => {
                let mut best: Vec<(usize, Vec3)> = alive
                    .iter()
                    .map(|&k| (k, self.mobs.list[k].body.center()))
                    .filter(|(_, c)| {
                        (*c - eye).length() < range
                            && (*c - eye).normalize_or_zero().dot(fwd) > 0.5
                            && clear(self, eye, *c)
                    })
                    .collect();
                best.sort_by(|a, b| (a.1 - eye).length().total_cmp(&(b.1 - eye).length()));
                best.truncate(count.max(1));
                match best.is_empty() {
                    false => {
                        for (k, c) in best {
                            hits.push(k);
                            let d = c - eye;
                            for s in 0..5 {
                                self.particles.glow(
                                    eye + fwd * 0.6 + Vec3::Y * (s as f32 * 0.05),
                                    d * 3.0,
                                    glow,
                                    tint,
                                    0.09,
                                    0.32,
                                );
                            }
                        }
                    }
                    true => {
                        for s in 0..5 {
                            self.particles.glow(
                                eye + fwd * 0.6,
                                (fwd + Vec3::Y * (s as f32 * 0.04)) * 10.0,
                                glow,
                                tint,
                                0.08,
                                0.5,
                            );
                        }
                    }
                }
            }
        }
        let riff = (self.surv.rng.f32() * 4.0) as u8 % 4;
        self.sfx(Sfx::Riff(inst as u8, riff));
        self.anim.start_swing();
        let landed = !hits.is_empty();
        for k in hits {
            let crit = self.surv.rng.f32() < st.crit;
            let dmg = i.damage * st.dmg * if crit { 1.5 } else { 1.0 };
            let m = &mut self.mobs.list[k];
            let died = m.hurt(dmg, eye);
            let (c, h) = (m.body.center(), self.mobs.reg.defs[m.def].height);
            let def = &self.mobs.reg.defs[m.def];
            self.last_hit = Some((
                def.display_name.clone(),
                m.health.max(0.0),
                m.max_health,
                self.now + 3.0,
            ));
            self.mob_hit_fx(c, h, died);
        }
        if landed {
            self.bard.insp = (self.bard.insp + 1.0).min(self.bard.stats.max_insp);
            self.sfx(Sfx::MobHit);
        }
        true
    }

    /// The inspiration notes right of the hotbar, while an instrument is in hand, with the
    /// song playing written above them.
    pub(crate) fn draw_inspiration(&mut self, w: f32, h: f32) {
        if !self.survival || self.held_inst().is_none() {
            return;
        }
        let s = self.ui.scale;
        let max = self.bard.stats.max_insp.max(1.0);
        let notes = (max / 2.0).ceil() as i32;
        // One note per 2 inspiration, two rows of ten at most.
        let x0 = w / 2.0 + 95.0 * s;
        let y0 = h - 20.0 * s;
        for k in 0..notes {
            let (col, row) = (k % 10, k / 10);
            let fill = ((self.bard.insp - k as f32 * 2.0) / 2.0).clamp(0.0, 1.0);
            draw_note(
                &mut self.ui,
                x0 + col as f32 * 9.0 * s,
                y0 - row as f32 * 10.0 * s,
                s,
                fill,
            );
        }
        let rows = (notes + 9) / 10;
        let label = match self.bard.song.as_ref() {
            Some(run) => {
                let (name, col, _) = song_info(INSTRUMENTS[run.inst].song);
                (format!("Canção: {name}"), [col[0], col[1], col[2], 255])
            }
            None => (
                format!("Inspiração {:.0}/{:.0}", self.bard.insp.floor(), max),
                [250, 214, 140, 230],
            ),
        };
        let ts = (s * 2.0 / 3.0).round().max(1.0);
        self.ui.label(
            x0,
            y0 - (rows as f32 * 10.0 - 1.0) * s,
            &label.0,
            label.1,
            ts,
        );
    }
}

impl Game {
    /// Test hook (MC_BARD=instrument[,attack][,long]): the instrument in hand, bard gear on,
    /// three zombies in front, the song started and, with `attack`, one sonic attack.
    pub(crate) fn test_bard(&mut self, scene: &str) {
        let mut parts = scene.split(',');
        let name = parts.next().unwrap_or("lute");
        let rest: Vec<&str> = parts.collect();
        let attack = rest.contains(&"attack");
        // `long`: the song goes on for ten minutes (multiplayer tests).
        let long = rest.contains(&"long");
        self.survival = true;
        if let Some(id) = self.items.id(name) {
            self.surv.inv.slots[0] = Some(voxel::item::ItemStack::new(id, 1));
            self.selected = 0;
        }
        for (k, n) in ["bard_songbook", "echo_shell", "conductor_baton"]
            .iter()
            .enumerate()
        {
            if let Some(id) = self.items.id(n) {
                self.t5.acc[k] = Some(voxel::item::ItemStack::new(id, 1));
            }
        }
        self.bard.stats = self.bard_gear();
        self.bard.started = true;
        self.bard.insp = self.bard.stats.max_insp;
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let side = flat.cross(Vec3::Y);
        for x in [-1.5f32, 0.0, 1.5] {
            let at = self.body.pos + flat * 5.0 + side * x;
            let y = self
                .world
                .surface_y(at.x.floor() as i32, at.z.floor() as i32)
                .map(|y| y as f32 + 1.0)
                .unwrap_or(at.y);
            self.mobs.spawn("zombie", Vec3::new(at.x, y, at.z));
        }
        if let Some(inst) = self.held_inst() {
            let len = if long { 600.0 } else { crate::mods::melodies::melody_seconds(0, inst as u8).clamp(2.0, 40.0) };
            self.melody_cd = len;
            self.mods.melodies.playing =
                Some(crate::mods::melodies::Playing { inst, tune: 0, len });
            self.bard_song_start(inst, len);
            self.song_pulse(inst, self.body.pos, 1.0, self.bard.stats.level, true, true);
            if attack {
                self.bard_attack();
            }
            for m in self
                .mobs
                .list
                .iter()
                .filter(|m| self.mobs.reg.defs[m.def].name == "zombie")
            {
                log::info!(
                    "bard test: zombie {:.1}/{:.1} debuffs {:?} scare {:.1}",
                    m.health,
                    m.max_health,
                    m.debuffs,
                    m.scare
                );
            }
            log::info!("bard test: effects {:?}", self.prog.effects);
        }
    }
}

/// A quaver, 7 × 8 interface pixels, filled from the bottom.
fn draw_note(ui: &mut crate::ui::UiBatch, x: f32, y: f32, s: f32, fill: f32) {
    const ROWS: [&str; 8] = [
        "..####.", "..#..##", "..#....", "..#....", "..#....", "###....", "####...", ".##....",
    ];
    for (r, row) in ROWS.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            if ch != '#' {
                continue;
            }
            let full = (8 - r) as f32 / 8.0 <= fill + 0.01;
            let col = if full {
                [250, 200, 80, 255]
            } else {
                [60, 44, 20, 220]
            };
            ui.rect(x + c as f32 * s, y + r as f32 * s, s, s, col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::{STAGES, TARGET};

    /// How much of a sonic attack's damage lands on one target, in 3D: cones and lines need
    /// aim, rings need the bard close, seeking notes always find it.
    fn factor(w: Wave) -> f32 {
        match w {
            Wave::Cone { .. } => 0.95,
            Wave::Line { .. } => 0.9,
            Wave::Ring { .. } => 0.85,
            Wave::Seek { .. } => 1.0,
        }
    }

    /// Single-target damage per second of an instrument, with its own song on the bard.
    fn dps(i: &Inst) -> f32 {
        let crit = 1.0 + 0.04 * 0.5;
        let mut d = i.damage / i.cd * factor(i.wave) * crit;
        let lvl = i.level as f32 + 1.0;
        d *= match i.song {
            SONG_VALOR => 1.0 + 0.06 * lvl,
            SONG_TEMPO => 1.0 + 0.08 * lvl,
            SONG_FOCUS => 1.0 + 0.04 * lvl * 0.5,
            SONG_RALLY => 1.0 + 0.05 * lvl * 0.5,
            _ => 1.0,
        };
        d
    }

    #[test]
    fn bard_sheet() {
        for (s, name) in STAGES.iter().enumerate() {
            let best = INSTRUMENTS
                .iter()
                .filter(|i| i.stage as usize == s)
                .map(dps)
                .fold(0.0, f32::max);
            if best > 0.0 {
                println!(
                    "{name}: melhor instrumento {best:.1} DPS ({:.0}% da meta {})",
                    best / TARGET[s] * 100.0,
                    TARGET[s]
                );
            }
        }
        for i in INSTRUMENTS {
            let d = dps(i);
            let share = d / TARGET[i.stage as usize];
            println!("{:12} {:>5.1} DPS  {:>4.0}%", i.name, d, share * 100.0);
            // A support class: a little under the others alone (75%-100% of the stage target).
            assert!(
                (0.75..=1.0).contains(&share),
                "{} does {:.0}% of its stage",
                i.name,
                share * 100.0
            );
        }
    }

    #[test]
    fn every_instrument_is_a_bard_instrument() {
        assert_eq!(INSTRUMENTS.len(), crate::mods::melodies::INSTRUMENTS.len());
        for (k, n) in crate::mods::melodies::INSTRUMENTS.iter().enumerate() {
            assert_eq!(INSTRUMENTS[k].name, *n);
            assert!(is_song(INSTRUMENTS[k].song));
            assert!(describe(n).is_some());
        }
        for id in FIRST_SONG..=LAST_SONG {
            assert_ne!(song_info(id).0, "?");
            assert!(!song_desc(id).is_empty());
            assert!((id as u32) < 128, "the immunity mask is a u128");
        }
        for g in GEAR {
            assert!(
                ACCESSORIES.iter().any(|a| a.name == g.0),
                "{} is no accessory",
                g.0
            );
        }
    }

    #[test]
    fn songs_stack_by_kind() {
        let mut s = Stats::default();
        apply_song(&mut s, SONG_VALOR, 2.0);
        apply_song(&mut s, SONG_WARD, 1.0);
        assert!((s.dmg[MELEE] - 1.12).abs() < 1e-4);
        assert!((s.defense - 3.0).abs() < 1e-4);
    }

    /// The bard's clothes keep up with the stage's armour (balance.rs's floor) and every piece
    /// is an item with a drawn design.
    #[test]
    fn bard_clothes_protect() {
        let items = crate::game_data(crate::embedded::BLOCKS_RON, crate::embedded::ITEMS_RON, crate::embedded::RECIPES_RON).unwrap().1;
        for (prefix, stage, ..) in SETS {
            let d = crate::balance::set_defense(&items, prefix);
            let (top, best) = crate::balance::best_defense(&items, *stage);
            println!("{prefix}: defesa {d} de {top} ({best})");
            assert!(d >= crate::balance::ARMOUR_FLOOR * top, "{prefix} has {d} of {top}");
            for k in ["helmet", "chestplate", "leggings", "boots"] {
                let n = format!("{prefix}_{k}");
                assert!(items.id(&n).is_some(), "{n}");
                assert_eq!(set_of(&n).map(|s| s.0), Some(*prefix));
            }
        }
    }

    #[test]
    fn weakened_monsters_hit_softer() {
        assert_eq!(hit_scale(&[]), 1.0);
        assert_eq!(hit_scale(&[(WEAKNESS, 2.0)]), 0.75);
    }
}
