//! Real recorded sounds of the world: blocks broken, placed and mined, footsteps walking and
//! running on every kind of ground (sound_bank.rs, credits in assets/sounds/world/CREDITS.md).
//!
//! Every material has its own recordings for each action. A play picks one recording at
//! random and changes its speed and loudness a little, so a hundred steps on gravel never
//! sound like one sample repeated. Related materials share recordings at another speed
//! (deepslate is stone played slower and denser, soul sand is sand that sinks).
//!
//! hit_sounds.rs (bodies being hit, voices) and swing_sounds.rs (weapons and tools) use the
//! same bank through `render`.

use crate::audio::Material;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex, OnceLock};

pub const RATE: f32 = 44100.0;

/// One way a material sounds for an action: recordings to pick from (groups of the bank),
/// how fast to play them, how loud, how bright, and maybe a second recording laid on top.
#[derive(Clone, Copy)]
pub struct Act {
    pub pick: &'static [&'static str],
    pub rate: f32,
    pub gain: f32,
    /// Low-pass cutoff in Hz: below 3000 dull, 6000 open.
    pub lp: f32,
    pub extra: Option<(&'static str, f32, f32)>,
}

const fn act(pick: &'static [&'static str], rate: f32, gain: f32, lp: f32) -> Act {
    Act { pick, rate, gain, lp, extra: None }
}

const fn with(pick: &'static [&'static str], rate: f32, gain: f32, lp: f32, extra: (&'static str, f32, f32)) -> Act {
    Act { pick, rate, gain, lp, extra: Some(extra) }
}

/// The four sounds of a material.
pub struct Look {
    pub step: Act,
    pub hit: Act,
    pub brk: Act,
    pub place: Act,
}

