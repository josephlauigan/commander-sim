#!/bin/bash
# Re-measure after the AI audit: each JD deck on its band tiers (data/my_decks.toml), 2,000 games per profile,
# look-ahead AI, new code (ai-audit); plus Titania on main's code (Fblthp list, no before number otherwise).
OUT=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ai-audit/audit/ai-audit/remeasure
W=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees
run() { echo "$(date +%H:%M) start $2" >> $OUT/log.txt
  (cd $W/$1 && systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 nice -n 19 rust/target/release/commander-sim --deck $3 --pool all --games 2000 --jobs 10 > $OUT/$2.txt 2>&1)
  echo "$(date +%H:%M) done $2 exit $?" >> $OUT/log.txt; }
for d in yshtola sephfs kraum seph sauron sheoldred kefka cloud veyran titania; do run ai-audit $d $d; done
run main-ref titania-before titania
echo "$(date +%H:%M) QUEUE DONE" >> $OUT/log.txt
