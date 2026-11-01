//! Every ore of the overworld in one table, for the surface and the depths alike
//! (plan/minerios-analise.md).
//!
//! - Each ore has a depth curve like Java 1.18+: a lowest, a best and a highest y, with the
//!   chance growing in a straight line towards the best height (a triangle), so "where do I
//!   dig for diamonds" has an answer. The best heights follow the mining ladder (ladder.rs,
//!   plan/progressao-linear.md): copper ~0, iron ~-20, silver ~-60, gold ~-160, diamond
//!   ~-300, so each new pickaxe sends you deeper. Java's surface bonuses stay (iron in the
//!   mountains, gold in the badlands, emeralds on the peaks).
//! - Veins are compact blobs (Java's shape), scattered crystals (gems, emeralds) or not.
//! - A vein may cross into the next chunk: every chunk also draws the veins that start in its
//!   eight neighbours, so no vein is ever cut in half at a chunk border.
//! - In deepslate and tuff an ore turns into its deepslate twin (`deepslate_<name>`).
//! - Each vein has its own random numbers, so a slice of the deep world gets exactly the
//!   ores a whole generation would (the lazy depths stay seamless).

use super::Biome;
use crate::block::{BlockId, BlockRegistry, AIR};
use crate::rng::{hash3, Rng};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// Java's ore blob: spheres strung along a short line.
    Blob,
    /// Single crystals spread over a few blocks (gems, emeralds).
    Scatter,
}

#[derive(Clone, Debug)]
pub struct OreSpec {
    pub name: String,
    pub ore: BlockId,
    /// The deepslate twin (the ore itself when it has none).
    pub deep: BlockId,
    pub lo: i32,
    pub peak: i32,
    pub hi: i32,
    /// Veins started per chunk (the fraction is a chance of one more).
    pub count: f32,
    pub size: u32,
    pub shape: Shape,
    /// Only where the surface above is one of these (empty: anywhere).
    pub biomes: &'static [Biome],
}

const MOUNTAINS: &[Biome] = &[Biome::Mountains, Biome::SnowyPeaks];
const BADLANDS: &[Biome] = &[Biome::Badlands];
const ANY: &[Biome] = &[];

/// The metal pairs of the ore ladder (Terraria's idea): each world makes one of each pair
/// common and the other rare (`super::common_metal`), but both can be found.
/// (common, rare, lowest, best, highest y, veins per chunk, size)
pub const PAIRS: [(&str, &str, i32, i32, i32, f32, u32); 4] = [
    ("copper_ore", "tin_ore", -260, 0, 112, 16.0, 10),
    ("iron_ore", "lead_ore", -380, -20, 16, 20.0, 9),
    ("silver_ore", "tungsten_ore", -480, -60, -20, 10.0, 8),
    ("gold_ore", "platinum_ore", -600, -160, -60, 9.0, 8),
];

/// The rest: (ore, lowest, best, highest y, veins per chunk, size, shape, biomes).
#[allow(clippy::type_complexity)]
const SINGLES: &[(&str, i32, i32, i32, f32, u32, Shape, &[Biome])] = &[
    ("coal_ore", -120, 80, 200, 20.0, 14, Shape::Blob, ANY),
    // Java: lots of iron high up in the mountains, gold in the badlands, emeralds on the peaks.
    ("iron_ore", 80, 150, 255, 14.0, 9, Shape::Blob, MOUNTAINS),
    ("gold_ore", 32, 70, 160, 12.0, 9, Shape::Blob, BADLANDS),
    ("emerald_ore", 40, 130, 255, 8.0, 2, Shape::Scatter, MOUNTAINS),
    ("redstone_ore", -320, -80, -20, 10.0, 8, Shape::Blob, ANY),
    ("lapis_ore", -360, -30, 16, 5.0, 7, Shape::Blob, ANY),
    ("diamond_ore", -600, -300, -120, 6.0, 5, Shape::Blob, ANY),
    // Terraria's gems: amethyst nearest the top, ruby deepest.
    ("amethyst_ore", -420, -80, -30, 3.0, 4, Shape::Scatter, ANY),
    ("topaz_ore", -480, -140, -50, 3.0, 4, Shape::Scatter, ANY),
    ("sapphire_ore", -560, -230, -120, 2.5, 4, Shape::Scatter, ANY),
    ("ruby_ore", -590, -320, -160, 2.5, 4, Shape::Scatter, ANY),
];

