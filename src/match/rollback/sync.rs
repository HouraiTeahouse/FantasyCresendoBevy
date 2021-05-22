use super::{
    super::MAX_PLAYERS_PER_MATCH,
    input::{GameInput, InputQueue},
    Frame,
};
use bevy::log::*;

const MAX_PREDICTION_FRAMES: usize = 8;

pub enum RollbackError {
    ReachedPredictionBarrier,
}

pub trait SessionCallbacks {}

pub struct Config<Input> {
    callbacks: Box<dyn SessionCallbacks>,
    max_prediction_frames: i32,
    num_players: usize,
    _marker: std::marker::PhantomData<Input>,
}

#[derive(Default)]
pub struct ConnectionStatus {
    disconnected: bool,
    last_frame: Frame,
}

// struct Event {
//     enum {
//         ConfirmedInput,
//     } type;
//     union {
//         struct {
//         GameInput   input;
//         } confirmedInput;
//     } u;
// };

pub struct DisconnectedError(u64);

impl DisconnectedError {
    pub fn is_disconnected(&self, player: u64) -> bool {
        self.0 & (1 << player) != 0
    }
}

pub struct SavedFrame<T> {
    frame: super::Frame,
    data: Option<Box<T>>,
    checksum: i32,
}

impl<T> Default for SavedFrame<T> {
    fn default() -> Self {
        Self {
            frame: super::NULL_FRAME,
            data: None,
            checksum: 0,
        }
    }
}

pub struct SavedState<T> {
    head: usize,
    frames: [SavedFrame<T>; MAX_PREDICTION_FRAMES + 2],
}

impl<T> Default for SavedState<T> {
    fn default() -> Self {
        Self {
            head: 0,
            frames: Default::default(),
        }
    }
}

pub struct RollbackSync<State, Input> {
    saved_state: SavedState<State>,
    local_connect_status: [ConnectionStatus; MAX_PLAYERS_PER_MATCH],
    input_queues: Vec<InputQueue<Input>>,
    config: Config<Input>,
    rolling_back: bool,

    last_confirmed_frame: Frame,
    frame_count: Frame,
}

impl<State, Input: Default + Eq + Clone> RollbackSync<State, Input> {
    pub fn new(config: Config<Input>) -> Self {
        let input_queues = Self::create_queues(&config);
        Self {
            saved_state: Default::default(),
            local_connect_status: Default::default(),
            input_queues,
            config,

            rolling_back: false,
            last_confirmed_frame: super::NULL_FRAME,
            frame_count: 0,
        }
    }

    pub fn frame_count(&self) -> Frame {
        self.frame_count
    }

    pub fn in_rollback(&self) -> bool {
        self.rolling_back
    }

    pub fn set_last_confirmed_frame(&mut self, frame: Frame) {
        self.last_confirmed_frame = frame;
        if frame > 0 {
            for queue in self.input_queues.iter_mut() {
                queue.discard_confirmed_frames(frame - 1);
            }
        }
    }

    pub fn set_frame_delay(&mut self, queue: usize, delay: Frame) {
        self.input_queues[queue].set_frame_delay(delay);
    }

    pub fn increment_frame(&mut self) {
        self.frame_count += 1;
        self.save_current_frame();
    }

    pub fn add_local_input(
        &mut self,
        queue: usize,
        mut input: GameInput<Input>,
    ) -> Result<(), RollbackError> {
        let frames_behind = self.frame_count - self.last_confirmed_frame;
        if self.frame_count >= self.config.max_prediction_frames
            && frames_behind >= self.config.max_prediction_frames
        {
            warn!("Rejecting input from emulator: reached prediction barrier.");
            return Err(RollbackError::ReachedPredictionBarrier);
        }

        if self.frame_count == 0 {
            self.save_current_frame();
        }

        info!(
            "Sending undelayed local frame {} to queue {}.",
            self.frame_count, queue
        );
        input.frame = self.frame_count;
        self.input_queues[queue].add_input(input);

        Ok(())
    }

    pub fn add_remote_input(&mut self, queue: usize, input: GameInput<Input>) {
        self.input_queues[queue].add_input(input);
    }

    pub fn get_confirmed_inputs(
        &mut self,
        frame: Frame,
    ) -> Result<GameInput<Input>, DisconnectedError> {
        let mut disconnect_flags = 0;
        let mut output: GameInput<Input> = Default::default();
        for idx in 0..self.config.num_players {
            let input = if self.is_disconnected(idx, frame) {
                disconnect_flags |= 1 << idx;
                Default::default()
            } else {
                self.input_queues[idx]
                    .get_confirmed_input(frame)
                    .unwrap()
                    .clone()
            };
            output.inputs[idx] = input.inputs[0].clone();
        }

        if disconnect_flags != 0 {
            Err(DisconnectedError(disconnect_flags))
        } else {
            Ok(output)
        }
    }

