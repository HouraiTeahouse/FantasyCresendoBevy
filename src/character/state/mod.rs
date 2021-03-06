mod state;
mod transition;

use crate::character::frame_data::CharacterFrame;
use bevy::reflect::TypeUuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use self::state::*;
pub use self::transition::*;

#[derive(Debug, Default)]
pub struct PlayerState {
    pub state_id: StateId,
    pub frame: usize,
}

impl PlayerState {
    pub fn tick(&mut self) {
        self.frame += 1;
    }
}

#[derive(Default, Debug, Serialize, Deserialize, TypeUuid)]
#[uuid = "14fed4e9-98dc-444d-9791-dcd7f561f714"]
pub struct StateMachine(HashMap<StateId, State>);

impl StateMachine {
    /// Samples a frame from the states in the state machine given a player's current state.
    /// Returns None if the state does not exist or the frame number is invalid for the given
    /// frame.
    pub fn sample_frame(&self, player_state: &PlayerState) -> Option<&CharacterFrame> {
        let state = self.get_state(player_state.state_id)?;
        state.frame_data.get_frame(player_state.frame)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StateId, &State)> {
        self.0.iter()
    }

    pub fn states(&self) -> impl Iterator<Item = &State> {
        self.0.values()
    }

    pub fn states_mut(&mut self) -> impl Iterator<Item = &mut State> {
        self.0.values_mut()
    }

    /// Adds a new state to the state machine.  Returns the assigned ID of the state.
    pub fn add_state(&mut self, state: State) -> StateId {
        let id = self.0.len();
        self.0.insert(id, state);
        id
    }

    /// Gets an immutable reference to a state from the state machine, if it exists.
    pub fn get_state(&self, id: StateId) -> Option<&State> {
        self.0.get(&id)
    }

    /// Gets a mutable reference to a state from the state machine, if it exists.
    pub fn get_state_mut(&mut self, id: StateId) -> Option<&mut State> {
        self.0.get_mut(&id)
    }

    /// Checks if the state machine contains a given state ID.
    pub fn contains_state(&self, id: StateId) -> bool {
        self.0.contains_key(&id)
    }

    /// Removes a state from the state machine. All transitions that reference the state
    /// in transitions will also be removed.
    pub fn remove_state(&mut self, id: StateId) {
        self.0.remove(&id);
        for (_, state) in self.0.iter_mut() {
            state.remove_transitions_to(id)
        }
    }

    /// Creates a new transition starting from from and ending with to.
    /// This function returns None, if either of the two states does not correspond with a state ID
    /// in the current state machine.
    pub fn create_transition(
        &mut self,
        from: StateId,
        to: StateId,
    ) -> Option<&mut StateTransition> {
        if !self.0.contains_key(&to) {
            return None;
        }
        let transition = self.0.get_mut(&from)?.add_transition();
        transition.target_state = to;
        Some(transition)
    }
}

pub enum StateMachineValidationError {
    InvalidTransitionTarget { from: StateId, to: StateId },
}

/// Validates whether a state machine has entirely correct construction.
pub fn validate(machine: StateMachine) -> Result<(), StateMachineValidationError> {
    for (id, state) in machine.iter() {
        for transition in state.transitions.iter() {
            if !machine.contains_state(transition.target_state) {
                return Err(StateMachineValidationError::InvalidTransitionTarget {
                    from: *id,
                    to: transition.target_state,
                });
            }
        }
    }
    Ok(())
}
