#!/bin/bash
# run_final2.sh <file of name|deck|games> <out dir>: the rest of the final runs (lists already applied in bands-final;
# no checkout, no re-apply). Waits (up to 60 min) for a sim still running first.
L=$1; OUT=$2
W=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/bands-final
B=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/soul-stone/rust/target/release/commander-sim
cd $W || exit 1
for i in $(seq 1 120); do pgrep -f "[s]oul-stone/rust/target/release/commander-sim" >/dev/null || break; sleep 30; done
echo "$(date +%H:%M) done kefka (previous queue's last sim)" >> $OUT/log.txt
while IFS='|' read -r name deck n; do
  [ -z "$name" ] && continue
  echo "$(date +%H:%M) start $name ($n per profile)" >> $OUT/log.txt
  systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 nice -n 19 $B --deck "$deck" --pool all --games "$n" --jobs 22 > $OUT/$name.txt 2>&1
  echo "$(date +%H:%M) done $name exit $?" >> $OUT/log.txt
done < $L
echo "$(date +%H:%M) QUEUE DONE" >> $OUT/log.txt
