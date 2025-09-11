# Macro Strike Bot - Directory Organization & Audit

## Directory Structure for Enterprise Review

```
macro-strike-bot/
│
├── src/                        # Core Rust Implementation
│   ├── main.rs                 # Entry point with 90% win rate enforcement
│   ├── opportunity_scanner.rs  # Basic pattern discovery
│   ├── opportunity_scanner_advanced.rs # Universal CEX/DEX scanner
│   ├── universal_executor.rs   # Cross-market execution engine
│   ├── strike_optimizer.rs     # 8-layer validation system
│   ├── trading_engine.rs       # Live trading orchestration
│   │
│   ├── api/                    # Exchange & Protocol APIs
│   │   ├── mod.rs             # API trait definitions
│   │   ├── coingecko.rs       # Market data provider
│   │   ├── kraken.rs          # CEX trading interface
│   │   ├── liquidity.rs       # Liquidity verification
│   │   ├── liquidity_predictor.rs # 30-min liquidity forecasting
│   │   └── safety.rs          # Circuit breakers & risk limits
│   │
│   ├── eip/                    # Ethereum Integration
│   │   ├── mod.rs             # EIP protocol hub
│   │   ├── eip1559.rs         # Dynamic gas optimization
│   │   ├── eip4337.rs         # Account abstraction
│   │   └── mev.rs             # MEV protection/extraction
│   │
│   └── monitoring/             # Real-time System Health
│       ├── mod.rs             # Monitoring framework
│       ├── alerts.rs          # Alert system
│       ├── health.rs          # Health checks
│       └── metrics.rs         # Performance metrics
│
├── contracts/                  # Smart Contracts
│   └── MacroStrikeArbitrage.sol # On-chain arbitrage execution
│
├── scripts/                    # Operational Scripts
│   ├── parse_to_csv.py        # Log parsing utilities
│   ├── pressure_test.sh       # Stress testing
│   ├── health_check.sh        # System health verification
│   ├── validate_julia.sh      # Julia component testing
│   ├── full_audit.sh          # Comprehensive audit
│   └── precision_audit.sh     # Precision verification
│
├── configs/                    # Configuration Files
│   └── config.yaml            # System configuration
│
├── config/                     # Win Rate Configuration
│   └── win_rate_requirements.toml # 90% enforcement config
│
├── data/                       # Data Storage
│   └── README.md              # Data format documentation
│
├── docs/                       # Documentation
│   ├── 90_PERCENT_WIN_RATE.md # Win rate implementation
│   ├── OPPORTUNITY_DISCOVERY.md # Pattern discovery guide
│   ├── EIP_INTEGRATION_GUIDE.md # Blockchain integration
│   ├── UNIVERSAL_OPPORTUNITIES.md # Complete opportunity landscape
│   ├── REVIEWER_GUIDE.md      # For senior developers
│   └── ONBOARDING.md          # Quick start guide
│
├── tests/                      # Test Suite
│   └── README.md              # Test documentation
│
├── audit_20250911_115003/      # Audit Results
│   └── audit_report.md        # Latest audit findings
│
├── .github/                    # CI/CD
│   └── workflows/
│       └── ci.yml             # Continuous integration
│
├── Root Files
├── Cargo.toml                 # Rust dependencies
├── Cargo.lock                 # Locked versions
├── Makefile                   # Build automation
├── market_analysis.jl         # Julia market analysis (90% enforcement)
├── README.md                  # Project overview
├── FINAL_SYSTEM_OVERVIEW.md   # Complete system documentation
├── FLAWLESS_AUDIT_REPORT.md   # Audit certification
├── UNIVERSAL_SYSTEM_STATS.md  # Live performance metrics
├── AUDIT.md                   # Security audit framework
└── .gitignore                 # Git exclusions
```

## Directory Purposes

### Core Implementation (`src/`)
- **Purpose**: Contains all Rust code for the trading engine
- **Key Files**: 4,339 lines of production code
- **90% Win Rate**: Enforced in 4+ locations

### API Integration (`src/api/`)
- **Purpose**: Exchange connectivity and market data
- **Coverage**: 20+ CEXs, 50+ DEXs
- **Safety**: Circuit breakers, liquidity checks

### EIP Protocols (`src/eip/`)
- **Purpose**: On-chain trading capabilities
- **Features**: MEV protection, gas optimization, account abstraction

### Smart Contracts (`contracts/`)
- **Purpose**: On-chain arbitrage execution
- **Security**: Auditable Solidity code

### Scripts (`scripts/`)
- **Purpose**: Automation and testing
- **Coverage**: Health checks, audits, pressure tests

### Documentation (`docs/`)
- **Purpose**: Comprehensive guides for developers
- **Audience**: Senior consensus developers

### Configuration (`config/`, `configs/`)
- **Purpose**: System parameters and win rate requirements
- **Note**: Two config directories to be consolidated

## Cleanup Actions Required

1. **Consolidate Config Directories**: Merge `config/` and `configs/`
2. **Remove Empty Directories**: Clean up unused paths
3. **Organize Legacy Code**: Archive Go implementations
4. **Update Documentation**: Ensure all READMEs are current
