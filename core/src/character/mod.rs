use bevy_reflect::TypeUuid;
use serde::{Deserialize, Serialize};

pub mod frame_data;
pub mod state;

#[derive(Serialize, Deserialize, Debug, TypeUuid)]
#[uuid = "230e1b7b-5d32-4159-91c1-45e162e7b3fc"]
pub struct CharacterAsset {
    pub short_name: String,
    pub long_name: String,
    pub palletes: Vec<CharacterPallette>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterPallette {}
