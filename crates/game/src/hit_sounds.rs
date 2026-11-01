//! Blows landing on bodies and people's voices, from real recordings (sound_bank.rs).
//!
//! When anything is hurt, the blow sounds like what it lands on: flesh thuds, bones clack,
//! slimes squelch, golems clang, wood knocks, stone cracks, ghosts barely rustle. This layer
//! plays with each creature's own cry (mob_sounds.rs, owned by the creature threads), it
//! doesn't replace it. The player and the townsfolk get human voices for hurt and death.

use crate::audio::Sfx;
use crate::block_sounds::{play, Act, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

/// What a body is made of, for the sound of a blow landing on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    Flesh,
    Bone,
    Slime,
    Metal,
    Wood,
    Stone,
    /// Ghosts, wisps, spirits: a soft rustle.
    Spirit,
    /// Insects, spiders, crabs: a hard shell over flesh.
    Shell,
    Snow,
}

/// Whose voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Who {
    Man,
    Woman,
}

/// What the voice does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cry {
    Hurt,
    Death,
}

impl Body {
    #[allow(dead_code)]
    pub const ALL: [Body; 9] = [Body::Flesh, Body::Bone, Body::Slime, Body::Metal, Body::Wood, Body::Stone, Body::Spirit, Body::Shell, Body::Snow];

    fn act(self) -> Act {
        let a = |pick, rate, gain, lp, extra| Act { pick, rate, gain, lp, extra };
        match self {
            Body::Flesh => a(&["impact_flesh", "impact_punch"], 1.0, 0.8, 3500.0, None),
            Body::Bone => a(&["bone_knock"], 1.0, 0.8, 4000.0, Some(("impact_punch", 1.0, 0.35))),
            Body::Slime => a(&["soft_hit", "soft_heavy"], 0.95, 0.85, 3500.0, Some(("mud_step", 0.9, 0.45))),
            Body::Metal => a(&["impact_metal"], 0.85, 0.6, 3200.0, Some(("impact_punch", 0.9, 0.35))),
            Body::Wood => a(&["impact_wood"], 0.9, 0.75, 3500.0, Some(("wood_hit", 0.9, 0.4))),
            Body::Stone => a(&["impact_stone"], 0.8, 0.7, 3000.0, Some(("stone_hit", 0.8, 0.5))),
            Body::Spirit => a(&["cloth_rustle"], 0.8, 0.5, 3000.0, Some(("soft_hit", 0.85, 0.3))),
            Body::Shell => a(&["bone_knock"], 1.08, 0.7, 3800.0, Some(("impact_flesh", 1.0, 0.5))),
            Body::Snow => a(&["snow_hit"], 0.9, 0.8, 3800.0, Some(("impact_punch", 0.95, 0.3))),
        }
    }
}

/// What a mob's body is made of, from its name.
pub fn body_of(name: &str) -> Body {
    let has = |s: &str| name.contains(s);
    if has("skeleton") || matches!(name, "bogged" | "stray" | "bone_king" | "frost_archer") || has("skull") {
        Body::Bone
    } else if has("slime") || has("ooze") || has("magma_cube") || has("sludge") || has("swet") || has("jelly") || has("mimic_cube") {
        Body::Slime
    } else if has("iron_golem") || has("mech_") || has("sentry") || has("saucer") || has("copper_golem") || has("armor") || has("automaton") {
        Body::Metal
    } else if has("snow_golem") {
        Body::Snow
    } else if has("creaking") || has("treant") || has("forest_guardian") || has("ent_") || has("living_wood") {
        Body::Wood
    } else if has("obsidilith") || has("stone_golem") || has("golem") || has("titan") || has("gargoyle") || has("statue") {
        Body::Stone
    } else if has("vex") || has("allay") || has("ghost") || has("wraith") || has("spirit") || has("wisp") || has("phantom") || has("specter") || has("spectral") {
        Body::Spirit
    } else if has("spider") || has("silverfish") || has("endermite") || has("bee") || has("crab") || has("scorpion") || has("beetle") || has("ant") && !has("giant") || has("cockroach") || has("shulker") || has("tarantula") || has("leech") || has("tortoise") || has("turtle") || has("armadillo") {
        Body::Shell
    } else {
        Body::Flesh
    }
}

