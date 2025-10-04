#!/bin/bash
# Rebuild llama-cpp-python with Metal optimization for M4 Max

cd "$(dirname "$0")"
source venv/bin/activate

echo "🔨 Rebuilding llama-cpp-python with Metal support..."
CMAKE_ARGS="-DLLAMA_METAL=on -DLLAMA_METAL_EMBED_LIBRARY=on" \
  pip install --upgrade --force-reinstall --no-cache-dir llama-cpp-python

echo "✅ Done! GPU layers set to -1 (all layers on Metal GPU)"
echo "🚀 Restart your AI service to see 2-5x speedup"

