#!/bin/bash
# waits for the Thrasios swap queue, then runs the mulligan oracle
OUT=/tmp/claude-1000/-home-joseph-projects-commander-sim/3b6ecf94-b461-4a20-b513-cde99742cbcc/scratchpad/c4
W=/home/joseph/projects/commander-sim/.claude/worktrees/agent-aab7220aec7e5dc63
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
while kill -0 2059206 2>/dev/null; do sleep 30; done
echo "thrasios queue finished $(date +%T)"
cd $W/rust && $CAP cargo build --release -p sim-cli > $OUT/build.txt 2>&1 || { echo "BUILD FAILED"; exit 1; }
cd $W
for k in thrasios t6 yshtola thanos aragorn sheoldred cloud; do
  s=$(date +%s)
  $CAP rust/target/release/commander-sim --mulligan-oracle $k --hands 1000 --playouts 40 --seed 500000 > $OUT/$k.txt 2>&1
  echo "$k exit $? $(( $(date +%s) - s ))s"
done
echo "C4 ALL DONE"
