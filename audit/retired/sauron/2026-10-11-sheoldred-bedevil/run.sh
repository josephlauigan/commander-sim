#!/bin/bash
# Sauron: Sheoldred, the Apocalypse -> Bedevil (2026-10-11), to see what Sauron loses if Sheoldred leads her own
# mono-black deck. The list has Sheoldred; the variant has Bedevil (binder) instead, so the delta is Bedevil minus
# Sheoldred. Look-ahead AI, loose profile, 1,000 paired games per tier (same seeds, seats and opponents' draws).
cd "$(dirname "$0")/../../.."
D=audit/sauron/2026-10-11-sheoldred-bedevil
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck sauron --pool $t --profile loose --games 1000 --ai lookahead \
    --jobs 22 --brief --swap "Sheoldred, the Apocalypse=>Bedevil" 2>&1 | grep -v progress > $D/$t.txt
  echo "$t exit ${PIPESTATUS[0]}"
done
