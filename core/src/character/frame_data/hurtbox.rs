use super::{
    hitbox::{Hitbox, HitboxFlags},
    ScalableValue,
};
use crate::{geo::Capsule3D, player::PlayerId};
use bevy::{render::color::Color, transform::components::GlobalTransform};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HurtboxType {
    Inactive,
    Damageable,
    Intangible,
    Invincible,
    Grazing,
    Shield,
}

impl HurtboxType {
    pub fn color(&self) -> Color {
        match self {
            Self::Inactive => Color::GRAY,
            Self::Damageable => Color::YELLOW,
            Self::Intangible => Color::BLUE,
            Self::Invincible => Color::GREEN,
            Self::Grazing => Color::PURPLE,
            Self::Shield => Color::PINK,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hurtbox {
    pub id: u8,
    pub player: PlayerId,
    pub r#type: HurtboxType,
    pub collider: Capsule3D,
    // A multiplier for incoming damage.
    pub damage_multiplier: f32,
    // Additional knockback added to the player.
    pub knockback_force: ScalableValue,
}

impl Hurtbox {
    pub fn is_enabled(&self) -> bool {
        self.r#type != HurtboxType::Inactive
    }

    pub fn world_collider(&self, transform: &GlobalTransform) -> Capsule3D {
        let trs = transform.compute_matrix();
        Capsule3D {
            start: trs.transform_point3(self.collider.start),
            end: trs.transform_point3(self.collider.end),
            radius: transform.scale.max_element() * self.collider.radius,
        }
    }

    /// Checks if the hurtbox is available for collision.
    pub fn allows_collision(&self, hitbox: &Hitbox) -> bool {
        match self.r#type {
            HurtboxType::Inactive | HurtboxType::Intangible => false,
            HurtboxType::Grazing => !hitbox.flags.contains(HitboxFlags::PROJECTILE),
            _ => true,
        }
    }
}
