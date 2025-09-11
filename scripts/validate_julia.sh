#!/bin/bash
# Validate Julia Market Analysis Accuracy
# Tests prediction accuracy, execution signals, and confidence scoring

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== JULIA MARKET ANALYSIS VALIDATION ===${NC}\n"

# Test configurations
SYMBOLS=("BTC/USDT" "ETH/USDT" "SOL/USDT")
STRIKE_TYPES=("MacroArbitrage" "MacroMomentum" "MacroVolatility" "MacroLiquidity" "MacroFunding" "MacroFlash")
TEST_RUNS=10
RESULTS_DIR="test_runs/julia_validation_$(date +%Y%m%d_%H%M%S)"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Initialize counters
TOTAL_TESTS=0
SUCCESSFUL_CALLS=0
EXECUTE_SIGNALS=0
WAIT_SIGNALS=0
HIGH_CONFIDENCE=0
ERRORS=0

# CSV header
echo "timestamp,symbol,strike_type,price,confidence,expected_return,volatility,momentum,liquidity,precision_score,recommendation,execution_time_ms" > "$RESULTS_DIR/validation_results.csv"

echo -e "${YELLOW}Running validation tests...${NC}\n"

# Function to test Julia execution
test_julia_execution() {
    local symbol=$1
    local strike_type=$2
    local run_number=$3
    
    echo -n "[$run_number] Testing $symbol with $strike_type... "
    
    # Measure execution time
    local start_time=$(date +%s%N)
    
    # Run Julia script and capture output
    local output_file="$RESULTS_DIR/output_${symbol//\//_}_${strike_type}_${run_number}.json"
    local error_file="$RESULTS_DIR/error_${symbol//\//_}_${strike_type}_${run_number}.log"
    
    if julia market_analysis.jl "$symbol" "$strike_type" > "$output_file" 2> "$error_file"; then
        local end_time=$(date +%s%N)
        local exec_time_ms=$(( (end_time - start_time) / 1000000 ))
        
        # Parse JSON output
        if [ -s "$output_file" ] && grep -q "recommendation" "$output_file"; then
            # Extract values using grep and sed (portable across systems)
            local confidence=$(grep -o '"confidence":[0-9.]*' "$output_file" | sed 's/"confidence"://')
            local recommendation=$(grep -o '"recommendation":"[^"]*"' "$output_file" | sed 's/"recommendation":"//' | sed 's/"//')
            local precision=$(grep -o '"precision_score":[0-9.]*' "$output_file" | sed 's/"precision_score"://')
            local price=$(grep -o '"price":[0-9.]*' "$output_file" | sed 's/"price"://')
            local expected_return=$(grep -o '"expected_return":[0-9.]*' "$output_file" | sed 's/"expected_return"://')
            local volatility=$(grep -o '"volatility":[0-9.]*' "$output_file" | sed 's/"volatility"://')
            local momentum=$(grep -o '"momentum":[0-9.]*' "$output_file" | sed 's/"momentum"://')
            local liquidity=$(grep -o '"liquidity":[0-9.]*' "$output_file" | sed 's/"liquidity"://')
            
            # Record to CSV
            echo "$(date +%s),$symbol,$strike_type,$price,$confidence,$expected_return,$volatility,$momentum,$liquidity,$precision,$recommendation,$exec_time_ms" >> "$RESULTS_DIR/validation_results.csv"
            
            # Update counters
            ((SUCCESSFUL_CALLS++))
            
            if [ "$recommendation" = "EXECUTE" ]; then
                ((EXECUTE_SIGNALS++))
                echo -e "${GREEN}EXECUTE${NC} (conf: $confidence, prec: $precision, time: ${exec_time_ms}ms)"
            else
                ((WAIT_SIGNALS++))
                echo -e "${YELLOW}WAIT${NC} (conf: $confidence, prec: $precision, time: ${exec_time_ms}ms)"
            fi
            
            # Check high confidence
            if (( $(echo "$confidence > 0.85" | bc -l) )); then
                ((HIGH_CONFIDENCE++))
            fi
            
            # Validate ranges
            if (( $(echo "$confidence < 0 || $confidence > 1" | bc -l) )); then
                echo -e "  ${RED}WARNING: Confidence out of range!${NC}"
            fi
            if (( $(echo "$precision < 0 || $precision > 1" | bc -l) )); then
                echo -e "  ${RED}WARNING: Precision out of range!${NC}"
            fi
            
        else
            echo -e "${RED}FAILED${NC} - Invalid output format"
            ((ERRORS++))
            cat "$error_file" >> "$RESULTS_DIR/all_errors.log"
        fi
    else
        echo -e "${RED}ERROR${NC} - Script execution failed"
        ((ERRORS++))
        cat "$error_file" >> "$RESULTS_DIR/all_errors.log"
    fi
    
    ((TOTAL_TESTS++))
    
    # Small delay between tests
    sleep 0.5
}

