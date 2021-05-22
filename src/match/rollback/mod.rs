use super::{MatchState, MAX_PLAYERS_PER_MATCH};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

pub mod input;
pub mod sync;
pub mod time_sync;

// Approximately 2 seconds of frames.
const MAX_ROLLBACK_FRAMES: usize = 120;

type Frame = i32;
const NULL_FRAME: Frame = -1;

fn is_null(frame: Frame) -> bool {
    frame < 0
}

#[derive(Clone)]
pub struct SavedState {
    match_state: MatchState,
    players: [Option<SavedPlayerState>; MAX_PLAYERS_PER_MATCH],
}

#[derive(Clone)]
pub struct SavedPlayerState {}

#[derive(Clone)]
pub struct SavedStates {
    last_acked_state: usize,
    states: ConstGenericRingBuffer<SavedState, MAX_ROLLBACK_FRAMES>,
}

impl SavedStates {
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}
