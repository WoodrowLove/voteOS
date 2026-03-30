#!/usr/bin/env bash
# VoteOS Build Status Check
# Run periodically to monitor agent progress

echo "=== VoteOS Build Status ==="
echo "Time: $(date)"
echo ""

# Check SESSION_STATE
if [ -f SESSION_STATE.md ]; then
    echo "--- Session State ---"
    head -20 SESSION_STATE.md
    echo ""
fi

# Check build
echo "--- Build Status ---"
cargo check 2>&1 | tail -3
echo ""

# Check tests
echo "--- Test Status ---"
cargo test 2>&1 | grep "test result:" | tail -10
echo ""

# Check git status
echo "--- Git Status ---"
git log --oneline -5
echo ""
git diff --stat | tail -5
echo ""

# Check module files
echo "--- Module Files ---"
ls -la src/domain/*.rs 2>/dev/null || echo "No domain files yet"
ls -la src/workflows/*.rs 2>/dev/null || echo "No workflow files yet"
echo ""

# Check capability count
echo "--- Capabilities Implemented ---"
grep -r "pub async fn execute" src/workflows/ 2>/dev/null | wc -l
echo "workflow functions found"
