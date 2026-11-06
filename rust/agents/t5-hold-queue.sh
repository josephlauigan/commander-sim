#!/usr/bin/env bash
# t5-hold: Tier 5 and Tier 3 realism runs with the held-mana report (run by the coordinator, from anywhere)
set -euo pipefail
cd "$(dirname "$0")/../.."          # the worktree root
export PATH="$HOME/.cargo/bin:$PATH"
CAP=(systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0)
OUT=audit/realism/t5-hold-2026-11-06
mkdir -p "$OUT"
(cd rust && "${CAP[@]}" cargo build --release)
for tier in t5 t3; do
  echo "running $tier"
  "${CAP[@]}" rust/target/release/commander-sim --realism --pool "$tier" --ai adaptive --games 2000 --seed 900000 --jobs 23 \
    > "$OUT/$tier-adaptive-2000.txt"
done
echo "ALL DONE"
