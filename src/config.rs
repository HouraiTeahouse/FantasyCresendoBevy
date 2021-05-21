use bevy::prelude::Color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameConfig {
    pub player: PlayerConfig,
}

#[derive(Deserialize)]
pub struct PlayerConfig {
    pub colors: Vec<Color>,
}