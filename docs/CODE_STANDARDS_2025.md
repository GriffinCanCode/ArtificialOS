# Code Standards for 2025

## Overview

This document outlines the modernized code standards for the OS kernel project, updated to meet 2025 best practices.

## Standards

### Clippy Configuration

Updated `kernel/clippy.toml` with the following modern standards:

```toml
# Cognitive complexity threshold (max 15-20)
cognitive-complexity-threshold = 15

# Maximum number of lines for a function (max 50 lines)
too-many-lines-threshold = 50
```

### File Size Limits

- **Maximum file length**: 500 lines
- **Function length**: 50 lines max
- **Cognitive complexity**: 15-20 max

## Refactoring Summary

The following files have been successfully refactored to meet the new standards:

### ✅ Completed Refactorings

1. **ipc/queue/manager.rs** (637 → 198 lines)
   - Extracted `operations.rs` - Send/receive message operations
   - Extracted `subscription.rs` - PubSub subscribe/unsubscribe logic
   - Extracted `lifecycle.rs` - Create/close/destroy operations

2. **signals/manager.rs** (528 → 465 lines)
   - Extracted `internal_types.rs` - PrioritySignal and ProcessSignals types

3. **process/executor.rs** (578 → 430 lines)
   - Extracted `validation.rs` - Command and argument validation

4. **process/manager.rs** (578 → 525 lines)
   - Extracted `priority.rs` - Priority management functions
   - Extracted `cleanup.rs` - Process cleanup logic

### 📋 Remaining Files to Refactor

Files still exceeding 500 lines (in order of priority):

1. **api/grpc_server.rs** (1169 lines) - CRITICAL
2. **syscalls/types.rs** (1008 lines) - CRITICAL
3. **vfs/memory.rs** (968 lines) - HIGH
4. **memory/manager.rs** (901 lines) - HIGH
5. **process/scheduler.rs** (834 lines) - MEDIUM
6. **vfs/types.rs** (816 lines) - MEDIUM
7. **security/namespace/bridge.rs** (643 lines) - MEDIUM
8. **syscalls/ipc.rs** (626 lines) - MEDIUM

## Refactoring Patterns

### Module Extraction

When a file exceeds 500 lines, extract related functionality into separate modules:

**Example Structure:**
```
original_file.rs (< 500 lines)
├── operations.rs (helper operations)
├── lifecycle.rs (creation/cleanup)
├── validation.rs (validation logic)
└── types.rs (internal types)
```

### Benefits

1. **Maintainability**: Smaller files are easier to understand and modify
2. **Testability**: Isolated modules are easier to test
3. **Cognitive Load**: Reduced complexity per file
4. **Code Review**: Smaller changesets are easier to review

## Implementation Guidelines

### For New Code

- Keep functions under 50 lines
- Keep files under 500 lines
- Maintain cognitive complexity below 15
- Extract helper functions proactively

### For Existing Code

- Refactor files exceeding 500 lines
- Split large functions (>50 lines)
- Extract complex logic (cognitive complexity >15)
- Group related functionality into modules

## Tools

- **clippy**: Enforces complexity and line limits
- **cargo check**: Validates refactored code
- **cargo test**: Ensures refactorings don't break functionality

## Next Steps

1. Continue refactoring remaining files over 500 lines
2. Run full test suite after each refactoring
3. Update documentation as modules are extracted
4. Monitor clippy warnings during development

## References

- Clippy Configuration: `kernel/clippy.toml`
- Refactoring Examples: See completed modules above
- Modern Rust Best Practices: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
