#!/bin/bash
# Bristly Bill: Evolving Wilds, Terramorphic Expanse and Fabled Passage -> 3 Forest (2026-10-11). The list has the
# fetch lands; the variant has three more Forests, so the delta is Forests minus fetches. Look-ahead AI, loose
# profile, 1,000 paired games per tier.
cd "$(dirname "$0")/../../.."
D=audit/bill/2026-10-11-fetches-forests
for t in t1 t2 t3 t4 t5; do
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck bill --pool $t --profile loose --games 1000 --ai lookahead \
    --jobs 22 --brief --swap "Evolving Wilds=>Forest" --swap "Terramorphic Expanse=>Forest" \
    --swap "Fabled Passage=>Forest" 2>&1 | grep -v progress > $D/$t.txt
  echo "$t exit ${PIPESTATUS[0]}"
done
