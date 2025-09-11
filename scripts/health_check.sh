#!/bin/bash
# Health check for macro-strike-bot
# Verifies system readiness for live trading

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== MACRO STRIKE BOT HEALTH CHECK ===${NC}\n"

ERRORS=0
WARNINGS=0

# Function to check requirement
check() {
    local name=$1
    local command=$2
    local required=${3:-true}
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $name"
        return 0
    else
        if [ "$required" = true ]; then
            echo -e "${RED}✗${NC} $name - REQUIRED"
            ((ERRORS++))
        else
            echo -e "${YELLOW}⚠${NC} $name - RECOMMENDED"
            ((WARNINGS++))
        fi
        return 1
    fi
}

# System checks
echo -e "${YELLOW}System Requirements:${NC}"
check "Rust compiler" "rustc --version"
check "Cargo" "cargo --version"
check "Julia" "julia --version"
check "Python 3" "python3 --version"
check "Git" "git --version"

# Project checks
echo -e "\n${YELLOW}Project Setup:${NC}"
check "Cargo.toml exists" "test -f Cargo.toml"
check "main.rs exists" "test -f src/main.rs"
check "trading_engine.go exists" "test -f trading_engine.go"
check "market_analysis.jl exists" "test -f market_analysis.jl"
check "Makefile exists" "test -f Makefile"

# Build checks
echo -e "\n${YELLOW}Build Status:${NC}"
check "Debug build" "test -f target/debug/macro_strike_bot_fixed" false
check "Release build" "test -f target/release/macro_strike_bot_fixed" false

# Environment checks
echo -e "\n${YELLOW}Environment Variables:${NC}"
check "RUST_LOG set" "test -n \"${RUST_LOG:-}\"" false
check "SIM_MODE available" "true"

# API readiness (for live trading)
echo -e "\n${YELLOW}API Configuration (for live trading):${NC}"
check "COINGECKO_API_KEY" "test -n \"${COINGECKO_API_KEY:-}\"" false
check "KRAKEN_API_KEY" "test -n \"${KRAKEN_API_KEY:-}\"" false
check "KRAKEN_API_SECRET" "test -n \"${KRAKEN_API_SECRET:-}\"" false

# Directory structure
echo -e "\n${YELLOW}Directory Structure:${NC}"
check "scripts/ directory" "test -d scripts/"
check "data/ directory" "test -d data/"
check "reports/ directory" "test -d reports/"
check "docs/ directory" "test -d docs/"
check "test_runs/ directory" "test -d test_runs/" false

# Git status
echo -e "\n${YELLOW}Git Repository:${NC}"
if git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Git repository initialized"
    
    # Check for uncommitted changes
    if [ -n "$(git status --porcelain)" ]; then
        echo -e "${YELLOW}⚠${NC} Uncommitted changes detected"
        ((WARNINGS++))
    else
        echo -e "${GREEN}✓${NC} Working directory clean"
    fi
    
    # Check for remote
    if git remote -v | grep -q origin; then
        echo -e "${GREEN}✓${NC} Remote repository configured"
    else
        echo -e "${YELLOW}⚠${NC} No remote repository"
        ((WARNINGS++))
    fi
else
    echo -e "${RED}✗${NC} Not a git repository"
    ((ERRORS++))
fi

# Performance checks
echo -e "\n${YELLOW}Performance Readiness:${NC}"

# Check available memory (macOS)
if command -v vm_stat &> /dev/null; then
    FREE_BLOCKS=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    FREE_MB=$((FREE_BLOCKS * 4096 / 1024 / 1024))
    if [ $FREE_MB -gt 1000 ]; then
        echo -e "${GREEN}✓${NC} Sufficient memory available (${FREE_MB}MB free)"
    else
        echo -e "${YELLOW}⚠${NC} Low memory (${FREE_MB}MB free)"
        ((WARNINGS++))
    fi
fi

# Check CPU load
if command -v uptime &> /dev/null; then
    LOAD=$(uptime | awk -F'load averages:' '{print $2}' | awk '{print $1}')
    echo -e "${GREEN}✓${NC} CPU load: $LOAD"
fi

# Summary
echo -e "\n${YELLOW}=== HEALTH CHECK SUMMARY ===${NC}"

if [ $ERRORS -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed!${NC}"
        echo -e "${GREEN}System is ready for operation.${NC}"
        exit 0
    else
        echo -e "${YELLOW}⚠ $WARNINGS warnings found${NC}"
        echo -e "${GREEN}✓ No critical errors - system is operational${NC}"
        exit 0
    fi
else
    echo -e "${RED}✗ $ERRORS critical errors found${NC}"
    echo -e "${YELLOW}⚠ $WARNINGS warnings found${NC}"
    echo -e "${RED}System is NOT ready for operation${NC}"
    exit 1
fi
