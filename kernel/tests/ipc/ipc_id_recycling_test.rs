/*!
 * IPC ID Recycling Test
 * Verifies that deallocated IPC resource IDs (pipes, shared memory, queues) are recycled and reused
 * This prevents ID exhaustion which would occur at ~71 minutes for u32 at 1 alloc/μs
 */

use ai_os_kernel::ipc::core::manager::IPCManager;
use ai_os_kernel::ipc::core::types::QueueType;
use ai_os_kernel::memory::manager::MemoryManager;

#[test]
fn test_pipe_id_recycling() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    let reader_pid = 1;
    let writer_pid = 2;

    println!("Testing Pipe ID recycling");

    // Create three pipes
    let pipe1 = ipc_manager
        .pipes()
        .create(reader_pid, writer_pid, None)
        .expect("Failed to create pipe 1");
    let pipe2 = ipc_manager
        .pipes()
        .create(reader_pid, writer_pid, None)
        .expect("Failed to create pipe 2");
    let pipe3 = ipc_manager
        .pipes()
        .create(reader_pid, writer_pid, None)
        .expect("Failed to create pipe 3");

    println!("Created pipes: {}, {}, {}", pipe1, pipe2, pipe3);

    // Verify IDs are increasing (no recycling yet)
    assert!(pipe2 > pipe1, "Second pipe ID should be after first");
    assert!(pipe3 > pipe2, "Third pipe ID should be after second");

    // Destroy the middle pipe
    ipc_manager
        .pipes()
        .destroy(pipe2)
        .expect("Failed to destroy pipe 2");
    println!("Destroyed pipe {}", pipe2);

    // Create a new pipe - should recycle pipe2's ID
    let pipe4 = ipc_manager
        .pipes()
        .create(reader_pid, writer_pid, None)
        .expect("Failed to create pipe 4");
    println!("Created new pipe: {}", pipe4);

    // pipe4 should be recycled from pipe2
    assert_eq!(
        pipe4, pipe2,
        "Pipe ID should be recycled from destroyed pipe"
    );
    println!("✓ Pipe ID recycling verified: pipe4 reuses pipe2's ID");

    // Clean up
    ipc_manager
        .pipes()
        .destroy(pipe1)
        .expect("Failed to destroy pipe1");
    ipc_manager
        .pipes()
        .destroy(pipe3)
        .expect("Failed to destroy pipe3");
    ipc_manager
        .pipes()
        .destroy(pipe4)
        .expect("Failed to destroy pipe4");

    println!("✓ Pipe ID recycling test completed successfully");
}

#[test]
fn test_shm_id_recycling() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    let owner_pid = 1;
    let size = 4096;

    println!("Testing Shared Memory ID recycling");

    // Create three segments
    let seg1 = ipc_manager
        .shm()
        .create(size, owner_pid)
        .expect("Failed to create segment 1");
    let seg2 = ipc_manager
        .shm()
        .create(size, owner_pid)
        .expect("Failed to create segment 2");
    let seg3 = ipc_manager
        .shm()
        .create(size, owner_pid)
        .expect("Failed to create segment 3");

    println!("Created segments: {}, {}, {}", seg1, seg2, seg3);

    // Verify IDs are increasing (no recycling yet)
    assert!(seg2 > seg1, "Second segment ID should be after first");
    assert!(seg3 > seg2, "Third segment ID should be after second");

    // Destroy the middle segment
    ipc_manager
        .shm()
        .destroy(seg2, owner_pid)
        .expect("Failed to destroy segment 2");
    println!("Destroyed segment {}", seg2);

    // Create a new segment - should recycle seg2's ID
    let seg4 = ipc_manager
        .shm()
        .create(size, owner_pid)
        .expect("Failed to create segment 4");
    println!("Created new segment: {}", seg4);

    // seg4 should be recycled from seg2
    assert_eq!(
        seg4, seg2,
        "Segment ID should be recycled from destroyed segment"
    );
    println!("✓ Shared memory ID recycling verified: seg4 reuses seg2's ID");

    // Clean up
    ipc_manager
        .shm()
        .destroy(seg1, owner_pid)
        .expect("Failed to destroy seg1");
    ipc_manager
        .shm()
        .destroy(seg3, owner_pid)
        .expect("Failed to destroy seg3");
    ipc_manager
        .shm()
        .destroy(seg4, owner_pid)
        .expect("Failed to destroy seg4");

    println!("✓ Shared memory ID recycling test completed successfully");
}

