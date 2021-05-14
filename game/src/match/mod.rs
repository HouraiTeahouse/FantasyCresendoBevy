use self::player::*;
use crate::AppState;
use bevy::{
    math::*,
    prelude::*,
    render::camera::{Camera, PerspectiveProjection},
    window::Windows,
};
use fc_core::{
    character::{frame_data::*, state::*},
    geo::*,
    input::*,
    player::{Player, PlayerId},
    stage::SpawnPoint,
};
use serde::{Deserialize, Serialize};

mod events;
mod hitbox;
mod input;
mod physics;
pub mod player;
pub mod rule;
mod stage;

pub const MAX_PLAYERS_PER_MATCH: usize = 4;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct MatchConfig {
    pub rule: rule::MatchRule,
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
    pub winner: rule::MatchWinner,
    pub players: [Option<PlayerResult>; MAX_PLAYERS_PER_MATCH],
}

impl MatchResult {
    pub fn from_config(config: &MatchConfig) -> Self {
        let mut players: [Option<PlayerResult>; MAX_PLAYERS_PER_MATCH] = Default::default();
        for idx in 0..MAX_PLAYERS_PER_MATCH {
            if config.players[idx].is_some() {
                players[idx] = Some(PlayerResult::default());
            }
        }
        Self {
            winner: rule::MatchWinner::Undecided,
            players,
        }
    }
}

#[derive(Debug, Default)]
pub struct PlayerResult {}

fn init_match(
    config: Res<MatchConfig>,
    spawn_points: Query<&SpawnPoint>,
    mut result: ResMut<MatchResult>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    assert!(config.validate().is_ok());

    // Clear any prior match results
    *result = MatchResult::from_config(&config);

    let mut state = MatchState {
        time_remaining: config.time.clone(),
        ..Default::default()
    };
    let mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.5,
        depth: 1.0,
        ..Default::default()
    }));
    // TODO(jamessliu): This will not work for a new match from a menu.
    // Systems need to be properly ordered to ensure that spawn points are added before players
    // are spawned.
    let mut spawn_points = spawn_points.iter();
    for (id, player_config) in config.players.iter().enumerate() {
        state.players[id] = player_config.as_ref().map(|cfg| {
            info!("Spawning player {}", id);
            let spawn_point = spawn_points
                .next()
                .expect("Stage does not have enough spawn points");
            let transform = Transform::from_translation(Vec3::from((spawn_point.position, 0.0)));
            let bundle = player::PlayerBundle {
                player: Player { id: id as u8 },
                damage: config.rule.create_damage(&cfg),
                body: player::PlayerBody {
                    ecb: player::EnvironmentCollisionBox {
                        left: 0.25,
                        right: 0.25,
                        top: 0.5,
                        bottom: 0.5,
                    },
                    location: player::PlayerLocation::Airborne(transform.translation.xy()),
                    ..Default::default()
                },
                input_source: cfg.input.clone(),
                pbr: PbrBundle {
                    mesh: mesh.clone(),
                    material: materials.add(player::get_player_color(id as PlayerId).into()),
                    transform,
                    ..Default::default()
                },
                ..Default::default()
            };
            player::spawn_player(&mut commands, bundle)
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

fn sample_frames(mut query: Query<(&mut CharacterFrame, &mut PlayerState, &StateMachine)>) {
    for (mut frame, mut state, state_machine) in query.iter_mut() {
        state.tick();
        if let Some(sampled) = state_machine.sample_frame(&state) {
            *frame = sampled.clone();
        }
    }
}

fn move_players(mut players: Query<(&mut PlayerBody, &PlayerInput)>) {
    for (mut body, input) in players.iter_mut() {
        if let PlayerLocation::Airborne(ref mut pos) = body.location {
            let movement = &input.current.movement;
            pos.x += f32::from(movement.x) * 0.1;
            pos.y += f32::from(movement.y) * 0.1;
        }
    }
}

fn update_player_transforms(mut players: Query<(&mut Transform, &PlayerBody)>) {
    for (mut transform, body) in players.iter_mut() {
        match body.location {
            PlayerLocation::Airborne(pos) => transform.translation = Vec3::from((pos, 0.0)),
            _ => {}
        }
    }
}

fn update_camera(
    mut cameras: Query<(&mut Transform, &Camera, &PerspectiveProjection)>,
    players: Query<(&GlobalTransform, &PlayerBody), With<Player>>,
    windows: Res<Windows>,
) {
    let mut total_bounds: Option<Bounds2D> = None;
    for (transform, body) in players.iter() {
        let mut bounds = Bounds2D::from(body.ecb.clone());
        bounds.center += transform.translation.xy();
        if let Some(ref mut total) = total_bounds {
            total.merge_with(bounds);
        } else {
            total_bounds = Some(bounds);
        }
    }

    if total_bounds.is_none() {
        return;
    }

    let total_bounds = total_bounds.unwrap();
    let (width, height) = total_bounds.size().into();
    let center = total_bounds.center;
    if height == 0.0 {
        return;
    }
    let rect_aspect = width / height;
    for (mut transform, camera, projection) in cameras.iter_mut() {
        if let Some(window) = windows.get(camera.window) {
            let (w_width, w_height) = (window.physical_width(), window.physical_height());
            if w_height == 0 {
                continue;
            }
            let aspect_ratio = (w_width as f32) / (w_height as f32);

            // camera distance
            let radius = (height - center.y).max(width - center.x);
            let camera_distance = &mut transform.translation.z;
            *camera_distance = radius / (projection.fov / 2.0).tan();
            if rect_aspect > aspect_ratio {
                if aspect_ratio > 1.0 {
                    *camera_distance /= aspect_ratio;
                } else {
                    *camera_distance *= aspect_ratio;
                }
            } else if width > height {
                *camera_distance /= rect_aspect;
            }
        } else {
            warn!("Cannot find associated window: {}", camera.window);
        }
        transform.translation.x = total_bounds.center.x;
        transform.translation.y = total_bounds.center.y;
    }
}

pub(self) fn on_match_update() -> SystemSet {
    SystemSet::on_update(AppState::MATCH)
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
                on_match_update()
                    .with_system(move_players.system())
                    .with_system(update_player_transforms.system())
                    .with_system(sample_frames.system())
                    .with_system(input::sample_input.system())
                    .with_system(hitbox::update_hitboxes.system())
                    .with_system(update_camera.system()),
            );
        stage::build(builder);
        events::build(builder);
        rule::build(builder);
    }
}
