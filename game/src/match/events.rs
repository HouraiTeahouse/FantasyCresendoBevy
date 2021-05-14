use super::player::PlayerDamage;
use bevy::prelude::*;
use fc_core::player::Player;

pub(super) struct PlayerDied {
    pub revive: bool,
    pub player: Player,
    pub damage: PlayerDamage,
}

pub fn build(builder: &mut AppBuilder) {
    builder.add_event::<PlayerDied>();
}
