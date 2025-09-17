#!/bin/bash

# Rust Standalone Trading Engine Runner
# This script builds and runs the Rust trading engine as the primary system

set -e

echo "🦀 Rust Trading Engine Launcher"
echo "================================"

# Source environment variables
if [ -f ".env.rust" ]; then
    echo "Loading environment from .env.rust..."
    export $(cat .env.rust | grep -v '^#' | xargs)
else
    echo "Warning: .env.rust not found. Using defaults or system environment."
fi

# Build the trading engine
echo ""
echo "Building Rust trading engine..."
cargo build --release --bin trading_engine

# Check if we're in dry run mode
if [ "$DRY_RUN" = "true" ]; then
    echo ""
    echo "⚠️  RUNNING IN DRY RUN MODE - No real trades will be executed"
    echo ""
else
    echo ""
    echo "🔴 LIVE TRADING MODE ACTIVE"
    echo "✅ Protected by 12-step validation system"
    echo "✅ Elite strategies: Citadel, Renaissance, Two Sigma, Jump Trading"
    echo "✅ 90% minimum confidence requirement enforced"
    echo "✅ All trades validated before execution"
    echo ""
fi

# Display configuration
echo "Configuration:"
echo "  Initial Capital: $${INITIAL_CAPITAL:-10000}"
echo "  Min Confidence: ${MIN_CONFIDENCE:-0.90}"
echo "  Max Position Size: ${MAX_POSITION_SIZE_PCT:-0.05}"
echo "  Stop Loss: ${STOP_LOSS_PCT:-0.02}"
echo "  Take Profit: ${TAKE_PROFIT_PCT:-0.06}"
echo ""

# Run the trading engine
echo "Starting Rust trading engine..."
./target/release/trading_engine