#[test]
fn test_queue_id_recycling() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    let owner_pid = 1;

    println!("Testing Queue ID recycling");

    // Create three queues
    let queue1 = ipc_manager
        .queues()
        .create(owner_pid, QueueType::Fifo, None)
        .expect("Failed to create queue 1");
    let queue2 = ipc_manager
        .queues()
        .create(owner_pid, QueueType::Fifo, None)
        .expect("Failed to create queue 2");
    let queue3 = ipc_manager
        .queues()
        .create(owner_pid, QueueType::Fifo, None)
        .expect("Failed to create queue 3");

    println!("Created queues: {}, {}, {}", queue1, queue2, queue3);

    // Verify IDs are increasing (no recycling yet)
    assert!(queue2 > queue1, "Second queue ID should be after first");
    assert!(queue3 > queue2, "Third queue ID should be after second");

    // Destroy the middle queue
    ipc_manager
        .queues()
        .destroy(queue2, owner_pid)
        .expect("Failed to destroy queue 2");
    println!("Destroyed queue {}", queue2);

    // Create a new queue - should recycle queue2's ID
    let queue4 = ipc_manager
        .queues()
        .create(owner_pid, QueueType::Fifo, None)
        .expect("Failed to create queue 4");
    println!("Created new queue: {}", queue4);

    // queue4 should be recycled from queue2
    assert_eq!(
        queue4, queue2,
        "Queue ID should be recycled from destroyed queue"
    );
    println!("✓ Queue ID recycling verified: queue4 reuses queue2's ID");

    // Clean up
    ipc_manager
        .queues()
        .destroy(queue1, owner_pid)
        .expect("Failed to destroy queue1");
    ipc_manager
        .queues()
        .destroy(queue3, owner_pid)
        .expect("Failed to destroy queue3");
    ipc_manager
        .queues()
        .destroy(queue4, owner_pid)
        .expect("Failed to destroy queue4");

    println!("✓ Queue ID recycling test completed successfully");
}

