#[macro_use]
extern crate bitflags;

#[windows_subsystem = "windows"]
use bevy::prelude::*;
use fc_core::input::*;
use std::collections::HashMap;

mod data;
#[cfg(debug_assertions)]
mod debug;
mod input;
mod r#match;
mod time;

use r#match::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppState {
    STARTUP,
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
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "Fantasy Crescendo".to_string(),
        vsync: true,
        ..Default::default()
    })
    .add_state(AppState::STARTUP)
    .add_plugins(DefaultPlugins)
    .add_plugin(input::FcInputPlugin)
    .add_plugin(data::FcAssetsPlugin)
    .add_plugin(r#match::FcMatchPlugin)
    .insert_resource(Msaa { samples: 1 })
    .insert_resource(MatchConfig {
        rule: r#match::rule::MatchRule::Stock(3),
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
    .add_startup_system(setup.system());

    // Optional Plugins
    #[cfg(debug_assertions)]
    app.add_plugin(debug::FcDebugPlugin);

    app.run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // cameras
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        perspective_projection: bevy::render::camera::PerspectiveProjection {
            fov: 60.0 * std::f32::consts::PI / 180.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