/// The player's voice: 0 a man's, 1 a woman's, 2 none (settings.rs `player_voice`).
static PLAYER_VOICE: AtomicU8 = AtomicU8::new(0);

pub fn set_player_voice(v: u8) {
    PLAYER_VOICE.store(v, Ordering::Relaxed);
}

pub fn player_voice() -> Option<Who> {
    match PLAYER_VOICE.load(Ordering::Relaxed) {
        0 => Some(Who::Man),
        1 => Some(Who::Woman),
        _ => None,
    }
}

/// The recordings of a voice (the bank holds them split into short grunts and long cries).
fn group(who: Who, cry: Cry) -> &'static str {
    match (who, cry) {
        (Who::Man, Cry::Hurt) => "voice_male_hurt",
        (Who::Man, Cry::Death) => "voice_male_death",
        (Who::Woman, Cry::Hurt) => "voice_female_hurt",
        (Who::Woman, Cry::Death) => "voice_female_death",
    }
}

/// A voice: a random grunt or cry of this person, slightly different each time.
pub fn voice(who: Who, cry: Cry, r: u32) -> Vec<f32> {
    let mut rng = Rng(r.wrapping_mul(0x2C1B_3C6D) | 1);
    let Some(i) = crate::block_sounds::pick(&[group(who, cry)], &mut rng) else {
        return Vec::new();
    };
    let (rate, gain) = match cry {
        Cry::Hurt => (0.97 + 0.08 * rng.f(), 0.55),
        Cry::Death => (0.95 + 0.05 * rng.f(), 0.7),
    };
    crate::block_sounds::render(i, rate, 3800.0, gain)
}

/// A blow on `body`; `strength` 0..255 (stronger: louder and a little deeper).
pub fn impact(body: Body, strength: u8, r: u32) -> Vec<f32> {
    let k = strength as f32 / 255.0;
    let a = body.act();
    let a = Act { rate: a.rate * (1.05 - 0.15 * k), gain: a.gain * (0.55 + 0.45 * k), ..a };
    play(&a, r)
}

/// The player hurt: a blow on flesh and a grunt.
pub fn player_hurt(r: u32) -> Vec<f32> {
    let mut out = impact(Body::Flesh, 150, r);
    if let Some(who) = player_voice() {
        mix(&mut out, &voice(who, Cry::Hurt, r ^ 0x55AA), (0.03 * crate::block_sounds::RATE) as usize);
    }
    out
}

/// The player dies: a heavy blow and a last cry.
pub fn player_death(r: u32) -> Vec<f32> {
    let mut out = impact(Body::Flesh, 230, r);
    if let Some(who) = player_voice() {
        mix(&mut out, &voice(who, Cry::Death, r ^ 0x1234), (0.05 * crate::block_sounds::RATE) as usize);
    }
    out
}

/// Adds `b` into `a` starting `at` samples in.
pub fn mix(a: &mut Vec<f32>, b: &[f32], at: usize) {
    if a.len() < at + b.len() {
        a.resize(at + b.len(), 0.0);
    }
    for (o, s) in a[at..].iter_mut().zip(b) {
        *o += s;
    }
    crate::block_sounds::limit(a);
}

/// Who was hurt last frame: each mob's hurt clock and health, to hear new blows.
#[derive(Default)]
pub struct HitState {
    seen: HashMap<u32, (f32, f32)>,
}

