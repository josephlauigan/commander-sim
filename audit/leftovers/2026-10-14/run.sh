#!/bin/bash
# Jodah-leftover tests (2026-10-14). Each argument is one job: "analyze:<deck>" (T1/T3/T5, 320 games, --analyze) or
# "test:<name>:<deck>:<swap>|<swap>..." (paired, 500 games per list per tier, T1-T5). Look-ahead AI, loose profile,
# nice 19. Threads per tier from the `jobs` file (kept by borrow.sh from ~/projects/mtg_sims/cpu-ledger.md).
# Resumable: finished tiers are skipped. Never edit this worktree's deck files while it runs.
cd "$(dirname "$0")/../../.."
D=audit/leftovers/2026-10-14
BIN=${BIN:-/tmp/claude-1000/-home-joseph-projects-mtg-sims/f5d987cb-18a2-4572-90f1-24faee8ab577/scratchpad/csim-left}
for job in "$@"; do
  IFS=':' read -r kind a b c <<< "$job"
  if [ "$kind" = analyze ]; then
    for t in t1 t3 t5; do
      out=$D/analyze-$a-$t.txt; grep -q "Card report" "$out" 2>/dev/null && continue
      J=$(cat $D/jobs)
      nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
        "$BIN" --deck $a --pool $t --profile loose --games 320 --ai lookahead --jobs $J --analyze > "$out" 2>&1
      echo "$(date +%H:%M) analyze $a $t jobs $J exit $? $(grep -m1 'Win rate' "$out")" | tee -a $D/log.txt
    done
  else
    args=(); IFS='|' read -ra parts <<< "$c"; for s in "${parts[@]}"; do args+=(--swap "$s"); done
    for t in t1 t2 t3 t4 t5; do
      out=$D/test-$a-$t.txt; grep -q "delta" "$out" 2>/dev/null && continue
      J=$(cat $D/jobs)
      nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 \
        "$BIN" --deck $b --pool $t --profile loose --games 500 --ai lookahead --jobs $J --quiet "${args[@]}" > "$out" 2>&1
      echo "$(date +%H:%M) test $a ($b) $t jobs $J exit $? $(grep -m1 'delta' "$out")" | tee -a $D/log.txt
    done
  fi
done
echo "$(date +%H:%M) QUEUE DONE: $*" | tee -a $D/log.txt
