#!/bin/bash
# Watchdog for run.py: restart the supervisor if it dies before the deadline (it resumes from results.jsonl).
# Start it detached so it outlives the shell that launched it:
#   setsid nohup audit/overnight/overnight.sh OUT --decks sauron,seph --stop-launch 06:00 --deadline 06:28 \
#       > /dev/null 2>&1 < /dev/null &
OUT="$1"; shift
cd "$(dirname "$0")/../.."
mkdir -p "$OUT"
dl=06:28                                          # the same --deadline run.py gets (its default)
args=("$@")
for ((i = 0; i < ${#args[@]}; i++)); do [ "${args[i]}" = "--deadline" ] && dl="${args[i+1]}"; done
end=$(date -d "today $dl" +%s); [ "$(date +%s)" -ge "$end" ] && end=$(date -d "tomorrow $dl" +%s)
while [ "$(date +%s)" -lt "$end" ]; do
  python3 audit/overnight/run.py run "$OUT" "$@" >> "$OUT/supervisor.log" 2>&1 && break
  echo "$(date +%H:%M:%S) WATCHDOG supervisor exited ($?), restarting in 10s" | tee -a "$OUT/events.log" >> "$OUT/supervisor.log"
  sleep 10
done
echo "$(date +%H:%M:%S) WATCHDOG done" >> "$OUT/events.log"
