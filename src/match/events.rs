use super::player::{Player, PlayerDamage};
use bevy::prelude::*;

pub(super) struct PlayerDied {
    pub revive: bool,
    pub player: Player,
    pub damage: PlayerDamage,
}

pub fn build(builder: &mut AppBuilder) {
    builder.add_event::<PlayerDied>();
}
