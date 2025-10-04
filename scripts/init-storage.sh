#!/bin/bash
# Initialize storage directories for AI-OS
# Required for kernel syscalls to persist data

STORAGE_ROOT="/tmp/ai-os-storage"

echo "🗄️  Initializing AI-OS storage directories..."

# Create directory structure
mkdir -p "${STORAGE_ROOT}/system/storage"
mkdir -p "${STORAGE_ROOT}/system/apps"
mkdir -p "${STORAGE_ROOT}/system/users"
mkdir -p "${STORAGE_ROOT}/system/sessions"

# Set permissions (writable by all for development)
chmod -R 755 "${STORAGE_ROOT}"

echo "✅ Storage directories created at ${STORAGE_ROOT}"
echo ""
echo "Structure:"
tree -L 3 "${STORAGE_ROOT}" 2>/dev/null || find "${STORAGE_ROOT}" -type d | sed 's|[^/]*/| |g'

echo ""
echo "Ready for backend services to persist data via kernel!"

