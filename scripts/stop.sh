#!/bin/bash

# Stop All Services
# Run from project root: ./scripts/stop.sh

echo "🛑 Stopping all services..."

# Stop all backend processes
pkill -f "ai_os_kernel" && echo "   ✅ Kernel stopped"
pkill -f "grpc_server" && echo "   ✅ AI Service stopped"
pkill -f "backend/bin/server" && echo "   ✅ Backend stopped"

# Stop UI
pkill -f "vite" && echo "   ✅ UI stopped"

echo ""
echo "✅ All services stopped"

