use serde::{Serialize, Deserialize};
use bevy_input::gamepad::Gamepad;

#[derive(Debug)]
pub struct MatchConfig {
    pub players: [PlayerConfig; MAX_PLAYERS_PER_MATCH],
}
