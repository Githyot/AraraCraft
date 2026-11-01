//! More procedural sounds for an immersive world: animal and monster voices, ambience (caves,
//! wind, waves, the Nether, the End, night animals), body sounds (landing, jumping, a heartbeat
//! at low health, gasping for air) and interface sounds. Same rules as sfx.rs: no sound files,
//! nothing shrill. Voices sit low, and every sound comes out a little different each time.

use std::f32::consts::{PI, TAU};

const RATE: f32 = 44100.0;

/// An extra sound (played through `Sfx::Fx`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fx {
    /// A real recorded creature sound (mob_sounds.rs): which clip, and how fast to play it
    /// (below 1 is slower and deeper).
    Clip(u16, f32),
    /// A long, low swell deep under ground.
    CaveDrone,
    /// Stones tumbling somewhere far off in the dark.
    CaveRumble,
    /// One drop of water landing.
    Drip,
    /// A gust of wind; the value is how strong (0..1).
    Wind(f32),
    /// Leaves moving in the wind over a forest.
    Canopy,
    /// A wave rolling in on a beach.
    Waves,
    /// Small water slapping against a shore.
    Lapping,
    /// Lava bubbling and popping.
    LavaPop,
    /// A fire crackling.
    FireCrackle,
    /// The Nether's deep, slow moan.
    NetherDrone,
    /// The hollow wind of the soul sand valleys.
    SoulWind,
    /// The End's empty hum.
    EndHum,
    /// "Lub-dub", at low health.
    Heartbeat,
    /// A deep breath after a long time under water.
    Gasp,
    /// Landing after a fall; the value is how hard (0..1).
    Land(f32),
    /// Pushing off the ground.
    Jump,
    /// A weapon or arm cutting the air.
    Swing,
    /// A weapon cutting the air: 0 a light blade .. 3 a greatsword or club (blade_fx.rs).
    Blade(u8),
    /// A weapon biting into a body: 0 wood or stone, 1 metal, 2 crystal, 3 magic.
    Bite(u8),
    /// Walking through tall grass and bushes.
    Brush,
    /// Opening the inventory: a soft leather flap.
    InvOpen,
    /// Closing it.
    InvClose,
    /// Worn armour shifting as you walk: true for metal, false for leather.
    Armor(bool),
    /// A fish slapping back into the water: a low wet plop.
    FishSplash,
    /// A flock passing over: soft, low wingbeats.
    Wings,
    /// Feet sliding over ice; the value is how fast (0..1). A soft, low scrape.
    Slide(f32),
    /// A gun going off (guns.rs `Shot`, gun_sfx.rs).
    Gun(u8),
    /// The trigger pulled on an empty gun.
    DryFire,
    /// Something thrown (assassin.rs): a knife or shuriken spinning off the hand (false), or a
    /// heavier bomb or flask lobbed (true).
    Throw(bool),
    /// A blade going in from behind: a dull, short, wet thud, no ring of metal.
    Stab,
    /// A smoke bomb bursting: a soft "pff" of air and a low rumble as the cloud rolls out.
    Smoke,
    /// The stealth meter full: a slow breath let out through the nose.
    Hush,
    /// The assassin's quick step: a short low rush of air and cloth.
    Dash,
    /// A bow being drawn: the wood creaking and the string stretching. The value is how heavy
    /// the bow is (0 light wood .. 2 heavy metal), heavier ones lower and slower.
    BowDraw(u8),
    /// The string let go: a low thrum and the limbs snapping forward (same weights).
    BowRelease(u8),
    /// An arrow striking: 0 wood, 1 stone, 2 soft ground, 3 flesh, 4 metal. In wood it quivers.
    ArrowHit(u8),
    /// A crossbow being cranked: low ratchet clicks and the latch.
    CrossbowLoad,
    /// A crossbow going off: a heavy clack and a short, deep string.
    CrossbowShot,
    /// A whip's crack at the tip; true when the tip caught a monster (a fuller smack).
    WhipCrack(bool),
    /// A tool wearing out and breaking in the hand: 0 wood (a dry crack), 1 stone (a chunk
    /// splitting off), 2 metal (a dull snap and a short ring), 3 gem (a low glassy chink).
    ToolBreak(u8),
    /// A drill (true) or chainsaw (false) biting into a block: one short, low burst of motor
    /// and grinding, played back to back while it digs.
    Motor(bool),
    /// A wooden fishing reel turning: a few dull ratchet clicks.
    Reel,
    /// A fish nibbling at the float: a tiny wet tap.
    Nibble,
    /// A fishing line snapping: a dull slap and the rod springing back.
    LineSnap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    /// Zombies.
    Groan,
    /// Skeletons: bones knocking together.
    Rattle,
    /// Spiders.
    Hiss,
    /// Cows and other big grazers.
    Moo,
    /// Pigs and boars.
    Grunt,
    /// Sheep.
    Bleat,
    /// Chickens.
    Cluck,
    /// Ducks and penguins.
    Quack,
    /// Small birds, kept to a low coo.
    Coo,
    /// Deer, foxes, rabbits: a soft snort through the nose.
    Snort,
    /// Bears and big cats.
    Growl,
    /// Ghosts, wraiths and void things.
    Wail,
    /// Witches, goblins, imps.
    Cackle,
    /// People: a short "hm" or "huh".
    Murmur,
    /// Giant stone and wood things.
    Rumble,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    Idle,
    Hurt,
    Death,
}

/// Which voice a mob has, from its name (None: it makes no sound of its own).
pub fn voice_of(name: &str, hostile: bool) -> Option<Voice> {
    let has = |s: &str| name.contains(s);
    Some(match name {
        "minecart" => return None,
        _ if has("zombie") => Voice::Groan,
        _ if has("skeleton") => Voice::Rattle,
        "spider" | "stalker" => Voice::Hiss,
        "cow" | "hippo" | "elephant" | "rhino" => Voice::Moo,
        "pig" | "boar" | "crimson_boar" => Voice::Grunt,
        "sheep" => Voice::Bleat,
        "chicken" => Voice::Cluck,
        "duck" | "penguin" => Voice::Quack,
        "bluejay" | "cardinal" => Voice::Coo,
        "deer" | "zebra" | "fox" | "rabbit" | "squirrel" => Voice::Snort,
        "bear" | "polar_bear" | "lion" => Voice::Growl,
        "lich" | "ember_wraith" | "void_blossom" => Voice::Wail,
        _ if has("void_walker") => Voice::Wail,
        "forest_guardian" | "obsidilith" | "void_guardian" => Voice::Rumble,
        "swamp_witch" | "goblin_trader" | "fire_imp" => Voice::Cackle,
        "player" | "brute" | "marauder" | "invoker" => Voice::Murmur,
        _ if has("npc_") => Voice::Murmur,
        _ if hostile => Voice::Groan,
        _ => Voice::Snort,
    })
}

