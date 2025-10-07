/*!
 * Signal Types
 * UNIX-style signal definitions and result types
 */

use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Signal operation result
pub type SignalResult<T> = Result<T, SignalError>;

/// Signal errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SignalError {
    #[error("Process not found: {0}")]
    ProcessNotFound(Pid),

    #[error("Invalid signal: {0}")]
    InvalidSignal(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Signal blocked: {0}")]
    SignalBlocked(Signal),

    #[error("Handler error: {0}")]
    HandlerError(String),

    #[error("Queue full: process {0} has too many pending signals")]
    QueueFull(Pid),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// UNIX-style signal numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Signal {
    /// Hangup detected on controlling terminal or death of controlling process
    SIGHUP = 1,
    /// Interrupt from keyboard (Ctrl+C)
    SIGINT = 2,
    /// Quit from keyboard (Ctrl+\)
    SIGQUIT = 3,
    /// Illegal instruction
    SIGILL = 4,
    /// Trace/breakpoint trap
    SIGTRAP = 5,
    /// Abort signal
    SIGABRT = 6,
    /// Bus error (bad memory access)
    SIGBUS = 7,
    /// Floating-point exception
    SIGFPE = 8,
    /// Kill signal (cannot be caught or ignored)
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// Invalid memory reference
    SIGSEGV = 11,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Broken pipe
    SIGPIPE = 13,
    /// Timer signal
    SIGALRM = 14,
    /// Termination signal
    SIGTERM = 15,
    /// Child process stopped or terminated
    SIGCHLD = 17,
    /// Continue if stopped
    SIGCONT = 18,
    /// Stop process (cannot be caught or ignored)
    SIGSTOP = 19,
    /// Stop typed at terminal (Ctrl+Z)
    SIGTSTP = 20,
    /// Terminal input for background process
    SIGTTIN = 21,
    /// Terminal output for background process
    SIGTTOU = 22,
    /// Urgent condition on socket
    SIGURG = 23,
    /// CPU time limit exceeded
    SIGXCPU = 24,
    /// File size limit exceeded
    SIGXFSZ = 25,
    /// Virtual alarm clock
    SIGVTALRM = 26,
    /// Profiling timer expired
    SIGPROF = 27,
    /// Window resize signal
    SIGWINCH = 28,
    /// I/O now possible
    SIGIO = 29,
    /// Power failure
    SIGPWR = 30,
    /// Bad system call
    SIGSYS = 31,
}

impl Signal {
    /// Convert from signal number
    pub fn from_number(n: u32) -> SignalResult<Self> {
        match n {
            1 => Ok(Signal::SIGHUP),
            2 => Ok(Signal::SIGINT),
            3 => Ok(Signal::SIGQUIT),
            4 => Ok(Signal::SIGILL),
            5 => Ok(Signal::SIGTRAP),
            6 => Ok(Signal::SIGABRT),
            7 => Ok(Signal::SIGBUS),
            8 => Ok(Signal::SIGFPE),
            9 => Ok(Signal::SIGKILL),
            10 => Ok(Signal::SIGUSR1),
            11 => Ok(Signal::SIGSEGV),
            12 => Ok(Signal::SIGUSR2),
            13 => Ok(Signal::SIGPIPE),
            14 => Ok(Signal::SIGALRM),
            15 => Ok(Signal::SIGTERM),
            17 => Ok(Signal::SIGCHLD),
            18 => Ok(Signal::SIGCONT),
            19 => Ok(Signal::SIGSTOP),
            20 => Ok(Signal::SIGTSTP),
            21 => Ok(Signal::SIGTTIN),
            22 => Ok(Signal::SIGTTOU),
            23 => Ok(Signal::SIGURG),
            24 => Ok(Signal::SIGXCPU),
            25 => Ok(Signal::SIGXFSZ),
            26 => Ok(Signal::SIGVTALRM),
            27 => Ok(Signal::SIGPROF),
            28 => Ok(Signal::SIGWINCH),
            29 => Ok(Signal::SIGIO),
            30 => Ok(Signal::SIGPWR),
            31 => Ok(Signal::SIGSYS),
            _ => Err(SignalError::InvalidSignal(n)),
        }
    }

