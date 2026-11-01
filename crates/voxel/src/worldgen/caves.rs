//! Cave biomes (YUNG's Cave Biomes idea): below the surface, large regions of the cave system
//! take on a look of their own. Lush caves grow moss on the floors and glowing vines from the
//! ceilings; frozen caves are lined with packed ice and snow; dripstone caves bristle with
//! stone spikes from the floor and the roof. Frozen caves also get frozen pools, blue ice and
//! icicles; lush caves get clay-bottomed pools and moss carpet; dripstone caves get columns
//! joining floor and roof, and shallow pools. Deep down, the deep dark: sculk over deepslate,
//! with sensors, shriekers and catalysts (Phase 6). Sugar cane grows on the shores here too.

use super::{Column, Generator};
use crate::block::{BlockId, BlockRegistry, AIR};
use crate::chunk::{Chunk, ChunkPos, CHUNK_HEIGHT};
use crate::rng::Rng;
use crate::noise::Perlin;
use super::SEA_LEVEL;

/// Noise for the shapes of the upper caves (above y 0): noodles, ravines, entrances.
#[derive(Clone)]
pub(super) struct CaveNoise {
    noodle_a: Perlin,
    noodle_b: Perlin,
    ravine: Perlin,
    ravine_mask: Perlin,
    ravine_depth: Perlin,
    entrance: Perlin,
}

impl CaveNoise {
    pub(super) fn new(seed: u64) -> Self {
        let p = |k: u64| Perlin::new(seed ^ 0xCA7E_0000 ^ k.wrapping_mul(0x9E37_79B9));
        CaveNoise { noodle_a: p(1), noodle_b: p(2), ravine: p(3), ravine_mask: p(4), ravine_depth: p(5), entrance: p(6) }
    }
}

/// What the upper caves do in one column, worked out once per column.
#[derive(Clone, Copy, Debug)]
pub(super) struct CaveCol {
    /// A ravine cuts this column open from the surface down to here (`i32::MAX`: no ravine).
    pub ravine_bottom: i32,
    /// Tunnels here may break through the surface (a cave mouth).
    pub entrance: bool,
}

/// Ravines and cave mouths stay off low ground, so seas and rivers don't pour in.
const DRY: i32 = SEA_LEVEL + 3;

impl Generator {
    /// Ravines and cave mouths of a column.
    pub(super) fn cave_col(&self, wx: i32, wz: i32, h: i32) -> CaveCol {
        let n = &self.cave_noise;
        let (x, z) = (wx as f32, wz as f32);
        let mut cc = CaveCol { ravine_bottom: i32::MAX, entrance: false };
        if h <= DRY {
            return cc;
        }
        cc.entrance = n.entrance.noise2(x / 150.0, z / 150.0) > 0.3;
        // Ravines: long narrow cuts along the zero line of a noise field, in some regions only.
        // The middle of the cut goes deepest, so the walls slope in like a real canyon.
        let mask = n.ravine_mask.noise2(x / 700.0, z / 700.0);
        if mask > 0.2 {
            // A little roughness on the line so the walls aren't ruled straight.
            let r = n.ravine.noise2(x / 320.0, z / 320.0).abs() + 0.003 * n.ravine_depth.noise2(x / 7.0 + 90.0, z / 7.0);
            let w = 0.022 + 0.016 * ((mask - 0.2) * 3.0).min(1.0);
            if r < w {
                // Near low ground (where water would run in) the cut grows shallow and narrow
                // and closes, instead of stopping at a flat wall.
                let low = [(10, 0), (-10, 0), (0, 10), (0, -10), (18, 0), (-18, 0), (0, 18), (0, -18)].iter().map(|&(dx, dz)| self.height_at(wx + dx, wz + dz)).min().unwrap_or(h);
                let fade = ((low - DRY) as f32 / 10.0).clamp(0.0, 1.0);
                let c = 1.0 - r / (w * fade.max(0.01));
                if c > 0.0 {
                    let depth = (42.0 + 24.0 * n.ravine_depth.noise2(x / 90.0, z / 90.0)) * fade;
                    let cut = depth * c.powf(0.6);
                    if cut >= 3.0 {
                        cc.ravine_bottom = (h - cut as i32).max(6);
                    }
                }
            }
        }
        cc
    }

