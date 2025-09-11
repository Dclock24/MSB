# Macro Strike Bot - Hybrid Build System
.PHONY: all build build-rust build-go run run-rust run-go sim sim-quick health pressure-test audit clean package help

# Default target
all: build

# Build targets
build: build-rust build-go
	@echo "✓ All components built successfully"

build-rust:
	@echo "Building Rust components..."
	@cargo build --release

build-go:
	@echo "Building Go components..."
	@go build -o macro_strike_bot trading_engine.go

# Run targets
run: run-rust

run-rust: build-rust
	@echo "Running Rust simulation..."
	@SIM_MODE=true RUST_LOG=info ./target/release/macro_strike_bot_fixed

run-go: build-go
	@echo "Running Go engine..."
	@./macro_strike_bot

# Live trading (GO engine)
live: build-go
	@echo "⚠️  LIVE TRADING MODE - USE WITH CAUTION"
	@LIVE_TRADING=1 ./macro_strike_bot

# Simulation targets
sim: build-rust
	@echo "Running full simulation (2500 trades)..."
	@mkdir -p reports
	@SIM_MODE=true SIM_TRADES=2500 RUST_LOG=info ./target/release/macro_strike_bot_fixed | tee reports/sim_report_$$(date +%Y%m%d_%H%M%S).txt

sim-quick: build-rust
	@echo "Running quick simulation (100 trades)..."
	@SIM_MODE=true SIM_TRADES=100 RUST_LOG=info ./target/release/macro_strike_bot_fixed

# Testing and validation
health:
	@bash scripts/health_check.sh

pressure-test: build
	@echo "Running pressure test..."
	@bash scripts/pressure_test.sh

test: build
	@echo "Running tests..."
	@cargo test
	@go test ./...

# Analysis and reporting
analyze: sim
	@echo "Analyzing simulation results..."
	@bash scripts/parse_sim_log.sh reports/sim_report_*.txt | tail -1
	@python3 scripts/parse_to_csv.py

# Setup targets
julia-setup:
	@bash scripts/setup_julia.sh

deps:
	@echo "Installing dependencies..."
	@cargo fetch
	@go mod download || true

# Audit and info
audit:
	@echo "=== System Audit ==="
	@echo "Rust version:" && rustc --version || echo "Rust not found"
	@echo "Cargo version:" && cargo --version || echo "Cargo not found"
	@echo "Go version:" && go version || echo "Go not found"
	@echo "Julia version:" && julia --version || echo "Julia not found"
	@echo "Python version:" && python3 --version || echo "Python not found"
	@echo ""
	@echo "=== Project Structure ==="
	@echo "Rust source:" && ls -la src/ | grep -E "\.rs$$" | wc -l | xargs echo "files"
	@echo "Go source:" && ls -la *.go | wc -l | xargs echo "files"
	@echo "Julia source:" && ls -la *.jl | wc -l | xargs echo "files"
	@echo ""
	@echo "=== Build Status ==="
	@test -f target/release/macro_strike_bot_fixed && echo "✓ Rust binary built" || echo "✗ Rust binary missing"
	@test -f macro_strike_bot && echo "✓ Go binary built" || echo "✗ Go binary missing"

# Cleanup
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -f macro_strike_bot
	@rm -rf test_runs/
	@find . -name "*.log" -type f -delete

clean-all: clean
	@echo "Cleaning all generated files..."
	@rm -rf data/*.csv
	@rm -rf reports/*.txt

# Packaging
package:
	@echo "Creating distribution package..."
	@git bundle create macro-strike-bot.bundle --all
	@tar --exclude-vcs --exclude='target' --exclude='*.log' --exclude='test_runs' \
		--exclude='macro-strike-bot.bundle' --exclude='macro_strike_bot' \
		-czf macro-strike-bot.tar.gz .
	@echo "✓ Created macro-strike-bot.bundle and macro-strike-bot.tar.gz"

# Help
help:
	@echo "Macro Strike Bot - Build Targets"
	@echo ""
	@echo "Building:"
	@echo "  make build      - Build all components (Rust + Go)"
	@echo "  make build-rust - Build Rust simulation engine"
	@echo "  make build-go   - Build Go trading engine"
	@echo ""
	@echo "Running:"
	@echo "  make run        - Run Rust simulation"
	@echo "  make run-go     - Run Go engine"
	@echo "  make live       - Run LIVE trading (Go engine)"
	@echo ""
	@echo "Testing:"
	@echo "  make sim        - Run full simulation (2500 trades)"
	@echo "  make sim-quick  - Run quick simulation (100 trades)"
	@echo "  make health     - Run system health check"
	@echo "  make pressure-test - Run pressure testing"
	@echo "  make test       - Run all tests"
	@echo ""
	@echo "Analysis:"
	@echo "  make analyze    - Run simulation and analyze results"
	@echo "  make audit      - Show system and project info"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make clean-all  - Clean everything"
	@echo "  make package    - Create distribution package"
	@echo "  make deps       - Install dependencies"
	@echo ""