/// How each material sounds. Rates below 1 are slower and deeper.
pub fn look(m: Material) -> Look {
    use Material as M;
    match m {
        M::Stone => Look {
            step: act(&["stone_step"], 1.0, 1.0, 5000.0),
            hit: act(&["stone_hit", "ceramic_hit"], 0.9, 0.9, 5000.0),
            brk: with(&["stone_dug"], 1.0, 1.0, 5000.0, ("stone_hit", 0.8, 0.6)),
            place: act(&["stone_place"], 1.0, 1.0, 5000.0),
        },
        M::Deepslate => Look {
            step: act(&["stone_step"], 0.82, 1.0, 4200.0),
            hit: act(&["stone_hit"], 0.78, 1.0, 4200.0),
            brk: with(&["stone_dug"], 0.82, 1.0, 4200.0, ("stone_place", 0.75, 0.5)),
            place: act(&["stone_place"], 0.85, 1.0, 4200.0),
        },
        M::Brick => Look {
            step: act(&["ceramic_step"], 1.0, 1.0, 5000.0),
            hit: act(&["ceramic_hit"], 1.0, 0.9, 5000.0),
            brk: act(&["ceramic_dug"], 1.0, 1.0, 5000.0),
            place: act(&["ceramic_place"], 1.0, 1.0, 5000.0),
        },
        M::Netherrack => Look {
            step: act(&["stone_step", "gravel_step"], 0.88, 0.9, 4200.0),
            hit: act(&["gravel_hit"], 0.8, 0.9, 4200.0),
            brk: act(&["gravel_dug", "stone_dug"], 0.85, 1.0, 3400.0),
            place: act(&["stone_place"], 0.9, 1.0, 4200.0),
        },
        M::Ore => Look {
            step: act(&["stone_step"], 0.95, 1.0, 5000.0),
            hit: act(&["stone_hit"], 0.85, 1.0, 5000.0),
            brk: with(&["stone_dug"], 0.95, 1.0, 5000.0, ("tool_mining", 0.85, 0.55)),
            place: act(&["stone_place"], 0.95, 1.0, 5000.0),
        },
        M::Metal => Look {
            step: act(&["metal_step"], 1.0, 0.9, 4500.0),
            hit: act(&["metal_hit"], 0.9, 0.75, 4000.0),
            brk: with(&["metal_dug"], 0.95, 0.9, 4000.0, ("metal_heavy", 0.85, 0.5)),
            place: act(&["metal_place"], 1.0, 0.9, 4500.0),
        },
        M::Copper => Look {
            step: act(&["metal_step"], 0.88, 0.9, 4000.0),
            hit: act(&["metal_hit"], 0.8, 0.7, 3600.0),
            brk: with(&["metal_dug"], 0.85, 0.9, 3600.0, ("stone_dug", 0.9, 0.4)),
            place: act(&["metal_place"], 0.85, 0.9, 4000.0),
        },
        M::Wood => Look {
            step: act(&["wood_step"], 1.0, 1.0, 5000.0),
            hit: act(&["wood_hit"], 1.0, 1.0, 5000.0),
            brk: act(&["wood_dug"], 1.0, 1.0, 5000.0),
            place: act(&["wood_place"], 1.0, 1.0, 5000.0),
        },
        M::Bamboo => Look {
            step: act(&["wood_step"], 1.12, 0.9, 5000.0),
            hit: act(&["bamboo_hit"], 1.0, 1.0, 5000.0),
            brk: with(&["bamboo_hit"], 0.9, 1.0, 5000.0, ("wood_dug", 1.15, 0.5)),
            place: act(&["bamboo_hit"], 0.95, 1.0, 5000.0),
        },
        M::Grass => Look {
            step: act(&["grass_step"], 1.0, 1.0, 4500.0),
            hit: act(&["grass_hit", "dirt_hit"], 1.0, 1.0, 4500.0),
            brk: with(&["soil_dug"], 1.0, 1.0, 4500.0, ("grass_hit", 0.95, 0.5)),
            place: act(&["soil_place"], 1.0, 1.0, 4500.0),
        },
        M::Dirt => Look {
            step: act(&["dirt_step"], 1.0, 1.0, 4500.0),
            hit: act(&["dirt_hit"], 1.0, 1.0, 4500.0),
            brk: with(&["soil_dug"], 0.95, 1.0, 4500.0, ("dirt_hit", 0.9, 0.5)),
            place: act(&["soil_place"], 0.95, 1.0, 4500.0),
        },
        M::Gravel => Look {
            step: act(&["gravel_step"], 1.0, 1.0, 4500.0),
            hit: act(&["gravel_hit"], 1.0, 1.0, 4500.0),
            brk: act(&["gravel_dug"], 0.97, 1.0, 3200.0),
            place: act(&["gravel_place"], 1.0, 1.0, 4500.0),
        },
        M::Sand => Look {
            step: act(&["sand_step"], 1.0, 1.0, 4200.0),
            hit: act(&["sand_hit"], 1.0, 0.9, 4200.0),
            brk: act(&["sand_dug"], 1.0, 1.0, 4200.0),
            place: act(&["sand_place"], 1.0, 1.0, 4200.0),
        },
        M::SoulSand => Look {
            step: with(&["sand_step"], 0.75, 1.0, 3500.0, ("mud_step", 0.8, 0.35)),
            hit: act(&["sand_hit"], 0.75, 0.9, 3500.0),
            brk: act(&["sand_dug"], 0.75, 1.0, 3500.0),
            place: act(&["sand_place"], 0.75, 1.0, 3500.0),
        },
        M::Mud => Look {
            step: act(&["mud_step"], 1.0, 1.0, 4000.0),
            hit: act(&["soft_hit"], 0.85, 0.9, 4000.0),
            brk: with(&["mud_step"], 0.85, 1.0, 4000.0, ("soft_heavy", 0.9, 0.6)),
            place: act(&["soft_heavy"], 0.9, 1.0, 4000.0),
        },
        M::Slime => Look {
            step: with(&["mud_step"], 1.15, 0.9, 4000.0, ("soft_hit", 1.1, 0.5)),
            hit: act(&["soft_hit"], 1.1, 0.9, 4000.0),
            brk: act(&["soft_heavy"], 1.05, 1.0, 4000.0),
            place: act(&["soft_heavy"], 1.12, 1.0, 4000.0),
        },
        M::Sculk => Look {
            step: act(&["mud_step"], 0.8, 0.7, 3200.0),
            hit: act(&["soft_hit"], 0.8, 0.8, 3200.0),
            brk: with(&["soft_heavy"], 0.8, 1.0, 3200.0, ("mud_step", 0.75, 0.5)),
            place: act(&["mud_step"], 0.8, 0.9, 3200.0),
        },
        M::Snow => Look {
            step: act(&["snow_step", "crust_step"], 1.0, 1.0, 4500.0),
            hit: act(&["snow_hit"], 1.0, 1.0, 4500.0),
            brk: with(&["snow_hit"], 0.9, 1.0, 4500.0, ("soft_hit", 0.9, 0.4)),
            place: act(&["snow_place"], 1.0, 1.0, 4500.0),
        },
        M::Ice => Look {
            step: act(&["ice_step"], 1.0, 1.0, 4500.0),
            hit: act(&["ice_hit"], 1.0, 0.9, 4500.0),
            brk: act(&["ice_dug", "glass_break"], 0.85, 0.9, 4000.0),
            place: act(&["glass_place"], 0.85, 1.0, 4500.0),
        },
        M::Glass => Look {
            step: act(&["glass_step"], 1.0, 0.9, 4500.0),
            hit: act(&["glass_hit"], 0.95, 0.8, 4200.0),
            brk: act(&["glass_break"], 0.95, 0.9, 4200.0),
            place: act(&["glass_place"], 1.0, 1.0, 4500.0),
        },
        M::Crystal => Look {
            step: act(&["glass_step", "ceramic_step"], 0.9, 0.9, 4200.0),
            hit: act(&["crystal_chime"], 0.65, 0.6, 3200.0),
            brk: with(&["glass_break"], 0.78, 0.9, 3800.0, ("crystal_chime", 0.6, 0.45)),
            place: act(&["crystal_chime"], 0.7, 0.6, 3200.0),
        },
        M::Cloth => Look {
            step: act(&["cloth_step"], 1.0, 1.0, 4500.0),
            hit: act(&["cloth_rustle"], 1.0, 0.9, 4500.0),
            brk: with(&["cloth_rustle"], 0.9, 1.0, 4500.0, ("soft_hit", 0.9, 0.5)),
            place: act(&["cloth_rustle"], 1.0, 1.0, 4500.0),
        },
        M::Leaves => Look {
            step: act(&["leaves_step"], 0.95, 0.8, 2800.0),
            hit: act(&["leaves_hit"], 1.0, 0.9, 3200.0),
            brk: act(&["leaves_dug"], 1.0, 1.0, 3600.0),
            place: act(&["leaves_place"], 1.0, 1.0, 3600.0),
        },
        M::Plant => Look {
            step: act(&["grass_step"], 1.05, 0.8, 4000.0),
            hit: act(&["plant_hit"], 1.0, 0.9, 3800.0),
            brk: act(&["plant_hit", "leaves_dug"], 0.95, 1.0, 3800.0),
            place: act(&["leaves_place"], 0.9, 1.0, 3800.0),
        },
        M::Straw => Look {
            step: act(&["grass_step", "leaves_step"], 0.85, 0.9, 3000.0),
            hit: act(&["plant_hit"], 0.85, 0.9, 3600.0),
            brk: with(&["leaves_dug"], 0.8, 1.0, 3600.0, ("soil_dug", 0.9, 0.5)),
            place: act(&["leaves_place"], 0.8, 1.0, 3600.0),
        },
        M::Moss => Look {
            step: with(&["grass_step"], 0.85, 0.85, 3600.0, ("cloth_step", 0.9, 0.5)),
            hit: act(&["cloth_rustle", "grass_hit"], 0.85, 0.9, 3600.0),
            brk: with(&["soil_dug"], 0.9, 1.0, 3600.0, ("cloth_rustle", 0.85, 0.4)),
            place: act(&["cloth_rustle"], 0.85, 1.0, 3600.0),
        },
        M::Bone => Look {
            step: act(&["ceramic_step"], 1.1, 0.9, 4500.0),
            hit: act(&["bone_knock"], 1.0, 0.9, 4500.0),
            brk: with(&["bone_knock"], 0.85, 1.0, 4500.0, ("stone_dug", 1.15, 0.4)),
            place: act(&["bone_knock"], 0.9, 1.0, 4500.0),
        },
    }
}

