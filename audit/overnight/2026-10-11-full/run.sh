#!/bin/bash
# Overnight 2026-10-11: all nine of JD's decks against the five tiers on the same code (overnight-models),
# look-ahead AI, loose profile, 960 games per cell, seeds 500000-500959 (the standard of the 10-07 run).
cd "$(dirname "$0")/../../.."
D=audit/overnight/2026-10-11-full
for d in kefka zada seph veyran sauron yshtola galadriel alela jodah; do
  for t in t1 t2 t3 t4 t5; do
    [ -s $D/$d-$t.txt ] && grep -q "win " $D/$d-$t.txt && continue   # done already (lets the script resume)
    start=$(date +%s)
    systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      ./rust/target/release/commander-sim --deck $d --pool $t --profile loose --games 960 --seed 500000 \
      --ai lookahead --jobs 22 2>&1 | grep -v progress > $D/$d-$t.txt
    rc=${PIPESTATUS[0]}
    echo "$(date +%H:%M) $d $t exit $rc $(( $(date +%s) - start ))s $(grep -h -m1 'win ' $D/$d-$t.txt)" | tee -a $D/log.txt
  done
done
echo "ALL DONE $(date +%H:%M)" | tee -a $D/log.txt
