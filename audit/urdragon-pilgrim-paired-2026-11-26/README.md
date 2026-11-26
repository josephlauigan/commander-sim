# Paired test, Ur-Dragon: Avacyn's Pilgrim in for Elvish Mystic, 2026-11-26

`run.sh` ran `old-vs-new.txt`: the worktree list (Pilgrim in) is the baseline and the variant is the OLD list (`Avacyn's Pilgrim=>Elvish Mystic`), so the printed
`delta +0.3` is old minus new, and the "swapped IN" row is the Mystic. t5 (the deck's tier), loose, 3,000 games per list, seed 500000, look-ahead AI, explain on.
Result: new 26.2%, old 26.5%, no clear effect. Limits: one seed and one profile; no colour-source counts in the output, so the white gain is not measured.
