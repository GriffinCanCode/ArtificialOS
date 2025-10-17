# AgentOS Documentation

Comprehensive documentation for the AgentOS project, organized by category for easy navigation.

## 📁 Directory Structure

### `/architecture`
System design, core architecture, and structural documentation.

- **`system-architecture.md`** - Core system architecture and data flow
- **`blueprint-dsl.md`** - Blueprint DSL specification for app definitions
- **`desktop-system.md`** - Desktop UI system and window management
- **`filesystem-structure.md`** - Virtual filesystem structure and paths

### `/performance`
Performance optimization strategies and implementation details.

- **`advanced-optimizations.md`** - Expert-level performance optimizations
- **`bincode-optimization.md`** - Binary serialization for high-performance IPC
- **`async-traits-migration.md`** - Async traits and dual-mode execution

### `/patterns`
Design patterns, coding standards, and best practices.

- **`code-standards.md`** - Code standards and quality guidelines
- **`graceful-with-fallback.md`** - Async shutdown pattern for background tasks
- **`sharded-slot.md`** - Lock-free synchronization pattern
- **`shard-manager.md`** - Sharding strategy for concurrent data structures

### `/features`
Specific feature implementations and technical details.

- **`iouring-syscalls.md`** - io_uring integration for async I/O
- **`mmap-support.md`** - Memory-mapped file support
- **`sync-primitives.md`** - Synchronization primitives (futex, semaphore, etc.)
- **`preemptive-scheduling.md`** - Preemptive scheduling implementation
- **`observability-validation.md`** - Observability system validation
- **`selection-shortcuts.md`** - Desktop selection and keyboard shortcuts

### `/ui-frontend`
User interface and frontend-specific documentation.

- **`cva-setup.md`** - Class Variance Authority setup for component variants
- **`launchpad.md`** - App launcher UI implementation

### `/apps`
Application development, ecosystem, and native apps.

- **`native-apps-dev-guide.md`** - Guide for developing native apps
- **`native-apps-plan.md`** - Native apps roadmap and architecture
- **`prebuilt-apps.md`** - Prebuilt system applications

### `/planning`
Roadmaps, future ideas, and implementation planning.

- **`future-ideas.md`** - Innovation roadmap and future enhancements
- **`implementation-plan.md`** - Original implementation plan
- **`implementation-guide.md`** - Step-by-step implementation guide

## 🔍 Quick Navigation

### Getting Started
1. Start with [`architecture/system-architecture.md`](architecture/system-architecture.md)
2. Review [`patterns/code-standards.md`](patterns/code-standards.md)
3. Explore specific features in [`/features`](features/)

### For Contributors
- Read [`patterns/code-standards.md`](patterns/code-standards.md) for coding guidelines
- Check [`planning/implementation-guide.md`](planning/implementation-guide.md) for implementation details
- Review relevant feature docs in [`/features`](features/)

### For App Developers
- Start with [`apps/native-apps-dev-guide.md`](apps/native-apps-dev-guide.md)
- Read [`architecture/blueprint-dsl.md`](architecture/blueprint-dsl.md)
- Check [`ui-frontend/cva-setup.md`](ui-frontend/cva-setup.md) for UI components

### For Performance Tuning
- Review [`performance/advanced-optimizations.md`](performance/advanced-optimizations.md)
- Check [`patterns/sharded-slot.md`](patterns/sharded-slot.md) for lock-free patterns
- See [`performance/bincode-optimization.md`](performance/bincode-optimization.md) for serialization

## 📚 Documentation Standards

### Naming Conventions
- **Folders**: lowercase with hyphens (e.g., `ui-frontend`, `performance`)
- **Files**: lowercase with hyphens (e.g., `system-architecture.md`, `code-standards.md`)

### File Structure
Each documentation file should include:
- Overview section at the top
- Clear headings and subheadings
- Code examples where applicable
- Cross-references to related documents

### Cross-References
When referencing other docs, use relative paths:
```markdown
See [System Architecture](architecture/system-architecture.md) for details.
```

## 🔄 Migration Notes

**Previous Structure** → **New Structure**

- `ARCHITECTURE.md` → `architecture/system-architecture.md`
- `CODE_STANDARDS_2025.md` → `patterns/code-standards.md`
- `NATIVE_APPS_DEV_GUIDE.md` → `apps/native-apps-dev-guide.md`
- `FUTURE_IDEAS.md` → `planning/future-ideas.md`
- *(All files reorganized with consistent naming)*

## 🤝 Contributing to Docs

When adding new documentation:
1. Choose the appropriate category folder
2. Use lowercase-with-hyphens naming
3. Add entry to this README
4. Include cross-references to related docs

## 📖 External Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TypeScript Best Practices](https://www.typescriptlang.org/docs/)
- [React 18 Documentation](https://react.dev/)

