//! The player's own body: seen in third person (F5) and as an arm in first person.
//! Animations follow the ideas of detailed-animation packs: the body turns lazily after the
//! head, arms and legs swing with speed, the player leans when sneaking, swims face down with
//! strokes, flails while falling, squashes on landing, breathes when idle and swings to hit.

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{IVec3, Mat4, Quat, Vec3};
use voxel::mob::MobDef;

use crate::entity::{Bones, MOUTH_PART};

/// Body part indexes in the "player" model of mobs.ron.
const HEAD: usize = 0;
const BODY: usize = 1;
const R_ARM: usize = 2;
const L_ARM: usize = 3;
const R_LEG: usize = 4;
const L_LEG: usize = 5;

/// What the body is doing this frame (filled in by the game).
#[derive(Default, Clone, Copy)]
pub struct Pose {
    pub pos: Vec3,
    /// Horizontal velocity.
    pub vel: Vec3,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub on_ground: bool,
    pub in_water: bool,
    pub head_in_water: bool,
    pub flying: bool,
    pub sneaking: f32,
    /// 0..1 lying down.
    pub crawling: f32,
    pub sprinting: bool,
    pub burning: bool,
    pub hurt: bool,
    /// In a cart or on a chair.
    pub sitting: bool,
    /// Something in the right hand: the arm is held a little forward, as in the classic game.
    pub holding: bool,
    /// A torch in the right hand: held up high ahead, lighting the way (torch.rs).
    pub torch: bool,
    /// A spear held in both hands (spear.rs): the body turns sideways, both fists on the shaft.
    pub spear: bool,
    /// A shield coming up to block: 0..1, and in which hand (shield.rs).
    pub shield_up: f32,
    pub shield_left: bool,
    /// 0..1 a bow raised and drawn: held out in the left hand, the right pulling the string.
    pub bow: f32,
    /// 0..1 a gun raised to aim (guns.rs); with `gun_two` the left hand holds its foregrip.
    pub gun: f32,
    pub gun_two: bool,
    /// Pulling up onto a ledge: 0 = not climbing, else 0..1 through the climb, and the
    /// ledge's top above the feet.
    pub mantle: f32,
    pub ledge: f32,
    /// Lying on the back in a bed whose head end is this way (the body's yaw, beds.rs);
    /// `pos` is then where the feet are.
    pub sleeping: Option<f32>,
    /// 0..1 the hand brought up to the mouth to drink or eat, the seconds since it started,
    /// and whether it is a bottle (the head tips back to drink).
    pub to_mouth: f32,
    pub sip: f32,
    pub bottle: bool,
    /// Killed: the body crumples and falls (fight_anim.rs).
    pub dead: bool,
}

/// Smoothed animation state that lives between frames.
#[derive(Default)]
pub struct Animator {
    pub body_yaw: f32,
    walk: f32,
    amount: f32,
    time: f32,
    air_time: f32,
    land: f32,
    swim: f32,
    lean: f32,
    /// 0..1 how far into a sprint: the body leans forward into the run.
    sprint: f32,
    /// Attack swing 0..1 (0 = not swinging).
    pub swing: f32,
    /// How the weapon moves (Better Combat idea, see `mods/combat.rs`): 0 chop down from
    /// above, 1 slash across, 2 thrust ahead, 3 uppercut from below.
    pub swing_style: u8,
    was_ground: bool,
    fall_speed: f32,
    /// 0..1 head under water, and the breaststroke phase for the first-person arms.
    pub underwater: f32,
    stroke: f32,
    /// 0..1 how open the mouth is (voice chat, voice.rs).
    pub talk: f32,
    /// Climbing a ledge (copied from the pose for the first-person arms).
    mantle: f32,
    ledge: f32,
    /// 0..1 how far the held item is raised: it drops after a hit or a change of item and
    /// comes back up as the weapon gets ready (`follow_hand`), like the classic game.
    pub hand_height: f32,
    hand_item: Option<u16>,
    /// 0..1 a bow brought up to shoot (`follow_bow`): from the right hand at rest to both hands.
    pub bow_aim: f32,
    /// Drinking or eating, copied from the pose each frame for the first-person hand.
    pub to_mouth: f32,
    pub sip: f32,
    pub bottle: bool,
    /// Seconds a swing takes (the weapon's own swing time, fight_anim.rs); 0 = the plain one.
    pub swing_time: f32,
    /// What kind of weapon is in hand, for its moves (fight_anim::Arms).
    pub arms: crate::fight_anim::Arms,
    /// Hit-stop: seconds the swing stays frozen at its impact, and the freeze waiting for the
    /// swing to reach it.
    hitstop: f32,
    stop_pending: f32,
    /// Seconds since the last hit taken, and which way it pushed (model space, flat).
    flinch_age: f32,
    flinch_dir: Vec3,
    was_hurt: bool,
    /// Seconds since a gun went off, and 0..1 how far the gun arm is raised to aim.
    shot_t: f32,
    pub gun_aim: f32,
    /// 0..1 through falling down dead.
    dead_t: f32,
}

/// Which classic item model places what is held in first person.
#[derive(Clone, Copy, PartialEq)]
pub enum HandModel {
    /// "handheld": weapons, gripped at the handle.
    Tool,
    /// "handheld" digging tools: the same grip, with the head up and facing the player.
    Digger,
    /// Drills and chainsaws: gripped by the handle, the bit or bar pointing ahead.
    Powered,
    /// "generated": flat pictures.
    Flat,
    Block,
}

fn wrap(a: f32) -> f32 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

impl Animator {
    /// A body lying still (a corpse), facing `yaw`.
    pub fn lying(yaw: f32) -> Animator {
        Animator { body_yaw: yaw, ..Default::default() }
    }

    pub fn start_swing(&mut self) {
        if self.swing <= 0.0 || self.swing > 0.5 {
            self.swing = 0.001;
            self.swing_style = crate::fight_anim::plain_style(self.arms);
            self.stop_pending = 0.0;
            self.hitstop = 0.0;
            if self.swing_style == crate::fight_anim::style::SHOOT {
                self.shot_t = 0.0;
            }
        }
    }

    /// Another player's swing as it arrived over the network.
    pub fn sync_swing(&mut self, swing: f32, style: u8) {
        if style == crate::fight_anim::style::SHOOT && swing > 0.0 {
            self.shot_t = 0.0;
        }
        self.swing = swing;
        self.swing_style = style;
    }

    /// The swing that just started hit something: it freezes for `secs` at its impact.
    pub fn land_hit(&mut self, secs: f32) {
        self.stop_pending = secs;
    }

    /// 0..1 the jolt of the last hit taken (for the camera's hurt tilt), and which way it
    /// pushed, in the body's own space (+x the body's left, +z ahead).
    pub fn flinch(&self) -> (f32, Vec3) {
        (crate::fight_anim::flinch(self.flinch_age), self.flinch_dir)
    }

    /// 0..1 how far through falling down dead.
    pub fn dead_fall(&self) -> f32 {
        self.dead_t
    }

    /// A spell cast: the staff or book is pushed out toward the aim and drawn back
    /// (fight_anim's cast move). A rapid caster (the flame book) keeps it out instead of
    /// restarting every puff.
    pub fn start_cast(&mut self) {
        let cast = crate::fight_anim::style::CAST;
        if self.swing_style == cast && self.swing > 0.0 && self.swing < 0.6 {
            self.swing = self.swing.min(0.35);
            return;
        }
        self.swing = 0.001;
        self.swing_style = cast;
        self.swing_time = 0.0;
    }

    /// A spear is held in both hands right now (not while swimming, climbing, sitting or
    /// lying down; then it rides in the fist like any other tool).
    pub fn spear_hold(&self, p: &Pose) -> bool {
        p.spear && self.swim < 0.5 && p.mantle <= 0.0 && !p.sitting && p.sleeping.is_none() && p.crawling * (1.0 - self.swim) < 0.5
    }

    /// The walking sway of the hands in first person (camera space).
    pub fn hand_sway(&self) -> Vec3 {
        let a = self.amount.min(1.0);
        Vec3::new(self.walk.sin() * 0.02 * a, -self.walk.cos().abs() * 0.025 * a, 0.0)
    }

    /// Both hands are on a ledge being climbed: nothing is held up in first person.
    pub fn hands_busy(&self) -> bool {
        self.mantle > 0.0
    }

    /// The held item drops when it changes and rises with the weapon's readiness `ready`
    /// (0..1); the classic game steps it at most 0.4 a tick.
    pub fn follow_hand(&mut self, item: Option<u16>, ready: f32, dt: f32) {
        let want = if item == self.hand_item { ready * ready * ready } else { 0.0 };
        let step = (dt * 20.0 * 0.4).min(0.4);
        self.hand_height += (want - self.hand_height).clamp(-step, step);
        if self.hand_height <= 0.0 {
            self.hand_item = item;
        }
    }

    /// A bow comes up while it is being drawn and goes back down after the shot.
    pub fn follow_bow(&mut self, drawing: bool, dt: f32) {
        let want = if drawing { 1.0 } else { 0.0 };
        let rate = if drawing { 7.0 } else { 4.0 };
        self.bow_aim += (want - self.bow_aim).clamp(-dt * rate, dt * rate);
    }

