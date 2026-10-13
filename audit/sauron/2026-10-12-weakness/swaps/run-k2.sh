#!/bin/bash
# Kefka K2 (2026-10-13): the Breach package (2K) plus assembly and storm help: Grapeshot (bulk, a second storm card),
# Gamble (finds Breach), Jeska's Will (mana and storm). 500 paired games per tier, Tiers 1-5, look-ahead, loose, 18
# threads. Waits for the Breach-slot screens on the 1D+3 list to finish.
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/swaps
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-da4c579}
while pgrep -f "breach-on-1d3/run[.]sh" >/dev/null; do sleep 20; done
args=(--swap "Duress=>Brain Freeze" --swap "Dark Inquiry=>Dark Ritual" --swap "Potioner's Trove=>Lotus Petal"
      --swap "Ichor Slick=>Grapeshot" --swap "Thought Vessel=>Gamble" --swap "Chaos Warp=>Jeska's Will")
for t in t1 t2 t3 t4 t5; do
  out=$D/K2-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    "$BIN" --deck kefka --pool $t --profile loose --games 500 --ai lookahead --jobs 18 --quiet "${args[@]}" > "$out" 2>&1
  echo "$(date +%H:%M) K2 $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) K2 DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/K2-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
