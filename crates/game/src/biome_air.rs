//! Each biome's own sky and haze (biome pass, 2026-11): the desert's pale dusty horizon, the
//! swamp's green murk, the pale garden's grey sky, the ash-brown air round a volcano, the thin
//! clear air of the peaks. `Biome::air` holds the colours; here they are sampled round the
//! player, averaged and eased in, so walking from one biome into another the sky changes
//! slowly over a few seconds instead of jumping at the border.

use std::time::Instant;

use voxel::worldgen::{BiomeAir, Dim};

use crate::Game;

/// Columns sampled round the player (a 5×5 grid this far apart).
const STEP: i32 = 12;
/// Seconds between samples.
const EVERY: f32 = 0.4;
/// How fast the air eases toward the sampled one (per second).
const EASE: f32 = 0.6;

pub struct AirState {
    pub cur: BiomeAir,
    target: BiomeAir,
    t: f32,
    last: Option<Instant>,
    first: bool,
}

impl Default for AirState {
    fn default() -> Self {
        let plain = voxel::worldgen::Biome::Plains.air();
        AirState { cur: plain, target: plain, t: 0.0, last: None, first: true }
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

impl Game {
    /// Samples the biomes round the player and eases the air toward them.
    pub(crate) fn update_biome_air(&mut self) {
        let now = Instant::now();
        let dt = self.biome_air.last.map(|l| now.duration_since(l).as_secs_f32()).unwrap_or(0.0).min(0.5);
        self.biome_air.last = Some(now);
        if self.dim != Dim::Overworld || !self.world_open {
            return;
        }
        self.biome_air.t -= dt;
        if self.biome_air.t <= 0.0 || self.biome_air.first {
            self.biome_air.t = EVERY;
            let p = self.body.pos.floor().as_ivec3();
            let (mut top, mut hor, mut amount, mut haze, mut n) = ([0.0f32; 3], [0.0f32; 3], 0.0, 0.0, 0.0);
            for dz in -2..=2 {
                for dx in -2..=2 {
                    let a = self.world.gen.biome_at(p.x + dx * STEP, p.z + dz * STEP).air();
                    // Closer columns count more.
                    let w = 1.0 / (1.0 + (dx * dx + dz * dz) as f32 * 0.35);
                    // A plain sky's colour shouldn't drag a strong neighbour's tint: weigh the
                    // colours by how much each biome tints.
                    for i in 0..3 {
                        top[i] += a.top[i] * w * a.amount;
                        hor[i] += a.horizon[i] * w * a.amount;
                    }
                    amount += a.amount * w;
                    haze += a.haze * w;
                    n += w;
                }
            }
            let plain = voxel::worldgen::Biome::Plains.air();
            let (top, hor) = if amount > 1e-4 { (top.map(|c| c / amount), hor.map(|c| c / amount)) } else { (plain.top, plain.horizon) };
            self.biome_air.target = BiomeAir { top, horizon: hor, amount: amount / n, haze: haze / n };
            if self.biome_air.first {
                self.biome_air.first = false;
                self.biome_air.cur = self.biome_air.target;
            }
        }
        let k = (dt * EASE).min(1.0);
        let (c, t) = (self.biome_air.cur, self.biome_air.target);
        self.biome_air.cur = BiomeAir {
            top: lerp3(c.top, t.top, k),
            horizon: lerp3(c.horizon, t.horizon, k),
            amount: c.amount + (t.amount - c.amount) * k,
            haze: c.haze + (t.haze - c.haze) * k,
        };
    }

    /// Leans the day sky toward the biome's colours (nights stay dark: the tint fades with the
    /// daylight).
    pub(crate) fn biome_sky(&self, top: &mut [f32; 3], horizon: &mut [f32; 3], daylight: f32) {
        if self.dim != Dim::Overworld || self.body.pos.y < 0.0 {
            return;
        }
        let a = self.biome_air.cur;
        let k = a.amount * daylight;
        if k <= 0.001 {
            return;
        }
        *top = lerp3(*top, a.top, k);
        *horizon = lerp3(*horizon, a.horizon, k);
    }

    /// Brings the haze nearer in murky biomes and pushes it out in clear ones.
    pub(crate) fn biome_haze(&self, start: f32, end: f32) -> (f32, f32) {
        if self.dim != Dim::Overworld || self.body.pos.y < 0.0 {
            return (start, end);
        }
        let h = self.biome_air.cur.haze;
        if h >= 1.0 {
            // Clear air: only the start moves out (the end is the edge of the world drawn).
            return ((start * h).min(end - 8.0), end);
        }
        (start * h, end - (end - end * h) * 0.6)
    }
}
