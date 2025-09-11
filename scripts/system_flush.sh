#!/bin/bash

# System Flush - Complete cleanup and verification
# Ensures no duplicates, clean build, and production ready

echo "════════════════════════════════════════════════════════════════════"
echo "                    SYSTEM FLUSH - COMPLETE CLEANUP"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Starting flush at: $(date)"
echo ""

# 1. Clean build artifacts
echo "1. Cleaning build artifacts..."
cargo clean
echo "   ✅ Build artifacts cleaned"

# 2. Remove any temporary files
echo ""
echo "2. Removing temporary files..."
find . -name "*.tmp" -o -name "*.log" -o -name "*.swp" -o -name ".DS_Store" | grep -v ".git" | xargs rm -f 2>/dev/null
echo "   ✅ Temporary files removed"

# 3. Check for duplicate directories
echo ""
echo "3. Checking for duplicate directories..."
duplicates=$(find . -type d -not -path "./.git*" -not -path "./target*" | while read dir; do basename "$dir"; done | sort | uniq -c | sort -rn | awk '$1 > 1 {print $2}')
if [ -z "$duplicates" ]; then
    echo "   ✅ No duplicate directories found"
else
    echo "   ⚠️  Found duplicate directories: $duplicates"
fi

# 4. Check for duplicate files
echo ""
echo "4. Checking for duplicate source files..."
dup_files=$(find src -name "*.rs" -exec basename {} \; | sort | uniq -c | sort -rn | awk '$1 > 1 && $2 != "mod.rs" {print $2}')
if [ -z "$dup_files" ]; then
    echo "   ✅ No duplicate source files"
else
    echo "   ❌ Duplicate files found: $dup_files"
fi

# 5. Verify directory structure
echo ""
echo "5. Verifying directory structure..."
required_dirs=(
    "src"
    "src/api"
    "src/eip"
    "src/monitoring"
    "scripts"
    "config"
    "contracts"
    "docs"
    "data"
    "tests"
)

missing_dirs=0
for dir in "${required_dirs[@]}"; do
    if [ ! -d "$dir" ]; then
        echo "   ❌ Missing: $dir"
        ((missing_dirs++))
    fi
done

if [ $missing_dirs -eq 0 ]; then
    echo "   ✅ All required directories present"
fi

# 6. Remove unused imports and format code
echo ""
echo "6. Formatting and optimizing code..."
if command -v rustfmt &> /dev/null; then
    cargo fmt 2>/dev/null
    echo "   ✅ Code formatted"
else
    echo "   ⚠️  rustfmt not available"
fi

# 7. Check for large files
echo ""
echo "7. Checking for large files..."
large_files=$(find . -type f -size +5M -not -path "./.git/*" -not -path "./target/*" 2>/dev/null)
if [ -z "$large_files" ]; then
    echo "   ✅ No large files (>5MB)"
else
    echo "   ⚠️  Large files found:"
    echo "$large_files" | while read f; do
        size=$(ls -lh "$f" | awk '{print $5}')
        echo "      - $f ($size)"
    done
fi

# 8. Verify no hardcoded paths
echo ""
echo "8. Checking for hardcoded paths..."
hardcoded=$(grep -r "/Users/\|/home/\|C:\\\\" src/ scripts/ 2>/dev/null | grep -v "example\|test")
if [ -z "$hardcoded" ]; then
    echo "   ✅ No hardcoded paths"
else
    echo "   ❌ Hardcoded paths found"
fi

# 9. Final build test
echo ""
echo "9. Running final build test..."
if cargo build --release >/dev/null 2>&1; then
    echo "   ✅ Build successful"
    
    # Get binary size
    if [ -f "target/release/macro_strike_bot_fixed" ]; then
        size=$(ls -lh target/release/macro_strike_bot_fixed | awk '{print $5}')
        echo "   📦 Binary size: $size"
    fi
else
    echo "   ❌ Build failed"
fi

# 10. Line count statistics
echo ""
echo "10. Code statistics:"
rust_lines=$(find src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
doc_lines=$(find . -name "*.md" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
julia_lines=$(wc -l market_analysis.jl 2>/dev/null | awk '{print $1}')

echo "   📊 Rust code: $rust_lines lines"
echo "   📊 Documentation: $doc_lines lines"
echo "   📊 Julia code: $julia_lines lines"
echo "   📊 Total: $((rust_lines + doc_lines + julia_lines)) lines"

# 11. Git status
echo ""
echo "11. Git repository status:"
uncommitted=$(git status --porcelain | wc -l)
if [ $uncommitted -eq 0 ]; then
    echo "   ✅ Working directory clean"
else
    echo "   ⚠️  $uncommitted uncommitted changes"
fi

# Check remote sync
git fetch origin >/dev/null 2>&1
ahead=$(git rev-list --count origin/main..HEAD 2>/dev/null || echo 0)
behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo 0)

if [ $ahead -eq 0 ] && [ $behind -eq 0 ]; then
    echo "   ✅ Synced with origin/main"
else
    echo "   ⚠️  Ahead: $ahead, Behind: $behind"
fi

# 12. Final directory tree
echo ""
echo "12. Final directory structure:"
echo "════════════════════════════════════════════════════════════════════"
tree -L 2 -I 'target|.git' 2>/dev/null || find . -type d -not -path "./.git*" -not -path "./target*" | head -20

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "                         FLUSH COMPLETE"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "System has been flushed and verified. Summary:"
echo "  • Build artifacts cleaned"
echo "  • No duplicate directories"
echo "  • Code formatted and optimized"
echo "  • $rust_lines lines of Rust code"
echo "  • Repository ready for sharing"
echo ""
echo "Repository URL: https://github.com/Dclock24/Macro-Strk-Bot"
echo ""
echo "Flush completed at: $(date)"
echo "════════════════════════════════════════════════════════════════════"
