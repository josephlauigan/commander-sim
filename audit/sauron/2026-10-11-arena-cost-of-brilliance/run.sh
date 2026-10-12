#!/bin/bash
# Sauron: Phyrexian Arena -> Cost of Brilliance (bulk) (2026-10-11), to see what Sauron loses if his Arena goes to the
# Sheoldred deck instead of buying her one. The list (with Bedevil in Sheoldred's old slot) has Arena; the variant has
# Cost of Brilliance, a draw-two that also puts a +1/+1 counter on his best creature (usually the Army). Look-ahead AI,
# loose profile, 1,000 paired games per tier (same seeds, seats and opponents' draws).
cd "$(dirname "$0")/../../.."
D=audit/sauron/2026-10-11-arena-cost-of-brilliance
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck sauron --pool $t --profile loose --games 1000 --ai lookahead \
    --jobs 22 --brief --swap "Phyrexian Arena=>Cost of Brilliance" 2>&1 | grep -v progress > $D/$t.txt
  echo "$t exit ${PIPESTATUS[0]} $(grep -m1 'delta' $D/$t.txt)" | tee -a $D/log.txt
done
