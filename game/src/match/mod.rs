use self::player::*;
use crate::AppState;
use bevy::prelude::*;
use fc_core::{
    character::{frame_data::*, state::*},
    input::*,
};
use serde::{Deserialize, Serialize};

pub mod hitbox;
mod input;
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

fn move_players(mut query: Query<(&mut Transform, &PlayerInput)>) {
    for (mut transform, input) in query.iter_mut() {
        transform.translation += Vec3::from(input.current.movement) * 0.01;
    }
}

pub struct FcMatchPlugin;

impl Plugin for FcMatchPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .insert_resource(MatchConfig::default())
            .add_system_set(SystemSet::on_enter(AppState::MATCH).with_system(init_match.system()))
            .add_system_set(SystemSet::on_exit(AppState::MATCH).with_system(cleanup_match.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MATCH)
                    .with_system(move_players.system())
                    .with_system(sample_frames.system())
                    .with_system(input::sample_input.system())
                    .with_system(hitbox::update_hitboxes.system())
                    .with_system(update_match_state.system()),
            );
    }
}