    /// Start a weapon swing of the given style right away (combo attacks).
    pub fn start_styled_swing(&mut self, style: u8) {
        self.swing = 0.001;
        self.swing_style = style;
        self.stop_pending = 0.0;
        self.hitstop = 0.0;
    }

    pub fn update(&mut self, p: &Pose, dt: f32) {
        if self.time == 0.0 {
            // The first frame: the body starts out facing where the player looks.
            self.body_yaw = p.look_yaw + PI;
            self.flinch_age = 10.0;
            self.shot_t = 10.0;
        }
        self.time += dt;
        if self.hitstop > 0.0 {
            // Frozen at the impact for a moment: the blow lands.
            self.hitstop -= dt;
        } else if self.swing > 0.0 {
            let before = self.swing;
            let time = if self.swing_time > 0.0 { self.swing_time } else { crate::fight_anim::PLAIN_TIME };
            self.swing += dt / time;
            let st = self.swing_style;
            if self.stop_pending > 0.0 && crate::fight_anim::past_impact(st, self.swing) && !crate::fight_anim::past_impact(st, before) {
                self.swing = crate::fight_anim::timing(st).hit;
                self.hitstop = self.stop_pending;
                self.stop_pending = 0.0;
            }
            if self.swing >= 1.0 {
                self.swing = 0.0;
                self.stop_pending = 0.0;
            }
        }
        // A gun stays up, aimed, for a moment after each shot.
        self.shot_t += dt;
        let aiming = self.arms == crate::fight_anim::Arms::Gun && self.shot_t < 1.6;
        self.gun_aim += ((aiming as u8 as f32) - self.gun_aim) * (dt * if aiming { 14.0 } else { 4.0 }).min(1.0);
        // A hit taken: the body jolts away from it (the knockback shows which way).
        self.flinch_age += dt;
        if p.hurt && !self.was_hurt && !p.dead {
            self.flinch_age = 0.0;
            let flat = Vec3::new(p.vel.x, 0.0, p.vel.z);
            let local = Quat::from_rotation_y(-self.body_yaw) * flat;
            self.flinch_dir = if local.length() > 0.5 { local.normalize() } else { -Vec3::Z };
        }
        self.was_hurt = p.hurt;
        self.dead_t = if p.dead { (self.dead_t + dt / 0.9).min(1.0) } else { 0.0 };
        let speed = Vec3::new(p.vel.x, 0.0, p.vel.z).length();
        let target = (speed / 4.3).min(1.6);
        self.amount += (target - self.amount) * (dt * 8.0).min(1.0);
        self.walk += speed * dt * if p.crawling > 0.5 { 4.0 } else if p.sprinting { 1.25 } else { 1.6 };
        // The facing direction in model space (+Z forward) for the camera yaw.
        let face = p.look_yaw + PI;
        // Moving: the body turns toward the movement; standing: it follows once the head
        // has turned more than ~50 degrees.
        let mut want = self.body_yaw;
        if speed > 0.5 {
            let move_yaw = p.vel.x.atan2(p.vel.z);
            let rel = wrap(move_yaw - face);
            // Walking backwards keeps facing forward.
            want = if rel.abs() > FRAC_PI_2 { face + wrap(rel + PI) * 0.5 } else { face + rel * 0.5 };
        } else {
            // Lying down the neck can't twist far, so the body follows sooner.
            let slack = 0.9 - 0.6 * p.crawling;
            let d = wrap(face - self.body_yaw);
            if d.abs() > slack {
                want = face - d.signum() * slack;
            }
        }
        self.body_yaw += wrap(want - self.body_yaw) * (dt * 7.0).min(1.0);

        if p.on_ground || p.in_water || p.flying {
            if !self.was_ground && self.air_time > 0.25 {
                self.land = (self.fall_speed / 14.0).clamp(0.2, 1.0);
            }
            self.air_time = 0.0;
            self.fall_speed = 0.0;
        } else {
            self.air_time += dt;
            self.fall_speed = self.fall_speed.max(-p.vel.y);
        }
        self.was_ground = p.on_ground || p.in_water || p.flying;
        self.land = (self.land - dt * 4.0).max(0.0);
        let running = p.sprinting && speed > 3.0 && !p.in_water && !p.flying && p.crawling < 0.5 && !p.sitting;
        self.sprint += ((running as u8 as f32) - self.sprint) * (dt * if running { 5.0 } else { 7.0 }).min(1.0);
        let under = p.head_in_water && !p.flying;
        self.underwater += ((under as u8 as f32) - self.underwater) * (dt * 6.0).min(1.0);
        // Slow treading strokes when still, quicker ones when swimming along.
        self.stroke += dt * (0.55 + (speed / 3.0).min(1.0) * 0.6);
        let swimming = p.head_in_water && speed > 1.0;
        self.swim += ((swimming as u8 as f32) - self.swim) * (dt * 5.0).min(1.0);
        self.mantle = p.mantle;
        self.ledge = p.ledge;
        if p.mantle > 0.0 {
            // Hanging on a ledge is neither a jump nor a fall.
            self.air_time = 0.0;
            self.fall_speed = 0.0;
        }
        let lean_target = if p.flying { (speed / 11.0).min(1.0) } else { 0.0 };
        self.lean += (lean_target - self.lean) * (dt * 4.0).min(1.0);
    }

