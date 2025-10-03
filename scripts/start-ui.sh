#!/bin/bash

# Start UI (Frontend)
# React/TypeScript/Electron
# Run from project root: ./scripts/start-ui.sh

cd "$(dirname "$0")/.." || exit

echo "=============================="
echo "🎨 Starting UI"
echo "=============================="
echo ""

cd ui || exit

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

echo "🚀 Starting Vite dev server..."
echo ""
npm run dev
