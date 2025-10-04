# AI-OS Kernel

A lightweight microkernel written in Rust for the AI-Powered Operating System.

## Features

- **Process Management**: Lightweight process isolation and management
- **Memory Management**: Safe memory allocation and sandboxing
- **gRPC Communication**: High-performance communication with backend services
- **IPC**: Inter-process communication for AI service integration

## Development

### Prerequisites

- Rust 1.70+ (stable)
- Cargo
- Protocol Buffers compiler

### Building

```bash
# Build in debug mode
make build

# Build in release mode
make build-release

# Run the kernel
make run
```

### Testing

```bash
# Run tests
make test

# Run tests with verbose output
make test-verbose
```

### Code Quality

```bash
# Format code
make fmt

# Check formatting
make fmt-check

# Run linter (clippy)
make lint

# Run all checks
make verify
```

### Development Tools

Install recommended development tools:

```bash
make install-tools
```

This installs:
- `rustfmt` - Code formatter
- `clippy` - Linter
- `rust-analyzer` - LSP server
- `cargo-audit` - Security vulnerability scanner
- `cargo-outdated` - Dependency checker
- `cargo-watch` - File watcher for auto-rebuild

### Security

Check for security vulnerabilities:

```bash
make audit
```

### Documentation

Generate and view documentation:

```bash
make doc
```

## Architecture

The kernel is built with the following components:

- **Process Management** (`process.rs`): Process lifecycle and scheduling
- **Memory Management** (`memory.rs`): Memory allocation and protection
- **Sandbox** (`sandbox.rs`): Process isolation and security
- **IPC** (`ipc.rs`): Inter-process communication
- **gRPC Server** (`grpc_server.rs`): External service communication
- **Syscall Handler** (`syscall.rs`): System call interface

## Configuration

### Code Formatting

Code formatting is configured in `rustfmt.toml`. The project follows:
- 100 character line width
- 4-space indentation
- Standard Rust formatting conventions

### Linting

Clippy configuration is in `clippy.toml`. The project enables:
- Pedantic warnings
- Nursery warnings
- Cargo warnings

### Editor Configuration

EditorConfig settings are in `.editorconfig` for consistent coding styles across editors.

## CI/CD

Run the full CI pipeline:

```bash
make ci
```

This runs:
1. Code check
2. Tests
3. Linting
4. Security audit

## Troubleshooting

### Build Failures

If you encounter build failures, try:

```bash
make clean
make build
```

### Dependency Issues

Update dependencies:

```bash
make update
```

Check for outdated dependencies:

```bash
make outdated
```

## Contributing

1. Format your code: `make fmt`
2. Run linter: `make lint`
3. Run tests: `make test`
4. Run full verification: `make verify`

## License

See the root LICENSE file for details.

