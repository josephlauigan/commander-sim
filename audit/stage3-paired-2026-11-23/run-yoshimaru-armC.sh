#!/usr/bin/env bash
# Arm C (veto-only, yoshimaru-veto cb9b3261): same fresh seed range as the big run: 3000 games per tier, seed 700000, loose.
OUT=/home/joseph/projects/commander-sim/.claude/worktrees/stage3/audit/stage3-paired-2026-11-23
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
wt=/home/joseph/projects/commander-sim/.claude/worktrees/yoshimaru-veto
cd $wt/rust || exit 1
flock /tmp/cargo-heavy.lock $CAP cargo build --release -p sim-cli > "$OUT/armC-build.log" 2>&1 || { echo "BUILD FAILED" >> "$OUT/big-status.txt"; exit 1; }
echo "start armC $(date +%T) $(git rev-parse --short HEAD)" >> "$OUT/big-status.txt"
flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck yoshimaru --pool all --analyze --games 3000 --seed 700000 > "$OUT/big-armC-yoshimaru-analyze.txt" 2>&1
echo "done armC exit $? $(date +%T)" >> "$OUT/big-status.txt"
echo "ARM C DONE" >> "$OUT/big-status.txt"