    /// Third person: the whole body at `p.pos`.
    pub fn bones(&self, def: &MobDef, p: &Pose, cam_block: IVec3, sky: f32, blk: f32) -> Bones {
        let mut b = Bones { light: [sky, blk, if p.hurt { 1.0 } else { 0.0 }, 0.0], ..Default::default() };
        let rel = p.pos - cam_block.as_vec3();
        let t = self.walk;
        let a = self.amount;
        let idle = (1.0 - a).max(0.0);
        let breathe = (self.time * 1.6).sin();
        let airborne = self.air_time > 0.12 && !p.flying;
        let crouch = p.sneaking;

        // Whole-body transform: squash on landing, crouch, swim and fly tilt.
        let mut root = Mat4::from_translation(rel) * Mat4::from_rotation_y(self.body_yaw);
        // A spring in the step: the body rises as each leg passes under it and the weight
        // shifts from foot to foot.
        let step = if p.on_ground && !p.sitting && p.crawling < 0.5 && !p.in_water { a.min(1.2) } else { 0.0 };
        root = root * Mat4::from_translation(Vec3::new(0.0, t.cos().abs() * (0.04 + p.sprinting as u8 as f32 * 0.03) * step, 0.0)) * Mat4::from_rotation_z(t.sin() * 0.02 * step);
        // Sprinting: the whole body leans forward from the feet into the run, sinking a touch.
        let run = self.sprint;
        if run > 0.001 {
            root = root * Mat4::from_rotation_x(run * 0.26) * Mat4::from_translation(Vec3::new(0.0, -run * 0.05, 0.0));
        }
        let squash = self.land * 0.12;
        root = root * Mat4::from_translation(Vec3::new(0.0, -crouch * 0.18 - squash * 0.5, 0.0));
        // Lying down: the body tips flat and sinks to the ground.
        let prone = p.crawling * (1.0 - self.swim);
        root = root * Mat4::from_translation(Vec3::new(0.0, -prone * 0.6, 0.0));
        // Swimming, flying and crawling pitch the body around its middle.
        let tilt = (self.swim * 1.35 + self.lean * 0.9 + prone * FRAC_PI_2).min(FRAC_PI_2);
        if tilt > 0.0 {
            root = root * Mat4::from_translation(Vec3::Y * 0.75) * Mat4::from_rotation_x(tilt) * Mat4::from_translation(-Vec3::Y * 0.75);
        }
        // A hit taken: the whole body rocks away from it, from the feet.
        let (jolt, push) = self.flinch();
        if jolt > 0.0 && !p.dead {
            let axis = Vec3::Y.cross(push).normalize_or_zero();
            if axis != Vec3::ZERO {
                root = root * Mat4::from_quat(Quat::from_axis_angle(axis, 0.2 * jolt));
            }
        }
        if p.dead {
            // Killed: the knees buckle, then the body topples the way the last hit pushed
            // (backwards when nothing did) and settles with a small bounce.
            let d = self.dead_t;
            let buckle = (d / 0.3).clamp(0.0, 1.0);
            let fall = ((d - 0.15) / 0.7).clamp(0.0, 1.0);
            let fall = fall * fall;
            let settle = if d > 0.85 { ((d - 0.85) / 0.15 * PI).sin() * 0.06 } else { 0.0 };
            let dir = if self.flinch_age < 3.0 { self.flinch_dir } else { -Vec3::Z };
            let axis = Vec3::Y.cross(dir).normalize_or_zero();
            let axis = if axis == Vec3::ZERO { Vec3::X } else { axis };
            root = root * Mat4::from_translation(Vec3::new(0.0, -0.12 * buckle * (1.0 - fall) + 0.14 * fall, 0.0)) * Mat4::from_quat(Quat::from_axis_angle(axis, fall * FRAC_PI_2 * 0.98 - settle));
        }
        root = root * Mat4::from_scale(Vec3::new(1.0 / 16.0, (1.0 - squash) / 16.0, 1.0 / 16.0));
        if let Some(yaw) = p.sleeping {
            // On the back: the head end toward -z, the back on the blanket, a little smaller
            // so the whole body fits on the bed.
            let k = 0.9 / 16.0;
            root = Mat4::from_translation(rel) * Mat4::from_rotation_y(yaw) * Mat4::from_translation(Vec3::Y * 2.0 * k) * Mat4::from_rotation_x(-FRAC_PI_2) * Mat4::from_scale(Vec3::splat(k));
        }

        let mut rot = [Quat::IDENTITY; 6];
        let mut shift = [Vec3::ZERO; 6];
        // Legs: stride, longer and faster when running.
        let stride = if p.sprinting { 1.05 } else { 0.8 };
        let leg = t.sin() * stride * a.min(1.2);
        rot[R_LEG] = Quat::from_rotation_x(leg);
        rot[L_LEG] = Quat::from_rotation_x(-leg);
        // Arms swing against the legs, a little outward; idle breathing sways them.
        let arm = -leg * (0.85 + run * 0.45);
        let sway = 0.04 + breathe * 0.03 * idle;
        // Holding something, the right arm swings half as far and stays raised 18° forward.
        let r_arm = if p.torch {
            arm * 0.25 - PI / 4.0
        } else if p.holding {
            arm * 0.5 - PI / 10.0
        } else {
            arm
        };
        rot[R_ARM] = Quat::from_rotation_x(r_arm + breathe * 0.03 * idle) * Quat::from_rotation_z(-sway);
        rot[L_ARM] = Quat::from_rotation_x(-arm - breathe * 0.03 * idle) * Quat::from_rotation_z(sway);
        if run > 0.001 {
            // Running: the chest drives forward over the hips, the head stays up looking ahead,
            // arms pump hard and the knees come up high.
            rot[BODY] = Quat::from_rotation_x(0.16 * run);
            rot[HEAD] = Quat::from_rotation_x(-0.32 * run);
        }
        if airborne && self.swim < 0.5 {
            if self.air_time < 0.5 && !p.in_water {
                // Jump: arms up and out, one knee forward.
                rot[R_ARM] = Quat::from_rotation_x(-0.5) * Quat::from_rotation_z(-0.35);
                rot[L_ARM] = Quat::from_rotation_x(-0.5) * Quat::from_rotation_z(0.35);
                rot[R_LEG] = Quat::from_rotation_x(-0.45);
                rot[L_LEG] = Quat::from_rotation_x(0.2);
            } else {
                // Long fall: arms flail.
                let f = self.time * 14.0;
                rot[R_ARM] = Quat::from_rotation_z(-1.2 - f.sin() * 0.4) * Quat::from_rotation_x(f.cos() * 0.4);
                rot[L_ARM] = Quat::from_rotation_z(1.2 + f.cos() * 0.4) * Quat::from_rotation_x(f.sin() * 0.4);
                rot[R_LEG] = Quat::from_rotation_x((f * 0.7).sin() * 0.5);
                rot[L_LEG] = Quat::from_rotation_x(-(f * 0.7).sin() * 0.5);
            }
        }
        if p.in_water && !p.head_in_water && a < 0.4 {
            // Wading: arms float out and paddle slowly.
            let w = self.time * 2.0;
            rot[R_ARM] = Quat::from_rotation_z(-0.6 - w.sin() * 0.15);
            rot[L_ARM] = Quat::from_rotation_z(0.6 + w.sin() * 0.15);
        }
        if self.swim > 0.05 {
            // Breaststroke and flutter kick.
            let s = self.time * 3.2;
            let k = self.swim;
            let stroke = (s.sin() * 0.5 + 0.5) * 2.4;
            rot[R_ARM] = rot[R_ARM].slerp(Quat::from_rotation_x(-PI + 0.3) * Quat::from_rotation_z(-stroke * 0.6), k);
            rot[L_ARM] = rot[L_ARM].slerp(Quat::from_rotation_x(-PI + 0.3) * Quat::from_rotation_z(stroke * 0.6), k);
            let kick = (self.time * 9.0).sin() * 0.35;
            rot[R_LEG] = rot[R_LEG].slerp(Quat::from_rotation_x(kick), k);
            rot[L_LEG] = rot[L_LEG].slerp(Quat::from_rotation_x(-kick), k);
        }
        if p.flying && self.lean > 0.1 {
            rot[R_LEG] = Quat::from_rotation_x(0.1);
            rot[L_LEG] = Quat::from_rotation_x(0.1);
            rot[R_ARM] = Quat::from_rotation_x(0.35) * Quat::from_rotation_z(-0.1);
            rot[L_ARM] = Quat::from_rotation_x(0.35) * Quat::from_rotation_z(0.1);
        }
        if crouch > 0.01 {
            // Sneak: lean forward, arms a bit forward, legs stay under the hips.
            rot[BODY] = Quat::from_rotation_x(0.5 * crouch);
            shift[HEAD] = Vec3::new(0.0, -1.0, 4.0) * crouch;
            shift[R_ARM] = Vec3::new(0.0, -1.0, 3.5) * crouch;
            shift[L_ARM] = Vec3::new(0.0, -1.0, 3.5) * crouch;
            rot[R_ARM] = rot[R_ARM] * Quat::from_rotation_x(-0.4 * crouch);
            rot[L_ARM] = rot[L_ARM] * Quat::from_rotation_x(-0.4 * crouch);
            shift[R_LEG] = Vec3::new(0.0, 0.0, -2.0) * crouch;
            shift[L_LEG] = Vec3::new(0.0, 0.0, -2.0) * crouch;
        }
        if prone > 0.05 {
            // Army crawl: the arms take turns reaching ahead and pulling, the legs push.
            let c = t;
            let move_k = a.min(1.0);
            let reach = |ph: f32| -PI + 0.5 + (ph.sin() * 0.55) * move_k;
            rot[R_ARM] = rot[R_ARM].slerp(Quat::from_rotation_x(reach(c)) * Quat::from_rotation_z(-0.25 - c.cos().max(0.0) * 0.3 * move_k), prone);
            rot[L_ARM] = rot[L_ARM].slerp(Quat::from_rotation_x(reach(c + PI)) * Quat::from_rotation_z(0.25 + (-c.cos()).max(0.0) * 0.3 * move_k), prone);
            let push = c.sin() * 0.35 * move_k;
            rot[R_LEG] = rot[R_LEG].slerp(Quat::from_rotation_x(-push) * Quat::from_rotation_z(0.08 + push.max(0.0) * 0.5), prone);
            rot[L_LEG] = rot[L_LEG].slerp(Quat::from_rotation_x(push) * Quat::from_rotation_z(-0.08 - (-push).max(0.0) * 0.5), prone);
            // The body rolls a little with each pull.
            rot[BODY] = rot[BODY] * Quat::from_rotation_y(c.sin() * 0.08 * move_k * prone);
            if a < 0.05 {
                // Still: chin on the arms, slow breathing.
                rot[R_ARM] = rot[R_ARM].slerp(Quat::from_rotation_x(-PI + 0.9) * Quat::from_rotation_y(0.6), prone);
                rot[L_ARM] = rot[L_ARM].slerp(Quat::from_rotation_x(-PI + 0.9) * Quat::from_rotation_y(-0.6), prone);
            }
        }
        if p.mantle > 0.0 {
            // Climbing a ledge: both hands on the edge, the body hauls itself up, then leans
            // over the top while one knee comes up onto it and the arms push down behind.
            let (f, over) = mantle_reach(p.mantle);
            let grab = (p.mantle / voxel::physics::MANTLE_GRAB).min(1.0);
            let angle = 0.3 * (1.0 - grab) + mantle_arm(f, p.ledge - 1.4) * grab;
            let kick = (self.time * 11.0).sin() * 0.25 * (1.0 - over);
            rot[R_ARM] = Quat::from_rotation_x(-angle) * Quat::from_rotation_z(-0.12);
            rot[L_ARM] = Quat::from_rotation_x(-angle) * Quat::from_rotation_z(0.12);
            rot[BODY] = Quat::from_rotation_x(0.12 + 0.35 * over);
            // Feet scrabble against the wall, then the right knee steps up.
            rot[R_LEG] = Quat::from_rotation_x(-0.25 + kick - 1.2 * (over * PI).sin());
            rot[L_LEG] = Quat::from_rotation_x(0.1 - kick + 0.35 * over);
        }
        if p.sitting {
            // Seated: thighs forward, a little apart; hands resting on the knees.
            rot[R_LEG] = Quat::from_rotation_x(-1.45) * Quat::from_rotation_z(0.1);
            rot[L_LEG] = Quat::from_rotation_x(-1.45) * Quat::from_rotation_z(-0.1);
            rot[R_ARM] = Quat::from_rotation_x(-0.55 + breathe * 0.02) * Quat::from_rotation_z(-0.05);
            rot[L_ARM] = Quat::from_rotation_x(-0.55 - breathe * 0.02) * Quat::from_rotation_z(0.05);
        }
        if p.burning {
            let f = self.time * 18.0;
            rot[R_ARM] = Quat::from_rotation_x(-2.4 + f.sin() * 0.6);
            rot[L_ARM] = Quat::from_rotation_x(-2.4 + f.cos() * 0.6);
        }
        if p.shield_up > 0.0 {
            // Blocking: the shield arm comes up in front, turned in across the chest.
            let (i, side) = if p.shield_left { (L_ARM, 1.0) } else { (R_ARM, -1.0) };
            let up = Quat::from_rotation_y(-side * 0.45) * Quat::from_rotation_x(-0.95 + arm * 0.15);
            rot[i] = rot[i].slerp(up, p.shield_up.clamp(0.0, 1.0));
        }
        let bow = p.bow.clamp(0.0, 1.0);
        if bow > 0.0 {
            // Drawing a bow: the right arm, which holds it, goes straight out towards the look;
            // the left reaches across and folds back to pull the string to the cheek. The bow
            // never changes hands.
            let yaw = wrap(p.look_yaw + PI - self.body_yaw).clamp(-1.0, 1.0);
            let pitch = p.look_pitch.clamp(-1.2, 1.2);
            let right = Quat::from_rotation_y(yaw + 0.1) * Quat::from_rotation_x(-FRAC_PI_2 - pitch);
            let left = Quat::from_rotation_y(yaw + 0.75) * Quat::from_rotation_x(-FRAC_PI_2 - pitch + 0.05) * Quat::from_rotation_y(-FRAC_PI_2);
            rot[L_ARM] = rot[L_ARM].slerp(left, bow);
            rot[R_ARM] = rot[R_ARM].slerp(right, bow);
            rot[BODY] = rot[BODY] * Quat::from_rotation_y(0.25 * bow);
        }
        let gun = p.gun.clamp(0.0, 1.0);
        if gun > 0.0 {
            // Aiming a gun: the right arm points it at the look; a long gun's stock comes in
            // to the shoulder and the left hand reaches across under the barrel.
            let yaw = wrap(p.look_yaw + PI - self.body_yaw).clamp(-1.0, 1.0);
            let pitch = p.look_pitch.clamp(-1.2, 1.2);
            if p.gun_two {
                let right = Quat::from_rotation_y(yaw + 0.26) * Quat::from_rotation_x(-FRAC_PI_2 - pitch + 0.25);
                let left = Quat::from_rotation_y(yaw + 0.8) * Quat::from_rotation_x(-FRAC_PI_2 - pitch + 0.05);
                rot[R_ARM] = rot[R_ARM].slerp(right, gun);
                rot[L_ARM] = rot[L_ARM].slerp(left, gun);
                rot[BODY] = rot[BODY] * Quat::from_rotation_y(-0.18 * gun);
            } else {
                let right = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-FRAC_PI_2 - pitch);
                rot[R_ARM] = rot[R_ARM].slerp(right, gun);
            }
        }
        if p.to_mouth > 0.0 {
            // Drinking or eating: the right hand comes up to the mouth, turned in, bobbing with
            // each swallow or bite; a bottle is tipped up and the head goes back with it.
            let k = p.to_mouth.clamp(0.0, 1.0);
            let bob = (p.sip * if p.bottle { 7.5 } else { 14.0 }).sin() * 0.07;
            let mouth = Quat::from_rotation_y(0.62) * Quat::from_rotation_x(-1.95 - if p.bottle { 0.25 } else { 0.0 } + bob) * Quat::from_rotation_z(0.15);
            rot[R_ARM] = rot[R_ARM].slerp(mouth, k);
            if p.bottle {
                rot[HEAD] = rot[HEAD] * Quat::from_rotation_x(-0.32 * k);
            }
        }
        // Knees and elbows (the limb meshes are split at the joint, see entity::joint).
        // Positive swings the lower half back: knees bend that way, elbows the other.
        let mut bend = [0.0f32; 6];
        let knee_k = if p.sprinting { 1.3 } else { 0.95 } * a.min(1.2);
        // A knee folds while its foot swings forward.
        bend[R_LEG] = (-t.cos()).max(0.0) * knee_k + 0.04 * a.min(1.0);
        bend[L_LEG] = t.cos().max(0.0) * knee_k + 0.04 * a.min(1.0);
        let pump = 1.0 * run;
        bend[R_ARM] = -(0.1 + pump + (-arm).max(0.0) * 0.45 + if p.holding { 0.3 } else { 0.0 });
        bend[L_ARM] = -(0.1 + pump + arm.max(0.0) * 0.45);
        if p.gun > 0.0 {
            // Aiming, the forearm holding the gun lines up with the look: straight for a
            // pistol; for a long gun the upper arm hangs a little lower (above) and the elbow
            // bends just enough to bring the barrel level, the left hand under it the same.
            let g = p.gun.min(1.0);
            let (r, l) = if p.gun_two { (-0.25, -0.05) } else { (0.0, bend[L_ARM]) };
            bend[R_ARM] = bend[R_ARM] * (1.0 - g) + r * g;
            bend[L_ARM] = bend[L_ARM] * (1.0 - g) + l * g;
        }
        if airborne && self.swim < 0.5 {
            if self.air_time < 0.5 && !p.in_water {
                bend[R_LEG] = 1.0;
                bend[L_LEG] = 0.35;
                bend[R_ARM] = -0.3;
                bend[L_ARM] = -0.3;
            } else {
                let f = self.time * 14.0;
                bend[R_LEG] = 0.45 + (f * 0.7).sin() * 0.3;
                bend[L_LEG] = 0.45 - (f * 0.7).sin() * 0.3;
                bend[R_ARM] = -0.25 - f.cos().max(0.0) * 0.4;
                bend[L_ARM] = -0.25 - f.sin().max(0.0) * 0.4;
            }
        }
        if self.swim > 0.05 {
            let k = self.swim;
            let kick = (self.time * 9.0).sin();
            bend[R_LEG] = bend[R_LEG] * (1.0 - k) + (0.25 + kick.max(0.0) * 0.4) * k;
            bend[L_LEG] = bend[L_LEG] * (1.0 - k) + (0.25 + (-kick).max(0.0) * 0.4) * k;
            let s = (self.time * 3.2).sin() * 0.5 + 0.5;
            bend[R_ARM] = bend[R_ARM] * (1.0 - k) - (0.2 + (1.0 - s) * 0.8) * k;
            bend[L_ARM] = bend[L_ARM] * (1.0 - k) - (0.2 + (1.0 - s) * 0.8) * k;
        }
        if crouch > 0.01 {
            bend[R_LEG] += 0.6 * crouch;
            bend[L_LEG] += 0.6 * crouch;
            bend[R_ARM] -= 0.3 * crouch;
            bend[L_ARM] -= 0.3 * crouch;
        }
        if prone > 0.05 {
            bend[R_LEG] = bend[R_LEG] * (1.0 - prone) + 0.25 * prone;
            bend[L_LEG] = bend[L_LEG] * (1.0 - prone) + 0.25 * prone;
            bend[R_ARM] = bend[R_ARM] * (1.0 - prone) - 0.7 * prone;
            bend[L_ARM] = bend[L_ARM] * (1.0 - prone) - 0.7 * prone;
        }
        if p.sitting {
            // Thighs forward, shins hanging straight down.
            bend[R_LEG] = 1.45;
            bend[L_LEG] = 1.45;
            bend[R_ARM] = -0.5;
            bend[L_ARM] = -0.5;
        }
        if p.burning {
            bend[R_ARM] = -0.8;
            bend[L_ARM] = -0.8;
        }
        if bow > 0.0 {
            // The bow arm straight, the drawing arm folded at the elbow.
            bend[L_ARM] = bend[L_ARM] * (1.0 - bow);
            bend[R_ARM] = bend[R_ARM] * (1.0 - bow) - 2.0 * bow;
        }
        // A gun raised to aim along the look, the arm straight out.
        let aim = self.gun_aim * (1.0 - self.swim) * (!p.sitting) as u8 as f32;
        // A 3D gun (guns.rs, `p.gun`) has its own aim pose above and kick in guns.rs.
        if aim > 0.0 && bow <= 0.0 && p.gun <= 0.0 {
            let yaw = wrap(p.look_yaw + PI - self.body_yaw).clamp(-1.0, 1.0);
            let q = Quat::from_rotation_y(yaw - 0.05) * Quat::from_rotation_x(-FRAC_PI_2 - p.look_pitch.clamp(-1.2, 1.2));
            rot[R_ARM] = rot[R_ARM].slerp(q, aim);
            bend[R_ARM] *= 1.0 - aim;
        }
        // An attack (fight_anim.rs): the arm winds up, snaps through the blow and follows
        // through; the chest coils and unwinds, the front foot steps into it.
        let mut swing_turn = Quat::IDENTITY;
        if self.swing > 0.0 && !self.spear_hold(p) && p.mantle <= 0.0 && p.gun <= 0.0 {
            let k = crate::fight_anim::pose3(self.swing_style, self.swing, p.look_pitch, rot[R_ARM], bend[R_ARM], (rot[L_ARM], bend[L_ARM]));
            let w = 1.0 - self.swim * 0.6;
            rot[R_ARM] = k.arm;
            bend[R_ARM] = k.elbow;
            if let Some((q, e)) = k.left {
                rot[L_ARM] = q;
                bend[L_ARM] = e;
            }
            swing_turn = Quat::from_rotation_y(k.twist * w);
            rot[BODY] = rot[BODY] * swing_turn * Quat::from_rotation_x(k.lean * w);
            let step = k.step * w * (p.on_ground && !p.sitting) as u8 as f32;
            rot[L_LEG] = Quat::from_rotation_x(-step) * rot[L_LEG];
            rot[R_LEG] = Quat::from_rotation_x(step * 0.45) * rot[R_LEG];
            bend[R_LEG] += step * 0.4;
        }
        // A hit taken: the arms fling out and the elbows give.
        let (jolt, push) = self.flinch();
        if jolt > 0.0 && !p.dead {
            rot[R_ARM] = Quat::from_rotation_z(-0.35 * jolt) * Quat::from_rotation_x(-0.25 * jolt) * rot[R_ARM];
            rot[L_ARM] = Quat::from_rotation_z(0.35 * jolt) * Quat::from_rotation_x(-0.25 * jolt) * rot[L_ARM];
            bend[R_ARM] -= 0.3 * jolt;
            bend[L_ARM] -= 0.3 * jolt;
            bend[R_LEG] += 0.25 * jolt;
            bend[L_LEG] += 0.25 * jolt;
            let _ = push;
        }
        // A spear in both hands: the torso turns sideways, the front foot forward, and both arms
        // reach the shaft (bent at the elbows); a thrust lunges into it (spear.rs).
        let spear = self.spear_hold(p);
        let spear_turn = crate::spear::torso_turn(self.swing);
        if spear {
            rot[BODY] = rot[BODY] * spear_turn;
            let e = crate::spear::thrust(self.swing);
            let stance = idle.min(1.0) * 0.18 + e * 0.3;
            rot[L_LEG] = Quat::from_rotation_x(-stance) * rot[L_LEG];
            rot[R_LEG] = Quat::from_rotation_x(stance * 0.8) * rot[R_LEG];
            let h = crate::spear::hold(self.swing);
            for (i, target) in [(R_ARM, h.right), (L_ARM, h.left)] {
                let (q, b, slide) = crate::spear::reach(&def.parts[i], target);
                rot[i] = q;
                bend[i] = b;
                shift[i] = slide;
            }
        }
        // Head follows the camera, relative to the body.
        let head_yaw = wrap(p.look_yaw + PI - self.body_yaw).clamp(-1.3, 1.3);
        // Lying down the head lifts to look ahead instead of at the ground.
        let head_lift = -prone * 1.25;
        rot[HEAD] = Quat::from_rotation_y(head_yaw) * Quat::from_rotation_x(head_lift - p.look_pitch * (1.0 - self.swim * 0.6) * (1.0 - prone * 0.5)) * rot[HEAD];
        if spear {
            // The head keeps looking ahead while the shoulders turn.
            rot[HEAD] = spear_turn.inverse() * rot[HEAD];
        }
        // Eyes on the target through the swing, too.
        rot[HEAD] = swing_turn.inverse() * rot[HEAD];
        if jolt > 0.0 && !p.dead {
            // The head snaps the way the hit pushed.
            let axis = Vec3::Y.cross(push).normalize_or_zero();
            if axis != Vec3::ZERO {
                rot[HEAD] = Quat::from_axis_angle(axis, 0.35 * jolt) * rot[HEAD];
            }
        }
        if p.dead {
            // Gone limp: arms out, the head lolls, the knees gave way.
            let d = self.dead_t;
            rot[R_ARM] = rot[R_ARM].slerp(Quat::from_rotation_z(-0.5) * Quat::from_rotation_x(-0.2), d);
            rot[L_ARM] = rot[L_ARM].slerp(Quat::from_rotation_z(0.45) * Quat::from_rotation_x(0.1), d);
            rot[R_LEG] = rot[R_LEG].slerp(Quat::from_rotation_z(0.08), d);
            rot[L_LEG] = rot[L_LEG].slerp(Quat::from_rotation_x(-0.15) * Quat::from_rotation_z(-0.1), d);
            rot[HEAD] = rot[HEAD].slerp(Quat::from_rotation_y(0.4) * Quat::from_rotation_x(0.2), d);
            rot[BODY] = rot[BODY].slerp(Quat::IDENTITY, d);
            bend[R_LEG] = 0.5 * (1.0 - d) + 0.15 * d;
            bend[L_LEG] = 0.35;
            bend[R_ARM] = -0.25;
            bend[L_ARM] = -0.4;
        }
        if p.sleeping.is_some() {
            // Still, arms along the sides, the head resting on the pillow, slow breaths.
            rot = [Quat::IDENTITY; 6];
            shift = [Vec3::ZERO; 6];
            rot[HEAD] = Quat::from_rotation_x(0.3);
            shift[HEAD] = Vec3::new(0.0, -1.0, 1.0);
            rot[R_ARM] = Quat::from_rotation_z(-0.06 - breathe * 0.015);
            rot[L_ARM] = Quat::from_rotation_z(0.06 + breathe * 0.015);
            rot[R_LEG] = Quat::from_rotation_z(0.03);
            rot[L_LEG] = Quat::from_rotation_z(-0.03);
            bend = [0.0, 0.0, -0.15, -0.15, 0.08, 0.08];
        }