# Run validation tests
for run in $(seq 1 $TEST_RUNS); do
    echo -e "\n${BLUE}=== Test Run $run/$TEST_RUNS ===${NC}"
    
    for symbol in "${SYMBOLS[@]}"; do
        for strike_type in "${STRIKE_TYPES[@]}"; do
            test_julia_execution "$symbol" "$strike_type" "$run"
        done
    done
    
    # Longer delay between runs
    sleep 2
done

# Analyze results
echo -e "\n${BLUE}=== VALIDATION SUMMARY ===${NC}"
echo "Total tests: $TOTAL_TESTS"
echo "Successful calls: $SUCCESSFUL_CALLS"
echo "Failed calls: $ERRORS"
echo "Success rate: $(( SUCCESSFUL_CALLS * 100 / TOTAL_TESTS ))%"
echo ""
echo "Signal Distribution:"
echo "- EXECUTE signals: $EXECUTE_SIGNALS ($(( EXECUTE_SIGNALS * 100 / SUCCESSFUL_CALLS ))%)"
echo "- WAIT signals: $WAIT_SIGNALS ($(( WAIT_SIGNALS * 100 / SUCCESSFUL_CALLS ))%)"
echo "- High confidence (>0.85): $HIGH_CONFIDENCE ($(( HIGH_CONFIDENCE * 100 / SUCCESSFUL_CALLS ))%)"

# Calculate statistics from CSV
echo -e "\n${BLUE}=== PERFORMANCE METRICS ===${NC}"

# Average execution time
AVG_EXEC_TIME=$(awk -F',' 'NR>1 {sum+=$12; count++} END {print sum/count}' "$RESULTS_DIR/validation_results.csv")
echo "Average execution time: ${AVG_EXEC_TIME}ms"

# Confidence statistics
echo -e "\nConfidence Statistics:"
awk -F',' 'NR>1 && $11=="EXECUTE" {sum+=$5; count++; if($5>max)max=$5; if(min==""||$5<min)min=$5} 
    END {print "  EXECUTE: avg=" sum/count ", min=" min ", max=" max}' "$RESULTS_DIR/validation_results.csv"
awk -F',' 'NR>1 && $11=="WAIT" {sum+=$5; count++; if($5>max)max=$5; if(min==""||$5<min)min=$5} 
    END {print "  WAIT: avg=" sum/count ", min=" min ", max=" max}' "$RESULTS_DIR/validation_results.csv"

# Precision statistics
echo -e "\nPrecision Score Statistics:"
awk -F',' 'NR>1 {sum+=$10; count++; if($10>max)max=$10; if(min==""||$10<min)min=$10} 
    END {print "  Overall: avg=" sum/count ", min=" min ", max=" max}' "$RESULTS_DIR/validation_results.csv"

