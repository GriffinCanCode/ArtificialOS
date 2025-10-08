/*!
 * Intelligent Shard Configuration Manager
 *
 * CPU-topology-aware shard count calculation for concurrent data structures.
 * Instead of hardcoded values, this dynamically computes optimal shard counts
 * based on the host system's CPU topology, ensuring efficient scaling from
 * embedded devices (1-4 cores) to high-end servers (128+ cores).
 *
 * Design Rationale:
 * - Power-of-2 shards enable fast modulo via bitwise AND
 * - CPU-proportional scaling: more cores = more beneficial parallelism
 * - Contention multipliers based on empirical access patterns
 * - One-time computation: zero runtime overhead after initialization
 */

use std::sync::OnceLock;

/// Global singleton for hardware-aware shard configuration
static SHARD_MANAGER: OnceLock<ShardManager> = OnceLock::new();

/// Hardware-aware shard configuration calculator
#[derive(Debug, Clone)]
pub struct ShardManager {
    cpu_count: usize,
    cache_line_size: usize,
}

impl ShardManager {
    /// Get or initialize the global shard manager instance
    fn instance() -> &'static Self {
        SHARD_MANAGER.get_or_init(|| {
            let cpu_count = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or_else(|_| {
                    // Fallback: reasonable default for unknown systems
                    log::warn!("Failed to detect CPU count, defaulting to 8");
                    8
                });

            let manager = Self {
                cpu_count,
                cache_line_size: Self::detect_cache_line_size(),
            };

            log::info!(
                "ShardManager initialized: {} CPUs, {} byte cache lines",
                manager.cpu_count,
                manager.cache_line_size
            );

            manager
        })
    }

    /// Detect L1 cache line size (used for padding to avoid false sharing)
    fn detect_cache_line_size() -> usize {
        // Most modern architectures use 64-byte cache lines
        // (x86-64, ARM64, RISC-V). Could be refined with CPUID on x86.
        64
    }

    /// Calculate optimal shard count for a given workload profile
    pub fn shards(profile: WorkloadProfile) -> usize {
        let mgr = Self::instance();
        let base = mgr.cpu_count;

        let multiplier = match profile {
            // High contention: Fine-grained locking via 4x CPU shards
            // Used for: memory blocks, storage maps, process tables
            // Rationale: Heavy concurrent access benefits from maximum parallelism
            WorkloadProfile::HighContention => 4,

            // Medium contention: Balance memory overhead vs parallelism (2x CPUs)
            // Used for: child tracking, sandboxes, pipe management
            // Rationale: Moderate access patterns don't justify 4x memory overhead
            WorkloadProfile::MediumContention => 2,

            // Low contention: Minimal sharding (1x CPUs)
            // Used for: spawn counts, metrics, infrequent lookups
            // Rationale: Rare contention makes extra shards wasteful
            WorkloadProfile::LowContention => 1,
        };

        // Ensure power of 2 for efficient hash distribution (modulo via bitwise AND)
        let calculated = (base * multiplier).next_power_of_two();

        // Clamp to reasonable bounds
        // Min: 8 shards (avoid extreme degeneration on 1-2 core systems)
        // Max: 512 shards (diminishing returns, excessive memory overhead)
        calculated.clamp(8, 512)
    }

    /// Get the CPU count detected at initialization
    pub fn cpu_count() -> usize {
        Self::instance().cpu_count
    }

    /// Get the cache line size
    pub fn cache_line_size() -> usize {
        Self::instance().cache_line_size
    }
}

/// Workload characterization for shard count calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadProfile {
    /// Heavy concurrent access (blocks, memory_storage, process tables)
    /// Shard count: 4x CPU cores
    HighContention,

    /// Moderate concurrent access (child_counts, sandboxes, pipes)
    /// Shard count: 2x CPU cores
    MediumContention,

    /// Light concurrent access (spawn_counts, metrics)
    /// Shard count: 1x CPU cores
    LowContention,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_calculation() {
        // Verify power-of-2 property
        for profile in [
            WorkloadProfile::HighContention,
            WorkloadProfile::MediumContention,
            WorkloadProfile::LowContention,
        ] {
            let shards = ShardManager::shards(profile);
            assert!(shards.is_power_of_two(), "Shards must be power of 2");
            assert!(shards >= 8, "Minimum 8 shards");
            assert!(shards <= 512, "Maximum 512 shards");
        }
    }

    #[test]
    fn test_contention_ordering() {
        // Verify high > medium > low
        let high = ShardManager::shards(WorkloadProfile::HighContention);
        let medium = ShardManager::shards(WorkloadProfile::MediumContention);
        let low = ShardManager::shards(WorkloadProfile::LowContention);

        assert!(high >= medium, "High contention should have most shards");
        assert!(medium >= low, "Medium should have more than low");
    }

    #[test]
    fn test_singleton_consistency() {
        // Verify multiple calls return same values
        let cpu1 = ShardManager::cpu_count();
        let cpu2 = ShardManager::cpu_count();
        assert_eq!(cpu1, cpu2);

        let cache1 = ShardManager::cache_line_size();
        let cache2 = ShardManager::cache_line_size();
        assert_eq!(cache1, cache2);
    }
}

