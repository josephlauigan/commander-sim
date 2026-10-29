#!/usr/bin/env bash
# phase 2: the kept setting (the defaults: AI_EVAL_ADAPT on, cap 24, batch 6, stop 0.5), the full-eval set
D=$(cd "$(dirname "$0")" && pwd)
"$D/run.sh" full-goldfish "X=1" --goldfish t6,scion --games 2000 --seed 500000 --jobs 22 --quiet
"$D/run.sh" full-realism-t6 "X=1" --realism --pool t6 --games 4000 --jobs 22 --quiet
for t in t0 t1 t2 t3 t4 t5; do
  "$D/run.sh" full-realism-$t "X=1" --realism --pool $t --games 4000 --jobs 22 --quiet
done
"$D/run.sh" full-all-decks "X=1" --all-decks --pool all --games 1000 --jobs 22 --quiet
echo "PHASE2 done" >> "$D/progress.log"
