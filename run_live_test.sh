#!/bin/bash

# Run Live Test for Hummingbot Array System
# Tests all new features: 7-day cycle, volume-based striking, 1-minute exits

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     HUMMINGBOT ARRAY LIVE TEST - PROFIT MODELING           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Building test binary..."
cargo build --bin live_test_hummingbot --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "Running live test..."
    echo ""
    cargo run --bin live_test_hummingbot --release
else
    echo ""
    echo "❌ Build failed. Please check for errors above."
    exit 1
fi