impl Voice {
    /// Average seconds between idle calls.
    pub fn chattiness(self) -> f32 {
        match self {
            Voice::Cluck => 7.0,
            Voice::Hiss | Voice::Groan | Voice::Rumble => 9.0,
            Voice::Rattle | Voice::Grunt | Voice::Quack | Voice::Coo => 10.0,
            Voice::Wail => 11.0,
            Voice::Cackle | Voice::Murmur => 13.0,
            Voice::Bleat => 13.0,
            Voice::Moo => 15.0,
            Voice::Snort => 16.0,
            Voice::Growl => 18.0,
        }
    }
}

pub fn synth(fx: Fx, seed: u32) -> Vec<f32> {
    let mut rng = Rng(seed.wrapping_mul(2654435761).wrapping_add(0x9e37_79b9) | 1);
    let r = &mut rng;
    let out = match fx {
        Fx::Clip(i, rate) => crate::mob_sounds::render(i, rate),
        Fx::CaveDrone => cave_drone(r),
        Fx::CaveRumble => cave_rumble(r),
        Fx::Drip => drip(r),
        Fx::Wind(s) => wind(r, s.clamp(0.0, 1.0), false),
        Fx::Canopy => wind(r, 0.5, true),
        Fx::Waves => waves(r),
        Fx::Lapping => lapping(r),
        Fx::LavaPop => lava_pop(r),
        Fx::FireCrackle => crackle(r),
        Fx::NetherDrone => nether_drone(r),
        Fx::SoulWind => soul_wind(r),
        Fx::EndHum => end_hum(r),
        Fx::Heartbeat => heartbeat(),
        Fx::Gasp => gasp(r),
        Fx::Land(s) => land(r, s.clamp(0.0, 1.0)),
        Fx::Jump => whoosh(r, 0.16, 260.0, 520.0, 0.35),
        Fx::Swing => whoosh(r, 0.24, 220.0, 750.0, 0.5),
        Fx::Blade(w) => {
            let w = w.min(3) as f32;
            whoosh(r, 0.18 + 0.07 * w, 250.0 - 40.0 * w, 820.0 - 130.0 * w, 0.42 + 0.08 * w)
        }
        Fx::Bite(k) => bite(r, k),
        Fx::Brush => brush(r),
        Fx::InvOpen => flap(r, true),
        Fx::InvClose => flap(r, false),
        Fx::Armor(metal) => armor(r, metal),
        Fx::FishSplash => fish_splash(r),
        Fx::Wings => wings(r),
        Fx::Slide(s) => slide(r, s.clamp(0.0, 1.0)),
        Fx::Gun(k) => crate::gun_sfx::gunshot(k, r.0),
        Fx::DryFire => crate::gun_sfx::dry_fire(r.0),
        Fx::Throw(heavy) => throw(r, heavy),
        Fx::Stab => stab(r),
        Fx::Smoke => smoke_puff(r),
        Fx::Hush => hush(r),
        Fx::Dash => whoosh(r, 0.3, 140.0, 480.0, 0.55),
        Fx::BowDraw(w) => bow_draw(r, w.min(2)),
        Fx::BowRelease(w) => bow_release(r, w.min(2), false),
        Fx::ArrowHit(m) => arrow_hit(r, m.min(4)),
        Fx::CrossbowLoad => crossbow_load(r),
        Fx::CrossbowShot => bow_release(r, 2, true),
        Fx::WhipCrack(hit) => whip_crack(r, hit),
        Fx::ToolBreak(m) => tool_break(r, m),
        Fx::Motor(drill) => motor(r, drill),
        Fx::Reel => reel(r),
        Fx::Nibble => nibble(r),
        Fx::LineSnap => line_snap(r),
    };
    out.into_iter().map(|s| s.clamp(-1.0, 1.0)).collect()
}

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

/// Two one-pole low-pass filters in a row.
struct LowPass {
    a: f32,
    y1: f32,
    y2: f32,
}

