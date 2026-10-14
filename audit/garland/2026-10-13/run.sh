#!/bin/zsh
# Verify the garland branch, then (only if every check passes) merge it into tymna-thrasios and run the overnight
# Garland measurement. Log: $S/garland_night.log
S=/tmp/claude-1000/-home-joseph-projects-mtg-sims/291e1f60-a736-4dbc-bd9b-964e999afb88/scratchpad
G=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/garland
T=/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/tymna-thrasios
CAP=(nice -n 19 systemd-run --user --scope -q -p MemoryMax=20G -p MemorySwapMax=0)
export PATH=$HOME/.cargo/bin:$PATH
log() { echo "$(date +%H:%M) $*" | tee -a $S/garland_night.log; }
cd $G/rust
w=$($CAP cargo build --release --all-targets -j 20 2>&1 | grep -cE '^warning'); log "build warnings: $w"
$CAP cargo test --release -j 20 --no-fail-fast > $S/gn-test.txt 2>&1
pass=$(awk '/^test result/{s+=$4; f+=$6} END{print s+0" "f+0}' $S/gn-test.txt); log "tests passed/failed: $pass"
failed=$(grep -E '^test .* FAILED$' $S/gn-test.txt | awk '{print $2}' | sort -u)
for t in ${(f)failed}; do  # the known practice timing flake: rerun alone
  r=$($CAP cargo test --release -j 20 -p practice -- --exact $t 2>&1 | grep -c "test result: ok. 1 passed"); log "rerun $t alone: ok=$r"
  [[ $r == 1 ]] || { log "FAIL: $t fails alone"; exit 1; }
done
[[ $w == 0 ]] || { log "FAIL: warnings"; exit 1; }
# same-seed: base = tymna-thrasios binary in its own worktree; new = garland binary in the garland worktree
for d in "sauron t3" "cloud t5" "thrasios t3"; do
  set -- ${=d}
  (cd $T && $CAP rust/target/release/commander-sim --deck $1 --pool $2 --profile loose --games 48 --ai lookahead --jobs 20 --seed 7 --brief 2>&1 | grep -v -iE 'progress|elapsed|total\)' > $S/ss-base.$1)
  (cd $G && $CAP rust/target/release/commander-sim --deck $1 --pool $2 --profile loose --games 48 --ai lookahead --jobs 20 --seed 7 --brief 2>&1 | grep -v -iE 'progress|elapsed|total\)' > $S/ss-new.$1)
  if diff -q $S/ss-base.$1 $S/ss-new.$1 >/dev/null; then log "same-seed $1: identical"; else log "FAIL: same-seed $1 differs"; exit 1; fi
done
# merge, snapshot, run
cd $T && git merge --ff-only -q garland && log "merged garland into tymna-thrasios: $(git log -1 --format=%h)"
cp $T/rust/target/release/commander-sim $S/csim-garland 2>/dev/null; cp $G/rust/target/release/commander-sim $S/csim-garland
D=$T/audit/garland/2026-10-13; mkdir -p $D
for t in t1 t2 t3 t4 t5; do
  (cd $T && $CAP $S/csim-garland --deck garland --pool $t --profile loose --games 2000 --ai lookahead --jobs 22 --seed 1013 --analyze > $D/garland-$t.txt 2>&1)
  log "garland $t exit $? $(grep -m1 '^Win rate' $D/garland-$t.txt)"
done
log DONE
