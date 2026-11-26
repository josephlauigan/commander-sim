#!/bin/bash
# Arzakon mana package: paired runs from the NEW list (the current branch list), 3,000 games per list per tier, loose, seed 500000.
cd /home/joseph/projects/commander-sim-wt-arz-mana || exit 1
OUT=/tmp/handoff/arz-sims
{ echo "disk: $(df -BG --output=avail / | tail -1)"; echo "commit: $(git rev-parse HEAD)"; echo "start: $(date -Is)"; } > $OUT/status.txt
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
BIN=./rust/target/release/commander-sim
# Run 1: the OLD list is the "swapped" arm (delta = old minus new)
$CAP $BIN --deck arzakon --pool all --games 3000 --swap "Mana Confluence=>Llanowar Elves" --swap "City of Brass=>Meteor Golem" --swap "Misty Rainforest=>Exsanguinate" --swap "Chromatic Lantern=>Coalition Relic" --quiet > $OUT/old-vs-new.txt 2>&1
echo "run1 done: $(date -Is) exit=$?" >> $OUT/status.txt
# Run 2: arm A2 (Avacyn's Pilgrim out, Meteor Golem kept) is the "swapped" arm (delta = A2 minus new)
$CAP $BIN --deck arzakon --pool all --games 3000 --swap "Avacyn's Pilgrim=>Meteor Golem" --quiet > $OUT/a2-vs-new.txt 2>&1
echo "run2 done: $(date -Is) exit=$?" >> $OUT/status.txt
echo "ALL SIMS DONE" >> $OUT/status.txt
