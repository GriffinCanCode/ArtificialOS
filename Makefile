.PHONY: help all setup clean install build start stop logs proto test format

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

##@ General

help: ## Display this help message
	@echo "$(CYAN)╔═══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║           AI-Powered OS - Makefile Commands              ║$(NC)"
	@echo "$(CYAN)╚═══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf ""} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(CYAN)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
	@echo ""

all: proto build ## Build all components (kernel, backend, ai-service, ui)
	@echo "$(GREEN)✅ All components built successfully$(NC)"

##@ Setup & Installation

setup: install-kernel install-ai install-backend install-ui ## Install all dependencies
	@echo "$(GREEN)✅ All dependencies installed$(NC)"

install-kernel: ## Install Rust/Cargo dependencies for kernel
	@echo "$(YELLOW)📦 Installing Rust kernel dependencies...$(NC)"
	@cd kernel && cargo fetch

install-ai: ## Setup Python virtual environment and install AI service dependencies
	@echo "$(YELLOW)📦 Setting up Python AI service...$(NC)"
	@cd ai-service && \
		if [ ! -d "venv" ]; then python3 -m venv venv; fi && \
		. venv/bin/activate && \
		pip install --upgrade pip && \
		pip install -r requirements.txt
	@echo "$(GREEN)✅ AI service dependencies installed$(NC)"

install-backend: ## Install Go backend dependencies
	@echo "$(YELLOW)📦 Installing Go backend dependencies...$(NC)"
	@cd backend && go mod download && go mod tidy
	@echo "$(GREEN)✅ Backend dependencies installed$(NC)"

install-ui: ## Install Node.js UI dependencies
	@echo "$(YELLOW)📦 Installing UI dependencies...$(NC)"
	@cd ui && npm install
	@echo "$(GREEN)✅ UI dependencies installed$(NC)"

##@ Protocol Buffers

proto: ## Compile all protocol buffer definitions
	@echo "$(YELLOW)🔨 Compiling protocol buffers...$(NC)"
	@export PATH=$$PATH:$$(go env GOPATH)/bin && ./scripts/compile-protos.sh
	@echo "$(GREEN)✅ Protocol buffers compiled$(NC)"

proto-go: ## Compile protocol buffers for Go only
	@echo "$(YELLOW)🔨 Compiling protocol buffers for Go...$(NC)"
	@export PATH=$$PATH:$$(go env GOPATH)/bin && \
		cd proto && protoc --go_out=../backend/proto/kernel --go_opt=paths=source_relative \
		--go-grpc_out=../backend/proto/kernel --go-grpc_opt=paths=source_relative \
		kernel.proto
	@export PATH=$$PATH:$$(go env GOPATH)/bin && \
		cd backend/proto && protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		ai.proto
	@echo "$(GREEN)✅ Go protocol buffers compiled$(NC)"

proto-python: ## Compile protocol buffers for Python only
	@echo "$(YELLOW)🔨 Compiling protocol buffers for Python...$(NC)"
	@cd ai-service && . venv/bin/activate && cd proto && python3 -m grpc_tools.protoc -I. --python_out=../src --grpc_python_out=../src ai.proto
	@echo "$(GREEN)✅ Python protocol buffers compiled$(NC)"

##@ Build

build: build-kernel build-backend build-ui ## Build all components
	@echo "$(GREEN)✅ All builds complete$(NC)"

build-kernel: ## Build Rust kernel (release mode)
	@echo "$(YELLOW)🔨 Building Rust kernel...$(NC)"
	@cd kernel && cargo build --release
	@echo "$(GREEN)✅ Kernel built$(NC)"

build-kernel-dev: ## Build Rust kernel (debug mode)
	@echo "$(YELLOW)🔨 Building Rust kernel (debug)...$(NC)"
	@cd kernel && cargo build
	@echo "$(GREEN)✅ Kernel built (debug)$(NC)"

build-backend: ## Build Go backend server
	@echo "$(YELLOW)🔨 Building Go backend...$(NC)"
	@cd backend && go build -o bin/server ./cmd/server
	@echo "$(GREEN)✅ Backend built$(NC)"

build-ui: ## Build UI for production
	@echo "$(YELLOW)🔨 Building UI...$(NC)"
	@cd ui && npm run build
	@echo "$(GREEN)✅ UI built$(NC)"

##@ Run & Start

start: init-storage ## Start everything (backend + electron UI)
	@echo "$(RED)      Closing shit quickly... $(NC)"
	@./scripts/stop.sh
	@echo "$(YELLOW)🚀 Starting backend stack...$(NC)"
	@./scripts/start-backend.sh
	@echo "$(YELLOW)⏳ Waiting for backend to be ready...$(NC)"
	@sleep 3
	@echo "$(YELLOW)🚀 Starting Electron UI...$(NC)"
	@./scripts/start-ui.sh

start-backend: init-storage ## Start complete backend stack (kernel + ai-service + backend)
	@./scripts/start-backend.sh

init-storage: ## Initialize storage directories for persistence
	@./scripts/init-storage.sh

