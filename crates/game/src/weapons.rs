//! The player's ranged and special weapons (bow, crossbow, trident, mace, spear, wind charges)
//! and everything enchantments change in play: harder hits, fire, knockback, sweeping, loot,
//! faster digging, silk touch and fortune, less wear, mending, protection, thorns, and the
//! boots and leggings that change how you move.

use glam::{IVec3, Vec3};
use voxel::item::ItemStack;
use voxel::physics::Terrain;
use voxel::AIR;

use crate::audio::{Material, Sfx};
use crate::effects::{self, PotionDef};
use crate::enchanting::*;
use crate::item_render::GpuItem;
use crate::progression::{DamageKind, Event};
use crate::Game;

#[derive(Clone, Copy)]
pub enum ShotKind {
    Arrow { potion: Option<&'static PotionDef>, pickup: bool },
    Trident { stack: ItemStack, returning: bool },
    WindCharge,
}

#[derive(Clone)]
pub struct Shot {
    pub pos: Vec3,
    pub vel: Vec3,
    pub kind: ShotKind,
    pub damage: f32,
    pub crit: bool,
    pub punch: u8,
    pub flame: bool,
    /// Mobs it can still pass through (piercing).
    pub pierce: u8,
    pub hit: Vec<usize>,
    pub loyalty: u8,
    pub channeling: bool,
    pub impaling: u8,
    pub age: f32,
    pub stuck: bool,
    /// What its arrow does besides damage (archery.rs).
    pub fx: crate::archery::ArrowFx,
    /// Lodged in a monster: it rides along until the monster dies.
    pub lodged: Option<crate::archery::Lodged>,
    /// The arrow item it is (drawn with its own picture; None: a plain arrow).
    pub item: Option<voxel::item::ItemId>,
}

impl Shot {
    fn new(pos: Vec3, vel: Vec3, kind: ShotKind, damage: f32) -> Shot {
        Shot { pos, vel, kind, damage, crit: false, punch: 0, flame: false, pierce: 0, hit: Vec::new(), loyalty: 0, channeling: false, impaling: 0, age: 0.0, stuck: false, fx: Default::default(), lodged: None, item: None }
    }
}

/// A fully drawn plain bow's arrow (ore bows multiply it, tr_weapons::bow_power).
const BOW_DAMAGE: f32 = 6.0;
const CROSSBOW_DAMAGE: f32 = 9.0;

/// Damage per second of a bow shooting full draws, or a crossbow reloaded each 1.25 s, for
/// the balance sheet (balance.rs). A full draw is always critical: on average a quarter more
/// and a point. A bow's arrows share the damage of a one-second draw by their draw time and
/// number (`fire_bow`), so this is the same for every draw time and volley when all land.
#[cfg(test)]
pub(crate) fn bow_dps(name: &str) -> Option<f32> {
    match name {
        "bow" => Some(BOW_DAMAGE * 1.25 + 1.0),
        "crossbow" => Some(CROSSBOW_DAMAGE / 1.35),
        _ => crate::tr_weapons::bow_power(name).map(|k| BOW_DAMAGE * k * 1.25 + 1.0),
    }
}

/// Undead for smite, arthropods for bane of arthropods.
fn arthropod(name: &str) -> bool {
    matches!(name, "spider" | "cave_spider" | "silverfish" | "endermite" | "bee")
}

fn water_mob(name: &str) -> bool {
    matches!(name, "duck" | "guardian" | "elder_guardian" | "drowned" | "squid" | "glow_squid" | "dolphin" | "turtle" | "axolotl" | "cod" | "salmon")
}

impl Game {
    fn held_ench(&self, id: u8) -> u8 {
        level(&self.held_stack(), id)
    }

    /// Highest level of an enchantment on the worn armor.
    pub(crate) fn armor_ench(&self, id: u8) -> u8 {
        self.surv.armor.iter().map(|s| level(s, id)).max().unwrap_or(0)
    }

    pub(crate) fn held_key(&self) -> String {
        self.held_stack().map(|s| self.items.get(s.item).name.clone()).unwrap_or_default()
    }

    // -----------------------------------------------------------------------------------------
    // Melee.

    /// Extra damage for a melee hit on mob `i`: enchantments, effects and the mace's smash.
    pub(crate) fn weapon_melee_bonus(&mut self, i: usize, dmg: f32) -> f32 {
        let name = self.mobs.reg.defs[self.mobs.list[i].def].name.clone();
        let mut d = dmg;
        let sharp = self.held_ench(SHARPNESS);
        if sharp > 0 {
            d += 0.5 * sharp as f32 + 0.5;
        }
        if effects::undead(&name) {
            d += 2.5 * self.held_ench(SMITE) as f32;
        }
        if arthropod(&name) {
            d += 2.5 * self.held_ench(BANE) as f32;
            if self.held_ench(BANE) > 0 {
                self.mobs.list[i].chill = self.mobs.list[i].chill.max(1.0 + 0.5 * self.held_ench(BANE) as f32);
            }
        }
        if self.held_key() == "trident" && (water_mob(&name) || self.mobs.list[i].body.in_water) {
            d += 2.5 * self.held_ench(IMPALING) as f32;
        }
        if self.held_key() == "mace" {
            let fall = (self.prog.mace_peak - self.body.pos.y).max(0.0);
            if fall > 1.5 && !self.body.on_ground {
                let f = fall;
                let smash = 4.0 * f.min(3.0) + 2.0 * (f - 3.0).clamp(0.0, 5.0) + (f - 8.0).max(0.0);
                d += smash + 0.5 * self.held_ench(DENSITY) as f32 * f;
                self.mace_smash(i, f);
            }
        }
        self.melee_effects(d)
    }

    fn mace_smash(&mut self, i: usize, fall: f32) {
        let at = self.mobs.list[i].body.pos;
        // Everything near the landing is thrown back; the player stops falling.
        for (k, m) in self.mobs.list.iter_mut().enumerate() {
            if k != i && m.alive() && (m.body.pos - at).length() < 3.5 {
                m.body.knockback(m.body.pos - at, 5.0);
            }
        }
        self.body.vel.y = 0.0;
        self.fall_peak = self.body.pos.y;
        self.prog.mace_peak = self.body.pos.y;
        let wind = self.held_ench(WIND_BURST);
        if wind > 0 {
            self.body.vel.y = 8.0 + 4.0 * wind as f32;
            self.prog.mace_peak = self.body.pos.y + 20.0;
        }
        let ground = self.world.block_or_air((at - Vec3::Y * 0.5).floor().as_ivec3());
        if ground != AIR {
            let (layer, tint) = self.block_look(ground);
            self.particles.block_bits(at.floor().as_ivec3(), layer, tint, 24, 1.4);
        }
        self.sfx(Sfx::Explode);
        self.prog.event(Event::MaceSmash(fall));
        self.wear_held(1);
    }

