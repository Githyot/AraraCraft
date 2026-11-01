//! Tame animals and mounts. A saddle on a horse, donkey, llama or camel lets you ride it:
//! right click to climb on, steer with the movement keys, jump with space and sneak to get
//! off. A bone may tame a wolf and raw fish may tame a cat; tamed ones follow you around,
//! fight monsters (wolves) or chase small animals (cats), and are saved with the world.

use std::path::Path;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use voxel::physics::Input;

use crate::Game;

const MOUNTS: &[&str] = &["horse", "donkey", "llama", "camel", "pig", "mule", "skeleton_horse", "zombie_horse", "camel_husk", "happy_ghast", "nautilus"];
/// (wild, tamed, the food that tames it, chance per try)
/// Horses that must be broken in before they take a saddle (Java): ride them bare-handed until
/// they stop bucking, food makes them calmer.
pub(crate) const BREAK_IN: [&str; 5] = ["horse", "donkey", "mule", "llama", "zombie_horse"];
/// Animals won over without changing what they are: (mob, food, chance). A parrot then follows
/// the player, an ocelot stops running from them.
pub(crate) const TRUSTS: [(&str, &[&str], f32); 3] = [
    ("parrot", &["wheat_seeds", "beetroot_seeds", "melon_seeds", "pumpkin_seeds"], 0.2),
    ("ocelot", &["cod", "salmon"], 0.33),
    // A nautilus takes fish, then a saddle, and carries you under water.
    ("nautilus", &["cod", "salmon", "pufferfish", "tropical_fish"], 0.33),
];

/// How much calmer a wild horse gets from a food (Java temper values).
pub(crate) fn temper_of(food: &str) -> u8 {
    match food {
        "wheat" | "apple" | "sugar" | "sweet_berries" => 3,
        "golden_carrot" => 5,
        "golden_apple" => 10,
        "hay_block" => 20,
        _ => 0,
    }
}

const TAMES: [(&str, &str, &[&str], f32); 2] = [("wolf", "wolf_tame", &["bone"], 0.33), ("cat", "cat_tame", &["cod", "salmon", "raw_cod", "raw_salmon"], 0.33)];

#[derive(Clone, Serialize, Deserialize)]
pub struct PetSave {
    pub name: String,
    pub pos: [f32; 3],
    pub saddled: bool,
    /// Which way it faces, and what an armor stand wears (stands.rs).
    #[serde(default)]
    pub yaw: f32,
    #[serde(default)]
    pub gear: Vec<Option<voxel::item::ItemStack>>,
    /// Farm animals and named ones (`Mob::keep`): coat, name tag, health, a baby's time left.
    #[serde(default)]
    pub variant: u8,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub health: Option<f32>,
    #[serde(default)]
    pub grow: f32,
    #[serde(default)]
    pub keep: bool,
    /// Won over (`Mob::tame`) and a wild horse's patience (`Mob::temper`).
    #[serde(default)]
    pub tame: bool,
    #[serde(default)]
    pub temper: u8,
}

impl PetSave {
    fn of(g: &Game, m: &voxel::mob::Mob, gear: Vec<Option<voxel::item::ItemStack>>) -> PetSave {
        PetSave {
            name: g.mobs.reg.defs[m.def].name.clone(),
            pos: m.body.pos.to_array(),
            saddled: m.saddled,
            yaw: m.yaw,
            gear,
            variant: m.variant,
            tag: m.name.clone(),
            health: Some(m.health),
            grow: m.grow,
            keep: m.keep,
            tame: m.tame,
            temper: m.temper,
        }
    }
}

pub fn load(dir: &Path) -> Vec<PetSave> {
    std::fs::read_to_string(dir.join("pets.ron")).ok().and_then(|t| ron::from_str(&t).ok()).unwrap_or_default()
}

impl Game {
    fn is_pet(&self, i: usize) -> bool {
        let m = &self.mobs.list[i];
        let n = self.mobs.reg.defs[m.def].name.as_str();
        // Animals the player made theirs (bred, fed, sheared, led) and named ones stay too.
        let kept = (m.keep || m.tame || m.name.is_some()) && !m.still && !m.ghost && !self.mobs.reg.defs[m.def].boss;
        n.ends_with("_tame") || m.saddled || kept || matches!(n, "armor_stand" | "snow_golem" | "allay")
    }

