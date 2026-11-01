//! The deep world (Terraria's idea): under y 0 the ground goes on for 768 more blocks.
//!
//! - Profundezas (0 to -160): deepslate, the old cave system carried on.
//! - Cavernas (-160 to about -600): plain stone again, with ever bigger caverns, long winding
//!   tunnels at five depths that run for thousands of blocks, chasms that drop hundreds of
//!   blocks (some all the way to the Underworld), lakes of water and, deeper, of lava.
//!   Underground biomes: the jungle, the ice and the desert carry on under their surface biomes;
//!   elsewhere glowing mushroom caverns, marble (calcite) and granite caves, spider nests and gem
//!   caverns.
//! - Submundo (the Underworld, below about -610): a huge open hell between an ash ceiling and an
//!   ash floor, a sea of lava, ember trees, magmarite ore and ruined obsidian towns.
//!
//! Speed: the 3D cave field is sampled on a 4×8×4 grid and interpolated, every other decision is
//! per column, the whole depth is filled into a flat buffer and packed into sections at once.
//! Structures (cabins on the long tunnels, mineshafts, shrines, gold vaults, ruined towers and
//! houses of the Underworld) are planned per 48-block cell from the seed alone, so each chunk
//! draws its own part of them without looking at its neighbours.

use glam::IVec3;

use super::{Biome, Column, Generator};
use crate::block::{BlockId, BlockRegistry, AIR};
use crate::chunk::{Chunk, ChunkPos, PalettedSection, BASE_SECTION, DEPTH, MIN_Y};
use crate::noise::Perlin;
use crate::rng::Rng;
use crate::structures::LootDef;

/// Deepslate gives way to stone again here.
pub const DEEPSLATE_BOTTOM: i32 = -160;
/// Roughly where the Underworld's ceiling hangs.
pub const UNDERWORLD_TOP: i32 = -600;
/// The lava sea of the Underworld.
pub const LAVA_SEA: i32 = -726;
/// The long tunnels: one network at each of these depths.
const TUNNEL_Y: [i32; 5] = [-90, -205, -320, -435, -540];
/// Structure planning cells (blocks).
const CELL: i32 = 48;

/// The layers of the world, top to bottom (for the depth meter, music and monsters).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    Sky,
    Surface,
    Underground,
    Deep,
    Caverns,
    Underworld,
}

impl Layer {
    pub fn at(y: i32) -> Layer {
        if y >= 160 {
            Layer::Sky
        } else if y >= 40 {
            Layer::Surface
        } else if y >= 0 {
            Layer::Underground
        } else if y >= DEEPSLATE_BOTTOM {
            Layer::Deep
        } else if y >= UNDERWORLD_TOP - 8 {
            Layer::Caverns
        } else {
            Layer::Underworld
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Layer::Sky => "Céu",
            Layer::Surface => "Superfície",
            Layer::Underground => "Subsolo",
            Layer::Deep => "Profundezas",
            Layer::Caverns => "Cavernas",
            Layer::Underworld => "Submundo",
        }
    }
}

/// Underground biome of a place.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UBiome {
    Plain,
    Jungle,
    Ice,
    Desert,
    Mushroom,
    Marble,
    Granite,
    Spider,
    Gems,
    Underworld,
}

impl UBiome {
    pub fn name(self) -> &'static str {
        match self {
            UBiome::Plain => "Cavernas",
            UBiome::Jungle => "Selva Subterrânea",
            UBiome::Ice => "Cavernas de Gelo",
            UBiome::Desert => "Deserto Subterrâneo",
            UBiome::Mushroom => "Cavernas de Cogumelos Brilhantes",
            UBiome::Marble => "Cavernas de Mármore",
            UBiome::Granite => "Cavernas de Granito",
            UBiome::Spider => "Ninho de Aranhas",
            UBiome::Gems => "Cavernas de Gemas",
            UBiome::Underworld => "Submundo",
        }
    }
}

#[derive(Clone)]
pub(super) struct DeepNoise {
    cave_a: Perlin,
    cave_b: Perlin,
    cheese: Perlin,
    cheese2: Perlin,
    tunnels: [Perlin; 5],
    tunnel_y: Perlin,
    chasm: Perlin,
    chasm_mask: Perlin,
    uw: Perlin,
    uw2: Perlin,
    region: [Perlin; 4],
    lake: Perlin,
    rock: Perlin,
}

impl DeepNoise {
    pub(super) fn new(seed: u64) -> Self {
        let p = |k: u64| Perlin::new(seed ^ 0xDEE9_0000 ^ k.wrapping_mul(0x9E37_79B9));
        DeepNoise {
            cave_a: p(1),
            cave_b: p(2),
            cheese: p(3),
            cheese2: p(4),
            tunnels: [p(10), p(11), p(12), p(13), p(14)],
            tunnel_y: p(15),
            chasm: p(20),
            chasm_mask: p(21),
            uw: p(30),
            uw2: p(31),
            region: [p(40), p(41), p(42), p(43)],
            lake: p(50),
            rock: p(60),
        }
    }
}

#[derive(Clone)]
pub(super) struct DeepPalette {
    stone: BlockId,
    deepslate: BlockId,
    bedrock: BlockId,
    lava: BlockId,
    water: BlockId,
    blobs: Vec<BlockId>,
    mud: BlockId,
    moss: BlockId,
    short_grass: BlockId,
    vines: BlockId,
    jungle_leaves: BlockId,
    bee_nest: BlockId,
    honeycomb: BlockId,
    packed_ice: BlockId,
    ice: BlockId,
    snow: BlockId,
    icicle: BlockId,
    snow_layer: BlockId,
    sandstone: BlockId,
    smooth_sandstone: BlockId,
    sand: BlockId,
    dead_bush: BlockId,
    calcite: BlockId,
    granite: BlockId,
    cobweb: BlockId,
    gems: Vec<BlockId>,
    amethyst_cluster: BlockId,
    spike_up: BlockId,
    spike_down: BlockId,
    ash: BlockId,
    ash_grass: BlockId,
    basalt: BlockId,
    blackstone: BlockId,
    magma: BlockId,
    obsidian: BlockId,
    magmarite: BlockId,
    ember_bloom: BlockId,
    cinder_log: BlockId,
    ember_leaves: BlockId,
    glow_moss: BlockId,
    /// Glowing mosses of the deep caverns and gem trees (Lote 22 of the Terraria side).
    mosses: Vec<BlockId>,
    gem_log: BlockId,
    gem_leaves: Vec<BlockId>,
    glow_mushroom: BlockId,
    glow_cap: BlockId,
    mushroom_stem: BlockId,
    life_crystal: BlockId,
    tuff: BlockId,
    andesite: BlockId,
    diorite: BlockId,
    // Building blocks.
    planks: [BlockId; 3],
    logs: [BlockId; 3],
    fence: BlockId,
    rail: BlockId,
    cobble: BlockId,
    mossy_cobble: BlockId,
    bricks: BlockId,
    mossy_bricks: BlockId,
    cracked_bricks: BlockId,
    chest: BlockId,
    torch: BlockId,
    lantern: BlockId,
    chain: BlockId,
    gold_block: BlockId,
    bookshelf: BlockId,
    obsidian_bricks: BlockId,
    infernal_bricks: BlockId,
    soul_lantern: BlockId,
    iron_bars: BlockId,
    hive: BlockId,
    larva: BlockId,
    honey: BlockId,
    spawner: BlockId,
    dirt: BlockId,
    lantern_hang: BlockId,
}

impl DeepPalette {
    pub(super) fn new(r: &BlockRegistry, seed: u64) -> Self {
        let g = |n: &str| r.id_or_air(n);
        DeepPalette {
            stone: g("stone"),
            deepslate: if g("deepslate") == AIR { g("stone") } else { g("deepslate") },
            bedrock: g("bedrock"),
            lava: g("lava"),
            water: g("water"),
            blobs: ["tuff", "granite", "andesite", "diorite", "dirt", "gravel", "clay", "dirt", "tuff"].iter().map(|n| g(n)).filter(|&b| b != AIR).collect(),
            mud: g("mud"),
            moss: g("moss_block"),
            short_grass: g("short_grass"),
            vines: g("cave_vines"),
            jungle_leaves: g("jungle_leaves"),
            bee_nest: g("bee_nest"),
            honeycomb: g("honeycomb_block"),
            packed_ice: g("packed_ice"),
            ice: g("ice"),
            snow: g("snow"),
            icicle: g("icicle"),
            snow_layer: g("snow_layer"),
            sandstone: g("sandstone"),
            smooth_sandstone: g("smooth_sandstone"),
            sand: g("sand"),
            dead_bush: g("dead_bush"),
            calcite: g("calcite"),
            granite: g("granite"),
            cobweb: g("cobweb"),
            gems: ["ruby_ore", "sapphire_ore", "topaz_ore", "amethyst_ore", "diamond_ore", "amethyst_block"].iter().map(|n| g(n)).filter(|&b| b != AIR).collect(),
            amethyst_cluster: g("amethyst_cluster"),
            spike_up: g("pointed_dripstone"),
            spike_down: g("pointed_dripstone_down"),
            ash: g("ash_block"),
            ash_grass: g("ash_grass"),
            basalt: g("basalt"),
            blackstone: g("blackstone"),
            magma: g("magma_block"),
            obsidian: g("obsidian"),
            magmarite: g("magmarite_ore"),
            ember_bloom: g("ember_bloom"),
            cinder_log: g("cinder_log"),
            ember_leaves: g("ember_leaves"),
            glow_moss: g("glow_moss"),
            mosses: ["neon_moss", "lava_moss", "helium_moss", "argon_moss", "xenon_moss", "krypton_moss"].iter().map(|n| g(n)).filter(|&b| b != AIR).collect(),
            gem_log: g("gem_tree_log"),
            gem_leaves: ["amethyst_gem_leaves", "topaz_gem_leaves", "sapphire_gem_leaves", "emerald_gem_leaves", "ruby_gem_leaves", "diamond_gem_leaves"].iter().map(|n| g(n)).filter(|&b| b != AIR).collect(),
            glow_mushroom: g("glow_mushroom"),
            glow_cap: g("glow_mushroom_block"),
            mushroom_stem: g("mushroom_stem"),
            life_crystal: g("life_crystal"),
            tuff: g("tuff"),
            andesite: g("andesite"),
            diorite: g("diorite"),
            planks: [g("oak_planks"), g("spruce_planks"), g("dark_oak_planks")],
            logs: [g("oak_log"), g("spruce_log"), g("dark_oak_log")],
            fence: g("oak_fence"),
            rail: g("rail"),
            cobble: g("cobblestone"),
            mossy_cobble: g("mossy_cobblestone"),
            bricks: g("stone_bricks"),
            mossy_bricks: g("mossy_stone_bricks"),
            cracked_bricks: g("cracked_stone_bricks"),
            chest: g("chest"),
            torch: g("torch"),
            lantern: g("lantern"),
            chain: g("chain"),
            gold_block: g("gold_block"),
            bookshelf: g("bookshelf"),
            obsidian_bricks: g("obsidian_bricks"),
            infernal_bricks: g("infernal_bricks"),
            soul_lantern: g("soul_lantern"),
            iron_bars: g("iron_bars"),
            hive: if g("hive_block") == AIR { g("honeycomb_block") } else { g("hive_block") },
            larva: g("hive_larva"),
            honey: if g("hive_honey") == AIR { g("honeycomb_block") } else { g("hive_honey") },
            spawner: g("spawner"),
            dirt: g("dirt"),
            lantern_hang: r.variant(g("lantern"), 1),
        }
    }

