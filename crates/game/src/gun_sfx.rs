//! Gunshots built the way real ones sound, not beeps (HP: no synthetic or high-pitched sounds).
//!
//! A real shot is a few things on top of each other, and each gun mixes them its own way:
//! - the blast: a thump of air with a short, dark burst of noise on top, deeper and longer the
//!   bigger the charge (a pistol's crack, a musket's boom, a shotgun's wide roar);
//! - the action: the mechanism moving, a few short wooden-or-metal knocks (a slide snapping, a
//!   hammer falling, a pump racked back and forth, a bolt);
//! - the land answering: a low rumble rolling away and one slap of echo off the ground, so a
//!   shot outside has its size;
//! - the odd ones: a flintlock's pan flashing just before the charge, a rocket's motor
//!   roaring off, gel burning, breath through a reed, an energy gun's crackle of discharge,
//!   wet bubbles, coins rattling.
//! Everything is filtered dark (under ~1.5 kHz) so it never hisses or pierces, and every shot
//! comes out a little different (the seed).

use std::f32::consts::TAU;

const RATE: f32 = 44100.0;

struct Rng(u32);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32
    }
    fn noise(&mut self) -> f32 {
        self.next() * 2.0 - 1.0
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
}

fn coef(cutoff: f32) -> f32 {
    1.0 - (-TAU * cutoff.max(1.0) / RATE).exp()
}

/// Two one-pole low-passes in a row.
struct Lp {
    a: f32,
    y1: f32,
    y2: f32,
}

impl Lp {
    fn new(cutoff: f32) -> Lp {
        Lp { a: coef(cutoff), y1: 0.0, y2: 0.0 }
    }
    fn set(&mut self, cutoff: f32) {
        self.a = coef(cutoff);
    }
    fn run(&mut self, x: f32) -> f32 {
        self.y1 += self.a * (x - self.y1);
        self.y2 += self.a * (self.y1 - self.y2);
        self.y2
    }
}

/// A two-pole resonator: rings at one frequency like a struck part.
struct Res {
    b1: f32,
    b2: f32,
    y1: f32,
    y2: f32,
    gain: f32,
}