        for (i, part) in def.parts.iter().enumerate().take(6) {
            let pivot = Vec3::new(part.pivot.0 as f32, part.pivot.1 as f32, part.pivot.2 as f32);
            // The body rotates around the hips, everything above follows it.
            let body = if i == BODY || i == HEAD || i == R_ARM || i == L_ARM {
                let hip = Vec3::new(0.0, 12.0, 0.0);
                Mat4::from_translation(hip) * Mat4::from_quat(rot[BODY]) * Mat4::from_translation(-hip)
            } else {
                Mat4::IDENTITY
            };
            let own = if i == BODY { Mat4::IDENTITY } else { Mat4::from_translation(pivot) * Mat4::from_quat(rot[i]) * Mat4::from_translation(-pivot) };
            b.m[i] = (root * body * Mat4::from_translation(shift[i]) * own).to_cols_array_2d();
            if bend[i] != 0.0 {
                b.set_bend(i, part, bend[i]);
            }
        }
        // Talking: the mouth opens downward from its top edge, two pixels above the chin.
        if self.talk > 0.03 && def.parts.len() <= MOUTH_PART {
            let top = def.parts[HEAD].origin.1 as f32 + 2.0;
            let open = Mat4::from_translation(Vec3::Y * top) * Mat4::from_scale(Vec3::new(1.0, 0.2 + self.talk * 1.1, 1.0)) * Mat4::from_translation(-Vec3::Y * top);
            b.m[MOUTH_PART] = (Mat4::from_cols_array_2d(&b.m[HEAD]) * open).to_cols_array_2d();
        }
        b
    }

    /// First person: only the right arm, held in front of the camera. `eye` is relative to
    /// the camera's block; `bob` is the walking phase for the sway.
    /// Camera space: x right, y up, -z forward.
    fn camera(eye: Vec3, yaw: f32, pitch: f32) -> Mat4 {
        Mat4::from_translation(eye) * Mat4::from_rotation_y(yaw) * Mat4::from_rotation_x(pitch)
    }

    /// The camera frame for the held item and the arm gripping it. They are laid out at their
    /// true sizes, the item as big against the arm as it is in the views from outside, with the
    /// fist about an arm's length ahead; then the whole of it is drawn smaller towards the eye
    /// (`HAND_NEAR`), which looks the same on screen but keeps it out of a wall in front.
    fn hand_camera(eye: Vec3, yaw: f32, pitch: f32) -> Mat4 {
        Self::camera(eye, yaw, pitch) * Mat4::from_scale(Vec3::splat(HAND_NEAR))
    }

    /// Where the right arm sits in camera space, with walking sway and the swing. The shoulder
    /// is the origin and the arm hangs along -Y from it.
    fn arm_place(&self) -> Mat4 {
        let s = if self.swing > 0.0 { (self.swing * PI).sin() } else { 0.0 };
        let s2 = if self.swing > 0.0 { (self.swing.sqrt() * PI).sin() } else { 0.0 };
        let bob = self.walk;
        let a = self.amount.min(1.0);
        let sway = Vec3::new((bob).sin() * 0.03 * a, -((bob).cos().abs()) * 0.035 * a, 0.0);
        let ph = self.swing.max(0.0);
        let (swing_off, turn, tip) = match self.swing_style {
            // Slash: the fist sweeps from the right across to the left.
            1 => (Vec3::new(0.2 * s - 0.55 * ph * s, 0.12 * s, -0.22 * s), 0.25 + s * (0.7 - 1.6 * ph), 1.75 + s * 0.15),
            // Thrust: straight into the screen and back.
            2 => (Vec3::new(-0.12 * s, 0.06 * s, -0.5 * s), 0.25 - s * 0.15, 1.75 + s * 0.05),
            // Uppercut: from low up past the middle.
            3 => (Vec3::new(-0.15 * s, -0.25 * s + 0.5 * ph * s, -0.2 * s), 0.25 - s * 0.4, 1.75 + s * (1.1 - 1.2 * ph)),
            // Cast: the fist pushes forward and a little up toward the crosshair.
            crate::fight_anim::style::CAST => (Vec3::new(-0.1 * s, 0.1 * s, -0.3 * s), 0.25 - s * 0.1, 1.75 + s * 0.25),
            _ => (Vec3::new(-0.18 * s2, 0.12 * (s2 * 2.0 * PI / PI).sin().abs() * 0.5, -0.25 * s), 0.25 - s * 0.6, 1.75 + s * 0.5),
        };
        // Tipping the arm forward makes it reach into the screen.
        Mat4::from_translation(Vec3::new(0.40, -0.46, -0.30) + sway + swing_off)
            * Mat4::from_rotation_y(turn)
            * Mat4::from_rotation_x(tip)
            * Mat4::from_rotation_z(0.15)
    }

    /// The held item in first person: the frame of the item model's centre (one block wide
    /// before its scale), relative to the camera's block. At rest a tool's handle sits in the
    /// lower right with the blade rising to the left and away, seen from its side so its
    /// thickness shows; it drops while the weapon is not ready (`hand_height`). The swings
    /// turn it around the fist through their wind-up, impact and follow-through
    /// (fight_anim.rs keys1), each style its own way.
    pub fn first_person_item(&self, model: HandModel, eye: Vec3, yaw: f32, pitch: f32) -> Mat4 {
        let s = self.swing.max(0.0);
        let a = self.amount.min(1.0);
        let sway = Vec3::new(self.walk.sin() * 0.02 * a, -self.walk.cos().abs() * 0.025 * a, 0.0) * HAND_REACH;
        let (rest, grip, fist) = hand_rest(model);
        let fist = fist + sway + Vec3::Y * -(1.0 - self.hand_height.clamp(0.0, 1.0)) * 0.6 * HAND_REACH;
        // Each attack winds up, snaps through and follows through (fight_anim.rs).
        let k = crate::fight_anim::pose1(self.swing_style, s);
        let (shift, turn) = (k.shift, k.turn);
        // Drinking or eating: the fist comes in to the middle under the eye and a bottle tips up
        // towards it, bobbing with each swallow (classic game: the item rises and shakes).
        let (shift, turn) = if self.to_mouth > 0.0 {
            let k = self.to_mouth.clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k);
            let bob = if self.bottle { (self.sip * 7.5).sin().abs() * 0.05 } else { (self.sip * 14.0).sin().abs() * 0.06 };
            let to = Vec3::new(-0.44, 0.12 + bob, 0.32) * k;
            let tip = if self.bottle {
                Quat::from_rotation_z(0.3 * k) * Quat::from_rotation_x(0.75 * k)
            } else {
                Quat::from_rotation_z(0.25 * k) * Quat::from_rotation_x(0.3 * k)
            };
            (shift + to / HAND_REACH, tip * turn)
        } else {
            (shift, turn)
        };
        // The swings were tuned with the fist two thirds of a block ahead; further out they
        // move as far to keep the same sweep on screen.
        Self::hand_camera(eye, yaw, pitch) * Mat4::from_translation(fist + shift * HAND_REACH) * Mat4::from_quat(turn) * rest * Mat4::from_translation(-grip)
    }

    /// The right arm gripping the held item placed by `first_person_item`: the fist on the
    /// handle (a tool's lower corner, under a picture or a block), the arm reaching in from
    /// the lower right.
    pub fn first_person_arm_holding(&self, def: &MobDef, item: Mat4, model: HandModel, eye: Vec3, yaw: f32, pitch: f32, sky: f32, blk: f32) -> Bones {
        let mut b = Bones { light: [sky, blk, 0.0, 0.0], ..Default::default() };
        for m in b.m.iter_mut() {
            *m = [[0.0f32; 4]; 4];
        }
        let cam = Self::hand_camera(eye, yaw, pitch);
        let (_, grip, _) = hand_rest(model);
        // In camera space; the item's own turn decides which way the knuckles face.
        let local = cam.inverse() * item;
        let fist = local.transform_point3(grip);
        // The shoulder keeps its place behind the fist (down, right and back), so the arm
        // follows a swing instead of stretching across the view.
        let shoulder = fist + Vec3::new(0.46, -0.66, 0.41);
        let along = local.transform_vector3(Vec3::new(1.0, 1.0, 0.0)).normalize();
        b.m[R_ARM] = arm_reaching(def, R_ARM, cam, fist, shoulder, along);
        b
    }

    /// A bow in first person, drawn to `stage` (0..3, see bowdraw.rs). At rest it stands in
    /// the right fist at the lower right, the limb out ahead to the left and the string towards
    /// the player; brought up (`bow_aim`) the left hand holds it out ahead, canted, and the
    /// right pulls the string back towards the eye, the arrow pointing along the view.
    pub fn first_person_bow(&self, shape: &crate::bowdraw::Shape, stage: usize, eye: Vec3, yaw: f32, pitch: f32) -> BowHold {
        let cam = Self::hand_camera(eye, yaw, pitch);
        let a = self.bow_aim.clamp(0.0, 1.0);
        let a = a * a * (3.0 - 2.0 * a);
        let flat = |v: glam::Vec2| Vec3::new(v.x, v.y, 0.0);
        let along = flat(shape.top - shape.bottom).normalize();
        let belly = flat(shape.belly);
        let w = self.amount.min(1.0);
        let sway = Vec3::new(self.walk.sin() * 0.02 * w, -self.walk.cos().abs() * 0.025 * w, 0.0);
        let rest_fist = Vec3::new(0.6, -0.5, -1.05) + sway + Vec3::Y * -(1.0 - self.hand_height.clamp(0.0, 1.0)) * 0.6;
        let rest = lay_bow(along, belly, Vec3::new(-0.18, 1.0, 0.1), Vec3::new(-1.0, 0.05, -0.65));
        // Held at full draw the arms tire and the bow trembles a little.
        let shake = if stage >= 3 { Vec3::new((self.time * 37.0).sin(), (self.time * 29.0).cos(), 0.0) * 0.004 } else { Vec3::ZERO };
        // Drawn, the bow stays in the right fist and stands just below the middle of the view,
        // its top canted left and its face half turned to the eye; the left hand brings the
        // string back to the lower left, so the arrow runs up from there to the middle.
        let drawn_grip = Vec3::new(-0.06, -0.26, -1.45) + sway * 0.4 + shake;
        let drawn = lay_bow(along, belly, Vec3::new(-0.4, 0.92, 0.0), Vec3::new(0.5, 0.3, -0.81));
        let q = rest.slerp(drawn, a);
        let grip = rest_fist.lerp(drawn_grip, a);
        // The bow is a little bigger drawn, as it comes up close to the eye.
        let size = 0.7 + 0.15 * a;
        let local = Mat4::from_translation(grip) * Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(size)) * Mat4::from_translation(-flat(shape.grip));
        let nock = local.transform_point3(flat(shape.nock(stage)));
        let up = q * along;
        let arrow = (stage > 0 && a > 0.4).then(|| {
            // Lying on the bow's face, its nock on the string, as long as a full draw and a
            // hand's width more.
            let dir = q * belly;
            let face = q * Vec3::Z;
            cam * arrow_on(nock + face * 0.012, dir, face, (shape.grip - shape.nock(3)).dot(shape.belly) * size + 0.2)
        });
        BowHold { bow: cam * local, arrow, right: grip, left: grip.lerp(nock, a), aim: a, up }
    }

    /// A gun in first person (guns.rs): at rest it sits low at the right, angled up at the
    /// crosshair; raised (`GunState::aim`) it comes in towards the middle of the view, a long
    /// gun's stock against the shoulder and the left hand on its foregrip. Each shot throws it
    /// back and the muzzle up.
    pub fn first_person_gun(&self, i: usize, st: &crate::guns::GunState, eye: Vec3, yaw: f32, pitch: f32) -> GunHold {
        use crate::guns;
        let cam = Self::hand_camera(eye, yaw, pitch);
        let fr = guns::frame(i);
        let two = guns::two_handed(i);
        let a = st.aim.clamp(0.0, 1.0);
        let a = a * a * (3.0 - 2.0 * a);
        let w = self.amount.min(1.0);
        let sway = Vec3::new(self.walk.sin() * 0.02 * w, -self.walk.cos().abs() * 0.025 * w, 0.0) * (1.0 - 0.6 * a);
        let drop = Vec3::Y * -(1.0 - self.hand_height.clamp(0.0, 1.0)) * 0.6;
        let rest = if two { Vec3::new(0.4, -0.5, -0.9) } else { Vec3::new(0.46, -0.5, -1.0) };
        let raised = if two { Vec3::new(0.2, -0.36, -0.78) } else { Vec3::new(0.16, -0.34, -1.08) };
        let fist = rest.lerp(raised, a) + sway + drop;
        // The barrel points at a spot far down the crosshair, so the shot seems to come out of it.
        let bore = fist + Vec3::Y * fr.muzzle.y * guns::PX;
        let dir = (Vec3::new(0.0, 0.0, -40.0) - bore).normalize();
        // At rest it is canted in a little; raised, level.
        let up = (Quat::from_axis_angle(dir, 0.16 * (1.0 - a)) * Vec3::Y).normalize();
        let up = (up - dir * up.dot(dir)).normalize();
        let q = Quat::from_mat3(&glam::Mat3::from_cols(up.cross(dir), up, dir));
        let r = st.recoil(i);
        let kick = Mat4::from_translation(Vec3::new(0.0, 0.0, -2.0 * r)) * Mat4::from_rotation_x(-0.28 * r) * Mat4::from_rotation_y(st.kick_side * 0.06 * r);
        let local = Mat4::from_translation(fist) * Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(guns::PX)) * kick;
        let left = fr.fore.filter(|_| two).map(|f| local.transform_point3(f));
        let along = local.transform_vector3(Vec3::new(0.0, -1.0, -0.35)).normalize();
        let barrel = local.transform_vector3(Vec3::Z).normalize();
        GunHold { gun: cam * local, muzzle: (cam * local).transform_point3(fr.muzzle), right: local.transform_point3(Vec3::new(0.0, -1.6, -0.6)), left, along, barrel }
    }

    /// The arms on a gun placed by `first_person_gun`.
    pub fn first_person_gun_arms(&self, def: &MobDef, hold: &GunHold, eye: Vec3, yaw: f32, pitch: f32, sky: f32, blk: f32) -> Bones {
        let mut b = Bones { light: [sky, blk, 0.0, 0.0], ..Default::default() };
        for m in b.m.iter_mut() {
            *m = [[0.0f32; 4]; 4];
        }
        let cam = Self::hand_camera(eye, yaw, pitch);
        let shoulder = hold.right + Vec3::new(0.46, -0.66, 0.41);
        b.m[R_ARM] = arm_reaching(def, R_ARM, cam, hold.right, shoulder, hold.along);
        if let Some(left) = hold.left {
            b.m[L_ARM] = arm_reaching(def, L_ARM, cam, left, Vec3::new(-0.7, -0.9, -0.45), hold.barrel);
        }
        b
    }

    /// Both arms on a bow placed by `first_person_bow`: the right fist always on the grip, the
    /// left on the string once the bow is raised.
    pub fn first_person_bow_arms(&self, def: &MobDef, hold: &BowHold, eye: Vec3, yaw: f32, pitch: f32, sky: f32, blk: f32) -> Bones {
        let mut b = Bones { light: [sky, blk, 0.0, 0.0], ..Default::default() };
        for m in b.m.iter_mut() {
            *m = [[0.0f32; 4]; 4];
        }
        let cam = Self::hand_camera(eye, yaw, pitch);
        let a = hold.aim;
        // Raised, the bow arm reaches out ahead from the right shoulder.
        let shoulder = (hold.right + Vec3::new(0.46, -0.66, 0.41)).lerp(Vec3::new(0.95, -0.75, -0.9), a);
        b.m[R_ARM] = arm_reaching(def, R_ARM, cam, hold.right, shoulder, hold.up);
        if a > 0.02 {
            // The left arm comes up from below the view to the string, its elbow out to the side.
            let fist = hold.left + Vec3::new(-0.15, -0.65, 0.25) * (1.0 - a);
            b.m[L_ARM] = arm_reaching(def, L_ARM, cam, fist, Vec3::new(-0.7, -0.95, -0.45), hold.up);
        }
        b
    }

    pub fn first_person_arm(&self, def: &MobDef, eye: Vec3, yaw: f32, pitch: f32, sky: f32, blk: f32) -> Bones {
        let mut b = Bones { light: [sky, blk, 0.0, 0.0], ..Default::default() };
        let zero = [[0.0f32; 4]; 4];
        for m in b.m.iter_mut() {
            *m = zero;
        }
        let cam = Self::camera(eye, yaw, pitch);
        let place = self.arm_place();
        let arm_model = |part: usize, place: Mat4| {
            let p = &def.parts[part];
            let pivot = Vec3::new(p.pivot.0 as f32, p.pivot.1 as f32, p.pivot.2 as f32);
            // Model space has +Z forward; turn it so the arm points into the screen (-Z).
            (cam * place * Mat4::from_rotation_y(PI) * Mat4::from_scale(Vec3::splat(1.0 / 16.0)) * Mat4::from_translation(-pivot)).to_cols_array_2d()
        };
        if self.mantle > 0.0 {
            // Climbing a ledge: both hands grip the edge in front and stay on it while the view
            // rises past, pressing down until they drop out of sight below. Placed level, not
            // tipped with the look, since the hands hold onto the world.
            let flat = Mat4::from_translation(eye) * Mat4::from_rotation_y(yaw);
            let (f, over) = mantle_reach(self.mantle);
            let grab = (self.mantle / voxel::physics::MANTLE_GRAB).min(1.0);
            // On the edge while it is in sight; once the view has risen past it the hands keep
            // pressing at the bottom of the view, then slide down out of it while swinging over.
            let ledge = self.ledge - voxel::physics::EYE_HEIGHT;
            let up = ledge.clamp(-0.32, 0.1) - over * 0.6;
            // While the edge is still above the view the hands show on the wall's face below
            // it, stretched up; from there on they rest on top.
            let below = ((ledge - up) / 0.15).clamp(0.0, 1.0);
            let ahead = (f.max(0.0) + 0.22) * (1.0 - below) + 0.27 * below;
            for (part, side) in [(R_ARM, 1.0f32), (L_ARM, -1.0f32)] {
                // Reaching out from below at first, then on the edge.
                let hand = Vec3::new(side * 0.27, up - (1.0 - grab) * 0.35, -ahead);
                let shoulder = Vec3::new(side * 0.42, -0.62, -0.1);
                let d = (hand - shoulder).normalize();
                let place = Mat4::from_translation(hand - d * 0.62) * Mat4::from_quat(Quat::from_rotation_arc(Vec3::NEG_Y, d));
                b.m[part] = arm_model(part, cam.inverse() * flat * place);
            }
            return b;
        }
        let w = self.underwater;
        if w > 0.05 && self.swing <= 0.0 {
            // Under water: both arms do the breaststroke. Reach ahead together, sweep out and
            // back, then fold in under the chin and reach again.
            let ph = self.stroke.fract() * 2.0 * PI;
            let sweep = (1.0 - ph.cos()) * 0.5; // 0 ahead, 1 fully out
            let pull = ph.sin().max(0.0); // pulling back during the outward half
            let fold = (-ph.sin()).max(0.0); // elbows in on the way back
            for (part, side) in [(R_ARM, 1.0f32), (L_ARM, -1.0f32)] {
                let shoulder = Vec3::new(side * (0.16 + sweep * 0.14), -0.31 - fold * 0.08 + (1.0 - w) * -0.5, -0.30 + pull * 0.12 + fold * 0.1);
                let place = Mat4::from_translation(shoulder)
                    * Mat4::from_rotation_y(-side * (sweep * 0.55 - 0.1))
                    * Mat4::from_rotation_x(1.62 - fold * 0.35 - pull * 0.15)
                    * Mat4::from_rotation_z(side * 0.1);
                b.m[part] = arm_model(part, place);
            }
            return b;
        }
        b.m[R_ARM] = arm_model(R_ARM, place);
        b
    }
}