    /// After a melee hit lands: fire, extra knockback, the sweep, the spear's lunge.
    pub(crate) fn after_melee(&mut self, i: usize, dmg: f32) {
        let eye = self.body.eye();
        let fire = self.held_ench(FIRE_ASPECT);
        let kb = self.held_ench(KNOCKBACK);
        let sweep = self.held_ench(SWEEPING);
        let sword = self.held_tool().is_some_and(|t| t.kind == voxel::block::ToolKind::Sword);
        let target = self.mobs.list[i].body.pos;
        {
            let m = &mut self.mobs.list[i];
            if fire > 0 {
                m.on_fire = m.on_fire.max(4.0 * fire as f32);
            }
            if kb > 0 {
                m.body.knockback(m.body.pos - eye, 3.0 * kb as f32);
            }
        }
        // A sword swung standing still catches the mobs next to the one hit (weapons with an
        // arc of their own, mods/combat.rs, already did).
        if sword && self.held_weapon().is_none() && !self.sprinting && self.body.on_ground && self.attack_charge_was_full() {
            let extra = 1.0 + dmg * if sweep > 0 { sweep as f32 / (sweep as f32 + 1.0) } else { 0.0 };
            for (k, m) in self.mobs.list.iter_mut().enumerate() {
                if k != i && m.alive() && (m.body.pos - target).length() < 1.8 && !self.mobs.reg.defs[m.def].town {
                    m.hurt(extra, eye);
                }
            }
            self.prog.sweep = 0.25;
        }
        // The spear's lunge: a quick dash forward with the hit.
        let lunge = self.held_ench(LUNGE);
        if self.held_key().ends_with("_spear") && lunge > 0 && self.surv.hunger > 6.0 {
            let f = self.forward();
            self.body.vel += Vec3::new(f.x, 0.1, f.z).normalize_or_zero() * (4.0 + 2.5 * lunge as f32);
            self.surv.exhaustion += 1.0;
        }
        self.prog.stats.add("damage_dealt", (dmg * 10.0) as u64);
        self.rpg_after_melee(i, dmg);
    }

    fn attack_charge_was_full(&self) -> bool {
        // The charge was used up by the hit just now; a fresh click after a full wait counts.
        self.prog.was_full
    }

    /// Reach for hitting mobs: 3 blocks in survival and 5 in creative, as in Java Edition;
    /// spears reach furthest of all (HP 2026-10-31: further than swords and every other melee
    /// weapon, whips included).
    pub(crate) fn weapon_reach(&self) -> f32 {
        if crate::spear::is_spear(&self.held_key()) {
            crate::mods::combat::SPEAR_REACH
        } else if crate::rpg::is_whip(&self.held_key()) {
            // Whips reach further (rpg.rs).
            5.0
        } else if self.survival {
            3.0
        } else {
            5.0
        }
    }

    // -----------------------------------------------------------------------------------------
    // Digging, drops and wear.

    /// Divides dig time: efficiency (with the right tool), haste, fatigue, under water.
    pub(crate) fn dig_speed_bonus(&self, id: voxel::BlockId) -> f32 {
        let mut k = 1.0;
        // The tool as it digs this block (a multi-tool turns into the right kind).
        if let Some(t) = self.held_tool_for(id) {
            let e = self.held_ench(EFFICIENCY) as f32;
            if e > 0.0 && self.reg.get(id).tool == Some(t.kind) {
                k *= (t.speed + e * e + 1.0) / t.speed.max(0.1);
            }
        }
        k *= self.dig_effects();
        k *= self.shears_speed(id);
        k *= self.skill_dig_speed(id);
        k *= self.tr3_dig_factor();
        k *= self.airborne_dig_factor();
        if self.body.head_in_water && self.armor_ench(AQUA_AFFINITY) == 0 {
            k /= 5.0;
        }
        k.max(0.0001)
    }

    /// Digging with the feet off the ground is five times slower, as in Java Edition, except
    /// when flying, swimming (water has its own rule), hanging from a hook or riding.
    pub(crate) fn airborne_dig_factor(&self) -> f32 {
        let b = &self.body;
        if b.on_ground || b.flying || b.in_water || self.t5.hook.attached() || self.t5.comp.mount.is_some() {
            1.0
        } else {
            0.2
        }
    }

    /// Silk touch keeps the block itself; fortune multiplies what ores give.
    pub(crate) fn ench_drops(&mut self, id: voxel::BlockId, drops: Vec<ItemStack>) -> Vec<ItemStack> {
        let tool = self.held_tool();
        if self.held_ench(SILK_TOUCH) > 0 && self.reg.get(id).drops.is_some() && self.items.can_harvest(&self.reg, id, tool.as_ref()) {
            if let Some(i) = self.items.of_block(self.reg.get(id).base) {
                return vec![ItemStack::new(i, 1)];
            }
        }
        let fortune = self.held_ench(FORTUNE) as u32;
        if fortune == 0 {
            return drops;
        }
        // Any ore that gives something other than itself (zanite, ambrosium, sulfur...).
        let from_ore = self.reg.get(id).name.ends_with("_ore");
        let ore_item = self.items.of_block(self.reg.get(id).base);
        drops
            .into_iter()
            .map(|mut d| {
                let n = &self.items.get(d.item).name;
                let ore = (from_ore && Some(d.item) != ore_item) || n.starts_with("raw_") || matches!(n.as_str(), "coal" | "diamond" | "emerald" | "lapis_lazuli" | "redstone" | "quartz" | "ruby" | "sapphire" | "topaz" | "amethyst" | "amethyst_shard" | "gold_nugget" | "glowstone_dust" | "echo_shard" | "flint" | "wheat_seeds" | "melon_slice");
                if ore {
                    let bonus = (self.prog.rng.below(fortune + 2) as i32 - 1).max(0) as u32;
                    d.count = (d.count as u32 * (bonus + 1)).min(voxel::item::PILE as u32) as u16;
                }
                d
            })
            .collect()
    }

    /// Unbreaking: the chance this use doesn't wear the item.
    pub(crate) fn unbreaking_saves(&mut self, s: &ItemStack, armor: bool) -> bool {
        let u = s.ench.get(UNBREAKING) as f32;
        if u == 0.0 {
            return false;
        }
        let keep = if armor { 0.6 + 0.4 / (u + 1.0) } else { 1.0 / (u + 1.0) };
        self.prog.rng.f32() >= keep
    }

    /// Mending: experience picked up repairs worn mending items first (2 durability per point).
    pub(crate) fn mend_with_xp(&mut self, xp: u32) -> u32 {
        let mut left = xp;
        let slots: Vec<(bool, usize)> = (0..4).map(|i| (true, i)).chain(std::iter::once((false, self.selected))).collect();
        for (armor, i) in slots {
            if left == 0 {
                break;
            }
            let st = if armor { self.surv.armor[i] } else { self.surv.inv.slots[i] };
            let Some(mut st) = st else { continue };
            if st.ench.get(MENDING) == 0 || st.damage == 0 {
                continue;
            }
            let fix = (left * 2).min(st.damage as u32);
            st.damage -= fix as u16;
            left -= fix.div_ceil(2);
            if armor {
                self.surv.armor[i] = Some(st);
            } else {
                self.surv.inv.slots[i] = Some(st);
            }
        }
        left
    }