impl Res {
    fn new(freq: f32, bandwidth: f32) -> Res {
        let r = (-std::f32::consts::PI * bandwidth / RATE).exp();
        Res { b1: 2.0 * r * (TAU * freq / RATE).cos(), b2: -r * r, y1: 0.0, y2: 0.0, gain: 1.0 - r }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = x * self.gain + self.b1 * self.y1 + self.b2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn n(secs: f32) -> usize {
    (secs * RATE) as usize
}

/// The charge going off at `at` seconds: a thump falling from `f0`, `decay` seconds long, with
/// a burst of noise darkened to `bright` Hz.
fn blast(out: &mut [f32], at: f32, f0: f32, decay: f32, bright: f32, amp: f32, r: &mut Rng) {
    let s0 = n(at);
    let mut lp = Lp::new(bright);
    let mut body = Lp::new(bright * 0.35);
    let mut ph = 0.0f32;
    for k in 0..n(decay * 6.0) {
        let Some(o) = out.get_mut(s0 + k) else { break };
        let t = k as f32 / RATE;
        let attack = (t / 0.0008).min(1.0);
        // The pitch drops fast at first, like a pressure wave spreading.
        let f = f0 * (1.0 + 1.6 * (-t / 0.008).exp());
        ph += f / RATE;
        let thump = (ph * TAU).sin() * (-t / decay).exp();
        let x = r.noise();
        let crack = lp.run(x) * (-t / (decay * 0.3)).exp() * 2.2;
        let roar = body.run(x) * (-t / (decay * 0.9)).exp() * 2.6;
        *o += (thump * 0.9 + crack + roar) * attack * amp;
    }
}

/// The air and the land answering: a low rumble that rolls off over `len` seconds.
fn tail(out: &mut [f32], at: f32, len: f32, amp: f32, r: &mut Rng) {
    let s0 = n(at);
    let mut lp = Lp::new(320.0);
    let mut slow = Lp::new(14.0);
    for k in 0..n(len) {
        let Some(o) = out.get_mut(s0 + k) else { break };
        let t = k as f32 / RATE;
        let swell = (t / 0.02).min(1.0);
        // A little unevenness, like the sound bouncing off hills.
        let wobble = 0.75 + (slow.run(r.noise()) * 30.0).clamp(-0.25, 0.25);
        lp.set(320.0 - 200.0 * (t / len));
        *o += lp.run(r.noise()) * swell * (-t / (len * 0.3)).exp() * wobble * amp * 2.4;
    }
}

/// A part of the mechanism knocking at `at`: two close modes, like steel on steel (or wood).
fn clack(out: &mut [f32], at: f32, f: f32, amp: f32, r: &mut Rng) {
    let s0 = n(at);
    let mut a = Res::new(f, 120.0);
    let mut b = Res::new(f * 1.47, 160.0);
    let mut lp = Lp::new(1400.0);
    for k in 0..n(0.05) {
        let Some(o) = out.get_mut(s0 + k) else { break };
        let t = k as f32 / RATE;
        let x = if k < 50 { r.noise() } else { 0.0 };
        *o += lp.run(a.run(x) * 7.0 + b.run(x) * 4.0) * (-t / 0.012).exp() * amp;
    }
}

/// A band of noise between `lo` and `hi`, shaped by `env` (0..1 through it).
fn hiss(out: &mut [f32], at: f32, len: f32, lo: f32, hi: f32, amp: f32, r: &mut Rng, env: impl Fn(f32) -> f32) {
    let s0 = n(at);
    let mut h = Lp::new(hi);
    let mut l = Lp::new(lo);
    let total = n(len).max(1);
    for k in 0..total {
        let Some(o) = out.get_mut(s0 + k) else { break };
        let x = h.run(r.noise());
        *o += (x - l.run(x)) * env(k as f32 / total as f32) * amp * 3.0;
    }
}

/// A wet plop: a bubble bursting, its pitch rising a little.
fn plop(out: &mut [f32], at: f32, f0: f32, amp: f32) {
    let s0 = n(at);
    let mut ph = 0.0f32;
    for k in 0..n(0.06) {
        let Some(o) = out.get_mut(s0 + k) else { break };
        let t = k as f32 / RATE;
        ph += f0 * (1.0 + t * 8.0) / RATE;
        *o += (ph * TAU).sin() * (t / 0.002).min(1.0) * (-t / 0.018).exp() * amp;
    }
}

/// One slap of echo off the ground, darker than the sound.
fn echo(v: &mut Vec<f32>, delay: f32, amp: f32) {
    let d = n(delay);
    let len = v.len();
    v.resize(len + d, 0.0);
    let mut lp = Lp::new(700.0);
    for k in (0..len).rev() {
        let x = v[k];
        v[k + d] += x * amp;
    }
    for x in v.iter_mut().skip(d) {
        *x = *x * 0.5 + lp.run(*x) * 0.5;
    }
}

/// Fade both ends so nothing clicks; bring the loudest to `peak`.
fn finish(mut v: Vec<f32>, peak: f32) -> Vec<f32> {
    let len = v.len();
    let f = n(0.004).min(len / 2);
    for k in 0..f {
        v[k] *= k as f32 / f as f32;
    }
    let tail = n(0.05).min(len / 2);
    for k in 0..tail {
        v[len - 1 - k] *= k as f32 / tail as f32;
    }
    let top = v.iter().fold(0.0f32, |a, x| a.max(x.abs()));
    if top > 1e-6 {
        let g = peak / top;
        v.iter_mut().for_each(|x| *x *= g);
    }
    v
}

/// Gunshot `kind` (guns::Shot as u8).
pub fn gunshot(kind: u8, seed: u32) -> Vec<f32> {
    let r = &mut Rng(seed | 1);
    // Every shot a little different: the charge, the room, the gun's wear.
    let p = r.range(0.92, 1.08);
    let mut v;
    match kind {
        // Flintlock: the flint strikes, the pan flashes with a fizz, then the charge goes off.
        0 => {
            v = vec![0.0; n(1.3)];
            clack(&mut v, 0.0, 820.0 * p, 0.35, r);
            hiss(&mut v, 0.012, 0.07, 300.0, 1300.0, 0.5, r, |k| (k * 3.14).sin());
            blast(&mut v, 0.075, 72.0 * p, 0.11, 1100.0, 1.0, r);
            tail(&mut v, 0.09, 1.1, 0.45, r);
            echo(&mut v, 0.19, 0.22);
        }
        // Pistol: a short crack, the slide snaps back and forth.
        1 => {
            v = vec![0.0; n(0.6)];
            blast(&mut v, 0.0, 115.0 * p, 0.045, 1500.0, 1.0, r);
            clack(&mut v, 0.03, 980.0 * p, 0.25, r);
            clack(&mut v, 0.065, 860.0 * p, 0.2, r);
            tail(&mut v, 0.01, 0.5, 0.3, r);
            echo(&mut v, 0.14, 0.18);
        }
        // Revolver and magnum: heavier, no slide, the cylinder clicks round.
        2 => {
            v = vec![0.0; n(0.85)];
            blast(&mut v, 0.0, 92.0 * p, 0.065, 1350.0, 1.0, r);
            tail(&mut v, 0.01, 0.75, 0.38, r);
            clack(&mut v, 0.22, 700.0 * p, 0.12, r);
            echo(&mut v, 0.17, 0.2);
        }
        // Musket: a deep boom with the pan's flash just before.
        3 => {
            v = vec![0.0; n(1.6)];
            hiss(&mut v, 0.0, 0.04, 300.0, 1200.0, 0.4, r, |k| (k * 3.14).sin());
            blast(&mut v, 0.035, 58.0 * p, 0.15, 1000.0, 1.0, r);
            tail(&mut v, 0.05, 1.4, 0.55, r);
            echo(&mut v, 0.23, 0.26);
        }
        // Shotgun: a wide, low roar, then the pump racked back and forward (or the action).
        4 => {
            v = vec![0.0; n(1.4)];
            blast(&mut v, 0.0, 54.0 * p, 0.14, 900.0, 1.0, r);
            tail(&mut v, 0.02, 1.1, 0.55, r);
            clack(&mut v, 0.42, 560.0 * p, 0.3, r);
            hiss(&mut v, 0.43, 0.08, 200.0, 700.0, 0.12, r, |k| 1.0 - k);
            clack(&mut v, 0.56, 480.0 * p, 0.32, r);
            echo(&mut v, 0.21, 0.24);
        }
        // Rifle: sharp and big, a long roll off the land, the bolt or action working.
        5 => {
            v = vec![0.0; n(1.3)];
            blast(&mut v, 0.0, 84.0 * p, 0.075, 1450.0, 1.0, r);
            tail(&mut v, 0.01, 1.2, 0.48, r);
            clack(&mut v, 0.05, 900.0 * p, 0.16, r);
            echo(&mut v, 0.26, 0.24);
        }
        // Automatic: light and short so a burst runs together, a soft mechanical rattle.
        6 => {
            v = vec![0.0; n(0.32)];
            blast(&mut v, 0.0, 125.0 * p, 0.028, 1400.0, 1.0, r);
            clack(&mut v, 0.022, 1000.0 * p, 0.18, r);
            tail(&mut v, 0.005, 0.28, 0.22, r);
        }
        // Minigun: shorter still and lower; at full speed it becomes one tearing roar.
        7 => {
            v = vec![0.0; n(0.24)];
            blast(&mut v, 0.0, 105.0 * p, 0.022, 1250.0, 1.0, r);
            tail(&mut v, 0.004, 0.22, 0.25, r);
        }
        // Rocket: a hollow thump out of the tube and the motor roaring away.
        8 => {
            v = vec![0.0; n(1.3)];
            blast(&mut v, 0.0, 50.0 * p, 0.09, 700.0, 0.8, r);
            hiss(&mut v, 0.02, 1.1, 180.0, 900.0, 0.9, r, |k| (k * 12.0).min(1.0) * (1.0 - k).powf(1.6));
            tail(&mut v, 0.02, 0.8, 0.3, r);
        }
        // Grenade launcher: the round "thoonk" of a fat barrel.
        9 => {
            v = vec![0.0; n(0.7)];
            let mut tube = Res::new(170.0 * p, 40.0);
            let mut lp = Lp::new(600.0);
            for k in 0..n(0.25) {
                let t = k as f32 / RATE;
                let x = if k < 300 { r.noise() } else { 0.0 };
                v[k] += lp.run(tube.run(x) * 22.0) * (-t / 0.06).exp();
            }
            blast(&mut v, 0.0, 80.0 * p, 0.05, 600.0, 0.6, r);
            tail(&mut v, 0.01, 0.5, 0.2, r);
        }
        // Flamethrower: gel burning out of the nozzle, a soft deep roar with a flutter.
        10 => {
            v = vec![0.0; n(0.26)];
            let mut lp = Lp::new(480.0);
            let mut flutter = Lp::new(30.0);
            for (k, o) in v.iter_mut().enumerate() {
                let t = k as f32 / RATE;
                let env = (t / 0.03).min(1.0) * (1.0 - t / 0.26).max(0.0).powf(0.7);
                let fl = 0.6 + (flutter.run(r.noise()) * 20.0).clamp(-0.4, 0.4);
                *o += lp.run(r.noise()) * env * fl * 3.0;
            }
        }
        // Blowpipe: a puff of breath down the cane and the seed knocking out.
        11 => {
            v = vec![0.0; n(0.25)];
            hiss(&mut v, 0.0, 0.11, 140.0, 650.0, 0.9, r, |k| (k * 3.14).sin().powf(0.6));
            clack(&mut v, 0.07, 420.0 * p, 0.25, r);
        }
        // Energy guns: a crackling discharge, like a spark jumping, over a low thump.
        12 => {
            v = vec![0.0; n(0.45)];
            blast(&mut v, 0.0, 70.0 * p, 0.05, 600.0, 0.6, r);
            let mut lp = Lp::new(1300.0);
            let mut t = 0.0;
            while t < 0.12 {
                let s0 = n(t);
                let len = n(r.range(0.002, 0.006));
                let a = (1.0 - t / 0.12) * r.range(0.4, 1.0);
                for k in 0..len {
                    if let Some(o) = v.get_mut(s0 + k) {
                        *o += lp.run(r.noise()) * a * 2.0;
                    }
                }
                t += r.range(0.004, 0.014);
            }
            tail(&mut v, 0.0, 0.4, 0.2, r);
        }
        // Bubble rifle: a gulp of air and a few wet pops.
        13 => {
            v = vec![0.0; n(0.4)];
            hiss(&mut v, 0.0, 0.06, 120.0, 500.0, 0.6, r, |k| 1.0 - k);
            for j in 0..4 {
                let f = r.range(260.0, 420.0);
                plop(&mut v, 0.02 + j as f32 * r.range(0.03, 0.06), f, 0.6);
            }
        }
        // Star cannon: a whoosh of falling-star air and a soft deep bloom.
        14 => {
            v = vec![0.0; n(0.9)];
            blast(&mut v, 0.0, 66.0 * p, 0.07, 700.0, 0.7, r);
            hiss(&mut v, 0.0, 0.7, 200.0, 1100.0, 0.7, r, |k| (k * 8.0).min(1.0) * (1.0 - k).powf(2.0));
        }
        // Coin gun: a pop and coins rattling in the hopper.
        15 => {
            v = vec![0.0; n(0.6)];
            blast(&mut v, 0.0, 100.0 * p, 0.04, 1100.0, 0.8, r);
            for j in 0..5 {
                let f = r.range(560.0, 820.0);
                clack(&mut v, 0.06 + j as f32 * r.range(0.025, 0.05), f, 0.18, r);
            }
            tail(&mut v, 0.0, 0.45, 0.25, r);
        }
        // An explosive bullet bursting: a small, dull bang.
        _ => {
            v = vec![0.0; n(0.6)];
            blast(&mut v, 0.0, 62.0 * p, 0.06, 800.0, 1.0, r);
            tail(&mut v, 0.0, 0.5, 0.35, r);
        }
    }
    let peak = match kind {
        6 | 7 | 10 => 0.55,
        11 | 13 => 0.5,
        _ => 0.85,
    };
    finish(v, peak)
}

/// An empty gun: the hammer or striker falls on nothing, a dry little click.
pub fn dry_fire(seed: u32) -> Vec<f32> {
    let r = &mut Rng(seed | 1);
    let mut v = vec![0.0; n(0.16)];
    clack(&mut v, 0.0, r.range(620.0, 720.0), 0.6, r);
    clack(&mut v, 0.035, r.range(480.0, 560.0), 0.3, r);
    finish(v, 0.35)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shots_differ_by_gun_and_by_seed() {
        let a = gunshot(1, 5);
        let b = gunshot(1, 9);
        let c = gunshot(4, 5);
        assert!(a != b, "every shot should be a little different");
        assert!(c.len() > a.len(), "a shotgun rolls on longer than a pistol");
        // Automatic fire must be short enough to run together at ten shots a second.
        assert!(gunshot(7, 1).len() < n(0.3));
    }
}