    pub fn synchrohnize_inputs(
        &mut self,
        frame: Frame,
    ) -> Result<GameInput<Input>, DisconnectedError> {
        let mut disconnect_flags = 0;
        let mut output: GameInput<Input> = Default::default();
        for idx in 0..self.config.num_players {
            if self.is_disconnected(idx, frame) {
                disconnect_flags |= 1 << idx;
            } else if let Some(confirmed) = self.input_queues[idx].get_confirmed_input(frame) {
                output.inputs[idx] = confirmed.inputs[0].clone();
            }
        }

        if disconnect_flags != 0 {
            Err(DisconnectedError(disconnect_flags))
        } else {
            Ok(output)
        }
    }

    pub fn check_simulation(&mut self, timeout: i32) {
        if let Some(seek_to) = self.check_simulation_consistency() {
            self.adjust_simulation(seek_to);
        }
    }

    fn find_saved_frame_index(&self, frame: Frame) -> usize {
        self.saved_state
            .frames
            .iter()
            .enumerate()
            .find(|(_, saved)| saved.frame == frame)
            .unwrap_or_else(|| panic!("Could not find saved frame index for frame: {}", frame))
            .0
    }

    pub fn get_last_saved_frame(&self) -> &SavedFrame<State> {
        let mut idx = self.saved_state.head - 1;
        if idx < 0 {
            idx = self.saved_state.frames.len() - 1;
        }
        &self.saved_state.frames[idx]
    }

    pub fn load_frame(&mut self, frame: Frame) {
        // find the frame in question
        if frame == self.frame_count {
            info!("Skipping NOP.");
            return;
        }

        // Move the head pointer back and load it up
        self.saved_state.head = self.find_saved_frame_index(frame);
        let state = &self.saved_state.frames[self.saved_state.head];

        info!("=== Loading frame info (checksum: {:08x}).", state.checksum);
        debug_assert!(state.data.is_some());
        // self.config.callbacks.load_game_state(state);

        // Reset framecount and the head of the state ring-buffer to point in
        // advance of the current frame (as if we had just finished executing it).
        self.frame_count = state.frame;
        self.saved_state.head = (self.saved_state.head + 1) % self.saved_state.frames.len();
    }

    pub fn save_current_frame(&mut self) {
        {
            let state = &mut self.saved_state.frames[self.saved_state.head];
            state.frame = self.frame_count;
            // let (save, checksum) = self.config.callbacks.save_game_state(state->frame);
            // state.data = Some(save);
            // state.checksum = checksum;
            info!(
                "=== Saved frame info {} (checksum: {:08x}).",
                state.frame, state.checksum
            );
        };
        self.saved_state.head = (self.saved_state.head + 1) % self.saved_state.frames.len();
    }

    pub fn adjust_simulation(&mut self, seek_to: Frame) {
        let frame_count = self.frame_count;
        let count = self.frame_count - seek_to;

        info!("Catching up");
        self.rolling_back = true;

        //  Flush our input queue and load the last frame.
        self.load_frame(seek_to);
        debug_assert!(self.frame_count == seek_to);

        // Advance frame by frame (stuffing notifications back to
        // the master).
        self.reset_prediction(self.frame_count);
        for _ in 0..count {
            // _callbacks.advance_frame(0);
        }
        debug_assert!(self.frame_count == frame_count);

        self.rolling_back = false;
        info!("---");
    }

    pub fn check_simulation_consistency(&self) -> Option<Frame> {
        self.input_queues
            .iter()
            .map(|queue| queue.first_incorrect_frame())
            .filter(|frame| !super::is_null(*frame))
            .min()
    }

    fn reset_prediction(&mut self, frame: Frame) {
        for queue in self.input_queues.iter_mut() {
            queue.reset_prediction(frame);
        }
    }

    fn is_disconnected(&self, player: usize, frame: Frame) -> bool {
        let status = &self.local_connect_status[player];
        status.disconnected && status.last_frame < frame
    }

    fn create_queues(config: &Config<Input>) -> Vec<InputQueue<Input>> {
        (0..config.num_players).map(|_| InputQueue::new()).collect()
    }
}

//    void AdjustSimulation(int seek_to);

//    bool CheckSimulationConsistency(int *seekTo);

//    UdpMsg::connect_status *_local_connect_status;

// bool
// Sync::GetEvent(Event &e)
// {
//    if (_event_queue.size()) {
//       e = _event_queue.front();
//       _event_queue.pop();
//       return true;
//    }
//    return false;
// }
