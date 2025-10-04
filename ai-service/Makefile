# AI Service Makefile

.PHONY: help install dev test lint format type-check clean run

help:
	@echo "AI Service - Available Commands:"
	@echo "  make install      - Install production dependencies"
	@echo "  make dev          - Install development dependencies"
	@echo "  make test         - Run tests with coverage"
	@echo "  make lint         - Run linter (ruff)"
	@echo "  make format       - Format code (black + ruff)"
	@echo "  make type-check   - Run type checker (mypy)"
	@echo "  make clean        - Clean cache and build files"
	@echo "  make run          - Run the server"
	@echo "  make check        - Run all checks (lint + type + test)"

install:
	pip install -r requirements.txt

dev:
	pip install -r requirements.txt
	pip install -e .

test:
	pytest tests/ -v --cov=src --cov-report=html --cov-report=term

lint:
	ruff check src/ tests/

format:
	black src/ tests/
	ruff check --fix src/ tests/

type-check:
	mypy src/

clean:
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".mypy_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".ruff_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "htmlcov" -exec rm -rf {} + 2>/dev/null || true
	rm -f .coverage

run:
	cd src && python3 -m server

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

