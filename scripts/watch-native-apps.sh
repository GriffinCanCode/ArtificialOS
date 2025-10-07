#!/bin/bash
# Watch and rebuild native apps on file changes
# Provides hot module replacement during development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NATIVE_APPS_DIR="$PROJECT_ROOT/apps/native"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}======================================"
echo "Native Apps Development Watcher"
echo -e "======================================${NC}"
echo ""

# Check for required tools
if ! command -v fswatch &> /dev/null; then
  echo -e "${YELLOW}⚠️  fswatch not found. Install it for better file watching:${NC}"
  echo "   macOS: brew install fswatch"
  echo "   Linux: apt-get install fswatch / yum install fswatch"
  echo ""
  echo "Falling back to basic watch mode..."
  USE_FSWATCH=false
else
  USE_FSWATCH=true
fi

# Parse arguments
APP_NAME=""
WATCH_MODE="auto"

while [[ $# -gt 0 ]]; do
  case $1 in
    -a|--app)
      APP_NAME="$2"
      shift 2
      ;;
    -m|--mode)
      WATCH_MODE="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  -a, --app <name>    Watch specific app only"
      echo "  -m, --mode <mode>   Watch mode: auto, build, serve"
      echo "  -h, --help          Show this help"
      echo ""
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

# Function to build app
build_app() {
  local app_dir=$1
  local app_name=$(basename "$app_dir")

  echo -e "${YELLOW}🔨 Rebuilding: $app_name${NC}"

  if (cd "$app_dir" && npm run build 2>&1 | grep -v "npm WARN"); then
    echo -e "${GREEN}✅ $app_name rebuilt successfully${NC}"
    echo ""
  else
    echo -e "${RED}❌ $app_name build failed${NC}"
    echo ""
  fi
}

# Function to start dev server for app
dev_server() {
  local app_dir=$1
  local app_name=$(basename "$app_dir")

  echo -e "${CYAN}🚀 Starting dev server: $app_name${NC}"
  (cd "$app_dir" && npm run dev) &
  local pid=$!
  echo "$pid" > "/tmp/native-app-${app_name}.pid"
}

# Function to watch app with fswatch
watch_app_fswatch() {
  local app_dir=$1
  local app_name=$(basename "$app_dir")

  echo -e "${GREEN}👀 Watching: $app_name${NC}"
  echo "   Path: $app_dir/src"
  echo ""

  fswatch -o "$app_dir/src" | while read num; do
    echo -e "${YELLOW}📝 Changes detected in $app_name${NC}"
    build_app "$app_dir"
  done &
}

# Function to watch app with basic polling
watch_app_poll() {
  local app_dir=$1
  local app_name=$(basename "$app_dir")
  local last_mod=$(find "$app_dir/src" -type f -name "*.ts" -o -name "*.tsx" -o -name "*.css" | xargs stat -f %m 2>/dev/null | sort -rn | head -1)

  echo -e "${GREEN}👀 Watching: $app_name${NC}"
  echo "   Path: $app_dir/src"
  echo ""

  while true; do
    sleep 2
    local current_mod=$(find "$app_dir/src" -type f -name "*.ts" -o -name "*.tsx" -o -name "*.css" | xargs stat -f %m 2>/dev/null | sort -rn | head -1)

    if [ "$current_mod" != "$last_mod" ]; then
      echo -e "${YELLOW}📝 Changes detected in $app_name${NC}"
      build_app "$app_dir"
      last_mod=$current_mod
    fi
  done &
}

# Cleanup on exit
cleanup() {
  echo ""
  echo -e "${YELLOW}Stopping watchers...${NC}"
  jobs -p | xargs -r kill 2>/dev/null

  # Kill dev servers
  for pid_file in /tmp/native-app-*.pid; do
    if [ -f "$pid_file" ]; then
      kill $(cat "$pid_file") 2>/dev/null || true
      rm "$pid_file"
    fi
  done

  echo -e "${GREEN}Done!${NC}"
  exit 0
}

trap cleanup INT TERM EXIT

# Main watch logic
if [ -n "$APP_NAME" ]; then
  # Watch specific app
  APP_DIR="$NATIVE_APPS_DIR/$APP_NAME"

  if [ ! -d "$APP_DIR" ]; then
    echo -e "${RED}❌ App not found: $APP_NAME${NC}"
    exit 1
  fi

  if [ ! -f "$APP_DIR/package.json" ]; then
    echo -e "${RED}❌ Invalid app: No package.json found${NC}"
    exit 1
  fi

  # Initial build
  build_app "$APP_DIR"

  # Start watching
  if [ "$WATCH_MODE" = "serve" ]; then
    dev_server "$APP_DIR"
    wait
  elif [ "$USE_FSWATCH" = true ]; then
    watch_app_fswatch "$APP_DIR"
    wait
  else
    watch_app_poll "$APP_DIR"
    wait
  fi
else
  # Watch all apps
  if [ ! -d "$NATIVE_APPS_DIR" ]; then
    echo -e "${RED}❌ No native apps directory found${NC}"
    exit 1
  fi

  APP_COUNT=0
  for app_dir in "$NATIVE_APPS_DIR"/*; do
    if [ ! -d "$app_dir" ] || [ ! -f "$app_dir/package.json" ]; then
      continue
    fi

    # Initial build
    build_app "$app_dir"

    # Start watching
    if [ "$USE_FSWATCH" = true ]; then
      watch_app_fswatch "$app_dir"
    else
      watch_app_poll "$app_dir"
    fi

    APP_COUNT=$((APP_COUNT + 1))
  done

  if [ $APP_COUNT -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No native apps found to watch${NC}"
    exit 0
  fi

  echo -e "${CYAN}======================================"
  echo "Watching $APP_COUNT app(s)"
  echo "Press Ctrl+C to stop"
  echo -e "======================================${NC}"
  echo ""

  # Wait for all background jobs
  wait
fi
