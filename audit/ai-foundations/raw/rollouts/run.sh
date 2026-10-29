#!/usr/bin/env bash
# run.sh NAME "ENV=.. ENV=.." ARGS... : one commander-sim run into this directory (stdout NAME.txt; EVF lines of
# stderr gzipped to NAME.tsv.gz when AI_EVAL_FALLBACK=1), timed, logged in progress.log. One run at a time.
set -u
D=$(cd "$(dirname "$0")" && pwd)
BIN=$D/../../../../rust/target/release/commander-sim
name=$1; envs=$2; shift 2
cd "$D/../../../.."
echo "start $name $(date +%T) $envs $*" >> "$D/progress.log"
t0=$(date +%s.%N)
env $envs "$BIN" "$@" > "$D/$name.txt" 2> >(grep '^EVF' | gzip > "$D/$name.tsv.gz")
rc=$?
t1=$(date +%s.%N)
wait
echo "done $name rc=$rc $(date +%T) secs=$(echo "$t1 - $t0" | bc)" >> "$D/progress.log"
