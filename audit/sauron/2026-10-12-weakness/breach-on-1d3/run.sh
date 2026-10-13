#!/bin/bash
# Breach-slot screens on top of the 1D+3 list (branch sauron-1d3, whose Sauron list is the TEST BASELINE with the
# 1D+3 package): 500 paired games per list per tier, Tiers 1-5, look-ahead, loose. 18 threads (an agent builds alongside).
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/breach-on-1d3
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-da4c579}
declare -A SW
SW[B1]='Underworld Breach=>Exsanguinate|Brain Freeze=>Torment of Hailfire|Dark Ritual=>Fellwar Stone|Lotus Petal=>Dimir Signet'
SW[B2]='Underworld Breach=>Fellwar Stone|Brain Freeze=>Dimir Signet|Dark Ritual=>Izzet Signet|Lotus Petal=>Exsanguinate'
for k in ${TESTS:-B1 B2}; do
  args=(); IFS='|' read -ra parts <<< "${SW[$k]}"; for s in "${parts[@]}"; do args+=(--swap "$s"); done
  for t in t1 t2 t3 t4 t5; do
    out=$D/$k-$t.txt
    grep -q "delta" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      "$BIN" --deck sauron --pool $t --profile loose --games 500 --ai lookahead --jobs 18 --quiet "${args[@]}" > "$out" 2>&1
    echo "$(date +%H:%M) $k $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
  done
  echo "$(date +%H:%M) $k DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/$k-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
done
echo "$(date +%H:%M) ALL DONE" | tee -a $D/log.txt