/// How far a vein reaches from where it starts, at most (blocks).
const REACH: i32 = 5;

pub struct OreTable {
    pub specs: Vec<OreSpec>,
    /// Ore → deepslate twin, for every ore that has one (also hard mode and rare ones).
    twins: Vec<(BlockId, BlockId)>,
}

/// The deepslate twin of an ore, by name ("iron_ore" → "deepslate_iron_ore").
pub fn twin_name(ore: &str) -> String {
    format!("deepslate_{ore}")
}

/// Every ore that has a deepslate twin.
pub const WITH_TWIN: &[&str] = &[
    "coal_ore", "copper_ore", "tin_ore", "iron_ore", "lead_ore", "silver_ore", "tungsten_ore", "gold_ore", "platinum_ore", "redstone_ore", "lapis_ore",
    "diamond_ore", "emerald_ore", "amethyst_ore", "topaz_ore", "sapphire_ore", "ruby_ore", "cobalt_ore", "palladium_ore", "mythril_ore", "orichalcum_ore",
    "adamantite_ore", "titanium_ore",
];

impl OreTable {
    pub fn new(r: &BlockRegistry, seed: u64) -> Self {
        let g = |n: &str| r.id_or_air(n);
        let twins: Vec<(BlockId, BlockId)> = WITH_TWIN.iter().map(|n| (g(n), g(&twin_name(n)))).filter(|&(a, b)| a != AIR && b != AIR).collect();
        let twin = |o: BlockId| twins.iter().find(|&&(a, _)| a == o).map_or(o, |&(_, d)| d);
        let mut specs = Vec::new();
        let mut push = |name: &str, lo, peak, hi, count, size, shape, biomes| {
            let ore = g(name);
            if ore != AIR {
                specs.push(OreSpec { name: name.to_string(), ore, deep: twin(ore), lo, peak, hi, count, size, shape, biomes });
            }
        };
        for (i, &(a, b, lo, peak, hi, count, size)) in PAIRS.iter().enumerate() {
            let (common, rare) = if super::common_metal(seed, i) { (a, b) } else { (b, a) };
            push(common, lo, peak, hi, count, size, Shape::Blob, ANY);
            push(rare, lo, peak, hi, count * 0.3, size.saturating_sub(2).max(3), Shape::Blob, ANY);
        }
        for &(n, lo, peak, hi, count, size, shape, biomes) in SINGLES {
            push(n, lo, peak, hi, count, size, shape, biomes);
        }
        OreTable { specs, twins }
    }

    /// The deepslate twin of `ore` (itself when it has none).
    pub fn twin(&self, ore: BlockId) -> BlockId {
        self.twins.iter().find(|&&(a, _)| a == ore).map_or(ore, |&(_, d)| d)
    }

