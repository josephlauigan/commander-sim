#!/bin/bash
# run_queue.sh <queue file> <out dir> <games> [extra args]: one paired swap test (or a plain run) per line, in order
Q=$1; OUT=$2; N=$3; shift 3
ROOT=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/soul-stone
B=$ROOT/rust/target/release/commander-sim
mkdir -p $OUT
while IFS='|' read -r name deck swaps; do
  [ -z "$name" ] && continue
  args=(--deck "$deck" --pool all --games "$N" --jobs 22)
  if [ -n "$swaps" ]; then IFS=';' read -ra sw <<< "$swaps"; for s in "${sw[@]}"; do args+=(--swap "$s"); done; fi
  echo "$(date +%H:%M) start $name" >> $OUT/log.txt
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 nice -n 19 $B "${args[@]}" "$@" > $OUT/$name.txt 2>&1
  echo "$(date +%H:%M) done $name exit $?" >> $OUT/log.txt
done < $Q
echo "$(date +%H:%M) QUEUE DONE" >> $OUT/log.txt
