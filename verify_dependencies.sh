#!/bin/bash
# 🔍 DEPENDENCY VERIFICATION SCRIPT
# =================================

echo "🔍 MACRO STRIKE BOT - DEPENDENCY VERIFICATION"
echo "============================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check Rust version
echo "📦 Checking Rust version..."
RUST_VERSION=$(rustc --version | awk '{print $2}')
REQUIRED_VERSION="1.70.0"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then
    echo -e "${GREEN}✅ Rust $RUST_VERSION (meets minimum $REQUIRED_VERSION)${NC}"
else
    echo -e "${RED}❌ Rust $RUST_VERSION (requires minimum $REQUIRED_VERSION)${NC}"
    echo "   Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

echo ""
echo "📦 Checking critical dependencies..."

# Check if Cargo.lock exists
if [ -f "Cargo.lock" ]; then
    echo -e "${GREEN}✅ Cargo.lock present${NC}"
else
    echo -e "${YELLOW}⚠️  No Cargo.lock found - running cargo build${NC}"
    cargo build --quiet
fi

# Check for security vulnerabilities
echo ""
echo "🔒 Security check..."
if command -v cargo-audit &> /dev/null; then
    cargo audit --quiet
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ No known vulnerabilities${NC}"
    else
        echo -e "${RED}❌ Vulnerabilities found - run 'cargo audit' for details${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit not installed${NC}"
    echo "   Install with: cargo install cargo-audit"
fi

# Check mathematical libraries
echo ""
echo "🧮 Mathematical libraries..."
if grep -q 'nalgebra = "0.32' Cargo.toml; then
    echo -e "${GREEN}✅ nalgebra (linear algebra) - OK${NC}"
else
    echo -e "${RED}❌ nalgebra missing or wrong version${NC}"
fi

if grep -q 'num-complex = "0.4' Cargo.toml; then
    echo -e "${GREEN}✅ num-complex (complex numbers) - OK${NC}"
else
    echo -e "${RED}❌ num-complex missing or wrong version${NC}"
fi

if grep -q 'special = "0.1' Cargo.toml; then
    echo -e "${GREEN}✅ special (math functions) - OK${NC}"
else
    echo -e "${RED}❌ special missing or wrong version${NC}"
fi

# Check async runtime
echo ""
echo "⚡ Async runtime..."
if grep -q 'tokio.*full' Cargo.toml; then
    echo -e "${GREEN}✅ tokio (full features) - OK${NC}"
else
    echo -e "${RED}❌ tokio not configured with full features${NC}"
fi

# Check API dependencies
echo ""
echo "🌐 API dependencies..."
if grep -q 'reqwest.*json' Cargo.toml; then
    echo -e "${GREEN}✅ reqwest (HTTP client) - OK${NC}"
else
    echo -e "${RED}❌ reqwest missing JSON feature${NC}"
fi

# System dependencies
echo ""
echo "💻 System dependencies..."

# Check OpenSSL
if command -v openssl &> /dev/null; then
    OPENSSL_VERSION=$(openssl version | awk '{print $2}')
    echo -e "${GREEN}✅ OpenSSL $OPENSSL_VERSION${NC}"
else
    echo -e "${RED}❌ OpenSSL not found (required for TLS)${NC}"
fi

# Check available memory
if command -v free &> /dev/null; then
    MEM_GB=$(free -g | awk 'NR==2{print $2}')
    if [ $MEM_GB -ge 4 ]; then
        echo -e "${GREEN}✅ Memory: ${MEM_GB}GB (minimum 4GB)${NC}"
    else
        echo -e "${YELLOW}⚠️  Memory: ${MEM_GB}GB (recommend 8GB+)${NC}"
    fi
fi

# Final build test
echo ""
echo "🔨 Testing build..."
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✅ Project builds successfully${NC}"
else
    echo -e "${RED}❌ Build errors detected${NC}"
    echo "   Run: cargo build --release"
fi

echo ""
echo "📊 DEPENDENCY SUMMARY"
echo "====================="

# Count status
TOTAL_CHECKS=10
PASSED=$(echo -e "$0" | grep -c "✅")

if [ $PASSED -eq $TOTAL_CHECKS ]; then
    echo -e "${GREEN}✅ ALL CHECKS PASSED - Ready for production!${NC}"
else
    echo -e "${YELLOW}⚠️  Some checks need attention${NC}"
    echo "   Review the warnings above and fix any issues."
fi

echo ""
echo "💡 Quick fixes:"
echo "  • Update Rust: rustup update"
echo "  • Update deps: cargo update"
echo "  • Add missing: cargo add <package>"
echo "  • Security scan: cargo install cargo-audit && cargo audit"
