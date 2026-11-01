//! Structures from data (the Towns and Towers idea): ruins, towers, houses and wells written as
//! layers of characters in `structures.ron`, placed by the generator on a coarse grid of
//! regions so each chunk can work out on its own which pieces of which buildings cross it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use glam::{IVec2, IVec3, Vec3};
use serde::Deserialize;

use crate::block::{BlockId, BlockRegistry, Shape};
use crate::chunk::{Chunk, ChunkPos, CHUNK_HEIGHT};
use crate::rng::Rng;
use crate::worldgen::{Dim, Generator, SEA_LEVEL};

fn one() -> f32 {
    1.0
}
fn one_u8() -> u8 {
    1
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootDef {
    pub item: String,
    #[serde(default = "one_u8")]
    pub min: u8,
    #[serde(default = "one_u8")]
    pub max: u8,
    #[serde(default = "one")]
    pub chance: f32,
}

/// A part of a building whose chests have their own loot (a library's books, a store room's
/// food): every chest inside `lo..=hi` (local, before turning) uses `loot` instead of the
/// building's list.
#[derive(Deserialize, Clone, Debug)]
pub struct ChestZone {
    pub lo: (i32, i32, i32),
    pub hi: (i32, i32, i32),
    pub loot: Vec<LootDef>,
}

impl ChestZone {
    fn holds(&self, x: i32, y: i32, z: i32) -> bool {
        (self.lo.0..=self.hi.0).contains(&x) && (self.lo.1..=self.hi.1).contains(&y) && (self.lo.2..=self.hi.2).contains(&z)
    }
}

/// How a structure finds its height.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Place {
    /// On fairly flat dry ground (or a cave floor in the Nether).
    #[default]
    Ground,
    /// Floating on open sea (ships): the bottom `sink` layers under the water line.
    Water,
    /// High in the air at this height, whatever is below (airships, sky temples).
    Sky(i32),
}

fn rough() -> i32 {
    4
}

#[derive(Deserialize, Clone, Debug)]
pub struct StructureDef {
    pub name: String,
    /// Biome names (as shown in the F3 screen) it can appear in.
    pub biomes: Vec<String>,
    /// One try per square of this many chunks.
    pub spacing: i32,
    #[serde(default = "one")]
    pub chance: f32,
    /// Character -> block name ("." is air, " " leaves the ground as it is).
    pub palette: Vec<(char, String)>,
    /// Bottom layer first; each layer is rows from north to south, each row west to east.
    pub layers: Vec<Vec<String>>,
    /// How many layers sit below the ground.
    #[serde(default)]
    pub sink: i32,
    /// Fills under the bottom layer down to the ground, so nothing floats on a slope.
    #[serde(default)]
    pub foundation: Option<String>,
    #[serde(default)]
    pub loot: Vec<LootDef>,
    /// Rooms whose chests have their own loot.
    #[serde(default)]
    pub chests: Vec<ChestZone>,
    /// Buildings set around this one in a ring (a village square and its houses).
    #[serde(default)]
    pub pieces: Vec<String>,
    /// Only placed as a piece of another structure, never on its own.
    #[serde(default)]
    pub piece_only: bool,
    #[serde(default)]
    pub place: Place,
    /// Most the ground may rise and fall under the footprint (blocks).
    #[serde(default = "rough")]
    pub rough: i32,
    /// Mobs that live inside, spawned once when the player first comes near: (name, count).
    #[serde(default)]
    pub mobs: Vec<(String, u8)>,
    /// What the building's monster spawners make (blazes in a Nether fortress); None leaves
    /// it to the game (zombies, skeletons or spiders, cave spiders among cobwebs).
    #[serde(default)]
    pub spawner: Option<String>,
    /// Things set in the building that are not blocks (a parked wagon), placed by the game.
    #[serde(default)]
    pub props: Vec<PropDef>,
}

/// Something in a structure that is not a block: what it is (the game knows the kinds), where
/// in the plan (blocks from the plan's corner, layer 0 at the bottom, before turning) and
/// which way it faces (degrees; 0 faces +Z, 90 faces +X).
#[derive(Deserialize, Clone, Debug)]
pub struct PropDef {
    pub kind: String,
    pub at: (f32, f32, f32),
    #[serde(default)]
    pub yaw: f32,
}

/// A prop of a building in the world.
#[derive(Clone, Debug)]
pub struct Prop {
    pub kind: String,
    pub pos: Vec3,
    /// Facing, as an entity's yaw (atan2 of x over z).
    pub yaw: f32,
    /// Which building and which of its props: the same for the same thing every time.
    pub key: (IVec3, usize),
}

#[derive(Deserialize)]
struct StructureFile {
    structures: Vec<StructureDef>,
}

pub struct Structure {
    pub def: StructureDef,
    /// x, y, z size before turning.
    size: IVec3,
    /// None keeps what the terrain has there.
    cells: Vec<Option<BlockId>>,
    foundation: Option<BlockId>,
    /// The block id each id becomes after k quarter turns (stairs, roofs and doors face around).
    turned: [Vec<BlockId>; 4],
    chest: Option<BlockId>,
}

/// One building somewhere in the world.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub index: usize,
    /// Lowest corner in world blocks (after turning).
    pub origin: IVec3,
    pub turns: u8,
}

pub struct Structures {
    pub list: Vec<Structure>,
    /// Blocks a foundation pushes through: air, water, plants, snow.
    soft: Vec<bool>,
    /// Trees and plants, cleared from around a building so they don't grow through it.
    plant: Vec<bool>,
    /// Water and lava: levelling the ground leaves them be.
    liquid: Vec<bool>,
    /// Structure indices from the one that gets its place first (villages, then the biggest)
    /// to the last; a building gives way to every one before it that it would cut into.
    order: Vec<usize>,
    /// Each structure's position in `order`.
    rank: Vec<usize>,
}

impl Structures {
    pub fn empty() -> Structures {
        Structures { list: Vec::new(), soft: Vec::new(), plant: Vec::new(), liquid: Vec::new(), order: Vec::new(), rank: Vec::new() }
    }

    pub fn from_ron(text: &str, reg: &BlockRegistry) -> Result<Structures, String> {
        Self::from_ron_many(&[text], reg)
    }