    /// Every ore block of chunk (`cx`, `cz`) with y in `ylo..yhi`, as (spec, local x, y, local z).
    /// Veins starting in the eight neighbours are drawn too. `biome` is asked only for the
    /// few ores tied to a surface biome.
    #[allow(clippy::too_many_arguments)]
    pub fn each(&self, seed: u64, cx: i32, cz: i32, ylo: i32, yhi: i32, biome: &dyn Fn(i32, i32) -> Biome, mut f: impl FnMut(&OreSpec, i32, i32, i32)) {
        let (bx, bz) = (cx * 16, cz * 16);
        let mut cells: Vec<(i32, i32, i32)> = Vec::with_capacity(64);
        for (k, spec) in self.specs.iter().enumerate() {
            if spec.hi + REACH < ylo || spec.lo - REACH >= yhi {
                continue;
            }
            for nz in cz - 1..=cz + 1 {
                for nx in cx - 1..=cx + 1 {
                    let mut rng = Rng::new(hash3(seed ^ 0x0E5E_7AB1 ^ (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15), nx as i64, nz as i64, spec.ore as i64));
                    let n = spec.count as u32 + rng.chance(spec.count.fract()) as u32;
                    for _ in 0..n {
                        let ox = nx * 16 + rng.below(16) as i32;
                        let oz = nz * 16 + rng.below(16) as i32;
                        let oy = triangle(&mut rng, spec.lo, spec.peak, spec.hi);
                        let vein = rng.next_u64();
                        if ox + REACH < bx || ox - REACH >= bx + 16 || oz + REACH < bz || oz - REACH >= bz + 16 || oy + REACH < ylo || oy - REACH >= yhi {
                            continue;
                        }
                        if !spec.biomes.is_empty() && !spec.biomes.contains(&biome(ox, oz)) {
                            continue;
                        }
                        cells.clear();
                        vein_cells(&mut Rng::new(vein), spec.shape, spec.size, ox, oy, oz, &mut cells);
                        for &(x, y, z) in &cells {
                            if (bx..bx + 16).contains(&x) && (bz..bz + 16).contains(&z) && (ylo..yhi).contains(&y) {
                                f(spec, x - bx, y, z - bz);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A height between `lo` and `hi`, most often near `peak`.
pub fn triangle(rng: &mut Rng, lo: i32, peak: i32, hi: i32) -> i32 {
    let (a, c, b) = (lo as f32, peak.clamp(lo, hi) as f32, hi as f32);
    if b <= a {
        return lo;
    }
    let u = rng.f32();
    let f = (c - a) / (b - a);
    let y = if u < f { a + (u * (b - a) * (c - a)).sqrt() } else { b - ((1.0 - u) * (b - a) * (b - c)).sqrt() };
    y.round() as i32
}

/// The blocks of one vein started at (`ox`, `oy`, `oz`), never more than `REACH` away.
pub fn vein_cells(rng: &mut Rng, shape: Shape, size: u32, ox: i32, oy: i32, oz: i32, out: &mut Vec<(i32, i32, i32)>) {
    match shape {
        Shape::Scatter => {
            for _ in 0..size.max(1) {
                let c = (ox + rng.range(-2, 2), oy + rng.range(-1, 1), oz + rng.range(-2, 2));
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        Shape::Blob => {
            // Java's ore feature, kept small: spheres along a short tilted line, fattest in
            // the middle.
            let s = size.max(2) as f32;
            let a = rng.f32() * std::f32::consts::PI;
            let len = (s / 8.0).min(2.5);
            let (sx, sz) = (a.sin() * len, a.cos() * len);
            let (x0, x1) = (ox as f32 + 0.5 + sx, ox as f32 + 0.5 - sx);
            let (z0, z1) = (oz as f32 + 0.5 + sz, oz as f32 + 0.5 - sz);
            let (y0, y1) = (oy as f32 + rng.range(-1, 1) as f32 + 0.5, oy as f32 + rng.range(-1, 1) as f32 + 0.5);
            let steps = size.max(2);
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let (cx, cy, cz) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, z0 + (z1 - z0) * t);
                let d = rng.f32() * s / 16.0;
                let r = (((std::f32::consts::PI * t).sin() + 1.0) * d + 1.0) / 2.0;
                let r = r.min(2.2);
                let r2 = r * r;
                for y in (cy - r).floor() as i32..=(cy + r).floor() as i32 {
                    for z in (cz - r).floor() as i32..=(cz + r).floor() as i32 {
                        for x in (cx - r).floor() as i32..=(cx + r).floor() as i32 {
                            let (dx, dy, dz) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy, z as f32 + 0.5 - cz);
                            if dx * dx + dy * dy + dz * dz < r2 && (x - ox).abs() <= REACH && (y - oy).abs() <= REACH && (z - oz).abs() <= REACH && !out.contains(&(x, y, z)) {
                                out.push((x, y, z));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_stays_inside_and_peaks() {
        let mut rng = Rng::new(5);
        let mut near = 0;
        for _ in 0..4000 {
            let y = triangle(&mut rng, -100, 20, 60);
            assert!((-100..=60).contains(&y));
            if (y - 20).abs() < 20 {
                near += 1;
            }
        }
        // A third of the range around the peak holds well over a third of the draws.
        assert!(near > 1600, "{near}");
    }

    #[test]
    fn blobs_are_compact_and_sized() {
        let mut total = 0;
        for s in 0..200u64 {
            let mut out = Vec::new();
            vein_cells(&mut Rng::new(s), Shape::Blob, 9, 0, 0, 0, &mut out);
            assert!(!out.is_empty());
            assert!(out.iter().all(|&(x, y, z)| x.abs() <= REACH && y.abs() <= REACH && z.abs() <= REACH));
            total += out.len();
        }
        let avg = total as f32 / 200.0;
        assert!((4.0..20.0).contains(&avg), "average blob of size 9 has {avg} blocks");
    }
}