    /// Is this block carved out by the upper caves? `h` is the column's surface.
    #[inline]
    pub(super) fn is_cave(&self, cc: &CaveCol, wx: i32, y: i32, wz: i32, h: i32) -> bool {
        if y < 1 {
            return false;
        }
        if y >= cc.ravine_bottom - 2 && y <= h {
            // Ledges: the walls step in and out a little as they go down.
            let j = self.cave_noise.ravine_depth.noise3(wx as f32 / 6.0, y as f32 / 5.0, wz as f32 / 6.0) * 4.0;
            if y as f32 >= cc.ravine_bottom as f32 + j.max(-2.0) || y == h {
                return true;
            }
        }
        // Tunnels stay under a roof, unless this is a cave mouth.
        let roof = if cc.entrance { h + 1 } else { h - 3 };
        if y > roof {
            return false;
        }
        let (x, yf, z) = (wx as f32, y as f32, wz as f32);
        // Spaghetti caves: where two noise fields both cross zero we get long winding tunnels.
        // At a cave mouth the tunnels flare wider as they come up, so they open to the sky.
        let t = if cc.entrance && y > h - 14 { 0.06 + 0.035 * (y - (h - 14)) as f32 / 14.0 } else { 0.06 };
        let a = self.cave_a.noise3(x / 48.0, yf / 32.0, z / 48.0);
        if a.abs() < t {
            let b = self.cave_b.noise3(x / 48.0, yf / 32.0, z / 48.0);
            if b.abs() < t {
                return true;
            }
        }
        // Noodle caves: thin twisting passages, one or two blocks across, that link the rest.
        let n = &self.cave_noise;
        let na = n.noodle_a.noise3(x / 22.0, yf / 16.0, z / 22.0);
        if na.abs() < 0.035 {
            let nb = n.noodle_b.noise3(x / 22.0, yf / 16.0, z / 22.0);
            if nb.abs() < 0.035 && y < h - 5 {
                return true;
            }
        }
        // Cheese caves: big open rooms deeper down.
        if y < 48 && y <= h - 3 {
            let c = self.cheese.noise3(x / 70.0, yf / 40.0, z / 70.0);
            let depth_bias = (48 - y) as f32 / 120.0;
            if c > 0.62 - depth_bias * 0.4 {
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
pub(super) struct CavePalette {
    stone: BlockId,
    moss: BlockId,
    vines: BlockId,
    packed_ice: BlockId,
    snow_layer: BlockId,
    dripstone: BlockId,
    spike_up: BlockId,
    spike_down: BlockId,
    grass: BlockId,
    sculk: BlockId,
    deepslate: BlockId,
    sensor: BlockId,
    shrieker: BlockId,
    catalyst: BlockId,
    vein: BlockId,
    sugar_cane: BlockId,
    sand: BlockId,
    grass_block: BlockId,
    dirt: BlockId,
    water: BlockId,
    lava: BlockId,
    ice: BlockId,
    blue_ice: BlockId,
    icicle: BlockId,
    clay: BlockId,
    carpet: BlockId,
    small_dripleaf: BlockId,
    big_dripleaf: BlockId,
    azalea: BlockId,
    flowering_azalea: BlockId,
    spore_blossom: BlockId,
    hanging_roots: BlockId,
    rooted_dirt: BlockId,
}

impl CavePalette {
    pub(super) fn new(r: &BlockRegistry) -> Self {
        let g = |n: &str| r.id_or_air(n);
        CavePalette {
            stone: g("stone"),
            moss: g("moss_block"),
            vines: g("cave_vines"),
            packed_ice: g("packed_ice"),
            snow_layer: g("snow_layer"),
            dripstone: g("dripstone_block"),
            spike_up: g("pointed_dripstone"),
            spike_down: g("pointed_dripstone_down"),
            grass: g("short_grass"),
            sculk: g("sculk"),
            deepslate: g("deepslate"),
            sensor: g("sculk_sensor"),
            shrieker: g("sculk_shrieker"),
            catalyst: g("sculk_catalyst"),
            vein: g("sculk_vein"),
            sugar_cane: g("sugar_cane"),
            sand: g("sand"),
            grass_block: g("grass_block"),
            dirt: g("dirt"),
            water: g("water"),
            lava: g("lava"),
            ice: g("ice"),
            blue_ice: g("blue_ice"),
            icicle: g("icicle"),
            clay: g("clay"),
            carpet: g("moss_carpet"),
            small_dripleaf: g("small_dripleaf"),
            big_dripleaf: g("big_dripleaf"),
            azalea: g("azalea"),
            flowering_azalea: g("flowering_azalea"),
            spore_blossom: g("spore_blossom"),
            hanging_roots: g("hanging_roots"),
            rooted_dirt: g("rooted_dirt"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaveBiome {
    Plain,
    Lush,
    Frozen,
    Dripstone,
    /// The deep dark: only below `DEEP_TOP`, sculk and deepslate.
    DeepDark,
}

/// The deep dark stays in the lowest caves.
const DEEP_TOP: usize = 30;

/// The highest a cave biome reaches: it stays well under the surface.
const TOP: usize = 56;

impl Generator {
    /// Which cave biome is under this column.
    pub fn cave_biome(&self, wx: i32, wz: i32) -> CaveBiome {
        let n = self.detail.fbm2(wx as f32 / 190.0 + 311.0, wz as f32 / 190.0 - 77.0, 3);
        let m = self.detail.fbm2(wx as f32 / 230.0 - 905.0, wz as f32 / 230.0 + 41.0, 2);
        let d = self.detail.fbm2(wx as f32 / 260.0 + 1703.0, wz as f32 / 260.0 - 2207.0, 2);
        if d > 0.33 && self.caves.sculk != AIR {
            CaveBiome::DeepDark
        } else if n > 0.28 {
            CaveBiome::Lush
        } else if n < -0.3 {
            CaveBiome::Frozen
        } else if m > 0.3 {
            CaveBiome::Dripstone
        } else {
            CaveBiome::Plain
        }
    }

    /// Sugar cane, one to three tall, on sand, grass or dirt right beside water at the surface.
    fn shore_cane(&self, chunk: &mut Chunk, rng: &mut Rng, cols: &[Column]) {
        let p = &self.caves;
        if p.sugar_cane == AIR {
            return;
        }
        for z in 1..15usize {
            for x in 1..15usize {
                let y = cols[z * 16 + x].height;
                if y < 1 || y + 4 >= CHUNK_HEIGHT as i32 {
                    continue;
                }
                let (uy, above) = (y as usize, y as usize + 1);
                let ground = chunk.get(x, uy, z);
                if ground != p.sand && ground != p.grass_block && ground != p.dirt || chunk.get(x, above, z) != AIR {
                    continue;
                }
                let wet = [(1, 0), (0, 1)].iter().any(|&(dx, dz)| chunk.get(x + dx, uy, z + dz) == p.water || chunk.get(x - dx, uy, z - dz) == p.water);
                if wet && rng.chance(0.18) {
                    let h = 1 + rng.below(3) as usize;
                    for k in 0..h {
                        if chunk.get(x, above + k, z) != AIR {
                            break;
                        }
                        chunk.set(x, above + k, z, p.sugar_cane);
                    }
                }
            }
        }
    }

    /// Ordinary caves get still pools in hollows of their floors: water, and lava low down
    /// (under y 12, like the old lava lakes). Only cells walled in on all four sides, so nothing
    /// runs off.
    fn plain_pools(&self, chunk: &mut Chunk, x: usize, z: usize, surface: i32, bx: i32, bz: i32) {
        let p = &self.caves;
        if !(1..15).contains(&x) || !(1..15).contains(&z) || p.water == AIR {
            return;
        }
        let (wx, wz) = (bx + x as i32, bz + z as i32);
        let patch = self.detail.fbm2(wx as f32 / 9.0 - 731.0, wz as f32 / 9.0 + 219.0, 2);
        if patch < 0.22 {
            return;
        }
        let top = (surface - 8).clamp(0, 48) as usize;
        for y in 2..top {
            if chunk.get(x, y, z) != AIR || chunk.get(x, y + 1, z) != AIR {
                continue;
            }
            let below = chunk.get(x, y - 1, z);
            if below == AIR || below == p.water || below == p.lava {
                continue;
            }
            let walled = [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)].iter().all(|&(a, b)| chunk.get(a, y - 1, b) != AIR);
            if walled {
                // The floor block itself becomes liquid, one deep, with rock under it.
                let liquid = if y < 12 && p.lava != AIR { p.lava } else { p.water };
                chunk.set(x, y - 1, z, liquid);
                if chunk.get(x, y - 2, z) == AIR {
                    chunk.set(x, y - 2, z, p.stone);
                }
            }
        }
    }

    pub(super) fn decorate_caves(&self, chunk: &mut Chunk, rng: &mut Rng, cols: &[Column], pos: ChunkPos) {
        let p = &self.caves;
        if p.moss == AIR {
            return;
        }
        let (bx, bz) = (pos.x * 16, pos.y * 16);
        self.shore_cane(chunk, rng, cols);
        for z in 0..16usize {
            for x in 0..16usize {
                let kind = self.cave_biome(bx + x as i32, bz + z as i32);
                if kind == CaveBiome::Plain {
                    self.plain_pools(chunk, x, z, cols[z * 16 + x].height, bx, bz);
                    continue;
                }
                let surface = cols[z * 16 + x].height.max(0) as usize;
                let top = if kind == CaveBiome::DeepDark { DEEP_TOP } else { TOP };
                let top = top.min(surface.saturating_sub(8)).min(CHUNK_HEIGHT - 2);
                for y in 12..top {
                    let here = chunk.get(x, y, z);
                    if here != p.stone {
                        continue;
                    }
                    let up = chunk.get(x, y + 1, z);
                    let down = chunk.get(x, y - 1, z);
                    let floor = up == AIR;
                    let ceiling = down == AIR;
                    if !floor && !ceiling {
                        continue;
                    }
                    // Pools: a floor cell walled in on all four sides (so the water stays put),
                    // in patches picked by noise.
                    let (wx, wz) = (bx + x as i32, bz + z as i32);
                    let walled = (1..15).contains(&x)
                        && (1..15).contains(&z)
                        && [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)].iter().all(|&(a, b)| chunk.get(a, y, b) != AIR);
                    let patch = self.detail.fbm2(wx as f32 / 7.0 + 51.0, wz as f32 / 7.0 - 13.0, 2);
                    let pool = floor && !ceiling && walled && patch > 0.12 && p.water != AIR;
                    match kind {
                        CaveBiome::Lush => {
                            if pool {
                                chunk.set(x, y, z, p.water);
                                if p.clay != AIR && chunk.get(x, y - 1, z) == p.stone {
                                    chunk.set(x, y - 1, z, p.clay);
                                }
                                continue;
                            }
                            chunk.set(x, y, z, p.moss);
                            if floor {
                                // Big dripleaves lean over the pools; small ones, azalea bushes,
                                // grass and moss carpet cover the rest.
                                let by_water = (1..15).contains(&x) && (1..15).contains(&z) && [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)].iter().any(|&(a, b)| chunk.get(a, y, b) == p.water);
                                let r = rng.f32();
                                let up = if by_water && r < 0.45 {
                                    p.big_dripleaf
                                } else if r < 0.16 {
                                    p.grass
                                } else if r < 0.30 {
                                    p.carpet
                                } else if r < 0.36 {
                                    p.small_dripleaf
                                } else if r < 0.39 {
                                    p.azalea
                                } else if r < 0.41 {
                                    p.flowering_azalea
                                } else {
                                    AIR
                                };
                                if up != AIR {
                                    chunk.set(x, y + 1, z, up);
                                }
                            }
                            if ceiling && rng.chance(0.05) && p.hanging_roots != AIR {
                                // Roots of the trees above come through the roof.
                                if p.rooted_dirt != AIR {
                                    chunk.set(x, y, z, p.rooted_dirt);
                                }
                                chunk.set(x, y - 1, z, p.hanging_roots);
                            } else if ceiling && rng.chance(0.015) && p.spore_blossom != AIR {
                                chunk.set(x, y - 1, z, p.spore_blossom);
                            } else if ceiling && rng.chance(0.2) && p.vines != AIR {
                                let len = 1 + rng.below(4) as usize;
                                for k in 1..=len {
                                    if y < k + 1 || chunk.get(x, y - k, z) != AIR || chunk.get(x, y - k - 1, z) != AIR {
                                        break;
                                    }
                                    chunk.set(x, y - k, z, p.vines);
                                }
                            }
                        }
                        CaveBiome::Frozen => {
                            // Frozen pools: smooth ice sheets on the floor, without snow.
                            if floor && patch > 0.05 && p.ice != AIR {
                                chunk.set(x, y, z, p.ice);
                                continue;
                            }
                            let blue = p.blue_ice != AIR && rng.chance(0.07);
                            chunk.set(x, y, z, if blue { p.blue_ice } else { p.packed_ice });
                            if floor && rng.chance(0.6) && p.snow_layer != AIR {
                                chunk.set(x, y + 1, z, p.snow_layer);
                            }
                            if ceiling && p.icicle != AIR && y > 2 && chunk.get(x, y - 2, z) == AIR && rng.chance(0.14) {
                                chunk.set(x, y - 1, z, p.icicle);
                            }
                        }
                        CaveBiome::Dripstone => {
                            if pool && patch > 0.25 {
                                chunk.set(x, y, z, p.water);
                                chunk.set(x, y - 1, z, p.dripstone);
                                continue;
                            }
                            if rng.chance(0.7) {
                                chunk.set(x, y, z, p.dripstone);
                            }
                            // Columns: where the roof is low, floor and roof grow together.
                            let gap = if floor { (1..8).find(|&k| y + k < CHUNK_HEIGHT && chunk.get(x, y + k, z) != AIR) } else { None };
                            if let Some(g) = gap.filter(|&g| g >= 3 && rng.chance(0.06)) {
                                for k in 1..g {
                                    chunk.set(x, y + k, z, p.dripstone);
                                }
                                continue;
                            }
                            if floor && rng.chance(0.1) {
                                let h = 1 + rng.below(3) as usize;
                                for k in 1..=h {
                                    if chunk.get(x, y + k, z) != AIR || chunk.get(x, y + k + 1, z) != AIR {
                                        break;
                                    }
                                    chunk.set(x, y + k, z, p.spike_up);
                                }
                            } else if ceiling && rng.chance(0.12) {
                                let h = 1 + rng.below(3) as usize;
                                for k in 1..=h {
                                    if y < k + 1 || chunk.get(x, y - k, z) != AIR || chunk.get(x, y - k - 1, z) != AIR {
                                        break;
                                    }
                                    chunk.set(x, y - k, z, p.spike_down);
                                }
                            }
                        }
                        CaveBiome::DeepDark => {
                            if !floor {
                                chunk.set(x, y, z, p.deepslate);
                                if ceiling && rng.chance(0.3) && p.vein != AIR {
                                    chunk.set(x, y, z, p.sculk);
                                }
                                continue;
                            }
                            chunk.set(x, y, z, if rng.chance(0.85) { p.sculk } else { p.deepslate });
                            let r = rng.f32();
                            let up = if r < 0.012 {
                                p.sensor
                            } else if r < 0.016 {
                                p.shrieker
                            } else if r < 0.0175 {
                                p.catalyst
                            } else if r < 0.08 {
                                p.vein
                            } else {
                                AIR
                            };
                            if up != AIR {
                                chunk.set(x, y + 1, z, up);
                            }
                        }
                        CaveBiome::Plain => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkPos;

    fn registry() -> BlockRegistry {
        BlockRegistry::from_ron_many(&[include_str!("../../../../packs/vanilla/blocks.ron"), include_str!("../../../../packs/vanilla/mods/blocks.ron")]).unwrap()
    }

    #[test]
    fn cave_biomes_get_their_features() {
        let reg = registry();
        let g = Generator::new(1234, &reg);
        let want = ["blue_ice", "icicle", "moss_carpet", "clay", "water", "big_dripleaf", "small_dripleaf", "azalea", "spore_blossom", "hanging_roots"].map(|n| reg.id(n).unwrap());
        let mut seen = [0u32; 10];
        let mut kinds = [0u32; 4];
        for cz in -40..40 {
            for cx in -40..40 {
                let k = g.cave_biome(cx * 16 + 8, cz * 16 + 8);
                if k == CaveBiome::Plain || kinds[k as usize] >= 12 {
                    continue;
                }
                kinds[k as usize] += 1;
                let c = g.generate(ChunkPos::new(cx, cz));
                for y in 12..TOP {
                    for z in 0..16 {
                        for x in 0..16 {
                            let id = c.get(x, y, z);
                            if let Some(i) = want.iter().position(|&w| w == id) {
                                seen[i] += 1;
                            }
                        }
                    }
                }
            }
        }
        println!("blue_ice, icicle, carpet, clay, water, big/small dripleaf, azalea, spore blossom, roots: {seen:?}");
        assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
    }

    /// The upper caves open to the sky now and then, ravines cut the land, and caves hold
    /// still pools of water and (low down) lava.
    #[test]
    fn mouths_ravines_and_pools() {
        let reg = registry();
        let g = Generator::new(12345, &reg);
        let (water, lava) = (reg.id("water").unwrap(), reg.id("lava").unwrap());
        let (mut rav, mut ent) = (0, 0);
        for z in (-2000..2000).step_by(8) {
            for x in (-2000..2000).step_by(8) {
                let h = g.height_at(x, z);
                let cc = g.cave_col(x, z, h);
                rav += (cc.ravine_bottom != i32::MAX) as u32;
                ent += cc.entrance as u32;
            }
        }
        assert!(rav > 100 && ent > 1000, "ravines {rav}, mouth regions {ent}");
        let (mut open_top, mut pools, mut lavas) = (0, 0, 0);
        for cz in 0..10 {
            for cx in 0..10 {
                let c = g.generate(ChunkPos::new(cx * 3, cz * 3));
                for z in 0..16 {
                    for x in 0..16 {
                        let h = g.height_at(cx * 48 + x as i32, cz * 48 + z as i32);
                        if h > super::SEA_LEVEL + 3 && h < 250 && c.get(x, h as usize, z) == AIR && c.get(x, h as usize - 3, z) == AIR {
                            open_top += 1;
                        }
                        for y in 1..48 {
                            let b = c.get(x, y, z);
                            pools += (b == water && y < super::SEA_LEVEL as usize - 4) as u32;
                            lavas += (b == lava) as u32;
                        }
                    }
                }
            }
        }
        println!("ravines {rav}, mouth regions {ent}, open tops {open_top}, cave water {pools}, lava {lavas}");
        assert!(open_top > 20 && pools > 50 && lavas > 0, "open tops {open_top}, water {pools}, lava {lavas}");
    }
}

#[cfg(test)]
mod probe {
    use super::*;
    use crate::chunk::{ChunkPos, MIN_Y};

    /// Cave statistics over a patch of world (run with --ignored --nocapture).
    #[test]
    #[ignore]
    fn cave_probe() {
        let reg = BlockRegistry::from_ron_many(&[include_str!("../../../../packs/vanilla/blocks.ron"), include_str!("../../../../packs/vanilla/mods/blocks.ron"), include_str!("../../../../packs/vanilla/deep/blocks.ron")]).unwrap();
        let g = Generator::new(12345, &reg);
        let (water, lava) = (reg.id("water").unwrap(), reg.id("lava").unwrap());
        let n = 12;
        let mut air = vec![0u64; 1024 / 16];
        let mut wat = vec![0u64; 1024 / 16];
        let mut lav = vec![0u64; 1024 / 16];
        let mut entrances = 0u64;
        let mut cols = 0u64;
        let t = std::time::Instant::now();
        for cz in 0..n {
            for cx in 0..n {
                let c = g.generate_full(ChunkPos::new(cx * 3, cz * 3));
                for z in 0..16 {
                    for x in 0..16 {
                        let h = g.height_at(cx * 48 + x as i32, cz * 48 + z as i32);
                        cols += 1;
                        // an entrance: open air from the surface down at least 6 blocks below it
                        if h > SEA && h < 250 && c.get_w(x, h, z) == AIR && c.get_w(x, h - 3, z) == AIR {
                            entrances += 1;
                        }
                        for y in MIN_Y..h.min(255) - 2 {
                            let b = c.get_w(x, y, z);
                            let i = ((y - MIN_Y) / 16) as usize;
                            if b == AIR {
                                air[i] += 1;
                            } else if b == water {
                                wat[i] += 1;
                            } else if b == lava {
                                lav[i] += 1;
                            }
                        }
                    }
                }
            }
        }
        let per = (n * n) as f32;
        println!("gen {:.1} ms/chunk (full depth)", t.elapsed().as_secs_f32() * 1000.0 / per);
        println!("entrance columns {entrances} of {cols}");
        let (mut rav, mut ent, mut tot) = (0, 0, 0);
        for z in (-3000..3000).step_by(5) {
            for x in (-3000..3000).step_by(5) {
                let h = g.height_at(x, z);
                let cc = g.cave_col(x, z, h);
                tot += 1;
                rav += (cc.ravine_bottom != i32::MAX) as u32;
                ent += cc.entrance as u32;
            }
        }
        println!("sampled {tot}: ravine {rav} ({:.2}%), entrance region {ent} ({:.1}%)", rav as f32 / tot as f32 * 100.0, ent as f32 / tot as f32 * 100.0);
        for i in (0..air.len()).rev() {
            let y = i as i32 * 16 + MIN_Y;
            if y > 120 {
                continue;
            }
            let v = 256.0 * 16.0 * per;
            println!("y {:5}: air {:5.1}%  water {:4.1}%  lava {:4.1}%", y, air[i] as f32 / v * 100.0, wat[i] as f32 / v * 100.0, lav[i] as f32 / v * 100.0);
        }
    }
    const SEA: i32 = super::super::SEA_LEVEL;

    /// Prints the lush cave chunks with the most plants (for screenshots).
    #[test]
    #[ignore]
    fn find_lush() {
        let reg = BlockRegistry::from_ron_many(&[include_str!("../../../../packs/vanilla/blocks.ron"), include_str!("../../../../packs/vanilla/mods/blocks.ron")]).unwrap();
        let g = Generator::new(12345, &reg);
        let want = ["big_dripleaf", "spore_blossom", "azalea", "flowering_azalea", "small_dripleaf"].map(|n| reg.id(n).unwrap());
        let mut best = Vec::new();
        for cz in -25..25 {
            for cx in -25..25 {
                if g.cave_biome(cx * 16 + 8, cz * 16 + 8) != CaveBiome::Lush {
                    continue;
                }
                let c = g.generate(ChunkPos::new(cx, cz));
                let (mut n, mut air, mut at) = (0, 0, (0, 0, 0));
                for y in 12..56 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let b = c.get(x, y, z);
                            if b == AIR && (y as i32) < g.height_at(cx * 16 + x as i32, cz * 16 + z as i32) - 4 {
                                air += 1;
                            }
                            if want.contains(&b) {
                                n += 1;
                                if b == want[0] {
                                    at = (cx * 16 + x as i32, y as i32, cz * 16 + z as i32);
                                }
                            }
                        }
                    }
                }
                best.push((n * 10 + air / 20, n, air, at));
            }
        }
        best.sort();
        for b in best.iter().rev().take(5) {
            println!("plants {} air {} dripleaf at {:?}", b.1, b.2, b.3);
        }
    }

    /// Prints a ravine and some cave mouths near spawn (for screenshots).
    #[test]
    #[ignore]
    fn find_cave_features() {
        let reg = BlockRegistry::from_ron_many(&[include_str!("../../../../packs/vanilla/blocks.ron"), include_str!("../../../../packs/vanilla/mods/blocks.ron")]).unwrap();
        let g = Generator::new(12345, &reg);
        let mut best = (i32::MAX, 0, 0, 0);
        let mut mouths = 0;
        for r in (0..1200).step_by(4) {
            for (x, z) in (-r..=r).step_by(4).flat_map(|k| [(k, -r), (k, r), (-r, k), (r, k)]) {
                let h = g.height_at(x, z);
                let cc = g.cave_col(x, z, h);
                if cc.ravine_bottom != i32::MAX && h - cc.ravine_bottom > 40 && best.1 < 4 {
                    best.1 += 1;
                    // Which way it runs: the neighbour that is also deep.
                    let dir = [(6, 0), (0, 6), (6, 6), (6, -6)].iter().copied().find(|&(dx, dz)| { let h2 = g.height_at(x + dx, z + dz); g.cave_col(x + dx, z + dz, h2).ravine_bottom < h2 - 20 });
                    println!("ravine at {x},{h},{z} bottom {} runs {dir:?}", cc.ravine_bottom);
                }
                if cc.entrance && mouths < 3 {
                    let c = g.generate(ChunkPos::new(x.div_euclid(16), z.div_euclid(16)));
                    let (lx, lz) = (x.rem_euclid(16) as usize, z.rem_euclid(16) as usize);
                    if c.get_w(lx, h, lz) == AIR && c.get_w(lx, h - 4, lz) == AIR && cc.ravine_bottom == i32::MAX {
                        println!("mouth at {x},{h},{z}");
                        mouths += 1;
                    }
                }
            }
        }
    }
}
