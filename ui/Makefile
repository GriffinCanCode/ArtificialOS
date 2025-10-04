# UI Makefile
# Manages development, building, testing, and deployment

.PHONY: help install dev build test test-watch test-ui test-coverage clean format lint preview

# Default target
help:
	@echo "UI Development Commands:"
	@echo "  make install        - Install dependencies"
	@echo "  make dev            - Start development server with Electron"
	@echo "  make dev-web        - Start web-only development server"
	@echo "  make build          - Build for production"
	@echo "  make preview        - Preview production build"
	@echo ""
	@echo "Testing Commands:"
	@echo "  make test           - Run tests in watch mode"
	@echo "  make test-run       - Run tests once"
	@echo "  make test-ui        - Open Vitest UI"
	@echo "  make test-coverage  - Generate coverage report"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint           - Run ESLint"
	@echo "  make format         - Format code with Prettier"
	@echo "  make format-check   - Check code formatting"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Remove build artifacts and dependencies"
	@echo "  make clean-coverage - Remove coverage reports"

# Install dependencies
install:
	npm install

# Development
dev:
	npm run dev

dev-web:
	npm run dev:react

# Build
build:
	npm run build

preview:
	npm run preview

# Testing
test:
	npm test

test-run:
	npm run test:run

test-ui:
	npm run test:ui

test-coverage:
	npm run test:coverage
	@echo "\nCoverage report generated in coverage/"
	@echo "Open coverage/index.html in your browser"

# Code Quality
lint:
	npm run lint

format:
	npm run format

format-check:
	npm run format:check

# Cleanup
clean:
	rm -rf node_modules dist build .vite

clean-coverage:
	rm -rf coverage

# Full clean and reinstall
reset: clean
	npm install
	@echo "\nProject reset complete!"

# CI/CD targets
ci-test:
	npm run test:run

ci-lint:
	npm run lint
	npm run format:check

ci-build:
	npm run build

ci: ci-lint ci-test ci-build
	@echo "\nCI pipeline complete!"

