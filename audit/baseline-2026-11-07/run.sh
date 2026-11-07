#!/usr/bin/env bash
# Baseline 2026-11-07 (JD, option A): every deck on its own tiers, 10,000 games per tier, then --analyze at 2,000 per
# tier. Look-ahead AI, loose profile (CLI defaults), seed 500000 (same as Baseline B 2026-11-06). Each run under the
# memory cap; a failed run is logged and the queue goes on. Usage: bash audit/baseline-2026-11-07/run.sh
cd /home/joseph/projects/commander-sim
OUT=audit/baseline-2026-11-07
BIN=rust/target/release/commander-sim
git rev-parse --short HEAD > $OUT/commit.txt
for d in thanos yshtola thrasios sheoldred aragorn cloud; do
  start=$(date +%s)
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    $BIN --deck $d --pool all --games 10000 --seed 500000 > $OUT/$d.txt 2>&1 < /dev/null
  echo "baseline $d exit=$? $(( ($(date +%s)-start)/60 ))m $(date +%H:%M)" | tee -a $OUT/status.txt
done
for d in thanos yshtola thrasios sheoldred aragorn cloud; do
  start=$(date +%s)
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    $BIN --deck $d --pool all --games 2000 --seed 500000 --analyze > $OUT/$d-analyze.txt 2>&1 < /dev/null
  echo "analyze $d exit=$? $(( ($(date +%s)-start)/60 ))m $(date +%H:%M)" | tee -a $OUT/status.txt
done
echo "ALL DONE $(date +%H:%M)" | tee -a $OUT/status.txt