impl crate::Game {
    /// Every creature that was hurt since the last frame gets the sound of the blow on its
    /// body, where it stands (any blow: swords, arrows, spells, falls, fire).
    pub(crate) fn body_impacts(&mut self) {
        if self.audio.is_none() {
            return;
        }
        let eye = self.body.eye();
        let mut blows = Vec::new();
        let mut keep = std::collections::HashSet::new();
        for m in &self.mobs.list {
            let key = m.phase.to_bits();
            keep.insert(key);
            let prev = self.hits.seen.insert(key, (m.hurt_age, m.health));
            let Some((age, health)) = prev else { continue };
            if m.hurt_age < age && m.hurt_age < 0.2 {
                let def = &self.mobs.reg.defs[m.def];
                let at = m.body.pos + glam::Vec3::Y * def.height * 0.6;
                if (at - eye).length() > 24.0 {
                    continue;
                }
                let lost = ((health - m.health) / m.max_health.max(1.0)).clamp(0.0, 1.0);
                let strength = (90.0 + 165.0 * lost.sqrt()) as u8;
                blows.push((body_of(&def.name), strength, at));
            }
        }
        self.hits.seen.retain(|k, _| keep.contains(k));
        for (body, strength, at) in blows.into_iter().take(4) {
            self.sfx_at(Sfx::Impact(body, strength), at, 0.8, 24.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bodies_sound_like_what_they_are() {
        assert_eq!(body_of("skeleton"), Body::Bone);
        assert_eq!(body_of("wither_skeleton"), Body::Bone);
        assert_eq!(body_of("ooze"), Body::Slime);
        assert_eq!(body_of("iron_golem"), Body::Metal);
        assert_eq!(body_of("creaking"), Body::Wood);
        assert_eq!(body_of("spider"), Body::Shell);
        assert_eq!(body_of("zombie"), Body::Flesh);
        assert_eq!(body_of("npc_guide"), Body::Flesh);
        assert_eq!(body_of("vex"), Body::Spirit);
        assert_eq!(body_of("giant"), Body::Flesh);
    }

    #[test]
    fn every_body_sounds_and_stronger_is_louder() {
        for b in Body::ALL {
            let soft: f32 = (0..8).map(|r| impact(b, 20, r).iter().fold(0.0f32, |p, s| p.max(s.abs()))).sum();
            let hard: f32 = (0..8).map(|r| impact(b, 250, r).iter().fold(0.0f32, |p, s| p.max(s.abs()))).sum();
            assert!(soft > 0.1, "{b:?} silent");
            assert!(hard > soft, "{b:?}: hard {hard} soft {soft}");
            for r in 0..4 {
                let v = impact(b, 200, r);
                let hz = crate::block_sounds::tests_brightness(&v);
                assert!(hz < 2600.0, "{b:?} too bright: {hz:.0} Hz");
            }
        }
    }

    #[test]
    fn voices_have_grunts_and_cries() {
        for who in [Who::Man, Who::Woman] {
            for cry in [Cry::Hurt, Cry::Death] {
                assert!(crate::block_sounds::group_len(group(who, cry)) >= 3, "{who:?} {cry:?}");
                let v = voice(who, cry, 3);
                assert!(v.len() > 4000);
                let hz = crate::block_sounds::tests_brightness(&v);
                assert!(hz < 2200.0, "{who:?} {cry:?} too bright: {hz:.0} Hz");
            }
        }
        set_player_voice(2);
        assert!(player_voice().is_none());
        set_player_voice(0);
        assert!(player_hurt(1).len() > 4000);
    }
}

#[cfg(test)]
mod export {
    use super::*;

    /// Writes blows on every kind of body and the player's voices as WAV files into
    /// $MC_SOUNDS_OUT (listening test).
    #[test]
    #[ignore]
    fn export_hit_sounds() {
        let Ok(dir) = std::env::var("MC_SOUNDS_OUT") else { return };
        for b in Body::ALL {
            let mut all = Vec::new();
            for (k, strength) in [60u8, 120, 180, 250, 120, 250].into_iter().enumerate() {
                all.extend(impact(b, strength, k as u32 * 97 + 11));
                all.extend(std::iter::repeat_n(0.0, 14000));
            }
            crate::block_sounds::export_wav(&format!("{dir}/dano-{b:?}.wav"), &all);
        }
        for (who, n) in [(0u8, "masculina"), (1, "feminina")] {
            set_player_voice(who);
            let mut all = Vec::new();
            for r in 0..5u32 {
                all.extend(player_hurt(r * 31 + 1));
                all.extend(std::iter::repeat_n(0.0, 16000));
            }
            all.extend(player_death(77));
            crate::block_sounds::export_wav(&format!("{dir}/jogador-voz-{n}.wav"), &all);
        }
        set_player_voice(0);
    }
}
