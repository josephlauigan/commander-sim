#!/bin/bash
# Sauron: Kefka, Court Mage -> Bedevil (2026-10-11). The list has Bedevil; the variant puts Kefka back,
# so the delta is Kefka minus Bedevil. Look-ahead AI, loose profile, 1,000 paired games per tier.
cd "$(dirname "$0")/../../.."
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck sauron --pool $t --profile loose --games 1000 --ai lookahead \
    --jobs 22 --brief --swap "Bedevil=>Kefka, Court Mage // Kefka, Ruler of Ruin" 2>&1 | grep -v progress > audit/sauron/2026-10-11-kefka-bedevil/$t.txt
  echo "$t exit $?"
done
