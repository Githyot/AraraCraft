//! Camera feel (thread "Analisar e melhorar a jogabilidade", plan/jogabilidade-analise.md):
//! small, cheap touches that make walking around feel smooth instead of mechanical.
//!
//! - Stepping up onto a slab or a block (the 0.6-block auto step in physics.rs) used to snap
//!   the eye up in one frame. Now the eye trails the body and catches up in about a tenth of
//!   a second, like stairs feel in other games.
//! - Landing from a jump or a fall dips the view a little on a spring, harder for longer
//!   falls, so the ground has weight. "Reduzir movimento" turns the dip off.

use glam::Vec3;

use crate::Game;

/// How far below the body the eye may trail after steps (a few slabs in a row).
const MAX_LAG: f32 = 0.9;
/// Catch-up speed of the trailing eye (per second, exponential).
const LAG_RATE: f32 = 16.0;
/// Landing spring: stiffness and damping (a little under critical, one soft bounce).
const STIFF: f32 = 140.0;
const DAMP: f32 = 18.0;
/// Deepest dip, in blocks.
const MAX_DIP: f32 = 0.22;

/// Dip speed given to the camera by landing at `fall_speed` (blocks per second, positive
/// down). A plain jump lands at about 9: a soft nod. A 10-block fall lands at about 25.
pub(crate) fn landing_kick(fall_speed: f32) -> f32 {
    if fall_speed < 5.0 {
        return 0.0;
    }
    (fall_speed - 4.0).min(30.0) * 0.085
}

/// One step of the landing spring: (offset, speed) after `dt`.
pub(crate) fn spring(dip: f32, v: f32, dt: f32) -> (f32, f32) {
    // Small sub-steps keep it stable at low frame rates.
    let n = (dt / 0.008).ceil().clamp(1.0, 16.0) as usize;
    let h = dt / n as f32;
    let (mut x, mut v) = (dip, v);
    for _ in 0..n {
        let a = -STIFF * x - DAMP * v;
        v += a * h;
        x += v * h;
    }
    if x.abs() < 1e-4 && v.abs() < 1e-3 {
        return (0.0, 0.0);
    }
    (x.clamp(-MAX_DIP, MAX_DIP), v)
}

impl Game {
    /// After the player's body moved this frame: start the eye trailing a step up, or the
    /// landing dip, and ease both back.
    pub(crate) fn camera_feel(&mut self, before: Vec3, was_ground: bool, vy_before: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let rise = self.body.pos.y - before.y;
        let walking = was_ground && self.body.on_ground && !self.body.flying && self.body.mantle.is_none();
        if walking && rise > 0.05 && rise < 0.65 {
            // The body popped up a step: the eye stays where it was and follows.
            self.eye_lag = (self.eye_lag - rise).max(-MAX_LAG);
        } else if rise.abs() > 1.5 {
            // Teleports, respawns, portals: no trailing across the jump.
            self.eye_lag = 0.0;
            self.dip = 0.0;
            self.dip_v = 0.0;
        }
        self.eye_lag *= (-LAG_RATE * dt).exp();
        if self.eye_lag.abs() < 1e-3 {
            self.eye_lag = 0.0;
        }
        let landed = !was_ground && self.body.on_ground && !self.body.in_water && !self.body.flying;
        if landed && !self.settings.reduce_motion {
            self.dip_v -= landing_kick(-vy_before);
        }
        let (d, v) = spring(self.dip, self.dip_v, dt);
        self.dip = d;
        self.dip_v = v;
    }

    /// "Pulo automático": walking on the ground into a step one block high (too tall for the
    /// 0.6 auto step) with room above it jumps by itself. Never while sneaking, crawling,
    /// swimming, flying or climbing a ladder.
    pub(crate) fn auto_jump_wanted(&self, wish: Vec3) -> bool {
        if !self.settings.auto_jump || !self.body.on_ground || self.body.flying || self.body.in_water || self.body.mantle.is_some() || self.body.height < 1.0 || self.crouch > 0.1 {
            return false;
        }
        let flat = Vec3::new(wish.x, 0.0, wish.z);
        if flat.length_squared() < 0.25 {
            return false;
        }
        let dir = flat.normalize();
        let feet = self.body.pos;
        let probe = feet + dir * (self.body.half_width + 0.35);
        let p = probe.floor().as_ivec3();
        let foot = (feet.y + 0.05).floor() as i32;
        let w = &self.world;
        let solid = |q: glam::IVec3| voxel::physics::Terrain::solid(w, q);
        let at = glam::IVec3::new(p.x, foot, p.z);
        if !solid(at) || voxel::physics::Terrain::climbable(w, at) {
            return false;
        }
        // Taller than a slab step (the walk handles those), not over a full block above the
        // feet, and room for the body on top.
        let top = at.y as f32 + voxel::physics::Terrain::top(w, at);
        let rise = top - feet.y;
        rise > 0.65 && rise <= 1.05 && !solid(at + glam::IVec3::Y) && !solid(at + glam::IVec3::Y * 2) && !solid((feet + glam::Vec3::Y * 2.2).floor().as_ivec3())
    }

    /// Height added to the eye by the camera feel (negative is lower).
    pub(crate) fn eye_offset(&self) -> f32 {
        self.eye_lag + self.dip.min(0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_jump_dips_less_than_a_fall() {
        assert_eq!(landing_kick(3.0), 0.0);
        assert!(landing_kick(9.0) < landing_kick(25.0));
    }

    #[test]
    fn the_spring_settles_back() {
        let (mut x, mut v) = (0.0, -landing_kick(25.0));
        let mut low: f32 = 0.0;
        for _ in 0..120 {
            (x, v) = spring(x, v, 1.0 / 60.0);
            low = low.min(x);
        }
        assert!(low < -0.05 && low >= -MAX_DIP, "dip {low}");
        assert_eq!((x, v), (0.0, 0.0));
    }

    #[test]
    fn low_frame_rates_stay_stable() {
        let (mut x, mut v) = (0.0, -2.0);
        for _ in 0..30 {
            (x, v) = spring(x, v, 0.1);
            assert!(x.abs() <= MAX_DIP);
        }
        assert!(x.abs() < 0.01);
    }
}
