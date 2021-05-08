use super::player::Player;
use bevy::prelude::*;
use fc_core::input::*;

pub(super) fn sample_input(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&InputSource, &mut PlayerInput), With<Player>>,
) {
    for (mapping, mut player_input) in query.iter_mut() {
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
    }
}
