#!/bin/zsh
# Verify the sephfs branch; only if every check passes, merge it into tymna-thrasios and run sephfs t1-t5.
S=/tmp/claude-1000/-home-joseph-projects-mtg-sims/291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad
F=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/sephfs
B=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/garland   # baseline: the merged code before sephfs
T=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/tymna-thrasios
CAP=(nice -n 19 systemd-run --user --scope -q -p MemoryMax=20G -p MemorySwapMax=0)
export PATH=$HOME/.cargo/bin:$PATH
log() { echo "$(date +%H:%M) $*" | tee -a $S/sephfs_run.log; }
cd $F/rust
w=$($CAP cargo build --release --all-targets -j 20 2>&1 | grep -cE '^warning'); log "build warnings: $w"
$CAP cargo test --release -j 20 --no-fail-fast > $S/sf-test.txt 2>&1
log "tests passed/failed: $(awk '/^test result/{s+=$4; f+=$6} END{print s+0" "f+0}' $S/sf-test.txt)"
for t in ${(f)"$(grep -E '^test .* FAILED$' $S/sf-test.txt | awk '{print $2}' | sort -u)"}; do
  [[ -z $t ]] && continue
  r=$($CAP cargo test --release -j 20 -p practice -- --exact $t 2>&1 | grep -c "test result: ok. 1 passed"); log "rerun $t alone: ok=$r"
  [[ $r == 1 ]] || { log "FAIL: $t fails alone"; exit 1; }
done
[[ $w == 0 ]] || { log "FAIL: warnings"; exit 1; }
for d in "sauron t3" "cloud t5" "thrasios t3" "garland t3" "seph t3" "sheoldred t3" "yshtola t3"; do
  set -- ${=d}
  (cd $B && $CAP rust/target/release/commander-sim --deck $1 --pool $2 --profile loose --games 48 --ai lookahead --jobs 20 --seed 7 --brief 2>&1 | grep -v -iE 'progress|elapsed|total\)' > $S/sf-base.$1)
  (cd $F && $CAP rust/target/release/commander-sim --deck $1 --pool $2 --profile loose --games 48 --ai lookahead --jobs 20 --seed 7 --brief 2>&1 | grep -v -iE 'progress|elapsed|total\)' > $S/sf-new.$1)
  if diff -q $S/sf-base.$1 $S/sf-new.$1 >/dev/null; then log "same-seed $1: identical"; else log "FAIL: same-seed $1 differs"; exit 1; fi
done
cd $T && git merge --ff-only -q sephfs && log "merged sephfs into tymna-thrasios: $(git log -1 --format=%h)" || { log "FAIL: merge"; exit 1; }
cp $F/rust/target/release/commander-sim $S/csim-sephfs
D=$T/audit/sephfs/2026-10-14; mkdir -p $D
for t in t1 t2 t3 t4 t5; do
  (cd $T && $CAP $S/csim-sephfs --deck sephfs --pool $t --profile loose --games 2000 --ai lookahead --jobs 22 --seed 1013 --analyze > $D/sephfs-$t.txt 2>&1)
  log "sephfs $t exit $? $(grep -m1 '^Win rate' $D/sephfs-$t.txt)"
done
log DONE