/// A gun held in first person (`Animator::first_person_gun`).
pub struct GunHold {
    /// Gun space (model pixels) to the camera's block, scaled.
    pub gun: Mat4,
    /// The muzzle, relative to the camera's block.
    pub muzzle: Vec3,
    /// Fists in camera space, which way the grip runs and the barrel points.
    right: Vec3,
    left: Option<Vec3>,
    along: Vec3,
    barrel: Vec3,
}

/// A bow held in first person (`Animator::first_person_bow`).
pub struct BowHold {
    /// The bow picture's frame (one block wide before its scale), relative to the camera's block.
    pub bow: Mat4,
    /// The nocked arrow's picture frame while the string is drawn.
    pub arrow: Option<Mat4>,
    /// Fists in camera space, the bow's 0..1 rise and its string's direction (for the knuckles).
    right: Vec3,
    left: Vec3,
    aim: f32,
    up: Vec3,
}

/// The arrow picture's nock end and its length from there to the point (the picture's
/// diagonal, one block wide before its scale).
const ARROW_NOCK: Vec3 = Vec3::new(-0.4, -0.4, 0.0);
const ARROW_SPAN: f32 = 1.13;

/// The frame of an arrow picture with its nock at `nock`, flying along `dir`, its face towards
/// `face`, `length` long.
pub(crate) fn arrow_on(nock: Vec3, dir: Vec3, face: Vec3, length: f32) -> Mat4 {
    let turn = lay_bow(Vec3::new(1.0, 1.0, 0.0), Vec3::new(-1.0, 1.0, 0.0), dir, face.cross(dir));
    Mat4::from_translation(nock) * Mat4::from_quat(turn) * Mat4::from_scale(Vec3::splat(length / ARROW_SPAN)) * Mat4::from_translation(-ARROW_NOCK)
}

