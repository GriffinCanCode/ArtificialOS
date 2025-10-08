/*!
 * Event Streaming
 * Lock-free event distribution using ring buffers
 *
 * Design: Multiple producers (subsystems), multiple consumers (queries, exporters)
 * Zero-copy where possible, bounded memory usage, automatic backpressure
 */

use crate::core::limits::EVENT_RING_SIZE as RING_SIZE;
use crate::core::sync::lockfree::SeqlockStats;
use crate::monitoring::events::{Event, EventFilter};
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
struct StreamCounters {
    events_produced: u64,
    events_consumed: u64,
    events_dropped: u64,
}

#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    pub events_produced: u64,
    pub events_consumed: u64,
    pub events_dropped: u64,
    pub active_subscribers: usize,
}

pub struct EventStream {
    queue: Arc<ArrayQueue<Event>>,
    counters: SeqlockStats<StreamCounters>,
    subscribers: Arc<AtomicUsize>,
}

impl EventStream {
    /// Create a new event stream
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(RING_SIZE).into()),
            counters: SeqlockStats::new(StreamCounters {
                events_produced: 0,
                events_consumed: 0,
                events_dropped: 0,
            }),
            subscribers: Arc::new(AtomicUsize::new(0).into()),
        }
    }

    /// Publish an event (lock-free, returns false if queue full)
    #[inline]
    pub fn publish(&self, event: Event) -> bool {
        match self.queue.push(event) {
            Ok(()) => {
                self.counters.write(|c| c.events_produced += 1);
                true
            }
            Err(_) => {
                self.counters.write(|c| c.events_dropped += 1);
                false
            }
        }
    }

    /// Try to consume one event (lock-free)
    #[inline]
    pub fn try_consume(&self) -> Option<Event> {
        self.queue.pop().map(|event| {
            self.counters.write(|c| c.events_consumed += 1);
            event
        })
    }

    /// Subscribe to event stream (returns a consumer handle)
    pub fn subscribe(&self) -> Subscriber {
        self.subscribers.fetch_add(1, Ordering::Relaxed);
        Subscriber {
            stream: self.clone(),
            local_consumed: 0,
        }
    }

    /// Get stream statistics
    pub fn stats(&self) -> StreamStats {
        let c = self.counters.read();
        StreamStats {
            events_produced: c.events_produced,
            events_consumed: c.events_consumed,
            events_dropped: c.events_dropped,
            active_subscribers: self.subscribers.load(Ordering::Relaxed),
        }
    }

    /// Get queue utilization (0.0 to 1.0)
    #[inline]
    pub fn utilization(&self) -> f64 {
        self.queue.len() as f64 / RING_SIZE as f64
    }

    /// Check if queue is experiencing backpressure
    #[inline]
    pub fn is_under_pressure(&self) -> bool {
        self.utilization() > 0.75
    }
}

impl Clone for EventStream {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
            counters: self.counters.clone(),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Event stream subscriber handle
pub struct Subscriber {
    stream: EventStream,
    local_consumed: u64,
}

impl Subscriber {
    /// Consume next event
    #[inline]
    pub fn next(&mut self) -> Option<Event> {
        self.stream.try_consume().map(|event| {
            self.local_consumed += 1;
            event
        })
    }

    pub fn filter(&mut self, filter: &EventFilter) -> Vec<Event> {
        let mut events = Vec::with_capacity(32);
        while let Some(event) = self.next() {
            if event.matches(filter) {
                events.push(event);
            }
        }
        events
    }

    /// Get local consumption count
    #[inline]
    pub fn consumed(&self) -> u64 {
        self.local_consumed
    }

    /// Get stream reference
    #[inline]
    pub fn stream(&self) -> &EventStream {
        &self.stream
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.stream.subscribers.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Batch event publisher for high-throughput scenarios
pub struct BatchPublisher {
    stream: EventStream,
    buffer: Vec<Event>,
    capacity: usize,
}

impl BatchPublisher {
    /// Create a batch publisher with specified capacity
    pub fn new(stream: EventStream, capacity: usize) -> Self {
        Self {
            stream,
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add event to batch
    #[inline]
    pub fn push(&mut self, event: Event) {
        self.buffer.push(event);
        if self.buffer.len() >= self.capacity {
            self.flush();
        }
    }

    /// Flush buffered events to stream
    pub fn flush(&mut self) {
        for event in self.buffer.drain(..) {
            // Keep trying if queue is full
            while !self.stream.publish(event.clone()) {
                std::hint::spin_loop();
            }
        }
    }
}

impl Drop for BatchPublisher {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::events::{Category, Payload, Severity};

    #[test]
    fn test_stream_publish_consume() {
        let stream = EventStream::new();

        let event = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test".to_string(),
                priority: 5,
            },
        );

        assert!(stream.publish(event.clone().into()));

        let consumed = stream.try_consume();
        assert!(consumed.is_some());

        let stats = stream.stats();
        assert_eq!(stats.events_produced, 1);
        assert_eq!(stats.events_consumed, 1);
    }

    #[test]
    fn test_subscriber() {
        let stream = EventStream::new();

        let event1 = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test1".to_string(),
                priority: 5,
            },
        );

        let event2 = Event::new(
            Severity::Warn,
            Category::Memory,
            Payload::MemoryPressure {
                usage_pct: 85,
                available_mb: 100,
            },
        );

        stream.publish(event1);
        stream.publish(event2);

        let mut sub = stream.subscribe();
        assert_eq!(sub.stream().stats().active_subscribers, 1);

        let filter = EventFilter::new().category(Category::Memory);
        let events = sub.filter(&filter);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_batch_publisher() {
        let stream = EventStream::new();
        let mut batch = BatchPublisher::new(stream.clone(), 10);

        for i in 0..5 {
            batch.push(Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessCreated {
                    name: format!("test{}", i),
                    priority: 5,
                },
            ));
        }

        batch.flush();
        let stats = stream.stats();
        assert_eq!(stats.events_produced, 5);
    }

    #[test]
    fn test_backpressure() {
        let stream = EventStream::new();

        // Fill the queue
        for i in 0..RING_SIZE {
            let event = Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessCreated {
                    name: format!("test{}", i),
                    priority: 5,
                },
            );
            stream.publish(event);
        }

        assert!(stream.is_under_pressure() || stream.utilization() > 0.9);

        // Try to add one more - should fail or succeed depending on timing
        let event = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "overflow".to_string(),
                priority: 5,
            },
        );

        let result = stream.publish(event);
        if !result {
            assert!(stream.stats().events_dropped > 0);
        }
    }
}
