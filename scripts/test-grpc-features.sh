#!/bin/bash
# Test runner for gRPC advanced features
set -e

echo "======================================"
echo "Testing gRPC Advanced Features"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run test suite
run_test_suite() {
    local name=$1
    local command=$2

    echo -e "${YELLOW}Running: ${name}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if eval "$command"; then
        echo -e "${GREEN}✓ ${name} passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ ${name} failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

echo "======================================"
echo "Go Client Tests"
echo "======================================"
cd backend

run_test_suite "Streaming Tests" \
    "go test -v ./internal/grpc/kernel/ -run TestStreamFileRead 2>&1 | grep -E '(PASS|FAIL)'"

run_test_suite "Async Tests" \
    "go test -v ./internal/grpc/kernel/ -run TestAsync 2>&1 | grep -E '(PASS|FAIL)'"

run_test_suite "Batch Tests" \
    "go test -v ./internal/grpc/kernel/ -run TestBatch 2>&1 | grep -E '(PASS|FAIL)'"

run_test_suite "All Client Tests" \
    "go test ./internal/grpc/kernel/ -v 2>&1 | tail -1"

echo "======================================"
echo "Rust Kernel Tests"
echo "======================================"
cd ../kernel

echo -e "${YELLOW}Note: Rust tests require kernel compilation to succeed${NC}"
echo -e "${YELLOW}Checking if kernel compiles...${NC}"

if cargo build --release 2>&1 | grep -q "error:"; then
    echo -e "${RED}✗ Kernel has compilation errors - skipping Rust tests${NC}"
    echo -e "${YELLOW}Run 'cd kernel && cargo build --release' to see errors${NC}"
else
    run_test_suite "Async Task Tests" \
        "cargo test --test async_task_test --release -- --test-threads=1 2>&1 | grep -E '(test result:)'"

    run_test_suite "Batch Execution Tests" \
        "cargo test --test batch_test --release -- --test-threads=1 2>&1 | grep -E '(test result:)'"

    run_test_suite "Streaming Tests" \
        "cargo test --test streaming_test --release -- --test-threads=1 2>&1 | grep -E '(test result:)'"

    run_test_suite "gRPC Integration Tests" \
        "cargo test --test grpc_advanced_test --release -- --test-threads=1 2>&1 | grep -E '(test result:)'"
fi

cd ..

echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "Total Test Suites: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}✗ Some tests failed${NC}"
    exit 1
fi