    /// Get signal number
    pub fn number(&self) -> u32 {
        *self as u32
    }

    /// Check if signal can be caught/blocked
    pub fn can_catch(&self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Check if signal is fatal by default
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Signal::SIGKILL
                | Signal::SIGTERM
                | Signal::SIGQUIT
                | Signal::SIGABRT
                | Signal::SIGSEGV
                | Signal::SIGILL
                | Signal::SIGBUS
                | Signal::SIGFPE
                | Signal::SIGSYS
        )
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Signal::SIGHUP => "Hangup",
            Signal::SIGINT => "Interrupt",
            Signal::SIGQUIT => "Quit",
            Signal::SIGILL => "Illegal instruction",
            Signal::SIGTRAP => "Trace/breakpoint trap",
            Signal::SIGABRT => "Aborted",
            Signal::SIGBUS => "Bus error",
            Signal::SIGFPE => "Floating point exception",
            Signal::SIGKILL => "Killed",
            Signal::SIGUSR1 => "User defined signal 1",
            Signal::SIGSEGV => "Segmentation fault",
            Signal::SIGUSR2 => "User defined signal 2",
            Signal::SIGPIPE => "Broken pipe",
            Signal::SIGALRM => "Alarm clock",
            Signal::SIGTERM => "Terminated",
            Signal::SIGCHLD => "Child status changed",
            Signal::SIGCONT => "Continued",
            Signal::SIGSTOP => "Stopped (signal)",
            Signal::SIGTSTP => "Stopped",
            Signal::SIGTTIN => "Stopped (tty input)",
            Signal::SIGTTOU => "Stopped (tty output)",
            Signal::SIGURG => "Urgent I/O condition",
            Signal::SIGXCPU => "CPU time limit exceeded",
            Signal::SIGXFSZ => "File size limit exceeded",
            Signal::SIGVTALRM => "Virtual timer expired",
            Signal::SIGPROF => "Profiling timer expired",
            Signal::SIGWINCH => "Window size changed",
            Signal::SIGIO => "I/O possible",
            Signal::SIGPWR => "Power failure",
            Signal::SIGSYS => "Bad system call",
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SIG{:?}({})", self, self.number())
    }
}

/// Signal disposition - what happens when signal is received
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalDisposition {
    /// Default action for the signal
    Default,
    /// Ignore the signal
    Ignore,
    /// Call custom handler
    Handle,
    /// Stop the process
    Stop,
    /// Continue the process
    Continue,
}

/// Pending signal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSignal {
    pub signal: Signal,
    pub sender_pid: Pid,
    pub timestamp: u64,
}

/// Signal handler action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalAction {
    /// Default behavior
    Default,
    /// Ignore signal
    Ignore,
    /// Custom handler ID
    Handler(u64),
    /// Terminate process
    Terminate,
    /// Stop process
    Stop,
    /// Continue process
    Continue,
}

impl SignalAction {
    /// Get disposition from action
    pub fn disposition(&self) -> SignalDisposition {
        match self {
            SignalAction::Default => SignalDisposition::Default,
            SignalAction::Ignore => SignalDisposition::Ignore,
            SignalAction::Handler(_) => SignalDisposition::Handle,
            SignalAction::Terminate => SignalDisposition::Default,
            SignalAction::Stop => SignalDisposition::Stop,
            SignalAction::Continue => SignalDisposition::Continue,
        }
    }
}

/// Signal statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStats {
    pub total_signals_sent: u64,
    pub total_signals_delivered: u64,
    pub total_signals_blocked: u64,
    pub total_signals_queued: u64,
    pub pending_signals: usize,
    pub handlers_registered: usize,
}

/// Process signal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSignalState {
    pub pid: Pid,
    pub pending_signals: Vec<PendingSignal>,
    pub blocked_signals: Vec<Signal>,
    pub handlers: Vec<(Signal, SignalAction)>,
}
