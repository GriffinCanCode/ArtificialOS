/*!
 * JSON Serialization Benchmarks
 * Verify 2-4x performance improvement with simd-json
 */

use ai_os_kernel::core::json;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};

// Test data structures matching real syscall types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SyscallResult {
    status: String,
    data: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct IpcMessage {
    id: u64,
    from: u32,
    to: u32,
    data: Vec<u8>,
    priority: u8,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct VfsEntry {
    name: String,
    size: u64,
    file_type: String,
    modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    cpu_time: u64,
    memory: u64,
}

// Small payload benchmark (<1KB) - should use serde_json
fn bench_small_payload_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_payload_serialize");

    let small_result = SyscallResult {
        status: "success".to_string(),
        data: vec![1, 2, 3, 4, 5],
        timestamp: 1234567890,
    };

    group.throughput(Throughput::Bytes(
        serde_json::to_vec(&small_result).unwrap().len() as u64
    ));

    group.bench_function("optimized", |b| {
        b.iter(|| json::to_vec(black_box(&small_result)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&small_result)).unwrap());
    });

    group.finish();
}

// Medium payload benchmark (~2KB) - should use simd-json
fn bench_medium_payload_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_payload_serialize");

    let medium_message = IpcMessage {
        id: 123456,
        from: 1000,
        to: 2000,
        data: vec![0u8; 2048],
        priority: 5,
        timestamp: 1234567890,
    };

    let size = serde_json::to_vec(&medium_message).unwrap().len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("optimized", |b| {
        b.iter(|| json::to_vec(black_box(&medium_message)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&medium_message)).unwrap());
    });

    group.bench_function("simd_json_direct", |b| {
        b.iter(|| json::to_vec_simd(black_box(&medium_message)).unwrap());
    });

    group.finish();
}

// Large payload benchmark (~10KB) - should show best SIMD gains
fn bench_large_payload_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_payload_serialize");

    let large_batch: Vec<VfsEntry> = (0..100)
        .map(|i| VfsEntry {
            name: format!("file_{}.txt", i),
            size: 1024 * (i as u64),
            file_type: if i % 2 == 0 { "file" } else { "directory" }.to_string(),
            modified: 1234567890 + i as u64,
        })
        .collect();

    let size = serde_json::to_vec(&large_batch).unwrap().len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("optimized", |b| {
        b.iter(|| json::to_vec(black_box(&large_batch)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&large_batch)).unwrap());
    });

    group.bench_function("simd_json_direct", |b| {
        b.iter(|| json::to_vec_simd(black_box(&large_batch)).unwrap());
    });

    group.finish();
}

// Very large payload benchmark (~50KB) - maximum SIMD benefit
fn bench_xlarge_payload_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("xlarge_payload_serialize");

    let xlarge_list: Vec<ProcessInfo> = (0..500)
        .map(|i| ProcessInfo {
            pid: 1000 + i,
            name: format!("process_{}", i),
            state: "running".to_string(),
            cpu_time: 1000 * i as u64,
            memory: 1024 * 1024 * (i as u64),
        })
        .collect();

    let size = serde_json::to_vec(&xlarge_list).unwrap().len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("optimized", |b| {
        b.iter(|| json::to_vec(black_box(&xlarge_list)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&xlarge_list)).unwrap());
    });

    group.bench_function("simd_json_direct", |b| {
        b.iter(|| json::to_vec_simd(black_box(&xlarge_list)).unwrap());
    });

    group.finish();
}

// Deserialization benchmarks
fn bench_large_payload_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_payload_deserialize");

    let large_batch: Vec<VfsEntry> = (0..100)
        .map(|i| VfsEntry {
            name: format!("file_{}.txt", i),
            size: 1024 * (i as u64),
            file_type: if i % 2 == 0 { "file" } else { "directory" }.to_string(),
            modified: 1234567890 + i as u64,
        })
        .collect();

    let json_bytes = serde_json::to_vec(&large_batch).unwrap();
    let size = json_bytes.len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("optimized", |b| {
        b.iter(|| json::from_slice::<Vec<VfsEntry>>(black_box(&json_bytes)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_slice::<Vec<VfsEntry>>(black_box(&json_bytes)).unwrap());
    });

    group.bench_function("simd_json_direct", |b| {
        b.iter(|| json::from_slice_simd::<Vec<VfsEntry>>(black_box(&json_bytes)).unwrap());
    });

    group.finish();
}

// Benchmark different payload sizes
fn bench_payload_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_size_scaling");

    for size in [256, 512, 1024, 2048, 4096, 8192, 16384].iter() {
        let data = vec![0u8; *size];
        let message = IpcMessage {
            id: 123456,
            from: 1000,
            to: 2000,
            data,
            priority: 5,
            timestamp: 1234567890,
        };

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, _| {
            b.iter(|| json::to_vec(black_box(&message)).unwrap());
        });

        group.bench_with_input(BenchmarkId::new("serde_json", size), size, |b, _| {
            b.iter(|| serde_json::to_vec(black_box(&message)).unwrap());
        });

        if *size > 1024 {
            group.bench_with_input(BenchmarkId::new("simd_json", size), size, |b, _| {
                b.iter(|| json::to_vec_simd(black_box(&message)).unwrap());
            });
        }
    }

    group.finish();
}

// Real-world syscall result benchmark
fn bench_syscall_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("syscall_result");

    let result = SyscallResult {
        status: "success".to_string(),
        data: vec![0u8; 4096],
        timestamp: 1234567890,
    };

    let size = serde_json::to_vec(&result).unwrap().len();
    group.throughput(Throughput::Bytes(size as u64));

    group.bench_function("serialize_syscall_result", |b| {
        b.iter(|| json::serialize_syscall_result(black_box(&result)));
    });

    group.bench_function("optimized", |b| {
        b.iter(|| json::to_vec(black_box(&result)).unwrap());
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&result)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_payload_serialize,
    bench_medium_payload_serialize,
    bench_large_payload_serialize,
    bench_xlarge_payload_serialize,
    bench_large_payload_deserialize,
    bench_payload_size_scaling,
    bench_syscall_result,
);

criterion_main!(benches);
