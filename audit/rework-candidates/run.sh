#!/bin/bash
# Zada rework candidates against Tier 1 (2026-10-11): look-ahead AI, loose profile, 960 games, seeds 500000-500959,
# all on the same code. Zada's own Tier 1 figure (5.2%) is in audit/overnight/2026-10-11-full/.
cd "$(dirname "$0")/../.."
D=audit/rework-candidates
for d in torbran glissa jenova; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck $d --pool t1 --profile loose --games 960 --seed 500000 \
    --ai lookahead --jobs 22 2>&1 | grep -v progress > $D/$d-t1.txt
  echo "$d t1 exit ${PIPESTATUS[0]} $(grep -m1 '^Win rate' $D/$d-t1.txt)" | tee -a $D/log.txt
done