/// The turn that lays a picture's `along` and `across` (both in its plane) onto `to` and
/// towards `toward`.
pub(crate) fn lay_bow(along: Vec3, across: Vec3, to: Vec3, toward: Vec3) -> Quat {
    let along = along.normalize();
    let across = (across - along * across.dot(along)).normalize();
    let to = to.normalize();
    let toward = (toward - to * toward.dot(to)).normalize();
    let from = glam::Mat3::from_cols(along, across, along.cross(across));
    let onto = glam::Mat3::from_cols(to, toward, to.cross(toward));
    Quat::from_mat3(&(onto * from.transpose()))
}

/// The first-person arm `part` with its fist at `fist` (camera space), reaching from
/// `shoulder`, its palm facing the way `along` runs (a handle, a bow's grip).
fn arm_reaching(def: &MobDef, part: usize, cam: Mat4, fist: Vec3, shoulder: Vec3, along: Vec3) -> [[f32; 4]; 4] {
    let d = (fist - shoulder).normalize();
    // The palm wraps the handle: the arm's front (+z in the model, -z after the half turn)
    // faces the way the handle runs.
    let x = along.cross(d).normalize_or_zero();
    let z = d.cross(x);
    let turn = if x == Vec3::ZERO { Quat::from_rotation_arc(Vec3::NEG_Y, d) } else { Quat::from_mat3(&glam::Mat3::from_cols(x, -d, z)) };
    let p = &def.parts[part];
    let pivot = Vec3::new(p.pivot.0 as f32, p.pivot.1 as f32, p.pivot.2 as f32);
    let fist_px = Vec3::new(0.0, -8.5, 0.0);
    let place = Mat4::from_translation(fist) * Mat4::from_quat(turn) * Mat4::from_rotation_y(PI) * Mat4::from_scale(Vec3::splat(1.0 / 16.0)) * Mat4::from_translation(-fist_px - pivot);
    (cam * place).to_cols_array_2d()
}

