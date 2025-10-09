#!/bin/bash

# Initialize AgentOS storage structure
# Run from project root

# Use consistent path across kernel, backend, and frontend
STORAGE_ROOT="${KERNEL_STORAGE_PATH:-/tmp/ai-os-storage}"

echo "Initializing AgentOS storage structure at $STORAGE_ROOT..."

# Create system directories
mkdir -p "$STORAGE_ROOT/system/apps"
mkdir -p "$STORAGE_ROOT/system/sessions"
mkdir -p "$STORAGE_ROOT/system/users"
mkdir -p "$STORAGE_ROOT/system/config"
mkdir -p "$STORAGE_ROOT/system/logs"

# Create OS-specific user directories
mkdir -p "$STORAGE_ROOT/Home"
mkdir -p "$STORAGE_ROOT/Applications"
mkdir -p "$STORAGE_ROOT/Documents"
mkdir -p "$STORAGE_ROOT/Data"
mkdir -p "$STORAGE_ROOT/System"

# Create app-specific storage
mkdir -p "$STORAGE_ROOT/Data/storage"

# Create storage directories for all native apps
if [ -d "../apps/native" ]; then
  for manifest in ../apps/native/*/manifest.json; do
    if [ -f "$manifest" ]; then
      # Extract app ID from manifest.json (requires jq)
      if command -v jq >/dev/null 2>&1; then
        app_id=$(jq -r '.id' "$manifest" 2>/dev/null)
        if [ -n "$app_id" ] && [ "$app_id" != "null" ]; then
          mkdir -p "$STORAGE_ROOT/Data/storage/$app_id"
          echo "  Created storage for app: $app_id"
        fi
      fi
    fi
  done
fi

echo "Storage structure created at $STORAGE_ROOT"
echo ""
echo "Directory structure:"
echo "  $STORAGE_ROOT/"
echo "    ├── Home/              # User home directory"
echo "    ├── Applications/      # Installed applications"
echo "    ├── Documents/         # User documents"
echo "    ├── Data/              # App data storage"
echo "    ├── System/            # System configuration"
echo "    └── system/            # Backend system files"
echo "        ├── apps/          # App registry (.aiapp files)"
echo "        ├── sessions/      # Saved sessions"
echo "        └── users/         # User data"
echo ""
echo "Ready for backend services to persist data via kernel!"
