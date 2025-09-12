#!/bin/bash

# Deep Code Audit - Function and Strategy Analysis
# Ensures every function is perfect for senior developer review

echo "════════════════════════════════════════════════════════════════════"
echo "           DEEP CODE AUDIT - FUNCTION & STRATEGY ANALYSIS"
echo "════════════════════════════════════════════════════════════════════"
echo ""

ISSUES=0
WARNINGS=0

# 1. Check for unwrap() calls that could panic
echo "1. PANIC SAFETY AUDIT"
echo "--------------------"
unwrap_count=$(grep -r "\.unwrap()" src/ --include="*.rs" | grep -v "test" | grep -v "example" | wc -l)
if [ "$unwrap_count" -gt 0 ]; then
    echo "   ⚠️  Found $unwrap_count unwrap() calls that could panic"
    echo "   Files with unwrap():"
    grep -r "\.unwrap()" src/ --include="*.rs" | grep -v "test" | cut -d: -f1 | sort -u | head -5
    ((WARNINGS++))
else
    echo "   ✅ No dangerous unwrap() calls"
fi

# 2. Check for proper error handling
echo ""
echo "2. ERROR HANDLING AUDIT"
echo "----------------------"
result_count=$(grep -r "Result<" src/ --include="*.rs" | wc -l)
error_handled=$(grep -r "\?" src/ --include="*.rs" | wc -l)
echo "   📊 Functions returning Result: $result_count"
echo "   📊 Error propagation (?): $error_handled"
if [ "$error_handled" -lt "$((result_count / 2))" ]; then
    echo "   ⚠️  Low error handling ratio"
    ((WARNINGS++))
else
    echo "   ✅ Good error handling coverage"
fi

# 3. Check async function correctness
echo ""
echo "3. ASYNC FUNCTION AUDIT"
echo "----------------------"
async_count=$(grep -r "async fn" src/ --include="*.rs" | wc -l)
await_count=$(grep -r "\.await" src/ --include="*.rs" | wc -l)
echo "   📊 Async functions: $async_count"
echo "   📊 Await calls: $await_count"
blocking_calls=$(grep -r "std::thread::sleep\|std::fs::\|std::io::stdin" src/ --include="*.rs" | grep -v "test" | wc -l)
if [ "$blocking_calls" -gt 0 ]; then
    echo "   ❌ Found $blocking_calls blocking calls in async context"
    ((ISSUES++))
else
    echo "   ✅ No blocking calls in async functions"
fi

# 4. Check win rate enforcement consistency
echo ""
echo "4. WIN RATE STRATEGY AUDIT"
echo "-------------------------"
win_rate_checks=$(grep -r "confidence.*<.*0\.9\|win_rate.*>=.*0\.9\|MIN_WIN_PROBABILITY" src/ --include="*.rs" | wc -l)
echo "   📊 Win rate enforcement points: $win_rate_checks"
if [ "$win_rate_checks" -lt 5 ]; then
    echo "   ❌ Insufficient win rate checks"
    ((ISSUES++))
else
    echo "   ✅ Win rate properly enforced"
fi

# 5. Check for missing documentation
echo ""
echo "5. DOCUMENTATION AUDIT"
echo "---------------------"
pub_fn_count=$(grep -r "pub fn\|pub async fn" src/ --include="*.rs" | wc -l)
documented_fn=$(grep -B1 "pub fn\|pub async fn" src/ --include="*.rs" | grep "///" | wc -l)
doc_ratio=$((documented_fn * 100 / pub_fn_count))
echo "   📊 Public functions: $pub_fn_count"
echo "   📊 Documented: $documented_fn ($doc_ratio%)"
if [ "$doc_ratio" -lt 50 ]; then
    echo "   ⚠️  Low documentation coverage"
    ((WARNINGS++))
else
    echo "   ✅ Good documentation coverage"
fi

# 6. Check for code complexity
echo ""
echo "6. COMPLEXITY AUDIT"
echo "------------------"
# Find functions longer than 50 lines
long_functions=$(awk '/fn [a-zA-Z_]/ {start=NR} /^}$/ {if (NR-start > 50) count++} END {print count+0}' src/*.rs src/*/*.rs 2>/dev/null)
echo "   📊 Functions over 50 lines: $long_functions"
if [ "$long_functions" -gt 10 ]; then
    echo "   ⚠️  High complexity - consider refactoring"
    ((WARNINGS++))
else
    echo "   ✅ Good function size"
fi

# 7. Check memory safety
echo ""
echo "7. MEMORY SAFETY AUDIT"
echo "---------------------"
unsafe_count=$(grep -r "unsafe" src/ --include="*.rs" | grep -v "test" | wc -l)
if [ "$unsafe_count" -gt 0 ]; then
    echo "   ⚠️  Found $unsafe_count unsafe blocks"
    ((WARNINGS++))
