#!/usr/bin/env bash
# Spark Double hold: my own full test run, then paired sims against the main baseline (same seed 500000, 1500 games/tier, loose):
# urdragon (the deck it changes) and kyodai (no-leak check: output must match the baseline apart from timing lines).
WT=/home/joseph/projects/commander-sim/.claude/worktrees/spark-hold
OUT=/home/joseph/projects/commander-sim/.claude/worktrees/stage3/audit/stage3-paired-2026-11-23
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
cd "$WT/rust" || exit 1
git rev-parse --short HEAD > "$OUT/spark-commit.txt"
flock /tmp/cargo-heavy.lock $CAP cargo test --release > "$OUT/spark-tests.log" 2>&1
echo "tests exit $?" >> "$OUT/spark-status.txt"
grep -E "^test result" "$OUT/spark-tests.log" | awk '{p+=$4; f+=$6; i+=$8; n++} END {print n" binaries, passed "p" failed "f" ignored "i}' >> "$OUT/spark-status.txt"
flock /tmp/cargo-heavy.lock $CAP cargo build --release -p sim-cli > "$OUT/spark-build.log" 2>&1 || { echo "BUILD FAILED" >> "$OUT/spark-status.txt"; exit 1; }
for d in urdragon kyodai; do
  echo "start $d $(date +%T)" >> "$OUT/spark-status.txt"
  flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck $d --pool all --analyze > "$OUT/spark-$d-analyze.txt" 2>&1
  echo "done $d exit $? $(date +%T)" >> "$OUT/spark-status.txt"
done
echo "SPARK ALL DONE" >> "$OUT/spark-status.txt"
