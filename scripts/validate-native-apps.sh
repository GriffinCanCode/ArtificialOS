#!/bin/bash
# Validate native app manifests and structure
# Ensures apps follow the correct patterns and conventions

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
echo "Native Apps Validation"
echo -e "======================================${NC}"
echo ""

# Validation counters
TOTAL_APPS=0
VALID_APPS=0
INVALID_APPS=0
WARNINGS=0

# Required manifest fields
REQUIRED_FIELDS=("id" "name" "type" "version" "icon" "category" "author" "exports")

# Valid app types
VALID_TYPES=("native_web" "native_proc")

# Validate manifest JSON
validate_manifest() {
  local manifest_file=$1
  local app_name=$2
  local errors=0
  local warnings=0

  echo -e "${YELLOW}📋 Validating: $app_name${NC}"

  # Check if manifest exists
  if [ ! -f "$manifest_file" ]; then
    echo -e "  ${RED}✗ manifest.json not found${NC}"
    return 1
  fi

  # Check if valid JSON
  if ! jq empty "$manifest_file" 2>/dev/null; then
    echo -e "  ${RED}✗ Invalid JSON format${NC}"
    return 1
  fi

  # Check required fields
  for field in "${REQUIRED_FIELDS[@]}"; do
    if ! jq -e ".$field" "$manifest_file" >/dev/null 2>&1; then
      echo -e "  ${RED}✗ Missing required field: $field${NC}"
      errors=$((errors + 1))
    fi
  done

  # Check app type
  local app_type=$(jq -r '.type' "$manifest_file" 2>/dev/null)
  if [ ! -z "$app_type" ]; then
    local valid_type=false
    for valid in "${VALID_TYPES[@]}"; do
      if [ "$app_type" = "$valid" ]; then
        valid_type=true
        break
      fi
    done

    if [ "$valid_type" = false ]; then
      echo -e "  ${RED}✗ Invalid type: $app_type (must be: ${VALID_TYPES[*]})${NC}"
      errors=$((errors + 1))
    fi
  fi

  # Check ID format (lowercase, hyphen-separated)
  local app_id=$(jq -r '.id' "$manifest_file" 2>/dev/null)
  if [ ! -z "$app_id" ]; then
    if ! echo "$app_id" | grep -qE '^[a-z][a-z0-9-]*$'; then
      echo -e "  ${YELLOW}⚠ ID should be lowercase with hyphens: $app_id${NC}"
      warnings=$((warnings + 1))
    fi
  fi

  # Check version format (semver)
  local version=$(jq -r '.version' "$manifest_file" 2>/dev/null)
  if [ ! -z "$version" ]; then
    if ! echo "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+'; then
      echo -e "  ${YELLOW}⚠ Version should follow semver: $version${NC}"
      warnings=$((warnings + 1))
    fi
  fi

  # Check exports
  local exports=$(jq -r '.exports.component' "$manifest_file" 2>/dev/null)
  if [ -z "$exports" ] || [ "$exports" = "null" ]; then
    echo -e "  ${RED}✗ Missing exports.component${NC}"
    errors=$((errors + 1))
  fi

  if [ $errors -eq 0 ]; then
    if [ $warnings -eq 0 ]; then
      echo -e "  ${GREEN}✓ Valid manifest${NC}"
    else
      echo -e "  ${GREEN}✓ Valid${NC} ${YELLOW}($warnings warnings)${NC}"
    fi
    return 0
  else
    echo -e "  ${RED}✗ Invalid manifest ($errors errors, $warnings warnings)${NC}"
    return 1
  fi
}

# Validate app structure
validate_structure() {
  local app_dir=$1
  local app_name=$2
  local errors=0

  # Check required files
  local required_files=("src/index.tsx" "src/App.tsx" "package.json" "tsconfig.json")

  for file in "${required_files[@]}"; do
    if [ ! -f "$app_dir/$file" ]; then
      echo -e "  ${RED}✗ Missing required file: $file${NC}"
      errors=$((errors + 1))
    fi
  done

  # Check for vite.config.ts or vite.config.js
  if [ ! -f "$app_dir/vite.config.ts" ] && [ ! -f "$app_dir/vite.config.js" ]; then
    echo -e "  ${YELLOW}⚠ Missing vite.config.ts (recommended)${NC}"
    WARNINGS=$((WARNINGS + 1))
  fi

  # Check src directory structure
  if [ ! -d "$app_dir/src" ]; then
    echo -e "  ${RED}✗ Missing src directory${NC}"
    errors=$((errors + 1))
  fi

  return $errors
}

