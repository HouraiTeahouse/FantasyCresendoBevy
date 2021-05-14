use crate::geo::*;
use crate::player::{Facing, Player};
use bevy::{math::Vec2, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, TypeUuid)]
#[uuid = "c0176bef-fe0f-4384-ae04-c9efa9a1918c"]
pub struct StageAsset {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct BlastZone(pub Bounds2D);

#[derive(Debug, Clone)]
pub struct SpawnPoint {
    pub position: Vec2,
    pub facing: Facing,
}

#[derive(Debug, Clone)]
pub struct RespawnPoint {
    pub position: Vec2,
    pub facing: Facing,
    pub occupied_by: Option<Player>,
}

#[derive(Debug, Clone, Default)]
pub struct Surface {
    pub start: SurfacePoint,
    pub end: SurfacePoint,
    pub flags: SurfaceFlags,
}

impl Surface {
    pub fn new(start: impl Into<SurfacePoint>, end: impl Into<SurfacePoint>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
            ..Default::default()
        }
    }

    pub fn ceiling(start: impl Into<SurfacePoint>, end: impl Into<SurfacePoint>) -> Self {
        Self {
            flags: SurfaceFlags::CEILING,
            ..Self::new(start, end)
        }
    }

    pub fn floor(start: impl Into<SurfacePoint>, end: impl Into<SurfacePoint>) -> Self {
        Self {
            flags: SurfaceFlags::FLOOR,
            ..Self::new(start, end)
        }
    }

    pub fn wall(start: impl Into<SurfacePoint>, end: impl Into<SurfacePoint>) -> Self {
        Self {
            flags: SurfaceFlags::WALL,
            ..Self::new(start, end)
        }
    }

    /// Gets a reference to the left most facing point on the surface.
    pub fn left(&self) -> &SurfacePoint {
        if self.start.point.x < self.end.point.x {
            &self.start
        } else {
            &self.end
        }
    }

    /// Gets a reference to the right most facing point on the surface.
    pub fn right(&self) -> &SurfacePoint {
        if self.start.point.x < self.end.point.x {
            &self.end
        } else {
            &self.start
        }
    }

    pub fn is_wall(&self) -> bool {
        self.flags.contains(SurfaceFlags::WALL)
    }

    pub fn is_ceiling(&self) -> bool {
        self.flags.contains(SurfaceFlags::CEILING)
    }

    pub fn is_floor(&self) -> bool {
        self.flags.contains(SurfaceFlags::FLOOR)
    }

    pub fn as_segment(&self) -> LineSegment2D {
        LineSegment2D::new(self.start.point, self.end.point)
    }

    /// Computes a position along the surface.
    /// If position is not in the range [0, 1], it will use the full 2D line projection.
    pub fn lerp(&self, position: f32) -> Vec2 {
        Vec2::lerp(self.start.point, self.end.point, position)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SurfacePoint {
    pub point: Vec2,
    pub grabbable: bool,
}

impl From<Vec2> for SurfacePoint {
    fn from(point: Vec2) -> Self {
        Self {
            point,
            ..Default::default()
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct SurfaceFlags : u8 {
        const FLOOR = 1 << 0;
        const CEILING = 1 << 1;
        const WALL = 1 << 2;
        const PASSTHROUGH = 1 << 3;
    }
}
