#!/usr/bin/env bash
set -euo pipefail

# Security Audit Script
# Runs comprehensive security checks across all services

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=================================="
echo "AgentOS Security Audit"
echo "=================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
ISSUES_FOUND=0

# ============================================================================
# Rust Kernel Security Audit
# ============================================================================

echo "🔍 Auditing Rust Kernel..."
cd "$PROJECT_ROOT/kernel"

if command -v cargo-audit &> /dev/null; then
    echo "Running cargo audit..."
    if cargo audit; then
        echo -e "${GREEN}✓ No vulnerabilities found in Rust dependencies${NC}"
    else
        echo -e "${RED}✗ Vulnerabilities found in Rust dependencies${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ cargo-audit not installed. Install with: cargo install cargo-audit${NC}"
fi

if command -v cargo-deny &> /dev/null; then
    echo "Running cargo deny..."
    if cargo deny check; then
        echo -e "${GREEN}✓ Passed cargo deny checks${NC}"
    else
        echo -e "${RED}✗ Failed cargo deny checks${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ cargo-deny not installed. Install with: cargo install cargo-deny${NC}"
fi

echo ""

# ============================================================================
# Go Backend Security Audit
# ============================================================================

echo "🔍 Auditing Go Backend..."
cd "$PROJECT_ROOT/backend"

if command -v gosec &> /dev/null; then
    echo "Running gosec..."
    if gosec -quiet ./...; then
        echo -e "${GREEN}✓ No security issues found with gosec${NC}"
    else
        echo -e "${RED}✗ Security issues found with gosec${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ gosec not installed. Install with: go install github.com/securego/gosec/v2/cmd/gosec@latest${NC}"
fi

if command -v govulncheck &> /dev/null; then
    echo "Running govulncheck..."
    if govulncheck ./...; then
        echo -e "${GREEN}✓ No vulnerabilities found in Go dependencies${NC}"
    else
        echo -e "${RED}✗ Vulnerabilities found in Go dependencies${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ govulncheck not installed. Install with: go install golang.org/x/vuln/cmd/govulncheck@latest${NC}"
fi

# Check for hardcoded secrets
echo "Checking for hardcoded secrets..."
if command -v gitleaks &> /dev/null; then
    if gitleaks detect --source . --no-git --quiet; then
        echo -e "${GREEN}✓ No hardcoded secrets found${NC}"
    else
        echo -e "${RED}✗ Potential secrets detected${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ gitleaks not installed. Install from: https://github.com/gitleaks/gitleaks${NC}"
fi

echo ""

# ============================================================================
# Python AI Service Security Audit
# ============================================================================

echo "🔍 Auditing Python AI Service..."
cd "$PROJECT_ROOT/ai-service"

# Activate venv if it exists
if [ -d "venv" ]; then
    source venv/bin/activate
fi

if command -v bandit &> /dev/null; then
    echo "Running bandit..."
    if bandit -r src/ -ll -q; then
        echo -e "${GREEN}✓ No security issues found with bandit${NC}"
    else
        echo -e "${RED}✗ Security issues found with bandit${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ bandit not installed. Install with: pip install bandit${NC}"
fi

if command -v safety &> /dev/null; then
    echo "Running safety..."
    if safety check --json; then
        echo -e "${GREEN}✓ No vulnerabilities found in Python dependencies${NC}"
    else
        echo -e "${RED}✗ Vulnerabilities found in Python dependencies${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
else
    echo -e "${YELLOW}⚠ safety not installed. Install with: pip install safety${NC}"
fi

echo ""

# ============================================================================
# TypeScript UI Security Audit
# ============================================================================

echo "🔍 Auditing TypeScript UI..."
cd "$PROJECT_ROOT/ui"

echo "Running npm audit..."
if npm audit --audit-level=high; then
    echo -e "${GREEN}✓ No high-severity vulnerabilities found${NC}"
else
    echo -e "${RED}✗ High-severity vulnerabilities found in npm dependencies${NC}"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi

echo ""

# ============================================================================
# Summary
# ============================================================================

echo "=================================="
echo "Security Audit Summary"
echo "=================================="

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✓ No security issues found!${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $ISSUES_FOUND security issue(s)${NC}"
    echo ""
    echo "Please review the output above and address the findings."
    exit 1
fi
