#!/bin/bash
# Homes for Sauron's cut Sword of Fire and Ice, and a replacement for Veyran's Jeska's Will (which moves to Kefka),
# from the Sauron/Kefka cuts (2026-10-13). run-flux.sh: Flux Channeler only (JD), 1,000 paired games per list per tier, Tiers 1-5, look-ahead, loose, 22 threads.
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/sofi-veyran
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-649adf3}
declare -A DECK SW
DECK[S-cloud]=cloud;     SW[S-cloud]='Take Up the Shield=>Sword of Fire and Ice'
DECK[S-titania]=titania; SW[S-titania]='Silhana Starfletcher=>Sword of Fire and Ice'
DECK[S-alela]=alela;     SW[S-alela]='Page, Loose Leaf=>Sword of Fire and Ice'
DECK[V-breach]=veyran;   SW[V-breach]="Jeska's Will=>Underworld Breach"
DECK[V-flux]=veyran;     SW[V-flux]="Jeska's Will=>Flux Channeler"
DECK[V-trove]=veyran;    SW[V-trove]="Jeska's Will=>Potioner's Trove"
DECK[V-vessel]=veyran;   SW[V-vessel]="Jeska's Will=>Thought Vessel"
for k in V-flux; do
  for t in t1 t2 t3 t4 t5; do
    out=$D/$k-$t.txt
    grep -q "delta" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      "$BIN" --deck "${DECK[$k]}" --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --quiet --swap "${SW[$k]}" > "$out" 2>&1
    echo "$(date +%H:%M) $k $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
  done
  echo "$(date +%H:%M) $k DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/$k-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
done
echo "$(date +%H:%M) FLUX DONE" | tee -a $D/log.txt
