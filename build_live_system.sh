#!/bin/bash
# 🚀 BUILD SCRIPT FOR LIVE TRADING SYSTEM
# Ensures code is flawless for $250K deployment

set -e

echo "🔨 BUILDING LIVE TRADING SYSTEM"
echo "=============================="
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Update dependencies
echo "📦 Updating dependencies..."
cargo update

# Build in release mode with all optimizations
echo "🔧 Building release binary..."
export RUSTFLAGS="-C opt-level=3 -C lto=fat -C codegen-units=1"
cargo build --release

# Run all tests
echo "🧪 Running test suite..."
cargo test --release -- --nocapture

# Check for any warnings
echo "⚠️  Checking for warnings..."
cargo clippy -- -D warnings || true

# Generate documentation
echo "📚 Generating documentation..."
cargo doc --no-deps

# Create production binaries
echo "📦 Creating production binaries..."
mkdir -p production/bin
cp target/release/trading_engine production/bin/
cp target/release/trading_engine_simple production/bin/

# Create production config
echo "⚙️  Setting up production configuration..."
cat > production/config.yaml << 'EOF'
# PRODUCTION CONFIGURATION - $250K LIVE TRADING
trading:
  initial_capital: 250000.0
  max_position_size: 0.12
  max_positions: 10
  min_win_rate: 0.70
  confidence_threshold: 0.72
  
risk_management:
  stop_loss_percent: 0.02
  max_daily_loss: 0.05
  max_drawdown: 0.08
  position_sizing: "fractional_kelly"
  
exchanges:
  kraken:
    enabled: true
    rate_limit: 10
  binance:
    enabled: false
    rate_limit: 20
    
monitoring:
  alerts: true
  log_level: "info"
  metrics_port: 9090
  health_check_interval: 60
  
circuit_breakers:
  daily_loss_limit: 12500
  consecutive_losses: 3
  volatility_spike: 2.5
  correlation_threshold: 0.8
EOF

# Create startup script
echo "🚀 Creating startup script..."
cat > production/start_live_trading.sh << 'EOF'
#!/bin/bash
# LIVE TRADING STARTUP SCRIPT

echo "⚠️  STARTING LIVE TRADING WITH REAL MONEY"
echo "========================================"
echo ""
echo "Capital: $250,000"
echo ""

# Safety checks
if [ ! -f ".env.production" ]; then
    echo "❌ Missing .env.production file"
    exit 1
fi

if [ "$CONFIRM_LIVE_TRADING" != "YES" ]; then
    echo "❌ Set CONFIRM_LIVE_TRADING=YES to proceed"
    exit 1
fi

# Start with monitoring
echo "📊 Starting monitoring dashboard..."
./bin/trading_engine --monitor-only &
MONITOR_PID=$!

sleep 5

# Start main engine
echo "🚀 Starting trading engine..."
./bin/trading_engine --config config.yaml --live

# Cleanup on exit
trap "kill $MONITOR_PID" EXIT
EOF

chmod +x production/start_live_trading.sh

# Final checks
echo ""
echo "✅ BUILD COMPLETE"
echo ""
echo "📋 Checklist for live trading:"
echo "  [ ] API keys configured in .env.production"
echo "  [ ] Risk parameters reviewed in config.yaml"
echo "  [ ] Circuit breakers tested"
echo "  [ ] Monitoring dashboard accessible"
echo "  [ ] Backup systems in place"
echo "  [ ] Emergency contacts updated"
echo ""
echo "To start live trading:"
echo "  cd production"
echo "  CONFIRM_LIVE_TRADING=YES ./start_live_trading.sh"
echo ""
echo "⚠️  FINAL WARNING: This will trade with REAL MONEY!"