#[test]
fn test_ipc_id_exhaustion_prevention() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    let pid = 1;
    let iterations = 100;

    println!("Testing that IPC ID recycling prevents ID space exhaustion");
    println!(
        "Allocating and deallocating {} times to verify IDs are reused\n",
        iterations
    );

    // Test pipes
    {
        let mut pipe_ids = Vec::new();

        // First pass: create pipes
        for i in 0..iterations {
            let pipe_id = ipc_manager
                .pipes()
                .create(pid, pid, None)
                .expect(&format!("Failed to create pipe {}", i));
            pipe_ids.push(pipe_id);
        }
        println!(
            "  Created {} pipes (IDs 1-{})",
            iterations,
            pipe_ids.last().unwrap()
        );

        // Second pass: destroy all
        for pipe_id in &pipe_ids {
            ipc_manager
                .pipes()
                .destroy(*pipe_id)
                .expect(&format!("Failed to destroy pipe {}", pipe_id));
        }
        println!("  Destroyed all {} pipes", iterations);

        // Third pass: create again - should reuse IDs
        let mut recycled_ids = Vec::new();
        for i in 0..iterations {
            let pipe_id = ipc_manager
                .pipes()
                .create(pid, pid, None)
                .expect(&format!("Failed to create recycled pipe {}", i));
            recycled_ids.push(pipe_id);
        }
        println!(
            "  Recreated {} pipes (IDs 1-{})",
            iterations,
            recycled_ids.last().unwrap()
        );

        // Verify that recycled IDs are from the same space (should be all recycled)
        let original_max = *pipe_ids.iter().max().unwrap();
        let recycled_count = recycled_ids
            .iter()
            .filter(|id| **id <= original_max)
            .count();

        println!(
            "  Recycled {} out of {} pipe IDs from free list",
            recycled_count, iterations
        );
        assert!(
            recycled_count > 0,
            "At least some pipe IDs should be recycled"
        );

        // Clean up
        for pipe_id in recycled_ids {
            ipc_manager
                .pipes()
                .destroy(pipe_id)
                .expect("Failed to cleanup pipe");
        }
        println!("✓ Pipe ID exhaustion prevention verified\n");
    }

    // Test shared memory
    {
        let size = 1024;
        let mut shm_ids = Vec::new();

        // First pass: create segments
        for i in 0..iterations {
            let shm_id = ipc_manager
                .shm()
                .create(size, pid)
                .expect(&format!("Failed to create segment {}", i));
            shm_ids.push(shm_id);
        }
        println!(
            "  Created {} segments (IDs 1-{})",
            iterations,
            shm_ids.last().unwrap()
        );

        // Second pass: destroy all
        for shm_id in &shm_ids {
            ipc_manager
                .shm()
                .destroy(*shm_id, pid)
                .expect(&format!("Failed to destroy segment {}", shm_id));
        }
        println!("  Destroyed all {} segments", iterations);

        // Third pass: create again - should reuse IDs
        let mut recycled_ids = Vec::new();
        for i in 0..iterations {
            let shm_id = ipc_manager
                .shm()
                .create(size, pid)
                .expect(&format!("Failed to create recycled segment {}", i));
            recycled_ids.push(shm_id);
        }
        println!(
            "  Recreated {} segments (IDs 1-{})",
            iterations,
            recycled_ids.last().unwrap()
        );

        // Verify that recycled IDs are from the same space
        let original_max = *shm_ids.iter().max().unwrap();
        let recycled_count = recycled_ids
            .iter()
            .filter(|id| **id <= original_max)
            .count();

        println!(
            "  Recycled {} out of {} segment IDs from free list",
            recycled_count, iterations
        );
        assert!(
            recycled_count > 0,
            "At least some segment IDs should be recycled"
        );

        // Clean up
        for shm_id in recycled_ids {
            ipc_manager
                .shm()
                .destroy(shm_id, pid)
                .expect("Failed to cleanup segment");
        }
        println!("✓ Shared memory ID exhaustion prevention verified\n");
    }

    // Test queues
    {
        let mut queue_ids = Vec::new();

        // First pass: create queues
        for i in 0..iterations {
            let queue_id = ipc_manager
                .queues()
                .create(pid, QueueType::Fifo, None)
                .expect(&format!("Failed to create queue {}", i));
            queue_ids.push(queue_id);
        }
        println!(
            "  Created {} queues (IDs 1-{})",
            iterations,
            queue_ids.last().unwrap()
        );

        // Second pass: destroy all
        for queue_id in &queue_ids {
            ipc_manager
                .queues()
                .destroy(*queue_id, pid)
                .expect(&format!("Failed to destroy queue {}", queue_id));
        }
        println!("  Destroyed all {} queues", iterations);

        // Third pass: create again - should reuse IDs
        let mut recycled_ids = Vec::new();
        for i in 0..iterations {
            let queue_id = ipc_manager
                .queues()
                .create(pid, QueueType::Fifo, None)
                .expect(&format!("Failed to create recycled queue {}", i));
            recycled_ids.push(queue_id);
        }
        println!(
            "  Recreated {} queues (IDs 1-{})",
            iterations,
            recycled_ids.last().unwrap()
        );

        // Verify that recycled IDs are from the same space
        let original_max = *queue_ids.iter().max().unwrap();
        let recycled_count = recycled_ids
            .iter()
            .filter(|id| **id <= original_max)
            .count();

        println!(
            "  Recycled {} out of {} queue IDs from free list",
            recycled_count, iterations
        );
        assert!(
            recycled_count > 0,
            "At least some queue IDs should be recycled"
        );

        // Clean up
        for queue_id in recycled_ids {
            ipc_manager
                .queues()
                .destroy(queue_id, pid)
                .expect("Failed to cleanup queue");
        }
        println!("✓ Queue ID exhaustion prevention verified\n");
    }

    println!("✓ IPC ID exhaustion prevention test completed successfully");
}

