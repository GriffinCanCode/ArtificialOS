#!/bin/bash

# Start Complete Backend Stack
# Kernel (Rust) + AI Service (Python gRPC) + Backend (Go)
# Run from project root: ./scripts/start-backend.sh

cd "$(dirname "$0")/.." || exit

echo "=============================="
echo "🚀 Starting Backend Stack"
echo "=============================="
echo ""

# Create logs directory
mkdir -p logs

# Kill any existing backend processes by port (more reliable)
echo "🧹 Cleaning up existing processes..."

# Kill process on port 50051 (Kernel)
KERNEL_PORT_PID=$(lsof -ti :50051 2>/dev/null)
if [ ! -z "$KERNEL_PORT_PID" ]; then
    echo "   🔴 Killing old kernel process on port 50051 (PID: $KERNEL_PORT_PID)"
    kill -9 $KERNEL_PORT_PID 2>/dev/null || true
fi

# Kill process on port 50052 (AI gRPC)
AI_PORT_PID=$(lsof -ti :50052 2>/dev/null)
if [ ! -z "$AI_PORT_PID" ]; then
    echo "   🔴 Killing old AI service process on port 50052 (PID: $AI_PORT_PID)"
    kill -9 $AI_PORT_PID 2>/dev/null || true
fi

# Kill process on port 8000 (Backend)
BACKEND_PORT_PID=$(lsof -ti :8000 2>/dev/null)
if [ ! -z "$BACKEND_PORT_PID" ]; then
    echo "   🔴 Killing old backend process on port 8000 (PID: $BACKEND_PORT_PID)"
    kill -9 $BACKEND_PORT_PID 2>/dev/null || true
fi

# Also try pattern matching as backup
pkill -f "ai_os_kernel" 2>/dev/null || true
pkill -f "backend/bin/server" 2>/dev/null || true
pkill -f "grpc_server" 2>/dev/null || true

sleep 2

# Start Kernel (Rust)
echo "1️⃣  Starting Rust Kernel..."
cd kernel || exit
if [ ! -f "target/release/kernel" ]; then
    echo "   Building kernel..."
    cargo build --release 2>&1 | tee ../logs/kernel-build.log
fi
./target/release/kernel > ../logs/kernel.log 2>&1 &
KERNEL_PID=$!
echo "   ✅ Kernel started (PID: $KERNEL_PID)"
cd ..

sleep 2

# Start Python AI gRPC Service
echo "2️⃣  Starting Python AI gRPC Service..."
cd ai-service || exit
if [ ! -d "venv" ]; then
    echo "   ❌ Virtual environment not found. Please run: python3 -m venv venv"
    exit 1
fi
source venv/bin/activate
PYTHONPATH=src python3 -m grpc_server > ../logs/ai-grpc.log 2>&1 &
AI_PID=$!
echo "   ✅ AI gRPC Service started (PID: $AI_PID)"
cd ..

sleep 2

# Start Go Backend
echo "3️⃣  Starting Go Backend..."
cd backend || exit
if [ ! -f "bin/server" ]; then
    echo "   Building backend..."
    go build -o bin/server ./cmd/server
fi
./bin/server -port 8000 -kernel localhost:50051 -ai localhost:50052 > ../logs/backend.log 2>&1 &
BACKEND_PID=$!
echo "   ✅ Backend started (PID: $BACKEND_PID)"
cd ..

echo ""
echo "=============================="
echo "✅ Backend Stack Running"
echo "=============================="
echo ""
echo "🌐 Services:"
echo "   - Kernel:      localhost:50051"
echo "   - AI gRPC:     localhost:50052"
echo "   - Backend:     localhost:8000"
echo ""
echo "📊 Logs:"
echo "   - Kernel:      logs/kernel.log"
echo "   - AI gRPC:     logs/ai-grpc.log"
echo "   - Backend:     logs/backend.log"
echo ""
echo "🛑 To stop: pkill -f kernel && pkill -f grpc_server && pkill -f backend"
echo "📺 Tail logs: tail -f logs/backend.log"
echo ""
