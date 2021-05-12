use self::player::*;
use crate::AppState;
use bevy::{prelude::*, render::camera::Camera};
use bevy_rapier3d::{
    na::Vector3, physics::RigidBodyHandleComponent, rapier::dynamics::RigidBodySet,
};
use fc_core::{
    character::{frame_data::*, state::*},
    input::*,
};
use serde::{Deserialize, Serialize};

mod hitbox;
mod input;
mod physics;
pub mod player;

pub const MAX_PLAYERS_PER_MATCH: usize = 4;

#[derive(Clone, Deserialize, Serialize)]
pub enum MatchRule {
    Score,
    Stamina(f32),
    Stock(u8),
}

impl Default for MatchRule {
    fn default() -> Self {
        Self::Score
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct MatchConfig {
    pub rule: MatchRule,
    /// Optional max time for the match. If not none, the
    /// match will prematurely end if time reaches zero.
    pub time: Option<u32>,
    pub players: [Option<PlayerConfig>; MAX_PLAYERS_PER_MATCH],
}

impl MatchConfig {
    pub fn active_player_count(&self) -> usize {
        self.players.iter().flatten().count()
    }

    pub fn validate(&self) -> Result<(), MatchConfigValidationError> {
        if self.active_player_count() < 2 {
            Err(MatchConfigValidationError::NotEnoughPlayers)
        } else {
            Ok(())
        }
    }
}

pub enum MatchConfigValidationError {
    NotEnoughPlayers,
}

#[derive(Debug, Default)]
pub struct MatchState {
    /// The number of frames remaining before the end of the match.
    /// If None, the match has no set time limit.
    pub time_remaining: Option<u32>,
    pub players: [Option<Entity>; MAX_PLAYERS_PER_MATCH],
}

#[derive(Debug, Default)]
pub struct MatchResult {
    pub players: [Option<PlayerResult>; MAX_PLAYERS_PER_MATCH],
}

#[derive(Debug)]
pub struct PlayerResult {}

fn init_match(
    config: Res<MatchConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    assert!(config.validate().is_ok());
    let mut state = MatchState {
        time_remaining: config.time.clone(),
        ..Default::default()
    };
    let mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.5,
        depth: 1.0,
        ..Default::default()
    }));
    for (id, player_config) in config.players.iter().enumerate() {
        state.players[id] = player_config.as_ref().map(|cfg| {
            info!("Spawning player {}", id);
            player::spawn_player(
                &mut commands,
                player::PlayerBundle {
                    player: Player { id: id as u8 },
                    damage: player::PlayerDamage::new(&config.rule, &cfg),
                    input_source: cfg.input.clone(),
                    ecb: player::EnvironmentCollisionBox {
                        left: 0.25,
                        right: 0.25,
                        top: 0.5,
                        bottom: 0.5,
                    },
                    pbr: PbrBundle {
                        mesh: mesh.clone(),
                        material: materials.add(player::get_player_color(id as PlayerId).into()),
                        transform: Transform::from_xyz(id as f32 * 2.0 - 4.0, 0.5, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        });
    }
    commands.insert_resource(state);
}

fn cleanup_match(state: Res<MatchState>, mut commands: Commands) {
    // Despawn players
    for player in state.players.iter() {
        if let Some(entity) = player {
            commands.entity(*entity).despawn_recursive();
        }
    }
    commands.remove_resource::<MatchState>();
}

fn update_match_state(mut state: ResMut<MatchState>) {
    if let Some(ref mut time) = state.time_remaining {
        if *time > 0 {
            *time -= 1;
        }
    }
}

fn sample_frames(mut query: Query<(&mut CharacterFrame, &mut PlayerState, &StateMachine)>) {
    for (mut frame, mut state, state_machine) in query.iter_mut() {
        state.tick();
        if let Some(sampled) = state_machine.sample_frame(&state) {
            *frame = sampled.clone();
        }
    }
}

fn move_players(
    query: Query<(&RigidBodyHandleComponent, &PlayerInput)>,
    mut rigidbodies: ResMut<RigidBodySet>,
) {
    for (component, input) in query.iter() {
        if let Some(rb) = rigidbodies.get_mut(component.handle()) {
            let movement = &input.current.movement;
            let mut transform = *rb.position();
            transform.translation.x += f32::from(movement.x) * 0.1;
            transform.translation.y += f32::from(movement.y) * 0.1;
            rb.set_position(transform, true);
        }
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, With<Camera>>,
    players: Query<&GlobalTransform, With<Player>>,
) {
    // TODO(james7132): Make this movement more smooth
    let mut position = Vec2::ZERO;
    let mut count: u16 = 0;
    for transform in players.iter() {
        position += Vec2::from(transform.translation);
        count += 1;
    }

    if count == 0 {
        return;
    }
    let average_pos = position / f32::from(count);

    for mut transform in camera.iter_mut() {
        transform.translation.x = average_pos.x;
        transform.translation.y = average_pos.y;
    }
}

pub struct FcMatchPlugin;

impl Plugin for FcMatchPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .insert_resource(MatchConfig::default())
            .insert_resource(MatchResult::default())
            .add_system_set(SystemSet::on_enter(AppState::MATCH).with_system(init_match.system()))
            .add_system_set(SystemSet::on_exit(AppState::MATCH).with_system(cleanup_match.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MATCH)
                    .with_system(move_players.system())
                    .with_system(sample_frames.system())
                    .with_system(input::sample_input.system())
                    .with_system(hitbox::update_hitboxes.system())
                    .with_system(update_match_state.system())
                    .with_system(update_camera.system()),
            );
    }
}