impl LowPass {
    fn new(cutoff: f32) -> LowPass {
        LowPass {
            a: coef(cutoff),
            y1: 0.0,
            y2: 0.0,
        }
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

fn coef(cutoff: f32) -> f32 {
    1.0 - (-TAU * cutoff.max(1.0) / RATE).exp()
}

/// Band of noise between two corners: a low-pass minus a lower low-pass.
struct Band {
    hi: LowPass,
    lo: LowPass,
}

impl Band {
    fn new(lo: f32, hi: f32) -> Band {
        Band {
            hi: LowPass::new(hi),
            lo: LowPass::new(lo),
        }
    }
    fn set(&mut self, lo: f32, hi: f32) {
        self.hi.set(hi);
        self.lo.set(lo);
    }
    fn run(&mut self, x: f32) -> f32 {
        let h = self.hi.run(x);
        h - self.lo.run(h)
    }
}

/// Two-pole resonator: rings at one frequency, like a vowel formant.
struct Resonator {
    b1: f32,
    b2: f32,
    y1: f32,
    y2: f32,
    gain: f32,
}

impl Resonator {
    fn new(freq: f32, bandwidth: f32) -> Resonator {
        let r = (-PI * bandwidth / RATE).exp();
        let b1 = 2.0 * r * (TAU * freq / RATE).cos();
        Resonator {
            b1,
            b2: -r * r,
            y1: 0.0,
            y2: 0.0,
            gain: 1.0 - r,
        }
    }
    fn run(&mut self, x: f32) -> f32 {
        let y = x * self.gain + self.b1 * self.y1 + self.b2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn samples(dur: f32) -> usize {
    (dur * RATE) as usize
}

/// Fade both ends so nothing clicks.
fn edges(v: &mut [f32], fade: f32) {
    let n = v.len();
    let f = samples(fade).min(n / 2);
    for k in 0..f {
        let g = k as f32 / f as f32;
        v[k] *= g;
        v[n - 1 - k] *= g;
    }
}

/// A dry wooden knock, the building block of bone rattles.
fn knock(out: &mut [f32], at: usize, freq: f32, amp: f32, rng: &mut Rng) {
    let mut r = Resonator::new(freq, 160.0);
    let mut lp = LowPass::new(1200.0);
    for k in 0..samples(0.05) {
        let Some(o) = out.get_mut(at + k) else { break };
        let t = k as f32 / RATE;
        let x = if k < 60 { rng.noise() } else { 0.0 };
        *o += lp.run(r.run(x) * 6.0) * amp * (-t / 0.012).exp();
    }
}

fn cave_drone(rng: &mut Rng) -> Vec<f32> {
    let dur = 6.0;
    let n = samples(dur);
    let f = rng.range(38.0, 52.0);
    let mut band = Band::new(70.0, 260.0);
    let mut swell = LowPass::new(0.6);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            let env = (k * PI).sin().powf(1.5);
            let mov = 1.0 + (swell.run(rng.noise()) * 40.0).clamp(-0.3, 0.3);
            let tone = (t * f * TAU).sin() * 0.5
                + (t * f * 1.498 * TAU).sin() * 0.25
                + (t * f * 0.5 * TAU).sin() * 0.3;
            (tone * mov + band.run(rng.noise()) * 1.2) * env * 0.7
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn cave_rumble(rng: &mut Rng) -> Vec<f32> {
    let dur = 2.4;
    let mut out = vec![0.0; samples(dur + 0.1)];
    let mut low = LowPass::new(160.0);
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE;
        *o = low.run(rng.noise())
            * 2.5
            * (t / 0.2).min(1.0)
            * (1.0 - t / (dur + 0.1)).max(0.0).powf(2.0);
    }
    // Stones knocking as they roll, fewer and softer as it settles.
    for i in 0..18 {
        let t = (i as f32 / 18.0).powf(1.4) * dur;
        knock(
            &mut out,
            samples(t),
            rng.range(180.0, 420.0),
            0.5 * (1.0 - t / dur),
            rng,
        );
    }
    edges(&mut out, 0.02);
    out
}

/// One bubble-like tone whose pitch rises as it closes.
fn plop(out: &mut [f32], start: usize, f0: f32, amp: f32, len: f32) {
    let mut phase = 0.0f32;
    for k in 0..samples(len) {
        let Some(o) = out.get_mut(start + k) else {
            break;
        };
        let t = k as f32 / RATE;
        phase += f0 * (1.0 + 1.8 * t / len) / RATE;
        *o += (phase * TAU).sin() * amp * (-t / (len * 0.3)).exp() * (t * 2000.0).min(1.0);
    }
}

/// A fish falling back into the water: a short burst of dull spray over a low "plop", and a
/// couple of drops after. All of it kept low (nothing over about 1.5 kHz).
fn fish_splash(rng: &mut Rng) -> Vec<f32> {
    let dur = 0.55;
    let mut out = vec![0.0; samples(dur)];
    let mut lp = LowPass::new(rng.range(700.0, 1000.0));
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE;
        *o += lp.run(rng.noise()) * 2.2 * (t / 0.004).min(1.0) * (-t / 0.045).exp();
    }
    plop(&mut out, samples(0.01), rng.range(150.0, 230.0), 0.45, 0.14);
    for _ in 0..2 {
        let at = samples(rng.range(0.12, 0.3));
        plop(&mut out, at, rng.range(260.0, 380.0), 0.07, 0.05);
    }
    let mut soft = LowPass::new(1500.0);
    out.iter_mut().for_each(|x| *x = soft.run(*x));
    edges(&mut out, 0.02);
    out
}

/// A wooden reel: three or four soft ratchet clicks, low and woody (no metal ring).
fn reel(rng: &mut Rng) -> Vec<f32> {
    let dur = 0.24;
    let mut out = vec![0.0; samples(dur)];
    let n = 3 + (rng.next() * 2.0) as usize;
    let gap = rng.range(0.045, 0.06);
    for i in 0..n {
        let at = samples(0.01 + i as f32 * gap + rng.range(0.0, 0.006));
        knock(&mut out, at, rng.range(340.0, 470.0), rng.range(0.35, 0.5), rng);
    }
    // The line rubbing over the spool under it.
    let mut lp = LowPass::new(500.0);
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE;
        *o += lp.run(rng.noise()) * 0.12 * (t / dur * PI).sin();
    }
    edges(&mut out, 0.01);
    out
}

/// A fish tapping the float: a very small, low plop with a breath of spray.
fn nibble(rng: &mut Rng) -> Vec<f32> {
    let mut out = vec![0.0; samples(0.22)];
    plop(&mut out, samples(0.005), rng.range(200.0, 290.0), 0.35, 0.06);
    let mut lp = LowPass::new(800.0);
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE;
        *o += lp.run(rng.noise()) * 0.5 * (t / 0.003).min(1.0) * (-t / 0.02).exp();
    }
    let mut soft = LowPass::new(1200.0);
    out.iter_mut().for_each(|x| *x = soft.run(*x));
    edges(&mut out, 0.01);
    out
}

/// The line giving way: a dull slap, a low thud of the rod springing back, a little rush of air.
fn line_snap(rng: &mut Rng) -> Vec<f32> {
    let dur = 0.4;
    let mut out = vec![0.0; samples(dur)];
    let mut lp = LowPass::new(900.0);
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE;
        *o += lp.run(rng.noise()) * 1.6 * (t / 0.002).min(1.0) * (-t / 0.03).exp();
    }
    knock(&mut out, samples(0.01), rng.range(150.0, 200.0), 0.8, rng);
    let air = whoosh(rng, 0.22, 200.0, 500.0, 0.25);
    for (k, a) in air.iter().enumerate() {
        if let Some(o) = out.get_mut(samples(0.05) + k) {
            *o += a;
        }
    }
    let mut soft = LowPass::new(1200.0);
    out.iter_mut().for_each(|x| *x = soft.run(*x));
    edges(&mut out, 0.01);
    out
}

/// Wingbeats of a few birds flying over: each beat a soft, low push of air, the birds slightly
/// out of step, swelling as they come over and fading as they go.
fn wings(rng: &mut Rng) -> Vec<f32> {
    let dur = rng.range(2.2, 3.0);
    let n = samples(dur);
    let mut out = vec![0.0; n];
    let birds = 3 + (rng.next() * 3.0) as usize;
    for _ in 0..birds {
        let rate = rng.range(4.5, 6.5);
        let mut t = rng.range(0.0, 0.3);
        let amp = rng.range(0.5, 1.0);
        while t < dur - 0.1 {
            let mut band = Band::new(120.0, rng.range(450.0, 650.0));
            let at = samples(t);
            for k in 0..samples(0.09) {
                let Some(o) = out.get_mut(at + k) else { break };
                let s = k as f32 / RATE;
                *o += band.run(rng.noise()) * amp * (s / 0.012).min(1.0) * (-s / 0.03).exp();
            }
            t += 1.0 / rate * rng.range(0.9, 1.1);
            // Now and then a bird glides for a moment.
            if rng.next() < 0.12 {
                t += rng.range(0.3, 0.6);
            }
        }
    }
    for (i, o) in out.iter_mut().enumerate() {
        *o *= (i as f32 / n as f32 * PI).sin().powf(1.2) * 1.6;
    }
    edges(&mut out, 0.05);
    out
}

