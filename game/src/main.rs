use bevy::prelude::*;
use fc_core::{
    character::{frame_data::*, state::*},
    input::PlayerInput,
    input::*,
    player::*,
};
use std::collections::HashMap;

mod input;

#[derive(Bundle)]
struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub input: PlayerInput,
    pub damage: PlayerDamage,
    pub state: PlayerState,
    pub state_machine: StateMachine,
    pub frame: CharacterFrame,
    pub input_source: InputSource,
}

#[derive(Bundle)]
struct HitboxBundle {
    pub hitbox: hitbox::Hitbox,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub state: hitbox::HitboxState,
}

pub struct Players(Vec<Entity>);

fn create_players(
    mut players: ResMut<Players>, 
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut buttons: HashMap<Buttons, Vec<KeyCode>> = HashMap::new();
    buttons.insert(Buttons::ATTACK, vec![KeyCode::F]);
    buttons.insert(Buttons::SPECIAL, vec![KeyCode::D]);
    buttons.insert(Buttons::JUMP, vec![KeyCode::I, KeyCode::A, KeyCode::S]);
    buttons.insert(Buttons::SHIELD, vec![KeyCode::Q, KeyCode::W]);
    players.0.clear();
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    for player_id in 0..4 {
        players.0.push(
            commands
                .spawn_bundle(PlayerBundle {
                    player: Player { id: player_id },
                    transform: Transform::default(),
                    global_transform: GlobalTransform::default(),
                    input: PlayerInput::default(),
                    damage: PlayerDamage::Stock {
                        stocks: 3,
                        damage: 0.0,
                    },
                    state: PlayerState {
                        state_id: 0,
                        frame: 0,
                    },
                    frame: CharacterFrame::default(),
                    state_machine: StateMachine::default(),
                    input_source: InputSource::Keyboard {
                        movement: ButtonAxis2D::<KeyCode> {
                            horizontal: ButtonAxis1D::<KeyCode> {
                                pos: KeyCode::L,
                                neg: KeyCode::J,
                            },
                            vertical: ButtonAxis1D::<KeyCode> {
                                pos: KeyCode::I,
                                neg: KeyCode::K,
                            },
                        },
                        smash: ButtonAxis2D::<KeyCode> {
                            horizontal: ButtonAxis1D::<KeyCode> {
                                pos: KeyCode::L,
                                neg: KeyCode::J,
                            },
                            vertical: ButtonAxis1D::<KeyCode> {
                                pos: KeyCode::I,
                                neg: KeyCode::K,
                            },
                        },
                        buttons: ButtonMapping::<KeyCode>(buttons.clone()),
                    },
                })
                .insert_bundle(PbrBundle {
                    mesh: mesh.clone(), 
                    material: material.clone(),
                    transform: Transform::from_xyz(player_id as f32 * 2.0 - 4.0, 0.5, 0.0),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for id in 0..CHARACTER_HITBOX_COUNT {
                        parent.spawn_bundle(HitboxBundle {
                            hitbox: hitbox::Hitbox::default(),
                            transform: Transform::default(),
                            global_transform: GlobalTransform::default(),
                            state: hitbox::HitboxState {
                                id: id as u8,
                                player: player_id,
                                enabled: false,
                                previous_position: None,
                            },
                        });
                    }
                })
                .id(),
        );
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

fn update_hitboxes(
    player_map: Res<Players>,
    players: Query<&CharacterFrame, With<Player>>,
    mut hitboxes: Query<&mut hitbox::HitboxState>,
) {
    for mut state in hitboxes.iter_mut() {
        state.enabled = player_map
            .0
            .get(state.player as usize)
            .and_then(|entity| players.get(*entity).ok())
            .map(|frame| (frame.active_hitboxes & (1 << state.id)) != 0)
            .unwrap_or(false);
    }
}

fn move_players(mut query: Query<(&mut Transform, &PlayerInput)>) {
    for (mut transform, input) in query.iter_mut() {
        transform.translation += Vec3::from(input.current.movement) * 0.001;
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::FcInputPlugin)
        .insert_resource(Players(Vec::new()))
        .insert_resource(Msaa { samples: 1 })
        .add_startup_system(create_players.system())
        .add_startup_system(setup.system())
        .add_system(sample_frames.system())
        .add_system(update_hitboxes.system())
        .add_system(move_players.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
