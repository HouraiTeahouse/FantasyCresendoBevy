use super::{
    on_match_update,
    physics::{Body, StageContext},
    player::PlayerDamage,
    MatchState,
};
use bevy::prelude::*;
use fc_core::{
    character::frame_data::{hitbox::Hitbox, hurtbox::Hurtbox, *},
    geo::Capsule3D,
    player::{Player, PlayerId},
};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct HitboxState {
    pub id: u8,
    pub player: PlayerId,
    pub enabled: bool,
    pub previous_position: Option<Vec3>,
}

impl HitboxState {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.previous_position = None;
        }
    }
}

#[derive(Bundle, Default)]
pub(super) struct HitboxBundle {
    pub hitbox: Hitbox,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub state: HitboxState,
}

#[derive(Clone, Debug)]
pub struct HitCollision {
    pub hitbox: Hitbox,
    pub hitbox_state: HitboxState,
    pub hurtbox: Hurtbox,
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

fn update_hitboxes(
    match_state: Res<MatchState>,
    players: Query<&CharacterFrame, With<Player>>,
    mut hitboxes: Query<&mut HitboxState>,
) {
    for mut state in hitboxes.iter_mut() {
        let id = state.id;
        let player = match_state.players[state.player as usize];
        state.set_enabled(
            player
                .and_then(|entity| players.get(entity).ok())
                .map(|frame| (frame.active_hitboxes & (1 << id)) != 0)
                .unwrap_or(false),
        );
    }
}

fn collide_hitboxes(
    hitboxes: Query<(&Hitbox, &HitboxState, &GlobalTransform)>,
    hurtboxes: Query<(&Hurtbox, &GlobalTransform)>,
    mut hits: EventWriter<HitCollision>,
) {
    for (hitbox, state, hit_transform) in hitboxes.iter() {
        if !state.enabled {
            continue;
        }
        let hit_collider = Capsule3D {
            start: state.previous_position.unwrap_or(hit_transform.translation),
            end: hit_transform.translation,
            radius: hitbox.radius,
        };
        for (hurtbox, hurt_transform) in hurtboxes.iter() {
            if !hurtbox.allows_collision(hitbox) {
                continue;
            }
            // TODO(james7132): Figure out a better way to do this than to
            // recompute the collider for every active hitbox.
            let hurt_collider = hurtbox.world_collider(hurt_transform);
            if hit_collider.intersects(&hurt_collider) {
                hits.send(HitCollision {
                    hitbox: hitbox.clone(),
                    hitbox_state: state.clone(),
                    hurtbox: hurtbox.clone(),
                });
            }
        }
    }
}

const BASE_KNOCKBACK_SCALING: f32 = 0.1;
const IMPACT_KNOCKBACK_SCALING: f32 = 0.05;

fn hit_players(
    mut hits: EventReader<HitCollision>,
    match_state: Res<MatchState>,
    mut players: Query<(&mut PlayerDamage, &mut Body), With<Player>>,
    mut stage: StageContext,
) {
    let mut player_hits: HashMap<PlayerId, HitCollision> = HashMap::new();
    for hit in hits.iter() {
        if let Some(collision) = player_hits.get_mut(&hit.hurtbox.player) {
            if hit.hitbox.priority > collision.hitbox.priority {
                *collision = hit.clone();
            }
        } else {
            player_hits.insert(hit.hurtbox.player, hit.clone());
        }
    }
    for (player_id, hit) in player_hits.iter() {
        let player = match_state.players[*player_id as usize]
            .and_then(|entity| players.get_mut(entity).ok());
        if let Some((mut damage, mut body)) = player {
            let hitbox = &hit.hitbox;
            let hurtbox = &hit.hurtbox;

            let damage_dealt = hitbox.damage * hurtbox.damage_multiplier;
            damage.apply_damage(damage_dealt);
            let damage_launch = (BASE_KNOCKBACK_SCALING + damage_dealt * IMPACT_KNOCKBACK_SCALING)
                * damage.knockback_scaling();
            let scaler = hitbox.knockback_force + hurtbox.knockback_force;
            let knockback_angle = hitbox.knockback_angle;
            let knockback = scaler.evaluate(damage_launch)
                * Vec2::new(libm::cosf(knockback_angle), libm::sinf(knockback_angle));
            body.launch(knockback, &mut stage);
        } else {
            warn!("Registered hit for unknown player ID: {}", player_id)
        }
    }
}

pub(super) fn build(builder: &mut AppBuilder) {
    builder.add_event::<HitCollision>().add_system_set(
        on_match_update()
            .with_system(update_hitboxes.system())
            .with_system(collide_hitboxes.system()),
    );
}