start-ui: ## Start UI development server
	@./scripts/start-ui.sh

start-electron: start-ui ## Start Electron app (alias for start-ui)

dev: init-storage ## Start everything (backend + ui in development mode)
	@echo "$(YELLOW)🚀 Starting full development environment...$(NC)"
	@echo "$(YELLOW)Starting backend in background...$(NC)"
	@./scripts/start-backend.sh &
	@sleep 3
	@echo "$(YELLOW)Starting UI...$(NC)"
	@./scripts/start-ui.sh

run-kernel: ## Run kernel directly (must be built first)
	@echo "$(YELLOW)🚀 Starting kernel...$(NC)"
	@cd kernel && ./target/release/kernel

run-backend: ## Run backend directly (must be built first)
	@echo "$(YELLOW)🚀 Starting backend...$(NC)"
	@cd backend && ./bin/server -port 8000 -kernel localhost:50051 -ai localhost:50052

run-ai: ## Run AI gRPC service directly
	@echo "$(YELLOW)🚀 Starting AI service...$(NC)"
	@cd ai-service && . venv/bin/activate && PYTHONPATH=src python3 -m grpc_server

##@ Stop & Clean

stop: ## Stop all running services
	@./scripts/stop.sh

clean: clean-kernel clean-backend clean-ui clean-logs ## Clean all build artifacts and logs
	@echo "$(GREEN)✅ All cleaned$(NC)"

clean-kernel: ## Clean Rust kernel build artifacts
	@echo "$(YELLOW)🧹 Cleaning kernel...$(NC)"
	@cd kernel && cargo clean
	@echo "$(GREEN)✅ Kernel cleaned$(NC)"

clean-backend: ## Clean Go backend build artifacts
	@echo "$(YELLOW)🧹 Cleaning backend...$(NC)"
	@cd backend && rm -rf bin/ && go clean
	@echo "$(GREEN)✅ Backend cleaned$(NC)"

clean-ui: ## Clean UI build artifacts
	@echo "$(YELLOW)🧹 Cleaning UI...$(NC)"
	@cd ui && rm -rf dist/ node_modules/.vite
	@echo "$(GREEN)✅ UI cleaned$(NC)"

