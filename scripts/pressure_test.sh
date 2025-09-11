#!/bin/bash
# Pressure test for macro-strike-bot
# Tests multiple concurrent trades and verifies system stability

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Starting Pressure Test...${NC}"

# Create test directory
TEST_DIR="test_runs/pressure_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$TEST_DIR"

# Configuration
ITERATIONS=10
PARALLEL_RUNS=3
SIM_TRADES=5000

echo "Configuration:"
echo "- Iterations: $ITERATIONS"
echo "- Parallel runs: $PARALLEL_RUNS"
echo "- Trades per run: $SIM_TRADES"
echo "- Output dir: $TEST_DIR"
echo ""

# Function to run a single test
run_test() {
    local id=$1
    local output_file="$TEST_DIR/run_${id}.log"
    
    echo -e "${YELLOW}[Run $id] Starting...${NC}"
    
    # Run with increased trade count
    SIM_MODE=true SIM_TRADES=$SIM_TRADES cargo run --release > "$output_file" 2>&1
    
    if [ $? -eq 0 ]; then
        # Extract metrics
        total_trades=$(grep "Total Trades:" "$output_file" | awk '{print $3}')
        win_rate=$(grep "Win Rate:" "$output_file" | awk '{print $3}')
        avg_win=$(grep "Avg Win:" "$output_file" | awk '{print $3}')
        avg_loss=$(grep "Avg Loss:" "$output_file" | awk '{print $3}')
        
        echo -e "${GREEN}[Run $id] Completed - Trades: $total_trades, Win Rate: $win_rate${NC}"
        echo "$id,$total_trades,$win_rate,$avg_win,$avg_loss" >> "$TEST_DIR/summary.csv"
    else
        echo -e "${RED}[Run $id] FAILED${NC}"
        return 1
    fi
}

# Initialize summary
echo "run_id,total_trades,win_rate,avg_win,avg_loss" > "$TEST_DIR/summary.csv"

# Run tests
echo -e "\n${YELLOW}Running $ITERATIONS iterations...${NC}"

for i in $(seq 1 $ITERATIONS); do
    echo -e "\n${YELLOW}=== Iteration $i/$ITERATIONS ===${NC}"
    
    # Run tests in parallel
    for j in $(seq 1 $PARALLEL_RUNS); do
        run_test "${i}_${j}" &
    done
    
    # Wait for parallel runs to complete
    wait
    
    # Small delay between iterations
    sleep 2
done

# Analyze results
echo -e "\n${YELLOW}Analyzing results...${NC}"

# Check for consistency
if [ -f "$TEST_DIR/summary.csv" ]; then
    # Calculate statistics using awk
    awk -F',' 'NR>1 {
        total_runs++
        sum_trades += $2
        sum_winrate += $3
        if ($3 < min_winrate || min_winrate == 0) min_winrate = $3
        if ($3 > max_winrate) max_winrate = $3
    }
    END {
        if (total_runs > 0) {
            avg_trades = sum_trades / total_runs
            avg_winrate = sum_winrate / total_runs
            
            print "\n=== PRESSURE TEST SUMMARY ==="
            print "Total test runs: " total_runs
            print "Average trades processed: " avg_trades
            print "Average win rate: " avg_winrate "%"
            print "Win rate range: " min_winrate "% - " max_winrate "%"
            
            # Check stability
            range = max_winrate - min_winrate
            if (range < 5) {
                print "\n✓ STABLE: Win rate variance < 5%"
            } else {
                print "\n⚠ UNSTABLE: Win rate variance = " range "%"
            }
        }
    }' "$TEST_DIR/summary.csv"
fi

# Check for errors
ERROR_COUNT=$(grep -l "ERROR\|PANIC\|panic\|error" "$TEST_DIR"/*.log 2>/dev/null | wc -l)
if [ $ERROR_COUNT -gt 0 ]; then
    echo -e "\n${RED}⚠ Found errors in $ERROR_COUNT runs${NC}"
    echo "Check logs in: $TEST_DIR"
else
    echo -e "\n${GREEN}✓ No errors detected${NC}"
fi

# Memory usage check (macOS specific)
if command -v vm_stat &> /dev/null; then
    echo -e "\n${YELLOW}System Memory Status:${NC}"
    vm_stat | grep -E "(free|active|inactive|wired down)" | head -4
fi

echo -e "\n${GREEN}Pressure test complete!${NC}"
echo "Results saved to: $TEST_DIR"
