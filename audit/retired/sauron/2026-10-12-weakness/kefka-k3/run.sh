#!/bin/bash
# Kefka K3 (2026-10-13): Kefka (Twinflame list) vs Kefka + the Breach package, Grapeshot, two tutors (Grim Tutor,
# Diabolic Intent) and Jeska's Will (JD), 1,000
# paired games per list per tier, Tiers 1-5, look-ahead, loose, 22 threads; code 649adf3 (with the hold fix).
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/kefka-k3
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-649adf3}

args=(--swap "Duress=>Brain Freeze" --swap "Dark Inquiry=>Dark Ritual" --swap "Potioner's Trove=>Lotus Petal"
      --swap "Ichor Slick=>Grapeshot" --swap "Thought Vessel=>Grim Tutor" --swap "Chaos Warp=>Diabolic Intent"
      --swap "Postmortem Professor=>Jeska's Will")
for t in t1 t2 t3 t4 t5; do
  out=$D/k3-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    "$BIN" --deck kefka --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --quiet "${args[@]}" > "$out" 2>&1
  echo "$(date +%H:%M) K3 $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) K3 DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/k3-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
