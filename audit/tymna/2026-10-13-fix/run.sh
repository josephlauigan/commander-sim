#!/bin/bash
# Tymna partner comparison, rerun after the tymna-fix AI pass (2026-10-13): thrasios vs kraum, Tiers 1-5, 1,000 games per deck per tier, look-ahead AI,
# loose profile, the same seed for both decks in each tier, --analyze for the Tymna counters. JD allowed the whole
# machine: --jobs 22, nice 19. Resumable: a finished cell is skipped. Binary: a snapshot, so builds can't touch it.
cd "$(dirname "$0")/../../.."
D=audit/tymna/2026-10-13-fix
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad/csim-tymna2}
for t in t1 t2 t3 t4 t5; do
  for d in thrasios kraum; do
    out=$D/$d-$t.txt
    grep -q "total" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      "$BIN" --deck $d --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --seed 1013 --analyze > "$out" 2>&1
    echo "$(date +%H:%M) $d $t exit $?" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) DONE" | tee -a $D/log.txt