# Validate TypeScript configuration
validate_typescript() {
  local app_dir=$1
  local errors=0

  if [ ! -f "$app_dir/tsconfig.json" ]; then
    return 0
  fi

  # Check for strict mode
  if ! jq -e '.compilerOptions.strict == true' "$app_dir/tsconfig.json" >/dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠ TypeScript strict mode not enabled${NC}"
    WARNINGS=$((WARNINGS + 1))
  fi

  # Check for jsx setting
  local jsx=$(jq -r '.compilerOptions.jsx' "$app_dir/tsconfig.json" 2>/dev/null)
  if [ "$jsx" != "react-jsx" ] && [ "$jsx" != "react" ]; then
    echo -e "  ${YELLOW}⚠ Unexpected jsx setting: $jsx${NC}"
    WARNINGS=$((WARNINGS + 1))
  fi

  return $errors
}

# Validate dependencies
validate_dependencies() {
  local app_dir=$1
  local errors=0

  if [ ! -f "$app_dir/package.json" ]; then
    return 0
  fi

  # Check for required dependencies
  if ! jq -e '.dependencies.react' "$app_dir/package.json" >/dev/null 2>&1; then
    echo -e "  ${RED}✗ Missing dependency: react${NC}"
    errors=$((errors + 1))
  fi

  if ! jq -e '.dependencies["react-dom"]' "$app_dir/package.json" >/dev/null 2>&1; then
    echo -e "  ${RED}✗ Missing dependency: react-dom${NC}"
    errors=$((errors + 1))
  fi

  # Check for required dev dependencies
  if ! jq -e '.devDependencies.typescript' "$app_dir/package.json" >/dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠ Missing devDependency: typescript${NC}"
    WARNINGS=$((WARNINGS + 1))
  fi

  if ! jq -e '.devDependencies.vite' "$app_dir/package.json" >/dev/null 2>&1; then
    echo -e "  ${RED}✗ Missing devDependency: vite${NC}"
    errors=$((errors + 1))
  fi

  return $errors
}

# Check if jq is installed
if ! command -v jq &> /dev/null; then
  echo -e "${RED}❌ jq is not installed. Please install it:${NC}"
  echo "   macOS: brew install jq"
  echo "   Linux: apt-get install jq / yum install jq"
  exit 1
fi

# Check if native apps directory exists
if [ ! -d "$NATIVE_APPS_DIR" ]; then
  echo -e "${YELLOW}⚠️  No native apps directory found${NC}"
  exit 0
fi

# Validate each app
for app_dir in "$NATIVE_APPS_DIR"/*; do
  if [ ! -d "$app_dir" ]; then
    continue
  fi

  app_name=$(basename "$app_dir")
  TOTAL_APPS=$((TOTAL_APPS + 1))

  echo ""

  # Validate manifest
  if validate_manifest "$app_dir/manifest.json" "$app_name"; then
    # Validate structure
    if validate_structure "$app_dir" "$app_name"; then
      # Validate TypeScript
      validate_typescript "$app_dir"

      # Validate dependencies
      if validate_dependencies "$app_dir"; then
        VALID_APPS=$((VALID_APPS + 1))
      else
        INVALID_APPS=$((INVALID_APPS + 1))
      fi
    else
      INVALID_APPS=$((INVALID_APPS + 1))
    fi
  else
    INVALID_APPS=$((INVALID_APPS + 1))
  fi
done

# Summary
echo ""
echo -e "${CYAN}======================================"
echo "Validation Summary"
echo -e "======================================${NC}"
echo "Total apps:     $TOTAL_APPS"
echo -e "${GREEN}✓ Valid:        $VALID_APPS${NC}"
echo -e "${RED}✗ Invalid:      $INVALID_APPS${NC}"
echo -e "${YELLOW}⚠ Warnings:     $WARNINGS${NC}"
echo ""

if [ $INVALID_APPS -eq 0 ]; then
  echo -e "${GREEN}🎉 All apps are valid!${NC}"
  exit 0
else
  echo -e "${RED}⚠️  Some apps have validation errors${NC}"
  exit 1
fi
