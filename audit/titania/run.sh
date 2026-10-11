#!/bin/bash
# Titania (key titania, no-buy list) against every tier (2026-10-11): look-ahead AI, loose profile, 960 games per tier, seeds
# 500000-500959, the same settings as the 9-deck table in decklists/pool/pool-results.md section 0d.
cd "$(dirname "$0")/../.."
D=audit/titania
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck titania --pool $t --profile loose --games 960 --seed 500000 \
    --ai lookahead --jobs 22 2>&1 | grep -v progress > $D/titania-$t.txt
  echo "titania $t exit ${PIPESTATUS[0]} $(grep -m1 '^Win rate' $D/titania-$t.txt)" | tee -a $D/log.txt
done
