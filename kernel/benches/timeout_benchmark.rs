/*!
 * Timeout Executor Benchmarks
 *
 * Measures the performance impact of the microoptimized timeout infrastructure.
 */

use ai_os_kernel::core::guard::TimeoutPolicy;
use ai_os_kernel::syscalls::{TimeoutError, TimeoutExecutor};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
enum TestError {
    WouldBlock,
    Fatal,
}

/// Benchmark: Immediate success (no retries needed)
fn bench_immediate_success(c: &mut Criterion) {
    let executor = TimeoutExecutor::disabled(); // No timeout overhead

    c.bench_function("timeout/immediate_success", |b| {
        b.iter(|| {
            let result = executor.execute_with_retry(
                || Ok::<i32, TestError>(black_box(42)),
                |_| false,
                TimeoutPolicy::None,
                "test",
            );
            black_box(result)
        })
    });
}

/// Benchmark: Success after N retries (adaptive backoff)
fn bench_retry_then_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout/retry_adaptive_backoff");

    for retry_count in [1, 3, 5, 10, 20, 50].iter() {
        let executor = TimeoutExecutor::new(None); // Enabled but no observer overhead

        group.bench_with_input(
            BenchmarkId::from_parameter(retry_count),
            retry_count,
            |b, &retries| {
                b.iter(|| {
                    let counter = Arc::new(AtomicU32::new(0));
                    let counter_clone = counter.clone();

                    let result = executor.execute_with_retry(
                        || {
                            let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                            if count < retries {
                                Err(TestError::WouldBlock)
                            } else {
                                Ok(42)
                            }
                        },
                        |e| matches!(e, TestError::WouldBlock),
                        TimeoutPolicy::None, // No timeout
                        "test",
                    );
                    black_box(result)
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Comparison with old yield_now() approach
fn bench_retry_with_yield(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout/comparison");

    // Simulate old approach: yield_now() on every retry
    group.bench_function("old_yield_every_retry", |b| {
        b.iter(|| {
            let mut counter = 0u32;
            loop {
                if counter >= 5 {
                    break Ok::<i32, TestError>(42);
                }
                counter += 1;
                std::thread::yield_now(); // Old expensive approach
            }
        })
    });

    // New adaptive approach
    let executor = TimeoutExecutor::new(None);
    group.bench_function("new_adaptive_backoff", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            executor.execute_with_retry(
                || {
                    let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                    if count < 5 {
                        Err(TestError::WouldBlock)
                    } else {
                        Ok(42)
                    }
                },
                |e| matches!(e, TestError::WouldBlock),
                TimeoutPolicy::None,
                "test",
            )
        })
    });

    group.finish();
}

/// Benchmark: Deadline-based timeout (single operation)
fn bench_deadline_timeout(c: &mut Criterion) {
    let executor = TimeoutExecutor::new(None);

    c.bench_function("timeout/deadline_check", |b| {
        b.iter(|| {
            let result = executor.execute_with_deadline(
                || {
                    // Simulate fast operation
                    std::hint::black_box(42);
                    Ok::<i32, TestError>(42)
                },
                TimeoutPolicy::Io(Duration::from_secs(30)),
                "test_io",
            );
            black_box(result)
        })
    });
}

/// Benchmark: Timeout enforcement overhead (time checks)
fn bench_timeout_check_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout/check_overhead");

    // Without timeout (baseline)
    let executor_disabled = TimeoutExecutor::disabled();
    group.bench_function("disabled", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            executor_disabled.execute_with_retry(
                || {
                    let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                    if count < 10 {
                        Err(TestError::WouldBlock)
                    } else {
                        Ok(42)
                    }
                },
                |e| matches!(e, TestError::WouldBlock),
                TimeoutPolicy::None,
                "test",
            )
        })
    });

    // With timeout enabled (measures overhead)
    let executor_enabled = TimeoutExecutor::new(None);
    group.bench_function("enabled", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            executor_enabled.execute_with_retry(
                || {
                    let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                    if count < 10 {
                        Err(TestError::WouldBlock)
                    } else {
                        Ok(42)
                    }
                },
                |e| matches!(e, TestError::WouldBlock),
                TimeoutPolicy::Io(Duration::from_secs(30)),
                "test",
            )
        })
    });

    group.finish();
}

/// Benchmark: Spin vs Yield vs Sleep phases
fn bench_backoff_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout/backoff_phases");

    let executor = TimeoutExecutor::new(None);

    // Spin phase (0-15 retries)
    group.bench_function("spin_phase_10_retries", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            executor.execute_with_retry(
                || {
                    let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                    if count < 10 {
                        Err(TestError::WouldBlock)
                    } else {
                        Ok(42)
                    }
                },
                |e| matches!(e, TestError::WouldBlock),
                TimeoutPolicy::None,
                "test",
            )
        })
    });

    // Yield phase (16-99 retries)
    group.bench_function("yield_phase_50_retries", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            executor.execute_with_retry(
                || {
                    let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                    if count < 50 {
                        Err(TestError::WouldBlock)
                    } else {
                        Ok(42)
                    }
                },
                |e| matches!(e, TestError::WouldBlock),
                TimeoutPolicy::None,
                "test",
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_immediate_success,
    bench_retry_then_success,
    bench_retry_with_yield,
    bench_deadline_timeout,
    bench_timeout_check_overhead,
    bench_backoff_phases,
);

criterion_main!(benches);