    /// Hits on the player: protection enchantments, resistance, fire resistance, slow falling and
    /// golden absorption hearts. Thorns hurts the attacker back.
    pub(crate) fn protect(&mut self, amount: f32, from: Option<Vec3>) -> f32 {
        let kind = std::mem::take(&mut self.prog.next_kind);
        let kind = if kind == DamageKind::Attack && from.is_none() { DamageKind::Magic } else { kind };
        if kind == DamageKind::Fire && self.has_effect(effects::FIRE_RESISTANCE) {
            return 0.0;
        }
        if kind == DamageKind::Fall && self.has_effect(effects::SLOW_FALLING) {
            return 0.0;
        }
        let mut epf = 0u32;
        for s in self.surv.armor.iter().flatten() {
            epf += s.ench.get(PROTECTION) as u32;
            epf += 2 * match kind {
                DamageKind::Fire => s.ench.get(FIRE_PROTECTION),
                DamageKind::Explosion => s.ench.get(BLAST_PROTECTION),
                DamageKind::Projectile => s.ench.get(PROJECTILE_PROTECTION),
                _ => 0,
            } as u32;
            if kind == DamageKind::Fall {
                epf += 3 * s.ench.get(FEATHER_FALLING) as u32;
            }
        }
        if matches!(kind, DamageKind::Starve) {
            epf = 0;
        }
        let mut a = amount * (1.0 - epf.min(20) as f32 * 0.04);
        a *= (1.0 - 0.2 * self.effect_level(effects::RESISTANCE) as f32).max(0.0);
        if self.prog.absorb > 0.0 {
            let used = self.prog.absorb.min(a);
            self.prog.absorb -= used;
            a -= used;
        }
        // Thorns: the attacker gets hurt back now and then.
        if let (Some(f), DamageKind::Attack) = (from, kind) {
            let t = self.armor_ench(THORNS) as f32;
            if t > 0.0 && self.prog.rng.f32() < 0.15 * t {
                let dmg = 1.0 + self.prog.rng.below(4) as f32;
                let me = self.body.pos;
                if let Some(m) = self.mobs.list.iter_mut().filter(|m| m.alive()).min_by(|a, b| (a.body.pos - f).length().total_cmp(&(b.body.pos - f).length())) {
                    if (m.body.pos - f).length() < 3.0 {
                        m.hurt(dmg, me);
                    }
                }
            }
        }
        if a > 0.0 {
            self.prog.stats.add("damage_taken", (a * 10.0) as u64);
        }
        a
    }

    // -----------------------------------------------------------------------------------------
    // Moving: speed effects, swift sneak, soul speed, depth strider, frost walker, levitation...

    pub(crate) fn pre_step(&mut self, sneaking: bool) {
        let mut m = self.move_multiplier();
        let ss = self.armor_ench(SWIFT_SNEAK) as f32;
        if sneaking && ss > 0.0 {
            m *= (0.3 + 0.15 * ss) / 0.3;
        }
        let soul = self.armor_ench(SOUL_SPEED) as f32;
        if soul > 0.0 {
            let below = self.world.block_or_air((self.body.pos - Vec3::Y * 0.2).floor().as_ivec3());
            let n = &self.reg.get(below).name;
            if n == "soul_sand" || n == "soul_soil" {
                m *= 1.3 + 0.105 * soul;
            }
        }
        self.body.speed_override = if (m - 1.0).abs() > 0.001 { Some(voxel::physics::WALK_SPEED * m) } else { None };
        self.prog.prev_ground = self.body.on_ground;
        if !self.body.on_ground {
            self.prog.mace_peak = self.prog.mace_peak.max(self.body.pos.y);
        } else {
            self.prog.mace_peak = self.body.pos.y;
        }
    }

