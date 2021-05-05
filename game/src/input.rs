use bevy::prelude::*;
use fc_core::{input::*, player::Player};
use std::collections::HashSet;

#[derive(Default)]
struct GamepadLobby(HashSet<Gamepad>);

fn gamepad_connection_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.0.insert(*gamepad);
                info!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.0.remove(gamepad);
                info!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}

fn sample_input(
    lobby: Res<GamepadLobby>,
    keyboard: Res<Input<KeyCode>>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    gamepad_axes: Res<Input<KeyCode>>,
    mut query: Query<(&InputSource, &mut PlayerInput), With<Player>>,
) {
    for (mapping, mut player_input) in query.iter_mut() {
        player_input.tick();
        match mapping {
            InputSource::None => {}
            InputSource::Keyboard {
                movement,
                smash,
                buttons,
            } => {
                player_input.current = PlayerInputFrame {
                    movement: movement.sample(&keyboard),
                    smash: smash.sample(&keyboard),
                    buttons: buttons.evaluate_all(&keyboard),
                };
            }
            InputSource::Gamepad { buttons } => {}
        }
        if player_input.previous != player_input.current {
            info!("{:?}", player_input);
        }
    }
}

pub struct FcInputPlugin;

impl Plugin for FcInputPlugin {
    fn build(&self, builder: &mut bevy::prelude::AppBuilder) {
        builder
            .init_resource::<GamepadLobby>()
            .add_system(gamepad_connection_system.system())
            .add_system(sample_input.system());
    }
}
