# 🚀 MACRO STRIKE BOT ACTIVATION GUIDE
## $250K-$500K Capital Configuration

## ✅ PRE-ACTIVATION CHECKLIST

### 1. API Keys Setup
- [ ] Kraken API Key & Secret
- [ ] CoinGecko API Key (optional but recommended)
- [ ] Exchange accounts funded with trading capital

### 2. System Requirements Verified
- [ ] Rust installed and building successfully
- [ ] Julia installed (optional for advanced analytics)
- [ ] Minimum 8GB RAM available
- [ ] Stable internet connection

### 3. Capital Ready
- [ ] $250K available in exchange accounts
- [ ] Funds distributed across exchanges (if using multiple)
- [ ] Stablecoin pairs ready (USDT/USDC)

## 🔧 STEP 1: CONFIGURE YOUR API KEYS

Edit the production configuration file:

```bash
nano env.production.250k
```

Replace these values with your actual credentials:
```
KRAKEN_API_KEY=your_actual_kraken_api_key
KRAKEN_API_SECRET=your_actual_kraken_api_secret
COINGECKO_API_KEY=your_actual_coingecko_key
```

## 🧪 STEP 2: TEST IN DRY RUN MODE (REQUIRED)

### A. Ensure dry run is enabled:
```bash
# In env.production.250k:
LIVE_TRADING=0
DRY_RUN=1
```

### B. Run 24-hour test:
```bash
./launch_250k.sh
```

### C. Verify Performance Metrics:
- Win rate > 90%
- Daily return > 1%
- No system errors
- All strategies executing

## 💰 STEP 3: ACTIVATE LIVE TRADING

### A. Update configuration for live mode:
```bash
# Edit env.production.250k:
LIVE_TRADING=1
DRY_RUN=0
```

### B. Start with conservative settings:
```bash
# Recommended Week 1 settings:
POSITION_SIZE_USD=15000.0          # Start with $15K positions
MAX_POSITIONS=4                    # Only 4 concurrent trades
MIN_CONFIDENCE=0.92                # Higher confidence requirement
```

### C. Launch live trading:
```bash
./launch_250k.sh
```

## 📊 STEP 4: MONITORING YOUR BOT

### Real-Time Monitoring Commands:

1. **Watch live trades**:
```bash
tail -f logs/250k_*.log | grep "Trade"
```

2. **Monitor P&L**:
```bash
tail -f logs/250k_*.log | grep "profit\|loss"
```

3. **Check win rate**:
```bash
grep "Trade Result" logs/250k_*.log | grep -c "profit"
```

### Daily Performance Check:
```bash
# Create this script as daily_report.sh
#!/bin/bash
LOG=$(ls -t logs/250k_*.log | head -1)
echo "=== DAILY PERFORMANCE REPORT ==="
echo "Total Trades: $(grep -c "Trade executed" $LOG)"
echo "Winning Trades: $(grep "Trade Result" $LOG | grep -c "profit")"
echo "Total P&L: $(grep "Daily P&L" $LOG | tail -1)"
```

## 🎯 WEEK 1 STRATEGY (CONSERVATIVE START)

### Focus: Ultra-Fast Cascade Detection Only
- Trade only the strongest signals
- 2-4 trades per day maximum
- Target: $2,000-$3,000 daily profit
- Prove 90%+ win rate

### Settings:
```bash
ENABLE_CASCADE_DETECTION=true
ENABLE_MICROSTRUCTURE=false
ENABLE_VOLATILITY_SURFACE=false
ENABLE_CROSS_EXCHANGE=false
```

## 📈 SCALING SCHEDULE

### Week 2: Add Microstructure
```bash
ENABLE_MICROSTRUCTURE=true
POSITION_SIZE_USD=17500.0
MAX_POSITIONS=6
```

### Week 3: Full Deployment
```bash
ENABLE_VOLATILITY_SURFACE=true
ENABLE_CROSS_EXCHANGE=true
POSITION_SIZE_USD=20000.0
MAX_POSITIONS=8
```

### Month 2: Scale Up
```bash
# After $50K+ profits
POSITION_SIZE_USD=25000.0
MIN_CONFIDENCE=0.91
```

## 🛡️ SAFETY PROTOCOLS

### Emergency Stop Commands:

1. **Pause trading immediately**:
```bash
kill -TERM $(pgrep trading_engine)
```

2. **Close all positions**:
```bash
# Add to your launch script:
trap 'echo "Closing all positions..."; close_all_positions' EXIT
```

### Daily Safety Checks:
- [ ] Max drawdown < 8%
- [ ] Win rate > 87%
- [ ] No correlation issues
- [ ] All systems responding

## 📞 SUPPORT & TROUBLESHOOTING

### Common Issues:

1. **"Insufficient liquidity"**
   - Reduce position size by 20%
   - Check exchange order books

2. **"Confidence below threshold"**
   - Market conditions unfavorable
   - Wait for better setups

3. **"Circuit breaker triggered"**
   - System protecting capital
   - Review recent trades
   - Restart after analysis

### Performance Optimization:
- Run during high-liquidity sessions
- Focus on BTC/ETH during Week 1
- Add altcoins after proven success

## 🎉 SUCCESS MILESTONES

### Week 1: Foundation
- [ ] 90%+ win rate achieved
- [ ] $10K-$15K profit
- [ ] System stability proven

### Month 1: Validation
- [ ] $50K-$75K profit
- [ ] All strategies profitable
- [ ] Ready to scale

### Month 3: Scale
- [ ] $200K+ total profit
- [ ] Capital grown to $450K+
- [ ] Increase to $500K base

### Month 6: Acceleration
- [ ] $500K+ total profit
- [ ] Capital exceeded $1M
- [ ] Full system deployment

## 🚀 ACTIVATION COMMAND

When ready, activate your bot with:

```bash
# Final activation
source env.production.250k
./launch_250k.sh
```

Welcome to the future of algorithmic trading! 🎊