    fn or(&self, a: BlockId, b: BlockId) -> BlockId {
        if a == AIR {
            b
        } else {
            a
        }
    }
}

/// What the deep generator decided for one column.
#[derive(Clone, Copy, Default)]
struct DeepCol {
    uw_floor: i32,
    uw_ceil: i32,
    /// (centre y, half height) of the long tunnel crossing this column, per network.
    tunnels: [Option<(f32, f32)>; 5],
    /// Chasm: (top, bottom, how far in from the edge 0..1).
    chasm: Option<(i32, i32)>,
    /// Lake surface (water above -420, lava below), when this column is in a lake region.
    lake: Option<i32>,
    surface: SurfaceKind,
    /// Region noises: mushroom, marble/granite, spider, gems.
    region: [f32; 4],
}

#[derive(Clone, Copy, Default, PartialEq)]
enum SurfaceKind {
    #[default]
    Other,
    Jungle,
    Ice,
    Desert,
}

fn surface_kind(b: Biome) -> SurfaceKind {
    match b {
        Biome::Jungle | Biome::BambooJungle | Biome::SparseJungle | Biome::MataAtlantica => SurfaceKind::Jungle,
        Biome::Snowy | Biome::Taiga | Biome::SnowyPeaks | Biome::FrozenOcean | Biome::SnowyTaiga | Biome::IceSpikes | Biome::SnowySlopes | Biome::Grove | Biome::JaggedPeaks => SurfaceKind::Ice,
        Biome::Desert | Biome::Badlands | Biome::WoodedBadlands | Biome::ErodedBadlands | Biome::Caatinga => SurfaceKind::Desert,
        _ => SurfaceKind::Other,
    }
}

/// The part of the deep being generated: local heights `lo..hi` (from the bottom of the world),
/// laid out like sections (x + z*16 + y*256).
struct Buf {
    d: Vec<BlockId>,
    lo: usize,
    hi: usize,
}

impl Buf {
    #[inline(always)]
    fn i(&self, x: usize, ly: usize, z: usize) -> usize {
        x | (z << 4) | ((ly - self.lo) << 8)
    }

    /// World heights covered.
    fn ys(&self) -> std::ops::Range<i32> {
        self.lo as i32 + MIN_Y..self.hi as i32 + MIN_Y
    }
}

#[inline(always)]
fn ly_of(y: i32) -> usize {
    (y - MIN_Y) as usize
}

/// A random number in 0..1 that only depends on the place (so any slice of the deep can be
/// generated on its own and still match).
#[inline(always)]
fn cell_rand(seed: u64, x: i32, y: i32, z: i32, salt: u64) -> f32 {
    (crate::rng::hash3(seed ^ salt, x as i64, y as i64, z as i64) >> 40) as f32 / (1u64 << 24) as f32
}

/// The kinds of buildings down there.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeepKind {
    /// A wooden cabin on a long tunnel.
    Cabin,
    /// A long mine corridor with timber frames and rails.
    Mineshaft,
    /// A mossy stone shrine with a pedestal and a treasure chest.
    Shrine,
    /// A sealed stone-brick vault deep down, lined with gold.
    Vault,
    /// A ruined obsidian house of the Underworld.
    RuinedHouse,
    /// A ruined tower of the Underworld.
    RuinedTower,
    /// A giant bees' hive in the underground jungle, with the royal larva at its heart.
    Hive,
    /// A whole abandoned mine like Java's: a dirt room with corridors branching out of it,
    /// crossings, stairs between levels, plank bridges over caves, rails, chests and cave
    /// spider nests.
    MineNetwork,
}

impl DeepKind {
    pub fn name(self) -> &'static str {
        match self {
            DeepKind::Cabin => "cabana_subterranea",
            DeepKind::Mineshaft => "mina_abandonada",
            DeepKind::Shrine => "santuario_da_espada",
            DeepKind::Vault => "cofre_dourado",
            DeepKind::RuinedHouse => "casa_em_ruinas",
            DeepKind::RuinedTower => "torre_em_ruinas",
            DeepKind::Hive => "colmeia_gigante",
            DeepKind::MineNetwork => "rede_de_minas",
        }
    }
}

/// One planned building: kind, corner (lowest x, y, z), size and a seed for its details.
#[derive(Clone, Copy, Debug)]
pub struct DeepPlan {
    pub kind: DeepKind,
    pub origin: IVec3,
    pub size: IVec3,
    seed: u64,
}

/// Blocks of a building being drawn into one chunk's buffer (writes outside it are dropped).
struct Draw<'a> {
    buf: &'a mut Buf,
    bx: i32,
    bz: i32,
}

impl Draw<'_> {
    #[inline]
    fn inside(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let (lx, lz) = (x - self.bx, z - self.bz);
        let ys = self.buf.ys();
        if (0..16).contains(&lx) && (0..16).contains(&lz) && y >= ys.start.max(MIN_Y + 5) && y < ys.end.min(0) {
            Some(self.buf.i(lx as usize, ly_of(y), lz as usize))
        } else {
            None
        }
    }
    fn set(&mut self, x: i32, y: i32, z: i32, id: BlockId) {
        if id == BlockId::MAX {
            return;
        }
        if let Some(i) = self.inside(x, y, z) {
            self.buf.d[i] = id;
        }
    }
    fn get(&self, x: i32, y: i32, z: i32) -> Option<BlockId> {
        self.inside(x, y, z).map(|i| self.buf.d[i])
    }
    fn fill(&mut self, a: IVec3, b: IVec3, id: BlockId) {
        let ys = self.buf.ys();
        for y in a.y.max(ys.start)..=b.y.min(ys.end - 1) {
            for z in a.z..=b.z {
                for x in a.x..=b.x {
                    self.set(x, y, z, id);
                }
            }
        }
    }
}

impl Generator {
    fn deep_col(&self, wx: i32, wz: i32, col: &Column) -> DeepCol {
        self.deep_col_in(wx, wz, col, MIN_Y..0)
    }

    /// Only what matters for heights `ys` is worked out (the Underworld's shape is skipped
    /// higher up, tunnels far from the slice too).
    fn deep_col_in(&self, wx: i32, wz: i32, col: &Column, ys: std::ops::Range<i32>) -> DeepCol {
        let n = &self.deep;
        let (x, z) = (wx as f32, wz as f32);
        let (floor, ceil) = if ys.start < -560 {
            let floor = -712.0 + 18.0 * n.uw.fbm2(x / 140.0, z / 140.0, 3) + 9.0 * n.uw2.noise2(x / 38.0, z / 38.0);
            let hang = (n.uw2.noise2(x / 23.0 + 50.0, z / 23.0).max(0.0) * 26.0).powf(1.1);
            (floor, (-626.0 + 12.0 * n.uw.noise2(x / 90.0 + 300.0, z / 90.0) - hang).max(floor + 16.0))
        } else {
            (-712.0, -626.0)
        };
        let mut tunnels = [None; 5];
        for (k, t) in tunnels.iter_mut().enumerate() {
            if TUNNEL_Y[k] + 30 < ys.start || TUNNEL_Y[k] - 30 >= ys.end {
                continue;
            }
            let l = n.tunnels[k].noise2(x / 260.0, z / 260.0).abs();
            if l < 0.03 {
                let cy = TUNNEL_Y[k] as f32 + 16.0 * n.tunnel_y.noise2(x / 150.0 + k as f32 * 40.0, z / 150.0);
                let hh = (3.4 + 1.6 * n.tunnel_y.noise2(x / 37.0, z / 37.0 + k as f32 * 9.0)) * (1.0 - l / 0.03).sqrt();
                if hh > 0.8 {
                    *t = Some((cy, hh));
                }
            }
        }
        let mask = n.chasm_mask.noise2(x / 700.0, z / 700.0);
        let c = n.chasm.noise2(x / 480.0, z / 480.0).abs();
        let width = 0.010 + 0.006 * (mask - 0.3).max(0.0);
        let chasm = (mask > 0.3 && c < width).then(|| {
            // The deepest ones open into the Underworld.
            let bottom = if mask > 0.55 { ceil as i32 } else { -560 + (mask * 200.0) as i32 };
            let top = -14 - ((1.0 - c / width) * 4.0) as i32;
            (top, bottom)
        });
        let lw = n.lake.noise2(x / 120.0, z / 120.0);
        let lake = (lw > 0.5).then(|| {
            let mut r = Rng::for_pos(self.seed, wx.div_euclid(96), wz.div_euclid(96), 0x1A4E);
            r.range(-570, -40)
        });
        DeepCol {
            uw_floor: floor as i32,
            uw_ceil: ceil as i32,
            tunnels,
            chasm,
            lake,
            surface: surface_kind(col.biome),
            region: [
                n.region[0].noise2(x / 260.0, z / 260.0),
                n.region[1].noise2(x / 300.0, z / 300.0),
                n.region[2].noise2(x / 150.0, z / 150.0),
                n.region[3].noise2(x / 280.0, z / 280.0),
            ],
        }
    }

    fn ubiome_of(dc: &DeepCol, y: i32) -> UBiome {
        if y < dc.uw_ceil + 2 {
            return UBiome::Underworld;
        }
        if y > -14 || y < UNDERWORLD_TOP + 20 {
            return UBiome::Plain;
        }
        if y > -420 {
            match dc.surface {
                SurfaceKind::Jungle => return UBiome::Jungle,
                SurfaceKind::Ice => return UBiome::Ice,
                SurfaceKind::Desert => return UBiome::Desert,
                SurfaceKind::Other => {}
            }
        }
        let [m, k, s, g] = dc.region;
        if m > 0.42 && (-400..-60).contains(&y) {
            UBiome::Mushroom
        } else if g > 0.45 && y < -300 {
            UBiome::Gems
        } else if k > 0.5 && (-320..-60).contains(&y) {
            UBiome::Marble
        } else if k < -0.5 && (-360..-60).contains(&y) {
            UBiome::Granite
        } else if s > 0.55 && (-450..-40).contains(&y) {
            UBiome::Spider
        } else {
            UBiome::Plain
        }
    }

    /// The underground biome at a place (F3 screen, music, monsters).
    pub fn ubiome_at(&self, wx: i32, y: i32, wz: i32) -> UBiome {
        if self.dim != super::Dim::Overworld || y >= 0 {
            return UBiome::Plain;
        }
        let col = self.column(wx, wz);
        Self::ubiome_of(&self.deep_col_in(wx, wz, &col, y..y + 1), y)
    }

    /// The Underworld's floor height at a column.
    pub fn underworld_floor(&self, wx: i32, wz: i32) -> i32 {
        let col = Column { height: 0, biome: Biome::Plains, variant: Biome::Plains };
        self.deep_col(wx, wz, &col).uw_floor
    }

    /// Cave density on the coarse grid: below zero is open.
    fn density(&self, x: f32, y: f32, z: f32) -> f32 {
        let n = &self.deep;
        let d = (-y / 600.0).clamp(0.0, 1.0);
        let a = n.cave_a.noise3(x / 72.0, y / 34.0, z / 72.0);
        let b = n.cave_b.noise3(x / 72.0, y / 34.0, z / 72.0);
        // The Profundezas (deepslate, 0 to -160) used to be nearly solid between y -16 and -60:
        // a bump of extra tunnels and rooms there, fading out at both ends.
        let pf = ((-y - 1.0) / 12.0).clamp(0.0, 1.0).min(((y + 210.0) / 70.0).clamp(0.0, 1.0));
        let sp = a.abs().max(b.abs()) - (0.05 + 0.035 * d + 0.022 * pf);
        let c = n.cheese.noise3(x / 96.0, y / 46.0, z / 96.0) + 0.45 * n.cheese2.noise3(x / 38.0, y / 22.0, z / 38.0);
        let ch = (0.66 - 0.26 * d - 0.12 * pf) - c;
        sp.min(ch)
    }

