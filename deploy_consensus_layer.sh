#!/bin/bash

# Consensus Layer Deployment Script
# Production-ready deployment for blockchain integration

set -e

echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║              CONSENSUS LAYER DEPLOYMENT - PRODUCTION READY                 ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
echo "🔍 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install Rust first."
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo "⚠️  Node.js not found (optional for some tools)"
fi

echo "✅ Prerequisites check complete"
echo ""

# Build system
echo "🔨 Building production release..."
cargo build --release --features "eip" 2>&1 | grep -E "(Compiling|Finished|error)" || cargo build --release

if [ ! -f target/release/run_1500_trades ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Run 1500 trade test
echo "🧪 Running 1500 trade validation test..."
echo ""

RUST_LOG=info target/release/run_1500_trades

TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo ""
    echo "✅ 1500 trade test PASSED"
else
    echo ""
    echo "❌ 1500 trade test FAILED (exit code: $TEST_EXIT_CODE)"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "                    DEPLOYMENT VALIDATION COMPLETE"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "📋 Next Steps:"
echo "  1. Review test_results_*.json"
echo "  2. Configure consensus layer RPC endpoints"
echo "  3. Set up API keys and secure storage"
echo "  4. Deploy to production environment"
echo ""
echo "🚀 System is ready for consensus layer deployment!"
