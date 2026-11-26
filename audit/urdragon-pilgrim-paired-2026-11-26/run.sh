#!/bin/bash
# Ur-Dragon: Elvish Mystic out, Avacyn's Pilgrim in. Paired run from the NEW list (the worktree list), 3,000 games, t5 (the deck's tier),
# loose, seed 500000, explain on. The variant is the OLD list: `Avacyn's Pilgrim=>Elvish Mystic`, so a printed delta is old minus new
# (a printed negative number means the new list is better); the explain's "swapped IN" rows are the old card (the Mystic).
cd /home/joseph/projects/commander-sim-wt-urd-pilgrim || exit 1
D=audit/urdragon-pilgrim-paired-2026-11-26
{ echo "disk: $(df -BG --output=avail / | tail -1)"; echo "commit: $(git rev-parse HEAD)"; echo "start: $(date -Is)"; } > $D/status.txt
CAP="systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0"
BIN=/home/joseph/projects/commander-sim-wt-arz-mana/rust/target/release/commander-sim
$CAP $BIN --deck urdragon --pool all --games 3000 --swap "Avacyn's Pilgrim=>Elvish Mystic" --quiet > $D/old-vs-new.txt 2>&1
echo "run done: $(date -Is) exit=$?" >> $D/status.txt
echo "ALL SIMS DONE" >> $D/status.txt
