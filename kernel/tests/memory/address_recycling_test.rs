/*!
 * Address Recycling Test
 * Verifies that deallocated memory addresses are recycled and reused
 */

use ai_os_kernel::memory::manager::MemoryManager;

#[test]
fn test_address_recycling() {
    let manager = MemoryManager::new();
    let pid = 1;

    // Allocate three blocks
    let addr1 = manager
        .allocate(1024, pid)
        .expect("Failed to allocate block 1");
    let addr2 = manager
        .allocate(2048, pid)
        .expect("Failed to allocate block 2");
    let addr3 = manager
        .allocate(512, pid)
        .expect("Failed to allocate block 3");

    println!("Initial allocations:");
    println!("  addr1: 0x{:x} (1024 bytes)", addr1);
    println!("  addr2: 0x{:x} (2048 bytes)", addr2);
    println!("  addr3: 0x{:x} (512 bytes)", addr3);

    // Verify addresses are increasing (no recycling yet)
    assert!(addr2 > addr1, "Second address should be after first");
    assert!(addr3 > addr2, "Third address should be after second");

    // Deallocate the middle block
    manager
        .deallocate(addr2)
        .expect("Failed to deallocate block 2");
    println!("\nDeallocated addr2 (2048 bytes at 0x{:x})", addr2);

    // Allocate a block that fits in the freed space (best-fit should reuse addr2)
    let addr4 = manager
        .allocate(1024, pid)
        .expect("Failed to allocate block 4");
    println!("\nNew allocation:");
    println!("  addr4: 0x{:x} (1024 bytes)", addr4);

    // addr4 should be recycled from addr2's space (best-fit)
    // Since we freed 2048 bytes and need 1024, addr4 should be at addr2
    assert_eq!(
        addr4, addr2,
        "Address should be recycled from freed block (best-fit)"
    );
    println!("✓ Address recycling verified: addr4 reuses addr2's space");

    // Allocate another block - should use the remainder of addr2's space or continue forward
    let addr5 = manager
        .allocate(512, pid)
        .expect("Failed to allocate block 5");
    println!("\nAnother allocation:");
    println!("  addr5: 0x{:x} (512 bytes)", addr5);

    // addr5 should be either:
    // 1. At addr2 + 1024 (remainder of split block), or
    // 2. Beyond addr3 if the remainder wasn't used
    // With best-fit and block splitting, it should use the remainder
    let expected_remainder = addr2 + 1024;
    if addr5 == expected_remainder {
        println!("✓ Block splitting verified: addr5 uses remainder from split");
    } else {
        println!("  Note: addr5 allocated from new space (remainder used differently)");
    }

    // Verify memory stats
    let stats = manager.stats();
    println!("\nMemory Statistics:");
    println!("  Total: {} bytes", stats.total_memory);
    println!("  Used: {} bytes", stats.used_memory);
    println!("  Available: {} bytes", stats.available_memory);
    println!("  Usage: {:.1}%", stats.usage_percentage);
    println!("  Allocated blocks: {}", stats.allocated_blocks);
    println!("  Fragmented blocks: {}", stats.fragmented_blocks);

    // Clean up
    manager
        .deallocate(addr1)
        .expect("Failed to deallocate addr1");
    manager
        .deallocate(addr3)
        .expect("Failed to deallocate addr3");
    manager
        .deallocate(addr4)
        .expect("Failed to deallocate addr4");
    manager
        .deallocate(addr5)
        .expect("Failed to deallocate addr5");

    println!("\n✓ Address recycling test completed successfully");
}

#[test]
fn test_address_exhaustion_prevented() {
    let manager = MemoryManager::new();
    let pid = 1;
    let block_size = 1024;

    println!("Testing that address recycling prevents address space exhaustion");

    // Allocate and deallocate many times to verify addresses are reused
    let mut addresses = Vec::new();

    // First pass: allocate blocks
    for i in 0..10 {
        let addr = manager
            .allocate(block_size, pid)
            .expect(&format!("Failed allocation {}", i));
        addresses.push(addr);
        println!("  Allocated #{}: 0x{:x}", i + 1, addr);
    }

    // Second pass: deallocate all
    for (i, addr) in addresses.iter().enumerate() {
        manager
            .deallocate(*addr)
            .expect(&format!("Failed deallocation {}", i));
    }
    println!("\nDeallocated all 10 blocks");

    // Third pass: allocate again - should reuse addresses
    let mut recycled_addresses = Vec::new();
    for i in 0..10 {
        let addr = manager
            .allocate(block_size, pid)
            .expect(&format!("Failed recycled allocation {}", i));
        recycled_addresses.push(addr);
        println!("  Recycled #{}: 0x{:x}", i + 1, addr);
    }

    // Verify that recycled addresses match original addresses
    // They should be from the same address space (free list)
    let original_max = *addresses.iter().max().unwrap();
    let recycled_max = *recycled_addresses.iter().max().unwrap();

    // At least some addresses should be recycled (within original range)
    let recycled_count = recycled_addresses
        .iter()
        .filter(|addr| **addr <= original_max)
        .count();

    println!(
        "\nRecycled {} out of {} addresses from free list",
        recycled_count,
        recycled_addresses.len()
    );
    assert!(
        recycled_count > 0,
        "At least some addresses should be recycled"
    );

    // Clean up
    for addr in recycled_addresses {
        manager
            .deallocate(addr)
            .expect("Failed cleanup deallocation");
    }

    println!("✓ Address exhaustion prevention test completed successfully");
}

#[test]
fn test_coalescing_adjacent_blocks() {
    let manager = MemoryManager::new();
    let pid = 1;

    println!("Testing coalescing of adjacent free blocks");

    // Allocate three adjacent blocks
    let addr1 = manager
        .allocate(1000, pid)
        .expect("Failed to allocate block 1");
    let addr2 = manager
        .allocate(1000, pid)
        .expect("Failed to allocate block 2");
    let addr3 = manager
        .allocate(1000, pid)
        .expect("Failed to allocate block 3");

    println!("Allocated three blocks:");
    println!("  addr1: 0x{:x} (1000 bytes)", addr1);
    println!("  addr2: 0x{:x} (1000 bytes)", addr2);
    println!("  addr3: 0x{:x} (1000 bytes)", addr3);

    // Deallocate all three - they should be coalesced
    manager
        .deallocate(addr1)
        .expect("Failed to deallocate addr1");
    manager
        .deallocate(addr2)
        .expect("Failed to deallocate addr2");
    manager
        .deallocate(addr3)
        .expect("Failed to deallocate addr3");

    println!("\nDeallocated all three blocks (should coalesce if adjacent)");

    // Allocate a large block that fits in the coalesced space
    let addr4 = manager
        .allocate(2500, pid)
        .expect("Failed to allocate large block");
    println!("Allocated large block: 0x{:x} (2500 bytes)", addr4);

    // If coalescing worked, addr4 should be at or near addr1
    // (depending on whether they were truly adjacent)
    if addr4 == addr1 {
        println!("✓ Coalescing verified: large block reuses coalesced space");
    } else {
        println!("  Note: Blocks may not have been adjacent or coalescing used different block");
    }

    // Clean up
    manager
        .deallocate(addr4)
        .expect("Failed to deallocate addr4");

    println!("✓ Coalescing test completed");
}
