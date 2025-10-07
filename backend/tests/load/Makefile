.PHONY: help install light medium heavy stress grpc ws report clean

help:
	@echo "Load Testing - Available Commands:"
	@echo ""
	@echo "Setup:"
	@echo "  make install        - Install vegeta load testing tool"
	@echo ""
	@echo "HTTP API Tests:"
	@echo "  make light          - Light load (10 req/sec for 30s)"
	@echo "  make medium         - Medium load (50 req/sec for 60s)"
	@echo "  make heavy          - Heavy load (100 req/sec for 60s)"
	@echo "  make stress         - Stress test (200 req/sec for 120s)"
	@echo ""
	@echo "gRPC Tests:"
	@echo "  make grpc           - gRPC load test (1000 requests, 10 workers)"
	@echo ""
	@echo "WebSocket Tests:"
	@echo "  make ws             - WebSocket load test (100 connections)"
	@echo ""
	@echo "Reports:"
	@echo "  make report         - Generate HTML report from last test"
	@echo "  make clean          - Clean up test results"

install:
	@echo "Installing vegeta..."
	go install github.com/tsenart/vegeta@latest

light:
	@echo "Running light load test (10 req/sec for 30s)..."
	vegeta attack -targets=targets/api.txt -rate=10 -duration=30s | \
		vegeta report | tee results/light.txt

medium:
	@echo "Running medium load test (50 req/sec for 60s)..."
	vegeta attack -targets=targets/api.txt -rate=50 -duration=60s | \
		vegeta report | tee results/medium.txt

heavy:
	@echo "Running heavy load test (100 req/sec for 60s)..."
	vegeta attack -targets=targets/api.txt -rate=100 -duration=60s | \
		vegeta report | tee results/heavy.txt

stress:
	@echo "Running stress test (200 req/sec for 120s)..."
	vegeta attack -targets=targets/api.txt -rate=200 -duration=120s | \
		vegeta report | tee results/stress.txt

grpc:
	@echo "Running gRPC load test..."
	@mkdir -p results
	go run grpc_load_test.go -addr localhost:50051 -requests 1000 -workers 10 | \
		tee results/grpc.txt

report:
	@echo "Generating HTML report..."
	vegeta attack -targets=targets/api.txt -rate=50 -duration=60s | \
		vegeta report -type=html > results/report.html
	@echo "Report generated: results/report.html"

clean:
	@echo "Cleaning up test results..."
	rm -rf results/*
	@echo "Clean complete"
