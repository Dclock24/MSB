#!/usr/bin/env bash
set -euo pipefail

echo "Building Go engine..."
go build -o macro_strike_bot trading_engine.go

echo "Running (live=$LIVE_TRADING)" 
./macro_strike_bot
