#!/bin/bash
# Test script for folder creation in file-explorer
# Verifies the fix for folder creation issue

set -e

echo "==================================="
echo "Testing Folder Creation Fix"
echo "==================================="
echo ""

# 1. Check storage path configuration
echo "1. Checking storage path configuration..."
STORAGE_PATH="${KERNEL_STORAGE_PATH:-/tmp/ai-os-storage}"
echo "   Storage path: $STORAGE_PATH"

# 2. Initialize storage if needed
echo ""
echo "2. Initializing storage structure..."
./scripts/init-storage.sh

# 3. Verify directory structure
echo ""
echo "3. Verifying directory structure..."
if [ -d "$STORAGE_PATH" ]; then
    echo "   ✓ Storage root exists: $STORAGE_PATH"
    ls -la "$STORAGE_PATH" | head -n 10
else
    echo "   ✗ Storage root missing: $STORAGE_PATH"
    exit 1
fi

# 4. Test direct folder creation
echo ""
echo "4. Testing direct folder creation..."
TEST_FOLDER="$STORAGE_PATH/test-$(date +%s)"
mkdir -p "$TEST_FOLDER"
if [ -d "$TEST_FOLDER" ]; then
    echo "   ✓ Direct folder creation works: $TEST_FOLDER"
    rm -rf "$TEST_FOLDER"
else
    echo "   ✗ Direct folder creation failed"
    exit 1
fi

# 5. Verify permissions
echo ""
echo "5. Checking permissions..."
ls -ld "$STORAGE_PATH"
if [ -w "$STORAGE_PATH" ]; then
    echo "   ✓ Storage path is writable"
else
    echo "   ✗ Storage path is not writable"
    exit 1
fi

echo ""
echo "==================================="
echo "Pre-flight checks complete!"
echo "==================================="
echo ""
echo "Next steps:"
echo "1. Start the kernel: make kernel"
echo "2. Start the backend: make backend"
echo "3. Start the UI: make ui"
echo "4. Open file-explorer and try creating a folder"
echo ""
echo "Logs to check:"
echo "- Kernel: logs/kernel.log"
echo "- Backend: logs/backend.log"
echo ""
echo "Expected path mapping:"
echo "- Frontend/Native app: /storage"
echo "- Kernel VFS mount: /storage → $STORAGE_PATH"
echo "- Backend base path: $STORAGE_PATH"
echo ""

