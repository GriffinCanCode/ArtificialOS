#!/bin/bash
# Start UI Service

echo "🎨 Starting AI-Powered OS - UI"
echo "==============================="

cd "$(dirname "$0")/ui"

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

