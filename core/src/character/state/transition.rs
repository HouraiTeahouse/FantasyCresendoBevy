use super::state::StateId;
use serde::{Serialize, Deserialize};
use crate::input::Buttons;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StateTransition {
    /// Optional debug name for the transition.
    pub name: Option<String>,
    /// The target state the state machine will transition to upon satisfying the transition 
    /// conditions.
    pub target_state: StateId,
    /// A list of conditions that all need to be satisifed for the transition to occur.
    pub transition: Vec<TransitionCondition>
}

/// A singular condition for a state transition.
#[derive(Debug, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// Fires true when the state's animation has completed. Functionally the same as
    /// using PassedFrame with the length of the state.
    StateEnd, 
    /// Fires true when the state has passed a specific frame. Good for creating early 
    /// cancellations to states.
    PassedFrame(usize),
    /// Fires true when one or more buttons are held simultaneously.
    ButtonHeld(Buttons),
    /// Fires true when one or more buttons are pressed simultaneously. Typically 
    /// should only used with one button, as players would need to have frame perfect
    /// presses on multiple buttons.
    ButtonTapped(Buttons),
}