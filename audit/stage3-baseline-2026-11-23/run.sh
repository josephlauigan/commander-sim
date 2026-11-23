#!/usr/bin/env bash
# Stage 3 baseline: --analyze, each deck over its own tiers, loose profile, default games/seed. Built from main f1ffd896.
cd "$(dirname "$0")/../../rust" || exit 1
OUT="$(cd ../audit/stage3-baseline-2026-11-23 && pwd)"
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
git rev-parse --short HEAD > "$OUT/commit.txt"
flock /tmp/cargo-heavy.lock $CAP cargo build --release -p sim-cli > "$OUT/build.log" 2>&1 || { echo "BUILD FAILED"; exit 1; }
for d in yshtola sheoldred kyodai urdragon yoshimaru; do
  echo "start $d $(date +%T)" >> "$OUT/status.txt"
  flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck $d --pool all --analyze > "$OUT/$d-analyze.txt" 2>&1
  echo "done $d exit $? $(date +%T)" >> "$OUT/status.txt"
done
echo "ALL DONE"
