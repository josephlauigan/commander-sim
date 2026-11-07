#!/bin/bash
# Comet Storm for Torment of Hailfire on Sauron's proposed 12-swap list (TEST BASELINE on branch sauron-1d3), 2026-10-13:
# 1,000 paired games per list per tier, Tiers 1-5, look-ahead, loose, 22 threads; code 9495a5c.
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/comet
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-9495a5c}
for t in t1 t2 t3 t4 t5; do
  out=$D/comet-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    "$BIN" --deck sauron --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --quiet --swap "Torment of Hailfire=>Comet Storm" > "$out" 2>&1
  echo "$(date +%H:%M) comet $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) COMET DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/comet-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
