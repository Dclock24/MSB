# Reviewer Guide

## Open these first
1. `trading_engine.go` (Go) — execution & risk controls
2. `market_analysis.jl` (Julia) — signals & precision scoring
3. `reports/sim_report.txt` — end-to-end simulation output

## Runbook (local)
```bash
# Build
go build -o macro_strike_bot trading_engine.go

# Julia deps
scripts/setup_julia.sh

# Simulation + CSV/TXT reports
SIM_MODE=1 ORDER_RISK_PCT=1 ./macro_strike_bot
python3 scripts/parse_to_csv.py
# Check: data/sim_per_trade.csv, data/sim_summary_by_symbol.csv, reports/sim_report.txt
```

## Checklist
- Accuracy gate: EXECUTE + adjusted confidence ≥ 0.80 (engine)
- Guardrails: drawdown stop, time-window stop, target stop, consecutive-miss stop
- Live path: market buy → timed exit → realized PnL, retries/backoff, peak-cap tracking
- SIM model: leverage clamp (3–5x), TP/SL/fees, risk-based position sizing
- Data layer: caching, rate limiting, exponential backoff, numeric clamping
- Env-only secrets; `.env.example` complete; no secrets in repo
- CI: builds Go; Julia script returns valid JSON for a sample call

## Known limits
- Live leverage is spot-only. For 3–5x/35x, integrate a derivatives venue, on-exchange TP/SL, fee/precision sizing, and margin/liquidation buffers.
