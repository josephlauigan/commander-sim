#!/usr/bin/env bash
# Larger paired Yoshimaru run on a fresh seed range: 3000 games per tier, seed 700000, loose, both arms.
# Arm A = main with the Silence knob (stage4-protect-knob 223d437b: the Spark change only touches the urdragon seat);
# Arm B = yoshimaru-ai 93884b50 (the package, merged with main).
OUT=/home/joseph/projects/commander-sim/.claude/worktrees/stage3/audit/stage3-paired-2026-11-23
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
for arm in knob:stage4-knob yoshimaru-ai:yoshimaru-ai; do
  name=${arm%%:*}; wt=/home/joseph/projects/commander-sim/.claude/worktrees/${arm##*:}
  echo "start $name $(date +%T) $(git -C $wt rev-parse --short HEAD)" >> "$OUT/big-status.txt"
  (cd $wt/rust && flock /tmp/cargo-heavy.lock $CAP ./target/release/commander-sim --deck yoshimaru --pool all --analyze --games 3000 --seed 700000 > "$OUT/big-$name-yoshimaru-analyze.txt" 2>&1)
  echo "done $name exit $? $(date +%T)" >> "$OUT/big-status.txt"
done
echo "BIG ALL DONE" >> "$OUT/big-status.txt"
