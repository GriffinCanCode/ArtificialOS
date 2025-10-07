#!/bin/bash
# Script to update all DashMap files to use ahash

files=(
    "src/vfs/memory.rs"
    "src/vfs/mount.rs"
    "src/ipc/mmap.rs"
    "src/ipc/pipe/manager.rs"
    "src/ipc/queue/manager.rs"
    "src/ipc/shm/manager.rs"
    "src/ipc/core/manager.rs"
    "src/syscalls/fd.rs"
    "src/process/executor.rs"
    "src/signals/manager.rs"
    "src/signals/callbacks.rs"
    "src/security/namespace/simulation.rs"
    "src/security/namespace/macos.rs"
    "src/security/namespace/linux.rs"
    "src/security/ebpf/simulation.rs"
)

for file in "${files[@]}"; do
    echo "Processing $file..."

    # Add ahash import after dashmap import
    if ! grep -q "use ahash::RandomState;" "$file"; then
        sed -i '' '/use dashmap::DashMap;/a\
use ahash::RandomState;
' "$file"
    fi

    # Replace DashMap::new() with DashMap::with_hasher(RandomState::new())
    sed -i '' 's/DashMap::new()/DashMap::with_hasher(RandomState::new())/g' "$file"

    # Replace DashMap::with_capacity(N) with DashMap::with_capacity_and_hasher(N, RandomState::new())
    sed -i '' 's/DashMap::with_capacity(\([^)]*\))/DashMap::with_capacity_and_hasher(\1, RandomState::new())/g' "$file"

    # Replace DashMap::with_shard_amount(N) with DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), N)
    sed -i '' 's/DashMap::with_shard_amount(\([^)]*\))/DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), \1)/g' "$file"
done

echo "Done!"
