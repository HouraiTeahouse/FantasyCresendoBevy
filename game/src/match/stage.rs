use super::{
    events::PlayerDied,
    on_match_update,
    physics::{Body, Location},
    player::PlayerDamage,
};
use crate::time::FrameTimer;
use bevy::{ecs::system::SystemParam, math::*, prelude::*};
use fc_core::{
    geo::{Bounds2D, LineSegment2D},
    player::{Facing, Player},
    stage::{BlastZone, RespawnPoint, SpawnPoint, Surface},
};

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
fn kill_players(
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
    for (mut damage, mut body, transform, player) in players.iter_mut() {
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
    }
}

pub(super) fn build(builder: &mut AppBuilder) {
    builder
        .add_startup_system(setup_stage.system())
        .add_system_set(on_match_update().with_system(kill_players.system()));
}
