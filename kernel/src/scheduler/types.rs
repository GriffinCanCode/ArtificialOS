/*!
 * Scheduler Syscall Types
 * Domain types for scheduler operations
 */

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Scheduler policy configuration
///
/// # Performance
/// - Packed C layout for efficient policy checks
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerPolicy {
    /// Round-robin with fixed time quantum
    RoundRobin,
    /// Priority-based preemptive scheduling
    Priority,
    /// Fair scheduling (CFS-inspired)
    Fair,
}

impl SchedulerPolicy {
    /// Parse from string representation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "round_robin" | "roundrobin" | "rr" => Ok(Self::RoundRobin),
            "priority" | "prio" => Ok(Self::Priority),
            "fair" | "cfs" => Ok(Self::Fair),
            _ => Err(format!("Invalid policy '{}'. Valid: round_robin, priority, fair", s).into()),
        }
    }

    /// Convert to string representation
    ///
    /// # Performance
    /// Hot path - frequently called for logging and serialization
    #[inline(always)]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RoundRobin => "round_robin",
            Self::Priority => "priority",
            Self::Fair => "fair",
        }
    }
}

impl Serialize for SchedulerPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SchedulerPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Time quantum configuration
///
/// # Performance
/// - Packed C layout for efficient time slice calculations
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeQuantum {
    pub micros: u64,
}

impl TimeQuantum {
    /// Create new time quantum
    pub fn new(micros: u64) -> Result<Self, String> {
        if micros < 1_000 || micros > 1_000_000 {
            return Err(format!(
                "Invalid quantum: {} must be between 1ms (1000μs) and 1s (1000000μs)",
                micros
            ));
        }
        Ok(Self { micros })
    }

    /// Get microseconds
    ///
    /// # Performance
    /// Hot path - called on every time slice calculation
    #[inline(always)]
    pub const fn as_micros(&self) -> u64 {
        self.micros
    }

    /// Get milliseconds
    ///
    /// # Performance
    /// Hot path - frequently used in scheduler statistics
    #[inline(always)]
    pub const fn as_millis(&self) -> f64 {
        self.micros as f64 / 1000.0
    }
}

impl<'de> Deserialize<'de> for TimeQuantum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            micros: u64,
        }

        let inner = Inner::deserialize(deserializer)?;
        Self::new(inner.micros).map_err(serde::de::Error::custom)
    }
}

/// Priority adjustment operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityOp {
    /// Increase priority
    Boost,
    /// Decrease priority
    Lower,
    /// Set to specific value
    Set(u8),
}

/// Priority bounds
pub const MIN_PRIORITY: u8 = 0;
pub const MAX_PRIORITY: u8 = 10;
pub const DEFAULT_PRIORITY: u8 = 5;

/// Validate priority value
///
/// # Performance
/// Hot path - called on every priority change operation
#[inline(always)]
pub fn validate_priority(priority: u8) -> Result<u8, String> {
    if priority > MAX_PRIORITY {
        // Cold path - validation failure
        #[cold]
        #[inline(never)]
        fn make_error(priority: u8) -> String {
            format!("Priority {} exceeds maximum ({})", priority, MAX_PRIORITY)
        }
        Err(make_error(priority))
    } else {
        Ok(priority)
    }
}

/// Apply priority operation
pub fn apply_priority_op(current: u8, op: PriorityOp) -> Result<u8, String> {
    match op {
        PriorityOp::Boost => {
            let new_priority = (current + 1).min(MAX_PRIORITY);
            if new_priority == current {
                Err(format!("Already at maximum priority ({})", MAX_PRIORITY))
            } else {
                Ok(new_priority)
            }
        }
        PriorityOp::Lower => {
            let new_priority = current.saturating_sub(1);
            if new_priority == current {
                Err(format!("Already at minimum priority ({})", MIN_PRIORITY))
            } else {
                Ok(new_priority)
            }
        }
        PriorityOp::Set(value) => validate_priority(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_parsing() {
        assert_eq!(
            SchedulerPolicy::from_str("round_robin").unwrap(),
            SchedulerPolicy::RoundRobin
        );
        assert_eq!(
            SchedulerPolicy::from_str("priority").unwrap(),
            SchedulerPolicy::Priority
        );
        assert_eq!(
            SchedulerPolicy::from_str("fair").unwrap(),
            SchedulerPolicy::Fair
        );
        assert!(SchedulerPolicy::from_str("invalid").is_err());
    }

    #[test]
    fn test_time_quantum_validation() {
        assert!(TimeQuantum::new(500).is_err()); // Too small
        assert!(TimeQuantum::new(1_000).is_ok()); // Min
        assert!(TimeQuantum::new(10_000).is_ok()); // Valid
        assert!(TimeQuantum::new(1_000_000).is_ok()); // Max
        assert!(TimeQuantum::new(2_000_000).is_err()); // Too large
    }

    #[test]
    fn test_priority_operations() {
        // Boost
        assert_eq!(apply_priority_op(5, PriorityOp::Boost).unwrap(), 6);
        assert!(apply_priority_op(MAX_PRIORITY, PriorityOp::Boost).is_err());

        // Lower
        assert_eq!(apply_priority_op(5, PriorityOp::Lower).unwrap(), 4);
        assert!(apply_priority_op(MIN_PRIORITY, PriorityOp::Lower).is_err());

        // Set
        assert_eq!(apply_priority_op(5, PriorityOp::Set(8)).unwrap(), 8);
        assert!(apply_priority_op(5, PriorityOp::Set(15)).is_err());
    }
}
