use super::transition::StateTransition;
use crate::character::frame_data::StateFrameData;
use serde::{Deserialize, Serialize};

pub type StateId = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    /// A debug name for the state.
    pub name: String,
    /// Outgoing transitions originating from this state. There can be multiple transitions.
    pub transitions: Vec<StateTransition>,
    pub frame_data: StateFrameData,
}

impl State {
    /// Creates a new transition starting from this state.
    pub(crate) fn add_transition(&mut self) -> &mut StateTransition {
        self.transitions.push(StateTransition::default());
        self.transitions.last_mut().unwrap()
    }

    pub(crate) fn remove_transitions_to(&mut self, state: StateId) {
        self.transitions
            .retain(|transition| transition.target_state != state);
    }
}
