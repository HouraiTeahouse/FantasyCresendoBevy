use bevy::prelude::*;
use fc_core::{input::*, player::Player};

pub(super) fn sample_input(
    keyboard: Res<Input<KeyCode>>,
    mut players: Query<(&InputSource, &mut PlayerInput), With<Player>>,
) {
    players.for_each_mut(|(mapping, mut player_input)| {
        player_input.tick();
        match mapping {
            InputSource::None | InputSource::CPU => {}
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
            InputSource::Gamepad { .. } => {}
        }
    });
}
