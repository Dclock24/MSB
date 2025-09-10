#!/bin/bash
# Build script for Macro Strike Bot

echo "🚀 Building Macro Strike Bot..."

# Check if Julia is installed
if ! command -v julia &> /dev/null; then
    echo "❌ Julia is not installed. Please install Julia first."
    echo "   Visit: https://julialang.org/downloads/"
    exit 1
fi

# Check if Go is installed
if ! command -v go &> /dev/null; then
    echo "❌ Go is not installed. Please install Go first."
    echo "   Visit: https://golang.org/dl/"
    exit 1
fi

# Install Julia dependencies
echo "📦 Installing Julia dependencies..."
julia -e "using Pkg; Pkg.add([\"HTTP\", \"JSON\", \"Statistics\", \"Random\", \"Dates\"])"

# Test Julia script
echo "🧪 Testing Julia market analysis..."
julia market_analysis.jl "WETH/USDC" "MacroMomentum"

if [ $? -eq 0 ]; then
    echo "✅ Julia market analysis working"
else
    echo "❌ Julia market analysis failed"
    exit 1
fi

# Build Go trading engine
echo "🔨 Building Go trading engine..."
go build -o macro_strike_bot trading_engine.go

if [ $? -eq 0 ]; then
    echo "✅ Go trading engine built successfully"
    echo "🎯 Ready to run: ./macro_strike_bot"
else
    echo "❌ Go build failed"
    exit 1
fi

echo "🏁 Build complete! Run with: ./macro_strike_bot"
