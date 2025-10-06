#!/bin/bash

# Initialize AgentOS storage structure
# Run from project root

STORAGE_ROOT="/tmp/agentos-storage"

echo "Initializing AgentOS storage structure..."

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
