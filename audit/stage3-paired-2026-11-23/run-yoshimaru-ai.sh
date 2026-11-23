#!/usr/bin/env bash
# Yoshimaru AI package: my own full test run, then yoshimaru --analyze over its tiers (t4, t5),
# same seed 500000 / 1500 games per tier / loose as the main baseline.
WT=/home/joseph/projects/commander-sim/.claude/worktrees/yoshimaru-ai
OUT=/home/joseph/projects/commander-sim/.claude/worktrees/stage3/audit/stage3-paired-2026-11-23
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
cd "$WT/rust" || exit 1
git rev-parse --short HEAD > "$OUT/yoshimaru-ai-commit.txt"
flock /tmp/cargo-heavy.lock $CAP cargo test --release > "$OUT/yoshimaru-ai-tests.log" 2>&1
echo "tests exit $?" >> "$OUT/yoshimaru-ai-status.txt"
grep -E "^test result" "$OUT/yoshimaru-ai-tests.log" | awk '{p+=$4; f+=$6; i+=$8; n++} END {print n" binaries, passed "p" failed "f" ignored "i}' >> "$OUT/yoshimaru-ai-status.txt"
flock /tmp/cargo-heavy.lock $CAP cargo build --release -p sim-cli > "$OUT/yoshimaru-ai-build.log" 2>&1 || { echo "BUILD FAILED" >> "$OUT/yoshimaru-ai-status.txt"; exit 1; }
echo "start yoshimaru $(date +%T)" >> "$OUT/yoshimaru-ai-status.txt"
flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck yoshimaru --pool all --analyze > "$OUT/yoshimaru-ai-yoshimaru-analyze.txt" 2>&1
echo "done yoshimaru exit $? $(date +%T)" >> "$OUT/yoshimaru-ai-status.txt"
echo "YOSHIMARU-AI ALL DONE" >> "$OUT/yoshimaru-ai-status.txt"