/// What an action does to its material's sound: base loudness (steps are quiet, a block
/// breaking is loud).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Step,
    Run,
    Sneak,
    Hit,
    Break,
    Place,
}

/// Mono samples at 44.1 kHz for `action` on material `m`; `r` is a random seed.
pub fn block(m: Material, action: Action, r: u32) -> Vec<f32> {
    let l = look(m);
    let (a, gain, rate, lp) = match action {
        Action::Step => (l.step, 0.34, 1.0, 1.0),
        // Running lands harder: louder, a touch quicker and brighter.
        Action::Run => (l.step, 0.5, 1.05, 1.08),
        Action::Sneak => (l.step, 0.14, 0.95, 0.8),
        Action::Hit => (l.hit, 0.45, 1.0, 1.0),
        Action::Break => (l.brk, 0.85, 1.0, 1.0),
        Action::Place => (l.place, 0.7, 1.0, 1.0),
    };
    let a = Act { rate: a.rate * rate, gain: a.gain * gain, lp: a.lp * lp, ..a };
    play(&a, r)
}

/// One play of an act: a random recording of its groups, its layer on top, with a little
/// random change of speed and loudness.
pub fn play(a: &Act, r: u32) -> Vec<f32> {
    let mut rng = Rng(r.wrapping_mul(2654435761).wrapping_add(0x9E37_79B9) | 1);
    let Some(clip) = pick(a.pick, &mut rng) else {
        return Vec::new();
    };
    let rate = a.rate * (0.93 + 0.14 * rng.f());
    let gain = a.gain * (0.85 + 0.25 * rng.f());
    let mut out = render(clip, rate, a.lp, gain);
    if let Some((group, xr, xg)) = a.extra {
        if let Some(c) = pick(&[group], &mut rng) {
            let layer = render(c, xr * (0.93 + 0.14 * rng.f()), a.lp, gain * xg);
            if layer.len() > out.len() {
                out.resize(layer.len(), 0.0);
            }
            for (o, s) in out.iter_mut().zip(layer) {
                *o += s;
            }
        }
    }
    limit(&mut out);
    out
}