fn drip(rng: &mut Rng) -> Vec<f32> {
    let mut out = vec![0.0; samples(0.5)];
    plop(&mut out, 0, rng.range(330.0, 520.0), 0.5, 0.08);
    // A softer echo of it off the cave walls.
    plop(&mut out, samples(0.11), rng.range(300.0, 450.0), 0.12, 0.08);
    let mut lp = LowPass::new(1000.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x));
    out
}

fn wind(rng: &mut Rng, strength: f32, leaves: bool) -> Vec<f32> {
    let dur = rng.range(3.5, 5.0);
    let n = samples(dur);
    let mut band = Band::new(70.0, 400.0);
    let mut gust = LowPass::new(0.8);
    let mut leaf = Band::new(300.0, 1400.0);
    let mut leaf_amp = LowPass::new(8.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            let g = (gust.run(rng.noise()) * 50.0).clamp(-1.0, 1.0) * 0.5 + 0.5;
            let lift = 0.3 + 0.7 * g;
            // Stronger gusts open the wind up a little, but it never whistles.
            band.set(70.0, 260.0 + 360.0 * strength * lift);
            let env = (k * PI).sin().powf(0.8);
            let mut y = band.run(rng.noise()) * (0.6 + 1.6 * strength) * lift;
            if leaves {
                let flutter = (leaf_amp.run(rng.noise()) * 12.0).abs().min(1.0);
                y = y * 0.4 + leaf.run(rng.noise()) * flutter * 1.3 * lift;
            }
            y * env * 1.5
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn waves(rng: &mut Rng) -> Vec<f32> {
    let dur = rng.range(4.5, 6.0);
    let n = samples(dur);
    let crest = rng.range(0.35, 0.45);
    let mut lp = LowPass::new(300.0);
    let mut floor = LowPass::new(60.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            // Builds as it rolls in, breaks, then washes out with a brighter fizz.
            let env = if k < crest {
                (k / crest).powf(2.0)
            } else {
                (1.0 - (k - crest) / (1.0 - crest)).powf(1.2)
            };
            let cut = if k < crest {
                220.0 + 300.0 * k / crest
            } else {
                520.0 + 380.0 * (1.0 - (k - crest) / (1.0 - crest)).powf(3.0)
            };
            lp.set(cut);
            let x = lp.run(rng.noise());
            (x - floor.run(x) * 0.6) * env * 2.2
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn lapping(rng: &mut Rng) -> Vec<f32> {
    let dur = 1.6;
    let mut out = vec![0.0; samples(dur)];
    for _ in 0..rng.range(3.0, 5.0) as usize {
        let at = samples(rng.range(0.0, dur - 0.35));
        let mut lp = LowPass::new(rng.range(450.0, 700.0));
        let len = rng.range(0.15, 0.3);
        for k in 0..samples(len) {
            let t = k as f32 / RATE;
            let e = (t / 0.02).min(1.0) * (1.0 - t / len).powf(2.0);
            out[at + k] += lp.run(rng.noise()) * e * 1.5;
        }
        plop(
            &mut out,
            at + samples(0.03),
            rng.range(220.0, 340.0),
            0.08,
            0.07,
        );
    }
    edges(&mut out, 0.02);
    out
}

fn lava_pop(rng: &mut Rng) -> Vec<f32> {
    let mut out = vec![0.0; samples(0.6)];
    // A thick bubble bursting: a low plop and a short sputter.
    plop(&mut out, 0, rng.range(90.0, 140.0), 0.8, 0.14);
    let mut lp = LowPass::new(900.0);
    let at = samples(0.05);
    for k in 0..samples(0.3) {
        let t = k as f32 / RATE;
        let x = if rng.next() < 0.04 {
            rng.noise() * 3.0
        } else {
            0.0
        };
        out[at + k] += lp.run(x) * (-t / 0.1).exp() * 0.8;
    }
    edges(&mut out, 0.004);
    out
}

fn crackle(rng: &mut Rng) -> Vec<f32> {
    let dur = 1.6;
    let n = samples(dur);
    let mut roar = Band::new(60.0, 300.0);
    let mut out: Vec<f32> = (0..n).map(|_| roar.run(rng.noise()) * 0.5).collect();
    for _ in 0..30 {
        let at = samples(rng.range(0.0, dur - 0.05));
        knock(
            &mut out,
            at,
            rng.range(500.0, 1100.0),
            rng.range(0.15, 0.45),
            rng,
        );
    }
    edges(&mut out, 0.05);
    out
}

fn nether_drone(rng: &mut Rng) -> Vec<f32> {
    let dur = 7.0;
    let n = samples(dur);
    let f = rng.range(34.0, 46.0);
    let mut moan = Resonator::new(rng.range(260.0, 360.0), 60.0);
    let mut breath = LowPass::new(500.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            let env = (k * PI).sin().powf(2.0);
            let bend = 1.0 + 0.05 * (t * 0.35 * TAU).sin();
            let tone = (t * f * bend * TAU).sin() * 0.6 + (t * f * 2.01 * TAU).sin() * 0.2;
            let m = moan.run(breath.run(rng.noise())) * 9.0;
            (tone + m) * env * 0.7
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn soul_wind(rng: &mut Rng) -> Vec<f32> {
    let dur = 5.0;
    let n = samples(dur);
    let mut r1 = Resonator::new(rng.range(180.0, 230.0), 25.0);
    let mut r2 = Resonator::new(rng.range(280.0, 330.0), 30.0);
    let mut lp = LowPass::new(700.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            let env = (k * PI).sin().powf(1.5) * (0.7 + 0.3 * (t * 0.7 * TAU).sin());
            let x = lp.run(rng.noise());
            (r1.run(x) * 14.0 + r2.run(x) * 9.0) * env
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn end_hum(rng: &mut Rng) -> Vec<f32> {
    let dur = 6.0;
    let n = samples(dur);
    let f = rng.range(55.0, 70.0);
    let mut air = Band::new(80.0, 240.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            let env = (k * PI).sin().powf(1.8);
            // Two tones a hair apart beat slowly against each other.
            let tone = (t * f * TAU).sin() * 0.4
                + (t * (f + 0.7) * TAU).sin() * 0.4
                + (t * f * 1.5 * TAU).sin() * 0.12;
            (tone + air.run(rng.noise()) * 0.9) * env * 0.7
        })
        .collect();
    edges(&mut out, 0.05);
    out
}

fn heartbeat() -> Vec<f32> {
    let mut out = vec![0.0; samples(0.6)];
    for (start, amp, f) in [(0.0, 1.0, 55.0), (0.24, 0.7, 50.0)] {
        let at = samples(start);
        for k in 0..samples(0.18) {
            let t = k as f32 / RATE;
            out[at + k] +=
                (t * f * (1.0 - t) * TAU).sin() * amp * (t / 0.005).min(1.0) * (-t / 0.05).exp();
        }
    }
    edges(&mut out, 0.004);
    out
}

fn gasp(rng: &mut Rng) -> Vec<f32> {
    let dur = 0.75;
    let n = samples(dur);
    let mut band = Band::new(300.0, 1200.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let k = t / dur;
            // A sharp pull of air, then a slower breath out.
            let env = if k < 0.35 {
                (k / 0.35).powf(0.6)
            } else {
                (1.0 - (k - 0.35) / 0.65).powf(1.5) * 0.7
            };
            band.set(300.0, if k < 0.35 { 1200.0 } else { 800.0 });
            band.run(rng.noise()) * env * 1.8
        })
        .collect();
    edges(&mut out, 0.01);
    out
}

fn land(rng: &mut Rng, strength: f32) -> Vec<f32> {
    let dur = 0.25 + strength * 0.2;
    let n = samples(dur);
    let mut lp = LowPass::new(260.0 + strength * 200.0);
    let f = rng.range(48.0, 60.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let thump = (t * f * TAU).sin() * (-t / (0.05 + strength * 0.05)).exp();
            let dirt = lp.run(rng.noise()) * (-t / 0.04).exp() * 2.0;
            (thump + dirt) * (0.5 + strength * 0.5)
        })
        .collect();
    edges(&mut out, 0.004);
    out
}

/// A thrown blade: a whoosh that flutters as it spins (a knife about 14 turns a second), or for
/// a heavy throw a lower, smooth swish.
fn throw(rng: &mut Rng, heavy: bool) -> Vec<f32> {
    let dur = if heavy { 0.32 } else { rng.range(0.26, 0.34) };
    let n = samples(dur);
    let (lo, hi) = if heavy { (110.0, 380.0) } else { (180.0, 620.0) };
    let spin = rng.range(12.0, 17.0);
    let mut band = Band::new(lo * 0.5, lo);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let t = i as f32 / RATE;
            let env = if k < 0.25 { k / 0.25 } else { (1.0 - k) / 0.75 };
            band.set(lo * 0.4, lo + (hi - lo) * env);
            let flutter = if heavy { 1.0 } else { 0.55 + 0.45 * (t * spin * TAU).sin().abs() };
            band.run(rng.noise()) * env.powf(1.3) * flutter * 1.6
        })
        .collect();
    edges(&mut out, 0.004);
    out
}

/// A stab: a low thump (the hit through cloth and flesh) with a short, muffled tear on top.
fn stab(rng: &mut Rng) -> Vec<f32> {
    let n = samples(0.2);
    let f = rng.range(72.0, 92.0);
    let mut lp = LowPass::new(520.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let a = (t / 0.003).min(1.0);
            let thump = (t * f * TAU).sin() * (-t / 0.045).exp();
            let tear = lp.run(rng.noise()) * (-t / 0.03).exp() * 1.8;
            (thump * 0.9 + tear) * a * 0.8
        })
        .collect();
    edges(&mut out, 0.004);
    out
}

/// A smoke bomb: a soft burst of air that rolls out and dies away, over a faint low rumble.
fn smoke_puff(rng: &mut Rng) -> Vec<f32> {
    let n = samples(0.9);
    let mut lp = LowPass::new(480.0);
    let mut low = LowPass::new(90.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let a = (t / 0.012).min(1.0);
            let air = lp.run(rng.noise()) * (-t / 0.22).exp() * 2.0;
            let roll = low.run(rng.noise()) * (-t / 0.4).exp() * 6.0;
            (air + roll) * a * 0.7
        })
        .collect();
    edges(&mut out, 0.01);
    out
}

/// A slow breath out through the nose: dark, quiet, swelling and fading.
fn hush(rng: &mut Rng) -> Vec<f32> {
    let n = samples(0.6);
    let mut lp = LowPass::new(340.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            lp.run(rng.noise()) * (k * PI).sin().powf(1.8) * 1.4
        })
        .collect();
    edges(&mut out, 0.01);
    out
}

