#!/bin/bash
# Sauron weakness screen (2026-10-12), queued test 1D: the combined one-sided control package (1A + 1C), 500 games per list per tier, Tiers 1-5, look-ahead
# AI, loose profile. Screening for big effects only (about ±2 points over all tiers). Resumable: a tier whose output
# already has its result line is skipped. Threads: 22, or 8 while another session builds or runs a sim (the load average would count our own last tier).
cd "$(dirname "$0")/../../../.."
D=audit/sauron/2026-10-12-weakness/swaps
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-da4c579}
declare -A DECK SW
DECK[1A]=sauron; SW[1A]='Toxic Deluge=>Ral and the Implicit Maze|Blasphemous Act=>Massacre Wurm'
DECK[1B]=sauron; SW[1B]='Toxic Deluge=>Widespread Brutality|Blasphemous Act=>Crippling Fear'
DECK[1C]=sauron; SW[1C]="Toxic Deluge=>Chandra's Ignition|Blasphemous Act=>Inferno Titan"
DECK[2A]=sauron; SW[2A]='Underworld Breach=>Exsanguinate|Brain Freeze=>Torment of Hailfire|Dark Ritual=>Talisman of Indulgence|Lotus Petal=>Rakdos Signet'
DECK[2B]=sauron; SW[2B]='Underworld Breach=>Grishnákh, Brash Instigator|Brain Freeze=>Inferno Titan|Dark Ritual=>Lazotep Reaver|Lotus Petal=>Grim Initiate'
DECK[2C]=sauron; SW[2C]='Underworld Breach=>Talisman of Indulgence|Brain Freeze=>Rakdos Signet|Dark Ritual=>Fellwar Stone|Lotus Petal=>Dreadhorde Invasion'
DECK[2K]=kefka;  SW[2K]="Duress=>Brain Freeze|Dark Inquiry=>Dark Ritual|Potioner's Trove=>Lotus Petal"
DECK[3]=sauron;  SW[3]="Deepglow Skate=>Talisman of Indulgence|Conqueror's Flail=>Rakdos Signet|Sword of Fire and Ice=>Dreadhorde Invasion|Flux Channeler=>Lazotep Reaver"
DECK[1D]=sauron; SW[1D]="Toxic Deluge=>Ral and the Implicit Maze|Blasphemous Act=>Massacre Wurm|Chaos Warp=>Chandra's Ignition|Spell Pierce=>Inferno Titan"
while pgrep -f "bash audit/sauron/2026-10-12-weakness/swaps/run[.]sh" >/dev/null; do sleep 30; done  # after the first eight
for k in 1D; do
  args=(); IFS='|' read -ra parts <<< "${SW[$k]}"; for s in "${parts[@]}"; do args+=(--swap "$s"); done
  for t in t1 t2 t3 t4 t5; do
    out=$D/$k-$t.txt
    grep -q "delta" "$out" 2>/dev/null && continue
    jobs=22; pgrep -x rustc >/dev/null && jobs=8; pgrep -f "target/release/commander-sim" >/dev/null && jobs=8  # other sessions' builds or sims
    nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
      "$BIN" --deck "${DECK[$k]}" --pool $t --profile loose --games 500 --ai lookahead --jobs $jobs --quiet "${args[@]}" > "$out" 2>&1
    echo "$(date +%H:%M) $k $t exit $? jobs $jobs $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
  done
done
echo "$(date +%H:%M) 1D DONE" | tee -a $D/log.txt
