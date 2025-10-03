#!/bin/bash

# AI-OS System Shutdown Script
# Stops kernel + AI service cleanly

echo "=================================="
echo "🛑 AI-OS System Shutdown"
echo "=================================="
echo ""

# Get script directory
SCRIPT_DIR="$(dirname "$0")"
LOGS_DIR="$SCRIPT_DIR/../logs"

# Stop by PID files if they exist
if [ -f "$LOGS_DIR/kernel.pid" ]; then
    KERNEL_PID=$(cat "$LOGS_DIR/kernel.pid")
    echo "Stopping Kernel (PID $KERNEL_PID)..."
    kill -TERM $KERNEL_PID 2>/dev/null || true
    sleep 1
    kill -9 $KERNEL_PID 2>/dev/null || true
    rm "$LOGS_DIR/kernel.pid"
fi

if [ -f "$LOGS_DIR/ai-service.pid" ]; then
    AI_PID=$(cat "$LOGS_DIR/ai-service.pid")
    echo "Stopping AI Service (PID $AI_PID)..."
    kill -TERM $AI_PID 2>/dev/null || true
    sleep 1
    kill -9 $AI_PID 2>/dev/null || true
    rm "$LOGS_DIR/ai-service.pid"
fi

# Force cleanup by port
echo ""
echo "Cleaning up ports..."
lsof -ti:8000 | xargs kill -9 2>/dev/null || true
lsof -ti:50051 | xargs kill -9 2>/dev/null || true

# Force cleanup by process name
pkill -9 -f "python3.*main.py" 2>/dev/null || true
pkill -9 kernel 2>/dev/null || true

sleep 1

# Verify shutdown
echo ""
if lsof -ti:8000 > /dev/null 2>&1 || lsof -ti:50051 > /dev/null 2>&1; then
    echo "⚠️  Warning: Some ports still in use"
else
    echo "✅ All ports cleared"
fi

echo ""
echo "✅ System shutdown complete!"
echo "=================================="

