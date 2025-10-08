/*!
 * Collection
 * Central orchestration for observability data collection
 */

mod bridge;
mod collector;

pub use bridge::{
    collector as global_collector, emit_from_span, emit_from_span_with_pid, init_collector,
};
pub use collector::Collector;
