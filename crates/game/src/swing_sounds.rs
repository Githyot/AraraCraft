//! Weapons and tools cutting the air and working blocks, from real recordings
//! (sound_bank.rs).
//!
//! Every swing sounds like what swings: a dagger whistles short and light, a sword swishes,
//! a spear thrusts, an axe or a pickaxe whooshes heavy, a hammer or a greatsword drags a deep
//! long whoosh. The weapon's speed shapes it too: the quicker its swings, the shorter and
//! lighter each one; slow weapons sound slower and deeper. Wood is lighter than iron, and
//! netherite heavier still. Mining, chopping and hoeing add the tool's own bite on top of the
//! block's sound (a pickaxe rings on stone and clinks on metal, an axe bites into wood).

use crate::audio::{Material, Sfx};
use crate::block_sounds::{play, Act};
use voxel::block::ToolKind;

/// What is being swung.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Swing {
    Fist,
    Dagger,
    Sword,
    Greatsword,
    Axe,
    Pickaxe,
    Shovel,
    Hoe,
    Spear,
    Halberd,
    Hammer,
}

/// A tool working a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
}

impl Swing {
    #[allow(dead_code)]
    pub const ALL: [Swing; 11] = [
        Swing::Fist,
        Swing::Dagger,
        Swing::Sword,
        Swing::Greatsword,
        Swing::Axe,
        Swing::Pickaxe,
        Swing::Shovel,
        Swing::Hoe,
        Swing::Spear,
        Swing::Halberd,
        Swing::Hammer,
    ];

    fn act(self) -> Act {
        let a = |pick, rate, gain, lp, extra| Act { pick, rate, gain, lp, extra };
        match self {
            Swing::Fist => a(&["swing_light"], 1.05, 0.22, 2600.0, None),
            Swing::Dagger => a(&["swing_light"], 1.05, 0.45, 3800.0, None),
            Swing::Sword => a(&["swing_sword"], 1.0, 0.55, 3800.0, None),
            Swing::Greatsword => a(&["swing_heavy"], 0.8, 0.65, 3200.0, Some(("swing_sword", 0.75, 0.4))),
            Swing::Axe => a(&["swing_heavy"], 0.95, 0.55, 3400.0, None),
            Swing::Pickaxe => a(&["swing_heavy"], 1.0, 0.45, 3400.0, None),
            Swing::Shovel => a(&["swing_spear"], 1.0, 0.4, 3400.0, None),
            Swing::Hoe => a(&["swing_light", "swing_spear"], 0.95, 0.4, 3400.0, None),
            Swing::Spear => a(&["swing_spear"], 1.0, 0.55, 3600.0, None),
            Swing::Halberd => a(&["swing_heavy"], 0.82, 0.6, 3200.0, Some(("swing_spear", 0.8, 0.35))),
            Swing::Hammer => a(&["swing_heavy"], 0.7, 0.7, 2800.0, None),
        }
    }
}