/// Boot soles gliding over ice: a hushed scrape that swells and fades, with a faint low hum
/// of the ice under it. Kept dark so it never hisses.
fn slide(rng: &mut Rng, speed: f32) -> Vec<f32> {
    let dur = 0.4 + speed * 0.15;
    let n = samples(dur);
    let mut lp = LowPass::new(420.0 + speed * 260.0);
    let hum = rng.range(70.0, 82.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let t = i as f32 / RATE;
            let env = (k * PI).sin().powf(1.5);
            let scrape = lp.run(rng.noise()) * 2.2;
            let under = (t * hum * TAU).sin() * 0.25;
            (scrape + under) * env * (0.3 + speed * 0.35)
        })
        .collect();
    edges(&mut out, 0.01);
    out
}

/// Air moving: a band of noise sweeping up and back down.
fn whoosh(rng: &mut Rng, dur: f32, lo: f32, hi: f32, gain: f32) -> Vec<f32> {
    let n = samples(dur);
    let mut band = Band::new(lo * 0.5, lo);
    let peak = rng.range(0.4, 0.55);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let s = if k < peak {
                k / peak
            } else {
                1.0 - (k - peak) / (1.0 - peak)
            };
            band.set(lo * 0.4, lo + (hi - lo) * s);
            band.run(rng.noise()) * s.powf(1.5) * gain * 3.0
        })
        .collect();
    edges(&mut out, 0.004);
    out
}

