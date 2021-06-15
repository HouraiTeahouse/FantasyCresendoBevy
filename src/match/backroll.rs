use super::{
    input::{InputSource, PlayerInputFrame},
    MatchConfig,
};
use crate::time::DELTA_TIME;
use bevy::{core::FixedTimestep, prelude::*};
use bevy_backroll::backroll::PlayerHandle;
use bevy_backroll::*;
use bytemuck::Zeroable;

pub type P2PSession = bevy_backroll::backroll::P2PSession<BackrollConfig>;

pub struct BackrollConfig;

impl backroll::Config for BackrollConfig {
    type Input = PlayerInputFrame;
    type State = GameState;
}

#[derive(Clone, PartialEq, Hash)]
pub struct GameState {}

const MATCH_UPDATE_LABEL: &str = "MATCH_UPDATE";

pub struct FcBackrollPlugin;

impl Plugin for FcBackrollPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .add_plugin(BackrollPlugin::<BackrollConfig>::default())
            .with_rollback_run_criteria::<BackrollConfig, _>(
                FixedTimestep::step(DELTA_TIME.into()).with_label(MATCH_UPDATE_LABEL),
            )
            .with_input_sampler_system::<BackrollConfig, _>(sample_input.system())
            .with_world_save_system::<BackrollConfig, _>(save_world.system())
            .with_world_load_system::<BackrollConfig, _>(load_world.system());
    }
}

fn sample_input(
    handle: In<PlayerHandle>,
    keyboard: Res<Input<KeyCode>>,
    config: Res<MatchConfig>,
) -> PlayerInputFrame {
    let player = config.players.get(handle.0 .0).unwrap().as_ref().unwrap();
    match &player.input {
        InputSource::None | InputSource::CPU => {
            panic!("Cannot get local input for player {:?}", handle.0 .0);
        }
        InputSource::Keyboard {
            movement,
            smash,
            buttons,
        } => PlayerInputFrame {
            movement: movement.sample(&keyboard),
            smash: smash.sample(&keyboard),
            buttons: buttons.evaluate_all(&keyboard),
        },
        InputSource::Gamepad { .. } => PlayerInputFrame::zeroed(),
    }
}

fn save_world() -> GameState {
    GameState {}
}

fn load_world(state: In<GameState>) {}
