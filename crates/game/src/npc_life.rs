//! A day in the life of the town folk (plan/npcs-analise.md, lote N1).
//!
//! The mob AI (voxel/mob.rs) only lets a town NPC potter about a few blocks from home. On top
//! of it, four times a second, this file gives each townsperson near a player a routine:
//! - by day they go out: to the middle of town, to a neighbour's door, to the square of their
//!   village, linger there a while and come back;
//! - at dusk, in the rain, in a blood moon or when a monster comes near they head home;
//! - at night, with a free bed in their room, they lie down in it until the morning.
//!
//! Getting there takes a real path: a small A* over the blocks (two free cells to stand in,
//! ground under them, one block up or two down a step, doors pass), so folk walk round walls
//! and in through the door instead of pushing against the house. They open wooden doors on
//! the way and close them behind themselves. Someone stuck, or far from every player, is
//! simply put home at night, like Terraria's folk.

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use glam::{IVec3, Vec3};
use voxel::block::Shape;
use voxel::mob::{Act, Kind};
use voxel::{bed, AIR};

use crate::Game;

const PASS: f32 = 0.25;
/// Only folk this close to a player follow the routine (the rest stand at home).
const NEAR: f32 = 72.0;
/// How far a monster has to be for folk to stay out.
const DANGER: f32 = 12.0;
/// Most cells one path search may look at.
const MAX_NODES: usize = 3500;