/// Keeps a mix under full scale.
pub fn limit(out: &mut [f32]) {
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.95 {
        let g = 0.95 / peak;
        out.iter_mut().for_each(|s| *s *= g);
    }
}

pub struct Rng(pub u32);

impl Rng {
    pub fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Clip numbers of each group (`stone_step` holds stone_step_1, stone_step_2, ...).
fn groups() -> &'static HashMap<&'static str, Vec<u16>> {
    static G: OnceLock<HashMap<&'static str, Vec<u16>>> = OnceLock::new();
    G.get_or_init(|| {
        let mut g: HashMap<&'static str, Vec<u16>> = HashMap::new();
        for (i, (name, _)) in crate::sound_bank::CLIPS.iter().enumerate() {
            if let Some(cut) = name.rfind('_') {
                g.entry(&name[..cut]).or_default().push(i as u16);
            }
        }
        g
    })
}

/// How many recordings a group has (0: no such group).
#[allow(dead_code)]
pub fn group_len(group: &str) -> usize {
    groups().get(group).map_or(0, |v| v.len())
}

/// A random clip from these groups, each recording equally likely.
pub fn pick(from: &[&str], rng: &mut Rng) -> Option<u16> {
    let g = groups();
    let total: usize = from.iter().map(|n| g.get(n).map_or(0, |v| v.len())).sum();
    if total == 0 {
        return None;
    }
    let mut k = ((rng.f() * total as f32) as usize).min(total - 1);
    for n in from {
        let v = g.get(n).map_or(&[][..], |v| &v[..]);
        if k < v.len() {
            return Some(v[k]);
        }
        k -= v.len();
    }
    None
}

type Decoded = Arc<(Vec<f32>, u32)>;

/// A recording decoded to mono (each one decoded once, then kept).
fn decoded(i: u16) -> Option<Decoded> {
    static CACHE: OnceLock<Mutex<HashMap<u16, Decoded>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(d) = cache.lock().ok()?.get(&i) {
        return Some(d.clone());
    }
    use rodio::Source;
    let bytes: &'static [u8] = crate::sound_bank::CLIPS.get(i as usize)?.1;
    let src = rodio::Decoder::new(Cursor::new(bytes)).ok()?;
    let (ch, sr) = (src.channels().max(1) as usize, src.sample_rate());
    let all: Vec<f32> = src.convert_samples::<f32>().collect();
    let mono = all.chunks(ch).map(|c| c.iter().sum::<f32>() / ch as f32).collect();
    let d = Arc::new((mono, sr));
    cache.lock().ok()?.insert(i, d.clone());
    Some(d)
}

/// Recording `i` played at `rate` (1 = as recorded, below 1 slower and deeper), through a
/// gentle low-pass at `lp` Hz, at `gain`; mono samples at 44.1 kHz.
pub fn render(i: u16, rate: f32, lp: f32, gain: f32) -> Vec<f32> {
    let Some(d) = decoded(i) else {
        return Vec::new();
    };
    let (src, sr) = (&d.0, d.1 as f32);
    if src.len() < 2 {
        return Vec::new();
    }
    let step = sr / RATE * rate.clamp(0.3, 2.0);
    let n = ((src.len() - 1) as f32 / step) as usize;
    let k = 1.0 - (-std::f32::consts::TAU * lp.clamp(500.0, 12000.0) / RATE).exp();
    let (mut a, mut b) = (0.0f32, 0.0f32);
    let mut out: Vec<f32> = (0..n)
        .map(|j| {
            let p = j as f32 * step;
            let i = p as usize;
            let f = p - i as f32;
            let s = src[i] * (1.0 - f) + src[(i + 1).min(src.len() - 1)] * f;
            a += (s - a) * k;
            b += (a - b) * k;
            b * gain
        })
        .collect();
    // Short fades so nothing starts or ends on a click.
    let fade = (RATE * 0.004) as usize;
    let len = out.len();
    for j in 0..fade.min(len / 2) {
        let g = j as f32 / fade as f32;
        out[j] *= g;
        out[len - 1 - j] *= g;
    }
    out
}

/// Effective pitch from how fast the wave changes (tests: low and full, never shrill).
#[cfg(test)]
pub fn tests_brightness(v: &[f32]) -> f32 {
    let e: f32 = v.iter().map(|s| s * s).sum();
    let d: f32 = v.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
    RATE / std::f32::consts::PI * ((d / e.max(1e-9)).sqrt() / 2.0).min(1.0).asin()
}

/// Writes mono 44.1 kHz samples as a WAV file (listening tests).
#[cfg(test)]
pub fn export_wav(path: &str, all: &[f32]) {
    let data: Vec<u8> = all.iter().flat_map(|s| ((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()).collect();
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    for v in [16u32, 1 | (1 << 16), 44100, 88200, 2 | (16 << 16)] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data.len() as u32).to_le_bytes());
    b.extend_from_slice(&data);
    std::fs::write(path, b).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_material_names_real_recordings() {
        for m in Material::ALL {
            let l = look(m);
            for a in [l.step, l.hit, l.brk, l.place] {
                for g in a.pick {
                    assert!(group_len(g) > 0, "{m:?}: no recordings in group {g}");
                }
                if let Some((g, _, _)) = a.extra {
                    assert!(group_len(g) > 0, "{m:?}: no recordings in group {g}");
                }
            }
        }
    }

    #[test]
    fn every_material_and_action_sounds_and_varies() {
        for m in Material::ALL {
            for act in [Action::Step, Action::Run, Action::Sneak, Action::Hit, Action::Break, Action::Place] {
                let a = block(m, act, 1);
                assert!(a.len() > 1000, "{m:?} {act:?} too short: {}", a.len());
                let peak = a.iter().fold(0.0f32, |p, s| p.max(s.abs()));
                assert!(peak > 0.02 && peak <= 0.951, "{m:?} {act:?} peak {peak}");
                assert!(a.last().unwrap().abs() < 0.01, "{m:?} {act:?} ends on a click");
                // Two plays are never the same sample at the same speed.
                let b = block(m, act, 2);
                assert!(a.len() != b.len() || a != b, "{m:?} {act:?} repeats exactly");
            }
        }
    }

    #[test]
    fn running_is_louder_than_walking_and_sneaking_quieter() {
        let loud = |act| (0..12u32).map(|r| block(Material::Gravel, act, r).iter().fold(0.0f32, |p, s| p.max(s.abs()))).sum::<f32>();
        assert!(loud(Action::Run) > loud(Action::Step));
        assert!(loud(Action::Sneak) < loud(Action::Step));
    }

    #[test]
    fn nothing_is_shrill() {
        let mut worst = 0.0f32;
        for m in Material::ALL {
            for act in [Action::Step, Action::Run, Action::Hit, Action::Break, Action::Place] {
                for r in 0..6 {
                    let v = block(m, act, r);
                    let hz = tests_brightness(&v);
                    println!("{m:?} {act:?} {hz:.0}");
                    worst = worst.max(hz);
                }
            }
        }
        assert!(worst < 2600.0, "too bright: {worst:.0} Hz");
    }

    #[test]
    fn every_recording_decodes() {
        for (i, (name, _)) in crate::sound_bank::CLIPS.iter().enumerate() {
            let v = render(i as u16, 1.0, 8000.0, 1.0);
            assert!(v.len() > 500, "{name} too short: {}", v.len());
            assert!(v.iter().any(|s| s.abs() > 0.3), "{name} is nearly silent");
        }
    }
}

#[cfg(test)]
mod export {
    use super::*;

