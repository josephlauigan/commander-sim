#!/bin/bash
# run_final.sh <final list file> <out dir> <games>: in the disposable bands-final worktree, apply each deck's final swaps
# (never committed), re-export, and measure the list alone (both profiles) on its band tiers.
L=$1; OUT=$2; N=$3
W=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/bands-final
B=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/soul-stone/rust/target/release/commander-sim
A=/tmp/claude-1000/-home-joseph-projects-mtg-sims/457f1396-d572-4d7d-a64f-590642808818/scratchpad/apply_swaps.sh
mkdir -p $OUT; cd $W || exit 1
git checkout -q -- . 
while IFS='|' read -r name deck swaps; do
  [ -z "$name" ] && continue
  [ -n "$swaps" ] && { $A "$deck" "$swaps" >> $OUT/apply.log 2>&1 || { echo "$(date +%H:%M) APPLY FAILED $name" >> $OUT/log.txt; continue; }; }
done < $L
nice -n 19 /home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/soul-stone/rust/target/release/cardgen export >> $OUT/apply.log 2>&1
git diff --stat >> $OUT/apply.log
while IFS='|' read -r name deck swaps; do
  [ -z "$name" ] && continue
  echo "$(date +%H:%M) start $name" >> $OUT/log.txt
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 nice -n 19 $B --deck "$deck" --pool all --games "$N" --jobs 22 > $OUT/$name.txt 2>&1
  echo "$(date +%H:%M) done $name exit $?" >> $OUT/log.txt
done < $L
echo "$(date +%H:%M) QUEUE DONE" >> $OUT/log.txt
