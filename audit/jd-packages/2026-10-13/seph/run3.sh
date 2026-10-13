#!/bin/bash
# Sephiroth + Phyrexian Altar + Phyrexian Tower + Sephiroth, Fabled SOLDIER (for Flickering Hound) (JD owns both, unused by the Sheoldred test), 2026-10-13: paired,
# 500 games per list per tier, Tiers 1-5, look-ahead AI, loose profile, nice 19. Binary: jd-packages 8c71f7f
# (includes PR #61's Tower fix and sacrifice-outlet AI). Resumable.
cd "$(dirname "$0")/../../../.."
D=audit/jd-packages/2026-10-13/seph
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad/csim-pkg3}
for t in t1 t2 t3 t4 t5; do
  out=$D/pkg3-seph-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
    "$BIN" --deck seph --pool $t --profile loose --games 500 --ai lookahead --jobs ${JOBS:-22} --quiet \
    --swap "Swamp=>Phyrexian Tower" --swap "Mind Stone=>Phyrexian Altar" --swap "Flickering Hound=>Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel" > "$out" 2>&1
  echo "$(date +%H:%M) seph $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) DONE seph3" | tee -a $D/log.txt
