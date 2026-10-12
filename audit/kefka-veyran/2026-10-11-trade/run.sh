#!/bin/bash
# Veyran -> Kefka block trade (2026-10-11). Look-ahead AI, loose profile, 1,000 paired games per tier (same seeds,
# seats and opponents' draws), after the kefka-ai-fix merge. --jobs 11: another session shares the machine.
# Usage: run.sh <test>   tests: veyran-full kefka-full kefka-protection kefka-breach kefka-engines
cd "$(dirname "$0")/../../.."
D=audit/kefka-veyran/2026-10-11-trade
T=$1
PROT=(--swap "Arcane Omens=>Deflecting Swat" --swap "Triskelion=>Slip Out the Back")
BREACH=(--swap "Inner Fire=>Underworld Breach")
ENG=(--swap "Mystic Confluence=>Niv-Mizzet, the Firemind" --swap "Splatter Technique=>Jin-Gitaxias, Progress Tyrant"
     --swap "Mountain=>Steam Vents" --swap "Mountain=>Sulfur Falls" --swap "Mountain=>Spirebluff Canal"
     --swap "Mountain=>Stormcarved Coast")
VEY=(--swap "Niv-Mizzet, the Firemind=>Mystic Confluence" --swap "Jin-Gitaxias, Progress Tyrant=>Splatter Technique"
     --swap "Underworld Breach=>Lightning Strike" --swap "Deflecting Swat=>Return the Favor"
     --swap "Slip Out the Back=>Spell Pierce" --swap "Steam Vents=>Island" --swap "Sulfur Falls=>Island"
     --swap "Spirebluff Canal=>Mountain" --swap "Stormcarved Coast=>Mountain")
case $T in
  veyran-full) DECK=veyran; SW=("${VEY[@]}") ;;
  kefka-full) DECK=kefka; SW=("${PROT[@]}" "${BREACH[@]}" "${ENG[@]}") ;;
  kefka-protection) DECK=kefka; SW=("${PROT[@]}") ;;
  kefka-breach) DECK=kefka; SW=("${BREACH[@]}") ;;
  kefka-engines) DECK=kefka; SW=("${ENG[@]}") ;;
  *) echo "unknown test $T"; exit 2 ;;
esac
mkdir -p $D/$T
for t in t1 t2 t3 t4 t5; do
  nice -n 19 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
    ./rust/target/release/commander-sim --deck $DECK --pool $t --profile loose --games 1000 --ai lookahead \
    --jobs 11 --brief "${SW[@]}" 2>&1 | grep -v progress > $D/$T/$t.txt
  echo "$T $t exit ${PIPESTATUS[0]}: $(grep -m1 'win ' $D/$T/$t.txt)"
done