#[test]
fn test_mixed_ipc_id_recycling() {
    let memory_manager = MemoryManager::new();
    let ipc_manager = IPCManager::new(memory_manager);

    let pid = 1;

    println!("Testing mixed IPC resource ID recycling");
    println!("Creating and destroying pipes, segments, and queues in random order\n");

    // Create a mix of resources
    let pipe1 = ipc_manager
        .pipes()
        .create(pid, pid, None)
        .expect("Failed to create pipe1");
    let shm1 = ipc_manager
        .shm()
        .create(1024, pid)
        .expect("Failed to create shm1");
    let queue1 = ipc_manager
        .queues()
        .create(pid, QueueType::Fifo, None)
        .expect("Failed to create queue1");
    let pipe2 = ipc_manager
        .pipes()
        .create(pid, pid, None)
        .expect("Failed to create pipe2");
    let shm2 = ipc_manager
        .shm()
        .create(2048, pid)
        .expect("Failed to create shm2");
    let queue2 = ipc_manager
        .queues()
        .create(pid, QueueType::Priority, None)
        .expect("Failed to create queue2");

    println!("Created resources:");
    println!("  Pipes: {}, {}", pipe1, pipe2);
    println!("  Segments: {}, {}", shm1, shm2);
    println!("  Queues: {}, {}", queue1, queue2);

    // Destroy first of each
    ipc_manager
        .pipes()
        .destroy(pipe1)
        .expect("Failed to destroy pipe1");
    ipc_manager
        .shm()
        .destroy(shm1, pid)
        .expect("Failed to destroy shm1");
    ipc_manager
        .queues()
        .destroy(queue1, pid)
        .expect("Failed to destroy queue1");

    println!("\nDestroyed first of each resource type");

    // Create new resources - should recycle
    let pipe3 = ipc_manager
        .pipes()
        .create(pid, pid, None)
        .expect("Failed to create pipe3");
    let shm3 = ipc_manager
        .shm()
        .create(1024, pid)
        .expect("Failed to create shm3");
    let queue3 = ipc_manager
        .queues()
        .create(pid, QueueType::Fifo, None)
        .expect("Failed to create queue3");

    println!("\nCreated new resources (should be recycled):");
    println!("  Pipe: {} (expected {})", pipe3, pipe1);
    println!("  Segment: {} (expected {})", shm3, shm1);
    println!("  Queue: {} (expected {})", queue3, queue1);

    // Verify recycling
    assert_eq!(pipe3, pipe1, "Pipe ID should be recycled");
    assert_eq!(shm3, shm1, "Segment ID should be recycled");
    assert_eq!(queue3, queue1, "Queue ID should be recycled");

    println!("\n✓ All IDs successfully recycled");

    // Clean up
    ipc_manager
        .pipes()
        .destroy(pipe2)
        .expect("Failed to destroy pipe2");
    ipc_manager
        .pipes()
        .destroy(pipe3)
        .expect("Failed to destroy pipe3");
    ipc_manager
        .shm()
        .destroy(shm2, pid)
        .expect("Failed to destroy shm2");
    ipc_manager
        .shm()
        .destroy(shm3, pid)
        .expect("Failed to destroy shm3");
    ipc_manager
        .queues()
        .destroy(queue2, pid)
        .expect("Failed to destroy queue2");
    ipc_manager
        .queues()
        .destroy(queue3, pid)
        .expect("Failed to destroy queue3");

    println!("✓ Mixed IPC ID recycling test completed successfully");
}
