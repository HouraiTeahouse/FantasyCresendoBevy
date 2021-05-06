use crate::player::PlayerId;
use bevy_math::Vec3;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct HitboxFlags : u8 {
        const MIRROR_DIRECTION = 1 << 0;
        const TRASCENDENT_PRIORITY = 1 << 1;
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Hitbox {
    pub flags: HitboxFlags,
    pub radius: f32,
    pub priority: u16,
    pub damage: ScalableValue,
    // Knockback angle of the hitbox in radians.
    pub knockback_angle: f32,
    pub knockback_force: ScalableValue,
    pub hitstun: ScalableValue,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ScalableValue {
    pub base: f32,
    pub growth: f32,
}

impl ScalableValue {
    pub fn evaluate(&self, scaling: f32) -> f32 {
        self.base + scaling * self.growth
    }
}

#[derive(Clone, Debug)]
pub struct HitboxState {
    pub id: u8,
    pub player: PlayerId,
    pub enabled: bool,
    pub previous_position: Option<Vec3>,
}