    /// Several structure files read as one list (the base game, then mod packs).
    pub fn from_ron_many(texts: &[&str], reg: &BlockRegistry) -> Result<Structures, String> {
        let mut file = StructureFile { structures: Vec::new() };
        for text in texts.iter().filter(|t| !t.trim().is_empty()) {
            let f: StructureFile = ron::from_str(text).map_err(|e| format!("structures.ron: {e}"))?;
            file.structures.extend(f.structures);
        }
        let mut list = Vec::new();
        for def in file.structures {
            let sy = def.layers.len() as i32;
            let sz = def.layers.iter().map(|l| l.len()).max().unwrap_or(0) as i32;
            let sx = def.layers.iter().flat_map(|l| l.iter().map(|r| r.chars().count())).max().unwrap_or(0) as i32;
            if sx == 0 || sy == 0 || sz == 0 {
                continue;
            }
            let mut cells = vec![None; (sx * sy * sz) as usize];
            for (y, layer) in def.layers.iter().enumerate() {
                for (z, row) in layer.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        let id = match ch {
                            ' ' => None,
                            '.' => Some(crate::block::AIR),
                            c => match def.palette.iter().find(|p| p.0 == c) {
                                Some((_, name)) => Some(reg.id(name).ok_or_else(|| format!("structure {}: unknown block {name}", def.name))?),
                                None => return Err(format!("structure {}: '{c}' is not in the palette", def.name)),
                            },
                        };
                        cells[((y as i32 * sz + z as i32) * sx + x as i32) as usize] = id;
                    }
                }
            }
            pair_beds(&mut cells, IVec3::new(sx, sy, sz), reg);
            let mut turned: [Vec<BlockId>; 4] = Default::default();
            for (k, t) in turned.iter_mut().enumerate() {
                *t = (0..reg.blocks.len())
                    .map(|i| {
                        let b = &reg.blocks[i];
                        // Blocks facing one of four ways turn with the structure (an Oriented
                        // block facing up or down stays as it is).
                        let turns = matches!(b.shape, Shape::Stairs | Shape::Roof | Shape::Door | Shape::Trapdoor | Shape::Bed) || b.shape == Shape::Oriented && b.variant & 7 < 4;
                        if k == 0 || !turns {
                            return i as BlockId;
                        }
                        let v = (b.variant & !3) | ((b.variant & 3) + k as u8) & 3;
                        let base = &reg.blocks[b.base as usize].name;
                        if v == 0 {
                            b.base
                        } else {
                            reg.id(&format!("{base}:{v}")).unwrap_or(i as BlockId)
                        }
                    })
                    .collect();
            }
            let foundation = def.foundation.as_deref().and_then(|n| reg.id(n));
            let chest = reg.id("chest");
            list.push(Structure { def, size: IVec3::new(sx, sy, sz), cells, foundation, turned, chest });
        }
        let soft = reg.blocks.iter().map(|b| matches!(b.shape, Shape::Liquid | Shape::Cross | Shape::Layer | Shape::Invisible) || b.name == "air").collect();
        let plant = reg.blocks.iter().map(|b| b.shape == Shape::Cross || b.name.ends_with("_leaves") || b.name.ends_with("_log") || b.name == "vine").collect();
        let liquid = reg.blocks.iter().map(|b| b.shape == Shape::Liquid).collect();
        let mut order: Vec<usize> = (0..list.len()).collect();
        order.sort_by_key(|&i| {
            let s: &Structure = &list[i];
            (s.def.pieces.is_empty(), -(s.size.x * s.size.z * s.size.y), i)
        });
        let mut rank = vec![0; list.len()];
        for (k, &i) in order.iter().enumerate() {
            rank[i] = k;
        }
        Ok(Structures { list, soft, plant, liquid, order, rank })
    }
}

/// Buildings name a bed once, as a single cell: make it a whole bed, the foot there and the
/// head in the next free cell (a neighbouring bed cell first, then air, then a cell left
/// as it is), turned toward it.
fn pair_beds(cells: &mut [Option<BlockId>], size: IVec3, reg: &BlockRegistry) {
    let at = |x: i32, y: i32, z: i32| (x >= 0 && z >= 0 && x < size.x && z < size.z).then(|| ((y * size.z + z) * size.x + x) as usize);
    let lone = |c: Option<BlockId>| c.is_some_and(|id| reg.get(id).shape == Shape::Bed && reg.get(id).variant == 0);
    let mut done = vec![false; cells.len()];
    for y in 0..size.y {
        for z in 0..size.z {
            for x in 0..size.x {
                let i = at(x, y, z).unwrap();
                if done[i] || !lone(cells[i]) {
                    continue;
                }
                let id = cells[i].unwrap();
                // Toward north, east, south, west (the bed variants' facings).
                let ways = [(0, -1), (1, 0), (0, 1), (-1, 0)];
                let pick = |ok: &dyn Fn(Option<BlockId>) -> bool| ways.iter().enumerate().find_map(|(f, &(dx, dz))| at(x + dx, y, z + dz).filter(|&j| !done[j] && ok(cells[j])).map(|j| (f as u8, j)));
                let Some((f, j)) = pick(&|c| lone(c) && c == cells[i]).or_else(|| pick(&|c| c == Some(crate::block::AIR))).or_else(|| pick(&|c| c.is_none())) else {
                    continue;
                };
                cells[i] = Some(reg.variant(id, f));
                cells[j] = Some(reg.variant(id, f | crate::bed::HEAD));
                done[i] = true;
                done[j] = true;
            }
        }
    }
}

impl Structure {
    fn footprint(&self, turns: u8) -> IVec2 {
        if turns & 1 == 0 {
            IVec2::new(self.size.x, self.size.z)
        } else {
            IVec2::new(self.size.z, self.size.x)
        }
    }

    /// The block at a position inside the turned footprint (local, 0-based).
    fn cell(&self, turns: u8, x: i32, y: i32, z: i32) -> Option<BlockId> {
        let (sx, sz) = (self.size.x, self.size.z);
        let (ux, uz) = self.unturn(turns, x, z);
        if ux < 0 || uz < 0 || ux >= sx || uz >= sz || y < 0 || y >= self.size.y {
            return None;
        }
        self.cells[((y * sz + uz) * sx + ux) as usize].map(|id| self.turned[turns as usize][id as usize])
    }

