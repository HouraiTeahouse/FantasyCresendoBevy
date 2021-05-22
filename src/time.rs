// TODO(james7132): Make these game config options.
pub const DELTA_TIME: f32 = 1.0 / 60.0;

/// A simple timer for keeping track of countdowns in the number of game ticks
/// that pass. Supports
#[derive(Clone, Debug)]
pub struct FrameTimer(u16);

impl FrameTimer {
    pub fn new(frames: u16) -> Self {
        Self(frames)
    }

    pub fn tick(&mut self) {
        if !self.is_done() {
            self.0 -= 1;
        }
    }

    pub fn reset(&mut self, frames: u16) {
        self.0 = frames;
    }

    pub fn is_done(&self) -> bool {
        self.0 == 0
    }
}
