/*!
 * Segregated Free List
 * Efficient memory allocation data structure
 */

use crate::core::types::{Address, Size};
use std::collections::BTreeMap;

/// Free block for address recycling
#[derive(Debug, Clone)]
pub(super) struct FreeBlock {
    pub address: Address,
    pub size: Size,
}

/// Size classes for segregated free lists
/// Modern memory allocators use segregated lists for O(1) lookup
pub(super) const SMALL_BLOCK_MAX: Size = 4 * 1024; // 4KB
pub(super) const MEDIUM_BLOCK_MAX: Size = 64 * 1024; // 64KB

/// Segregated free list for efficient memory allocation
/// - Small blocks (<4KB): O(1) best-fit within size class
/// - Medium blocks (4KB-64KB): O(1) best-fit within size class
/// - Large blocks (>64KB): O(log n) using BTreeMap
#[derive(Debug)]
pub(super) struct SegregatedFreeList {
    /// Small allocations: <4KB - most common case
    /// Bucketed by powers of 2 for cache-friendly access
    small_blocks: Vec<Vec<FreeBlock>>, // 12 buckets: 64, 128, 256, ..., 2048, 4096 bytes

    /// Medium allocations: 4KB-64KB
    /// Bucketed by 4KB increments
    medium_blocks: Vec<Vec<FreeBlock>>, // 15 buckets: 8KB, 12KB, 16KB, ..., 64KB

    /// Large allocations: >64KB
    /// BTreeMap for O(log n) lookup by size
    large_blocks: BTreeMap<Size, Vec<FreeBlock>>,
}

impl SegregatedFreeList {
    pub fn new() -> Self {
        Self {
            small_blocks: vec![Vec::new(); 12], // 12 power-of-2 buckets
            medium_blocks: vec![Vec::new(); 15], // 15 4KB-increment buckets
            large_blocks: BTreeMap::new(),
        }
    }

    fn small_bucket_index(size: Size) -> Option<usize> {
        if size > SMALL_BLOCK_MAX {
            return None;
        }
        // Power of 2 bucketing: 64, 128, 256, 512, 1024, 2048, 4096
        // log2(size/64) = bucket index
        let bucket = if size <= 64 {
            0
        } else {
            // Round up to next power of 2, then get bucket
            let next_pow2 = size.next_power_of_two();
            (next_pow2.trailing_zeros() - 6) as usize // -6 because 2^6 = 64 is bucket 0
        };
        Some(bucket.min(11))
    }

    fn medium_bucket_index(size: Size) -> Option<usize> {
        if size <= SMALL_BLOCK_MAX || size > MEDIUM_BLOCK_MAX {
            return None;
        }
        // 4KB increment bucketing: 8KB, 12KB, 16KB, ..., 64KB
        // (size / 4KB) - 2 = bucket index (subtract 2 because 4KB is small, 8KB is first medium)
        let bucket = (size / (4 * 1024)).saturating_sub(2);
        Some(bucket.min(14))
    }

    pub fn insert(&mut self, block: FreeBlock) {
        if let Some(idx) = Self::small_bucket_index(block.size) {
            self.small_blocks[idx].push(block);
        } else if let Some(idx) = Self::medium_bucket_index(block.size) {
            self.medium_blocks[idx].push(block);
        } else {
            // Large block: use BTreeMap for O(log n) lookup
            self.large_blocks.entry(block.size).or_default().push(block);
        }
    }

    pub fn find_best_fit(&mut self, size: Size) -> Option<FreeBlock> {
        // Try small buckets first (O(1))
        if size <= SMALL_BLOCK_MAX {
            if let Some(start_bucket) = Self::small_bucket_index(size) {
                // Search current bucket and larger buckets
                for bucket_idx in start_bucket..self.small_blocks.len() {
                    if let Some(block) = self.small_blocks[bucket_idx].pop() {
                        return Some(block);
                    }
                }
            }
            // Fallthrough to medium buckets if no small block found
        }

        // Try medium buckets (O(1))
        if size <= MEDIUM_BLOCK_MAX {
            let start_bucket = Self::medium_bucket_index(size.max(SMALL_BLOCK_MAX + 1))
                .unwrap_or(0);
            for bucket_idx in start_bucket..self.medium_blocks.len() {
                if let Some(block) = self.medium_blocks[bucket_idx].pop() {
                    return Some(block);
                }
            }
            // Fallthrough to large blocks if no medium block found
        }

        // Try large blocks (O(log n) via BTreeMap)
        // Get all size buckets >= requested size
        let large_sizes: Vec<Size> = self
            .large_blocks
            .range(size..)
            .map(|(s, _)| *s)
            .collect();

        for block_size in large_sizes {
            if let Some(blocks) = self.large_blocks.get_mut(&block_size) {
                if let Some(block) = blocks.pop() {
                    // Clean up empty entries
                    if blocks.is_empty() {
                        self.large_blocks.remove(&block_size);
                    }
                    return Some(block);
                }
            }
        }

        None
    }

    pub fn len(&self) -> usize {
        let small_count: usize = self.small_blocks.iter().map(|v| v.len()).sum();
        let medium_count: usize = self.medium_blocks.iter().map(|v| v.len()).sum();
        let large_count: usize = self.large_blocks.values().map(|v| v.len()).sum();
        small_count + medium_count + large_count
    }

    pub fn get_all_sorted(&mut self) -> Vec<FreeBlock> {
        let mut all_blocks = Vec::new();

        // Extract all blocks
        for bucket in &mut self.small_blocks {
            all_blocks.append(bucket);
        }
        for bucket in &mut self.medium_blocks {
            all_blocks.append(bucket);
        }
        for blocks in self.large_blocks.values_mut() {
            all_blocks.append(blocks);
        }
        self.large_blocks.clear();

        // Sort by address
        all_blocks.sort_by_key(|b| b.address);
        all_blocks
    }

    pub fn reinsert_all(&mut self, blocks: Vec<FreeBlock>) {
        for block in blocks {
            self.insert(block);
        }
    }
}
