/*!
 * Process Integration
 * Links signal outcomes to process state changes
 */

use super::handler::SignalOutcome;
use crate::process::types::ProcessState;

/// Convert signal outcome to process state change
pub fn outcome_to_state(outcome: SignalOutcome) -> Option<ProcessState> {
    match outcome {
        SignalOutcome::Terminated => Some(ProcessState::Terminated),
        SignalOutcome::Stopped => Some(ProcessState::Waiting),
        SignalOutcome::Continued => Some(ProcessState::Running),
        SignalOutcome::HandlerInvoked(_) | SignalOutcome::Ignored => None,
    }
}

/// Apply signal outcome to process
pub trait ProcessSignalIntegration {
    /// Apply signal outcome to process state
    fn apply_signal_outcome(&self, pid: u32, outcome: SignalOutcome) -> Result<(), String>;
}

/// Helper to check if outcome requires immediate action
pub fn requires_immediate_action(outcome: &SignalOutcome) -> bool {
    matches!(
        outcome,
        SignalOutcome::Terminated | SignalOutcome::Stopped | SignalOutcome::Continued
    )
}

/// Helper to check if outcome should interrupt execution
pub fn should_interrupt(outcome: &SignalOutcome) -> bool {
    matches!(outcome, SignalOutcome::Terminated | SignalOutcome::Stopped)
}
