#!/bin/bash

# AI-OS System Startup Script
# Starts kernel + AI service with proper cleanup and port management

set -e

echo "=================================="
echo "🚀 AI-OS System Startup"
echo "=================================="
echo ""

# Step 1: Clean up any existing processes
echo "Step 1: Cleaning up existing processes..."
lsof -ti:8000 | xargs kill -9 2>/dev/null || true
lsof -ti:50051 | xargs kill -9 2>/dev/null || true
pkill -9 -f "python3.*main.py" 2>/dev/null || true
pkill -9 kernel 2>/dev/null || true
sleep 1
echo "  ✅ Cleanup complete"

# Step 2: Start Rust Kernel
echo ""
echo "Step 2: Starting Rust Kernel..."
cd "$(dirname "$0")/../kernel"
RUST_LOG=info cargo run --quiet 2>&1 | tee ../logs/kernel.log &
KERNEL_PID=$!
echo "  Kernel PID: $KERNEL_PID"
echo $KERNEL_PID > ../logs/kernel.pid

# Wait for kernel to start
echo "  Waiting for kernel to initialize..."
sleep 4

# Verify kernel is running
if ! ps -p $KERNEL_PID > /dev/null 2>&1; then
    echo "  ❌ Kernel failed to start!"
    cat ../logs/kernel.log
    exit 1
fi

# Verify gRPC port is open
if ! lsof -ti:50051 > /dev/null 2>&1; then
    echo "  ⚠️  Warning: Kernel gRPC port (50051) not open yet"
else
    echo "  ✅ Kernel gRPC server ready on port 50051"
fi

# Step 3: Start Python AI Service
echo ""
echo "Step 3: Starting Python AI Service..."
cd ../ai-service
source venv/bin/activate

# Start AI service
python3 src/main.py 2>&1 | tee ../logs/ai-service.log &
AI_PID=$!
echo "  AI Service PID: $AI_PID"
echo $AI_PID > ../logs/ai-service.pid

# Wait for AI service to start
echo "  Waiting for AI service to initialize..."
sleep 6

# Verify AI service is running
if ! ps -p $AI_PID > /dev/null 2>&1; then
    echo "  ❌ AI Service failed to start!"
    cat ../logs/ai-service.log
    exit 1
fi

# Verify HTTP port is open
if ! lsof -ti:8000 > /dev/null 2>&1; then
    echo "  ⚠️  Warning: AI Service HTTP port (8000) not open"
else
    echo "  ✅ AI Service HTTP server ready on port 8000"
fi

# Step 4: Test integration
echo ""
echo "Step 4: Testing system integration..."
sleep 2

# Test health endpoint
echo ""
echo "  📊 System Health:"
curl -s http://localhost:8000/health | python3 -m json.tool | grep -A 20 "status"

# Check kernel connection
KERNEL_CONNECTED=$(curl -s http://localhost:8000/health | python3 -c 'import json, sys; print(json.load(sys.stdin)["kernel"]["connected"])' 2>/dev/null || echo "false")
DEFAULT_PID=$(curl -s http://localhost:8000/health | python3 -c 'import json, sys; print(json.load(sys.stdin)["kernel"]["default_pid"])' 2>/dev/null || echo "null")

echo ""
echo "  🔌 Kernel Integration:"
echo "    Connected: $KERNEL_CONNECTED"
echo "    Default PID: $DEFAULT_PID"

if [ "$KERNEL_CONNECTED" = "True" ]; then
    echo "    ✅ Kernel and AI Service are connected!"
else
    echo "    ⚠️  Kernel connection pending or failed"
    echo ""
    echo "  Recent AI Service logs:"
    tail -20 ../logs/ai-service.log | grep -i "kernel\|grpc\|error" || echo "    (no kernel-related logs)"
fi

# Step 5: Summary
echo ""
echo "=================================="
echo "📋 System Status Summary"
echo "=================================="
echo "  Kernel:      Running (PID $KERNEL_PID) - Port 50051"
echo "  AI Service:  Running (PID $AI_PID) - Port 8000"
echo ""
echo "  Logs:"
echo "    Kernel:     tail -f logs/kernel.log"
echo "    AI Service: tail -f logs/ai-service.log"
echo ""
echo "  To stop:"
echo "    kill $KERNEL_PID $AI_PID"
echo "    or run: ./scripts/stop-system.sh"
echo ""
echo "✅ System started successfully!"
echo "=================================="