    /// Fill local sections `lo..hi` (below `BASE_SECTION`) of an Overworld chunk. Any slice can be
    /// made on its own and comes out the same as the whole: the chunk gets its top sections when
    /// it is made and the rest only when someone goes down there (`Chunk::deep_lo`).
    pub fn generate_deep(&self, chunk: &mut Chunk, pos: ChunkPos, lo: usize, hi: usize) {
        let (lo, hi) = (lo.min(BASE_SECTION), hi.min(BASE_SECTION));
        if lo >= hi {
            return;
        }
        let cols: Vec<Column> = (0..256).map(|i| self.column(pos.x * 16 + (i & 15) as i32, pos.y * 16 + (i >> 4) as i32)).collect();
        self.generate_deep_with(chunk, &cols, pos, lo, hi);
    }

    pub(super) fn generate_deep_with(&self, chunk: &mut Chunk, cols: &[Column], pos: ChunkPos, lo: usize, hi: usize) {
        let p = &self.deep_pal;
        let (bx, bz) = (pos.x * 16, pos.y * 16);
        // One section more above and below, so cave floors and hanging things at the edges of
        // the slice come out right.
        let (blo, bhi) = (lo.saturating_sub(1) * 16, ((hi + 1).min(BASE_SECTION)) * 16);
        let ys = blo as i32 + MIN_Y..bhi as i32 + MIN_Y;
        let dcols: Vec<DeepCol> = (0..256).map(|i| self.deep_col_in(bx + (i & 15) as i32, bz + (i >> 4) as i32, &cols[i], ys.clone())).collect();
        let mut buf = Buf { d: vec![AIR; (bhi - blo) * 256], lo: blo, hi: bhi };
        let seed = self.seed;

        // Coarse density grid: every 4 blocks across, 8 up.
        const GY: usize = DEPTH / 8 + 1;
        let low_uw = dcols.iter().map(|d| d.uw_ceil).min().unwrap_or(UNDERWORLD_TOP);
        let g_lo = (ly_of(low_uw) / 8).saturating_sub(1).max(blo / 8);
        let g_hi = (bhi / 8 + 1).min(GY - 1);
        let mut grid = vec![0f32; 5 * 5 * GY];
        for gz in 0..5 {
            for gx in 0..5 {
                for gy in g_lo..=g_hi {
                    let y = gy as i32 * 8 + MIN_Y;
                    grid[(gx + gz * 5) * GY + gy] = self.density((bx + gx as i32 * 4) as f32, y as f32, (bz + gz as i32 * 4) as f32);
                }
            }
        }

        let rock = &self.deep.rock;
        // Which 16-block layers of the buffer have any opening (the decorator skips the rest).
        let mut open_layer = vec![false; (bhi - blo) / 16];
        for z in 0..16usize {
            for x in 0..16usize {
                let dc = &dcols[x + z * 16];
                let (wx, wz) = (bx + x as i32, bz + z as i32);
                let (gx, fx) = (x / 4, (x % 4) as f32 / 4.0);
                let (gz, fz) = (z / 4, (z % 4) as f32 / 4.0);
                let g = |gx: usize, gz: usize, gy: usize| grid[(gx + gz * 5) * GY + gy];
                let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
                let wob = rock.noise2(wx as f32 / 40.0, wz as f32 / 40.0) * 6.0;
                let band = |y: i32, k: u32| ((((y as f32 + wob) / 5.0).floor() as i32 as u32).wrapping_mul(2_654_435_761) >> 13).wrapping_add(k) % 8;
                // Deepslate's lower edge wobbles.
                let ds_edge = DEEPSLATE_BOTTOM + (rock.noise2(wx as f32 / 30.0, wz as f32 / 30.0) * 10.0) as i32;
                let roof = dc.uw_ceil + 6;
                let mut gy_at = usize::MAX;
                let (mut c0, mut c1) = (0.0, 0.0);
                for ly in blo..bhi {
                    let y = ly as i32 + MIN_Y;
                    let i = buf.i(x, ly, z);
                    // Bedrock floor.
                    if ly == 0 || (ly < 5 && cell_rand(seed, wx, y, wz, 0xBED) * 5.0 >= ly as f32) {
                        buf.d[i] = p.bedrock;
                        continue;
                    }
                    // The Underworld.
                    if y <= dc.uw_ceil {
                        let id = if y <= dc.uw_floor {
                            if y > dc.uw_floor - 5 {
                                p.ash
                            } else if band(y, 0) < 3 {
                                p.basalt
                            } else {
                                p.or(p.ash, p.blackstone)
                            }
                        } else if y <= LAVA_SEA {
                            p.lava
                        } else if y == dc.uw_ceil {
                            p.ash
                        } else {
                            AIR
                        };
                        if id == AIR {
                            open_layer[(ly - blo) / 16] = true;
                        }
                        buf.d[i] = id;
                        continue;
                    }
                    // Its ceiling: ash and basalt, hot rock above.
                    if y < dc.uw_ceil + 14 {
                        let r = band(y, 3);
                        buf.d[i] = if r == 0 { p.magma } else if r < 4 { p.basalt } else { p.or(p.ash, p.blackstone) };
                        if dc.chasm.is_some_and(|c| c.1 <= dc.uw_ceil) {
                            buf.d[i] = AIR;
                            open_layer[(ly - blo) / 16] = true;
                        }
                        continue;
                    }
                    let base = if y > -6 {
                        if cell_rand(seed, wx, y, wz, 0xD5) * 6.0 < (-y) as f32 { p.deepslate } else { p.stone }
                    } else if y > ds_edge {
                        p.deepslate
                    } else {
                        p.stone
                    };
                    // Caves: the interpolated field, the long tunnels, the chasms.
                    let gy = ly / 8;
                    let open = if gy + 1 < GY && y > roof {
                        if gy != gy_at {
                            gy_at = gy;
                            c0 = l(l(g(gx, gz, gy), g(gx + 1, gz, gy), fx), l(g(gx, gz + 1, gy), g(gx + 1, gz + 1, gy), fx), fz);
                            c1 = l(l(g(gx, gz, gy + 1), g(gx + 1, gz, gy + 1), fx), l(g(gx, gz + 1, gy + 1), g(gx + 1, gz + 1, gy + 1), fx), fz);
                        }
                        let d = l(c0, c1, (ly % 8) as f32 / 8.0);
                        // Stay closed right under y 0 so the old caves above keep their floors.
                        d < if y > -4 { -0.05 } else { 0.0 }
                    } else {
                        false
                    };
                    let tunnel = dc.tunnels.iter().flatten().any(|&(cy, hh)| (y as f32 - cy).abs() < hh);
                    let chasm = dc.chasm.is_some_and(|(t, b)| y <= t && y >= b);
                    buf.d[i] = if open || tunnel || chasm {
                        open_layer[(ly - blo) / 16] = true;
                        match dc.lake {
                            Some(l) if y <= l && !chasm => {
                                if y < -420 {
                                    p.lava
                                } else {
                                    p.water
                                }
                            }
                            _ => AIR,
                        }
                    } else {
                        base
                    };
                }
            }
        }

        self.deep_blobs(&mut buf, pos, &dcols);
        self.deep_biomes(&mut buf, &dcols, pos);
        self.deep_ores(&mut buf, pos, &dcols);
        self.deep_decorate(&mut buf, &dcols, pos, &open_layer);
        {
            let mut d = Draw { buf: &mut buf, bx, bz };
            for plan in self.deep_plans_near(pos) {
                self.build_deep(&mut d, &plan);
            }
        }
        for s in lo..hi {
            let a = s * 16 - blo;
            chunk.sections[s] = PalettedSection::from_slice(&buf.d[a * 256..(a + 16) * 256]);
        }
        chunk.deep_lo = chunk.deep_lo.min(lo);
    }

