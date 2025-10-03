#!/bin/bash
# Start UI Service

echo "🎨 Starting AI-Powered OS - UI"
echo "==============================="

cd "$(dirname "$0")/../ui"

# Kill any existing process on port 5173
echo "Checking for existing processes on port 5173..."
if lsof -ti:5173 >/dev/null 2>&1; then
    echo "Killing existing process on port 5173..."
    lsof -ti:5173 | xargs kill -9 2>/dev/null || true
    sleep 1
    echo "✅ Port 5173 freed"
fi

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Start the UI
echo ""
echo "Starting Electron UI..."
echo ""

npm run dev