    /// A position in the turned footprint back in the building's own (unturned) frame.
    fn unturn(&self, turns: u8, x: i32, z: i32) -> (i32, i32) {
        // Undo the turn: each quarter turn maps (x, z) -> (sz - 1 - z, x).
        let (mut ux, mut uz) = (x, z);
        let mut w = self.footprint(turns);
        for _ in 0..turns {
            // Inverse of (x, z) -> (h - 1 - z, x) on a w.x by w.y area whose source was w.y by w.x.
            let (px, pz) = (uz, w.x - 1 - ux);
            ux = px;
            uz = pz;
            w = IVec2::new(w.y, w.x);
        }
        (ux, uz)
    }

    /// The loot list of a chest at local (turned) `l`: its room's, else the building's.
    fn chest_loot(&self, turns: u8, l: IVec3) -> &[LootDef] {
        let (ux, uz) = self.unturn(turns, l.x, l.z);
        self.def.chests.iter().find(|c| c.holds(ux, l.y, uz)).map_or(&self.def.loot, |c| &c.loot)
    }
}

impl Structure {
    /// The longer side of the footprint.
    pub fn side(&self) -> i32 {
        self.size.x.max(self.size.z)
    }

    /// How far from its try's corner the building (and a village's houses) can reach.
    fn reach(&self) -> i32 {
        if self.def.pieces.is_empty() {
            self.side()
        } else {
            self.side() + VILLAGE_REACH
        }
    }

    /// Deep in the ground with only a stair or shaft coming up (strongholds, ancient cities,
    /// trial chambers): the surface's shape doesn't matter to them.
    pub fn buried(&self) -> bool {
        self.def.place == Place::Ground && self.def.sink >= 8 && self.def.sink >= self.size.y - 5
    }

    /// Wrecks and ruins on the sea floor.
    pub fn sea_floor(&self) -> bool {
        self.def.biomes.iter().any(|b| b == "Ocean" || b == "Frozen Ocean")
    }

    /// Buildings on land whose ground gets levelled around them.
    fn levelled(&self) -> bool {
        self.def.place == Place::Ground && !self.buried() && !self.sea_floor()
    }
}

impl Structures {
    /// The box a building fills in the world: lowest corner and one past the highest.
    fn bounds(&self, p: &Placement) -> (IVec3, IVec3) {
        let s = &self.list[p.index];
        let f = s.footprint(p.turns);
        (p.origin, p.origin + IVec3::new(f.x, s.size.y, f.y))
    }
}

/// A building that got its place, with a village's houses.
#[derive(Clone, Debug)]
pub(crate) struct Site {
    pub p: Placement,
    pub pieces: Vec<Placement>,
}

impl Site {
    fn all(&self) -> impl Iterator<Item = &Placement> {
        std::iter::once(&self.p).chain(self.pieces.iter())
    }
}

type CacheKey = (usize, i32, i32);

/// Where the buildings of each try region went, worked out once: deciding means looking at
/// every bigger building around, and every chunk, chest and dweller check asks again.
pub struct PlaceCache {
    shards: Vec<Mutex<HashMap<CacheKey, Option<Arc<Site>>>>>,
}

impl Default for PlaceCache {
    fn default() -> Self {
        PlaceCache { shards: (0..16).map(|_| Mutex::new(HashMap::new())).collect() }
    }
}

impl PlaceCache {
    fn shard(&self, k: &CacheKey) -> &Mutex<HashMap<CacheKey, Option<Arc<Site>>>> {
        let h = (k.1 as u32).wrapping_mul(0x9E37_79B9) ^ (k.2 as u32).wrapping_mul(0x85EB_CA6B) ^ k.0 as u32;
        &self.shards[(h >> 7) as usize % self.shards.len()]
    }

    fn get(&self, si: usize, rx: i32, rz: i32) -> Option<Option<Arc<Site>>> {
        let k = (si, rx, rz);
        self.shard(&k).lock().ok()?.get(&k).cloned()
    }

    fn put(&self, si: usize, rx: i32, rz: i32, site: Option<Arc<Site>>) {
        let k = (si, rx, rz);
        if let Ok(mut m) = self.shard(&k).lock() {
            // A long trip far and wide: start over rather than grow without end.
            if m.len() > 200_000 {
                m.clear();
            }
            m.insert(k, site);
        }
    }
}

/// A structure's own seed for its dice (FNV-1a of its name).
fn name_salt(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3))
}

/// The middle value of five heights.
fn median5(mut hs: [i32; 5]) -> i32 {
    hs.sort_unstable();
    hs[2]
}

/// Least room left between two buildings (blocks).
const GAP: i32 = 3;
/// How far round a building on land the ground is eased into its level (blocks).
const TERRAFORM: i32 = 6;

/// How far the pieces of a village reach from its square.
const VILLAGE_REACH: i32 = 48;

impl Structure {
    /// The size before turning (x, layers, z).
    pub fn size(&self) -> IVec3 {
        self.size
    }

    /// Footprint after `turns` quarter turns (x, z).
    pub fn turned_footprint(&self, turns: u8) -> IVec2 {
        self.footprint(turns)
    }
}

impl Generator {
    /// The side of a building's try region, in blocks.
    fn region(&self, s: &Structure) -> i32 {
        s.def.spacing.max(2) * 16 * self.spread / 100
    }

    /// Extra try regions to look at on each side: the End's buildings move to the middle of
    /// an island, which may lie a little outside their region.
    fn slack(&self) -> i32 {
        i32::from(self.dim == Dim::End)
    }

    /// The building `si` of try region (rx, rz) once everything bigger near it has had its
    /// place: None when its try failed or it would cut into one of those (cached).
    pub(crate) fn site(&self, si: usize, rx: i32, rz: i32) -> Option<Arc<Site>> {
        if let Some(hit) = self.place_cache.get(si, rx, rz) {
            return hit;
        }
        let site = self.site_uncached(si, rx, rz).map(Arc::new);
        self.place_cache.put(si, rx, rz, site.clone());
        site
    }

