#!/bin/bash
# Sephiroth baseline analysis before the Phyrexian Altar + Phyrexian Tower test (2026-10-13): T1/T3/T5, 320 games.
cd "$(dirname "$0")/../../../.."
D=audit/jd-packages/2026-10-13/seph
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad/csim-pkg3}
for t in t1 t3 t5; do
  out=$D/analyze-seph-$t.txt
  grep -q "Card report" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
    "$BIN" --deck seph --pool $t --profile loose --games 320 --ai lookahead --jobs ${JOBS:-12} --analyze > "$out" 2>&1
  echo "$(date +%H:%M) seph $t exit $? $(grep -m1 'Win rate' "$out")" | tee -a $D/log.txt
done
