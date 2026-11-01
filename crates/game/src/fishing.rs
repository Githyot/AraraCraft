//! Fishing: cast a rod, wait, watch a fish come in, feel the nibbles, strike on the bite.
//!
//! The bite comes in stages, like Java: a quiet wait (Lure shortens it, rain too), then a trail
//! of ripples running over the water to the float while the fish comes in and nibbles (the
//! float twitches), then the bite proper: the float is yanked under with a splash and there is
//! a moment to strike.
//!
//! In 3D the fish is there in the water, so the bigger and rarer ones fight (plan/pesca-analise.md,
//! lote 2): the float runs off over the water, you hold the button to reel and follow it with
//! the rod. Reeling while it runs strains the line (the bar by the crosshair): past the top it
//! snaps; let it go slack too long and the fish slips the hook. Common fish come straight in.
//!
//! What comes up follows Java's table (85% fish, 10% junk, 5% treasure, moved by Luck of the
//! Sea), with treasure only from open water. The Terraria side (power, crates, the angler's
//! fish) is in tr_fishing.rs, the modular rod in mods/aquaculture.rs.
//!
//! The float is a real little 3D model (red top, white belly, a stem), and the line hangs slack
//! while you wait, goes taut and trembles when a fish pulls.

use glam::{IVec3, Quat, Vec3};
use voxel::block::AIR;
use voxel::item::ItemStack;
use voxel::physics::Terrain;

use crate::audio::Sfx;
use crate::particles::GpuParticle;
use crate::renderer::LineVertex;
use crate::sfx_extra::Fx;
use crate::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// Nothing near yet.
    Wait,
    /// A fish is coming in: ripples run to the float, small nibbles.
    Approach,
    /// It bit: strike now.
    Bite,
    /// A big one on the line.
    Fight,
}

/// What kind of thing is on the hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fish,
    Junk,
    Treasure,
    /// Crates, the angler's fish, lava fish (tr_fishing.rs).
    Special,
}

#[derive(Clone, Copy, Debug)]
struct Catch {
    name: &'static str,
    kind: Kind,
    /// 0 common .. 3 very rare: how hard it fights.
    tier: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct Fight {
    /// 0..1: at 1 the line snaps.
    tension: f32,
    /// Seconds the fish can still pull hard.
    stamina: f32,
    /// Seconds left of the current run (0: resting) and of the rest after it.
    run: f32,
    rest: f32,
    /// Which way it runs (flat).
    dir: Vec3,
    /// Seconds the line has been slack.
    slack: f32,
    /// Timers for the reel clicks and the splashes.
    tick: f32,
    splash: f32,
}

pub struct Bobber {
    pos: Vec3,
    vel: Vec3,
    in_water: bool,
    landed: bool,
    phase: Phase,
    /// Seconds left of the wait / the approach / the bite window.
    wait: f32,
    /// The approach: how long it lasts and where the fish starts from (flat direction).
    approach: f32,
    approach_len: f32,
    from: Vec3,
    /// Next nibble and the next ripple of the trail.
    nibble: f32,
    trail: f32,
    /// How far the float is pulled under now (drawn), and its tilt from the nibbles.
    dip: f32,
    twitch: f32,
    catch: Option<Catch>,
    fight: Fight,
    age: f32,
}

/// (item, weight): Java's fish first, then ours.
const FISH: &[(&str, u32)] = &[("cod", 45), ("salmon", 22), ("pufferfish", 10), ("tropical_fish", 2), ("trout", 12), ("catfish", 7), ("tilapia", 8)];
/// Extra fish that only live in some biomes (Aquaculture idea): (biomes, item, weight).
const LOCAL_FISH: &[(&[&str], &str, u32)] = &[
    (&["Jungle", "Swamp", "Mangrove Swamp"], "piranha", 18),
    (&["Jungle", "Swamp", "Mangrove Swamp"], "pirarucu", 8),
    (&["Snowy Plains", "Taiga", "Mountains"], "arctic_char", 22),
    (&["Plains", "Forest", "Birch Forest", "Dark Forest", "Cherry Grove", "Savanna"], "perch", 18),
    (&["Ocean", "Beach"], "tuna", 16),
];
/// Java's junk (the water bottle and tripwire hook left out), bamboo only in the jungle.
const JUNK: &[(&str, u32)] = &[("lily_pad", 17), ("leather_boots", 10), ("leather", 10), ("bone", 10), ("bowl", 10), ("rotten_flesh", 10), ("stick", 5), ("string", 5), ("ink_sac", 1)];
/// Java's treasure, and a small purse of coins.
const TREASURE: &[(&str, u32)] = &[("bow", 1), ("enchanted_book", 1), ("fishing_rod", 1), ("name_tag", 1), ("nautilus_shell", 1), ("saddle", 1), ("coins", 1)];

/// Height of the top of the float's stem over its centre, where the line ties on.
const FLOAT_TOP: f32 = 0.4;

/// How hard a fish fights: 0 comes straight in, 1 uncommon, 2 rare, 3 very rare.
fn fish_tier(name: &str) -> u8 {
    match name {
        "catfish" | "pufferfish" | "tuna" | "halibut" | "pollock" | "gar" | "arctic_char" | "grouper" | "piranha" | "lava_koi" | "tropical_fish" => 1,
        "pirarucu" => 2,
        n if crate::tr_fishing::is_quest_fish(n) => 2,
        _ => 0,
    }
}

impl Game {
    /// Seconds before a fish starts coming in: Java's 5..30 s, Lure taking 5 s a level, rain
    /// bringing them up, then the rod, power and skill.
    /// Terraria: more fishing power brings up the rarer fish more often.
    fn rarity_weight(&self, name: &str, w: u32) -> u32 {
        let power = self.fishing_power();
        let k = match fish_tier(name) {
            0 => 1.0,
            1 => (0.4 + power / 50.0).min(2.0),
            _ => (0.15 + power / 70.0).min(2.0),
        };
        ((w as f32 * k).round() as u32).max(1)
    }