    /// Writes each material's walk, run, hit, break and place sounds as WAV files into
    /// $MC_SOUNDS_OUT (listening test).
    #[test]
    #[ignore]
    fn export_block_sounds() {
        let Ok(dir) = std::env::var("MC_SOUNDS_OUT") else { return };
        for m in Material::ALL {
            let mut all = Vec::new();
            for (k, act) in [Action::Step, Action::Step, Action::Step, Action::Run, Action::Run, Action::Run, Action::Hit, Action::Hit, Action::Break, Action::Place].into_iter().enumerate() {
                all.extend(block(m, act, k as u32 * 7919 + 3));
                let gap = if matches!(act, Action::Step | Action::Run) { 9000 } else { 16000 };
                all.extend(std::iter::repeat_n(0.0, gap));
            }
            export_wav(&format!("{dir}/{m:?}.wav"), &all);
        }
    }


}

#[cfg(test)]
mod materials {
    use super::*;

    /// Every block name in the base game's packs.
    fn all_block_names() -> Vec<String> {
        let mut names = Vec::new();
        let mut stack = vec![std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../packs/vanilla"))];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.file_name().is_some_and(|n| n == "blocks.ron") {
                    let text = std::fs::read_to_string(&p).unwrap();
                    for line in text.lines() {
                        if let Some(i) = line.find("(name: \"") {
                            let rest = &line[i + 8..];
                            if let Some(j) = rest.find('"') {
                                names.push(rest[..j].to_string());
                            }
                        }
                    }
                }
            }
        }
        names.sort();
        names.dedup();
        names
    }

    #[test]
    fn blocks_sound_like_their_texture() {
        use Material as M;
        let cases = [
            ("stone", M::Stone), ("cobblestone", M::Stone), ("stone_bricks", M::Stone), ("sandstone", M::Stone),
            ("deepslate", M::Deepslate), ("tuff_bricks", M::Deepslate), ("obsidian", M::Deepslate), ("polished_blackstone", M::Deepslate),
            ("bricks", M::Brick), ("white_terracotta", M::Brick), ("red_concrete", M::Brick), ("mud_bricks", M::Brick), ("quartz_block", M::Brick),
            ("netherrack", M::Netherrack), ("crimson_nylium", M::Netherrack),
            ("iron_ore", M::Ore), ("deepslate_diamond_ore", M::Ore), ("mythril_ore", M::Ore),
            ("iron_block", M::Metal), ("anvil", M::Metal), ("platinum_block", M::Metal), ("iron_bars", M::Metal), ("lantern", M::Metal), ("rail", M::Metal),
            ("cut_copper", M::Copper), ("waxed_oxidized_copper", M::Copper),
            ("oak_planks", M::Wood), ("birch_slab", M::Wood), ("crimson_stem", M::Wood), ("spruce_chair", M::Wood), ("chest", M::Wood), ("bookshelf", M::Wood), ("pumpkin", M::Wood), ("torch", M::Wood),
            ("bamboo", M::Bamboo),
            ("grass_block", M::Grass), ("mycelium", M::Grass), ("podzol", M::Grass),
            ("dirt", M::Dirt), ("coarse_dirt", M::Dirt), ("farmland", M::Dirt), ("rooted_dirt", M::Dirt),
            ("gravel", M::Gravel), ("suspicious_gravel", M::Gravel),
            ("sand", M::Sand), ("red_sand", M::Sand), ("suspicious_sand", M::Sand),
            ("soul_sand", M::SoulSand), ("soul_soil", M::SoulSand),
            ("mud", M::Mud), ("clay", M::Mud),
            ("slime_block", M::Slime), ("honey_block", M::Slime),
            ("sculk", M::Sculk), ("sculk_sensor", M::Sculk),
            ("snow", M::Snow), ("snow_layer", M::Snow), ("snowy_grass_block", M::Snow),
            ("ice", M::Ice), ("packed_ice", M::Ice), ("blue_ice", M::Ice),
            ("glass", M::Glass), ("red_stained_glass", M::Glass), ("glass_pane", M::Glass), ("glowstone", M::Glass),
            ("amethyst_block", M::Crystal), ("budding_amethyst", M::Crystal), ("life_crystal", M::Crystal),
            ("white_wool", M::Cloth), ("red_carpet", M::Cloth), ("red_bed", M::Cloth), ("white_banner", M::Cloth),
            ("oak_leaves", M::Leaves), ("cherry_leaves", M::Leaves),
            ("poppy", M::Plant), ("wheat_stage3", M::Plant), ("oak_sapling", M::Plant), ("short_grass", M::Plant), ("sugar_cane", M::Plant), ("vine", M::Plant), ("brown_mushroom", M::Plant), ("cactus", M::Plant),
            ("hay_block", M::Straw), ("thatch", M::Straw),
            ("moss_block", M::Moss), ("moss_carpet", M::Moss), ("nether_wart_block", M::Moss),
            ("bone_block", M::Bone), ("wither_skeleton_skull", M::Bone),
        ];
        let mut wrong = Vec::new();
        for (name, want) in cases {
            let got = Material::of(name);
            if got != want {
                wrong.push(format!("{name}: {got:?} (want {want:?})"));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    /// Lists every block by material (MC_LIST_MATERIALS=1 prints them all).
    #[test]
    fn most_blocks_have_a_fitting_material() {
        let names = all_block_names();
        assert!(names.len() > 1000, "found only {} blocks", names.len());
        let mut by: HashMap<String, Vec<String>> = HashMap::new();
        for n in &names {
            by.entry(format!("{:?}", Material::of(n))).or_default().push(n.clone());
        }
        if std::env::var("MC_LIST_MATERIALS").is_ok() {
            let mut keys: Vec<_> = by.keys().cloned().collect();
            keys.sort();
            for k in keys {
                println!("{k} ({}): {}", by[&k].len(), by[&k].join(" "));
            }
        }
        // Stone is the fallback; it should hold stones, not everything left over.
        let stone = by.get("Stone").map_or(0, |v| v.len());
        assert!(stone * 5 < names.len(), "{stone} of {} blocks fall back to stone", names.len());
        assert!(by.len() == Material::ALL.len(), "some materials are never used: {:?}", by.keys());
    }
}
