use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Default)]
pub struct GamepadLobby(HashSet<Gamepad>);

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

pub struct FcInputPlugin;

impl Plugin for FcInputPlugin {
    fn build(&self, builder: &mut bevy::prelude::AppBuilder) {
        builder
            .init_resource::<GamepadLobby>()
            .add_system(gamepad_connection_system.system());
    }
}
