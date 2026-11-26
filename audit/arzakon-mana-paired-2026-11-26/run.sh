#!/bin/bash
# Arzakon mana package, second round (engine fixes, arm A2 list = Meteor Golem kept, Avacyn's Pilgrim cut): paired runs from the
# NEW list, 3,000 games per list per tier, loose, seed 500000. Delta = variant minus this list.
cd /home/joseph/projects/commander-sim-wt-arz-mana || exit 1
OUT=/tmp/handoff/arz-sims2; mkdir -p $OUT
{ echo "disk: $(df -BG --output=avail / | tail -1)"; echo "commit: $(git rev-parse HEAD)"; echo "start: $(date -Is)"; } > $OUT/status.txt
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
(cd rust && $CAP cargo build --release 2>&1 | tail -2) >> $OUT/status.txt
BIN=./rust/target/release/commander-sim
$CAP $BIN --deck arzakon --pool all --games 3000 --swap "Mana Confluence=>Llanowar Elves" --swap "City of Brass=>Avacyn's Pilgrim" --swap "Misty Rainforest=>Exsanguinate" --swap "Chromatic Lantern=>Coalition Relic" --quiet > $OUT/old-vs-new.txt 2>&1
echo "run1 done: $(date -Is) exit=$?" >> $OUT/status.txt
$CAP $BIN --deck arzakon --pool all --games 3000 --swap "Meteor Golem=>Avacyn's Pilgrim" --quiet > $OUT/a-vs-new.txt 2>&1
echo "run2 done: $(date -Is) exit=$?" >> $OUT/status.txt
echo "ALL SIMS DONE" >> $OUT/status.txt
