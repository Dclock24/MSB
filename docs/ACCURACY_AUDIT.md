# Accuracy & Dependency Audit

Scope: Go live engine (`trading_engine.go`), Julia analysis (`market_analysis.jl`), Rust sim (`src/main.rs`).

## Summary
- Live path executes only on EXECUTE from Julia with adjusted confidence (confidence × precision_score) ≥ 0.80.
- Orders placed on Kraken: market buy sized by USD, timed market sell exit, realized PnL computed from Kraken order prices and volumes.
- Julia analysis uses CoinGecko + Kraken (with caching, rate limiting, retries). Values are validated/clamped to avoid unstable math.

## Live Trading Accuracy
- Entry: market order via `/0/private/AddOrder` with `type=buy`, `ordertype=market`, volume sized by `ORDER_USD_SIZE / entry_price`.
- Fill detection: polls `/0/private/QueryOrders` until `vol_exec>0` or 30s timeout.
- Exit: market sell for filled volume. Exit price polled from order info.
- PnL: `(sellPrice - buyPrice) * filledVolume`, recorded in USD; capital and totals updated atomically.
- Logging: `LIVE ORDER` and `LIVE EXIT` with txids; status `Hit`/`Miss` based on PnL sign.

Risk: Market slippage and partial fills; holding time set to 20s; can be tuned.

## Signal Accuracy (Julia)
- Recommendation gate: `adjusted_confidence = min(0.99, confidence * type_mult)`, proceed only if `>0.85` (Julia) and engine threshold 0.80.
- Precision score: balanced weights across volatility, momentum, liquidity, and simple technicals; all inputs validated and clamped.
- Data sources: CoinGecko and Kraken ticker endpoints; caching (30s), explicit sleeps (1–1.5s), retry with exponential backoff.

## Dependencies
- Go stdlib + no external modules; external: Kraken HTTPS API; Julia runtime via `exec.Command`.
- Julia packages: `HTTP`, `JSON`, `Statistics`, `Dates`, `Random`.
- Rust: `tokio`, `serde`, `log`, `env_logger`, `rand` (not used in live engine).

## Env & Secrets
- `.env.example`: `LIVE_TRADING`, `KRAKEN_API_KEY`, `KRAKEN_API_SECRET`, `ORDER_USD_SIZE`.
- No secrets stored in repo; keys read from env.

## Gaps / Next Steps
- Optional: replace timed exit with limit/TP/SL management.
- Optional: fetch live best bid/ask before sizing; handle fees and precision per asset.
- Optional: enrich Julia with more TA and verified PnL tracking per symbol.

