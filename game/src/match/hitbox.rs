use super::MatchState;
use bevy::prelude::*;
use fc_core::character::frame_data::*;
use fc_core::player::{Player, PlayerId};

#[derive(Clone, Debug, Default)]
pub struct HitboxState {
    pub id: u8,
    pub player: PlayerId,
    pub enabled: bool,
    pub previous_position: Option<Vec3>,
}

#[derive(Bundle, Default)]
pub(super) struct HitboxBundle {
    pub hitbox: hitbox::Hitbox,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub state: HitboxState,
}

pub(super) fn create_player_hitboxes(player: PlayerId) -> impl Iterator<Item = HitboxBundle> {
    (0..CHARACTER_HITBOX_COUNT as u8)
        .into_iter()
        .map(move |id| HitboxBundle {
            state: HitboxState {
                id,
                player,
                ..Default::default()
            },
            ..Default::default()
        })
}

pub fn update_hitboxes(
    match_state: Res<MatchState>,
    players: Query<&CharacterFrame, With<Player>>,
    mut hitboxes: Query<&mut HitboxState>,
) {
    for mut state in hitboxes.iter_mut() {
        state.enabled = match_state.players[state.player as usize]
            .and_then(|entity| players.get(entity).ok())
            .map(|frame| (frame.active_hitboxes & (1 << state.id)) != 0)
            .unwrap_or(false);
    }
}
