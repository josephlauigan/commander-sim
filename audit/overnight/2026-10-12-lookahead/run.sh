#!/bin/bash
# Overnight 2026-10-11/12: all ten of JD's decks against the five tiers, look-ahead AI, loose profile, on main 66e43bf.
# Rounds of 960 games per cell; round r uses seeds 500000+960r .. +959 (round 0 = the standard 500000-500959).
# A new round starts only if it can finish (at the last round's pace) before STOP. Resumable: finished cells are skipped.
# Each cell: memory cap 24G, timeout 20 min (normal ~1.5-2.5 min), one retry, then marked FAIL and skipped.
cd "$(dirname "$0")/../../.."
D=audit/overnight/2026-10-12-lookahead
STOP=$(date -d 'tomorrow 08:45' +%s); [ $(date +%H) -lt 12 ] && STOP=$(date -d 'today 08:45' +%s)
DECKS="seph veyran sauron galadriel yshtola alela jodah kefka titania sheoldred"
EST=5400   # first-round estimate (s); later rounds use the previous round's time
log() { echo "$(date +%H:%M) $*" | tee -a $D/log.txt; }

for r in $(seq 0 19); do
  R=$D/r$r; mkdir -p $R
  todo=0; for d in $DECKS; do for t in t1 t2 t3 t4 t5; do grep -qs "^Win rate" $R/$d-$t.txt || todo=1; done; done
  if [ $todo = 1 ] && [ $(( $(date +%s) + EST )) -gt $STOP ]; then
    log "STOP before round $r (est ${EST}s would pass 08:45)"; break
  fi
  start=$(date +%s); seed=$((500000 + 960 * r))
  for d in $DECKS; do
    for t in t1 t2 t3 t4 t5; do
      grep -qs "^Win rate" $R/$d-$t.txt && continue
      for try in 1 2; do
        c0=$(date +%s)
        systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
          timeout 1200 ./rust/target/release/commander-sim --deck $d --pool $t --profile loose --games 960 \
          --seed $seed --ai lookahead --jobs 24 2>&1 | grep -v progress > $R/$d-$t.txt
        rc=${PIPESTATUS[0]}
        if [ $rc = 0 ] && grep -q "^Win rate" $R/$d-$t.txt; then
          log "r$r $d $t ok $(( $(date +%s) - c0 ))s $(grep -m1 '^Win rate' $R/$d-$t.txt | awk '{print $3}')"; break
        fi
        log "FAIL r$r $d $t exit $rc try $try ($(( $(date +%s) - c0 ))s)"
        mv $R/$d-$t.txt $R/$d-$t.fail$try.txt
      done
    done
  done
  EST=$(( $(date +%s) - start ))
  log "ROUND $r done in ${EST}s | $(bash $D/summary.sh brief)"
done
log "ALL DONE"