    pub(crate) fn save_pets(&self) {
        let list: Vec<PetSave> = (0..self.mobs.list.len())
            .filter(|&i| self.mobs.list[i].alive() && self.is_pet(i))
            .map(|i| {
                let m = &self.mobs.list[i];
                let gear = self.stands.gear.get(&m.net_id).map(|g| g.0.to_vec()).unwrap_or_default();
                PetSave::of(self, m, gear)
            })
            .chain(self.pets_waiting.iter().cloned())
            .collect();
        if let Ok(t) = ron::ser::to_string_pretty(&list, ron::ser::PrettyConfig::default()) {
            let _ = voxel::save::write_atomic(self.world.save.dir.join("pets.ron"), t);
        }
    }

    /// Right click on an animal: saddle it, ride it, or feed it to tame it.
    pub(crate) fn pet_use(&mut self) -> bool {
        let Some((i, t)) = self.mobs.pick(self.body.eye(), self.forward(), 4.0) else { return false };
        if self.target.is_some_and(|h| h.distance < t && self.reg.get(self.world.block_or_air(h.block)).solid) {
            return false;
        }
        let name = self.mobs.reg.defs[self.mobs.list[i].def].name.clone();
        let held = self.held_stack().map(|s| self.items.get(s.item).name.clone()).unwrap_or_default();
        if MOUNTS.contains(&name.as_str()) {
            if !self.mobs.list[i].saddled {
                if held != "saddle" {
                    self.toast("Use uma sela para poder montar", 2.5);
                    return true;
                }
                if name == "nautilus" && !self.mobs.list[i].tame {
                    self.toast("Dome primeiro: dê peixe ao náutilo", 3.0);
                    return true;
                }
                if BREAK_IN.contains(&name.as_str()) && !self.mobs.list[i].tame {
                    self.toast("Dome primeiro: monte nele de mão vazia até ele parar de pular", 3.5);
                    return true;
                }
                self.mobs.list[i].saddled = true;
                if self.survival {
                    let sel = self.selected;
                    self.surv.inv.take(sel, 1);
                }
                self.toast("Selado! Clique com o direito para montar", 3.0);
                return true;
            }
            if self.crouch > 0.5 || self.mobs.list.iter().any(|m| m.ridden) {
                return false;
            }
            self.mobs.list[i].ridden = true;
            self.toast("Agache (Shift) para descer", 2.5);
            return true;
        }
        if let Some(&(_, food, chance)) = TRUSTS.iter().find(|t| t.0 == name) {
            if !food.contains(&held.as_str()) || self.mobs.list[i].tame {
                return false;
            }
            if self.survival {
                let sel = self.selected;
                self.surv.inv.take(sel, 1);
            }
            let glow = self.glow_layer();
            let at = self.mobs.list[i].body.pos + Vec3::Y;
            if self.surv.rng.f32() < chance {
                let m = &mut self.mobs.list[i];
                m.tame = true;
                m.keep = true;
                for k in 0..10 {
                    let a = k as f32 * 0.63;
                    self.particles.glow(at + Vec3::new(a.cos() * 0.4, 0.2, a.sin() * 0.4), Vec3::Y * 0.8, glow, [1.0, 0.35, 0.45], 0.06, 1.2);
                }
                self.toast(
                    match name.as_str() {
                        "parrot" => "Domado! O papagaio agora te segue",
                        "nautilus" => "Domado! Coloque uma sela no náutilo para nadar com ele",
                        _ => "A jaguatirica confia em você e não foge mais",
                    },
                    3.0,
                );
                self.prog.event(crate::progression::Event::Token("tamed".into()));
            } else {
                self.particles.glow(at, Vec3::Y * 0.6, glow, [0.5, 0.5, 0.5], 0.05, 1.0);
            }
            return true;
        }
        if let Some(&(_, tame, food, chance)) = TAMES.iter().find(|t| t.0 == name) {
            if !food.contains(&held.as_str()) {
                return false;
            }
            if self.survival {
                let sel = self.selected;
                self.surv.inv.take(sel, 1);
            }
            let glow = self.glow_layer();
            let at = self.mobs.list[i].body.pos + Vec3::Y;
            if self.surv.rng.f32() < chance {
                if let Some(d) = self.mobs.reg.id(tame) {
                    let m = &mut self.mobs.list[i];
                    m.def = d;
                    m.health = self.mobs.reg.defs[d].health;
                    m.max_health = m.health;
                }
                for k in 0..10 {
                    let a = k as f32 * 0.63;
                    self.particles.glow(at + Vec3::new(a.cos() * 0.4, 0.2, a.sin() * 0.4), Vec3::Y * 0.8, glow, [1.0, 0.35, 0.45], 0.06, 1.2);
                }
                self.toast("Domado! Agora ele te segue", 3.0);
                self.prog.event(crate::progression::Event::Token("tamed".into()));
                self.count_tamed_coat(&name, i);
            } else {
                self.particles.glow(at, Vec3::Y * 0.6, glow, [0.5, 0.5, 0.5], 0.05, 1.0);
            }
            return true;
        }
        false
    }

