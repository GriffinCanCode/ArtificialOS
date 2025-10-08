/*!
 * Synchronization Primitives Benchmarks
 *
 * Compare performance of futex, condvar, and spinwait strategies
 */

use ai_os_kernel::core::sync::{WaitQueue, SyncConfig, StrategyType};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn bench_wake_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("wake_latency");

    for strategy in [StrategyType::Futex, StrategyType::Condvar, StrategyType::SpinWait] {
        let config = SyncConfig {
            strategy,
            spin_duration: Duration::from_micros(10),
            max_spins: 100,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", strategy)),
            &config,
            |b, config| {
                b.iter(|| {
                    let queue = Arc::new(WaitQueue::<u64>::new(config.clone()));
                    let queue_clone = queue.clone();

                    let handle = thread::spawn(move || {
                        queue_clone.wait(1, Some(Duration::from_secs(1)))
                    });

                    // Immediate wake
                    queue.wake_one(1);
                    handle.join().unwrap().ok();
                });
            },
        );
    }

    group.finish();
}

fn bench_throughput_single_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_single_key");

    for strategy in [StrategyType::Futex, StrategyType::Condvar] {
        let config = SyncConfig {
            strategy,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", strategy)),
            &config,
            |b, config| {
                let queue = Arc::new(WaitQueue::<u64>::new(config.clone()));

                b.iter(|| {
                    let queue_clone = queue.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..100 {
                            queue_clone.wait(42, Some(Duration::from_millis(100))).ok();
                        }
                    });

                    for _ in 0..100 {
                        queue.wake_one(42);
                    }

                    handle.join().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_multi_waiter_wake(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_waiter_wake");

    for num_waiters in [1, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_waiters),
            &num_waiters,
            |b, &num_waiters| {
                b.iter(|| {
                    let queue = Arc::new(WaitQueue::<u64>::with_defaults());

                    let handles: Vec<_> = (0..num_waiters)
                        .map(|_| {
                            let queue_clone = queue.clone();
                            thread::spawn(move || {
                                queue_clone.wait(100, Some(Duration::from_secs(1)))
                            })
                        })
                        .collect();

                    // Give threads time to park
                    thread::sleep(Duration::from_millis(10));

                    // Wake all
                    queue.wake_all(100);

                    for handle in handles {
                        handle.join().unwrap().ok();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_contended_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("contended_keys");

    for num_keys in [1, 10, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_keys),
            &num_keys,
            |b, &num_keys| {
                b.iter(|| {
                    let queue = Arc::new(WaitQueue::<u64>::with_defaults());

                    let handles: Vec<_> = (0..num_keys)
                        .map(|i| {
                            let queue_clone = queue.clone();
                            thread::spawn(move || {
                                queue_clone.wait(i, Some(Duration::from_millis(100)))
                            })
                        })
                        .collect();

                    thread::sleep(Duration::from_millis(10));

                    // Wake all keys
                    for i in 0..num_keys {
                        queue.wake_one(i);
                    }

                    for handle in handles {
                        handle.join().unwrap().ok();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_spin_vs_park(c: &mut Criterion) {
    let mut group = c.benchmark_group("spin_vs_park");

    for delay_us in [1, 10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("spinwait", delay_us),
            &delay_us,
            |b, &delay_us| {
                let config = SyncConfig {
                    strategy: StrategyType::SpinWait,
                    spin_duration: Duration::from_micros(delay_us),
                    max_spins: 1000,
                };
                let queue = Arc::new(WaitQueue::<u64>::new(config));

                b.iter(|| {
                    let queue_clone = queue.clone();
                    let handle = thread::spawn(move || {
                        queue_clone.wait(200, Some(Duration::from_millis(100)))
                    });

                    thread::sleep(Duration::from_micros(delay_us / 2));
                    queue.wake_one(200);
                    handle.join().unwrap().ok();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("futex", delay_us),
            &delay_us,
            |b, &delay_us| {
                let config = SyncConfig {
                    strategy: StrategyType::Futex,
                    ..Default::default()
                };
                let queue = Arc::new(WaitQueue::<u64>::new(config));

                b.iter(|| {
                    let queue_clone = queue.clone();
                    let handle = thread::spawn(move || {
                        queue_clone.wait(200, Some(Duration::from_millis(100)))
                    });

                    thread::sleep(Duration::from_micros(delay_us / 2));
                    queue.wake_one(200);
                    handle.join().unwrap().ok();
                });
            },
        );
    }

    group.finish();
}

fn bench_no_contention_overhead(c: &mut Criterion) {
    c.bench_function("wake_no_waiters", |b| {
        let queue = WaitQueue::<u64>::with_defaults();

        b.iter(|| {
            // Wake with no waiters (should be fast)
            black_box(queue.wake_one(999));
        });
    });
}

criterion_group!(
    benches,
    bench_wake_latency,
    bench_throughput_single_key,
    bench_multi_waiter_wake,
    bench_contended_keys,
    bench_spin_vs_park,
    bench_no_contention_overhead
);

criterion_main!(benches);
