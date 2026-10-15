#!/bin/zsh
# Full pool run 2026-10-14/15: JD decks (+ Avery's Alela/Galadriel) vs Tiers 0-5, 4,000 games per cell, look-ahead AI,
# loose profile, seed 1015, --any-tier. Cells in cells.txt as "deck tier phase"; phase X runs binary csim-X from
# worktree run-x (a frozen snapshot of calib-1014). Resumable; threads from ./jobs; games from ./games (default 4000).
S=/tmp/claude-1000/-home-joseph-projects-mtg-sims/291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad
R=$S/run1014
WT=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees
CAP=(nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0)
log() { echo "$(date '+%m-%d %H:%M') $*" >> $R/log.txt; }
while read -r d t ph; do
  out=$R/$d-$t.txt; grep -q '^Win rate' $out 2>/dev/null && continue
  J=$(cat $R/jobs 2>/dev/null || echo 22); N=$(cat $R/games 2>/dev/null || echo 4000)
  w=$WT/run-${(L)ph}
  (cd $w && $CAP $S/csim-$ph --deck $d --pool $t --any-tier --profile loose --games $N --ai lookahead --jobs $J --seed 1015 --analyze < /dev/null > $out 2>&1)
  log "cell $d $t phase $ph exit $? jobs $J games $N $(grep -m1 '^Win rate' $out)"
done < $R/cells.txt
log "BATCH DONE"
