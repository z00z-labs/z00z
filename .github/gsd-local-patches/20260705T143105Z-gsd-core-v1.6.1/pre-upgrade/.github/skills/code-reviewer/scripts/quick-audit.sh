#!/bin/bash

# Quick Security Audit Script
# Runs basic SAST tools available in most Node.js projects
# Usage: bash scripts/quick-audit.sh

set -e

echo "🔍 Running Quick Security Audit..."
echo "================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
CRITICAL_ISSUES=0
HIGH_ISSUES=0

# 1. npm audit
echo "📦 Running npm audit..."
if command -v npm &> /dev/null; then
    if npm audit --audit-level=high; then
        echo -e "${GREEN}✓ No high/critical vulnerabilities found${NC}"
    else
        echo -e "${RED}✗ npm audit found vulnerabilities${NC}"
        CRITICAL_ISSUES=$((CRITICAL_ISSUES + 1))
    fi
else
    echo -e "${YELLOW}⚠ npm not found, skipping${NC}"
fi
echo ""

# 2. ESLint
echo "🔧 Running ESLint..."
if command -v npx &> /dev/null && [ -f ".eslintrc.js" ] || [ -f ".eslintrc.json" ] || [ -f "eslint.config.js" ]; then
    if npx eslint . --ext .js,.jsx,.ts,.tsx --max-warnings 10; then
        echo -e "${GREEN}✓ ESLint passed${NC}"
    else
        echo -e "${RED}✗ ESLint found issues${NC}"
        HIGH_ISSUES=$((HIGH_ISSUES + 1))
    fi
else
    echo -e "${YELLOW}⚠ ESLint not configured, skipping${NC}"
fi
echo ""

# 3. Prettier check
echo "💅 Running Prettier..."
if command -v npx &> /dev/null && [ -f ".prettierrc" ] || [ -f ".prettierrc.json" ] || [ -f "prettier.config.js" ]; then
    if npx prettier --check . 2>/dev/null; then
        echo -e "${GREEN}✓ Code formatting is consistent${NC}"
    else
        echo -e "${YELLOW}⚠ Code formatting inconsistencies found (auto-fixable)${NC}"
        echo "  Run: npx prettier --write ."
    fi
else
    echo -e "${YELLOW}⚠ Prettier not configured, skipping${NC}"
fi
echo ""

# 4. TypeScript type check
echo "📘 Running TypeScript type check..."
if command -v npx &> /dev/null && [ -f "tsconfig.json" ]; then
    if npx tsc --noEmit; then
        echo -e "${GREEN}✓ TypeScript type check passed${NC}"
    else
        echo -e "${RED}✗ TypeScript type errors found${NC}"
        HIGH_ISSUES=$((HIGH_ISSUES + 1))
    fi
else
    echo -e "${YELLOW}⚠ TypeScript not configured, skipping${NC}"
fi
echo ""

# 5. Check for hardcoded secrets (basic grep)
echo "🔑 Checking for hardcoded secrets..."
if grep -r -i -E "(api_key|apikey|api-key|password|secret|token|auth|credentials).*=.*['\"][a-zA-Z0-9]{20,}['\"]" src/ --exclude-dir=node_modules --exclude-dir=.git 2>/dev/null; then
    echo -e "${RED}✗ Potential hardcoded secrets found!${NC}"
    echo "  Review the files above and move secrets to environment variables"
    CRITICAL_ISSUES=$((CRITICAL_ISSUES + 1))
else
    echo -e "${GREEN}✓ No obvious hardcoded secrets detected${NC}"
fi
echo ""

# Final summary
echo "================================="
echo "📊 Audit Summary"
echo "================================="
echo ""

if [ $CRITICAL_ISSUES -gt 0 ]; then
    echo -e "${RED}❌ CRITICAL ISSUES: $CRITICAL_ISSUES${NC}"
    echo "   Must be fixed before deployment"
fi

if [ $HIGH_ISSUES -gt 0 ]; then
    echo -e "${YELLOW}⚠️  HIGH PRIORITY ISSUES: $HIGH_ISSUES${NC}"
    echo "   Should be fixed within 48 hours"
fi

if [ $CRITICAL_ISSUES -eq 0 ] && [ $HIGH_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✅ ALL CHECKS PASSED${NC}"
    echo "   Code is ready for review"
fi

echo ""
echo "For detailed analysis, consider running:"
echo "  - sonar-scanner (if SonarQube configured)"
echo "  - codeql database analyze (if CodeQL installed)"
echo "  - snyk test (if Snyk configured)"
echo ""

# Exit with error code if critical issues found
if [ $CRITICAL_ISSUES -gt 0 ]; then
    exit 1
fi

exit 0