    pub(crate) fn post_step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let jb = self.effect_level(effects::JUMP_BOOST) as f32;
        if jb > 0.0 && self.prog.prev_ground && !self.body.on_ground && self.body.vel.y > 8.0 {
            self.body.vel.y += 1.8 * jb;
        }
        let lev = self.effect_level(effects::LEVITATION) as f32;
        if lev > 0.0 {
            let target = 0.9 * lev;
            self.body.vel.y += (target - self.body.vel.y) * (dt * 4.0).min(1.0);
            self.fall_peak = self.body.pos.y;
            self.prog.wind_from = self.body.pos.y.max(0.01);
        }
        if self.has_effect(effects::SLOW_FALLING) && self.body.vel.y < -1.6 {
            self.body.vel.y = -1.6;
            self.fall_peak = self.body.pos.y;
        }
        // In water: depth strider and dolphin's grace push the swim along.
        if self.body.in_water && !self.body.flying {
            let ds = self.armor_ench(DEPTH_STRIDER) as f32;
            let dg = self.has_effect(effects::DOLPHINS_GRACE);
            let k = 1.0 + ds / 3.0 * 0.8 + if dg { 1.2 } else { 0.0 };
            if k > 1.0 {
                let h = Vec3::new(self.body.vel.x, 0.0, self.body.vel.z);
                let cap = 2.2 * k;
                let boosted = h * (1.0 + (k - 1.0) * (dt * 6.0).min(1.0));
                let v = if boosted.length() > cap { boosted.normalize() * cap } else { boosted };
                self.body.vel.x = v.x;
                self.body.vel.z = v.z;
            }
        }
        // Frost walker: still water under the feet freezes for a while.
        let fw = self.armor_ench(FROST_WALKER) as i32;
        if fw > 0 && self.body.on_ground && !self.body.in_water {
            let water = self.reg.id_or_air("water");
            let ice = self.reg.id_or_air("frosted_ice");
            if ice != AIR {
                let c = (self.body.pos - Vec3::Y * 0.5).floor().as_ivec3();
                let r = 2 + fw;
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r * r {
                            continue;
                        }
                        let p = c + IVec3::new(dx, 0, dz);
                        if self.world.block_or_air(p) == water && self.world.block_or_air(p + IVec3::Y) == AIR {
                            self.world.set_block(p, ice);
                            let t = 6.0 + self.prog.rng.f32() * 6.0;
                            self.prog.frost.push((p, t));
                        }
                    }
                }
            }
        }
        if self.prog.spin > 0.0 {
            self.prog.spin -= dt;
            self.fall_peak = self.body.pos.y;
        }
        // Statistics: distance on foot, by swimming, flying; jumps.
        let moved = (self.body.pos - self.prog.last_pos).length().min(4.0);
        self.prog.last_pos = self.body.pos;
        self.prog.stat_walk += moved;
        if self.prog.stat_walk >= 1.0 {
            let n = self.prog.stat_walk as u64;
            self.prog.stat_walk -= n as f32;
            let key = if self.body.in_water { "cm_swum" } else if self.body.flying { "cm_flown" } else if self.riding.is_some() { "cm_by_cart" } else { "cm_walked" };
            self.prog.stats.add(key, n * 100);
        }
        if self.prog.prev_ground && !self.body.on_ground && self.body.vel.y > 8.0 {
            self.prog.stats.add("jumps", 1);
        }
    }

    // -----------------------------------------------------------------------------------------
    // Bows, crossbows, tridents, wind charges.

    /// Right click with a ranged weapon or a wind charge. True when it is one (the draw happens
    /// while the button is held, in `update_weapons`).
    pub(crate) fn weapon_use(&mut self, name: &str) -> bool {
        if crate::tr_weapons::bow_power(name).is_some() {
            return true;
        }
        match name {
            "bow" | "trident" => true,
            "crossbow" => true,
            "wind_charge" => {
                if self.prog.wind_t > 0.0 {
                    return true;
                }
                self.prog.wind_t = 0.5;
                let f = self.forward();
                let from = self.body.eye() + f * 0.5;
                self.prog.shots.push(Shot::new(from, f * 24.0, ShotKind::WindCharge, 1.0));
                if self.survival {
                    let sel = self.selected;
                    self.surv.inv.take(sel, 1);
                }
                self.anim.start_swing();
                self.sfx(Sfx::Cast(5));
                true
            }
            _ => false,
        }
    }

    /// An arrow from the inventory: tipped arrows first if one is in hand's reach... the first
    /// arrow stack found, like vanilla (offhand, then hotbar, then the rest).
    pub(crate) fn find_arrow(&self) -> Option<(usize, Option<&'static PotionDef>)> {
        let off = self.surv.offhand.map(|s| self.items.get(s.item).name.clone());
        if let Some(n) = off {
            if n == "arrow" || n == "spectral_arrow" || crate::tr3_tools::is_arrow(&n) {
                return Some((usize::MAX, None));
            }
            if let Some((effects::Form::Arrow, p)) = effects::potion_of(&n) {
                return Some((usize::MAX, Some(p)));
            }
        }
        for (i, s) in self.surv.inv.slots.iter().enumerate() {
            let Some(s) = s else { continue };
            let n = &self.items.get(s.item).name;
            if n == "arrow" || n == "spectral_arrow" || crate::tr3_tools::is_arrow(n) {
                return Some((i, None));
            }
            if let Some((effects::Form::Arrow, p)) = effects::potion_of(n) {
                return Some((i, Some(p)));
            }
        }
        None
    }

    fn use_arrow(&mut self, slot: usize) {
        if !self.survival {
            return;
        }
        if slot == usize::MAX {
            if let Some(s) = &mut self.surv.offhand {
                s.count -= 1;
                if s.count == 0 {
                    self.surv.offhand = None;
                }
            }
        } else {
            self.surv.inv.take(slot, 1);
        }
    }

    pub(crate) fn update_weapons(&mut self, dt: f32, use_held: bool) {
        self.prog.wind_t -= dt;
        self.prog.sweep = (self.prog.sweep - dt).max(0.0);
        let mut name = self.held_key();
        // Ore bows (tr_weapons.rs) draw like the plain bow.
        if crate::tr_weapons::bow_power(&name).is_some() {
            name = "bow".into();
        }
        let playing = self.screen == crate::Screen::Playing;
        let holding = use_held && playing;
        match name.as_str() {
            "bow" => {
                // Held down it shoots by itself each time the string is fully back, like
                // Terraria's bows; let go early, it shoots what was drawn.
                let draw = self.bow_draw_time();
                if holding {
                    let own_ammo = crate::archery::bow_stats(&self.held_key()).is_some_and(|b| b.trick == crate::archery::Trick::Ice);
                    if own_ammo || self.find_arrow().is_some() || !self.survival || self.held_ench(INFINITY) > 0 {
                        if self.prog.draw == 0.0 && dt > 0.0 {
                            // The wood creaks as the string comes back (quieter on quick bows).
                            let w = crate::archery::bow_weight(&self.held_key());
                            let eye = self.body.eye();
                            self.sfx_at(Sfx::Fx(crate::sfx_extra::Fx::BowDraw(w)), eye, (draw / 0.8).min(1.0) * 0.55, 16.0);
                        }
                        self.prog.draw += dt;
                        if self.prog.draw >= draw {
                            self.fire_bow();
                        }
                    }
                } else if self.prog.was_using && self.prog.draw > 0.1 * draw {
                    self.fire_bow();
                }
            }
            "crossbow" => {
                // A firework rocket in the other hand is loaded before arrows (like Java).
                let rocket = self.surv.offhand.is_some_and(|s| self.items.get(s.item).name == "firework_rocket");
                if holding && self.prog.loaded && !self.prog.was_using {
                    // Loaded: a fresh press shoots (here, not in surv_use, so it never opens
                    // the door it aims at).
                    self.fire_crossbow();
                } else if holding && !self.prog.loaded {
                    if rocket || self.find_arrow().is_some() || !self.survival {
                        self.prog.draw += dt;
                        let need = (1.25 - 0.25 * self.held_ench(QUICK_CHARGE) as f32).max(0.1);
                        if self.prog.draw >= need {
                            self.prog.loaded = true;
                            self.prog.draw = 0.0;
                            self.prog.loaded_ammo = (None, None, false);
                            if rocket {
                                self.prog.loaded_ammo.2 = true;
                                if self.survival {
                                    self.surv.offhand = self.surv.offhand.and_then(|mut s| {
                                        s.count -= 1;
                                        (s.count > 0).then_some(s)
                                    });
                                }
                            } else if let Some((slot, p)) = self.find_arrow() {
                                let st = if slot == usize::MAX { self.surv.offhand } else { self.surv.inv.slots[slot] };
                                self.prog.loaded_ammo = (st.map(|s| s.item), p, false);
                                self.use_arrow(slot);
                            }
                            self.sfx(Sfx::Fx(crate::sfx_extra::Fx::CrossbowLoad));
                        }
                    }
                } else if !holding {
                    self.prog.draw = 0.0;
                }
            }
            "trident" => {
                if holding {
                    self.prog.draw += dt;
                } else if self.prog.was_using && self.prog.draw > 0.5 {
                    self.throw_trident();
                }
            }
            _ => self.prog.draw = 0.0,
        }
        if !holding && name != "crossbow" {
            self.prog.draw = if self.prog.was_using { 0.0 } else { self.prog.draw.min(0.0) };
        }
        self.prog.was_using = holding;
        self.update_weapon_shots(dt);
    }

    /// How far the bow is drawn (0..1), for the arm pose and the zoom.
    pub(crate) fn bow_pull(&self) -> f32 {
        let n = self.held_key();
        if let Some(k) = test_bow_pull().filter(|_| crate::bowdraw::is_bow(&n)) {
            return k;
        }
        if n == "bow" || crate::tr_weapons::bow_power(&n).is_some() {
            (self.prog.draw / self.bow_draw_time()).min(1.0)
        } else if n == "crossbow" {
            if self.prog.loaded {
                1.0
            } else {
                (self.prog.draw / 1.25).min(1.0)
            }
        } else {
            0.0
        }
    }

    /// Seconds the held bow takes to draw fully (archery.rs).
    fn bow_draw_time(&self) -> f32 {
        crate::archery::bow_stats(&self.held_key()).map_or(1.0, |b| b.draw)
    }

    fn fire_bow(&mut self) {
        let stats = crate::archery::bow_stats(&self.held_key()).unwrap_or(crate::archery::BowStats { draw: 1.0, arrows: 1, spread: 0.0, speed: 1.0, fire: false, trick: crate::archery::Trick::None });
        let pull = (self.prog.draw / stats.draw).min(1.0);
        self.prog.draw = 0.0;
        let pull = (pull * pull + pull * 2.0) / 3.0;
        if pull < 0.1 {
            return;
        }
        let infinity = self.held_ench(INFINITY) > 0;
        // The frost bow makes its own shards of ice.
        let own_ammo = stats.trick == crate::archery::Trick::Ice;
        let arrow = if own_ammo { None } else { self.find_arrow() };
        if self.survival && arrow.is_none() && !infinity && !own_ammo {
            return;
        }
        let potion = arrow.and_then(|a| a.1);
        let mut pickup = self.survival && !own_ammo;
        let arrow_item = arrow.and_then(|(slot, _)| if slot == usize::MAX { self.surv.offhand } else { self.surv.inv.slots[slot] }).map(|s| s.item);
        let arrow_name = arrow_item.map(|i| self.items.get(i).name.clone());
        // A Terraria arrow (tr3_tools.rs) bites its own way and isn't picked up again.
        let tr_arrow = arrow_name.as_deref().and_then(crate::tr3_tools::arrow_traits);
        if tr_arrow.is_some() {
            pickup = false;
        }
        // The molten quiver lights plain arrows (rpg.rs).
        let lit = self.t5.rpg.stats.flame_arrows && matches!(arrow_name.as_deref(), None | Some("arrow"));
        let mut fx = if lit { crate::archery::arrow_fx("flaming_arrow") } else { arrow_name.as_deref().map(crate::archery::arrow_fx).unwrap_or_default() };
        if stats.fire {
            fx.burn = fx.burn.max(4.0);
            fx.trail = fx.trail.or(Some([1.4, 0.6, 0.2]));
        }
        if let Some((slot, p)) = arrow {
            // The hunter's quiver sometimes saves the arrow (rpg.rs).
            let saved = self.t5.rpg.stats.ammo_save > 0.0 && self.surv.rng.f32() < self.t5.rpg.stats.ammo_save;
            if !(infinity && p.is_none()) && !saved {
                self.use_arrow(slot);
            } else {
                pickup = false;
            }
        }
        let power = self.held_ench(POWER) as f32;
        let ranged = self.t5.rpg.stats.dmg[crate::rpg::RANGED];
        let ore = crate::tr_weapons::bow_power(&self.held_key()).unwrap_or(1.0);
        // The volley shares a one-second draw's damage (see `bow_dps`).
        let share = stats.draw / stats.arrows.max(1) as f32;
        let jitter: Vec<f32> = (0..stats.arrows).map(|_| self.surv.rng.f32() * 2.0 - 1.0).collect();
        let dirs = crate::archery::volley(self.forward(), stats.arrows, stats.spread, &jitter);
        // A trick bow changes what flies (archery.rs `turn`).
        let turned = crate::archery::turn(stats.trick, if own_ammo { Some("ice") } else { arrow_name.as_deref() });
        // The storm bow's arrows come down from the sky around the spot aimed at.
        let rain_at = (stats.trick == crate::archery::Trick::Rain).then(|| {
            // On the monster aimed at, if any, else where the look meets the ground.
            let (eye, f) = (self.body.eye(), self.forward());
            match self.mobs.pick(eye, f, 64.0) {
                Some((k, _)) => self.mobs.list[k].body.pos,
                None => self.aim_point(64.0),
            }
        });
        let mut starts: Vec<(Vec3, Vec3)> = Vec::new();
        for f in dirs {
            if let Some(at) = rain_at {
                let a = self.surv.rng.f32() * std::f32::consts::TAU;
                let r = 0.5 + self.surv.rng.f32() * 2.5;
                let land = at + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                let from = land + Vec3::new(self.surv.rng.f32() * 4.0 - 2.0, 16.0 + self.surv.rng.f32() * 4.0, self.surv.rng.f32() * 4.0 - 2.0);
                starts.push((from, (land - from).normalize_or_zero()));
                let _ = f;
            } else if let Some(t) = &turned {
                for k in 0..t.n {
                    // Bees and bats leave in a little swarm.
                    let wob = if t.n > 1 { Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 0.35 } else { Vec3::ZERO };
                    let _ = k;
                    starts.push((self.body.eye() + f * 0.3, (f + wob).normalize_or_zero()));
                }
            } else {
                starts.push((self.body.eye() + f * 0.3, f));
            }
        }
        let swarm = turned.as_ref().is_some_and(|t| t.fx.orb);
        for (i, (from, f)) in starts.into_iter().enumerate() {
            // One arrow comes from the quiver; the extra ones of a volley aren't picked up.
            let kind = ShotKind::Arrow { potion, pickup: pickup && i == 0 && !swarm };
            let speed = if swarm { 30.0 } else { 60.0 * pull * (1.0 + self.t5.rpg.stats.arrow_speed) };
            let mut s = Shot::new(from, f * speed * stats.speed, kind, 0.0);
            s.damage = (BOW_DAMAGE * pull) * if power > 0.0 { 1.0 + 0.25 * (power + 1.0) } else { 1.0 } * ranged * ore * share;
            s.crit = pull >= 1.0 || pull > 0.6 && self.surv.rng.f32() < self.t5.rpg.stats.crit[crate::rpg::RANGED];
            s.punch = self.held_ench(PUNCH);
            s.flame = self.held_ench(FLAME) > 0 || fx.burn > 0.0;
            s.fx = fx;
            s.item = if lit { self.items.id("flaming_arrow").or(arrow_item) } else { arrow_item };
            if let Some((k, pierce, flame)) = tr_arrow {
                s.damage *= k;
                s.pierce = s.pierce.max(pierce);
                s.flame |= flame;
            }
            if let Some(t) = &turned {
                // The trick's own bite on top of the arrow's (a burning arrow stays burning).
                let keep = s.fx;
                s.fx = crate::archery::ArrowFx { burn: t.fx.burn.max(keep.burn), debuff: t.fx.debuff.or(keep.debuff), trail: t.fx.trail.or(keep.trail), glow: keep.glow, ..t.fx };
                s.damage *= t.dmg;
                s.pierce = s.pierce.saturating_add(t.pierce);
                s.flame |= t.fx.burn > 0.0;
                if let Some(look) = t.look {
                    s.item = self.items.id(look).or(s.item);
                }
                if t.fx.orb {
                    s.kind = ShotKind::Arrow { potion: None, pickup: false };
                }
            }
            if rain_at.is_some() {
                s.fx.straight = true;
                s.vel = f * 50.0;
            }
            self.prog.shots.push(s);
        }
        self.wear_held(1);
        self.sfx(Sfx::Fx(crate::sfx_extra::Fx::BowRelease(crate::archery::bow_weight(&self.held_key()))));
        self.prog.stats.add("arrows_shot", 1);
    }

    fn fire_crossbow(&mut self) {
        self.prog.loaded = false;
        let f = self.forward();
        let multi = self.held_ench(MULTISHOT) > 0;
        let pierce = self.held_ench(PIERCING);
        let angles: &[f32] = if multi { &[-0.17, 0.0, 0.17] } else { &[0.0] };
        // The loaded arrow keeps its kind: potion, Terraria bite, or a rocket.
        let (item, potion, rocket) = std::mem::take(&mut self.prog.loaded_ammo);
        let name = if rocket { Some("firework_rocket".to_string()) } else { item.map(|i| self.items.get(i).name.clone()) };
        let fx = name.as_deref().map(crate::archery::arrow_fx).unwrap_or_default();
        let tr = name.as_deref().and_then(crate::tr3_tools::arrow_traits);
        let look = if rocket { self.items.id("firework_rocket") } else { item };
        for &a in angles {
            let dir = Vec3::new(f.x * a.cos() - f.z * a.sin(), f.y, f.x * a.sin() + f.z * a.cos()).normalize_or_zero();
            let pickup = a == 0.0 && self.survival && !rocket && tr.is_none();
            let mut s = Shot::new(self.body.eye() + dir * 0.3, dir * if rocket { 40.0 } else { 63.0 }, ShotKind::Arrow { potion, pickup }, CROSSBOW_DAMAGE * self.t5.rpg.stats.dmg[crate::rpg::RANGED]);
            s.pierce = pierce;
            s.crit = self.surv.rng.f32() < self.t5.rpg.stats.crit[crate::rpg::RANGED];
            s.fx = fx;
            s.item = look;
            s.flame = fx.burn > 0.0;
            if let Some((k, p, flame)) = tr {
                s.damage *= k;
                s.pierce = s.pierce.max(p);
                s.flame |= flame;
            }
            self.prog.shots.push(s);
        }
        self.wear_held(if multi { 3 } else { 1 });
        self.sfx(Sfx::Fx(crate::sfx_extra::Fx::CrossbowShot));
    }

    fn throw_trident(&mut self) {
        self.prog.draw = 0.0;
        let Some(st) = self.held_stack() else { return };
        let riptide = st.ench.get(RIPTIDE);
        let f = self.forward();
        if riptide > 0 {
            // Riptide: only in water or rain; the player is flung instead.
            if self.body.in_water || self.weather.rain > 0.3 {
                self.body.vel = f * (10.0 + 6.0 * riptide as f32);
                self.prog.spin = 1.0;
                self.sfx(Sfx::Splash(0.6));
                self.wear_held(1);
            }
            return;
        }
        let mut s = Shot::new(self.body.eye() + f * 0.4, f * 50.0, ShotKind::Trident { stack: st, returning: false }, 8.0);
        s.loyalty = st.ench.get(LOYALTY);
        s.channeling = st.ench.get(CHANNELING) > 0;
        s.impaling = st.ench.get(IMPALING);
        self.prog.shots.push(s);
        if self.survival {
            let sel = self.selected;
            self.surv.inv.slots[sel] = None;
        }
        self.sfx(Sfx::Cast(0));
    }

    fn update_weapon_shots(&mut self, dt: f32) {
        self.test_volleys();
        self.step_weapon_shots(dt);
    }

    /// Test hook: MC_SHOOT=n shoots n full draws of the held bow at the start, fanned across
    /// the view, and lets them fly until they land (MC_ARROW=name gives them that arrow's
    /// bite), so a screenshot shows them lodged in blocks and monsters.
    fn test_volleys(&mut self) {
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let Some(n) = std::env::var("MC_SHOOT").ok().and_then(|v| v.parse::<u32>().ok()) else { return };
        if self.held_bow().is_none() || DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        if let Ok(d) = std::env::var("MC_WALL") {
            // MC_WALL=distance: a plank wall that far ahead to catch the arrows.
            let d: f32 = d.parse().unwrap_or(10.0);
            let f = self.forward();
            let fwd = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
            let side = Vec3::new(-fwd.z, 0.0, fwd.x);
            let planks = self.reg.id_or_air("oak_planks");
            for w in -9..=9 {
                for h in -1..7 {
                    let p = (self.body.pos + fwd * d + side * w as f32 + Vec3::Y * h as f32).floor().as_ivec3();
                    self.world.set_block(p, planks);
                }
            }
        }
        let fx = std::env::var("MC_ARROW").ok().map(|a| crate::archery::arrow_fx(&a));
        let (yaw, pitch) = (self.yaw, self.pitch);
        for k in 0..n {
            let t = if n > 1 { k as f32 / (n - 1) as f32 - 0.5 } else { 0.0 };
            self.yaw = yaw + t * 0.7;
            self.pitch = pitch + ((k * 7 % 5) as f32 / 4.0 - 0.5) * 0.25;
            let before = self.prog.shots.len();
            self.prog.draw = 10.0;
            self.fire_bow();
            if let Some(fx) = fx {
                for s in &mut self.prog.shots[before..] {
                    s.fx = fx;
                    s.flame |= fx.burn > 0.0;
                }
            }
        }
        self.yaw = yaw;
        self.pitch = pitch;
        for _ in 0..200 {
            self.step_weapon_shots(0.01);
        }
        // MC_SHOOT_VIEW=dx,dy,dz,yaw,pitch: then step aside to look at where they landed.
        if let Some(v) = std::env::var("MC_SHOOT_VIEW").ok().map(|v| v.split(',').filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<_>>()).filter(|v| v.len() == 5) {
            self.body.pos += Vec3::new(v[0], v[1], v[2]);
            self.yaw = v[3].to_radians();
            self.pitch = v[4].to_radians();
        }
    }

    fn step_weapon_shots(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let eye = self.body.eye();
        let mut i = 0;
        while i < self.prog.shots.len() {
            let mut remove = false;
            {
                let s = &mut self.prog.shots[i];
                s.age += dt;
                if s.age > 60.0 {
                    remove = true;
                }
            }
            // A loyal trident flies home.
            if let ShotKind::Trident { returning: true, stack } = self.prog.shots[i].kind {
                let s = &mut self.prog.shots[i];
                let to = eye - s.pos;
                s.stuck = false;
                s.vel = to.normalize_or_zero() * (20.0 + 6.0 * s.loyalty as f32);
                s.pos += s.vel * dt;
                if to.length() < 1.2 {
                    self.give_back_stack(stack);
                    remove = true;
                }
            } else if let Some(l) = self.prog.shots[i].lodged {
                // Riding in a monster: where it is now, gone with it (or after a while).
                match self.mobs.list.iter().find(|m| m.net_id == l.mob && m.alive()) {
                    Some(m) if self.prog.shots[i].age < 20.0 => {
                        let (tip, dir) = l.place(m.body.pos, m.yaw);
                        let s = &mut self.prog.shots[i];
                        s.pos = tip;
                        s.vel = dir * 0.001;
                    }
                    _ => remove = true,
                }
            } else if !self.prog.shots[i].stuck {
                let (pos, vel, kind, fx) = {
                    let s = &self.prog.shots[i];
                    (s.pos, s.vel, s.kind, s.fx)
                };
                let g = if matches!(kind, ShotKind::WindCharge) || fx.straight || fx.homing > 0.0 { 0.0 } else { 20.0 };
                let mut v = vel;
                v.y -= g * dt;
                if fx.homing > 0.0 {
                    v = self.seek(pos, v, fx.homing * dt, &self.prog.shots[i].hit);
                }
                if fx.life > 0.0 && self.prog.shots[i].age > fx.life {
                    // A bee or bat runs out of breath: a puff of its light.
                    let glow = self.glow_layer();
                    if let Some(c) = fx.trail {
                        for _ in 0..4 {
                            let r = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5);
                            self.particles.glow(pos, r, glow, c, 0.06, 0.4);
                        }
                    }
                    self.prog.shots.swap_remove(i);
                    continue;
                }
                if fx.split > 0 && self.prog.shots[i].age > 0.18 {
                    self.split_shot(i);
                }
                let next = pos + v * dt;
                // Mobs on the way.
                let hit = self.mobs.pick(pos, v.normalize_or_zero(), (next - pos).length() + 0.3).filter(|(k, _)| !self.prog.shots[i].hit.contains(k));
                if let Some((k, _)) = hit {
                    remove = self.shot_hits_mob(i, k);
                } else if Terrain::solid(&self.world, next.floor().as_ivec3()) {
                    match kind {
                        ShotKind::WindCharge => {
                            self.wind_burst(pos, 1.0);
                            remove = true;
                        }
                        ShotKind::Trident { stack, .. } => {
                            let s = &mut self.prog.shots[i];
                            s.stuck = true;
                            if s.loyalty > 0 {
                                s.kind = ShotKind::Trident { stack, returning: true };
                            }
                        }
                        _ if fx.bounce > 0 => {
                            // Glances off: the side of the block it crossed into turns it back.
                            let (a, b) = (pos.floor(), next.floor());
                            let mut w = v;
                            for axis in 0..3 {
                                if a[axis] != b[axis] {
                                    let mut probe = pos;
                                    probe[axis] = next[axis];
                                    if Terrain::solid(&self.world, probe.floor().as_ivec3()) {
                                        w[axis] = -w[axis];
                                    }
                                }
                            }
                            if w == v {
                                w = -v;
                            }
                            let s = &mut self.prog.shots[i];
                            s.vel = w * 0.9;
                            s.fx.bounce -= 1;
                            s.hit.clear();
                            let glow = self.glow_layer();
                            for _ in 0..5 {
                                let r = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 3.0;
                                self.particles.glow(pos, r, glow, fx.trail.unwrap_or([1.0; 3]), 0.05, 0.25);
                            }
                            self.sfx_at(Sfx::Fx(crate::sfx_extra::Fx::ArrowHit(4)), pos, 0.5, 20.0);
                        }
                        _ if fx.orb => {
                            // Bees, bats and the phoenix don't stick in walls: they burst.
                            let glow = self.glow_layer();
                            let c = fx.trail.unwrap_or([1.0; 3]);
                            for _ in 0..6 {
                                let r = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32(), self.surv.rng.f32() - 0.5) * 2.0;
                                self.particles.glow(pos, r, glow, c, 0.07, 0.4);
                            }
                            remove = true;
                        }
                        _ => {
                            // Point first into the block, at the angle it flew.
                            let world = &self.world;
                            let tip = crate::archery::lodge_in_block(next, v, |p| Terrain::solid(world, p.floor().as_ivec3()));
                            let s = &mut self.prog.shots[i];
                            s.stuck = true;
                            s.pos = tip;
                            s.vel = v.normalize_or_zero() * 0.001;
                            let id = self.world.block_or_air((tip + v.normalize_or_zero() * 0.05).floor().as_ivec3());
                            let surface = crate::archery::hit_surface(Material::of(&self.reg.get(id).name));
                            self.sfx_at(Sfx::Fx(crate::sfx_extra::Fx::ArrowHit(surface)), tip, 0.9, 24.0);
                            if fx.burst > 0.0 {
                                self.arrow_burst(tip, fx.burst);
                            }
                            if fx.vanish {
                                remove = true;
                            }
                        }
                    }
                } else {
                    let s = &mut self.prog.shots[i];
                    s.pos = next;
                    s.vel = v;
                    let trail = s.fx.trail.or(s.flame.then_some([1.4, 0.6, 0.2]));
                    let orb = s.fx.orb;
                    let age = s.age;
                    if let Some(c) = trail {
                        let glow = self.glow_layer();
                        if orb {
                            // A creature of light: its body is a bright knot of glow, its wings a
                            // flutter of sparks (drawn every frame, no arrow picture).
                            let body = [c[0] * 1.3, c[1] * 1.3, c[2] * 1.3];
                            self.particles.glow(next, Vec3::ZERO, glow, body, 0.13, 0.08);
                            let r = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 1.5;
                            self.particles.glow(next, r, glow, c, 0.05, 0.35);
                        } else if (age * 20.0) as i32 % 2 == 0 {
                            self.particles.glow(next, Vec3::ZERO, glow, c, 0.05, 0.3);
                        }
                    }
                }
            }
            // Picking arrows and tridents back up.
            let s = &self.prog.shots[i];
            if s.stuck && s.lodged.is_none() && (s.pos - (self.body.pos + Vec3::Y)).length() < 1.4 && !remove {
                match s.kind {
                    ShotKind::Arrow { pickup: true, potion } => {
                        let name = match potion {
                            Some(p) => format!("tipped_arrow_{}", p.key),
                            None => "arrow".to_string(),
                        };
                        if let Some(id) = self.items.id(&name) {
                            if self.surv.inv.add(&self.items, ItemStack::new(id, 1)).is_none() {
                                self.sfx(Sfx::Pickup);
                                remove = true;
                            }
                        }
                    }
                    ShotKind::Trident { stack, .. } => {
                        self.give_back_stack(stack);
                        remove = true;
                    }
                    ShotKind::Arrow { pickup: false, .. } if s.age > 1.0 => remove = true,
                    _ => {}
                }
            }
            if remove {
                self.prog.shots.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn give_back_stack(&mut self, stack: ItemStack) {
        if !self.survival {
            return;
        }
        let sel = self.selected;
        if self.surv.inv.slots[sel].is_none() {
            self.surv.inv.slots[sel] = Some(stack);
        } else if let Some(left) = self.surv.inv.add(&self.items, stack) {
            self.throw_stack(left);
        }
        self.sfx(Sfx::Pickup);
    }

    /// A shot reaches mob `k`. True when the shot is used up.
    fn shot_hits_mob(&mut self, i: usize, k: usize) -> bool {
        let eye = self.body.eye();
        let (kind, mut dmg, crit, punch, flame, pierce, channel, imp, vel) = {
            let s = &self.prog.shots[i];
            (s.kind, s.damage, s.crit, s.punch, s.flame, s.pierce, s.channeling, s.impaling, s.vel)
        };
        if crit {
            dmg += self.prog.rng.below((dmg / 2.0 + 2.0) as u32) as f32;
        }
        let name = self.mobs.reg.defs[self.mobs.list[k].def].name.clone();
        if let ShotKind::WindCharge = kind {
            let at = self.mobs.list[k].body.pos;
            self.wind_burst(at, 1.0);
            return true;
        }
        if matches!(kind, ShotKind::Trident { .. }) && (water_mob(&name) || self.mobs.list[k].body.in_water) {
            dmg += 2.5 * imp as f32;
        }
        self.orichalcum_petals(k, dmg);
        if crit {
            self.combat_fx.crit(self.mobs.list[k].net_id);
        }
        let m = &mut self.mobs.list[k];
        m.hurt(dmg, eye);
        if punch > 0 {
            m.body.knockback(vel, 3.0 * punch as f32);
        }
        let fx = self.prog.shots[i].fx;
        if flame {
            m.on_fire = m.on_fire.max(5.0).max(fx.burn);
        }
        if fx.freeze > 0.0 {
            m.frozen = m.frozen.max(fx.freeze);
        }
        let at = m.body.pos;
        if let Some((id, secs)) = fx.debuff {
            self.add_mob_debuff(k, id, secs);
        }
        if fx.burst > 0.0 {
            let p = self.prog.shots[i].pos;
            self.arrow_burst(p, fx.burst);
        }
        if fx.stars > 0 {
            self.call_stars(at, fx.stars, dmg);
        }
        if fx.glow > 0.0 {
            self.add_mob_debuff(k, crate::effects::GLOWING, fx.glow);
        }
        self.last_hit = Some((self.mobs.reg.defs[self.mobs.list[k].def].display_name.clone(), self.mobs.list[k].health.max(0.0), self.mobs.list[k].max_health, self.now + 3.0));
        self.sfx(Sfx::MobHit);
        if matches!(kind, ShotKind::Arrow { .. }) {
            let p = self.prog.shots[i].pos;
            self.sfx_at(Sfx::Fx(crate::sfx_extra::Fx::ArrowHit(3)), p, 0.8, 24.0);
        }
        if let ShotKind::Arrow { potion: Some(p), .. } = kind {
            // Tipped arrows give an eighth of the potion.
            let eff: Vec<(u8, u8, f32)> = p.effects.iter().map(|&(e, a, t)| (e, a, t / 8.0)).collect();
            self.potion_on_mob(k, &name, &eff, 1.0, eye);
        }
        if channel && self.weather.state.thunder {
            self.weather.flash = 1.0;
            self.sfx(Sfx::Thunder(1.0));
            let m = &mut self.mobs.list[k];
            m.hurt(5.0, at + Vec3::Y * 5.0);
            m.on_fire = m.on_fire.max(8.0);
        }
        self.prog.shots[i].hit.push(k);
        match kind {
            ShotKind::Trident { stack, .. } => {
                let s = &mut self.prog.shots[i];
                s.vel = -s.vel * 0.1;
                s.stuck = true;
                s.pos = at + Vec3::Y;
                if s.loyalty > 0 {
                    s.kind = ShotKind::Trident { stack, returning: true };
                }
                false
            }
            ShotKind::Arrow { .. } if pierce == 0 && self.mobs.list[k].alive() && !fx.vanish => {
                // It stays in the monster, point first where it went in.
                let m = &self.mobs.list[k];
                let s = &self.prog.shots[i];
                let d = s.vel.normalize_or_zero();
                let (lo, hi) = m.body.aabb();
                let tip = crate::archery::into_box(s.pos - d * 1.5, d, lo, hi, 0.15).unwrap_or(s.pos);
                let l = crate::archery::Lodged::new(m, tip, d);
                let s = &mut self.prog.shots[i];
                s.lodged = Some(l);
                s.pos = tip;
                s.vel = d * 0.001;
                s.age = 0.0;
                false
            }
            _ => {
                if pierce > 0 {
                    self.prog.shots[i].pierce -= 1;
                    false
                } else {
                    true
                }
            }
        }
    }

    /// A seeking shot at `pos` flying along `v` turns by at most `turn` radians toward the nearest
    /// hostile monster ahead of it within 24 blocks (not one it already struck).
    fn seek(&self, pos: Vec3, v: Vec3, turn: f32, done: &[usize]) -> Vec3 {
        let speed = v.length();
        let d = v / speed.max(1e-4);
        let mut best: Option<(f32, Vec3)> = None;
        for (k, m) in self.mobs.list.iter().enumerate() {
            if !m.alive() || done.contains(&k) || !self.is_hostile(k) {
                continue;
            }
            let to = m.body.center() - pos;
            let dist = to.length();
            if dist > 24.0 || to.dot(d) < dist * 0.2 {
                continue;
            }
            if best.is_none_or(|b| dist < b.0) {
                best = Some((dist, to / dist));
            }
        }
        let Some((_, want)) = best else { return v };
        let a = d.angle_between(want);
        if a < 1e-4 {
            return want * speed;
        }
        let k = (turn / a).min(1.0);
        d.lerp(want, k).normalize_or(d) * speed
    }

    /// A splitting arrow parts into seekers: it keeps one share and sends the others off to the
    /// sides, all of them now turning toward monsters.
    fn split_shot(&mut self, i: usize) {
        let s = self.prog.shots[i].clone();
        let n = s.fx.split.max(1);
        let d = s.vel.normalize_or_zero();
        let side = d.cross(Vec3::Y).normalize_or(Vec3::X);
        let share = s.damage / n as f32;
        {
            let o = &mut self.prog.shots[i];
            o.fx.split = 0;
            o.fx.homing = 4.0;
            o.damage = share;
        }
        let glow = self.glow_layer();
        for k in 1..n {
            let a = if k % 2 == 1 { 0.3 } else { -0.3 } * k.div_ceil(2) as f32;
            let dir = glam::Quat::from_axis_angle(d.cross(side).normalize_or(Vec3::Y), a) * d;
            let mut c = s.clone();
            c.vel = dir * s.vel.length();
            c.fx.split = 0;
            c.fx.homing = 4.0;
            c.damage = share;
            c.kind = ShotKind::Arrow { potion: None, pickup: false };
            self.prog.shots.push(c);
        }
        for _ in 0..6 {
            let r = Vec3::new(self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5, self.surv.rng.f32() - 0.5) * 2.0;
            self.particles.glow(s.pos, r, glow, s.fx.trail.unwrap_or([1.0; 3]), 0.06, 0.3);
        }
    }

    /// A hellfire arrow bursts: monsters near take fire and a blow (no blocks broken).
    fn arrow_burst(&mut self, at: Vec3, radius: f32) {
        let glow = self.glow_layer();
        for k in 0..14 {
            let a = k as f32 * 0.45;
            self.particles.glow(at, Vec3::new(a.cos() * 3.0, 1.5 + (k % 3) as f32, a.sin() * 3.0), glow, [1.5, 0.5, 0.1], 0.12, 0.45);
        }
        self.sfx(Sfx::Explode);
        for m in self.mobs.list.iter_mut() {
            let d = m.body.pos + Vec3::Y * 0.5 - at;
            if m.alive() && d.length() < radius {
                m.hurt(4.0 * (1.0 - d.length() / radius) + 1.0, at);
                m.on_fire = m.on_fire.max(4.0);
            }
        }
    }

    /// A holy arrow calls stars down on where it hit: they fall from high above and pierce.
    fn call_stars(&mut self, at: Vec3, n: u8, dmg: f32) {
        for j in 0..n {
            let off = Vec3::new(self.surv.rng.f32() * 6.0 - 3.0, 14.0 + j as f32 * 2.0, self.surv.rng.f32() * 6.0 - 3.0);
            let from = at + off;
            let vel = (at + Vec3::Y * 0.8 - from).normalize_or_zero() * 40.0;
            let mut s = Shot::new(from, vel, ShotKind::Arrow { potion: None, pickup: false }, (dmg * 0.5).max(2.0));
            s.pierce = 2;
            s.fx = crate::archery::ArrowFx { straight: true, trail: Some([1.4, 1.3, 0.8]), ..Default::default() };
            self.prog.shots.push(s);
        }
    }

    /// A wind charge bursts: everything near is thrown away from it (the player upward too).
    pub(crate) fn wind_burst(&mut self, at: Vec3, power: f32) {
        self.sfx(Sfx::Cast(5));
        let glow = self.glow_layer();
        for k in 0..16 {
            let a = k as f32 * 0.39;
            self.particles.glow(at, Vec3::new(a.cos() * 4.0, 1.0, a.sin() * 4.0), glow, [0.85, 0.9, 1.2], 0.12, 0.5);
        }
        for m in self.mobs.list.iter_mut() {
            let d = m.body.pos - at;
            if m.alive() && d.length() < 3.0 {
                m.body.knockback(d, 6.0 * power);
                m.body.vel.y += 4.0 * power;
            }
        }
        let d = self.body.pos + Vec3::Y * 0.5 - at;
        if d.length() < 3.0 {
            let away = d.normalize_or_zero();
            self.body.vel += away * 8.0 * power + Vec3::Y * 9.0 * power;
            // A wind charge jump doesn't count the height it started from as a fall.
            self.fall_peak = self.body.pos.y;
        }
        // Light blocks get pushed: doors, trapdoors, levers flip (the game's toggle code).
    }

    /// The player's arrows, tridents and wind charges for drawing.
    pub(crate) fn shot_instances(&self, cam_block: IVec3) -> Vec<GpuItem> {
        let sprite = |name: &str| {
            self.items.id(name).and_then(|i| match self.items.get(i).look {
                voxel::item::Look::Sprite(l) => Some((self.item_base + l) as f32),
                _ => None,
            })
        };
        let c = cam_block.as_vec3();
        let mut out = Vec::new();
        for s in &self.prog.shots {
            let (layer, size) = match s.kind {
                ShotKind::Arrow { .. } if s.item.is_some() => (s.item.and_then(|i| match self.items.get(i).look {
                    voxel::item::Look::Sprite(l) => Some((self.item_base + l) as f32),
                    _ => None,
                }).or(sprite("arrow")), 0.5),
                ShotKind::Arrow { potion: Some(p), .. } => (sprite(&format!("tipped_arrow_{}", p.key)).or(sprite("arrow")), 0.5),
                ShotKind::Arrow { .. } => (sprite("arrow"), 0.5),
                ShotKind::Trident { .. } => (sprite("trident"), 1.1),
                ShotKind::WindCharge => (sprite("wind_charge"), 0.45),
            };
            let Some(layer) = layer else { continue };
            if s.fx.orb {
                // Bees, bats, the phoenix: their glow is their body (step_weapon_shots).
                continue;
            }
            // Lit by the air at its feathers, not the inside of the block its point is in.
            let l = self.world.light((s.pos - s.vel.normalize_or_zero() * 0.4).floor().as_ivec3());
            let (sky, blk) = ((l >> 4) as f32 / 15.0, (l & 15) as f32 / 15.0);
            let p = s.pos - c;
            if matches!(s.kind, ShotKind::WindCharge) {
                out.push(GpuItem { pos: [p.x, p.y, p.z, size], info: [s.age * 8.0, 0.0, sky, blk], layers: [layer, layer, layer, 0.0], rot: [0.0; 4], ..bytemuck::Zeroable::zeroed() });
                continue;
            }
            // Its point is where it is (in the block or monster once it lands).
            let len = if matches!(s.kind, ShotKind::Trident { .. }) { 1.3 } else { 0.6 };
            out.push(self.world_arrow(layer, p, s.vel, len, l));
        }
        out
    }
}

/// Test hook: MC_BOW_PULL=0..1 holds a bow drawn that far (for screenshots).
fn test_bow_pull() -> Option<f32> {
    static PULL: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *PULL.get_or_init(|| std::env::var("MC_BOW_PULL").ok().and_then(|v| v.parse().ok()))
}