/// A whip: the rope's swish rising, then the snap at the tip. A real crack is a tiny sonic
/// boom; here it is a dark, short smack (no hiss), with a leathery thud when it lands on hide.
fn whip_crack(rng: &mut Rng, hit: bool) -> Vec<f32> {
    let mut out = whoosh(rng, 0.14, 180.0, 520.0, 0.35);
    let snap_at = out.len();
    out.extend(std::iter::repeat_n(0.0, samples(0.18)));
    let mut r = Resonator::new(rng.range(300.0, 360.0), 220.0);
    let mut lp = LowPass::new(1100.0);
    let body = rng.range(95.0, 120.0);
    for k in 0..samples(0.16) {
        let t = k as f32 / RATE;
        let burst = if k < 90 { rng.noise() } else { 0.0 };
        let snap = lp.run(r.run(burst) * 7.0) * (-t / 0.018).exp();
        let thud = if hit { (t * body * TAU).sin() * (-t / 0.035).exp() * 0.6 } else { 0.0 };
        if let Some(o) = out.get_mut(snap_at.saturating_sub(samples(0.01)) + k) {
            *o += (snap + thud) * 0.9;
        }
    }
    edges(&mut out, 0.004);
    out
}

/// A blade biting into a body: a short, damp thud of flesh and cloth under it, and on top what
/// the weapon is made of. Wood and stone knock; metal adds a brief low ring that the body
/// muffles at once; crystal a soft glassy chime; magic a low "whumf" of air. All of it kept
/// dark (nothing above about a kilohertz rings), as HP dislikes shrill and synthetic sounds.
fn bite(rng: &mut Rng, kind: u8) -> Vec<f32> {
    let dur = match kind {
        2 => 0.32,
        3 => 0.3,
        _ => 0.2,
    };
    let n = samples(dur);
    let mut body = LowPass::new(rng.range(380.0, 460.0));
    let mut cut = Band::new(320.0, 900.0);
    let thud = rng.range(78.0, 96.0);
    // Ringing partials of the blade, not quite harmonic so it sounds like metal, not a tone.
    let base = rng.range(290.0, 340.0);
    let ring = [(base, 1.0), (base * 1.51, 0.55), (base * 2.27, 0.3)];
    let glass = rng.range(560.0, 640.0);
    let mut air = LowPass::new(300.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let flesh = body.run(rng.noise()) * (-t / 0.035).exp() * 2.2 + (t * thud * TAU).sin() * (-t / 0.05).exp() * 0.7;
            let edge = cut.run(rng.noise()) * (-t / 0.012).exp() * 0.8;
            let extra = match kind {
                0 => (t * thud * 1.8 * TAU).sin() * (-t / 0.03).exp() * 0.5,
                1 => ring.iter().map(|&(f, a)| (t * f * TAU).sin() * a).sum::<f32>() * (-t / 0.06).exp() * 0.22,
                2 => ((t * glass * TAU).sin() + (t * glass * 1.5 * TAU).sin() * 0.4) * (-t / 0.11).exp() * 0.2,
                _ => {
                    let f = 140.0 - 60.0 * (t / dur);
                    (t * f * TAU).sin() * (t * PI / dur).sin() * 0.5 + air.run(rng.noise()) * (t * PI / dur).sin() * 1.4
                }
            };
            (flesh + if kind == 0 { 0.0 } else { edge } + extra) * 0.6
        })
        .collect();
    edges(&mut out, 0.004);
    out
}

fn brush(rng: &mut Rng) -> Vec<f32> {
    let dur = rng.range(0.28, 0.38);
    let n = samples(dur);
    let mut band = Band::new(220.0, 950.0);
    let mut flutter = LowPass::new(25.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            let f = (flutter.run(rng.noise()) * 8.0).abs().min(1.0);
            band.run(rng.noise()) * (k * PI).sin() * (0.4 + f) * 1.4
        })
        .collect();
    edges(&mut out, 0.005);
    out
}

fn flap(rng: &mut Rng, open: bool) -> Vec<f32> {
    let dur = 0.16;
    let n = samples(dur);
    let mut lp = LowPass::new(if open { 750.0 } else { 600.0 });
    let f = if open { 140.0 } else { 115.0 };
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            let e = (t / 0.004).min(1.0) * (-t / 0.03).exp();
            (lp.run(rng.noise()) * 1.4 + (t * f * TAU).sin() * 0.5) * e * 0.7
        })
        .collect();
    edges(&mut out, 0.003);
    out
}

fn armor(rng: &mut Rng, metal: bool) -> Vec<f32> {
    let dur = 0.22;
    let mut out = vec![0.0; samples(dur)];
    if metal {
        // A few soft, low clinks of plates touching.
        for _ in 0..3 {
            let at = samples(rng.range(0.0, 0.08));
            let f = rng.range(320.0, 520.0);
            for k in 0..samples(0.12) {
                let Some(o) = out.get_mut(at + k) else { break };
                let t = k as f32 / RATE;
                *o += ((t * f * TAU).sin() * 0.25 + (t * f * 1.41 * TAU).sin() * 0.1)
                    * (-t / 0.03).exp();
            }
        }
    } else {
        let mut lp = LowPass::new(650.0);
        for (i, o) in out.iter_mut().enumerate() {
            let t = i as f32 / RATE;
            *o = lp.run(rng.noise()) * (t / dur * PI).sin() * 0.8;
        }
    }
    let mut lp = LowPass::new(1100.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x));
    edges(&mut out, 0.004);
    out
}

/// A plucked string (Karplus-Strong): a burst of soft noise running round a delay line as
/// long as one period, each pass a little duller, so it rings like gut or sinew, not a beep.
fn string_pluck(out: &mut [f32], at: usize, freq: f32, amp: f32, decay: f32, rng: &mut Rng) {
    let period = (RATE / freq) as usize;
    let mut lp = LowPass::new(700.0);
    let mut line: Vec<f32> = (0..period).map(|_| lp.run(rng.noise())).collect();
    let mut k = 0;
    for i in 0..samples(decay * 4.0) {
        let Some(o) = out.get_mut(at + i) else { break };
        let next = line[(k + 1) % period];
        let y = (line[k] + next) * 0.5 * 0.996;
        line[k] = y;
        k = (k + 1) % period;
        *o += y * amp * (-(i as f32 / RATE) / decay).exp();
    }
}

