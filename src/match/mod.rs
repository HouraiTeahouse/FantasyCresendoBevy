use self::{player::*, stage::*};
use crate::{
    character::{frame_data::*, state::*},
    geo::*,
    player::Player,
    time::DELTA_TIME,
    AppState,
};
use bevy::{
    core::FixedTimestep,
    math::*,
    prelude::*,
    render::camera::{Camera, PerspectiveProjection},
    window::Windows,
};
use serde::{Deserialize, Serialize};

pub mod events;
pub mod hitbox;
pub mod input;
pub mod physics;
pub mod player;
pub mod rule;
pub mod stage;

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
        for (idx, player) in config.players.iter().enumerate() {
            if player.is_some() {
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
) {
    assert!(config.validate().is_ok());

    // Clear any prior match results
    *result = MatchResult::from_config(&config);

    let mut state = MatchState {
        time_remaining: config.time.clone(),
        ..Default::default()
    };
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
                body: physics::Body {
                    ecb: physics::EnvironmentCollisionBox(Bounds2D {
                        center: Vec2::new(0.0, 0.5),
                        extents: Vec2::new(0.25, 0.5),
                    }),
                    location: physics::Location::Airborne(transform.translation.xy()),
                    gravity: 1.0,
                    ..Default::default()
                },
                movement: PlayerMovement {
                    jump_power: vec![2.5, 1.5],
                    short_jump_power: 0.9,
                    max_fall_speed: 2.0,
                    fast_fall_speed: 5.0,
                    ..Default::default()
                },
                transform,
                input_source: cfg.input.clone(),
                // pbr: PbrBundle {
                //     mesh: mesh.clone(),
                //     material: materials.add(player::get_player_color(id as PlayerId).into()),
                //     transform,
                //     ..Default::default()
                // },
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

fn sample_frames(mut players: Query<(&mut CharacterFrame, &mut PlayerState, &StateMachine)>) {
    players.for_each_mut(|(mut frame, mut state, state_machine)| {
        state.tick();
        if let Some(sampled) = state_machine.sample_frame(&state) {
            *frame = sampled.clone();
        }
    });
}

fn update_camera(
    mut cameras: Query<(&mut Transform, &Camera, &PerspectiveProjection)>,
    players: Query<(&GlobalTransform, &physics::Body), With<Player>>,
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
    cameras.for_each_mut(|(mut transform, camera, projection)| {
        if let Some(window) = windows.get(camera.window) {
            let (w_width, w_height) = (window.physical_width(), window.physical_height());
            if w_height == 0 {
                return;
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
    });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct MatchUpdateStage;

const MATCH_UPDATE_LABEL: &str = "MATCH_UPDATE";

pub struct FcMatchPlugin;

impl Plugin for FcMatchPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .insert_resource(MatchConfig::default())
            .insert_resource(MatchResult::default())
            .add_system_set(SystemSet::on_enter(AppState::MATCH).with_system(init_match.system()))
            .add_system_set(SystemSet::on_exit(AppState::MATCH).with_system(cleanup_match.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MATCH).with_system(update_camera.system()),
            )
            .add_stage_after(
                CoreStage::Update,
                MatchUpdateStage,
                // INVARIANT: SystemStage::single_threaded **must** follow system insertion order.
                SystemStage::single_threaded()
                    .with_run_criteria(
                        FixedTimestep::step(DELTA_TIME.into()).with_label(MATCH_UPDATE_LABEL),
                    )
                    // Update inputs
                    .with_system(input::sample_input.system().label("SAMPLE_INPUT"))
                    // Run physics updates
                    .with_system(
                        physics::move_players
                            .system()
                            .label("MOVE_PLAYERS")
                            .after("SAMPLE_INPUT"),
                    )
                    .with_system(
                        physics::update_bodies
                            .system()
                            .label("UPDATE_BODIES")
                            .after("MOVE_PLAYERS"),
                    )
                    // Update animations
                    .with_system(
                        sample_frames
                            .system()
                            .label("SAMPLE_FRAMES")
                            .after("UPDATE_BODIES"),
                    )
                    // Updated hitboxes and players
                    .with_system(
                        hitbox::update_hitboxes
                            .system()
                            .label("UPDATE_HITBOXES")
                            .after("SAMPLE_FRAMES"),
                    )
                    .with_system(
                        hitbox::collide_hitboxes
                            .system()
                            .label("COLLIDE_HITBOXES")
                            .after("UPDATE_HITBOXES"),
                    )
                    .with_system(
                        hitbox::hit_players
                            .system()
                            .label("HIT_PLAYERS")
                            .after("COLLIDE_HITBOXES"),
                    )
                    .with_system(
                        stage::kill_players
                            .system()
                            .label("KILL PLAYERS")
                            .after("HIT_PLAYERS"),
                    )
                    // Evaluate the match state
                    .with_system(
                        rule::update_match_state
                            .system()
                            .label("UPDATE_MATCH_STATE")
                            .after("KILL_PLAYERS"),
                    )
                    .with_system(
                        rule::on_player_died
                            .system()
                            .label("ON_PLAYER_DIED")
                            .after("UPDATE_MATCH_STATE"),
                    )
                    .with_system(
                        rule::finish_match
                            .system()
                            .label("FINISH_MATCH")
                            .after("ON_PLAYER_DIED"),
                    ),
            );
        stage::build(builder);
        hitbox::build(builder);
        events::build(builder);
    }
}
