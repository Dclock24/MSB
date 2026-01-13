#!/bin/bash
# 🏎️ MACRO STRIKE BOT - FERRARI MODE 🏎️
# =====================================
# Elite Trading Engine with Revolutionary Strategies

set -e

# Colors for beautiful output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

clear

echo -e "${RED}🏎️  MACRO STRIKE BOT - FERRARI MODE 🏎️${NC}"
echo -e "${WHITE}========================================${NC}"
echo ""
echo -e "${CYAN}Initializing Elite Trading Systems...${NC}"
echo ""

# Check if .env.rust exists
if [ ! -f ".env.rust" ]; then
    echo -e "${RED}❌ ERROR: .env.rust not found!${NC}"
    echo -e "${YELLOW}Please create .env.rust with your API keys:${NC}"
    echo ""
    echo "KRAKEN_API_KEY=your_key_here"
    echo "KRAKEN_API_SECRET=your_secret_here"
    echo "COINGECKO_API_KEY=your_key_here"
    echo ""
    exit 1
fi

# Source environment variables
echo -e "${GREEN}✓ Loading Ferrari configuration...${NC}"
export $(cat .env.rust | grep -v '^#' | xargs)

# Display configuration
echo ""
echo -e "${PURPLE}🔧 CONFIGURATION${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "💰 Initial Capital:    ${GREEN}\$${INITIAL_CAPITAL:-100000}${NC}"
echo -e "🎯 Min Confidence:     ${GREEN}${MIN_CONFIDENCE:-0.90} (90%)${NC}"
echo -e "📊 Position Size:      ${GREEN}${MAX_POSITION_SIZE_PCT:-0.05} (5%)${NC}"
echo -e "🛑 Stop Loss:          ${RED}${STOP_LOSS_PCT:-0.02} (2%)${NC}"
echo -e "💸 Take Profit:        ${GREEN}${TAKE_PROFIT_PCT:-0.06} (6%)${NC}"
echo -e "🔄 Max Positions:      ${YELLOW}${MAX_POSITIONS:-5}${NC}"
echo ""

# Strategy weights
echo -e "${PURPLE}⚡ STRATEGY ALLOCATION${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "🏛️  Elite Strategies:        ${BLUE}30%${NC} (Citadel, Renaissance, Two Sigma)"
echo -e "⚛️  Quantum Strategies:      ${PURPLE}20%${NC} (Quantum-inspired algorithms)"
echo -e "🚀 Revolutionary Strategies: ${RED}50%${NC} (30-second cascade, cross-chain)"
echo ""

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust not installed! Installing...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Build the Ferrari engine
echo -e "${YELLOW}🔨 Building Ferrari Engine...${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Clean build for maximum performance
cargo clean
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin trading_engine

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Ferrari engine built successfully!${NC}"
else
    echo -e "${RED}❌ Build failed! Check error messages above.${NC}"
    exit 1
fi

# Pre-flight checks
echo ""
echo -e "${CYAN}🔍 PRE-FLIGHT CHECKS${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Check API keys
if [ "$KRAKEN_API_KEY" = "your_kraken_api_key_here" ]; then
    echo -e "${RED}❌ Kraken API key not configured${NC}"
    READY=false
else
    echo -e "${GREEN}✓ Kraken API configured${NC}"
    READY=true
fi

if [ "$COINGECKO_API_KEY" = "your_coingecko_api_key_here" ]; then
    echo -e "${YELLOW}⚠ CoinGecko API key not configured (optional)${NC}"
else
    echo -e "${GREEN}✓ CoinGecko API configured${NC}"
fi

# Trading mode check
if [ "$DRY_RUN" = "true" ]; then
    echo -e "${YELLOW}📋 DRY RUN MODE - No real trades${NC}"
    MODE="DRY RUN"
    MODE_COLOR=$YELLOW
else
    echo -e "${RED}💰 LIVE TRADING MODE - Real money at risk!${NC}"
    MODE="LIVE TRADING"
    MODE_COLOR=$RED
    
    # Extra confirmation for live mode
    echo ""
    echo -e "${RED}⚠️  WARNING: Live trading will use real money!${NC}"
    echo -e "${WHITE}Press ENTER to confirm or Ctrl+C to cancel...${NC}"
    read -r
fi

echo ""
echo -e "${PURPLE}🚀 FERRARI ENGINE FEATURES${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "✓ ${GREEN}Superior Strike Validator${NC} - Modular validation system"
echo -e "✓ ${BLUE}Elite Strategies${NC} - Citadel, Renaissance, Two Sigma, Jump, DE Shaw"
echo -e "✓ ${PURPLE}Quantum Algorithms${NC} - Quantum tunneling, fractional Brownian motion"
echo -e "✓ ${RED}Revolutionary Tech${NC} - 30-second cascade prediction"
echo -e "✓ ${CYAN}Ultra-Fast Execution${NC} - <200ms total latency"
echo -e "✓ ${YELLOW}Cross-Chain Arbitrage${NC} - Atomic execution across 5+ chains"
echo -e "✓ ${GREEN}90% Win Rate Enforcement${NC} - Multiple validation layers"
echo ""

# Final countdown
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${MODE_COLOR}Starting Ferrari Engine in $MODE mode...${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Add slight delay for dramatic effect
for i in 3 2 1; do
    echo -e "${RED}$i...${NC}"
    sleep 1
done

echo ""
echo -e "${GREEN}🏁 ENGINES START! 🏁${NC}"
echo ""

# Launch the Ferrari
RUST_LOG=${RUST_LOG:-info} ./target/release/trading_engine

# Capture exit code
EXIT_CODE=$?

# Post-run summary
echo ""
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ Ferrari engine stopped gracefully${NC}"
else
    echo -e "${RED}❌ Ferrari engine stopped with error code: $EXIT_CODE${NC}"
fi

echo -e "${CYAN}Check logs for detailed performance metrics${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

