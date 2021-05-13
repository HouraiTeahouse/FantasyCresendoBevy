use super::on_match_update;
use super::player::{Player, PlayerDamage};
use bevy::{math::*, prelude::*};
use fc_core::{geo::Bounds2D, stage::BlastZone};

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
}

fn kill_players(
    blast_zones: Query<&BlastZone>,
    mut players: Query<(&mut PlayerDamage, &GlobalTransform, &Player)>,
    mut died: EventWriter<PlayerDied>,
) {
    let bounds: Vec<&Bounds2D> = blast_zones.iter().map(|bz| &bz.0).collect();
    for (mut damage, transform, player) in players.iter_mut() {
        let position = transform.translation.xy();
        if damage.is_alive() && !bounds.iter().any(|bounds| bounds.contains_point(position)) {
            damage.kill();
            let revive = damage.can_revive();
            if revive {
                damage.revive();
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