/// How much smaller the first-person hand is drawn, towards the eye (`hand_camera`).
const HAND_NEAR: f32 = 0.25;
/// How much further ahead the fist is than the two thirds of a block the swing offsets were
/// written for.
const HAND_REACH: f32 = 1.6;

/// A held item at rest in first person: its turn and scale (from the item model's centre
/// frame), the point of it the fist holds (in that frame, one block wide), and where the fist
/// is in camera space.
fn hand_rest(model: HandModel) -> (Mat4, Vec3, Vec3) {
    // The turn that lays the picture's `along` and its face (+z) onto `to` and `facing`.
    let lay = |along: Vec3, to: Vec3, facing: Vec3| {
        let to = to.normalize();
        let n = (facing - to * facing.dot(to)).normalize();
        let from = glam::Mat3::from_cols(along, Vec3::Z.cross(along), Vec3::Z);
        let onto = glam::Mat3::from_cols(to, n.cross(to), n);
        Quat::from_mat3(&(onto * from.transpose()))
    };
    match model {
        // The blade (the picture's diagonal) rises up, left and away; its face turned to
        // the left so the edge shows.
        HandModel::Tool => {
            let q = lay(Vec3::new(1.0, 1.0, 0.0).normalize(), Vec3::new(-0.25, 0.8, -0.55), Vec3::new(-1.0, 0.15, 0.3));
            (Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(0.8)), Vec3::new(-0.3, -0.3, 0.0), Vec3::new(0.62, -0.52, -1.05))
        }
        // A pickaxe, axe, shovel or hoe: the handle rises a little to the right with the head
        // across the top, its face towards the eye, like the classic hand, not pointing ahead
        // like a blade.
        HandModel::Digger => {
            let q = lay(Vec3::new(1.0, 1.0, 0.0).normalize(), Vec3::new(0.3, 0.9, 0.12), Vec3::new(-0.4, 0.0, 0.9));
            (Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(0.8)), Vec3::new(-0.3, -0.3, 0.0), Vec3::new(0.5, -0.56, -1.05))
        }
        // A drill or chainsaw: the picture's diagonal (grip to bit) points ahead and a little
        // up and in, its side turned to the eye so the motor and the bit both show.
        HandModel::Powered => {
            let q = lay(Vec3::new(1.0, 1.0, 0.0).normalize(), Vec3::new(-0.3, 0.42, -1.0), Vec3::new(-0.6, 0.25, 0.75));
            (Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(0.85)), Vec3::new(-0.1, -0.38, 0.0), Vec3::new(0.52, -0.5, -1.0))
        }
        // A picture stands up in the hand, turned a little towards the middle.
        HandModel::Flat => {
            let q = lay(Vec3::Y, Vec3::new(-0.1, 1.0, -0.15), Vec3::new(-0.55, 0.15, 0.8));
            (Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(0.46)), Vec3::new(0.0, -0.4, 0.0), Vec3::new(0.58, -0.55, -1.1))
        }
        // A block sits on the fist, turned 45° and tipped towards the eye.
        HandModel::Block => {
            let q = Quat::from_rotation_x(0.25) * Quat::from_rotation_y(f32::to_radians(45.0));
            (Mat4::from_quat(q) * Mat4::from_scale(Vec3::splat(0.32)), Vec3::new(0.0, -0.62, 0.1), Vec3::new(0.6, -0.6, -1.2))
        }
    }
}

