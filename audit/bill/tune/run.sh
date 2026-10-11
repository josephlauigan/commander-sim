#!/bin/bash
# Bill after the bill-tune AI fix (lethal doubling counts blockers; no "bigger attack" doubling), same list and
# settings as audit/bill/run.sh (15.9% before): 960 look-ahead games per tier, seeds 500000-500959.
cd "$(dirname "$0")/../../.."
D=audit/bill/tune
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck bill --pool $t --profile loose --games 960 --seed 500000 \
    --ai lookahead --jobs 22 2>&1 | grep -v progress > $D/bill-$t.txt
  echo "bill $t exit ${PIPESTATUS[0]} $(grep -m1 '^Win rate' $D/bill-$t.txt)" | tee -a $D/log.txt
done
