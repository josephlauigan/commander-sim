#!/bin/bash
# Sheoldred (key sheoldred, bulk-only list) and the Jodah trade blocks (2026-10-11). Look-ahead AI, loose profile.
#   base        : Sheoldred against every tier, 960 games per tier, seeds 500000-500959 (pool-results.md section 0d)
#   sheo-A/B/C  : Jodah's cards into Sheoldred, paired, 1,000 games per list per tier
#   jodah-A/B/C : the same cards out of Jodah, refilled from bulk, paired, 1,000 games per list per tier
#   sheo-ABC, jodah-ABC : all three blocks at once
#   upgrade     : the upgrade package (buys) in the list with Memory Jar: Seizan, Underworld Dreams, Dark Deal, Painful
#                 Lesson for Ribskiff, Gravblade Heavy, Multiply by Zero, Locust Spray
# Block A (wheel): Memory Jar.             Sheoldred cuts Carnage Altar;   Jodah gets Szadek, Lord of Secrets.
# Block B (sweeper + removal): Toxic Deluge, Bitter Triumph.
#                                          Sheoldred cuts Depressurize, Multiply by Zero;
#                                          Jodah gets Lagrella, the Magpie and a spare Wander Off.
# Block C (body): Korlash, Heir to Blackblade. Sheoldred cuts Gravblade Heavy; Jodah gets Sisters of Stone Death.
# 11 worker threads: another session shares the machine. Usage: run.sh <mode>...
cd "$(dirname "$0")/../.."
D=audit/sheoldred
SIM=./rust/target/release/commander-sim
cap() { systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 "$@"; }

SA=(--swap "Carnage Altar=>Memory Jar")
SB=(--swap "Depressurize=>Toxic Deluge" --swap "Multiply by Zero=>Bitter Triumph")
SC=(--swap "Gravblade Heavy=>Korlash, Heir to Blackblade")
SU=(--swap "Ribskiff=>Seizan, Perverter of Truth" --swap "Locust Spray=>Painful Lesson" --swap "Gravblade Heavy=>Underworld Dreams" --swap "Multiply by Zero=>Dark Deal")
JA=(--swap "Memory Jar=>Szadek, Lord of Secrets")
JB=(--swap "Toxic Deluge=>Lagrella, the Magpie" --swap "Bitter Triumph=>Wander Off")
JC=(--swap "Korlash, Heir to Blackblade=>Sisters of Stone Death")

paired() { # name deck swaps...
  local name=$1 deck=$2; shift 2
  mkdir -p $D/$name
  for t in t1 t2 t3 t4 t5; do
    cap $SIM --deck $deck --pool $t --profile loose --games 1000 --ai lookahead --jobs 11 --brief "$@" 2>&1 \
      | grep -v progress > $D/$name/$t.txt
    echo "$name $t exit ${PIPESTATUS[0]} $(grep -m1 'delta' $D/$name/$t.txt)" | tee -a $D/log.txt
  done
}

for mode in "$@"; do
  case $mode in
    base)
      mkdir -p $D/base
      for t in t1 t2 t3 t4 t5; do
        cap $SIM --deck sheoldred --pool $t --profile loose --games 960 --seed 500000 --ai lookahead --jobs 11 2>&1 \
          | grep -v progress > $D/base/$t.txt
        echo "base $t exit ${PIPESTATUS[0]} $(grep -m1 '^Win rate' $D/base/$t.txt)" | tee -a $D/log.txt
      done ;;
    sheo-A) paired $mode sheoldred "${SA[@]}" ;;
    sheo-B) paired $mode sheoldred "${SB[@]}" ;;
    sheo-C) paired $mode sheoldred "${SC[@]}" ;;
    sheo-ABC) paired $mode sheoldred "${SA[@]}" "${SB[@]}" "${SC[@]}" ;;
    upgrade) paired $mode sheoldred "${SU[@]}" ;;
    jodah-A) paired $mode jodah "${JA[@]}" ;;
    jodah-B) paired $mode jodah "${JB[@]}" ;;
    jodah-C) paired $mode jodah "${JC[@]}" ;;
    jodah-ABC) paired $mode jodah "${JA[@]}" "${JB[@]}" "${JC[@]}" ;;
    *) echo "unknown mode $mode"; exit 2 ;;
  esac
done
