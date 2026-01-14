# Strike Box - Institutional Bilateral Execution Framework

**Version:** 1.0.0  
**Author:** Byt Block LLC  
**License:** Proprietary  
**Grade:** Institutional Enterprise

## Overview

Strike Box is an institutional-grade bilateral execution framework designed for high-frequency, high-capital trading operations. It provides comprehensive risk management, position sizing, portfolio control, and audit capabilities.

## Features

### Core Capabilities

- **Multi-Gate Risk Validation** - 10+ validation gates before entry
- **Safety Scoring System** - 4-factor analysis (liquidity, holders, age, contract)
- **Liquidity-Based Position Sizing** - Dynamic sizing based on pool depth
- **Partial Exit Strategy** - Multiple take-profit levels with partial exits
- **Trailing Stop Loss** - Dynamic stop loss management
- **Portfolio State Management** - Real-time exposure and drawdown tracking
- **Operational Commands** - Command-based control system
- **Comprehensive Audit Logging** - Entry/exit/rejection logging

### Risk Management

- Daily/Weekly/Monthly drawdown limits
- Net exposure bounds (-30% to +70%)
- Market crash detection (15% trigger)
- Liquidity crisis detection (50% trigger)
- Execution failure tracking
- Data feed staleness monitoring

### Position Management

- **Long Book:** Up to 10 positions, 70% of capital
- **Short Book:** Up to 3 positions, 30% of capital
- Single position max: 2% of capital
- Liquidity-based scaling (0.5% - 2%)
- No position stacking (one per token)

### Exit Strategy

**Long Positions:**
- TP1: 15% gain (exit 33% of position)
- TP2: 30% gain (exit 33% of position)
- TP3: 50% gain (exit 34% of position)
- Stop Loss: 5% default, 8% for volatile tokens
- Trailing Stop: Activates at 15% gain, 10% distance

**Short Positions:**
- TP1: 10% gain (exit 33% of position)
- TP2: 20% gain (exit 33% of position)
- TP3: 30% gain (exit 34% of position)
- Stop Loss: 8% fixed
- Squeeze protection

## Usage

### Basic Setup

```rust
use strike_box::*;
use rust_decimal::Decimal;

// Initialize with default config
let config = StrikeBoxConfig::default();
let mut engine = StrikeBoxEngine::new(config, Decimal::new(1_000_000, 0));
```

### Token Validation

```rust
// Create token snapshot
let token = TokenSnapshot {
    token_address: "0x...".to_string(),
    token_symbol: "TOKEN".to_string(),
    liquidity_usd: Decimal::new(750_000, 0),
    bid_depth_usd: Decimal::new(375_000, 0),
    ask_depth_usd: Decimal::new(375_000, 0),
    holder_count: 60,
    top_10_concentration_pct: Decimal::new(45, 2),
    largest_wallet_pct: Decimal::new(12, 2),
    token_age_hours: 48,
    contract_verified: true,
    is_proxy_contract: false,
    deployment_timestamp: Utc::now(),
    snapshot_timestamp: Utc::now(),
};

// Validate entry
let validation = engine.validate_entry(&token, Direction::Long);
if validation.all_passed {
    // Calculate position size
    let size = engine.calculate_position_size(&token, Direction::Long);
    // Execute entry...
} else {
    // Log rejection
    engine.log_rejection(&token, Direction::Long, &validation);
}
```

### Operational Commands

```rust
// System control
engine.execute_command(OperationalCommand::PauseLongs);
engine.execute_command(OperationalCommand::PauseShorts);
engine.execute_command(OperationalCommand::PauseAll);
engine.execute_command(OperationalCommand::Resume);

// Status queries
engine.execute_command(OperationalCommand::Status);
engine.execute_command(OperationalCommand::Exposure);
engine.execute_command(OperationalCommand::Risk);
engine.execute_command(OperationalCommand::Health);

// Emergency controls
engine.execute_command(OperationalCommand::CloseAll);
```

## Configuration

All parameters are configurable through `StrikeBoxConfig`:

```rust
let mut config = StrikeBoxConfig::default();

// Token validation
config.token_validation.liquidity_min_usd = Decimal::new(500_000, 0);
config.token_validation.liquidity_max_usd = Decimal::new(1_000_000, 0);
config.token_validation.holder_count_min = 25;

// Safety scoring
config.safety_scoring.long_entry_min = Decimal::new(60, 2);
config.safety_scoring.short_entry_min = Decimal::new(50, 2);

// Position sizing
config.position_sizing.long_book_max_pct = Decimal::new(70, 2);
config.position_sizing.short_book_max_pct = Decimal::new(30, 2);

// Stop loss
config.stop_loss.long_default_pct = Decimal::new(5, 2);
config.stop_loss.short_fixed_pct = Decimal::new(8, 2);

// Take profit
config.take_profit.long_tp1_pct = Decimal::new(15, 2);
config.take_profit.long_tp2_pct = Decimal::new(30, 2);
config.take_profit.long_tp3_pct = Decimal::new(50, 2);

// Risk controller
config.risk_controller.daily_drawdown_halt_pct = Decimal::new(5, 2);
config.risk_controller.weekly_drawdown_halt_pct = Decimal::new(10, 2);
```

## Architecture

### Core Components

1. **TokenValidationConfig** - Token eligibility requirements
2. **SafetyScoreConfig** - Scoring weights and thresholds
3. **PositionSizingConfig** - Position and book limits
4. **StopLossConfig** - Stop loss parameters
5. **TakeProfitConfig** - Take profit levels and exit percentages
6. **TimeControlConfig** - Time-based controls
7. **RiskControllerConfig** - Risk management limits

### Data Structures

- **TokenSnapshot** - Token state at validation time
- **SafetyScore** - Calculated safety score with risk level
- **Position** - Active position with P&L tracking
- **PositionBook** - Collection of positions (long/short)
- **PortfolioState** - Overall portfolio state
- **RiskValidation** - Multi-gate validation result

### Validation Gates

1. System state check
2. Liquidity range validation
3. Safety score threshold
4. Token age requirement
5. Contract verification
6. Holder distribution
7. Book capacity check
8. No position stacking
9. Squeeze risk (shorts only)
10. Net exposure bounds

## Testing

Run tests with:

```bash
cargo test
```

The test suite includes:
- Default config validation
- Safety score calculations
- Liquidity scaler tests
- Engine validation tests
- Stop loss calculations
- Take profit calculations
- Operational command tests

## License

Proprietary - Byt Block LLC

## Support

For integration support and licensing inquiries, contact Byt Block LLC.

