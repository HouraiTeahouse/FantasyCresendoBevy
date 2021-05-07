use bevy::prelude::*;
use fc_core::input::*;
use std::collections::HashMap;

mod input;
mod r#match;

use r#match::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppState {
    MATCH,
}

fn create_input_source(arrow: ButtonAxis2D<KeyCode>) -> InputSource {
    let mut buttons: HashMap<Buttons, Vec<KeyCode>> = HashMap::new();
    buttons.insert(Buttons::ATTACK, vec![KeyCode::F]);
    buttons.insert(Buttons::SPECIAL, vec![KeyCode::D]);
    buttons.insert(Buttons::JUMP, vec![KeyCode::I, KeyCode::A, KeyCode::S]);
    buttons.insert(Buttons::SHIELD, vec![KeyCode::Q, KeyCode::W]);
    InputSource::Keyboard {
        movement: arrow.clone(),
        smash: arrow.clone(),
        buttons: ButtonMapping::<KeyCode>(buttons.clone()),
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::FcInputPlugin)
        .add_plugin(r#match::FcMatchPlugin)
        .add_state(AppState::MATCH)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(MatchConfig {
            rule: MatchRule::Stock(3),
            time: None,
            players: [
                Some(player::PlayerConfig {
                    character_id: 0,
                    pallete: 0,
                    default_damage: 0.0,
                    input: create_input_source(ButtonAxis2D::<KeyCode> {
                        horizontal: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::D,
                            neg: KeyCode::A,
                        },
                        vertical: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::W,
                            neg: KeyCode::S,
                        },
                    }),
                }),
                Some(player::PlayerConfig {
                    character_id: 0,
                    pallete: 0,
                    default_damage: 0.0,
                    input: create_input_source(ButtonAxis2D::<KeyCode> {
                        horizontal: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::L,
                            neg: KeyCode::J,
                        },
                        vertical: ButtonAxis1D::<KeyCode> {
                            pos: KeyCode::I,
                            neg: KeyCode::K,
                        },
                    }),
                }),
                None,
                None,
            ],
        })
        .add_startup_system(setup.system())
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
        mesh: meshes.add(Mesh::from(shape::Plane { size: 500.0 })),
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
        transform: Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