clean-logs: ## Clean all log files
	@echo "$(YELLOW)🧹 Cleaning logs...$(NC)"
	@rm -f logs/*.log logs/*.pid
	@echo "$(GREEN)✅ Logs cleaned$(NC)"

deep-clean: clean ## Deep clean (includes node_modules and venv)
	@echo "$(RED)🧹 Deep cleaning (removing node_modules and venv)...$(NC)"
	@rm -rf ui/node_modules
	@rm -rf ai-service/venv
	@echo "$(GREEN)✅ Deep clean complete$(NC)"

##@ Testing

test: test-backend test-kernel ## Run all tests
	@echo "$(GREEN)✅ All tests passed$(NC)"

test-kernel: ## Run Rust kernel tests
	@echo "$(YELLOW)🧪 Running kernel tests...$(NC)"
	@cd kernel && cargo test
	@echo "$(GREEN)✅ Kernel tests passed$(NC)"

test-backend: ## Run Go backend tests
	@echo "$(YELLOW)🧪 Running backend tests...$(NC)"
	@cd backend && go test -v ./...
	@echo "$(GREEN)✅ Backend tests passed$(NC)"

test-ai: ## Run Python AI service tests
	@echo "$(YELLOW)🧪 Running AI service tests...$(NC)"
	@cd ai-service && . venv/bin/activate && pytest
	@echo "$(GREEN)✅ AI service tests passed$(NC)"

##@ Code Quality

format: format-kernel format-backend format-ai format-ui ## Format all code
	@echo "$(GREEN)✅ All code formatted$(NC)"

format-kernel: ## Format Rust kernel code
	@echo "$(YELLOW)✨ Formatting kernel...$(NC)"
	@cd kernel && cargo fmt
	@echo "$(GREEN)✅ Kernel formatted$(NC)"

format-backend: ## Format Go backend code
	@echo "$(YELLOW)✨ Formatting backend...$(NC)"
	@cd backend && go fmt ./...
	@echo "$(GREEN)✅ Backend formatted$(NC)"

format-ai: ## Format Python AI service code
	@echo "$(YELLOW)✨ Formatting AI service...$(NC)"
	@cd ai-service && . venv/bin/activate && black src/
	@echo "$(GREEN)✅ AI service formatted$(NC)"

format-ui: ## Format UI code
	@echo "$(YELLOW)✨ Formatting UI...$(NC)"
	@cd ui && npm run format
	@echo "$(GREEN)✅ UI formatted$(NC)"

lint-backend: ## Lint Go backend code
	@echo "$(YELLOW)🔍 Linting backend...$(NC)"
	@cd backend && golangci-lint run ./... || echo "$(YELLOW)⚠️  golangci-lint not installed or found issues$(NC)"

lint-ui: ## Lint UI code
	@echo "$(YELLOW)🔍 Linting UI...$(NC)"
	@cd ui && npm run lint

##@ Logs & Monitoring

logs: ## Tail all logs
	@echo "$(CYAN)📊 Tailing all logs (Ctrl+C to exit)...$(NC)"
	@tail -f logs/*.log 2>/dev/null || echo "$(YELLOW)No logs found. Start services first.$(NC)"

logs-kernel: ## Tail kernel logs
	@tail -f logs/kernel.log

logs-ai: ## Tail AI service logs
	@tail -f logs/ai-grpc.log

logs-backend: ## Tail backend logs
	@tail -f logs/backend.log

show-logs: ## Show recent logs from all services
	@echo "$(CYAN)╔══════════════ Kernel Logs ══════════════╗$(NC)"
	@tail -n 20 logs/kernel.log 2>/dev/null || echo "No kernel logs"
	@echo ""
	@echo "$(CYAN)╔══════════════ AI Service Logs ══════════════╗$(NC)"
	@tail -n 20 logs/ai-grpc.log 2>/dev/null || echo "No AI service logs"
	@echo ""
	@echo "$(CYAN)╔══════════════ Backend Logs ══════════════╗$(NC)"
	@tail -n 20 logs/backend.log 2>/dev/null || echo "No backend logs"

##@ Status & Info

status: ## Check status of all services
	@echo "$(CYAN)╔═══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║                    Service Status                        ║$(NC)"
	@echo "$(CYAN)╚═══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(YELLOW)Kernel (port 50051):$(NC)"
	@lsof -i :50051 > /dev/null 2>&1 && echo "  $(GREEN)✅ Running$(NC)" || echo "  $(RED)❌ Not running$(NC)"
	@echo ""
	@echo "$(YELLOW)AI Service (port 50052):$(NC)"
	@lsof -i :50052 > /dev/null 2>&1 && echo "  $(GREEN)✅ Running$(NC)" || echo "  $(RED)❌ Not running$(NC)"
	@echo ""
	@echo "$(YELLOW)Backend (port 8000):$(NC)"
	@lsof -i :8000 > /dev/null 2>&1 && echo "  $(GREEN)✅ Running$(NC)" || echo "  $(RED)❌ Not running$(NC)"
	@echo ""
	@echo "$(YELLOW)UI (port 5173):$(NC)"
	@lsof -i :5173 > /dev/null 2>&1 && echo "  $(GREEN)✅ Running$(NC)" || echo "  $(RED)❌ Not running$(NC)"
	@echo ""

ps: ## Show running processes
	@echo "$(CYAN)Running AI-OS processes:$(NC)"
	@ps aux | grep -E "(ai_os_kernel|grpc_server|backend/bin/server|vite)" | grep -v grep || echo "No processes running"

ports: ## Show which ports are in use
	@echo "$(CYAN)Port usage:$(NC)"
	@echo "  Port 50051 (Kernel):  " && (lsof -i :50051 > /dev/null 2>&1 && echo "$(GREEN)IN USE$(NC)" || echo "$(RED)FREE$(NC)")
	@echo "  Port 50052 (AI):      " && (lsof -i :50052 > /dev/null 2>&1 && echo "$(GREEN)IN USE$(NC)" || echo "$(RED)FREE$(NC)")
	@echo "  Port 8000 (Backend):  " && (lsof -i :8000 > /dev/null 2>&1 && echo "$(GREEN)IN USE$(NC)" || echo "$(RED)FREE$(NC)")
	@echo "  Port 5173 (UI):       " && (lsof -i :5173 > /dev/null 2>&1 && echo "$(GREEN)IN USE$(NC)" || echo "$(RED)FREE$(NC)")

info: ## Display project information
	@echo "$(CYAN)╔═══════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║              AI-Powered OS - Project Info                ║$(NC)"
	@echo "$(CYAN)╚═══════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(YELLOW)Components:$(NC)"
	@echo "  • Kernel (Rust):      kernel/"
	@echo "  • Backend (Go):       backend/"
	@echo "  • AI Service (Python): ai-service/"
	@echo "  • UI (React/Electron): ui/"
	@echo ""
	@echo "$(YELLOW)Services:$(NC)"
	@echo "  • Kernel gRPC:     localhost:50051"
	@echo "  • AI gRPC:         localhost:50052"
	@echo "  • Backend HTTP:    localhost:8000"
	@echo "  • UI Dev Server:   localhost:5173"
	@echo ""
	@echo "$(YELLOW)Quick Start:$(NC)"
	@echo "  1. make setup       # Install all dependencies"
	@echo "  2. make proto       # Compile protocol buffers"
	@echo "  3. make build       # Build all components"
	@echo "  4. make dev         # Start everything"
	@echo ""

##@ Quick Commands

electron: start-ui ## Quick alias to start electron app

backend: start-backend ## Quick alias to start backend

restart: stop ## Restart all services
	@sleep 2
	@$(MAKE) dev

rebuild: clean build ## Clean and rebuild everything
	@echo "$(GREEN)✅ Rebuild complete$(NC)"

fresh: deep-clean setup proto build ## Fresh install (deep clean + setup + build)
	@echo "$(GREEN)✅ Fresh installation complete$(NC)"