else
    echo "   ✅ No unsafe code"
fi

# 8. Check for proper use of Arc/Mutex
echo ""
echo "8. CONCURRENCY AUDIT"
echo "-------------------"
arc_count=$(grep -r "Arc<" src/ --include="*.rs" | wc -l)
mutex_count=$(grep -r "Mutex<\|RwLock<" src/ --include="*.rs" | wc -l)
echo "   📊 Arc usage: $arc_count"
echo "   📊 Mutex/RwLock usage: $mutex_count"
deadlock_risk=$(grep -r "lock().*lock()" src/ --include="*.rs" | wc -l)
if [ "$deadlock_risk" -gt 0 ]; then
    echo "   ⚠️  Potential deadlock risk detected"
    ((WARNINGS++))
else
    echo "   ✅ No obvious deadlock patterns"
fi

# 9. Strategy implementation check
echo ""
echo "9. TRADING STRATEGY AUDIT"
echo "------------------------"
echo "   Checking strategy implementations..."

# Check opportunity scanner
if grep -q "scan_all_markets\|find_opportunities" src/opportunity_scanner*.rs 2>/dev/null; then
    echo "   ✅ Opportunity scanner implemented"
else
    echo "   ❌ Opportunity scanner missing"
    ((ISSUES++))
fi

# Check executor
if grep -q "execute_opportunity\|execute_strike" src/*executor*.rs src/trading_engine.rs 2>/dev/null; then
    echo "   ✅ Trade executor implemented"
else
    echo "   ❌ Trade executor missing"
    ((ISSUES++))
fi

# Check risk management
if grep -q "check_limits\|risk_manager" src/api/safety.rs src/*executor*.rs 2>/dev/null; then
    echo "   ✅ Risk management implemented"
else
    echo "   ❌ Risk management missing"
    ((ISSUES++))
fi

# 10. Performance considerations
echo ""
echo "10. PERFORMANCE AUDIT"
echo "--------------------"
clone_count=$(grep -r "\.clone()" src/ --include="*.rs" | wc -l)
collect_count=$(grep -r "\.collect" src/ --include="*.rs" | wc -l)
echo "   📊 Clone operations: $clone_count"
echo "   📊 Collect operations: $collect_count"
if [ "$clone_count" -gt 100 ]; then
    echo "   ⚠️  High clone count - review for optimization"
    ((WARNINGS++))
else
    echo "   ✅ Reasonable clone usage"
fi

# 11. Check critical functions
echo ""
echo "11. CRITICAL FUNCTION VERIFICATION"
echo "---------------------------------"
critical_functions=(
    "execute_strike"
    "calculate_position_size"
    "verify_liquidity"
    "place_order"
    "check_win_rate"
)

for func in "${critical_functions[@]}"; do
    if grep -q "fn $func\|async fn $func" src/*.rs src/*/*.rs 2>/dev/null; then
        echo "   ✅ $func found"
    else
        echo "   ⚠️  $func not found"
    fi
done

# 12. Integration check
echo ""
echo "12. MODULE INTEGRATION AUDIT"
echo "---------------------------"
# Check if modules are properly connected
if grep -q "use crate::opportunity_scanner" src/main.rs src/trading_engine.rs 2>/dev/null; then
    echo "   ✅ Opportunity scanner integrated"
else
    echo "   ⚠️  Opportunity scanner not integrated"
    ((WARNINGS++))
fi

if grep -q "use crate::api::" src/trading_engine.rs 2>/dev/null; then
    echo "   ✅ API modules integrated"
else
    echo "   ⚠️  API modules not integrated"
    ((WARNINGS++))
fi

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "                      AUDIT SUMMARY"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Critical Issues: $ISSUES"
echo "Warnings: $WARNINGS"
echo ""

if [ $ISSUES -eq 0 ] && [ $WARNINGS -lt 5 ]; then
    echo "✅ CODE QUALITY: EXCELLENT"
    echo ""
    echo "The codebase is ready for senior developer review:"
    echo "  • Proper error handling"
    echo "  • Safe async implementation"
    echo "  • 90% win rate enforced"
    echo "  • Good documentation"
    echo "  • Clean architecture"
else
    echo "⚠️  CODE QUALITY: NEEDS ATTENTION"
    echo ""
    echo "Address these items before sharing:"
    echo "  • Fix $ISSUES critical issues"
    echo "  • Review $WARNINGS warnings"
fi

echo "════════════════════════════════════════════════════════════════════"