fn bow_draw(rng: &mut Rng, weight: u8) -> Vec<f32> {
    let w = weight as f32;
    let dur = 0.42 + 0.1 * w;
    let mut out = vec![0.0; samples(dur)];
    // Stick-slip of the limbs and the string in the nock: grains coming faster as it bends,
    // through the wood's low body.
    let mut body = Resonator::new(230.0 - 40.0 * w, 90.0);
    let mut body2 = Resonator::new(470.0 - 70.0 * w, 140.0);
    let mut t = rng.range(0.0, 0.02);
    while t < dur - 0.03 {
        let k = t / dur;
        let at = samples(t);
        let len = samples(0.004);
        for i in 0..len {
            if let Some(o) = out.get_mut(at + i) {
                *o += rng.noise() * (1.0 - i as f32 / len as f32) * (0.4 + 0.6 * k);
            }
        }
        t += rng.range(0.6, 1.4) / (25.0 + 55.0 * k);
    }
    let mut lp = LowPass::new(900.0);
    for o in out.iter_mut() {
        let x = *o;
        *o = lp.run(body.run(x) * 9.0 + body2.run(x) * 5.0) * 0.9;
    }
    edges(&mut out, 0.01);
    out
}

fn bow_release(rng: &mut Rng, weight: u8, crossbow: bool) -> Vec<f32> {
    let w = weight as f32;
    let dur = 0.5;
    let mut out = vec![0.0; samples(dur)];
    // The string's thrum, low and short as the limbs damp it.
    let f = if crossbow { 82.0 } else { rng.range(118.0, 132.0) - 14.0 * w };
    string_pluck(&mut out, 0, f, 0.9, if crossbow { 0.06 } else { 0.09 }, rng);
    // The limbs (or the crossbow's stock) knocking forward.
    knock(&mut out, 0, if crossbow { 120.0 } else { 150.0 - 20.0 * w }, if crossbow { 1.0 } else { 0.55 }, rng);
    if crossbow {
        knock(&mut out, samples(0.004), 260.0, 0.5, rng);
    }
    // The arrow leaving, a short breath of air.
    let air = whoosh(rng, 0.14, 180.0, 520.0, 0.18);
    for (o, a) in out.iter_mut().zip(air) {
        *o += a;
    }
    let mut lp = LowPass::new(1000.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x) * 1.2);
    edges(&mut out, 0.003);
    out
}

fn arrow_hit(rng: &mut Rng, surface: u8) -> Vec<f32> {
    let dur = if surface == 0 { 0.45 } else { 0.3 };
    let mut out = vec![0.0; samples(dur)];
    match surface {
        0 => {
            // A solid thunk, then the shaft quivering in the wood.
            knock(&mut out, 0, 120.0, 1.0, rng);
            knock(&mut out, 0, 210.0, 0.4, rng);
            let f = rng.range(150.0, 175.0);
            let wob = rng.range(18.0, 24.0);
            for (i, o) in out.iter_mut().enumerate().skip(samples(0.01)) {
                let t = i as f32 / RATE;
                *o += (t * f * TAU).sin() * (t * wob * TAU).sin().abs() * 0.3 * (-t / 0.1).exp();
            }
        }
        1 => {
            // A dull clack on stone and the shaft skittering.
            knock(&mut out, 0, 260.0, 0.9, rng);
            knock(&mut out, 0, 410.0, 0.45, rng);
            for _ in 0..3 {
                let at = samples(rng.range(0.03, 0.12));
                knock(&mut out, at, rng.range(300.0, 420.0), 0.2, rng);
            }
        }
        2 => {
            // Into earth: a soft, gritty thud.
            let mut lp = LowPass::new(420.0);
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                *o = lp.run(rng.noise()) * 2.5 * (-t / 0.035).exp() + (t * 70.0 * TAU).sin() * 0.5 * (-t / 0.05).exp();
            }
        }
        3 => {
            // Into a body: a deep, wet thump.
            let mut lp = LowPass::new(320.0);
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                *o = lp.run(rng.noise()) * 2.0 * (-t / 0.05).exp() + (t * 62.0 * TAU).sin() * 0.7 * (-t / 0.06).exp();
            }
        }
        _ => {
            // Glancing off metal: a low clink that rings a little.
            let f = rng.range(300.0, 360.0);
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                *o = ((t * f * TAU).sin() * 0.35 + (t * f * 1.47 * TAU).sin() * 0.15) * (-t / 0.07).exp();
            }
            knock(&mut out, 0, 200.0, 0.4, rng);
        }
    }
    let mut lp = LowPass::new(1100.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x));
    edges(&mut out, 0.003);
    out
}

fn crossbow_load(rng: &mut Rng) -> Vec<f32> {
    let dur = 0.55;
    let mut out = vec![0.0; samples(dur)];
    // The cord dragged back: low ratchet clicks, then the latch snapping shut.
    for k in 0..4 {
        let at = samples(0.03 + k as f32 * rng.range(0.085, 0.1));
        knock(&mut out, at, rng.range(330.0, 380.0), 0.45, rng);
    }
    knock(&mut out, samples(0.44), 180.0, 0.9, rng);
    knock(&mut out, samples(0.441), 300.0, 0.4, rng);
    let mut lp = LowPass::new(1000.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x) * 1.3);
    edges(&mut out, 0.003);
    out
}

/// A tool breaking: a hard transient (crack, split, snap) over the material's own body, all
/// kept low (HP: nothing high-pitched).
fn tool_break(rng: &mut Rng, mat: u8) -> Vec<f32> {
    let dur = 0.45;
    let n = samples(dur);
    let mut out = vec![0.0; n];
    // The snap itself: a short burst of noise, darker for wood and stone.
    let mut lp = LowPass::new(match mat {
        0 => 900.0,
        1 => 700.0,
        _ => 1100.0,
    });
    for (i, o) in out.iter_mut().enumerate().take(samples(0.06)) {
        let t = i as f32 / RATE;
        *o += lp.run(rng.noise()) * 2.6 * (-t / 0.012).exp();
    }
    match mat {
        0 => {
            // Splinters: a few small cracks just after.
            for _ in 0..4 {
                let at = samples(rng.range(0.01, 0.09));
                let mut b = Band::new(300.0, 900.0);
                for k in 0..samples(0.03) {
                    let Some(o) = out.get_mut(at + k) else { break };
                    let t = k as f32 / RATE;
                    *o += b.run(rng.noise()) * 1.4 * (-t / 0.006).exp();
                }
            }
        }
        1 => {
            // A heavy chunk and grit falling.
            let f = rng.range(70.0, 90.0);
            let mut grit = LowPass::new(500.0);
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                *o += (t * f * TAU).sin() * 0.6 * (-t / 0.05).exp() + grit.run(rng.noise()) * 0.9 * (-t / 0.12).exp();
            }
        }
        _ => {
            // A struck bar: two low partials that die away (gems a little longer and purer).
            let gem = mat == 3;
            let f = if gem { rng.range(520.0, 600.0) } else { rng.range(330.0, 420.0) };
            let decay = if gem { 0.16 } else { 0.08 };
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                let ring = (t * f * TAU).sin() * 0.35 + (t * f * if gem { 2.0 } else { 1.47 } * TAU).sin() * 0.12;
                *o += ring * (-t / decay).exp();
            }
        }
    }
    let mut lp = LowPass::new(1200.0);
    out.iter_mut().for_each(|x| *x = lp.run(*x) * 0.8);
    edges(&mut out, 0.004);
    out
}

