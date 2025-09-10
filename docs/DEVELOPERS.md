# Developers Guide

## Repo Layout
- `trading_engine.go` — Go live engine (Kraken). Ownership: Trading Core Team
- `market_analysis.jl` — Julia analysis service. Ownership: Quant/Research
- `src/main.rs` — Rust legacy simulation. Ownership: Dev Tools
- `scripts/` — Build/run scripts
- `configs/` — Config templates
- `docs/` — Specs, audits, runbooks
- `.github/workflows/` — CI workflows

## Common Tasks
- Build Go: `go build -o macro_strike_bot trading_engine.go`
- Run live: `LIVE_TRADING=1 KRAKEN_API_KEY=... KRAKEN_API_SECRET=... ./macro_strike_bot`
- Test Julia: `julia market_analysis.jl WETH/USDC MacroMomentum`

## Conventions
- Go: explicit error handling, atomic counters for shared metrics
- Julia: cache API calls, clamp metrics, avoid panic on API variance
- Logs: structured single-line; no secrets

## Roadmap
- Replace timed exit with managed TP/SL
- Add fee modeling and minimum lot precision per pair
- Dockerize Julia and Go services

