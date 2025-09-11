# Test Suite

Comprehensive test coverage for the Macro Strike Bot.

## Test Categories

### Unit Tests
- Pattern recognition accuracy
- Win rate calculations
- Risk management rules
- API mocking

### Integration Tests
- CEX API connectivity
- DEX transaction simulation
- Cross-market arbitrage flows
- Liquidity verification

### Performance Tests
- Latency benchmarks
- Throughput testing
- Memory usage profiling
- Concurrent operation limits

## Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run specific test
cargo test test_90_percent_win_rate

# Run benchmarks
cargo bench
```

## Test Requirements

All tests must verify:
1. 90% win rate enforcement
2. Risk limits compliance
3. Liquidity adequacy
4. Execution correctness