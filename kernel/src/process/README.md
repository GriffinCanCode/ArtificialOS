# Process Module Organization

## Structure

```
process/
├── core/                    # Core types and traits
│   ├── mod.rs
│   ├── types.rs            # ProcessInfo, ProcessState, ExecutionConfig, errors
│   └── traits.rs           # ProcessLifecycle, ProcessExecutor, ProcessScheduler
│
├── execution/               # Process execution & OS interaction
│   ├── mod.rs
│   ├── executor.rs         # OS-level process spawning
│   ├── validation.rs       # Security validation (private)
│   └── preemption.rs       # OS-level preemption (SIGSTOP/SIGCONT)
│
├── lifecycle/               # Process lifecycle management
│   ├── mod.rs
│   ├── lifecycle.rs        # LifecycleRegistry for initialization hooks
│   ├── cleanup.rs          # OS process and scheduler cleanup (private)
│   └── budget.rs           # Resource budgeting and tracking
│
├── management/              # Process manager implementations
│   ├── mod.rs
│   ├── manager.rs          # Main ProcessManager implementation
│   ├── manager_builder.rs  # Builder pattern for ProcessManager
│   ├── manager_scheduler.rs # Scheduler integration methods
│   └── priority.rs         # Priority management utilities (private)
│
├── scheduler/               # CPU scheduling
│   ├── mod.rs
│   ├── atomic_stats.rs     # Lock-free statistics
│   ├── entry.rs            # Scheduling entries
│   ├── operations.rs       # Add, remove, schedule operations
│   ├── policy.rs           # Policy management
│   ├── stats.rs            # Statistics reporting
│   └── task.rs             # Autonomous scheduler task (renamed from scheduler_task.rs)
│
├── resources/               # Resource cleanup system
│   ├── mod.rs
│   ├── fds.rs              # File descriptor cleanup
│   ├── ipc.rs              # IPC cleanup
│   ├── mappings.rs         # Memory mapping cleanup
│   ├── memory.rs           # Memory cleanup
│   ├── rings.rs            # Zero-copy and io_uring cleanup
│   ├── signals.rs          # Signal cleanup
│   ├── sockets.rs          # Socket cleanup
│   ├── tasks.rs            # Async task cleanup
│   └── README.md           # Resource cleanup documentation
│
└── mod.rs                   # Root module with re-exports
```
