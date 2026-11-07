#!/bin/bash
# Final confirmation (2026-10-13): Sauron's current list vs the proposed list (1D + 3 + B1, twelve swaps), 1,000
# paired games per list per tier, Tiers 1-5, look-ahead, loose, 22 threads. Waits for the Kefka K2 screen.
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/final
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-da4c579}
while pgrep -f "swaps/run-k2[.]sh" >/dev/null; do sleep 20; done
args=(--swap "Toxic Deluge=>Ral and the Implicit Maze" --swap "Blasphemous Act=>Massacre Wurm"
      --swap "Chaos Warp=>Chandra's Ignition" --swap "Spell Pierce=>Inferno Titan"
      --swap "Deepglow Skate=>Talisman of Indulgence" --swap "Conqueror's Flail=>Rakdos Signet"
      --swap "Sword of Fire and Ice=>Dreadhorde Invasion" --swap "Flux Channeler=>Lazotep Reaver"
      --swap "Underworld Breach=>Exsanguinate" --swap "Brain Freeze=>Torment of Hailfire"
      --swap "Dark Ritual=>Fellwar Stone" --swap "Lotus Petal=>Dimir Signet")
for t in t1 t2 t3 t4 t5; do
  out=$D/final-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    "$BIN" --deck sauron --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --quiet "${args[@]}" > "$out" 2>&1
  echo "$(date +%H:%M) final $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) FINAL DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/final-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
