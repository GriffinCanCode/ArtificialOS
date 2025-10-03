#!/bin/bash
# Start AI Service

echo "🚀 Starting AI-Powered OS - AI Service"
echo "========================================"

cd "$(dirname "$0")/../ai-service"

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
fi

# Activate and install dependencies
source venv/bin/activate
pip install -q -r requirements.txt

# Start the service
echo ""
echo "Starting service on http://localhost:8000"
echo "Press Ctrl+C to stop"
echo ""

python3 src/main.py

