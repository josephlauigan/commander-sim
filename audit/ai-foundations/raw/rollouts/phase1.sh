#!/usr/bin/env bash
# phase 1: the current eval (switch off) and a setting, Tier 6 and Tier 3 at 1,000 games, diagnostic and timing runs
# phase1.sh TAG "ENV..." ; one run at a time
D=$(cd "$(dirname "$0")" && pwd)
tag=$1; envs=$2
for t in t6 t3; do
  "$D/run.sh" "$tag-$t-diag" "X=1 $envs AI_EVAL_FALLBACK=1" --realism --pool $t --games 1000 --jobs 22 --quiet
  "$D/run.sh" "$tag-$t" "X=1 $envs" --realism --pool $t --games 1000 --jobs 22 --quiet
done
echo "PHASE1 $tag done" >> "$D/progress.log"
