use super::ScalableValue;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering, PartialOrd};

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct HitboxFlags : u8 {
        const MIRROR_DIRECTION = 1 << 0;
        const PROJECTILE = 1 << 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitboxPriority {
    Transcendent,
    Normal(u16),
}

impl Default for HitboxPriority {
    fn default() -> Self {
        Self::Normal(0)
    }
}

impl Ord for HitboxPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Transcendent, Self::Transcendent) => Ordering::Equal,
            (Self::Transcendent, Self::Normal(_)) => Ordering::Greater,
            (Self::Normal(_), Self::Transcendent) => Ordering::Less,
            (Self::Normal(a), Self::Normal(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for HitboxPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Hitbox {
    pub flags: HitboxFlags,
    pub radius: f32,
    pub priority: HitboxPriority,
    pub damage: f32,
    // Knockback angle of the hitbox in radians.
    pub knockback_angle: f32,
    pub knockback_force: ScalableValue,
    pub hitstun: ScalableValue,
}
