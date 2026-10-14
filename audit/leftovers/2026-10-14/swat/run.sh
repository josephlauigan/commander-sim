#!/bin/bash
# Kefka Deflecting Swat fix (kefka-swat 5911749) vs base (leftovers-apply), same seeds, 1,000 games per tier, T1-T5,
# look-ahead, loose. Not paired by list (a code change): same seed range for both binaries.
cd "$(dirname "$0")/../../../.."
D=audit/leftovers/2026-10-14/swat
S=/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad
for t in t1 t2 t3 t4 t5; do
  for v in base fix; do
    out=$D/$v-$t.txt; grep -q "Win rate" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
      $S/csim-swat-$v --deck kefka --pool $t --profile loose --games 1000 --ai lookahead --jobs ${JOBS:-22} --analyze > "$out" 2>&1
    echo "$(date +%H:%M) $v $t $(grep -m1 'Win rate' "$out" | cut -c1-60) | swat $(grep -m1 'Deflecting Swat' "$out" | sed -E 's/ +/ /g' | cut -c1-80)" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) DONE" | tee -a $D/log.txt
