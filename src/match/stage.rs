use super::{
    events::PlayerDied,
    physics::{Body, Facing, Location},
    player::{Player, PlayerDamage},
};
use crate::{
    geo::{Bounds2D, LineSegment2D},
    time::FrameTimer,
};
use bevy::{ecs::system::SystemParam, math::*, prelude::*, reflect::TypeUuid};
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

    /// Checks if one of the ends of the surface is one of ends.
    pub fn has_end(&self, point: Vec2) -> bool {
        self.start.point == point || self.end.point == point
    }

    /// Gets the other end of the surface, if available.
    /// If the provided point is not either end, returns None.
    pub fn other(&self, point: Vec2) -> Option<&SurfacePoint> {
        if point == self.start.point {
            Some(&self.end)
        } else if point == self.end.point {
            Some(&self.start)
        } else {
            None
        }
    }

    /// Gets the total change in X across the surface.
    pub fn delta_x(&self) -> f32 {
        (self.end.point.x - self.start.point.x).abs()
    }

    /// Gets the total change in Y across the surface.
    pub fn delta_y(&self) -> f32 {
        (self.end.point.y - self.start.point.y).abs()
    }

    pub fn contains_x(&self, x: f32) -> bool {
        x >= self.left().point.x && x <= self.right().point.x
    }

    pub fn contains_y(&self, y: f32) -> bool {
        y >= self.left().point.y && y <= self.right().point.y
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

// TODO(james7132): Make this a game config option.
const MAX_RESPAWN_FRAMES: u16 = 300;

#[derive(SystemParam)]
pub struct StageContext<'a> {
    pub surfaces: Query<'a, (Entity, &'static Surface)>,
    pub respawn_points: Query<'a, &'static mut RespawnPoint>,
}

impl<'a> StageContext<'a> {
    pub fn surface(&self, entity: Entity) -> &Surface {
        self.surfaces.get(entity).expect("Missing surface.").1
    }

    pub fn respawn_point(&mut self, entity: Entity) -> Mut<RespawnPoint> {
        self.respawn_points
            .get_mut(entity)
            .expect("Missing respawn point.")
    }

    /// Checks if a body's motion intersects with stage geometry.
    pub fn collision_check(&self, movement: LineSegment2D) -> Option<Location> {
        if movement.start.y < movement.end.y {
            return None;
        }

        for (entity, surface) in self.surfaces.iter() {
            let segment = surface.as_segment();
            if movement.intersects(segment) {
                return Some(Location::Surface {
                    surface: entity,
                    position: movement.end.x,
                });
            }
        }
        None
    }
}

fn setup_stage(mut commands: Commands) {
    commands.spawn().insert(BlastZone(Bounds2D {
        center: Vec2::ZERO,
        extents: Vec2::new(10.0, 10.0),
    }));

    // Add spawn points.
    commands.spawn().insert(SpawnPoint {
        position: Vec2::new(-3.0, 0.0),
        facing: Facing::Right,
    });
    commands.spawn().insert(SpawnPoint {
        position: Vec2::new(-1.0, 0.0),
        facing: Facing::Right,
    });
    commands.spawn().insert(SpawnPoint {
        position: Vec2::new(1.0, 0.0),
        facing: Facing::Right,
    });
    commands.spawn().insert(SpawnPoint {
        position: Vec2::new(3.0, 0.0),
        facing: Facing::Right,
    });

    commands
        .spawn()
        .insert(Surface::floor(Vec2::new(-10.0, 0.0), Vec2::new(0.0, -1.0)));
    commands
        .spawn()
        .insert(Surface::floor(Vec2::new(0.0, -1.0), Vec2::new(10.0, 0.0)));
    commands
        .spawn()
        .insert(Surface::floor(Vec2::new(-7.0, 3.0), Vec2::new(-3.0, 3.0)));
    commands
        .spawn()
        .insert(Surface::floor(Vec2::new(3.0, 3.0), Vec2::new(7.0, 3.0)));
    commands
        .spawn()
        .insert(Surface::floor(Vec2::new(-1.5, 6.0), Vec2::new(1.5, 6.0)));

    // Add respawn points.
    commands.spawn().insert(RespawnPoint {
        position: Vec2::new(-6.0, 4.0),
        facing: Facing::Right,
        occupied_by: None,
    });
    commands.spawn().insert(RespawnPoint {
        position: Vec2::new(-2.0, 4.0),
        facing: Facing::Right,
        occupied_by: None,
    });
    commands.spawn().insert(RespawnPoint {
        position: Vec2::new(2.0, 4.0),
        facing: Facing::Right,
        occupied_by: None,
    });
    commands.spawn().insert(RespawnPoint {
        position: Vec2::new(6.0, 4.0),
        facing: Facing::Right,
        occupied_by: None,
    });
}

// TODO(james7132): This is fucknormous, simplify or split this system.
pub(super) fn kill_players(
    blast_zones: Query<&BlastZone>,
    mut respawn_points: Query<(Entity, &mut RespawnPoint)>,
    mut players: Query<(&mut PlayerDamage, &mut Body, &Transform, &Player)>,
    mut died: EventWriter<PlayerDied>,
) {
    let mut respawn_points: Vec<(Entity, Mut<RespawnPoint>)> = respawn_points
        .iter_mut()
        .filter(|p| p.1.occupied_by.is_none())
        .collect();
    let bounds: Vec<&Bounds2D> = blast_zones.iter().map(|bz| &bz.0).collect();
    players.for_each_mut(|(mut damage, mut body, transform, player)| {
        let position = transform.translation.xy();
        if damage.is_alive() && !bounds.iter().any(|bounds| bounds.contains_point(position)) {
            damage.kill();
            let revive = damage.can_revive();
            if revive {
                damage.revive();
                let (respawn_entity, mut respawn_point) = respawn_points
                    .pop()
                    .expect("Player died and no available respawn points!");
                respawn_point.occupied_by = Some(player.clone());
                body.location = Location::Respawning {
                    point: respawn_entity,
                    remaining_time: FrameTimer::new(MAX_RESPAWN_FRAMES),
                };
                body.velocity = Vec2::ZERO;
                body.facing = respawn_point.facing;
            }
            died.send(PlayerDied {
                revive,
                player: player.clone(),
                damage: damage.clone(),
            });
        }
    });
}

pub(super) fn build(builder: &mut AppBuilder) {
    builder.add_startup_system(setup_stage.system());
}
