/*!
 * Bridge
 * Integrates distributed tracing with event streaming
 *
 * Allows tracing spans to automatically emit events to Collector
 */

use super::collector::Collector;
use super::events::{Category, Event, Payload, Severity};
use crate::core::types::Pid;
use std::sync::Arc;

/// Global collector bridge (optional integration point)
static GLOBAL_COLLECTOR: std::sync::OnceLock<Arc<Collector>> = std::sync::OnceLock::new();

/// Initialize global collector for span-to-event bridge
pub fn init_collector(collector: Collector) {
    GLOBAL_COLLECTOR.get_or_init(|| Arc::new(collector));
}

/// Get global collector if initialized
#[inline]
pub fn collector() -> Option<&'static Arc<Collector>> {
    GLOBAL_COLLECTOR.get()
}

/// Emit event from tracing span context
#[inline]
pub fn emit_from_span(severity: Severity, category: Category, payload: Payload) {
    if let Some(collector) = collector() {
        collector.emit(Event::new(severity, category, payload));
    }
}

/// Emit event with PID from tracing span context
#[inline]
pub fn emit_from_span_with_pid(severity: Severity, category: Category, payload: Payload, pid: Pid) {
    if let Some(collector) = collector() {
        collector.emit(Event::new(severity, category, payload).with_pid(pid));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_initialization() {
        let collector = Collector::new();
        init_collector(collector);

        assert!(GLOBAL_COLLECTOR.get().is_some());
    }

    #[test]
    fn test_emit_without_collector() {
        // Should not panic
        emit_from_span(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test".to_string(),
                priority: 5,
            },
        );
    }
}
