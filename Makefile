.PHONY: build run live test audit package

build:
	go build -o macro_strike_bot trading_engine.go

run: build
	./macro_strike_bot

live: build
	LIVE_TRADING=1 ./macro_strike_bot

julia-setup:
	scripts/setup_julia.sh

audit:
	@echo "Go version:" && go version
	@echo "Julia version:" && julia --version || true
	@echo "Modules:" && ls -la

package:
	git bundle create macro-strike-bot.bundle --all
	tar --exclude-vcs --exclude=macro-strike-bot.bundle --exclude=macro_strike_bot -czf macro-strike-bot.tar.gz .
	@echo "Created macro-strike-bot.bundle and macro-strike-bot.tar.gz"

pressure-test:
	@echo "Running pressure test (SIM_MODE=1, 2,500 trades)..."
	SIM_MODE=1 ORDER_RISK_PCT=1 ./macro_strike_bot > sim2500.log 2>&1 || true
	python3 scripts/parse_to_csv.py || true
	@echo "Done. See data/ and reports/"
