use bevy::prelude::*;
use fc_core::{input::PlayerInput, input::*, player::Player};
use std::collections::HashMap;

mod input;

fn create_players(mut commands: Commands) {
    let mut buttons: HashMap<Buttons, Vec<KeyCode>> = HashMap::new();
    buttons.insert(Buttons::ATTACK, vec![KeyCode::F]);
    buttons.insert(Buttons::SPECIAL, vec![KeyCode::D]);
    buttons.insert(Buttons::JUMP, vec![KeyCode::I, KeyCode::A, KeyCode::S]);
    buttons.insert(Buttons::SHIELD, vec![KeyCode::Q, KeyCode::W]);
    commands
        .spawn()
        .insert(Player { id: 0 })
        .insert(PlayerInput::default())
        .insert(InputSource::Keyboard {
            movement: ButtonAxis2D::<KeyCode> {
                horizontal: ButtonAxis1D::<KeyCode> {
                    pos: KeyCode::I,
                    neg: KeyCode::K,
                },
                vertical: ButtonAxis1D::<KeyCode> {
                    pos: KeyCode::H,
                    neg: KeyCode::L,
                },
            },
            smash: ButtonAxis2D::<KeyCode> {
                horizontal: ButtonAxis1D::<KeyCode> {
                    pos: KeyCode::I,
                    neg: KeyCode::K,
                },
                vertical: ButtonAxis1D::<KeyCode> {
                    pos: KeyCode::H,
                    neg: KeyCode::L,
                },
            },
            buttons: ButtonMapping::<KeyCode>(buttons),
        });
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::FcInputPlugin)
        .add_startup_system(create_players.system())
        .run();
}
