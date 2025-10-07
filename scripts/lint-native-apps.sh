#!/bin/bash
# Lint and type-check native apps
# Ensures code quality across all native apps

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NATIVE_APPS_DIR="$PROJECT_ROOT/apps/native"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}======================================"
echo "Native Apps Linting"
echo -e "======================================${NC}"
echo ""

# Parse arguments
FIX_MODE=false
APP_NAME=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --fix)
      FIX_MODE=true
      shift
      ;;
    -a|--app)
      APP_NAME="$2"
      shift 2
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

# Counters
TOTAL_APPS=0
PASSED_APPS=0
FAILED_APPS=0

# Lint app
lint_app() {
  local app_dir=$1
  local app_name=$2

  echo -e "${YELLOW}🔍 Linting: $app_name${NC}"

  # Check if package.json exists
  if [ ! -f "$app_dir/package.json" ]; then
    echo -e "  ${RED}✗ No package.json found${NC}"
    return 1
  fi

  # Check if node_modules exists
  if [ ! -d "$app_dir/node_modules" ]; then
    echo -e "  ${YELLOW}⚠ No node_modules, installing...${NC}"
    (cd "$app_dir" && npm install --silent)
  fi

  local lint_failed=false

  # TypeScript type checking
  echo -e "  ${CYAN}→ Type checking...${NC}"
  if (cd "$app_dir" && npx tsc --noEmit 2>&1); then
    echo -e "  ${GREEN}✓ TypeScript: no errors${NC}"
  else
    echo -e "  ${RED}✗ TypeScript: type errors found${NC}"
    lint_failed=true
  fi

  # ESLint (if configured)
  if [ -f "$app_dir/.eslintrc" ] || [ -f "$app_dir/.eslintrc.json" ] || [ -f "$app_dir/.eslintrc.js" ]; then
    echo -e "  ${CYAN}→ Running ESLint...${NC}"
    if [ "$FIX_MODE" = true ]; then
      if (cd "$app_dir" && npx eslint src --ext .ts,.tsx --fix); then
        echo -e "  ${GREEN}✓ ESLint: passed (with fixes)${NC}"
      else
        echo -e "  ${RED}✗ ESLint: errors found${NC}"
        lint_failed=true
      fi
    else
      if (cd "$app_dir" && npx eslint src --ext .ts,.tsx); then
        echo -e "  ${GREEN}✓ ESLint: passed${NC}"
      else
        echo -e "  ${RED}✗ ESLint: errors found${NC}"
        lint_failed=true
      fi
    fi
  else
    echo -e "  ${YELLOW}⚠ ESLint not configured${NC}"
  fi

  # Prettier (if configured)
  if [ -f "$app_dir/.prettierrc" ] || [ -f "$app_dir/.prettierrc.json" ] || [ -f "$app_dir/.prettierrc.js" ]; then
    echo -e "  ${CYAN}→ Running Prettier...${NC}"
    if [ "$FIX_MODE" = true ]; then
      if (cd "$app_dir" && npx prettier --write "src/**/*.{ts,tsx,css}" 2>&1 >/dev/null); then
        echo -e "  ${GREEN}✓ Prettier: formatted${NC}"
      else
        echo -e "  ${RED}✗ Prettier: errors${NC}"
        lint_failed=true
      fi
    else
      if (cd "$app_dir" && npx prettier --check "src/**/*.{ts,tsx,css}" 2>&1 >/dev/null); then
        echo -e "  ${GREEN}✓ Prettier: passed${NC}"
      else
        echo -e "  ${YELLOW}⚠ Prettier: formatting needed${NC}"
      fi
    fi
  fi

  echo ""

  if [ "$lint_failed" = true ]; then
    return 1
  else
    return 0
  fi
}

# Check if native apps directory exists
if [ ! -d "$NATIVE_APPS_DIR" ]; then
  echo -e "${YELLOW}⚠️  No native apps directory found${NC}"
  exit 0
fi

# Lint specific app or all apps
if [ -n "$APP_NAME" ]; then
  # Lint specific app
  APP_DIR="$NATIVE_APPS_DIR/$APP_NAME"

  if [ ! -d "$APP_DIR" ]; then
    echo -e "${RED}❌ App not found: $APP_NAME${NC}"
    exit 1
  fi

  TOTAL_APPS=1
  if lint_app "$APP_DIR" "$APP_NAME"; then
    PASSED_APPS=1
  else
    FAILED_APPS=1
  fi
else
  # Lint all apps
  for app_dir in "$NATIVE_APPS_DIR"/*; do
    if [ ! -d "$app_dir" ]; then
      continue
    fi

    app_name=$(basename "$app_dir")
    TOTAL_APPS=$((TOTAL_APPS + 1))

    if lint_app "$app_dir" "$app_name"; then
      PASSED_APPS=$((PASSED_APPS + 1))
    else
      FAILED_APPS=$((FAILED_APPS + 1))
    fi
  done
fi

# Summary
echo -e "${CYAN}======================================"
echo "Linting Summary"
echo -e "======================================${NC}"
echo "Total apps:   $TOTAL_APPS"
echo -e "${GREEN}✓ Passed:     $PASSED_APPS${NC}"
echo -e "${RED}✗ Failed:     $FAILED_APPS${NC}"
echo ""

if [ $FAILED_APPS -eq 0 ]; then
  echo -e "${GREEN}🎉 All apps passed linting!${NC}"
  exit 0
else
  echo -e "${RED}⚠️  Some apps have linting errors${NC}"
  if [ "$FIX_MODE" = false ]; then
    echo "Tip: Run with --fix to automatically fix issues"
  fi
  exit 1
fi