/// How weapons are told apart by name (mods and Terraria weapons included); None for things
/// that make their own sound when used (bows, guns, wands, staffs, books, instruments).
pub fn swing_of(name: &str, kind: Option<ToolKind>) -> Option<Swing> {
    let has = |s: &str| name.contains(s);
    const OWN_SOUND: [&str; 16] = [
        "bow", "gun", "wand", "staff", "rod", "tome", "book", "launcher", "blaster", "rifle", "pistol", "shotgun", "flute",
        "lute", "horn", "drum",
    ];
    if name.is_empty() {
        return Some(Swing::Fist);
    }
    if has("greatsword") || has("claymore") || has("zweihander") || has("broadsword") || has("cleaver") {
        Some(Swing::Greatsword)
    } else if has("spear") || has("trident") || has("lance") || has("pike") && !has("pickaxe") || has("javelin") || has("glaive") {
        Some(Swing::Spear)
    } else if has("halberd") || has("scythe") || has("sickle") {
        Some(Swing::Halberd)
    } else if has("hammer") || has("mace") || has("club") || has("maul") || has("flail") || has("war_axe") {
        Some(Swing::Hammer)
    } else if has("dagger") || has("knife") || has("shiv") || has("kunai") || has("sai") && name.ends_with("sai") {
        Some(Swing::Dagger)
    } else if has("sword") || has("blade") || has("saber") || has("sabre") || has("rapier") || has("cutlass") || has("katana") || has("machete") || has("whip") {
        Some(Swing::Sword)
    } else if let Some(k) = kind {
        Some(match k {
            ToolKind::Sword => Swing::Sword,
            ToolKind::Axe => Swing::Axe,
            ToolKind::Pickaxe => Swing::Pickaxe,
            ToolKind::Shovel => Swing::Shovel,
            ToolKind::Hoe => Swing::Hoe,
        })
    } else if OWN_SOUND.iter().any(|w| has(w)) {
        None
    } else {
        // Anything else in the hand (a block, food, a stick): just the arm moving.
        Some(Swing::Fist)
    }
}

/// How heavy the weapon's material is: below 1 lighter (wood, gold), above 1 heavier.
pub fn heft(name: &str) -> f32 {
    let has = |s: &str| name.contains(s);
    if has("wooden") || has("wood_") || has("bamboo") || has("cactus") {
        0.8
    } else if has("golden") || has("gold_") || has("copper") || has("tin_") {
        0.9
    } else if has("stone") || has("flint") || has("bone") {
        1.0
    } else if has("netherite") || has("obsidian") || has("adamantite") || has("titanium") || has("chlorophyte") || has("luminite") {
        1.18
    } else if has("diamond") || has("mythril") || has("orichalcum") || has("cobalt") || has("palladium") {
        1.08
    } else {
        1.0
    }
}

/// Packs a swing's speed (seconds per swing) into the sound's byte: twentieths of a second.
pub fn speed_byte(cooldown: f32) -> u8 {
    (cooldown * 20.0).round().clamp(1.0, 255.0) as u8
}

/// A swing of `what` that takes `speed` twentieths of a second, of a material of `heft`.
pub fn swing(what: Swing, speed: u8, heft: f32, r: u32) -> Vec<f32> {
    let secs = speed as f32 / 20.0;
    let a = what.act();
    // Quicker than a sword (0.625 s): shorter and lighter; slower: longer and deeper.
    let quick = (0.625 / secs.clamp(0.15, 3.0)).powf(0.22).clamp(0.78, 1.18);
    let rate = a.rate * quick / heft.clamp(0.7, 1.3).powf(0.6);
    let gain = a.gain * (0.85 + 0.25 * (heft - 0.8)).clamp(0.8, 1.15) * (1.15 - 0.15 * quick);
    play(&Act { rate, gain, ..a }, r)
}

/// A tool biting into a block, laid over the block's own sound; empty when the tool does
/// nothing special on that material.
pub fn tool_use(tool: Tool, m: Material, r: u32) -> Vec<f32> {
    use Material as M;
    let a = |pick, rate, gain, lp| Act { pick, rate, gain, lp, extra: None };
    let act = match (tool, m) {
        (Tool::Pickaxe, M::Stone | M::Brick | M::Netherrack | M::Bone | M::Ice) => a(&["tool_pick_stone"], 1.0, 0.35, 4200.0),
        (Tool::Pickaxe, M::Deepslate) => a(&["tool_pick_stone"], 0.85, 0.38, 3800.0),
        (Tool::Pickaxe, M::Ore | M::Crystal) => a(&["tool_pick_stone", "tool_mining"], 0.95, 0.38, 4000.0),
        (Tool::Pickaxe, M::Metal | M::Copper) => a(&["tool_pick_metal"], 0.9, 0.3, 3800.0),
        (Tool::Axe, M::Wood | M::Bamboo) => a(&["tool_axe", "tool_chop"], 1.0, 0.45, 4200.0),
        (Tool::Axe, M::Leaves | M::Plant | M::Straw) => a(&["tool_chop"], 1.1, 0.25, 3800.0),
        (Tool::Shovel, M::Dirt | M::Grass | M::Sand | M::Gravel | M::Snow | M::Mud | M::SoulSand | M::Moss) => a(&["soil_dug"], 1.1, 0.28, 3800.0),
        (Tool::Hoe, M::Dirt | M::Grass | M::Moss | M::Mud | M::Plant | M::Leaves | M::Straw) => a(&["tool_hoe"], 1.0, 0.3, 3800.0),
        _ => return Vec::new(),
    };
    play(&act, r)
}

