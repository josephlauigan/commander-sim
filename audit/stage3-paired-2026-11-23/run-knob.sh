#!/usr/bin/env bash
# protect_before_go knob (on for yoshimaru): my own full test run, then yoshimaru --analyze over its tiers (t4, t5),
# same seed 500000 / 1500 games per tier / loose as the main baseline.
WT=/home/joseph/projects/commander-sim/.claude/worktrees/stage4-knob
OUT=/home/joseph/projects/commander-sim/.claude/worktrees/stage3/audit/stage3-paired-2026-11-23
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
cd "$WT/rust" || exit 1
git rev-parse --short HEAD > "$OUT/knob-commit.txt"
flock /tmp/cargo-heavy.lock $CAP cargo test --release > "$OUT/knob-tests.log" 2>&1
echo "tests exit $?" >> "$OUT/knob-status.txt"
grep -E "^test result" "$OUT/knob-tests.log" | awk '{p+=$4; f+=$6; i+=$8; n++} END {print n" binaries, passed "p" failed "f" ignored "i}' >> "$OUT/knob-status.txt"
flock /tmp/cargo-heavy.lock $CAP cargo build --release -p sim-cli > "$OUT/knob-build.log" 2>&1 || { echo "BUILD FAILED" >> "$OUT/knob-status.txt"; exit 1; }
echo "start yoshimaru $(date +%T)" >> "$OUT/knob-status.txt"
flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck yoshimaru --pool all --analyze > "$OUT/knob-yoshimaru-analyze.txt" 2>&1
echo "done yoshimaru exit $? $(date +%T)" >> "$OUT/knob-status.txt"
echo "KNOB ALL DONE" >> "$OUT/knob-status.txt"
