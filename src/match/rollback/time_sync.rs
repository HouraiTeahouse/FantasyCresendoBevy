use super::{input::GameInput, Frame};
use bevy::log::info;
use std::convert::TryFrom;

const FRAME_WINDOW_SIZE: usize = 40;
const MIN_UNIQUE_FRAMES: usize = 10;
const MIN_FRAME_ADVANTAGE: super::Frame = 3;
const MAX_FRAME_ADVANTAGE: super::Frame = 9;

pub struct TimeSync<T> {
    local: [Frame; FRAME_WINDOW_SIZE],
    remote: [Frame; FRAME_WINDOW_SIZE],
    last_inputs: [GameInput<T>; MIN_UNIQUE_FRAMES],
    iteration: u32,
}

impl<T: Default> Default for TimeSync<T> {
    fn default() -> Self {
        Self {
            local: [0; FRAME_WINDOW_SIZE],
            remote: [0; FRAME_WINDOW_SIZE],
            last_inputs: Default::default(),
            iteration: 0,
        }
    }
}

impl<T: PartialEq> TimeSync<T> {
    pub fn advance_frame(&mut self, input: GameInput<T>, advantage: Frame, radvantage: Frame) {
        // Remember the last frame and frame advantage
        let frame = usize::try_from(input.frame).unwrap();
        self.last_inputs[frame % self.last_inputs.len()] = input;
        self.local[frame % self.local.len()] = advantage;
        self.remote[frame % self.remote.len()] = radvantage;
    }

    pub fn recommend_frame_wait_duration(&mut self, require_idle_input: bool) -> super::Frame {
        // Average our local and remote frame advantages
        let sum = self.local.iter().sum::<Frame>() as f32;
        let advantage = sum / (self.local.len() as f32);

        let sum = self.remote.iter().sum::<Frame>() as f32;
        let radvantage = sum / (self.remote.len() as f32);

        self.iteration += 1;

        // See if someone should take action.  The person furthest ahead
        // needs to slow down so the other user can catch up.
        // Only do this if both clients agree on who's ahead!!
        if advantage >= radvantage {
            return 0;
        }

        // Both clients agree that we're the one ahead.  Split
        // the difference between the two to figure out how long to
        // sleep for.
        let sleep_frames = (((radvantage - advantage) / 2.0) + 0.5) as Frame;

        info!(
            "iteration {}:  sleep frames is {}",
            self.iteration, sleep_frames
        );

        // Some things just aren't worth correcting for.  Make sure
        // the difference is relevant before proceeding.
        if sleep_frames < MIN_FRAME_ADVANTAGE {
            return 0;
        }

        // Make sure our input had been "idle enough" before recommending
        // a sleep.  This tries to make the emulator sleep while the
        // user's input isn't sweeping in arcs (e.g. fireball motions in
        // Street Fighter), which could cause the player to miss moves.
        if require_idle_input {
            for idx in 0..self.last_inputs.len() {
                if self.last_inputs[idx] != self.last_inputs[0] {
                    info!(
                        "iteration {}: rejecting due to input stuff at position {}...!!!",
                        self.iteration, idx
                    );
                    return 0;
                }
            }
        }

        // Success!!! Recommend the number of frames to sleep and adjust
        std::cmp::min(sleep_frames, MAX_FRAME_ADVANTAGE)
    }
}
