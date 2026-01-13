#!/bin/bash
# 🚀 QUICK START ACTIVATION SCRIPT
# ================================

echo "🚀 MACRO STRIKE BOT - QUICK ACTIVATION"
echo "====================================="
echo ""

# Check if user has set API keys
if grep -q "your_real_api_key_here" .env; then
    echo "⚠️  IMPORTANT: You need to add your API keys first!"
    echo ""
    echo "1. Edit the .env file:"
    echo "   nano .env"
    echo ""
    echo "2. Replace these lines with your actual keys:"
    echo "   KRAKEN_API_KEY=your_actual_key_here"
    echo "   KRAKEN_API_SECRET=your_actual_secret_here"
    echo ""
    echo "3. Then run this script again!"
    exit 1
fi

# Verify build
echo "🔨 Verifying system build..."
if cargo build --release --quiet 2>/dev/null; then
    echo "✅ System built successfully"
else
    echo "❌ Build failed. Running full build..."
    cargo build --release
fi

# Create necessary directories
mkdir -p logs data

echo ""
echo "📊 ACTIVATION OPTIONS:"
echo "======================"
echo ""
echo "1) START IN TEST MODE (Recommended first)"
echo "   - Paper trading with real market data"
echo "   - Verify 90% win rate"
echo "   - No real money at risk"
echo ""
echo "2) START LIVE TRADING"
echo "   - Trade with real $250K"
echo "   - Conservative settings"
echo "   - Full monitoring enabled"
echo ""
read -p "Select option (1 or 2): " choice

case $choice in
    1)
        echo ""
        echo "🧪 STARTING TEST MODE..."
        echo "========================"
        export LIVE_TRADING=0
        export DRY_RUN=1
        ./launch_250k.sh
        ;;
    2)
        echo ""
        echo "⚠️  LIVE TRADING CONFIRMATION"
        echo "=============================="
        echo "This will trade with REAL MONEY!"
        echo "Starting capital: $250,000"
        echo ""
        read -p "Type 'START LIVE' to confirm: " confirm
        if [ "$confirm" = "START LIVE" ]; then
            export LIVE_TRADING=1
            export DRY_RUN=0
            ./launch_250k.sh
        else
            echo "Cancelled."
        fi
        ;;
    *)
        echo "Invalid option. Please run again and select 1 or 2."
        ;;
esac
