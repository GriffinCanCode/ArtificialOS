/*!
 * Collection
 * Central orchestration for observability data collection
 */

mod collector;
mod bridge;

pub use collector::Collector;
pub use bridge::{
    collector as global_collector, emit_from_span, emit_from_span_with_pid, init_collector,
};

