#!/bin/bash

# Compile Protocol Buffers
# Run from project root

echo "🔨 Compiling Protocol Buffers..."

# Compile kernel proto for Go
echo "📦 Compiling kernel.proto for Go..."
cd proto
protoc --go_out=../backend/proto/kernel --go_opt=paths=source_relative \
    --go-grpc_out=../backend/proto/kernel --go-grpc_opt=paths=source_relative \
    kernel.proto

# Compile AI proto for Go
echo "📦 Compiling ai.proto for Go..."
cd ../backend/proto
protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    ai.proto

# Compile AI proto for Python
echo "📦 Compiling ai.proto for Python..."
cd ../../ai-service
source venv/bin/activate
cd proto
python3 -m grpc_tools.protoc -I. --python_out=../src --grpc_python_out=../src ai.proto

echo "✅ Protocol buffer compilation complete!"