/// A powered tool at work: a low engine note with its harmonics (rougher for the chainsaw's
/// two-stroke) and the grinding of the bit or chain in the block, with a little flutter.
fn motor(rng: &mut Rng, drill: bool) -> Vec<f32> {
    let dur = 0.3;
    let n = samples(dur);
    let f = if drill { rng.range(92.0, 100.0) } else { rng.range(62.0, 70.0) };
    let mut grind = Band::new(if drill { 260.0 } else { 180.0 }, if drill { 700.0 } else { 520.0 });
    let mut tone = LowPass::new(if drill { 650.0 } else { 520.0 });
    let mut ph = 0.0f32;
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE;
            ph = (ph + f * (1.0 + 0.02 * (t * 23.0 * TAU).sin()) / RATE) % 1.0;
            // A rounded saw (the engine), filtered dark.
            let saw = 2.0 * ph - 1.0;
            let buzz = tone.run(saw) * if drill { 0.9 } else { 1.1 };
            let chug = if drill { 1.0 } else { 0.75 + 0.25 * (t * 18.0 * TAU).sin() };
            (buzz * chug + grind.run(rng.noise()) * 0.9) * 0.5
        })
        .collect();
    edges(&mut out, 0.02);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brightness(v: &[f32]) -> f32 {
        let zc = v
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();
        zc as f32 / (v.len() as f32 / RATE) / 2.0
    }

    pub fn every_fx() -> Vec<Fx> {
        let all = vec![
            Fx::CaveDrone,
            Fx::CaveRumble,
            Fx::Drip,
            Fx::Wind(0.1),
            Fx::Wind(1.0),
            Fx::Canopy,
            Fx::Waves,
            Fx::Lapping,
            Fx::LavaPop,
            Fx::FireCrackle,
            Fx::NetherDrone,
            Fx::SoulWind,
            Fx::EndHum,
            Fx::Heartbeat,
            Fx::Gasp,
            Fx::Land(0.0),
            Fx::Land(1.0),
            Fx::Jump,
            Fx::Swing,
            Fx::Blade(0),
            Fx::Blade(3),
            Fx::Bite(0),
            Fx::Bite(1),
            Fx::Bite(2),
            Fx::Bite(3),
            Fx::Brush,
            Fx::InvOpen,
            Fx::InvClose,
            Fx::Armor(true),
            Fx::Armor(false),
            Fx::Slide(0.0),
            Fx::Slide(1.0),
            Fx::DryFire,
            Fx::Throw(false),
            Fx::Throw(true),
            Fx::Stab,
            Fx::Smoke,
            Fx::Hush,
            Fx::Dash,
            Fx::BowDraw(0),
            Fx::BowDraw(2),
            Fx::BowRelease(0),
            Fx::BowRelease(2),
            Fx::ArrowHit(0),
            Fx::ArrowHit(1),
            Fx::ArrowHit(2),
            Fx::ArrowHit(3),
            Fx::ArrowHit(4),
            Fx::CrossbowLoad,
            Fx::CrossbowShot,
            Fx::WhipCrack(false),
            Fx::WhipCrack(true),
            Fx::ToolBreak(0),
            Fx::ToolBreak(1),
            Fx::ToolBreak(2),
            Fx::ToolBreak(3),
            Fx::Motor(true),
            Fx::Motor(false),
            Fx::Reel,
            Fx::Nibble,
            Fx::LineSnap,
        ];
        let mut all = all;
        all.extend((0..=16).map(Fx::Gun));
        all
    }

    #[test]
    fn extra_sounds_are_low_and_clean() {
        for fx in every_fx() {
            for seed in [1, 7, 99] {
                let v = synth(fx, seed);
                assert!(!v.is_empty() && v.iter().all(|x| x.is_finite()), "{fx:?}");
                let peak = v.iter().fold(0.0f32, |a, x| a.max(x.abs()));
                assert!(peak > 0.02, "{fx:?} is silent ({peak})");
                let b = brightness(&v);
                assert!(b < 1300.0, "{fx:?} too bright: {b} Hz");
                // Nothing should end on a click.
                assert!(v.last().unwrap().abs() < 0.05, "{fx:?} ends abruptly");
            }
        }
    }

    /// Writes every sound to WAV files for listening: MC_SOUNDS_OUT=dir cargo test -- --ignored
    #[test]
    #[ignore]
    fn export_sounds() {
        let Ok(dir) = std::env::var("MC_SOUNDS_OUT") else {
            return;
        };
        std::fs::create_dir_all(&dir).unwrap();
        for fx in every_fx() {
            let name = format!("{fx:?}")
                .replace(['(', ')', ',', ' ', '.'], "_")
                .replace("__", "_");
            let v = synth(fx, 7);
            let mut b = Vec::new();
            let len = (v.len() * 2) as u32;
            b.extend_from_slice(b"RIFF");
            b.extend_from_slice(&(36 + len).to_le_bytes());
            b.extend_from_slice(b"WAVEfmt ");
            b.extend_from_slice(&16u32.to_le_bytes());
            b.extend_from_slice(&1u16.to_le_bytes());
            b.extend_from_slice(&1u16.to_le_bytes());
            b.extend_from_slice(&44100u32.to_le_bytes());
            b.extend_from_slice(&88200u32.to_le_bytes());
            b.extend_from_slice(&2u16.to_le_bytes());
            b.extend_from_slice(&16u16.to_le_bytes());
            b.extend_from_slice(b"data");
            b.extend_from_slice(&len.to_le_bytes());
            for x in v {
                b.extend_from_slice(&((x * 32000.0) as i16).to_le_bytes());
            }
            std::fs::write(format!("{dir}/{}.wav", name.trim_end_matches('_')), b).unwrap();
        }
    }

    #[test]
    fn every_mob_name_gets_a_sensible_voice() {
        assert_eq!(voice_of("zombie", true), Some(Voice::Groan));
        assert_eq!(voice_of("ash_skeleton", true), Some(Voice::Rattle));
        assert_eq!(voice_of("void_walker_soul", true), Some(Voice::Wail));
        assert_eq!(voice_of("npc_guide", false), Some(Voice::Murmur));
        assert_eq!(voice_of("minecart", false), None);
    }
}
