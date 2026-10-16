#!/bin/zsh
# Ramp-bonus A/B 2026-10-15: every deck in my_decks.toml vs its matched tiers, old search (SEARCH_RAMP_W=0) vs new
# (ramp_w 1), same seed 1015, loose profile, look-ahead, 2,000 games per cell. Cells "deck tier w" in cells.txt.
# Frozen binary ./csim (built from branch ramp-value). Resumable; threads from ./jobs, games from ./games.
R=${0:A:h}  # outputs land next to this script
WT=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ramp-value
CAP=(nice -n 19 systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0)
log() { echo "$(date '+%m-%d %H:%M') $*" >> $R/log.txt; }
while read -r d t w; do
  out=$R/$d-$t-w$w.txt; grep -q '^Win rate' $out 2>/dev/null && continue
  J=$(cat $R/jobs); N=$(cat $R/games)
  (cd $WT && SEARCH_RAMP_W=$w $CAP $R/csim --deck $d --pool $t --profile loose --games $N --ai lookahead --jobs $J --seed 1015 --analyze < /dev/null > $out 2>&1)
  log "cell $d $t w$w exit $? jobs $J games $N $(grep -m1 '^Win rate' $out)"
done < $R/cells.txt
log "BATCH DONE"
touch $R/DONE
