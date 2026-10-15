#!/bin/zsh
# Sephiroth FS: The One Ring for Caged Zombie (2026-10-14), paired
# against the current list (same seeds), Tiers 1-5, 1,000 games per list per tier, look-ahead, loose, --jobs 22, nice 19.
S=/tmp/claude-1000/-home-joseph-projects-mtg-sims/291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad
T=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/tymna-thrasios
D=$T/audit/sephfs/2026-10-14-ring
CAP=(nice -n 19 systemd-run --user --scope -q -p MemoryMax=20G -p MemorySwapMax=0)
export PATH=$HOME/.cargo/bin:$PATH
log() { echo "$(date +%H:%M) $*" | tee -a $D/log.txt; }
(cd $T/rust && $CAP cargo build --release -p sim-cli -j 22 2>&1 | tail -1) && cp $T/rust/target/release/commander-sim $S/csim-ring && log "built $(git -C $T log -1 --format=%h)"
for t in t1 t2 t3 t4 t5; do
  out=$D/ring-$t.txt; grep -q "delta" $out 2>/dev/null && continue
  (cd $T && $CAP $S/csim-ring --deck sephfs --pool $t --profile loose --games 1000 --ai lookahead --jobs 22 --seed 1014 --swap "Caged Zombie=>The One Ring" > $out 2>&1)
  log "$t exit $? $(grep -m1 -i 'delta' $out)"
done
log DONE