    fn site_uncached(&self, si: usize, rx: i32, rz: i32) -> Option<Site> {
        let structs = self.structures();
        let s = &structs.list[si];
        let r = self.region(s);
        let p = self.try_place(si, s, rx, rz, r)?;
        let higher = &structs.order[..structs.rank[si]];
        if self.blocked(structs, higher, &p) || self.crowded(structs, si, rx, rz, &p) {
            return None;
        }
        // A village keeps the houses that fit; a house that would cut into a bigger building
        // is left out.
        let pieces = self.village_pieces(structs, s, &p, rx, rz).into_iter().filter(|q| !self.blocked(structs, higher, q) && !self.crowded(structs, si, rx, rz, q)).collect();
        Some(Site { p, pieces })
    }

    /// Whether `p` would cut into a building of the structures `higher`.
    fn blocked(&self, structs: &Structures, higher: &[usize], p: &Placement) -> bool {
        let (lo, hi) = structs.bounds(p);
        for &sj in higher {
            let o = &structs.list[sj];
            let r = self.region(o);
            let ext = o.reach();
            // A village's houses reach out on every side of its square's corner.
            let back = ext - o.side();
            let k = self.slack();
            for rz in (lo.z - ext - GAP).div_euclid(r) - k..=(hi.z + GAP + back).div_euclid(r) + k {
                for rx in (lo.x - ext - GAP).div_euclid(r) - k..=(hi.x + GAP + back).div_euclid(r) + k {
                    let Some(site) = self.site(sj, rx, rz) else { continue };
                    if site.all().any(|q| {
                        let (a, b) = structs.bounds(q);
                        a.x < hi.x + GAP && lo.x < b.x + GAP && a.z < hi.z + GAP && lo.z < b.z + GAP && a.y < hi.y && lo.y < b.y
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Whether `p` (of structure `si`'s try in region (rx, rz)) would cut into the same
    /// structure's building of an earlier region: a village's houses reach past its region.
    /// The earlier one's own try is taken as it is, without asking what it gave way to, so
    /// this never chains from region to region.
    fn crowded(&self, structs: &Structures, si: usize, rx: i32, rz: i32, p: &Placement) -> bool {
        let s = &structs.list[si];
        let r = self.region(s);
        let (lo, hi) = structs.bounds(p);
        let ext = s.reach();
        let back = ext - s.side();
        let k = self.slack();
        for oz in (lo.z - ext - GAP).div_euclid(r) - k..=(hi.z + GAP + back).div_euclid(r) + k {
            for ox in (lo.x - ext - GAP).div_euclid(r) - k..=(hi.x + GAP + back).div_euclid(r) + k {
                if (oz, ox) >= (rz, rx) {
                    continue;
                }
                let Some(q) = self.try_place(si, s, ox, oz, r) else { continue };
                let pieces = self.village_pieces(structs, s, &q, ox, oz);
                if std::iter::once(&q).chain(pieces.iter()).any(|q| {
                    let (a, b) = structs.bounds(q);
                    a.x < hi.x + GAP && lo.x < b.x + GAP && a.z < hi.z + GAP && lo.z < b.z + GAP && a.y < hi.y && lo.y < b.y
                }) {
                    return true;
                }
            }
        }
        false
    }

    /// Buildings whose footprint may cross this chunk.
    pub fn placements_near(&self, pos: ChunkPos) -> Vec<Placement> {
        self.placements_touching(pos, 0)
    }

    /// Buildings whose footprint, grown by `margin` blocks, crosses this chunk.
    fn placements_touching(&self, pos: ChunkPos, margin: i32) -> Vec<Placement> {
        let mut out = Vec::new();
        let structs = self.structures();
        let hits = |p: &Placement, f: IVec2| p.origin.x - margin <= pos.x * 16 + 15 && p.origin.x + f.x + margin > pos.x * 16 && p.origin.z - margin <= pos.y * 16 + 15 && p.origin.z + f.y + margin > pos.y * 16;
        if let Some(sc) = &self.showcase {
            // The test world: one of each, where the game set them out.
            return sc.buildings.iter().filter(|p| hits(p, structs.list[p.index].footprint(p.turns))).copied().collect();
        }
        for (si, s) in structs.list.iter().enumerate() {
            if s.def.piece_only {
                continue;
            }
            let r = self.region(s);
            let ext = s.reach() + margin;
            let k = self.slack();
            let x0 = (pos.x * 16 - ext).div_euclid(r) - k;
            let x1 = (pos.x * 16 + 15 + margin + (ext - s.side())).div_euclid(r) + k;
            let z0 = (pos.y * 16 - ext).div_euclid(r) - k;
            let z1 = (pos.y * 16 + 15 + margin + (ext - s.side())).div_euclid(r) + k;
            for rz in z0..=z1 {
                for rx in x0..=x1 {
                    if let Some(site) = self.site(si, rx, rz) {
                        out.extend(site.all().filter(|q| hits(q, structs.list[q.index].footprint(q.turns))).copied());
                    }
                }
            }
        }
        out
    }

    /// The buildings of a village around its square: set in a ring, each turned to face the
    /// square, each only where the ground is fairly flat and dry.
    fn village_pieces(&self, structs: &Structures, s: &Structure, center: &Placement, rx: i32, rz: i32) -> Vec<Placement> {
        let mut out: Vec<Placement> = Vec::new();
        if s.def.pieces.is_empty() {
            return out;
        }
        let mut rng = Rng::for_pos(self.seed, rx, rz, 9100);
        let f0 = s.footprint(center.turns);
        let (cx, cz) = (center.origin.x + f0.x / 2, center.origin.z + f0.y / 2);
        let n = s.def.pieces.len();
        let a0 = rng.f32() * std::f32::consts::TAU;
        for (k, name) in s.def.pieces.iter().enumerate() {
            let Some(pi) = structs.list.iter().position(|p| &p.def.name == name) else { continue };
            let piece = &structs.list[pi];
            let a = a0 + k as f32 / n as f32 * std::f32::consts::TAU + (rng.f32() - 0.5) * 0.3;
            let dist = 15.0 + rng.f32() * 9.0 + (k % 2) as f32 * 8.0;
            let (dx, dz) = (a.cos(), a.sin());
            // The door (south in the plan) toward the square.
            let turns = if dz.abs() > dx.abs() {
                if dz < 0.0 { 0 } else { 2 }
            } else if dx > 0.0 {
                1
            } else {
                3
            };
            let f = piece.footprint(turns);
            let (px, pz) = (cx + (dx * dist) as i32 - f.x / 2, cz + (dz * dist) as i32 - f.y / 2);
            // Not on top of another piece.
            if out.iter().any(|o| {
                let g = structs.list[o.index].footprint(o.turns);
                px < o.origin.x + g.x + 2 && o.origin.x < px + f.x + 2 && pz < o.origin.z + g.y + 2 && o.origin.z < pz + f.y + 2
            }) {
                continue;
            }
            let hs = [self.height_at(px, pz), self.height_at(px + f.x - 1, pz), self.height_at(px, pz + f.y - 1), self.height_at(px + f.x - 1, pz + f.y - 1), self.height_at(px + f.x / 2, pz + f.y / 2)];
            let (lo, hi) = (*hs.iter().min().unwrap(), *hs.iter().max().unwrap());
            // The ground is levelled around each house (`terraform`), so a gentle slope will do.
            if lo < SEA_LEVEL || hi - lo > 6 {
                continue;
            }
            out.push(Placement { index: pi, origin: IVec3::new(px, median5(hs) + 1 - piece.def.sink, pz), turns });
        }
        out
    }

    /// Village squares near (x, z): their middle, for whoever wants to people them.
    pub fn villages_near(&self, x: i32, z: i32, range: i32) -> Vec<IVec3> {
        let structs = self.structures();
        let mut out = Vec::new();
        for (si, s) in structs.list.iter().enumerate() {
            if s.def.pieces.is_empty() {
                continue;
            }
            let r = self.region(s);
            for rz in (z - range).div_euclid(r)..=(z + range).div_euclid(r) {
                for rx in (x - range).div_euclid(r)..=(x + range).div_euclid(r) {
                    if let Some(site) = self.site(si, rx, rz) {
                        let p = site.p;
                        let f = s.footprint(p.turns);
                        out.push(IVec3::new(p.origin.x + f.x / 2, p.origin.y + s.def.sink + 1, p.origin.z + f.y / 2));
                    }
                }
            }
        }
        out
    }

    /// Every building (village pieces too) whose try lands in the square x0..x1, z0..z1: for
    /// maps, `--find` and the structure audit test.
    pub fn placements_in(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<Placement> {
        let structs = self.structures();
        let mut out = Vec::new();
        for (si, s) in structs.list.iter().enumerate() {
            if s.def.piece_only {
                continue;
            }
            let r = self.region(s);
            for rz in z0.div_euclid(r)..=z1.div_euclid(r) {
                for rx in x0.div_euclid(r)..=x1.div_euclid(r) {
                    if let Some(site) = self.site(si, rx, rz) {
                        out.extend(site.all().copied());
                    }
                }
            }
        }
        out
    }

    /// One building's try in region (rx, rz): the dice say whether there is one at all, then a
    /// few spots are tried (more for big buildings, which need more room) until one suits it.
    fn try_place(&self, si: usize, s: &Structure, rx: i32, rz: i32, r: i32) -> Option<Placement> {
        if self.showcase.is_some() {
            return None;
        }
        // The dice go by the building's name, not its place in the list, so adding a
        // structure to a file doesn't move every one after it.
        let salt = name_salt(&s.def.name);
        let mut rng = Rng::for_pos(self.seed, rx, rz, salt);
        if rng.f32() >= s.def.chance {
            return None;
        }
        let side = s.side();
        let tries = if side >= 32 { 6 } else if side >= 16 { 3 } else { 2 };
        for k in 0..tries {
            // The first spot is the one the game always picked, so most buildings stay put.
            if k > 0 {
                rng = Rng::for_pos(self.seed, rx, rz, salt.wrapping_add(100_000 * k as u64));
            }
            if let Some(p) = self.try_spot(si, s, rx, rz, r, &mut rng) {
                return Some(p);
            }
        }
        None
    }

    fn try_spot(&self, si: usize, s: &Structure, rx: i32, rz: i32, r: i32, rng: &mut Rng) -> Option<Placement> {
        let turns = rng.below(4) as u8;
        let f = s.footprint(turns);
        let mut ox = rx * r + rng.range(0, (r - f.x).max(1));
        let mut oz = rz * r + rng.range(0, (r - f.y).max(1));
        if self.dim == Dim::End {
            // The End's buildings stand on its far islands: pick one in this region and build
            // in its middle (a random spot would nearly always be over the void).
            let cell = crate::worldgen::END_CELL;
            let cells = (r / cell).max(1);
            let cx = (rx * r).div_euclid(cell) + rng.range(0, cells - 1);
            let cz = (rz * r).div_euclid(cell) + rng.range(0, cells - 1);
            let (ix, iz) = self.end_island(cx, cz)?;
            ox = ix - f.x / 2;
            oz = iz - f.y / 2;
        }
        let (cx, cz) = (ox + f.x / 2, oz + f.y / 2);
        let biome = self.biome_at(cx, cz);
        if !s.def.biomes.iter().any(|b| b == biome.name() || b == biome.base().name()) {
            return None;
        }
        if let Place::Sky(h) = s.def.place {
            return Some(Placement { index: si, origin: IVec3::new(ox, h, oz), turns });
        }
        if s.def.place == Place::Water {
            // Open sea under the whole hull.
            let deep = [(ox, oz), (ox + f.x - 1, oz), (ox, oz + f.y - 1), (ox + f.x - 1, oz + f.y - 1), (cx, cz)].iter().all(|&(x, z)| self.height_at(x, z) < SEA_LEVEL - 3);
            if !deep {
                return None;
            }
            return Some(Placement { index: si, origin: IVec3::new(ox, SEA_LEVEL + 1 - s.def.sink, oz), turns });
        }
        let corners = [(ox, oz), (ox + f.x - 1, oz), (ox, oz + f.y - 1), (ox + f.x - 1, oz + f.y - 1), (cx, cz)];
        if self.dim == Dim::End {
            // On the island's top: ships float high above it; the rest want the middle and at
            // least three corners on land (a keel of rock fills under the rest).
            let ground = self.end_top(cx, cz)?;
            if s.def.name.starts_with("navio") {
                return Some(Placement { index: si, origin: IVec3::new(ox, ground + 22, oz), turns });
            }
            let tops: Vec<i32> = corners.iter().filter_map(|&(x, z)| self.end_top(x, z)).collect();
            if tops.len() < corners.len() - 2 {
                return None;
            }
            let (lo, hi) = (*tops.iter().min().unwrap(), *tops.iter().max().unwrap());
            if hi - lo > s.def.rough + 4 {
                return None;
            }
            return Some(Placement { index: si, origin: IVec3::new(ox, ground + 1 - s.def.sink, oz), turns });
        }
        if self.dim == Dim::Aether {
            // On the Aether's islands, on fairly flat ground.
            let mut hs = Vec::with_capacity(5);
            for &(x, z) in &corners {
                hs.push(self.aether_land(x, z)?.0);
            }
            let (lo, hi) = (*hs.iter().min().unwrap(), *hs.iter().max().unwrap());
            if hi - lo > s.def.rough {
                return None;
            }
            return Some(Placement { index: si, origin: IVec3::new(ox, hs[4] + 1 - s.def.sink, oz), turns });
        }
        if matches!(self.dim, Dim::Nether | Dim::Otherside) {
            // On a cave floor above the lava sea (or the Otherside's pools), with room overhead.
            let solid = |x: i32, y: i32, z: i32| if self.dim == Dim::Nether { self.nether_solid(x, y, z) } else { self.otherside_solid(x, y, z) };
            let tall = s.size.y + 2;
            let mut ground = None;
            for y in crate::worldgen::NETHER_LAVA + 1..crate::worldgen::NETHER_ROOF - tall {
                if solid(cx, y, cz) && (1..=tall).all(|k| !solid(cx, y + k, cz)) {
                    ground = Some(y);
                    break;
                }
            }
            let ground = ground?;
            let flat = [(ox, oz), (ox + f.x - 1, oz), (ox, oz + f.y - 1), (ox + f.x - 1, oz + f.y - 1)]
                .iter()
                .all(|&(x, z)| !solid(x, ground + 2, z) && !solid(x, ground + s.size.y, z));
            if !flat {
                return None;
            }
            return Some(Placement { index: si, origin: IVec3::new(ox, ground + 1 - s.def.sink, oz), turns });
        }
        // Wrecks and the like sit on the sea floor, everything else on dry land.
        let sea_floor = s.sea_floor();
        if s.buried() {
            // Deep under the ground, only a stair or a shaft comes up: the shape of the
            // surface doesn't matter, only that it is dry land (or sea floor).
            let g = self.height_at(cx, cz);
            if if sea_floor { g > SEA_LEVEL - 6 } else { g < SEA_LEVEL } {
                return None;
            }
            return Some(Placement { index: si, origin: IVec3::new(ox, g + 1 - s.def.sink, oz), turns });
        }
        let hs = corners.map(|(x, z)| self.height_at(x, z));
        let lo = *hs.iter().min().unwrap();
        let hi = *hs.iter().max().unwrap();
        // The ground is levelled around buildings on land (`terraform`), so the bigger ones
        // may stand on a gentler slope than their plan says.
        let rough = if sea_floor { s.def.rough } else { s.def.rough + s.side() / 6 };
        if (if sea_floor { hi > SEA_LEVEL - 6 } else { lo < SEA_LEVEL }) || hi - lo > rough {
            return None;
        }
        let ground = if sea_floor { hs[4] } else { median5(hs) };
        Some(Placement { index: si, origin: IVec3::new(ox, ground + 1 - s.def.sink, oz), turns })
    }

    /// Build the parts of nearby buildings that fall in this chunk.
    pub fn place_structures(&self, chunk: &mut Chunk) {
        let pos = chunk.pos;
        let structs = self.structures();
        let near = self.placements_touching(pos, TERRAFORM);
        // First the ground is levelled round every building, then they are all built, so
        // the levelling of one never cuts into another.
        if self.dim == Dim::Overworld && self.showcase.is_none() {
            for p in &near {
                let s = &structs.list[p.index];
                if s.levelled() {
                    self.terraform(chunk, structs, s, p);
                }
            }
        }
        for p in near {
            let s = &structs.list[p.index];
            let f = s.footprint(p.turns);
            for lz in 0..16 {
                for lx in 0..16 {
                    let (wx, wz) = (pos.x * 16 + lx, pos.y * 16 + lz);
                    let (x, z) = (wx - p.origin.x, wz - p.origin.z);
                    // A clearing one block wider than the building, free of trees.
                    if s.def.place == Place::Ground && x >= -1 && z >= -1 && x <= f.x && z <= f.y {
                        for y in p.origin.y.max(1)..(p.origin.y + s.size.y + 6).min(CHUNK_HEIGHT as i32) {
                            let cur = chunk.get(lx as usize, y as usize, lz as usize);
                            if structs.plant.get(cur as usize).copied().unwrap_or(false) {
                                chunk.set(lx as usize, y as usize, lz as usize, crate::block::AIR);
                            }
                        }
                    }
                    if x < 0 || z < 0 || x >= f.x || z >= f.y {
                        continue;
                    }
                    for y in 0..s.size.y {
                        let wy = p.origin.y + y;
                        if wy <= 0 || wy >= CHUNK_HEIGHT as i32 {
                            continue;
                        }
                        if let Some(id) = s.cell(p.turns, x, y, z) {
                            chunk.set(lx as usize, wy as usize, lz as usize, id);
                        }
                    }
                    // A foundation under solid bottom cells, down to the ground.
                    if let (Some(fd), Some(b)) = (s.foundation, s.cell(p.turns, x, 0, z)) {
                        if b != crate::block::AIR {
                            let mut y = p.origin.y - 1;
                            // Over the End's void a foundation is a short rocky keel, not a column.
                            let floor = if self.dim == Dim::End { p.origin.y - 10 } else { 0 };
                            while y > floor.max(0) {
                                let cur = chunk.get(lx as usize, y as usize, lz as usize);
                                if cur != crate::block::AIR && !structs.soft.get(cur as usize).copied().unwrap_or(false) {
                                    break;
                                }
                                chunk.set(lx as usize, y as usize, lz as usize, fd);
                                y -= 1;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Level the ground of a building on land and ease it into the land around (Minecraft's
    /// "beardifier" idea): inside the footprint the ground is cut or filled to the floor's
    /// height; outside, for `TERRAFORM` blocks, the cut and the fill fall away in a slope, so
    /// a house on a hillside sits on a little terrace instead of a hill through its walls.
    fn terraform(&self, chunk: &mut Chunk, structs: &Structures, s: &Structure, p: &Placement) {
        let pos = chunk.pos;
        let f = s.footprint(p.turns);
        let g = p.origin.y + s.def.sink - 1;
        if g <= 2 || g >= CHUNK_HEIGHT as i32 - 2 {
            return;
        }
        let solid = |b: BlockId| b != crate::block::AIR && !structs.soft.get(b as usize).copied().unwrap_or(false) && !structs.plant.get(b as usize).copied().unwrap_or(false);
        for lz in 0..16i32 {
            for lx in 0..16i32 {
                let (wx, wz) = (pos.x * 16 + lx, pos.y * 16 + lz);
                let dx = (p.origin.x - wx).max(wx - (p.origin.x + f.x - 1)).max(0);
                let dz = (p.origin.z - wz).max(wz - (p.origin.z + f.y - 1)).max(0);
                let d = ((dx * dx + dz * dz) as f32).sqrt();
                if d > TERRAFORM as f32 {
                    continue;
                }
                let (ux, uz) = (lx as usize, lz as usize);
                // The column's ground as the generator left it.
                let Some(h) = (1..CHUNK_HEIGHT as i32 - 1).rev().find(|&y| solid(chunk.get(ux, y as usize, uz))) else { continue };
                let top = chunk.get(ux, h as usize, uz);
                let below = chunk.get(ux, h as usize - 1, uz);
                let sub = if solid(below) { below } else { top };
                let jitter = Rng::for_pos(self.seed, wx, wz, 0x7E44).f32();
                let slope = if d <= 0.0 { 0 } else { (d * d * 0.5 + jitter) as i32 };
                let cut = g + if d <= 0.0 { 0 } else { slope + 1 };
                let fill = g - slope;
                if h > cut && h - cut <= 40 {
                    for y in cut + 1..CHUNK_HEIGHT as i32 {
                        let b = chunk.get(ux, y as usize, uz);
                        if b != crate::block::AIR && !structs.liquid.get(b as usize).copied().unwrap_or(false) {
                            chunk.set(ux, y as usize, uz, crate::block::AIR);
                        }
                    }
                    if solid(chunk.get(ux, cut as usize, uz)) {
                        chunk.set(ux, cut as usize, uz, top);
                    }
                } else if h < fill && fill - h <= 16 {
                    chunk.set(ux, h as usize, uz, sub);
                    for y in h + 1..fill {
                        chunk.set(ux, y as usize, uz, sub);
                    }
                    chunk.set(ux, fill as usize, uz, top);
                    // Plants that stood on the old ground would be buried or float: gone.
                    for y in fill + 1..(fill + 3).min(CHUNK_HEIGHT as i32) {
                        let b = chunk.get(ux, y as usize, uz);
                        if structs.plant.get(b as usize).copied().unwrap_or(false) || structs.soft.get(b as usize).copied().unwrap_or(false) && !structs.liquid.get(b as usize).copied().unwrap_or(false) {
                            chunk.set(ux, y as usize, uz, crate::block::AIR);
                        }
                    }
                }
            }
        }
    }

    /// The props (parked wagons and the like) of the buildings within `range` blocks of (x, z).
    pub fn props_near(&self, x: i32, z: i32, range: i32) -> Vec<Prop> {
        let structs = self.structures();
        let mut out = Vec::new();
        let mut found: Vec<Placement> = Vec::new();
        if let Some(sc) = &self.showcase {
            found = sc.buildings.clone();
        }
        for (si, s) in structs.list.iter().enumerate() {
            if s.def.props.is_empty() || s.def.piece_only {
                continue;
            }
            let r = self.region(s);
            for rz in (z - range).div_euclid(r)..=(z + range).div_euclid(r) {
                for rx in (x - range).div_euclid(r)..=(x + range).div_euclid(r) {
                    if let Some(site) = self.site(si, rx, rz) {
                        found.push(site.p);
                    }
                }
            }
        }
        for p in found {
            let s = &structs.list[p.index];
            for (k, d) in s.def.props.iter().enumerate() {
                // Turn the plan point and the facing the way the building was turned:
                // each quarter turn maps (u, v) on a w by d plan to (d - v, u).
                let (mut u, mut v) = (d.at.0, d.at.2);
                let (mut fx, mut fz) = (d.yaw.to_radians().sin(), d.yaw.to_radians().cos());
                let (mut w, mut dd) = (s.size.x as f32, s.size.z as f32);
                for _ in 0..p.turns {
                    (u, v) = (dd - v, u);
                    (fx, fz) = (-fz, fx);
                    (w, dd) = (dd, w);
                }
                let _ = w;
                let pos = Vec3::new(p.origin.x as f32 + u, p.origin.y as f32 + d.at.1, p.origin.z as f32 + v);
                if (pos.x - x as f32).abs() <= range as f32 && (pos.z - z as f32).abs() <= range as f32 {
                    out.push(Prop { kind: d.kind.clone(), pos, yaw: fx.atan2(fz), key: (p.origin, k) });
                }
            }
        }
        out
    }

    /// The nearest building called `name` within `regions` tries of (x, z): its lowest corner and
    /// its turned size.
    pub fn find_structure(&self, name: &str, x: i32, z: i32, regions: i32) -> Option<(IVec3, IVec3)> {
        let structs = self.structures();
        let (si, s) = structs.list.iter().enumerate().find(|(_, s)| s.def.name == name)?;
        let r = self.region(s);
        let (cx, cz) = (x.div_euclid(r), z.div_euclid(r));
        let mut best: Option<(i32, Placement)> = None;
        for rz in cz - regions..=cz + regions {
            for rx in cx - regions..=cx + regions {
                if let Some(p) = self.site(si, rx, rz).map(|site| site.p) {
                    let d = (p.origin.x - x).abs() + (p.origin.z - z).abs();
                    if best.is_none_or(|b| d < b.0) {
                        best = Some((d, p));
                    }
                }
            }
        }
        best.map(|(_, p)| {
            let f = s.footprint(p.turns);
            (p.origin, IVec3::new(f.x, s.size.y, f.y))
        })
    }

    /// The mob a spawner at `p` makes, when the building it stands in says.
    pub fn structure_spawner(&self, p: IVec3) -> Option<String> {
        let pos = ChunkPos::new(p.x.div_euclid(16), p.z.div_euclid(16));
        let structs = self.structures();
        self.placements_near(pos).into_iter().find_map(|pl| {
            let s = &structs.list[pl.index];
            let (lo, hi) = structs.bounds(&pl);
            let inside = p.x >= lo.x && p.y >= lo.y && p.z >= lo.z && p.x < hi.x && p.y < hi.y && p.z < hi.z;
            inside.then(|| s.def.spawner.clone()).flatten()
        })
    }

    /// The coins (in copper) a chest the generator put at `p` holds on average: more in
    /// bigger and guarded buildings, much more past the Overworld, and deeper down.
    pub fn structure_purse(&self, p: IVec3) -> u64 {
        let pos = ChunkPos::new(p.x.div_euclid(16), p.z.div_euclid(16));
        let structs = self.structures();
        let dim = match self.dim {
            Dim::Overworld => 1.0,
            Dim::Nether => 4.0,
            Dim::Aether => 6.0,
            Dim::Otherside => 8.0,
            Dim::End => 12.0,
        };
        for pl in self.placements_near(pos) {
            let s = &structs.list[pl.index];
            let l = p - pl.origin;
            if s.chest.is_some() && s.cell(pl.turns, l.x, l.y, l.z) == s.chest {
                let guarded = !s.def.mobs.is_empty() || s.def.spawner.is_some() || s.buried();
                let base = 15.0 + s.side() as f32 * 2.0;
                return (base * if guarded { 1.6 } else { 1.0 } * dim) as u64;
            }
        }
        if self.deep_chest(p).is_some() {
            return 40 + p.y.unsigned_abs() as u64 * 2;
        }
        0
    }

    /// How much more a world chest at `p` holds than an Overworld surface one: later worlds
    /// (Nether, Aether, Otherside, End) and deep buried rooms are worth the trip.
    pub fn loot_scale(&self, p: IVec3) -> f32 {
        let dim = match self.dim {
            Dim::Overworld => 1.0,
            Dim::Nether => 1.5,
            Dim::Aether => 1.75,
            Dim::Otherside | Dim::End => 2.0,
        };
        let deep = if self.dim == Dim::Overworld && p.y < 0 { 1.0 + (-p.y as f32 / 400.0).min(1.0) } else { 1.0 };
        dim * deep
    }

    /// The loot of a chest the generator put at `p`, if it did.
    pub fn structure_chest(&self, p: IVec3) -> Option<&[LootDef]> {
        let pos = ChunkPos::new(p.x.div_euclid(16), p.z.div_euclid(16));
        let structs = self.structures();
        for pl in self.placements_near(pos) {
            let s = &structs.list[pl.index];
            let l = p - pl.origin;
            if s.chest.is_some() && s.cell(pl.turns, l.x, l.y, l.z) == s.chest {
                return Some(s.chest_loot(pl.turns, l));
            }
        }
        self.deep_chest(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_and_mod_structures_load() {
        let base = include_str!("../../../packs/vanilla/blocks.ron");
        let mods = include_str!("../../../packs/vanilla/mods/blocks.ron");
        let reg = BlockRegistry::from_ron_many(&[base, mods]).unwrap();
        let s = Structures::from_ron_many(
            &[include_str!("../../../packs/vanilla/structures.ron"), include_str!("../../../packs/vanilla/mods/structures.ron")],
            &reg,
        )
        .unwrap();
        let mut names = std::collections::HashSet::new();
        for st in &s.list {
            assert!(names.insert(st.def.name.clone()), "two structures called {}", st.def.name);
            assert!(st.size.x <= 64 && st.size.z <= 64 && st.size.y <= 80, "{} is too big", st.def.name);
        }
    }

    #[test]
    fn a_lone_bed_becomes_a_whole_bed_and_turns() {
        let reg = BlockRegistry::from_ron(include_str!("../../../packs/vanilla/blocks.ron")).unwrap();
        let text = r###"(structures: [(name: "t", biomes: [], spacing: 4, palette: [('b', "red_bed"), ('#', "stone")], layers: [["b.", "##"]])])"###;
        let st = &Structures::from_ron(text, &reg).unwrap().list[0];
        let red = reg.id("red_bed").unwrap();
        // North is outside, so the head goes east, into the air.
        assert_eq!(st.cell(0, 0, 0, 0), Some(reg.variant(red, 1)));
        assert_eq!(st.cell(0, 1, 0, 0), Some(reg.variant(red, 1 | crate::bed::HEAD)));
        // A quarter turn: the foot at (1, 0) points south to its head at (1, 1).
        assert_eq!(st.cell(1, 1, 0, 0), Some(reg.variant(red, 2)));
        assert_eq!(st.cell(1, 1, 0, 1), Some(reg.variant(red, 2 | crate::bed::HEAD)));
    }

    #[test]
    fn turning_keeps_every_cell() {
        let reg = BlockRegistry::from_ron(include_str!("../../../packs/vanilla/blocks.ron")).unwrap();
        let text = r##"(structures: [(name: "t", biomes: [], spacing: 4, palette: [('#', "stone"), ('c', "cobblestone")], layers: [["#c.", "..."]])])"##;
        let s = Structures::from_ron(text, &reg).unwrap();
        let st = &s.list[0];
        let stone = reg.id("stone");
        let cob = reg.id("cobblestone");
        // 3 wide (x) by 2 deep (z); one turn makes it 2 wide by 3 deep.
        assert_eq!(st.cell(0, 0, 0, 0), stone);
        assert_eq!(st.cell(0, 1, 0, 0), cob);
        assert_eq!(st.footprint(1), IVec2::new(2, 3));
        // (x, z) -> (sz - 1 - z, x): stone (0,0) -> (1,0), cobble (1,0) -> (1,1).
        assert_eq!(st.cell(1, 1, 0, 0), stone);
        assert_eq!(st.cell(1, 1, 0, 1), cob);
        for k in 0..4u8 {
            let f = st.footprint(k);
            let n = (0..f.x).flat_map(|x| (0..f.y).map(move |z| (x, z))).filter(|&(x, z)| st.cell(k, x, 0, z).is_some()).count();
            assert_eq!(n, 6, "turn {k}");
        }
    }
}

