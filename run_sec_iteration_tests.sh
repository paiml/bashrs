#!/bin/bash
# SEC Batch Iteration Tests - Run after all baselines complete
# Following EXTREME TDD - GREEN phase (tests already written)

set -e

echo "=== SEC Batch Iteration Tests ==="
echo "Running iteration tests for SEC002, SEC004-SEC008"
echo "All 37 mutation coverage tests have been pre-written"
echo ""

# SEC002: 8 mutation tests added (75.0% → 90%+ target)
echo "[1/6] SEC002 Iteration Test (32 mutants expected)"
cargo mutants --file rash/src/linter/rules/sec002.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec002_iter1.log
echo ""

# SEC004: 7 mutation tests added (baseline TBD → 90%+ target)
echo "[2/6] SEC004 Iteration Test (26 mutants expected)"
cargo mutants --file rash/src/linter/rules/sec004.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec004_iter1.log
echo ""

# SEC005: 5 mutation tests added (73.1% → 90%+ target)
echo "[3/6] SEC005 Iteration Test (26 mutants expected)"
cargo mutants --file rash/src/linter/rules/sec005.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec005_iter1.log
echo ""

# SEC006: 4 mutation tests added (baseline TBD → 90%+ target)
echo "[4/6] SEC006 Iteration Test (14 mutants expected)"
cargo mutants --file rash/src/linter/rules/sec006.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec006_iter1.log
echo ""

# SEC007: 4 mutation tests added (baseline TBD → 90%+ target)
echo "[5/6] SEC007 Iteration Test (9 mutants expected)"
cargo mutants --file rash/src/linter/rules/sec007.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec007_iter1.log
echo ""

# SEC008: 5 mutation tests added (baseline TBD → 90%+ target)
echo "[6/6] SEC008 Iteration Test (mutant count TBD)"
cargo mutants --file rash/src/linter/rules/sec008.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec008_iter1.log
echo ""

echo "=== All SEC Iteration Tests Complete ==="
echo "Analyze results and calculate final kill rates"