    /// How much water is joined to the float (Terraria's lake size, counted in blocks in 3D),
    /// up to `CAP`. A puddle fishes badly.
    fn water_volume(&self, at: Vec3, lava: bool) -> u32 {
        const CAP: usize = 300;
        let want = if lava { 2 } else { 1 };
        let start = IVec3::new(at.x.floor() as i32, (at.y - 0.85).round() as i32, at.z.floor() as i32);
        let mut seen = std::collections::HashSet::new();
        let mut todo = vec![start];
        while let Some(p) = todo.pop() {
            if seen.len() >= CAP || (p - start).abs().max_element() > 16 || seen.contains(&p) {
                continue;
            }
            if self.world.fluid(self.world.block_or_air(p)) != want {
                continue;
            }
            seen.insert(p);
            for d in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z, IVec3::NEG_Y, IVec3::Y] {
                todo.push(p + d);
            }
        }
        seen.len() as u32
    }

    fn wait_for_bite(&mut self) -> f32 {
        let lure = self.rod_ench(crate::enchanting::LURE) as f32;
        let base = (5.0 + self.surv.rng.f32() * 25.0 - lure * 5.0).max(1.5);
        let rain = if self.weather.rain > 0.5 { 0.7 } else { 1.0 };
        base * rain * self.rod_bite_factor() * self.tr_bite_factor() * self.skill_bite_factor()
    }

    /// Level of an enchantment on the rod in hand.
    fn rod_ench(&self, id: u8) -> u8 {
        crate::enchanting::level(&self.held_stack(), id)
    }

    /// Right click with the rod: cast, strike, or reel in.
    pub(crate) fn use_rod(&mut self) -> bool {
        // Holding the button repeats the use every ~0.22 s: only a fresh press counts.
        let held_on = self.now - self.rod_last_use < 0.3;
        self.rod_last_use = self.now;
        if held_on {
            return true;
        }
        if let Some(mut b) = self.bobber.take() {
            match b.phase {
                Phase::Fight => {
                    // Reeling is the button held (update_fishing).
                    self.bobber = Some(b);
                    return true;
                }
                Phase::Bite => {
                    let c = b.catch.unwrap_or(Catch { name: "cod", kind: Kind::Fish, tier: 0 });
                    if c.tier > 0 {
                        self.start_fight(&mut b, c);
                        self.bobber = Some(b);
                        self.anim.start_swing();
                        return true;
                    }
                    self.land_catch(c, b.pos);
                }
                _ => {
                    if b.landed && !b.in_water {
                        // Reeling a float stuck in the ground costs a little wear (Java: 2).
                        self.wear_rod(2);
                    }
                }
            }
            self.sfx_here(Sfx::Fx(Fx::Reel), 0.5);
            self.anim.start_swing();
            return true;
        }
        let eye = self.body.eye();
        let fwd = self.forward();
        self.bobber = Some(Bobber {
            pos: eye + fwd * 0.6,
            vel: fwd * 13.0 + Vec3::Y * 3.0,
            in_water: false,
            landed: false,
            phase: Phase::Wait,
            wait: 0.0,
            approach: 0.0,
            approach_len: 0.0,
            from: Vec3::X,
            nibble: 0.0,
            trail: 0.0,
            dip: 0.0,
            twitch: 0.0,
            catch: None,
            fight: Fight::default(),
            age: 0.0,
        });
        self.sfx_here(Sfx::Fx(Fx::Swing), 0.55);
        self.anim.start_swing();
        true
    }

    fn wear_rod(&mut self, n: u16) {
        if self.survival && self.held_name().as_deref().is_some_and(crate::mods::aquaculture::is_rod) {
            self.wear_held(self.rod_wear(n));
        }
    }

    pub(crate) fn held_name(&self) -> Option<String> {
        self.held_stack().map(|s| self.items.get(s.item).name.clone())
    }