    /// Lumps of other rock (tuff, granite, dirt, gravel, clay...), planned per chunk and drawn
    /// across chunk edges.
    fn deep_blobs(&self, buf: &mut Buf, pos: ChunkPos, dcols: &[DeepCol]) {
        let p = &self.deep_pal;
        if p.blobs.is_empty() {
            return;
        }
        let ys = buf.ys();
        for dz in -1..=1 {
            for dx in -1..=1 {
                let mut r = Rng::for_pos(self.seed, pos.x + dx, pos.y + dz, 0xB10B);
                for _ in 0..14 {
                    let cx = (pos.x + dx) * 16 + r.range(0, 15);
                    let cz = (pos.y + dz) * 16 + r.range(0, 15);
                    let cy = r.range(UNDERWORLD_TOP + 30, -4);
                    let rad = 2.5 + r.f32() * 3.5;
                    let id = p.blobs[r.below(p.blobs.len() as u32) as usize];
                    let ri = rad.ceil() as i32;
                    let (x0, x1) = ((cx - ri).max(pos.x * 16), (cx + ri).min(pos.x * 16 + 15));
                    let (z0, z1) = ((cz - ri).max(pos.y * 16), (cz + ri).min(pos.y * 16 + 15));
                    let (y0, y1) = ((cy - ri).max(ys.start), (cy + ri).min(ys.end - 1).min(-1));
                    if x0 > x1 || z0 > z1 || y0 > y1 {
                        continue;
                    }
                    for y in y0..=y1 {
                        let dy = (y - cy) as f32 * 1.4;
                        for z in z0..=z1 {
                            for x in x0..=x1 {
                                let d2 = ((x - cx) * (x - cx) + (z - cz) * (z - cz)) as f32 + dy * dy;
                                if d2 > rad * rad {
                                    continue;
                                }
                                let (lx, lz) = ((x - pos.x * 16) as usize, (z - pos.y * 16) as usize);
                                if y < dcols[lx + lz * 16].uw_ceil + 14 {
                                    continue;
                                }
                                let i = buf.i(lx, ly_of(y), lz);
                                if buf.d[i] == p.stone || buf.d[i] == p.deepslate {
                                    buf.d[i] = id;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Underground biomes take over the rock around them.
    fn deep_biomes(&self, buf: &mut Buf, dcols: &[DeepCol], pos: ChunkPos) {
        let p = &self.deep_pal;
        let rock = &self.deep.rock;
        let ys = buf.ys();
        for lz in 0..16usize {
            for lx in 0..16usize {
                let dc = &dcols[lx + lz * 16];
                let [m, k, s, _] = dc.region;
                // Most columns have nothing to change.
                if dc.surface == SurfaceKind::Other && m <= 0.42 && k.abs() <= 0.5 {
                    let _ = s;
                    continue;
                }
                let (x, z) = ((pos.x * 16 + lx as i32) as f32, (pos.y * 16 + lz as i32) as f32);
                let wob = rock.noise2(x / 40.0, z / 40.0) * 6.0;
                let band = |y: i32, k: u32| ((((y as f32 + wob) / 5.0).floor() as i32 as u32).wrapping_mul(2_654_435_761) >> 13).wrapping_add(k) % 8;
                let top = (-10).min(ys.end);
                let bottom = (dc.uw_ceil + 14).max(MIN_Y + 5).max(ys.start);
                for y in bottom..top {
                    let i = buf.i(lx, ly_of(y), lz);
                    let b = buf.d[i];
                    if b == AIR || b == p.water || b == p.lava {
                        continue;
                    }
                    let new = match Self::ubiome_of(dc, y) {
                        UBiome::Plain | UBiome::Underworld | UBiome::Spider | UBiome::Gems => continue,
                        UBiome::Jungle => {
                            if band(y, 5) == 0 && (y & 3) != 0 {
                                p.honeycomb
                            } else {
                                p.mud
                            }
                        }
                        UBiome::Ice => match band(y, 1) {
                            0 | 1 => p.ice,
                            2 | 3 => p.snow,
                            _ => p.packed_ice,
                        },
                        UBiome::Desert => {
                            if band(y, 2) < 3 {
                                p.smooth_sandstone
                            } else {
                                p.sandstone
                            }
                        }
                        UBiome::Mushroom => p.mud,
                        UBiome::Marble => p.calcite,
                        UBiome::Granite => p.granite,
                    };
                    if new != AIR && (b == p.stone || b == p.deepslate || p.blobs.contains(&b)) {
                        buf.d[i] = new;
                    }
                }
            }
        }
    }

    fn deep_ores(&self, buf: &mut Buf, pos: ChunkPos, dcols: &[DeepCol]) {
        let p = &self.deep_pal;
        let mut rng = Rng::for_pos(self.seed, pos.x, pos.y, 0x0DE5);
        let ys = buf.ys();
        let host = |b: BlockId| b == p.stone || b == p.mud || b == p.granite || b == p.calcite || b == p.packed_ice || b == p.sandstone || b == p.andesite || b == p.diorite;
        let walk = |buf: &mut Buf, rng: &mut Rng, ore: BlockId, mut x: i32, mut y: i32, mut z: i32, n: u32, hosts: &dyn Fn(BlockId) -> bool| {
            for _ in 0..n {
                if (0..16).contains(&x) && (0..16).contains(&z) && y >= ys.start.max(MIN_Y + 5) && y < ys.end.min(0) {
                    let i = buf.i(x as usize, ly_of(y), z as usize);
                    if hosts(buf.d[i]) {
                        buf.d[i] = ore;
                    }
                }
                match rng.below(6) {
                    0 => x += 1,
                    1 => x -= 1,
                    2 => y += 1,
                    3 => y -= 1,
                    4 => z += 1,
                    _ => z -= 1,
                }
            }
        };
        // Every ore of the table (ores.rs); veins from the neighbour chunks too.
        let biome = |x: i32, z: i32| self.base_biome_at(x, z);
        self.ores.each(self.seed, pos.x, pos.y, ys.start.max(MIN_Y + 5), ys.end.min(0), &biome, |s, x, y, z| {
            let i = buf.i(x as usize, ly_of(y), z as usize);
            let b = buf.d[i];
            if b == p.deepslate || b == p.tuff {
                buf.d[i] = s.deep;
            } else if host(b) {
                buf.d[i] = s.ore;
            }
        });
        // Magmarite: in the Underworld's floor and ceiling.
        let hot = |b: BlockId| b == p.ash || b == p.basalt || b == p.blackstone;
        for _ in 0..12 {
            let (x, z) = (rng.range(1, 14), rng.range(1, 14));
            let dc = &dcols[(x + z * 16) as usize];
            let y = if rng.chance(0.5) { rng.range(dc.uw_floor - 20, dc.uw_floor) } else { rng.range(dc.uw_ceil, dc.uw_ceil + 12) };
            let n = rng.range(6, 12) as u32;
            if p.magmarite != AIR {
                walk(buf, &mut rng, p.magmarite, x, y, z, n, &hot);
            }
        }
    }

    /// Cave floors and ceilings: moss and vines in the jungle, glowing mushrooms, icicles,
    /// cobwebs, gem crystals, stalagmites; ash grass, ember blooms and ember trees in the
    /// Underworld; life crystals here and there.
    fn deep_decorate(&self, buf: &mut Buf, dcols: &[DeepCol], pos: ChunkPos, open_layer: &[bool]) {
        let p = &self.deep_pal;
        let seed = self.seed;
        let solid = |b: BlockId| b != AIR && b != p.water && b != p.lava;
        let mut trees: Vec<(i32, i32, i32)> = Vec::new();
        let mut mushrooms: Vec<(i32, i32, i32)> = Vec::new();
        let mut gem_trees: Vec<(i32, i32, i32)> = Vec::new();
        // Patches of one glowing moss, some tens of blocks across, in the deep caverns.
        let moss_at = |wx: i32, y: i32, wz: i32| -> Option<BlockId> {
            if y > -60 || p.mosses.is_empty() || cell_rand(seed, wx >> 5, y >> 4, wz >> 5, 77) > 0.18 {
                return None;
            }
            Some(p.mosses[(cell_rand(seed, wx >> 5, y >> 4, wz >> 5, 78) * p.mosses.len() as f32) as usize % p.mosses.len()])
        };
        let (blo, bhi) = (buf.lo, buf.hi);
        for z in 0..16usize {
            for x in 0..16usize {
                let dc = &dcols[x + z * 16];
                let (wx, wz) = (pos.x * 16 + x as i32, pos.y * 16 + z as i32);
                for ly in (blo + 1).max(6)..(bhi - 1).min(DEPTH - 2) {
                    if !open_layer[(ly - blo) / 16] {
                        continue;
                    }
                    let i = buf.i(x, ly, z);
                    if buf.d[i] != AIR {
                        continue;
                    }
                    let y = ly as i32 + MIN_Y;
                    let below = buf.d[i - 256];
                    let above = buf.d[i + 256];
                    if !solid(below) && !solid(above) {
                        continue;
                    }
                    let r = |salt: u64| cell_rand(seed, wx, y, wz, salt);
                    let ub = Self::ubiome_of(dc, y);
                    if solid(below) {
                        // A floor.
                        match ub {
                            UBiome::Underworld => {
                                if below == p.ash && p.ash_grass != AIR && r(1) < 0.55 {
                                    buf.d[i - 256] = p.ash_grass;
                                    if r(2) < 0.05 {
                                        buf.d[i] = p.ember_bloom;
                                    } else if r(3) < 0.006 && (2..14).contains(&x) && (2..14).contains(&z) {
                                        trees.push((wx, y, wz));
                                    }
                                }
                            }
                            UBiome::Jungle => {
                                if below == p.mud {
                                    buf.d[i - 256] = p.moss;
                                    let k = r(1);
                                    if k < 0.3 {
                                        buf.d[i] = p.short_grass;
                                    } else if k < 0.34 {
                                        buf.d[i] = p.jungle_leaves;
                                    } else if k < 0.342 {
                                        buf.d[i] = p.bee_nest;
                                    }
                                }
                            }
                            UBiome::Mushroom => {
                                if below == p.mud && p.glow_moss != AIR {
                                    buf.d[i - 256] = p.glow_moss;
                                    let k = r(1);
                                    if k < 0.18 {
                                        buf.d[i] = p.glow_mushroom;
                                    } else if k < 0.19 && (2..14).contains(&x) && (2..14).contains(&z) {
                                        mushrooms.push((wx, y, wz));
                                    }
                                }
                            }
                            UBiome::Ice => {
                                if r(1) < 0.5 {
                                    buf.d[i] = p.snow_layer;
                                }
                            }
                            UBiome::Desert => {
                                let k = r(1);
                                if k < 0.3 {
                                    buf.d[i - 256] = p.sand;
                                } else if k < 0.32 {
                                    buf.d[i] = p.dead_bush;
                                }
                            }
                            UBiome::Gems => {
                                if r(1) < 0.06 {
                                    buf.d[i] = p.amethyst_cluster;
                                }
                            }
                            UBiome::Spider => {
                                if r(1) < 0.1 {
                                    buf.d[i] = p.cobweb;
                                }
                            }
                            _ => {
                                // Still pools in hollows of the floor: water, lava from y -100 down.
                                if (below == p.stone || below == p.deepslate) && (1..15).contains(&x) && (1..15).contains(&z) && cell_rand(seed, wx >> 3, y >> 3, wz >> 3, 0x9001) < 0.25 {
                                    let walled = [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)].iter().all(|&(a, b)| buf.d[buf.i(a, ly - 1, b)] != AIR);
                                    if walled && ly >= blo + 2 && solid(buf.d[i - 512]) {
                                        buf.d[i - 256] = if y <= -100 { p.lava } else { p.water };
                                        continue;
                                    }
                                }
                                if (below == p.stone || below == p.deepslate) && r(13) < 0.75 {
                                    if let Some(m) = moss_at(wx, y, wz) {
                                        buf.d[i - 256] = m;
                                    }
                                }
                                if y < -100 && p.gem_log != AIR && r(14) < 0.0012 && (2..14).contains(&x) && (2..14).contains(&z) {
                                    gem_trees.push((wx, y, wz));
                                } else if below == p.stone && y < -40 && r(4) < 0.0004 {
                                    buf.d[i] = p.life_crystal;
                                } else if below == p.stone && r(1) < 0.012 {
                                    buf.d[i] = p.spike_up;
                                }
                            }
                        }
                    } else {
                        // A ceiling.
                        match ub {
                            UBiome::Jungle => {
                                if r(5) < 0.2 {
                                    let len = 1 + (r(6) * 6.0) as usize;
                                    for k in 0..len {
                                        if ly < blo + k + 1 {
                                            break;
                                        }
                                        let j = i - 256 * k;
                                        if buf.d[j] != AIR {
                                            break;
                                        }
                                        buf.d[j] = p.vines;
                                    }
                                }
                            }
                            UBiome::Ice => {
                                if r(5) < 0.08 {
                                    buf.d[i] = p.icicle;
                                }
                            }
                            UBiome::Spider => {
                                if r(5) < 0.25 {
                                    buf.d[i] = p.cobweb;
                                }
                            }
                            UBiome::Underworld => {
                                if r(5) < 0.01 {
                                    let len = 2 + (r(6) * 5.0) as usize;
                                    for k in 0..len {
                                        if ly < blo + k + 1 {
                                            break;
                                        }
                                        let j = i - 256 * k;
                                        if buf.d[j] != AIR {
                                            break;
                                        }
                                        buf.d[j] = p.basalt;
                                    }
                                }
                            }
                            UBiome::Plain => {
                                if (above == p.stone || above == p.deepslate) && r(13) < 0.75 {
                                    if let Some(m) = moss_at(wx, y, wz) {
                                        buf.d[i + 256] = m;
                                    }
                                }
                                if r(5) < 0.015 {
                                    buf.d[i] = p.spike_down;
                                }
                            }
                            _ => {}
                        }
                    }
                    // Gem caverns: crystals grow out of the walls.
                    if ub == UBiome::Gems && !p.gems.is_empty() && r(7) < 0.05 {
                        let (sx, sz) = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)][(r(8) * 4.0) as usize % 4];
                        let (nx, nz) = (x as i32 + sx, z as i32 + sz);
                        if (0..16).contains(&nx) && (0..16).contains(&nz) {
                            let j = buf.i(nx as usize, ly, nz as usize);
                            if buf.d[j] == p.stone || buf.d[j] == p.deepslate {
                                buf.d[j] = p.gems[(r(9) * p.gems.len() as f32) as usize % p.gems.len()];
                            }
                        }
                    }
                }
            }
        }
        let mut d = Draw { buf, bx: pos.x * 16, bz: pos.y * 16 };
        for (wx, y, wz) in trees {
            let h = 4 + (cell_rand(seed, wx, y, wz, 11) * 5.0) as i32;
            for k in 0..h {
                d.set(wx, y + k, wz, p.cinder_log);
            }
            for dy in -1i32..=1 {
                for dz in -2i32..=2 {
                    for dx in -2i32..=2 {
                        if dx.abs() + dz.abs() + dy.abs() < 4 && d.get(wx + dx, y + h + dy, wz + dz) == Some(AIR) {
                            d.set(wx + dx, y + h + dy, wz + dz, p.ember_leaves);
                        }
                    }
                }
            }
        }
        // Gem trees: a short stone trunk under a crown of gem leaves, deeper ones of rarer gems.
        for (wx, y, wz) in gem_trees {
            let h = 3 + (cell_rand(seed, wx, y, wz, 15) * 3.0) as i32;
            let depth = ((-y - 100) as f32 / 450.0).clamp(0.0, 0.999);
            let n = p.gem_leaves.len();
            let k = ((depth * n as f32) as usize + (cell_rand(seed, wx, y, wz, 16) * 2.0) as usize).min(n - 1);
            let leaves = p.gem_leaves[k];
            for k in 0..h {
                if d.get(wx, y + k, wz) != Some(AIR) {
                    break;
                }
                d.set(wx, y + k, wz, p.gem_log);
            }
            for dy in -1i32..=1 {
                for dz in -2i32..=2 {
                    for dx in -2i32..=2 {
                        if dx * dx + dz * dz + dy * dy * 2 <= 5 && d.get(wx + dx, y + h + dy, wz + dz) == Some(AIR) {
                            d.set(wx + dx, y + h + dy, wz + dz, leaves);
                        }
                    }
                }
            }
        }
        for (wx, y, wz) in mushrooms {
            let h = 3 + (cell_rand(seed, wx, y, wz, 12) * 4.0) as i32;
            for k in 0..h {
                d.set(wx, y + k, wz, p.mushroom_stem);
            }
            for dz in -2i32..=2 {
                for dx in -2i32..=2 {
                    if dx.abs() + dz.abs() < 4 {
                        d.set(wx + dx, y + h, wz + dz, p.glow_cap);
                    }
                }
            }
        }
    }

    /// Buildings planned in one cell (the same answer from every chunk).
    fn deep_plans_in(&self, cx: i32, cz: i32) -> Vec<DeepPlan> {
        let mut r = Rng::for_pos(self.seed, cx, cz, 0xDEE95);
        let mut out = Vec::new();
        let (x0, z0) = (cx * CELL, cz * CELL);
        let spread = self.spread.max(50) as f32 / 100.0;
        let chance = |r: &mut Rng, p: f32| r.chance(p / spread);
        // A cabin on a long tunnel: find where one crosses this cell.
        if chance(&mut r, 0.55) {
            let k = r.below(TUNNEL_Y.len() as u32) as usize;
            let mut best: Option<(f32, i32, i32)> = None;
            for sz in 0..6 {
                for sx in 0..6 {
                    let (wx, wz) = (x0 + 4 + sx * 8, z0 + 4 + sz * 8);
                    let l = self.deep.tunnels[k].noise2(wx as f32 / 260.0, wz as f32 / 260.0).abs();
                    if best.is_none_or(|b| l < b.0) {
                        best = Some((l, wx, wz));
                    }
                }
            }
            if let Some((l, wx, wz)) = best.filter(|b| b.0 < 0.012) {
                let _ = l;
                let cy = TUNNEL_Y[k] as f32 + 16.0 * self.deep.tunnel_y.noise2(wx as f32 / 150.0 + k as f32 * 40.0, wz as f32 / 150.0);
                let size = IVec3::new(r.range(7, 10), 5, r.range(6, 8));
                out.push(DeepPlan { kind: DeepKind::Cabin, origin: IVec3::new(wx - size.x / 2, cy as i32 - 2, wz - size.z / 2), size, seed: r.next_u64() });
            }
        }
        if chance(&mut r, 0.22) {
            let along_x = r.chance(0.5);
            let len = r.range(50, 110);
            let y = r.range(-420, -60);
            let size = if along_x { IVec3::new(len, 4, 3) } else { IVec3::new(3, 4, len) };
            out.push(DeepPlan { kind: DeepKind::Mineshaft, origin: IVec3::new(x0 + r.range(0, CELL - 4), y, z0 + r.range(0, CELL - 4)), size, seed: r.next_u64() });
        }
        if chance(&mut r, 0.06) {
            let y = r.range(-380, -80);
            out.push(DeepPlan { kind: DeepKind::Shrine, origin: IVec3::new(x0 + r.range(4, 30), y, z0 + r.range(4, 30)), size: IVec3::new(11, 8, 11), seed: r.next_u64() });
        }
        if chance(&mut r, 0.05) {
            let y = r.range(-580, -380);
            out.push(DeepPlan { kind: DeepKind::Vault, origin: IVec3::new(x0 + r.range(4, 32), y, z0 + r.range(4, 32)), size: IVec3::new(9, 6, 9), seed: r.next_u64() });
        }
        // A giant hive, only under the jungle.
        // (its own random stream, so the other buildings stay where they were before hives).
        let mut rh = Rng::for_pos(self.seed, cx, cz, 0x41BE);
        if chance(&mut rh, 0.3) {
            let (wx, wz) = (x0 + rh.range(14, CELL - 14), z0 + rh.range(14, CELL - 14));
            let y = rh.range(-300, -50);
            if surface_kind(self.column(wx, wz).biome) == SurfaceKind::Jungle {
                let size = IVec3::new(rh.range(20, 26), rh.range(14, 18), rh.range(20, 26));
                out.push(DeepPlan { kind: DeepKind::Hive, origin: IVec3::new(wx, y, wz) - size / 2, size, seed: rh.next_u64() });
            }
        }
        // A mine network (its own stream too). It stays inside 104 blocks of its cell's corner,
        // which `deep_plans_near` looks back over.
        let mut rm = Rng::for_pos(self.seed, cx, cz, 0x313E);
        if chance(&mut rm, 0.09) {
            let y = rm.range(-84, -36);
            out.push(DeepPlan { kind: DeepKind::MineNetwork, origin: IVec3::new(x0 + rm.range(0, 8), y, z0 + rm.range(0, 8)), size: MINE_BOX, seed: rm.next_u64() });
        }
        // The Underworld's ruins stand on its floor (never in the lava sea).
        if chance(&mut r, 0.5) {
            let tower = r.chance(0.35);
            let size = if tower { IVec3::new(9, r.range(16, 26), 9) } else { IVec3::new(r.range(11, 15), r.range(9, 13), r.range(9, 12)) };
            let (wx, wz) = (x0 + r.range(2, CELL - size.x - 2), z0 + r.range(2, CELL - size.z - 2));
            let mut lo = i32::MAX;
            for (dx, dz) in [(0, 0), (size.x - 1, 0), (0, size.z - 1), (size.x - 1, size.z - 1), (size.x / 2, size.z / 2)] {
                lo = lo.min(self.underworld_floor(wx + dx, wz + dz));
            }
            if lo > LAVA_SEA + 1 {
                let kind = if tower { DeepKind::RuinedTower } else { DeepKind::RuinedHouse };
                out.push(DeepPlan { kind, origin: IVec3::new(wx, lo + 1, wz), size, seed: r.next_u64() });
            }
        }
        out
    }

    /// Buildings that may reach into a chunk.
    pub fn deep_plans_near(&self, pos: ChunkPos) -> Vec<DeepPlan> {
        if self.dim != super::Dim::Overworld {
            return Vec::new();
        }
        let (bx, bz) = (pos.x * 16, pos.y * 16);
        let (c0x, c0z) = ((bx - 112).div_euclid(CELL), (bz - 112).div_euclid(CELL));
        let (c1x, c1z) = ((bx + 16).div_euclid(CELL), (bz + 16).div_euclid(CELL));
        let mut out = Vec::new();
        for cz in c0z..=c1z {
            for cx in c0x..=c1x {
                for plan in self.deep_plans_in(cx, cz) {
                    let (lo, hi) = (plan.origin, plan.origin + plan.size);
                    if hi.x + 2 >= bx && lo.x - 2 < bx + 16 && hi.z + 2 >= bz && lo.z - 2 < bz + 16 {
                        out.push(plan);
                    }
                }
            }
        }
        out
    }

    /// Where a building's chests are.
    fn deep_chests(&self, plan: &DeepPlan) -> Vec<IVec3> {
        let o = plan.origin;
        let s = plan.size;
        let mut r = Rng::new(plan.seed ^ 0xC4E5);
        match plan.kind {
            DeepKind::Cabin => vec![o + IVec3::new(1, 1, s.z - 2)],
            DeepKind::Mineshaft => {
                let len = s.x.max(s.z);
                let at = r.range(6, len - 6);
                vec![if s.x > s.z { o + IVec3::new(at, 1, 0) } else { o + IVec3::new(0, 1, at) }]
            }
            DeepKind::Shrine => vec![o + IVec3::new(5, 2, 7)],
            DeepKind::Vault => vec![o + IVec3::new(2, 1, 4), o + IVec3::new(6, 1, 4)],
            DeepKind::RuinedHouse => vec![o + IVec3::new(2, 1, 2), o + IVec3::new(s.x - 3, 5, s.z - 3)],
            DeepKind::RuinedTower => vec![o + IVec3::new(4, 1, 4), o + IVec3::new(4, s.y - 5, 4)],
            DeepKind::Hive => Vec::new(),
            DeepKind::MineNetwork => mine_layout(plan).chests,
        }
    }

    /// The loot of a deep chest at `p`, if a building put one there.
    pub fn deep_chest(&self, p: IVec3) -> Option<&[LootDef]> {
        if p.y >= 0 {
            return None;
        }
        let pos = ChunkPos::new(p.x.div_euclid(16), p.z.div_euclid(16));
        for plan in self.deep_plans_near(pos) {
            if self.deep_chests(&plan).contains(&p) {
                return Some(&self.deep_loot[plan.kind as usize]);
            }
        }
        None
    }

    /// The nearest deep building of a kind (for `--find` and the explorer's map).
    pub fn find_deep(&self, kind: &str, from: IVec3, radius_cells: i32) -> Option<IVec3> {
        let (fx, fz) = (from.x.div_euclid(CELL), from.z.div_euclid(CELL));
        let mut best: Option<(i64, IVec3)> = None;
        for cz in fz - radius_cells..=fz + radius_cells {
            for cx in fx - radius_cells..=fx + radius_cells {
                for plan in self.deep_plans_in(cx, cz) {
                    if plan.kind.name() != kind {
                        continue;
                    }
                    let c = plan.origin + plan.size / 2;
                    let d = (c - from).as_i64vec3().length_squared();
                    if best.is_none_or(|b| d < b.0) {
                        best = Some((d, c));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    fn build_deep(&self, d: &mut Draw, plan: &DeepPlan) {
        let p = &self.deep_pal;
        let o = plan.origin;
        let s = plan.size;
        let mut r = Rng::new(plan.seed);
        let chests = self.deep_chests(plan);
        let v = |x: i32, y: i32, z: i32| o + IVec3::new(x, y, z);
        match plan.kind {
            DeepKind::Cabin => {
                let w = r.below(3) as usize;
                let (plank, log) = (p.planks[w], p.logs[w]);
                // Shell, then hollow it out.
                d.fill(v(0, 0, 0), v(s.x - 1, s.y - 1, s.z - 1), plank);
                d.fill(v(1, 1, 1), v(s.x - 2, s.y - 2, s.z - 2), AIR);
                for (x, z) in [(0, 0), (s.x - 1, 0), (0, s.z - 1), (s.x - 1, s.z - 1)] {
                    d.fill(v(x, 0, z), v(x, s.y - 1, z), log);
                }
                // Doorways at both ends, onto the tunnel.
                d.fill(v(0, 1, s.z / 2), v(0, 2, s.z / 2), AIR);
                d.fill(v(s.x - 1, 1, s.z / 2), v(s.x - 1, 2, s.z / 2), AIR);
                // Decay: a few planks gone, cobwebs in the corners.
                for _ in 0..(s.x * s.z / 6) {
                    let (x, y, z) = (r.range(0, s.x - 1), r.range(0, s.y - 1), r.range(0, s.z - 1));
                    if x == 0 || z == 0 || x == s.x - 1 || z == s.z - 1 || y == s.y - 1 {
                        d.set(o.x + x, o.y + y, o.z + z, AIR);
                    }
                }
                d.set(o.x + s.x - 2, o.y + s.y - 2, o.z + 1, p.cobweb);
                d.set(o.x + 1, o.y + s.y - 2, o.z + 1, p.cobweb);
                d.set(o.x + s.x / 2, o.y + s.y - 2, o.z + s.z / 2, p.lantern);
                d.set(o.x + s.x - 2, o.y + 1, o.z + s.z - 2, p.bookshelf);
            }
            DeepKind::Mineshaft => {
                let along_x = s.x > s.z;
                let len = s.x.max(s.z);
                let w = r.below(2) as usize;
                for k in 0..len {
                    let at = |a: i32, y: i32| if along_x { v(k, y, a) } else { v(a, y, k) };
                    for a in 0..3 {
                        for y in 1..4 {
                            let q = at(a, y);
                            d.set(q.x, q.y, q.z, AIR);
                        }
                        // A floor of planks where it crosses a cave.
                        let f = at(a, 0);
                        if d.get(f.x, f.y, f.z).is_some_and(|b| b == AIR || b == p.water || b == p.lava) {
                            d.set(f.x, f.y, f.z, p.planks[w]);
                        }
                    }
                    let mid = at(1, 1);
                    if r.chance(0.8) {
                        d.set(mid.x, mid.y, mid.z, p.rail);
                    }
                    // Timber frames.
                    if k % 5 == 0 {
                        for a in [0, 2] {
                            for y in 1..3 {
                                let q = at(a, y);
                                d.set(q.x, q.y, q.z, p.fence);
                            }
                        }
                        for a in 0..3 {
                            let q = at(a, 3);
                            d.set(q.x, q.y, q.z, p.planks[w]);
                        }
                        if k % 15 == 0 {
                            let q = at(1, 2);
                            d.set(q.x, q.y, q.z, p.chain);
                            let q = at(0, 2);
                            d.set(q.x, q.y, q.z, p.torch);
                        }
                    } else if r.chance(0.05) {
                        let q = at(r.range(0, 2), 3);
                        d.set(q.x, q.y, q.z, p.cobweb);
                    }
                }
            }
            DeepKind::Shrine => {
                d.fill(v(0, 0, 0), v(s.x - 1, s.y - 1, s.z - 1), p.mossy_cobble);
                d.fill(v(1, 1, 1), v(s.x - 2, s.y - 2, s.z - 2), AIR);
                for _ in 0..30 {
                    let q = v(r.range(0, s.x - 1), r.range(0, s.y - 1), r.range(0, s.z - 1));
                    if d.get(q.x, q.y, q.z).is_some_and(|b| b != AIR) {
                        d.set(q.x, q.y, q.z, if r.chance(0.5) { p.mossy_bricks } else { p.moss });
                    }
                }
                // Pedestal under the chest, lit from both sides.
                d.fill(v(4, 1, 6), v(6, 1, 8), p.bricks);
                d.set(o.x + 2, o.y + 1, o.z + 2, p.torch);
                d.set(o.x + s.x - 3, o.y + 1, o.z + 2, p.torch);
                // A way in.
                d.fill(v(s.x / 2, 1, 0), v(s.x / 2, 2, 0), AIR);
                for k in 1..10 {
                    d.fill(v(s.x / 2, 1, -k), v(s.x / 2, 2, -k), AIR);
                }
            }
            DeepKind::Vault => {
                d.fill(v(0, 0, 0), v(s.x - 1, s.y - 1, s.z - 1), p.bricks);
                d.fill(v(1, 1, 1), v(s.x - 2, s.y - 2, s.z - 2), AIR);
                for _ in 0..20 {
                    let q = v(r.range(0, s.x - 1), r.range(0, s.y - 1), r.range(0, s.z - 1));
                    if d.get(q.x, q.y, q.z).is_some_and(|b| b == p.bricks) {
                        d.set(q.x, q.y, q.z, if r.chance(0.5) { p.cracked_bricks } else { p.mossy_bricks });
                    }
                }
                d.fill(v(1, 0, 1), v(s.x - 2, 0, s.z - 2), p.gold_block);
                d.set(o.x + s.x / 2, o.y + s.y - 2, o.z + s.z / 2, p.lantern);
            }
            DeepKind::MineNetwork => self.build_mine(d, plan),
            DeepKind::Hive => {
                // An egg of hive wall, hollow, its floor a pool of honeycomb, the larva in the middle.
                let c = (o + s / 2).as_vec3();
                let half = s.as_vec3() / 2.0;
                for y in 0..s.y {
                    for z in 0..s.z {
                        for x in 0..s.x {
                            let q = o + IVec3::new(x, y, z);
                            let e = (q.as_vec3() + 0.5 - c) / half;
                            let r2 = e.length_squared();
                            if r2 > 1.0 {
                                continue;
                            }
                            let shell = r2 > 0.62 + 0.08 * cell_rand(plan.seed, q.x, q.y, q.z, 3);
                            let id = if shell {
                                p.hive
                            } else if e.y < -0.55 {
                                p.honey
                            } else {
                                AIR
                            };
                            d.set(q.x, q.y, q.z, id);
                        }
                    }
                }
                // Hive pillars from floor to ceiling, and the larva on a mound.
                for k in 0..4 {
                    let a = k as f32 * 1.57 + r.f32();
                    let (px, pz) = (c.x + a.cos() * half.x * 0.45, c.z + a.sin() * half.z * 0.45);
                    for y in 0..s.y {
                        d.set(px as i32, o.y + y, pz as i32, p.hive);
                    }
                }
                let base = IVec3::new(c.x as i32, o.y + (s.y as f32 * 0.22) as i32, c.z as i32);
                d.fill(base - IVec3::new(1, 1, 1), base + IVec3::new(1, 0, 1), p.hive);
                d.set(base.x, base.y + 1, base.z, p.or(p.larva, p.honeycomb));
            }
            DeepKind::RuinedHouse | DeepKind::RuinedTower => {
                let wall = |r: &mut Rng| if r.chance(0.7) { p.or(p.obsidian_bricks, p.obsidian) } else { p.or(p.infernal_bricks, p.obsidian) };
                // Foundation down to the ash.
                for z in 0..s.z {
                    for x in 0..s.x {
                        for y in (o.y - 12..o.y).rev() {
                            if d.get(o.x + x, y, o.z + z).is_some_and(|b| b == AIR || b == p.lava) {
                                d.set(o.x + x, y, o.z + z, p.or(p.obsidian_bricks, p.obsidian));
                            } else {
                                break;
                            }
                        }
                    }
                }
                for y in 0..s.y {
                    for z in 0..s.z {
                        for x in 0..s.x {
                            let edge = x == 0 || z == 0 || x == s.x - 1 || z == s.z - 1;
                            let floor = y % 5 == 0;
                            let id = if edge || floor { wall(&mut r) } else { AIR };
                            d.set(o.x + x, o.y + y, o.z + z, id);
                        }
                    }
                    // Windows with bars.
                    if y % 5 == 2 {
                        for (x, z) in [(s.x / 2, 0), (s.x / 2, s.z - 1), (0, s.z / 2), (s.x - 1, s.z / 2)] {
                            d.set(o.x + x, o.y + y, o.z + z, p.iron_bars);
                        }
                    }
                }
                // Holes between floors (a ladder of ruin), a door, lanterns on chains.
                for f in (5..s.y).step_by(5) {
                    d.fill(v(1, f, 1), v(2, f, 2), AIR);
                }
                d.fill(v(s.x / 2, 1, 0), v(s.x / 2, 2, 0), AIR);
                for f in (0..s.y - 2).step_by(5) {
                    d.set(o.x + s.x / 2, o.y + f + 4, o.z + s.z / 2, p.chain);
                    d.set(o.x + s.x / 2, o.y + f + 3, o.z + s.z / 2, p.soul_lantern);
                }
                // Ruin: the top is broken off.
                let top = s.y - 1;
                for _ in 0..(s.x * s.z) {
                    let (x, z) = (r.range(0, s.x - 1), r.range(0, s.z - 1));
                    let drop = r.range(0, 4);
                    for y in (top - drop)..=top {
                        d.set(o.x + x, o.y + y, o.z + z, AIR);
                    }
                }
            }
        }
        for c in chests {
            d.set(c.x, c.y, c.z, p.chest);
            // Something to stand the chest on.
            if d.get(c.x, c.y - 1, c.z).is_some_and(|b| b == AIR) {
                d.set(c.x, c.y - 1, c.z, p.cobble);
            }
        }
    }
}

/// The box a mine network grows inside (corner at its plan's origin).
const MINE_BOX: IVec3 = IVec3::new(96, 26, 96);
/// Floor of the mine's central room, above the box's bottom.
const MINE_ROOM_Y: i32 = 12;
/// Most pieces in one mine.
const MINE_PIECES: usize = 72;

/// One piece of a mine network. `a` and `c` are floor blocks: the air starts one above.
#[derive(Clone, Copy, Debug)]
enum MinePiece {
    /// The dirt room everything starts from.
    Room { lo: IVec3, hi: IVec3 },
    /// A corridor 3 wide and 3 high, `len` blocks from `a` along `dir`, timber frames every 4.
    Hall { a: IVec3, dir: u8, len: i32, rails: bool, seed: u64 },
    /// Steps going one block down (or up) for each block forward.
    Stairs { a: IVec3, dir: u8, len: i32, down: bool },
    /// A 5x5 crossing around `c`, sometimes twice as tall.
    Cross { c: IVec3, tall: bool },
}

/// A planned mine network: its pieces, chests and spawners.
struct MineLayout {
    pieces: Vec<(MinePiece, IVec3, IVec3)>,
    chests: Vec<IVec3>,
    spawners: Vec<IVec3>,
}

fn mine_dir(dir: u8) -> IVec3 {
    [IVec3::X, IVec3::Z, IVec3::NEG_X, IVec3::NEG_Z][(dir & 3) as usize]
}

fn mine_side(dir: u8) -> IVec3 {
    mine_dir(dir + 1)
}

/// The blocks a piece takes (inclusive), with its frames and floor.
fn mine_box(p: &MinePiece) -> (IVec3, IVec3) {
    let span = |a: IVec3, b: IVec3, side: IVec3| {
        let (q0, q1) = (a - side, b + side);
        (q0.min(q1).min(a + side).min(b - side), q0.max(q1).max(a + side).max(b - side))
    };
    match *p {
        MinePiece::Room { lo, hi } => (lo, hi),
        MinePiece::Hall { a, dir, len, .. } => {
            let (lo, hi) = span(a, a + mine_dir(dir) * (len - 1), mine_side(dir));
            (lo, hi + IVec3::Y * 3)
        }
        MinePiece::Stairs { a, dir, len, down } => {
            let b = a + mine_dir(dir) * (len - 1) + IVec3::Y * if down { -(len - 1) } else { len - 1 };
            let (lo, hi) = span(a, b, mine_side(dir));
            (lo, hi + IVec3::Y * 4)
        }
        MinePiece::Cross { c, tall } => (c - IVec3::new(2, 0, 2), c + IVec3::new(2, if tall { 6 } else { 3 }, 2)),
    }
}

fn boxes_touch(a: (IVec3, IVec3), b: (IVec3, IVec3)) -> bool {
    a.0.cmple(b.1).all() && b.0.cmple(a.1).all()
}

/// Grows a mine from its room, the same way from every chunk.
fn mine_layout(plan: &DeepPlan) -> MineLayout {
    let mut r = Rng::new(plan.seed ^ 0x313E);
    let (blo, bhi) = (plan.origin, plan.origin + plan.size - 1);
    let mut out = MineLayout { pieces: Vec::new(), chests: Vec::new(), spawners: Vec::new() };
    let fits = |out: &MineLayout, b: (IVec3, IVec3)| b.0.cmpge(blo).all() && b.1.cmple(bhi).all() && !out.pieces.iter().any(|q| boxes_touch((q.1, q.2), b));
    // The room in the middle.
    let y = plan.origin.y + MINE_ROOM_Y;
    let (w, dp) = (r.range(9, 13), r.range(9, 13));
    let c = plan.origin + plan.size / 2;
    let rlo = IVec3::new(c.x - w / 2, y, c.z - dp / 2);
    let rhi = IVec3::new(rlo.x + w - 1, y + r.range(4, 6), rlo.z + dp - 1);
    let room = MinePiece::Room { lo: rlo, hi: rhi };
    out.pieces.push((room, rlo, rhi));
    let mut queue = std::collections::VecDeque::new();
    for dir in 0..4u8 {
        for _ in 0..r.range(1, 2) {
            let start = match dir {
                0 => IVec3::new(rhi.x + 1, y, r.range(rlo.z + 1, rhi.z - 1)),
                1 => IVec3::new(r.range(rlo.x + 1, rhi.x - 1), y, rhi.z + 1),
                2 => IVec3::new(rlo.x - 1, y, r.range(rlo.z + 1, rhi.z - 1)),
                _ => IVec3::new(r.range(rlo.x + 1, rhi.x - 1), y, rlo.z - 1),
            };
            queue.push_back((start, dir, 0u32));
        }
    }
    let cross_at = |out: &mut MineLayout, r: &mut Rng, queue: &mut std::collections::VecDeque<(IVec3, u8, u32)>, e: IVec3, dir: u8, depth: u32| {
        let piece = MinePiece::Cross { c: e + mine_dir(dir) * 2, tall: r.chance(0.25) };
        let b = mine_box(&piece);
        if !fits(out, b) {
            return;
        }
        out.pieces.push((piece, b.0, b.1));
        let cc = e + mine_dir(dir) * 2;
        for turn in [0u8, 1, 3] {
            if r.chance(0.72) {
                let nd = (dir + turn) & 3;
                queue.push_back((cc + mine_dir(nd) * 3, nd, depth + 1));
            }
        }
    };
    while let Some((s, dir, depth)) = queue.pop_front() {
        if out.pieces.len() >= MINE_PIECES || depth > 9 {
            continue;
        }
        let roll = r.f32();
        if depth > 0 && roll < 0.14 {
            let down = r.chance(0.6);
            let piece = MinePiece::Stairs { a: s, dir, len: 5, down };
            let b = mine_box(&piece);
            if fits(&out, b) {
                out.pieces.push((piece, b.0, b.1));
                let e = s + mine_dir(dir) * 5 + IVec3::Y * if down { -5 } else { 5 };
                queue.push_back((e, dir, depth + 1));
            }
            continue;
        }
        if depth > 0 && roll < 0.3 {
            cross_at(&mut out, &mut r, &mut queue, s, dir, depth);
            continue;
        }
        // A corridor, shortened until it fits.
        let mut len = r.range(2, 7) * 4 + 1;
        let rails = r.chance(0.5);
        let seed = r.next_u64();
        let placed = loop {
            let piece = MinePiece::Hall { a: s, dir, len, rails, seed };
            let b = mine_box(&piece);
            if fits(&out, b) {
                out.pieces.push((piece, b.0, b.1));
                break true;
            }
            len -= 4;
            if len < 5 {
                break false;
            }
        };
        if !placed {
            continue;
        }
        // A chest against one wall, or a cave spider nest in the middle.
        let (d, side) = (mine_dir(dir), mine_side(dir));
        if r.chance(0.4) {
            let k = r.range(0, len / 4 - 1) * 4 + r.range(1, 3);
            out.chests.push(s + d * k + side * if r.chance(0.5) { 1 } else { -1 } + IVec3::Y);
        }
        if out.spawners.len() < 2 && len >= 9 && r.chance(0.12) {
            out.spawners.push(s + d * (len / 2 / 4 * 4 + 2) + IVec3::Y);
        }
        let e = s + d * len;
        let next = r.f32();
        if next < 0.5 {
            cross_at(&mut out, &mut r, &mut queue, e, dir, depth);
        } else if next < 0.8 {
            queue.push_back((e, dir, depth + 1));
        }
    }
    out
}

impl Generator {
    /// Draws the parts of a mine network that fall in this chunk.
    fn build_mine(&self, d: &mut Draw, plan: &DeepPlan) {
        let p = &self.deep_pal;
        let lay = mine_layout(plan);
        let wood = (plan.seed % 3) as usize;
        let (plank, log) = (p.planks[wood], p.logs[wood]);
        let (cx0, cz0) = (d.bx, d.bz);
        let open = |b: BlockId| b == AIR || b == p.water || b == p.lava;
        // A floor block that bridges caves, water and lava.
        let floor = |d: &mut Draw, q: IVec3, with: BlockId| {
            if d.get(q.x, q.y, q.z).is_some_and(open) {
                d.set(q.x, q.y, q.z, with);
            }
        };
        for (piece, lo, hi) in &lay.pieces {
            if hi.x < cx0 - 1 || lo.x > cx0 + 16 || hi.z < cz0 - 1 || lo.z > cz0 + 16 {
                continue;
            }
            match *piece {
                MinePiece::Room { lo, hi } => {
                    d.fill(lo + IVec3::Y, hi, AIR);
                    d.fill(lo, IVec3::new(hi.x, lo.y, hi.z), p.dirt);
                    // A lantern hanging from the middle of the ceiling and one in a corner.
                    let m = (lo + hi) / 2;
                    d.set(m.x, hi.y, m.z, p.chain);
                    d.set(m.x, hi.y - 1, m.z, p.lantern_hang);
                    d.set(lo.x, lo.y + 1, lo.z, p.torch);
                }
                MinePiece::Hall { a, dir, len, rails, seed } => {
                    let (dv, sv) = (mine_dir(dir), mine_side(dir));
                    let mut r = Rng::new(seed);
                    for k in 0..len {
                        let m = a + dv * k;
                        for s in -1..=1 {
                            let q = m + sv * s;
                            d.fill(q + IVec3::Y, q + IVec3::Y * 3, AIR);
                            floor(d, q, plank);
                        }
                        if k % 4 == 0 {
                            // A timber frame: two posts and a beam; logs under the posts on bridges.
                            for s in [-1, 1] {
                                let q = m + sv * s;
                                d.set(q.x, q.y + 1, q.z, p.fence);
                                d.set(q.x, q.y + 2, q.z, p.fence);
                                if d.get(q.x, q.y - 1, q.z).is_some_and(open) {
                                    for down in 1..8 {
                                        if !d.get(q.x, q.y - down, q.z).is_some_and(open) {
                                            break;
                                        }
                                        d.set(q.x, q.y - down, q.z, p.fence);
                                    }
                                }
                            }
                            for s in -1..=1 {
                                let q = m + sv * s + IVec3::Y * 3;
                                d.set(q.x, q.y, q.z, plank);
                            }
                            if k % 8 == 4 && r.chance(0.55) {
                                let q = m + sv * if r.chance(0.5) { 1 } else { -1 } + dv + IVec3::Y;
                                d.set(q.x, q.y, q.z, p.torch);
                            }
                        } else {
                            // Rotten beams fallen out and cobwebs in the ceiling corners.
                            if r.chance(0.07) {
                                let q = m + sv * r.range(-1, 1) + IVec3::Y * 3;
                                d.set(q.x, q.y, q.z, p.cobweb);
                            }
                        }
                        if rails && r.chance(0.85) {
                            d.set(m.x, m.y + 1, m.z, p.rail);
                        }
                    }
                    // A dead end gets a pile of gravel or a cave-in.
                    let e = a + dv * (len - 1);
                    if r.chance(0.2) {
                        d.set(e.x, e.y + 1, e.z, p.cobble);
                    }
                }
                MinePiece::Stairs { a, dir, len, down } => {
                    let (dv, sv) = (mine_dir(dir), mine_side(dir));
                    for k in 0..len {
                        let m = a + dv * k + IVec3::Y * if down { -k } else { k };
                        for s in -1..=1 {
                            let q = m + sv * s;
                            d.fill(q + IVec3::Y, q + IVec3::Y * 4, AIR);
                            floor(d, q, plank);
                        }
                    }
                }
                MinePiece::Cross { c, tall } => {
                    let top = if tall { 6 } else { 3 };
                    d.fill(c + IVec3::new(-2, 1, -2), c + IVec3::new(2, top, 2), AIR);
                    for z in -2..=2 {
                        for x in -2..=2 {
                            floor(d, c + IVec3::new(x, 0, z), plank);
                        }
                    }
                    // Log pillars in the corners hold a plank ceiling.
                    for (x, z) in [(-2, -2), (2, -2), (-2, 2), (2, 2)] {
                        d.fill(c + IVec3::new(x, 1, z), c + IVec3::new(x, top, z), log);
                    }
                    if tall {
                        d.fill(c + IVec3::new(-2, top, -2), c + IVec3::new(2, top, 2), plank);
                        d.fill(c + IVec3::new(-1, top, -1), c + IVec3::new(1, top, 1), AIR);
                        d.set(c.x, c.y + top, c.z, p.chain);
                        d.set(c.x, c.y + top - 1, c.z, p.lantern_hang);
                    }
                }
            }
        }
        // Cave spider nests: the spawner wrapped in webs (which also tells the game what it makes).
        for s in &lay.spawners {
            let mut r = Rng::new(plan.seed ^ (s.x as u64) << 20 ^ s.z as u64);
            for _ in 0..14 {
                let q = *s + IVec3::new(r.range(-3, 3), r.range(0, 2), r.range(-3, 3));
                if d.get(q.x, q.y, q.z) == Some(AIR) {
                    d.set(q.x, q.y, q.z, p.cobweb);
                }
            }
            d.set(s.x, s.y, s.z, p.spawner);
            d.set(s.x, s.y + 1, s.z, p.cobweb);
        }
    }
}

/// Loot tables of the deep buildings, in `DeepKind` order.
pub(super) fn deep_loot() -> Vec<Vec<LootDef>> {
    let l = |item: &str, min: u8, max: u8, chance: f32| LootDef { item: item.to_string(), min, max, chance };
    vec![
        // Cabin.
        vec![
            l("torch", 6, 16, 1.0),
            l("ladder", 4, 12, 0.5),
            l("bread", 1, 4, 0.6),
            l("iron_ingot", 2, 6, 0.5),
            l("leap_boots", 1, 1, 0.12),
            l("cloud_bottle", 1, 1, 0.12),
            l("rocket_boots", 1, 1, 0.08),
            l("grappling_hook", 1, 1, 0.15),
            l("climbing_claws", 1, 1, 0.08),
            l("lucky_horseshoe", 1, 1, 0.06),
            l("healing_potion", 1, 2, 0.4),
            l("depth_meter", 1, 1, 0.1),
        ],
        // Mineshaft.
        vec![
            l("rail", 4, 16, 0.6),
            l("torch", 4, 12, 0.8),
            l("coal", 3, 9, 0.6),
            l("iron_ingot", 1, 5, 0.5),
            l("gold_ingot", 1, 3, 0.3),
            l("leap_boots", 1, 1, 0.06),
            l("sandstorm_bottle", 1, 1, 0.05),
            l("grappling_hook", 1, 1, 0.12),
            l("bread", 1, 3, 0.5),
        ],
        // Shrine.
        vec![
            l("enchanted_book", 1, 1, 0.8),
            l("diamond", 1, 3, 0.6),
            l("blizzard_bottle", 1, 1, 0.25),
            l("charge_shield", 1, 1, 0.2),
            l("climbing_gear", 1, 1, 0.2),
            l("gold_ingot", 3, 8, 0.6),
            l("life_crystal", 1, 1, 0.4),
        ],
        // Vault.
        vec![
            l("gold_ingot", 6, 16, 1.0),
            l("platinum_ingot", 2, 6, 0.6),
            l("diamond", 2, 5, 0.7),
            l("emerald", 2, 8, 0.6),
            l("cloud_balloon", 1, 1, 0.2),
            l("rocket_boots", 1, 1, 0.2),
            l("water_walking_boots", 1, 1, 0.2),
            l("diamond_hook", 1, 1, 0.2),
            l("enchanted_book", 1, 2, 0.6),
        ],
        // Ruined house (Underworld).
        vec![
            l("magmarite_ingot", 3, 9, 0.8),
            l("obsidian", 4, 12, 0.6),
            l("obsidian_charm", 1, 1, 0.12),
            l("magma_boots", 1, 1, 0.12),
            l("potion_fire_resistance", 1, 2, 0.4),
            l("magmarite_hook", 1, 1, 0.12),
            l("glowstone_dust", 4, 12, 0.6),
            l("gold_ingot", 2, 8, 0.6),
        ],
        // Ruined tower (Underworld).
        vec![
            l("magmarite_ingot", 5, 14, 1.0),
            l("comet_boots", 1, 1, 0.12),
            l("magmarite_hook", 1, 1, 0.2),
            l("obsidian_charm", 1, 1, 0.2),
            l("diamond", 2, 6, 0.6),
            l("enchanted_book", 1, 2, 0.5),
            l("ancestral_heart", 1, 1, 0.08),
        ],
        // Hive (no chests).
        vec![],
        // Mine network: what miners left behind, a little richer than a lone corridor.
        vec![
            l("rail", 4, 16, 0.5),
            l("powered_rail", 1, 6, 0.25),
            l("torch", 6, 16, 0.7),
            l("coal", 3, 10, 0.6),
            l("raw_iron", 2, 7, 0.5),
            l("iron_ingot", 1, 5, 0.4),
            l("gold_ingot", 1, 4, 0.3),
            l("redstone", 3, 9, 0.35),
            l("lapis_lazuli", 3, 9, 0.25),
            l("diamond", 1, 2, 0.12),
            l("bread", 1, 3, 0.5),
            l("melon_seeds", 2, 4, 0.2),
            l("name_tag", 1, 1, 0.12),
            l("enchanted_book", 1, 1, 0.1),
            l("iron_pickaxe", 1, 1, 0.08),
            l("grappling_hook", 1, 1, 0.1),
            l("depth_meter", 1, 1, 0.08),
            l("healing_potion", 1, 2, 0.3),
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::SECTIONS;

    fn registry() -> BlockRegistry {
        BlockRegistry::from_ron_many(&[include_str!("../../../../packs/vanilla/blocks.ron"), include_str!("../../../../packs/vanilla/fase6/blocks.ron"), include_str!("../../../../packs/vanilla/mods/blocks.ron"), include_str!("../../../../packs/vanilla/deep/blocks.ron"), include_str!("../../../../packs/vanilla/tr/blocks.ron")]).unwrap()
    }

    #[test]
    fn slices_match_the_whole() {
        let reg = registry();
        let g = Generator::new(77, &reg);
        for pos in [ChunkPos::new(0, 0), ChunkPos::new(-7, 13), ChunkPos::new(40, -22)] {
            let full = g.generate_full(pos);
            let mut part = g.generate(pos);
            assert!(part.deep_lo > 0);
            let lo = part.deep_lo;
            g.generate_deep(&mut part, pos, 10, lo);
            g.generate_deep(&mut part, pos, 0, 10);
            assert_eq!(part.deep_lo, 0);
            for s in 0..SECTIONS {
                for i in 0..SECTION_VOLUME_TEST {
                    assert_eq!(full.sections[s].get(i), part.sections[s].get(i), "chunk {pos} section {s} cell {i}");
                }
            }
        }
    }
    const SECTION_VOLUME_TEST: usize = crate::chunk::SECTION_VOLUME;

    /// Ores by depth (plan/minerios-analise.md): diamonds deep down as the ladder wants, ores in
    /// deepslate wear their deepslate twin, and every metal of the ladder shows up.
    /// `cargo test -p voxel ore_census -- --nocapture` prints the table.
    #[test]
    fn ore_census() {
        let reg = registry();
        let g = Generator::new(1234, &reg);
        let bands = [(64, 256), (0, 64), (-160, 0), (-400, -160), (-610, -400)];
        let mut counts: std::collections::BTreeMap<String, [u32; 5]> = Default::default();
        let mut stone_in_deepslate = 0;
        let mut twin_in_deepslate = 0;
        for cz in 0..4 {
            for cx in 0..4 {
                let c = g.generate_full(ChunkPos::new(cx * 5, cz * 5));
                for y in MIN_Y..256 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let n = &reg.get(c.get_w(x, y, z)).name;
                            if !n.ends_with("_ore") {
                                continue;
                            }
                            let b = bands.iter().position(|&(lo, hi)| (lo..hi).contains(&y));
                            if let Some(b) = b {
                                counts.entry(n.clone()).or_default()[b] += 1;
                            }
                            if (-150..-10).contains(&y) {
                                if n.starts_with("deepslate_") {
                                    twin_in_deepslate += 1;
                                } else {
                                    stone_in_deepslate += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("{:28} {:>7} {:>7} {:>7} {:>7} {:>7}", "ore", "64+", "0..64", "-160..0", "-400..", "-610..");
        for (n, c) in &counts {
            println!("{:28} {:>7} {:>7} {:>7} {:>7} {:>7}", n, c[0], c[1], c[2], c[3], c[4]);
        }
        println!("deepslate band: {twin_in_deepslate} twins, {stone_in_deepslate} stone ores (in mud, granite...)");
        // The ladder (plan/progressao-linear.md): no diamond above -120, most of them deep.
        let shallow = counts.get("diamond_ore").map_or(0, |c| c[0] + c[1]) + counts.get("deepslate_diamond_ore").map_or(0, |c| c[0] + c[1]);
        assert_eq!(shallow, 0, "diamonds above y 0");
        let deep_diamonds = counts.get("diamond_ore").map_or(0, |c| c[3]);
        assert!(deep_diamonds > 0, "no diamonds in the caverns");
        assert!(twin_in_deepslate > stone_in_deepslate * 4);
        for m in ["copper", "iron", "silver", "gold", "coal", "redstone", "lapis"] {
            let total: u32 = counts.iter().filter(|(n, _)| n.contains(m)).map(|(_, c)| c.iter().sum::<u32>()).sum();
            assert!(total > 0, "no {m}");
        }
    }

    #[test]
    fn layers_from_top_to_bottom() {
        let reg = registry();
        let g = Generator::new(5, &reg);
        let c = g.generate_full(ChunkPos::new(3, 3));
        let (bedrock, lava, ash) = (reg.id("bedrock").unwrap(), reg.id("lava").unwrap(), reg.id("ash_block").unwrap());
        assert_eq!(c.get_w(4, MIN_Y, 4), bedrock);
        // The Underworld: a lava sea or ash floor at the bottom, open space above it.
        let mut open = 0;
        let mut hot = 0;
        for z in 0..16 {
            for x in 0..16 {
                for y in MIN_Y + 5..UNDERWORLD_TOP - 10 {
                    let b = c.get_w(x, y, z);
                    open += (b == AIR) as u32;
                    hot += (b == lava || b == ash) as u32;
                }
            }
        }
        assert!(open > 256 * 20, "the Underworld is open ({open})");
        assert!(hot > 256 * 4, "lava and ash down there ({hot})");
        // Deepslate just below y 0, stone in the caverns.
        let ds = reg.id("deepslate").unwrap();
        let n = (0..16).flat_map(|x| (0..16).map(move |z| (x, z))).filter(|&(x, z)| c.get_w(x, -60, z) == ds).count();
        assert!(n > 100, "{n}");
    }

    #[test]
    fn long_tunnels_and_buildings_exist() {
        let reg = registry();
        let g = Generator::new(1234, &reg);
        let mut kinds = std::collections::HashSet::new();
        for cz in -12..12 {
            for cx in -12..12 {
                for p in g.deep_plans_in(cx, cz) {
                    kinds.insert(p.kind);
                }
            }
        }
        for k in [DeepKind::Cabin, DeepKind::Mineshaft, DeepKind::RuinedHouse, DeepKind::RuinedTower] {
            assert!(kinds.contains(&k), "{k:?}");
        }
        // A tunnel network crosses a few thousand blocks without breaking.
        let col = Column { height: 0, biome: Biome::Plains, variant: Biome::Plains };
        let mut hits = 0;
        for x in (0..4000).step_by(4) {
            for z in (0..400).step_by(4) {
                if g.deep_col(x, z, &col).tunnels.iter().any(|t| t.is_some()) {
                    hits += 1;
                }
            }
        }
        assert!(hits > 200, "{hits}");
        // Chests of buildings come with loot.
        let p = g.deep_plans_in(0, 0).into_iter().chain(g.deep_plans_in(1, 0)).chain(g.deep_plans_in(0, 1)).next();
        if let Some(p) = p {
            let c = g.deep_chests(&p)[0];
            if c.y < 0 {
                assert!(g.deep_chest(c).is_some());
            }
        }
    }

    #[test]
    fn mine_networks_branch_and_hold_chests() {
        let reg = registry();
        let g = Generator::new(1234, &reg);
        let mut nets = Vec::new();
        for cz in -12..12 {
            for cx in -12..12 {
                nets.extend(g.deep_plans_in(cx, cz).into_iter().filter(|p| p.kind == DeepKind::MineNetwork));
            }
        }
        // One every ~11 cells of 48x48 blocks.
        assert!(nets.len() >= 25, "only {} mines", nets.len());
        let sizes: Vec<usize> = nets.iter().map(|p| mine_layout(p).pieces.len()).collect();
        let big = sizes.iter().filter(|&&n| n >= 20).count();
        assert!(big * 3 >= nets.len() * 2, "mines too small: {sizes:?}");
        let mine = nets.iter().max_by_key(|p| mine_layout(p).chests.len()).unwrap();
        let lay = mine_layout(mine);
        assert!(lay.pieces.iter().any(|p| matches!(p.0, MinePiece::Cross { .. })));
        // Its chest is in the world, with the mine's loot and standing in the open.
        let c = lay.chests[0];
        assert_eq!(g.deep_chest(c).map(|l| l.len()), Some(g.deep_loot[DeepKind::MineNetwork as usize].len()));
        let chunk = g.generate_full(ChunkPos::new(c.x.div_euclid(16), c.z.div_euclid(16)));
        let at = |q: IVec3| chunk.get_w(q.x.rem_euclid(16) as usize, q.y, q.z.rem_euclid(16) as usize);
        assert_eq!(at(c), g.deep_pal.chest);
        // Pieces never leave the box (so every chunk that needs them finds them).
        for p in &nets {
            for q in mine_layout(p).pieces {
                assert!(q.1.cmpge(p.origin).all() && q.2.cmplt(p.origin + p.size).all());
            }
        }
    }
}
