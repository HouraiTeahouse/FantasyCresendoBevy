use bevy::prelude::*;
use fc_core::{geo::*, player::Facing};

bitflags! {
    pub struct PhysicsGroups : u16 {
        const PLAYER = 1 << 0;
        const STAGE = 1 << 1;
        const HITBOX = 1 << 2;
    }
}

#[derive(Debug)]
pub enum Location {
    Airborne(Vec2),
    Respawning(Entity),
    Surface { surface: Entity, position: f32 },
}

impl Default for Location {
    fn default() -> Self {
        Self::Airborne(Vec2::ZERO)
    }
}

#[derive(Debug, Default)]
pub struct Body {
    pub mass: f32,
    pub facing: Facing,
    pub location: Location,
    pub velocity: Vec2,
    pub ecb: EnvironmentCollisionBox,
}

#[derive(Debug, Default, Clone)]
pub struct EnvironmentCollisionBox(pub Bounds2D);

impl EnvironmentCollisionBox {
    pub fn top(&self) -> Vec2 {
        self.0.center + Vec2::new(0.0, self.0.extents.y)
    }

    pub fn bottom(&self) -> Vec2 {
        self.0.center - Vec2::new(0.0, self.0.extents.y)
    }

    pub fn left(&self) -> Vec2 {
        self.0.center - Vec2::new(self.0.extents.x, 0.0)
    }

    pub fn right(&self) -> Vec2 {
        self.0.center + Vec2::new(self.0.extents.x, 0.0)
    }

    pub fn translate(&mut self, delta: Vec2) {
        self.0.center += delta;
    }

    pub fn segments(&self) -> [LineSegment2D; 4] {
        [
            LineSegment2D::new(self.bottom(), self.left()),
            LineSegment2D::new(self.left(), self.top()),
            LineSegment2D::new(self.top(), self.right()),
            LineSegment2D::new(self.right(), self.bottom()),
        ]
    }
}

impl From<EnvironmentCollisionBox> for Bounds2D {
    fn from(value: EnvironmentCollisionBox) -> Self {
        value.0.clone()
    }
}