/// How far ahead of the body the ledge's edge is (blocks, negative once the body has swung
/// past it), and 0..1 through the swing over the top.
fn mantle_reach(u: f32) -> (f32, f32) {
    use voxel::physics::MANTLE_PULL;
    let x = ((u - MANTLE_PULL) / (1.0 - MANTLE_PULL)).clamp(0.0, 1.0);
    let over = x * x * (3.0 - 2.0 * x);
    (0.4 - 0.8 * over, over)
}

/// The arm's forward swing (radians from hanging down) to reach a hand `ahead` blocks in front
/// and `up` blocks above the shoulder.
fn mantle_arm(ahead: f32, up: f32) -> f32 {
    ahead.atan2(-up)
}

impl Animator {
    /// First-person view bobbing while walking on the ground, like the classic option.
    pub fn view_bob(&self, yaw: f32, on_ground: bool) -> Vec3 {
        if !on_ground {
            return Vec3::ZERO;
        }
        let a = self.amount.min(1.2);
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        right * self.walk.sin() * 0.035 * a + Vec3::Y * -(self.walk.cos().abs()) * 0.05 * a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_item_drops_on_change_and_rises_when_ready() {
        let mut a = Animator::default();
        for _ in 0..20 {
            a.follow_hand(Some(1), 1.0, 0.05);
        }
        assert!((a.hand_height - 1.0).abs() < 1e-4);
        // Another item: down first, then back up.
        a.follow_hand(Some(2), 1.0, 0.05);
        assert!(a.hand_height < 1.0);
        for _ in 0..20 {
            a.follow_hand(Some(2), 1.0, 0.05);
        }
        assert!((a.hand_height - 1.0).abs() < 1e-4);
        // Not ready after a hit: it hangs low.
        for _ in 0..20 {
            a.follow_hand(Some(2), 0.2, 0.05);
        }
        assert!(a.hand_height < 0.05);
    }

    #[test]
    fn drawn_bow_pulls_the_string_back_towards_the_eye() {
        let shape = crate::bowdraw::Shape::fallback();
        let a = Animator { hand_height: 1.0, bow_aim: 1.0, ..Default::default() };
        let full = a.first_person_bow(&shape, 3, Vec3::ZERO, 0.0, 0.0);
        let half = a.first_person_bow(&shape, 1, Vec3::ZERO, 0.0, 0.0);
        // The string hand comes back towards the eye the further it is drawn, behind the grip.
        assert!(full.left.z > half.left.z && half.left.z > full.right.z);
        // The bow stays in the right fist, which is on the right; the string hand on the left.
        assert!(full.right.x > full.left.x);
        // The arrow sits on the string and points away from the eye, past the grip.
        let arrow = full.arrow.expect("arrow on the string");
        // (Drawn towards the eye like the other held items: undo that to compare with the fists.)
        let (nock, tip) = (arrow.transform_point3(ARROW_NOCK) / HAND_NEAR, arrow.transform_point3(-ARROW_NOCK) / HAND_NEAR);
        assert!(nock.distance(full.left) < 0.05 && tip.z < full.right.z);
        // At rest nothing is nocked and the right hand holds the grip.
        let rest = Animator { hand_height: 1.0, ..Default::default() }.first_person_bow(&shape, 0, Vec3::ZERO, 0.0, 0.0);
        assert!(rest.arrow.is_none() && rest.right.distance(rest.left) < 1e-4);
    }

    #[test]
    fn sword_rests_lower_right_with_the_fist_on_the_handle() {
        let a = Animator { hand_height: 1.0, ..Default::default() };
        let m = a.first_person_item(HandModel::Tool, Vec3::ZERO, 0.0, 0.0);
        let (_, grip, fist) = hand_rest(HandModel::Tool);
        let fist = fist * HAND_NEAR;
        assert!(m.transform_point3(grip).distance(fist) < 1e-4);
        // Drawn near the eye, so the tip stays out of a wall the player is against.
        assert!(m.transform_point3(Vec3::new(0.45, 0.45, 0.0)).z > -0.45);
        // The tip is above, left of and further away than the handle.
        let tip = m.transform_point3(Vec3::new(0.45, 0.45, 0.0));
        assert!(tip.y > fist.y && tip.x < fist.x && tip.z < fist.z);
    }

    #[test]
    fn pickaxe_rests_head_up_facing_the_eye() {
        let a = Animator { hand_height: 1.0, ..Default::default() };
        let m = a.first_person_item(HandModel::Digger, Vec3::ZERO, 0.0, 0.0);
        let (_, grip, _) = hand_rest(HandModel::Digger);
        let fist = m.transform_point3(grip);
        // The head is above the fist, not ahead of it, and its face turns to the eye.
        let head = m.transform_point3(Vec3::new(0.45, 0.45, 0.0));
        let up = head - fist;
        assert!(up.y > 0.0 && up.y > -up.z * 3.0);
        assert!(m.transform_vector3(Vec3::Z).normalize().z > 0.7);
    }
}

