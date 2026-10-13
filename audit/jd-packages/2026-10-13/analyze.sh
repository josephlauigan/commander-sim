#!/bin/bash
# Baseline analyses for JD's four package tests (2026-10-13): Kefka, Veyran, Sauron, Titania on the current lists,
# Tiers 1/3/5, 320 games, look-ahead AI, loose profile. 4 threads at nice 19 (shared machine: 7 per session, see
# ~/projects/mtg_sims/cpu-ledger.md). Resumable: a finished file is skipped.
cd "$(dirname "$0")/../../.."
D=audit/jd-packages/2026-10-13
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad/csim-pkg}
for deck in kefka veyran sauron titania; do
  for t in t1 t3 t5; do
    out=$D/analyze-$deck-$t.txt
    grep -q "Card report" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
      "$BIN" --deck $deck --pool $t --profile loose --games 320 --ai lookahead --jobs 4 --analyze > "$out" 2>&1
    echo "$(date +%H:%M) $deck $t exit $? $(grep -m1 'Win rate' "$out")" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) DONE" | tee -a $D/log.txt
