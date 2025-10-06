#!/bin/bash

# Stop All Services
# Run from project root: ./scripts/stop.sh

echo "Stopping all services..."

# Stop backend processes by port (more reliable)
# Kill process on port 50051 (Kernel)
KERNEL_PORT_PID=$(lsof -ti :50051 2>/dev/null)
if [ ! -z "$KERNEL_PORT_PID" ]; then
    kill -9 $KERNEL_PORT_PID 2>/dev/null && echo "   Kernel stopped (PID: $KERNEL_PORT_PID)"
else
    pkill -f "ai_os_kernel" 2>/dev/null && echo "   Kernel stopped" || echo "   Kernel not running"
fi

# Kill process on port 50052 (AI gRPC)
AI_PORT_PID=$(lsof -ti :50052 2>/dev/null)
if [ ! -z "$AI_PORT_PID" ]; then
    kill -9 $AI_PORT_PID 2>/dev/null && echo "   AI Service stopped (PID: $AI_PORT_PID)"
else
    pkill -f "ai-service.*server" 2>/dev/null && echo "   AI Service stopped" || echo "   AI Service not running"
fi

# Kill process on port 8000 (Backend)
BACKEND_PORT_PID=$(lsof -ti :8000 2>/dev/null)
if [ ! -z "$BACKEND_PORT_PID" ]; then
    kill -9 $BACKEND_PORT_PID 2>/dev/null && echo "   Backend stopped (PID: $BACKEND_PORT_PID)"
else
    pkill -f "backend/bin/server" 2>/dev/null && echo "   Backend stopped" || echo "   Backend not running"
fi

# Stop UI
pkill -f "vite" 2>/dev/null && echo "   UI stopped" || echo "   UI not running"

echo ""
echo "All services stopped"