    /// Bare-handed on a wild horse: it takes the rider for a moment, then either calms down
    /// (tame, hearts) or bucks them off, a little more patient next time (Java).
    pub(crate) fn break_in(&mut self, i: usize) {
        let glow = self.glow_layer();
        let m = &mut self.mobs.list[i];
        let at = m.body.pos + Vec3::Y * 1.4;
        let roll = (self.surv.rng.f32() * 100.0) as u8;
        if roll < m.temper {
            m.tame = true;
            m.keep = true;
            for k in 0..10 {
                let a = k as f32 * 0.63;
                self.particles.glow(at + Vec3::new(a.cos() * 0.5, 0.2, a.sin() * 0.5), Vec3::Y * 0.8, glow, [1.0, 0.35, 0.45], 0.06, 1.2);
            }
            self.toast("Domado! Agora coloque uma sela para montar", 3.0);
            self.prog.event(crate::progression::Event::Token("tamed".into()));
        } else {
            m.temper = (m.temper + 5).min(100);
            let me = self.body.pos;
            m.startle(me);
            // Rearing: the rider is thrown back and up.
            let f = self.forward();
            self.body.vel = Vec3::new(-f.x * 6.0, 7.0, -f.z * 6.0);
            self.toast("Ele te derrubou! Tente de novo, ou dê comida para acalmá-lo", 3.0);
        }
        let name = self.mobs.reg.defs[self.mobs.list[i].def].name.clone();
        if let Some(fx) = crate::mob_sounds::pick(&name, crate::sfx_extra::Call::Idle, 1.0, self.surv.rng.f32(), self.surv.rng.f32()) {
            self.sfx_at(crate::audio::Sfx::Fx(fx), at, 0.8, 16.0);
        }
        self.anim.start_swing();
    }

    /// Every coat tamed at least once: "Um Catálogo Completo de Gatos" and "A Matilha Completa".
    fn count_tamed_coat(&mut self, wild: &str, i: usize) {
        let m = &self.mobs.list[i];
        let coats = self.mobs.reg.defs[m.def].variants.len();
        if coats < 2 {
            return;
        }
        let v = m.variant;
        self.prog.stats.add(&format!("tamed_coat:{wild}:{v}"), 1);
        let all = (0..coats).all(|k| self.prog.stats.get(&format!("tamed_coat:{wild}:{k}")) > 0);
        if all {
            let token = if wild == "cat" { "cats_all" } else { "wolves_all" };
            self.prog.event(crate::progression::Event::Token(token.into()));
        }
    }

    /// The player on a mount: it runs where they steer. True while riding.
    pub(crate) fn mount_step(&mut self, wish: Vec3, jump: bool, sneak: bool, dt: f32) -> bool {
        let Some(i) = self.mobs.list.iter().position(|m| m.ridden) else { return false };
        // A pig only goes where a carrot on a stick points it.
        let wish = self.pig_steer(wish);
        let alive = self.mobs.list[i].alive();
        let f = self.forward();
        let m = &mut self.mobs.list[i];
        if sneak || !alive {
            m.ridden = false;
            let side = Vec3::new(m.yaw.cos(), 0.0, -m.yaw.sin());
            self.body.pos = m.body.pos + side * 1.2 + Vec3::Y * 0.2;
            self.body.vel = Vec3::ZERO;
            return false;
        }
        let def = &self.mobs.reg.defs[m.def];
        m.yaw = f.x.atan2(f.z);
        m.body.speed_override = Some(def.speed * 2.0);
        let before = m.body.pos;
        // A happy ghast flies: jump climbs, looking down sinks.
        m.body.flying = def.flies;
        let sink = def.flies && f.y < -0.45;
        let swims = def.aquatic && m.body.in_water;
        m.body.step(&self.world, &Input { wish, jump, sneak: sink, sprint: swims, crawl: false }, dt);
        // A nautilus dives where the rider looks.
        if swims && wish.length_squared() > 0.01 {
            m.body.vel.y += (f.y * 4.0 - m.body.vel.y) * (dt * 4.0).min(1.0);
        }
        let speed = ((m.body.pos - before) * Vec3::new(1.0, 0.0, 1.0)).length() / dt.max(1e-4);
        m.walk_speed += (speed - m.walk_speed) * (dt * 10.0).min(1.0);
        m.walk += m.walk_speed * dt * 2.2;
        let breathe = def.aquatic;
        self.body.pos = m.body.pos + Vec3::Y * (def.height * def.scale.max(0.5) * 0.55);
        self.body.vel = m.body.vel;
        self.body.on_ground = true;
        self.body.fall_distance = 0.0;
        // Riding a nautilus: its breath keeps the rider's (Java's Breath of the Nautilus).
        if breathe {
            self.add_effect(crate::effects::WATER_BREATHING, 0, 1.0, true);
        }
        true
    }

