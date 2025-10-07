#!/bin/bash
# Build all native TypeScript/React apps
# This script compiles native apps into bundled JavaScript for production

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NATIVE_APPS_DIR="$PROJECT_ROOT/apps/native"
DIST_DIR="$PROJECT_ROOT/apps/dist"

echo "======================================"
echo "Building Native Apps"
echo "======================================"
echo ""

# Check if native apps directory exists
if [ ! -d "$NATIVE_APPS_DIR" ]; then
  echo "⚠️  No native apps directory found at: $NATIVE_APPS_DIR"
  echo "   Creating directory..."
  mkdir -p "$NATIVE_APPS_DIR"
  exit 0
fi

# Create dist directory if it doesn't exist
mkdir -p "$DIST_DIR"

# Count apps
APP_COUNT=0
SUCCESS_COUNT=0
FAILED_COUNT=0

# Build each app
for app_dir in "$NATIVE_APPS_DIR"/*; do
  if [ ! -d "$app_dir" ]; then
    continue
  fi

  APP_NAME=$(basename "$app_dir")
  APP_COUNT=$((APP_COUNT + 1))

  echo "📦 Building: $APP_NAME"
  echo "   Path: $app_dir"

  # Check if package.json exists
  if [ ! -f "$app_dir/package.json" ]; then
    echo "   ⚠️  No package.json found, skipping..."
    FAILED_COUNT=$((FAILED_COUNT + 1))
    echo ""
    continue
  fi

  # Check if node_modules exists
  if [ ! -d "$app_dir/node_modules" ]; then
    echo "   📥 Installing dependencies..."
    (cd "$app_dir" && npm install --silent)
  fi

  # Build the app
  echo "   🔨 Building..."
  if (cd "$app_dir" && npm run build --silent 2>&1); then
    echo "   ✅ Build successful"
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
  else
    echo "   ❌ Build failed"
    FAILED_COUNT=$((FAILED_COUNT + 1))
  fi

  echo ""
done

# Summary
echo "======================================"
echo "Build Summary"
echo "======================================"
echo "Total apps:   $APP_COUNT"
echo "✅ Succeeded:  $SUCCESS_COUNT"
echo "❌ Failed:     $FAILED_COUNT"
echo ""

if [ $FAILED_COUNT -eq 0 ]; then
  echo "🎉 All apps built successfully!"
  exit 0
else
  echo "⚠️  Some apps failed to build."
  exit 1
fi