# Strike type analysis
echo -e "\n${BLUE}=== STRIKE TYPE ANALYSIS ===${NC}"
for strike_type in "${STRIKE_TYPES[@]}"; do
    EXECUTE_COUNT=$(grep ",$strike_type," "$RESULTS_DIR/validation_results.csv" | grep ",EXECUTE," | wc -l)
    TOTAL_COUNT=$(grep ",$strike_type," "$RESULTS_DIR/validation_results.csv" | tail -n +2 | wc -l)
    if [ $TOTAL_COUNT -gt 0 ]; then
        EXECUTE_RATE=$(( EXECUTE_COUNT * 100 / TOTAL_COUNT ))
        echo "$strike_type: $EXECUTE_COUNT/$TOTAL_COUNT executions ($EXECUTE_RATE%)"
    fi
done

# Check for consistency
echo -e "\n${BLUE}=== CONSISTENCY CHECKS ===${NC}"

# Check if same inputs produce consistent outputs
for symbol in "${SYMBOLS[@]}"; do
    for strike_type in "${STRIKE_TYPES[@]}"; do
        # Get all confidence values for this combination
        CONF_VALUES=$(grep ",$symbol,$strike_type," "$RESULTS_DIR/validation_results.csv" | cut -d',' -f5 | tail -n +2)
        if [ -n "$CONF_VALUES" ]; then
            # Calculate standard deviation
            STD_DEV=$(echo "$CONF_VALUES" | awk '{sum+=$1; sumsq+=$1*$1; n++} END {print sqrt(sumsq/n - (sum/n)^2)}')
            echo "$symbol/$strike_type confidence std dev: $STD_DEV"
            
            # Flag high variance
            if (( $(echo "$STD_DEV > 0.1" | bc -l) )); then
                echo -e "  ${YELLOW}WARNING: High variance detected${NC}"
            fi
        fi
    done
done

# Generate summary report
cat > "$RESULTS_DIR/validation_report.txt" << EOF
Julia Market Analysis Validation Report
Generated: $(date)

TEST CONFIGURATION
==================
Symbols tested: ${SYMBOLS[@]}
Strike types: ${STRIKE_TYPES[@]}
Test runs: $TEST_RUNS
Total tests: $TOTAL_TESTS

RESULTS SUMMARY
===============
Success rate: $(( SUCCESSFUL_CALLS * 100 / TOTAL_TESTS ))%
Error rate: $(( ERRORS * 100 / TOTAL_TESTS ))%

Signal distribution:
- EXECUTE: $EXECUTE_SIGNALS ($(( EXECUTE_SIGNALS * 100 / SUCCESSFUL_CALLS ))%)
- WAIT: $WAIT_SIGNALS ($(( WAIT_SIGNALS * 100 / SUCCESSFUL_CALLS ))%)

Performance:
- Average execution time: ${AVG_EXEC_TIME}ms
- High confidence rate: $(( HIGH_CONFIDENCE * 100 / SUCCESSFUL_CALLS ))%

RECOMMENDATIONS
===============
EOF

# Add recommendations based on results
if [ $ERRORS -gt 0 ]; then
    echo "- ERROR: $ERRORS failures detected. Check error logs." >> "$RESULTS_DIR/validation_report.txt"
fi

if (( $(echo "$AVG_EXEC_TIME > 1000" | bc -l) )); then
    echo "- WARNING: High execution time (>1s). Consider optimization." >> "$RESULTS_DIR/validation_report.txt"
fi

EXECUTE_RATE=$(( EXECUTE_SIGNALS * 100 / SUCCESSFUL_CALLS ))
if [ $EXECUTE_RATE -lt 20 ] || [ $EXECUTE_RATE -gt 80 ]; then
    echo "- WARNING: Unusual EXECUTE rate ($EXECUTE_RATE%). Review confidence thresholds." >> "$RESULTS_DIR/validation_report.txt"
fi

echo -e "\n${GREEN}✓ Validation complete!${NC}"
echo "Results saved to: $RESULTS_DIR"
echo "Full report: $RESULTS_DIR/validation_report.txt"

# Return exit code based on success
if [ $ERRORS -eq 0 ] && [ $SUCCESSFUL_CALLS -gt 0 ]; then
    exit 0
else
    exit 1
fi
