/*!
 * Streaming Syscall Tests
 * Tests for streaming file operations
 */

use ai_os_kernel::api::streaming::StreamingManager;
use ai_os_kernel::security::{Capability, SandboxConfig, SandboxManager};
use ai_os_kernel::syscalls::SyscallExecutor;
use futures::StreamExt;
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> (SyscallExecutor, SandboxManager, TempDir, u32) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    let mut config = SandboxConfig::standard(pid);
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    (executor, sandbox_manager, temp_dir, pid)
}

#[tokio::test]
async fn test_stream_file_read_small() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Hello, streaming world!").unwrap();

    let stream = streaming_manager
        .stream_file_read(pid, test_file, Some(8))
        .await
        .expect("Stream should be created");

    tokio::pin!(stream);

    let mut chunks = vec![];
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => chunks.push(data),
            Err(e) => panic!("Stream error: {}", e),
        }
    }

    // Verify data
    let full_data: Vec<u8> = chunks.into_iter().flatten().collect();
    assert_eq!(full_data, b"Hello, streaming world!");
}

#[tokio::test]
async fn test_stream_file_read_large_chunks() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    // Create a larger file
    let test_file = temp_dir.path().join("large.txt");
    let test_data = vec![b'X'; 1024 * 100]; // 100KB
    fs::write(&test_file, &test_data).unwrap();

    let stream = streaming_manager
        .stream_file_read(pid, test_file, Some(64 * 1024)) // 64KB chunks
        .await
        .expect("Stream should be created");

    tokio::pin!(stream);

    let mut total_bytes = 0;
    let mut chunk_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                total_bytes += data.len();
                chunk_count += 1;
            }
            Err(e) => panic!("Stream error: {}", e),
        }
    }

    assert_eq!(total_bytes, 1024 * 100);
    assert!(chunk_count >= 2, "Should have multiple chunks");
}

#[tokio::test]
async fn test_stream_file_read_nonexistent() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let nonexistent = temp_dir.path().join("nonexistent.txt");

    let result = streaming_manager
        .stream_file_read(pid, nonexistent, None)
        .await;

    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[tokio::test]
async fn test_stream_file_write_small() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let test_file = temp_dir.path().join("output.txt");
    let test_data = vec![b"chunk1".to_vec(), b"chunk2".to_vec(), b"chunk3".to_vec()];

    let data_stream = futures::stream::iter(test_data.clone());

    let total_bytes = streaming_manager
        .stream_file_write(pid, test_file.clone(), data_stream)
        .await
        .expect("Write should succeed");

    assert_eq!(total_bytes, 18); // 6 + 6 + 6

    // Verify file content
    let content = fs::read(&test_file).unwrap();
    assert_eq!(content, b"chunk1chunk2chunk3");
}

#[tokio::test]
async fn test_stream_file_write_large() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let test_file = temp_dir.path().join("large_output.txt");

    // Create 10 chunks of 10KB each
    let chunks: Vec<Vec<u8>> = (0..10)
        .map(|_| vec![b'Y'; 10 * 1024])
        .collect();

    let data_stream = futures::stream::iter(chunks);

    let total_bytes = streaming_manager
        .stream_file_write(pid, test_file.clone(), data_stream)
        .await
        .expect("Write should succeed");

    assert_eq!(total_bytes, 10 * 10 * 1024);

    // Verify file size
    let metadata = fs::metadata(&test_file).unwrap();
    assert_eq!(metadata.len(), 10 * 10 * 1024);
}

#[tokio::test]
async fn test_stream_file_write_empty() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let test_file = temp_dir.path().join("empty.txt");
    let data_stream = futures::stream::iter(Vec::<Vec<u8>>::new());

    let total_bytes = streaming_manager
        .stream_file_write(pid, test_file.clone(), data_stream)
        .await
        .expect("Write should succeed");

    assert_eq!(total_bytes, 0);
    assert!(test_file.exists());
    assert_eq!(fs::read(&test_file).unwrap().len(), 0);
}

#[tokio::test]
async fn test_stream_read_default_chunk_size() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor);

    let test_file = temp_dir.path().join("test.txt");
    let test_data = vec![b'Z'; 128 * 1024]; // 128KB
    fs::write(&test_file, &test_data).unwrap();

    // Use default chunk size (64KB)
    let stream = streaming_manager
        .stream_file_read(pid, test_file, None)
        .await
        .expect("Stream should be created");

    tokio::pin!(stream);

    let mut total_bytes = 0;
    let mut first_chunk_size = None;

    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                if first_chunk_size.is_none() {
                    first_chunk_size = Some(data.len());
                }
                total_bytes += data.len();
            }
            Err(e) => panic!("Stream error: {}", e),
        }
    }

    assert_eq!(total_bytes, 128 * 1024);
    // First chunk should be default size (64KB)
    assert_eq!(first_chunk_size, Some(64 * 1024));
}

#[tokio::test]
async fn test_stream_round_trip() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let streaming_manager = StreamingManager::new(executor.clone());

    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.txt");

    // Write test data
    let original_data = b"Round trip test data for streaming!";
    fs::write(&input_file, original_data).unwrap();

    // Stream read
    let read_stream = streaming_manager
        .stream_file_read(pid, input_file, Some(10))
        .await
        .expect("Read stream should be created");

    tokio::pin!(read_stream);

    // Collect chunks
    let mut chunks = vec![];
    while let Some(result) = read_stream.next().await {
        chunks.push(result.expect("Should read successfully"));
    }

    // Stream write
    let write_stream = futures::stream::iter(chunks);
    let streaming_manager2 = StreamingManager::new(executor);

    streaming_manager2
        .stream_file_write(pid, output_file.clone(), write_stream)
        .await
        .expect("Write should succeed");

    // Verify
    let output_data = fs::read(&output_file).unwrap();
    assert_eq!(output_data, original_data);
}
