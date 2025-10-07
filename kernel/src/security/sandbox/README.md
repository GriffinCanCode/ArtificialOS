# Sandbox System

## Overview
Sandbox system addressing critical architectural concerns with a clean, modular design.

## Architectural Improvements

### 1. **Granular Capability System** ✅
**Before:** Blanket permissions (e.g., `ReadFile` for ALL files)
**After:** Path-specific permissions with smart matching

```rust
// Old: All-or-nothing
Capability::ReadFile

// New: Granular control
Capability::ReadFile(Some("/tmp".into()))  // Only /tmp
Capability::ReadFile(None)                  // All files (wildcard)
```

**Implementation:** `capability.rs`
- `grants()` method checks if one capability encompasses another
- Smart path prefix matching for hierarchical control
- Example: `ReadFile(Some("/tmp"))` grants access to `/tmp/test.txt`

### 2. **TOCTOU-Safe Path Handling** ✅
**Problem:** Path could change between check and use (Time-of-Check-to-Time-of-Use)

**Solution:** `path.rs` - PathHandle with early canonicalization
```rust
// Canonicalize once, use everywhere
let handle = PathHandle::try_new(path)?;
// handle.as_path() always returns the same canonical path
```

**Benefits:**
- Eliminates race conditions
- Consistent path representation
- Handles non-existent paths gracefully (parent canonicalization)

### 3. **Fine-Grained Network Control** ✅
**Before:** Boolean `allow_network: bool` (all-or-nothing)
**After:** Rich network rule system

```rust
pub enum NetworkRule {
    AllowAll,                                    // Unrestricted
    AllowHost { host: String, port: Option<u16> },  // Specific host:port
    AllowCIDR(String),                           // IP range (e.g., "192.168.0.0/24")
    BlockHost { host: String, port: Option<u16> },  // Explicit deny
}
```

**Implementation:** `network.rs`
- Wildcard domain matching (`*.example.com`)
- CIDR block matching for IPv4/IPv6
- Port-specific restrictions
- Priority-based rule evaluation (blocks take precedence)

## Module Structure

```
sandbox/
├── mod.rs           - Module exports
├── manager.rs       - SandboxManager (orchestration)
├── config.rs        - SandboxConfig methods
├── capability.rs    - Granular capability checking
├── network.rs       - Network access control
└── path.rs          - TOCTOU-safe path handling
```

**Design Philosophy:** Smart implementations, not verbose ones
- Each file has a single, focused responsibility
- Clean separation of concerns
- Zero duplication between modules

## Usage Examples

### Granular File Access
```rust
let mut config = SandboxConfig::minimal(pid);

// Grant read access only to /tmp
config.grant_capability(Capability::ReadFile(Some("/tmp".into())));

// Check specific file access
let can_read = manager.check_file_operation(
    pid,
    FileOperation::Read,
    Path::new("/tmp/test.txt")
); // true

let can_read = manager.check_file_operation(
    pid,
    FileOperation::Read,
    Path::new("/etc/passwd")
); // false
```

### Network Access Control
```rust
let mut config = SandboxConfig::minimal(pid);

// Allow only specific API endpoint
config.network_rules.push(NetworkRule::AllowHost {
    host: "api.example.com".to_string(),
    port: Some(443),
});

// Allow entire internal network
config.network_rules.push(NetworkRule::AllowCIDR("10.0.0.0/8".to_string()));

// Check access
manager.check_network_access(pid, "api.example.com", Some(443)); // true
manager.check_network_access(pid, "evil.com", Some(80));         // false
```

### TOCTOU-Safe Operations
```rust
// Canonicalize path once
let path = PathHandle::try_new(user_input)?;

// Check access with canonical path
if !manager.check_path_access(pid, path.as_path()) {
    return Err("Access denied");
}

// Use same canonical path for actual operation
let file = File::open(path.as_path())?;
```

## Testing

All modules include comprehensive unit tests:
- `capability.rs`: Granular permission matching
- `network.rs`: Rule evaluation, wildcard/CIDR matching  
- `path.rs`: Canonicalization, TOCTOU safety

Run tests: `cargo test --lib security::sandbox`

## Migration Notes

**Breaking Changes:**
- `Capability` enum variants now carry data
- `SandboxConfig.allow_network` → `SandboxConfig.network_rules`

**Update Pattern:**
```rust
// Old
Capability::ReadFile
config.allow_network = true

// New  
Capability::ReadFile(None)  // or Some(path) for granular
config.network_rules.push(NetworkRule::AllowAll)
```

All syscall handlers and API endpoints have been updated to use the new format.