    fn pick(&mut self, table: &[(&'static str, u32)]) -> Option<&'static str> {
        let total: u32 = table.iter().map(|t| t.1).sum();
        let mut r = self.surv.rng.below(total.max(1));
        for &(name, w) in table {
            if r < w {
                return Some(name);
            }
            r -= w;
        }
        None
    }

    /// Java's open water: a 5×5 patch of water two deep around the float, with nothing over it.
    /// Treasure only bites there.
    pub(crate) fn open_water(&self, at: Vec3) -> bool {
        let c = IVec3::new(at.x.floor() as i32, (at.y - 0.85).round() as i32, at.z.floor() as i32);
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in -1..=2 {
                    let p = c + IVec3::new(dx, dy, dz);
                    let id = self.world.block_or_air(p);
                    let wet = self.world.fluid(id) == 1;
                    if dy <= 0 && !wet {
                        return false;
                    }
                    if dy > 0 && (wet || Terrain::solid(&self.world, p)) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Fish, junk or treasure, by the rod's odds and Luck of the Sea.
    fn odds(&self, at: Vec3) -> (f32, f32) {
        let (fish_p, junk_p) = self.rod_odds();
        let luck = self.rod_ench(crate::enchanting::LUCK_OF_THE_SEA) as f32;
        let junk = (junk_p - 0.0195 * luck).max(0.01);
        let treasure = if self.open_water(at) { (1.0 - fish_p - junk_p + 0.021 * luck).max(0.0) } else { 0.0 };
        let fish = (1.0 - junk - treasure).max(0.0);
        let sum = fish + junk + treasure;
        (fish / sum, junk / sum)
    }

    /// What is on the hook, decided when it bites.
    fn decide_catch(&mut self, at: Vec3) -> Catch {
        if let Some(name) = self.tr_special_catch(at) {
            let tier = fish_tier(name);
            return Catch { name, kind: Kind::Special, tier };
        }
        let roll = self.surv.rng.f32();
        let biome = self.world.gen.base_biome_at(at.x.floor() as i32, at.z.floor() as i32).name();
        let more = crate::mods::aquaculture::MORE_FISH.iter().filter(|f| f.0.contains(&biome)).map(|f| (f.1, f.2));
        let fish: Vec<(&'static str, u32)> = FISH
            .iter()
            .copied()
            .chain(LOCAL_FISH.iter().filter(|f| f.0.contains(&biome)).map(|f| (f.1, f.2)))
            .chain(more)
            .map(|(n, w)| (n, self.rarity_weight(n, self.rod_fish_weight(n, w))))
            .collect();
        let jungle = matches!(biome, "Jungle" | "Bamboo Jungle");
        let junk: Vec<(&'static str, u32)> = JUNK.iter().chain(crate::mods::aquaculture::MORE_JUNK).copied().chain(jungle.then_some(("bamboo", 10))).collect();
        let (fish_p, junk_p) = self.odds(at);
        let (table, kind): (&[(&'static str, u32)], Kind) = if roll < fish_p {
            (&fish, Kind::Fish)
        } else if roll < fish_p + junk_p {
            (&junk, Kind::Junk)
        } else {
            (TREASURE, Kind::Treasure)
        };
        let name = self.pick(table).unwrap_or("cod");
        let tier = if kind == Kind::Fish { fish_tier(name) } else { 0 };
        Catch { name, kind, tier }
    }

    /// The thing itself: treasure bows and rods come worn and enchanted, books enchanted.
    fn catch_stack(&mut self, c: Catch) -> Option<ItemStack> {
        let id = self.items.id(c.name)?;
        let mut st = ItemStack::new(id, 1);
        if c.kind == Kind::Treasure {
            match c.name {
                "bow" | "fishing_rod" => {
                    for (e, l) in crate::enchanting::roll(&self.items, id, 30, &mut self.surv.rng) {
                        st.ench.set(e, l);
                    }
                    if let Some(max) = self.items.get(id).max_wear() {
                        // Java: 0..25% of the durability left.
                        let left = (max as f32 * self.surv.rng.f32() * 0.25) as u16;
                        st.damage = max.saturating_sub(left.max(1));
                    }
                }
                "enchanted_book" => st.ench = crate::enchanting::random_any(&self.items, id, &mut self.surv.rng),
                _ => {}
            }
        } else if c.kind == Kind::Junk && c.name == "leather_boots" {
            if let Some(max) = self.items.get(id).max_wear() {
                st.damage = (max as f32 * (0.1 + self.surv.rng.f32() * 0.8)) as u16;
            }
        }
        Some(st)
    }

    /// The catch comes out of the water and flies to the player.
    fn land_catch(&mut self, c: Catch, at: Vec3) {
        let to = self.body.eye() - at;
        let vel = Vec3::new(to.x, to.y + to.length().sqrt() * 1.6, to.z);
        if c.name == "coins" {
            let v = crate::money::SILVER * (5 + self.surv.rng.below(20)) as u64;
            self.drop_coins(at + Vec3::Y * 0.2, v);
        } else if let Some(st) = self.catch_stack(c) {
            self.spawn_drop(at + Vec3::Y * 0.2, st, vel);
        }
        if c.kind == Kind::Fish || c.kind == Kind::Special {
            self.tr_used_bait();
        }
        self.sfx_at(Sfx::Fx(Fx::FishSplash), at, 0.6, 24.0);
        self.particles.splash(at, self.reg.get(self.reg.id_or_air("water")).faces[2], 0.3);
        self.particles.ripple(at, 1.2);
        if self.survival {
            // Java: 1 to 6 experience.
            let n = 1 + self.surv.rng.below(6);
            self.spawn_orbs(self.body.pos + Vec3::Y, n);
        }
        self.wear_rod(1);
        let fish = matches!(c.kind, Kind::Fish | Kind::Special);
        self.skill_fish_caught(fish);
        self.rod_caught(Some(c.name).filter(|_| c.kind == Kind::Fish), at);
    }

    fn start_fight(&mut self, b: &mut Bobber, c: Catch) {
        b.phase = Phase::Fight;
        let t = c.tier as f32;
        b.fight = Fight { tension: 0.15, stamina: 2.5 + t * 2.5, run: 0.0, rest: 0.2, dir: Vec3::X, slack: 0.0, tick: 0.0, splash: 0.0 };
        self.sfx_at(Sfx::Fx(Fx::FishSplash), b.pos, 0.7, 24.0);
        self.toast(if c.tier >= 2 { "Um peixe grande! Segure para puxar e acompanhe com a vara" } else { "Peixe brigando! Segure para puxar" }, 2.5);
    }

    /// Is the float's water cell still water here (the float stays on the surface)?
    fn float_ok(&self, p: Vec3, lava: bool) -> bool {
        let c = IVec3::new(p.x.floor() as i32, (p.y - 0.85).round() as i32, p.z.floor() as i32);
        self.world.fluid(self.world.block_or_air(c)) == if lava { 2 } else { 1 }
    }

    /// One step of a fight. Returns false when it is over (caught, snapped or slipped).
    fn fight_step(&mut self, b: &mut Bobber, dt: f32) -> bool {
        let c = b.catch.unwrap_or(Catch { name: "cod", kind: Kind::Fish, tier: 1 });
        let t = c.tier as f32;
        let reeling = self.mouse_held[1] || self.pad_held.contains(&crate::controller::Action::Use);
        let me = self.body.pos;
        let mut to_me = me - b.pos;
        to_me.y = 0.0;
        let dist = to_me.length();
        let to_me = to_me.normalize_or_zero();
        let lava = self.t5.tr.fish.lava;
        let mut f = b.fight;
        // The fish: runs and rests, weaker as it tires.
        if f.run > 0.0 {
            f.run -= dt;
            if f.run <= 0.0 {
                f.rest = 0.8 + self.surv.rng.f32() * 1.4;
            }
        } else {
            f.rest -= dt;
            if f.rest <= 0.0 && f.stamina > 0.0 {
                f.run = (0.8 + self.surv.rng.f32() * 1.2) * (1.0 + t * 0.25);
                // Mostly away from the player, swinging to the sides.
                let a = (self.surv.rng.f32() - 0.5) * 2.4;
                f.dir = Quat::from_rotation_y(a) * -to_me;
                if f.dir.length_squared() < 0.01 {
                    f.dir = Vec3::X;
                }
                self.sfx_at(Sfx::Fx(Fx::FishSplash), b.pos, 0.35, 20.0);
                if c.tier >= 2 && self.surv.rng.chance(0.4) && !lava {
                    // A big one breaks the surface.
                    self.particles.fish(b.pos, [0.55, 0.6, 0.62]);
                }
            }
        }
        let running = f.run > 0.0 && f.stamina > 0.0;
        // Following the fish with the rod (looking at it) eases the strain.
        let look = Vec3::new(self.forward().x, 0.0, self.forward().z).normalize_or_zero();
        let aim = look.dot(-to_me).clamp(-1.0, 1.0);
        let aim_k = 1.45 - 0.65 * aim.max(0.0);
        let line_k = self.fight_line_factor();
        let mut moved = Vec3::ZERO;
        if running {
            moved += f.dir * (1.5 + 0.5 * t) * dt;
        }
        if reeling {
            let rise = if running { 0.32 + 0.16 * t } else { 0.05 };
            f.tension += rise * aim_k * line_k * dt;
            moved += to_me * if running { 1.0 } else { 3.0 } * dt;
            f.stamina -= dt * if running { 1.0 } else { 0.4 };
            f.slack = 0.0;
            f.tick -= dt;
            if f.tick <= 0.0 {
                f.tick = if running { 0.3 } else { 0.18 };
                self.sfx_here(Sfx::Fx(Fx::Reel), 0.35 + 0.2 * f.tension);
            }
        } else {
            f.tension -= 0.55 * dt;
            if f.tension <= 0.02 {
                f.slack += dt;
            }
        }
        f.tension = f.tension.clamp(0.0, 1.2);
        let next = b.pos + moved;
        let blocked = !self.float_ok(next, lava);
        if !blocked {
            b.pos = next;
        } else if running {
            // A bank in the way: it turns.
            f.dir = Quat::from_rotation_y(1.6) * f.dir;
        }
        // Splashes while it pulls.
        f.splash -= dt;
        if running && f.splash <= 0.0 {
            f.splash = 0.15;
            if !lava {
                let water = self.reg.get(self.reg.id_or_air("water")).faces[2];
                self.particles.splash(b.pos, water, 0.12);
                self.particles.ripple(b.pos, 0.5);
            }
        }
        b.fight = f;
        if f.tension >= if self.tr_strong_line() { 1.15 } else { 1.0 } {
            self.toast("A linha arrebentou!", 2.5);
            self.sfx_here(Sfx::Fx(Fx::LineSnap), 0.6);
            self.wear_rod(2);
            self.rod_lost_bait();
            return false;
        }
        if f.slack > 2.6 || dist > 30.0 {
            self.toast("O peixe se soltou do anzol", 2.5);
            return false;
        }
        if dist < 2.6 || (reeling && blocked && dist < 6.0) {
            self.land_catch(c, b.pos);
            return false;
        }
        true
    }

    pub(crate) fn update_fishing(&mut self, dt: f32) {
        if self.bobber.is_none() {
            return;
        }
        if !self.held_name().as_deref().is_some_and(crate::mods::aquaculture::is_rod) {
            self.bobber = None;
            return;
        }
        let mut b = self.bobber.take().unwrap();
        b.age += dt;
        b.twitch *= (-dt * 9.0).exp();
        let water = self.reg.get(self.reg.id_or_air("water")).faces[2];
        if !b.in_water {
            if !b.landed {
                b.vel.y -= 20.0 * dt;
                b.vel *= 1.0 - 0.4 * dt;
                let next = b.pos + b.vel * dt;
                let cell = next.floor().as_ivec3();
                let id = self.world.block_or_air(cell);
                let fluid = self.world.fluid(id);
                if fluid == 1 || (fluid == 2 && self.lava_rod()) {
                    self.t5.tr.fish.lava = fluid == 2;
                    b.in_water = true;
                    b.pos = Vec3::new(next.x, cell.y as f32 + 0.85, next.z);
                    // Float to the top of the water.
                    let mut top = cell;
                    while self.world.fluid(self.world.block_or_air(top + IVec3::Y)) == fluid && top.y < cell.y + 16 {
                        top.y += 1;
                    }
                    b.pos.y = top.y as f32 + 0.85;
                    b.vel = Vec3::ZERO;
                    // Small water fishes worse: full power from 150 blocks of water joined up.
                    let vol = self.water_volume(b.pos, fluid == 2);
                    self.t5.tr.fish.pool = (vol as f32 / 150.0).clamp(0.15, 1.0);
                    if vol < 25 {
                        self.toast("Pouca água aqui: os peixes quase não vêm", 2.5);
                    }
                    b.wait = self.wait_for_bite();
                    if vol < 25 {
                        b.wait *= 2.0;
                    }
                    b.dip = 0.3;
                    self.particles.splash(b.pos, self.reg.get(id).faces[2], 0.15);
                    if fluid == 1 {
                        self.particles.ripple(b.pos, 0.8);
                    }
                    self.sfx_at(Sfx::Fx(Fx::FishSplash), b.pos, 0.22, 20.0);
                } else if Terrain::solid(&self.world, cell) {
                    b.landed = true;
                    b.vel = Vec3::ZERO;
                } else {
                    b.pos = next;
                }
            }
        } else {
            let lava = self.t5.tr.fish.lava;
            match b.phase {
                Phase::Wait => {
                    b.wait -= dt;
                    if b.wait <= 0.0 {
                        // Java: the fish takes 1..4 s to come in, from a little way off.
                        b.phase = Phase::Approach;
                        b.approach_len = 1.0 + self.surv.rng.f32() * 3.0;
                        b.approach = b.approach_len;
                        let a = self.surv.rng.f32() * std::f32::consts::TAU;
                        b.from = Vec3::new(a.cos(), 0.0, a.sin());
                        b.nibble = 0.4 + self.surv.rng.f32() * 0.8;
                    }
                }
                Phase::Approach => {
                    b.approach -= dt;
                    // The trail: ripples running in over the surface.
                    b.trail -= dt;
                    if b.trail <= 0.0 && !lava {
                        b.trail = 0.11;
                        let d = (b.approach / b.approach_len).max(0.0) * (1.2 + b.approach_len * 1.1);
                        let p = b.pos + b.from * d;
                        if self.float_ok(p, false) {
                            let s = Vec3::new(p.x, p.y.floor() + 0.88, p.z);
                            self.particles.ripple(s, 0.3);
                            if self.surv.rng.chance(0.35) {
                                self.particles.bubbles(s - Vec3::Y * 0.4, water, 1);
                            }
                        }
                    }
                    // Nibbles: the float twitches.
                    b.nibble -= dt;
                    if b.nibble <= 0.0 && b.approach > 0.3 {
                        b.nibble = 0.35 + self.surv.rng.f32() * 0.7;
                        b.twitch = 1.0;
                        self.sfx_at(Sfx::Fx(Fx::Nibble), b.pos, 0.3, 14.0);
                        if !lava {
                            self.particles.ripple(b.pos, 0.35);
                        }
                    }
                    if b.approach <= 0.0 {
                        b.phase = Phase::Bite;
                        // Java: 0.5..1 s to strike, longer with a good line or the sonar potion.
                        b.wait = (0.6 + self.surv.rng.f32() * 0.5) * self.rod_bite_window() * self.tr_bite_window();
                        let c = self.decide_catch(b.pos);
                        b.catch = Some(c);
                        b.twitch = 1.5;
                        self.sfx_at(Sfx::Fx(Fx::FishSplash), b.pos, 0.5, 22.0);
                        self.particles.splash(b.pos, water, 0.25);
                        if !lava {
                            self.particles.ripple(b.pos, 0.9);
                            self.particles.bubbles(b.pos - Vec3::Y * 0.3, water, 4);
                        }
                        self.pads.rumble(0.3, 0.15, 0.12);
                    }
                }
                Phase::Bite => {
                    b.wait -= dt;
                    if b.wait <= 0.0 {
                        // Got away.
                        b.phase = Phase::Wait;
                        b.catch = None;
                        b.wait = self.wait_for_bite();
                    }
                }
                Phase::Fight => {
                    if !self.fight_step(&mut b, dt) {
                        return;
                    }
                }
            }
        }
        // How far the float is pulled under.
        let target = match b.phase {
            Phase::Bite => 0.32,
            Phase::Fight if b.fight.run > 0.0 => 0.24,
            Phase::Fight => 0.1,
            _ => 0.0,
        };
        b.dip += (target - b.dip) * (1.0 - (-dt * 14.0).exp());
        if b.pos.distance(self.body.pos) > 34.0 || b.pos.y < voxel::chunk::MIN_Y as f32 || b.age > 300.0 {
            return;
        }
        self.bobber = Some(b);
    }

    /// Where the float is drawn: bobbing on water, pulled under while a fish bites.
    fn bobber_pos(&self) -> Option<Vec3> {
        let b = self.bobber.as_ref()?;
        let mut p = b.pos;
        if b.in_water {
            p.y += (self.now * 2.6).sin() * 0.035 - b.dip - b.twitch * 0.07;
            if b.phase == Phase::Bite || (b.phase == Phase::Fight && b.fight.run > 0.0) {
                // Jerked about by the fish.
                p.x += (self.now * 31.0).sin() * 0.04;
                p.z += (self.now * 27.0).cos() * 0.04;
            }
        }
        Some(p)
    }

    /// My float for the other players: where it is drawn, and the line's strain.
    pub(crate) fn bobber_net(&self) -> Option<[f32; 4]> {
        let p = self.bobber_pos()?;
        let b = self.bobber.as_ref()?;
        let strain = match b.phase {
            Phase::Fight => b.fight.tension.clamp(0.0, 1.0),
            Phase::Bite => 0.5,
            _ if b.in_water => 0.0,
            _ => 0.2,
        };
        Some([p.x, p.y, p.z, strain])
    }

    /// The floats of the other players in this dimension: (float, strain, their rod tip).
    fn remote_floats(&self) -> Vec<(Vec3, f32, Vec3)> {
        let dim = self.dim.as_u8();
        self.net
            .remotes
            .values()
            .filter(|r| r.state.dim == dim)
            .filter_map(|r| {
                let b = r.state.bobber?;
                let (pos, yaw) = r.shown();
                let fwd = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
                let right = fwd.cross(Vec3::Y).normalize_or_zero();
                Some((Vec3::new(b[0], b[1], b[2]), b[3], pos + Vec3::Y * 1.9 + fwd * 0.9 + right * 0.35))
            })
            .collect()
    }

    /// The float as a little 3D model: white belly, red top, a dark stem. Boxes drawn like the
    /// sign letters (relative to the camera's block). The other players' floats too.
    pub(crate) fn bobber_boxes(&self, cam_block: IVec3) -> Vec<GpuParticle> {
        let mut out = Vec::new();
        for (p, strain, _) in self.remote_floats() {
            out.extend(self.float_model(p, cam_block, (self.now * 1.9).sin() * 0.08 + strain * 0.3, 0.0));
        }
        if let (Some(p), Some(b)) = (self.bobber_pos(), self.bobber.as_ref()) {
            let tilt = (self.now * 1.9).sin() * 0.08 + b.twitch * 0.35 + if b.phase == Phase::Bite { 0.4 } else { 0.0 };
            out.extend(self.float_model(p, cam_block, tilt, b.age * 0.3));
        }
        out
    }

    fn float_model(&self, p: Vec3, cam_block: IVec3, tilt: f32, turn: f32) -> Vec<GpuParticle> {
        let light = self.world.light((p + Vec3::Y * 0.3).floor().as_ivec3());
        let l = ((light >> 4) as f32 / 15.0, (light & 15) as f32 / 15.0);
        let c = p - cam_block.as_vec3();
        let rot = Quat::from_rotation_y(turn) * Quat::from_rotation_x(tilt);
        let red = [0.82, 0.12, 0.1];
        let white = [0.93, 0.92, 0.88];
        // Sized up a little from Java's so it reads at fishing distance; the waterline crosses
        // the white belly.
        let parts: [(Vec3, Vec3, [f32; 3]); 5] = [
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.24, 0.14, 0.24), white),
            (Vec3::new(0.0, -0.095, 0.0), Vec3::new(0.15, 0.05, 0.15), white),
            (Vec3::new(0.0, 0.14, 0.0), Vec3::new(0.24, 0.14, 0.24), red),
            (Vec3::new(0.0, 0.235, 0.0), Vec3::new(0.15, 0.05, 0.15), red),
            (Vec3::new(0.0, 0.33, 0.0), Vec3::new(0.03, 0.14, 0.03), [0.16, 0.13, 0.1]),
        ];
        parts.iter().map(|&(o, s, t)| GpuParticle::plain(c + rot * o, rot, s, t, l)).collect()
    }

    /// The fishing line from the rod tip to the float: slack and sagging while you wait, taut
    /// and trembling while a fish pulls.
    pub(crate) fn fishing_line(&self, cam_block: IVec3, third: bool) -> Vec<LineVertex> {
        let c = cam_block.as_vec3();
        let mut out = Vec::new();
        for (p, strain, tip) in self.remote_floats() {
            let end = p + Vec3::Y * FLOAT_TOP;
            let len = tip.distance(end);
            let sag = (len * 0.12 * (1.0 - strain)).clamp(0.02, 2.2);
            out.extend(line_verts(tip, end, sag, strain * 0.05, Some(p.y - 0.85 + 0.9).filter(|_| strain < 0.3), c, self.now));
        }
        let (Some(end), Some(b)) = (self.bobber_pos(), self.bobber.as_ref()) else {
            return out;
        };
        let fwd = self.forward();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let tip = if third {
            self.body.pos + Vec3::Y * 1.9 + Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero() * 0.9 + right * 0.35
        } else if let Some(place) = self.held_item_frame(self.body.eye() - c) {
            // The tip of the rod as it is drawn in the hand (the picture's top right corner;
            // the hand is drawn scaled towards the eye, so the line starts there too).
            place.transform_point3(Vec3::new(0.44, 0.46, 0.0)) + c
        } else {
            self.body.eye() + fwd * 0.9 + right * 0.45 + Vec3::Y * 0.12
        };
        let end = end + Vec3::Y * FLOAT_TOP;
        // A slack line lies on the water instead of sinking through it.
        let floor = b.in_water.then(|| b.pos.y - 0.85 + 0.9);
        let len = tip.distance(end);
        let (sag, shake) = match b.phase {
            _ if !b.in_water => ((len * 0.06).min(1.0), 0.0),
            Phase::Wait | Phase::Approach => ((len * 0.12).min(2.2), 0.0),
            Phase::Bite => ((len * 0.03).min(0.5), 0.03),
            Phase::Fight => ((len * 0.05 * (1.0 - b.fight.tension)).max(0.02), 0.01 + 0.05 * b.fight.tension),
        };
        out.extend(line_verts(tip, end, sag, shake, floor, c, self.now));
        out
    }

    /// The strain on the line while a fish fights: a bar under the crosshair.
    pub(crate) fn draw_fish_hud(&mut self, w: f32, h: f32) {
        let Some(b) = self.bobber.as_ref() else {
            return;
        };
        if b.phase != Phase::Fight {
            return;
        }
        let f = b.fight;
        let s = self.ui.scale;
        let bw = 70.0 * s;
        let x = (w - bw) / 2.0;
        let y = h / 2.0 + 16.0 * s;
        self.ui.rect(x - s, y - s, bw + 2.0 * s, 6.0 * s, [0, 0, 0, 150]);
        let k = f.tension.clamp(0.0, 1.0);
        let col = if k < 0.55 {
            [90, 190, 90, 230]
        } else if k < 0.8 {
            [220, 190, 70, 230]
        } else {
            [220, 70, 60, 240]
        };
        self.ui.rect(x, y, bw * k, 4.0 * s, col);
        let text = if f.run > 0.0 && f.stamina > 0.0 {
            "Puxando!"
        } else if f.stamina <= 0.0 {
            "Cansou: recolha"
        } else {
            "Recolha"
        };
        let tw = self.ui.text_width(text, s);
        self.ui.text((w - tw) / 2.0, y + 7.0 * s, text, [230, 235, 240, 230]);
    }

    /// Test hook (MC_PESCA=espera|beliscada|mordida|briga|lancando): a round pond in front, the
    /// rod in hand and the float in the water at that moment of the bite. Frozen there for the
    /// screenshot (update_fishing gets no time in screenshot mode).
    pub(crate) fn test_fishing(&mut self, scene: &str) {
        let p = self.body.pos.floor().as_ivec3();
        let f = self.forward();
        let dir = if f.x.abs() > f.z.abs() { IVec3::new(f.x.signum() as i32, 0, 0) } else { IVec3::new(0, 0, f.z.signum() as i32) };
        let side = IVec3::new(-dir.z, 0, dir.x);
        let g = |n: &str| self.reg.id_or_air(n);
        let (grass, dirt, water, stone, sand) = (g("grass_block"), g("dirt"), g("water"), g("stone"), g("sand"));
        let ground = p.y - 1;
        for a in -14..=14 {
            for b in -4..=22 {
                let col = IVec3::new(p.x, ground, p.z) + side * a + dir * b;
                let r2 = a * a + (b - 10) * (b - 10);
                let pond = r2 < 64;
                let depth = if r2 < 30 {
                    3
                } else if r2 < 50 {
                    2
                } else {
                    1
                };
                for h in -5..12 {
                    let q = col + IVec3::Y * h;
                    let id = if h < -3 {
                        stone
                    } else if h <= 0 {
                        if pond && h > -depth {
                            water
                        } else if h == 0 {
                            if r2 < 80 {
                                sand
                            } else {
                                grass
                            }
                        } else {
                            dirt
                        }
                    } else {
                        AIR
                    };
                    self.world.set_block(q, id);
                }
            }
        }
        if let Some(rod) = self.items.id("fishing_rod") {
            self.surv.inv.slots[self.selected] = Some(ItemStack::new(rod, 1));
        }
        let at = (IVec3::new(p.x, ground, p.z) + dir * 9 + side).as_vec3() + Vec3::new(0.5, 0.85, 0.5);
        let mut b = Bobber {
            pos: at,
            vel: Vec3::ZERO,
            in_water: true,
            landed: false,
            phase: Phase::Wait,
            wait: 999.0,
            approach: 2.0,
            approach_len: 3.0,
            from: Vec3::new(side.x as f32, 0.0, side.z as f32),
            nibble: 999.0,
            trail: 0.0,
            dip: 0.0,
            twitch: 0.0,
            catch: Some(Catch { name: "pirarucu", kind: Kind::Fish, tier: 2 }),
            fight: Fight::default(),
            age: 3.0,
        };
        match scene {
            "lancando" => {
                b.in_water = false;
                b.pos = self.body.eye() + f * 4.0 + Vec3::Y * 0.6;
            }
            "beliscada" => {
                b.phase = Phase::Approach;
                b.twitch = 1.0;
                // The trail of ripples so far.
                for k in 0..6 {
                    let d = 1.2 + k as f32 * 0.7;
                    let q = b.pos + b.from * d;
                    self.particles.ripple(Vec3::new(q.x, q.y.floor() + 0.88, q.z), 0.3 + k as f32 * 0.05);
                }
            }
            "mordida" => {
                b.phase = Phase::Bite;
                b.dip = 0.32;
                b.twitch = 1.0;
                let wl = self.reg.get(water).faces[2];
                self.particles.splash(b.pos, wl, 0.25);
                self.particles.ripple(b.pos, 0.9);
            }
            "briga" => {
                b.phase = Phase::Fight;
                b.dip = 0.24;
                b.pos -= (dir.as_vec3()) * 2.0;
                b.fight = Fight { tension: 0.72, stamina: 99.0, run: 99.0, rest: 0.0, dir: side.as_vec3(), slack: 0.0, tick: 0.0, splash: 0.0 };
                let wl = self.reg.get(water).faces[2];
                self.particles.splash(b.pos, wl, 0.2);
                self.particles.ripple(b.pos, 0.6);
                self.particles.fish(b.pos + side.as_vec3() * 0.8, [0.55, 0.6, 0.62]);
            }
            _ => {}
        }
        self.bobber = Some(b);
        log::info!("pesca: scene {scene} at {at}");
        if scene == "simular" {
            self.test_fishing_sim();
        }
    }

    /// MC_PESCA=simular: fish 20 times by the rules, a player striking on every bite and
    /// easing off the reel when the line strains; logs what happened.
    fn test_fishing_sim(&mut self) {
        let dt = 0.05;
        let mut log = Vec::new();
        for round in 0..20 {
            self.bobber = None;
            let at = self.body.pos + self.forward() * 6.0;
            self.rod_last_use = -9.0;
            self.use_rod();
            let mut t = 0.0;
            let mut outcome = String::from("tempo esgotado");
            let mut fought = false;
            while t < 90.0 {
                t += dt;
                self.now += dt;
                let phase = self.bobber.as_ref().map(|b| b.phase);
                let tension = self.bobber.as_ref().map_or(0.0, |b| b.fight.tension);
                self.mouse_held[1] = phase == Some(Phase::Fight) && tension < 0.7;
                let drops = self.surv.drops.len();
                if phase == Some(Phase::Bite) && self.bobber.as_ref().is_some_and(|b| b.wait < 0.3) {
                    self.rod_last_use = -9.0;
                    self.use_rod();
                }
                fought |= phase == Some(Phase::Fight);
                self.update_fishing(dt);
                if self.bobber.is_none() {
                    outcome = if self.surv.drops.len() > drops || phase == Some(Phase::Bite) { "pegou".into() } else { "perdeu".into() };
                    break;
                }
                let _ = at;
            }
            self.mouse_held[1] = false;
            log.push(format!("{round}: {outcome} em {t:.1}s{}", if fought { " (brigou)" } else { "" }));
        }
        log::info!("pesca simulada:\n{}", log.join("\n"));
    }
}

/// The line from `tip` to `end` (world positions) as line pieces relative to `c`: hanging by
/// `sag` in the middle, trembling by `shake`, kept above `floor` (lying on the water).
fn line_verts(tip: Vec3, end: Vec3, sag: f32, shake: f32, floor: Option<f32>, c: Vec3, now: f32) -> Vec<LineVertex> {
    let n = 16;
    let side = (end - tip).cross(Vec3::Y).normalize_or_zero();
    let point = |i: usize| {
        let t = i as f32 / n as f32;
        let belly = 4.0 * t * (1.0 - t);
        let tremble = side * (now * 47.0 + t * 19.0).sin() * shake * belly + Vec3::Y * (now * 39.0 + t * 13.0).cos() * shake * 0.6 * belly;
        let mut q = tip.lerp(end, t) - Vec3::Y * sag * belly + tremble;
        if let Some(f) = floor {
            q.y = q.y.max(f);
        }
        q - c
    };
    let col = [0.12, 0.12, 0.12, 0.9];
    (0..n).flat_map(|i| [LineVertex { pos: point(i).to_array(), color: col }, LineVertex { pos: point(i + 1).to_array(), color: col }]).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn catch_tables_name_real_items() {
        fn read(dir: &std::path::Path, out: &mut String) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    read(&p, out);
                } else if p.extension().is_some_and(|x| x == "ron") {
                    out.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
                }
            }
        }
        let mut all = String::new();
        read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../packs/vanilla")), &mut all);
        let names = super::FISH.iter().chain(super::JUNK).chain(super::TREASURE).map(|t| t.0).chain(super::LOCAL_FISH.iter().map(|t| t.1));
        let missing: Vec<&str> = names.filter(|&n| n != "coins" && !all.contains(&format!("name: \"{n}\""))).collect();
        assert!(missing.is_empty(), "{missing:?}");
    }

    #[test]
    fn big_fish_fight_small_ones_do_not() {
        assert_eq!(super::fish_tier("cod"), 0);
        assert!(super::fish_tier("tuna") >= 1);
        assert!(super::fish_tier("pirarucu") >= 2);
        assert!(super::fish_tier("mud_king") >= 2);
    }
}
