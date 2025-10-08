/*!
 * Bincode vs JSON Benchmark
 * Compares serialization performance for IPC types
 */

use ai_os_kernel::ipc::types::Message;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn create_message(size: usize) -> Message {
    Message::new(1, 2, vec![0u8; size])
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    for size in [100, 1000, 10000, 100000].iter() {
        let msg = create_message(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        // JSON serialization
        group.bench_with_input(BenchmarkId::new("json", size), size, |b, _| {
            b.iter(|| {
                let bytes = serde_json::to_vec(black_box(&msg)).unwrap();
                black_box(bytes);
            });
        });

        // Bincode serialization
        group.bench_with_input(BenchmarkId::new("bincode", size), size, |b, _| {
            b.iter(|| {
                let bytes = bincode::serialize(black_box(&msg)).unwrap();
                black_box(bytes);
            });
        });
    }

    group.finish();
}

fn benchmark_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");

    for size in [100, 1000, 10000, 100000].iter() {
        let msg = create_message(*size);

        let json_bytes = serde_json::to_vec(&msg).unwrap();
        let bincode_bytes = bincode::serialize(&msg).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));

        // JSON deserialization
        group.bench_with_input(BenchmarkId::new("json", size), size, |b, _| {
            b.iter(|| {
                let msg: Message = serde_json::from_slice(black_box(&json_bytes)).unwrap();
                black_box(msg);
            });
        });

        // Bincode deserialization
        group.bench_with_input(BenchmarkId::new("bincode", size), size, |b, _| {
            b.iter(|| {
                let msg: Message = bincode::deserialize(black_box(&bincode_bytes)).unwrap();
                black_box(msg);
            });
        });
    }

    group.finish();
}

fn benchmark_size(c: &mut Criterion) {
    let group = c.benchmark_group("payload_size");

    for size in [100, 1000, 10000, 100000].iter() {
        let msg = create_message(*size);

        let json_size = serde_json::to_vec(&msg).unwrap().len();
        let bincode_size = bincode::serialize(&msg).unwrap().len();

        println!(
            "Size {}: JSON = {} bytes, Bincode = {} bytes, Ratio = {:.2}x",
            size,
            json_size,
            bincode_size,
            json_size as f64 / bincode_size as f64
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_serialization,
    benchmark_deserialization,
    benchmark_size
);
criterion_main!(benches);
