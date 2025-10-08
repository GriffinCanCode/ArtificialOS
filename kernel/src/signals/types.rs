/*!
 * Signal Types
 * UNIX-style signal definitions and result types
 */

use crate::core::serde::is_zero_u64;
use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Signal operation result
///
/// # Must Use
/// Signal operations can fail and must be handled to prevent undefined behavior
pub type SignalResult<T> = Result<T, SignalError>;

/// Signal errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "details")]
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
///
/// # Performance
/// - Packed C layout for efficient signal dispatch
/// - Copy-optimized for frequent passing in signal handlers
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Signal {
    // Standard signals (1-31)
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPWR,
    SIGSYS,

    /// Real-time signal with priority (34-63)
    /// Higher numbers = higher priority
    SIGRT(u32),
}

/// Real-time signal range
pub const SIGRTMIN: u32 = 34;
pub const SIGRTMAX: u32 = 63;

impl Signal {
    /// Convert from signal number
    #[must_use]
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
            n @ SIGRTMIN..=SIGRTMAX => Ok(Signal::SIGRT(n)),
            _ => Err(SignalError::InvalidSignal(n)),
        }
    }

    /// Get signal number
    ///
    /// # Performance
    /// Hot path - frequently called in signal routing and prioritization
    #[inline(always)]
    #[must_use]
    pub const fn number(&self) -> u32 {
        match self {
            Signal::SIGHUP => 1,
            Signal::SIGINT => 2,
            Signal::SIGQUIT => 3,
            Signal::SIGILL => 4,
            Signal::SIGTRAP => 5,
            Signal::SIGABRT => 6,
            Signal::SIGBUS => 7,
            Signal::SIGFPE => 8,
            Signal::SIGKILL => 9,
            Signal::SIGUSR1 => 10,
            Signal::SIGSEGV => 11,
            Signal::SIGUSR2 => 12,
            Signal::SIGPIPE => 13,
            Signal::SIGALRM => 14,
            Signal::SIGTERM => 15,
            Signal::SIGCHLD => 17,
            Signal::SIGCONT => 18,
            Signal::SIGSTOP => 19,
            Signal::SIGTSTP => 20,
            Signal::SIGTTIN => 21,
            Signal::SIGTTOU => 22,
            Signal::SIGURG => 23,
            Signal::SIGXCPU => 24,
            Signal::SIGXFSZ => 25,
            Signal::SIGVTALRM => 26,
            Signal::SIGPROF => 27,
            Signal::SIGWINCH => 28,
            Signal::SIGIO => 29,
            Signal::SIGPWR => 30,
            Signal::SIGSYS => 31,
            Signal::SIGRT(n) => *n,
        }
    }

    /// Check if this is a real-time signal
    ///
    /// # Performance
    /// Hot path - frequently checked for signal priority routing
    #[inline(always)]
    #[must_use]
    pub const fn is_realtime(&self) -> bool {
        matches!(self, Signal::SIGRT(_))
    }

    /// Get priority (higher = more urgent, RT signals > standard)
    ///
    /// # Performance
    /// Hot path - critical for signal queue ordering
    #[inline(always)]
    #[must_use]
    pub const fn priority(&self) -> u32 {
        match self {
            Signal::SIGRT(n) => 1000 + (*n as u32), // RT signals always higher priority
            _ => self.number(),
        }
    }

    /// Check if signal can be caught/blocked
    ///
    /// # Performance
    /// Hot path - checked on every signal delivery
    #[inline(always)]
    #[must_use]
    pub const fn can_catch(&self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Check if signal is fatal by default
    ///
    /// # Performance
    /// Hot path - determines signal handling behavior
    #[inline(always)]
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
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
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Signal::SIGHUP => "Hangup".to_string(),
            Signal::SIGINT => "Interrupt".to_string(),
            Signal::SIGQUIT => "Quit".to_string(),
            Signal::SIGILL => "Illegal instruction".to_string(),
            Signal::SIGTRAP => "Trace/breakpoint trap".to_string(),
            Signal::SIGABRT => "Aborted".to_string(),
            Signal::SIGBUS => "Bus error".to_string(),
            Signal::SIGFPE => "Floating point exception".to_string(),
            Signal::SIGKILL => "Killed".to_string(),
            Signal::SIGUSR1 => "User defined signal 1".to_string(),
            Signal::SIGSEGV => "Segmentation fault".to_string(),
            Signal::SIGUSR2 => "User defined signal 2".to_string(),
            Signal::SIGPIPE => "Broken pipe".to_string(),
            Signal::SIGALRM => "Alarm clock".to_string(),
            Signal::SIGTERM => "Terminated".to_string(),
            Signal::SIGCHLD => "Child status changed".to_string(),
            Signal::SIGCONT => "Continued".to_string(),
            Signal::SIGSTOP => "Stopped (signal)".to_string(),
            Signal::SIGTSTP => "Stopped".to_string(),
            Signal::SIGTTIN => "Stopped (tty input)".to_string(),
            Signal::SIGTTOU => "Stopped (tty output)".to_string(),
            Signal::SIGURG => "Urgent I/O condition".to_string(),
            Signal::SIGXCPU => "CPU time limit exceeded".to_string(),
            Signal::SIGXFSZ => "File size limit exceeded".to_string(),
            Signal::SIGVTALRM => "Virtual timer expired".to_string(),
            Signal::SIGPROF => "Profiling timer expired".to_string(),
            Signal::SIGWINCH => "Window size changed".to_string(),
            Signal::SIGIO => "I/O possible".to_string(),
            Signal::SIGPWR => "Power failure".to_string(),
            Signal::SIGSYS => "Bad system call".to_string(),
            Signal::SIGRT(n) => format!("Real-time signal {}", n),
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
#[serde(rename_all = "snake_case")]
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
///
/// # Performance
/// - Cache-line aligned for fast signal queue operations
#[repr(C, align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSignal {
    pub signal: Signal,
    pub sender_pid: Pid,
    pub timestamp: u64,
}

/// Signal handler action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
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
    #[inline]
    #[must_use]
    pub const fn disposition(&self) -> SignalDisposition {
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
#[serde(rename_all = "snake_case")]
pub struct SignalStats {
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub total_signals_sent: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub total_signals_delivered: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub total_signals_blocked: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub total_signals_queued: u64,
    #[serde(skip_serializing_if = "crate::core::serde::is_zero_usize")]
    pub pending_signals: usize,
    #[serde(skip_serializing_if = "crate::core::serde::is_zero_usize")]
    pub handlers_registered: usize,
}

/// Process signal state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProcessSignalState {
    pub pid: Pid,
    #[serde(skip_serializing_if = "crate::core::serde::is_empty_vec")]
    pub pending_signals: Vec<PendingSignal>,
    #[serde(skip_serializing_if = "crate::core::serde::is_empty_vec")]
    pub blocked_signals: Vec<Signal>,
    #[serde(skip_serializing_if = "crate::core::serde::is_empty_vec")]
    pub handlers: Vec<(Signal, SignalAction)>,
}
