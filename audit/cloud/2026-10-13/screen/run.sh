#!/bin/bash
# Cloud revisit screen (2026-10-13): donor tests D1-D3 (what each deck loses by giving Cloud a card), then C3 (Cloud
# with SoFI + ramp + SoFF + SoHH + Champion's Helm + Smothering Tithe). Paired, 500 games per list per tier, Tiers 1-5,
# look-ahead AI, loose profile, 8 threads at nice 19 (shared machine). Resumable: a finished tier is skipped.
# Usage: run.sh [TEST ...]   (default D1 D2 D3)
cd "$(dirname "$0")/../../../.."
D=audit/cloud/2026-10-13/screen
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/bdfb9e18-5da6-4d07-ab18-c71ffb5f5d3f/scratchpad/csim-base}
declare -A DECK SW
DECK[D1]=sauron;    SW[D1]='Sword of Feast and Famine=>Chaos Warp|Sword of Hearth and Home=>Spell Pierce'
DECK[D2]=seph;      SW[D2]='Smothering Tithe=>Momentary Blink'
DECK[D3]=sheoldred; SW[D3]="Champion's Helm=>Carnage Altar"
for x in c3 c4; do X=${x^^}; [ -f $D/$x.swaps ] && { DECK[$X]=cloud; SW[$X]=$(paste -sd"|" $D/$x.swaps); }; done
tests=("$@"); [ ${#tests[@]} -eq 0 ] && tests=(D1 D2 D3)
for k in "${tests[@]}"; do
  args=(); IFS='|' read -ra parts <<< "${SW[$k]}"; for s in "${parts[@]}"; do args+=(--swap "$s"); done
  for t in t1 t2 t3 t4 t5; do
    out=$D/$k-$t.txt
    grep -q "delta" "$out" 2>/dev/null && continue
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      "$BIN" --deck "${DECK[$k]}" --pool $t --profile loose --games 500 --ai lookahead --jobs 8 --quiet "${args[@]}" > "$out" 2>&1
    echo "$(date +%H:%M) $k $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) DONE ${tests[*]}" | tee -a $D/log.txt
