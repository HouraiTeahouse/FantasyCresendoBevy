use bevy::prelude::*;
use fc_core::{input::PlayerInput, input::*, player::*, character::{frame_data::*, state::*}};
use std::collections::HashMap;

mod input;

#[derive(Bundle)]
struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub input: PlayerInput,
    pub damage: PlayerDamage,
    pub state: PlayerState,
    pub state_machine: StateMachine,
    pub frame: CharacterFrame,
    pub input_source: InputSource,
}

#[derive(Bundle)]
struct HitboxBundle {
    pub hitbox: hitbox::Hitbox,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub state: hitbox::HitboxState,
}

fn create_players(mut commands: Commands) {
    let mut buttons: HashMap<Buttons, Vec<KeyCode>> = HashMap::new();
    buttons.insert(Buttons::ATTACK, vec![KeyCode::F]);
    buttons.insert(Buttons::SPECIAL, vec![KeyCode::D]);
    buttons.insert(Buttons::JUMP, vec![KeyCode::I, KeyCode::A, KeyCode::S]);
    buttons.insert(Buttons::SHIELD, vec![KeyCode::Q, KeyCode::W]);
    for player_id in 0..4 {
        commands.spawn_bundle(PlayerBundle {
            player: Player { id: player_id },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            input: PlayerInput::default(),
            damage: PlayerDamage::Stock {
                stocks: 3,
                damage: 0.0,
            },
            state: PlayerState {
                state_id: 0,
                frame: 0
            },
            frame: CharacterFrame::default(),
            state_machine: StateMachine::default(),
            input_source: InputSource::Keyboard {
                movement: ButtonAxis2D::<KeyCode> {
                    horizontal: ButtonAxis1D::<KeyCode> {
                        pos: KeyCode::I,
                        neg: KeyCode::K,
                    },
                    vertical: ButtonAxis1D::<KeyCode> {
                        pos: KeyCode::H,
                        neg: KeyCode::L,
                    },
                },
                smash: ButtonAxis2D::<KeyCode> {
                    horizontal: ButtonAxis1D::<KeyCode> {
                        pos: KeyCode::I,
                        neg: KeyCode::K,
                    },
                    vertical: ButtonAxis1D::<KeyCode> {
                        pos: KeyCode::H,
                        neg: KeyCode::L,
                    },
                },
                buttons: ButtonMapping::<KeyCode>(buttons.clone()),
            },
        }).with_children(|parent| {
            for id in 0..CHARACTER_HITBOX_COUNT {
                parent.spawn_bundle(HitboxBundle {
                    hitbox: hitbox::Hitbox::default(),
                    transform: Transform::default(),
                    global_transform: GlobalTransform::default(),
                    state: hitbox::HitboxState {
                        id: id as u8,
                        player: player_id,
                        enabled: false,
                        previous_position: None,
                    },
                });
            }
        });
    }
}

fn sample_frames(mut query: Query<(&mut CharacterFrame, &mut PlayerState, &StateMachine)>) {
    for (mut frame, mut state, state_machine) in query.iter_mut() {
        state.tick();
        if let Some(sampled) = state_machine.sample_frame(&state) {
            *frame = sampled.clone();
        } 
    }
}

fn update_hitboxes(
    players: Query<(&Player, &CharacterFrame)>,
    mut query: Query<&mut hitbox::HitboxState>) {
    let mut active: HashMap<PlayerId, HitboxActiveBitflag> = HashMap::new();
    for (player, frame) in players.iter() {
        active.insert(player.id, frame.active_hitboxes);
    }
    for mut state in query.iter_mut() {
        let active_mask = active.get(&state.player).cloned().unwrap_or(0);
        state.enabled = (active_mask & (1 << state.id)) != 0;
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::FcInputPlugin)
        .add_startup_system(create_players.system())
        .add_system(sample_frames.system())
        .add_system(update_hitboxes.system())
        .run();
}
