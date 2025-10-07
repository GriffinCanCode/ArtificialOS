/*!
 * Signal Handler
 * Executes signal actions on processes
 */

use super::callbacks::CallbackRegistry;
use super::types::{Signal, SignalAction, SignalResult};
use crate::core::types::Pid;
use log::{debug, info};
use std::sync::Arc;

/// Signal handler executor
pub struct SignalHandler {
    callbacks: Arc<CallbackRegistry>,
}

impl SignalHandler {
    pub fn new(callbacks: Arc<CallbackRegistry>) -> Self {
        Self { callbacks }
    }

    /// Execute signal action
    pub fn execute(
        &self,
        pid: Pid,
        signal: Signal,
        action: SignalAction,
    ) -> SignalResult<SignalOutcome> {
        debug!(
            "Executing signal {:?} on PID {} with action {:?}",
            signal, pid, action
        );

        match action {
            SignalAction::Default => self.execute_default(pid, signal),
            SignalAction::Ignore => {
                debug!("Ignoring signal {:?} for PID {}", signal, pid);
                Ok(SignalOutcome::Ignored)
            }
            SignalAction::Handler(handler_id) => {
                self.callbacks.execute(handler_id, pid, signal)?;
                info!(
                    "Handler {} executed for signal {:?} on PID {}",
                    handler_id, signal, pid
                );
                Ok(SignalOutcome::HandlerInvoked(handler_id))
            }
            SignalAction::Terminate => {
                info!("Terminating PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Terminated)
            }
            SignalAction::Stop => {
                info!("Stopping PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Stopped)
            }
            SignalAction::Continue => {
                info!("Continuing PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Continued)
            }
        }
    }

    /// Execute default signal action
    fn execute_default(&self, pid: Pid, signal: Signal) -> SignalResult<SignalOutcome> {
        match signal {
            // Fatal signals
            Signal::SIGKILL
            | Signal::SIGABRT
            | Signal::SIGSEGV
            | Signal::SIGILL
            | Signal::SIGBUS
            | Signal::SIGFPE
            | Signal::SIGSYS => {
                info!("Terminating PID {} due to fatal signal {:?}", pid, signal);
                Ok(SignalOutcome::Terminated)
            }

            // Termination signals (can be caught)
            Signal::SIGTERM | Signal::SIGQUIT | Signal::SIGINT => {
                info!("Terminating PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Terminated)
            }

            // Stop signals
            Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN | Signal::SIGTTOU => {
                info!("Stopping PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Stopped)
            }

            // Continue signal
            Signal::SIGCONT => {
                info!("Continuing PID {} due to signal {:?}", pid, signal);
                Ok(SignalOutcome::Continued)
            }

            // Ignored by default
            Signal::SIGCHLD | Signal::SIGURG | Signal::SIGWINCH => {
                debug!("Ignoring signal {:?} for PID {} (default)", signal, pid);
                Ok(SignalOutcome::Ignored)
            }

            // User-defined and other signals - ignored by default
            Signal::SIGUSR1
            | Signal::SIGUSR2
            | Signal::SIGHUP
            | Signal::SIGPIPE
            | Signal::SIGALRM
            | Signal::SIGTRAP
            | Signal::SIGXCPU
            | Signal::SIGXFSZ
            | Signal::SIGVTALRM
            | Signal::SIGPROF
            | Signal::SIGIO
            | Signal::SIGPWR => {
                debug!("Ignoring signal {:?} for PID {} (default)", signal, pid);
                Ok(SignalOutcome::Ignored)
            }

            // Real-time signals - ignored by default
            Signal::SIGRT(_) => {
                debug!("Ignoring RT signal {:?} for PID {} (default)", signal, pid);
                Ok(SignalOutcome::Ignored)
            }
        }
    }

    /// Validate signal can be sent
    pub fn validate_signal(&self, signal: Signal) -> SignalResult<()> {
        // All signals are valid, but some cannot be caught
        if !signal.can_catch() {
            debug!("Signal {:?} cannot be caught or ignored", signal);
        }
        Ok(())
    }

    /// Get default action for signal
    pub fn default_action(&self, signal: Signal) -> SignalAction {
        if signal.is_fatal() {
            SignalAction::Terminate
        } else {
            match signal {
                Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN | Signal::SIGTTOU => {
                    SignalAction::Stop
                }
                Signal::SIGCONT => SignalAction::Continue,
                Signal::SIGRT(_) | _ => SignalAction::Ignore,
            }
        }
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new(Arc::new(CallbackRegistry::new()))
    }
}

/// Signal execution outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOutcome {
    /// Signal was delivered and handler invoked
    HandlerInvoked(u64),
    /// Signal was ignored
    Ignored,
    /// Process terminated
    Terminated,
    /// Process stopped
    Stopped,
    /// Process continued
    Continued,
}

impl SignalOutcome {
    /// Check if outcome requires process termination
    pub fn is_fatal(&self) -> bool {
        matches!(self, SignalOutcome::Terminated)
    }

    /// Check if outcome changes process state
    pub fn changes_state(&self) -> bool {
        matches!(
            self,
            SignalOutcome::Terminated | SignalOutcome::Stopped | SignalOutcome::Continued
        )
    }
}
