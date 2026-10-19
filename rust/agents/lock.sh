#!/bin/zsh
# t3 byte-identical lock (t6-ai brief): from rust/
cd "$(dirname $0)/.."
nice -n 19 systemd-run --user --scope -q -p MemoryMax=14G -p MemorySwapMax=0 ./target/release/commander-sim --calibrate within --pool t3 --games 300 --seed 500000 --jobs 5 2>&1 | grep -vEi "progress|elapsed|minutes|seconds|games/h|worker threads|s total" > /tmp/claude-1000/t6ai-report/lock-out.txt
grep -vEi "progress|elapsed|minutes|seconds|games/h|worker threads|s total" agents/perf-lock-t3.txt > /tmp/claude-1000/t6ai-report/lock-ref.txt
diff /tmp/claude-1000/t6ai-report/lock-ref.txt /tmp/claude-1000/t6ai-report/lock-out.txt && echo LOCK IDENTICAL
