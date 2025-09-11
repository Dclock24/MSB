# Data Directory

This directory stores:
- Historical market data
- Opportunity logs
- Performance metrics
- Backtest results

## Data Format

All data files use JSON format for consistency and parseability.

### Opportunity Log Format
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "opportunity_id": "CEX_ARB_BINANCE_COINBASE_12345",
  "type": "CexToCexArbitrage",
  "profit_usd": 125.50,
  "win_rate": 0.94,
  "executed": true,
  "result": "success"
}
```