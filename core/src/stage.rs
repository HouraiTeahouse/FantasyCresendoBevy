use crate::geo::Bounds2D;
use crate::player::{Facing, Player};
use bevy::{math::Vec2, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, TypeUuid)]
#[uuid = "c0176bef-fe0f-4384-ae04-c9efa9a1918c"]
pub struct StageAsset {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct BlastZone(pub Bounds2D);

#[derive(Debug, Clone)]
pub struct SpawnPoint {
    pub position: Vec2,
    pub facing: Facing,
}

#[derive(Debug, Clone)]
pub struct RespawnPoint {
    pub position: Vec2,
    pub facing: Facing,
    pub occupied_by: Option<Player>,
}
