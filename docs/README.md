# AgentOS Documentation

Comprehensive documentation for the AgentOS project, organized by category for easy navigation.

## Directory Structure

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

### `/monitoring`
Observability, monitoring, and system tracing documentation.

- **`journey-tracking.md`** - Request journey tracking and observability

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

## Quick Navigation

### Getting Started
1. Read [`DOCUMENTATION_STANDARDS.md`](./DOCUMENTATION_STANDARDS.md)
2. Start with [`architecture/system-architecture.md`](architecture/system-architecture.md)
3. Review [`patterns/code-standards.md`](patterns/code-standards.md)
4. Explore specific features in [`/features`](features/)

### For Contributors
- Read [`DOCUMENTATION_STANDARDS.md`](./DOCUMENTATION_STANDARDS.md) before writing docs
- Read [`patterns/code-standards.md`](patterns/code-standards.md) for coding guidelines
- Check [`planning/implementation-guide.md`](planning/implementation-guide.md) for implementation details
- Review relevant feature docs in [`/features`](features/)

### For Documentation Maintainers
- Refer to [`DOCUMENTATION_STANDARDS.md`](./DOCUMENTATION_STANDARDS.md) for writing guidelines
- Use the checklist in the standards document before considering docs complete
- Keep all examples tested and current with the codebase
- Update documentation when code changes

### For App Developers
- Start with [`apps/native-apps-dev-guide.md`](apps/native-apps-dev-guide.md)
- Read [`architecture/blueprint-dsl.md`](architecture/blueprint-dsl.md)
- Check [`ui-frontend/cva-setup.md`](ui-frontend/cva-setup.md) for UI components

### For Performance Tuning
- Review [`performance/advanced-optimizations.md`](performance/advanced-optimizations.md)
- Check [`patterns/sharded-slot.md`](patterns/sharded-slot.md) for lock-free patterns
- See [`performance/bincode-optimization.md`](performance/bincode-optimization.md) for serialization

## Contributing to Documentation

When adding or updating documentation:

1. **Read the standards first** - Review [`DOCUMENTATION_STANDARDS.md`](./DOCUMENTATION_STANDARDS.md)
2. **Choose appropriate category** - Place docs in the relevant folder
3. **Use consistent naming** - Use lowercase-with-hyphens (e.g., `feature-name.md`)
4. **Verify accuracy** - All code examples must work, all paths must be correct
5. **Remove marketing language** - Use technical, developer voice
6. **Add to this README** - Include entry in the appropriate section
7. **Cross-reference** - Link to related documents
8. **Use the checklist** - Run through the documentation checklist before committing

## External Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TypeScript Best Practices](https://www.typescriptlang.org/docs/)
- [React 18 Documentation](https://react.dev/)