    /// Tame animals follow the player; saved ones come back when the player is near.
    pub(crate) fn update_pets(&mut self, dt: f32) {
        if dt == 0.0 {
            return;
        }
        let me = self.body.pos;
        // Pets left far behind wait in the save instead of vanishing with the wild ones.
        let mut k = 0;
        while k < self.mobs.list.len() {
            let m = &self.mobs.list[k];
            if m.alive() && !m.ridden && m.body.pos.distance(me) > 120.0 && self.is_pet(k) {
                let m = self.mobs.list.remove(k);
                let gear = self.stands.gear.remove(&m.net_id).map(|g| g.0.to_vec()).unwrap_or_default();
                let save = PetSave::of(self, &m, gear);
                self.pets_waiting.push(save);
            } else {
                k += 1;
            }
        }
        let mut k = 0;
        while k < self.pets_waiting.len() {
            let p = Vec3::from(self.pets_waiting[k].pos);
            if p.distance(me) < 64.0 && self.world.block(p.floor().as_ivec3()).is_some() {
                let pet = self.pets_waiting.remove(k);
                if self.mobs.spawn(&pet.name, p) {
                    if let Some(m) = self.mobs.list.last_mut() {
                        m.saddled = pet.saddled;
                        m.yaw = pet.yaw;
                        m.keep = pet.keep;
                        // (Saddled before taming existed: tame.)
                        m.tame = pet.tame || pet.saddled;
                        m.temper = pet.temper;
                        m.name = pet.tag.clone();
                        if pet.variant != voxel::mob::VARIANT_UNSET {
                            m.variant = pet.variant;
                        }
                        if let Some(h) = pet.health {
                            m.health = h.clamp(1.0, m.max_health);
                        }
                        if pet.grow > 0.0 {
                            let def = &self.mobs.reg.defs[m.def];
                            let parent = m.parent;
                            m.make_baby(def, parent);
                            m.grow = pet.grow;
                        }
                        if pet.gear.iter().any(|g| g.is_some()) {
                            let mut gear: crate::stands::Gear = Default::default();
                            for (i, g) in pet.gear.iter().take(5).enumerate() {
                                gear[i] = *g;
                            }
                            let (id, pos) = (m.net_id, m.body.pos);
                            self.stands.gear.insert(id, (gear, pos));
                        }
                    }
                }
            } else {
                k += 1;
            }
        }
        for m in self.mobs.list.iter_mut() {
            let n = self.mobs.reg.defs[m.def].name.as_str();
            if !m.alive() || m.ridden || !(n.ends_with("_tame") || (m.tame && n == "parrot")) {
                continue;
            }
            let d = m.body.pos.distance(me);
            if d > 28.0 {
                m.body.pos = me - self.body.vel.normalize_or_zero() * 2.0 + Vec3::Y * 0.5;
            } else if d > 6.0 && m.prey.is_none() {
                m.go_to(me, 0.5);
            }
        }
    }
}

#[cfg(test)]
mod taming_tests {
    use super::*;

    #[test]
    fn every_tamable_animal_exists() {
        let reg = crate::mobs::registry_for_tests();
        for n in BREAK_IN.iter().chain(TRUSTS.iter().map(|t| &t.0)).chain(MOUNTS.iter()) {
            assert!(reg.id(n).is_some(), "{n} is not a mob");
        }
        assert!(temper_of("hay_block") > temper_of("wheat"));
        assert_eq!(temper_of("dirt"), 0);
    }
}
