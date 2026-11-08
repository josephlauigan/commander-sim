#!/bin/bash
set -u
cd /home/joseph/projects/commander-sim
OUT=/tmp/claude-1000/-home-joseph-projects-commander-sim/3b6ecf94-b461-4a20-b513-cde99742cbcc/scratchpad/thrasios
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
(cd rust && $CAP cargo build --release -p sim-cli) > $OUT/build.txt 2>&1 || { echo "BUILD FAILED"; exit 1; }
BIN=rust/target/release/commander-sim
$CAP $BIN --deck thrasios --pool t6 --games 5000 --seed 500000 \
  --swap "Savannah=>Watery Grave" > $OUT/s1.txt 2>&1; echo "S1 exit $?"
$CAP $BIN --deck thrasios --pool t6 --games 5000 --seed 500000 \
  --swap "Savannah=>Watery Grave" --swap "Breeding Pool=>Swamp" > $OUT/s2.txt 2>&1; echo "S2 exit $?"
echo "ALL DONE"
