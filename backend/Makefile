.PHONY: build test run proto clean deps fmt lint lint-fix check coverage help

# Default target
.DEFAULT_GOAL := help

# Build the server
build: ## Build the server binary
	@echo "Building server..."
	@mkdir -p bin
	go build -o bin/server ./cmd/server
	@echo "Build complete: bin/server"

# Run tests
test: ## Run all tests
	@echo "Running tests..."
	go test -v -race ./...

# Run tests with coverage
coverage: ## Run tests with coverage report
	@echo "Running tests with coverage..."
	@mkdir -p coverage
	go test -v -race -coverprofile=coverage.txt -covermode=atomic ./...
	go tool cover -html=coverage.txt -o coverage.html
	@echo "Coverage report generated: coverage.html"

# Run the server
run: ## Run the server locally
	go run ./cmd/server -port 8000 -kernel localhost:50051 -ai localhost:50052

# Generate protobuf code
proto: ## Generate protobuf code from .proto files
	@echo "Generating protobuf code..."
	protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		proto/ai.proto
	cd ../proto && protoc --go_out=../backend/proto/kernel --go_opt=paths=source_relative \
		--go-grpc_out=../backend/proto/kernel --go-grpc_opt=paths=source_relative \
		kernel.proto
	@echo "Protobuf generation complete"

# Clean build artifacts
clean: ## Remove build artifacts and caches
	@echo "Cleaning..."
	rm -rf bin/
	rm -rf coverage/
	rm -f coverage.txt coverage.html
	go clean -cache -testcache
	@echo "Clean complete"

# Install dependencies
deps: ## Download and tidy dependencies
	@echo "Installing dependencies..."
	go mod download
	go mod tidy
	@echo "Dependencies installed"

# Format code
fmt: ## Format Go code
	@echo "Formatting code..."
	gofmt -s -w .
	goimports -w .
	@echo "Format complete"

# Lint code
lint: ## Run linters
	@echo "Running linters..."
	golangci-lint run ./...

# Lint and fix auto-fixable issues
lint-fix: ## Run linters and auto-fix issues
	@echo "Running linters with auto-fix..."
	golangci-lint run --fix ./...

# Check everything (fmt, lint, test)
check: fmt lint test ## Run all checks (format, lint, test)
	@echo "All checks passed!"

# Install development tools
tools: ## Install development tools
	@echo "Installing development tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install golang.org/x/tools/cmd/goimports@latest
	@echo "Tools installed"

# Help target
help: ## Show this help message
	@echo "AI-OS Backend - Available targets:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""
