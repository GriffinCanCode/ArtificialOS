# AI Service Makefile

.PHONY: help install dev test test-unit test-integration test-fast test-watch test-benchmark test-slow test-grpc test-no-cov coverage lint format type-check clean run check proto

help:
	@echo "AI Service - Available Commands:"
	@echo ""
	@echo "Setup:"
	@echo "  make install        - Install production dependencies"
	@echo "  make dev            - Install development dependencies"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests with coverage"
	@echo "  make test-unit      - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-fast      - Run tests in parallel (fast)"
	@echo "  make test-watch     - Run tests in watch mode"
	@echo "  make test-benchmark - Run performance benchmarks"
	@echo "  make coverage       - Open HTML coverage report"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint           - Run linter (ruff)"
	@echo "  make format         - Format code (black + ruff)"
	@echo "  make type-check     - Run type checker (mypy)"
	@echo "  make check          - Run all checks (lint + type + test)"
	@echo ""
	@echo "Running:"
	@echo "  make run            - Run the gRPC server"
	@echo "  make clean          - Clean cache and build files"

install:
	pip install -r requirements.txt

dev:
	pip install -r requirements.txt
	pip install -e .

test:
	@echo "Running all tests with coverage..."
	PYTHONPATH=src pytest tests/ -v

test-unit:
	@echo "Running unit tests..."
	PYTHONPATH=src pytest tests/ -v -m unit

test-integration:
	@echo "Running integration tests..."
	PYTHONPATH=src pytest tests/ -v -m integration

test-fast:
	@echo "Running tests in parallel..."
	PYTHONPATH=src pytest tests/ -v -n auto

test-watch:
	@echo "Running tests in watch mode..."
	PYTHONPATH=src pytest-watch tests/ -- -v

test-benchmark:
	@echo "Running performance benchmarks..."
	PYTHONPATH=src pytest tests/ -v -m benchmark --benchmark-only

test-slow:
	@echo "Running slow tests..."
	PYTHONPATH=src pytest tests/ -v -m slow

test-grpc:
	@echo "Running gRPC tests..."
	PYTHONPATH=src pytest tests/ -v -m grpc

test-no-cov:
	@echo "Running tests without coverage (faster)..."
	PYTHONPATH=src pytest tests/ -v --no-cov

coverage:
	@echo "Opening coverage report..."
	open htmlcov/index.html || xdg-open htmlcov/index.html

lint:
	@echo "Running linter..."
	ruff check src/ tests/

format:
	@echo "Formatting code..."
	black src/ tests/
	ruff check --fix src/ tests/

type-check:
	@echo "Running type checker..."
	PYTHONPATH=src mypy src/

clean:
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".mypy_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".ruff_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "htmlcov" -exec rm -rf {} + 2>/dev/null || true
	rm -f .coverage

run:
	PYTHONPATH=src python3 -m server

check: lint type-check test
	@echo "✅ All checks passed!"

# Development workflow
watch-test:
	pytest-watch tests/

# Proto compilation
proto:
	python3 -m grpc_tools.protoc \
		-I../proto \
		--python_out=src \
		--grpc_python_out=src \
		../proto/ai.proto

