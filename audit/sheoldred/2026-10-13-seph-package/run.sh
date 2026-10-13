#!/bin/bash
# Sheoldred + Sephiroth package (2026-10-13), on the PR #60 baseline plus the sacrifice-outlet fixes (sheoldred-sac,
# 4bc8c1a): 1,000 paired games per list per tier, Tiers 1-5, look-ahead, loose, 7 threads (the session's share).
cd "$(dirname "$0")/../../.."
D=audit/sheoldred/2026-10-13-seph-package
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/c350433b-93cb-4e55-8509-d4568b87a0f8/scratchpad/commander-sim-4bc8c1a}
args=(--swap "Dark Deal=>Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel" --swap "Apex Witchstalker=>Phyrexian Altar"
      --swap "Masterful Flourish=>Carrion Feeder" --swap "Commander's Sphere=>Common Crook"
      --swap "Potioner's Trove=>Callous Inspector" --swap "Swamp=>Phyrexian Tower")
for t in t1 t2 t3 t4 t5; do
  out=$D/pkg-$t.txt
  grep -q "delta" "$out" 2>/dev/null && continue
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
    "$BIN" --deck sheoldred --pool $t --profile loose --games 1000 --ai lookahead --jobs 7 --quiet "${args[@]}" > "$out" 2>&1
  echo "$(date +%H:%M) pkg $t exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
done
echo "$(date +%H:%M) PKG DONE: $(for t in t1 t2 t3 t4 t5; do grep -m1 -o 'delta [-+0-9.]*' $D/pkg-$t.txt | awk '{print $2}'; done | paste -sd' ' | awk '{s=0; for(i=1;i<=NF;i++) s+=$i; printf "tiers %s avg %+.1f", $0, s/NF}')" | tee -a $D/log.txt