/// The held thing as a tool for `tool_use`.
pub fn tool_of(kind: Option<ToolKind>) -> Option<Tool> {
    match kind? {
        ToolKind::Pickaxe => Some(Tool::Pickaxe),
        ToolKind::Axe => Some(Tool::Axe),
        ToolKind::Shovel => Some(Tool::Shovel),
        ToolKind::Hoe => Some(Tool::Hoe),
        ToolKind::Sword => None,
    }
}

/// The heft of the last swing started (for the Sfx, which only carries kind and speed).
static HEFT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F80_0000);

pub fn last_heft() -> f32 {
    f32::from_bits(HEFT.load(std::sync::atomic::Ordering::Relaxed))
}

impl crate::Game {
    /// Called once a frame: a swing that has just started sounds like the thing swung. While
    /// mining, the tool's bite plays instead; right after placing a block, nothing extra.
    pub(crate) fn swing_sounds(&mut self) {
        let s = self.anim.swing;
        let started = s > 0.0 && (self.swing_prev <= 0.0 || s + 1e-4 < self.swing_prev);
        self.swing_prev = s;
        if !started || self.audio.is_none() {
            return;
        }
        let stack = self.held_stack();
        let (name, kind) = match stack {
            Some(st) => {
                let it = self.items.get(st.item);
                (it.name.clone(), it.tool.as_ref().map(|t| t.kind))
            }
            None => (String::new(), None),
        };
        if let Some((b, _)) = self.mining {
            let id = self.world.block_or_air(b);
            if id != voxel::block::AIR {
                if let Some(t) = tool_of(kind) {
                    let m = Material::of(&self.reg.get(id).name);
                    self.sfx_here(Sfx::ToolUse(t, m), 1.0);
                }
            }
            return;
        }
        if self.place_cd > 0.12 {
            return;
        }
        let Some(what) = swing_of(&name, kind) else { return };
        HEFT.store(heft(&name).to_bits(), std::sync::atomic::Ordering::Relaxed);
        let cd = self.attack_cooldown();
        self.sfx_here(Sfx::Swing(what, speed_byte(cd)), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapons_are_told_apart() {
        assert_eq!(swing_of("iron_sword", Some(ToolKind::Sword)), Some(Swing::Sword));
        assert_eq!(swing_of("diamond_greatsword", Some(ToolKind::Sword)), Some(Swing::Greatsword));
        assert_eq!(swing_of("iron_spear", Some(ToolKind::Sword)), Some(Swing::Spear));
        assert_eq!(swing_of("stone_pickaxe", Some(ToolKind::Pickaxe)), Some(Swing::Pickaxe));
        assert_eq!(swing_of("iron_axe", Some(ToolKind::Axe)), Some(Swing::Axe));
        assert_eq!(swing_of("pwnhammer", None), Some(Swing::Hammer));
        assert_eq!(swing_of("copper_dagger", None), Some(Swing::Dagger));
        assert_eq!(swing_of("bow", None), None);
        assert_eq!(swing_of("water_staff", None), None);
        assert_eq!(swing_of("", None), Some(Swing::Fist));
        assert_eq!(swing_of("cobblestone", None), Some(Swing::Fist));
    }

    fn len_and_peak(v: &[f32]) -> (usize, f32) {
        (v.len(), v.iter().fold(0.0f32, |p, s| p.max(s.abs())))
    }

    #[test]
    fn quick_weapons_are_shorter_and_slow_ones_longer() {
        let avg = |what, secs: f32, heft| (0..10u32).map(|r| len_and_peak(&swing(what, speed_byte(secs), heft, r)).0).sum::<usize>();
        assert!(avg(Swing::Sword, 0.3, 1.0) < avg(Swing::Sword, 1.2, 1.0));
        assert!(avg(Swing::Hammer, 1.4, 1.0) > avg(Swing::Dagger, 0.3, 1.0));
        assert!(avg(Swing::Axe, 1.0, 0.8) < avg(Swing::Axe, 1.0, 1.18), "wood lighter than netherite");
    }

    #[test]
    fn every_swing_and_tool_sounds_and_is_not_shrill() {
        for w in Swing::ALL {
            for r in 0..5 {
                let v = swing(w, 12, 1.0, r);
                let (n, peak) = len_and_peak(&v);
                assert!(n > 2000 && peak > 0.05, "{w:?} silent");
                let hz = crate::block_sounds::tests_brightness(&v);
                assert!(hz < 2600.0, "{w:?} too bright: {hz:.0} Hz");
            }
        }
        for (t, m) in [(Tool::Pickaxe, Material::Stone), (Tool::Pickaxe, Material::Metal), (Tool::Axe, Material::Wood), (Tool::Shovel, Material::Dirt), (Tool::Hoe, Material::Grass)] {
            for r in 0..5 {
                let v = tool_use(t, m, r);
                assert!(v.len() > 1000, "{t:?} on {m:?} silent");
                let hz = crate::block_sounds::tests_brightness(&v);
                assert!(hz < 2600.0, "{t:?} on {m:?} too bright: {hz:.0} Hz");
            }
        }
        assert!(tool_use(Tool::Axe, Material::Stone, 1).is_empty());
    }
}

#[cfg(test)]
mod export {
    use super::*;

    /// Writes swings of every weapon at a few speeds, and tools at work, as WAV files into
    /// $MC_SOUNDS_OUT (listening test).
    #[test]
    #[ignore]
    fn export_swing_sounds() {
        let Ok(dir) = std::env::var("MC_SOUNDS_OUT") else { return };
        for w in Swing::ALL {
            let mut all = Vec::new();
            for (k, (secs, heft)) in [(0.3, 1.0), (0.625, 0.8), (0.625, 1.0), (1.0, 1.18), (1.4, 1.0)].into_iter().enumerate() {
                for j in 0..2 {
                    all.extend(swing(w, speed_byte(secs), heft, (k * 2 + j) as u32 * 31 + 7));
                    all.extend(std::iter::repeat_n(0.0, (secs * 44100.0) as usize));
                }
            }
            crate::block_sounds::export_wav(&format!("{dir}/golpe-{w:?}.wav"), &all);
        }
        for (t, m, n) in [(Tool::Pickaxe, Material::Stone, "picareta-pedra"), (Tool::Pickaxe, Material::Metal, "picareta-metal"), (Tool::Axe, Material::Wood, "machado-madeira"), (Tool::Shovel, Material::Dirt, "pa-terra"), (Tool::Hoe, Material::Grass, "enxada-grama")] {
            let mut all = Vec::new();
            for r in 0..6u32 {
                let mut v = crate::block_sounds::block(m, crate::block_sounds::Action::Hit, r * 13 + 1);
                crate::hit_sounds::mix(&mut v, &tool_use(t, m, r * 17 + 5), 0);
                all.extend(v);
                all.extend(std::iter::repeat_n(0.0, 8000));
            }
            crate::block_sounds::export_wav(&format!("{dir}/{n}.wav"), &all);
        }
    }
}
