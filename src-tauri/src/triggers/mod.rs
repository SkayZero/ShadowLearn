pub mod manager;
pub mod state_machine;
pub mod trigger_loop;

pub use manager::{TriggerDecision, TriggerManager, TriggerStats};
pub use state_machine::{
    get_state_explanation, get_state_history, get_trigger_state, CooldownReason, OpportunityPreview,
    StateTransition, TriggerEvent, TriggerState, TriggerStateMachine,
};
