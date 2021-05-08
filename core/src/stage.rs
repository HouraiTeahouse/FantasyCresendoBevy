use bevy_reflect::TypeUuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, TypeUuid)]
#[uuid = "c0176bef-fe0f-4384-ae04-c9efa9a1918c"]
pub struct StageAsset {
    pub name: String,
}
