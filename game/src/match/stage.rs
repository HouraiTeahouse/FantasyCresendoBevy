use super::on_match_update;
use super::player::{PlayerDamage, PlayerLocation, PlayerBody};
use bevy::{math::*, prelude::*};
use fc_core::{
    geo::Bounds2D,
    player::{Facing, Player},
    stage::{BlastZone, RespawnPoint, SpawnPoint},
};

pub(super) struct PlayerDied {
    pub revive: bool,
    pub player: Player,
    pub damage: PlayerDamage,
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
    mut respawn_points: Query<&mut RespawnPoint>,
    mut players: Query<(&mut PlayerDamage, &mut PlayerBody, &Transform, &Player)>,
    mut died: EventWriter<PlayerDied>,
) {
    let mut respawn_points: Vec<Mut<RespawnPoint>> = respawn_points.iter_mut().filter(|p| p.occupied_by.is_none()).collect();
    let bounds: Vec<&Bounds2D> = blast_zones.iter().map(|bz| &bz.0).collect();
    for (mut damage, mut body, transform, player) in players.iter_mut() {
        let position = transform.translation.xy();
        if damage.is_alive() && !bounds.iter().any(|bounds| bounds.contains_point(position)) {
            damage.kill();
            let revive = damage.can_revive();
            if revive {
                damage.revive();
                let mut respawn_point = respawn_points.pop().expect("Player died and no available respawn points!");
                respawn_point.occupied_by = Some(player.clone());
                body.location = PlayerLocation::Airborne(respawn_point.position);
                body.facing = respawn_point.facing;
            }
            info!("Player {} died: {:?}", player.id, damage);
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
        .add_event::<PlayerDied>()
        .add_startup_system(setup_stage.system())
        .add_system_set(on_match_update().with_system(kill_players.system()));
}
