use bevy_reflect::TypeUuid;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub mod frame_data;
pub mod state;

pub struct AssetReference<T> {
    _data: PhantomData<T>,
}

#[derive(Deserialize, Debug, TypeUuid)]
#[uuid = "230e1b7b-5d32-4159-91c1-45e162e7b3fc"]
pub struct CharacterAsset {
    pub short_name: String,
    pub long_name: String,
    pub palletes: Vec<CharacterPallette>,
}

#[derive(Debug, Deserialize)]
pub struct CharacterPallette {}
