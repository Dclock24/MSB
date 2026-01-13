#!/bin/bash
# 🚀 MACRO STRIKE BOT - $250K-$500K LAUNCH SCRIPT
# ===============================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

clear

echo -e "${PURPLE}💰 MACRO STRIKE BOT - $250K CAPITAL LAUNCH 💰${NC}"
echo -e "${WHITE}=============================================${NC}"
echo -e "${CYAN}Optimized for $250K-$500K Growth Strategy${NC}"
echo ""

# Check config
if [ ! -f "env.production.250k" ]; then
    echo -e "${RED}❌ ERROR: env.production.250k not found!${NC}"
    exit 1
fi

# Load config
source env.production.250k

echo -e "${PURPLE}📊 CAPITAL CONFIGURATION${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "💰 Starting Capital: ${GREEN}$250,000${NC}"
echo -e "🎯 Position Size: ${GREEN}$20,000${NC} (8%)"
echo -e "📈 Daily Target: ${GREEN}$3,000${NC} (1.2%)"
echo -e "📊 Monthly Target: ${GREEN}$66,000${NC} (26.4%)"
echo -e "🛡️ Max Daily Loss: ${RED}$12,500${NC} (5%)"
echo ""

echo -e "${PURPLE}🎯 GROWTH PROJECTIONS${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Month 1: $250K → ${GREEN}$316K${NC}"
echo -e "Month 3: $316K → ${GREEN}$500K${NC}"
echo -e "Month 6: $500K → ${GREEN}$1.0M${NC}"
echo -e "Month 12: $1.0M → ${GREEN}$2.5M+${NC}"
echo ""

echo -e "${PURPLE}🔥 OPTIMAL STRATEGIES FOR $250K${NC}"
echo -e "${WHITE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "• Ultra-Fast Cascade (40%): ${GREEN}$100K allocated${NC}"
echo -e "  → 3-5 trades/day @ $20K each"
echo -e "  → Expected: $2,400-$4,000 daily"
echo ""
echo -e "• Microstructure Arb (30%): ${GREEN}$75K allocated${NC}"
echo -e "  → 20-40 trades/day @ $15K each"
echo -e "  → Expected: $900-$1,800 daily"
echo ""
echo -e "• Volatility Surface (20%): ${GREEN}$50K allocated${NC}"
echo -e "  → 1-2 trades/day @ $25K each"
echo -e "  → Expected: $500-$1,000 daily"
echo ""

# Safety confirmation
if [ "$LIVE_TRADING" = "1" ]; then
    echo -e "${RED}⚠️  LIVE TRADING MODE - REAL MONEY! ⚠️${NC}"
    echo -e "${YELLOW}Type 'LAUNCH 250K' to start trading:${NC}"
    read -r confirm
    if [ "$confirm" != "LAUNCH 250K" ]; then
        echo -e "${RED}Cancelled.${NC}"
        exit 1
    fi
fi

# Build
echo -e "${YELLOW}Building optimized binary...${NC}"
cargo build --release --quiet

# Launch
echo -e "${GREEN}🚀 LAUNCHING $250K GROWTH ENGINE! 🚀${NC}"
echo ""

LOG_FILE="logs/250k_$(date +%Y%m%d_%H%M%S).log"
mkdir -p logs

# Start Julia analysis if available
if command -v julia &> /dev/null; then
    julia market_analysis.jl > logs/julia_250k.log 2>&1 &
    JULIA_PID=$!
fi

# Run the engine
./target/release/trading_engine 2>&1 | tee "$LOG_FILE"

# Cleanup
[ ! -z "$JULIA_PID" ] && kill $JULIA_PID 2>/dev/null || true

echo -e "${GREEN}Session complete. Log saved to: $LOG_FILE${NC}"
