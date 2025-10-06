# Makefile for AI-OS Kernel (Rust)
.PHONY: help build build-release clean test check fmt fmt-check lint clippy run run-release doc install-tools audit security bench coverage watch

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

## help: Display this help message
help:
	@echo "$(BLUE)AgentOS Kernel - Available targets:$(NC)"
	@echo ""
	@sed -n 's/^##//p' $(MAKEFILE_LIST) | column -t -s ':' | sed -e 's/^/ /'
	@echo ""

## build: Build the project in debug mode
build:
	@echo "$(BLUE)Building kernel (debug)...$(NC)"
	cargo build

## build-release: Build the project in release mode with optimizations
build-release:
	@echo "$(BLUE)Building kernel (release)...$(NC)"
	cargo build --release

## clean: Remove build artifacts
clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	cargo clean

## test: Run all tests
test:
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test --all-features

## test-verbose: Run tests with verbose output
test-verbose:
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cargo test --all-features -- --nocapture --test-threads=1

## test-unit: Run only unit tests
test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	cargo test --test memory_test --test process_test --test syscall_test --test ipc_test --all-features

## test-integration: Run only integration tests
test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	cargo test --test integration_tests --all-features

## test-memory: Run memory tests
test-memory:
	@echo "$(BLUE)Running memory tests...$(NC)"
	cargo test --test memory_test --all-features -- --nocapture

## test-process: Run process tests
test-process:
	@echo "$(BLUE)Running process tests...$(NC)"
	cargo test --test process_test --all-features -- --nocapture

## test-syscall: Run syscall tests
test-syscall:
	@echo "$(BLUE)Running syscall tests...$(NC)"
	cargo test --test syscall_test --all-features -- --nocapture

## test-ipc: Run IPC tests
test-ipc:
	@echo "$(BLUE)Running IPC tests...$(NC)"
	cargo test --test ipc_test --all-features -- --nocapture

## test-quick: Run tests with minimal output
test-quick:
	@echo "$(BLUE)Running quick tests...$(NC)"
	cargo test --all-features --quiet

## check: Quick compile check without producing binaries
check:
	@echo "$(BLUE)Checking code...$(NC)"
	cargo check --all-targets --all-features

## fmt: Format code with rustfmt
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt --all

## fmt-check: Check if code is formatted correctly
fmt-check:
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cargo fmt --all -- --check

## lint: Run clippy with warnings as errors
lint:
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy --all-targets --all-features -- -D warnings

## clippy: Run clippy with pedantic warnings
clippy:
	@echo "$(BLUE)Running clippy (pedantic)...$(NC)"
	cargo clippy --all-targets --all-features -- \
		-W clippy::pedantic \
		-W clippy::nursery \
		-W clippy::cargo \
		-A clippy::module_name_repetitions \
		-A clippy::missing_errors_doc

## run: Run the kernel in debug mode
run:
	@echo "$(BLUE)Running kernel (debug)...$(NC)"
	RUST_LOG=debug cargo run

## run-release: Run the kernel in release mode
run-release:
	@echo "$(BLUE)Running kernel (release)...$(NC)"
	RUST_LOG=info cargo run --release

## doc: Generate and open documentation
doc:
	@echo "$(BLUE)Generating documentation...$(NC)"
	cargo doc --no-deps --open

## doc-all: Generate documentation including dependencies
doc-all:
	@echo "$(BLUE)Generating documentation (with deps)...$(NC)"
	cargo doc --open

## install-tools: Install development tools (rustfmt, clippy, cargo-audit, etc.)
install-tools:
	@echo "$(BLUE)Installing development tools...$(NC)"
	rustup component add rustfmt clippy rust-src rust-analyzer
	cargo install cargo-audit cargo-outdated cargo-edit cargo-watch cargo-expand
	@echo "$(GREEN)Development tools installed successfully!$(NC)"

## audit: Check for security vulnerabilities
audit:
	@echo "$(BLUE)Checking for security vulnerabilities...$(NC)"
	cargo audit

## security: Run comprehensive security checks
security: audit
	@echo "$(BLUE)Running security analysis...$(NC)"
	@command -v cargo-deny >/dev/null 2>&1 || { echo "$(YELLOW)cargo-deny not installed. Run: cargo install cargo-deny$(NC)"; exit 0; }
	cargo deny check

## bench: Run benchmarks
bench:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench

## coverage: Generate code coverage report (requires cargo-tarpaulin)
coverage:
	@echo "$(BLUE)Generating code coverage...$(NC)"
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { echo "$(YELLOW)cargo-tarpaulin not installed. Run: cargo install cargo-tarpaulin$(NC)"; exit 1; }
	cargo tarpaulin --out Html --out Lcov --output-dir coverage

## watch: Watch for changes and rebuild
watch:
	@echo "$(BLUE)Watching for changes...$(NC)"
	cargo watch -x check -x test -x run

## outdated: Check for outdated dependencies
outdated:
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	cargo outdated

## update: Update dependencies
update:
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cargo update

## verify: Run all verification checks (format, lint, test)
verify: fmt-check clippy test
	@echo "$(GREEN)All verification checks passed!$(NC)"

## pre-commit: Run pre-commit checks
pre-commit: fmt lint test
	@echo "$(GREEN)Pre-commit checks passed!$(NC)"

## ci: Run CI pipeline (check, test, lint, audit)
ci: check test lint audit
	@echo "$(GREEN)CI pipeline passed!$(NC)"

## release: Build release binary and strip symbols
release: clean build-release
	@echo "$(GREEN)Release build complete!$(NC)"
	@ls -lh target/release/kernel

## proto: Generate protobuf files
proto:
	@echo "$(BLUE)Generating protobuf files...$(NC)"
	cargo build
	@echo "$(GREEN)Protobuf files generated!$(NC)"

## deps: Show dependency tree
deps:
	@echo "$(BLUE)Dependency tree:$(NC)"
	cargo tree

## bloat: Analyze binary size
bloat:
	@echo "$(BLUE)Analyzing binary size...$(NC)"
	@command -v cargo-bloat >/dev/null 2>&1 || { echo "$(YELLOW)cargo-bloat not installed. Run: cargo install cargo-bloat$(NC)"; exit 1; }
	cargo bloat --release

## expand: Expand macros for debugging
expand:
	@echo "$(BLUE)Expanding macros...$(NC)"
	cargo expand

## fix: Automatically fix lint warnings
fix:
	@echo "$(BLUE)Fixing lint warnings...$(NC)"
	cargo fix --allow-dirty --allow-staged
	cargo clippy --fix --allow-dirty --allow-staged

## all: Build everything
all: clean verify build-release doc
	@echo "$(GREEN)Build complete!$(NC)"

