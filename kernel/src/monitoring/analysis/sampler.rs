/*!
 * Adaptive Sampling
 * Intelligent overhead control via dynamic sampling rates
 *
 * Strategy: Monitor system load and observability overhead, adjust sampling
 * to maintain target overhead percentage (default 1-2%)
 */

use crate::core::sync::lockfree::SeqlockStats;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Target overhead as percentage of CPU time (1-100)
const TARGET_OVERHEAD_PCT: u8 = 2;

/// Sampling adjustment interval (number of events)
use crate::core::limits::SAMPLING_ADJUSTMENT_INTERVAL as ADJUSTMENT_INTERVAL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleDecision {
    Accept,
    Reject,
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
struct SamplerCounters {
    evaluated: u64,
    accepted: u64,
}

pub struct Sampler {
    rate: Arc<AtomicU8>,
    counters: SeqlockStats<SamplerCounters>,
    overhead_pct: Arc<AtomicU8>,
    category_rates: [Arc<AtomicU8>; 9],
}

impl Sampler {
    /// Create a new adaptive sampler starting at 100% rate
    pub fn new() -> Self {
        Self {
            rate: Arc::new(AtomicU8::new(100).into()),
            counters: SeqlockStats::new(SamplerCounters {
                evaluated: 0,
                accepted: 0,
            }),
            overhead_pct: Arc::new(AtomicU8::new(0).into()),
            category_rates: [
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
                Arc::new(AtomicU8::new(100).into()),
            ],
        }
    }

    /// Decide whether to sample this event (fast path)
    #[inline]
    pub fn should_sample(&self) -> SampleDecision {
        let evaluated = self.counters.write_batch(|c| {
            c.evaluated += 1;
            c.evaluated
        });

        if evaluated % ADJUSTMENT_INTERVAL == 0 {
            self.adjust_rate();
        }

        let rate = self.rate.load(Ordering::Relaxed);

        if rate < 100 {
            let random = self.fast_random() % 100;
            if random >= rate as u64 {
                return SampleDecision::Reject;
            }
        }

        self.counters.write(|c| c.accepted += 1);
        SampleDecision::Accept
    }

    /// Decide whether to sample for a specific category
    #[inline]
    pub fn should_sample_category(&self, category_idx: usize) -> SampleDecision {
        if category_idx >= self.category_rates.len() {
            return self.should_sample();
        }

        let rate = self.category_rates[category_idx].load(Ordering::Relaxed);

        if rate < 100 {
            let random = self.fast_random() % 100;
            if random >= rate as u64 {
                return SampleDecision::Reject;
            }
        }

        self.counters.write(|c| c.accepted += 1);
        SampleDecision::Accept
    }

    /// Update estimated overhead percentage
    pub fn update_overhead(&self, overhead_pct: u8) {
        self.overhead_pct.store(overhead_pct, Ordering::Relaxed);

        // Trigger immediate adjustment if overhead too high
        if overhead_pct > TARGET_OVERHEAD_PCT * 2 {
            self.adjust_rate();
        }
    }

    /// Adjust sampling rate based on overhead
    fn adjust_rate(&self) {
        let current_overhead = self.overhead_pct.load(Ordering::Relaxed);
        let current_rate = self.rate.load(Ordering::Relaxed);

        let new_rate = if current_overhead > TARGET_OVERHEAD_PCT {
            // Reduce sampling rate
            let reduction = ((current_overhead - TARGET_OVERHEAD_PCT) as u16 * 10).min(50);
            current_rate.saturating_sub(reduction as u8).max(1)
        } else if current_overhead < TARGET_OVERHEAD_PCT && current_rate < 100 {
            // Increase sampling rate
            let increase = ((TARGET_OVERHEAD_PCT - current_overhead) as u16 * 5).min(20);
            current_rate.saturating_add(increase as u8).min(100)
        } else {
            current_rate
        };

        self.rate.store(new_rate, Ordering::Relaxed);
    }

    /// Set category sampling rate manually
    pub fn set_category_rate(&self, category_idx: usize, rate: u8) {
        if category_idx < self.category_rates.len() {
            self.category_rates[category_idx].store(rate.min(100), Ordering::Relaxed);
        }
    }

    /// Get current sampling rate
    #[inline]
    pub fn rate(&self) -> u8 {
        self.rate.load(Ordering::Relaxed)
    }

    /// Get acceptance rate (actual samples / evaluated)
    pub fn acceptance_rate(&self) -> f64 {
        let c = self.counters.read();
        if c.evaluated == 0 {
            1.0
        } else {
            c.accepted as f64 / c.evaluated as f64
        }
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.counters.replace(SamplerCounters {
            evaluated: 0,
            accepted: 0,
        });
    }

    /// Fast random number generator (xorshift)
    #[inline]
    fn fast_random(&self) -> u64 {
        // Thread-local xorshift state
        thread_local! {
            static STATE: std::cell::Cell<u64> = std::cell::Cell::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_nanos(1)) // Fallback for broken clocks
                    .as_nanos() as u64
            );
        }

        STATE.with(|state| {
            let mut x = state.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            state.set(x);
            x
        })
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Sampler {
    fn clone(&self) -> Self {
        Self {
            rate: Arc::clone(&self.rate),
            counters: self.counters.clone(),
            overhead_pct: Arc::clone(&self.overhead_pct),
            category_rates: [
                Arc::clone(&self.category_rates[0]),
                Arc::clone(&self.category_rates[1]),
                Arc::clone(&self.category_rates[2]),
                Arc::clone(&self.category_rates[3]),
                Arc::clone(&self.category_rates[4]),
                Arc::clone(&self.category_rates[5]),
                Arc::clone(&self.category_rates[6]),
                Arc::clone(&self.category_rates[7]),
                Arc::clone(&self.category_rates[8]),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_basic() {
        let sampler = Sampler::new();
        assert_eq!(sampler.rate(), 100);

        // Should accept at 100%
        assert_eq!(sampler.should_sample(), SampleDecision::Accept);
    }

    #[test]
    fn test_sampler_rate_adjustment() {
        let sampler = Sampler::new();

        // Simulate high overhead
        sampler.update_overhead(10);
        assert!(sampler.rate() < 100);

        // Simulate low overhead
        sampler.update_overhead(1);
        // Rate might increase (may take adjustment interval)
    }

    #[test]
    fn test_acceptance_rate() {
        let sampler = Sampler::new();

        for _ in 0..100 {
            sampler.should_sample();
        }

        let rate = sampler.acceptance_rate();
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_category_sampling() {
        let sampler = Sampler::new();

        // Set low rate for category 0
        sampler.set_category_rate(0, 10);

        let mut accepted = 0;
        let total = 1000;

        for _ in 0..total {
            if sampler.should_sample_category(0) == SampleDecision::Accept {
                accepted += 1;
            }
        }

        // Should accept roughly 10%
        let rate = accepted as f64 / total as f64;
        assert!(rate > 0.05 && rate < 0.20, "Rate: {}", rate);
    }

    #[test]
    fn test_reset() {
        let sampler = Sampler::new();

        for _ in 0..100 {
            sampler.should_sample();
        }

        sampler.reset();
        let c = sampler.counters.read();
        assert_eq!(c.evaluated, 0);
        assert_eq!(c.accepted, 0);
    }
}
