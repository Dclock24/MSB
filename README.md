# Macro Strike Bot

Production-ready HFT bot with:
- Go live trading engine (Kraken spot, market buy + timed exit, realized PnL)
- Julia market analysis (CoinGecko + Kraken data fusion, precision scoring)
- Rust legacy simulator kept for dev only

## Quick Start
- Build Go engine:
  - `go build -o macro_strike_bot trading_engine.go`
- Configure env (copy and set):
  - `cp .env.example .env` and export needed vars
- Run live (Kraken):
  - `LIVE_TRADING=1 ./macro_strike_bot`

## Components
- `trading_engine.go`: Go engine, accuracy gating (EXECUTE + conf≥0.80), Kraken live trade + exit
- `market_analysis.jl`: Julia analysis; caching, rate limiting, robust validation
- `src/main.rs`: legacy Rust sim (not used in live mode)
- `scripts/`: helpers to build and run
- `docs/`: audits and developer docs
- `configs/`: env templates

## Safety
- Trades only when Julia recommends EXECUTE and adjusted confidence ≥ 0.80 (accuracy mode)
- Live path: market buy sized by USD, timed market exit, realized PnL logged

## Requirements
- Go 1.20+
- Julia 1.9+ with packages: HTTP, JSON, Statistics, Dates, Random
- Kraken API key/secret (spot trading)

See `docs/ACCURACY_AUDIT.md` and `docs/DEVELOPERS.md`.