/// Mobs that are town folk for the routine: the moving-in NPCs, the village settlers and the
/// goblin trader.
pub(crate) fn is_folk(name: &str) -> bool {
    name.starts_with("npc_") || name.starts_with("settler_") || name == "goblin_trader"
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Goal {
    Home(Vec3),
    Bed(IVec3),
    Out(Vec3),
}

struct Plan {
    goal: Goal,
    path: Vec<IVec3>,
    k: usize,
    /// Seconds without getting closer to the next step.
    stuck: f32,
    best: f32,
    fails: u8,
}

#[derive(Default)]
struct Folk {
    plan: Option<Plan>,
    /// Out of the house until this clock time (by day), then home for a while until `next_out`.
    out_until: f32,
    next_out: f32,
    out_at: Option<Vec3>,
}

#[derive(Default)]
pub struct NpcLife {
    clock: f32,
    pass_t: f32,
    folk: HashMap<u32, Folk>,
    /// Doors folk opened: (lower half, seconds before it may close again).
    doors: Vec<(IVec3, f32)>,
}

/// One step of a path: a cell to stand in.
#[derive(PartialEq, Eq)]
struct Node {
    f: u32,
    p: IVec3,
}

impl Ord for Node {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        o.f.cmp(&self.f).then_with(|| (self.p.x, self.p.y, self.p.z).cmp(&(o.p.x, o.p.y, o.p.z)))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

impl Game {
    /// A cell a body can be in: air, plants, torches, carpets, any door but an iron one.
    fn npc_free(&self, p: IVec3) -> bool {
        let id = self.world.block_or_air(p);
        if id == AIR {
            return true;
        }
        let b = self.reg.get(id);
        if b.shape == Shape::Door {
            return !self.reg.get(b.base).name.starts_with("iron_");
        }
        !b.solid && !self.world.is_liquid(id) && !matches!(b.shape, Shape::Bed | Shape::Fence | Shape::FenceGate)
    }

    /// Ground to stand on.
    fn npc_floor(&self, p: IVec3) -> bool {
        let id = self.world.block_or_air(p);
        id != AIR && (self.reg.get(id).solid || self.reg.get(id).shape == Shape::Bed)
    }

    /// A body can stand at `p`: two free cells over firm ground.
    pub(crate) fn npc_stand(&self, p: IVec3) -> bool {
        self.npc_free(p) && self.npc_free(p + IVec3::Y) && self.npc_floor(p - IVec3::Y)
    }

    /// The cell a body at `pos` stands in (or the nearest standing cell around it).
    fn npc_cell(&self, pos: Vec3) -> Option<IVec3> {
        let c = (pos + Vec3::Y * 0.2).floor().as_ivec3();
        [IVec3::ZERO, -IVec3::Y, IVec3::Y, IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z].into_iter().map(|d| c + d).find(|&p| self.npc_stand(p))
    }

    /// A* from `from` to any cell of `goals` (within a box round `from`); the cells to walk
    /// through, `from` excluded. None when there is no way.
    pub(crate) fn npc_path(&self, from: IVec3, goals: &[IVec3]) -> Option<Vec<IVec3>> {
        if goals.is_empty() {
            return None;
        }
        if goals.contains(&from) {
            return Some(Vec::new());
        }
        let h = |p: IVec3| goals.iter().map(|g| ((g.x - p.x).abs() + (g.z - p.z).abs() + (g.y - p.y).abs()) as u32 * 10).min().unwrap_or(0);
        let mut open = BinaryHeap::new();
        let mut came: HashMap<IVec3, IVec3> = HashMap::new();
        let mut cost: HashMap<IVec3, u32> = HashMap::new();
        cost.insert(from, 0);
        open.push(Node { f: h(from), p: from });
        let mut seen = 0;
        while let Some(Node { p, .. }) = open.pop() {
            if goals.contains(&p) {
                let mut path = vec![p];
                let mut c = p;
                while let Some(&b) = came.get(&c) {
                    if b == from {
                        break;
                    }
                    path.push(b);
                    c = b;
                }
                path.reverse();
                return Some(path);
            }
            seen += 1;
            if seen > MAX_NODES {
                return None;
            }
            let g = cost[&p];
            for d in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
                let q = p + d;
                if (q - from).abs().max_element() > 40 {
                    continue;
                }
                // Level, a step up (with room over the head to jump), or down one or two.
                let next = if self.npc_stand(q) {
                    Some((q, 10))
                } else if self.npc_stand(q + IVec3::Y) && self.npc_free(p + IVec3::Y * 2) {
                    Some((q + IVec3::Y, 16))
                } else if self.npc_free(q) && self.npc_free(q + IVec3::Y) {
                    if self.npc_stand(q - IVec3::Y) {
                        Some((q - IVec3::Y, 12))
                    } else if self.npc_free(q - IVec3::Y) && self.npc_stand(q - IVec3::Y * 2) {
                        Some((q - IVec3::Y * 2, 16))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let Some((n, c)) = next else { continue };
                // Doors cost a little more, so folk use them only when they lead somewhere.
                let c = c + if self.reg.get(self.world.block_or_air(n)).shape == Shape::Door { 4 } else { 0 };
                let ng = g + c;
                if cost.get(&n).is_none_or(|&old| ng < old) {
                    cost.insert(n, ng);
                    came.insert(n, p);
                    open.push(Node { f: ng + h(n), p: n });
                }
            }
        }
        None
    }

    /// A free bed in the room round `home`: (head cell, facing). Beds others lie in are taken.
    fn npc_bed(&self, home: Vec3, me: usize) -> Option<(IVec3, u8)> {
        let start = home.floor().as_ivec3();
        let mut seen = HashSet::from([start]);
        let mut queue = VecDeque::from([start]);
        let mut beds = Vec::new();
        while let Some(p) = queue.pop_front() {
            if seen.len() > 500 || (p - start).abs().max_element() > 10 {
                break;
            }
            for d in [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y, IVec3::Z, IVec3::NEG_Z] {
                let q = p + d;
                let b = self.reg.get(self.world.block_or_air(q));
                if b.shape == Shape::Bed {
                    if let Some(h) = self.bed_head(q) {
                        if !beds.contains(&h) {
                            beds.push(h);
                        }
                    }
                    continue;
                }
                // The room only: doors and walls stop the search.
                if b.shape != Shape::Door && self.npc_free(q) && seen.insert(q) {
                    queue.push_back(q);
                }
            }
        }
        beds.into_iter().find(|(h, _)| {
            !self.mobs.list.iter().enumerate().any(|(j, m)| j != me && m.alive() && m.lying && m.body.pos.distance(h.as_vec3() + Vec3::splat(0.5)) < 2.2)
        })
    }

    /// Cells to walk to for a goal.
    fn npc_goal_cells(&self, goal: Goal) -> Vec<IVec3> {
        let around = |c: IVec3| -> Vec<IVec3> {
            let mut v: Vec<IVec3> = [IVec3::ZERO, IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z].iter().flat_map(|&d| [c + d, c + d - IVec3::Y, c + d + IVec3::Y]).filter(|&p| self.npc_stand(p)).collect();
            v.dedup();
            v
        };
        match goal {
            Goal::Home(h) => {
                let v = around(h.floor().as_ivec3());
                if v.is_empty() {
                    self.npc_cell(h).into_iter().collect()
                } else {
                    v
                }
            }
            Goal::Out(p) => around(p.floor().as_ivec3()),
            Goal::Bed(head) => {
                // Beside either half of the bed.
                let Some((h, f)) = self.bed_head(head) else { return Vec::new() };
                let [x, z] = bed::dir(f);
                let foot = h - IVec3::new(x, 0, z);
                let mut v = Vec::new();
                for c in [h, foot] {
                    for d in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
                        let p = c + d;
                        if self.npc_stand(p) && !v.contains(&p) {
                            v.push(p);
                        }
                    }
                }
                v
            }
        }
    }

    /// Somewhere outdoors to spend some of the day: near the middle of town (or the village
    /// square), on open ground under the sky.
    fn npc_outing(&mut self, centre: Vec3, from: Vec3) -> Option<Vec3> {
        for _ in 0..14 {
            let a = self.surv.rng.f32() * std::f32::consts::TAU;
            let r = 2.0 + self.surv.rng.f32() * 9.0;
            let x = (centre.x + a.cos() * r).floor() as i32;
            let z = (centre.z + a.sin() * r).floor() as i32;
            let y0 = centre.y.floor() as i32;
            for dy in [0, 1, -1, 2, -2, 3, -3, 4, -4] {
                let p = IVec3::new(x, y0 + dy, z);
                if !self.npc_stand(p) {
                    continue;
                }
                // Under the open sky (not in someone's house or under a roof).
                let sky = (2..20).all(|k| {
                    let id = self.world.block_or_air(p + IVec3::Y * k);
                    id == AIR || !self.reg.get(id).solid
                });
                if sky && p.as_vec3().distance(from) > 2.0 {
                    return Some(p.as_vec3() + Vec3::new(0.5, 0.0, 0.5));
                }
            }
        }
        None
    }

    pub(crate) fn update_npc_life(&mut self, dt: f32) {
        if dt == 0.0 {
            return;
        }
        self.npc_life.clock += dt;
        self.close_npc_doors(dt);
        self.npc_life.pass_t -= dt;
        if self.npc_life.pass_t > 0.0 {
            return;
        }
        self.npc_life.pass_t = PASS;
        let step = PASS;
        let now = self.npc_life.clock;
        let night = self.is_night();
        let late = (13000.0..23000.0).contains(&self.time_of_day);
        let rain = self.weather.state.raining;
        let blood = self.t5.hard.blood_moon();
        let player = self.body.pos;
        // The middle of the player's town.
        let homes: Vec<Vec3> = self.town.npcs.iter().map(|n| Vec3::from(n.home)).filter(|h| h.distance(player) < 120.0).collect();
        let town_mid = (!homes.is_empty()).then(|| homes.iter().copied().sum::<Vec3>() / homes.len() as f32);
        let hostiles: Vec<Vec3> = self.mobs.list.iter().filter(|m| m.alive() && !m.ghost && self.mobs.reg.defs[m.def].kind == Kind::Hostile && !self.mobs.reg.defs[m.def].town).map(|m| m.body.pos).collect();
        let mut path_budget = 2;
        let ids: Vec<u32> = self.mobs.list.iter().map(|m| m.net_id).collect();
        self.npc_life.folk.retain(|id, _| ids.contains(id));
        for i in 0..self.mobs.list.len() {
            let m = &self.mobs.list[i];
            let name = &self.mobs.reg.defs[m.def].name;
            if !m.alive() || !is_folk(name) || m.ridden || m.leashed || m.frozen > 0.0 || m.ghost {
                continue;
            }
            let Some(home) = m.home else { continue };
            let pos = m.body.pos;
            let far = pos.distance(player) > NEAR;
            let id = m.net_id;
            let settler = name.starts_with("settler_");
            let danger = hostiles.iter().any(|h| h.distance(pos) < DANGER);
            let stay_in = night || rain || blood || danger;

            // Morning: up and out of bed.
            if !night && self.mobs.list[i].lying {
                let m = &mut self.mobs.list[i];
                m.sleeping = false;
                m.lying = false;
                m.body.pos.y += 0.1;
            }
            if self.mobs.list[i].lying {
                continue;
            }

            // What does this one want now?
            let f = self.npc_life.folk.entry(id).or_insert_with(|| Folk { next_out: now + 5.0 + (id % 17) as f32, ..Default::default() });
            let goal = if stay_in {
                f.out_at = None;
                Goal::Home(home)
            } else if let Some(at) = f.out_at.filter(|_| now < f.out_until) {
                Goal::Out(at)
            } else {
                if f.out_at.is_some() {
                    // Back home for a while.
                    f.out_at = None;
                    f.next_out = now + 25.0 + (id % 23) as f32;
                }
                Goal::Home(home)
            };
            let wants_out = !stay_in && f.out_at.is_none() && now >= f.next_out;
            let goal = if wants_out {
                let centre = if settler { home } else { town_mid.unwrap_or(home) };
                match self.npc_outing(centre, pos) {
                    Some(at) => {
                        let f = self.npc_life.folk.get_mut(&id).unwrap();
                        f.out_at = Some(at);
                        f.out_until = now + 40.0 + (id % 31) as f32;
                        Goal::Out(at)
                    }
                    None => {
                        self.npc_life.folk.get_mut(&id).unwrap().next_out = now + 30.0;
                        goal
                    }
                }
            } else {
                goal
            };
            // At night, a bed in the room.
            let goal = if matches!(goal, Goal::Home(_)) && late {
                self.npc_bed(home, i).map(|(h, _)| Goal::Bed(h)).unwrap_or(goal)
            } else {
                goal
            };
            let target = goal_pos(goal);

            // Already there? Standing in one of the goal's cells (a distance alone would count
            // the far side of a wall).
            let here = self.npc_cell(pos);
            let there = here.is_some_and(|c| self.npc_goal_cells(goal).iter().any(|g| *g == c || (g.y - c.y).abs() <= 1 && Vec3::new(g.x as f32 + 0.5 - pos.x, 0.0, g.z as f32 + 0.5 - pos.z).length() < 0.6)) || Vec3::new(target.x - pos.x, 0.0, target.z - pos.z).length() < 0.7 && (target.y - pos.y).abs() < 1.0;
            if there {
                let busy_chat = self.mobs.list[i].social.act != Act::None;
                let f = self.npc_life.folk.get_mut(&id).unwrap();
                f.plan = None;
                let m = &mut self.mobs.list[i];
                m.anchor = match goal {
                    Goal::Out(at) => Some(at),
                    _ => None,
                };
                if let Goal::Bed(h) = goal {
                    if late && !busy_chat && player.distance(pos) > 3.5 {
                        self.npc_lie_down(i, h);
                    }
                }
                continue;
            }
            // Out of sight at night and no way home: Terraria folk just turn up at home.
            if far {
                if stay_in && pos.distance(home) > 3.0 {
                    let m = &mut self.mobs.list[i];
                    m.body.pos = home;
                    m.body.vel = Vec3::ZERO;
                    m.anchor = None;
                }
                continue;
            }
            // Chats stop when it's time to go in.
            if self.mobs.list[i].social.act != Act::None {
                if !stay_in {
                    continue;
                }
                self.mobs.list[i].social.act = Act::None;
            }
            let m = &mut self.mobs.list[i];
            m.anchor = match goal {
                Goal::Out(at) => Some(at),
                _ => None,
            };
            // Follow (or work out) the way there.
            let need_new = self.npc_life.folk[&id].plan.as_ref().is_none_or(|p| p.goal != goal || p.k >= p.path.len());
            if need_new {
                if path_budget == 0 {
                    continue;
                }
                path_budget -= 1;
                let fails = self.npc_life.folk[&id].plan.as_ref().map(|p| if p.goal == goal { p.fails } else { 0 }).unwrap_or(0);
                let path = self.npc_cell(pos).and_then(|from| self.npc_path(from, &self.npc_goal_cells(goal)));
                let f = self.npc_life.folk.get_mut(&id).unwrap();
                match path {
                    Some(path) if !path.is_empty() => f.plan = Some(Plan { goal, path, k: 0, stuck: 0.0, best: f32::MAX, fails }),
                    _ => {
                        // No way: give up going out; at night, slip home when nobody watches.
                        f.plan = Some(Plan { goal, path: Vec::new(), k: 0, stuck: 0.0, best: f32::MAX, fails: fails + 1 });
                        if matches!(goal, Goal::Out(_)) {
                            f.out_at = None;
                            f.next_out = now + 40.0;
                        } else if stay_in && pos.distance(player) > 20.0 {
                            let m = &mut self.mobs.list[i];
                            m.body.pos = home;
                            m.body.vel = Vec3::ZERO;
                        }
                        continue;
                    }
                }
            }
            // Walk the next step.
            let (cell, ahead) = {
                let plan = self.npc_life.folk.get_mut(&id).unwrap().plan.as_mut().unwrap();
                let c = plan.path[plan.k];
                let at = c.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
                let flat = Vec3::new(at.x - pos.x, 0.0, at.z - pos.z).length();
                if flat < 0.65 && (at.y - pos.y).abs() < 1.2 {
                    plan.k += 1;
                    plan.best = f32::MAX;
                    plan.stuck = 0.0;
                } else if flat < plan.best - 0.05 {
                    plan.best = flat;
                    plan.stuck = 0.0;
                } else {
                    plan.stuck += step;
                }
                if plan.stuck > 3.0 {
                    // Pushed off the way (or a block went down in it): look again.
                    plan.k = plan.path.len();
                    plan.fails += 1;
                }
                let k = plan.k.min(plan.path.len() - 1);
                (plan.path[k], plan.path.get(k + 1).copied())
            };
            for c in std::iter::once(cell).chain(ahead) {
                self.npc_open_door(c);
            }
            if std::env::var("MC_NPC_DEBUG").is_ok() {
                let b = self.reg.get(self.world.block_or_air(cell));
                log::info!("npc_life debug: {} pos {:.2?} goal {:?} step {:?} ({:?} v{}) path {:?}", name_of(self, i), pos, goal, cell, b.shape, b.variant, self.npc_life.folk[&id].plan.as_ref().map(|p| (p.k, p.path.len(), p.fails)));
            }
            let m = &mut self.mobs.list[i];
            m.go_to(cell.as_vec3() + Vec3::new(0.5, 0.0, 0.5), 2.0);
            m.sleeping = false;
        }
    }

    /// Lie down in the bed whose head is at `head`.
    fn npc_lie_down(&mut self, i: usize, head: IVec3) {
        let Some((h, facing)) = self.bed_head(head) else { return };
        let [x, z] = bed::dir(facing);
        let dir = Vec3::new(x as f32, 0.0, z as f32);
        let feet = h.as_vec3() + Vec3::new(0.5, bed::TOP as f32 / 16.0, 0.5) - dir * 1.45;
        let m = &mut self.mobs.list[i];
        m.body.pos = feet;
        m.body.vel = Vec3::ZERO;
        // The model's head lies along its -z: turn so it points at the pillow.
        m.yaw = (-dir.x).atan2(-dir.z);
        m.fall_asleep(600.0);
        m.lying = true;
        m.anchor = None;
    }

    /// A closed wooden door at `c` (either half): open it for a passer-by.
    fn npc_open_door(&mut self, c: IVec3) {
        let b = self.reg.get(self.world.block_or_air(c));
        if b.shape != Shape::Door || self.reg.get(b.base).name.starts_with("iron_") {
            return;
        }
        let lower = if b.variant & 8 != 0 { c - IVec3::Y } else { c };
        if let Some(d) = self.npc_life.doors.iter_mut().find(|d| d.0 == lower) {
            d.1 = 2.5;
            return;
        }
        if b.variant & 4 == 0 {
            self.set_door(lower, |v| v | 4);
            self.door_follow(lower);
            self.sfx_at(crate::audio::Sfx::Place(crate::audio::Material::Wood), lower.as_vec3() + Vec3::splat(0.5), 0.6, 16.0);
            self.npc_life.doors.push((lower, 2.5));
        }
    }

    /// Doors folk opened swing shut once nobody stands in them.
    fn close_npc_doors(&mut self, dt: f32) {
        let mut shut = Vec::new();
        for d in &mut self.npc_life.doors {
            d.1 -= dt;
            if d.1 <= 0.0 {
                let c = d.0.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
                let busy = self.body.pos.distance(c) < 1.3 || self.mobs.list.iter().any(|m| m.alive() && m.body.pos.distance(c) < 1.1);
                if busy {
                    d.1 = 0.5;
                } else {
                    shut.push(d.0);
                }
            }
        }
        self.npc_life.doors.retain(|d| !shut.contains(&d.0));
        for lower in shut {
            let b = self.reg.get(self.world.block_or_air(lower));
            if b.shape == Shape::Door && b.variant & 4 != 0 {
                self.set_door(lower, |v| v & !4);
                self.door_follow(lower);
                self.sfx_at(crate::audio::Sfx::Place(crate::audio::Material::Wood), lower.as_vec3() + Vec3::splat(0.5), 0.5, 16.0);
            }
        }
    }

    /// Test hook MC_NPC_LIFE=night|day|bed: two cottages with the guide and the merchant, one
    /// bed in each; the clock is set and the routine runs for MC_NPC_T seconds (default 20)
    /// before the screenshot, so the folk are seen where it took them.
    pub(crate) fn test_npc_life(&mut self, what: &str) {
        let corners = self.test_houses();
        let corners_for_roof = corners.clone();
        // A bed in each cottage, along the right-hand wall, head to the back.
        let bed_id = self.reg.id_or_air("red_bed");
        if what != "nobed" {
            for o in corners {
                let r = self.place_bed(o + IVec3::new(3, 1, 3), bed_id, 0);
                log::info!("npc_life: bed at {:?}: {:?} (id {bed_id})", o + IVec3::new(3, 1, 3), r);
            }
        }
        // Everyone starts outside, in front of the houses.
        let f = self.forward();
        let flat = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let mut k = 0.0;
        let start = self.body.pos + flat * 2.0;
        for m in self.mobs.list.iter_mut().filter(|m| m.home.is_some()) {
            m.body.pos = start + Vec3::new(k - 1.0, 0.0, 0.0);
            k += 1.5;
        }
        // MC_NPC_ROOF=0: no roofs, the camera up high looking down into the rooms.
        let open_top = std::env::var("MC_NPC_ROOF").is_ok_and(|v| v == "0");
        self.time_of_day = match what {
            "day" => 4000.0,
            _ => 14500.0,
        };
        let secs = std::env::var("MC_NPC_T").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(20.0);
        let a = self.time_of_day / 24000.0 * std::f32::consts::TAU;
        let daylight = (a.sin() * 2.2 + 0.45).clamp(0.0, 1.0);
        let dt = 0.05;
        for _ in 0..(secs / dt) as usize {
            self.update_npc_life(dt);
            let out = self.mobs.update(&self.world, self.body.pos, true, daylight, dt);
            self.social_effects(&out);
            self.mobs.animate(self.body.pos, dt);
        }
        for m in self.mobs.list.iter().filter(|m| m.home.is_some()) {
            let d = &self.mobs.reg.defs[m.def];
            log::info!("npc_life: {} at {:.1?} home {:.1?} lying {} sleeping {}", d.name, m.body.pos, m.home.unwrap(), m.lying, m.sleeping);
        }
        if open_top {
            for o in &corners_for_roof {
                for x in 0..5 {
                    for z in 0..5 {
                        self.world.set_block(*o + IVec3::new(x, 5, z), AIR);
                    }
                }
            }
            let mid = corners_for_roof.iter().map(|o| o.as_vec3()).sum::<Vec3>() / corners_for_roof.len().max(1) as f32 + Vec3::new(2.5, 0.0, 2.5);
            self.body.pos = mid - flat * 2.5 + Vec3::Y * 9.0;
            self.body.flying = true;
            self.yaw = (-flat.x).atan2(-flat.z);
            self.pitch = -1.3;
        }
    }
}

fn name_of(g: &Game, i: usize) -> String {
    g.mobs.reg.defs[g.mobs.list[i].def].name.clone()
}

fn goal_pos(g: Goal) -> Vec3 {
    match g {
        Goal::Home(h) => h,
        Goal::Bed(h) => h.as_vec3() + Vec3::splat(0.5),
        Goal::Out(p) => p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folk_names() {
        assert!(is_folk("npc_guide"));
        assert!(is_folk("settler_farmer"));
        assert!(is_folk("goblin_trader"));
        assert!(!is_folk("zombie"));
        assert!(!is_folk("cat_tame"));
    }

    #[test]
    fn nodes_pop_cheapest_first() {
        let mut h = BinaryHeap::new();
        h.push(Node { f: 30, p: IVec3::ZERO });
        h.push(Node { f: 10, p: IVec3::X });
        h.push(Node { f: 20, p: IVec3::Y });
        assert_eq!(h.pop().unwrap().f, 10);
        assert_eq!(h.pop().unwrap().f, 20);
    }
}
